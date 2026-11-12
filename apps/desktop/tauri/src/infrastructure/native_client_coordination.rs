//! The mutation owner retains the original SQLite locks in THIS process.
//! Official admission/initialization is first performed unchanged by the
//! existing helper. A gap during handover grants no authority: all four file
//! generations and the official-owned journal receipt must match after native
//! acquisition. Nothing recreates a database or writes a copied schema.
use super::*;
use crate::infrastructure::macos_client_originals::{
    claim_gate::ClaimGate, ownership_receipt, CoordinationFilePin,
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

pub struct NativeLease {
    claim: RefCell<Option<rusqlite::Connection>>,
    connections: Vec<rusqlite::Connection>,
    claim_gate: RefCell<Option<ClaimGate>>,
    claim_mode: Cell<ClaimMode>,
    physical_client: RefCell<Option<PinnedDirectory>>,
    pins: Vec<CoordinationFilePin>,
    client: PinnedDirectory,
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
    pub fn require_context(
        &self,
        context: &crate::state::ProcessContext,
        codex_home: &Path,
    ) -> AppResult<()> {
        self.check()?;
        if !std::sync::Arc::ptr_eq(&context.runtime, &self.runtime)
            || context.working_directory != self.profile.path()
            || context.opencodex_home != self.home.path()
            || codex_home != self.client.path()
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
        client.recheck().map_err(io)?;
        self.profile.recheck().map_err(io)?;
        self.home.recheck().map_err(io)?;
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
            if client.path() != self.client.path()
                && fs::symlink_metadata(self.client.path())
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
        if path != self.client.path()
            && fs::symlink_metadata(self.client.path())
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
        let connection = rusqlite::Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE
                | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX
                | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
        )
        .map_err(io)?;
        connection.busy_timeout(Duration::ZERO).map_err(io)?;
        connection
            .execute_batch(if i == 0 {
                "BEGIN EXCLUSIVE"
            } else {
                "BEGIN IMMEDIATE"
            })
            .map_err(|_| fail("client_native_coordination_busy"))?;
        require_original_sqlite_file(&connection)?;
        if i == 0 {
            *lease.claim.borrow_mut() = Some(connection);
        } else {
            lease.connections.push(connection);
        }
        for pin in &lease.pins {
            pin.recheck().map_err(io)?;
        }
        checkpoint(i)?;
    }
    lease.check()?;
    Ok(lease)
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
    let root = context
        .runtime
        .managed_installation_root()
        .ok_or_else(|| fail("client_native_coordination_source"))?;
    program_selection::verify_current(&root, expected)?;
    let mut lease = if relocation {
        acquire_mode(context, admission, deadline, checkpoint, true)?
    } else {
        acquire(context, admission, deadline, checkpoint)?
    };
    let result = operation(&lease);
    // Check BEFORE releasing; an elapsed callback never turns into success by
    // unlocking, or by completing subsequent image checks after the deadline.
    let valid = lease.check();
    let release = lease.release();
    let value = result?;
    valid?;
    release?;
    let (target, platform) = program_layout::host_target()?;
    let selected = program_selection::read_complete(&root, platform, target)?
        .ok_or_else(|| fail("client_native_coordination_source"))?;
    if selected != *expected
        || context.runtime.executable().as_ref() != Some(&expected.stable_entry)
    {
        return Err(fail("client_native_coordination_source"));
    }
    if Instant::now() >= deadline {
        return Err(fail("client_native_coordination_timeout"));
    }
    Ok(value)
}

#[cfg(test)]
#[path = "native_client_coordination_tests.rs"]
mod tests;
