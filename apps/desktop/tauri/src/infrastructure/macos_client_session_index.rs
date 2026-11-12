//! Desktop-owned exclusive current-session index. Independent references are
//! committed here before another application/CLI action may observe them.
//! No directory scanning, backup discovery or implicit recovery after a crash.
//! This index is not service ownership or a client mutation permit.
use super::session::References;
use super::*;
use crate::state::ProcessContext;
use rusqlite::{params, Connection, OptionalExtension};

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
    RestoreRolledBack,
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
fn validate(head: &Head) -> io::Result<()> {
    if matches!(
        head.state,
        State::RestorePrepared | State::Restored | State::RestoreRolledBack
    ) != head.restore_plan.is_some()
        || matches!(
            head.state,
            State::Restored
                | State::RestoreRolledBack
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
        State::Stopped | State::RestorePrepared | State::Restored | State::RestoreRolledBack => {
            Some((3, true))
        }
        State::Preparing | State::CaptureReserved => {
            return Err(denied("unpublished originals in preparing index"))
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
        client.path(),
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
        Self::open_inner(context, data_root, client_root, true)
    }
    pub fn open_existing(
        context: &ProcessContext,
        data_root: &Path,
        client_root: &Path,
    ) -> io::Result<Self> {
        Self::open_inner(context, data_root, client_root, false)
    }
    fn open_inner(
        context: &ProcessContext,
        data_root: &Path,
        client_root: &Path,
        allow_create: bool,
    ) -> io::Result<Self> {
        let data = PinnedDirectory::open(data_root)?;
        data.private()?;
        let client = PinnedDirectory::open(client_root)?;
        if data_root.starts_with(client_root) || client_root.starts_with(data_root) {
            return Err(denied("session index/client roots overlap"));
        }
        let binding = scope(context, &data, &client)?;
        let manager = data.child_private("manager-state", allow_create)?;
        if observation::entry(&manager, FILE)?.is_none()
            && observation::entry(&manager, "client-originals")?.is_some()
        {
            return Err(denied(
                "originals without an index cannot initialize a new baseline",
            ));
        }
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
        let path = manager.path().join(FILE);
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
            if stored != binding {
                return Err(denied(
                    "session index bound to different directory generation or HOME",
                ));
            }
            Ok(())
        })();
        if let Err(error) = setup {
            let _ = connection.execute_batch("ROLLBACK");
            return Err(error);
        }
        connection.execute_batch("COMMIT").map_err(database_error)?;
        pin.recheck()?;
        let result = Self {
            connection: Some(connection),
            pin,
            action_file,
            action_parent: manager,
            action_identity,
            action_protection,
            binding,
            data,
            client,
            profile: context.working_directory.clone(),
        };
        result.head()?;
        Ok(result)
    }
    fn connection(&self) -> io::Result<&Connection> {
        self.connection
            .as_ref()
            .ok_or_else(|| denied("session index closed"))
    }
    fn check(&self) -> io::Result<()> {
        self.data.private()?;
        self.client.recheck()?;
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
        Ok(())
    }
    pub fn head(&self) -> io::Result<Option<Head>> {
        self.check()?;
        read_head(self.connection()?)
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
        };
        self.write(previous.as_ref(), &head)?;
        Ok(head)
    }
    /// This durable commit PRECEDES the first capture directory or blob.
    pub fn reserve_capture(&self, expected: &Head) -> io::Result<Head> {
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
        };
        self.write(Some(expected), &next)?;
        Ok(next)
    }
    /// Reference publication only. This does not authorize applying the plan;
    /// the native service/family proof and mutation permit remain separate.
    pub fn record_restore_plan(&self, expected: &Head, id: &str, sha256: &str) -> io::Result<Head> {
        if expected.state != State::Stopped || expected.restore_plan.is_some() {
            return Err(denied("restore plan cannot replace prior session decision"));
        }
        let next = Head {
            revision: expected
                .revision
                .checked_add(1)
                .ok_or_else(|| denied("session revision overflow"))?,
            ticket: expected.ticket.clone(),
            state: State::RestorePrepared,
            references: expected.references.clone(),
            restore_plan: Some(RestorePlan {
                id: id.into(),
                sha256: sha256.into(),
            }),
            completion_sha256: None,
        };
        validate(&next)?;
        self.write(Some(expected), &next)?;
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
            state: State::RestorePrepared,
            references: expected.references.clone(),
            restore_plan: Some(RestorePlan {
                id: id.into(),
                sha256: sha256.into(),
            }),
            completion_sha256: None,
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
                state: State::RestorePrepared,
                references: expected.references.clone(),
                restore_plan: expected.restore_plan.clone(),
                completion_sha256: None,
            };
            self.write(Some(&next), &pending)?;
            return Err(error);
        }
        Ok(next)
    }
    fn write(&self, expected: Option<&Head>, next: &Head) -> io::Result<()> {
        validate(next)?;
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
