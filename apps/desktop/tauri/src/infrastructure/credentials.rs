//! 同步相关凭据的本机文件后端（WP-37）。
//!
//! 为什么不再走系统钥匙串：发布包是 ad-hoc 签名，每次构建的 CDHash 都变，
//! 钥匙串条目的 ACL 认不出新版本，于是每次读取都弹系统授权窗口；而同步页一打开
//! 就会连读多次口令，用户看到的就是「总是要输密码」。
//!
//! 现在把 WebDAV 口令与同步保护口令写进数据根下：
//! macOS 写 `manager-state/credentials/<account>.json` 文件 0600、目录 0700。
//! Windows 用同一路径，值先用 DPAPI（当前用户作用域）加密再落盘。
//! 两端都不需要证书、不弹系统密码。代价写在 0.2.6 的 WP-37 里：macOS 侧是明文落盘
//! （受 0600 与 FileVault 保护），这是「不要证书、不弹密码」的必然交换。
//!
//! 边界：配置、日志与 DTO 只保留 `cred_` 引用；真实值只出现在本模块与调用方内存。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use zeroize::Zeroizing;

use crate::errors::AppError;
use crate::infrastructure::keychain::{
    account_key, ENCRYPTION_PASSWORD_PURPOSE, WEBDAV_CREDENTIAL_PURPOSE,
};

/// 凭据目录相对数据根的位置。
const CREDENTIALS_DIR: &str = "manager-state/credentials";

/// 数据根在启动时接一次；未接线时所有读写都明确失败，不退回到其他目录。
static ROOT: OnceLock<PathBuf> = OnceLock::new();

pub fn configure(root: &Path) {
    let _ = ROOT.set(root.to_path_buf());
}

fn root() -> Result<&'static Path, AppError> {
    ROOT.get()
        .map(PathBuf::as_path)
        .ok_or(AppError::NotConfigured)
}

/// 进程内缓存：打开一次同步页会连读多次口令，没有缓存就每次都落盘。
/// 键是 `account_key`，写入与删除时同步失效，避免读到旧值。
fn cache() -> &'static Mutex<BTreeMap<String, Zeroizing<String>>> {
    static CACHE: OnceLock<Mutex<BTreeMap<String, Zeroizing<String>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

fn path_for(root: &Path, account: &str) -> PathBuf {
    // 账户位只含 ASCII 安全字符（`ocx.dav.cred_<uuid>`），这里再做一次白名单，防止路径穿越。
    let safe: String = account
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    root.join(CREDENTIALS_DIR).join(format!("{safe}.json"))
}

/// 落盘结构。`account` 参与校验：把 A 的凭据文件改名成 B 不能被当成 B 的口令。
#[derive(serde::Serialize, serde::Deserialize)]
struct Stored {
    account: String,
    value: String,
    protection: String,
}

fn encode(account: &str, password: &str) -> Result<Vec<u8>, AppError> {
    #[cfg(windows)]
    let (value, protection) = (dpapi::protect(password)?, "dpapi");
    #[cfg(not(windows))]
    let (value, protection) = (password.to_string(), "plain");
    serde_json::to_vec(&Stored {
        account: account.to_string(),
        value,
        protection: protection.to_string(),
    })
    .map_err(|_| AppError::NotConfigured)
}

fn decode(bytes: &[u8], account: &str) -> Result<Zeroizing<String>, AppError> {
    let stored: Stored = serde_json::from_slice(bytes).map_err(|_| AppError::NotConfigured)?;
    if stored.account != account {
        return Err(AppError::NotConfigured);
    }
    #[cfg(windows)]
    let value = match stored.protection.as_str() {
        "dpapi" => dpapi::unprotect(&stored.value)?,
        _ => return Err(AppError::NotConfigured),
    };
    #[cfg(not(windows))]
    let value = match stored.protection.as_str() {
        "plain" => stored.value,
        // 非 Windows 读到 DPAPI 密文说明数据根被跨平台搬运过，不能假装能解。
        _ => return Err(AppError::NotConfigured),
    };
    if value.is_empty() {
        return Err(AppError::NotConfigured);
    }
    Ok(Zeroizing::new(value))
}

fn store(purpose: &str, ref_id: &str, password: &str) -> Result<(), AppError> {
    if password.is_empty() {
        return Err(AppError::NotConfigured);
    }
    let root = root()?;
    let account = account_key(purpose, ref_id)?;
    let dir = root.join(CREDENTIALS_DIR);
    std::fs::create_dir_all(&dir).map_err(|_| AppError::NotConfigured)?;
    crate::infrastructure::platform::set_mode(&dir, 0o700).map_err(|_| AppError::NotConfigured)?;
    let bytes = encode(&account, password)?;
    crate::infrastructure::atomic_write::atomic_write(&path_for(root, &account), &bytes, 0o600)?;
    if let Ok(mut guard) = cache().lock() {
        guard.insert(account, Zeroizing::new(password.to_string()));
    }
    Ok(())
}

