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
use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::RngCore;
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
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct LocalEnvelope {
    format: String,
    version: u32,
    id: String,
    provider: String,
    binding: String,
    nonce_b64: String,
    ciphertext_b64: String,
}
fn local_aad(id: &str, provider: &str, binding: &str) -> ModelResult<Vec<u8>> {
    Ok(serde_json::to_vec(
        &serde_json::json!({"format":"OCXD_LOCAL_AUTH","version":1,"id":id,"provider":provider,"binding":binding}),
    )?)
}

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Auth {
    base_url: Option<String>,
    adapter: Option<String>,
    auth_mode: Option<String>,
    api_key_transport: Option<String>,
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
    #[serde(default, rename = "addedAt", skip_serializing_if = "Option::is_none")]
    added_at: Option<u64>,
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
                added_at: None,
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
            || slot.id.len() != 8
            || !slot
                .id
                .bytes()
                .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
            || (slot.id != key_id(&slot.key) && !slot.key.starts_with("keychain:"))
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
            .is_some_and(|key| !auth.api_key_pool.iter().any(|s| &s.key == key))
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
        .or_else(|| auth.api_key_pool.first())
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
    for slot in &auth.api_key_pool {
        if slot.id != key_id(&slot.key) && slot.key != format!("keychain:{provider}/{}", slot.id) {
            return Err(invalid(
                "provider_auth_reference",
                "安全引用与原Key池身份不对应，未读取",
            ));
        }
    }
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
    let mut key = Zeroizing::new([0_u8; 32]);
    let mut nonce = [0_u8; 12];
    rand::rng().fill_bytes(key.as_mut());
    rand::rng().fill_bytes(&mut nonce);
    let document = Zeroizing::new(serde_json::to_vec(value)?);
    if document.len() > 4 * 1024 * 1024 {
        return Err(invalid("provider_auth_limit", "本机授权保护正文超过4MiB"));
    }
    let cipher = Aes256Gcm::new_from_slice(key.as_ref())
        .map_err(|_| invalid("provider_auth_cipher", "本机授权保护未就绪"))?;
    let aad = local_aad(&id, provider, &value.binding)?;
    let ciphertext = cipher
        .encrypt(
            Nonce::from_slice(&nonce),
            Payload {
                msg: &document,
                aad: &aad,
            },
        )
        .map_err(|_| invalid("provider_auth_cipher", "本机授权保护失败"))?;
    let bytes = serde_json::to_vec(&LocalEnvelope {
        format: "OCXD_LOCAL_AUTH".into(),
        version: 1,
        id: id.clone(),
        provider: provider.into(),
        binding: value.binding.clone(),
        nonce_b64: STANDARD.encode(nonce),
        ciphertext_b64: STANDARD.encode(ciphertext),
    })?;
    let encoded_key = Zeroizing::new(STANDARD.encode(key.as_ref()));
    credentials.put(&account(&id), encoded_key.as_str())?;
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
    let document = if serde_json::from_slice::<serde_json::Value>(&bytes)
        .map_err(|_| invalid("provider_auth_source", "授权保护文件无法解析"))?["format"]
        == "OCXD_LOCAL_AUTH"
    {
        let envelope: LocalEnvelope = serde_json::from_slice(&bytes)
            .map_err(|_| invalid("provider_auth_source", "本机授权保护格式未知"))?;
        if envelope.version != 1
            || envelope.id != backup.id
            || envelope.provider != backup.provider
            || envelope.ciphertext_b64.len() > 6 * 1024 * 1024
        {
            return Err(invalid(
                "provider_auth_scope",
                "本机授权保护身份、版本或大小不同",
            ));
        }
        let key = Zeroizing::new(
            STANDARD
                .decode(password.as_bytes())
                .map_err(|_| invalid("provider_auth_keychain", "本机保护密钥编码无效"))?,
        );
        let nonce = STANDARD
            .decode(&envelope.nonce_b64)
            .map_err(|_| invalid("provider_auth_cipher", "保护随机数无效"))?;
        if key.len() != 32 || nonce.len() != 12 {
            return Err(invalid("provider_auth_cipher", "保护密钥或随机数长度无效"));
        }
        let ciphertext = STANDARD
            .decode(&envelope.ciphertext_b64)
            .map_err(|_| invalid("provider_auth_cipher", "保护载荷编码无效"))?;
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|_| invalid("provider_auth_cipher", "保护密钥无效"))?;
        let aad = local_aad(&envelope.id, &envelope.provider, &envelope.binding)?;
        Zeroizing::new(
            cipher
                .decrypt(
                    Nonce::from_slice(&nonce),
                    Payload {
                        msg: &ciphertext,
                        aad: &aad,
                    },
                )
                .map_err(|_| invalid("provider_auth_cipher", "本机授权保护标签校验失败"))?,
        )
    } else {
        let opened = portable_file::open(&bytes, Some(password.as_str()))?;
        if !opened.protected
            || opened.manifest.id != backup.id
            || opened.manifest.sections != vec!["providerAuthorization"]
        {
            return Err(invalid("provider_auth_digest", "旧保护文件身份不同"));
        }
        opened.document
    };
    if sha256(&document) != backup.digest {
        return Err(invalid("provider_auth_digest", "授权保护正文摘要改变"));
    }
    let value: Protected = serde_json::from_slice(&document)
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
    if current.base_url != expected.base_url
        || current.adapter != expected.adapter
        || current.auth_mode.as_deref().unwrap_or("key")
            != expected.auth_mode.as_deref().unwrap_or("key")
        || current.api_key_transport != expected.api_key_transport
    {
        return false;
    }
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
        && current.api_key.as_ref().is_none_or(|key| {
            current
                .api_key_pool
                .iter()
                .find(|s| &s.key == key)
                .map(|s| expected_key(&s.id, key))
                .unwrap_or_else(|| expected_key(&key_id(key), key))
        });
    current_ok
        && (current.api_key.is_none()
            || current.api_key == expected.api_key
            || current
                .api_key_pool
                .last()
                .is_some_and(|slot| Some(&slot.key) == current.api_key.as_ref()))
        && current.api_key_pool.iter().enumerate().all(|(i, slot)| {
            expected.api_key_pool.get(i).is_some_and(|s| {
                s.id == slot.id && s.label == slot.label && s.added_at == slot.added_at
            })
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
    management.reload_provider(&backup.provider).await?;
    let public =
        super::upstream::read(context, super::upstream::ReadOperation::ProviderSnapshot).await?;
    let fresh = read(context, &backup.provider)?;
    if !is_owned_subset(&fresh, &value.auth) {
        return Err(invalid(
            "provider_auth_drift",
            "授权恢复前目标又发生变化，未写入",
        ));
    }
    management
        .restore_authorization_fields(
            &backup.provider,
            &public["providers"][&backup.provider],
            serde_json::json!({"apiKeyPool":value.auth.api_key_pool,"headers":value.auth.headers}),
        )
        .await?;
    if value.auth.api_key.is_some() {
        if let Some(active) = &backup.state.active_id {
            management.activate_key(&backup.provider, active).await?;
        }
    }
    verify(root, context, backup).await
}

fn auth_digest(auth: &Auth) -> ModelResult<String> {
    Ok(sha256(&Zeroizing::new(serde_json::to_vec(auth)?)))
}
fn transport_matches(left: &Auth, right: &Auth) -> bool {
    left.base_url == right.base_url
        && left.adapter == right.adapter
        && left.auth_mode.as_deref().unwrap_or("key") == right.auth_mode.as_deref().unwrap_or("key")
        && left.api_key_transport == right.api_key_transport
}
fn empty_auth(transport: &Auth) -> Auth {
    Auth {
        base_url: transport.base_url.clone(),
        adapter: transport.adapter.clone(),
        auth_mode: transport.auth_mode.clone(),
        api_key_transport: transport.api_key_transport.clone(),
        api_key: None,
        api_key_pool: vec![],
        headers: BTreeMap::new(),
    }
}
fn from_backup(
    root: &Path,
    context: &ProcessContext,
    provider: &str,
    backup: Option<&Backup>,
    transport: &Auth,
) -> ModelResult<Auth> {
    let Some(backup) = backup else {
        return Ok(empty_auth(transport));
    };
    if backup.provider != provider {
        return Err(invalid("provider_auth_scope", "授权回退目标不属于当前渠道"));
    }
    let value = unseal(root, backup, &System)?;
    if value.binding != super::runtime::runtime_binding(context)? {
        return Err(invalid("provider_auth_scope", "授权回退所属运行根不同"));
    }
    references_match(&value, provider)?;
    let auth = serde_json::from_slice(&Zeroizing::new(serde_json::to_vec(&value.auth)?))
        .map_err(|_| invalid("provider_auth_source", "受保护授权无法重建"))?;
    Ok(auth)
}
pub async fn matches(
    root: &Path,
    context: &ProcessContext,
    provider: &str,
    backup: Option<&Backup>,
) -> ModelResult<bool> {
    let current = read(context, provider)?;
    let expected = from_backup(root, context, provider, backup, &current)?;
    Ok(auth_digest(&current)? == auth_digest(&expected)?
        && Management::bind(context).await?.key_state(provider).await? == state(&expected))
}
/// Called during the review, before allowing a historical private direction to
/// replace an existing pool. The current materials are independently protected.
pub fn review_change(
    root: &Path,
    context: &ProcessContext,
    provider: &str,
    before: Option<&Backup>,
    target: Option<&Backup>,
) -> ModelResult<()> {
    let current = read(context, provider)?;
    let original = from_backup(root, context, provider, before, &current)?;
    let desired = from_backup(root, context, provider, target, &current)?;
    if auth_digest(&current)? != auth_digest(&original)? {
        return Err(invalid(
            "provider_auth_drift",
            "授权比较期间当前选择已改变，请重新比较",
        ));
    }
    if !transport_matches(&current, &desired) {
        return Err(invalid(
            "provider_auth_direction",
            "恢复前后服务或认证方式不同，请先明确映射到正确连接，未发送旧授权",
        ));
    }
    Ok(())
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Transition {
    operation: String,
    provider: String,
    from_digest: String,
    target_digest: String,
    pool_digest: String,
    phase: String,
}
fn change_path(
    root: &Path,
    operation: &str,
    provider: &str,
    rollback: bool,
) -> ModelResult<PathBuf> {
    require_uuid(operation)?;
    let slug = sha256(provider.as_bytes());
    let slug = slug.trim_start_matches("sha256:");
    let path = root
        .join("manager-state/provider-auth-transitions")
        .join(format!(
            "{operation}-{slug}-{}.json",
            if rollback { "rollback" } else { "forward" }
        ));
    safety::check_path(root, &path, true)?;
    Ok(path)
}
fn save_change(file: &Path, value: &Transition) -> ModelResult<()> {
    let bytes = serde_json::to_vec(value)?;
    atomic_write(file, &bytes, 0o600)?;
    if safety::read_file(file, 16384)? != bytes {
        return Err(invalid("provider_auth_transition", "授权回退日志未读回"));
    }
    Ok(())
}
fn phase_allows(phase: &str, current: &str, from: &str, target: &str, pool: &str) -> bool {
    match phase {
        "prepared" => current == from,
        "pending_pool" => current == from || current == pool,
        "pool_written" => current == pool,
        "pending_active" => current == pool || current == target,
        "done" => current == target,
        _ => false,
    }
}
fn pool_auth(auth: &Auth) -> ModelResult<Auth> {
    let mut value: Auth = serde_json::from_slice(&Zeroizing::new(serde_json::to_vec(auth)?))
        .map_err(|_| invalid("provider_auth_source", "授权暂态无法解析"))?;
    if let Some(mut key) = value.api_key.take() {
        key.zeroize();
    }
    Ok(value)
}
fn pool_digest(auth: &Auth) -> ModelResult<String> {
    auth_digest(&pool_auth(auth)?)
}

fn read_change(
    file: &Path,
    operation: &str,
    provider: &str,
    from: &Auth,
    target: &Auth,
) -> ModelResult<Option<Transition>> {
    if !file.exists() {
        return Ok(None);
    }
    let record: Transition = serde_json::from_slice(&safety::read_file(file, 16384)?)?;
    if record.operation != operation
        || record.provider != provider
        || record.from_digest != auth_digest(from)?
        || record.target_digest != auth_digest(target)?
        || record.pool_digest != pool_digest(target)?
    {
        return Err(invalid(
            "provider_auth_transition",
            "原授权回退范围或目标已改变，请沿原计划核对",
        ));
    }
    Ok(Some(record))
}
async fn write_change(
    root: &Path,
    context: &ProcessContext,
    operation: &str,
    provider: &str,
    from: &Auth,
    target: &Auth,
    rollback: bool,
) -> ModelResult<()> {
    let file = change_path(root, operation, provider, rollback)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&file),
        true,
    )?;
    let _guard = crate::infrastructure::locking::TargetFileLock::lock(&file)?;
    let mut current = read(context, provider)?;
    if !transport_matches(&current, target) {
        return Err(invalid(
            "provider_auth_direction",
            "目标连接在授权回退前改变，未发送旧授权",
        ));
    }
    let mut record = read_change(&file, operation, provider, from, target)?.unwrap_or(Transition {
        operation: operation.into(),
        provider: provider.into(),
        from_digest: auth_digest(from)?,
        target_digest: auth_digest(target)?,
        pool_digest: pool_digest(target)?,
        phase: "prepared".into(),
    });
    let actual = auth_digest(&current)?;
    if !phase_allows(
        &record.phase,
        &actual,
        &record.from_digest,
        &record.target_digest,
        &record.pool_digest,
    ) {
        return Err(invalid(
            "provider_auth_drift",
            "当前授权不属于本次回退已核验的副作用，拒绝覆盖后来修改",
        ));
    }
    if actual == record.target_digest {
        record.phase = "done".into();
        return save_change(&file, &record);
    }
    let management = Management::bind(context).await?;
    management.reload_provider(provider).await?;
    if actual != record.pool_digest {
        record.phase = "pending_pool".into();
        save_change(&file, &record)?;
        let public =
            super::upstream::read(context, super::upstream::ReadOperation::ProviderSnapshot)
                .await?;
        if auth_digest(&read(context, provider)?)? != actual {
            return Err(invalid(
                "provider_auth_drift",
                "授权写入前检测到后来修改，未覆盖",
            ));
        }
        management
            .restore_authorization_fields(
                provider,
                &public["providers"][provider],
                serde_json::json!({"apiKeyPool":target.api_key_pool,"headers":target.headers}),
            )
            .await?;
        current = read(context, provider)?;
        if auth_digest(&current)? != record.pool_digest {
            return Err(invalid(
                "provider_auth_readback",
                "授权池和认证头未读回本次目标，原计划待核对",
            ));
        }
    }
    record.phase = "pool_written".into();
    save_change(&file, &record)?;
    if let Some(active) = state(target).active_id.filter(|_| target.api_key.is_some()) {
        record.phase = "pending_active".into();
        save_change(&file, &record)?;
        if auth_digest(&read(context, provider)?)? != record.pool_digest {
            return Err(invalid(
                "provider_auth_drift",
                "恢复活动选择前授权又改变，未覆盖",
            ));
        }
        management.activate_key(provider, &active).await?;
    }
    if auth_digest(&read(context, provider)?)? != record.target_digest
        || management.key_state(provider).await? != state(target)
    {
        return Err(invalid(
            "provider_auth_readback",
            "授权回退或活动选择读回不同，保留原执行记录",
        ));
    }
    record.phase = "done".into();
    save_change(&file, &record)
}
pub async fn change(
    root: &Path,
    context: &ProcessContext,
    operation: &str,
    provider: &str,
    before: Option<&Backup>,
    target: Option<&Backup>,
) -> ModelResult<()> {
    let current = read(context, provider)?;
    let original = from_backup(root, context, provider, before, &current)?;
    let desired = from_backup(root, context, provider, target, &current)?;
    write_change(
        root, context, operation, provider, &original, &desired, false,
    )
    .await
}
pub fn inspect_change(
    root: &Path,
    context: &ProcessContext,
    operation: &str,
    provider: &str,
    before: Option<&Backup>,
    target: Option<&Backup>,
) -> ModelResult<()> {
    let current = read(context, provider)?;
    let original = from_backup(root, context, provider, before, &current)?;
    let desired = from_backup(root, context, provider, target, &current)?;
    let file = change_path(root, operation, provider, false)?;
    let actual = auth_digest(&current)?;
    let reverse = change_path(root, operation, provider, true)?;
    if reverse.exists() {
        let record: Transition = serde_json::from_slice(&safety::read_file(&reverse, 16384)?)?;
        let from = if record.from_digest == pool_digest(&desired)? {
            pool_auth(&desired)?
        } else {
            from_backup(root, context, provider, target, &current)?
        };
        let record = read_change(&reverse, operation, provider, &from, &original)?
            .ok_or_else(|| invalid("provider_auth_transition", "补偿日志缺失"))?;
        if !phase_allows(
            &record.phase,
            &actual,
            &record.from_digest,
            &record.target_digest,
            &record.pool_digest,
        ) {
            return Err(invalid(
                "provider_auth_drift",
                "授权补偿后存在后来修改，拒绝重放",
            ));
        }
        return Ok(());
    }
    if let Some(record) = read_change(&file, operation, provider, &original, &desired)? {
        if !phase_allows(
            &record.phase,
            &actual,
            &record.from_digest,
            &record.target_digest,
            &record.pool_digest,
        ) {
            return Err(invalid(
                "provider_auth_drift",
                "原授权回退后存在无法归属的后来修改，拒绝补偿",
            ));
        }
    } else if actual != auth_digest(&original)? {
        return Err(invalid(
            "provider_auth_drift",
            "授权在回退开始前已经改变，未覆盖",
        ));
    }
    Ok(())
}
pub async fn rollback_change(
    root: &Path,
    context: &ProcessContext,
    operation: &str,
    provider: &str,
    before: Option<&Backup>,
    target: Option<&Backup>,
) -> ModelResult<()> {
    let current = read(context, provider)?;
    let original = from_backup(root, context, provider, before, &current)?;
    let desired = from_backup(root, context, provider, target, &current)?;
    let rollback = change_path(root, operation, provider, true)?;
    if rollback.exists() {
        // Resume the reverse intent from its own frozen digests. Its starting
        // snapshot was the forward target or exact pool-written intermediate.
        let record: Transition = serde_json::from_slice(&safety::read_file(&rollback, 16384)?)?;
        let mut from =
            serde_json::from_slice::<Auth>(&Zeroizing::new(serde_json::to_vec(&desired)?))
                .map_err(|_| invalid("provider_auth_source", "授权回退暂态无效"))?;
        if record.from_digest == pool_digest(&desired)? {
            if let Some(mut key) = from.api_key.take() {
                key.zeroize();
            }
        }
        return write_change(root, context, operation, provider, &from, &original, true).await;
    }
    inspect_change(root, context, operation, provider, before, target)?;
    if auth_digest(&current)? == auth_digest(&original)? {
        return Ok(());
    }
    write_change(
        root, context, operation, provider, &current, &original, true,
    )
    .await
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
            base_url: Some("https://fixture.example.test/v1".into()),
            adapter: Some("openai-chat".into()),
            auth_mode: Some("key".into()),
            api_key_transport: None,
            api_key: Some(b.into()),
            api_key_pool: vec![
                Slot {
                    added_at: None,
                    id: key_id(a),
                    key: a.into(),
                    label: Some("first".into()),
                },
                Slot {
                    added_at: None,
                    id: key_id(b),
                    key: b.into(),
                    label: Some("second".into()),
                },
            ],
            headers: BTreeMap::from([("X-Authorization".into(), "fixture-header-SECRET".into())]),
        }
    }
    #[test]
    fn transition_phases_only_admit_the_exact_issued_write_or_active_choice() {
        let expected = [
            ("prepared", vec!["from"]),
            ("pending_pool", vec!["from", "pool"]),
            ("pool_written", vec!["pool"]),
            ("pending_active", vec!["pool", "target"]),
            ("done", vec!["target"]),
            ("invalid", vec![]),
        ];
        for (phase, allowed) in expected {
            for actual in ["from", "pool", "target", "later"] {
                assert_eq!(
                    phase_allows(phase, actual, "from", "target", "pool"),
                    allowed.contains(&actual),
                    "{phase}/{actual}"
                );
            }
        }
        let target = auth();
        let mut pool = pool_auth(&target).unwrap();
        assert!(pool.api_key.is_none());
        assert_eq!(auth_digest(&pool).unwrap(), pool_digest(&target).unwrap());
        pool.api_key_pool[0].label = Some("later label".into());
        assert_ne!(auth_digest(&pool).unwrap(), pool_digest(&target).unwrap());
    }
    #[test]
    fn transition_log_binds_operation_provider_full_pool_and_direction() {
        let root = tempfile::tempdir().unwrap();
        let operation = uuid::Uuid::new_v4().to_string();
        let target = auth();
        let original = empty_auth(&target);
        let file = change_path(root.path(), &operation, "selected", false).unwrap();
        save_change(
            &file,
            &Transition {
                operation: operation.clone(),
                provider: "selected".into(),
                from_digest: auth_digest(&original).unwrap(),
                target_digest: auth_digest(&target).unwrap(),
                pool_digest: pool_digest(&target).unwrap(),
                phase: "pending_active".into(),
            },
        )
        .unwrap();
        assert!(
            read_change(&file, &operation, "selected", &original, &target)
                .unwrap()
                .is_some()
        );
        assert!(read_change(&file, &operation, "other", &original, &target).is_err());
        assert!(read_change(
            &file,
            &uuid::Uuid::new_v4().to_string(),
            "selected",
            &original,
            &target
        )
        .is_err());
        assert!(read_change(&file, &operation, "selected", &target, &original).is_err());
        let mut later = auth();
        later.api_key_pool[0].added_at = Some(555);
        assert!(read_change(&file, &operation, "selected", &original, &later).is_err());
        assert!(!String::from_utf8(std::fs::read(file).unwrap())
            .unwrap()
            .contains("SECRET"));
    }
    #[test]
    fn local_machine_key_aad_rejects_binding_tampering_even_when_file_checksum_is_recomputed() {
        let root = tempfile::tempdir().unwrap();
        let store = Memory::default();
        let value = Protected {
            provider: "selected".into(),
            binding: "fixture-root".into(),
            auth: auth(),
            reference_digests: Default::default(),
        };
        let mut backup = freeze(root.path(), "selected", &value, &store).unwrap();
        let file = path(root.path(), &backup.id).unwrap();
        let mut envelope: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&file).unwrap()).unwrap();
        envelope["binding"] = json!("another-runtime-root");
        let bytes = serde_json::to_vec(&envelope).unwrap();
        backup.file_digest = sha256(&bytes);
        std::fs::write(&file, &bytes).unwrap();
        assert_eq!(
            unseal(root.path(), &backup, &store).err().unwrap().code,
            "provider_auth_cipher"
        );
        envelope["version"] = json!(500);
        let bytes = serde_json::to_vec(&envelope).unwrap();
        backup.file_digest = sha256(&bytes);
        std::fs::write(file, bytes).unwrap();
        assert_eq!(
            unseal(root.path(), &backup, &store).err().unwrap().code,
            "provider_auth_scope"
        );
    }
    #[test]
    fn previous_password_protected_local_backups_remain_readable() {
        let root = tempfile::tempdir().unwrap();
        let store = Memory::default();
        let value = Protected {
            provider: "selected".into(),
            binding: "fixture-root".into(),
            auth: auth(),
            reference_digests: Default::default(),
        };
        let id = uuid::Uuid::new_v4().to_string();
        let document = Zeroizing::new(serde_json::to_vec(&value).unwrap());
        let bytes = portable_file::seal(
            portable_file::Manifest {
                id: id.clone(),
                created_at: chrono::Utc::now().to_rfc3339(),
                sections: vec!["providerAuthorization".into()],
                secret_objects: 3,
                document_digest: String::new(),
            },
            &document,
            Some("fixture-legacy-machine-passphrase"),
        )
        .unwrap();
        store
            .put(&account(&id), "fixture-legacy-machine-passphrase")
            .unwrap();
        atomic_write(&path(root.path(), &id).unwrap(), &bytes, 0o600).unwrap();
        let backup = Backup {
            id,
            provider: "selected".into(),
            digest: sha256(&document),
            file_digest: sha256(&bytes),
            state: state(&value.auth),
        };
        assert_eq!(
            unseal(root.path(), &backup, &store)
                .unwrap()
                .auth
                .api_key_pool
                .len(),
            2
        );
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
            base_url: expected.base_url.clone(),
            adapter: expected.adapter.clone(),
            auth_mode: expected.auth_mode.clone(),
            api_key_transport: expected.api_key_transport.clone(),
            api_key: Some(expected.api_key_pool[0].key.clone()),
            api_key_pool: vec![Slot {
                added_at: None,
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
        partial.headers.clear();
        partial.base_url = Some("https://other.example.test/v1".into());
        assert!(!is_owned_subset(&partial, &expected));
    }
    #[test]
    fn keychain_migrated_pool_identity_is_preserved_instead_of_hashing_the_reference() {
        let first = "keychain:selected/0123abcd";
        let second = "keychain:selected/8765dcba";
        let value = Auth {
            api_key: Some(second.into()),
            api_key_pool: vec![
                Slot {
                    id: "0123abcd".into(),
                    key: first.into(),
                    label: Some("original first".into()),
                    added_at: Some(100),
                },
                Slot {
                    id: "8765dcba".into(),
                    key: second.into(),
                    label: None,
                    added_at: Some(200),
                },
            ],
            base_url: None,
            adapter: None,
            auth_mode: None,
            api_key_transport: None,
            headers: BTreeMap::new(),
        };
        validate(&value).unwrap();
        assert_eq!(state(&value).active_id.as_deref(), Some("8765dcba"));
        assert_ne!(key_id(first), "0123abcd");
        assert!(is_owned_subset(&value, &value));
        let root = tempfile::tempdir().unwrap();
        let store = Memory::default();
        let backup = freeze(
            root.path(),
            "selected",
            &Protected {
                provider: "selected".into(),
                binding: "fixture".into(),
                auth: value,
                reference_digests: BTreeMap::new(),
            },
            &store,
        )
        .unwrap();
        let restored = unseal(root.path(), &backup, &store).unwrap();
        assert_eq!(restored.auth.api_key_pool[0].id, "0123abcd");
        assert_eq!(restored.auth.api_key_pool[0].added_at, Some(100));
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
