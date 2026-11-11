use super::*;
use std::os::unix::fs::{symlink, DirBuilderExt, MetadataExt, PermissionsExt};

struct Fixture {
    _temp: tempfile::TempDir,
    context: Context,
    record: Record,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let profile = root.join("profile");
        let client = profile.join("codex");
        let home = profile.join("home");
        let control = root.join("control");
        for path in [&profile, &client, &home, &control] {
            fs::DirBuilder::new().mode(0o700).create(path).unwrap();
        }
        let paths = [
            client.join(".opencodex-native-main.claim.sqlite"),
            root.join("native.sqlite"),
            root.join("catalog.sqlite"),
        ];
        for path in &paths {
            let mut file =
                crate::infrastructure::platform::create_new_private_file(path, 0o600).unwrap();
            file.write_all(b"").unwrap();
            drop(file);
            let c = rusqlite::Connection::open_with_flags(
                path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
            )
            .unwrap();
            c.execute_batch("CREATE TABLE marker (v INTEGER); INSERT INTO marker VALUES(1)")
                .unwrap();
            c.close().unwrap();
        }
        let locks = std::array::from_fn(|i| {
            let s = fs::metadata(&paths[i]).unwrap();
            LockIdentity {
                device: s.dev().to_string(),
                inode: s.ino().to_string(),
            }
        });
        let record = Record {
            schema: SCHEMA.into(),
            kind: "held".into(),
            operation: uuid::Uuid::new_v4().to_string(),
            pid: 42,
            context_sha256: "a".repeat(64),
            locks,
            paths,
            owner: "absent".into(),
        };
        let context = Context {
            runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(
            ),
            user_profile: profile,
            codex_home: client,
            opencodex_home: home,
            control_root: control,
        };
        Self {
            _temp: temp,
            context,
            record,
        }
    }
}

/// A separate real OS process probes SQLite, because another connection in the
/// same process cannot prove fcntl exclusion against an official writer.
pub(super) fn external_probe(paths: &[PathBuf; 3], blocked: bool) {
    let output=Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::official_client_coordination::native::tests::external_native_sqlite_probe","--nocapture"]).env("OCXD_NATIVE_SQLITE_PATHS",serde_json::to_string(paths).unwrap()).env("OCXD_NATIVE_SQLITE_BLOCKED",if blocked{"1"}else{"0"}).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn external_native_sqlite_probe() {
    let Some(paths) = std::env::var_os("OCXD_NATIVE_SQLITE_PATHS") else {
        return;
    };
    let paths: [PathBuf; 3] = serde_json::from_str(paths.to_str().unwrap()).unwrap();
    let blocked = std::env::var("OCXD_NATIVE_SQLITE_BLOCKED").unwrap() == "1";
    for path in &paths {
        assert!(path.is_absolute());
        let c = rusqlite::Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
        )
        .unwrap();
        c.busy_timeout(Duration::ZERO).unwrap();
        match c.execute_batch("BEGIN IMMEDIATE") {
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
            other => panic!("native SQLite probe unexpected: {other:?}"),
        }
    }
}

#[test]
fn callback_expiry_does_not_unlock_a_native_mutation_while_its_owner_still_runs() {
    let f = Fixture::new();
    let mut lease = acquire(
        &f.context,
        &f.record,
        Instant::now() + Duration::from_millis(100),
        &|_| Ok(()),
    )
    .unwrap();
    external_probe(&f.record.paths, true);
    thread::sleep(Duration::from_millis(120));
    assert!(lease.check().is_err());
    external_probe(&f.record.paths, true);
    lease.release().unwrap();
    external_probe(&f.record.paths, false);
}

