//! Local content protection: metadata stores only an enabled flag and opaque reference.
use crate::{
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::{
        backup::safety,
        model_config::{schema::*, store::sha256},
    },
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use zeroize::Zeroizing;
const FILE: &str = "manager-state/transfer-protection.json";
#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Metadata {
    enabled: bool,
    reference: Option<String>,
    #[serde(default)]
    previous_reference: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub enabled: bool,
    pub has_password: bool,
}
trait Vault {
    fn get(&self, account: &str) -> ModelResult<Option<Zeroizing<String>>>;
    fn put(&self, account: &str, value: &str) -> ModelResult<()>;
}
struct Native;
impl Vault for Native {
    fn get(&self, account: &str) -> ModelResult<Option<Zeroizing<String>>> {
        let e = keyring::Entry::new(
            crate::infrastructure::keychain::keychain_service_name(),
            account,
        )
        .map_err(|_| invalid("sync_protection", "同步保护安全存储不可用"))?;
        match e.get_password() {
            Ok(v) => Ok(Some(Zeroizing::new(v))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(invalid(
                "sync_protection",
                "无法读取同步保护口令，原配置保持",
            )),
        }
    }
    fn put(&self, account: &str, value: &str) -> ModelResult<()> {
        let e = keyring::Entry::new(
            crate::infrastructure::keychain::keychain_service_name(),
            account,
        )
        .map_err(|_| invalid("sync_protection", "同步保护安全存储不可用"))?;
        e.set_password(value)
            .map_err(|_| invalid("sync_protection", "同步保护口令未保存"))?;
        if self.get(account)?.as_deref().map(|v| v.as_str()) != Some(value) {
            return Err(invalid("sync_protection", "同步保护口令未读回"));
        }
        Ok(())
    }
}
fn metadata(root: &Path) -> ModelResult<Metadata> {
    let p = root.join(FILE);
    safety::check_path(root, &p, true)?;
    if !p.exists() {
        return Ok(Metadata::default());
    }
    let v: Metadata = serde_json::from_slice(&safety::read_file(&p, 4096)?)?;
    if let Some(r) = &v.reference {
        require_uuid(r)?
    }
    if let Some(r) = &v.previous_reference {
        require_uuid(r)?;
    }
    Ok(v)
}
fn account(root: &Path, reference: &str) -> String {
    format!(
        "transfer.protection.{}.{}",
        sha256(root.to_string_lossy().as_bytes()),
        reference
    )
}
fn status_with(root: &Path, vault: &impl Vault) -> ModelResult<Status> {
    let m = metadata(root)?;
    Ok(Status {
        enabled: m.enabled,
        has_password: match m.reference {
            Some(r) => vault.get(&account(root, &r))?.is_some(),
            None => false,
        },
    })
}
pub fn status(root: &Path) -> ModelResult<Status> {
    status_with(root, &Native)
}
fn save_with(
    root: &Path,
    enabled: bool,
    password: Option<&str>,
    vault: &impl Vault,
) -> ModelResult<Status> {
    let p = root.join(FILE);
    safety::check_path(root, &p, true)?;
    let _g = TargetFileLock::lock(&p)?;
    let mut m = metadata(root)?;
    if let Some(password) = password.filter(|s| !s.is_empty()) {
        if password.len() < 8 || password.len() > 4096 {
            return Err(invalid("sync_protection", "保护口令须为8至4096字节"));
        }
        let current = match &m.reference {
            Some(r) => vault.get(&account(root, r))?,
            None => None,
        };
        if current.as_deref().map(|s| s.as_str()) != Some(password) {
            let r = uuid::Uuid::new_v4().to_string();
            vault.put(&account(root, &r), password)?;
            m.previous_reference = m.reference.clone();
            m.reference = Some(r)
        }
    }
    if enabled {
        let available = match &m.reference {
            Some(r) => vault.get(&account(root, r))?.is_some(),
            None => false,
        };
        if !available {
            return Err(invalid("passphrase_required", "请填写保护口令"));
        }
    }
    m.enabled = enabled;
    atomic_write(&p, &serde_json::to_vec(&m)?, 0o600)?;
    status_with(root, vault)
}
pub fn save(root: &Path, enabled: bool, password: Option<&str>) -> ModelResult<Status> {
    save_with(root, enabled, password, &Native)
}
pub fn password(root: &Path) -> ModelResult<Option<Zeroizing<String>>> {
    let m = metadata(root)?;
    if !m.enabled {
        return Ok(None);
    }
    let r = m
        .reference
        .ok_or_else(|| invalid("passphrase_required", "请填写保护口令"))?;
    Native
        .get(&account(root, &r))?
        .map(Some)
        .ok_or_else(|| invalid("passphrase_required", "本机未保存该保护口令，请填写后重试"))
}
/// Reading existing content can use a retained password even when encryption
/// of future uploads is disabled. The two decisions must stay independent.
pub fn read_password(root: &Path) -> ModelResult<Option<Zeroizing<String>>> {
    let m = metadata(root)?;
    match m.reference {
        Some(r) => Native.get(&account(root, &r)),
        None => Ok(None),
    }
}
pub fn previous_password(root: &Path) -> ModelResult<Option<Zeroizing<String>>> {
    let m = metadata(root)?;
    match m.previous_reference {
        Some(r) => Native.get(&account(root, &r)),
        None => Ok(None),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, collections::BTreeMap};
    #[derive(Default)]
    struct Memory(RefCell<BTreeMap<String, String>>);
    impl Vault for Memory {
        fn get(&self, a: &str) -> ModelResult<Option<Zeroizing<String>>> {
            Ok(self.0.borrow().get(a).cloned().map(Zeroizing::new))
        }
        fn put(&self, a: &str, v: &str) -> ModelResult<()> {
            self.0.borrow_mut().insert(a.into(), v.into());
            Ok(())
        }
    }
    #[test]
    fn metadata_never_contains_secret_and_disable_does_not_delete_it() {
        let t = tempfile::tempdir().unwrap();
        let v = Memory::default();
        assert!(save_with(t.path(), true, None, &v).is_err());
        save_with(t.path(), true, Some("fixture-protection-pass"), &v).unwrap();
        let b = std::fs::read_to_string(t.path().join(FILE)).unwrap();
        assert!(!b.contains("fixture-protection-pass"));
        save_with(t.path(), false, None, &v).unwrap();
        assert!(!status_with(t.path(), &v).unwrap().enabled);
        assert!(status_with(t.path(), &v).unwrap().has_password);
        save_with(t.path(), true, None, &v).unwrap();
        assert!(status_with(t.path(), &v).unwrap().enabled);
    }
    #[test]
    fn identities_are_root_bound() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        assert_ne!(account(a.path(), "fixture"), account(b.path(), "fixture"));
    }
    #[test]
    #[ignore = "requires OPENCODEX_SANDBOX=1 and a native credential store; run explicitly on each OS"]
    fn native_secure_store_round_trip_is_isolated_and_metadata_contains_no_password() {
        assert!(
            crate::modules::test_sandbox::enabled(),
            "native test must use the sandbox credential service"
        );
        let root = tempfile::tempdir().unwrap();
        let mut references = vec![];
        let result = (|| {
            save(root.path(), true, Some("fixture-native-protection-A"))?;
            references.push(metadata(root.path())?.reference.unwrap());
            assert_eq!(
                password(root.path())?.unwrap().as_str(),
                "fixture-native-protection-A"
            );
            save(root.path(), false, None)?;
            assert!(password(root.path())?.is_none());
            assert_eq!(
                read_password(root.path())?.unwrap().as_str(),
                "fixture-native-protection-A"
            );
            save(root.path(), true, Some("fixture-native-protection-B"))?;
            references.push(metadata(root.path())?.reference.unwrap());
            assert_eq!(
                previous_password(root.path())?.unwrap().as_str(),
                "fixture-native-protection-A"
            );
            assert_eq!(
                password(root.path())?.unwrap().as_str(),
                "fixture-native-protection-B"
            );
            let content = std::fs::read_to_string(root.path().join(FILE))?;
            assert!(!content.contains("fixture-native-protection"));
            ModelResult::Ok(())
        })();
        for reference in references {
            keyring::Entry::new(
                crate::infrastructure::keychain::keychain_service_name(),
                &account(root.path(), &reference),
            )
            .unwrap()
            .delete_credential()
            .unwrap();
        }
        result.unwrap();
    }
}
