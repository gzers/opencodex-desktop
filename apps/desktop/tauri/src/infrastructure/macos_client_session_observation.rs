//! Observation never initializes, repairs, migrates or publishes an index.
//! Acquire the separate shared action lock BEFORE opening the SQLite inode:
//! otherwise closing an auxiliary fd could release a same-process writer's
//! POSIX locks. Busy/unsafe/interrupted state is unknown, never an empty index.
use super::*;
use std::fs;
use std::os::unix::fs::FileExt;
fn no_sidecars(manager: &PinnedDirectory) -> io::Result<()> {
    for suffix in ["-journal", "-wal", "-shm"] {
        if entry(manager, &format!("{FILE}{suffix}"))?.is_some() {
            return Err(denied(
                "interrupted index sidecar requires action-time recovery",
            ));
        }
    }
    Ok(())
}
fn rollback_header(pin: &CoordinationFilePin) -> io::Result<()> {
    let size = pin.file.metadata()?.len();
    if !(100..=MAX_INDEX).contains(&size) {
        return Err(denied("index observation size unknown"));
    }
    let mut header = [0u8; 100];
    pin.file.read_exact_at(&mut header, 0)?;
    if &header[..16] != b"SQLite format 3\0" || header[18] != 1 || header[19] != 1 {
        return Err(denied("index is not a complete rollback-mode snapshot"));
    }
    Ok(())
}

