//! Selected-provider authorization protection; no secrets in preview IPC.
use super::{
    management::{KeyState, Management},
    schema::*,
    store::sha256,
};
use crate::{
    infrastructure::atomic_write::atomic_write,
    modules::{backup::safety, sync::portable_file},
    state::ProcessContext,
};
use serde::{
    de::{DeserializeSeed, IgnoredAny, MapAccess, Visitor},
    Deserialize, Serialize,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    path::{Path, PathBuf},
};
use zeroize::{Zeroize, Zeroizing};

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Auth {
    api_key: Option<String>,
    #[serde(default)]
    api_key_pool: Vec<Slot>,
    #[serde(default)]
    headers: BTreeMap<String, String>,
}
#[derive(Serialize, Deserialize)]
struct Slot {
    id: String,
    key: String,
    label: Option<String>,
}
impl Drop for Auth {
    fn drop(&mut self) {
        if let Some(key) = &mut self.api_key {
            key.zeroize();
        }
        for slot in &mut self.api_key_pool {
            slot.key.zeroize();
        }
        for value in self.headers.values_mut() {
            value.zeroize();
        }
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Protected {
    provider: String,
    binding: String,
    auth: Auth,
    reference_digests: BTreeMap<String, String>,
}
struct Selected<'a> {
    provider: &'a str,
    providers: bool,
}
impl<'de> DeserializeSeed<'de> for Selected<'_> {
    type Value = Option<Auth>;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(self)
    }
}
impl<'de> Visitor<'de> for Selected<'_> {
    type Value = Option<Auth>;
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("selected provider configuration")
    }
    fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
        let mut found = None;
        let mut names = BTreeSet::new();
        while let Some(name) = map.next_key::<String>()? {
            if !names.insert(name.clone()) {
                return Err(serde::de::Error::custom("duplicate source field"));
            }
            if self.providers && name == self.provider {
                found = Some(map.next_value::<Auth>()?);
            } else if !self.providers && name == "providers" {
                found = map.next_value_seed(Selected {
                    provider: self.provider,
                    providers: true,
                })?;
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(found)
    }
}
fn parse(bytes: &[u8], provider: &str) -> ModelResult<Auth> {
    let mut decoder = serde_json::Deserializer::from_slice(bytes);
    let mut auth = Selected {
        provider,
        providers: false,
    }
    .deserialize(&mut decoder)
    .map_err(|_| invalid("provider_auth_source", "所选渠道授权源结构无效，未执行删除"))?
    .ok_or_else(|| invalid("provider_auth_source", "所选渠道授权源不存在"))?;
    if auth.api_key_pool.is_empty() {
        if let Some(key) = &auth.api_key {
            auth.api_key_pool.push(Slot {
                id: key_id(key),
                key: key.clone(),
                label: None,
            });
        }
    }
    decoder
        .end()
        .map_err(|_| invalid("provider_auth_source", "渠道授权源含多余数据"))?;
    validate(&auth)?;
    Ok(auth)
}
fn read(context: &ProcessContext, provider: &str) -> ModelResult<Auth> {
    let path = context.opencodex_home.join("config.json");
    safety::check_path(&context.opencodex_home, &path, false)?;
    let bytes = Zeroizing::new(safety::read_file(&path, 16 * 1024 * 1024)?);
    parse(&bytes, provider)
}
fn key_id(key: &str) -> String {
    sha256(key.as_bytes()).trim_start_matches("sha256:")[..8].into()
}
fn validate(auth: &Auth) -> ModelResult<()> {
    let key_ok = |key: &str| {
        !key.trim().is_empty()
            && key.trim() == key
            && key.len() <= 8192
            && !key.bytes().any(|b| b == 13 || b == 10)
    };
    if auth.api_key.as_ref().is_some_and(|key| !key_ok(key))
        || auth.api_key_pool.len() > 256
        || auth.headers.len() > 128
    {
        return Err(invalid(
            "provider_auth_source",
            "渠道授权字段无效或超过上限",
        ));
    }
    let mut ids = BTreeSet::new();
    for slot in &auth.api_key_pool {
        if !key_ok(&slot.key)
            || slot.id != key_id(&slot.key)
            || !ids.insert(&slot.id)
            || slot
                .label
                .as_ref()
                .is_some_and(|s| s.len() > 256 || s.chars().any(char::is_control))
        {
            return Err(invalid("provider_auth_source", "Key池标识、标签或秘密无效"));
        }
    }
    if !auth.api_key_pool.is_empty()
        && auth
            .api_key
            .as_ref()
            .is_none_or(|key| !auth.api_key_pool.iter().any(|s| &s.key == key))
    {
        return Err(invalid(
            "provider_auth_source",
            "活动Key不在池内，未执行删除",
        ));
    }
    let mut names = BTreeSet::new();
    for (name, value) in &auth.headers {
        if !names.insert(name.to_lowercase())
            || reqwest::header::HeaderName::from_bytes(name.as_bytes()).is_err()
            || value.len() > 16384
            || reqwest::header::HeaderValue::from_str(value).is_err()
        {
            return Err(invalid("provider_auth_source", "所选渠道认证头无效或重复"));
        }
    }
    Ok(())
}
fn state(auth: &Auth) -> KeyState {
    let ids = if auth.api_key_pool.is_empty() {
        auth.api_key.iter().map(|key| key_id(key)).collect()
    } else {
        auth.api_key_pool.iter().map(|s| s.id.clone()).collect()
    };
    let active_id = auth
        .api_key_pool
        .iter()
        .find(|s| Some(&s.key) == auth.api_key.as_ref())
        .map(|s| s.id.clone())
        .or_else(|| auth.api_key.as_ref().map(|key| key_id(key)));
    KeyState { ids, active_id }
}
trait SafeStore {
    fn get(&self, account: &str) -> ModelResult<Option<Zeroizing<String>>>;
    fn put(&self, account: &str, value: &str) -> ModelResult<()>;
}
struct System;
impl SafeStore for System {
    fn get(&self, account: &str) -> ModelResult<Option<Zeroizing<String>>> {
        let entry = keyring::Entry::new(
            crate::infrastructure::keychain::keychain_service_name(),
            account,
        )
        .map_err(|_| invalid("provider_auth_keychain", "授权恢复安全存储不可用"))?;
        match entry.get_password() {
            Ok(s) => Ok(Some(Zeroizing::new(s))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(invalid(
                "provider_auth_keychain",
                "授权恢复安全存储无法读取",
            )),
        }
    }
    fn put(&self, account: &str, value: &str) -> ModelResult<()> {
        if self.get(account)?.is_some() {
            return Err(invalid(
                "provider_auth_keychain",
                "授权保护账户位已存在，未覆盖",
            ));
        }
        let entry = keyring::Entry::new(
            crate::infrastructure::keychain::keychain_service_name(),
            account,
        )
        .map_err(|_| invalid("provider_auth_keychain", "授权恢复安全存储不可用"))?;
        entry
            .set_password(value)
            .map_err(|_| invalid("provider_auth_keychain", "授权保护密钥未写入"))?;
        if self.get(account)?.as_deref().map(|s| s.as_str()) != Some(value) {
            return Err(invalid("provider_auth_keychain", "授权保护密钥未读回"));
        }
        Ok(())
    }
}
fn reference_digests(auth: &Auth, provider: &str) -> ModelResult<BTreeMap<String, String>> {
    let mut digests = BTreeMap::new();
    for raw in auth
        .api_key
        .iter()
        .chain(auth.api_key_pool.iter().map(|s| &s.key))
        .chain(auth.headers.values())
    {
        if let Some(name) = raw.strip_prefix('$') {
            let name = name
                .strip_prefix('{')
                .and_then(|s| s.strip_suffix('}'))
                .unwrap_or(name);
            if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_') {
                return Err(invalid(
                    "provider_auth_reference",
                    "授权环境引用格式无法安全解析",
                ));
            }
            let value = std::env::var(name)
                .ok()
                .map(Zeroizing::new)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| invalid("provider_auth_reference", "授权环境引用不可用"))?;
            digests.insert(raw.clone(), sha256(value.as_bytes()));
        }
        if let Some(account) = raw.strip_prefix("keychain:") {
            if account != provider && !account.starts_with(&format!("{provider}/")) {
                return Err(invalid(
                    "provider_auth_reference",
                    "所选渠道引用了其他渠道安全凭据，未读取",
                ));
            }
            let value = keyring::Entry::new("opencodex.provider-api-key.v1", account)
                .map_err(|_| invalid("provider_auth_reference", "官方安全凭据不可用"))?
                .get_password()
                .map(Zeroizing::new)
                .map_err(|_| invalid("provider_auth_reference", "官方安全凭据无法读取"))?;
            if value.is_empty() {
                return Err(invalid("provider_auth_reference", "官方安全凭据为空"));
            }
            digests.insert(raw.clone(), sha256(value.as_bytes()));
        }
    }
    Ok(digests)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Backup {
    pub id: String,
    pub provider: String,
    pub digest: String,
    pub file_digest: String,
    pub state: KeyState,
}
fn path(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let path = root
        .join("manager-state/provider-auth-backups")
        .join(format!("{id}.ocxdconf"));
    safety::check_path(root, &path, true)?;
    Ok(path)
}
fn account(id: &str) -> String {
    format!("model-provider-protection:{id}")
}
fn freeze(
    root: &Path,
    provider: &str,
    value: &Protected,
    credentials: &impl SafeStore,
) -> ModelResult<Backup> {
    let id = uuid::Uuid::new_v4().to_string();
    let password = Zeroizing::new(format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    ));
    let document = Zeroizing::new(serde_json::to_vec(value)?);
    let bytes = portable_file::seal(
        portable_file::Manifest {
            id: id.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            sections: vec!["providerAuthorization".into()],
            secret_objects: value
                .auth
                .api_key_pool
                .len()
                .max(usize::from(value.auth.api_key.is_some()))
                + value.auth.headers.len(),
            document_digest: String::new(),
        },
        &document,
        Some(password.as_str()),
    )?;
    credentials.put(&account(&id), password.as_str())?;
    atomic_write(&path(root, &id)?, &bytes, 0o600)?;
    let backup = Backup {
        id,
        provider: provider.into(),
        digest: sha256(&document),
        file_digest: sha256(&bytes),
        state: state(&value.auth),
    };
    unseal(root, &backup, credentials)?;
    Ok(backup)
}
fn unseal(root: &Path, backup: &Backup, credentials: &impl SafeStore) -> ModelResult<Protected> {
    let bytes = safety::read_file(&path(root, &backup.id)?, portable_file::FILE_LIMIT as u64)?;
    if sha256(&bytes) != backup.file_digest {
        return Err(invalid("provider_auth_digest", "授权保护文件摘要改变"));
    }
    let password = credentials
        .get(&account(&backup.id))?
        .ok_or_else(|| invalid("provider_auth_keychain", "授权恢复密钥不存在"))?;
    let opened = portable_file::open(&bytes, Some(password.as_str()))?;
    if !opened.protected
        || opened.manifest.id != backup.id
        || opened.manifest.sections != vec!["providerAuthorization"]
        || sha256(&opened.document) != backup.digest
    {
        return Err(invalid("provider_auth_digest", "授权保护身份或内容改变"));
    }
    let value: Protected = serde_json::from_slice(&opened.document)
        .map_err(|_| invalid("provider_auth_source", "授权保护文档无法解析"))?;
    validate(&value.auth)?;
    if value.provider != backup.provider {
        return Err(invalid("provider_auth_scope", "授权保护文件属于另一渠道"));
    }
    if state(&value.auth) != backup.state {
        return Err(invalid("provider_auth_digest", "授权保护Key池与清单不同"));
    }
    Ok(value)
}
fn references_match(value: &Protected, provider: &str) -> ModelResult<()> {
    if reference_digests(&value.auth, provider)? != value.reference_digests {
        return Err(invalid(
            "provider_auth_reference",
            "授权引用的实际内容已改变，未恢复旧配置",
        ));
    }
    Ok(())
}
pub async fn capture(
    root: &Path,
    context: &ProcessContext,
    provider: &str,
    expected: &KeyState,
) -> ModelResult<Option<Backup>> {
    let auth = read(context, provider)?;
    if &state(&auth) != expected {
        return Err(invalid(
            "provider_auth_drift",
            "官方授权源与活动Key池不同，未删除",
        ));
    }
    if expected.ids.is_empty() && auth.headers.is_empty() {
        return Ok(None);
    }
    let reference_digests = reference_digests(&auth, provider)?;
    let backup = freeze(
        root,
        provider,
        &Protected {
            provider: provider.into(),
            binding: super::runtime::runtime_binding(context)?,
            auth,
            reference_digests,
        },
        &System,
    )?;
    verify(root, context, &backup).await?;
    Ok(Some(backup))
}
pub fn check_backup(root: &Path, backup: &Backup) -> ModelResult<()> {
    references_match(&unseal(root, backup, &System)?, &backup.provider)
}
pub fn verify_absent(context: &ProcessContext, provider: &str) -> ModelResult<()> {
    let current = read(context, provider)?;
    if current.api_key.is_some() || !current.api_key_pool.is_empty() || !current.headers.is_empty()
    {
        return Err(invalid(
            "provider_auth_drift",
            "原本无授权的渠道出现新的Key或认证头，未覆盖",
        ));
    }
    Ok(())
}
pub async fn verify(root: &Path, context: &ProcessContext, backup: &Backup) -> ModelResult<()> {
    let value = unseal(root, backup, &System)?;
    if value.binding != super::runtime::runtime_binding(context)? {
        return Err(invalid(
            "provider_auth_scope",
            "授权保护文件的官方运行根不同",
        ));
    }
    references_match(&value, &backup.provider)?;
    let current = read(context, &backup.provider)?;
    let left = Zeroizing::new(serde_json::to_vec(&current)?);
    let right = Zeroizing::new(serde_json::to_vec(&value.auth)?);
    if left.as_slice() != right.as_slice() {
        return Err(invalid(
            "provider_auth_drift",
            "授权字段或Key池标签在确认后改变，未继续",
        ));
    }
    if Management::bind(context)
        .await?
        .key_state(&backup.provider)
        .await?
        != backup.state
    {
        return Err(invalid("provider_auth_drift", "官方活动Key池在确认后改变"));
    }
    Ok(())
}
fn is_owned_subset(current: &Auth, expected: &Auth) -> bool {
    let expected_key = |id: &str, key: &str| {
        expected
            .api_key_pool
            .iter()
            .any(|s| s.id == id && s.key == key)
            || (expected.api_key_pool.is_empty()
                && expected.api_key.as_deref() == Some(key)
                && key_id(key) == id)
    };
    let current_ok = current
        .api_key_pool
        .iter()
        .all(|s| expected_key(&s.id, &s.key))
        && current
            .api_key
            .as_ref()
            .is_none_or(|key| expected_key(&key_id(key), key));
    current_ok
        && (current.api_key.is_none()
            || current.api_key == expected.api_key
            || current
                .api_key_pool
                .last()
                .is_some_and(|slot| Some(&slot.key) == current.api_key.as_ref()))
        && current.api_key_pool.iter().enumerate().all(|(i, slot)| {
            expected
                .api_key_pool
                .get(i)
                .is_some_and(|s| s.id == slot.id && s.label == slot.label)
        })
        && current
            .headers
            .iter()
            .all(|(name, value)| expected.headers.get(name) == Some(value))
}
pub fn inspect_target(root: &Path, context: &ProcessContext, backup: &Backup) -> ModelResult<bool> {
    let expected = unseal(root, backup, &System)?;
    if expected.binding != super::runtime::runtime_binding(context)? {
        return Err(invalid(
            "provider_auth_scope",
            "授权恢复来源与当前运行根不同",
        ));
    }
    references_match(&expected, &backup.provider)?;
    let current = read(context, &backup.provider)?;
    if !is_owned_subset(&current, &expected.auth) {
        return Err(invalid(
            "provider_auth_drift",
            "恢复目标存在其他授权、顺序或标签，未覆盖",
        ));
    }
    let left = Zeroizing::new(serde_json::to_vec(&current)?);
    let right = Zeroizing::new(serde_json::to_vec(&expected.auth)?);
    Ok(left.as_slice() == right.as_slice())
}
pub async fn restore(root: &Path, context: &ProcessContext, backup: &Backup) -> ModelResult<()> {
    let value = unseal(root, backup, &System)?;
    if value.binding != super::runtime::runtime_binding(context)? {
        return Err(invalid(
            "provider_auth_scope",
            "授权恢复来源与当前运行根不同",
        ));
    }
    references_match(&value, &backup.provider)?;
    let current = read(context, &backup.provider)?;
    if !is_owned_subset(&current, &value.auth) {
        return Err(invalid(
            "provider_auth_drift",
            "恢复目标存在其他授权，未覆盖",
        ));
    }
    let left = Zeroizing::new(serde_json::to_vec(&current)?);
    let right = Zeroizing::new(serde_json::to_vec(&value.auth)?);
    if left.as_slice() == right.as_slice() {
        return verify(root, context, backup).await;
    }
    let management = Management::bind(context).await?;
    if value.auth.api_key_pool.is_empty() {
        if let Some(key) = &value.auth.api_key {
            management.add_key(&backup.provider, key).await?;
        }
    } else {
        for slot in &value.auth.api_key_pool {
            if !current.api_key_pool.iter().any(|s| s.id == slot.id) {
                management
                    .add_key_labeled(&backup.provider, &slot.key, slot.label.as_deref())
                    .await?;
            }
        }
    }
    if let Some(active) = &backup.state.active_id {
        management.activate_key(&backup.provider, active).await?;
    }
    if !value.auth.headers.is_empty() {
        management
            .patch_headers(&backup.provider, &value.auth.headers)
            .await?;
    }
    verify(root, context, backup).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::cell::RefCell;
    #[derive(Default)]
    struct Memory(RefCell<BTreeMap<String, String>>);
    impl SafeStore for Memory {
        fn get(&self, account: &str) -> ModelResult<Option<Zeroizing<String>>> {
            Ok(self.0.borrow().get(account).cloned().map(Zeroizing::new))
        }
        fn put(&self, account: &str, value: &str) -> ModelResult<()> {
            self.0.borrow_mut().insert(account.into(), value.into());
            Ok(())
        }
    }
    fn auth() -> Auth {
        let a = "fixture-first-SECRET";
        let b = "fixture-second-SECRET";
        Auth {
            api_key: Some(b.into()),
            api_key_pool: vec![
                Slot {
                    id: key_id(a),
                    key: a.into(),
                    label: Some("first".into()),
                },
                Slot {
                    id: key_id(b),
                    key: b.into(),
                    label: Some("second".into()),
                },
            ],
            headers: BTreeMap::from([("X-Authorization".into(), "fixture-header-SECRET".into())]),
        }
    }
    #[test]
    fn protects_exact_pool_order_labels_active_selection_and_headers_without_plaintext_in_files_or_metadata(
    ) {
        let root = tempfile::tempdir().unwrap();
        let store = Memory::default();
        let value = Protected {
            provider: "selected".into(),
            binding: "fixture".into(),
            auth: auth(),
            reference_digests: Default::default(),
        };
        let backup = freeze(root.path(), "selected", &value, &store).unwrap();
        let bytes = std::fs::read(path(root.path(), &backup.id).unwrap()).unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("SECRET"));
        assert!(!serde_json::to_string(&backup).unwrap().contains("SECRET"));
        let opened = unseal(root.path(), &backup, &store).unwrap();
        let mut wrong = backup.clone();
        wrong.provider = "other-provider".into();
        assert_eq!(
            unseal(root.path(), &wrong, &store).err().unwrap().code,
            "provider_auth_scope"
        );
        assert_eq!(
            serde_json::to_value(&opened.auth).unwrap(),
            serde_json::to_value(&value.auth).unwrap()
        );
        assert_eq!(
            backup.state.active_id,
            Some(key_id("fixture-second-SECRET"))
        );
        store.0.borrow_mut().clear();
        assert_eq!(
            unseal(root.path(), &backup, &store).err().unwrap().code,
            "provider_auth_keychain"
        );
    }
    #[test]
    fn selected_source_projection_ignores_other_provider_credentials_and_rejects_duplicate_source_keys(
    ) {
        let bytes=serde_json::to_vec(&json!({"otherPrivate":"ignore-SECRET","providers":{"other":{"apiKey":44},"selected":{"apiKey":"chosen-SECRET","unknown":{"credentials":"ignored-SECRET"}}}})).unwrap();
        let selected = parse(&bytes, "selected").unwrap();
        let text = Zeroizing::new(serde_json::to_string(&selected).unwrap());
        assert!(text.contains("chosen-SECRET"));
        assert!(!text.contains("ignore"));
        assert!(parse(
            br#"{"providers":{"selected":{"apiKey":"a"}},"providers":{"selected":{"apiKey":"b"}}}"#,
            "selected"
        )
        .is_err());
        assert!(parse(
            br#"{"providers":{"selected":{"headers":{"X-A":"a","x-a":"b"}}}}"#,
            "selected"
        )
        .is_err());
    }
    #[test]
    fn partial_restore_is_only_an_exact_owned_prefix_and_does_not_allow_new_credentials_or_labels()
    {
        let expected = auth();
        let mut partial = Auth {
            api_key: Some(expected.api_key_pool[0].key.clone()),
            api_key_pool: vec![Slot {
                id: expected.api_key_pool[0].id.clone(),
                key: expected.api_key_pool[0].key.clone(),
                label: expected.api_key_pool[0].label.clone(),
            }],
            headers: BTreeMap::new(),
        };
        assert!(is_owned_subset(&partial, &expected));
        partial.api_key_pool[0].label = Some("later edit".into());
        assert!(!is_owned_subset(&partial, &expected));
        partial.api_key_pool[0].label = expected.api_key_pool[0].label.clone();
        partial
            .headers
            .insert("X-Later".into(), "new-SECRET".into());
        assert!(!is_owned_subset(&partial, &expected));
    }
    #[test]
    fn a_new_private_header_on_previously_empty_auth_invalidates_deletion() {
        let root = tempfile::tempdir().unwrap();
        let context = ProcessContext {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::resolved(
                root.path().join("fixture-ocx"),
            ),
            working_directory: root.path().into(),
            opencodex_home: root.path().into(),
        };
        std::fs::write(
            root.path().join("config.json"),
            br#"{"providers":{"selected":{}}}"#,
        )
        .unwrap();
        verify_absent(&context, "selected").unwrap();
        std::fs::write(
            root.path().join("config.json"),
            br#"{"providers":{"selected":{"headers":{"X-New":"later-SECRET"}}}}"#,
        )
        .unwrap();
        assert_eq!(
            verify_absent(&context, "selected").unwrap_err().code,
            "provider_auth_drift"
        );
    }
    #[test]
    fn ciphertext_or_metadata_tampering_refuses_before_any_provider_write() {
        let root = tempfile::tempdir().unwrap();
        let store = Memory::default();
        let value = Protected {
            provider: "selected".into(),
            binding: "fixture".into(),
            auth: auth(),
            reference_digests: Default::default(),
        };
        let backup = freeze(root.path(), "selected", &value, &store).unwrap();
        let file = path(root.path(), &backup.id).unwrap();
        let mut bytes = std::fs::read(&file).unwrap();
        let end = bytes.len() - 1;
        bytes[end] ^= 1;
        std::fs::write(file, bytes).unwrap();
        assert_eq!(
            unseal(root.path(), &backup, &store).err().unwrap().code,
            "provider_auth_digest"
        );
    }
}
