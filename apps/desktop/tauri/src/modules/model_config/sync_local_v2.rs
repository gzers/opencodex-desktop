//! Host-only persistent saga, not a global atomic file/keychain transaction.
//! No IPC authority, root discovery, network, or secret-bearing Debug types.
use super::{
    file_objects_v2::{self, AuthorizationValidator, SavedObjects},
    objects_overlay_v2::{self, SelectionV2, UsageV2},
    portable_v2::PortableChannelV2,
    provider_auth::{self, PortableAuthorization},
    saved_objects_capture_v2::{self, CaptureCredentialsV2, CaptureRequestV2},
    schema::{invalid, ModelResult},
    sync_host_v2::{DirectionV2, LocalTargetBindingV2, TargetExpectationV2, TrustedMappingOwnerV2},
    sync_v2::types::*,
};
use crate::{
    infrastructure::{
        atomic_write::atomic_write,
        locking::{lock_path_for, TargetFileLock},
    },
    modules::{
        backup::{self, safety, BackupAction},
        sync::{portable_assets, portable_manager},
    },
};
use serde::{Deserialize, Serialize};
use sha2::{Digest as _, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
};
use zeroize::Zeroizing;

/// Trusted host boundary: use the exact D/H and explicit preview request.
pub trait SourceSnapshotProviderV2: Send + Sync {
    fn current(&self, d: &Path, h: &Path, request: &CaptureRequestV2) -> ModelResult<SourceGuard>;
    fn root(&self, d: &Path, h: &Path, request: &CaptureRequestV2) -> ModelResult<String>;
    fn target(&self, d: &Path, h: &Path, request: &CaptureRequestV2) -> ModelResult<String>;
    /// Recovery proof for unfinished NON-model writes after the model receipt.
    /// Check the frozen revision or THIS operation's receipted after-revision,
    /// never the original Absent fact or current capture. Otherwise fail closed.
    fn verify_applied_mapping(&self, _operation_id: &str, _binding: &LocalTargetBindingV2) -> ModelResult<()> {
        Err(invalid("host_mapping_changed", "映射恢复需要独立版本证明"))
    }
}
impl LocalApplication for LocalApplicationV2 {
    fn root_revision(&self) -> SyncResult<Digest> {
        self.source
            .root(&self.d, &self.h, &self.capture)
            .map_err(|_| failure(ErrorCode::SyncSourceChanged))
    }
    fn current_source(&self) -> SyncResult<SourceGuard> {
        let g = self
            .source
            .current(&self.d, &self.h, &self.capture)
            .map_err(|_| failure(ErrorCode::SyncSourceChanged))?;
        if g.scope_digest != self.scope {
            return Err(failure(ErrorCode::SyncScopeChanged));
        }
        Ok(g)
    }
    fn target_digest(&self, scope: &str) -> SyncResult<Digest> {
        if scope != self.scope {
            return Err(failure(ErrorCode::SyncScopeChanged));
        }
        self.source
            .target(&self.d, &self.h, &self.capture)
            .map_err(|_| failure(ErrorCode::SyncLocalChanged))
    }
    fn apply_once(
        &self,
        id: &str,
        bytes: &[u8],
        schema: &PayloadSchemaRef,
        scope: &str,
        expected: &str,
        backup: bool,
    ) -> SyncResult<()> {
        canonical_id(id)?;
        if scope != self.scope {
            return Err(failure(ErrorCode::SyncScopeChanged));
        }
        let objects = self.decode_selected(bytes, schema)?;
        // LocalFile password adoption needs a root-scoped credential installer.
        // Fail closed rather than silently drop passwords or use another vault.
        if !objects.partitions().webdav_passwords.is_empty() {
            return Err(failure(ErrorCode::SyncAuthRequired));
        }
        let _lock = self.lock(id)?;
        let mut j = self.freeze(id, bytes, schema, scope, expected, backup, false)?;
        if let Some(handle) = &j.model {
            if self.model_applied(id, handle)? && self.complete(id, handle)? {
                // Own participant receipts, not today's Absent/capture facts.
                self.verify_plan_guards(&j)?;
                if j.progress != LocalProgressV2::Applied {
                    j.progress = LocalProgressV2::Applied;
                    self.save(j)?;
                }
                return Ok(());
            }
        }
        let model_applied = j
            .model
            .as_ref()
            .map(|h| self.model_applied(id, h))
            .transpose()?
            .unwrap_or(false);
        self.verify_mapping(model_applied)?;
        if j.model.is_none() {
            self.verify_mapping(false)?;
            j.model = Some(
                self.models
                    .prepare(id, &objects, &self.selection, expected)
                    .map_err(|_| failure(ErrorCode::SyncLocalChanged))?,
            );
            j = self.save(j)?;
        }
        let handle = j
            .model
            .clone()
            .ok_or_else(|| failure(ErrorCode::SyncJournalInvalid))?;
        let stage = stage_id(id);
        if j.manager_guard.is_none() {
            // Persisted original stage wins after interruption; no new before.
            if !portable_manager::exists(&self.d, id)
                .map_err(|_| failure(ErrorCode::SyncLocalStateIo))?
            {
                if !portable_manager::exists(&self.d, &stage)
                    .map_err(|_| failure(ErrorCode::SyncLocalStateIo))?
                {
                    portable_manager::prepare(&self.d, &stage, objects.partitions().manager)
                        .map_err(|_| failure(ErrorCode::SyncLocalStateIo))?;
                    sync_parent(&self.d, &self.manager_path(&stage))?;
                }
                let s = &self.selection.manager;
                portable_manager::select(
                    &self.d,
                    &stage,
                    id,
                    s.preferences,
                    s.extensions,
                    &s.webdav_ids,
                    &s.account_ids,
                )
                .map_err(|_| failure(ErrorCode::SyncLocalStateIo))?;
                sync_parent(&self.d, &self.manager_path(id))?;
            }
            j.manager_guard = Some(self.guard(&self.manager_path(id), true)?);
            j = self.save(j)?;
        }
        if j.asset_guard.is_none() {
            let path = self.asset_path(id);
            checked(&self.d, &path)?;
            if !path.exists() {
                portable_assets::prepare(&self.d, &self.h, id, objects.partitions().assets)
                    .map_err(|_| failure(ErrorCode::SyncLocalStateIo))?;
                sync_parent(&self.d, &path)?;
            }
            j.asset_guard = Some(self.guard(&path, false)?);
            j.progress = LocalProgressV2::Prepared;
            j = self.save(j)?;
        }
        self.verify_guards(&j)?;
        if j.progress == LocalProgressV2::Applied {
            if !self.complete(id, &handle)? {
                return Err(failure(ErrorCode::SyncOperationUnknown));
            }
            return Ok(());
        }
        if !j.protected {
            if self.model_applied(id, &handle)? {
                return Err(failure(ErrorCode::SyncOperationUnknown));
            }
            if self.target_digest(scope)? != expected {
                return Err(failure(ErrorCode::SyncLocalChanged));
            }
            portable_manager::verify(&self.d, id)
                .map_err(|_| failure(ErrorCode::SyncLocalChanged))?;
            portable_assets::verify(&self.d, id, objects.partitions().assets)
                .map_err(|_| failure(ErrorCode::SyncLocalChanged))?;
            if j.targets.is_none() {
                let mut targets = self.manager_targets(id)?;
                targets.extend(self.asset_targets(id)?);
                targets.sort();
                targets.dedup();
                j.targets = Some(
                    targets
                        .into_iter()
                        .map(|(anchor, path)| {
                            Ok(Target {
                                image: image(&anchor, &path)?,
                                anchor,
                                path,
                            })
                        })
                        .collect::<SyncResult<_>>()?,
                );
                j = self.save(j)?;
            }
            // Interrupted backup may repeat protection, never preparation or
            // business writes. Compare every image to the frozen baseline.
            for t in j
                .targets
                .as_ref()
                .ok_or_else(|| failure(ErrorCode::SyncJournalInvalid))?
            {
                if image(&t.anchor, &t.path)? != t.image {
                    return Err(failure(ErrorCode::SyncLocalChanged));
                }
                if backup {
                    j.backups.extend(protect_tree(&self.d, &t.anchor, &t.path)?);
                }
                if image(&t.anchor, &t.path)? != t.image {
                    return Err(failure(ErrorCode::SyncLocalChanged));
                }
            }
            self.verify_mapping(false)?;
            self.models
                .protect(id, &handle, backup)
                .map_err(|_| failure(ErrorCode::SyncBackupFailed))?;
            self.verify_guards(&j)?;
            j.protected = true;
            j.progress = LocalProgressV2::Protected;
            j = self.save(j)?;
        }
        if !self.model_applied(id, &handle)? {
            self.verify_backups(&j)?;
            self.verify_mapping(false)?;
            self.models
                .protect(id, &handle, backup)
                .map_err(|_| failure(ErrorCode::SyncBackupFailed))?;
            self.verify_guards(&j)?;
            portable_manager::verify(&self.d, id)
                .map_err(|_| failure(ErrorCode::SyncLocalChanged))?;
            portable_assets::verify(&self.d, id, objects.partitions().assets)
                .map_err(|_| failure(ErrorCode::SyncLocalChanged))?;
            self.verify_mapping(false)?;
            let r = self
                .models
                .apply_once(id, &objects, &handle, backup)
                .map_err(|_| failure(ErrorCode::SyncOperationUnknown))?;
            if r.operation_id != id || r.plan != handle || !self.model_applied(id, &handle)? {
                return Err(failure(ErrorCode::SyncOperationUnknown));
            }
        }
        j.progress = j.progress.max(LocalProgressV2::ModelsApplied);
        j = self.save(j)?;
        self.verify_backups(&j)?;
        self.verify_guards(&j)?;
        self.verify_mapping_for_remaining(&j, &objects)?;
        portable_manager::verify(&self.d, id).map_err(|_| failure(ErrorCode::SyncLocalChanged))?;
        portable_manager::apply(&self.d, id)
            .map_err(|_| failure(ErrorCode::SyncOperationUnknown))?;
        sync_parent(&self.d, &self.manager_path(id))?;
        j.progress = j.progress.max(LocalProgressV2::ManagerApplied);
        j = self.save(j)?;
        self.verify_backups(&j)?;
        self.verify_guards(&j)?;
        self.verify_mapping_for_remaining(&j, &objects)?;
        portable_assets::verify(&self.d, id, objects.partitions().assets)
            .map_err(|_| failure(ErrorCode::SyncLocalChanged))?;
        let credential_ids: Vec<_> = self
            .selection
            .extension_credentials
            .skill_names
            .iter()
            .map(|s| format!("skill:{s}"))
            .chain(
                self.selection
                    .extension_credentials
                    .mcp_names
                    .iter()
                    .map(|s| format!("mcp:{s}")),
            )
            .collect();
        portable_assets::apply_selected(
            &self.d,
            id,
            objects.partitions().assets,
            !credential_ids.is_empty(),
            Some(&credential_ids),
        )
        .map_err(|_| failure(ErrorCode::SyncOperationUnknown))?;
        sync_parent(&self.d, &self.asset_path(id))?;
        j.progress = LocalProgressV2::AssetsApplied;
        j = self.save(j)?;
        self.verify_guards(&j)?;
        if !self.complete(id, &handle)? {
            return Err(failure(ErrorCode::SyncOperationUnknown));
        }
        j.progress = LocalProgressV2::Applied;
        self.save(j)?;
        Ok(())
    }
    fn export_once(
        &self,
        id: &str,
        bytes: &[u8],
        schema: &PayloadSchemaRef,
        scope: &str,
    ) -> SyncResult<Uuid> {
        canonical_id(id)?;
        if scope != self.scope {
            return Err(failure(ErrorCode::SyncScopeChanged));
        }
        self.decode_selected(bytes, schema)?;
        let _lock = self.lock(id)?;
        let mut j = self.freeze(id, bytes, schema, scope, "", false, true)?;
        let p = self.d.join("exports").join(format!("{id}.ocxdconf"));
        checked(&self.d, &p)?;
        if p.exists() {
            private(&p)?;
            let have = Zeroizing::new(
                safety::read_file(&p, file_objects_v2::FILE_LIMIT as u64)
                    .map_err(|_| failure(ErrorCode::SyncLocalStateIo))?,
            );
            if have.as_slice() != bytes {
                return Err(failure(ErrorCode::SyncOperationCollision));
            }
        } else {
            if j.progress == LocalProgressV2::Exported {
                return Err(failure(ErrorCode::SyncLocalChanged));
            }
            publish_new(&self.d, &p, bytes)?;
        }
        let have = Zeroizing::new(
            safety::read_file(&p, file_objects_v2::FILE_LIMIT as u64)
                .map_err(|_| failure(ErrorCode::SyncLocalStateIo))?,
        );
        if hash(&have) != j.payload_hash {
            return Err(failure(ErrorCode::SyncLocalChanged));
        }
        if j.progress != LocalProgressV2::Exported {
            j.progress = LocalProgressV2::Exported;
            self.save(j)?;
        }
        Ok(id.into())
    }
}