#[test]
fn partial_acquisition_error_or_scope_drop_release_connections_before_raw_pins() {
    let f = Fixture::new();
    assert!(acquire(
        &f.context,
        &f.record,
        Instant::now() + Duration::from_secs(5),
        &|i| if i == 1 {
            Err(fail("fixture_failure"))
        } else {
            Ok(())
        }
    )
    .is_err());
    external_probe(&f.record.paths, false);
    let lease = acquire(
        &f.context,
        &f.record,
        Instant::now() + Duration::from_secs(5),
        &|_| Ok(()),
    )
    .unwrap();
    external_probe(&f.record.paths, true);
    drop(lease);
    external_probe(&f.record.paths, false);
}

#[test]
fn an_existing_competing_writer_refuses_without_retries_or_changed_budget() {
    let f = Fixture::new();
    let c = rusqlite::Connection::open_with_flags(
        &f.record.paths[1],
        rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
    )
    .unwrap();
    c.execute_batch("BEGIN IMMEDIATE").unwrap();
    let began = Instant::now();
    assert!(acquire(
        &f.context,
        &f.record,
        Instant::now() + Duration::from_secs(3),
        &|_| Ok(())
    )
    .is_err());
    assert!(began.elapsed() < Duration::from_secs(1));
    c.execute_batch("ROLLBACK").unwrap();
    c.close().unwrap();
    external_probe(&f.record.paths, false);
}

#[test]
fn substituted_lock_and_unsafe_mode_are_refused_without_repair_or_callback() {
    for unsafe_mode in [false, true] {
        let f = Fixture::new();
        let path = &f.record.paths[1];
        if unsafe_mode {
            fs::set_permissions(path, fs::Permissions::from_mode(0o644)).unwrap();
        } else {
            let saved = path.with_extension("saved");
            fs::rename(path, &saved).unwrap();
            symlink(saved, path).unwrap();
        }
        assert!(acquire(
            &f.context,
            &f.record,
            Instant::now() + Duration::from_secs(3),
            &|_| panic!("no lock callback for untrusted file")
        )
        .is_err());
        if unsafe_mode {
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o644
            );
        }
    }
}

#[test]
fn admitted_journal_fingerprint_cannot_be_rebased_after_helper_release() {
    let f = Fixture::new();
    fs::write(
        f.context.codex_home.join("opencodex-journal.json"),
        b"new owner facts",
    )
    .unwrap();
    assert!(acquire(
        &f.context,
        &f.record,
        Instant::now() + Duration::from_secs(3),
        &|_| panic!("changed owner cannot call work")
    )
    .is_err());
    assert_eq!(
        fs::read(f.context.codex_home.join("opencodex-journal.json")).unwrap(),
        b"new owner facts"
    );
    external_probe(&f.record.paths, false);
}

#[test]
fn ownership_file_change_while_held_is_rejected_without_dropping_the_three_locks() {
    let f = Fixture::new();
    let mut lease = acquire(
        &f.context,
        &f.record,
        Instant::now() + Duration::from_secs(5),
        &|_| Ok(()),
    )
    .unwrap();
    fs::write(
        f.context.codex_home.join("opencodex-journal.json"),
        b"unexpected owner change",
    )
    .unwrap();
    assert!(lease.check().is_err());
    external_probe(&f.record.paths, true);
    lease.release().unwrap();
    external_probe(&f.record.paths, false);
}

#[test]
fn sqlite_vfs_checks_its_actual_open_inode_instead_of_trusting_a_pathname() {
    let f = Fixture::new();
    let lease = acquire(
        &f.context,
        &f.record,
        Instant::now() + Duration::from_secs(5),
        &|_| Ok(()),
    )
    .unwrap();
    let path = &f.record.paths[1];
    let displaced = path.with_extension("original-inode");
    fs::rename(path, &displaced).unwrap();
    fs::write(path, b"unknown replacement database").unwrap();
    assert!(require_original_sqlite_file(&lease.connections[1]).is_err());
    assert!(lease.check().is_err());
    assert_eq!(fs::read(path).unwrap(), b"unknown replacement database");
    drop(lease);
    assert!(displaced.is_file());
}

