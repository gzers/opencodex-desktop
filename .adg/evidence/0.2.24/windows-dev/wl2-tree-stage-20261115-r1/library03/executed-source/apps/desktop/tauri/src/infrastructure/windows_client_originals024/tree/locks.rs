//! Exact adopted stable-lock release observations. No file creation, owner
//! replacement, SQLite write, shared state repair or Closed authority here.
use super::*;
use rusqlite::{Connection, OpenFlags};
use std::{mem::ManuallyDrop, os::windows::io::FromRawHandle, time::Duration};

const LOCKS: [&str; 2] = [
    ".opencodex-native-main.claim.sqlite",
    ".opencodex-native-main.owner.sqlite",
];
fn sqlite_error(error: rusqlite::Error) -> io::Error {
    io::Error::other(error)
}
struct Pair {
    connections: Vec<Connection>,
}
impl Pair {
    fn release(&mut self) -> io::Result<()> {
        let mut failure = None;
        while let Some(connection) = self.connections.pop() {
            if let Err(error) = connection.execute_batch("ROLLBACK") {
                failure = Some(sqlite_error(error));
            }
            if let Err((connection, error)) = connection.close() {
                drop(connection);
                failure = Some(sqlite_error(error));
            }
        }
        match failure {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}
impl Drop for Pair {
    fn drop(&mut self) {
        let _ = self.release();
    }
}
fn same_object(file: &File, expected: &Entry, end: Instant) -> io::Result<()> {
    if id(file, false)? != expected.id
        || native_stamp(file, end).map_err(failure)? != expected.security
    {
        return Err(invalid());
    }
    check(end)
}
fn sqlite_object(connection: &Connection, expected: &Entry, end: Instant) -> io::Result<()> {
    let mut handle: windows_sys::Win32::Foundation::HANDLE = std::ptr::null_mut();
    let code = unsafe {
        rusqlite::ffi::sqlite3_file_control(
            connection.handle(),
            c"main".as_ptr(),
            rusqlite::ffi::SQLITE_FCNTL_WIN32_GET_HANDLE,
            (&mut handle as *mut windows_sys::Win32::Foundation::HANDLE).cast(),
        )
    };
    if code != rusqlite::ffi::SQLITE_OK
        || handle.is_null()
        || handle == windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE
    {
        return Err(io::Error::other(format!(
            "SQLite native handle unavailable raw={code}"
        )));
    }
    // Borrowed SQLite-owned HANDLE. Never close it or transfer ownership.
    let borrowed = ManuallyDrop::new(unsafe { File::from_raw_handle(handle) });
    same_object(&borrowed, expected, end)
}
fn no_sidecars(root: &Path) -> io::Result<()> {
    for name in LOCKS {
        for suffix in ["-journal", "-wal", "-shm"] {
            require_absent(&root.join(format!("{name}{suffix}")))?;
        }
    }
    Ok(())
}
pub(super) fn prove(
    root: &Path,
    root_handle: &File,
    entries: &[Entry],
    end: Instant,
) -> io::Result<()> {
    check(end)?;
    let parent = PinnedDirectory::open(root.parent().ok_or_else(invalid)?)?;
    let original_root = entries
        .iter()
        .find(|entry| entry.relative.is_empty() && entry.directory)
        .ok_or_else(invalid)?;
    if id(root_handle, true)? != original_root.id
        || native_stamp(root_handle, end).map_err(failure)? != original_root.security
    {
        return Err(invalid());
    }
    no_sidecars(root)?;
    let mut pins = Vec::new();
    for name in LOCKS {
        let expected = entries.iter().find(|entry| entry.relative == name);
        if let Some(expected) = expected {
            if expected.directory {
                return Err(invalid());
            }
            let file = OpenOptions::new()
                .access_mode(0x20000)
                .share_mode(1 | 2)
                .custom_flags(0x200000)
                .open(root.join(name))?;
            same_object(&file, expected, end)?;
            pins.push((file, expected));
        } else {
            require_absent(&root.join(name))?;
        }
    }
    // Pins deny delete/replacement while allowing data opens needed by SQLite.
    let mut pair = Pair {
        connections: Vec::new(),
    };
    for (_, expected) in &pins {
        check(end)?;
        let connection = Connection::open_with_flags(
            root.join(&expected.relative),
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_NOFOLLOW
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(sqlite_error)?;
        connection
            .busy_timeout(Duration::ZERO)
            .map_err(sqlite_error)?;
        sqlite_object(&connection, expected, end)?;
        connection
            .execute_batch("BEGIN EXCLUSIVE")
            .map_err(sqlite_error)?;
        pair.connections.push(connection);
    }
    for (connection, (_, expected)) in pair.connections.iter().zip(&pins) {
        if connection.is_autocommit() {
            return Err(invalid());
        }
        sqlite_object(connection, expected, end)?;
    }
    check(end)?;
    pair.release()?;
    for (file, expected) in &pins {
        same_object(file, expected, end)?;
    }
    parent.recheck()?;
    if id(root_handle, true)? != original_root.id
        || native_stamp(root_handle, end).map_err(failure)? != original_root.security
    {
        return Err(invalid());
    }
    no_sidecars(root)?;
    for name in LOCKS {
        if !entries.iter().any(|e| e.relative == name) {
            require_absent(&root.join(name))?;
        }
    }
    check(end)
}

#[cfg(test)]
mod tests {
    use super::super::tests::Fixture;
    use super::*;
    fn end() -> Instant {
        Instant::now() + Duration::from_secs(10)
    }
    fn make_pair(root: &Path) {
        for name in LOCKS {
            let c = Connection::open(root.join(name)).unwrap();
            c.execute_batch("CREATE TABLE coordination (token TEXT); INSERT INTO coordination VALUES ('synthetic');").unwrap();
            c.close().unwrap();
        }
    }
    #[test]
    fn wl2_r4_stable_locks_exact_native_sqlite_pair_releases_without_mutation() {
        let temp = Fixture::new();
        let root = temp.path().join(".codex");
        std::fs::create_dir(&root).unwrap();
        make_pair(&root);
        let mut original = RootTree::capture(&root, end()).unwrap();
        prove(
            &root,
            original.root.as_ref().unwrap(),
            &original.entries,
            end(),
        )
        .unwrap();
        original.check_original(end()).unwrap();
        let custody = create_private_directory(&temp.path().join("custody")).unwrap();
        let journal = create_private_directory(&temp.path().join("journal")).unwrap();
        original
            .move_to_custody(custody.path(), &journal.path().join("intent.json"), end())
            .unwrap();
        original.create_working_copy(end()).unwrap();
        original.restore_original(end()).unwrap();
        original.verify_restored(end()).unwrap();
        println!("native two exact stable SQLite files restored same IDs/bytes/8projections; actual Win32 VFS handles, both zero-timeout exclusive locks acquired/released; no journal or Closed authority");
    }
    #[test]
    fn wl2_r4_stable_locks_busy_and_unknown_journal_refuse_and_keep_scene() {
        let mut temp = Fixture::new();
        let root = temp.path().join(".codex");
        std::fs::create_dir(&root).unwrap();
        make_pair(&root);
        let mut original = RootTree::capture(&root, end()).unwrap();
        let held = Connection::open(root.join(LOCKS[1])).unwrap();
        held.busy_timeout(Duration::ZERO).unwrap();
        held.execute_batch("BEGIN EXCLUSIVE").unwrap();
        let error = prove(
            &root,
            original.root.as_ref().unwrap(),
            &original.entries,
            end(),
        )
        .unwrap_err();
        let native = error
            .get_ref()
            .and_then(|error| error.downcast_ref::<rusqlite::Error>())
            .expect("preserve actual SQLite error type");
        assert!(
            matches!(native,rusqlite::Error::SqliteFailure(code,_) if code.extended_code==rusqlite::ffi::SQLITE_BUSY)
        );
        assert!(!held.is_autocommit());
        println!("native original second stable lock held: rawSQLite=5 {error:?}; no forced unlock or success");
        held.execute_batch("ROLLBACK").unwrap();
        held.close().unwrap();
        original.check_original(end()).unwrap();
        let journal = root.join(format!("{}-journal", LOCKS[0]));
        std::fs::write(&journal, b"external synthetic unknown journal").unwrap();
        assert!(prove(
            &root,
            original.root.as_ref().unwrap(),
            &original.entries,
            end()
        )
        .is_err());
        assert_eq!(
            std::fs::read(&journal).unwrap(),
            b"external synthetic unknown journal"
        );
        assert!(
            RootTree::capture(&root, end()).is_err(),
            "unknown sidecar refuses before capture mutation"
        );
        drop(original);
        temp.retain();
    }
    #[test]
    fn wl2_r4_stable_locks_absence_does_not_create_or_adopt_new_lock() {
        let mut temp = Fixture::new();
        let root = temp.path().join(".codex");
        std::fs::create_dir(&root).unwrap();
        let original = RootTree::capture(&root, end()).unwrap();
        prove(
            &root,
            original.root.as_ref().unwrap(),
            &original.entries,
            end(),
        )
        .unwrap();
        assert!(names(&root).unwrap().is_empty());
        std::fs::write(root.join(LOCKS[0]), b"foreign synthetic new lock").unwrap();
        assert!(prove(
            &root,
            original.root.as_ref().unwrap(),
            &original.entries,
            end()
        )
        .is_err());
        drop(original);
        temp.retain();
    }
}