pub(in crate::infrastructure::macos_client_originals) fn entry(
    parent: &PinnedDirectory,
    child: &str,
) -> io::Result<Option<libc::stat>> {
    let child = name(OsStr::new(child))?;
    let mut value = std::mem::MaybeUninit::uninit();
    if unsafe {
        libc::fstatat(
            parent.file().as_raw_fd(),
            child.as_ptr(),
            value.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } != 0
    {
        let e = io::Error::last_os_error();
        return if e.kind() == io::ErrorKind::NotFound {
            Ok(None)
        } else {
            Err(e)
        };
    }
    Ok(Some(unsafe { value.assume_init() }))
}
fn action_pin(parent: &PinnedDirectory, file: &File) -> io::Result<(Identity, Protection)> {
    parent.private()?;
    let s = stat(file)?;
    let path = entry(parent, "client-session-action.lock")?
        .ok_or_else(|| denied("action lock disappeared"))?;
    let p = protection(file, &s)?;
    if identity(&path) != identity(&s)
        || path.st_mode != s.st_mode
        || path.st_uid != s.st_uid
        || path.st_gid != s.st_gid
        || path.st_flags != s.st_flags
        || path.st_nlink != 1
        || s.st_nlink != 1
        || s.st_mode & libc::S_IFMT != libc::S_IFREG
        || p.uid != unsafe { libc::geteuid() }
        || p.mode != 0o600
        || p.acl.is_some()
        || p.flags != 0
        || p.xattrs
            .iter()
            .any(|(k, v)| k != b"com.apple.provenance" || v.len() > 64)
    {
        return Err(denied("unsafe observation action lock; not repaired"));
    }
    Ok((identity(&s), p))
}

pub fn inspect(context: &ProcessContext, data_root: &Path) -> io::Result<Option<Head>> {
    inspect_map(context, data_root, |head| Ok(head.cloned()))
}
/// Keep the shared action lock until the projection has finished reading its
/// independently referenced original evidence. No later unlocked second read.
pub(super) fn inspect_map<T>(
    context: &ProcessContext,
    data_root: &Path,
    project: impl FnOnce(Option<&Head>) -> io::Result<T>,
) -> io::Result<T> {
    inspect_map_history(context, data_root, |head, _history| project(head))
}
pub(super) fn inspect_map_history<T>(
    context: &ProcessContext,
    data_root: &Path,
    project: impl FnOnce(Option<&Head>, &[Head]) -> io::Result<T>,
) -> io::Result<T> {
    let data = PinnedDirectory::open(data_root)?;
    data.private()?;
    if entry(&data, "manager-state")?.is_none() {
        return project(None, &[]);
    }
    let manager = data.child_private("manager-state", false)?;
    let action = entry(&manager, "client-session-action.lock")?;
    let index = entry(&manager, FILE)?;
    if action.is_none() && index.is_none() {
        return if entry(&manager, "client-originals")?.is_none() {
            project(None, &[])
        } else {
            Err(denied("originals remain without their independent index"))
        };
    }
    if action.is_none() || index.is_none() {
        return Err(denied("incomplete current session initializer"));
    }
    let action_file = open_at(
        manager.file(),
        OsStr::new("client-session-action.lock"),
        false,
    )?;
    let action_before = action_pin(&manager, &action_file)?;
    action_file
        .try_lock_shared()
        .map_err(|_| denied("current session action is busy"))?;
    if action_pin(&manager, &action_file)? != action_before {
        return Err(denied("observation lock changed"));
    }
    let index_stat = entry(&manager, FILE)?.ok_or_else(|| denied("current index disappeared"))?;
    if index_stat.st_size <= 0 || index_stat.st_size as u64 > MAX_INDEX {
        return Err(denied("current index size unknown"));
    }
    let client_path = fs::canonicalize(context.working_directory.join(".codex"))?;
    let client = PinnedDirectory::open(&client_path)?;
    if data_root.starts_with(&client_path) || client_path.starts_with(data_root) {
        return Err(denied("session roots overlap"));
    }
    let binding = scope(context, &data, &client)?;
    // Pins outlive the connection. Never open/close a second index fd after
    // SQLite has acquired a process-associated record lock.
    let pin = CoordinationFilePin::open(&manager.path().join(FILE))?;
    no_sidecars(&manager)?;
    rollback_header(&pin)?;
    // immutable forbids SQLite recovery/sidecar creation even if a foreign
    // writer changes the header. The outer flock excludes cooperative writers;
    // header/sidecar/VFS/path checks still reject interrupted or changed views.
    let mut uri = reqwest::Url::from_file_path(manager.path().join(FILE))
        .map_err(|_| denied("index URI unknown"))?;
    uri.query_pairs_mut()
        .append_pair("mode", "ro")
        .append_pair("immutable", "1");
    let connection = Connection::open_with_flags(
        uri.as_str(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
            | rusqlite::OpenFlags::SQLITE_OPEN_URI
            | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW
            | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(database_error)?;
    connection
        .busy_timeout(std::time::Duration::ZERO)
        .map_err(database_error)?;
    unsafe {
        rusqlite::ffi::sqlite3_limit(
            connection.handle(),
            rusqlite::ffi::SQLITE_LIMIT_LENGTH,
            65536,
        );
    }
    connection
        .execute_batch("PRAGMA query_only=ON; BEGIN")
        .map_err(database_error)?;
    let version: i64 = connection
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(database_error)?;
    if version != 3 {
        return Err(denied("unsupported current index; not migrated"));
    }
    validate_schema(&connection)?;
    let stored: Vec<u8> = connection
        .query_row("SELECT document FROM binding WHERE singleton=1", [], |r| {
            r.get(0)
        })
        .map_err(database_error)?;
    if stored != binding {
        return Err(denied("observation HOME or directory generation differs"));
    }
    let result = read_head(&connection)?;
    let history = if let Some(head) = result
        .as_ref()
        .filter(|h| matches!(h.state, State::Preparing | State::CancelledBeforeCapture))
    {
        vault_history::read(&connection, head)?
    } else {
        Vec::new()
    };
    if let Some(head) = result
        .as_ref()
        .filter(|h| h.state == State::CancelledBeforeCapture)
    {
        preparation::verify_registered(
            &data,
            &client,
            &context.working_directory,
            &binding,
            head,
            &history,
        )?;
    }
    if let Some(head) = result.as_ref().filter(|h| h.state == State::Unchanged) {
        unchanged::verify_registered(data.path(), client.path(), &context.working_directory, head)?;
    }
    if let Some(head) = result
        .as_ref()
        .filter(|head| matches!(head.state, State::Restored | State::RestoreRolledBack))
    {
        completion::verify_registered(
            data.path(),
            client.path(),
            &context.working_directory,
            head,
        )?;
    }
    if result.is_none() && entry(&manager, "client-originals")?.is_some() {
        return Err(denied("originals remain without current references"));
    }
    let mut moved = 1i32;
    let status = unsafe {
        rusqlite::ffi::sqlite3_file_control(
            connection.handle(),
            c"main".as_ptr(),
            rusqlite::ffi::SQLITE_FCNTL_HAS_MOVED,
            (&mut moved as *mut i32).cast(),
        )
    };
    if status != rusqlite::ffi::SQLITE_OK || moved != 0 {
        return Err(denied("observed SQLite file changed"));
    }
    data.private()?;
    client.recheck()?;
    pin.recheck()?;
    no_sidecars(&manager)?;
    rollback_header(&pin)?;
    if scope(context, &data, &client)? != binding
        || action_pin(&manager, &action_file)? != action_before
    {
        return Err(denied("observation context changed"));
    }
    connection.execute_batch("COMMIT").map_err(database_error)?;
    connection
        .close()
        .map_err(|_| denied("observation connection did not close"))?;
    pin.recheck()?;
    let projected = project(result.as_ref(), &history)?;
    data.private()?;
    client.recheck()?;
    pin.recheck()?;
    if scope(context, &data, &client)? != binding
        || action_pin(&manager, &action_file)? != action_before
    {
        return Err(denied(
            "observation context changed during evidence projection",
        ));
    }
    Ok(projected)
}
