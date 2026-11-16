//! W-L2 native boundaries for unchanged official 2.99.0 (r4 coverage decision).
//!
//! These are implementation contracts, not a new source of native authority.
//! Associated proof types must retain their original native objects. No proof
//! can be reconstructed by deserializing a receipt. Production stays fail-closed
//! until a Windows adapter implements and validates all four boundaries.
use super::official_lifecycle024::{ActionWindow, ServiceObservation, StopAccepted};
use crate::errors::AppResult;
use std::time::Instant;

pub const CONTRACT_VERSION: u32 = 3;
pub const OFFICIAL_PACKAGE_SHA256: &str =
    "b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c";
pub const OFFICIAL_TASK: &str = "opencodex-proxy";
pub const OFFICIAL_SCM_SERVICE: &str = "opencodex-proxy-native";

/// Query-only rights: READ_CONTROL | ACCESS_SYSTEM_SECURITY. Generic read/data
/// access is a separate content-capture need, never a substitute for SACL rights.
/// Do not enable a privilege, elevate, impersonate or repair an original ACL to
/// obtain these rights. A standard token may legitimately be unable to open it.
pub const FULL_DESCRIPTOR_QUERY_ACCESS: u32 = 0x0102_0000;
/// BACKUP_SECURITY_INFORMATION asks the OS for all parts of its descriptor.
/// The nine named projections (0x1ff) alone do not justify "complete" coverage.
/// Use the pinned file handle with GetSecurityInfo(SE_FILE_OBJECT); unsupported,
/// denied, omitted or unverified portions refuse capture, not an O/G/D fallback.
pub const FULL_DESCRIPTOR_SECURITY_INFORMATION: u32 = 0x0001_0000;
/// An independent diagnostic projection only; it cannot authorize custody,
/// restoration or ClosedEligibility even when its read succeeds unchanged.
pub const OWNER_GROUP_DACL_DIAGNOSTIC_INFORMATION: u32 = 0x0000_0007;

/// Explicit new-session policy, never selected as fallback after a full-SACL
/// read fails. No native adapter is switched by these declarations. Until the
/// new native capture/tree/custody/restore proofs exist, production still refuses.
pub const BOUNDED_SECURITY_SCOPE: &str = "windows-bounded-access-v1";
pub const BOUNDED_SECURITY_QUERY_ACCESS: u32 = 0x0002_0000; // READ_CONTROL
/// O/G/D + label + resource attributes + CAP scope + trust label + access filter.
/// Every projection must be queried separately and as the combined set. Check
/// QuerySecurityAccessMask and supported OS semantics; required unreadable or
/// unsupported projection refuses. Neither SACL(8) nor BACKUP(0x10000) is queried.
pub const BOUNDED_SECURITY_INFORMATION: u32 = 0x0000_01f7;
pub const BOUNDED_POLICY_PROJECTIONS: [u32; 5] = [0x10, 0x20, 0x40, 0x80, 0x100];
pub const ORIGINALS_RECORD_SCHEMA_V2: &str = "windows-client-originals-v2";
pub const CURRENT_REFERENCE_SCHEMA_V2: &str = "windows-client-current-v2";
/// r6 storage layout for NEW sessions only. Existing directories stay at their
/// original paths/IDs; ordinary files enter same-volume per-node custody. This
/// permits claim exclusion throughout each critical mutation without renaming
/// a directory containing the still-open claim. It is not a runtime capability.
pub const ANCHORED_STORAGE_STRATEGY: &str = "windows-anchored-directories-v1";
pub const ANCHORED_ORIGINALS_SCHEMA_V3: &str = "windows-client-originals-v3";
pub const ANCHORED_CURRENT_SCHEMA_V3: &str = "windows-client-current-v3";
/// Exact existing Codex coordinators, never a Grok/general SQLite exception.
/// Journals/WAL/SHM and absent-root retirement are not exempted by these names.
/// Existence, original identity, scoped security and release remain native facts.
pub const ANCHORED_CODEX_STABLE_FILES: [&str; 2] = [
    ".opencodex-native-main.claim.sqlite",
    ".opencodex-native-main.owner.sqlite",
];
/// This is an assurance scope, not a descriptor-preservation certificate. Native
/// code must retain the original object and independently read back each included
/// projection. Excluded audit-SACL/control and future extensions remain unknown;
/// they are never asserted absent, equal or fully restored, even on the same ID.
/// Old full-v1 and diagnostic partial records cannot mint this scope's eligibility.
pub const BOUNDED_SECURITY_EXCLUDED_ASSURANCE: &str = "audit-sacl-and-unrequested-security-unknown";

/// Original process generation, never a PID-only capability. The native adapter
/// must independently observe token SID and creation time from a retained handle.
#[derive(Debug, Eq, PartialEq)]
pub struct ProcessGeneration {
    pub pid: u32,
    pub creation_filetime: u64,
    pub token_user_sid: String,
}

/// Raw child outcome remains unchanged in every result, including diagnostics.
#[derive(Debug, Eq, PartialEq)]
pub enum ChildExit {
    Exited(u32),
    StillRunning,
    Unreadable,
}

/// Classification only. A diagnostic nonzero is neither service success nor
/// proof that a supervisor is absent. Unknown roles remain a veto.
#[derive(Debug, Eq, PartialEq)]
pub enum ChildRole {
    Launcher,
    Service,
    ScmRegistrationQuery,
    SchedulerRegistrationQuery,
    OtherRequired,
    Unknown,
}

/// Native original owner: current SID/Profile/D/H/I/source and original Job
/// generation. Not serializable; matching strings in a journal are insufficient.
pub trait NativeSession {
    fn check_current(&mut self, deadline: Instant) -> AppResult<()>;
}

