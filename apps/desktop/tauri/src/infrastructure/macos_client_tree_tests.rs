use super::*;
use std::{
    cell::Cell,
    fs,
    os::unix::fs::{symlink, PermissionsExt},
    time::Instant,
};
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    client: PathBuf,
}
#[test]
#[ignore = "explicit retained own native fixture only; no original/session writes or service actions"]
fn inspect_retained_native_tree_without_rebasing_originals() {
    let path = PathBuf::from(std::env::var_os("OCXD_TREE_RETAINED").expect("explicit owned root"));
    assert!(
        path.is_absolute()
            && path.canonicalize().unwrap() == path
            && path.starts_with("/private/tmp")
            && path.ancestors().any(|p| p.file_name().is_some_and(|n| n
                .to_string_lossy()
                .starts_with("ocxd024-versioned-lifecycle-")))
    );
    let snapshot = TreeSnapshot::read(&PinnedDirectory::open(&path).unwrap());
    match snapshot {
        Ok(value) => println!(
            "RETAINED_TREE_READ entries={} valid={}",
            value.entry_count(),
            value.valid()
        ),
        Err(e) => println!("RETAINED_TREE_REFUSAL kind={:?} detail={}", e.kind(), e),
    }
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::Builder::new()
            .prefix("ocxd024-tree-fixture-")
            .tempdir()
            .unwrap();
        let root = temp.path().canonicalize().unwrap();
        let client = root.join("client");
        fs::create_dir(&client).unwrap();
        fs::set_permissions(&client, fs::Permissions::from_mode(0o700)).unwrap();
        Self {
            _temp: temp,
            root,
            client,
        }
    }
    fn pin(&self) -> PinnedDirectory {
        PinnedDirectory::open(&self.client).unwrap()
    }
}
#[test]
fn recovery_remainder_exempts_only_separately_reviewed_exact_plan_objects() {
    let f = Fixture::new();
    let plan = uuid::Uuid::new_v4().to_string();
    fs::write(f.client.join("config.toml"), b"native='original'").unwrap();
    fs::set_permissions(
        f.client.join("config.toml"),
        fs::Permissions::from_mode(0o644),
    )
    .unwrap();
    let original = TreeSnapshot::read(&f.pin()).unwrap();
    fs::write(f.client.join(NAMES[0]), b"plan target").unwrap();
    let stage = f.client.join(format!(".ocxd-restore-{plan}-0"));
    fs::write(&stage, b"plan stage").unwrap();
    assert!(!original.preplan_remainder_matches(&TreeSnapshot::read(&f.pin()).unwrap(), false));
    assert!(original.recovery_remainder_matches(
        &TreeSnapshot::read(&f.pin()).unwrap(),
        &plan,
        false
    ));
    assert!(!original.recovery_remainder_matches(
        &TreeSnapshot::read(&f.pin()).unwrap(),
        "../bad",
        false
    ));
    fs::write(f.client.join(".ocxd-restore-foreign-0"), b"unknown stage").unwrap();
    assert!(!original.recovery_remainder_matches(
        &TreeSnapshot::read(&f.pin()).unwrap(),
        &plan,
        false
    ));
    fs::remove_file(f.client.join(".ocxd-restore-foreign-0")).unwrap();
    fs::remove_file(&stage).unwrap();
    assert!(original.preplan_remainder_matches(&TreeSnapshot::read(&f.pin()).unwrap(), false));
    fs::write(f.client.join("config.toml"), b"outside config edit").unwrap();
    assert!(!original.recovery_remainder_matches(
        &TreeSnapshot::read(&f.pin()).unwrap(),
        &plan,
        false
    ));
    fs::write(f.client.join("config.toml"), b"native='original'").unwrap();
    assert!(original.recovery_remainder_matches(
        &TreeSnapshot::read(&f.pin()).unwrap(),
        &plan,
        false
    ));
    fs::set_permissions(
        f.client.join("config.toml"),
        fs::Permissions::from_mode(0o600),
    )
    .unwrap();
    assert!(!original.recovery_remainder_matches(
        &TreeSnapshot::read(&f.pin()).unwrap(),
        &plan,
        false
    ));
}
#[test]
fn complete_inventory_keeps_literal_paths_and_empty_directories_and_hashes_without_serialized_contents(
) {
    let f = Fixture::new();
    fs::create_dir(f.client.join("agents")).unwrap();
    fs::create_dir(f.client.join("empty")).unwrap();
    fs::write(
        f.client.join("agents/AGENTS.md"),
        b"private agent instructions",
    )
    .unwrap();
    fs::write(f.client.join("auth.json"), b"synthetic-secret-token").unwrap();
    fs::write(f.client.join("literal\\name"), b"literal").unwrap();
    fs::create_dir(f.client.join("literal")).unwrap();
    fs::write(f.client.join("literal/name"), b"nested").unwrap();
    let pin = f.pin();
    let before = TreeSnapshot::read(&pin).unwrap();
    assert_eq!(before.entry_count(), 7);
    assert_eq!(before, TreeSnapshot::read(&pin).unwrap());
    let raw = encode(&before).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    for path in [
        "agents/AGENTS.md",
        "empty",
        "auth.json",
        "literal\\name",
        "literal/name",
    ] {
        assert!(value["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["path"] == path));
    }
    let text = String::from_utf8(raw).unwrap();
    assert!(
        !text.contains("synthetic-secret-token") && !text.contains("private agent instructions")
    );
    assert!(!format!("{before:?}").contains("auth.json"));
}
#[test]
fn actual_other_process_replaces_an_open_parent_with_an_outside_symlink_and_capture_refuses() {
    const CHILD: &str = "OCXD024_TREE_PARENT_SWAP";
    const TEST:&str="infrastructure::macos_client_originals::tree::tests::actual_other_process_replaces_an_open_parent_with_an_outside_symlink_and_capture_refuses";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        assert!(
            root.starts_with("/private/tmp")
                && root
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("ocxd024-tree-fixture-")
        );
        fs::rename(root.join("client/nested"), root.join("retained-nested")).unwrap();
        symlink(root.join("outside"), root.join("client/nested")).unwrap();
        return;
    }
    let f = Fixture::new();
    fs::create_dir(f.client.join("nested")).unwrap();
    fs::create_dir(f.root.join("outside")).unwrap();
    fs::write(f.client.join("nested/target"), b"original").unwrap();
    fs::write(
        f.root.join("outside/target"),
        b"outside must not be traversed",
    )
    .unwrap();
    let once = Cell::new(false);
    let error = TreeSnapshot::read_checked(&f.pin(), &|path| {
        if path == "nested/target" && !once.replace(true) {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", TEST, "--nocapture"])
                .env(CHILD, &f.root)
                .output()?;
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    })
    .unwrap_err();
    assert!(once.get());
    assert_eq!(
        error.to_string(),
        "client tree directory changed during capture"
    );
    assert_eq!(
        fs::read(f.root.join("retained-nested/target")).unwrap(),
        b"original"
    );
    assert_eq!(
        fs::read(f.root.join("outside/target")).unwrap(),
        b"outside must not be traversed"
    );
    println!("FD_RELATIVE_TREE_PARENT_SWAP refused=true originalAndOutsideUnchanged=true");
}
#[test]
fn late_file_modification_and_unreadable_special_or_linked_objects_never_become_absent() {
    let f = Fixture::new();
    fs::write(f.client.join("data"), b"original").unwrap();
    let seen = Cell::new(0);
    let error = TreeSnapshot::read_checked(&f.pin(), &|path| {
        if path == "data" {
            seen.set(seen.get() + 1);
            if seen.get() == 2 {
                fs::write(f.client.join("data"), b"changed!")?;
            }
        }
        Ok(())
    })
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "client tree object changed during observation"
    );
    assert_eq!(fs::read(f.client.join("data")).unwrap(), b"changed!");
    for kind in ["symlink", "hardlink", "fifo"] {
        let f = Fixture::new();
        fs::write(f.root.join("outside"), b"outside").unwrap();
        let path = f.client.join("unknown");
        match kind {
            "symlink" => symlink(f.root.join("outside"), &path).unwrap(),
            "hardlink" => fs::hard_link(f.root.join("outside"), &path).unwrap(),
            _ => {
                let c = CString::new(path.as_os_str().as_bytes()).unwrap();
                assert_eq!(unsafe { libc::mkfifo(c.as_ptr(), 0o600) }, 0);
            }
        }
        let started = Instant::now();
        assert!(TreeSnapshot::read(&f.pin()).is_err(), "{kind}");
        assert!(started.elapsed() < std::time::Duration::from_secs(1));
        assert_eq!(fs::read(f.root.join("outside")).unwrap(), b"outside");
    }
}
#[test]
fn inventory_does_not_release_an_active_same_process_sqlite_claim_lock() {
    const CHILD: &str = "OCXD024_TREE_CLAIM_PROBE";
    const TEST:&str="infrastructure::macos_client_originals::tree::tests::inventory_does_not_release_an_active_same_process_sqlite_claim_lock";
    if let Some(path) = std::env::var_os(CHILD) {
        let p = PathBuf::from(path);
        assert!(p.starts_with("/private/tmp") && p.file_name() == Some(OsStr::new(CLAIM)));
        let c =
            rusqlite::Connection::open_with_flags(p, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
                .unwrap();
        c.busy_timeout(std::time::Duration::ZERO).unwrap();
        assert!(
            matches!(c.execute_batch("BEGIN IMMEDIATE"),Err(rusqlite::Error::SqliteFailure(e,_)) if e.code==rusqlite::ErrorCode::DatabaseBusy)
        );
        println!("TREE_CLAIM_LOCK_RETAINED");
        return;
    }
    let f = Fixture::new();
    let path = f.client.join(CLAIM);
    let file = crate::infrastructure::platform::create_new_private_file(&path, 0o600).unwrap();
    drop(file);
    let c =
        rusqlite::Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
            .unwrap();
    c.execute_batch("CREATE TABLE fixture(v INTEGER);BEGIN EXCLUSIVE")
        .unwrap();
    let capture = TreeSnapshot::read(&f.pin()).unwrap();
    assert!(matches!(&capture.entries[0], Entry::Coordination { .. }));
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture"])
        .env(CHILD, &path)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{} {}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(String::from_utf8_lossy(&result.stdout).contains("TREE_CLAIM_LOCK_RETAINED"));
    println!("{}", String::from_utf8_lossy(&result.stdout));
    c.execute_batch("ROLLBACK").unwrap();
    c.close().unwrap();
    let check = rusqlite::Connection::open(&path).unwrap();
    check.execute_batch("BEGIN EXCLUSIVE;ROLLBACK").unwrap();
}

#[test]
fn empty_official_claim_initial_transaction_is_observed_without_reading_its_live_journal() {
    let f = Fixture::new();
    fs::write(f.client.join("config.toml"), b"native original config").unwrap();
    let vault_path = f.root.join("vault");
    fs::create_dir(&vault_path).unwrap();
    fs::set_permissions(&vault_path, fs::Permissions::from_mode(0o700)).unwrap();
    let path = f.client.join(CLAIM);
    drop(crate::infrastructure::platform::create_new_private_file(&path, 0o600).unwrap());
    let c =
        rusqlite::Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)
            .unwrap();
    c.execute_batch("BEGIN EXCLUSIVE").unwrap();
    let directory = names(f.pin().file()).unwrap();
    println!("EMPTY_CLAIM_ACTIVE_FILES {:?}", directory);
    let capture = TreeSnapshot::read(&f.pin());
    assert!(
        capture.is_ok(),
        "empty claim inventory: {:?}",
        capture.err()
    );
    let capture = capture.unwrap();
    let vault = PinnedDirectory::open(&vault_path).unwrap();
    capture.archive(&f.pin(), &vault).unwrap();
    capture.verify_archive(&vault).unwrap();
    assert_eq!(
        capture
            .archived_file(&vault, "config.toml")
            .unwrap()
            .as_slice(),
        b"native original config"
    );
    let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::tree::tests::inventory_does_not_release_an_active_same_process_sqlite_claim_lock","--nocapture"]).env("OCXD024_TREE_CLAIM_PROBE",&path).output().unwrap();
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("TREE_CLAIM_LOCK_RETAINED"));
    println!("EMPTY_CLAIM_AND_JOURNAL_LOCK_RETAINED");
    c.execute_batch("ROLLBACK").unwrap();
    c.close().unwrap();
}
#[test]
fn inventory_depth_file_size_and_decoded_structure_are_bounded() {
    let f = Fixture::new();
    let file = File::create(f.client.join("oversize")).unwrap();
    file.set_len(MAX_FILE + 1).unwrap();
    assert!(TreeSnapshot::read(&f.pin()).is_err());
    let f = Fixture::new();
    let mut path = f.client.clone();
    for _ in 0..=MAX_DEPTH {
        path = path.join("nested");
        fs::create_dir(&path).unwrap();
    }
    assert!(TreeSnapshot::read(&f.pin()).is_err());
    let f = Fixture::new();
    fs::write(f.client.join("data"), b"original").unwrap();
    let original = TreeSnapshot::read(&f.pin()).unwrap();
    for mutation in ["hash", "path", "total"] {
        let mut value = serde_json::to_value(&original).unwrap();
        match mutation {
            "hash" => value["entries"][0]["sha256"] = "invalid".into(),
            "path" => value["entries"][0]["path"] = "../escape".into(),
            _ => value["totalBytes"] = 0.into(),
        };
        let decoded: TreeSnapshot = serde_json::from_value(value).unwrap();
        assert!(!decoded.valid(), "{mutation}");
    }
}

