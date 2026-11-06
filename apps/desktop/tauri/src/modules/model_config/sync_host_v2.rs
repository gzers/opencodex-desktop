//! Production composition of the saved-capture, explicit overlay and portable
//! codec owners. No v1 fallback, UI authority, local apply transaction, or network
//! request is implemented here. Passphrases and captured objects are never DTOs.
use super::{
    content_versions::TimeKind,
    file_objects_v2::{self, AuthorizationValidator, PublicSummary, SavedObjects},
    objects_overlay_v2::{self, SelectionV2, UsageV2},
    portable_v2::PortableChannelV2,
    provider_auth::{self, PortableAuthorization},
    schema::{ModelError, ModelResult},
    sync_v2::{store::digest, types::*},
};
use serde::Serialize;
use std::sync::Arc;
use zeroize::Zeroizing;

/// Actual capture adapters must call the single saved-only capture owner with
/// explicit D/H and frozen CaptureRequest. No Debug/Serialize or raw UI source.
pub struct HostCapturedV2 {
    pub objects: SavedObjects,
    pub source_guard: SourceGuard,
    pub root_revision: Digest,
}
pub trait CaptureProviderV2: Send + Sync {
    fn capture_saved(&self, selection: &SelectionV2) -> ModelResult<HostCapturedV2>;
    /// None denotes the legacy SAME identity domain only, never an implicit
    /// mapping. A mapped provider must also verify the trusted target facts.
    fn target_binding(
        &self,
        _selection: &SelectionV2,
    ) -> ModelResult<Option<LocalTargetBindingV2>> {
        Ok(None)
    }
    fn verify_target(&self, _binding: &LocalTargetBindingV2) -> ModelResult<()> {
        Err(super::schema::invalid(
            "host_mapping_required",
            "需要后端目标映射校验",
        ))
    }
}
/// Host-only identity contract, NOT a new transfer/document/IPC format. L0's
/// mapping owner freezes/validates these fields and persists identities only
/// with the successful apply transaction. No label/email matching occurs here.
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalTargetBindingV2 {
    pub mapping_revision: Digest,
    pub local_scope: SelectionV2,
    pub remote_scope: SelectionV2,
    pub direction: DirectionV2,
    pub slots: Vec<TargetSlotV2>,
    pub digest: Digest,
}
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetSlotV2 {
    pub kind: String,
    pub portable_id: String,
    pub local_id: String,
    pub expected: TargetExpectationV2,
}
#[derive(Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "digest", rename_all = "snake_case")]
pub enum TargetExpectationV2 {
    Present(Digest),
    ExpectedAbsent(Digest),
}
impl LocalTargetBindingV2 {
    pub fn freeze(
        mapping_revision: Digest,
        local_scope: SelectionV2,
        remote_scope: SelectionV2,
        direction: DirectionV2,
        slots: Vec<TargetSlotV2>,
    ) -> SyncResult<Self> {
        require_digest(&mapping_revision)?;
        validate_remote_selection(&local_scope)?;
        validate_remote_selection(&remote_scope)?;
        if direction == DirectionV2::NoChange || slots.len() > 10000 {
            return Err(failure(ErrorCode::SyncScopeChanged, Step::SourceGuard));
        }
        let mut identities = std::collections::BTreeSet::new();
        let mut locals = std::collections::BTreeSet::new();
        for slot in &slots {
            if slot.kind != "channel"
                || slot.portable_id.is_empty()
                || slot.local_id.is_empty()
                || !identities.insert((slot.kind.clone(), slot.portable_id.clone()))
                || !locals.insert((slot.kind.clone(), slot.local_id.clone()))
            {
                return Err(failure(ErrorCode::SyncScopeChanged, Step::SourceGuard));
            }
            require_uuid(&slot.portable_id)?;
            require_uuid(&slot.local_id)?;
            if !remote_scope
                .channels
                .iter()
                .any(|c| c.portable_channel_id == slot.portable_id)
            {
                return Err(failure(ErrorCode::SyncScopeChanged, Step::SourceGuard));
            }
            let local_present = local_scope
                .channels
                .iter()
                .any(|c| c.portable_channel_id == slot.local_id);
            if matches!(slot.expected, TargetExpectationV2::Present(_)) != local_present {
                return Err(failure(ErrorCode::SyncScopeChanged, Step::SourceGuard));
            }
            match &slot.expected {
                TargetExpectationV2::Present(d) | TargetExpectationV2::ExpectedAbsent(d) => {
                    require_digest(d)?;
                }
            }
        }
        if remote_scope.channels.len() != slots.len()
            || local_scope.channels.iter().any(|c| {
                !slots.iter().any(|s| {
                    s.local_id == c.portable_channel_id
                        && matches!(s.expected, TargetExpectationV2::Present(_))
                })
            })
        {
            return Err(failure(ErrorCode::SyncScopeChanged, Step::SourceGuard));
        }
        let material = serde_json::json!({"domain":"OCXD_LOCAL_TARGET_BINDING_V2","mappingRevision":mapping_revision,"localScope":local_scope,"remoteScope":remote_scope,"direction":direction,"slots":slots});
        let digest = digest(&super::sync_v2::store::canonical(&material)?);
        Ok(Self {
            mapping_revision,
            local_scope,
            remote_scope,
            direction,
            slots,
            digest,
        })
    }
    fn validate(&self, remote: &SelectionV2) -> SyncResult<()> {
        let fresh = Self::freeze(
            self.mapping_revision.clone(),
            self.local_scope.clone(),
            self.remote_scope.clone(),
            self.direction.clone(),
            self.slots.clone(),
        )?;
        if self.remote_scope != *remote || self.digest != fresh.digest {
            return Err(failure(ErrorCode::SyncScopeChanged, Step::SourceGuard));
        }
        Ok(())
    }
}
/// L0 owns actual locked mapping/expected-present-or-absent facts and stable
/// portable<->local IDs. Projection rewrites only identity references through
/// that mapping owner and must return a strictly validated SavedObjects. No
/// model matcher, target allocator, disk mapping writer or fallback is copied.
pub trait TrustedMappingOwnerV2: Send + Sync {
    fn verify(&self, binding: &LocalTargetBindingV2) -> ModelResult<()>;
    fn project_saved_to_remote(
        &self,
        objects: SavedObjects,
        binding: &LocalTargetBindingV2,
    ) -> ModelResult<SavedObjects>;
}
pub struct MappedCaptureProviderV2 {
    local: Arc<dyn CaptureProviderV2>,
    mapping: Arc<dyn TrustedMappingOwnerV2>,
    binding: LocalTargetBindingV2,
}
impl MappedCaptureProviderV2 {
    pub fn new(
        local: Arc<dyn CaptureProviderV2>,
        mapping: Arc<dyn TrustedMappingOwnerV2>,
        binding: LocalTargetBindingV2,
    ) -> SyncResult<Self> {
        binding.validate(&binding.remote_scope)?;
        mapping.verify(&binding).map_err(map_host_error)?;
        Ok(Self {
            local,
            mapping,
            binding,
        })
    }
}
impl CaptureProviderV2 for MappedCaptureProviderV2 {
    fn capture_saved(&self, remote: &SelectionV2) -> ModelResult<HostCapturedV2> {
        self.binding.validate(remote).map_err(|_| {
            super::schema::invalid("host_mapping_changed", "远端与本机选区映射已变化")
        })?;
        self.mapping.verify(&self.binding)?;
        let local = self.local.capture_saved(&self.binding.local_scope)?;
        let objects = self
            .mapping
            .project_saved_to_remote(local.objects, &self.binding)?;
        self.mapping.verify(&self.binding)?;
        let saved_source_digest=digest(&super::sync_v2::store::canonical(&serde_json::json!({"domain":"OCXD_MAPPED_SAVED_SOURCE_V2","localSaved":local.source_guard.saved_source_digest,"targetBinding":self.binding.digest})).map_err(|_|super::schema::invalid("host_mapping_changed","目标事实无法核对"))?);
        let scope_digest=digest(&super::sync_v2::store::canonical(&serde_json::json!({"domain":"OCXD_MAPPED_SCOPE_V2","localScopeGuard":local.source_guard.scope_digest,"targetBinding":self.binding})).map_err(|_|super::schema::invalid("host_mapping_changed","目标范围无法核对"))?);
        Ok(HostCapturedV2 {
            objects,
            source_guard: SourceGuard {
                saved_source_digest,
                scope_digest,
                mcp_sources_digest: local.source_guard.mcp_sources_digest,
            },
            root_revision: local.root_revision,
        })
    }
    fn target_binding(&self, remote: &SelectionV2) -> ModelResult<Option<LocalTargetBindingV2>> {
        self.binding
            .validate(remote)
            .map_err(|_| super::schema::invalid("host_mapping_changed", "目标范围已改变"))?;
        self.mapping.verify(&self.binding)?;
        Ok(Some(self.binding.clone()))
    }
    fn verify_target(&self, binding: &LocalTargetBindingV2) -> ModelResult<()> {
        if *binding != self.binding {
            return Err(super::schema::invalid(
                "host_mapping_changed",
                "目标绑定已改变",
            ));
        }
        self.mapping.verify(binding)
    }
}
/// Production adapter to the ONE immutable, registered saved-object capture
/// implementation. The request, D/H and credential owner are fixed for this
/// session. UI metadata, drafts and arbitrary public providers are not substitutes.
pub struct SavedCaptureProviderV2 {
    root: std::path::PathBuf,
    home: std::path::PathBuf,
    request: super::saved_objects_capture_v2::CaptureRequestV2,
    credentials: Arc<dyn super::saved_objects_capture_v2::CaptureCredentialsV2 + Send + Sync>,
}
impl SavedCaptureProviderV2 {
    pub fn new(
        root: std::path::PathBuf,
        home: std::path::PathBuf,
        request: super::saved_objects_capture_v2::CaptureRequestV2,
        credentials: Arc<dyn super::saved_objects_capture_v2::CaptureCredentialsV2 + Send + Sync>,
    ) -> SyncResult<Self> {
        validate_remote_selection(&request.selection)?;
        if !root.is_absolute() || !home.is_absolute() || !root.is_dir() || !home.is_dir() {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::SourceGuard));
        }
        Ok(Self {
            root,
            home,
            request,
            credentials,
        })
    }
    pub fn selection(&self) -> &SelectionV2 {
        &self.request.selection
    }
}
impl CaptureProviderV2 for SavedCaptureProviderV2 {
    fn capture_saved(&self, selection: &SelectionV2) -> ModelResult<HostCapturedV2> {
        if self.request.selection != *selection {
            return Err(super::schema::invalid(
                "capture_scope_invalid",
                "会话范围已改变，需重新预览",
            ));
        }
        let captured = super::saved_objects_capture_v2::capture(
            &self.root,
            &self.home,
            &self.request,
            self.credentials.as_ref(),
        )?;
        if file_objects_v2::save_state(&captured.objects)? != captured.save_state
            || file_objects_v2::semantic_digest(&captured.objects)? != captured.semantic_digest
        {
            return Err(super::schema::invalid(
                "capture_stale",
                "捕获材料未保持同一保存事实",
            ));
        }
        Ok(HostCapturedV2 {
            objects: captured.objects,
            source_guard: captured.source_guard,
            root_revision: captured.root_revision,
        })
    }
}
/// L0 can delegate LocalApplication's source/root facts here. This deliberately
/// recaptures for NEW write admission, never from original-operation inspection.
pub struct CurrentSourceProviderV2 {
    capture: Arc<dyn CaptureProviderV2>,
    selection: SelectionV2,
    root_revision: Digest,
}
impl CurrentSourceProviderV2 {
    pub fn new(
        capture: Arc<dyn CaptureProviderV2>,
        selection: SelectionV2,
        root_revision: Digest,
    ) -> SyncResult<Self> {
        require_digest(&root_revision)?;
        validate_remote_selection(&selection)?;
        Ok(Self {
            capture,
            selection,
            root_revision,
        })
    }
    fn current(&self) -> SyncResult<HostCapturedV2> {
        let current = capture_checked(self.capture.as_ref(), &self.selection)?;
        if current.root_revision != self.root_revision {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::SourceGuard));
        }
        Ok(current)
    }
    pub fn current_source(&self) -> SyncResult<SourceGuard> {
        Ok(self.current()?.source_guard)
    }
    pub fn root_revision(&self) -> SyncResult<Digest> {
        Ok(self.current()?.root_revision)
    }
}
/// Only the provider already attached and validated by L1 is used. There is no
/// free-standing caller-supplied public object and no protocol/adapter guessing.
pub struct PayloadAuthorizationV2;
impl AuthorizationValidator for PayloadAuthorizationV2 {
    fn validate(
        &self,
        channel: &PortableChannelV2,
        authorization: &PortableAuthorization,
    ) -> ModelResult<()> {
        let public = channel
            .connection
            .as_ref()
            .and_then(|c| c.provider.as_ref())
            .ok_or_else(|| {
                super::schema::invalid(
                    "portable_authorization_context_required",
                    "授权缺少已验证的服务公开来源",
                )
            })?;
        provider_auth::validate_portable(public, authorization)
    }
}
/// Operation-memory only. Neither secret is persisted in a journal/DTO, and both
/// are cleared when the session drops. Re-encryption is performed ONCE at freeze.
#[derive(Default)]
pub struct SessionSecretsV2 {
    pub read_passphrase: Option<Zeroizing<String>>,
    pub write_passphrase: Option<Zeroizing<String>>,
}
impl SessionSecretsV2 {
    fn read(&self) -> Option<&str> {
        self.read_passphrase.as_ref().map(|v| v.as_str())
    }
    fn write(&self) -> Option<&str> {
        self.write_passphrase.as_ref().map(|v| v.as_str())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DirectionV2 {
    Upload,
    Download,
    NoChange,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomaticReasonV2 {
    SameSemanticContent,
    BothEmpty,
    CloudEmpty,
    LocalEmpty,
    LocalSavedLater,
    CloudSavedLater,
    UnreliableSaveTime,
    EqualTimeDifferentContent,
    ExplicitUpload,
    ExplicitDownload,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceDecisionV2 {
    pub chosen_direction: DirectionV2,
    pub automatic_reason: AutomaticReasonV2,
    pub local_saved_time: Option<String>,
    pub cloud_saved_time: Option<String>,
    pub no_change: bool,
    pub can_upload: bool,
    pub can_download: bool,
    pub local_summary: PublicSummary,
    pub cloud_summary: Option<PublicSummary>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BaselineBindingV2 {
    pub head: CommitRef,
    pub payload_digest: Digest,
}
/// A complete material proven by the graph/transport owner, or a positively
/// observed empty directory. Unknown/error/invalid must NEVER use Empty.
pub enum CloudMaterialV2<'a> {
    Empty,
    Valid(&'a PayloadInput),
    Unavailable(SyncFailure),
}
/// Constructible only by preview_semantics. Contains hashes and public summaries,
/// not payload bytes, credentials, passwords, URLs, or arbitrary root paths.
pub struct HostPreviewV2 {
    pub decision: SourceDecisionV2,
    selection: SelectionV2,
    source_guard: SourceGuard,
    root_revision: Digest,
    baseline: Option<BaselineBindingV2>,
    target_binding: Option<LocalTargetBindingV2>,
}
impl HostPreviewV2 {
    pub fn source_guard(&self) -> &SourceGuard {
        &self.source_guard
    }
    pub fn root_revision(&self) -> &str {
        &self.root_revision
    }
    pub fn target_binding(&self) -> Option<&LocalTargetBindingV2> {
        self.target_binding.as_ref()
    }
    pub fn baseline(&self) -> Option<&BaselineBindingV2> {
        self.baseline.as_ref()
    }
}
fn validate_remote_selection(selection: &SelectionV2) -> SyncResult<()> {
    if selection.usage != UsageV2::RemoteSync
        || !selection.manager.webdav_ids.is_empty()
        || !selection.manager.webdav_password_ids.is_empty()
    {
        return Err(failure(ErrorCode::SyncScopeChanged, Step::SourceGuard));
    }
    Ok(())
}
fn capture_checked(
    provider: &dyn CaptureProviderV2,
    selection: &SelectionV2,
) -> SyncResult<HostCapturedV2> {
    validate_remote_selection(selection)?;
    if let Some(binding) = provider.target_binding(selection).map_err(map_host_error)? {
        binding.validate(selection)?;
        provider.verify_target(&binding).map_err(map_host_error)?;
    }
    let captured = provider.capture_saved(selection).map_err(map_host_error)?;
    require_guard(&captured.source_guard)?;
    require_digest(&captured.root_revision)?;
    captured
        .objects
        .validate_remote_sync_scope()
        .map_err(map_host_error)?;
    // SavedObjects' constructor is already strict. Revalidation also verifies
    // the host authorization callback against the actual attached provider.
    let _ = file_objects_v2::encode_document(&captured.objects, Some(&PayloadAuthorizationV2))
        .map_err(map_host_error)?;
    Ok(captured)
}
pub fn map_host_error(error: ModelError) -> SyncFailure {
    let (code, cause) = match error.code.as_str() {
        "passphrase_required" => (ErrorCode::SyncPassphraseRequired, Cause::Passphrase),
        "portable_authentication" => (ErrorCode::SyncPassphraseInvalid, Cause::Passphrase),
        "model_schema_unsupported"
        | "portable_objects_v2_schema"
        | "portable_objects_v2_version" => (
            ErrorCode::SyncPayloadSchemaUnsupported,
            Cause::UnsupportedSchema,
        ),
        "portable_limit" | "model_limit" | "asset_limit" => (ErrorCode::SyncLimit, Cause::TooLarge),
        "capture_stale" | "capture_source_changed" | "model_read_stale" | "model_save_stale" => {
            (ErrorCode::SyncSourceChanged, Cause::SourceChanged)
        }
        "host_mapping_changed"
        | "host_mapping_required"
        | "mcp_source"
        | "capture_scope_invalid"
        | "capture_mcp_source_changed"
        | "overlay_scope_invalid" => (ErrorCode::SyncScopeChanged, Cause::ScopeChanged),
        "host_credential_read" => (ErrorCode::SyncLocalStateIo, Cause::LocalIo),
        "capture_root_invalid" | "model_transaction_pending" => {
            (ErrorCode::SyncPlanChanged, Cause::SourceChanged)
        }
        "capture_credential_missing" | "provider_auth_reference" | "extension_credential" => {
            (ErrorCode::SyncAuthRequired, Cause::Unauthorized)
        }
        _ => (ErrorCode::SyncPayloadInvalid, Cause::Protocol),
    };
    let mut out = failure(code, Step::Decode);
    out.cause = Some(cause);
    // No operation exists at this boundary. Do not create an unbound retry/verify.
    out.recovery_actions.retain(|a| {
        !matches!(
            a.kind,
            RecoveryKind::VerifyOriginalUpload | RecoveryKind::ResumeOriginalUpload
        )
    });
    out
}
fn decode_payload(bytes: &[u8], secrets: &SessionSecretsV2) -> SyncResult<SavedObjects> {
    let decoded = file_objects_v2::decode(
        bytes,
        secrets.read(),
        &file_objects_v2::supported_schema(),
        Some(&PayloadAuthorizationV2),
    )
    .map_err(map_host_error)?;
    decoded
        .objects
        .validate_remote_sync_scope()
        .map_err(map_host_error)?;
    Ok(decoded.objects)
}
fn snapshot_guard_matches(c: &HostCapturedV2, guard: &SourceGuard, root: &str) -> SyncResult<()> {
    if c.root_revision != root {
        return Err(failure(ErrorCode::SyncPlanChanged, Step::SourceGuard));
    }
    if c.source_guard.scope_digest != guard.scope_digest
        || c.source_guard.mcp_sources_digest != guard.mcp_sources_digest
    {
        return Err(failure(ErrorCode::SyncScopeChanged, Step::SourceGuard));
    }
    if c.source_guard.saved_source_digest != guard.saved_source_digest {
        return Err(failure(ErrorCode::SyncSourceChanged, Step::SourceGuard));
    }
    Ok(())
}
fn saved_time(
    objects: &SavedObjects,
    keys: impl Iterator<Item = String>,
) -> ModelResult<Option<String>> {
    let atoms = file_objects_v2::save_state(objects)?;
    let receipts = objects.partitions().content_versions;
    let mut latest = None;
    let mut any = false;
    for key in keys {
        any = true;
        let Some(content) = atoms.get(&key) else {
            return Ok(None);
        };
        let Some(v) = receipts.iter().find(|v| {
            v.scope_key == key && v.content_digest == *content && v.time_kind == TimeKind::Saved
        }) else {
            return Ok(None);
        };
        let Some(instant) = v.instant() else {
            return Ok(None);
        };
        if latest.as_ref().is_none_or(|(old, _)| instant > *old) {
            latest = Some((instant, v.saved_at.clone()));
        }
    }
    Ok(if any { latest.map(|(_, s)| s) } else { None })
}
fn has_selected_cloud(
    objects: &SavedObjects,
    selection: &SelectionV2,
    local: &SavedObjects,
) -> bool {
    let parts = objects.partitions();
    for c in &selection.channels {
        let Some(channel) = parts
            .models
            .payload
            .policy
            .as_ref()
            .and_then(|p| p.channels.iter().find(|ch| ch.id == c.portable_channel_id))
        else {
            return false;
        };
        if c.model_ids.as_ref().is_some_and(|ids| {
            ids.iter()
                .any(|id| !channel.models.iter().any(|m| m.raw_model_id == *id))
        }) {
            return false;
        }
        if c.family_ids.as_ref().is_some_and(|ids| {
            ids.iter()
                .any(|id| !parts.models.payload.templates.iter().any(|t| t.id == *id))
        }) {
            return false;
        }
        if c.include_connection && channel.connection.is_none()
            || c.include_key && parts.channel_keys.get(&c.portable_channel_id).is_none()
            || c.include_authorization
                && !parts
                    .channel_authorizations
                    .contains_key(&c.portable_channel_id)
        {
            return false;
        }
    }
    let library = super::schema_v2::LibraryV2 {
        schema_version: 2,
        templates: parts.models.payload.templates.clone(),
        submission_receipts: Default::default(),
    };
    for reference in selection
        .template_refs
        .iter()
        .chain(&selection.adopt_template_defaults)
    {
        if super::schema_v2::revision(&library, reference).is_err() {
            return false;
        }
    }
    if selection.manager.preferences && parts.manager.preferences.is_none()
        || selection.manager.extensions && parts.manager.extensions.is_none()
    {
        return false;
    }
    for id in &selection.manager.account_ids {
        let Some(source) = local
            .partitions()
            .manager
            .accounts
            .iter()
            .find(|a| a.id == *id)
        else {
            if parts.manager.accounts.iter().any(|a| a.id == *id) {
                continue;
            }
            return false;
        };
        let candidates: Vec<_> = parts
            .manager
            .accounts
            .iter()
            .filter(|a| {
                crate::modules::sync::portable_accounts::same_subject(a, source)
                    || (a.id == source.id
                        && objects_overlay_v2::account_source_identity(a).ok()
                            == objects_overlay_v2::account_source_identity(source).ok())
            })
            .collect();
        if candidates.len() != 1 {
            return false;
        }
    }
    if selection
        .skill_names
        .iter()
        .any(|name| !parts.assets.skills.iter().any(|s| s.name == *name))
        || selection
            .mcp_sources
            .iter()
            .any(|m| !parts.assets.mcps.iter().any(|c| c.name == m.name))
    {
        return false;
    }
    if selection
        .extension_credentials
        .skill_names
        .iter()
        .any(|name| {
            !parts
                .assets
                .skills
                .iter()
                .any(|s| s.name == *name && s.contains_secrets)
        })
        || selection
            .extension_credentials
            .mcp_names
            .iter()
            .any(|name| {
                !parts
                    .assets
                    .mcps
                    .iter()
                    .any(|m| m.name == *name && !m.secrets.0.is_empty())
            })
    {
        return false;
    }
    true
}
/// Read-only semantic decision, BEFORE prepare/encryption/publication. Invalid
/// cloud/credentials propagate as real errors; a valid unknown-time cloud wins
/// the default, but application remains gated by exact selected material.
pub fn preview_semantics(
    provider: &dyn CaptureProviderV2,
    selection: SelectionV2,
    cloud: CloudMaterialV2<'_>,
    secrets: &SessionSecretsV2,
    forced: Option<DirectionV2>,
) -> SyncResult<HostPreviewV2> {
    let target_binding = provider
        .target_binding(&selection)
        .map_err(map_host_error)?;
    if let Some(binding) = &target_binding {
        binding.validate(&selection)?;
        provider.verify_target(binding).map_err(map_host_error)?;
    }
    let captured = capture_checked(provider, &selection)?;
    let guard = captured.source_guard.clone();
    let root = captured.root_revision.clone();
    let local_summary = captured.objects.summary();
    let local_atoms = file_objects_v2::save_state(&captured.objects).map_err(map_host_error)?;
    let local_time =
        saved_time(&captured.objects, local_atoms.keys().cloned()).map_err(map_host_error)?;
    let can_upload = !local_atoms.is_empty()
        && has_selected_cloud(&captured.objects, &selection, &captured.objects)
        && target_binding
            .as_ref()
            .is_none_or(|b| b.direction == DirectionV2::Upload);
    let (baseline, binding) = match cloud {
        CloudMaterialV2::Empty => (None, None),
        CloudMaterialV2::Unavailable(e) => return Err(e),
        CloudMaterialV2::Valid(input) => {
            require_uuid(&input.head.operation_id)?;
            require_digest(&input.head.digest)?;
            let baseline = decode_payload(&input.bytes, secrets)?;
            (
                Some(baseline),
                Some(BaselineBindingV2 {
                    head: input.head.clone(),
                    payload_digest: digest(&input.bytes),
                }),
            )
        }
    };
    let cloud_summary = baseline.as_ref().map(SavedObjects::summary);
    let can_download = baseline
        .as_ref()
        .is_some_and(|b| has_selected_cloud(b, &selection, &captured.objects));
    let cloud_time = baseline
        .as_ref()
        .map(|b| {
            // A complete-global atom for [A,B] cannot date a projected [A]
            // scope. Keep it unknown instead of borrowing unselected save time.
            let local_domain: Vec<_> = captured
                .objects
                .models()
                .payload
                .policy
                .iter()
                .flat_map(|p| p.channels.iter().map(|c| &c.id))
                .collect();
            let cloud_domain: Vec<_> = b
                .models()
                .payload
                .policy
                .iter()
                .flat_map(|p| p.channels.iter().map(|c| &c.id))
                .collect();
            if local_domain != cloud_domain {
                Ok(None)
            } else {
                saved_time(b, local_atoms.keys().cloned())
            }
        })
        .transpose()
        .map_err(map_host_error)?
        .flatten();
    let before = baseline
        .as_ref()
        .map(file_objects_v2::semantic_digest)
        .transpose()
        .map_err(map_host_error)?;
    let mapped_receive = target_binding
        .as_ref()
        .is_some_and(|b| b.direction == DirectionV2::Download);
    let after = if mapped_receive {
        // A receive mapping may have no local objects at all. Never overlay
        // remote selections onto missing local UUIDs just to fabricate a diff.
        // The incoming complete payload is validated above; L0 produces the
        // exact mapped receive diff and applies with this target binding.
        let _ = baseline
            .as_ref()
            .ok_or_else(|| failure(ErrorCode::SyncPlanChanged, Step::DownloadGuard))?;
        file_objects_v2::semantic_digest(&captured.objects).map_err(map_host_error)?
    } else {
        let projected = objects_overlay_v2::overlay_selected(
            baseline,
            captured.objects,
            selection.clone(),
            Some(&PayloadAuthorizationV2),
        )
        .map_err(map_host_error)?;
        file_objects_v2::semantic_digest(&projected).map_err(map_host_error)?
    };
    let no_change =
        before.as_ref().is_some_and(|v| *v == after) || binding.is_none() && !can_upload;
    let (mut chosen, mut reason) = if no_change {
        (
            DirectionV2::NoChange,
            if binding.is_none() {
                AutomaticReasonV2::BothEmpty
            } else {
                AutomaticReasonV2::SameSemanticContent
            },
        )
    } else if binding.is_none() {
        (DirectionV2::Upload, AutomaticReasonV2::CloudEmpty)
    } else if !can_upload {
        (DirectionV2::Download, AutomaticReasonV2::LocalEmpty)
    } else {
        match (
            local_time
                .as_deref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()),
            cloud_time
                .as_deref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok()),
        ) {
            (Some(l), Some(r)) if l > r => {
                (DirectionV2::Upload, AutomaticReasonV2::LocalSavedLater)
            }
            (Some(l), Some(r)) if l < r => {
                (DirectionV2::Download, AutomaticReasonV2::CloudSavedLater)
            }
            (Some(_), Some(_)) => (
                DirectionV2::Download,
                AutomaticReasonV2::EqualTimeDifferentContent,
            ),
            _ => (DirectionV2::Download, AutomaticReasonV2::UnreliableSaveTime),
        }
    };
    if let Some(force) = forced {
        if force == DirectionV2::NoChange
            || force == DirectionV2::Upload && !can_upload
            || force == DirectionV2::Download && !can_download
        {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::SourceGuard));
        }
        if !no_change {
            reason = if force == DirectionV2::Upload {
                AutomaticReasonV2::ExplicitUpload
            } else {
                AutomaticReasonV2::ExplicitDownload
            };
            chosen = force;
        }
    }
    // Preview/capture must still describe the same saved source when returned.
    let current = capture_checked(provider, &selection)?;
    snapshot_guard_matches(&current, &guard, &root)?;
    Ok(HostPreviewV2 {
        decision: SourceDecisionV2 {
            chosen_direction: chosen,
            automatic_reason: reason,
            local_saved_time: local_time,
            cloud_saved_time: cloud_time,
            no_change,
            can_upload,
            can_download,
            local_summary,
            cloud_summary,
        },
        selection,
        source_guard: guard,
        root_revision: root,
        baseline: binding,
        target_binding,
    })
}
/// The actual PayloadCodec used by SyncContext. Immutable selection, expected
/// capture/root and precise baseline identity come from a verified preview.
pub struct HostPayloadCodecV2 {
    provider: Arc<dyn CaptureProviderV2>,
    preview: HostPreviewV2,
    secrets: SessionSecretsV2,
    own_frozen: std::sync::Mutex<std::collections::BTreeSet<Digest>>,
}
impl HostPayloadCodecV2 {
    pub fn new(
        provider: Arc<dyn CaptureProviderV2>,
        preview: HostPreviewV2,
        secrets: SessionSecretsV2,
    ) -> Self {
        Self {
            provider,
            preview,
            secrets,
            own_frozen: Default::default(),
        }
    }
    pub fn preview(&self) -> &HostPreviewV2 {
        &self.preview
    }
    pub fn current_source_provider(&self) -> SyncResult<CurrentSourceProviderV2> {
        CurrentSourceProviderV2::new(
            self.provider.clone(),
            self.preview.selection.clone(),
            self.preview.root_revision.clone(),
        )
    }
    fn compose(
        &self,
        guard: &SourceGuard,
        baselines: &[PayloadInput],
        resolution: Option<&Resolution>,
    ) -> SyncResult<SavedObjects> {
        if baselines.len() > 1 {
            return Err(failure(ErrorCode::SyncRemoteConflict, Step::SourceGuard));
        }
        if let Some(r) = resolution {
            if r.selection_mode != SelectionMode::Head
                || baselines.len() != 1
                || r.selected_head.as_ref() != Some(&baselines[0].head)
            {
                return Err(failure(ErrorCode::SyncRemoteConflict, Step::SourceGuard));
            }
            if r.scope_digest != guard.scope_digest {
                return Err(failure(ErrorCode::SyncScopeChanged, Step::SourceGuard));
            }
        }
        if guard.scope_digest != self.preview.source_guard.scope_digest
            || guard.mcp_sources_digest != self.preview.source_guard.mcp_sources_digest
        {
            return Err(failure(ErrorCode::SyncScopeChanged, Step::SourceGuard));
        }
        if guard.saved_source_digest != self.preview.source_guard.saved_source_digest {
            return Err(failure(ErrorCode::SyncSourceChanged, Step::SourceGuard));
        }
        match (&self.preview.baseline, baselines.first()) {
            (None, None) => (),
            (Some(binding), Some(input))
                if binding.head == input.head && binding.payload_digest == digest(&input.bytes) => {
            }
            _ => return Err(failure(ErrorCode::SyncPlanChanged, Step::SourceGuard)),
        }
        if self
            .provider
            .target_binding(&self.preview.selection)
            .map_err(map_host_error)?
            != self.preview.target_binding
        {
            return Err(failure(ErrorCode::SyncScopeChanged, Step::SourceGuard));
        }
        let local = capture_checked(self.provider.as_ref(), &self.preview.selection)?;
        snapshot_guard_matches(&local, guard, &self.preview.root_revision)?;
        let baseline = baselines
            .first()
            .map(|p| decode_payload(&p.bytes, &self.secrets))
            .transpose()?;
        let result = objects_overlay_v2::overlay_selected(
            baseline,
            local.objects,
            self.preview.selection.clone(),
            Some(&PayloadAuthorizationV2),
        )
        .map_err(map_host_error)?;
        result
            .validate_remote_sync_scope()
            .map_err(map_host_error)?;
        Ok(result)
    }
}
impl PayloadCodec for HostPayloadCodecV2 {
    fn supported_schemas(&self) -> Vec<PayloadSchemaRef> {
        vec![file_objects_v2::supported_schema()]
    }
    fn freeze_selected(
        &self,
        guard: &SourceGuard,
        baselines: &[PayloadInput],
        resolution: Option<&Resolution>,
    ) -> SyncResult<FrozenPayload> {
        if self.preview.decision.no_change
            || self.preview.decision.chosen_direction != DirectionV2::Upload
        {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::SourceGuard));
        }
        let result = self.compose(guard, baselines, resolution)?;
        let id = uuid::Uuid::new_v4().to_string();
        let created = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let encoded = file_objects_v2::encode(
            &result,
            &id,
            &created,
            self.secrets.write(),
            Some(&PayloadAuthorizationV2),
        )
        .map_err(map_host_error)?;
        let current = capture_checked(self.provider.as_ref(), &self.preview.selection)?;
        snapshot_guard_matches(&current, guard, &self.preview.root_revision)?;
        let protected = encoded.protected;
        // The immediate core validation of newly frozen bytes must use THIS
        // operation's write protection, not the baseline's old read password.
        // Only known exact hashes get that key; other incoming files still use
        // the explicit read passphrase. No key/material is journaled here.
        {
            let mut own = self
                .own_frozen
                .lock()
                .map_err(|_| failure(ErrorCode::SyncLocalStateIo, Step::Persist))?;
            if own.len() >= Budgets::default().heads {
                return Err(failure(ErrorCode::SyncLimit, Step::Persist));
            }
            own.insert(encoded.file_digest.clone());
        }
        let mut bytes = encoded.into_bytes();
        Ok(FrozenPayload {
            bytes: std::mem::take(&mut *bytes),
            schema: file_objects_v2::supported_schema(),
            protection_mode: if protected {
                ProtectionMode::Protected
            } else {
                ProtectionMode::Plain
            },
        })
    }
    fn inspect_and_validate(&self, bytes: &[u8], schema: &PayloadSchemaRef) -> SyncResult<()> {
        // Never ask capture for today's source/selection/MCP/credential state.
        if *schema != file_objects_v2::supported_schema() {
            return Err(failure(
                ErrorCode::SyncPayloadSchemaUnsupported,
                Step::Decode,
            ));
        }
        let own = self
            .own_frozen
            .lock()
            .map_err(|_| failure(ErrorCode::SyncLocalStateIo, Step::Decode))?
            .contains(&digest(bytes));
        if own {
            let decoded = file_objects_v2::decode(
                bytes,
                self.secrets.write(),
                schema,
                Some(&PayloadAuthorizationV2),
            )
            .map_err(map_host_error)?;
            decoded
                .objects
                .validate_remote_sync_scope()
                .map_err(map_host_error)
        } else {
            decode_payload(bytes, &self.secrets).map(|_| ())
        }
    }
}
/// Root-aware credential owner injected by L0. The exact saved ref_id is passed
/// inside the saved endpoint; endpoint_id is NEVER a credential reference.
pub trait EndpointCredentialsV2: Send + Sync {
    fn read_password(
        &self,
        root: &std::path::Path,
        endpoint: &crate::modules::sync::config::SyncEndpointConfig,
    ) -> ModelResult<Zeroizing<String>>;
}
/// D-bound production read, never configure() or the singleton/cache API.
pub struct SystemEndpointCredentialsV2;
impl EndpointCredentialsV2 for SystemEndpointCredentialsV2 {
    fn read_password(
        &self,
        root: &std::path::Path,
        endpoint: &crate::modules::sync::config::SyncEndpointConfig,
    ) -> ModelResult<Zeroizing<String>> {
        let value = crate::infrastructure::credentials::load_optional_at(
            root,
            crate::infrastructure::keychain::WEBDAV_CREDENTIAL_PURPOSE,
            &endpoint.credential_ref.ref_id,
        )
        .map_err(|_| {
            super::schema::invalid(
                "host_credential_read",
                "原连接凭据当前无法读取或未通过归属校验",
            )
        })?;
        value
            .ok_or_else(|| super::schema::invalid("capture_credential_missing", "原连接凭据不存在"))
    }
}
struct EndpointSnapshotV2 {
    endpoint: crate::modules::sync::config::SyncEndpointConfig,
    handle: ConnectionHandle,
}
/// Original endpoint metadata is host-owned in memory; it is not a UI handle
/// constructor or a new persistence format. L0 restores its own saved metadata
/// explicitly after a restart, rather than falling back to the active endpoint.
pub struct EndpointResolverV2 {
    root: std::path::PathBuf,
    credentials: Arc<dyn EndpointCredentialsV2>,
    originals: std::sync::Mutex<std::collections::BTreeMap<String, EndpointSnapshotV2>>,
}
pub struct BoundEndpointV2 {
    handle: ConnectionHandle,
    base: String,
    directory: String,
    username: String,
    password: Zeroizing<String>,
}
impl BoundEndpointV2 {
    pub fn handle(&self) -> &ConnectionHandle {
        &self.handle
    }
    /// Bind the adapter supplied by L0. This builds no new transport and makes
    /// no request. Actual networking remains the separately owned adapter.
    pub fn bind_file_transport(
        &self,
        transport: &crate::infrastructure::webdav_config_v2::FileWebDavTransport,
    ) -> SyncResult<()> {
        transport
            .bind(
                self.handle.clone(),
                &self.base,
                &self.directory,
                self.username.clone(),
                self.password.to_string(),
            )
            .map_err(|e| transport_failure(e, Step::SourceGuard))
    }
}
struct NormalEndpointV2 {
    base: String,
    directory: String,
    username: String,
}
fn normalize_endpoint(
    endpoint: &crate::modules::sync::config::SyncEndpointConfig,
) -> SyncResult<NormalEndpointV2> {
    fn components(raw: &str) -> SyncResult<Vec<String>> {
        if raw.contains(['\\', '?', '#']) {
            return Err(failure(ErrorCode::SyncEndpointChanged, Step::SourceGuard));
        }
        let raw = raw.strip_prefix('/').unwrap_or(raw);
        let raw = raw.strip_suffix('/').unwrap_or(raw);
        if raw.is_empty() {
            return Ok(vec![]);
        }
        raw.split('/')
            .map(|part| {
                let decoded = percent_encoding::percent_decode_str(part)
                    .decode_utf8()
                    .map_err(|_| failure(ErrorCode::SyncEndpointChanged, Step::SourceGuard))?;
                if decoded.is_empty()
                    || matches!(decoded.as_ref(), "." | "..")
                    || decoded.contains(['/', '\\', '%'])
                    || decoded.chars().any(char::is_control)
                {
                    return Err(failure(ErrorCode::SyncEndpointChanged, Step::SourceGuard));
                }
                Ok(decoded.into_owned())
            })
            .collect()
    }
    require_uuid(&endpoint.endpoint_id)?;
    let raw = endpoint.url.trim();
    let raw_path = raw
        .split_once("://")
        .and_then(|(_, s)| s.find('/').map(|i| &s[i..]))
        .unwrap_or("");
    let parts = components(raw_path)?;
    let remote = components(endpoint.remote_path.trim())?;
    let mut base = reqwest::Url::parse(raw)
        .map_err(|_| failure(ErrorCode::SyncEndpointChanged, Step::SourceGuard))?;
    if base.scheme() != "https"
        || base.host_str().is_none()
        || !base.username().is_empty()
        || base.password().is_some()
        || base.query().is_some()
        || base.fragment().is_some()
        || remote.is_empty()
        || endpoint.username.trim().is_empty()
    {
        return Err(failure(ErrorCode::SyncEndpointChanged, Step::SourceGuard));
    }
    {
        let mut path = base
            .path_segments_mut()
            .map_err(|_| failure(ErrorCode::SyncEndpointChanged, Step::SourceGuard))?;
        path.clear();
        for part in parts {
            path.push(&part);
        }
    }
    let directory = remote
        .iter()
        .map(|part| {
            percent_encoding::utf8_percent_encode(part, percent_encoding::NON_ALPHANUMERIC)
                .to_string()
        })
        .collect::<Vec<_>>()
        .join("/");
    Ok(NormalEndpointV2 {
        base: base.to_string().trim_end_matches('/').into(),
        directory,
        username: endpoint.username.trim().into(),
    })
}
fn endpoint_handle(
    endpoint: &crate::modules::sync::config::SyncEndpointConfig,
    n: &NormalEndpointV2,
    password: &str,
) -> SyncResult<ConnectionHandle> {
    if password.is_empty() {
        return Err(failure(ErrorCode::SyncAuthRequired, Step::SourceGuard));
    }
    let secret_digest = digest(password.as_bytes());
    let binding_id = digest(&super::sync_v2::store::canonical(
        &serde_json::json!({"url":n.base,"remotePath":n.directory,"username":n.username,"secretDigest":secret_digest}),
    )?);
    let revision = digest(&super::sync_v2::store::canonical(
        &serde_json::json!({"endpointId":endpoint.endpoint_id,"bindingId":binding_id,"credentialRef":endpoint.credential_ref}),
    )?);
    Ok(ConnectionHandle {
        id: format!("{}:{}", endpoint.endpoint_id, &revision[7..]),
        binding_id,
        revision,
    })
}
impl EndpointResolverV2 {
    pub fn new(
        root: std::path::PathBuf,
        credentials: Arc<dyn EndpointCredentialsV2>,
    ) -> SyncResult<Self> {
        if !root.is_absolute()
            || !root.is_dir()
            || std::fs::symlink_metadata(&root)
                .map_err(|_| failure(ErrorCode::SyncLocalStateIo, Step::SourceGuard))?
                .file_type()
                .is_symlink()
        {
            return Err(failure(ErrorCode::SyncPlanChanged, Step::SourceGuard));
        }
        Ok(Self {
            root,
            credentials,
            originals: Default::default(),
        })
    }
    fn saved_config(&self) -> SyncResult<crate::modules::sync::config::SyncConfig> {
        let store = crate::modules::sync::config::SyncConfigStore::new(&self.root);
        // Only the fixed owner path is read. Check its existing directory and
        // file without following links; parsing/validation remains SyncConfigStore.
        for (path, directory) in [
            (self.root.join("manager-state"), true),
            (store.path(), false),
        ] {
            match std::fs::symlink_metadata(&path) {
                Ok(m)
                    if m.file_type().is_symlink()
                        || (directory && !m.is_dir())
                        || (!directory && !m.is_file()) =>
                {
                    return Err(failure(ErrorCode::SyncLocalStateIo, Step::SourceGuard))
                }
                Ok(_) => (),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                Err(_) => return Err(failure(ErrorCode::SyncLocalStateIo, Step::SourceGuard)),
            }
        }
        store
            .load()
            .map_err(|_| failure(ErrorCode::SyncEndpointChanged, Step::SourceGuard))
    }
    fn resolve_saved(
        &self,
        endpoint: &crate::modules::sync::config::SyncEndpointConfig,
    ) -> SyncResult<BoundEndpointV2> {
        let normalized = normalize_endpoint(endpoint)?;
        let password = self
            .credentials
            .read_password(&self.root, endpoint)
            .map_err(map_host_error)?;
        let handle = endpoint_handle(endpoint, &normalized, &password)?;
        Ok(BoundEndpointV2 {
            handle,
            base: normalized.base,
            directory: normalized.directory,
            username: normalized.username,
            password,
        })
    }
    pub fn resolve_active(&self) -> SyncResult<BoundEndpointV2> {
        let config = self.saved_config()?;
        let endpoint = config
            .active()
            .ok_or_else(|| failure(ErrorCode::SyncEndpointChanged, Step::SourceGuard))?;
        let bound = self.resolve_saved(endpoint)?;
        self.originals
            .lock()
            .map_err(|_| failure(ErrorCode::SyncLocalStateIo, Step::Persist))?
            .insert(
                bound.handle.id.clone(),
                EndpointSnapshotV2 {
                    endpoint: endpoint.clone(),
                    handle: bound.handle.clone(),
                },
            );
        Ok(bound)
    }
    /// Restore only L0's trusted ORIGINAL saved record plus the original journal
    /// handle. Caller-supplied UI endpoint fields have no authority here.
    pub fn restore_original(
        &self,
        endpoint: crate::modules::sync::config::SyncEndpointConfig,
        expected: &ConnectionHandle,
    ) -> SyncResult<BoundEndpointV2> {
        crate::modules::sync::config::validate_config(&crate::modules::sync::config::SyncConfig {
            schema_version: 1,
            endpoints: vec![endpoint.clone()],
        })
        .map_err(|_| failure(ErrorCode::SyncEndpointChanged, Step::SourceGuard))?;
        let bound = self.resolve_saved(&endpoint)?;
        if bound.handle != *expected {
            return Err(failure(ErrorCode::SyncEndpointChanged, Step::SourceGuard));
        }
        self.originals
            .lock()
            .map_err(|_| failure(ErrorCode::SyncLocalStateIo, Step::Persist))?
            .insert(
                expected.id.clone(),
                EndpointSnapshotV2 {
                    endpoint,
                    handle: expected.clone(),
                },
            );
        Ok(bound)
    }
    pub fn resolve_original(&self, expected: &ConnectionHandle) -> SyncResult<BoundEndpointV2> {
        let endpoint = {
            let originals = self
                .originals
                .lock()
                .map_err(|_| failure(ErrorCode::SyncLocalStateIo, Step::Persist))?;
            let original = originals
                .get(&expected.id)
                .ok_or_else(|| failure(ErrorCode::SyncEndpointChanged, Step::Verify))?;
            if original.handle != *expected {
                return Err(failure(ErrorCode::SyncEndpointChanged, Step::Verify));
            }
            original.endpoint.clone()
        };
        let config = self.saved_config()?;
        // A different active endpoint never replaces the saved original. If the
        // same endpoint ID has been edited, however, refuse its new revision
        // BEFORE reading any replacement credential reference.
        if let Some(current) = config
            .endpoints
            .iter()
            .find(|e| e.endpoint_id == endpoint.endpoint_id)
        {
            let old = normalize_endpoint(&endpoint)?;
            let new = normalize_endpoint(current)?;
            if old.base != new.base
                || old.directory != new.directory
                || old.username != new.username
                || current.credential_ref != endpoint.credential_ref
            {
                return Err(failure(ErrorCode::SyncEndpointChanged, Step::Verify));
            }
        }
        let resolved = self.resolve_saved(&endpoint)?;
        if resolved.handle != *expected {
            return Err(failure(ErrorCode::SyncEndpointChanged, Step::Verify));
        }
        Ok(resolved)
    }
}
#[derive(Serialize)]
#[serde(tag = "kind", content = "result", rename_all = "snake_case")]
pub enum HostPreparationV2 {
    NoChange(SourceDecisionV2),
    Upload(UploadPlan),
    Download(DownloadPlan),
}
/// Reusable owner-level session. Only L0 composes LocalApplication and decides
/// when a prepared plan receives explicit confirmation; prepare never PUTs.
pub struct HostSessionV2 {
    codec: HostPayloadCodecV2,
    connection: ConnectionHandle,
    observation: RemoteObservation,
}
impl HostSessionV2 {
    pub fn new(
        codec: HostPayloadCodecV2,
        bound: &BoundEndpointV2,
        observation: RemoteObservation,
    ) -> SyncResult<Self> {
        if observation.binding_id != bound.handle.binding_id {
            return Err(failure(ErrorCode::SyncEndpointChanged, Step::Observe));
        }
        let unique_generation = observation
            .generation
            .as_ref()
            .is_some_and(|g| observation.generations.as_slice() == [g.clone()]);
        match observation.state {
            RemoteState::Empty
                if codec.preview.baseline.is_none()
                    && observation.generations.is_empty()
                    && observation.heads.is_empty() => {}
            RemoteState::GenerationEmpty
                if codec.preview.baseline.is_none()
                    && unique_generation
                    && observation.heads.is_empty() => {}
            RemoteState::Ready | RemoteState::Conflict
                if unique_generation
                    && codec
                        .preview
                        .baseline
                        .as_ref()
                        .is_some_and(|b| observation.heads.contains(&b.head))
                    && ((observation.state == RemoteState::Ready
                        && observation.heads.len() == 1)
                        || (observation.state == RemoteState::Conflict
                            && observation.heads.len() > 1)) => {}
            RemoteState::InitializationAmbiguous | RemoteState::Conflict => {
                return Err(failure(ErrorCode::SyncRemoteConflict, Step::Observe))
            }
            _ => return Err(failure(ErrorCode::SyncRemoteInvalid, Step::Observe)),
        }
        Ok(Self {
            codec,
            connection: bound.handle.clone(),
            observation,
        })
    }
    fn context<'a>(
        &'a self,
        store: &'a super::sync_v2::store::Store,
        transport: &'a dyn ConfigSyncTransport,
        local: &'a dyn LocalApplication,
    ) -> SyncResult<super::sync_v2::SyncContext<'a>> {
        super::sync_v2::SyncContext::new(
            store,
            transport,
            &self.codec,
            local,
            self.connection.clone(),
        )
    }
    fn admit_new_write(&self) -> SyncResult<()> {
        let current = capture_checked(self.codec.provider.as_ref(), &self.codec.preview.selection)?;
        snapshot_guard_matches(
            &current,
            &self.codec.preview.source_guard,
            &self.codec.preview.root_revision,
        )
    }
    pub fn source_provider(&self) -> SyncResult<CurrentSourceProviderV2> {
        self.codec.current_source_provider()
    }
    pub fn decision(&self) -> &SourceDecisionV2 {
        &self.codec.preview.decision
    }
    pub async fn prepare(
        &self,
        resolver: &EndpointResolverV2,
        store: &super::sync_v2::store::Store,
        transport: &dyn ConfigSyncTransport,
        local: &dyn LocalApplication,
    ) -> SyncResult<HostPreparationV2> {
        resolver.resolve_original(&self.connection)?;
        self.admit_new_write()?;
        if self.codec.preview.decision.no_change {
            return Ok(HostPreparationV2::NoChange(
                self.codec.preview.decision.clone(),
            ));
        }
        let cx = self.context(store, transport, local)?;
        let preview = &self.codec.preview;
        match preview.decision.chosen_direction {
            DirectionV2::Upload if self.observation.state == RemoteState::Conflict => {
                let head = preview
                    .baseline
                    .as_ref()
                    .ok_or_else(|| failure(ErrorCode::SyncRemoteConflict, Step::SourceGuard))?
                    .head
                    .clone();
                let generation = self
                    .observation
                    .generation
                    .clone()
                    .ok_or_else(|| failure(ErrorCode::SyncRemoteConflict, Step::SourceGuard))?;
                let preview_digest = digest(&super::sync_v2::store::canonical(
                    &serde_json::json!({"viewDigest":self.observation.view_digest,"sourceGuard":preview.source_guard,"decision":preview.decision,"selectedHead":head}),
                )?);
                cx.prepare_resolution(PrepareResolutionRequest {
                    connection_id: self.connection.id.clone(),
                    generation,
                    heads: self.observation.heads.clone(),
                    expected_view_digest: self.observation.view_digest.clone(),
                    source_guard: preview.source_guard.clone(),
                    resolution: Resolution {
                        selection_mode: SelectionMode::Head,
                        selected_head: Some(head),
                        scope_digest: preview.source_guard.scope_digest.clone(),
                        preview_digest,
                    },
                })
                .await
                .map(HostPreparationV2::Upload)
            }
            DirectionV2::Upload => cx
                .prepare_upload(PrepareUploadRequest {
                    connection_id: self.connection.id.clone(),
                    expected_view_digest: self.observation.view_digest.clone(),
                    source_guard: preview.source_guard.clone(),
                    initialize_on_confirm: preview.baseline.is_none(),
                })
                .await
                .map(HostPreparationV2::Upload),
            DirectionV2::Download
                if preview.decision.can_download
                    && self.observation.state == RemoteState::Ready =>
            {
                cx.prepare_download(PrepareDownloadRequest {
                    connection_id: self.connection.id.clone(),
                    generation: self
                        .observation
                        .generation
                        .clone()
                        .ok_or_else(|| failure(ErrorCode::SyncPlanChanged, Step::DownloadGuard))?,
                    head: preview
                        .baseline
                        .as_ref()
                        .ok_or_else(|| failure(ErrorCode::SyncPlanChanged, Step::DownloadGuard))?
                        .head
                        .clone(),
                    expected_view_digest: self.observation.view_digest.clone(),
                    scope_digest: preview.source_guard.scope_digest.clone(),
                    backup_before_update: true,
                })
                .await
                .map(HostPreparationV2::Download)
            }
            _ => Err(failure(ErrorCode::SyncPlanChanged, Step::SourceGuard)),
        }
    }
    pub async fn apply_download(
        &self,
        r: ConfirmDownloadRequest,
        resolver: &EndpointResolverV2,
        store: &super::sync_v2::store::Store,
        transport: &dyn ConfigSyncTransport,
        local: &dyn LocalApplication,
    ) -> SyncResult<OperationFact> {
        resolver.resolve_original(&self.connection)?;
        if let Some(binding) = &self.codec.preview.target_binding {
            self.codec
                .provider
                .verify_target(binding)
                .map_err(map_host_error)?;
        }
        self.context(store, transport, local)?
            .apply_download(r)
            .await
    }
    pub async fn publish_upload(
        &self,
        r: ConfirmUploadRequest,
        resolver: &EndpointResolverV2,
        store: &super::sync_v2::store::Store,
        transport: &dyn ConfigSyncTransport,
        local: &dyn LocalApplication,
    ) -> SyncResult<OperationFact> {
        resolver.resolve_original(&self.connection)?;
        let journal = store.journal(&r.operation_id)?;
        // Any repeated confirmation after a possible send is the core's
        // read-only verification route, not authority to recapture a new source.
        if journal.fact.last_write_step == WriteStep::None {
            self.admit_new_write()?;
        }
        self.context(store, transport, local)?
            .publish_upload(r)
            .await
    }
    pub async fn verify_original_upload(
        &self,
        r: OperationRequest,
        resolver: &EndpointResolverV2,
        store: &super::sync_v2::store::Store,
        transport: &dyn ConfigSyncTransport,
        local: &dyn LocalApplication,
    ) -> SyncResult<OperationFact> {
        // No current selection/root/capture check here. Original endpoint and
        // original frozen journal/files determine the historical storage fact.
        resolver.resolve_original(&self.connection)?;
        self.context(store, transport, local)?
            .verify_original_upload(r)
            .await
    }
    pub async fn resume_original_upload(
        &self,
        r: ResumeUploadRequest,
        resolver: &EndpointResolverV2,
        store: &super::sync_v2::store::Store,
        transport: &dyn ConfigSyncTransport,
        local: &dyn LocalApplication,
    ) -> SyncResult<OperationFact> {
        resolver.resolve_original(&self.connection)?;
        let verified = self
            .context(store, transport, local)?
            .verify_original_upload(OperationRequest {
                operation_id: r.operation_id.clone(),
            })
            .await;
        if store.journal(&r.operation_id)?.fact.storage_status == StorageStatus::Stored {
            return verified;
        }
        match verified {
            Err(e) if e.code != ErrorCode::SyncOperationUnknown => return Err(e),
            _ => (),
        }
        self.admit_new_write()?;
        self.context(store, transport, local)?
            .resume_original_upload(r)
            .await
    }
}
/// Historical upload owner can be reconstructed from L0's trusted original
/// connection + existing journal, with NO preview, CaptureProvider or current
/// source available. This is essential after restart/local edits or bad drafts.
pub struct OriginalUploadOwnerV2 {
    connection: ConnectionHandle,
    inspection: InspectionCodecV2,
}
struct InspectionCodecV2 {
    secrets: SessionSecretsV2,
}
impl PayloadCodec for InspectionCodecV2 {
    fn supported_schemas(&self) -> Vec<PayloadSchemaRef> {
        vec![file_objects_v2::supported_schema()]
    }
    fn freeze_selected(
        &self,
        _: &SourceGuard,
        _: &[PayloadInput],
        _: Option<&Resolution>,
    ) -> SyncResult<FrozenPayload> {
        Err(failure(ErrorCode::SyncPlanChanged, Step::Verify))
    }
    fn inspect_and_validate(&self, bytes: &[u8], schema: &PayloadSchemaRef) -> SyncResult<()> {
        if *schema != file_objects_v2::supported_schema() {
            return Err(failure(
                ErrorCode::SyncPayloadSchemaUnsupported,
                Step::Decode,
            ));
        }
        decode_payload(bytes, &self.secrets).map(|_| ())
    }
}
impl OriginalUploadOwnerV2 {
    pub fn new(original: &BoundEndpointV2, read_passphrase: Option<Zeroizing<String>>) -> Self {
        Self {
            connection: original.handle.clone(),
            inspection: InspectionCodecV2 {
                secrets: SessionSecretsV2 {
                    read_passphrase,
                    write_passphrase: None,
                },
            },
        }
    }
    pub fn read_result(
        &self,
        store: &super::sync_v2::store::Store,
        request: OperationRequest,
    ) -> SyncResult<OperationFact> {
        let journal = store.journal(&request.operation_id)?;
        if journal.plan.binding_id != self.connection.binding_id {
            return Err(failure(ErrorCode::SyncEndpointChanged, Step::Verify));
        }
        Ok(journal.fact)
    }
    pub async fn verify(
        &self,
        request: OperationRequest,
        resolver: &EndpointResolverV2,
        store: &super::sync_v2::store::Store,
        transport: &dyn ConfigSyncTransport,
        local: &dyn LocalApplication,
    ) -> SyncResult<OperationFact> {
        resolver.resolve_original(&self.connection)?;
        super::sync_v2::SyncContext::new(
            store,
            transport,
            &self.inspection,
            local,
            self.connection.clone(),
        )?
        .verify_original_upload(request)
        .await
    }
}
#[cfg(test)]
#[path = "sync_host_v2_tests.rs"]
mod tests;