/// 只有明确的「文件不存在」才算没有；读失败、解析失败、身份不符都不能当成「没配置」，
/// 否则会把一次故障静默降级成「请重新输入密码」。
pub fn load_optional(purpose: &str, ref_id: &str) -> Result<Option<Zeroizing<String>>, AppError> {
    let root = root()?;
    let account = account_key(purpose, ref_id)?;
    if let Ok(guard) = cache().lock() {
        if let Some(value) = guard.get(&account) {
            return Ok(Some(Zeroizing::new(value.to_string())));
        }
    }
    let path = path_for(root, &account);
    let bytes = match std::fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(AppError::NotConfigured),
    };
    let value = decode(&bytes, &account)?;
    if let Ok(mut guard) = cache().lock() {
        guard.insert(account, Zeroizing::new(value.to_string()));
    }
    Ok(Some(value))
}

/// 幂等删除：文件本来不存在也算成功，但删除后必须读回确认确实没了。
pub fn remove(purpose: &str, ref_id: &str) -> Result<(), AppError> {
    let root = root()?;
    let account = account_key(purpose, ref_id)?;
    let path = path_for(root, &account);
    if let Err(error) = std::fs::remove_file(&path) {
        if error.kind() != std::io::ErrorKind::NotFound {
            return Err(AppError::NotConfigured);
        }
    }
    if let Ok(mut guard) = cache().lock() {
        guard.remove(&account);
    }
    if path.exists() {
        return Err(AppError::NotConfigured);
    }
    Ok(())
}

pub fn store_webdav_password(ref_id: &str, password: &str) -> Result<(), AppError> {
    store(WEBDAV_CREDENTIAL_PURPOSE, ref_id, password)
}

pub fn load_webdav_password(ref_id: &str) -> Result<String, AppError> {
    Ok(load_optional(WEBDAV_CREDENTIAL_PURPOSE, ref_id)?
        .ok_or(AppError::NotConfigured)?
        .to_string())
}

/// 与既有调用点同名：purpose 只接受 WebDAV / 同步保护两类，其他用途仍归系统钥匙串。
pub fn load_optional_keychain_password(
    purpose: &str,
    ref_id: &str,
) -> Result<Option<String>, AppError> {
    Ok(load_optional(purpose, ref_id)?.map(|value| value.to_string()))
}

pub fn load_keychain_password(purpose: &str, ref_id: &str) -> Result<String, AppError> {
    Ok(load_optional(purpose, ref_id)?
        .ok_or(AppError::NotConfigured)?
        .to_string())
}

pub fn remove_keychain_password(purpose: &str, ref_id: &str) -> Result<(), AppError> {
    remove(purpose, ref_id)
}

/// WebDAV 口令与同步保护口令是两个不同的秘密，落在两个账户位，互不覆盖。
pub fn store_encryption_password(ref_id: &str, password: &str) -> Result<(), AppError> {
    store(ENCRYPTION_PASSWORD_PURPOSE, ref_id, password)
}

pub fn load_encryption_password(ref_id: &str) -> Result<String, AppError> {
    load_keychain_password(ENCRYPTION_PASSWORD_PURPOSE, ref_id)
}

pub fn delete_webdav_password(ref_id: &str) -> Result<(), AppError> {
    remove(WEBDAV_CREDENTIAL_PURPOSE, ref_id)
}

pub fn delete_encryption_password(ref_id: &str) -> Result<(), AppError> {
    remove(ENCRYPTION_PASSWORD_PURPOSE, ref_id)
}

#[cfg(windows)]
mod dpapi {
    use super::AppError;
    use windows_sys::Win32::Foundation::LocalFree;
    use windows_sys::Win32::Security::Cryptography::{
        CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB,
    };

    fn blob(bytes: &[u8]) -> CRYPT_INTEGER_BLOB {
        CRYPT_INTEGER_BLOB {
            cbData: bytes.len() as u32,
            pbData: bytes.as_ptr() as *mut u8,
        }
    }

    fn take(out: CRYPT_INTEGER_BLOB) -> Vec<u8> {
        if out.pbData.is_null() {
            return Vec::new();
        }
        let bytes = unsafe { std::slice::from_raw_parts(out.pbData, out.cbData as usize).to_vec() };
        unsafe { LocalFree(out.pbData as *mut core::ffi::c_void) };
        bytes
    }

