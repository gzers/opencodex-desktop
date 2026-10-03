//! macOS Keychain 受控凭据引用。
//!
//! 只存储随机 `ref_id` 对应的密钥值；配置、日志与 DTO 只保留引用。

use security_framework::passwords::{delete_generic_password, set_generic_password};
use uuid::Uuid;

use crate::errors::AppError;

pub const KEYCHAIN_SERVICE_NAME: &str = "OpenCodex Desktop";
pub const WEBDAV_CREDENTIAL_PURPOSE: &str = "webdav_credential";
pub const ENCRYPTION_PASSWORD_PURPOSE: &str = "encryption_password";

pub fn account_key(purpose: &str, ref_id: &str) -> Result<String, AppError> {
    if !matches!(
        purpose,
        WEBDAV_CREDENTIAL_PURPOSE | ENCRYPTION_PASSWORD_PURPOSE
    ) {
        return Err(AppError::NotConfigured);
    }
    if !ref_id.starts_with("cred_")
        || ref_id.len() != 5 + 36
        || !ref_id[5..]
            .chars()
            .all(|c| c.is_ascii_hexdigit() || c == '-')
    {
        return Err(AppError::NotConfigured);
    }
    let prefix = if purpose == WEBDAV_CREDENTIAL_PURPOSE {
        "ocx.dav"
    } else {
        "ocx.sync"
    };
    Ok(format!("{prefix}.{ref_id}"))
}

pub fn new_ref_id() -> String {
    format!("cred_{}", Uuid::new_v4())
}

/// 按 purpose 写入独立账户位；不同 purpose 落在不同 keychain 账户。
fn store_keychain_password(purpose: &str, ref_id: &str, password: &str) -> Result<(), AppError> {
    if password.is_empty() {
        return Err(AppError::NotConfigured);
    }
    let account = account_key(purpose, ref_id)?;
    set_generic_password(KEYCHAIN_SERVICE_NAME, &account, password.as_bytes())
        .map_err(|_| AppError::NotConfigured)
}

pub fn store_webdav_password(ref_id: &str, password: &str) -> Result<(), AppError> {
    store_keychain_password(WEBDAV_CREDENTIAL_PURPOSE, ref_id, password)
}

/// WebDAV 口令与同步加密口令是**两个不同的秘密**，必须分开存。
/// 二者此前都写在 `ocx.dav.<ref_id>` 上，后写覆盖先写：WebDAV 认证拿到的是
/// 加密口令，而 `load_encryption_password` 读的 `ocx.sync.<ref_id>` 从未被写入。
pub fn store_encryption_password(ref_id: &str, password: &str) -> Result<(), AppError> {
    store_keychain_password(ENCRYPTION_PASSWORD_PURPOSE, ref_id, password)
}

pub fn load_webdav_password(ref_id: &str) -> Result<String, AppError> {
    load_keychain_password(WEBDAV_CREDENTIAL_PURPOSE, ref_id)
}

pub fn load_encryption_password(ref_id: &str) -> Result<String, AppError> {
    load_keychain_password(ENCRYPTION_PASSWORD_PURPOSE, ref_id)
}

pub fn load_keychain_password(purpose: &str, ref_id: &str) -> Result<String, AppError> {
    let account = account_key(purpose, ref_id)?;
    security_framework::passwords::get_generic_password(KEYCHAIN_SERVICE_NAME, &account)
        .map_err(|_| AppError::NotConfigured)
        .map(|value| String::from_utf8_lossy(&value).to_string())
}

pub fn delete_webdav_password(ref_id: &str) -> Result<(), AppError> {
    let account = account_key(WEBDAV_CREDENTIAL_PURPOSE, ref_id)?;
    delete_generic_password(KEYCHAIN_SERVICE_NAME, &account).map_err(|_| AppError::NotConfigured)
}

pub fn delete_encryption_password(ref_id: &str) -> Result<(), AppError> {
    let account = account_key(ENCRYPTION_PASSWORD_PURPOSE, ref_id)?;
    delete_generic_password(KEYCHAIN_SERVICE_NAME, &account).map_err(|_| AppError::NotConfigured)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn account_key_matches_frozen_shape() {
        let ref_id = "cred_12345678-1234-1234-1234-123456789abc";
        assert_eq!(
            account_key(WEBDAV_CREDENTIAL_PURPOSE, ref_id).unwrap(),
            "ocx.dav.cred_12345678-1234-1234-1234-123456789abc"
        );
        assert_eq!(
            account_key(ENCRYPTION_PASSWORD_PURPOSE, ref_id).unwrap(),
            "ocx.sync.cred_12345678-1234-1234-1234-123456789abc"
        );
        assert!(account_key("other", ref_id).is_err());
        assert!(account_key(WEBDAV_CREDENTIAL_PURPOSE, "bad").is_err());
    }

    #[test]
    fn new_ref_id_is_valid() {
        let ref_id = new_ref_id();
        assert!(account_key(WEBDAV_CREDENTIAL_PURPOSE, &ref_id).is_ok());
    }

    #[test]
    fn webdav_and_encryption_purposes_use_different_accounts() {
        let ref_id = new_ref_id();
        assert_ne!(
            account_key(WEBDAV_CREDENTIAL_PURPOSE, &ref_id).unwrap(),
            account_key(ENCRYPTION_PASSWORD_PURPOSE, &ref_id).unwrap(),
            "两个口令必须落在不同 keychain 账户位，否则后写会覆盖先写"
        );
    }

    /// 真机往返护栏：同一个 `ref_id` 下，WebDAV 口令与加密口令必须各存各的。
    /// 这正是历史缺陷所在（两者都写 `ocx.dav.<ref_id>`，后者覆盖前者，
    /// 而 `ocx.sync.<ref_id>` 从未被写入）。
    ///
    /// 会访问系统钥匙串，故默认忽略；改动凭据存储后手动执行：
    /// `cargo test --offline -- --ignored --nocapture keychain_round_trip`
    #[test]
    #[ignore]
    fn keychain_round_trip_keeps_both_secrets_separate() {
        let ref_id = new_ref_id();
        store_webdav_password(&ref_id, "dav-secret").expect("store webdav");
        store_encryption_password(&ref_id, "enc-secret").expect("store encryption");

        assert_eq!(load_webdav_password(&ref_id).unwrap(), "dav-secret");
        assert_eq!(load_encryption_password(&ref_id).unwrap(), "enc-secret");

        delete_webdav_password(&ref_id).expect("delete webdav");
        delete_encryption_password(&ref_id).expect("delete encryption");
    }
}
