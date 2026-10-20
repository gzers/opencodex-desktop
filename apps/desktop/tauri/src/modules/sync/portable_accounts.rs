//! Account information is a separate object from a portable authorization grant.
//! Read only allowlisted metadata, without refreshing tokens or opening linked
//! browser/Orca stores. No adapter is declared portable without cross-device evidence.
use crate::{
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::{
        backup::safety,
        model_config::{schema::*, store::sha256},
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use zeroize::Zeroizing;

pub const IMPORTED: &str = "manager-state/imported-account-info.json";
const IDS: &str = "manager-state/account-catalog-ids.json";
const LIMIT: u64 = 8 * 1024 * 1024;
const MAX_ACCOUNTS: usize = 1000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AccountInfo {
    pub id: String,
    pub provider: String,
    pub subject: Option<String>,
    pub label: String,
    pub email: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<AccountOrigin>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AccountOrigin {
    pub version: u32,
    pub account_platform: String,
    pub owner: String,
    pub auth_kind: String,
    pub source_namespace: String,
    pub source_account_id: String,
    pub tenant: Option<String>,
}
fn platform(provider: &str) -> Option<&'static str> {
    match provider {
        "chatgpt" => Some("chatgpt"),
        "google-antigravity" => Some("antigravity"),
        "kiro" => Some("kiro"),
        "muse" => Some("muse"),
        _ => None,
    }
}
pub fn information_supported(account: &AccountInfo) -> bool {
    let Some(expected) = platform(&account.provider) else {
        return false;
    };
    // Old information-only records remain readable without inventing their owner.
    account.origin.as_ref().is_none_or(|origin| {
        origin.version == 1
            && origin.account_platform == expected
            && origin.owner == "opencodex"
            && origin.source_namespace
                == if account.provider == "chatgpt" {
                    "codex-account-pool"
                } else {
                    "oauth-account-pool"
                }
            && match account.provider.as_str() {
                "kiro" => {
                    ["unknown", "kiro_desktop", "aws_sso_oidc"].contains(&origin.auth_kind.as_str())
                }
                _ => origin.auth_kind == "oauth",
            }
    })
}
/// Cross-device association needs a known subject and the same origin domain.
/// Labels, email, raw slot IDs and absent origin never identify a new-format account.
pub fn same_subject(a: &AccountInfo, b: &AccountInfo) -> bool {
    if a.provider != b.provider || a.subject.is_none() || a.subject != b.subject {
        return false;
    }
    match (&a.origin, &b.origin) {
        (Some(a), Some(b)) => {
            a.version == b.version
                && a.account_platform == b.account_platform
                && a.owner == b.owner
                && a.auth_kind == b.auth_kind
                && a.source_namespace == b.source_namespace
                && a.tenant == b.tenant
        }
        // Exact old object identity is preserved; no cross-ID legacy inference.
        _ => a.id == b.id,
    }
}
fn same_object(a: &AccountInfo, b: &AccountInfo) -> bool {
    a.provider == b.provider
        && a.subject == b.subject
        && match (&a.origin, &b.origin) {
            (Some(left), Some(right)) => {
                if a.subject.is_some() {
                    same_subject(a, b)
                } else {
                    left == right
                }
            }
            _ => a.id == b.id,
        }
}
pub fn same_identity(a: &AccountInfo, b: &AccountInfo) -> bool {
    (a.id == b.id && same_object(a, b)) || same_subject(a, b)
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LoginTransfer {
    ManualImportDisabled,
    CrossDeviceEvidenceMissing,
    DedicatedAdapterRequired,
    UnsupportedProvider,
}
pub fn login_transfer(provider: &str) -> LoginTransfer {
    match provider {
        "chatgpt" => LoginTransfer::ManualImportDisabled,
        "google-antigravity" => LoginTransfer::CrossDeviceEvidenceMissing,
        "kiro" | "muse" => LoginTransfer::DedicatedAdapterRequired,
        _ => LoginTransfer::UnsupportedProvider,
    }
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogEntry {
    pub account: AccountInfo,
    pub login_transfer: LoginTransfer,
    pub local_source: bool,
    pub information_supported: bool,
}
fn text_ok(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}
pub fn validate(accounts: &[AccountInfo]) -> ModelResult<()> {
    let mut ids = BTreeSet::new();
    let mut identities = BTreeSet::new();
    if accounts.len() > MAX_ACCOUNTS {
        return Err(invalid("account_info", "账户信息超过数量上限"));
    }
    for account in accounts {
        require_uuid(&account.id)?;
        if !ids.insert(&account.id)
            || !text_ok(&account.provider, 128)
            || !account
                .provider
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
            || !text_ok(&account.label, 256)
            || account.subject.as_ref().is_some_and(|s| !text_ok(s, 512))
            || account
                .email
                .as_ref()
                .is_some_and(|s| !text_ok(s, 254) || !s.contains('@'))
            || account.subject.as_ref().is_some_and(|s| {
                !identities.insert((
                    &account.provider,
                    s,
                    account.origin.as_ref().map(|o| {
                        (
                            &o.account_platform,
                            &o.owner,
                            &o.auth_kind,
                            &o.source_namespace,
                            &o.tenant,
                        )
                    }),
                ))
            })
        {
            return Err(invalid("account_info", "账户身份重复或显示信息无效"));
        }
        if let Some(origin) = &account.origin {
            if origin.version != 1
                || !text_ok(&origin.account_platform, 128)
                || !text_ok(&origin.owner, 128)
                || !text_ok(&origin.auth_kind, 128)
                || !text_ok(&origin.source_namespace, 128)
                || !text_ok(&origin.source_account_id, 512)
                || origin
                    .tenant
                    .as_ref()
                    .is_some_and(|value| !text_ok(value, 512))
            {
                return Err(invalid("account_origin", "账户平台或来源信息无效"));
            }
        }
    }
    Ok(())
}
fn read(root: &Path, relative: &str) -> ModelResult<Option<Zeroizing<Vec<u8>>>> {
    let path = root.join(relative);
    safety::check_path(root, &path, true)?;
    match std::fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
        Ok(_) => safety::read_file(&path, LIMIT)
            .map(Zeroizing::new)
            .map(Some)
            .map_err(Into::into),
    }
}
pub fn imported(root: &Path) -> ModelResult<Vec<AccountInfo>> {
    let accounts = read(root, IMPORTED)?
        .map(|bytes| serde_json::from_slice::<Vec<AccountInfo>>(&bytes))
        .transpose()
        .map_err(|_| invalid("account_info", "本机账户信息记录无法解析"))?
        .unwrap_or_default();
    validate(&accounts)?;
    Ok(accounts)
}
/// Conflicting stable IDs never rename another identity. Email/alias alone does
/// not merge accounts. Existing information wins on a matching provider subject.
pub fn merge(
    mut local: Vec<AccountInfo>,
    incoming: &[AccountInfo],
) -> ModelResult<Vec<AccountInfo>> {
    validate(&local)?;
    validate(incoming)?;
    for account in incoming {
        if !information_supported(account) {
            return Err(invalid(
                "account_capability",
                "尚未支持此账户平台或来源的信息采用",
            ));
        }
        if let Some(existing) = local.iter().find(|a| a.id == account.id) {
            if !same_object(existing, account) {
                return Err(invalid(
                    "account_identity_conflict",
                    "相同账户标识指向不同身份，未覆盖",
                ));
            }
        } else if !local.iter().any(|a| same_subject(a, account)) {
            local.push(account.clone());
        }
    }
    validate(&local)?;
    Ok(local)
}

// Upstream read projections deliberately have no access/refresh/token fields.
// Serde skips unknown values without retaining them in a generic JSON document.
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CredentialInfo {
    account_id: Option<String>,
    email: Option<String>,
    kiro: Option<KiroInfo>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct KiroInfo {
    auth_type: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OAuthAccount {
    id: String,
    alias: Option<String>,
    #[serde(default)]
    credential: CredentialInfo,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OAuthSource {
    accounts: Option<Vec<OAuthAccount>>,
    account_id: Option<String>,
    email: Option<String>,
    access: Option<serde::de::IgnoredAny>,
    refresh: Option<serde::de::IgnoredAny>,
    expires: Option<u64>,
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexConfig {
    #[serde(default)]
    codex_accounts: Vec<CodexInfo>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexInfo {
    id: String,
    email: Option<String>,
    alias: Option<String>,
    chatgpt_account_id: Option<String>,
}
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexRecord {
    deleted_at: Option<u64>,
    credential: Option<CodexCredentialIdentity>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CodexCredentialIdentity {
    source_subject: Option<String>,
}
struct SourceInfo {
    provider: String,
    source_id: String,
    subject: Option<String>,
    label: String,
    email: Option<String>,
    auth_kind: String,
    tenant: Option<String>,
    legacy_subject: Option<String>,
}
fn source_infos(upstream: &Path) -> ModelResult<Vec<SourceInfo>> {
    let mut infos = vec![];
    if let Some(bytes) = read(upstream, "auth.json")? {
        let values: BTreeMap<String, OAuthSource> = serde_json::from_slice(&bytes)
            .map_err(|_| invalid("account_source", "官方OAuth账户结构无法识别，未导出"))?;
        for (provider, value) in values {
            // ChatGPT's scratch slot is not the authoritative Codex pool.
            if provider == "chatgpt" {
                continue;
            }
            let accounts = if let Some(accounts) = value.accounts {
                accounts
            } else {
                if value.access.is_none() || value.refresh.is_none() || value.expires.is_none() {
                    return Err(invalid(
                        "account_source",
                        "官方OAuth账户结构无法识别，未导出",
                    ));
                }
                vec![OAuthAccount {
                    id: "legacy".into(),
                    alias: None,
                    credential: CredentialInfo {
                        account_id: value.account_id,
                        email: value.email,
                        kiro: None,
                    },
                }]
            };
            for account in accounts {
                let subject = account.credential.account_id;
                let legacy_subject = subject.clone();
                let auth_kind = if provider == "kiro" {
                    account
                        .credential
                        .kiro
                        .and_then(|k| k.auth_type)
                        .unwrap_or_else(|| "unknown".into())
                } else {
                    "oauth".into()
                };
                let email = account.credential.email;
                let label = account
                    .alias
                    .or_else(|| email.clone())
                    .unwrap_or_else(|| provider.clone());
                infos.push(SourceInfo {
                    provider: provider.clone(),
                    source_id: account.id.clone(),
                    subject,
                    label,
                    email,
                    auth_kind,
                    tenant: None,
                    legacy_subject,
                });
            }
        }
    }
    if let Some(bytes) = read(upstream, "config.json")? {
        let config: CodexConfig = serde_json::from_slice(&bytes)
            .map_err(|_| invalid("account_source", "官方账户配置无法识别，未导出"))?;
        let records: BTreeMap<String, CodexRecord> = read(upstream, "codex-accounts.json")?
            .map(|b| serde_json::from_slice(&b))
            .transpose()
            .map_err(|_| invalid("account_source", "官方账户撤销记录无法识别，未导出"))?
            .unwrap_or_default();
        for account in config.codex_accounts {
            if records
                .get(&account.id)
                .is_some_and(|r| r.deleted_at.is_some())
            {
                continue;
            }
            infos.push(SourceInfo {
                provider: "chatgpt".into(),
                source_id: account.id.clone(),
                subject: records
                    .get(&account.id)
                    .and_then(|r| r.credential.as_ref())
                    .and_then(|c| c.source_subject.clone()),
                tenant: account.chatgpt_account_id.clone(),
                legacy_subject: account.chatgpt_account_id,
                label: account
                    .alias
                    .or_else(|| account.email.clone())
                    .unwrap_or_else(|| "ChatGPT".into()),
                email: account.email,
                auth_kind: "oauth".into(),
            });
        }
    }
    if infos.len() > MAX_ACCOUNTS {
        return Err(invalid("account_source", "官方账户数量超过上限"));
    }
    Ok(infos)
}
pub fn catalog(root: &Path, upstream: &Path) -> ModelResult<Vec<CatalogEntry>> {
    if !upstream.is_absolute() {
        return Err(invalid("account_source", "官方账户数据根未绑定"));
    }
    let sources = source_infos(upstream)?;
    let path = root.join(IDS);
    safety::check_path(root, &path, true)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&path),
        true,
    )?;
    let _guard = TargetFileLock::lock(&path)?;
    let mut ids: BTreeMap<String, String> = read(root, IDS)?
        .map(|bytes| serde_json::from_slice(&bytes))
        .transpose()
        .map_err(|_| invalid("account_registry", "账户标识登记无法解析"))?
        .unwrap_or_default();
    if ids.len() > 20000 || ids.values().any(|id| require_uuid(id).is_err()) {
        return Err(invalid("account_registry", "账户标识登记无效"));
    }
    let before = ids.clone();
    let imported = imported(root)?;
    let mut local = vec![];
    for source in sources {
        if !text_ok(&source.source_id, 512) {
            return Err(invalid("account_source", "官方账户标识无效"));
        }
        let legacy_identity = sha256(&serde_json::to_vec(&(
            &source.provider,
            if source.legacy_subject.is_some() {
                "subject"
            } else {
                "source"
            },
            source.legacy_subject.as_ref().unwrap_or(&source.source_id),
            if source.legacy_subject.is_none() {
                Some((upstream, &source.email))
            } else {
                None
            },
        ))?);
        let origin = AccountOrigin {
            version: 1,
            account_platform: platform(&source.provider).unwrap_or("unknown").into(),
            owner: "opencodex".into(),
            auth_kind: source.auth_kind,
            source_namespace: if source.provider == "chatgpt" {
                "codex-account-pool"
            } else {
                "oauth-account-pool"
            }
            .into(),
            source_account_id: source.source_id.clone(),
            tenant: source.tenant,
        };
        let identity = sha256(&serde_json::to_vec(&(
            "account-origin-v1",
            &source.provider,
            &origin.account_platform,
            &origin.owner,
            &origin.auth_kind,
            &origin.source_namespace,
            &origin.tenant,
            &source.subject,
            if source.subject.is_none() {
                Some((upstream, &source.source_id))
            } else {
                None
            },
        ))?);
        let mut account = AccountInfo {
            id: String::new(),
            provider: source.provider,
            subject: source.subject,
            label: source.label,
            email: source.email,
            origin: Some(origin),
        };
        let old_id = ids
            .get(&legacy_identity)
            .filter(|id| {
                !local.iter().any(|a: &AccountInfo| &a.id == *id)
                    && !imported.iter().any(|a| {
                        &a.id == *id
                            && (a.provider != account.provider || a.subject != account.subject)
                    })
            })
            .cloned();
        let id = ids
            .entry(identity)
            .or_insert_with(|| {
                imported
                    .iter()
                    .find(|a| same_subject(a, &account))
                    .map(|a| a.id.clone())
                    .or(old_id)
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
            })
            .clone();
        account.id = id;
        if let Some(previous) = local.iter().find(|a: &&AccountInfo| a.id == account.id) {
            if previous != &account {
                return Err(invalid(
                    "account_identity_conflict",
                    "官方账户存在重复身份，请先处理",
                ));
            }
        } else {
            local.push(account);
        }
    }
    validate(&local)?;
    let supported: Vec<_> = imported
        .iter()
        .filter(|a| information_supported(a))
        .cloned()
        .collect();
    let mut all = merge(local.clone(), &supported)?;
    // Keep old, unavailable metadata visible but never make it selectable by a preset.
    all.extend(
        imported
            .iter()
            .filter(|a| !information_supported(a))
            .cloned(),
    );
    validate(&all)?;
    if ids.len() > 20000 {
        return Err(invalid("account_registry", "账户标识登记超过上限"));
    }
    if ids != before {
        atomic_write(&path, &serde_json::to_vec(&ids)?, 0o600)?;
    }
    Ok(all
        .into_iter()
        .map(|account| CatalogEntry {
            login_transfer: login_transfer(&account.provider),
            local_source: local.iter().any(|a| a.id == account.id),
            information_supported: information_supported(&account),
            account,
        })
        .collect())
}
pub fn export(
    root: &Path,
    upstream: Option<&Path>,
    selected: &[String],
) -> ModelResult<Vec<AccountInfo>> {
    if selected.is_empty() {
        return Ok(vec![]);
    }
    if selected.len() > MAX_ACCOUNTS
        || selected.iter().collect::<BTreeSet<_>>().len() != selected.len()
    {
        return Err(invalid("account_scope", "所选账户重复或超过上限"));
    }
    let upstream = upstream.ok_or_else(|| invalid("account_source", "官方账户数据根未绑定"))?;
    let accounts = catalog(root, upstream)?;
    selected
        .iter()
        .map(|id| {
            accounts
                .iter()
                .find(|entry| &entry.account.id == id)
                .filter(|entry| entry.information_supported)
                .map(|entry| entry.account.clone())
                .ok_or_else(|| invalid("account_scope", "所选账户已不存在，请重新选择"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn info(provider: &str, subject: Option<&str>) -> AccountInfo {
        AccountInfo {
            id: uuid::Uuid::new_v4().to_string(),
            provider: provider.into(),
            subject: subject.map(str::to_string),
            label: "测试账户".into(),
            email: Some("fixture@example.test".into()),
            origin: None,
        }
    }
    fn origin(platform: &str, namespace: &str, slot: &str) -> AccountOrigin {
        AccountOrigin {
            version: 1,
            account_platform: platform.into(),
            owner: "opencodex".into(),
            auth_kind: "oauth".into(),
            source_namespace: namespace.into(),
            source_account_id: slot.into(),
            tenant: None,
        }
    }
    #[test]
    fn chatgpt_shared_tenant_and_email_do_not_collapse_two_source_accounts() {
        let root = tempfile::tempdir().unwrap();
        let source = tempfile::tempdir().unwrap();
        write(
            source.path(),
            "config.json",
            json!({"codexAccounts":[
            {"id":"personal","email":"same@example.test","chatgptAccountId":"shared-tenant"},
            {"id":"work","email":"same@example.test","chatgptAccountId":"shared-tenant"}]}),
        );
        let rows = catalog(root.path(), source.path()).unwrap();
        assert_eq!(rows.len(), 2);
        assert_ne!(rows[0].account.id, rows[1].account.id);
        assert!(rows
            .iter()
            .all(|row| row.account.subject.is_none() && row.information_supported));
        let chosen = export(
            root.path(),
            Some(source.path()),
            &[rows[1].account.id.clone()],
        )
        .unwrap();
        assert_eq!(chosen.len(), 1);
        assert_eq!(chosen[0].origin.as_ref().unwrap().source_account_id, "work");
        let again = catalog(root.path(), source.path()).unwrap();
        assert_eq!(rows[0].account.id, again[0].account.id);
        assert_eq!(rows[1].account.id, again[1].account.id);
        write(
            source.path(),
            "codex-accounts.json",
            json!({"personal":{"credential":{"sourceSubject":"subject-p","refreshToken":"NEVER-EXPORT"}},"work":{"credential":{"sourceSubject":"subject-w"}}}),
        );
        let identified = catalog(root.path(), source.path()).unwrap();
        assert_eq!(identified[0].account.subject.as_deref(), Some("subject-p"));
        assert_eq!(identified[1].account.subject.as_deref(), Some("subject-w"));
        assert!(!serde_json::to_string(&identified)
            .unwrap()
            .contains("NEVER-EXPORT"));
    }
    #[test]
    fn origin_tenant_and_capability_bind_merge_and_old_records_remain_unclassified() {
        let legacy = info("chatgpt", Some("subject"));
        let encoded = serde_json::to_value(&legacy).unwrap();
        assert!(encoded.get("origin").is_none());
        let mut a = legacy.clone();
        a.id = uuid::Uuid::new_v4().to_string();
        a.origin = Some(origin("chatgpt", "codex-account-pool", "slot"));
        assert_eq!(merge(vec![legacy.clone()], &[a.clone()]).unwrap().len(), 2);
        let mut other = a.clone();
        other.id = uuid::Uuid::new_v4().to_string();
        other.origin.as_mut().unwrap().tenant = Some("another-tenant".into());
        assert_eq!(merge(vec![a.clone()], &[other.clone()]).unwrap().len(), 2);
        assert!(!same_identity(&a, &other));
        other.origin.as_mut().unwrap().account_platform = "kiro".into();
        assert!(!information_supported(&other));
        assert_eq!(
            merge(vec![a.clone()], &[other]).unwrap_err().code,
            "account_capability"
        );
        let mut unsupported = a;
        unsupported.origin.as_mut().unwrap().owner = "platform-native".into();
        assert_eq!(
            merge(vec![], &[unsupported]).unwrap_err().code,
            "account_capability"
        );
        let mut old_other = legacy.clone();
        old_other.id = uuid::Uuid::new_v4().to_string();
        assert!(!same_identity(&legacy, &old_other));
    }
    fn write(root: &Path, name: &str, value: serde_json::Value) {
        std::fs::create_dir_all(root).unwrap();
        std::fs::write(root.join(name), serde_json::to_vec(&value).unwrap()).unwrap();
    }
    #[test]
    fn metadata_is_stable_across_refresh_and_never_exports_tokens_or_linked_paths() {
        let root = tempfile::tempdir().unwrap();
        let source = tempfile::tempdir().unwrap();
        let mut value = json!({"google-antigravity":{"activeAccountId":"slot", "accounts":[{
            "id":"slot", "alias":"测试", "credential":{"accountId":"google-sub", "email":"fixture@example.test",
                "access":"fixture-access-SECRET", "refresh":"fixture-refresh-SECRET", "expires":1,
                "sourceAuthPath":"/private/never-open", "unknownSecret":"unknown-SECRET"}}]}});
        write(source.path(), "auth.json", value.clone());
        let first = catalog(root.path(), source.path()).unwrap();
        value["google-antigravity"]["accounts"][0]["credential"]["refresh"] =
            json!("rotated-SECRET");
        write(source.path(), "auth.json", value);
        let second = catalog(root.path(), source.path()).unwrap();
        assert_eq!(first[0].account, second[0].account);
        let payload = export(
            root.path(),
            Some(source.path()),
            &[first[0].account.id.clone()],
        )
        .unwrap();
        let encoded = serde_json::to_string(&payload).unwrap();
        assert!(
            !encoded.contains("SECRET")
                && !encoded.contains("private")
                && !encoded.contains("refresh")
        );
        assert_eq!(
            first[0].login_transfer,
            LoginTransfer::CrossDeviceEvidenceMissing
        );
        let registry = std::fs::read_to_string(root.path().join(IDS)).unwrap();
        assert!(!registry.contains("fixture@example.test") && !registry.contains("SECRET"));
    }
    #[test]
    fn scratch_chatgpt_and_deleted_pool_accounts_are_excluded_without_refresh() {
        let root = tempfile::tempdir().unwrap();
        let source = tempfile::tempdir().unwrap();
        write(
            source.path(),
            "auth.json",
            json!({"chatgpt":{"accountId":"scratch", "access":"SECRET"}}),
        );
        write(
            source.path(),
            "config.json",
            json!({"codexAccounts":[
            {"id":"a","email":"a@example.test","chatgptAccountId":"subject-a"},
            {"id":"b","email":"b@example.test","chatgptAccountId":"subject-b"}]}),
        );
        write(
            source.path(),
            "codex-accounts.json",
            json!({"a":{"generation":2,"credential":{"refreshToken":"SECRET"}},"b":{"deletedAt":10}}),
        );
        let found = catalog(root.path(), source.path()).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].account.subject, None);
        assert_eq!(
            found[0].account.origin.as_ref().unwrap().tenant.as_deref(),
            Some("subject-a")
        );
        assert_eq!(found[0].login_transfer, LoginTransfer::ManualImportDisabled);
    }
    #[test]
    fn email_or_alias_never_merges_different_or_unidentified_subjects() {
        let a = info("chatgpt", Some("a"));
        let mut b = info("chatgpt", Some("b"));
        let local = merge(vec![a.clone()], &[b.clone()]).unwrap();
        assert_eq!(local.len(), 2);
        b.id = a.id.clone();
        assert_eq!(
            merge(local, &[b]).unwrap_err().code,
            "account_identity_conflict"
        );
        let a = info("kiro", None);
        let b = info("kiro", None);
        assert_eq!(merge(vec![a], &[b]).unwrap().len(), 2);
    }
    #[test]
    fn same_subject_keeps_local_label_and_ids_and_never_writes_official_stores() {
        let root = tempfile::tempdir().unwrap();
        let source = tempfile::tempdir().unwrap();
        let mut local = info("google-antigravity", Some("subject"));
        local.origin = Some(origin("antigravity", "oauth-account-pool", "old-slot"));
        std::fs::create_dir_all(root.path().join("manager-state")).unwrap();
        write(root.path(), IMPORTED, json!([local]));
        let value = json!({"google-antigravity":{"accounts":[{"id":"upstream","alias":"本机官方别名","credential":{"accountId":"subject"}}]}});
        write(source.path(), "auth.json", value.clone());
        let result = catalog(root.path(), source.path()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].account.id, local.id);
        assert_eq!(result[0].account.label, "本机官方别名");
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(
                &std::fs::read(source.path().join("auth.json")).unwrap()
            )
            .unwrap(),
            value
        );
        assert_eq!(imported(root.path()).unwrap(), vec![local]);
    }
    #[test]
    fn malformed_multi_shape_and_unknown_incoming_grants_are_rejected() {
        let root = tempfile::tempdir().unwrap();
        let source = tempfile::tempdir().unwrap();
        write(
            source.path(),
            "auth.json",
            json!({"google-antigravity":{"accounts":"bad"}}),
        );
        assert_eq!(
            catalog(root.path(), source.path()).unwrap_err().code,
            "account_source"
        );
        let mut value = serde_json::to_value(info("chatgpt", Some("subject"))).unwrap();
        value["refreshToken"] = json!("SECRET");
        assert!(serde_json::from_value::<AccountInfo>(value).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn source_symlinks_are_not_followed() {
        let root = tempfile::tempdir().unwrap();
        let source = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        write(outside.path(), "auth.json", json!({}));
        std::os::unix::fs::symlink(
            outside.path().join("auth.json"),
            source.path().join("auth.json"),
        )
        .unwrap();
        assert!(catalog(root.path(), source.path()).is_err());
    }
}
