use super::super::tree::tests::Fixture;
use super::*;
use native::{create_private_directory, PrivateRecord};
use rusqlite::Connection;
use std::time::Duration;
fn end() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
fn prepare(f: &Fixture) -> [PathBuf; 4] {
    let dir = create_private_directory(&f.path().join("private-locks")).unwrap();
    std::array::from_fn(|i| {
        let path = dir.path().join(format!("synthetic-{i}.sqlite"));
        drop(PrivateRecord::create(&path, b"", 16).unwrap());
        let connection = Connection::open(&path).unwrap();
        connection.execute_batch("PRAGMA journal_mode=DELETE; CREATE TABLE coordination (value INTEGER); INSERT INTO coordination VALUES(1)").unwrap();
        connection.close().unwrap();
        path
    })
}
fn competitor(path: &Path) -> Connection {
    let c = Connection::open(path).unwrap();
    c.busy_timeout(Duration::ZERO).unwrap();
    c
}
fn busy(result: rusqlite::Result<()>) {
    assert!(
        matches!(result,Err(rusqlite::Error::SqliteFailure(code,_)) if code.code==rusqlite::ErrorCode::DatabaseBusy || code.code==rusqlite::ErrorCode::DatabaseLocked)
    );
}
fn originals(paths: &[PathBuf; 4]) -> Vec<(FileIdentity, SnapshotData, Vec<u8>)> {
    paths
        .iter()
        .map(|p| {
            let f = open(p).unwrap();
            (
                file_id(&f).unwrap(),
                native_stamp(&f, end()).unwrap(),
                std::fs::read(p).unwrap(),
            )
        })
        .collect()
}
#[test]
fn wl2_r5_coordination_original_ranges_exclude_real_sqlite_writer_without_data_writes() {
    let f = Fixture::new();
    let paths = prepare(&f);
    let before_images = originals(&paths);
    let deadline = end();
    let mut lease = NativeLease::acquire_paths(&paths, deadline).unwrap();
    for (i, path) in paths.iter().enumerate() {
        let c = competitor(path);
        busy(c.execute_batch("BEGIN IMMEDIATE"));
        if !c.is_autocommit() {
            c.execute_batch("ROLLBACK").unwrap();
        }
        if i > 0 {
            c.execute_batch("BEGIN").unwrap();
            assert_eq!(
                c.query_row("SELECT value FROM coordination", [], |r| r.get::<_, i32>(0))
                    .unwrap(),
                1
            );
            c.execute_batch("ROLLBACK").unwrap();
        } else {
            c.execute_batch("BEGIN").unwrap();
            assert!(c
                .query_row("SELECT value FROM coordination", [], |r| r.get::<_, i32>(0))
                .is_err());
            c.execute_batch("ROLLBACK").unwrap();
        }
        c.close().unwrap();
        lease.check(deadline).unwrap();
    }
    lease.release(deadline).unwrap();
    for path in &paths {
        let c = competitor(path);
        c.execute_batch("BEGIN IMMEDIATE; ROLLBACK").unwrap();
        c.close().unwrap();
    }
    assert!(originals(&paths) == before_images);
    println!("r5 Windows native claim EXCLUSIVE and N/L/K shared+reserved ranges exclude actual SQLite writers; N/L/K readers coexist, claim reader refuses; release same4 IDs/8projections/bytes, no production DB open/schema/official process");
}
#[test]
fn wl2_r5_coordination_busy_each_role_unwinds_only_owned_partial_locks() {
    for index in 0..4 {
        let mut f = Fixture::new();
        let paths = prepare(&f);
        let before_images = originals(&paths);
        let c = competitor(&paths[index]);
        c.execute_batch("BEGIN IMMEDIATE").unwrap();
        let error = match NativeLease::acquire_paths(&paths, end()) {
            Err(e) => e,
            Ok(_) => panic!("foreign writer admitted"),
        };
        assert_eq!(error.raw_os_error(), Some(33));
        println!("r5 coordination heldRole={index} rawWin32={:?} partial callback not admitted, original foreign SQLite transaction untouched",error.raw_os_error());
        assert!(!c.is_autocommit());
        c.execute_batch("ROLLBACK").unwrap();
        c.close().unwrap();
        for path in &paths {
            let after = competitor(path);
            after.execute_batch("BEGIN EXCLUSIVE; ROLLBACK").unwrap();
            after.close().unwrap();
        }
        assert!(originals(&paths) == before_images);
        f.retain();
    }
}
#[test]
fn wl2_r5_coordination_sidecar_header_ads_and_multiple_link_refuse() {
    for variant in 0..5 {
        let mut f = Fixture::new();
        let paths = prepare(&f);
        match variant {
            0 => std::fs::write(
                format!("{}-journal", paths[2].display()),
                b"foreign journal",
            )
            .unwrap(),
            1 => std::fs::write(&paths[2], b"not sqlite").unwrap(),
            2 => {
                let mut raw = std::fs::read(&paths[2]).unwrap();
                raw[18] = 2;
                std::fs::write(&paths[2], raw).unwrap();
            }
            3 => std::fs::write(format!("{}:foreign", paths[2].display()), b"unknown stream")
                .unwrap(),
            _ => std::fs::hard_link(&paths[2], paths[2].with_extension("alias")).unwrap(),
        }
        let raw = std::fs::read(&paths[2]).unwrap();
        assert!(NativeLease::acquire_paths(&paths, end()).is_err());
        assert_eq!(std::fs::read(&paths[2]).unwrap(), raw);
        // First two partial roles must never remain held by a failed acquire.
        for path in &paths[..2] {
            let c = competitor(path);
            c.execute_batch("BEGIN EXCLUSIVE; ROLLBACK").unwrap();
            c.close().unwrap();
        }
        println!("r5 coordination rejected variant={variant} without repair, delete or fallback; original bytes retained");
        f.retain();
    }
}
#[test]
fn wl2_r5_coordination_expiry_keeps_locks_until_scope_release_and_never_succeeds() {
    let f = Fixture::new();
    let paths = prepare(&f);
    let before_images = originals(&paths);
    assert!(NativeLease::acquire_paths(&paths, Instant::now()).is_err());
    let deadline = Instant::now() + Duration::from_secs(1);
    let mut lease = NativeLease::acquire_paths(&paths, deadline).unwrap();
    assert!(lease.check(deadline + Duration::from_millis(1)).is_err());
    assert!(
        lease.check(deadline).is_err(),
        "failed validation must not automatically re-admit the same lease"
    );
    // Force a private expired observation without changing the original outer
    // action budgets or waiting for a replacement timer/worker to release it.
    lease.deadline = Instant::now();
    assert!(lease.check(lease.deadline).is_err());
    let c = competitor(&paths[3]);
    busy(c.execute_batch("BEGIN IMMEDIATE"));
    c.close().unwrap();
    assert!(lease.release(Instant::now()).is_err());
    let c = competitor(&paths[3]);
    c.execute_batch("BEGIN IMMEDIATE; ROLLBACK").unwrap();
    c.close().unwrap();
    assert!(originals(&paths) == before_images);
    println!("r5 expired private lease cannot authorize callback; actual locks survive until stack release, own locks released despite Timeout, no timer grant or rawexit rewrite");
}
#[test]
fn wl2_r5_coordination_missing_files_and_duplicate_names_never_create_locks() {
    let f = Fixture::new();
    let root = f.path().join("never-created");
    let paths = std::array::from_fn(|i| root.join(format!("{i}.sqlite")));
    assert!(NativeLease::acquire_paths(&paths, end()).is_err());
    assert!(!root.exists());
    let paths = prepare(&f);
    let duplicate = std::array::from_fn(|_| paths[0].clone());
    assert!(NativeLease::acquire_paths(&duplicate, end()).is_err());
    println!("r5 missing namespace and duplicate role paths refuse with no mkdir/create/init or namespace fallback");
}
#[test]
fn wl2_r5_coordination_raw_data_drift_poisons_lease_without_overwriting_conflict() {
    use std::io::Write;
    let mut f = Fixture::new();
    let paths = prepare(&f);
    let deadline = end();
    let mut lease = NativeLease::acquire_paths(&paths, deadline).unwrap();
    // SQLite lock ranges do not stop a hostile writer to unrelated data bytes.
    // Do not claim global write prevention: the complete recheck must veto it.
    let mut foreign = OpenOptions::new().write(true).open(&paths[2]).unwrap();
    foreign.write_all(b"foreign").unwrap();
    foreign.sync_all().unwrap();
    drop(foreign);
    let raw = std::fs::read(&paths[2]).unwrap();
    assert!(raw.starts_with(b"foreign"));
    assert!(lease.check(deadline).is_err());
    assert!(lease.check(deadline).is_err());
    assert!(lease.release(deadline).is_err());
    assert_eq!(std::fs::read(&paths[2]).unwrap(), raw);
    for p in &paths[..2] {
        let c = competitor(p);
        c.execute_batch("BEGIN EXCLUSIVE; ROLLBACK").unwrap();
        c.close().unwrap();
    }
    println!("r5 raw byte writer bypasses SQLite coordination ranges but native generation/content recheck poisons lease; conflict preserved, no overwrite/compensation or false success; own kernel ranges unwind");
    f.retain();
}
#[test]
fn wl2_r5_coordination_namespace_uses_original_knownfolder_sid_not_environment() {
    let token = native::current_identity().unwrap();
    let expected = official_runtime_root(&token).unwrap();
    let child=std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact","infrastructure::windows_client_originals024::coordination::tests::isolated_namespace_child","--ignored","--nocapture"])
        .env("OCX_R5_NAMESPACE_EXPECTED",&expected).env("LOCALAPPDATA","Z:/foreign-local-app-data")
        .env("USERPROFILE","Z:/foreign-profile").env("HOME","Z:/foreign-home").output().unwrap();
    assert!(
        child.status.success(),
        "child raw={:?}: {}",
        child.status.code(),
        String::from_utf8_lossy(&child.stderr)
    );
    assert!(String::from_utf8_lossy(&child.stdout)
        .contains("namespace native TokenUser knownfolder independent"));
}
#[test]
#[ignore = "private no-I/O namespace child, executed only by its parent case"]
fn isolated_namespace_child() {
    let token = native::current_identity().unwrap();
    let root = official_runtime_root(&token).unwrap();
    assert_eq!(
        root,
        PathBuf::from(std::env::var_os("OCX_R5_NAMESPACE_EXPECTED").unwrap())
    );
    assert!(root.ends_with(PathBuf::from("OpenCodex/Runtime/v1").join(token.sid.to_uppercase())));
    assert_ne!(
        root,
        PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap())
    );
    println!("namespace native TokenUser knownfolder independent from caller HOME/USERPROFILE/LOCALAPPDATA; no directory creation");
}
