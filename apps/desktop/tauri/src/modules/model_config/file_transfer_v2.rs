//! Host-only LocalFile owner. L0 retains this owner across dialogs/errors and
//! registers only public DTO methods. No v1 fallback or implicit root discovery.
use super::{
    config_file::v2::{self as config, receive::ReceiveFileHostV2},
    file_objects_v2::{self as codec, AuthorizationValidator, PublicSummary, SavedObjects},
    objects_overlay_v2::{self, SelectionV2, UsageV2},
    portable_v2::PortableChannelV2,
    provider_auth::{self, PortableAuthorization},
    receive_models_v2::{self, ReceiveBindingModeV2, ReceiveBindingV2},
    saved_objects_capture_v2::{self, CaptureCredentialsV2, CaptureRequestV2, CapturedSnapshotV2},
    schema::{invalid, ModelError, ModelResult},
    schema_v2::{self, LibraryV2, PolicyV2},
    source_v2::ReadBaseV2,
    store_v2,
    sync_local_v2::{self, LocalApplicationV2, ModelApplyV2, SourceSnapshotProviderV2},
    sync_v2::types::{LocalApplication, PayloadSchemaRef, SourceGuard},
};
use crate::modules::{
    backup::safety,
    extensions::{discovery, projection},
    preferences,
    sync::{portable_accounts, portable_assets, portable_manager},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    sync::Arc,
};
use zeroize::Zeroizing;