fn failure(code: ErrorCode) -> SyncFailure {
    SyncFailure {
        code,
        step: Step::ApplyLocal,
        retryability: Retryability::SameOperationGuarded,
        cause: None,
        http_status: None,
        recovery_actions: vec![],
    }
}
fn canonical_id(id: &str) -> SyncResult<()> {
    if uuid::Uuid::parse_str(id).is_ok_and(|u| u.to_string() == id && u.get_version_num() == 4) {
        Ok(())
    } else {
        Err(failure(ErrorCode::SyncOperationCollision))
    }
}
fn hash(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
fn json<T: Serialize>(v: &T) -> SyncResult<Vec<u8>> {
    serde_json::to_vec(v).map_err(|_| failure(ErrorCode::SyncJournalInvalid))
}
fn checked(anchor: &Path, p: &Path) -> SyncResult<()> {
    safety::check_path(anchor, p, true).map_err(|_| failure(ErrorCode::SyncLocalStateIo))
}
fn private(p: &Path) -> SyncResult<()> {
    safety::inspect(p).map_err(|_| failure(ErrorCode::SyncJournalInvalid))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if std::fs::metadata(p)
            .map_err(|_| failure(ErrorCode::SyncJournalInvalid))?
            .permissions()
            .mode()
            & 0o777
            != 0o600
        {
            return Err(failure(ErrorCode::SyncJournalInvalid));
        }
    }
    Ok(())
}
fn sync_parent(root: &Path, p: &Path) -> SyncResult<()> {
    safety::sync_directories(
        root,
        p.parent()
            .ok_or_else(|| failure(ErrorCode::SyncLocalStateIo))?,
    )
    .map_err(|_| failure(ErrorCode::SyncLocalStateIo))
}
fn stage_id(id: &str) -> String {
    let digest = Sha256::digest(format!("OCXD_LOCAL_MANAGER_STAGE_V2:{id}").as_bytes());
    let mut bytes = [0; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 15) | 64;
    bytes[8] = (bytes[8] & 63) | 128;
    uuid::Uuid::from_bytes(bytes).to_string()
}
// Bounded sorted tree hash includes modes/empty directories, never contents.
fn tree(
    anchor: &Path,
    path: &Path,
    out: &mut BTreeMap<PathBuf, String>,
    budget: &mut usize,
) -> SyncResult<()> {
    checked(anchor, path)?;
    if out.len() >= 20000 {
        return Err(failure(ErrorCode::SyncLimit));
    }
    let meta = match std::fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            out.insert(path.into(), "absent".into());
            return Ok(());
        }
        Err(_) => return Err(failure(ErrorCode::SyncLocalStateIo)),
    };
    safety::inspect(path).map_err(|_| failure(ErrorCode::SyncLocalStateIo))?;
    if meta.is_dir() {
        out.insert(path.into(), "directory".into());
        let mut children = std::fs::read_dir(path)
            .map_err(|_| failure(ErrorCode::SyncLocalStateIo))?
            .map(|p| {
                p.map(|p| p.path())
                    .map_err(|_| failure(ErrorCode::SyncLocalStateIo))
            })
            .collect::<SyncResult<Vec<_>>>()?;
        children.sort();
        for child in children {
            tree(anchor, &child, out, budget)?;
        }
    } else {
        let b = Zeroizing::new(
            safety::read_file(path, 16 * 1024 * 1024)
                .map_err(|_| failure(ErrorCode::SyncLocalStateIo))?,
        );
        *budget = budget
            .checked_sub(b.len())
            .ok_or_else(|| failure(ErrorCode::SyncLimit))?;
        #[cfg(unix)]
        let mode = {
            use std::os::unix::fs::PermissionsExt;
            meta.permissions().mode() & 0o777
        };
        #[cfg(not(unix))]
        let mode = 0;
        out.insert(path.into(), format!("{}:{mode}", hash(&b)));
    }
    Ok(())
}
fn image(anchor: &Path, p: &Path) -> SyncResult<String> {
    let mut out = BTreeMap::new();
    tree(anchor, p, &mut out, &mut (64 * 1024 * 1024))?;
    Ok(hash(&json(&out)?))
}
fn protect_tree(d: &Path, anchor: &Path, p: &Path) -> SyncResult<Vec<BackupRef>> {
    let mut entries = BTreeMap::new();
    let mut receipts = vec![];
    tree(anchor, p, &mut entries, &mut (64 * 1024 * 1024))?;
    for (path, kind) in entries {
        if kind == "absent" || kind == "directory" {
            continue;
        }
        let b = Zeroizing::new(
            safety::read_file(&path, 16 * 1024 * 1024)
                .map_err(|_| failure(ErrorCode::SyncBackupFailed))?,
        );
        let record = backup::backup_file(
            d,
            BackupAction::Import,
            &path,
            &b,
            chrono::Utc::now(),
            Some("同步接收采用前保护".into()),
        )
        .map_err(|_| failure(ErrorCode::SyncBackupFailed))?;
        if !backup::verify_backup(&record).map_err(|_| failure(ErrorCode::SyncBackupFailed))? {
            return Err(failure(ErrorCode::SyncBackupFailed));
        }
        receipts.push(BackupRef {
            manifest_hash: hash(&json(&record.manifest)?),
            directory: record.directory,
        });
    }
    Ok(receipts)
}
fn publish_new(d: &Path, p: &Path, bytes: &[u8]) -> SyncResult<()> {
    // Publish a complete fsynced file without replacing concurrent destinations.
    let parent = p
        .parent()
        .ok_or_else(|| failure(ErrorCode::SyncLocalStateIo))?;
    std::fs::create_dir_all(parent).map_err(|_| failure(ErrorCode::SyncLocalStateIo))?;
    checked(d, parent)?;
    let mut tmp = tempfile::NamedTempFile::new_in(parent)
        .map_err(|_| failure(ErrorCode::SyncLocalStateIo))?;
    crate::infrastructure::platform::set_mode(tmp.path(), 0o600)
        .map_err(|_| failure(ErrorCode::SyncLocalStateIo))?;
    use std::io::Write;
    tmp.write_all(bytes)
        .and_then(|_| tmp.as_file().sync_all())
        .map_err(|_| failure(ErrorCode::SyncLocalStateIo))?;
    std::fs::hard_link(tmp.path(), p).map_err(|_| failure(ErrorCode::SyncOperationCollision))?;
    drop(tmp);
    sync_parent(d, p)
}
#[cfg(test)]
#[path = "sync_local_v2_tests.rs"]
mod tests;
/// Production delegation; do not duplicate the saved-source capturer.
pub struct SavedSourceSnapshotProviderV2<C>(pub C);
impl<C: CaptureCredentialsV2 + Send + Sync> SourceSnapshotProviderV2
    for SavedSourceSnapshotProviderV2<C>
{
    fn current(&self, d: &Path, h: &Path, r: &CaptureRequestV2) -> ModelResult<SourceGuard> {
        Ok(saved_objects_capture_v2::capture(d, h, r, &self.0)?.source_guard)
    }
    fn root(&self, d: &Path, h: &Path, r: &CaptureRequestV2) -> ModelResult<String> {
        let _ = r;
        super::config_file::v2::with_saved_config(d, h, |_, _, _, _, _, revision| {
            Ok(revision.into())
        })
        .map_err(|_| invalid("local_source_root", "保存源根绑定无法核对"))
    }
    fn target(&self, d: &Path, h: &Path, r: &CaptureRequestV2) -> ModelResult<String> {
        Ok(saved_objects_capture_v2::capture(d, h, r, &self.0)?
            .source_guard
            .saved_source_digest)
    }
}

