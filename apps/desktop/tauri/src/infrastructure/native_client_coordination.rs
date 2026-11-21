//! The mutation owner retains the original SQLite locks in THIS process.
//! Official admission/initialization is first performed unchanged by the
//! existing helper. A gap during handover grants no authority: all four file
//! generations and the official-owned journal receipt must match after native
//! acquisition. Nothing recreates a database or writes a copied schema.
use super::*;
use crate::infrastructure::macos_client_originals::{
    claim_gate::ClaimGate, ownership_receipt, CoordinationFilePin, Identity, Protection,
};
use std::{
    cell::{Cell, RefCell},
    marker::PhantomData,
};

/// Borrow-scoped native ownership. There is no timer thread or external helper
/// that can release these connections while the callback continues. Expired
/// work must stop and remains a failure, but locks last until its stack unwinds.
/// This proves coordinator ownership only, not service exit or file provenance.
#[path = "native_config_write_guard.rs"]
pub mod config;
#[path = "native_client_release.rs"]
pub mod released;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Reacquisition {
    selection: program_selection::CurrentSelection,
    image: program_selection::ImageManifest,
    stable_entry: PathBuf,
    logical: PathBuf,
    profile: PathBuf,
    home: PathBuf,
    directories: [(Identity, Protection); 3],
    paths: [PathBuf; 4],
    files: [(Identity, Protection); 4],
    owner: String,
}

