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
}
impl HostPreviewV2 {
    pub fn source_guard(&self) -> &SourceGuard {
        &self.source_guard
    }
    pub fn root_revision(&self) -> &str {
        &self.root_revision
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
        "mcp_source"
        | "capture_scope_invalid"
        | "capture_mcp_source_changed"
        | "overlay_scope_invalid" => (ErrorCode::SyncScopeChanged, Cause::ScopeChanged),
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
    let captured = capture_checked(provider, &selection)?;
    let guard = captured.source_guard.clone();
    let root = captured.root_revision.clone();
    let local_summary = captured.objects.summary();
    let local_atoms = file_objects_v2::save_state(&captured.objects).map_err(map_host_error)?;
    let local_time =
        saved_time(&captured.objects, local_atoms.keys().cloned()).map_err(map_host_error)?;
    let can_upload = !local_atoms.is_empty();
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
    let projected = objects_overlay_v2::overlay_selected(
        baseline,
        captured.objects,
        selection.clone(),
        Some(&PayloadAuthorizationV2),
    )
    .map_err(map_host_error)?;
    let after = file_objects_v2::semantic_digest(&projected).map_err(map_host_error)?;
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
    })
}
/// The actual PayloadCodec used by SyncContext. Immutable selection, expected
/// capture/root and precise baseline identity come from a verified preview.
pub struct HostPayloadCodecV2 {
    provider: Arc<dyn CaptureProviderV2>,
    preview: HostPreviewV2,
    secrets: SessionSecretsV2,
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
        decode_payload(bytes, &self.secrets).map(|_| ())
    }
}
#[cfg(test)]
#[path = "sync_host_v2_tests.rs"]
mod tests;