#[test]
fn complete_tree_and_three_artifact_snapshots_must_describe_the_same_bytes_and_identities() {
    let f = Fixture::new();
    fs::write(f.client.join(NAMES[0]), b"catalog").unwrap();
    let root = f.pin();
    let tree = TreeSnapshot::read(&root).unwrap();
    let capture = Capture::read(&root).unwrap();
    assert!(tree.matches_artifacts(&capture));
    let mut altered = tree.clone();
    if let Entry::File { sha256, .. } = &mut altered.entries[0] {
        *sha256 = "a".repeat(64);
    } else {
        panic!("file expected");
    }
    assert!(
        altered.valid(),
        "valid-shaped hash alone does not bind the artifacts"
    );
    assert!(!altered.matches_artifacts(&capture));
    fs::write(f.client.join(NAMES[0]), b"changed").unwrap();
    assert!(!tree.matches_artifacts(&Capture::read(&root).unwrap()));
}

#[test]
fn unreadable_tree_file_and_coordination_sidecar_are_preserved_and_refused() {
    let f = Fixture::new();
    let path = f.client.join("unreadable");
    fs::write(&path, b"original").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    assert!(TreeSnapshot::read(&f.pin()).is_err());
    assert_eq!(fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"original");
    fs::write(
        f.client.join(format!("{CLAIM}-journal")),
        b"original coordination journal",
    )
    .unwrap();
    assert!(TreeSnapshot::read(&f.pin()).is_err());
    assert_eq!(
        fs::read(f.client.join(format!("{CLAIM}-journal"))).unwrap(),
        b"original coordination journal"
    );
}
