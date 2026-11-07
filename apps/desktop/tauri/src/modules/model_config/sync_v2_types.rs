//! Frozen L2-sync-v1 DTOs and injectable, credential-free host boundaries.
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
pub type SyncResult<T> = Result<T, SyncFailure>;

pub type Uuid = String;
pub type Digest = String;
pub type Timestamp = String;
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SchemaRef {
    pub format: String,
    pub version: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelSchemaRef {
    pub format: String,
    pub format_version: u64,
    pub schema_version: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PayloadSchemaRef {
    pub envelope: SchemaRef,
    pub document: SchemaRef,
    pub model: ModelSchemaRef,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Generation {
    pub format: String,
    pub format_version: u32,
    pub generation_id: Uuid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GenerationRef {
    pub generation_id: Uuid,
    pub digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommitRef {
    pub operation_id: Uuid,
    pub digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Resolution {
    pub selection_mode: SelectionMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_head: Option<CommitRef>,
    pub scope_digest: Digest,
    pub preview_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Commit {
    pub format: String,
    pub format_version: u32,
    pub generation_id: Uuid,
    pub operation_id: Uuid,
    pub parents: Vec<CommitRef>,
    pub payload_digest: Digest,
    pub payload_bytes: u64,
    pub payload_schema: PayloadSchemaRef,
    pub intent: Intent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<Resolution>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Relation {
    pub local: LocalRelation,
    pub remote: RemoteRelation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecoveryAction {
    pub kind: RecoveryKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_plan_id: Option<Uuid>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<GenerationRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub head: Option<CommitRef>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SyncFailure {
    pub code: ErrorCode,
    pub step: Step,
    pub retryability: Retryability,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<Cause>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u64>,
    pub recovery_actions: Vec<RecoveryAction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecoveryCandidate {
    pub generation: GenerationRef,
    pub head: CommitRef,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RemoteObservation {
    pub dto_version: u32,
    pub binding_id: Digest,
    pub state: RemoteState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<GenerationRef>,
    pub generations: Vec<GenerationRef>,
    pub heads: Vec<CommitRef>,
    pub orphan_count: u64,
    pub stale_count: u64,
    pub view_digest: Digest,
    pub observed_at: Timestamp,
    pub problems: Vec<SyncFailure>,
    pub recovery_candidates: Vec<RecoveryCandidate>,
    pub recovery_actions: Vec<RecoveryAction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceGuard {
    pub saved_source_digest: Digest,
    pub scope_digest: Digest,
    pub mcp_sources_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationFact {
    pub dto_version: u32,
    pub operation_id: Uuid,
    pub kind: OperationKind,
    pub phase: Phase,
    pub last_write_step: WriteStep,
    pub storage_status: StorageStatus,
    pub local_apply_status: LocalApplyStatus,
    pub relation: Relation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<GenerationRef>,
    pub payload_digest: Digest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<CommitRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stored_at: Option<Timestamp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observation: Option<RemoteObservation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<SyncFailure>,
    pub recovery_actions: Vec<RecoveryAction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObserveRequest {
    pub connection_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OperationRequest {
    pub operation_id: Uuid,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrepareUploadRequest {
    pub connection_id: String,
    pub expected_view_digest: Digest,
    pub source_guard: SourceGuard,
    pub initialize_on_confirm: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UploadPlan {
    pub dto_version: u32,
    pub operation_id: Uuid,
    pub plan_digest: Digest,
    pub binding_id: Digest,
    pub initialization: Initialization,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation: Option<GenerationRef>,
    pub parents: Vec<CommitRef>,
    pub intent: Intent,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<Resolution>,
    pub payload_digest: Digest,
    pub payload_bytes: u64,
    pub payload_schema: PayloadSchemaRef,
    pub protection_mode: ProtectionMode,
    pub source_guard: SourceGuard,
    pub expected_view_digest: Digest,
    pub prepared_at: Timestamp,
    pub expires_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfirmUploadRequest {
    pub operation_id: Uuid,
    pub plan_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResumeUploadRequest {
    pub operation_id: Uuid,
    pub expected_view_digest: Digest,
    pub plan_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrepareResolutionRequest {
    pub connection_id: String,
    pub generation: GenerationRef,
    pub heads: Vec<CommitRef>,
    pub expected_view_digest: Digest,
    pub source_guard: SourceGuard,
    pub resolution: Resolution,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrepareDownloadRequest {
    pub connection_id: String,
    pub generation: GenerationRef,
    pub head: CommitRef,
    pub expected_view_digest: Digest,
    pub scope_digest: Digest,
    pub backup_before_update: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DownloadPlan {
    pub dto_version: u32,
    pub operation_id: Uuid,
    pub plan_digest: Digest,
    pub binding_id: Digest,
    pub generation: GenerationRef,
    pub head: CommitRef,
    pub payload_digest: Digest,
    pub scope_digest: Digest,
    pub target_digest: Digest,
    pub expected_view_digest: Digest,
    pub backup_before_update: bool,
    pub prepared_at: Timestamp,
    pub expires_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfirmDownloadRequest {
    pub operation_id: Uuid,
    pub plan_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrepareRecoveryRequest {
    pub connection_id: String,
    pub source_binding_id: Digest,
    pub generation: GenerationRef,
    pub head: CommitRef,
    pub expected_view_digest: Digest,
    pub scope_digest: Digest,
    pub mode: RecoveryMode,
    pub backup_before_update: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecoveryPlan {
    pub dto_version: u32,
    pub recovery_plan_id: Uuid,
    pub plan_digest: Digest,
    pub kind: RecoveryPlanKind,
    pub source_binding_id: Digest,
    pub generation: GenerationRef,
    pub head: CommitRef,
    pub payload_digest: Digest,
    pub payload_schema: PayloadSchemaRef,
    pub scope_digest: Digest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_digest: Option<Digest>,
    pub expected_view_digest: Digest,
    pub backup_before_update: bool,
    pub prepared_at: Timestamp,
    pub expires_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfirmRecoveryRequest {
    pub recovery_plan_id: Uuid,
    pub plan_digest: Digest,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecoveryReceipt {
    pub dto_version: u32,
    pub recovery_plan_id: Uuid,
    pub kind: RecoveryPlanKind,
    pub outcome: RecoveryOutcome,
    pub source_binding_id: Digest,
    pub generation: GenerationRef,
    pub head: CommitRef,
    pub payload_digest: Digest,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_id: Option<Uuid>,
    pub finished_at: Timestamp,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure: Option<SyncFailure>,
    pub recovery_actions: Vec<RecoveryAction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Journal {
    pub journal_version: u32,
    pub operation_id: Uuid,
    pub plan: UploadPlan,
    pub fact: OperationFact,
    pub root_revision: Digest,
    pub connection_revision: Digest,
    pub payload_handle: Uuid,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generation_bytes_utf8: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit_bytes_utf8: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_digest: Option<Digest>,
    pub previous_failures: Vec<SyncFailure>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SelectionMode {
    #[serde(rename = "head")]
    Head,
    #[serde(rename = "merged")]
    Merged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Intent {
    #[serde(rename = "update")]
    Update,
    #[serde(rename = "resolve")]
    Resolve,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LocalRelation {
    #[serde(rename = "not_compared")]
    NotCompared,
    #[serde(rename = "observed_equal")]
    ObservedEqual,
    #[serde(rename = "changed")]
    Changed,
    #[serde(rename = "unknown")]
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RemoteRelation {
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "observed_equal")]
    ObservedEqual,
    #[serde(rename = "descendant")]
    Descendant,
    #[serde(rename = "conflict")]
    Conflict,
    #[serde(rename = "initialization_ambiguous")]
    InitializationAmbiguous,
    #[serde(rename = "generation_changed")]
    GenerationChanged,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RecoveryKind {
    #[serde(rename = "verify_original_upload")]
    VerifyOriginalUpload,
    #[serde(rename = "refresh_remote")]
    RefreshRemote,
    #[serde(rename = "repreview_upload")]
    RepreviewUpload,
    #[serde(rename = "repreview_download")]
    RepreviewDownload,
    #[serde(rename = "choose_conflict_source")]
    ChooseConflictSource,
    #[serde(rename = "prepare_recovery_download")]
    PrepareRecoveryDownload,
    #[serde(rename = "prepare_recovery_export")]
    PrepareRecoveryExport,
    #[serde(rename = "select_empty_directory")]
    SelectEmptyDirectory,
    #[serde(rename = "resume_original_upload")]
    ResumeOriginalUpload,
    #[serde(rename = "return_to_original_connection")]
    ReturnToOriginalConnection,
    #[serde(rename = "unlock_payload")]
    UnlockPayload,
    #[serde(rename = "inspect_local_protection")]
    InspectLocalProtection,
    #[serde(rename = "free_remote_space")]
    FreeRemoteSpace,
    #[serde(rename = "retry_later")]
    RetryLater,
    #[serde(rename = "inspect_error_details")]
    InspectErrorDetails,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ErrorCode {
    #[serde(rename = "sync_generation_ambiguous")]
    SyncGenerationAmbiguous,
    #[serde(rename = "sync_generation_changed")]
    SyncGenerationChanged,
    #[serde(rename = "sync_listing_incomplete")]
    SyncListingIncomplete,
    #[serde(rename = "sync_remote_invalid")]
    SyncRemoteInvalid,
    #[serde(rename = "sync_remote_conflict")]
    SyncRemoteConflict,
    #[serde(rename = "sync_operation_unknown")]
    SyncOperationUnknown,
    #[serde(rename = "sync_operation_collision")]
    SyncOperationCollision,
    #[serde(rename = "sync_source_changed")]
    SyncSourceChanged,
    #[serde(rename = "sync_scope_changed")]
    SyncScopeChanged,
    #[serde(rename = "sync_endpoint_changed")]
    SyncEndpointChanged,
    #[serde(rename = "sync_payload_schema_unsupported")]
    SyncPayloadSchemaUnsupported,
    #[serde(rename = "sync_payload_invalid")]
    SyncPayloadInvalid,
    #[serde(rename = "sync_limit")]
    SyncLimit,
    #[serde(rename = "sync_remote_full")]
    SyncRemoteFull,
    #[serde(rename = "sync_auth_required")]
    SyncAuthRequired,
    #[serde(rename = "sync_network")]
    SyncNetwork,
    #[serde(rename = "sync_collection_missing")]
    SyncCollectionMissing,
    #[serde(rename = "sync_passphrase_required")]
    SyncPassphraseRequired,
    #[serde(rename = "sync_passphrase_invalid")]
    SyncPassphraseInvalid,
    #[serde(rename = "sync_plan_expired")]
    SyncPlanExpired,
    #[serde(rename = "sync_plan_changed")]
    SyncPlanChanged,
    #[serde(rename = "sync_local_changed")]
    SyncLocalChanged,
    #[serde(rename = "sync_backup_failed")]
    SyncBackupFailed,
    #[serde(rename = "sync_journal_invalid")]
    SyncJournalInvalid,
    #[serde(rename = "sync_local_state_io")]
    SyncLocalStateIo,
    #[serde(rename = "sync_cancelled")]
    SyncCancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Step {
    #[serde(rename = "observe")]
    Observe,
    #[serde(rename = "initialize")]
    Initialize,
    #[serde(rename = "source_guard")]
    SourceGuard,
    #[serde(rename = "payload_put")]
    PayloadPut,
    #[serde(rename = "payload_get")]
    PayloadGet,
    #[serde(rename = "commit_put")]
    CommitPut,
    #[serde(rename = "commit_get")]
    CommitGet,
    #[serde(rename = "verify")]
    Verify,
    #[serde(rename = "download_guard")]
    DownloadGuard,
    #[serde(rename = "apply_local")]
    ApplyLocal,
    #[serde(rename = "persist")]
    Persist,
    #[serde(rename = "decode")]
    Decode,
    #[serde(rename = "recovery")]
    Recovery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Retryability {
    #[serde(rename = "never")]
    Never,
    #[serde(rename = "after_refresh")]
    AfterRefresh,
    #[serde(rename = "same_operation_read_only")]
    SameOperationReadOnly,
    #[serde(rename = "same_operation_guarded")]
    SameOperationGuarded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Cause {
    #[serde(rename = "missing")]
    Missing,
    #[serde(rename = "unauthorized")]
    Unauthorized,
    #[serde(rename = "forbidden")]
    Forbidden,
    #[serde(rename = "timeout")]
    Timeout,
    #[serde(rename = "disconnected")]
    Disconnected,
    #[serde(rename = "tls")]
    Tls,
    #[serde(rename = "redirect")]
    Redirect,
    #[serde(rename = "invalid_listing")]
    InvalidListing,
    #[serde(rename = "invalid_href")]
    InvalidHref,
    #[serde(rename = "partial_response")]
    PartialResponse,
    #[serde(rename = "too_large")]
    TooLarge,
    #[serde(rename = "insufficient_storage")]
    InsufficientStorage,
    #[serde(rename = "server")]
    Server,
    #[serde(rename = "protocol")]
    Protocol,
    #[serde(rename = "invalid_json")]
    InvalidJson,
    #[serde(rename = "invalid_digest")]
    InvalidDigest,
    #[serde(rename = "missing_parent")]
    MissingParent,
    #[serde(rename = "cycle")]
    Cycle,
    #[serde(rename = "unsupported_schema")]
    UnsupportedSchema,
    #[serde(rename = "source_changed")]
    SourceChanged,
    #[serde(rename = "scope_changed")]
    ScopeChanged,
    #[serde(rename = "binding_changed")]
    BindingChanged,
    #[serde(rename = "local_io")]
    LocalIo,
    #[serde(rename = "backup")]
    Backup,
    #[serde(rename = "passphrase")]
    Passphrase,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "expired")]
    Expired,
    #[serde(rename = "collision")]
    Collision,
    #[serde(rename = "generation_changed")]
    GenerationChanged,
    #[serde(rename = "generation_ambiguous")]
    GenerationAmbiguous,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RemoteState {
    #[serde(rename = "empty")]
    Empty,
    #[serde(rename = "generation_empty")]
    GenerationEmpty,
    #[serde(rename = "ready")]
    Ready,
    #[serde(rename = "conflict")]
    Conflict,
    #[serde(rename = "initialization_ambiguous")]
    InitializationAmbiguous,
    #[serde(rename = "incomplete")]
    Incomplete,
    #[serde(rename = "invalid")]
    Invalid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OperationKind {
    #[serde(rename = "upload")]
    Upload,
    #[serde(rename = "download")]
    Download,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Phase {
    #[serde(rename = "prepared")]
    Prepared,
    #[serde(rename = "initialization_may_be_written")]
    InitializationMayBeWritten,
    #[serde(rename = "generation_verified")]
    GenerationVerified,
    #[serde(rename = "payload_may_be_written")]
    PayloadMayBeWritten,
    #[serde(rename = "payload_verified")]
    PayloadVerified,
    #[serde(rename = "commit_may_be_written")]
    CommitMayBeWritten,
    #[serde(rename = "settled")]
    Settled,
    #[serde(rename = "cancelled_before_write")]
    CancelledBeforeWrite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum WriteStep {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "generation")]
    Generation,
    #[serde(rename = "payload")]
    Payload,
    #[serde(rename = "commit")]
    Commit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StorageStatus {
    #[serde(rename = "not_attempted")]
    NotAttempted,
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "stored")]
    Stored,
    #[serde(rename = "invalid")]
    Invalid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LocalApplyStatus {
    #[serde(rename = "not_applicable")]
    NotApplicable,
    #[serde(rename = "not_attempted")]
    NotAttempted,
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "applied")]
    Applied,
    #[serde(rename = "invalid")]
    Invalid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Initialization {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "on_confirm")]
    OnConfirm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ProtectionMode {
    #[serde(rename = "plain")]
    Plain,
    #[serde(rename = "protected")]
    Protected,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RecoveryMode {
    #[serde(rename = "download")]
    Download,
    #[serde(rename = "export")]
    Export,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RecoveryPlanKind {
    #[serde(rename = "recovery_download")]
    RecoveryDownload,
    #[serde(rename = "recovery_export")]
    RecoveryExport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RecoveryOutcome {
    #[serde(rename = "applied")]
    Applied,
    #[serde(rename = "exported")]
    Exported,
    #[serde(rename = "unknown")]
    Unknown,
    #[serde(rename = "failed")]
    Failed,
}

/// No URL, password or key material crosses this boundary. L0 resolves the ID
/// against the original binding/revision; verify never resolves the active UI connection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionHandle {
    pub id: String,
    pub binding_id: Digest,
    pub revision: Digest,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SyncRelativePath(String);
impl SyncRelativePath {
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn generation(g: &str) -> SyncResult<Self> {
        require_uuid(g)?;
        Ok(Self(format!("config-v2-generation-{g}.json")))
    }
    pub fn data(g: &str, o: &str) -> SyncResult<Self> {
        require_uuid(g)?;
        require_uuid(o)?;
        Ok(Self(format!("config-v2-data-{g}-{o}.ocxdconf")))
    }
    pub fn commit(g: &str, o: &str) -> SyncResult<Self> {
        require_uuid(g)?;
        require_uuid(o)?;
        Ok(Self(format!("config-v2-commit-{g}-{o}.json")))
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ListLimits {
    pub max_entries: u32,
    pub max_xml_bytes: u64,
    pub timeout_ms: u32,
}
#[derive(Clone, Copy, Debug)]
pub struct ReadLimits {
    pub max_bytes: u64,
    pub timeout_ms: u32,
}
#[derive(Clone, Copy, Debug)]
pub struct WriteLimits {
    pub max_bytes: u64,
    pub timeout_ms: u32,
}
#[derive(Clone, Debug)]
pub struct ChildEntry {
    pub name: String,
    pub is_collection: bool,
    pub content_length: Option<u64>,
}
#[derive(Clone, Debug)]
pub struct ChildListing {
    pub entries: Vec<ChildEntry>,
    pub http_status: u16,
    pub response_bytes: usize,
}
pub struct FileRead {
    pub body: Vec<u8>,
    pub http_status: u16,
}
#[derive(Clone, Copy, Debug)]
pub struct PutResponse {
    pub http_status: u16,
}
#[derive(Clone, Copy, Debug)]
pub enum TransportKind {
    NotFound,
    Unauthorized,
    Forbidden,
    Timeout,
    Disconnected,
    Tls,
    Redirect,
    InvalidListing,
    InvalidHref,
    PartialResponse,
    TooLarge,
    InsufficientStorage,
    Server,
    Protocol,
}
#[derive(Clone, Copy, Debug)]
pub enum Delivery {
    NotSent,
    MayHaveBeenSent,
    ResponseReceived,
}
#[derive(Clone, Debug)]
pub struct SyncTransportError {
    pub kind: TransportKind,
    pub http_status: Option<u16>,
    pub delivery: Delivery,
}
#[async_trait]
pub trait ConfigSyncTransport: Send + Sync {
    async fn list_children(
        &self,
        connection: &ConnectionHandle,
        limits: ListLimits,
    ) -> Result<ChildListing, SyncTransportError>;
    async fn get_file(
        &self,
        connection: &ConnectionHandle,
        path: &SyncRelativePath,
        limits: ReadLimits,
    ) -> Result<FileRead, SyncTransportError>;
    async fn put_file(
        &self,
        connection: &ConnectionHandle,
        path: &SyncRelativePath,
        frozen_body: &[u8],
        limits: WriteLimits,
    ) -> Result<PutResponse, SyncTransportError>;
}
/// Payload bytes deliberately have no Debug/Serialize implementation. The codec
/// is provided by L0/L1; no model structure is interpreted in this module.
pub struct FrozenPayload {
    pub bytes: Vec<u8>,
    pub schema: PayloadSchemaRef,
    pub protection_mode: ProtectionMode,
}
pub struct PayloadInput {
    pub head: CommitRef,
    pub bytes: Vec<u8>,
}
pub trait PayloadCodec: Send + Sync {
    fn supported_schemas(&self) -> Vec<PayloadSchemaRef>;
    fn freeze_selected(
        &self,
        guard: &SourceGuard,
        baselines: &[PayloadInput],
        resolution: Option<&Resolution>,
    ) -> SyncResult<FrozenPayload>;
    fn inspect_and_validate(&self, bytes: &[u8], schema: &PayloadSchemaRef) -> SyncResult<()>;
}
/// Implementations must capture source facts consistently and must enforce
/// expected target + backup + once-only receipt in the same application transaction.
/// A crash after apply/export but before L2 receipt persistence must be recoverable
/// by repeating the SAME operation ID, without overwriting later local changes.
pub trait LocalApplication: Send + Sync {
    fn root_revision(&self) -> SyncResult<Digest>;
    fn current_source(&self) -> SyncResult<SourceGuard>;
    /// One fresh observation may prove both facts. The default preserves the
    /// independent owner APIs; production upload supplies a single capture.
    fn source_context(&self) -> SyncResult<(Digest, SourceGuard)> {
        Ok((self.root_revision()?, self.current_source()?))
    }
    fn target_digest(&self, scope_digest: &str) -> SyncResult<Digest>;
    fn apply_once(
        &self,
        operation_id: &str,
        bytes: &[u8],
        schema: &PayloadSchemaRef,
        scope_digest: &str,
        expected_target: &str,
        backup: bool,
    ) -> SyncResult<()>;
    fn export_once(
        &self,
        recovery_id: &str,
        bytes: &[u8],
        schema: &PayloadSchemaRef,
        scope_digest: &str,
    ) -> SyncResult<Uuid>;
}
#[derive(Clone, Debug)]
pub struct Budgets {
    pub payload_bytes: usize,
    pub document_bytes: usize,
    pub metadata_bytes: usize,
    pub generations: usize,
    pub commits: usize,
    pub heads: usize,
    pub listing_entries: usize,
    pub listing_bytes: usize,
    pub observation_bytes: usize,
    pub requests: usize,
    pub request_ms: u64,
    pub observation_ms: u64,
    pub listing_rounds: usize,
    pub get_attempts: usize,
    pub preview_ms: i64,
}
impl Default for Budgets {
    fn default() -> Self {
        Self {
            payload_bytes: 64 * 1024 * 1024,
            document_bytes: 32 * 1024 * 1024,
            metadata_bytes: 64 * 1024,
            generations: 64,
            commits: 4096,
            heads: 64,
            listing_entries: 10000,
            listing_bytes: 16 * 1024 * 1024,
            observation_bytes: 32 * 1024 * 1024,
            requests: 8192,
            request_ms: 30000,
            observation_ms: 90000,
            listing_rounds: 3,
            get_attempts: 3,
            preview_ms: 900000,
        }
    }
}
impl Budgets {
    pub fn validate(&self) -> SyncResult<()> {
        let d = Self::default();
        if self.payload_bytes == 0
            || self.payload_bytes > d.payload_bytes
            || self.document_bytes == 0
            || self.document_bytes > d.document_bytes
            || self.metadata_bytes == 0
            || self.metadata_bytes > d.metadata_bytes
            || self.generations == 0
            || self.generations > d.generations
            || self.commits == 0
            || self.commits > d.commits
            || self.heads == 0
            || self.heads > d.heads
            || self.listing_entries == 0
            || self.listing_entries > d.listing_entries
            || self.listing_bytes == 0
            || self.listing_bytes > d.listing_bytes
            || self.observation_bytes == 0
            || self.observation_bytes > d.observation_bytes
            || self.requests == 0
            || self.requests > d.requests
            || self.request_ms == 0
            || self.request_ms > d.request_ms
            || self.observation_ms == 0
            || self.observation_ms > d.observation_ms
            || self.listing_rounds == 0
            || self.listing_rounds > d.listing_rounds
            || self.get_attempts == 0
            || self.get_attempts > d.get_attempts
            || self.preview_ms <= 0
            || self.preview_ms > d.preview_ms
        {
            return Err(failure(ErrorCode::SyncLimit, Step::Observe));
        }
        Ok(())
    }
}
pub fn require_uuid(value: &str) -> SyncResult<()> {
    let parsed = uuid::Uuid::parse_str(value)
        .map_err(|_| failure(ErrorCode::SyncRemoteInvalid, Step::Decode))?;
    if parsed.get_version_num() != 4 || parsed.to_string() != value {
        return Err(failure(ErrorCode::SyncRemoteInvalid, Step::Decode));
    }
    Ok(())
}
pub fn require_digest(value: &str) -> SyncResult<()> {
    if value.len() != 71
        || !value.starts_with("sha256:")
        || !value[7..]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err(failure(ErrorCode::SyncRemoteInvalid, Step::Decode));
    }
    Ok(())
}
pub fn require_guard(value: &SourceGuard) -> SyncResult<()> {
    require_digest(&value.saved_source_digest)?;
    require_digest(&value.scope_digest)?;
    require_digest(&value.mcp_sources_digest)
}
pub fn action(kind: RecoveryKind) -> RecoveryAction {
    RecoveryAction {
        kind,
        operation_id: None,
        recovery_plan_id: None,
        generation: None,
        head: None,
    }
}
impl SyncFailure {
    pub fn for_operation(mut self, id: &str) -> Self {
        for a in &mut self.recovery_actions {
            if matches!(
                a.kind,
                RecoveryKind::VerifyOriginalUpload | RecoveryKind::ResumeOriginalUpload
            ) {
                a.operation_id = Some(id.into());
            }
        }
        self
    }
}
impl fmt::Display for SyncFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} at {:?}", self.code, self.step)
    }
}
impl std::error::Error for SyncFailure {}
pub fn transport_failure(e: SyncTransportError, step: Step) -> SyncFailure {
    let (code, cause) = match e.kind {
        TransportKind::NotFound => (
            if step == Step::Observe {
                ErrorCode::SyncCollectionMissing
            } else {
                ErrorCode::SyncListingIncomplete
            },
            Cause::Missing,
        ),
        TransportKind::Unauthorized => (ErrorCode::SyncAuthRequired, Cause::Unauthorized),
        TransportKind::Forbidden => (ErrorCode::SyncAuthRequired, Cause::Forbidden),
        TransportKind::TooLarge => (ErrorCode::SyncLimit, Cause::TooLarge),
        TransportKind::InsufficientStorage => {
            (ErrorCode::SyncRemoteFull, Cause::InsufficientStorage)
        }
        TransportKind::InvalidListing => (ErrorCode::SyncListingIncomplete, Cause::InvalidListing),
        TransportKind::InvalidHref => (ErrorCode::SyncRemoteInvalid, Cause::InvalidHref),
        TransportKind::PartialResponse => {
            (ErrorCode::SyncListingIncomplete, Cause::PartialResponse)
        }
        TransportKind::Timeout => (ErrorCode::SyncNetwork, Cause::Timeout),
        TransportKind::Disconnected => (ErrorCode::SyncNetwork, Cause::Disconnected),
        TransportKind::Tls => (ErrorCode::SyncNetwork, Cause::Tls),
        TransportKind::Redirect => (ErrorCode::SyncNetwork, Cause::Redirect),
        TransportKind::Server => (ErrorCode::SyncNetwork, Cause::Server),
        TransportKind::Protocol => (ErrorCode::SyncNetwork, Cause::Protocol),
    };
    let mut f = failure(code, step);
    f.cause = Some(cause);
    f.http_status = e.http_status.map(u64::from);
    f
}

pub fn failure(code: ErrorCode, step: Step) -> SyncFailure {
    let (retryability, kinds) = match code {
        ErrorCode::SyncGenerationAmbiguous => (
            Retryability::AfterRefresh,
            vec![
                RecoveryKind::PrepareRecoveryDownload,
                RecoveryKind::PrepareRecoveryExport,
                RecoveryKind::SelectEmptyDirectory,
                RecoveryKind::VerifyOriginalUpload,
            ],
        ),
        ErrorCode::SyncGenerationChanged => (
            Retryability::AfterRefresh,
            vec![
                RecoveryKind::VerifyOriginalUpload,
                RecoveryKind::RefreshRemote,
                RecoveryKind::RepreviewUpload,
            ],
        ),
        ErrorCode::SyncListingIncomplete => (
            Retryability::AfterRefresh,
            vec![RecoveryKind::RefreshRemote, RecoveryKind::RetryLater],
        ),
        ErrorCode::SyncRemoteInvalid => (
            Retryability::Never,
            vec![
                RecoveryKind::InspectErrorDetails,
                RecoveryKind::PrepareRecoveryExport,
            ],
        ),
        ErrorCode::SyncRemoteConflict => (
            Retryability::AfterRefresh,
            vec![
                RecoveryKind::ChooseConflictSource,
                RecoveryKind::RefreshRemote,
                RecoveryKind::VerifyOriginalUpload,
            ],
        ),
        ErrorCode::SyncOperationUnknown => (
            Retryability::SameOperationReadOnly,
            vec![
                RecoveryKind::VerifyOriginalUpload,
                RecoveryKind::RetryLater,
                RecoveryKind::ResumeOriginalUpload,
            ],
        ),
        ErrorCode::SyncOperationCollision => (
            Retryability::Never,
            vec![
                RecoveryKind::InspectErrorDetails,
                RecoveryKind::RepreviewUpload,
            ],
        ),
        ErrorCode::SyncSourceChanged => (
            Retryability::AfterRefresh,
            vec![
                RecoveryKind::RepreviewUpload,
                RecoveryKind::VerifyOriginalUpload,
            ],
        ),
        ErrorCode::SyncScopeChanged => (
            Retryability::AfterRefresh,
            vec![
                RecoveryKind::RepreviewUpload,
                RecoveryKind::VerifyOriginalUpload,
            ],
        ),
        ErrorCode::SyncEndpointChanged => (
            Retryability::AfterRefresh,
            vec![
                RecoveryKind::ReturnToOriginalConnection,
                RecoveryKind::RefreshRemote,
            ],
        ),
        ErrorCode::SyncPayloadSchemaUnsupported => {
            (Retryability::Never, vec![RecoveryKind::InspectErrorDetails])
        }
        ErrorCode::SyncPayloadInvalid => (
            Retryability::Never,
            vec![
                RecoveryKind::InspectErrorDetails,
                RecoveryKind::RepreviewUpload,
            ],
        ),
        ErrorCode::SyncLimit => (
            Retryability::AfterRefresh,
            vec![
                RecoveryKind::InspectErrorDetails,
                RecoveryKind::SelectEmptyDirectory,
            ],
        ),
        ErrorCode::SyncRemoteFull => (
            Retryability::SameOperationReadOnly,
            vec![
                RecoveryKind::FreeRemoteSpace,
                RecoveryKind::VerifyOriginalUpload,
            ],
        ),
        ErrorCode::SyncAuthRequired => (
            Retryability::AfterRefresh,
            vec![
                RecoveryKind::ReturnToOriginalConnection,
                RecoveryKind::RetryLater,
            ],
        ),
        ErrorCode::SyncNetwork => (
            Retryability::SameOperationReadOnly,
            vec![RecoveryKind::RetryLater, RecoveryKind::VerifyOriginalUpload],
        ),
        ErrorCode::SyncCollectionMissing => (
            Retryability::AfterRefresh,
            vec![RecoveryKind::ReturnToOriginalConnection],
        ),
        ErrorCode::SyncPassphraseRequired => (
            Retryability::AfterRefresh,
            vec![RecoveryKind::UnlockPayload],
        ),
        ErrorCode::SyncPassphraseInvalid => (
            Retryability::AfterRefresh,
            vec![RecoveryKind::UnlockPayload],
        ),
        ErrorCode::SyncPlanExpired => (
            Retryability::AfterRefresh,
            vec![RecoveryKind::RepreviewUpload, RecoveryKind::RefreshRemote],
        ),
        ErrorCode::SyncPlanChanged => (
            Retryability::AfterRefresh,
            vec![RecoveryKind::RepreviewUpload, RecoveryKind::RefreshRemote],
        ),
        ErrorCode::SyncLocalChanged => (
            Retryability::AfterRefresh,
            vec![
                RecoveryKind::RepreviewDownload,
                RecoveryKind::InspectLocalProtection,
            ],
        ),
        ErrorCode::SyncBackupFailed => (
            Retryability::AfterRefresh,
            vec![RecoveryKind::InspectLocalProtection],
        ),
        ErrorCode::SyncJournalInvalid => (
            Retryability::Never,
            vec![
                RecoveryKind::InspectLocalProtection,
                RecoveryKind::VerifyOriginalUpload,
            ],
        ),
        ErrorCode::SyncLocalStateIo => (
            Retryability::SameOperationReadOnly,
            vec![
                RecoveryKind::InspectLocalProtection,
                RecoveryKind::VerifyOriginalUpload,
            ],
        ),
        ErrorCode::SyncCancelled => (
            Retryability::AfterRefresh,
            vec![RecoveryKind::RepreviewUpload],
        ),
    };
    SyncFailure {
        code,
        step,
        retryability,
        cause: None,
        http_status: None,
        recovery_actions: kinds.into_iter().map(action).collect(),
    }
}
