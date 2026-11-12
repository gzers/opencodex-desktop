use super::*;
use serde_json::json;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};

fn message(kind: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({"schema":SCHEMA,"kind":kind,"operation":"fd510ba0-ec79-47cf-8456-ac2efaa87ce3","pid":42,"contextSha256":"a".repeat(64),"locks":[{"device":"1","inode":"2"},{"device":"1","inode":"3"},{"device":"1","inode":"4"},{"device":"1","inode":"5"}],"paths":["/private/claim.sqlite","/private/native.sqlite","/private/lifecycle.sqlite","/private/catalog.sqlite"],"owner":"absent"})).unwrap()
}
#[test]
fn release_notice_is_absent_until_complete_and_never_overwrites_another_receipt() {
    let temp = tempfile::tempdir_in("/private/tmp").unwrap();
    let root = temp.path().canonicalize().unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let pin = PinnedDirectory::open(&root).unwrap();
    let operation = uuid::Uuid::new_v4().to_string();
    publish_release_checked(&pin, &operation, &|| {
        assert!(!root.join("release").exists());
        std::thread::sleep(Duration::from_millis(20));
        assert!(!root.join("release").exists());
        Ok(())
    })
    .unwrap();
    assert_eq!(
        fs::read(root.join("release")).unwrap(),
        format!("{operation}\n").as_bytes()
    );
    let after = fs::metadata(root.join("release")).unwrap();
    assert_eq!(after.nlink(), 1);
    assert_eq!(after.permissions().mode() & 0o777, 0o600);
    let other = uuid::Uuid::new_v4().to_string();
    assert!(publish_release(&pin, &other).is_err());
    assert_eq!(
        fs::read(root.join("release")).unwrap(),
        format!("{operation}\n").as_bytes()
    );
}
#[test]
fn interrupted_release_staging_never_exposes_partial_final_nonce() {
    let temp = tempfile::tempdir_in("/private/tmp").unwrap();
    let root = temp.path().canonicalize().unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let pin = PinnedDirectory::open(&root).unwrap();
    let operation = uuid::Uuid::new_v4().to_string();
    assert!(
        publish_release_checked(&pin, &operation, &|| Err(fail("fixture_interrupted"))).is_err()
    );
    assert!(!root.join("release").exists());
    let staged = root.join(format!(".release-{operation}.tmp"));
    assert!(staged.exists());
    assert_eq!(fs::read(staged).unwrap(), b"");
    assert!(publish_release(&pin, &operation).is_err());
    assert!(!root.join("release").exists());
}