/// Durable originals, including original absence, same-volume object custody,
/// security descriptor and file identity. All associated values are opaque,
/// non-Clone/non-serde in the adapter; no default implementation grants recovery.
/// r4 permits BOUNDED_SECURITY_SCOPE for NEW v2 sessions; r6 carries the same
/// assurance into new v3 anchored-storage sessions without converting old ones.
/// Complete
/// included projections + native custody and final restoration are still required;
/// an old diagnostic O/G/D read cannot authorize any mutation. Full-v1 records
/// keep their full-descriptor requirements and are never converted/downgraded.
/// Unreadable included information is not absent. A later read failure preserves
/// pending custody/Required, and bounded Closed never claims full-SACL restoration.
pub trait ClientOriginals {
    type Owner: NativeSession;
    type OriginalSession;
    type ReadySession;
    type PreparedRestore;
    type NativeNormalExit;
    type SupervisorAbsent;
    type ClosedEligibility;

    /// Persist and flush original custody before the first official mutation.
    fn capture(
        &mut self,
        owner: &mut Self::Owner,
        window: &ActionWindow,
    ) -> AppResult<Self::OriginalSession>;
    fn record_ready(
        &mut self,
        original: Self::OriginalSession,
        owner: &mut Self::Owner,
        ready: &ServiceObservation,
        deadline: Instant,
    ) -> AppResult<Self::ReadySession>;
    /// Bracket official teardown with the original generation and immutable
    /// stop deadline. Failure must leave durable custody recoverable, not drop it.
    fn pre_stop(
        &mut self,
        ready: Self::ReadySession,
        owner: &mut Self::Owner,
        stop_deadline: Instant,
    ) -> AppResult<Self::PreparedRestore>;
    /// Restore original objects and independently read back bytes, IDs, security,
    /// absence and the finite released-lock exception inside the same deadline.
    fn complete_stop(
        &mut self,
        prepared: Self::PreparedRestore,
        owner: &mut Self::Owner,
        native_exit: Self::NativeNormalExit,
        supervisor: Self::SupervisorAbsent,
        official: &StopAccepted,
        stop_deadline: Instant,
    ) -> AppResult<Self::ClosedEligibility>;
    /// A previously consumed/closed or failed generation never authorizes a
    /// different source/H/SID. Retry requires this explicit, revalidated boundary.
    fn authorize_successor(
        &mut self,
        closed: Self::ClosedEligibility,
        owner: &mut Self::Owner,
        window: &ActionWindow,
    ) -> AppResult<()>;
}

/// The out-of-process host is the original creator, not a later PID adopter.
/// Private SID pipe + OS client PID/FILETIME + bounded fresh challenge precede
/// any manager attach. Serialized rendezvous data grants no ownership rights.
/// Current ordinary-user acceptance requires an existing interactive logon and
/// native IsProcessInJob(current,NULL)==FALSE on bootstrap and the suspended
/// original broker. DETACHED_PROCESS is not Job escape/lifetime evidence.
/// Check before assigning the broker's own no-kill-on-close child Job; later
/// manager replacement must retain the same original broker process handle.
pub trait OwnerContinuity {
    type Owner: NativeSession;
    type ManagerConnection;
    type AttachedManager;

    fn authenticate_manager(
        &mut self,
        connection: Self::ManagerConnection,
        deadline: Instant,
    ) -> AppResult<Self::AttachedManager>;
    /// Owner and attached manager independently revalidate source/H/SID and the
    /// original host generation; handles are non-inheritable and never kill-on-drop.
    fn with_owner<T>(
        &mut self,
        manager: &Self::AttachedManager,
        deadline: Instant,
        operation: impl FnOnce(&mut Self::Owner) -> AppResult<T>,
    ) -> AppResult<T>;
    fn detach_manager(&mut self, manager: Self::AttachedManager) -> AppResult<()>;
}

/// Targeted read-only SCM/Task Scheduler queries and exact own artifacts, not
/// process-name scans, HTTP success, Job empty, or a diagnostic's exit code.
pub trait SupervisorAbsence {
    type Owner: NativeSession;
    type Absent;

    fn observe_absent(
        &mut self,
        owner: &mut Self::Owner,
        stop_deadline: Instant,
    ) -> AppResult<Self::Absent>;
    /// Requery at the final restoration boundary; no cross-action cached absence.
    fn recheck_absent(
        &mut self,
        owner: &mut Self::Owner,
        observed: &Self::Absent,
        stop_deadline: Instant,
    ) -> AppResult<()>;
}

/// Complete historical Job accounting, including short-lived children. Native
/// evidence binds command, image ID, parent generation, Job and pinned call-site.
/// No filename/exit-code allowlist and no nonzero-to-zero rewriting is permitted.
pub trait DiagnosticRoles {
    type Owner: NativeSession;
    type RetainedChild;
    type ProvenRole;
    type NormalExit;
    type SupervisorAbsent;

    fn classify(
        &mut self,
        owner: &mut Self::Owner,
        child: &Self::RetainedChild,
        deadline: Instant,
    ) -> AppResult<Self::ProvenRole>;
    /// A query's nonzero can only be evaluated as a completed diagnostic with
    /// exact provenance and independent native namespace evidence. It stays raw
    /// in the audit. Root/service/required nonzero and unknown roles always veto.
    fn complete_family(
        &mut self,
        owner: &mut Self::Owner,
        supervisor: &Self::SupervisorAbsent,
        stop_deadline: Instant,
    ) -> AppResult<Self::NormalExit>;
}
