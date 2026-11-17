//! Exact adopted rollback-mode stable-lock release observations. Production
//! uses read-only original handles and native byte-range locking, never SQLite
//! database open/recovery, data writes, owner replacement or Closed authority.
use super::*;
use windows_sys::Win32::{
    Storage::FileSystem::{
        LockFileEx, UnlockFileEx, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY,
    },
    System::IO::OVERLAPPED,
};
const LOCKS: [&str; 2] = [
    ".opencodex-native-main.claim.sqlite",
    ".opencodex-native-main.owner.sqlite",
];
// SQLite rollback-mode Windows VFS protocol: pending1+reserved1+shared510.
const PENDING_BYTE: u32 = 0x40000000;
const LOCK_BYTES: u32 = 512;
struct Guard {
    file: File,
    overlap: OVERLAPPED,
    held: bool,
}
impl Guard {
    fn acquire(file: File) -> io::Result<Self> {
        let mut out = Self {
            file,
            overlap: unsafe { std::mem::zeroed() },
            held: false,
        };
        out.overlap.Anonymous.Anonymous.Offset = PENDING_BYTE;
        if unsafe {
            LockFileEx(
                out.file.as_raw_handle(),
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0,
                LOCK_BYTES,
                0,
                &mut out.overlap,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        out.held = true;
        Ok(out)
    }
    fn release(&mut self) -> io::Result<()> {
        if !self.held {
            return Err(invalid());
        }
        if unsafe {
            UnlockFileEx(
                self.file.as_raw_handle(),
                0,
                LOCK_BYTES,
                0,
                &mut self.overlap,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        self.held = false;
        Ok(())
    }
}
impl Drop for Guard {
    fn drop(&mut self) {
        if self.held {
            let _ = self.release();
        }
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
fn no_sidecars(root: &Path) -> io::Result<()> {
    for name in LOCKS {
        for suffix in ["-journal", "-wal", "-shm"] {
            require_absent(&root.join(format!("{name}{suffix}")))?;
        }
    }
    Ok(())
}
fn acquire(root: &Path, entries: &[Entry], end: Instant) -> io::Result<Vec<Guard>> {
    no_sidecars(root)?;
    let mut guards = Vec::new();
    for name in LOCKS {
        check(end)?;
        if let Some(expected) = entries.iter().find(|entry| entry.relative == name) {
            if expected.directory {
                return Err(invalid());
            }
            let file = OpenOptions::new()
                .access_mode(0x80020000)
                .share_mode(1 | 2)
                .custom_flags(0x200000)
                .open(root.join(name))?;
            same_object(&file, expected, end)?;
            let guard = Guard::acquire(file)?;
            same_object(&guard.file, expected, end)?;
            guards.push(guard);
        } else {
            require_absent(&root.join(name))?;
        }
    }
    check(end)?;
    Ok(guards)
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
    let mut guards = acquire(root, entries, end)?;
    // Both ranges are held simultaneously on the exact objects. This is a
    // window observation, not proof against future/mmap/malicious raw writes.
    no_sidecars(root)?;
    check(end)?;
    for guard in &mut guards {
        guard.release()?;
    }
    for (guard, expected) in guards.iter().zip(
        LOCKS
            .into_iter()
            .filter_map(|name| entries.iter().find(|entry| entry.relative == name)),
    ) {
        same_object(&guard.file, expected, end)?;
    }
    parent.recheck()?;
    if id(root_handle, true)? != original_root.id
        || native_stamp(root_handle, end).map_err(failure)? != original_root.security
    {
        return Err(invalid());
    }
    no_sidecars(root)?;
    for name in LOCKS {
        if !entries.iter().any(|entry| entry.relative == name) {
            require_absent(&root.join(name))?;
        }
    }
    check(end)
}
#[cfg(test)]
mod tests {
    use super::super::tests::Fixture;
    use super::*;
    use rusqlite::Connection;
    use std::time::Duration;
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
        let mut held = acquire(&root, &original.entries, end()).unwrap();
        let contender = Connection::open(root.join(LOCKS[0])).unwrap();
        contender.busy_timeout(Duration::ZERO).unwrap();
        let error = contender.execute_batch("BEGIN EXCLUSIVE").unwrap_err();
        assert!(
            matches!(error,rusqlite::Error::SqliteFailure(code,_) if code.extended_code==rusqlite::ffi::SQLITE_BUSY)
        );
        contender.close().unwrap();
        for guard in &mut held {
            guard.release().unwrap();
        }
        drop(held);
        let custody = create_private_directory(&temp.path().join("custody")).unwrap();
        let journal = create_private_directory(&temp.path().join("journal")).unwrap();
        original
            .move_to_custody(custody.path(), &journal.path().join("intent.json"), end())
            .unwrap();
        original.create_working_copy(end()).unwrap();
        original.restore_original(end()).unwrap();
        original.verify_restored(end()).unwrap();
        println!("native two exact stable SQLite files restored same IDs/bytes/8projections; actual READ-only native handles, both fail-immediate exclusive protocol ranges acquired/released, real SQLite contention refused; no journal or Closed authority");
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
        assert_eq!(error.raw_os_error(), Some(33));
        assert!(!held.is_autocommit());
        println!("native original second stable lock held: rawWin32=33 {error:?}; no forced unlock or success");
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
