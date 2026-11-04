//! Recoverable local provider rename. Registered under config_file so shared
//! model_config::mod.rs needs no parallel edit. Secret values never enter the
//! journal; exact file images use the existing protected backup directory.
use crate::{
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::{
        backup::{manager::acquire_preferences_transaction, safety},
        model_config::{
            agent, config_file_mapping::ProviderRename, protocol_groups, provider_auth,
            provider_fields, schema::*, store::sha256,
        },
        preferences,
    },
    state::ProcessContext,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;

const PENDING: &str = "manager-state/provider-rename-pending.json";
const IDS: &str = "manager-state/model-execution-source-ids.json";
const LAYOUT: &str = "manager-state/model-channel-layout.json";
const SERVICE: &str = "opencodex.provider-api-key.v1";
const LIMIT: u64 = 16 * 1024 * 1024;
fn fail(code: &str) -> ModelError {
    invalid(
        code,
        match code {
            "rename_cleanup_pending" => {
                "改名已写入，安全凭据清理尚未完成；请刷新重试，当前事务已保留"
            }
            "rename_recovery_required" => {
                "改名事务尚未完整恢复，未覆盖新的外部修改；请保留事务保护文件并重试"
            }
            "rename_credential_changed" => "渠道安全凭据在改名期间改变，未覆盖，请重新核对",
            _ => "渠道改名校验或写入失败，原配置和当前编辑已保留",
        },
    )
}
fn optional(root: &Path, path: &Path) -> ModelResult<Option<Zeroizing<Vec<u8>>>> {
    safety::check_path(root, path, true)?;
    match std::fs::symlink_metadata(path) {
        Ok(_) => Ok(Some(Zeroizing::new(safety::read_file(path, LIMIT)?))),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}
fn lock(root: &Path, path: &Path) -> ModelResult<TargetFileLock> {
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(path),
        true,
    )?;
    Ok(TargetFileLock::lock(path)?)
}
fn account(raw: &str) -> ModelResult<Option<Zeroizing<String>>> {
    match keyring::Entry::new(SERVICE, raw)
        .map_err(|_| fail("rename_credential"))?
        .get_password()
    {
        Ok(value) if !value.is_empty() => Ok(Some(Zeroizing::new(value))),
        Err(keyring::Error::NoEntry) => Ok(None),
        _ => Err(fail("rename_credential")),
    }
}
fn account_digest(raw: &str) -> ModelResult<Option<String>> {
    Ok(account(raw)?.map(|s| sha256(s.as_bytes())))
}
fn remove_account(raw: &str, expected: &str) -> ModelResult<()> {
    if let Some(hash) = account_digest(raw)? {
        if hash != expected {
            return Err(fail("rename_credential_changed"));
        }
        keyring::Entry::new(SERVICE, raw)
            .map_err(|_| fail("rename_credential"))?
            .delete_credential()
            .map_err(|_| fail("rename_credential"))?;
        if account_digest(raw)?.is_some() {
            return Err(fail("rename_credential"));
        }
    }
    Ok(())
}
fn references(value: &Value, out: &mut BTreeSet<String>) {
    match value {
        Value::String(raw) if raw.starts_with("keychain:") => {
            out.insert(raw[9..].into());
        }
        Value::Array(values) => {
            for value in values {
                references(value, out)
            }
        }
        Value::Object(values) => {
            for value in values.values() {
                references(value, out)
            }
        }
        _ => (),
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CredentialMove {
    from: String,
    to: Option<String>,
    digest: String,
}
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Target {
    Config,
    Layout,
    Preferences,
    SourceIds,
    ClientEffects,
}
impl Target {
    fn path(self, root: &Path, home: &Path) -> (PathBuf, PathBuf) {
        match self {
            Self::Config => (home.to_owned(), home.join("config.json")),
            Self::Layout => (root.to_owned(), root.join(LAYOUT)),
            Self::Preferences => (
                root.to_owned(),
                root.join(preferences::PREFERENCES_RELATIVE_PATH),
            ),
            Self::SourceIds => (root.to_owned(), root.join(IDS)),
            Self::ClientEffects => (root.to_owned(), root.join(super::rename_effects::PATH)),
        }
    }
}
struct Image {
    target: Target,
    before: Option<Zeroizing<Vec<u8>>>,
    after: Zeroizing<Vec<u8>>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileRecord {
    target: Target,
    before: Option<String>,
    after: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    version: u32,
    id: String,
    home_digest: String,
    committed: bool,
    files: Vec<FileRecord>,
    credentials: Vec<CredentialMove>,
}
fn home_digest(home: &Path) -> ModelResult<String> {
    Ok(sha256(&serde_json::to_vec(home)?))
}
fn backup_dir(root: &Path, id: &str) -> ModelResult<PathBuf> {
    require_uuid(id)?;
    let path = root
        .join("backups/model-config")
        .join(format!("rename-{id}"));
    safety::check_path(root, &path, true)?;
    Ok(path)
}
fn persist(root: &Path, journal: &Journal) -> ModelResult<()> {
    let path = root.join(PENDING);
    safety::check_path(root, &path, true)?;
    atomic_write(&path, &serde_json::to_vec(journal)?, 0o600)?;
    Ok(())
}
fn bytes_for(
    root: &Path,
    journal: &Journal,
    index: usize,
    after: bool,
    digest: &str,
) -> ModelResult<Zeroizing<Vec<u8>>> {
    let path = backup_dir(root, &journal.id)?.join(format!(
        "{index}.{}",
        if after { "after" } else { "before" }
    ));
    let bytes = optional(root, &path)?.ok_or_else(|| fail("rename_recovery_required"))?;
    if sha256(&bytes) != digest {
        return Err(fail("rename_recovery_required"));
    }
    Ok(bytes)
}
fn current_digest(root: &Path, home: &Path, target: Target) -> ModelResult<Option<String>> {
    let (anchor, path) = target.path(root, home);
    Ok(optional(&anchor, &path)?.map(|b| sha256(&b)))
}

/// Serializes ordinary file readers/writers with rename recovery. A restart
/// must finish/compensate a pending transaction before returning any snapshot.
pub(super) struct Admission {
    _lock: TargetFileLock,
}
pub(super) fn admit(root: &Path, home: &Path) -> ModelResult<Admission> {
    let guard = Admission {
        _lock: lock(root, &root.join(PENDING))?,
    };
    if optional(root, &root.join(PENDING))?.is_some() {
        let _raw = lock(home, &home.join("config.json"))?;
        let _layout = lock(root, &root.join(LAYOUT))?;
        let _preferences = acquire_preferences_transaction(root)?;
        let _ids = lock(root, &root.join(IDS))?;
        let _groups = lock(root, &protocol_groups::rename_guard_path(root))?;
        let _client_effects = lock(root, &root.join(super::rename_effects::PATH))?;
        recover_locked(root, home).map_err(|e| {
            if e.code == "rename_cleanup_pending" {
                e
            } else {
                fail("rename_recovery_required")
            }
        })?;
    }
    Ok(guard)
}

fn recover_locked(root: &Path, home: &Path) -> ModelResult<()> {
    let Some(bytes) = optional(root, &root.join(PENDING))? else {
        return Ok(());
    };
    let journal: Journal =
        serde_json::from_slice(&bytes).map_err(|_| fail("rename_recovery_required"))?;
    if journal.version != 1
        || journal.home_digest != home_digest(home)?
        || journal.files.len() > 5
        || journal.credentials.len() > 1024
    {
        return Err(fail("rename_recovery_required"));
    }
    require_uuid(&journal.id)?;
    if journal.committed {
        for file in &journal.files {
            if current_digest(root, home, file.target)?.as_deref() != Some(file.after.as_str()) {
                return Err(fail("rename_recovery_required"));
            }
        }
        for credential in &journal.credentials {
            if let Some(to) = &credential.to {
                if account_digest(to)
                    .map_err(|_| fail("rename_cleanup_pending"))?
                    .as_deref()
                    != Some(credential.digest.as_str())
                {
                    return Err(fail("rename_cleanup_pending"));
                }
            }
            remove_account(&credential.from, &credential.digest)
                .map_err(|_| fail("rename_cleanup_pending"))?;
        }
    } else {
        let mut failed = false;
        for (index, file) in journal.files.iter().enumerate().rev() {
            let actual = current_digest(root, home, file.target)?;
            if actual == file.before {
                continue;
            }
            if actual.as_deref() != Some(file.after.as_str()) {
                failed = true;
                continue;
            }
            let (anchor, path) = file.target.path(root, home);
            safety::check_path(&anchor, &path, false)?;
            let restored = if let Some(hash) = &file.before {
                let before = bytes_for(root, &journal, index, false, hash)?;
                atomic_write(&path, &before, 0o600).map_err(ModelError::from)
            } else {
                std::fs::remove_file(&path).map_err(ModelError::from)
            };
            if restored.is_err() || current_digest(root, home, file.target)? != file.before {
                failed = true;
            }
        }
        if failed {
            return Err(fail("rename_recovery_required"));
        }
        for credential in &journal.credentials {
            if account_digest(&credential.from)?.as_deref() != Some(credential.digest.as_str()) {
                return Err(fail("rename_recovery_required"));
            }
            if let Some(to) = &credential.to {
                remove_account(to, &credential.digest)
                    .map_err(|_| fail("rename_recovery_required"))?;
            }
        }
    }
    safety::check_path(root, &root.join(PENDING), false)?;
    std::fs::remove_file(root.join(PENDING))?;
    Ok(())
}

/// Locks remain held through prepare, callback, commit, snapshot readback and
/// compensation. Ordinary imports/saves share the same admission above.
pub(super) struct Transaction<'a> {
    root: &'a Path,
    home: &'a Path,
    images: Vec<Image>,
    guards: Vec<(PathBuf, Option<Zeroizing<Vec<u8>>>)>,
    credentials: Vec<CredentialMove>,
    _preferences: crate::modules::backup::manager::PreferencesProtectionGuard,
    _ids: TargetFileLock,
    _groups: TargetFileLock,
    _client_effects: TargetFileLock,
}
impl<'a> Transaction<'a> {
    pub(super) fn prepare(
        root: &'a Path,
        home: &'a Path,
        original: &Value,
        config: &Value,
        renames: &[ProviderRename],
        context: Option<&ProcessContext>,
    ) -> ModelResult<Self> {
        let _preferences = acquire_preferences_transaction(root)?;
        let _ids = lock(root, &root.join(IDS))?;
        let group_path = protocol_groups::rename_guard_path(root);
        let _groups = lock(root, &group_path)?;
        let client_effects_path = root.join(super::rename_effects::PATH);
        let _client_effects = lock(root, &client_effects_path)?;
        let client_effects_before = optional(root, &client_effects_path)?;
        let client_effects_after = super::rename_effects::next(
            home,
            client_effects_before.as_deref().map(|b| b.as_slice()),
            renames,
        )?;
        let groups = optional(root, &group_path)?;
        protocol_groups::validate_rename_targets(groups.as_deref().map(|b| b.as_slice()), renames)?;
        let mut tx = Self {
            root,
            home,
            images: vec![],
            guards: vec![(group_path, groups)],
            credentials: vec![],
            _preferences,
            _ids,
            _groups,
            _client_effects,
        };
        tx.images.push(Image {
            target: Target::ClientEffects,
            before: client_effects_before,
            after: Zeroizing::new(client_effects_after),
        });
        let prefs_path = root.join(preferences::PREFERENCES_RELATIVE_PATH);
        if let Some(bytes) = optional(root, &prefs_path)? {
            let mut doc: Value = serde_json::from_slice(&bytes)?;
            let before = doc.clone();
            agent::rename_settings_document(&mut doc, renames, config, context)?;
            if doc != before {
                tx.images.push(Image {
                    target: Target::Preferences,
                    before: Some(bytes),
                    after: Zeroizing::new(serde_json::to_vec_pretty(&doc)?),
                });
            } else {
                tx.guards.push((prefs_path, Some(bytes)));
            }
        } else {
            tx.guards.push((prefs_path, None));
        }
        let ids_path = root.join(IDS);
        if let Some(bytes) = optional(root, &ids_path)? {
            let mut doc: BTreeMap<String, String> = serde_json::from_slice(&bytes)?;
            let before = doc.clone();
            for rename in renames {
                if let Some(id) = doc.remove(&rename.from) {
                    if id != rename.channel_id || doc.contains_key(&rename.to) {
                        return Err(invalid("execution_identity", "执行身份登记冲突，未覆盖"));
                    }
                    doc.insert(rename.to.clone(), id);
                }
            }
            if doc != before {
                tx.images.push(Image {
                    target: Target::SourceIds,
                    before: Some(bytes),
                    after: Zeroizing::new(serde_json::to_vec(&doc)?),
                });
            } else {
                tx.guards.push((ids_path, Some(bytes)));
            }
        } else {
            tx.guards.push((ids_path, None));
        }
        let mut final_refs = BTreeSet::new();
        references(config, &mut final_refs);
        let mut moves = BTreeMap::new();
        for rename in renames {
            let raw = &original["providers"][&rename.from];
            if let Some(pool) = raw["apiKeyPool"].as_array() {
                for slot in pool {
                    if let Some(key) = slot["key"].as_str().filter(|s| s.starts_with("keychain:")) {
                        if key
                            != format!(
                                "keychain:{}/{}",
                                rename.from,
                                slot["id"].as_str().unwrap_or("")
                            )
                        {
                            return Err(invalid(
                                "provider_auth_reference",
                                "安全引用与原 Key 池身份不对应，未迁移",
                            ));
                        }
                    }
                }
            }
            let mut source_refs = BTreeSet::new();
            for field in provider_fields::PRIVATE {
                if let Some(value) = raw.get(*field) {
                    references(value, &mut source_refs);
                }
            }
            for source in source_refs {
                provider_auth::owned_account(&format!("keychain:{source}"), &rename.from)?;
                if final_refs.contains(&source) {
                    return Err(invalid(
                        "provider_auth_reference",
                        "其它当前配置仍引用此渠道的安全账户，未迁移",
                    ));
                }
                let destination = format!("{}{}", rename.to, &source[rename.from.len()..]);
                let to = final_refs.contains(&destination).then_some(destination);
                let digest = account_digest(&source)?.ok_or_else(|| fail("rename_credential"))?;
                if to
                    .as_ref()
                    .map(|to| account_digest(to))
                    .transpose()?
                    .flatten()
                    .is_some()
                {
                    return Err(invalid(
                        "provider_auth_reference",
                        "目标安全凭据账户已存在，未覆盖",
                    ));
                }
                moves.insert(
                    source.clone(),
                    CredentialMove {
                        from: source,
                        to,
                        digest,
                    },
                );
            }
        }
        tx.credentials = moves.into_values().collect();
        Ok(tx)
    }
    pub(super) fn files(
        &mut self,
        raw_before: &[u8],
        raw_after: &[u8],
        layout_after: &[u8],
    ) -> ModelResult<()> {
        let layout_before = optional(self.root, &self.root.join(LAYOUT))?;
        self.images.insert(
            0,
            Image {
                target: Target::Config,
                before: Some(Zeroizing::new(raw_before.to_vec())),
                after: Zeroizing::new(raw_after.to_vec()),
            },
        );
        self.images.insert(
            1,
            Image {
                target: Target::Layout,
                before: layout_before,
                after: Zeroizing::new(layout_after.to_vec()),
            },
        );
        Ok(())
    }
    fn recheck(&self) -> ModelResult<()> {
        for image in &self.images {
            let (root, path) = image.target.path(self.root, self.home);
            if optional(&root, &path)?.as_deref() != image.before.as_deref() {
                return Err(invalid(
                    "file_changed",
                    "改名确认后配置或执行设置改变，请重新核对",
                ));
            }
        }
        for (path, before) in &self.guards {
            if optional(self.root, path)?.as_deref() != before.as_deref() {
                return Err(invalid(
                    "file_changed",
                    "改名确认后渠道引用改变，请重新核对",
                ));
            }
        }
        for credential in &self.credentials {
            if account_digest(&credential.from)?.as_deref() != Some(credential.digest.as_str()) {
                return Err(fail("rename_credential_changed"));
            }
            if let Some(to) = &credential.to {
                if account_digest(to)?.is_some() {
                    return Err(fail("rename_credential_changed"));
                }
            }
        }
        Ok(())
    }
    pub(super) fn commit<T>(&self, readback: impl FnOnce() -> ModelResult<T>) -> ModelResult<T> {
        self.recheck()?;
        if optional(self.root, &self.root.join(PENDING))?.is_some() {
            return Err(fail("rename_recovery_required"));
        }
        let mut journal = Journal {
            version: 1,
            id: uuid::Uuid::new_v4().to_string(),
            home_digest: home_digest(self.home)?,
            committed: false,
            files: vec![],
            credentials: self.credentials.clone(),
        };
        let dir = backup_dir(self.root, &journal.id)?;
        std::fs::create_dir_all(&dir)?;
        crate::infrastructure::platform::set_mode(&dir, 0o700)?;
        for (index, image) in self.images.iter().enumerate() {
            let before = if let Some(bytes) = &image.before {
                atomic_write(&dir.join(format!("{index}.before")), bytes, 0o600)?;
                Some(sha256(bytes))
            } else {
                None
            };
            atomic_write(&dir.join(format!("{index}.after")), &image.after, 0o600)?;
            journal.files.push(FileRecord {
                target: image.target,
                before,
                after: sha256(&image.after),
            });
        }
        persist(self.root, &journal)?;
        let result = (|| {
            checkpoint("prepared")?;
            for (index, credential) in self.credentials.iter().enumerate() {
                if let Some(to) = &credential.to {
                    let value =
                        account(&credential.from)?.ok_or_else(|| fail("rename_credential"))?;
                    if sha256(value.as_bytes()) != credential.digest || account(to)?.is_some() {
                        return Err(fail("rename_credential_changed"));
                    }
                    keyring::Entry::new(SERVICE, to)
                        .map_err(|_| fail("rename_credential"))?
                        .set_password(&value)
                        .map_err(|_| fail("rename_credential"))?;
                    if account_digest(to)?.as_deref() != Some(credential.digest.as_str()) {
                        return Err(fail("rename_credential"));
                    }
                    checkpoint(&format!("credential:{index}"))?;
                }
            }
            for (index, image) in self.images.iter().enumerate() {
                let (root, path) = image.target.path(self.root, self.home);
                if optional(&root, &path)?.as_deref() != image.before.as_deref() {
                    return Err(invalid("file_changed", "改名写入期间配置改变，未覆盖"));
                }
                safety::check_path(&root, &path, true)?;
                checkpoint(&format!("before_file:{index}"))?;
                atomic_write(&path, &image.after, 0o600)?;
                checkpoint(&format!("file:{index}"))?;
            }
            for (index, image) in self.images.iter().enumerate() {
                if current_digest(self.root, self.home, image.target)?.as_deref()
                    != Some(journal.files[index].after.as_str())
                {
                    return Err(fail("rename_readback"));
                }
            }
            for (path, before) in &self.guards {
                if optional(self.root, path)?.as_deref() != before.as_deref() {
                    return Err(fail("rename_readback"));
                }
            }
            for credential in &self.credentials {
                if account_digest(&credential.from)?.as_deref() != Some(credential.digest.as_str())
                {
                    return Err(fail("rename_credential_changed"));
                }
                if let Some(to) = &credential.to {
                    if account_digest(to)?.as_deref() != Some(credential.digest.as_str()) {
                        return Err(fail("rename_credential"));
                    }
                }
            }
            let snapshot = readback()?;
            checkpoint("readback")?;
            journal.committed = true;
            persist(self.root, &journal)?;
            checkpoint("committed")?;
            Ok(snapshot)
        })();
        if result
            .as_ref()
            .err()
            .is_some_and(|e| e.code == "rename_interrupted")
        {
            return result;
        }
        match result {
            Ok(snapshot) => {
                checkpoint("cleanup")?;
                recover_locked(self.root, self.home).map_err(|e| {
                    if e.code == "rename_recovery_required" {
                        e
                    } else {
                        fail("rename_cleanup_pending")
                    }
                })?;
                Ok(snapshot)
            }
            Err(error) => {
                recover_locked(self.root, self.home)
                    .map_err(|_| fail("rename_recovery_required"))?;
                Err(error)
            }
        }
    }
}

#[cfg(not(test))]
fn checkpoint(_: &str) -> ModelResult<()> {
    Ok(())
}
#[cfg(test)]
type TestHook = Option<Box<dyn FnMut(&str) -> ModelResult<()>>>;
#[cfg(test)]
thread_local! {pub(super) static TEST_HOOK:std::cell::RefCell<TestHook>=const {std::cell::RefCell::new(None)};}
#[cfg(test)]
fn checkpoint(point: &str) -> ModelResult<()> {
    TEST_HOOK.with(|hook| match &mut *hook.borrow_mut() {
        Some(f) => f(point),
        None => Ok(()),
    })
}
