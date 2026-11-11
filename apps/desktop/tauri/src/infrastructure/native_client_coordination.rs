//! The mutation owner retains the original SQLite locks in THIS process.
//! Official admission/initialization is first performed unchanged by the
//! existing helper. A gap during handover grants no authority: all three file
//! generations and the official-owned journal receipt must match after native
//! acquisition. Nothing recreates a database or writes a copied schema.
use super::*;
use crate::infrastructure::macos_client_originals::{ownership_receipt, CoordinationFilePin};
use std::{cell::Cell, marker::PhantomData};

/// Borrow-scoped native ownership. There is no timer thread or external helper
/// that can release these connections while the callback continues. Expired
/// work must stop and remains a failure, but locks last until its stack unwinds.
/// This proves coordinator ownership only, not service exit or file provenance.
pub struct NativeLease {
    connections: Vec<rusqlite::Connection>,
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
        self.client.recheck().map_err(io)?;
        self.profile.recheck().map_err(io)?;
        self.home.recheck().map_err(io)?;
        for pin in &self.pins {
            pin.recheck().map_err(io)?;
        }
        if ownership_receipt(&self.client).map_err(io)? != self.owner {
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
    fn release(&mut self) -> AppResult<()> {
        // Close every SQLite connection before dropping ANY raw file pin for
        // these inodes. POSIX record locks belong to a process, not one fd.
        self.active.set(false);
        let mut failed = false;
        while let Some(connection) = self.connections.pop() {
            if connection.execute_batch("ROLLBACK").is_err() {
                failed = true;
            }
            if let Err((connection, _)) = connection.close() {
                drop(connection);
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
    let mut lease = NativeLease {
        connections: Vec::new(),
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
    // Exact official claim -> N -> K databases and transaction strengths;
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
        lease.connections.push(connection);
        require_original_sqlite_file(lease.connections.last().unwrap())?;
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
    let (admission, selected) = with_coordination_inner(
        context,
        deadline,
        |helper| {
            helper.check()?;
            Ok((
                helper
                    .held
                    .clone()
                    .ok_or_else(|| fail("client_native_coordination_missing_admission"))?,
                helper.selected.clone(),
            ))
        },
        false,
    )?;
    with_admission(context, &admission, &selected, deadline, operation, &|_| {
        Ok(())
    })
}
fn with_admission<T>(
    context: &Context,
    admission: &Record,
    expected: &program_selection::SelectedProgram,
    deadline: Instant,
    operation: impl FnOnce(&NativeLease) -> AppResult<T>,
    checkpoint: &impl Fn(usize) -> AppResult<()>,
) -> AppResult<T> {
    let root = context
        .runtime
        .managed_installation_root()
        .ok_or_else(|| fail("client_native_coordination_source"))?;
    program_selection::verify_current(&root, expected)?;
    let mut lease = acquire(context, admission, deadline, checkpoint)?;
    let result = operation(&lease);
    // Check BEFORE releasing; an elapsed callback never turns into success by
    // unlocking, or by completing subsequent image checks after the deadline.
    let valid = lease.check();
    let release = lease.release();
    let value = result?;
    valid?;
    release?;
    let (target, platform) = program_layout::host_target()?;
    let selected = program_selection::read(&root, platform, target)?
        .ok_or_else(|| fail("client_native_coordination_source"))?;
    if selected != *expected
        || context.runtime.executable().as_ref() != Some(&expected.stable_entry)
    {
        return Err(fail("client_native_coordination_source"));
    }
    program_image::verify(&selected)?;
    if Instant::now() >= deadline {
        return Err(fail("client_native_coordination_timeout"));
    }
    Ok(value)
}

#[cfg(test)]
#[path = "native_client_coordination_tests.rs"]
mod tests;