/// Host-only digest token. Never put source bytes or a secret into this token.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelApplyPlanHandle {
    token: String,
}
impl ModelApplyPlanHandle {
    pub fn new(token: String) -> ModelResult<Self> {
        if !super::schema_v2::valid_digest(&token) {
            return Err(invalid("local_model_handle", "模型计划句柄无效"));
        }
        Ok(Self { token })
    }
    pub fn token(&self) -> &str {
        &self.token
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct ModelApplyReceipt {
    pub operation_id: String,
    pub plan: ModelApplyPlanHandle,
}
#[derive(Clone, PartialEq, Eq)]
pub enum ModelApplyStatus {
    Prepared,
    Applied(ModelApplyReceipt),
    Unknown,
}
/// prepare freezes all model targets and reopens the original plan by id.
/// protect saves/verifies ALL model/config backups before any business write.
/// status checks the model transaction's OWN receipt; Applied must remain
/// Applied after a user's later edit, and apply_once must not rewrite it.
pub trait ModelApplyV2: Send + Sync {
    fn prepare(
        &self,
        id: &str,
        objects: &SavedObjects,
        scope: &SelectionV2,
        expected_target: &str,
    ) -> ModelResult<ModelApplyPlanHandle>;
    fn protect(&self, id: &str, handle: &ModelApplyPlanHandle, backup: bool) -> ModelResult<()>;
    fn apply_once(
        &self,
        id: &str,
        objects: &SavedObjects,
        handle: &ModelApplyPlanHandle,
        backup: bool,
    ) -> ModelResult<ModelApplyReceipt>;
    fn status(&self, id: &str, handle: &ModelApplyPlanHandle) -> ModelResult<ModelApplyStatus>;
}

struct ActualAuthorization;
/// Pure host preview boundary. Manager/assets and credential values are reduced
/// to the exact receive selection. Model public objects/version evidence remain
/// available to ModelApplyV2, whose planner MUST apply only the supplied scope.
/// A receive without connection adoption is valid for an existing target; the
/// no-baseline EXPORT overlay would incorrectly require a new connection here.
pub fn select_received_objects_v2(
    objects: SavedObjects,
    selection: &SelectionV2,
) -> ModelResult<SavedObjects> {
    let encoded = file_objects_v2::encode_document(&objects, Some(&ActualAuthorization))?;
    let baseline = file_objects_v2::decode_document(&encoded, Some(&ActualAuthorization))?;
    let validated = objects_overlay_v2::overlay_selected(
        Some(baseline),
        objects,
        selection.clone(),
        Some(&ActualAuthorization),
    )?;
    let mut full = validated.into_captured();
    let models = full.models.clone();
    let keys = std::mem::take(&mut full.channel_keys);
    let mut authorizations = std::mem::take(&mut full.channel_authorizations);
    let versions = full.content_versions.clone();
    let mut non_model = selection.clone();
    non_model.channels.clear();
    non_model.template_refs.clear();
    non_model.adopt_template_defaults.clear();
    non_model.include_global_template_refs = false;
    let selected = objects_overlay_v2::overlay_selected(
        None,
        SavedObjects::capture(full, Some(&ActualAuthorization))?,
        non_model,
        Some(&ActualAuthorization),
    )?;
    let mut selected = selected.into_captured();
    selected.models = models;
    selected.content_versions = versions;
    for c in &selection.channels {
        if c.include_key {
            let value = keys
                .get(&c.portable_channel_id)
                .ok_or_else(|| invalid("overlay_selection_missing", "所选凭据不存在"))?;
            selected
                .channel_keys
                .insert(c.portable_channel_id.clone(), Zeroizing::new(value.into()))?;
        }
        if c.include_authorization {
            selected.channel_authorizations.insert(
                c.portable_channel_id.clone(),
                authorizations
                    .remove(&c.portable_channel_id)
                    .ok_or_else(|| invalid("overlay_selection_missing", "所选授权不存在"))?,
            );
        }
    }
    SavedObjects::capture(selected, Some(&ActualAuthorization))
}
impl AuthorizationValidator for ActualAuthorization {
    fn validate(&self, c: &PortableChannelV2, a: &PortableAuthorization) -> ModelResult<()> {
        provider_auth::validate_portable(
            c.connection
                .as_ref()
                .and_then(|c| c.provider.as_ref())
                .ok_or_else(|| invalid("local_authorization_direction", "授权来源无效"))?,
            a,
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LocalProgressV2 {
    Frozen,
    Prepared,
    Protected,
    ModelsApplied,
    ManagerApplied,
    AssetsApplied,
    Applied,
    Exported,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    version: u32,
    id: String,
    export: bool,
    root: PathBuf,
    home: PathBuf,
    root_revision: String,
    payload_hash: String,
    payload_len: usize,
    schema: PayloadSchemaRef,
    scope: String,
    selection_hash: String,
    request_hash: String,
    #[serde(
        default,
        rename = "targetBindingDigest",
        skip_serializing_if = "Option::is_none"
    )]
    target_binding_digest: Option<String>,
    expected_target: String,
    backup: bool,
    progress: LocalProgressV2,
    model: Option<ModelApplyPlanHandle>,
    manager_guard: Option<String>,
    asset_guard: Option<String>,
    targets: Option<Vec<Target>>,
    backups: Vec<BackupRef>,
    protected: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckedJournal {
    checksum: String,
    journal: Journal,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Target {
    anchor: PathBuf,
    path: PathBuf,
    image: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BackupRef {
    directory: PathBuf,
    manifest_hash: String,
}

/// Construct in the host with explicit selection/context. No Debug/Serialize.
pub struct LocalApplicationV2 {
    d: PathBuf,
    h: PathBuf,
    selection: SelectionV2,
    capture: CaptureRequestV2,
    scope: String,
    source: Arc<dyn SourceSnapshotProviderV2>,
    models: Arc<dyn ModelApplyV2>,
    payload_passphrase: Option<Zeroizing<String>>,
    mapped: Option<MappedLocalContext>,
}
struct MappedLocalContext {
    binding: LocalTargetBindingV2,
    owner: Arc<dyn TrustedMappingOwnerV2>,
    root_revision: String,
}
impl LocalApplicationV2 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        d: PathBuf,
        h: PathBuf,
        selection: SelectionV2,
        capture: CaptureRequestV2,
        scope: String,
        source: Arc<dyn SourceSnapshotProviderV2>,
        models: Arc<dyn ModelApplyV2>,
        payload_passphrase: Option<Zeroizing<String>>,
    ) -> SyncResult<Self> {
        for p in [&d, &h] {
            if !p.is_absolute()
                || !safety::inspect(p)
                    .map_err(|_| failure(ErrorCode::SyncLocalStateIo))?
                    .is_dir()
            {
                return Err(failure(ErrorCode::SyncLocalStateIo));
            }
        }
        let d = std::fs::canonicalize(d).map_err(|_| failure(ErrorCode::SyncLocalStateIo))?;
        let h = std::fs::canonicalize(h).map_err(|_| failure(ErrorCode::SyncLocalStateIo))?;
        if !super::schema_v2::valid_digest(&scope) {
            return Err(failure(ErrorCode::SyncScopeChanged));
        }
        // Same object identities; incoming/local MCP definitions may differ.
        let mut incoming = selection.clone();
        let mut local = capture.selection.clone();
        for s in [&mut incoming, &mut local] {
            for m in &mut s.mcp_sources {
                m.definition_digest.clear();
            }
        }
        if incoming != local {
            return Err(failure(ErrorCode::SyncScopeChanged));
        }
        if (!selection.skill_names.is_empty() || !selection.mcp_sources.is_empty())
            && !capture.assets_reviewed
        {
            return Err(failure(ErrorCode::SyncScopeChanged));
        }
        Ok(Self {
            d,
            h,
            selection,
            capture,
            scope,
            source,
            models,
            payload_passphrase,
            mapped: None,
        })
    }
    /// The binding is trusted host memory only; journal stores its digest.
    #[allow(clippy::too_many_arguments)]
    pub fn new_mapped(
        d: PathBuf,
        h: PathBuf,
        selection: SelectionV2,
        capture: CaptureRequestV2,
        scope: String,
        source: Arc<dyn SourceSnapshotProviderV2>,
        models: Arc<dyn ModelApplyV2>,
        payload_passphrase: Option<Zeroizing<String>>,
        binding: LocalTargetBindingV2,
        owner: Arc<dyn TrustedMappingOwnerV2>,
    ) -> SyncResult<Self> {
        let fresh = LocalTargetBindingV2::freeze(
            binding.mapping_revision.clone(),
            binding.local_scope.clone(),
            binding.remote_scope.clone(),
            binding.direction.clone(),
            binding.slots.clone(),
        )?;
        if binding.digest != fresh.digest
            || binding.direction != DirectionV2::Download
            || selection != binding.remote_scope
            || capture.selection != binding.local_scope
        {
            return Err(failure(ErrorCode::SyncScopeChanged));
        }
        // Only channel identities may cross domains. Keep all other explicit
        // choices/context and the existing actual MCP digest relationship.
        let mut remote = selection.clone();
        let mut local = capture.selection.clone();
        remote.channels.clear();
        local.channels.clear();
        for s in [&mut remote, &mut local] {
            for m in &mut s.mcp_sources {
                m.definition_digest.clear();
            }
        }
        if remote != local {
            return Err(failure(ErrorCode::SyncScopeChanged));
        }
        for slot in &binding.slots {
            if matches!(slot.expected, TargetExpectationV2::Present(_)) {
                let mut r = selection
                    .channels
                    .iter()
                    .find(|c| c.portable_channel_id == slot.portable_id)
                    .ok_or_else(|| failure(ErrorCode::SyncScopeChanged))?
                    .clone();
                r.portable_channel_id = slot.local_id.clone();
                if !capture.selection.channels.contains(&r) {
                    return Err(failure(ErrorCode::SyncScopeChanged));
                }
            }
        }
        owner
            .verify(&binding)
            .map_err(|_| failure(ErrorCode::SyncPlanChanged))?;
        let mut host = Self::new(
            d,
            h,
            capture.selection.clone(),
            capture,
            scope,
            source,
            models,
            payload_passphrase,
        )?;
        host.selection = selection;
        // The injected provider must expose Host's MAPPED guard, never the raw
        // local capture guard. It retains the original explicit MCP clients.
        host.current_source()?;
        let root_revision = host.root_revision()?;
        host.mapped = Some(MappedLocalContext {
            binding,
            owner,
            root_revision,
        });
        Ok(host)
    }
    fn binding_digest(&self) -> Option<String> {
        self.mapped.as_ref().map(|m| m.binding.digest.clone())
    }
    fn verify_mapping(&self, model_applied: bool) -> SyncResult<()> {
        if let Some(m) = &self.mapped {
            if model_applied {
                return Ok(());
            }
            m.owner
                .verify(&m.binding)
                .map_err(|_| failure(ErrorCode::SyncPlanChanged))?;
            if self.root_revision()? != m.root_revision {
                return Err(failure(ErrorCode::SyncSourceChanged));
            }
        }
        Ok(())
    }
    fn verify_mapping_for_remaining(&self, j: &Journal, objects: &SavedObjects) -> SyncResult<()> {
        if let Some(m) = &self.mapped {
            let manager_done = self.manager_completed(&j.id)?;
            let asset_done = portable_assets::apply_progress(&self.d, &j.id)
                .map_err(|_| failure(ErrorCode::SyncJournalInvalid))?
                .0;
            let p = objects.partitions();
            let manager_work = !manager_done
                && (p.manager.preferences.is_some()
                    || p.manager.extensions.is_some()
                    || !p.manager.accounts.is_empty()
                    || !p.manager.webdav.is_empty());
            let asset_work =
                !asset_done && (!p.assets.skills.is_empty() || !p.assets.mcps.is_empty());
            if manager_work || asset_work {
                self.source
                    .verify_applied_mapping(&j.id, &m.binding)
                    .map_err(|_| failure(ErrorCode::SyncPlanChanged))?;
            }
        }
        Ok(())
    }
    pub fn progress(&self, id: &str) -> SyncResult<Option<LocalProgressV2>> {
        canonical_id(id)?;
        self.load(id).map(|j| j.map(|j| j.progress))
    }
    fn path(&self, id: &str) -> SyncResult<PathBuf> {
        canonical_id(id)?;
        let p = self
            .d
            .join("manager-state/sync-local-v2")
            .join(format!("{id}.json"));
        checked(&self.d, &p)?;
        Ok(p)
    }
    fn lock(&self, id: &str) -> SyncResult<TargetFileLock> {
        let path = self.path(id)?;
        checked(&self.d, &lock_path_for(&path))?;
        TargetFileLock::lock(&path).map_err(|_| failure(ErrorCode::SyncLocalStateIo))
    }
    fn load(&self, id: &str) -> SyncResult<Option<Journal>> {
        let p = self.path(id)?;
        if !p.exists() {
            return Ok(None);
        }
        private(&p)?;
        let bytes = safety::read_file(&p, 4 * 1024 * 1024)
            .map_err(|_| failure(ErrorCode::SyncJournalInvalid))?;
        let c: CheckedJournal =
            serde_json::from_slice(&bytes).map_err(|_| failure(ErrorCode::SyncJournalInvalid))?;
        // Exact emitted representation rejects duplicate keys and extra fields.
        if json(&c)? != bytes
            || hash(&json(&c.journal)?) != c.checksum
            || c.journal.id != id
            || c.journal.version != 2
        {
            return Err(failure(ErrorCode::SyncJournalInvalid));
        }
        let j = c.journal;
        if j.root != self.d || j.home != self.h {
            return Err(failure(ErrorCode::SyncSourceChanged));
        }
        Ok(Some(j))
    }
    fn save(&self, j: Journal) -> SyncResult<Journal> {
        let p = self.path(&j.id)?;
        let c = CheckedJournal {
            checksum: hash(&json(&j)?),
            journal: j,
        };
        atomic_write(&p, &json(&c)?, 0o600).map_err(|_| failure(ErrorCode::SyncLocalStateIo))?;
        sync_parent(&self.d, &p)?;
        Ok(c.journal)
    }
    #[allow(clippy::too_many_arguments)]
    fn freeze(
        &self,
        id: &str,
        bytes: &[u8],
        schema: &PayloadSchemaRef,
        scope: &str,
        expected: &str,
        backup: bool,
        export: bool,
    ) -> SyncResult<Journal> {
        let selection_hash = hash(&json(&self.selection)?);
        let request_hash = hash(&json(&self.capture)?);
        if let Some(j) = self.load(id)? {
            if j.export != export
                || j.payload_hash != hash(bytes)
                || j.payload_len != bytes.len()
                || j.schema != *schema
                || j.scope != scope
                || j.expected_target != expected
                || j.backup != backup
                || j.selection_hash != selection_hash
                || j.request_hash != request_hash
                || j.target_binding_digest != self.binding_digest()
            {
                return Err(failure(ErrorCode::SyncOperationCollision));
            }
            let root = match &self.mapped {
                Some(m) => m.root_revision.clone(),
                None => self.root_revision()?,
            };
            if j.root_revision != root {
                return Err(failure(ErrorCode::SyncSourceChanged));
            }
            return Ok(j);
        }
        if !export && self.target_digest(scope)? != expected {
            return Err(failure(ErrorCode::SyncLocalChanged));
        }
        if !export {
            self.verify_mapping(false)?;
        }
        if !export {
            for p in [
                self.manager_path(id),
                self.manager_path(&stage_id(id)),
                self.asset_path(id),
            ] {
                checked(&self.d, &p)?;
                if p.exists() {
                    return Err(failure(ErrorCode::SyncOperationCollision));
                }
            }
        }
        self.save(Journal {
            version: 2,
            id: id.into(),
            export,
            root: self.d.clone(),
            home: self.h.clone(),
            root_revision: self.root_revision()?,
            payload_hash: hash(bytes),
            payload_len: bytes.len(),
            schema: schema.clone(),
            scope: scope.into(),
            selection_hash,
            request_hash,
            target_binding_digest: self.binding_digest(),
            expected_target: expected.into(),
            backup,
            progress: LocalProgressV2::Frozen,
            model: None,
            manager_guard: None,
            asset_guard: None,
            targets: None,
            backups: vec![],
            protected: false,
        })
    }
    fn decode_selected(&self, bytes: &[u8], schema: &PayloadSchemaRef) -> SyncResult<SavedObjects> {
        if *schema != file_objects_v2::supported_schema() {
            return Err(failure(ErrorCode::SyncPayloadSchemaUnsupported));
        }
        let decoded = file_objects_v2::decode(
            bytes,
            self.payload_passphrase.as_ref().map(|s| s.as_str()),
            schema,
            Some(&ActualAuthorization),
        )
        .map_err(|e| {
            failure(match e.code.as_str() {
                "passphrase_required" => ErrorCode::SyncPassphraseRequired,
                "portable_authentication" => ErrorCode::SyncPassphraseInvalid,
                _ => ErrorCode::SyncPayloadInvalid,
            })
        })?;
        if self.selection.usage == UsageV2::RemoteSync {
            decoded
                .objects
                .validate_remote_sync_scope()
                .map_err(|_| failure(ErrorCode::SyncScopeChanged))?;
        }
        select_received_objects_v2(decoded.objects, &self.selection)
            .map_err(|_| failure(ErrorCode::SyncPayloadInvalid))
    }
    fn manager_path(&self, id: &str) -> PathBuf {
        self.d
            .join("manager-state/portable-manager-plans")
            .join(format!("{id}.json"))
    }
    fn asset_path(&self, id: &str) -> PathBuf {
        self.d
            .join("manager-state/extension-import-plans")
            .join(format!("{id}.json"))
    }
    fn guard(&self, p: &Path, manager: bool) -> SyncResult<String> {
        checked(&self.d, p)?;
        private(p)?;
        let bytes = Zeroizing::new(
            safety::read_file(p, file_objects_v2::FILE_LIMIT as u64)
                .map_err(|_| failure(ErrorCode::SyncJournalInvalid))?,
        );
        let mut v: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|_| failure(ErrorCode::SyncJournalInvalid))?;
        let allowed: &[&str] = if manager {
            &["id", "files"]
        } else {
            &[
                "id",
                "home",
                "digest",
                "storeBefore",
                "skillsBefore",
                "sourceBefore",
                "sourceRoot",
                "portableModes",
            ]
        };
        v.as_object_mut()
            .ok_or_else(|| failure(ErrorCode::SyncJournalInvalid))?
            .retain(|k, _| allowed.contains(&k.as_str()));
        Ok(hash(&json(&v)?))
    }
    fn verify_guards(&self, j: &Journal) -> SyncResult<()> {
        self.verify_plan_guards(j)?;
        if let Some(targets) = &j.targets {
            let mut actual = self.manager_targets(&j.id)?;
            actual.extend(self.asset_targets(&j.id)?);
            actual.sort();
            actual.dedup();
            if actual
                != targets
                    .iter()
                    .map(|t| (t.anchor.clone(), t.path.clone()))
                    .collect::<Vec<_>>()
            {
                return Err(failure(ErrorCode::SyncLocalChanged));
            }
        }
        Ok(())
    }
    fn verify_plan_guards(&self, j: &Journal) -> SyncResult<()> {
        if j.manager_guard.as_ref() != Some(&self.guard(&self.manager_path(&j.id), true)?)
            || j.asset_guard.as_ref() != Some(&self.guard(&self.asset_path(&j.id), false)?)
        {
            return Err(failure(ErrorCode::SyncJournalInvalid));
        }
        Ok(())
    }
    fn verify_backups(&self, j: &Journal) -> SyncResult<()> {
        for r in &j.backups {
            if !r.directory.starts_with(self.d.join("backups")) {
                return Err(failure(ErrorCode::SyncJournalInvalid));
            }
            checked(&self.d, &r.directory).map_err(|_| failure(ErrorCode::SyncBackupFailed))?;
            let raw = safety::read_file(&r.directory.join(backup::MANIFEST_NAME), 64 * 1024)
                .map_err(|_| failure(ErrorCode::SyncBackupFailed))?;
            let manifest: backup::BackupManifest =
                serde_json::from_slice(&raw).map_err(|_| failure(ErrorCode::SyncBackupFailed))?;
            if hash(&json(&manifest)?) != r.manifest_hash
                || !backup::verify_backup(&backup::BackupRecord {
                    manifest,
                    directory: r.directory.clone(),
                })
                .map_err(|_| failure(ErrorCode::SyncBackupFailed))?
            {
                return Err(failure(ErrorCode::SyncBackupFailed));
            }
        }
        Ok(())
    }
    fn complete(&self, id: &str, handle: &ModelApplyPlanHandle) -> SyncResult<bool> {
        Ok(self.manager_completed(id)?
            && portable_assets::apply_progress(&self.d, id)
                .map_err(|_| failure(ErrorCode::SyncJournalInvalid))?
                .0
            && self.model_applied(id, handle)?)
    }
    fn manager_completed(&self, id: &str) -> SyncResult<bool> {
        if !portable_manager::exists(&self.d, id)
            .map_err(|_| failure(ErrorCode::SyncJournalInvalid))?
        {
            return Ok(false);
        }
        let manager = Zeroizing::new(
            safety::read_file(&self.manager_path(id), file_objects_v2::FILE_LIMIT as u64)
                .map_err(|_| failure(ErrorCode::SyncJournalInvalid))?,
        );
        let manager: serde_json::Value =
            serde_json::from_slice(&manager).map_err(|_| failure(ErrorCode::SyncJournalInvalid))?;
        Ok(manager["completed"] == true)
    }
    fn manager_targets(&self, id: &str) -> SyncResult<Vec<(PathBuf, PathBuf)>> {
        let path = self.manager_path(id);
        checked(&self.d, &path)?;
        let raw = Zeroizing::new(
            safety::read_file(&path, file_objects_v2::FILE_LIMIT as u64)
                .map_err(|_| failure(ErrorCode::SyncJournalInvalid))?,
        );
        let v: serde_json::Value =
            serde_json::from_slice(&raw).map_err(|_| failure(ErrorCode::SyncJournalInvalid))?;
        v["files"]
            .as_array()
            .ok_or_else(|| failure(ErrorCode::SyncJournalInvalid))?
            .iter()
            .map(|f| {
                let p = self.d.join(
                    f["path"]
                        .as_str()
                        .ok_or_else(|| failure(ErrorCode::SyncJournalInvalid))?,
                );
                checked(&self.d, &p)?;
                Ok((self.d.clone(), p))
            })
            .collect()
    }
    fn asset_targets(&self, id: &str) -> SyncResult<Vec<(PathBuf, PathBuf)>> {
        Ok(portable_assets::backup_targets(&self.d, id)
            .map_err(|_| failure(ErrorCode::SyncLocalChanged))?
            .into_iter()
            .map(|(_, a, p)| {
                // The default source may not exist yet. Keep a stable existing
                // D/H trust anchor across its creation by the asset participant.
                let a = if p.starts_with(&self.d) {
                    self.d.clone()
                } else if p.starts_with(&self.h) {
                    self.h.clone()
                } else {
                    a
                };
                (a, p)
            })
            .collect())
    }
    fn model_applied(&self, id: &str, h: &ModelApplyPlanHandle) -> SyncResult<bool> {
        match self
            .models
            .status(id, h)
            .map_err(|_| failure(ErrorCode::SyncLocalStateIo))?
        {
            ModelApplyStatus::Applied(r) if r.operation_id == id && r.plan == *h => Ok(true),
            ModelApplyStatus::Prepared => Ok(false),
            _ => Err(failure(ErrorCode::SyncOperationUnknown)),
        }
    }
}