#[test]
fn world_writable_owner_receipt_is_not_admitted_or_repaired_by_hashing_it() {
    let f = Fixture::new();
    let path = f.context.codex_home.join("opencodex-journal.json");
    fs::write(&path, b"synthetic current owner bytes").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o666)).unwrap();
    let root = PinnedDirectory::open(&f.context.codex_home).unwrap();
    assert!(ownership_receipt(&root).is_err());
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o666
    );
    assert_eq!(fs::read(path).unwrap(), b"synthetic current owner bytes");
}

#[cfg(target_arch = "aarch64")]
#[test]
#[ignore = "explicit pinned official299 bundle and new isolated profile; one native handover round only"]
fn official299_native_handover_holds_through_work_and_preserves_original_ownership() {
    let root =
        PathBuf::from(std::env::var_os("OCXD_NATIVE_COORDINATION_ROOT").expect("new private root"));
    assert!(
        root.is_absolute()
            && root.canonicalize().unwrap() == root
            && root.starts_with("/private/tmp")
    );
    assert!(root
        .parent()
        .unwrap()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("ocxd024-native-coordination-"));
    let profile = root.join("profile");
    assert_eq!(
        std::env::var_os("HOME"),
        Some(profile.clone().into_os_string())
    );
    let client = profile.join("codex");
    let home = profile.join("home");
    let data = profile.join("desktop");
    let controls = root.join("control");
    for path in [&client, &home, &data, &controls] {
        fs::DirBuilder::new().mode(0o700).create(path).unwrap();
    }
    fs::write(home.join("config.json"), b"{\"providers\":{}}\n").unwrap();
    fs::write(client.join("config.toml"), b"model = \"native-fixture\"\n").unwrap();
    fs::write(client.join("opencodex-catalog.json"), b"{\"models\":[]}\n").unwrap();
    let owner=serde_json::to_vec(&serde_json::json!({"version":1,"originalConfig":"synthetic native original","originalProfile":null,"opencodexHome":home})).unwrap();
    let bytes = fs::read(root.join("resources/runtime-bundle.manifest.json")).unwrap();
    let pin = super::super::super::hash::sha256_hex(&bytes);
    assert_eq!(
        pin,
        "e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3"
    );
    let reuse = std::env::var_os("OCXD_NATIVE_COORDINATION_REUSE").map(PathBuf::from);
    let program = reuse.clone().unwrap_or_else(|| root.join("program"));
    if reuse.is_none() {
        crate::modules::runtime::bundled::versioned::provision(
            &program,
            &data,
            &root.join("resources/runtime-bundle-0.2.24-darwin-arm64.tar.gz"),
            &bytes,
            &pin,
        )
        .unwrap();
    }
    let selected = program_selection::read(
        &program,
        crate::modules::data_root::layout::Platform::Macos,
        "darwin-arm64",
    )
    .unwrap()
    .unwrap();
    assert_eq!(selected.selection.version, "2.99.0");
    assert_eq!(
        selected.image.official_archive_sha256,
        "b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c"
    );
    program_image::verify(&selected).unwrap();
    // A synthetic journal without an initialized official coordinator is
    // intentionally refused as ambiguous prior state. Initialize this NEW
    // private namespace through the actual official constructor/commit before
    // placing a known current-HOME journal; do not fabricate a SQLite schema.
    let package = selected.paths.package_root.join(PACKAGE_SUBPATH);
    let init = r#"const [pkg,home]=process.argv.slice(-2);const {resolveEffectiveUserIdentity,resolveCodexCoordinatorDatabasePath}=await import(pkg+'/src/codex/user-identity.ts');const {openCodexCoordinatorTransaction}=await import(pkg+'/src/codex/transition-state.ts');const tx=openCodexCoordinatorTransaction(resolveCodexCoordinatorDatabasePath(resolveEffectiveUserIdentity(),home));try{tx.commit();}finally{tx.close();}"#;
    let initialized = Command::new(&selected.paths.bun)
        .args(["--eval", init, "--"])
        .arg(&package)
        .arg(&client)
        .env_clear()
        .env("HOME", &profile)
        .env("CODEX_HOME", &client)
        .env("OPENCODEX_HOME", &home)
        .env("PATH", "/usr/bin:/bin:/usr/sbin:/sbin")
        .current_dir(&root)
        .output()
        .unwrap();
    fs::write(
        root.join("official-initialize.stdout.log"),
        &initialized.stdout,
    )
    .unwrap();
    fs::write(
        root.join("official-initialize.stderr.log"),
        &initialized.stderr,
    )
    .unwrap();
    assert!(
        initialized.status.success(),
        "official private coordinator init failed; see retained stderr"
    );
    fs::write(client.join("opencodex-journal.json"), &owner).unwrap();
    let runtime = crate::modules::runtime::RuntimeHandle::initialize_versioned(&data, &program);
    let context = Context {
        runtime,
        user_profile: profile,
        codex_home: client.clone(),
        opencodex_home: home.clone(),
        control_root: controls,
    };
    let mut report = serde_json::json!({"accepted":false,"scope":"real official299 admission then same-process SQLite ownership; no service or client restore","result":"incomplete"});
    let evidence = root.join("native-summary.json");
    let write = |r: &serde_json::Value| {
        fs::write(&evidence, serde_json::to_vec_pretty(r).unwrap()).unwrap()
    };
    write(&report);
    let start = Instant::now();
    let result = with_native_coordination(
        &context,
        Instant::now() + Duration::from_secs(10),
        |lease| {
            lease.check()?;
            let hash = super::super::super::hash::sha256_hex(client.to_str().unwrap().as_bytes());
            let namespace =
                PathBuf::from(format!("/private/tmp/opencodex-runtime-v1-{}", unsafe {
                    libc::geteuid()
                }));
            let paths = [
                client.join(".opencodex-native-main.claim.sqlite"),
                namespace
                    .join("native-write-locks")
                    .join(format!("{hash}.sqlite")),
                namespace
                    .join("catalog-write-locks")
                    .join(format!("{hash}.sqlite")),
            ];
            external_probe(&paths, true);
            let originals =
                crate::infrastructure::macos_client_originals::Originals::begin(&data, &client)
                    .map_err(io)?;
            // Hold beyond the old helper's 2s window; raw-fd snapshot operations
            // must also not accidentally release process-associated SQLite locks.
            thread::sleep(Duration::from_millis(2100));
            lease.check()?;
            external_probe(&paths, true);
            originals
                .read(crate::infrastructure::macos_client_originals::Phase::Original)
                .map_err(io)?;
            external_probe(&paths, true);
            Ok(paths)
        },
    );
    report["elapsedMs"] = serde_json::json!(start.elapsed().as_millis());
    report["nativeResult"] = serde_json::json!(result.is_ok());
    if let Err(ref error) = result {
        report["error"] = serde_json::json!(error.to_string());
    }
    write(&report);
    let paths = result.unwrap();
    external_probe(&paths, false);
    assert_eq!(
        fs::read(client.join("opencodex-journal.json")).unwrap(),
        owner
    );
    assert_eq!(
        fs::read(home.join("config.json")).unwrap(),
        b"{\"providers\":{}}\n"
    );
    assert_eq!(
        fs::read(client.join("config.toml")).unwrap(),
        b"model = \"native-fixture\"\n"
    );
    assert!(!home.join("runtime-port.json").exists());
    assert!(!client.join("auth.json").exists());
    report["externalWriterExcludedBeforeAndAfterOld2sWindow"] = serde_json::json!(true);
    report["snapshotReadsDidNotReleaseLocks"] = serde_json::json!(true);
    report["nativeConnectionsReleasedBeforeRawFdPins"] = serde_json::json!(true);
    report["originalOwnershipBytesPreserved"] = serde_json::json!(true);
    report["result"] = serde_json::json!("passed");
    write(&report);
    println!("OFFICIAL299_NATIVE_HANDOVER {report}");
}
