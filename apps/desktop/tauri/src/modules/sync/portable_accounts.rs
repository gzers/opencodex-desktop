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
            || account
                .subject
                .as_ref()
                .is_some_and(|s| !identities.insert((&account.provider, s)))
        {
            return Err(invalid("account_info", "账户身份重复或显示信息无效"));
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
        if let Some(existing) = local.iter().find(|a| a.id == account.id) {
            if existing.provider != account.provider || existing.subject != account.subject {
                return Err(invalid(
                    "account_identity_conflict",
                    "相同账户标识指向不同身份，未覆盖",
                ));
            }
        } else if !local.iter().any(|a| {
            a.provider == account.provider && a.subject.is_some() && a.subject == account.subject
        }) {
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
}
struct SourceInfo {
    provider: String,
    source_id: String,
    subject: Option<String>,
    label: String,
    email: Option<String>,
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
                    },
                }]
            };
            for account in accounts {
                let subject = account.credential.account_id;
                let email = account.credential.email;
                let label = account
                    .alias
                    .or_else(|| email.clone())
                    .unwrap_or_else(|| provider.clone());
                infos.push(SourceInfo {
                    provider: provider.clone(),
                    source_id: account.id,
                    subject,
                    label,
                    email,
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
                source_id: account.id,
                subject: account.chatgpt_account_id,
                label: account
                    .alias
                    .or_else(|| account.email.clone())
                    .unwrap_or_else(|| "ChatGPT".into()),
                email: account.email,
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
        let identity = sha256(&serde_json::to_vec(&(
            &source.provider,
            if source.subject.is_some() {
                "subject"
            } else {
                "source"
            },
            source.subject.as_ref().unwrap_or(&source.source_id),
            if source.subject.is_none() {
                Some((upstream, &source.email))
            } else {
                None
            },
        ))?);
        let id = ids
            .entry(identity)
            .or_insert_with(|| {
                imported
                    .iter()
                    .find(|a| {
                        a.provider == source.provider
                            && source.subject.is_some()
                            && a.subject == source.subject
                    })
                    .map(|a| a.id.clone())
                    .unwrap_or_else(|| uuid::Uuid::new_v4().to_string())
            })
            .clone();
        let account = AccountInfo {
            id,
            provider: source.provider,
            subject: source.subject,
            label: source.label,
            email: source.email,
        };
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
    let all = merge(local.clone(), &imported)?;
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
        }
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
        assert_eq!(found[0].account.subject.as_deref(), Some("subject-a"));
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
        let local = info("google-antigravity", Some("subject"));
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
