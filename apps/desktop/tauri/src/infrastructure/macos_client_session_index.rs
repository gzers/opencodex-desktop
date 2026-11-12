//! Desktop-owned exclusive current-session index. Independent references are
//! committed here before another application/CLI action may observe them.
//! No directory scanning, backup discovery or implicit recovery after a crash.
//! This index is not service ownership or a client mutation permit.
use super::session::References;
use super::*;
use crate::state::ProcessContext;
use rusqlite::{params, Connection, OptionalExtension};
use std::time::Instant;

#[path = "macos_client_root_binding.rs"]
mod root_binding;
const FILE: &str = "client-session-index.sqlite";
const MAX_INDEX: u64 = 16 * 1024 * 1024;
const TABLES: [(&str, &str); 3] = [
    ("binding", "CREATE TABLE binding(singleton INTEGER PRIMARY KEY CHECK(singleton=1), document BLOB NOT NULL)"),
    ("current_session", "CREATE TABLE current_session(singleton INTEGER PRIMARY KEY CHECK(singleton=1), revision INTEGER NOT NULL CHECK(revision>0), document BLOB NOT NULL)"),
    ("events", "CREATE TABLE events(revision INTEGER PRIMARY KEY, document BLOB NOT NULL)"),
];
#[path = "macos_client_session_observation.rs"]
pub(super) mod observation;
pub use observation::inspect;
/// Failure-response restriction only; this does not establish process state.
pub fn unresolved_after_failure(context: &ProcessContext, data: &Path) -> bool {
    match inspect(context, data) {
        Ok(None) => false,
        Ok(Some(h)) => !matches!(
            h.state,
            State::Ready
                | State::Restored
                | State::Unchanged
                | State::CancelledBeforeCapture
                | State::CancelledCapture
        ),
        Err(_) => true,
    }
}
#[path = "macos_client_recovery_continue.rs"]
pub mod continuation;
#[path = "macos_client_recovery_finalize.rs"]
pub mod finalization;
#[path = "macos_client_missing_plan.rs"]
pub mod missing_plan;
#[path = "macos_client_prepare_cancel.rs"]
pub mod preparation;
#[path = "macos_client_session_preview.rs"]
pub mod preview;
#[path = "macos_client_recovery_rearm.rs"]
pub mod rearm;
#[path = "macos_client_recovery_rollback.rs"]
pub mod rollback;
#[path = "macos_client_unchanged.rs"]
pub mod unchanged;
#[path = "macos_client_vault_history.rs"]
mod vault_history;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Preparing,
    CaptureReserved,
    Captured,
    Ready,
    StopPrepared,
    Stopped,
    RestorePrepared,
    RecoveryPrepared,
    RestoreRolledBack,
    RootRetirementPrepared,
    RootAbsent,
    Restored,
    Unchanged,
    CancelledBeforeCapture,
    CancelledCapture,
    Failed,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Head {
    pub revision: u64,
    pub ticket: String,
    pub state: State,
    pub references: Option<References>,
    pub restore_plan: Option<RestorePlan>,
    pub completion_sha256: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub root_retirement: Option<RootRetirement>,
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RootRetirement {
    pub source_revision: u64,
    pub plan: RestorePlan,
}
/// Borrow of the actually held independent index, not a serializable Head.
/// Only record verification below constructs it; its current CAS generation
/// is rechecked throughout movement, before any root terminal close.
pub struct RootRetirementRegistration<'a> {
    index: &'a Index,
    prepared: Head,
}
impl RootRetirementRegistration<'_> {
    pub(crate) fn require(&self, source: &Head, plan: &RestorePlan) -> io::Result<()> {
        let Some(movement) = self.prepared.root_retirement.as_ref() else {
            return Err(denied("root plan not independently registered"));
        };
        if self.prepared.state != State::RootRetirementPrepared
            || movement.source_revision != source.revision
            || movement.plan != *plan
            || self.prepared.ticket != source.ticket
            || self.prepared.references != source.references
            || self.prepared.restore_plan != source.restore_plan
            || self.index.head()?.as_ref() != Some(&self.prepared)
        {
            return Err(denied("current root retirement registration changed"));
        }
        let raw: Vec<u8> = self
            .index
            .connection()?
            .query_row(
                "SELECT document FROM events WHERE revision=?1",
                [movement.source_revision],
                |r| r.get(0),
            )
            .map_err(database_error)?;
        let registered: Head = decode(&raw)?;
        if registered != *source {
            return Err(denied("root registration source event differs"));
        }
        Ok(())
    }
    pub(crate) fn plan(&self) -> io::Result<&RestorePlan> {
        self.prepared
            .root_retirement
            .as_ref()
            .map(|r| &r.plan)
            .ok_or_else(|| denied("root registration plan missing"))
    }
}
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RestorePlan {
    pub id: String,
    pub sha256: String,
}
impl std::fmt::Debug for Head {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Head")
            .field("revision", &self.revision)
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}
fn valid_hash(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
}
pub(super) fn validate(head: &Head) -> io::Result<()> {
    let root_phase = matches!(
        head.state,
        State::RootRetirementPrepared | State::RootAbsent
    );
    if root_phase != head.root_retirement.is_some()
        || head.root_retirement.as_ref().is_some_and(|r| {
            r.source_revision == 0
                || r.source_revision >= head.revision
                || uuid::Uuid::parse_str(&r.plan.id)
                    .map(|v| v.to_string() != r.plan.id)
                    .unwrap_or(true)
                || !valid_hash(&r.plan.sha256)
        })
    {
        return Err(denied(
            "root retirement reference does not match current phase",
        ));
    }
    if matches!(
        head.state,
        State::RestorePrepared
            | State::RecoveryPrepared
            | State::Restored
            | State::RestoreRolledBack
    ) != head.restore_plan.is_some()
        && !root_phase
        || matches!(
            head.state,
            State::Restored
                | State::RestoreRolledBack
                | State::RootAbsent
                | State::Unchanged
                | State::CancelledBeforeCapture
                | State::CancelledCapture
        ) != head.completion_sha256.is_some()
        || head
            .completion_sha256
            .as_ref()
            .is_some_and(|s| !valid_hash(s))
        || head.restore_plan.as_ref().is_some_and(|plan| {
            uuid::Uuid::parse_str(&plan.id)
                .map(|v| v.to_string() != plan.id)
                .unwrap_or(true)
                || !valid_hash(&plan.sha256)
        })
    {
        return Err(denied(
            "restore plan reference missing or unexpected for current phase",
        ));
    }
    if head.revision == 0
        || head.revision > i64::MAX as u64
        || uuid::Uuid::parse_str(&head.ticket)
            .map(|v| v.to_string() != head.ticket)
            .unwrap_or(true)
    {
        return Err(denied("invalid current session identity"));
    }
    let Some(refs) = &head.references else {
        return if matches!(
            head.state,
            State::Preparing
                | State::CaptureReserved
                | State::Failed
                | State::CancelledBeforeCapture
                | State::CancelledCapture
        ) {
            Ok(())
        } else {
            Err(denied("current session original references missing"))
        };
    };
    if uuid::Uuid::parse_str(&refs.attempt)
        .map(|v| v.to_string() != refs.attempt)
        .unwrap_or(true)
        || !valid_hash(&refs.original_sha256)
        || !valid_hash(&refs.scope_sha256)
        || refs.live_sha256.as_ref().is_some_and(|v| !valid_hash(v))
    {
        return Err(denied("invalid independent original references"));
    }
    let order = [Phase::Ready, Phase::PreStop, Phase::Stopped];
    if refs.phases.len() > 3
        || refs
            .phases
            .iter()
            .enumerate()
            .any(|(i, (phase, hash))| *phase != order[i] || !valid_hash(hash))
    {
        return Err(denied("current session phase order or hash invalid"));
    }
    let expected = match head.state {
        State::Captured => Some((0, false)),
        State::Ready => Some((1, true)),
        State::StopPrepared => Some((2, true)),
        State::Stopped | State::RestorePrepared => Some((3, true)),
        State::RecoveryPrepared => {
            if !matches!(refs.phases.len(), 1 | 2) {
                return Err(denied("current recovery prefix incomplete"));
            }
            Some((refs.phases.len(), true))
        }
        State::Restored | State::RestoreRolledBack => {
            if !matches!(refs.phases.len(), 1..=3) {
                return Err(denied("terminal recovery prefix incomplete"));
            }
            Some((refs.phases.len(), true))
        }
        State::Preparing | State::CaptureReserved => {
            return Err(denied("unpublished originals in preparing index"))
        }
        State::RootRetirementPrepared | State::RootAbsent => {
            if refs.phases.is_empty() {
                Some((0, false))
            } else {
                Some((refs.phases.len(), true))
            }
        }
        State::Failed
        | State::Unchanged
        | State::CancelledBeforeCapture
        | State::CancelledCapture => None,
    };
    if expected
        .is_some_and(|(n, live)| refs.phases.len() != n || refs.live_sha256.is_some() != live)
    {
        return Err(denied("current session phase evidence is incomplete"));
    }
    Ok(())
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> io::Result<T> {
    if bytes.len() > 1024 * 1024 {
        return Err(denied("session index payload over limit"));
    }
    serde_json::from_value(
        parse_strict_json_v2(bytes).map_err(|_| denied("session index JSON is invalid"))?,
    )
    .map_err(|_| denied("session index fields invalid"))
}
fn database_error(_: rusqlite::Error) -> io::Error {
    denied("current session index unavailable; original state retained")
}
fn validate_schema(connection: &Connection) -> io::Result<()> {
    let mut statement = connection
        .prepare("SELECT name,type,sql FROM sqlite_schema")
        .map_err(database_error)?;
    let definitions: Vec<(String, String, String)> = statement
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .map_err(database_error)?
        .collect::<Result<_, _>>()
        .map_err(database_error)?;
    if definitions.len() != TABLES.len()
        || definitions.iter().any(|(name, kind, sql)| {
            kind != "table" || !TABLES.iter().any(|(n, s)| name == n && sql == s)
        })
    {
        return Err(denied("unknown index schema; not adopted"));
    }
    Ok(())
}
fn read_head(connection: &Connection) -> io::Result<Option<Head>> {
    let row: Option<(u64, Vec<u8>)> = connection
        .query_row(
            "SELECT revision,document FROM current_session WHERE singleton=1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()
        .map_err(database_error)?;
    let Some((revision, bytes)) = row else {
        let events: u64 = connection
            .query_row("SELECT count(*) FROM events", [], |r| r.get(0))
            .map_err(database_error)?;
        return if events == 0 {
            Ok(None)
        } else {
            Err(denied("current references missing with retained events"))
        };
    };
    let head: Head = decode(&bytes)?;
    validate(&head)?;
    if head.revision != revision {
        return Err(denied("session index revision mismatch"));
    }
    let event: Vec<u8> = connection
        .query_row(
            "SELECT document FROM events WHERE revision=?1",
            [revision],
            |r| r.get(0),
        )
        .map_err(database_error)?;
    if event != bytes {
        return Err(denied("session event and current references differ"));
    }
    let latest: u64 = connection
        .query_row("SELECT max(revision) FROM events", [], |r| r.get(0))
        .map_err(database_error)?;
    if latest != revision {
        return Err(denied(
            "current references are not the latest committed event",
        ));
    }
    Ok(Some(head))
}
fn scope(
    context: &ProcessContext,
    data: &PinnedDirectory,
    client: &PinnedDirectory,
) -> io::Result<Vec<u8>> {
    scope_at(context, data, client, client.path())
}
fn scope_at(
    context: &ProcessContext,
    data: &PinnedDirectory,
    client: &PinnedDirectory,
    logical_client: &Path,
) -> io::Result<Vec<u8>> {
    let profile = PinnedDirectory::open(&context.working_directory)?;
    let home = PinnedDirectory::open(&context.opencodex_home)?;
    encode(&(
        unsafe { libc::geteuid() },
        data.path(),
        data.identity(),
        profile.path(),
        profile.identity(),
        home.path(),
        home.identity(),
        logical_client,
        client.identity(),
    ))
}

/// Each reference update is its own durable transaction. The separate native
/// lease on this lock file serializes the ENTIRE application action; dropping
/// an IPC observer cannot publish references or release a still-running owner.
pub struct Index {
    connection: Option<Connection>,
    pin: CoordinationFilePin,
    action_file: File,
    action_parent: PinnedDirectory,
    action_identity: Identity,
    action_protection: Protection,
    binding: Vec<u8>,
    data: PinnedDirectory,
    client: PinnedDirectory,
    profile: PathBuf,
    context_home: PathBuf,
    root_registration: Option<super::root_creation::References>,
    root_created_fd: Option<File>,
    root_creation_deadline: Option<Instant>,
}
/// Borrowed independent index authority, never constructed from capture files.
pub struct CaptureReservation<'a> {
    index: &'a Index,
    head: Head,
}
impl CaptureReservation<'_> {
    pub(super) fn require(&self, data: &Path, client: &Path, profile: &Path) -> io::Result<()> {
        if self.head.state != State::CaptureReserved
            || self.head.references.is_some()
            || self.index.head()?.as_ref() != Some(&self.head)
            || self.index.data.path() != data
            || self.index.client.path() != client
            || self.index.profile != profile
        {
            return Err(denied("capture reservation/index/root changed"));
        }
        Ok(())
    }
    pub(super) fn root_absence(&self) -> io::Result<Option<super::root_creation::References>> {
        self.require(
            self.index.data.path(),
            self.index.client.path(),
            &self.index.profile,
        )?;
        if self.index.root_registration.is_some()
            && (self.index.root_created_fd.is_none()
                || self
                    .index
                    .root_creation_deadline
                    .is_none_or(|d| Instant::now() >= d))
        {
            return Err(denied("first root capture lost original creation custody"));
        }
        Ok(self.index.root_registration.clone())
    }
    pub fn reopen_root_originals(
        &self,
        original_sha256: &str,
        phases: &[(Phase, String)],
    ) -> io::Result<Originals> {
        let origin = self
            .root_absence()?
            .ok_or_else(|| denied("root-aware reopen requires independently registered absence"))?;
        Originals::open_profile_origin(
            self.index.data.path(),
            self.index.client.path(),
            &self.index.profile,
            super::OriginalKeys {
                attempt: self.attempt(),
                original_sha256,
                expected_phases: phases,
            },
            Some(origin),
        )
    }
    pub(super) fn attempt(&self) -> &str {
        &self.head.ticket
    }
}
impl Index {
    pub fn data_root(&self) -> &Path {
        self.data.path()
    }
    pub fn control_root(&self) -> io::Result<PathBuf> {
        self.check()?;
        Ok(self
            .action_parent
            .child_private("client-coordination", true)?
            .path)
    }
    pub fn open(
        context: &ProcessContext,
        data_root: &Path,
        client_root: &Path,
    ) -> io::Result<Self> {
        Self::open_inner(context, data_root, client_root, true, None, &|_| Ok(()))
    }
    pub fn open_existing(
        context: &ProcessContext,
        data_root: &Path,
        client_root: &Path,
    ) -> io::Result<Self> {
        Self::open_inner(context, data_root, client_root, false, None, &|_| Ok(()))
    }
    /// Registers original absence and Preparing atomically, retaining the same
    /// creation action flock. It does NOT authorize ordinary capture yet.
    pub fn register_first_root(
        context: &ProcessContext,
        data: &Path,
        root: super::root_creation::PublishedRoot,
    ) -> io::Result<(Self, Head)> {
        Self::register_first_root_checked(context, data, root, &|_| Ok(()))
    }
    fn register_first_root_checked(
        context: &ProcessContext,
        data: &Path,
        root: super::root_creation::PublishedRoot,
        checkpoint: &dyn Fn(&str) -> io::Result<()>,
    ) -> io::Result<(Self, Head)> {
        let client = root.path().to_path_buf();
        let index = Self::open_inner(context, data, &client, true, Some(root), checkpoint)?;
        let head = index
            .head()?
            .ok_or_else(|| denied("first root has no atomic Preparing event"))?;
        if head.state != State::Preparing
            || head.revision != 1
            || index
                .root_registration
                .as_ref()
                .is_none_or(|r| r.id != head.ticket)
        {
            return Err(denied("first root initial event differs"));
        }
        Ok((index, head))
    }
    fn open_inner(
        context: &ProcessContext,
        data_root: &Path,
        client_root: &Path,
        allow_create: bool,
        root: Option<super::root_creation::PublishedRoot>,
        checkpoint: &dyn Fn(&str) -> io::Result<()>,
    ) -> io::Result<Self> {
        let data = PinnedDirectory::open(data_root)?;
        data.private()?;
        let physical = match std::fs::symlink_metadata(client_root) {
            Err(e) if e.kind() == io::ErrorKind::NotFound && root.is_none() => {
                if client_root != context.working_directory.join(".codex") {
                    return Err(denied(
                        "missing client path is not this profile logical root",
                    ));
                }
                observation::physical_root(context, data_root)?
            }
            _ => client_root.to_path_buf(),
        };
        let client = PinnedDirectory::open(&physical)?;
        let overlap_path = if physical != client_root {
            client_root
        } else {
            physical.as_path()
        };
        if data_root.starts_with(overlap_path) || overlap_path.starts_with(data_root) {
            return Err(denied("session index/client roots overlap"));
        }
        let manager = data.child_private("manager-state", allow_create)?;
        let base_binding = if super::root_creation::has_creation_evidence(&manager)? {
            scope_at(
                context,
                &data,
                &client,
                &context.working_directory.join(".codex"),
            )?
        } else {
            scope(context, &data, &client)?
        };
        let transfer = root
            .map(|root| root.into_transfer(&data, &context.working_directory, &client))
            .transpose()?;
        let origin = transfer.as_ref().map(|t| t.references.clone());
        let transfer_deadline = transfer.as_ref().map(|t| t.deadline);
        let binding = root_binding::encode_binding(&base_binding, origin.as_ref())?;
        if origin.is_none() && observation::entry(&manager, FILE)?.is_none() {
            super::root_creation::require_no_pending_root(&manager)?;
        }

        if observation::entry(&manager, FILE)?.is_none()
            && observation::entry(&manager, "client-originals")?.is_some()
        {
            return Err(denied(
                "originals without an index cannot initialize a new baseline",
            ));
        }
        let (action_file, action_identity, action_protection, root_created_fd) =
            if let Some(transfer) = transfer {
                if observation::action_pin(&manager, &transfer.action)?
                    != (
                        transfer.action_identity.clone(),
                        transfer.action_protection.clone(),
                    )
                {
                    return Err(denied("transferred root action changed"));
                }
                checkpoint("root_lock_transferred")?;
                (
                    transfer.action,
                    transfer.action_identity,
                    transfer.action_protection,
                    Some(transfer.created),
                )
            } else {
                let action_name = "client-session-action.lock";
                let action_cname = name(OsStr::new(action_name))?;
                let fd = unsafe {
                    libc::openat(
                        manager.file().as_raw_fd(),
                        action_cname.as_ptr(),
                        libc::O_RDWR
                            | if allow_create { libc::O_CREAT } else { 0 }
                            | libc::O_NOFOLLOW
                            | libc::O_CLOEXEC
                            | libc::O_NONBLOCK,
                        0o600,
                    )
                };
                if fd < 0 {
                    return Err(io::Error::last_os_error());
                }
                let action_file = unsafe { File::from_raw_fd(fd) };
                let action_stat = stat(&action_file)?;
                let action_protection = protection(&action_file, &action_stat)?;
                if action_stat.st_nlink != 1
                    || action_stat.st_mode & libc::S_IFMT != libc::S_IFREG
                    || action_stat.st_uid != unsafe { libc::geteuid() }
                    || action_stat.st_mode & 0o777 != 0o600
                    || acl(&action_file)?.is_some()
                    || action_protection.flags != 0
                    || action_protection
                        .xattrs
                        .iter()
                        .any(|(k, v)| k != b"com.apple.provenance" || v.len() > 64)
                {
                    return Err(denied("current session action lock unsafe; not repaired"));
                }
                action_file
                    .try_lock()
                    .map_err(|_| denied("another current session action is running"))?;
                let action_identity = identity(&action_stat);
                // Reopen/check this separate action lock BEFORE keeping its own handle;
                // this lock uses flock, not the SQLite fcntl locks on the index inode.
                let check = open_at(manager.file(), OsStr::new(action_name), false)?;
                if identity(&stat(&check)?) != action_identity {
                    return Err(denied("session action lock redirected"));
                }
                drop(check);
                (action_file, action_identity, action_protection, None)
            };
        let path = manager.path().join(FILE);
        let preserve_root_initializer =
            origin.is_none() && super::root_creation::has_creation_evidence(&manager)?;
        if preserve_root_initializer {
            let entry = observation::entry(&manager, FILE)?
                .ok_or_else(|| denied("unregistered root initializer has no index"))?;
            if entry.st_size < 100 || entry.st_size as u64 > MAX_INDEX {
                return Err(denied("uncommitted first-root initializer is preserved"));
            }
            for suffix in ["-journal", "-wal", "-shm"] {
                if observation::entry(&manager, &format!("{FILE}{suffix}"))?.is_some() {
                    return Err(denied(
                        "first-root initializer sidecar is preserved; no ordinary recovery",
                    ));
                }
            }
        }
        let mut created = false;
        match capture_file(&manager, FILE, MAX_INDEX as usize)? {
            Image::Absent => {
                if !allow_create {
                    return Err(denied("recovery index missing; not initialized"));
                }
                let file = crate::infrastructure::platform::create_new_private_file(&path, 0o600)?;
                file.sync_all()?;
                drop(file);
                manager.sync()?;
                created = true;
            }
            Image::File { bytes, .. } if bytes.is_empty() => {
                if !allow_create {
                    return Err(denied("recovery index empty; not initialized"));
                }
                created = true;
            }
            Image::File { .. } => {}
        }
        for suffix in ["-journal", "-wal", "-shm"] {
            match capture_file(&manager, &format!("{FILE}{suffix}"), MAX_INDEX as usize)? {
                Image::Absent => {}
                Image::File { protection, .. }
                    if protection.uid == unsafe { libc::geteuid() }
                        && protection.mode == 0o600
                        && protection.acl.is_none() => {}
                _ => return Err(denied("unsafe existing index sidecar")),
            }
        }
        let pin = CoordinationFilePin::open(&path)?;
        let connection = Connection::open_with_flags(
            &path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
                | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW
                | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(database_error)?;
        connection
            .busy_timeout(std::time::Duration::ZERO)
            .map_err(database_error)?;
        connection
            .execute_batch("PRAGMA synchronous=FULL; BEGIN IMMEDIATE")
            .map_err(database_error)?;
        let version: i64 = connection
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .map_err(database_error)?;
        let mut root_registration = None;
        let mut actual_binding = Vec::new();
        let setup = (|| -> io::Result<()> {
            if version == 0 && created {
                let tables: i64 = connection
                    .query_row(
                        "SELECT count(*) FROM sqlite_master WHERE type='table'",
                        [],
                        |r| r.get(0),
                    )
                    .map_err(database_error)?;
                if tables != 0 {
                    return Err(denied("unknown nonempty session index is not adopted"));
                }
                for (_, sql) in TABLES {
                    connection.execute_batch(sql).map_err(database_error)?;
                }
                connection
                    .execute_batch("PRAGMA user_version=3")
                    .map_err(database_error)?;
                connection
                    .execute(
                        "INSERT INTO binding(singleton,document) VALUES(1,?1)",
                        [&binding],
                    )
                    .map_err(database_error)?;
                if let Some(refs) = &origin {
                    let head = Head {
                        revision: 1,
                        ticket: refs.id.clone(),
                        state: State::Preparing,
                        references: None,
                        restore_plan: None,
                        completion_sha256: None,
                        root_retirement: None,
                    };
                    validate(&head)?;
                    let bytes = encode(&head)?;
                    connection
                        .execute(
                            "INSERT INTO events(revision,document) VALUES(1,?1)",
                            [&bytes],
                        )
                        .map_err(database_error)?;
                    connection.execute("INSERT INTO current_session(singleton,revision,document) VALUES(1,1,?1)",[&bytes]).map_err(database_error)?;
                }
            } else if version != 3 {
                return Err(denied(
                    "unsupported session index version or abandoned initializer",
                ));
            }
            validate_schema(&connection)?;
            let stored: Vec<u8> = connection
                .query_row("SELECT document FROM binding WHERE singleton=1", [], |r| {
                    r.get(0)
                })
                .map_err(database_error)?;
            let current = read_head(&connection)?;
            let (resolved, registered) = root_binding::resolve(
                context,
                &data,
                &manager,
                &stored,
                current.as_ref(),
                &connection,
            )?;
            if resolved.identity() != client.identity() {
                return Err(denied("indexed root generation changed during acquisition"));
            }
            root_registration = registered;
            if origin
                .as_ref()
                .is_some_and(|expected| root_registration.as_ref() != Some(expected))
            {
                return Err(denied("first root registration source changed"));
            }
            if let Some(refs) = &root_registration {
                root_binding::verify_initial_event(&connection, refs)?;
            }
            actual_binding = stored;
            if root_registration.is_some() && read_head(&connection)?.is_none() {
                return Err(denied("first root registration lacks its Preparing event"));
            }
            Ok(())
        })();
        if let Err(error) = setup {
            let _ = connection.execute_batch("ROLLBACK");
            return Err(error);
        }
        if origin.is_some() {
            checkpoint("before_root_index_commit")?;
        }
        if transfer_deadline.is_some_and(|d| Instant::now() >= d) {
            return Err(denied("root index transfer expired"));
        }
        connection.execute_batch("COMMIT").map_err(database_error)?;
        if origin.is_some() {
            checkpoint("root_index_committed")?;
        }
        pin.recheck()?;
        let result = Self {
            connection: Some(connection),
            pin,
            action_file,
            action_parent: manager,
            action_identity,
            action_protection,
            binding: actual_binding,
            data,
            client,
            profile: context.working_directory.clone(),
            context_home: context.opencodex_home.clone(),
            root_registration,
            root_created_fd,
            root_creation_deadline: transfer_deadline,
        };
        result.head()?;
        if transfer_deadline.is_some_and(|d| Instant::now() >= d) {
            return Err(denied("root index registration completed after deadline"));
        }
        Ok(result)
    }
    fn connection(&self) -> io::Result<&Connection> {
        self.connection
            .as_ref()
            .ok_or_else(|| denied("session index closed"))
    }
    fn check(&self) -> io::Result<()> {
        self.checked_head().map(|_| ())
    }
    fn checked_head(&self) -> io::Result<Option<Head>> {
        let observed_head = read_head(self.connection()?)?;
        self.data.private()?;
        if self.root_registration.is_none() {
            self.client.recheck()?;
        }
        if let Some(refs) = &self.root_registration {
            let stored: Vec<u8> = self
                .connection()?
                .query_row("SELECT document FROM binding WHERE singleton=1", [], |r| {
                    r.get(0)
                })
                .map_err(database_error)?;
            if stored != self.binding {
                return Err(denied("registered root binding changed during action"));
            }
            let context = ProcessContext {
                runtime:
                    crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(),
                working_directory: self.profile.clone(),
                opencodex_home: self.context_home.clone(),
            };
            let current = &observed_head;
            let (resolved, origin) = root_binding::resolve(
                &context,
                &self.data,
                &self.action_parent,
                &stored,
                current.as_ref(),
                self.connection()?,
            )?;
            if resolved.identity() != self.client.identity() || origin.as_ref() != Some(refs) {
                return Err(denied("indexed current root or origin changed"));
            }
            if let Some(fd) = &self.root_created_fd {
                if identity(&stat(fd)?) != *self.client.identity() {
                    return Err(denied("original root creation descriptor changed"));
                }
            }
        }
        self.action_parent.private()?;
        self.pin.recheck()?;
        if identity(&stat(&self.action_file)?) != self.action_identity {
            return Err(denied("session action owner changed"));
        }
        if protection(&self.action_file, &stat(&self.action_file)?)? != self.action_protection {
            return Err(denied(
                "session action lock permissions changed; not repaired",
            ));
        }
        let mut current = std::mem::MaybeUninit::uninit();
        let path = name(OsStr::new("client-session-action.lock"))?;
        if unsafe {
            libc::fstatat(
                self.action_parent.file().as_raw_fd(),
                path.as_ptr(),
                current.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } != 0
        {
            return Err(io::Error::last_os_error());
        }
        let current = unsafe { current.assume_init() };
        if identity(&current) != self.action_identity
            || current.st_mode & libc::S_IFMT != libc::S_IFREG
            || current.st_nlink != 1
            || current.st_uid != unsafe { libc::geteuid() }
            || current.st_mode & 0o777 != 0o600
        {
            return Err(denied("session action lock generation or access changed"));
        }
        let mut moved = 1i32;
        let connection = self.connection()?;
        let result = unsafe {
            rusqlite::ffi::sqlite3_file_control(
                connection.handle(),
                c"main".as_ptr(),
                rusqlite::ffi::SQLITE_FCNTL_HAS_MOVED,
                (&mut moved as *mut i32).cast(),
            )
        };
        if result != rusqlite::ffi::SQLITE_OK || moved != 0 {
            return Err(denied("session index native database changed"));
        }
        if read_head(self.connection()?)? != observed_head {
            return Err(denied("session head changed during checked observation"));
        }
        if self.root_registration.is_some() && observed_head.is_none() {
            return Err(denied("registered first root session disappeared"));
        }
        Ok(observed_head)
    }
    pub fn head(&self) -> io::Result<Option<Head>> {
        self.checked_head()
    }
    pub fn registered_root_originals(&self, expected: &Head) -> io::Result<Originals> {
        if self.head()?.as_ref() != Some(expected) {
            return Err(denied("root Original references changed before inspection"));
        }
        let origin = self
            .root_registration
            .as_ref()
            .ok_or_else(|| denied("ordinary existing-root index has no original absence"))?;
        root_binding::verify_capture(
            &self.binding,
            origin,
            &self.data,
            &self.profile,
            &self.client,
            expected,
        )?
        .ok_or_else(|| denied("root Original is not independently registered"))
    }
    pub fn has_original_root_absence(&self) -> bool {
        self.root_registration.is_some()
    }
    pub fn archived_root_originals(&self, expected: &Head) -> io::Result<Originals> {
        if expected.state != State::RootAbsent || self.head()?.as_ref() != Some(expected) {
            return Err(denied("root archive terminal not current"));
        }
        let root = self
            .root_registration
            .as_ref()
            .ok_or_else(|| denied("root archive has no independent origin"))?;
        let movement = expected
            .root_retirement
            .as_ref()
            .ok_or_else(|| denied("root archive movement not registered"))?;
        let raw: Vec<u8> = self
            .connection()?
            .query_row(
                "SELECT document FROM events WHERE revision=?1",
                [movement.source_revision],
                |r| r.get(0),
            )
            .map_err(database_error)?;
        let source: Head = decode(&raw)?;
        super::root_retirement::open_archived_originals(
            self.data.path(),
            &self.profile,
            &super::root_retirement::ArchiveReferences {
                source,
                original_root: root.clone(),
                plan_id: movement.plan.id.clone(),
                plan_sha256: movement.plan.sha256.clone(),
            },
        )
    }
    pub fn record_first_root_phase(&self, proof: &session::RootPhaseProof<'_>) -> io::Result<Head> {
        let origin = self
            .root_registration
            .as_ref()
            .ok_or_else(|| denied("root phase has no original absence"))?;
        let expected = proof.source();
        if expected.ticket != origin.id || self.head()?.as_ref() != Some(expected) {
            return Err(denied("root phase source changed"));
        }
        let refs = proof
            .require(self.data.path(), self.client.path(), &self.profile)
            .map_err(|_| denied("current native root phase proof unconfirmed"))?;
        let next = Head {
            revision: expected
                .revision
                .checked_add(1)
                .ok_or_else(|| denied("root phase revision overflow"))?,
            ticket: expected.ticket.clone(),
            state: proof.state(),
            references: Some(refs),
            restore_plan: None,
            completion_sha256: None,
            root_retirement: None,
        };
        self.write(Some(expected), &next)?;
        proof
            .require(self.data.path(), self.client.path(), &self.profile)
            .map_err(|_| denied("root phase commit lost current authority; references retained"))?;
        Ok(next)
    }
    fn cancel_history(&self, head: &Head) -> io::Result<Vec<Head>> {
        self.check()?;
        vault_history::read(self.connection()?, head)
    }
    pub fn reserve_start(&self) -> io::Result<Head> {
        let previous = self.head()?;
        if previous.as_ref().is_some_and(|head| {
            !matches!(
                head.state,
                State::Restored
                    | State::Unchanged
                    | State::CancelledBeforeCapture
                    | State::CancelledCapture
            )
        }) {
            return Err(denied(
                "unfinished original session blocks a new startup baseline",
            ));
        }
        if let Some(head) = &previous {
            if matches!(
                head.state,
                State::CancelledBeforeCapture | State::CancelledCapture
            ) {
                preparation::verify_registered(
                    &self.data,
                    &self.client,
                    &self.profile,
                    &self.binding,
                    head,
                    &self.cancel_history(head)?,
                )?;
            } else if head.state == State::Unchanged {
                unchanged::verify_registered(
                    self.data.path(),
                    self.client.path(),
                    &self.profile,
                    head,
                )?;
            } else {
                completion::verify_registered(
                    self.data.path(),
                    self.client.path(),
                    &self.profile,
                    head,
                )?;
            }
        } else if observation::entry(&self.action_parent, "client-originals")?.is_some() {
            return Err(denied(
                "retained originals cannot become a new startup baseline",
            ));
        }
        let head = Head {
            revision: previous
                .as_ref()
                .map(|h| h.revision.checked_add(1))
                .unwrap_or(Some(1))
                .ok_or_else(|| denied("session revision overflow"))?,
            ticket: uuid::Uuid::new_v4().to_string(),
            state: State::Preparing,
            references: None,
            restore_plan: None,
            completion_sha256: None,
            root_retirement: None,
        };
        self.write(previous.as_ref(), &head)?;
        Ok(head)
    }
    /// This durable commit PRECEDES the first capture directory or blob.
    pub fn reserve_capture(&self, expected: &Head) -> io::Result<Head> {
        if self.root_registration.is_some() {
            return Err(denied(
                "first-root original-absence capture is not integrated; no ordinary baseline",
            ));
        }

        self.reserve_capture_inner(expected)
    }
    pub fn reserve_first_root_capture(&self, expected: &Head) -> io::Result<Head> {
        self.check()?;
        let root = self
            .root_registration
            .as_ref()
            .ok_or_else(|| denied("first-root capture has no registered absence"))?;
        if self.root_created_fd.is_none()
            || self
                .root_creation_deadline
                .is_none_or(|d| Instant::now() >= d)
            || expected.ticket != root.id
            || expected.revision != 1
            || !super::tree::names(self.client.file())?.is_empty()
        {
            return Err(denied(
                "first-root capture lacks live empty-root creation custody",
            ));
        }
        self.reserve_capture_inner(expected)
    }
    fn reserve_capture_inner(&self, expected: &Head) -> io::Result<Head> {
        if expected.state != State::Preparing
            || expected.references.is_some()
            || expected.restore_plan.is_some()
            || expected.completion_sha256.is_some()
        {
            return Err(denied(
                "capture reservation cannot replace current originals",
            ));
        }
        let mut next = expected.clone();
        next.revision = next
            .revision
            .checked_add(1)
            .ok_or_else(|| denied("capture revision overflow"))?;
        next.state = State::CaptureReserved;
        validate(&next)?;
        self.write(Some(expected), &next)?;
        Ok(next)
    }
    /// Store the first initialized tree only with its separately registered
    /// original-root absence. No ordinary Captured/Ready transition is granted.
    pub fn record_first_root_originals(
        &self,
        context: &ProcessContext,
        expected: &Head,
        refs: References,
    ) -> io::Result<Head> {
        self.check()?;
        let origin = self
            .root_registration
            .as_ref()
            .ok_or_else(|| denied("root capture requires indexed original absence"))?;
        if self.root_created_fd.is_none()
            || self
                .root_creation_deadline
                .is_none_or(|d| Instant::now() >= d)
            || expected.state != State::CaptureReserved
            || expected.ticket != origin.id
            || refs.attempt != origin.id
            || !refs.phases.is_empty()
            || refs.live_sha256.is_some()
            || scope(context, &self.data, &self.client)?
                != root_binding::scope_bytes(&self.binding)?
        {
            return Err(denied(
                "root Original references do not match live indexed capture custody",
            ));
        }
        let originals = Originals::open_profile_origin(
            self.data.path(),
            self.client.path(),
            &self.profile,
            super::OriginalKeys {
                attempt: &refs.attempt,
                original_sha256: &refs.original_sha256,
                expected_phases: &refs.phases,
            },
            Some(origin.clone()),
        )?;
        session::verify_index_scope(&originals, &refs, &context.opencodex_home)
            .map_err(|_| denied("root Original startup scope is not bound to this context"))?;
        let next = Head {
            revision: expected
                .revision
                .checked_add(1)
                .ok_or_else(|| denied("root capture revision overflow"))?,
            ticket: expected.ticket.clone(),
            state: State::Captured,
            references: Some(refs),
            restore_plan: None,
            completion_sha256: None,
            root_retirement: None,
        };
        if self
            .root_creation_deadline
            .is_none_or(|d| Instant::now() >= d)
        {
            return Err(denied(
                "first-root Original registration expired before commit",
            ));
        }
        self.write(Some(expected), &next)?;
        if self
            .root_creation_deadline
            .is_none_or(|d| Instant::now() >= d)
        {
            return Err(denied(
                "first-root Original registered after deadline; exact references retained",
            ));
        }
        Ok(next)
    }
    pub fn capture_reservation<'a>(&'a self, head: &Head) -> io::Result<CaptureReservation<'a>> {
        let reservation = CaptureReservation {
            index: self,
            head: head.clone(),
        };
        reservation.require(self.data.path(), self.client.path(), &self.profile)?;
        Ok(reservation)
    }
    pub fn advance(
        &self,
        expected: &Head,
        state: State,
        references: Option<References>,
    ) -> io::Result<Head> {
        if self.root_registration.is_some() && state != State::Failed {
            return Err(denied(
                "root-aware original capture and restoration are not integrated",
            ));
        }
        let allowed = matches!(
            (expected.state, state),
            (State::Preparing, State::Captured)
                | (State::CaptureReserved, State::Captured)
                | (State::Captured, State::Ready)
                | (State::Ready, State::StopPrepared)
                | (State::StopPrepared, State::Stopped)
        ) || state == State::Failed
            && !matches!(
                expected.state,
                State::Stopped
                    | State::RestorePrepared
                    | State::RecoveryPrepared
                    | State::Restored
                    | State::RestoreRolledBack
                    | State::Unchanged
                    | State::CancelledBeforeCapture
                    | State::CancelledCapture
            );
        if !allowed {
            return Err(denied("current session transition is not allowed"));
        }
        if expected.state == State::CaptureReserved
            && (state != State::Captured
                || references
                    .as_ref()
                    .is_none_or(|r| r.attempt != expected.ticket))
        {
            return Err(denied(
                "captured references differ from independently reserved attempt",
            ));
        }
        if let Some(before) = &expected.references {
            let after = references
                .as_ref()
                .ok_or_else(|| denied("original references cannot be discarded"))?;
            if before.attempt != after.attempt
                || before.original_sha256 != after.original_sha256
                || before.scope_sha256 != after.scope_sha256
                || !after.phases.starts_with(&before.phases)
                || before.live_sha256.is_some() && before.live_sha256 != after.live_sha256
            {
                return Err(denied("original session references cannot be rebased"));
            }
        }
        let next = Head {
            revision: expected
                .revision
                .checked_add(1)
                .ok_or_else(|| denied("session revision overflow"))?,
            ticket: expected.ticket.clone(),
            state,
            references,
            restore_plan: None,
            completion_sha256: None,
            root_retirement: None,
        };
        self.write(Some(expected), &next)?;
        Ok(next)
    }
    /// Reference publication only. This does not authorize applying the plan;
    /// the native service/family proof and mutation permit remain separate.
    pub fn record_restore_plan(&self, expected: &Head, id: &str, sha256: &str) -> io::Result<Head> {
        if self.root_registration.is_some() {
            if expected.state != State::Stopped {
                return Err(denied(
                    "first-root restoration requires actual Stopped prefix",
                ));
            }
            let originals = self.registered_root_originals(expected)?;
            let pending = restore::Pending::open(&originals, id, sha256)?;
            pending.require_indexed_references(
                expected
                    .references
                    .as_ref()
                    .ok_or_else(|| denied("root restore references missing"))?,
            )?;
        }
        if !matches!(
            expected.state,
            State::Stopped | State::StopPrepared | State::Ready
        ) || expected.restore_plan.is_some()
        {
            return Err(denied("restore plan cannot replace prior session decision"));
        }
        let next = Head {
            revision: expected
                .revision
                .checked_add(1)
                .ok_or_else(|| denied("session revision overflow"))?,
            ticket: expected.ticket.clone(),
            state: if matches!(expected.state, State::StopPrepared | State::Ready) {
                State::RecoveryPrepared
            } else {
                State::RestorePrepared
            },
            references: expected.references.clone(),
            restore_plan: Some(RestorePlan {
                id: id.into(),
                sha256: sha256.into(),
            }),
            completion_sha256: None,
            root_retirement: None,
        };
        if matches!(expected.state, State::StopPrepared | State::Ready) {
            let refs = expected
                .references
                .as_ref()
                .ok_or_else(|| denied("recovery plan source absent"))?;
            let originals = Originals::open_for_profile(
                self.data.path(),
                self.client.path(),
                &self.profile,
                &refs.attempt,
                &refs.original_sha256,
                &refs.phases,
            )?;
            let pending = restore::Pending::open(&originals, id, sha256)?;
            pending.require_indexed_references(refs)?;
            pending.require_recovery_source(expected)?;
        }
        validate(&next)?;
        self.write(Some(expected), &next)?;
        Ok(next)
    }
    pub fn record_root_retirement(
        &self,
        expected: &Head,
        id: &str,
        sha256: &str,
    ) -> io::Result<Head> {
        self.check()?;
        if expected.root_retirement.is_some()
            || !matches!(
                expected.state,
                State::Captured | State::Stopped | State::RestorePrepared | State::RecoveryPrepared
            )
        {
            return Err(denied("root movement cannot replace prior decision"));
        }
        let originals = self.registered_root_originals(expected)?;
        let pending = super::root_retirement::Pending::open(&originals, id, sha256)?;
        pending.require_source(expected)?;
        pending.inspect_live()?;
        let next = Head {
            revision: expected
                .revision
                .checked_add(1)
                .ok_or_else(|| denied("root plan revision overflow"))?,
            ticket: expected.ticket.clone(),
            state: State::RootRetirementPrepared,
            references: expected.references.clone(),
            restore_plan: expected.restore_plan.clone(),
            completion_sha256: None,
            root_retirement: Some(RootRetirement {
                source_revision: expected.revision,
                plan: RestorePlan {
                    id: id.into(),
                    sha256: sha256.into(),
                },
            }),
        };
        self.write(Some(expected), &next)?;
        Ok(next)
    }
    pub fn root_retirement_registration<'a>(
        &'a self,
        source: &Head,
        prepared: &Head,
        plan: &RestorePlan,
    ) -> io::Result<RootRetirementRegistration<'a>> {
        let proof = RootRetirementRegistration {
            index: self,
            prepared: prepared.clone(),
        };
        proof.require(source, plan)?;
        Ok(proof)
    }
    pub fn close_root_absence(
        &self,
        expected: &Head,
        proof: &super::root_retirement::AuthorizedReadback,
    ) -> io::Result<Head> {
        let digest = proof
            .require(&self.data, &self.profile, expected)?
            .to_string();
        self.check()?;
        let next = Head {
            revision: expected
                .revision
                .checked_add(1)
                .ok_or_else(|| denied("root close revision overflow"))?,
            ticket: expected.ticket.clone(),
            state: State::RootAbsent,
            references: expected.references.clone(),
            restore_plan: expected.restore_plan.clone(),
            completion_sha256: Some(digest),
            root_retirement: expected.root_retirement.clone(),
        };
        self.write(Some(expected), &next)?;
        if let Err(error) = proof.require(&self.data, &self.profile, expected) {
            let mut pending = expected.clone();
            pending.revision = next
                .revision
                .checked_add(1)
                .ok_or_else(|| denied("root timeout demotion revision overflow"))?;
            self.write(Some(&next), &pending)?;
            return Err(error);
        }
        Ok(next)
    }
    pub fn close_restored(
        &self,
        expected: &Head,
        proof: &completion::ReadbackProof,
    ) -> io::Result<Head> {
        if proof.decision() != restore::Decision::Complete {
            return Err(denied("rollback readback cannot close restored"));
        }
        self.close_readback(expected, proof, State::Restored)
    }
    pub(super) fn close_preparation(
        &self,
        expected: &Head,
        proof: &preparation::Proof,
    ) -> io::Result<Head> {
        if self.root_registration.is_some() {
            return Err(denied(
                "first root original absence cannot be closed by ordinary client recovery",
            ));
        }
        proof.require(expected, &self.binding)?;
        let next = Head {
            revision: expected
                .revision
                .checked_add(1)
                .ok_or_else(|| denied("cancel revision overflow"))?,
            ticket: expected.ticket.clone(),
            state: if expected.state == State::CaptureReserved {
                State::CancelledCapture
            } else {
                State::CancelledBeforeCapture
            },
            references: None,
            restore_plan: None,
            completion_sha256: Some(proof.sha256().into()),
            root_retirement: None,
        };
        validate(&next)?;
        preparation::verify_registered(
            &self.data,
            &self.client,
            &self.profile,
            &self.binding,
            &next,
            &self.cancel_history(expected)?,
        )?;
        proof.require(expected, &self.binding)?;
        self.write(Some(expected), &next)?;
        if let Err(error) = proof.require(expected, &self.binding) {
            let mut pending = expected.clone();
            pending.revision = next
                .revision
                .checked_add(1)
                .ok_or_else(|| denied("cancel demotion overflow"))?;
            self.write(Some(&next), &pending)?;
            return Err(error);
        }
        Ok(next)
    }
    pub(super) fn close_unchanged(
        &self,
        expected: &Head,
        proof: &unchanged::Proof,
    ) -> io::Result<Head> {
        proof.require(expected, self.data.identity(), self.client.identity())?;
        let next = Head {
            revision: expected
                .revision
                .checked_add(1)
                .ok_or_else(|| denied("early close revision overflow"))?,
            ticket: expected.ticket.clone(),
            state: State::Unchanged,
            references: expected.references.clone(),
            restore_plan: None,
            completion_sha256: Some(proof.sha256().into()),
            root_retirement: None,
        };
        validate(&next)?;
        unchanged::verify_registered(self.data.path(), self.client.path(), &self.profile, &next)?;
        proof.require(expected, self.data.identity(), self.client.identity())?;
        self.write(Some(expected), &next)?;
        if let Err(error) = proof.require(expected, self.data.identity(), self.client.identity()) {
            let mut pending = expected.clone();
            pending.revision = next
                .revision
                .checked_add(1)
                .ok_or_else(|| denied("early demotion revision overflow"))?;
            self.write(Some(&next), &pending)?;
            return Err(error);
        }
        Ok(next)
    }
    pub(super) fn rearm_rolled_back(
        &self,
        expected: &Head,
        id: &str,
        sha256: &str,
    ) -> io::Result<Head> {
        if expected.state != State::RestoreRolledBack {
            return Err(denied("only registered rollback can rearm"));
        }
        completion::verify_registered(
            self.data.path(),
            self.client.path(),
            &self.profile,
            expected,
        )?;
        let refs = expected
            .references
            .as_ref()
            .ok_or_else(|| denied("rearm original references absent"))?;
        let originals = Originals::open_for_profile(
            self.data.path(),
            self.client.path(),
            &self.profile,
            &refs.attempt,
            &refs.original_sha256,
            &refs.phases,
        )?;
        let pending = restore::Pending::open(&originals, id, sha256)?;
        pending.require_parent(
            expected
                .restore_plan
                .as_ref()
                .ok_or_else(|| denied("rearm parent absent"))?,
        )?;
        if pending.review_recovery()?.terminal.is_some() {
            return Err(denied("new logical plan already terminated"));
        }
        let next = Head {
            revision: expected
                .revision
                .checked_add(1)
                .ok_or_else(|| denied("revision overflow"))?,
            ticket: expected.ticket.clone(),
            state: if expected
                .references
                .as_ref()
                .is_some_and(|r| matches!(r.phases.len(), 1 | 2))
            {
                State::RecoveryPrepared
            } else {
                State::RestorePrepared
            },
            references: expected.references.clone(),
            restore_plan: Some(RestorePlan {
                id: id.into(),
                sha256: sha256.into(),
            }),
            completion_sha256: None,
            root_retirement: None,
        };
        validate(&next)?;
        self.write(Some(expected), &next)?;
        Ok(next)
    }
    pub fn close_rolled_back(
        &self,
        expected: &Head,
        proof: &completion::ReadbackProof,
    ) -> io::Result<Head> {
        if proof.decision() != restore::Decision::Rollback {
            return Err(denied("complete readback cannot close rollback"));
        }
        self.close_readback(expected, proof, State::RestoreRolledBack)
    }
    fn close_readback(
        &self,
        expected: &Head,
        proof: &completion::ReadbackProof,
        state: State,
    ) -> io::Result<Head> {
        if self.root_registration.is_some() {
            return Err(denied(
                "first root original absence cannot be closed by ordinary client recovery",
            ));
        }
        proof.require_index(expected, self.data.identity(), self.client.identity())?;
        let next = Head {
            revision: expected
                .revision
                .checked_add(1)
                .ok_or_else(|| denied("session revision overflow"))?,
            ticket: expected.ticket.clone(),
            state,
            references: expected.references.clone(),
            restore_plan: expected.restore_plan.clone(),
            completion_sha256: Some(proof.sha256().into()),
            root_retirement: None,
        };
        completion::verify_registered(self.data.path(), self.client.path(), &self.profile, &next)?;
        proof.require_index(expected, self.data.identity(), self.client.identity())?;
        self.write(Some(expected), &next)?;
        if let Err(error) =
            proof.require_index(expected, self.data.identity(), self.client.identity())
        {
            // A slow durable commit cannot lend a fresh budget or leave a
            // successful gate behind after the action reports timeout. This
            // safety demotion is a new retained event, never success recovery.
            let pending = Head {
                revision: next
                    .revision
                    .checked_add(1)
                    .ok_or_else(|| denied("session revision overflow"))?,
                ticket: expected.ticket.clone(),
                state: expected.state,
                references: expected.references.clone(),
                restore_plan: expected.restore_plan.clone(),
                completion_sha256: None,
                root_retirement: None,
            };
            self.write(Some(&next), &pending)?;
            return Err(error);
        }
        Ok(next)
    }
    fn write(&self, expected: Option<&Head>, next: &Head) -> io::Result<()> {
        validate(next)?;
        if self.root_registration.is_some()
            && !(expected.is_some_and(|h| h.state == State::Preparing)
                && next.state == State::CaptureReserved
                || expected.is_some_and(|h| h.state == State::CaptureReserved)
                    && next.state == State::Captured
                || expected.is_some_and(|h| h.state == State::Captured)
                    && next.state == State::Ready
                || expected.is_some_and(|h| h.state == State::Ready)
                    && next.state == State::StopPrepared
                || expected.is_some_and(|h| h.state == State::StopPrepared)
                    && next.state == State::Stopped
                || expected.is_some_and(|h| h.state == State::Stopped)
                    && next.state == State::RestorePrepared
                || expected.is_some_and(|h| {
                    matches!(
                        h.state,
                        State::Captured
                            | State::Stopped
                            | State::RestorePrepared
                            | State::RecoveryPrepared
                    )
                }) && next.state == State::RootRetirementPrepared
                || expected.is_some_and(|h| h.state == State::RootRetirementPrepared)
                    && next.state == State::RootAbsent
                || expected.is_some_and(|h| h.state == State::RootAbsent)
                    && next.state == State::RootRetirementPrepared
                || next.state == State::Failed)
        {
            return Err(denied(
                "first root cannot enter an unintegrated ordinary lifecycle transition",
            ));
        }
        if expected
            .and_then(|h| h.root_retirement.as_ref())
            .is_some_and(|r| next.root_retirement.as_ref() != Some(r))
        {
            return Err(denied(
                "independent root retirement references cannot be discarded",
            ));
        }
        self.check()?;
        let c = self.connection()?;
        c.execute_batch("BEGIN IMMEDIATE").map_err(database_error)?;
        let run = (|| -> io::Result<()> {
            let binding: Vec<u8> = c
                .query_row("SELECT document FROM binding WHERE singleton=1", [], |r| {
                    r.get(0)
                })
                .map_err(database_error)?;
            if binding != self.binding || self.head()?.as_ref() != expected {
                return Err(denied("current session changed before reference commit"));
            }
            let bytes = encode(next)?;
            c.execute(
                "INSERT INTO events(revision,document) VALUES(?1,?2)",
                params![next.revision, &bytes],
            )
            .map_err(database_error)?;
            if let Some(expected) = expected {
                if c.execute("UPDATE current_session SET revision=?1,document=?2 WHERE singleton=1 AND revision=?3 AND document=?4",params![next.revision,&bytes,expected.revision,encode(expected)?]).map_err(database_error)?!=1{return Err(denied("session reference compare-and-swap failed"));}
            } else {
                c.execute(
                    "INSERT INTO current_session(singleton,revision,document) VALUES(1,?1,?2)",
                    params![next.revision, &bytes],
                )
                .map_err(database_error)?;
            }
            self.check()?;
            Ok(())
        })();
        if let Err(error) = run {
            let _ = c.execute_batch("ROLLBACK");
            return Err(error);
        }
        c.execute_batch("COMMIT").map_err(database_error)?;
        if self.head()?.as_ref() != Some(next) {
            return Err(denied("session references commit readback differs"));
        }
        Ok(())
    }
}
impl Drop for Index {
    fn drop(&mut self) {
        // Connection closed before pin FD drops: never release SQLite's fcntl
        // locks by closing another descriptor while a transaction is live.
        if let Some(connection) = self.connection.take() {
            let _ = connection.execute_batch("ROLLBACK");
            let _ = connection.close();
        }
    }
}

#[cfg(test)]
#[path = "macos_client_session_index_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "macos_client_root_registration_tests.rs"]
mod root_registration_tests;
