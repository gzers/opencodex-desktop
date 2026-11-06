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
    sync_local_v2::{self, LocalApplicationV2, SourceSnapshotProviderV2},
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
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportPlanV2 {
    pub id: String,
    pub scope: SelectionV2,
    pub summary: PublicSummary,
    pub encrypted: bool,
    pub filename: String,
    pub schema: PayloadSchemaRef,
    pub digest: String,
    pub length: usize,
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
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
}
#[derive(Clone, Debug, Serialize)]
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
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportDifferencesV2 {
    pub changed_channel_ids: Vec<String>,
    pub changed_template_refs: Vec<schema_v2::RefV2>,
    pub selected_summary: Option<PublicSummary>,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreviewV2 {
    pub operation_id: String,
    pub plan_digest: String,
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
}
/// Pending plans currently live for this owner's lifetime. L0 must retain the
/// owner across dialog cancellation/errors. Participant journals are durable;
/// process-restart reopening of these in-memory file plans is not implemented.
pub struct FileTransferOwnerV2 {
    d: PathBuf,
    h: PathBuf,
    credentials: Arc<Credentials>,
    exports: BTreeMap<String, Export>,
    imports: BTreeMap<String, Import>,
    draft: Option<Arc<dyn DraftFileCaptureV2>>,
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
        local(&request.capture.selection)?;
        let helper = self
            .draft
            .as_ref()
            .ok_or_else(|| err("file_draft_capture_not_connected"))?;
        let captured = helper.capture(
            &self.d,
            &self.h,
            &request.expected,
            &request.capture,
            self.credentials.as_ref(),
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
            },
        );
        Ok(plan)
    }
    /// Host-only immutable bridge to nativeSaveDialog exporter. Call only after
    /// explicit user destination choice, before writing. Marks unknown output.
    pub fn bytes(&mut self, id: &str) -> ModelResult<&[u8]> {
        let e = self
            .exports
            .get_mut(id)
            .ok_or_else(|| err("file_operation_missing"))?;
        e.attempted = true;
        Ok(&e.bytes)
    }
    /// L0 must supply actual written/read-back digest, not the planned digest.
    pub fn finish_export(
        &mut self,
        id: &str,
        actual_written_digest: &str,
    ) -> ModelResult<FileTransferReceiptV2> {
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
        Ok(receipt)
    }
    /// false: output may exist; retain exact bytes/id for retry.
    pub fn cancel_export(&mut self, id: &str) -> ModelResult<bool> {
        let e = self
            .exports
            .get(id)
            .ok_or_else(|| err("file_operation_missing"))?;
        if e.attempted {
            return Ok(false);
        }
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
            id,
            Import {
                original: bytes,
                passphrase,
                objects: decoded.objects,
                preview: preview.clone(),
                application,
                attempted: false,
                backup: None,
                receipt: None,
            },
        );
        Ok(preview)
    }
    pub fn reselect(
        &mut self,
        id: &str,
        expected_plan_digest: &str,
        scope: SelectionV2,
        bindings: ImportBindingsV2,
    ) -> ModelResult<ImportPreviewV2> {
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
        Ok(preview)
    }
    pub fn cancel_import(&mut self, id: &str) -> ModelResult<bool> {
        let op = self
            .imports
            .get(id)
            .ok_or_else(|| err("file_operation_missing"))?;
        if op.attempted {
            return Ok(false);
        }
        self.imports.remove(id);
        Ok(true)
    }
    pub fn confirm_import(
        &mut self,
        id: &str,
        expected_plan_digest: &str,
        backup_default_true: Option<bool>,
    ) -> ModelResult<FileTransferReceiptV2> {
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
        let p = op
            .application
            .as_ref()
            .ok_or_else(|| err("file_selection_required"))?;
        op.attempted = true;
        op.backup = Some(backup);
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
        op.receipt = Some(r.clone());
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
        out.push(object("account", &a.id));
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
        password: Option<&Zeroizing<String>>,
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
            let planned = receive_models_v2::plan(
                &inv.library,
                &inv.policy,
                selected.models(),
                scope,
                &bindings,
            )
            .map_err(model)?;
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
                password.map(|p| p.as_str()),
                Some(&Auth),
            )?
            .into_bytes();
            if inventory(&self.d, &self.h)?.digest != inv.digest
                || target_digest(&self.d, &self.h, scope, choices)? != target
            {
                return Err(err("capture_stale"));
            }
            let scope_digest = hash(&application_scope)?;
            let capture = CaptureRequestV2 {
                selection: application_scope.clone(),
                mcp_clients: choices.mcp_clients.clone(),
                assets_reviewed: choices.assets_reviewed,
            };
            let source = Arc::new(FileSource {
                scope: scope_digest.clone(),
                selection: scope.clone(),
                choices: choices.clone(),
            });
            let host = LocalApplicationV2::new(
                self.d.clone(),
                self.h.clone(),
                application_scope,
                capture,
                scope_digest.clone(),
                source,
                Arc::new(ReceiveFileHostV2 {
                    root: self.d.clone(),
                    home: self.h.clone(),
                    bindings,
                    trusted_mapping_write: None,
                }),
                password.cloned(),
            )
            .map_err(|_| err("file_local_context_invalid"))?;
            Ok(Prepared {
                host,
                payload,
                scope: scope_digest,
                target,
                selected,
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
    std::fs::create_dir_all(parent)?;
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