pub struct NativeLease {
    claim: RefCell<Option<rusqlite::Connection>>,
    connections: Vec<rusqlite::Connection>,
    claim_gate: RefCell<Option<ClaimGate>>,
    claim_mode: Cell<ClaimMode>,
    physical_client: RefCell<Option<PinnedDirectory>>,
    pins: Vec<CoordinationFilePin>,
    client: PinnedDirectory,
    logical_client: PathBuf,
    paths: [PathBuf; 4],
    owner: String,
    runtime: SharedRuntimeExecutable,
    profile: PinnedDirectory,
    home: PinnedDirectory,
    deadline: Instant,
    active: Cell<bool>,
    _not_send_or_sync: PhantomData<*mut ()>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ClaimMode {
    Sqlite,
    HandoffPending,
    Ofd,
}
impl NativeLease {
    pub(crate) fn freeze_reacquisition(
        &self,
        selected: &program_selection::SelectedProgram,
    ) -> AppResult<Reacquisition> {
        self.require_claim_handoff()?;
        if self.client.path() != self.logical_client || self.physical_client.borrow().is_some() {
            return Err(fail("client_native_coordination_reacquisition_scope"));
        }
        let freeze = |(identity, protection): (&Identity, &Protection)| {
            (identity.clone(), protection.clone())
        };
        Ok(Reacquisition {
            selection: selected.selection.clone(),
            image: selected.image.clone(),
            stable_entry: selected.stable_entry.clone(),
            logical: self.logical_client.clone(),
            profile: self.profile.path().to_owned(),
            home: self.home.path().to_owned(),
            directories: [&self.client, &self.profile, &self.home]
                .map(|pin| freeze(pin.coordination_expectation())),
            paths: self.paths.clone(),
            files: std::array::from_fn(|i| freeze(self.pins[i].coordination_expectation())),
            owner: self.owner.clone(),
        })
    }
    pub fn require_context(
        &self,
        context: &crate::state::ProcessContext,
        codex_home: &Path,
    ) -> AppResult<()> {
        self.check()?;
        if !std::sync::Arc::ptr_eq(&context.runtime, &self.runtime)
            || context.working_directory != self.profile.path()
            || context.opencodex_home != self.home.path()
            || codex_home != self.logical_client
        {
            return Err(fail("client_native_coordination_context"));
        }
        Ok(())
    }
    pub fn check(&self) -> AppResult<()> {
        if !self.active.get() || Instant::now() >= self.deadline {
            return Err(fail("client_native_coordination_expired"));
        }
        let physical = self.physical_client.borrow();
        let client = physical.as_ref().unwrap_or(&self.client);
        #[cfg(test)]
        let reference=crate::infrastructure::macos_client_originals::observation_metrics::lease_directories_reference_enabled();
        #[cfg(not(test))]
        let reference = false;
        if reference {
            client.recheck().map_err(io)?;
            self.profile.recheck().map_err(io)?;
            self.home.recheck().map_err(io)?;
        } else {
            // Metadata-only admission of all three roots. All saved inode and
            // complete Protection expectations remain, with two fresh full
            // path/fd passes before claim, pin or owner receipt observations.
            PinnedDirectory::recheck_set(&[client, &self.profile, &self.home]).map_err(io)?;
        }
        let ofd = self.claim_mode.get() == ClaimMode::Ofd;
        if self.claim_mode.get() == ClaimMode::HandoffPending {
            return Err(fail("client_native_coordination_handoff_pending"));
        }
        if ofd {
            self.claim_gate
                .borrow()
                .as_ref()
                .ok_or_else(|| fail("client_native_coordination_handoff_missing"))?
                .check_at(client)
                .map_err(io)?;
            if self.claim.borrow().is_some() {
                return Err(fail("client_native_coordination_handoff_connection"));
            }
            if client.path() != self.logical_client
                && fs::symlink_metadata(&self.logical_client)
                    .map_or_else(|e| e.kind() != std::io::ErrorKind::NotFound, |_| true)
            {
                return Err(fail("client_native_coordination_logical_root_reappeared"));
            }
        } else {
            let claim = self.claim.borrow();
            let claim = claim
                .as_ref()
                .ok_or_else(|| fail("client_native_coordination_unlocked"))?;
            if claim.is_autocommit() {
                return Err(fail("client_native_coordination_unlocked"));
            }
            require_original_sqlite_file(claim)?;
        }
        for pin in self.pins.iter().skip(usize::from(ofd)) {
            pin.recheck().map_err(io)?;
        }
        if ownership_receipt(client).map_err(io)? != self.owner {
            return Err(fail("client_native_coordination_owner_changed"));
        }
        if self.connections.len() != 3
            || self
                .connections
                .iter()
                .any(rusqlite::Connection::is_autocommit)
        {
            return Err(fail("client_native_coordination_unlocked"));
        }
        for connection in &self.connections {
            require_original_sqlite_file(connection)?;
        }
        Ok(())
    }
    /// Coordinator handoff only. This does not authorize filesystem movement
    /// or prove service exit. N/L/K stay held while only the claim changes its
    /// lock representation. A failed handoff poisons this lease for operations.
    pub fn begin_claim_handoff(&self) -> AppResult<()> {
        self.begin_claim_handoff_checked(&|| Ok(()))
    }
    pub(crate) fn require_claim_handoff(&self) -> AppResult<()> {
        self.check()?;
        if self.claim_mode.get() != ClaimMode::Ofd {
            return Err(fail("client_native_coordination_handoff_not_current"));
        }
        Ok(())
    }
    fn begin_claim_handoff_checked(
        &self,
        checkpoint: &impl Fn() -> AppResult<()>,
    ) -> AppResult<()> {
        self.check()?;
        if self.claim_mode.get() != ClaimMode::Sqlite || self.claim_gate.borrow().is_none() {
            return Err(fail("client_native_coordination_handoff_not_prepared"));
        }
        self.claim_mode.set(ClaimMode::HandoffPending);
        let claim = self
            .claim
            .borrow_mut()
            .take()
            .ok_or_else(|| fail("client_native_coordination_unlocked"))?;
        claim.execute_batch("ROLLBACK").map_err(io)?;
        claim
            .close()
            .map_err(|_| fail("client_native_coordination_claim_close"))?;
        // This callback is only an explicit test boundary in the private helper.
        // No operations gain authority while the claim is unheld.
        checkpoint()?;
        self.claim_gate
            .borrow_mut()
            .as_mut()
            .ok_or_else(|| fail("client_native_coordination_handoff_missing"))?
            .acquire(&self.client)
            .map_err(io)?;
        self.claim_mode.set(ClaimMode::Ofd);
        self.check()
    }
    /// Accept only a new physical pathname for the retained original root.
    /// The caller must separately bind it to an immutable movement plan.
    pub fn reanchor_claim(&self, path: &Path) -> AppResult<()> {
        if !self.active.get()
            || Instant::now() >= self.deadline
            || self.claim_mode.get() != ClaimMode::Ofd
        {
            return Err(fail("client_native_coordination_handoff_not_current"));
        }
        if path != self.logical_client
            && fs::symlink_metadata(&self.logical_client)
                .map_or_else(|e| e.kind() != std::io::ErrorKind::NotFound, |_| true)
        {
            return Err(fail("client_native_coordination_logical_root_reappeared"));
        }
        let parent = PinnedDirectory::open(path).map_err(io)?;
        self.claim_gate
            .borrow()
            .as_ref()
            .ok_or_else(|| fail("client_native_coordination_handoff_missing"))?
            .check_at(&parent)
            .map_err(io)?;
        *self.physical_client.borrow_mut() = Some(parent);
        self.check()
    }
    fn release(&mut self) -> AppResult<()> {
        // Close every SQLite connection before dropping ANY raw file pin for
        // these inodes. POSIX record locks belong to a process, not one fd.
        if !self.active.replace(false) {
            return Ok(());
        }
        let mut failed = false;
        if self.claim_mode.get() == ClaimMode::Ofd {
            let physical = self.physical_client.borrow();
            let client = physical.as_ref().unwrap_or(&self.client);
            if self
                .claim_gate
                .get_mut()
                .as_mut()
                .is_none_or(|gate| gate.finish_at(client).is_err())
            {
                failed = true;
            }
        } else if self.claim_mode.get() == ClaimMode::HandoffPending {
            failed = true;
        }
        while let Some(connection) = self.connections.pop() {
            if connection.execute_batch("ROLLBACK").is_err() {
                failed = true;
            }
            if let Err((connection, _)) = connection.close() {
                drop(connection);
                failed = true;
            }
        }
        if let Some(claim) = self.claim.get_mut().take() {
            if claim.execute_batch("ROLLBACK").is_err() {
                failed = true;
            }
            if let Err((claim, _)) = claim.close() {
                drop(claim);
                failed = true;
            }
        }
        if failed {
            return Err(fail("client_native_coordination_release"));
        }
        Ok(())
    }
}
impl Drop for NativeLease {
    fn drop(&mut self) {
        let _ = self.release();
    }
}

fn require_original_sqlite_file(connection: &rusqlite::Connection) -> AppResult<()> {
    // Inspect SQLite's actual VFS fd. Opening/closing an auxiliary descriptor
    // here would risk releasing this process's POSIX locks on the same inode.
    let mut moved = 1i32;
    let result = unsafe {
        rusqlite::ffi::sqlite3_file_control(
            connection.handle(),
            c"main".as_ptr(),
            rusqlite::ffi::SQLITE_FCNTL_HAS_MOVED,
            (&mut moved as *mut i32).cast(),
        )
    };
    if result != rusqlite::ffi::SQLITE_OK || moved != 0 {
        return Err(fail("client_native_coordination_vfs_changed"));
    }
    Ok(())
}

fn acquire(
    context: &Context,
    admission: &Record,
    deadline: Instant,
    checkpoint: &impl Fn(usize) -> AppResult<()>,
) -> AppResult<NativeLease> {
    acquire_mode(context, admission, deadline, checkpoint, false)
}
fn acquire_mode(
    context: &Context,
    admission: &Record,
    deadline: Instant,
    checkpoint: &impl Fn(usize) -> AppResult<()>,
    relocation: bool,
) -> AppResult<NativeLease> {
    if Instant::now() >= deadline {
        return Err(fail("client_native_coordination_timeout"));
    }
    if admission.paths[0]
        != context
            .codex_home
            .join(".opencodex-native-main.claim.sqlite")
    {
        return Err(fail("client_native_coordination_identity"));
    }
    let mut keys = std::collections::BTreeSet::new();
    let mut pins = Vec::new();
    for (path, expected) in admission.paths.iter().zip(&admission.locks) {
        if fs::canonicalize(path).map_err(io)? != *path {
            return Err(fail("client_native_coordination_identity"));
        }
        let pin = CoordinationFilePin::open(path).map_err(io)?;
        let key = pin.key();
        if key != (expected.device.clone(), expected.inode.clone()) || !keys.insert(key) {
            return Err(fail("client_native_coordination_identity"));
        }
        pins.push(pin);
    }
    let client = PinnedDirectory::open(&context.codex_home).map_err(io)?;
    if ownership_receipt(&client).map_err(io)? != admission.owner {
        return Err(fail("client_native_coordination_owner_changed"));
    }
    // Open the optional writable pin BEFORE any in-process SQLite transaction.
    let claim_gate = if relocation {
        let gate = ClaimGate::prepare(&client, deadline).map_err(io)?;
        if gate.key() != pins[0].key() {
            return Err(fail("client_native_coordination_identity"));
        }
        Some(gate)
    } else {
        None
    };
    let mut lease = NativeLease {
        claim: RefCell::new(None),
        connections: Vec::new(),
        claim_gate: RefCell::new(claim_gate),
        claim_mode: Cell::new(ClaimMode::Sqlite),
        physical_client: RefCell::new(None),
        pins,
        client,
        logical_client: context.codex_home.clone(),
        paths: admission.paths.clone(),
        owner: admission.owner.clone(),
        runtime: context.runtime.clone(),
        profile: PinnedDirectory::open(&context.user_profile).map_err(io)?,
        home: PinnedDirectory::open(&context.opencodex_home).map_err(io)?,
        deadline,
        active: Cell::new(true),
        _not_send_or_sync: PhantomData,
    };
    // Exact official claim -> N -> L -> K databases and transaction strengths;
    // callbacks never run after partial acquisition and no busy wait is added.
    for (i, path) in admission.paths.iter().enumerate() {
        if Instant::now() >= deadline {
            return Err(fail("client_native_coordination_timeout"));
        }
        let connection = open_connection(path)?;
        if i == 0 {
            let mut claim = lease.claim.borrow_mut();
            let connection = claim.insert(connection);
            begin(connection, "BEGIN EXCLUSIVE")?;
        } else {
            lease.connections.push(connection);
            begin(lease.connections.last().unwrap(), "BEGIN IMMEDIATE")?;
        }
        for pin in &lease.pins {
            pin.recheck().map_err(io)?;
        }
        checkpoint(i)?;
    }
    lease.check()?;
    Ok(lease)
}

fn open_connection(path: &Path) -> AppResult<rusqlite::Connection> {
    rusqlite::Connection::open_with_flags(
        path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
            | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX
            | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .map_err(io)
}

fn begin(connection: &rusqlite::Connection, transaction: &str) -> AppResult<()> {
    connection.busy_timeout(Duration::ZERO).map_err(io)?;
    connection
        .execute_batch(transaction)
        .map_err(|error| match error {
            rusqlite::Error::SqliteFailure(code, _)
                if matches!(
                    code.code,
                    rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
                ) =>
            {
                fail("client_native_coordination_busy")
            }
            _ => io(error),
        })?;
    require_original_sqlite_file(connection)
}

impl Reacquisition {
    pub(crate) fn require_selection(
        &self,
        selected: &program_selection::SelectedProgram,
    ) -> AppResult<()> {
        if self.selection != selected.selection
            || self.image != selected.image
            || self.stable_entry != selected.stable_entry
        {
            return Err(fail("client_native_coordination_source"));
        }
        Ok(())
    }

    fn acquire(
        &self,
        context: &Context,
        physical: &Path,
        deadline: Instant,
    ) -> AppResult<NativeLease> {
        let claim_name = ".opencodex-native-main.claim.sqlite";
        if context.codex_home != self.logical
            || context.user_profile != self.profile
            || context.opencodex_home != self.home
            || context.codex_home != context.user_profile.join(".codex")
            || self.paths[0] != self.logical.join(claim_name)
            || (physical != self.logical
                && !matches!(fs::symlink_metadata(&self.logical), Err(e) if e.kind() == std::io::ErrorKind::NotFound))
        {
            return Err(fail("client_native_coordination_reacquisition_scope"));
        }
        let client = PinnedDirectory::open(physical).map_err(io)?;
        let profile = PinnedDirectory::open(&context.user_profile).map_err(io)?;
        let home = PinnedDirectory::open(&context.opencodex_home).map_err(io)?;
        for (pin, expected) in [&client, &profile, &home]
            .into_iter()
            .zip(&self.directories)
        {
            if pin.coordination_expectation() != (&expected.0, &expected.1) {
                return Err(fail("client_native_coordination_reacquisition_directory"));
            }
        }
        let mut paths = self.paths.clone();
        paths[0] = physical.join(claim_name);
        let mut pins = Vec::with_capacity(paths.len());
        for (path, expected) in paths.iter().zip(&self.files) {
            let pin = CoordinationFilePin::open(path).map_err(io)?;
            if pin.coordination_expectation() != (&expected.0, &expected.1)
                || pins
                    .iter()
                    .any(|prior: &CoordinationFilePin| prior.key() == pin.key())
            {
                return Err(fail("client_native_coordination_reacquisition_file"));
            }
            pins.push(pin);
        }
        if ownership_receipt(&client).map_err(io)? != self.owner {
            return Err(fail("client_native_coordination_owner_changed"));
        }
        let mut claim = ClaimGate::prepare(&client, deadline).map_err(io)?;
        if claim.key() != pins[0].key() {
            return Err(fail("client_native_coordination_identity"));
        }
        claim
            .acquire(&client)
            .map_err(|error| match error.raw_os_error() {
                Some(libc::EAGAIN | libc::EACCES) => fail("client_native_coordination_busy"),
                _ => io(error),
            })?;
        let mut lease = NativeLease {
            claim: RefCell::new(None),
            connections: Vec::new(),
            claim_gate: RefCell::new(Some(claim)),
            claim_mode: Cell::new(ClaimMode::Ofd),
            physical_client: RefCell::new(None),
            pins,
            client,
            logical_client: self.logical.clone(),
            paths,
            owner: self.owner.clone(),
            runtime: context.runtime.clone(),
            profile,
            home,
            deadline,
            active: Cell::new(true),
            _not_send_or_sync: PhantomData,
        };
        for path in &lease.paths[1..] {
            if Instant::now() >= deadline {
                return Err(fail("client_native_coordination_timeout"));
            }
            lease.connections.push(open_connection(path)?);
            begin(lease.connections.last().unwrap(), "BEGIN IMMEDIATE")?;
            for pin in &lease.pins[1..] {
                pin.recheck().map_err(io)?;
            }
        }
        lease.check()?;
        Ok(lease)
    }
}

pub(crate) fn with_native_reacquisition<T>(
    context: &Context,
    expected: &program_selection::SelectedProgram,
    frozen: &Reacquisition,
    physical: &Path,
    deadline: Instant,
    operation: impl FnOnce(&NativeLease) -> AppResult<T>,
) -> AppResult<(T, released::ReleasedCoordination)> {
    frozen.require_selection(expected)?;
    let root = context
        .runtime
        .managed_installation_root()
        .ok_or_else(|| fail("client_native_coordination_source"))?;
    program_selection::verify_current(&root, expected)?;
    let lease = frozen.acquire(context, physical, deadline)?;
    run_and_release(context, &root, expected, lease, operation)
}

/// Called only after an immutable official admission has been obtained, then
/// rechecked under these native locks. The helper is already normally exited;
/// it cannot time out and release a mutation's locks from another process.
pub fn with_native_coordination<T>(
    context: &Context,
    deadline: Instant,
    operation: impl FnOnce(&NativeLease) -> AppResult<T>,
) -> AppResult<T> {
    let (admission, selected) = native_admission(context, deadline)?;
    with_admission(context, &admission, &selected, deadline, operation, &|_| {
        Ok(())
    })
}
/// Opt-in coordinator capability for a separately authorized root movement.
/// Official source/admission and final source checks are identical to ordinary
/// coordination. No caller obtains Stop or filesystem authority from this.
pub fn with_native_relocation_coordination<T>(
    context: &Context,
    deadline: Instant,
    operation: impl FnOnce(&NativeLease) -> AppResult<T>,
) -> AppResult<T> {
    let (admission, selected) = native_admission(context, deadline)?;
    with_admission_mode(
        context,
        &admission,
        &selected,
        deadline,
        operation,
        &|_| Ok(()),
        true,
    )
}
/// Return a nonserializable release receipt only after the original native
/// owner released all four coordinators and its final complete source passed.
pub fn with_native_relocation_release<T>(
    context: &Context,
    deadline: Instant,
    operation: impl FnOnce(&NativeLease) -> AppResult<T>,
) -> AppResult<(T, released::ReleasedCoordination)> {
    let (admission, selected) = native_admission(context, deadline)?;
    with_admission_mode_released(
        context,
        &admission,
        &selected,
        deadline,
        operation,
        &|_| Ok(()),
        true,
    )
}
fn with_admission<T>(
    context: &Context,
    admission: &Record,
    expected: &program_selection::SelectedProgram,
    deadline: Instant,
    operation: impl FnOnce(&NativeLease) -> AppResult<T>,
    checkpoint: &impl Fn(usize) -> AppResult<()>,
) -> AppResult<T> {
    with_admission_mode(
        context, admission, expected, deadline, operation, checkpoint, false,
    )
}
fn with_admission_mode<T>(
    context: &Context,
    admission: &Record,
    expected: &program_selection::SelectedProgram,
    deadline: Instant,
    operation: impl FnOnce(&NativeLease) -> AppResult<T>,
    checkpoint: &impl Fn(usize) -> AppResult<()>,
    relocation: bool,
) -> AppResult<T> {
    with_admission_mode_released(
        context, admission, expected, deadline, operation, checkpoint, relocation,
    )
    .map(|(value, _)| value)
}
fn with_admission_mode_released<T>(
    context: &Context,
    admission: &Record,
    expected: &program_selection::SelectedProgram,
    deadline: Instant,
    operation: impl FnOnce(&NativeLease) -> AppResult<T>,
    checkpoint: &impl Fn(usize) -> AppResult<()>,
    relocation: bool,
) -> AppResult<(T, released::ReleasedCoordination)> {
    let root = context
        .runtime
        .managed_installation_root()
        .ok_or_else(|| fail("client_native_coordination_source"))?;
    program_selection::verify_current(&root, expected)?;
    let lease = if relocation {
        acquire_mode(context, admission, deadline, checkpoint, true)?
    } else {
        acquire(context, admission, deadline, checkpoint)?
    };
    run_and_release(context, &root, expected, lease, operation)
}

fn run_and_release<T>(
    context: &Context,
    root: &Path,
    expected: &program_selection::SelectedProgram,
    mut lease: NativeLease,
    operation: impl FnOnce(&NativeLease) -> AppResult<T>,
) -> AppResult<(T, released::ReleasedCoordination)> {
    let deadline = lease.deadline;
    let result = operation(&lease);
    // Check BEFORE releasing; an elapsed callback never turns into success by
    // unlocking, or by completing subsequent image checks after the deadline.
    #[cfg(test)]
    let tail_started = Instant::now();
    let valid = lease.check();
    let mut released_paths = lease.paths.clone();
    if let Some(physical) = lease.physical_client.borrow().as_ref() {
        released_paths[0] = physical.path().join(".opencodex-native-main.claim.sqlite");
    }
    #[cfg(test)]
    let checked_at = Instant::now();
    let release = lease.release();
    #[cfg(test)]
    let released_at = Instant::now();
    let value = result?;
    valid?;
    release?;
    let (target, platform) = program_layout::host_target()?;
    let selected = program_selection::read_complete(&root, platform, target)?
        .ok_or_else(|| fail("client_native_coordination_source"))?;
    #[cfg(test)]
    if std::env::var_os("OCXD024_RETIREMENT_COST_PROFILE").as_deref()
        == Some(std::ffi::OsStr::new("1"))
        && context.user_profile.starts_with("/private/tmp")
        && context.user_profile.ancestors().any(|p| {
            p.file_name().is_some_and(|n| {
                n.to_string_lossy()
                    .starts_with("ocxd024-versioned-lifecycle-")
            })
        })
    {
        let now = Instant::now();
        eprintln!(
            "COORDINATION_TAIL_COST {}",
            serde_json::json!({"accepted":false,"leaseCheckUs":checked_at.duration_since(tail_started).as_micros(),"releaseUs":released_at.duration_since(checked_at).as_micros(),"completeSourceUs":now.duration_since(released_at).as_micros(),"remainingBeforeUs":deadline.saturating_duration_since(tail_started).as_micros(),"remainingAfterUs":deadline.saturating_duration_since(now).as_micros(),"expiredAfter":now>=deadline})
        );
    }
    if selected != *expected
        || context.runtime.executable().as_ref() != Some(&expected.stable_entry)
    {
        return Err(fail("client_native_coordination_source"));
    }
    if Instant::now() >= deadline {
        return Err(fail("client_native_coordination_timeout"));
    }
    let keys = std::array::from_fn(|i| {
        let (device, inode) = lease.pins[i].key();
        LockIdentity { device, inode }
    });
    // Close all original pins only AFTER all original SQLite connections have
    // closed, before a recipient can open a new release-probe connection.
    drop(lease);
    Ok((
        value,
        released::ReleasedCoordination {
            runtime: context.runtime.clone(),
            profile: context.user_profile.clone(),
            home: context.opencodex_home.clone(),
            logical: context.codex_home.clone(),
            paths: released_paths,
            keys,
            deadline,
        },
    ))
}

#[cfg(test)]
#[path = "native_client_coordination_tests.rs"]
mod tests;
