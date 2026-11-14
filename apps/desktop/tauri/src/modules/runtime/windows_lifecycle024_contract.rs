//! W-L2 native boundaries for unchanged official 2.99.0.
//!
//! These are implementation contracts, not a new source of native authority.
//! Associated proof types must retain their original native objects. No proof
//! can be reconstructed by deserializing a receipt. Production stays fail-closed
//! until a Windows adapter implements and validates all four boundaries.
use super::official_lifecycle024::{ActionWindow, ServiceObservation, StopAccepted};
use crate::errors::AppResult;
use std::time::Instant;

pub const CONTRACT_VERSION: u32 = 1;
pub const OFFICIAL_PACKAGE_SHA256: &str =
    "b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c";
pub const OFFICIAL_TASK: &str = "opencodex-proxy";
pub const OFFICIAL_SCM_SERVICE: &str = "opencodex-proxy-native";

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