#[test]
fn bounded_fixed_coordination_protocol_rejects_unknown_duplicate_and_bad_lock_identity() {
    for kind in ["held", "released"] {
        assert!(record(&message(kind)).is_ok());
    }
    for value in [
        json!({"schema":SCHEMA,"kind":"invented"}),
        json!({"schema":"other","kind":"held"}),
    ] {
        assert!(record(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    let mut value: serde_json::Value = serde_json::from_slice(&message("held")).unwrap();
    value["arbitraryOutput"] = json!("not allowed");
    assert!(record(&serde_json::to_vec(&value).unwrap()).is_err());
    value.as_object_mut().unwrap().remove("arbitraryOutput");
    value["locks"][0]["inode"] = json!("-1");
    assert!(record(&serde_json::to_vec(&value).unwrap()).is_err());
    assert!(
        record(br#"{"schema":"ocxd-client-coordination/1","kind":"held","kind":"released"}"#)
            .is_err()
    );
    let mut old: serde_json::Value = serde_json::from_slice(&message("held")).unwrap();
    old["schema"] = json!("ocxd-client-coordination/1");
    assert!(record(&serde_json::to_vec(&old).unwrap()).is_err());
    old["schema"] = json!(SCHEMA);
    old["locks"].as_array_mut().unwrap().remove(2);
    old["paths"].as_array_mut().unwrap().remove(2);
    assert!(record(&serde_json::to_vec(&old).unwrap()).is_err());
}

#[test]
fn failure_protocol_retains_only_fixed_stages_not_upstream_payloads() {
    for stage in [
        "input",
        "imports",
        "context",
        "claim",
        "native",
        "lifecycle",
        "catalog",
        "held",
        "released",
    ] {
        let bytes=serde_json::to_vec(&json!({"schema":SCHEMA,"kind":"failed","stage":stage,"acquired":false,"released":false})).unwrap();
        assert!(
            matches!(record(&bytes),Err(AppError::RuntimeManaged{code,..}) if code.starts_with("client_coordination_"))
        );
    }
    let value = json!({"schema":SCHEMA,"kind":"failed","stage":"untrusted error text","acquired":false,"released":false});
    let error = record(&serde_json::to_vec(&value).unwrap())
        .err()
        .unwrap()
        .to_string();
    assert!(!error.contains("untrusted error text"));
}

#[test]
fn real_pipe_reader_bounds_records_and_rejects_third_or_unterminated_output() {
    fn read(bytes: &[u8]) -> Vec<AppResult<Record>> {
        let root = tempfile::tempdir().unwrap();
        let input = root.path().join("bounded-wire");
        fs::write(&input, bytes).unwrap();
        let mut child = Command::new("/bin/cat")
            .arg(&input)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let (rx, reader) = spawn_reader(child.stdout.take().unwrap());
        assert!(child.wait().unwrap().success());
        reader.join().unwrap();
        rx.into_iter().collect()
    }
    let mut valid = message("held");
    valid.push(b'\n');
    valid.extend(message("released"));
    valid.push(b'\n');
    let rows = read(&valid);
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(Result::is_ok));
    let mut third = valid.clone();
    third.extend(message("held"));
    third.push(b'\n');
    let rows = read(&third);
    assert_eq!(rows.len(), 3);
    assert!(rows[2].is_err());
    assert!(read(&message("held"))[0].is_err());
    assert!(read(&vec![b'x'; MAX_RECORD + 1])[0].is_err());
}

#[test]
fn expired_deadline_refuses_before_source_or_control_access() {
    let temp = tempfile::tempdir().unwrap();
    let context = Context {
        runtime: super::super::runtime_executable::FixedRuntimeExecutable::unresolved(),
        user_profile: temp.path().join("profile"),
        codex_home: temp.path().join("codex"),
        opencodex_home: temp.path().join("home"),
        control_root: temp.path().join("controls"),
    };
    assert!(
        with_coordination(&context, Instant::now(), |_| -> AppResult<()> {
            panic!("callback must not run")
        })
        .is_err()
    );
    assert_eq!(fs::read_dir(temp.path()).unwrap().count(), 0);
}

#[test]
fn native_admission_never_treats_import_time_as_a_mutation_lease_or_extends_the_outer_deadline() {
    let start = Instant::now();
    let expires = start + MAX_HOLD;
    let deadline = start + Duration::from_secs(5);
    let cold_held = start + Duration::from_millis(2100);
    assert!(
        LeaseUse::BorrowedWork.expired(cold_held, expires, deadline),
        "public helper work retains two-second conservative window"
    );
    assert!(
        !LeaseUse::NativeAdmission.expired(cold_held, expires, deadline),
        "copying admission metadata is not native work"
    );
    for usage in [LeaseUse::BorrowedWork, LeaseUse::NativeAdmission] {
        assert!(usage.expired(deadline, expires, deadline));
        assert!(usage.expired(deadline + Duration::from_millis(1), expires, deadline));
    }
    // Official hold timer/policy and protocol are byte-identical; normal
    // released+exit and actual native lock reacquisition remain mandatory.
    assert_eq!(MAX_HOLD, Duration::from_millis(2000));
    assert!(HELPER.contains("const expires=performance.now()+p.holdMs"));
}

#[cfg(target_arch = "aarch64")]
#[test]
fn synthetic_or_unknown_archive_identity_never_runs_official_coordination() {
    let fixture = crate::modules::runtime::program_selection::tests::Fixture::new();
    let data = fixture.root.join("data");
    let runtime =
        crate::modules::runtime::RuntimeHandle::initialize_versioned(&data, &fixture.root);
    let context = Context {
        runtime,
        user_profile: fixture.root.join("profile"),
        codex_home: fixture.root.join("codex"),
        opencodex_home: fixture.root.join("home"),
        control_root: fixture.root.join("controls"),
    };
    assert!(with_coordination(
        &context,
        Instant::now() + Duration::from_secs(2),
        |_| -> AppResult<()> { panic!("synthetic provider must not spawn") }
    )
    .is_err());
    assert!(!context.control_root.exists());
}

#[cfg(target_arch = "aarch64")]
#[test]
#[ignore = "explicit pinned official299 complete resource, private root and kernel boundary; no service"]
fn official299_real_coordinator_excludes_other_connections_and_releases_after_failure() {
    use crate::modules::runtime::{self, RuntimeHandle};
    let root = PathBuf::from(
        std::env::var_os("OCXD_COORDINATION_ROOT").expect("explicit owned native root"),
    );
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
        .starts_with("ocxd024-coordination-"));
    let profile = root.join("profile");
    assert_eq!(
        std::env::var_os("HOME"),
        Some(profile.clone().into_os_string())
    );
    let client = profile.join(".codex");
    let home = profile.join("opencodex-home");
    let data = profile.join("desktop");
    let controls = root.join("control");
    for path in [&client, &home, &data, &controls] {
        fs::DirBuilder::new().mode(0o700).create(path).unwrap();
    }
    fs::write(home.join("config.json"), b"{\"providers\":{}}\n").unwrap();
    fs::write(client.join("config.toml"), b"model = \"native-fixture\"\n").unwrap();
    fs::write(client.join("opencodex-catalog.json"), b"{\"models\":[]}\n").unwrap();
    let config = fs::read(home.join("config.json")).unwrap();
    let native = fs::read(client.join("config.toml")).unwrap();
    let catalog = fs::read(client.join("opencodex-catalog.json")).unwrap();
    let descriptor = fs::read(root.join("resources/runtime-bundle.manifest.json")).unwrap();
    let pin = super::super::hash::sha256_hex(&descriptor);
    assert_eq!(
        pin,
        "e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3"
    );
    let program = root.join("program 中文 space");
    runtime::bundled::versioned::provision(
        &program,
        &data,
        &root.join("resources/runtime-bundle-0.2.24-darwin-arm64.tar.gz"),
        &descriptor,
        &pin,
    )
    .unwrap();
    let runtime = RuntimeHandle::initialize_versioned(&data, &program);
    let context = Context {
        runtime,
        user_profile: profile,
        codex_home: client.clone(),
        opencodex_home: home.clone(),
        control_root: controls,
    };
    let home_hash = super::super::hash::sha256_hex(client.to_str().unwrap().as_bytes());
    let namespace = PathBuf::from(format!("/private/tmp/opencodex-runtime-v1-{}", unsafe {
        libc::geteuid()
    }));
    let locks = [
        client.join(".opencodex-native-main.claim.sqlite"),
        namespace
            .join("native-write-locks")
            .join(format!("{home_hash}.sqlite")),
        namespace
            .join("catalog-write-locks")
            .join(format!("{home_hash}.sqlite")),
    ];
    let mut report = json!({"accepted":false,"scope":"real official299 coordination only; no service or restoration","officialVersion":"2.99.0","businessRequests":false});
    let evidence = root.join("native-summary.json");
    let write = |r: &serde_json::Value| {
        fs::write(&evidence, serde_json::to_vec_pretty(r).unwrap()).unwrap();
    };
    write(&report);
    let start = Instant::now();
    let originals = with_coordination(&context, Instant::now() + Duration::from_secs(5), |lease| {
        lease.check()?;
        for path in &locks {
            let conn = rusqlite::Connection::open_with_flags(
                path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
            )
            .map_err(io)?;
            conn.busy_timeout(Duration::ZERO).map_err(io)?;
            let result = conn.execute_batch("BEGIN IMMEDIATE");
            if !matches!(result,Err(rusqlite::Error::SqliteFailure(ref e,_)) if matches!(e.code,rusqlite::ErrorCode::DatabaseBusy|rusqlite::ErrorCode::DatabaseLocked))
            {
                return Err(fail("fixture_lock_did_not_exclude"));
            }
        }
        let originals =
            super::super::macos_client_originals::Originals::begin(&data, &client).map_err(io)?;
        lease.check()?;
        Ok(originals)
    });
    report["firstElapsedMs"] = json!(start.elapsed().as_millis());
    report["originalsCapturedUnderThreeOfficialLocks"] = json!(originals.is_ok());
    write(&report);
    let originals = originals.unwrap();
    let ids: Vec<_> = locks
        .iter()
        .map(|path| {
            let m = fs::symlink_metadata(path).unwrap();
            (m.dev(), m.ino())
        })
        .collect();
    let reacquire = || {
        for path in &locks {
            let c = rusqlite::Connection::open_with_flags(
                path,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
            )
            .unwrap();
            c.busy_timeout(Duration::ZERO).unwrap();
            c.execute_batch("BEGIN IMMEDIATE; ROLLBACK").unwrap();
        }
    };
    reacquire();
    report["realSqliteOtherConnectionExclusionAndReacquisition"] = json!(true);
    write(&report);
    let callback_failure = with_coordination(
        &context,
        Instant::now() + Duration::from_secs(5),
        |lease| -> AppResult<()> {
            lease.check()?;
            Err(fail("fixture_callback_failure"))
        },
    );
    assert!(
        matches!(callback_failure,Err(AppError::RuntimeManaged{ref code,..}) if code=="fixture_callback_failure")
    );
    reacquire();
    report["callbackFailurePreservedAndLocksReleased"] = json!(true);
    write(&report);
    let expired = with_coordination(&context, Instant::now() + Duration::from_secs(5), |_| {
        thread::sleep(Duration::from_millis(2100));
        Ok(())
    });
    assert!(expired.is_err());
    reacquire();
    report["expiredCallbackCannotReturnSuccess"] = json!(true);
    write(&report);
    // Only this test's freshly created lock is changed to exercise non-repair.
    fs::set_permissions(&locks[1], fs::Permissions::from_mode(0o644)).unwrap();
    assert!(with_coordination(
        &context,
        Instant::now() + Duration::from_secs(5),
        |_| -> AppResult<()> { panic!("unsafe old lock cannot call work") }
    )
    .is_err());
    assert_eq!(
        fs::metadata(&locks[1]).unwrap().permissions().mode() & 0o777,
        0o644
    );
    fs::set_permissions(&locks[1], fs::Permissions::from_mode(0o600)).unwrap();
    report["existingUnsafeLockRefusedWithoutRepair"] = json!(true);
    write(&report);
    let foreign = root.join("other-opencodex-home");
    fs::create_dir(&foreign).unwrap();
    let journal=serde_json::to_vec(&json!({"version":1,"originalConfig":"synthetic-original","originalProfile":null,"opencodexHome":foreign})).unwrap();
    fs::write(client.join("opencodex-journal.json"), &journal).unwrap();
    assert!(with_coordination(
        &context,
        Instant::now() + Duration::from_secs(5),
        |_| -> AppResult<()> { panic!("foreign owner cannot call work") }
    )
    .is_err());
    assert_eq!(
        fs::read(client.join("opencodex-journal.json")).unwrap(),
        journal
    );
    reacquire();
    report["foreignHomeOwnershipRefused"] = json!(true);
    write(&report);
    assert_eq!(fs::read(home.join("config.json")).unwrap(), config);
    assert_eq!(fs::read(client.join("config.toml")).unwrap(), native);
    assert_eq!(
        fs::read(client.join("opencodex-catalog.json")).unwrap(),
        catalog
    );
    assert!(!client.join("auth.json").exists());
    assert!(!home.join("runtime-port.json").exists());
    assert!(originals
        .read(super::super::macos_client_originals::Phase::Original)
        .is_ok());
    assert_eq!(
        locks
            .iter()
            .map(|path| {
                let m = fs::symlink_metadata(path).unwrap();
                (m.dev(), m.ino())
            })
            .collect::<Vec<_>>(),
        ids
    );
    report["configAndCatalogPreserved"] = json!(true);
    report["stableLockIdentitiesPreserved"] = json!(true);
    report["result"] = json!("passed");
    write(&report);
    println!("OFFICIAL299_COORDINATION {report}");
}
