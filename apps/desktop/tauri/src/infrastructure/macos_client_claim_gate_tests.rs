use super::*;
use std::{
    fs,
    os::unix::fs::{symlink, DirBuilderExt, PermissionsExt},
    process::{Command, Stdio},
    time::Duration,
};
const PROBE: &str =
    "infrastructure::macos_client_originals::claim_gate::tests::claim_gate_sqlite_child";
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    parent: PinnedDirectory,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap().join("original");
        fs::DirBuilder::new().mode(0o700).create(&root).unwrap();
        let c = rusqlite::Connection::open(root.join(CLAIM)).unwrap();
        c.execute_batch("CREATE TABLE marker(v); INSERT INTO marker VALUES(42)")
            .unwrap();
        c.close().unwrap();
        fs::set_permissions(root.join(CLAIM), fs::Permissions::from_mode(0o600)).unwrap();
        let parent = PinnedDirectory::open(&root).unwrap();
        Self {
            _temp: temp,
            root,
            parent,
        }
    }
    fn gate(&self) -> ClaimGate {
        ClaimGate::prepare(&self.parent, Instant::now() + Duration::from_secs(10)).unwrap()
    }
}
fn child(path: &Path, blocked: bool) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", PROBE, "--nocapture", "--test-threads=1"])
        .env("OCXD_CLAIM_GATE_PROBE", path)
        .env("OCXD_CLAIM_GATE_BLOCKED", if blocked { "1" } else { "0" })
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn claim_gate_sqlite_child() {
    let Some(path) = std::env::var_os("OCXD_CLAIM_GATE_PROBE") else {
        return;
    };
    let c = rusqlite::Connection::open_with_flags(
        Path::new(&path),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_NOFOLLOW,
    )
    .unwrap();
    c.busy_timeout(Duration::ZERO).unwrap();
    if let Some(marker) = std::env::var_os("OCXD_CLAIM_GATE_HOLD") {
        c.execute_batch("BEGIN EXCLUSIVE").unwrap();
        fs::write(marker, b"held").unwrap();
        let mut release = [0u8; 1];
        std::io::stdin().read_exact(&mut release).unwrap();
        assert_eq!(release, [1]);
        c.execute_batch("ROLLBACK").unwrap();
        c.close().unwrap();
        return;
    }
    let blocked = std::env::var("OCXD_CLAIM_GATE_BLOCKED").unwrap() == "1";
    match c.execute_batch("BEGIN EXCLUSIVE") {
        Ok(()) => {
            assert!(!blocked);
            c.execute_batch("ROLLBACK").unwrap();
        }
        Err(rusqlite::Error::SqliteFailure(e, _))
            if matches!(
                e.code,
                rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked
            ) =>
        {
            assert!(blocked)
        }
        other => panic!("unexpected compiled SQLite result: {other:?}"),
    }
    c.close().unwrap();
}
#[test]
fn claim_gate_relocation_keeps_compiled_sqlite_excluded_past_auxiliary_close() {
    let f = Fixture::new();
    let original = fs::read(f.root.join(CLAIM)).unwrap();
    let mut gate = f.gate();
    gate.acquire(&f.parent).unwrap();
    child(&f.root.join(CLAIM), true);
    // OFD ownership must survive both an unrelated fd close and an unrelated
    // in-process SQLite connection close, unlike the old process-owned probe.
    drop(File::open(f.root.join(CLAIM)).unwrap());
    rusqlite::Connection::open(f.root.join(CLAIM))
        .unwrap()
        .close()
        .unwrap();
    child(&f.root.join(CLAIM), true);
    let archive = f.root.with_file_name("retired");
    fs::rename(&f.root, &archive).unwrap();
    let moved = PinnedDirectory::open(&archive).unwrap();
    assert_eq!(moved.identity(), f.parent.identity());
    assert!(gate.check_at(&f.parent).is_err());
    gate.check_at(&moved).unwrap();
    child(&archive.join(CLAIM), true);
    drop(File::open(archive.join(CLAIM)).unwrap());
    child(&archive.join(CLAIM), true);
    gate.finish_at(&moved).unwrap();
    child(&archive.join(CLAIM), false);
    assert_eq!(fs::read(archive.join(CLAIM)).unwrap(), original);
    println!(
        "CLAIM_GATE_EVIDENCE {}",
        serde_json::json!({"accepted":false,"compiledSqliteExcludedBeforeAndAfterMove":true,"auxiliaryCloseDoesNotRelease":true,"sqliteConnectionCloseDoesNotRelease":true,"sameDirectoryIdentity":true,"normalReleaseProbed":true,"bytesUnchanged":true,"noProductionMovementAuthority":true})
    );
}
#[test]
fn claim_gate_failed_handoff_keeps_existing_sqlite_connection_locked() {
    let f = Fixture::new();
    // Pre-pin before SQLite, as required by the handoff lifetime contract.
    let mut gate = f.gate();
    let c = rusqlite::Connection::open(f.root.join(CLAIM)).unwrap();
    c.execute_batch("BEGIN EXCLUSIVE").unwrap();
    assert!(gate.acquire(&f.parent).is_err());
    // A failure must not close the prepared fd and silently release c.
    child(&f.root.join(CLAIM), true);
    c.execute_batch("ROLLBACK").unwrap();
    c.close().unwrap();
    gate.acquire(&f.parent).unwrap();
    child(&f.root.join(CLAIM), true);
    gate.finish_at(&f.parent).unwrap();
    child(&f.root.join(CLAIM), false);
}
#[test]
fn claim_gate_foreign_sqlite_owner_refuses_immediately_then_normal_handoff_succeeds() {
    let f = Fixture::new();
    let mut gate = f.gate();
    let marker = f.root.with_file_name("foreign-held");
    let mut owner = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", PROBE, "--nocapture", "--test-threads=1"])
        .env("OCXD_CLAIM_GATE_PROBE", f.root.join(CLAIM))
        .env("OCXD_CLAIM_GATE_HOLD", &marker)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let until = Instant::now() + Duration::from_secs(5);
    while !marker.exists() {
        assert!(owner.try_wait().unwrap().is_none());
        assert!(Instant::now() < until);
        std::thread::sleep(Duration::from_millis(2));
    }
    let started = Instant::now();
    assert!(gate.acquire(&f.parent).is_err());
    assert!(started.elapsed() < Duration::from_millis(500));
    child(&f.root.join(CLAIM), true);
    owner.stdin.take().unwrap().write_all(&[1]).unwrap();
    let output = owner.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    gate.acquire(&f.parent).unwrap();
    child(&f.root.join(CLAIM), true);
    gate.finish_at(&f.parent).unwrap();
    child(&f.root.join(CLAIM), false);
}
#[test]
fn claim_gate_expiry_refuses_without_unlocking_live_owner() {
    let f = Fixture::new();
    let mut gate = f.gate();
    gate.acquire(&f.parent).unwrap();
    gate.deadline = Instant::now();
    assert!(gate.check_at(&f.parent).is_err());
    child(&f.root.join(CLAIM), true);
    assert!(gate.finish_at(&f.parent).is_err());
    child(&f.root.join(CLAIM), false);
    assert!(ClaimGate::prepare(&f.parent, Instant::now()).is_err());
}
#[test]
fn claim_gate_replaced_or_unsafe_objects_refuse_without_repair() {
    for case in ["inode", "mode", "parent", "link", "hardlink"] {
        let f = Fixture::new();
        let mut gate = f.gate();
        gate.acquire(&f.parent).unwrap();
        let path = f.root.join(CLAIM);
        let retained = f.root.join("retained");
        match case {
            "inode" => {
                fs::rename(&path, &retained).unwrap();
                fs::copy(&retained, &path).unwrap();
            }
            "mode" => fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap(),
            "parent" => {
                let other = Fixture::new();
                assert!(gate.check_at(&other.parent).is_err());
                continue;
            }
            "link" => {
                fs::rename(&path, &retained).unwrap();
                symlink(&retained, &path).unwrap();
            }
            "hardlink" => fs::hard_link(&path, &retained).unwrap(),
            _ => unreachable!(),
        }
        assert!(gate.check_at(&f.parent).is_err());
        assert!(
            ClaimGate::prepare(&f.parent, Instant::now() + Duration::from_secs(5)).is_err()
                || case == "inode"
        );
        if case == "mode" {
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o644
            );
        }
    }
}
#[test]
fn claim_gate_new_logical_root_is_not_the_retained_archive() {
    let f = Fixture::new();
    let mut gate = f.gate();
    gate.acquire(&f.parent).unwrap();
    let archive = f.root.with_file_name("retired");
    fs::rename(&f.root, &archive).unwrap();
    fs::DirBuilder::new().mode(0o700).create(&f.root).unwrap();
    fs::write(f.root.join("user-data"), b"outside").unwrap();
    let replacement = PinnedDirectory::open(&f.root).unwrap();
    assert!(gate.check_at(&replacement).is_err());
    assert!(ClaimGate::prepare(&replacement, Instant::now() + Duration::from_secs(5)).is_err());
    assert!(!f.root.join(CLAIM).exists());
    assert_eq!(fs::read(f.root.join("user-data")).unwrap(), b"outside");
    let moved = PinnedDirectory::open(&archive).unwrap();
    gate.check_at(&moved).unwrap();
    child(&archive.join(CLAIM), true);
    drop(gate);
    child(&archive.join(CLAIM), false);
}
#[test]
fn claim_gate_actual_owner_exit_releases_original_inode_without_cleanup_signal() {
    const ENV: &str = "OCXD_CLAIM_GATE_EXIT_ROOT";
    if let Some(path) = std::env::var_os(ENV) {
        let parent = PinnedDirectory::open(Path::new(&path)).unwrap();
        let mut gate =
            ClaimGate::prepare(&parent, Instant::now() + Duration::from_secs(10)).unwrap();
        gate.acquire(&parent).unwrap();
        let archive = Path::new(&path).with_file_name("retired");
        fs::rename(Path::new(&path), &archive).unwrap();
        let moved = PinnedDirectory::open(&archive).unwrap();
        gate.check_at(&moved).unwrap();
        child(&archive.join(CLAIM), true);
        std::process::exit(91);
    }
    let f = Fixture::new();
    let before = fs::read(f.root.join(CLAIM)).unwrap();
    let output=Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::claim_gate::tests::claim_gate_actual_owner_exit_releases_original_inode_without_cleanup_signal","--nocapture","--test-threads=1"]).env(ENV,&f.root).output().unwrap();
    assert_eq!(
        output.status.code(),
        Some(91),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let archive = f.root.with_file_name("retired");
    assert!(!f.root.exists());
    child(&archive.join(CLAIM), false);
    assert_eq!(fs::read(archive.join(CLAIM)).unwrap(), before);
    println!(
        "CLAIM_GATE_EXIT_EVIDENCE {}",
        serde_json::json!({"accepted":false,"actualWaitExitCode":91,"originalInodeReleasedAfterMove":true,"noCleanupSignal":true,"bytesUnchanged":true})
    );
}
#[test]
fn claim_gate_release_owner_drop_is_reusable_but_cannot_be_double_acquired() {
    let f = Fixture::new();
    let mut gate = f.gate();
    assert!(gate.check_at(&f.parent).is_err());
    gate.acquire(&f.parent).unwrap();
    assert!(gate.acquire(&f.parent).is_err());
    child(&f.root.join(CLAIM), true);
    drop(gate);
    child(&f.root.join(CLAIM), false);
}