type Credentials = dyn CaptureCredentialsV2 + Send + Sync;
fn err(code: &str) -> ModelError {
    invalid(
        code,
        "配置文件操作未完成；原件与原操作已保留，请核对预览后重试",
    )
}
fn model(e: schema_v2::MatchErrorV2) -> ModelError {
    err(&e.code)
}
fn hash<T: Serialize>(v: &T) -> ModelResult<String> {
    Ok(schema_v2::digest(
        &schema_v2::canonical_bytes(v).map_err(model)?,
    ))
}
fn local(s: &SelectionV2) -> ModelResult<()> {
    if s.usage != UsageV2::LocalFile {
        return Err(err("file_transfer_local_scope_required"));
    }
    Ok(())
}
struct Auth;
impl AuthorizationValidator for Auth {
    fn validate(&self, c: &PortableChannelV2, a: &PortableAuthorization) -> ModelResult<()> {
        provider_auth::validate_portable(
            c.connection
                .as_ref()
                .and_then(|c| c.provider.as_ref())
                .ok_or_else(|| err("credential_scope"))?,
            a,
        )
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPlanV2 {
    pub id: String,
    pub scope: SelectionV2,
    #[serde(with = "summary_wire")]
    pub summary: PublicSummary,
    pub encrypted: bool,
    pub filename: String,
    pub schema: PayloadSchemaRef,
    pub digest: String,
    pub length: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FileTransferReceiptV2 {
    pub operation_id: String,
    pub plan_digest: String,
    pub file_digest: String,
    pub length: usize,
}
struct Export {
    plan: ExportPlanV2,
    bytes: Zeroizing<Vec<u8>>,
    attempted: bool,
    receipt: Option<FileTransferReceiptV2>,
    revision: Option<String>,
}
/// UI chooses channel identities only; never accepts an arbitrary provider ID.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum ChannelBindingChoiceV2 {
    Existing {
        portable_channel_id: String,
        local_channel_id: String,
    },
    New {
        portable_channel_id: String,
    },
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImportBindingsV2 {
    pub channels: Vec<ChannelBindingChoiceV2>,
    pub mcp_clients: BTreeMap<String, String>,
    pub assets_reviewed: bool,
    /// Explicitly adopt the file's namespace only for selected whole channels.
    #[serde(default)]
    pub adopt_namespace_channel_ids: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportObjectV2 {
    pub kind: String,
    pub id: String,
    pub label: String,
    /// Immutable library refs, never credential references.
    pub template_references: Vec<schema_v2::RefV2>,
    pub saved_default_revision_id: Option<String>,
    pub connection_available: bool,
    pub namespace_requires_adoption: bool,
    pub exported_model_ids: Option<Vec<String>>,
    pub exported_family_ids: Option<Vec<String>>,
    pub model_ids: Vec<String>,
    pub family_ids: Vec<String>,
    pub revision_ids: Vec<String>,
    pub existing_channel_ids: Vec<String>,
    pub same_id_exists: bool,
    pub new_identity_available: bool,
    pub has_key: bool,
    pub has_authorization: bool,
    pub has_credentials: bool,
    pub definition_digest: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account_info: Option<crate::modules::sync::portable_accounts::AccountInfo>,
    #[serde(default)]
    pub information_supported: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportDifferencesV2 {
    pub changed_channel_ids: Vec<String>,
    pub changed_template_refs: Vec<schema_v2::RefV2>,
    #[serde(with = "optional_summary_wire")]
    pub selected_summary: Option<PublicSummary>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreviewV2 {
    pub operation_id: String,
    pub plan_digest: String,
    #[serde(with = "summary_wire")]
    pub summary: PublicSummary,
    pub objects: Vec<ImportObjectV2>,
    pub selected_scope: SelectionV2,
    pub differences: ImportDifferencesV2,
    pub requires_backup: bool,
    pub can_confirm: bool,
    pub reason_code: Option<String>,
}
/// Trusted config_file helper extension. MUST hold admission/config/Store
/// locks, verify expected ReadBase, bind D/H/source/library/draftGeneration and
/// credential-action provenance, capture actual sources and recheck them.
/// No UI-supplied policy; never called by ordinary saved/WebDAV capture.
pub trait DraftFileCaptureV2: Send + Sync {
    fn capture(
        &self,
        d: &Path,
        h: &Path,
        expected: &ReadBaseV2,
        request: &CaptureRequestV2,
        credentials: &dyn CaptureCredentialsV2,
    ) -> ModelResult<CapturedSnapshotV2>;
    fn capture_with_intents(
        &self,
        d: &Path,
        h: &Path,
        expected: &ReadBaseV2,
        request: &CaptureRequestV2,
        credentials: &dyn CaptureCredentialsV2,
        intents: DraftFileIntentsV2,
    ) -> ModelResult<CapturedSnapshotV2> {
        if !intents.edits.is_empty() || !intents.channel_intents.is_empty() {
            return Err(err("file_draft_intents_not_connected"));
        }
        self.capture(d, h, expected, request, credentials)
    }
}
#[derive(Default)]
pub struct DraftFileIntentsV2 {
    pub edits: Vec<super::source_v2::ModelEditV2>,
    pub channel_intents: Vec<config::ChannelIntentV2>,
}
pub struct DraftExportRequestV2 {
    pub expected: ReadBaseV2,
    pub capture: CaptureRequestV2,
}
/// Trusted root-scoped participant. prepare_and_protect freezes before-images,
/// fresh refs, exact selected material and backup proofs under THIS operation;
/// it must not write credential/candidate business state yet. apply_once runs
/// after local apply, CAS-updates ONLY this operation's new inactive candidates,
/// installs credentials in D and persists its OWN receipt. Retrying must first
/// consult that receipt and preserve later user edits. Recover partial writes,
/// never enable endpoints. Implement with L0's store_at/remove_at helpers.
/// bind_passwords cannot mutate the manager plan after its guard was frozen.
pub trait RootScopedWebDavPasswordInstallerV2: Send + Sync {
    fn prepare_and_protect(
        &self,
        d: &Path,
        operation_id: &str,
        plan_digest: &str,
        objects: &SavedObjects,
        backup: bool,
    ) -> ModelResult<()>;
    fn apply_once(
        &self,
        d: &Path,
        operation_id: &str,
        plan_digest: &str,
        objects: &SavedObjects,
    ) -> ModelResult<()>;
    fn receipted(&self, d: &Path, operation_id: &str, plan_digest: &str) -> ModelResult<bool>;
}
struct Import {
    original: Zeroizing<Vec<u8>>,
    passphrase: Option<Zeroizing<String>>,
    objects: SavedObjects,
    choices: ImportBindingsV2,
    revision: Option<String>,
    preview: ImportPreviewV2,
    application: Option<Prepared>,
    attempted: bool,
    backup: Option<bool>,
    receipt: Option<FileTransferReceiptV2>,
}
struct Prepared {
    host: LocalApplicationV2,
    payload: Zeroizing<Vec<u8>>,
    scope: String,
    target: String,
    selected: SavedObjects,
    context: PreparedContext,
}
/// Frozen operations are authenticated/encrypted at rest. Reopening restores
/// exact original bytes/context and never invokes capture or today's preview.
pub struct FileTransferOwnerV2 {
    d: PathBuf,
    h: PathBuf,
    credentials: Arc<Credentials>,
    exports: BTreeMap<String, Export>,
    imports: BTreeMap<String, Import>,
    draft: Option<Arc<dyn DraftFileCaptureV2>>,
    recovery_keys: Arc<dyn FileRecoveryKeysV2>,
    namespace: Option<Arc<dyn NamespaceReceiveV2>>,
    passwords: Option<Arc<dyn RootScopedWebDavPasswordInstallerV2>>,
}
impl FileTransferOwnerV2 {
    pub fn new(d: PathBuf, h: PathBuf, credentials: Arc<Credentials>) -> ModelResult<Self> {
        for p in [&d, &h] {
            if !p.is_absolute() || !safety::inspect(p)?.is_dir() {
                return Err(err("file_transfer_root_invalid"));
            }
        }
        Ok(Self {
            d: d.canonicalize()?,
            h: h.canonicalize()?,
            credentials,
            exports: BTreeMap::new(),
            imports: BTreeMap::new(),
            draft: None,
            recovery_keys: Arc::new(SystemFileRecoveryKeysV2),
            namespace: None,
            passwords: Some(Arc::new(SystemWebDavPasswordInstallerV2)),
        })
    }
    pub fn export_plan(&self, id: &str) -> ModelResult<ExportPlanV2> {
        self.exports
            .get(id)
            .map(|o| o.plan.clone())
            .ok_or_else(|| err("file_operation_missing"))
    }
    pub fn import_preview(&self, id: &str) -> ModelResult<ImportPreviewV2> {
        self.imports
            .get(id)
            .map(|o| o.preview.clone())
            .ok_or_else(|| err("file_operation_missing"))
    }
    pub fn with_recovery_keys(mut self, keys: Arc<dyn FileRecoveryKeysV2>) -> Self {
        self.recovery_keys = keys;
        self
    }
    pub fn with_namespace_receive(mut self, helper: Arc<dyn NamespaceReceiveV2>) -> Self {
        self.namespace = Some(helper);
        self
    }
    pub fn with_draft_capture(mut self, helper: Arc<dyn DraftFileCaptureV2>) -> Self {
        self.draft = Some(helper);
        self
    }
    pub fn with_password_installer(
        mut self,
        helper: Arc<dyn RootScopedWebDavPasswordInstallerV2>,
    ) -> Self {
        self.passwords = Some(helper);
        self
    }
    pub fn prepare_export(
        &mut self,
        request: CaptureRequestV2,
        passphrase: Option<Zeroizing<String>>,
    ) -> ModelResult<ExportPlanV2> {
        local(&request.selection)?;
        let captured = saved_objects_capture_v2::capture(
            &self.d,
            &self.h,
            &request,
            self.credentials.as_ref(),
        )?;
        self.freeze_export(request.selection, captured, passphrase)
    }
    pub fn prepare_draft_export(
        &mut self,
        request: DraftExportRequestV2,
        passphrase: Option<Zeroizing<String>>,
    ) -> ModelResult<ExportPlanV2> {
        self.prepare_draft_export_with_intents(request, passphrase, DraftFileIntentsV2::default())
    }
    pub fn prepare_draft_export_with_intents(
        &mut self,
        request: DraftExportRequestV2,
        passphrase: Option<Zeroizing<String>>,
        intents: DraftFileIntentsV2,
    ) -> ModelResult<ExportPlanV2> {
        local(&request.capture.selection)?;
        let helper = self
            .draft
            .as_ref()
            .ok_or_else(|| err("file_draft_capture_not_connected"))?;
        let captured = helper.capture_with_intents(
            &self.d,
            &self.h,
            &request.expected,
            &request.capture,
            self.credentials.as_ref(),
            intents,
        )?;
        self.freeze_export(request.capture.selection, captured, passphrase)
    }
    fn freeze_export(
        &mut self,
        scope: SelectionV2,
        captured: CapturedSnapshotV2,
        passphrase: Option<Zeroizing<String>>,
    ) -> ModelResult<ExportPlanV2> {
        let id = uuid::Uuid::new_v4().to_string();
        let file = codec::encode(
            &captured.objects,
            &id,
            &chrono::Utc::now().to_rfc3339(),
            passphrase.as_ref().map(|p| p.as_str()),
            Some(&Auth),
        )?;
        let plan = ExportPlanV2 {
            id: id.clone(),
            scope,
            summary: captured.summary,
            encrypted: file.protected,
            filename: format!("OpenCodex-{id}.ocxdconf"),
            schema: file.schema.clone(),
            digest: file.file_digest.clone(),
            length: file.bytes().len(),
        };
        self.exports.insert(
            id,
            Export {
                plan: plan.clone(),
                bytes: file.into_bytes(),
                attempted: false,
                receipt: None,
                revision: None,
            },
        );
        self.persist_export(&plan.id)?;
        Ok(plan)
    }
    /// Host-only immutable bridge to nativeSaveDialog exporter. Call only after
    /// explicit user destination choice, before writing. Marks unknown output.
    pub fn bytes(&mut self, id: &str) -> ModelResult<&[u8]> {
        self.ensure_export(id)?;
        let e = self
            .exports
            .get_mut(id)
            .ok_or_else(|| err("file_operation_missing"))?;
        e.attempted = true;
        self.persist_export(id)?;
        Ok(&self
            .exports
            .get(id)
            .ok_or_else(|| err("file_operation_missing"))?
            .bytes)
    }
    /// L0 must supply actual written/read-back digest, not the planned digest.
    pub fn finish_export(
        &mut self,
        id: &str,
        actual_written_digest: &str,
    ) -> ModelResult<FileTransferReceiptV2> {
        self.ensure_export(id)?;
        let e = self
            .exports
            .get_mut(id)
            .ok_or_else(|| err("file_operation_missing"))?;
        if !e.attempted || e.plan.digest != actual_written_digest {
            return Err(err("file_output_unknown"));
        }
        let receipt = FileTransferReceiptV2 {
            operation_id: id.into(),
            plan_digest: e.plan.digest.clone(),
            file_digest: e.plan.digest.clone(),
            length: e.bytes.len(),
        };
        e.receipt = Some(receipt.clone());
        self.persist_export(id)?;
        Ok(receipt)
    }
    /// false: output may exist; retain exact bytes/id for retry.
    pub fn cancel_export(&mut self, id: &str) -> ModelResult<bool> {
        self.ensure_export(id)?;
        let e = self
            .exports
            .get(id)
            .ok_or_else(|| err("file_operation_missing"))?;
        if e.attempted {
            return Ok(false);
        }
        self.discard_recovery(id, e.revision.as_deref())?;
        self.exports.remove(id);
        Ok(true)
    }
    pub fn prepare_import(
        &mut self,
        bytes: Vec<u8>,
        passphrase: Option<Zeroizing<String>>,
        selected_scope: Option<SelectionV2>,
        bindings: ImportBindingsV2,
    ) -> ModelResult<ImportPreviewV2> {
        let bytes = Zeroizing::new(bytes);
        let decoded = decode(&bytes, passphrase.as_ref().map(|p| p.as_str()))?;
        let id = uuid::Uuid::new_v4().to_string();
        let scope = selected_scope.unwrap_or_else(|| SelectionV2 {
            usage: UsageV2::LocalFile,
            ..Default::default()
        });
        let (preview, application) = self.preview(
            &id,
            &bytes,
            &decoded.objects,
            &scope,
            &bindings,
            passphrase.as_ref(),
        )?;
        self.imports.insert(
            id.clone(),
            Import {
                original: bytes,
                passphrase: None,
                objects: decoded.objects,
                choices: bindings,
                revision: None,
                preview: preview.clone(),
                application,
                attempted: false,
                backup: None,
                receipt: None,
            },
        );
        self.persist_import(&id)?;
        Ok(preview)
    }
    pub fn reselect(
        &mut self,
        id: &str,
        expected_plan_digest: &str,
        scope: SelectionV2,
        bindings: ImportBindingsV2,
    ) -> ModelResult<ImportPreviewV2> {
        self.ensure_import(id)?;
        let op = self
            .imports
            .get(id)
            .ok_or_else(|| err("file_operation_missing"))?;
        if op.attempted {
            return Err(err("file_operation_frozen"));
        }
        if op.preview.plan_digest != expected_plan_digest {
            return Err(err("file_plan_changed"));
        }
        let (preview, application) = self.preview(
            id,
            &op.original,
            &op.objects,
            &scope,
            &bindings,
            op.passphrase.as_ref(),
        )?;
        let op = self
            .imports
            .get_mut(id)
            .ok_or_else(|| err("file_operation_missing"))?;
        op.preview = preview.clone();
        op.application = application;
        op.choices = bindings;
        self.persist_import(id)?;
        Ok(preview)
    }
    pub fn cancel_import(&mut self, id: &str) -> ModelResult<bool> {
        self.ensure_import(id)?;
        let op = self
            .imports
            .get(id)
            .ok_or_else(|| err("file_operation_missing"))?;
        if op.attempted {
            return Ok(false);
        }
        self.discard_recovery(id, op.revision.as_deref())?;
        self.imports.remove(id);
        Ok(true)
    }
    pub fn confirm_import(
        &mut self,
        id: &str,
        expected_plan_digest: &str,
        backup_default_true: Option<bool>,
    ) -> ModelResult<FileTransferReceiptV2> {
        self.ensure_import(id)?;
        self.refresh_import(id)?;
        let op = self
            .imports
            .get_mut(id)
            .ok_or_else(|| err("file_operation_missing"))?;
        if op.preview.plan_digest != expected_plan_digest {
            return Err(err("file_plan_changed"));
        }
        if let Some(r) = &op.receipt {
            return Ok(r.clone());
        }
        if !op.preview.can_confirm {
            return Err(err(op
                .preview
                .reason_code
                .as_deref()
                .unwrap_or("file_selection_required")));
        }
        let backup = backup_default_true.unwrap_or(true);
        if op.backup.is_some_and(|b| b != backup) {
            return Err(err("file_operation_frozen"));
        }
        op.attempted = true;
        op.backup = Some(backup);
        self.persist_import(id)?;
        let op = self
            .imports
            .get(id)
            .ok_or_else(|| err("file_operation_missing"))?;
        let p = op
            .application
            .as_ref()
            .ok_or_else(|| err("file_selection_required"))?;
        let has_passwords = !p.selected.partitions().webdav_passwords.is_empty();
        if has_passwords {
            self.passwords
                .as_ref()
                .ok_or_else(|| err("file_password_installer_not_connected"))?
                .prepare_and_protect(&self.d, id, expected_plan_digest, &p.selected, backup)
                .map_err(|e| err(&e.code))?;
        }
        p.host
            .apply_once(
                id,
                &p.payload,
                &codec::supported_schema(),
                &p.scope,
                &p.target,
                backup,
            )
            .map_err(|e| err(&format!("file_local_{:?}", e.code)))?;
        if has_passwords {
            let installer = self
                .passwords
                .as_ref()
                .ok_or_else(|| err("file_password_installer_not_connected"))?;
            installer
                .apply_once(&self.d, id, expected_plan_digest, &p.selected)
                .map_err(|e| err(&e.code))?;
            if !installer
                .receipted(&self.d, id, expected_plan_digest)
                .map_err(|e| err(&e.code))?
            {
                return Err(err("file_password_receipt_unknown"));
            }
        }
        let r = FileTransferReceiptV2 {
            operation_id: id.into(),
            plan_digest: expected_plan_digest.into(),
            file_digest: schema_v2::digest(&op.original),
            length: op.original.len(),
        };
        self.imports
            .get_mut(id)
            .ok_or_else(|| err("file_operation_missing"))?
            .receipt = Some(r.clone());
        if let Err(e) = self.persist_import(id) {
            self.imports
                .get_mut(id)
                .ok_or_else(|| err("file_operation_missing"))?
                .receipt = None;
            return Err(e);
        }
        Ok(r)
    }
}

fn decode(bytes: &[u8], password: Option<&str>) -> ModelResult<codec::DecodedFile> {
    if bytes.len() > codec::FILE_LIMIT {
        return Err(err("portable_limit"));
    }
    // Classification only; the codec remains the strict parser. Zeroize this
    // transient tree too: plain wrappers can contain secret-bearing extra data.
    let mut header: serde_json::Value =
        serde_json::from_slice(bytes).map_err(|_| err("portable_format"))?;
    let supported = header.get("format").and_then(|v| v.as_str()) == Some("OCXDCONF")
        && header.get("formatVersion").and_then(|v| v.as_u64()) == Some(3);
    erase_json(&mut header);
    if !supported {
        return Err(err("file_legacy_format_unsupported"));
    }
    let result = codec::decode(bytes, password, &codec::supported_schema(), Some(&Auth));
    if let Err(e) = &result {
        if matches!(
            e.code.as_str(),
            "portable_objects_v2_format"
                | "portable_objects_v2_version"
                | "model_schema_unsupported"
        ) {
            if let Ok(opened) = crate::modules::sync::portable_file::open(bytes, password) {
                if let Ok(mut doc) = serde_json::from_slice::<serde_json::Value>(&opened.document) {
                    let legacy = doc["format"] == "OCXD_CONFIG_OBJECTS"
                        || doc["format"] == "OCXD_MODEL_CONFIG"
                        || doc
                            .get("version")
                            .and_then(|v| v.as_u64())
                            .is_some_and(|v| v < 2)
                        || doc
                            .pointer("/models/formatVersion")
                            .and_then(|v| v.as_u64())
                            .is_some_and(|v| v < 6)
                        || doc
                            .pointer("/models/schemaVersion")
                            .and_then(|v| v.as_u64())
                            .is_some_and(|v| v < 2);
                    erase_json(&mut doc);
                    if legacy {
                        return Err(err("file_legacy_format_unsupported"));
                    }
                }
            }
        }
    }
    result
}
fn erase_json(v: &mut serde_json::Value) {
    use zeroize::Zeroize;
    match v {
        serde_json::Value::String(s) => s.zeroize(),
        serde_json::Value::Array(a) => a.iter_mut().for_each(erase_json),
        serde_json::Value::Object(o) => {
            for (mut k, mut v) in std::mem::take(o) {
                k.zeroize();
                erase_json(&mut v);
            }
        }
        _ => {}
    }
}
struct Inventory {
    library: LibraryV2,
    policy: PolicyV2,
    providers: BTreeSet<String>,
    digest: String,
}
fn inventory(d: &Path, h: &Path) -> ModelResult<Inventory> {
    config::with_saved_config(d, h, |library, policy, mappings, raw, raw_digest, root| {
        Ok(Inventory {
            library: library.clone(),
            policy: policy.clone(),
            providers: raw
                .get("providers")
                .and_then(|p| p.as_object())
                .map(|p| p.keys().cloned().collect())
                .unwrap_or_default(),
            digest: schema_v2::digest(&schema_v2::canonical_bytes(&(
                library, policy, mappings, raw_digest, root,
            ))?),
        })
    })
    .map_err(model)
}
fn binding_plan(
    inv: &Inventory,
    s: &SelectionV2,
    choices: &ImportBindingsV2,
) -> ModelResult<Vec<ReceiveBindingV2>> {
    let mut out = vec![];
    let mut seen = BTreeSet::new();
    for choice in &choices.channels {
        let (portable, local_id, mode) = match choice {
            ChannelBindingChoiceV2::Existing {
                portable_channel_id,
                local_channel_id,
            } => (
                portable_channel_id,
                local_channel_id,
                ReceiveBindingModeV2::Existing,
            ),
            ChannelBindingChoiceV2::New {
                portable_channel_id,
            } => (
                portable_channel_id,
                portable_channel_id,
                ReceiveBindingModeV2::New,
            ),
        };
        if !seen.insert(portable)
            || !s
                .channels
                .iter()
                .any(|c| &c.portable_channel_id == portable)
        {
            return Err(err("file_binding_invalid"));
        }
        schema_v2::check_id(portable).map_err(model)?;
        let provider = match mode {
            ReceiveBindingModeV2::Existing => inv
                .policy
                .channels
                .iter()
                .find(|c| &c.id == local_id)
                .ok_or_else(|| err("file_binding_inventory_changed"))?
                .connection
                .provider_id
                .clone(),
            ReceiveBindingModeV2::New => {
                if inv.policy.channels.iter().any(|c| &c.id == local_id) {
                    return Err(err("file_binding_identity_conflict"));
                }
                let p = format!("imported_{}", local_id.replace('-', ""));
                if inv.providers.contains(&p) {
                    return Err(err("file_binding_provider_conflict"));
                }
                p
            }
        };
        out.push(ReceiveBindingV2 {
            portable_channel_id: portable.clone(),
            local_channel_id: local_id.clone(),
            provider_id: provider,
            mode,
        });
    }
    if out.len() != s.channels.len() {
        return Err(err("file_binding_required"));
    }
    Ok(out)
}
fn object(kind: &str, id: &str) -> ImportObjectV2 {
    ImportObjectV2 {
        kind: kind.into(),
        id: id.into(),
        label: id.into(),
        template_references: vec![],
        saved_default_revision_id: None,
        connection_available: false,
        namespace_requires_adoption: false,
        exported_model_ids: None,
        exported_family_ids: None,
        model_ids: vec![],
        family_ids: vec![],
        revision_ids: vec![],
        existing_channel_ids: vec![],
        same_id_exists: false,
        new_identity_available: false,
        has_key: false,
        has_authorization: false,
        has_credentials: false,
        definition_digest: None,
        account_info: None,
        information_supported: false,
    }
}
fn object_inventory(o: &SavedObjects, inv: &Inventory) -> ModelResult<Vec<ImportObjectV2>> {
    let p = o.partitions();
    let mut out = vec![];
    for c in o.models().payload.policy.iter().flat_map(|p| &p.channels) {
        let mut row = object("channel", &c.id);
        row.label = c.label.clone();
        row.connection_available = c.connection.is_some();
        row.namespace_requires_adoption = c.namespace.is_some();
        row.exported_model_ids = c.model_subset.clone();
        row.exported_family_ids = c.family_scope.clone();
        row.model_ids = c.models.iter().map(|m| m.raw_model_id.clone()).collect();
        row.family_ids = c.family_scope.clone().unwrap_or_else(|| {
            c.family_overrides
                .keys()
                .cloned()
                .chain(c.template_refs.iter().map(|r| r.template_id.clone()))
                .chain(c.models.iter().filter_map(|m| {
                    m.binding
                        .as_ref()
                        .map(|b| b.reference().template_id.clone())
                }))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect()
        });
        row.existing_channel_ids = inv.policy.channels.iter().map(|c| c.id.clone()).collect();
        row.same_id_exists = inv.policy.channels.iter().any(|l| l.id == c.id);
        row.new_identity_available = !row.same_id_exists
            && !inv
                .providers
                .contains(&format!("imported_{}", c.id.replace('-', "")));
        row.has_key = p.channel_keys.get(&c.id).is_some();
        row.has_authorization = p.channel_authorizations.contains_key(&c.id);
        out.push(row);
    }
    for t in &o.models().payload.templates {
        let mut row = object("template", &t.id);
        row.label = t.label.clone();
        row.saved_default_revision_id = t.default_revision.clone();
        row.template_references = t
            .revisions
            .iter()
            .map(|r| schema_v2::RefV2 {
                template_id: t.id.clone(),
                revision_id: r.id.clone(),
                digest: r.digest.clone(),
            })
            .collect();
        row.revision_ids = t.revisions.iter().map(|r| r.id.clone()).collect();
        out.push(row);
    }
    if p.manager.preferences.is_some() {
        out.push(object("preferences", "preferences"));
    }
    if p.manager.extensions.is_some() {
        out.push(object("extensions", "extensions"));
    }
    for a in &p.manager.accounts {
        let mut row = object("account", &a.id);
        row.label = a.label.clone();
        row.information_supported = portable_accounts::information_supported(a);
        row.account_info = Some(a.clone());
        out.push(row);
    }
    for e in &p.manager.webdav {
        let mut row = object("webdav_candidate", &e.id);
        row.has_credentials = p.webdav_passwords.get(&e.id).is_some();
        out.push(row);
    }
    for s in &p.assets.skills {
        let mut row = object("skill", &s.name);
        row.has_credentials = s.contains_secrets;
        out.push(row);
    }
    for m in &p.assets.mcps {
        let mut row = object("mcp", &m.name);
        row.has_credentials = !m.secrets.0.is_empty();
        row.definition_digest = Some(objects_overlay_v2::mcp_source_digest(m)?);
        out.push(row);
    }
    Ok(out)
}
impl FileTransferOwnerV2 {
    #[allow(clippy::too_many_arguments)]
    fn preview(
        &self,
        id: &str,
        bytes: &[u8],
        objects: &SavedObjects,
        scope: &SelectionV2,
        choices: &ImportBindingsV2,
        _password: Option<&Zeroizing<String>>,
    ) -> ModelResult<(ImportPreviewV2, Option<Prepared>)> {
        local(scope)?;
        let inv = inventory(&self.d, &self.h)?;
        let rows = object_inventory(objects, &inv)?;
        // Validate receive scope before deriving file targets. In particular,
        // untrusted asset names never become paths before asset validation.
        let validated = narrow_received(objects, scope)
            .and_then(|o| sync_local_v2::select_received_objects_v2(o, scope));
        let target = match &validated {
            Ok(_)
                if choices.assets_reviewed
                    || (scope.skill_names.is_empty() && scope.mcp_sources.is_empty()) =>
            {
                target_digest(&self.d, &self.h, scope, choices)?
            }
            _ => inv.digest.clone(),
        };
        let plan_digest = hash(&(
            id,
            schema_v2::digest(bytes),
            scope,
            choices,
            &inv.digest,
            &target,
        ))?;
        let mut preview = ImportPreviewV2 {
            operation_id: id.into(),
            plan_digest,
            summary: objects.summary(),
            objects: rows,
            selected_scope: scope.clone(),
            differences: ImportDifferencesV2 {
                changed_channel_ids: vec![],
                changed_template_refs: vec![],
                selected_summary: None,
            },
            requires_backup: true,
            can_confirm: false,
            reason_code: None,
        };
        let result = (|| -> ModelResult<Prepared> {
            let selected = validated?;
            if *scope
                == (SelectionV2 {
                    usage: UsageV2::LocalFile,
                    ..Default::default()
                })
            {
                return Err(err("file_selection_required"));
            }
            if (!scope.skill_names.is_empty() || !scope.mcp_sources.is_empty())
                && !choices.assets_reviewed
            {
                return Err(err("capture_assets_review_required"));
            }
            if choices
                .mcp_clients
                .keys()
                .any(|n| !scope.mcp_sources.iter().any(|m| &m.name == n))
            {
                return Err(err("file_mcp_source_invalid"));
            }
            let bindings = binding_plan(&inv, scope, choices)?;
            validate_namespace_choice(&selected, scope, choices)?;
            let planned = if choices.adopt_namespace_channel_ids.is_empty() {
                receive_models_v2::plan(
                    &inv.library,
                    &inv.policy,
                    selected.models(),
                    scope,
                    &bindings,
                )
                .map_err(model)?
            } else {
                self.namespace
                    .as_ref()
                    .ok_or_else(|| err("file_namespace_receiver_not_connected"))?
                    .plan(
                        &inv.library,
                        &inv.policy,
                        &selected,
                        scope,
                        &bindings,
                        &choices.adopt_namespace_channel_ids,
                    )?
            };
            preview.differences.changed_channel_ids = planned.changed_channel_ids;
            preview.differences.changed_template_refs = planned.changed_template_refs;
            preview.differences.selected_summary = Some(selected.summary());
            if !selected.partitions().webdav_passwords.is_empty() && self.passwords.is_none() {
                return Err(err("file_password_installer_not_connected"));
            }
            // Freeze a password-free local application payload ONCE. Retain
            // original file/selected secrets for the same-operation installer.
            let mut application_scope = scope.clone();
            application_scope.manager.webdav_password_ids.clear();
            let mut application_objects = codec::decode_document(
                &codec::encode_document(&selected, Some(&Auth))?,
                Some(&Auth),
            )?
            .into_captured();
            application_objects.webdav_passwords = Default::default();
            let application_objects = SavedObjects::capture(application_objects, Some(&Auth))?;
            let payload = codec::encode(
                &application_objects,
                id,
                &chrono::Utc::now().to_rfc3339(),
                None,
                Some(&Auth),
            )?
            .into_bytes();
            if inventory(&self.d, &self.h)?.digest != inv.digest
                || target_digest(&self.d, &self.h, scope, choices)? != target
            {
                return Err(err("capture_stale"));
            }
            let scope_digest = hash(&application_scope)?;
            let context = PreparedContext {
                scope: scope_digest.clone(),
                target: target.clone(),
                selection: application_scope,
                source_selection: scope.clone(),
                choices: choices.clone(),
                bindings: bindings.iter().map(FrozenBinding::from).collect(),
            };
            let host = self.restore_host(&context)?;
            Ok(Prepared {
                host,
                payload,
                scope: scope_digest,
                target,
                selected,
                context,
            })
        })();
        match result {
            Ok(p) => {
                preview.can_confirm = true;
                Ok((preview, Some(p)))
            }
            Err(e) => {
                preview.reason_code = Some(e.code);
                Ok((preview, None))
            }
        }
    }
}
// The shared receive selector expects an already scoped snapshot. Narrow only
// the public model projection with v6's own source attachment/validation; do
// not discard the immutable original file or authorize a broader source.
fn narrow_received(objects: &SavedObjects, scope: &SelectionV2) -> ModelResult<SavedObjects> {
    let mut captured =
        codec::decode_document(&codec::encode_document(objects, Some(&Auth))?, Some(&Auth))?
            .into_captured();
    let mut sources = BTreeMap::new();
    for wanted in &scope.channels {
        let channel = captured
            .models
            .payload
            .policy
            .as_mut()
            .and_then(|p| {
                p.channels
                    .iter_mut()
                    .find(|c| c.id == wanted.portable_channel_id)
            })
            .ok_or_else(|| err("overlay_selection_missing"))?;
        if wanted.model_ids.is_none() && wanted.family_ids.is_none() {
            continue;
        }
        if wanted.model_ids.is_some() && wanted.family_ids.is_some() {
            return Err(err("overlay_scope_invalid"));
        }
        if let Some(ids) = &wanted.model_ids {
            let ids: BTreeSet<_> = ids.iter().cloned().collect();
            if ids.is_empty()
                || channel.family_scope.is_some()
                || !ids
                    .iter()
                    .all(|id| channel.models.iter().any(|m| &m.raw_model_id == id))
            {
                return Err(err("model_receive_scope_invalid"));
            }
            channel.models.retain(|m| ids.contains(&m.raw_model_id));
            channel.model_subset = Some(ids.into_iter().collect());
        }
        if let Some(ids) = &wanted.family_ids {
            let ids: BTreeSet<_> = ids.iter().cloned().collect();
            let available: BTreeSet<_> = channel
                .family_scope
                .clone()
                .unwrap_or_else(|| {
                    channel
                        .family_overrides
                        .keys()
                        .cloned()
                        .chain(channel.template_refs.iter().map(|r| r.template_id.clone()))
                        .chain(channel.models.iter().filter_map(|m| {
                            m.binding
                                .as_ref()
                                .map(|b| b.reference().template_id.clone())
                        }))
                        .collect()
                })
                .into_iter()
                .collect();
            if ids.is_empty()
                || channel.model_subset.is_some()
                || !ids.is_subset(&available)
                || wanted.include_connection
                || wanted.include_key
                || wanted.include_authorization
            {
                return Err(err("model_receive_scope_invalid"));
            }
            channel.models.retain(|m| {
                m.binding
                    .as_ref()
                    .is_some_and(|b| ids.contains(&b.reference().template_id))
            });
            channel
                .template_refs
                .retain(|r| ids.contains(&r.template_id));
            channel.family_overrides.retain(|id, _| ids.contains(id));
            channel.defaults = Default::default();
            channel.connection = None;
            channel.family_scope = Some(ids.into_iter().collect());
            // Family projection has no credential source direction.
            let keys = std::mem::take(&mut captured.channel_keys);
            for (id, value) in keys.into_entries() {
                if id != wanted.portable_channel_id {
                    captured.channel_keys.insert(id, value)?;
                }
            }
            captured
                .channel_authorizations
                .remove(&wanted.portable_channel_id);
        }
        let ids: BTreeSet<_> = channel
            .models
            .iter()
            .map(|m| m.raw_model_id.clone())
            .collect();
        channel.model_order.retain(|id| ids.contains(id));
        channel.naming.retain(|id, _| ids.contains(id));
        if let super::schema::ModelSelection::Whitelist(v) = &mut channel.selection {
            v.retain(|id| ids.contains(id));
        }
        if let Some(public) = channel.connection.as_mut().and_then(|c| c.provider.take()) {
            sources.insert(channel.id.clone(), public);
        }
    }
    let selected_ids: BTreeSet<_> = scope
        .channels
        .iter()
        .map(|c| c.portable_channel_id.clone())
        .collect();
    if let Some(p) = &mut captured.models.payload.policy {
        p.channels.retain(|c| selected_ids.contains(&c.id));
        p.channel_order.retain(|id| selected_ids.contains(id));
        if p.default_channel_id
            .as_ref()
            .is_some_and(|id| !selected_ids.contains(id))
        {
            p.default_channel_id = None;
        }
    }
    let keys = std::mem::take(&mut captured.channel_keys);
    for (id, value) in keys.into_entries() {
        if selected_ids.contains(&id) {
            captured.channel_keys.insert(id, value)?;
        }
    }
    captured
        .channel_authorizations
        .retain(|id, _| selected_ids.contains(id));
    captured.models.digest = hash(&captured.models.payload)?;
    captured.models =
        super::portable_v2::attach_public_sources_v2(&captured.models, &sources).map_err(model)?;
    SavedObjects::capture(captured, Some(&Auth))
}
struct FileSource {
    scope: String,
    selection: SelectionV2,
    choices: ImportBindingsV2,
}
impl SourceSnapshotProviderV2 for FileSource {
    fn current(&self, d: &Path, h: &Path, request: &CaptureRequestV2) -> ModelResult<SourceGuard> {
        Ok(SourceGuard {
            saved_source_digest: self.target(d, h, request)?,
            scope_digest: self.scope.clone(),
            mcp_sources_digest: hash(&self.choices.mcp_clients)?,
        })
    }
    fn root(&self, d: &Path, h: &Path, _r: &CaptureRequestV2) -> ModelResult<String> {
        hash(&(d, h))
    }
    fn target(&self, d: &Path, h: &Path, _r: &CaptureRequestV2) -> ModelResult<String> {
        target_digest(d, h, &self.selection, &self.choices)
    }
}
// Fixed host-owned business paths only; exclude journals/backups/temp files.
fn target_digest(
    d: &Path,
    h: &Path,
    s: &SelectionV2,
    choices: &ImportBindingsV2,
) -> ModelResult<String> {
    let inv = inventory(d, h)?;
    let mut paths = vec![(h.to_path_buf(), h.join("config.json"))];
    for p in [
        store_v2::LIBRARY_PATH,
        store_v2::POLICY_PATH,
        store_v2::LEDGER_PATH,
        store_v2::MAPPING_PATH,
        super::content_versions::SAVED_VERSIONS_PATH,
        "manager-state/model-v2-provider-identities.json",
        preferences::PREFERENCES_RELATIVE_PATH,
        projection::CONFIG_RELATIVE_PATH,
        portable_accounts::IMPORTED,
        portable_manager::CANDIDATES,
    ] {
        paths.push((d.to_path_buf(), d.join(p)));
    }
    paths.push((
        d.to_path_buf(),
        d.join("manager-state/mcp-definitions.json"),
    ));
    for name in &s.skill_names {
        if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\']) {
            return Err(err("file_asset_name_invalid"));
        }
        paths.push((d.to_path_buf(), discovery::skills_store_dir(d).join(name)));
    }
    // Skills use the existing source resolver. New incoming MCP definitions
    // need not exist locally: guard the selected client's fixed config path,
    // including absence, rather than invoking an export-only MCP resolver.
    for p in portable_assets::saved_source_paths(d, h, &s.skill_names, &[], &BTreeMap::new())? {
        let anchor = if p.starts_with(d) {
            d.to_path_buf()
        } else if p.starts_with(h) {
            h.to_path_buf()
        } else {
            p.parent()
                .ok_or_else(|| err("file_asset_source_invalid"))?
                .to_path_buf()
        };
        paths.push((anchor, p));
    }
    use crate::modules::extensions::{ClientTarget, CLIENT_IDS};
    for selected in &s.mcp_sources {
        let choice = choices.mcp_clients.get(&selected.name);
        if choice.is_some_and(|c| {
            c != "managed"
                && !CLIENT_IDS
                    .iter()
                    .any(|id| portable_assets::client_key(*id) == *c)
        }) {
            return Err(err("file_mcp_source_invalid"));
        }
        for id in CLIENT_IDS {
            if choice.is_some_and(|c| *c != portable_assets::client_key(id)) {
                continue;
            }
            let target = ClientTarget::user_target(id, h, true, true);
            paths.push((h.to_path_buf(), target.mcp_config_path));
        }
    }
    paths.sort();
    paths.dedup();
    let mut images = BTreeMap::new();
    let mut budget = 128 * 1024 * 1024;
    for (anchor, p) in paths {
        tree_image(&anchor, &p, &mut images, &mut budget)?;
    }
    hash(&(inv.digest, images))
}
fn tree_image(
    anchor: &Path,
    p: &Path,
    out: &mut BTreeMap<PathBuf, String>,
    budget: &mut usize,
) -> ModelResult<()> {
    safety::check_path(anchor, p, true)?;
    if out.len() >= 20000 {
        return Err(err("portable_limit"));
    }
    let meta = match std::fs::symlink_metadata(p) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            out.insert(p.into(), "absent".into());
            return Ok(());
        }
        Err(_) => return Err(err("file_target_io")),
    };
    safety::inspect(p)?;
    #[cfg(unix)]
    let mode = {
        use std::os::unix::fs::PermissionsExt;
        meta.permissions().mode() & 0o777
    };
    #[cfg(not(unix))]
    let mode = 0;
    if meta.is_dir() {
        out.insert(p.into(), format!("directory:{mode}"));
        let mut children = std::fs::read_dir(p)?
            .map(|e| e.map(|e| e.path()))
            .collect::<std::io::Result<Vec<_>>>()?;
        children.sort();
        for child in children {
            tree_image(anchor, &child, out, budget)?;
        }
    } else {
        let bytes = Zeroizing::new(safety::read_file(p, codec::DOCUMENT_LIMIT as u64)?);
        *budget = budget
            .checked_sub(bytes.len())
            .ok_or_else(|| err("portable_limit"))?;
        out.insert(p.into(), format!("{}:{mode}", schema_v2::digest(&bytes)));
    }
    Ok(())
}
#[cfg(test)]
#[path = "file_transfer_v2_tests.rs"]
mod tests;

/// Production password participant. Uses only explicit D-bound credential APIs.
/// Its checked private journal stores references/digests, never password values
/// or the file encryption passphrase. Interrupted writes resume with the same O.
pub struct SystemWebDavPasswordInstallerV2;
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PasswordJournal {
    version: u32,
    operation: String,
    root: PathBuf,
    plan: String,
    material: String,
    refs: BTreeMap<String, String>,
    backup: bool,
    before: Option<String>,
    backup_directory: Option<PathBuf>,
    backup_manifest: Option<crate::modules::backup::BackupManifest>,
    after: Option<String>,
    completed: bool,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CheckedPasswords {
    checksum: String,
    value: PasswordJournal,
}
fn password_path(d: &Path, id: &str) -> ModelResult<PathBuf> {
    if !uuid::Uuid::parse_str(id).is_ok_and(|u| u.to_string() == id && u.get_version_num() == 4) {
        return Err(err("file_password_operation_invalid"));
    }
    let p = d
        .join("manager-state/file-password-installs-v2")
        .join(format!("{id}.json"));
    safety::check_path(d, &p, true)?;
    Ok(p)
}
fn private_read(d: &Path, p: &Path) -> ModelResult<Option<Zeroizing<Vec<u8>>>> {
    safety::check_path(d, p, true)?;
    match std::fs::symlink_metadata(p) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(err("file_password_journal_io")),
        Ok(_) => {
            safety::inspect(p)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if std::fs::metadata(p)?.permissions().mode() & 0o777 != 0o600 {
                    return Err(err("file_password_journal_permissions"));
                }
            }
            Ok(Some(Zeroizing::new(safety::read_file(
                p,
                codec::DOCUMENT_LIMIT as u64,
            )?)))
        }
    }
}
fn password_load(d: &Path, id: &str) -> ModelResult<Option<PasswordJournal>> {
    let Some(bytes) = private_read(d, &password_path(d, id)?)? else {
        return Ok(None);
    };
    let doc: CheckedPasswords =
        serde_json::from_slice(&bytes).map_err(|_| err("file_password_journal_invalid"))?;
    if doc.checksum != hash(&doc.value)?
        || doc.value.version != 2
        || doc.value.operation != id
        || doc.value.root != d.canonicalize()?
    {
        return Err(err("file_password_journal_invalid"));
    }
    Ok(Some(doc.value))
}
fn password_save(d: &Path, j: PasswordJournal) -> ModelResult<()> {
    let path = password_path(d, &j.operation)?;
    let parent = path
        .parent()
        .ok_or_else(|| err("file_password_journal_io"))?;
    crate::infrastructure::platform::create_private_dir_all(parent, 0o700)?;
    safety::check_path(d, parent, false)?;
    crate::infrastructure::platform::set_mode(parent, 0o700)?;
    let bytes = serde_json::to_vec(&CheckedPasswords {
        checksum: hash(&j)?,
        value: j,
    })?;
    crate::infrastructure::atomic_write::atomic_write(&path, &bytes, 0o600)?;
    safety::sync_directories(d, parent)?;
    Ok(())
}
fn password_material(objects: &SavedObjects) -> ModelResult<String> {
    let p = objects.partitions();
    let secrets: BTreeMap<_, _> = p
        .webdav_passwords
        .ids()
        .map(|id| {
            (
                id,
                schema_v2::digest(p.webdav_passwords.get(id).unwrap_or_default().as_bytes()),
            )
        })
        .collect();
    hash(&("OCXD_LOCAL_FILE_PASSWORDS_V2", &p.manager.webdav, secrets))
}
fn verify_password_backup(d: &Path, j: &PasswordJournal) -> ModelResult<()> {
    use crate::modules::backup::{self, BackupRecord};
    if let (Some(directory), Some(manifest)) = (&j.backup_directory, &j.backup_manifest) {
        if !directory.starts_with(d.join("backups")) {
            return Err(err("file_password_backup_invalid"));
        }
        safety::check_path(d, directory, false)?;
        if !backup::verify_backup(&BackupRecord {
            directory: directory.clone(),
            manifest: manifest.clone(),
        })? {
            return Err(err("file_password_backup_invalid"));
        }
    } else if j.backup && j.before.is_some() {
        return Err(err("file_password_backup_missing"));
    }
    Ok(())
}
impl RootScopedWebDavPasswordInstallerV2 for SystemWebDavPasswordInstallerV2 {
    fn prepare_and_protect(
        &self,
        d: &Path,
        id: &str,
        plan: &str,
        objects: &SavedObjects,
        backup: bool,
    ) -> ModelResult<()> {
        use crate::modules::backup::{self, BackupAction};
        let path = password_path(d, id)?;
        let _guard = crate::infrastructure::locking::TargetFileLock::lock(&path)?;
        let material = password_material(objects)?;
        if let Some(j) = password_load(d, id)? {
            if j.plan != plan || j.material != material || j.backup != backup {
                return Err(err("file_password_operation_collision"));
            }
            if !j.completed {
                verify_password_backup(d, &j)?;
            }
            return Ok(());
        }
        let before = private_read(d, &d.join(portable_manager::CANDIDATES))?;
        let mut refs = BTreeMap::new();
        for id in objects.partitions().webdav_passwords.ids() {
            if !objects
                .partitions()
                .manager
                .webdav
                .iter()
                .any(|e| e.id == id)
            {
                return Err(err("credential_scope"));
            }
            refs.insert(id.to_owned(), crate::infrastructure::keychain::new_ref_id());
        }
        let record = if backup {
            before
                .as_ref()
                .map(|bytes| {
                    backup::backup_file(
                        d,
                        BackupAction::Import,
                        &d.join(portable_manager::CANDIDATES),
                        bytes,
                        chrono::Utc::now(),
                        Some("文件接收候选密码采用前保护".into()),
                    )
                })
                .transpose()?
        } else {
            None
        };
        let j = PasswordJournal {
            version: 2,
            operation: id.into(),
            root: d.canonicalize()?,
            plan: plan.into(),
            material,
            refs,
            backup,
            before: before.as_ref().map(|b| schema_v2::digest(b)),
            backup_directory: record.as_ref().map(|r| r.directory.clone()),
            backup_manifest: record.map(|r| r.manifest),
            after: None,
            completed: false,
        };
        verify_password_backup(d, &j)?;
        password_save(d, j)
    }
    fn apply_once(
        &self,
        d: &Path,
        id: &str,
        plan: &str,
        objects: &SavedObjects,
    ) -> ModelResult<()> {
        use crate::infrastructure::{credentials, keychain::WEBDAV_CREDENTIAL_PURPOSE};
        let path = password_path(d, id)?;
        let _guard = crate::infrastructure::locking::TargetFileLock::lock(&path)?;
        let mut j = password_load(d, id)?.ok_or_else(|| err("file_password_plan_missing"))?;
        if j.plan != plan || j.material != password_material(objects)? {
            return Err(err("file_password_operation_collision"));
        }
        if j.completed {
            return Ok(());
        }
        verify_password_backup(d, &j)?;
        let candidate_path = d.join(portable_manager::CANDIDATES);
        let _candidate_guard =
            crate::infrastructure::locking::TargetFileLock::lock(&candidate_path)?;
        // This is the exact after-image receipted by the local manager, not a
        // fresh candidate inventory, active endpoint config or guessed label.
        let mut applied = portable_manager::applied_files(d, id)?;
        let original = Zeroizing::new(
            applied
                .remove(portable_manager::CANDIDATES)
                .ok_or_else(|| err("file_password_manager_not_applied"))?,
        );
        let mut candidates: Vec<portable_manager::Candidate> = serde_json::from_slice(&original)
            .map_err(|_| err("file_password_candidate_invalid"))?;
        for (endpoint, reference) in &j.refs {
            let source = objects
                .partitions()
                .manager
                .webdav
                .iter()
                .find(|e| &e.id == endpoint)
                .ok_or_else(|| err("credential_scope"))?;
            let matching = candidates
                .iter_mut()
                .filter(|c| c.endpoint == *source)
                .collect::<Vec<_>>();
            // A duplicate exact endpoint requires operation-specific before IDs;
            // use the frozen before-image below to exclude pre-existing rows.
            let before_ids = manager_before_candidate_ids(d, id)?;
            let mut fresh = matching.into_iter().filter(|c| !before_ids.contains(&c.id));
            let target = fresh
                .next()
                .ok_or_else(|| err("file_password_candidate_missing"))?;
            if fresh.next().is_some() {
                return Err(err("file_password_candidate_ambiguous"));
            }
            target.credential_ref = Some(reference.clone());
        }
        let desired = serde_json::to_vec(&candidates)?;
        let after = schema_v2::digest(&desired);
        if j.after.as_ref().is_some_and(|v| v != &after) {
            return Err(err("file_password_operation_collision"));
        }
        let current = private_read(d, &candidate_path)?
            .ok_or_else(|| err("file_password_candidate_changed"))?;
        if current.as_slice() != original.as_slice() && schema_v2::digest(&current) != after {
            return Err(err("file_password_candidate_changed"));
        }
        if j.after.is_none() {
            j.after = Some(after.clone());
            password_save(d, j)?;
            j = password_load(d, id)?.ok_or_else(|| err("file_password_plan_missing"))?;
        }
        for (endpoint, reference) in &j.refs {
            let value = objects
                .partitions()
                .webdav_passwords
                .get(endpoint)
                .ok_or_else(|| err("credential_scope"))?;
            match credentials::load_optional_at(d, WEBDAV_CREDENTIAL_PURPOSE, reference)
                .map_err(|_| err("file_password_store_io"))?
            {
                Some(existing) if existing.as_str() != value => {
                    return Err(err("file_password_reference_collision"))
                }
                Some(_) => (),
                None => credentials::store_at(d, WEBDAV_CREDENTIAL_PURPOSE, reference, value)
                    .map_err(|_| err("file_password_store_io"))?,
            }
        }
        if schema_v2::digest(&current) != after {
            crate::infrastructure::atomic_write::atomic_write(&candidate_path, &desired, 0o600)?;
            safety::sync_directories(
                d,
                candidate_path
                    .parent()
                    .ok_or_else(|| err("file_password_candidate_invalid"))?,
            )?;
        }
        let actual = private_read(d, &candidate_path)?
            .ok_or_else(|| err("file_password_candidate_changed"))?;
        if schema_v2::digest(&actual) != after {
            return Err(err("file_password_candidate_changed"));
        }
        j.completed = true;
        password_save(d, j)
    }
    fn receipted(&self, d: &Path, id: &str, plan: &str) -> ModelResult<bool> {
        Ok(password_load(d, id)?.is_some_and(|j| j.plan == plan && j.completed))
    }
}
fn manager_before_candidate_ids(d: &Path, id: &str) -> ModelResult<BTreeSet<String>> {
    // Read only the fixed manager participant path for this canonical operation.
    let path = d
        .join("manager-state/portable-manager-plans")
        .join(format!("{id}.json"));
    let bytes = private_read(d, &path)?.ok_or_else(|| err("file_password_manager_not_applied"))?;
    let value: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| err("file_password_manager_invalid"))?;
    if value["id"].as_str() != Some(id) || value["completed"] != true {
        return Err(err("file_password_manager_not_applied"));
    }
    let rows = value["files"]
        .as_array()
        .ok_or_else(|| err("file_password_manager_invalid"))?;
    let row = rows
        .iter()
        .find(|r| r["path"] == portable_manager::CANDIDATES)
        .ok_or_else(|| err("file_password_candidate_missing"))?;
    if row["before"].is_null() {
        return Ok(BTreeSet::new());
    }
    let bytes: Vec<u8> = serde_json::from_value(row["before"].clone())
        .map_err(|_| err("file_password_manager_invalid"))?;
    let candidates: Vec<portable_manager::Candidate> =
        serde_json::from_slice(&bytes).map_err(|_| err("file_password_candidate_invalid"))?;
    Ok(candidates.into_iter().map(|c| c.id).collect())
}

/// Namespace support must use the same receive model transaction, never a
/// follow-up namespace write or another operation. L0 injects the shared engine
/// adapter when its explicit adoption API is available.
pub trait NamespaceReceiveV2: Send + Sync {
    fn plan(
        &self,
        library: &LibraryV2,
        policy: &PolicyV2,
        objects: &SavedObjects,
        scope: &SelectionV2,
        bindings: &[ReceiveBindingV2],
        adopt_ids: &[String],
    ) -> ModelResult<receive_models_v2::ReceiveModelsPlanV2>;
    fn consumer(
        &self,
        d: &Path,
        h: &Path,
        bindings: Vec<ReceiveBindingV2>,
        adopt_ids: &[String],
    ) -> ModelResult<Arc<dyn ModelApplyV2>>;
}
fn validate_namespace_choice(
    objects: &SavedObjects,
    scope: &SelectionV2,
    choices: &ImportBindingsV2,
) -> ModelResult<()> {
    let mut seen = BTreeSet::new();
    for id in &choices.adopt_namespace_channel_ids {
        let selected = scope
            .channels
            .iter()
            .find(|s| &s.portable_channel_id == id)
            .ok_or_else(|| err("file_namespace_scope_invalid"))?;
        let source = objects
            .models()
            .payload
            .policy
            .iter()
            .flat_map(|p| &p.channels)
            .find(|c| &c.id == id)
            .ok_or_else(|| err("file_namespace_scope_invalid"))?;
        if !seen.insert(id)
            || selected.model_ids.is_some()
            || selected.family_ids.is_some()
            || source.namespace.is_none()
        {
            return Err(err("file_namespace_scope_invalid"));
        }
    }
    for c in objects
        .models()
        .payload
        .policy
        .iter()
        .flat_map(|p| &p.channels)
    {
        if c.namespace.is_some() && !seen.contains(&c.id) {
            return Err(err("model_receive_namespace_adoption_required"));
        }
    }
    Ok(())
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FrozenBinding {
    portable: String,
    local: String,
    provider: String,
    new: bool,
}
impl From<&ReceiveBindingV2> for FrozenBinding {
    fn from(b: &ReceiveBindingV2) -> Self {
        Self {
            portable: b.portable_channel_id.clone(),
            local: b.local_channel_id.clone(),
            provider: b.provider_id.clone(),
            new: b.mode == ReceiveBindingModeV2::New,
        }
    }
}
impl FrozenBinding {
    fn binding(&self) -> ReceiveBindingV2 {
        ReceiveBindingV2 {
            portable_channel_id: self.portable.clone(),
            local_channel_id: self.local.clone(),
            provider_id: self.provider.clone(),
            mode: if self.new {
                ReceiveBindingModeV2::New
            } else {
                ReceiveBindingModeV2::Existing
            },
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct PreparedContext {
    scope: String,
    target: String,
    selection: SelectionV2,
    source_selection: SelectionV2,
    choices: ImportBindingsV2,
    bindings: Vec<FrozenBinding>,
}
impl FileTransferOwnerV2 {
    fn restore_host(&self, c: &PreparedContext) -> ModelResult<LocalApplicationV2> {
        local(&c.selection)?;
        local(&c.source_selection)?;
        if hash(&c.selection)? != c.scope || !schema_v2::valid_digest(&c.target) {
            return Err(err("file_recovery_context_invalid"));
        }
        let mut expected = c.source_selection.clone();
        expected.manager.webdav_password_ids.clear();
        if expected != c.selection {
            return Err(err("file_recovery_context_invalid"));
        }
        let bindings = c
            .bindings
            .iter()
            .map(FrozenBinding::binding)
            .collect::<Vec<_>>();
        let models = if c.choices.adopt_namespace_channel_ids.is_empty() {
            Arc::new(ReceiveFileHostV2 {
                root: self.d.clone(),
                home: self.h.clone(),
                bindings,
                trusted_mapping_write: None,
            }) as Arc<dyn ModelApplyV2>
        } else {
            self.namespace
                .as_ref()
                .ok_or_else(|| err("file_namespace_receiver_not_connected"))?
                .consumer(
                    &self.d,
                    &self.h,
                    bindings,
                    &c.choices.adopt_namespace_channel_ids,
                )?
        };
        LocalApplicationV2::new(
            self.d.clone(),
            self.h.clone(),
            c.selection.clone(),
            CaptureRequestV2 {
                selection: c.selection.clone(),
                mcp_clients: c.choices.mcp_clients.clone(),
                assets_reviewed: c.choices.assets_reviewed,
            },
            c.scope.clone(),
            Arc::new(FileSource {
                scope: c.scope.clone(),
                selection: c.source_selection.clone(),
                choices: c.choices.clone(),
            }),
            models,
            None,
        )
        .map_err(|_| err("file_local_context_invalid"))
    }
}

/// Recovery key boundary only. No file passphrase is stored, returned to IPC or
/// used as a device key. Tests inject fixed in-memory keys; never use keyring.
pub trait FileRecoveryKeysV2: Send + Sync {
    fn key(&self, d: &Path, h: &Path, create: bool) -> ModelResult<Zeroizing<[u8; 32]>>;
}
pub struct SystemFileRecoveryKeysV2;
impl FileRecoveryKeysV2 for SystemFileRecoveryKeysV2 {
    fn key(&self, d: &Path, h: &Path, create: bool) -> ModelResult<Zeroizing<[u8; 32]>> {
        use base64::Engine;
        use rand::RngCore;
        let account = hash(&("OCXD_FILE_RECOVERY_KEY_V2", d, h))?;
        let lock = d.join("manager-state/file-transfer-v2-key");
        safety::check_path(d, &lock, true)?;
        let _guard = crate::infrastructure::locking::TargetFileLock::lock(&lock)?;
        let entry = keyring::Entry::new("opencodex.file-transfer-recovery.v2", &account)
            .map_err(|_| err("file_recovery_key_unavailable"))?;
        let encoded = match entry.get_password() {
            Ok(value) => Zeroizing::new(value),
            Err(keyring::Error::NoEntry) if create => {
                let dir = d.join(RECOVERY_DIR);
                safety::check_path(d, &dir, true)?;
                if dir.exists()
                    && std::fs::read_dir(&dir)?.any(|p| {
                        p.is_ok_and(|p| p.path().extension().is_some_and(|e| e == "sealed"))
                    })
                {
                    return Err(err("file_recovery_key_missing"));
                }
                let mut key = Zeroizing::new([0; 32]);
                rand::rng().fill_bytes(key.as_mut());
                let value =
                    Zeroizing::new(base64::engine::general_purpose::STANDARD.encode(key.as_ref()));
                entry
                    .set_password(&value)
                    .map_err(|_| err("file_recovery_key_unavailable"))?;
                let read = Zeroizing::new(
                    entry
                        .get_password()
                        .map_err(|_| err("file_recovery_key_unavailable"))?,
                );
                if read.as_str() != value.as_str() {
                    return Err(err("file_recovery_key_unavailable"));
                }
                value
            }
            Err(keyring::Error::NoEntry) => return Err(err("file_recovery_key_missing")),
            Err(_) => return Err(err("file_recovery_key_unavailable")),
        };
        let bytes = Zeroizing::new(
            base64::engine::general_purpose::STANDARD
                .decode(encoded.as_bytes())
                .map_err(|_| err("file_recovery_key_invalid"))?,
        );
        if bytes.len() != 32 {
            return Err(err("file_recovery_key_invalid"));
        }
        let mut key = Zeroizing::new([0; 32]);
        key.copy_from_slice(&bytes);
        Ok(key)
    }
}
const RECOVERY_DIR: &str = "manager-state/file-transfer-v2";
const RECOVERY_PLAIN_LIMIT: usize = 256 * 1024 * 1024;
const RECOVERY_FILE_LIMIT: u64 = 384 * 1024 * 1024;
struct RecoveryBlob(Zeroizing<Vec<u8>>);
impl Serialize for RecoveryBlob {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use base64::Engine;
        let encoded = Zeroizing::new(base64::engine::general_purpose::STANDARD.encode(&*self.0));
        s.serialize_str(&encoded)
    }
}
impl<'de> Deserialize<'de> for RecoveryBlob {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        use base64::Engine;
        let text = Zeroizing::new(String::deserialize(d)?);
        let bytes = Zeroizing::new(
            base64::engine::general_purpose::STANDARD
                .decode(text.as_bytes())
                .map_err(|_| serde::de::Error::custom("invalid protected artifact"))?,
        );
        if bytes.len() > codec::FILE_LIMIT {
            return Err(serde::de::Error::custom("artifact limit"));
        }
        Ok(Self(bytes))
    }
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FrozenApplication {
    context: PreparedContext,
    payload: RecoveryBlob,
}
#[derive(Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
enum FrozenOperation {
    Export {
        plan: ExportPlanV2,
        bytes: RecoveryBlob,
        attempted: bool,
        receipt: Option<FileTransferReceiptV2>,
    },
    Import {
        original: RecoveryBlob,
        objects: RecoveryBlob,
        preview: Box<ImportPreviewV2>,
        choices: ImportBindingsV2,
        application: Option<FrozenApplication>,
        attempted: bool,
        backup: Option<bool>,
        receipt: Option<FileTransferReceiptV2>,
    },
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProtectedOperation {
    version: u32,
    id: String,
    nonce: String,
    ciphertext: String,
}
fn recovery_path(d: &Path, id: &str) -> ModelResult<PathBuf> {
    if !uuid::Uuid::parse_str(id).is_ok_and(|u| u.to_string() == id && u.get_version_num() == 4) {
        return Err(err("file_operation_id_invalid"));
    }
    let p = d.join(RECOVERY_DIR).join(format!("{id}.sealed"));
    safety::check_path(d, &p, true)?;
    Ok(p)
}
fn recovery_aad(d: &Path, h: &Path, id: &str) -> ModelResult<Vec<u8>> {
    schema_v2::canonical_bytes(&("OCXD_FILE_RECOVERY_V2", 2, d, h, id)).map_err(model)
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoverableFileOperationV2 {
    pub operation_id: String,
    pub kind: String,
    pub attempted: bool,
    pub completed: bool,
}
impl FileTransferOwnerV2 {
    fn save_recovery(
        &self,
        id: &str,
        expected: Option<&str>,
        value: &FrozenOperation,
    ) -> ModelResult<String> {
        use aes_gcm::{
            aead::{Aead, KeyInit, Payload},
            Aes256Gcm, Nonce,
        };
        use base64::Engine;
        use rand::RngCore;
        let path = recovery_path(&self.d, id)?;
        let parent = path.parent().ok_or_else(|| err("file_recovery_io"))?;
        crate::infrastructure::platform::create_private_dir_all(parent, 0o700)?;
        safety::check_path(&self.d, parent, false)?;
        crate::infrastructure::platform::set_mode(parent, 0o700)?;
        let _guard = crate::infrastructure::locking::TargetFileLock::lock(&path)?;
        let current = read_recovery_file(&self.d, &path)?;
        if current.as_ref().map(|b| schema_v2::digest(b)).as_deref() != expected {
            return Err(err("file_recovery_changed"));
        }
        let key = self
            .recovery_keys
            .key(&self.d, &self.h, expected.is_none())?;
        let plain = Zeroizing::new(serde_json::to_vec(value)?);
        if plain.len() > RECOVERY_PLAIN_LIMIT {
            return Err(err("file_recovery_limit"));
        }
        let mut nonce = [0; 12];
        rand::rng().fill_bytes(&mut nonce);
        let cipher = Aes256Gcm::new_from_slice(key.as_ref())
            .map_err(|_| err("file_recovery_key_invalid"))?;
        let ciphertext = cipher
            .encrypt(
                Nonce::from_slice(&nonce),
                Payload {
                    msg: &plain,
                    aad: &recovery_aad(&self.d, &self.h, id)?,
                },
            )
            .map_err(|_| err("file_recovery_protection_failed"))?;
        let bytes = serde_json::to_vec(&ProtectedOperation {
            version: 2,
            id: id.into(),
            nonce: base64::engine::general_purpose::STANDARD.encode(nonce),
            ciphertext: base64::engine::general_purpose::STANDARD.encode(ciphertext),
        })?;
        if bytes.len() as u64 > RECOVERY_FILE_LIMIT {
            return Err(err("file_recovery_limit"));
        }
        crate::infrastructure::atomic_write::atomic_write(&path, &bytes, 0o600)?;
        safety::sync_directories(&self.d, parent)?;
        Ok(schema_v2::digest(&bytes))
    }
    fn load_recovery(&self, id: &str) -> ModelResult<(FrozenOperation, String)> {
        use aes_gcm::{
            aead::{Aead, KeyInit, Payload},
            Aes256Gcm, Nonce,
        };
        use base64::Engine;
        let path = recovery_path(&self.d, id)?;
        let bytes =
            read_recovery_file(&self.d, &path)?.ok_or_else(|| err("file_operation_missing"))?;
        let doc: ProtectedOperation =
            serde_json::from_slice(&bytes).map_err(|_| err("file_recovery_invalid"))?;
        if doc.version != 2 || doc.id != id {
            return Err(err("file_recovery_invalid"));
        }
        let nonce = base64::engine::general_purpose::STANDARD
            .decode(doc.nonce)
            .map_err(|_| err("file_recovery_invalid"))?;
        let ciphertext = base64::engine::general_purpose::STANDARD
            .decode(doc.ciphertext)
            .map_err(|_| err("file_recovery_invalid"))?;
        if nonce.len() != 12 || ciphertext.len() > RECOVERY_PLAIN_LIMIT + 16 {
            return Err(err("file_recovery_limit"));
        }
        let key = self.recovery_keys.key(&self.d, &self.h, false)?;
        let cipher = Aes256Gcm::new_from_slice(key.as_ref())
            .map_err(|_| err("file_recovery_key_invalid"))?;
        let plain = Zeroizing::new(
            cipher
                .decrypt(
                    Nonce::from_slice(&nonce),
                    Payload {
                        msg: &ciphertext,
                        aad: &recovery_aad(&self.d, &self.h, id)?,
                    },
                )
                .map_err(|_| err("file_recovery_authentication"))?,
        );
        let value = serde_json::from_slice(&plain).map_err(|_| err("file_recovery_invalid"))?;
        Ok((value, schema_v2::digest(&bytes)))
    }
    fn persist_export(&mut self, id: &str) -> ModelResult<()> {
        let op = self
            .exports
            .get(id)
            .ok_or_else(|| err("file_operation_missing"))?;
        let frozen = FrozenOperation::Export {
            plan: op.plan.clone(),
            bytes: RecoveryBlob(Zeroizing::new(op.bytes.to_vec())),
            attempted: op.attempted,
            receipt: op.receipt.clone(),
        };
        let revision = self.save_recovery(id, op.revision.as_deref(), &frozen)?;
        self.exports
            .get_mut(id)
            .ok_or_else(|| err("file_operation_missing"))?
            .revision = Some(revision);
        Ok(())
    }
    fn persist_import(&mut self, id: &str) -> ModelResult<()> {
        let op = self
            .imports
            .get(id)
            .ok_or_else(|| err("file_operation_missing"))?;
        let frozen = FrozenOperation::Import {
            original: RecoveryBlob(Zeroizing::new(op.original.to_vec())),
            objects: RecoveryBlob(codec::encode_document(&op.objects, Some(&Auth))?),
            preview: Box::new(op.preview.clone()),
            choices: op.choices.clone(),
            application: op.application.as_ref().map(|p| FrozenApplication {
                context: p.context.clone(),
                payload: RecoveryBlob(Zeroizing::new(p.payload.to_vec())),
            }),
            attempted: op.attempted,
            backup: op.backup,
            receipt: op.receipt.clone(),
        };
        let revision = self.save_recovery(id, op.revision.as_deref(), &frozen)?;
        self.imports
            .get_mut(id)
            .ok_or_else(|| err("file_operation_missing"))?
            .revision = Some(revision);
        Ok(())
    }
    fn discard_recovery(&self, id: &str, expected: Option<&str>) -> ModelResult<()> {
        let path = recovery_path(&self.d, id)?;
        let _guard = crate::infrastructure::locking::TargetFileLock::lock(&path)?;
        let bytes =
            read_recovery_file(&self.d, &path)?.ok_or_else(|| err("file_operation_missing"))?;
        if Some(schema_v2::digest(&bytes)).as_deref() != expected {
            return Err(err("file_recovery_changed"));
        }
        std::fs::remove_file(&path)?;
        safety::sync_directories(
            &self.d,
            path.parent().ok_or_else(|| err("file_recovery_io"))?,
        )?;
        Ok(())
    }
    fn refresh_import(&mut self, id: &str) -> ModelResult<()> {
        let path = recovery_path(&self.d, id)?;
        let bytes =
            read_recovery_file(&self.d, &path)?.ok_or_else(|| err("file_operation_missing"))?;
        if self.imports.get(id).and_then(|o| o.revision.as_deref())
            != Some(schema_v2::digest(&bytes)).as_deref()
        {
            self.reopen_import(id)?;
        }
        Ok(())
    }
    fn ensure_export(&mut self, id: &str) -> ModelResult<()> {
        if !self.exports.contains_key(id) {
            self.reopen_export(id)?;
        }
        Ok(())
    }
    fn ensure_import(&mut self, id: &str) -> ModelResult<()> {
        if !self.imports.contains_key(id) {
            self.reopen_import(id)?;
        }
        Ok(())
    }
    pub fn reopen_export(&mut self, id: &str) -> ModelResult<ExportPlanV2> {
        let (frozen, revision) = self.load_recovery(id)?;
        let FrozenOperation::Export {
            plan,
            bytes,
            attempted,
            receipt,
        } = frozen
        else {
            return Err(err("file_operation_kind_mismatch"));
        };
        if plan.id != id
            || plan.digest != schema_v2::digest(&bytes.0)
            || plan.length != bytes.0.len()
            || plan.schema != codec::supported_schema()
            || receipt.as_ref().is_some_and(|r| {
                r.operation_id != id
                    || r.plan_digest != plan.digest
                    || r.file_digest != plan.digest
                    || r.length != plan.length
            })
        {
            return Err(err("file_recovery_invalid"));
        }
        self.exports.insert(
            id.into(),
            Export {
                plan: plan.clone(),
                bytes: bytes.0,
                attempted,
                receipt,
                revision: Some(revision),
            },
        );
        Ok(plan)
    }
    pub fn reopen_import(&mut self, id: &str) -> ModelResult<ImportPreviewV2> {
        let (frozen, revision) = self.load_recovery(id)?;
        let FrozenOperation::Import {
            original,
            objects,
            preview,
            choices,
            application,
            attempted,
            backup,
            receipt,
        } = frozen
        else {
            return Err(err("file_operation_kind_mismatch"));
        };
        let preview = *preview;
        if preview.operation_id != id
            || receipt.as_ref().is_some_and(|r| {
                r.operation_id != id
                    || r.plan_digest != preview.plan_digest
                    || r.file_digest != schema_v2::digest(&original.0)
                    || r.length != original.0.len()
            })
            || attempted != backup.is_some()
        {
            return Err(err("file_recovery_invalid"));
        }
        let objects = codec::decode_document(&objects.0, Some(&Auth))?;
        let application = if receipt.is_some() {
            None
        } else {
            application
                .map(|f| -> ModelResult<Prepared> {
                    if f.context.source_selection != preview.selected_scope
                        || hash(&f.context.choices)? != hash(&choices)?
                    {
                        return Err(err("file_recovery_context_invalid"));
                    }
                    let selected = sync_local_v2::select_received_objects_v2(
                        narrow_received(&objects, &preview.selected_scope)?,
                        &preview.selected_scope,
                    )?;
                    validate_namespace_choice(&selected, &preview.selected_scope, &choices)?;
                    let host = self.restore_host(&f.context)?;
                    Ok(Prepared {
                        host,
                        payload: f.payload.0,
                        scope: f.context.scope.clone(),
                        target: f.context.target.clone(),
                        selected,
                        context: f.context,
                    })
                })
                .transpose()?
        };
        self.imports.insert(
            id.into(),
            Import {
                original: original.0,
                passphrase: None,
                objects,
                choices,
                revision: Some(revision),
                preview: preview.clone(),
                application,
                attempted,
                backup,
                receipt,
            },
        );
        Ok(preview)
    }
    pub fn recoverable_operations(&self) -> ModelResult<Vec<RecoverableFileOperationV2>> {
        let dir = self.d.join(RECOVERY_DIR);
        safety::check_path(&self.d, &dir, true)?;
        if !dir.exists() {
            return Ok(vec![]);
        }
        let mut ids = vec![];
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension().is_none_or(|e| e != "sealed") {
                continue;
            }
            let id = path
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or_else(|| err("file_recovery_invalid"))?
                .to_owned();
            ids.push(id);
            if ids.len() > 256 {
                return Err(err("file_recovery_limit"));
            }
        }
        ids.sort();
        let mut out = vec![];
        for id in ids {
            let (op, _) = self.load_recovery(&id)?;
            let (kind, attempted, completed) = match op {
                FrozenOperation::Export {
                    attempted, receipt, ..
                } => ("export", attempted, receipt.is_some()),
                FrozenOperation::Import {
                    attempted, receipt, ..
                } => ("import", attempted, receipt.is_some()),
            };
            out.push(RecoverableFileOperationV2 {
                operation_id: id,
                kind: kind.into(),
                attempted,
                completed,
            });
        }
        Ok(out)
    }
}
fn read_recovery_file(d: &Path, path: &Path) -> ModelResult<Option<Zeroizing<Vec<u8>>>> {
    safety::check_path(d, path, true)?;
    match std::fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(err("file_recovery_io")),
        Ok(_) => {
            safety::inspect(path)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if std::fs::metadata(path)?.permissions().mode() & 0o777 != 0o600 {
                    return Err(err("file_recovery_permissions"));
                }
            }
            Ok(Some(Zeroizing::new(safety::read_file(
                path,
                RECOVERY_FILE_LIMIT,
            )?)))
        }
    }
}

mod summary_wire {
    use super::*;
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    pub(super) struct Wire {
        schema: PayloadSchemaRef,
        template_ids: Vec<String>,
        channel_ids: Vec<String>,
        account_count: usize,
        skill_count: usize,
        mcp_count: usize,
        has_preferences: bool,
        has_extensions: bool,
        webdav_count: usize,
        channel_key_count: usize,
        channel_authorization_count: usize,
        webdav_password_count: usize,
        extension_secret_count: usize,
        content_version_count: usize,
    }
    impl Wire {
        pub(super) fn summary(self) -> PublicSummary {
            PublicSummary {
                schema: self.schema,
                template_ids: self.template_ids,
                channel_ids: self.channel_ids,
                account_count: self.account_count,
                skill_count: self.skill_count,
                mcp_count: self.mcp_count,
                has_preferences: self.has_preferences,
                has_extensions: self.has_extensions,
                webdav_count: self.webdav_count,
                channel_key_count: self.channel_key_count,
                channel_authorization_count: self.channel_authorization_count,
                webdav_password_count: self.webdav_password_count,
                extension_secret_count: self.extension_secret_count,
                content_version_count: self.content_version_count,
            }
        }
    }
    pub fn serialize<S: serde::Serializer>(v: &PublicSummary, s: S) -> Result<S::Ok, S::Error> {
        v.serialize(s)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<PublicSummary, D::Error> {
        Wire::deserialize(d).map(Wire::summary)
    }
}
mod optional_summary_wire {
    use super::*;
    pub fn serialize<S: serde::Serializer>(
        v: &Option<PublicSummary>,
        s: S,
    ) -> Result<S::Ok, S::Error> {
        v.serialize(s)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        d: D,
    ) -> Result<Option<PublicSummary>, D::Error> {
        Option::<summary_wire::Wire>::deserialize(d).map(|v| v.map(summary_wire::Wire::summary))
    }
}