    fn base64(bytes: &[u8]) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(bytes)
    }

    fn from_base64(value: &str) -> Result<Vec<u8>, AppError> {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(value)
            .map_err(|_| AppError::NotConfigured)
    }

    /// 当前用户作用域加密：换 Windows 账户解不开，符合「本机凭据」的语义；全程无提示。
    pub fn protect(password: &str) -> Result<String, AppError> {
        let input = blob(password.as_bytes());
        let mut out = CRYPT_INTEGER_BLOB::default();
        let ok = unsafe {
            CryptProtectData(
                &input,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                &mut out,
            )
        };
        if ok == 0 {
            return Err(AppError::NotConfigured);
        }
        Ok(base64(&take(out)))
    }

    pub fn unprotect(value: &str) -> Result<String, AppError> {
        let raw = from_base64(value)?;
        if raw.is_empty() {
            return Err(AppError::NotConfigured);
        }
        let input = blob(&raw);
        let mut out = CRYPT_INTEGER_BLOB::default();
        let ok = unsafe {
            CryptUnprotectData(
                &input,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                0,
                &mut out,
            )
        };
        if ok == 0 {
            return Err(AppError::NotConfigured);
        }
        String::from_utf8(take(out)).map_err(|_| AppError::NotConfigured)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 单个测试进程只接一次根；所有用例共用同一临时目录，靠不同 ref_id 隔离。
    fn root() -> &'static Path {
        static DIR: OnceLock<tempfile::TempDir> = OnceLock::new();
        let dir = DIR.get_or_init(|| tempfile::tempdir().expect("tempdir"));
        configure(dir.path());
        dir.path()
    }

    #[test]
    fn webdav_and_encryption_use_different_files() {
        let _ = root();
        let ref_id = format!("cred_{}", uuid::Uuid::new_v4());
        store_webdav_password(&ref_id, "dav").expect("store dav");
        store_encryption_password(&ref_id, "enc").expect("store enc");
        assert_eq!(load_webdav_password(&ref_id).unwrap(), "dav");
        assert_eq!(load_encryption_password(&ref_id).unwrap(), "enc");
        delete_webdav_password(&ref_id).unwrap();
        assert!(load_webdav_password(&ref_id).is_err());
        // 删掉 WebDAV 口令不影响同步保护口令。
        assert_eq!(load_encryption_password(&ref_id).unwrap(), "enc");
        delete_encryption_password(&ref_id).unwrap();
    }

    #[test]
    fn missing_is_none_but_corrupt_is_an_error() {
        let dir = root();
        let ref_id = format!("cred_{}", uuid::Uuid::new_v4());
        assert!(load_optional(WEBDAV_CREDENTIAL_PURPOSE, &ref_id)
            .unwrap()
            .is_none());
        let account = account_key(WEBDAV_CREDENTIAL_PURPOSE, &ref_id).unwrap();
        let path = path_for(dir, &account);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"{not json").unwrap();
        assert!(load_optional(WEBDAV_CREDENTIAL_PURPOSE, &ref_id).is_err());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn renamed_file_is_rejected_instead_of_serving_another_account() {
        let dir = root();
        let a = format!("cred_{}", uuid::Uuid::new_v4());
        let b = format!("cred_{}", uuid::Uuid::new_v4());
        store_webdav_password(&a, "secret-a").expect("store a");
        let from = path_for(dir, &account_key(WEBDAV_CREDENTIAL_PURPOSE, &a).unwrap());
        let to = path_for(dir, &account_key(WEBDAV_CREDENTIAL_PURPOSE, &b).unwrap());
        std::fs::rename(&from, &to).unwrap();
        assert!(load_optional(WEBDAV_CREDENTIAL_PURPOSE, &b).is_err());
        std::fs::remove_file(&to).ok();
    }

    #[test]
    fn removal_is_idempotent_and_reads_back_absence() {
        let _ = root();
        let ref_id = format!("cred_{}", uuid::Uuid::new_v4());
        remove(WEBDAV_CREDENTIAL_PURPOSE, &ref_id).expect("absent is fine");
        store_webdav_password(&ref_id, "v").expect("store");
        remove(WEBDAV_CREDENTIAL_PURPOSE, &ref_id).expect("remove");
        assert!(load_optional(WEBDAV_CREDENTIAL_PURPOSE, &ref_id)
            .unwrap()
            .is_none());
    }

    #[test]
    fn refuses_unknown_purpose_instead_of_writing_a_file() {
        let _ = root();
        let ref_id = format!("cred_{}", uuid::Uuid::new_v4());
        assert!(account_key("other", &ref_id).is_err());
        assert!(store("other", &ref_id, "v").is_err());
    }
}
