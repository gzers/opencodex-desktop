//! Actual cross-process coordinator tests on synthetic SQLite, not service
//! exit, official config-holder relocation or whole-root mutation permission.
use super::*;
use crate::state::ProcessContext;
use std::io::Read;
pub(super) fn official_round(
    context: &Context,
    selected: &program_selection::SelectedProgram,
    root: &Path,
    owner: &[u8],
) {
    let hash = super::super::super::super::hash::sha256_hex(
        context.codex_home.to_str().unwrap().as_bytes(),
    );
    let namespace = PathBuf::from(format!("/private/tmp/opencodex-runtime-v1-{}", unsafe {
        libc::geteuid()
    }));
    let paths = [
        context
            .codex_home
            .join(".opencodex-native-main.claim.sqlite"),
        namespace
            .join("native-write-locks")
            .join(format!("{hash}.sqlite")),
        namespace.join("client-desktop-lifecycle.sqlite"),
        namespace
            .join("catalog-write-locks")
            .join(format!("{hash}.sqlite")),
    ];
    let archive = context.user_profile.join("retired-codex");
    let started = Instant::now();
    let result =
        with_native_relocation_coordination(context, started + Duration::from_secs(10), |native| {
            native.check()?;
            external_probe(&paths, true);
            official_writer_probe(context, selected, root, "blocked");
            native.begin_claim_handoff()?;
            external_probe(&paths, true);
            fs::rename(&context.codex_home, &archive).map_err(io)?;
            assert!(native.check().is_err(), "stale old anchor must refuse");
            native.reanchor_claim(&archive)?;
            let mut moved = paths.clone();
            moved[0] = archive.join(".opencodex-native-main.claim.sqlite");
            external_probe(&moved, true);
            official_writer_probe(context, selected, root, "blocked");
            assert!(
                !context.codex_home.exists(),
                "blocked official writers must not recreate root"
            );
            assert_eq!(
                fs::read(archive.join("opencodex-journal.json")).unwrap(),
                owner
            );
            native.check()?;
            Ok(())
        });
    let elapsed = started.elapsed();
    let report = serde_json::json!({"accepted":false,"scope":"actual unchanged official299 admission, source wrapper and K/L writer exclusion; synthetic root movement, no service/Stop or configuration writer relocation permission","result":if result.is_ok(){"passed"}else{"failed"},"elapsedMs":elapsed.as_millis(),"budgetMs":10000,"originalRootAbsent":!context.codex_home.exists(),"archiveExists":archive.is_dir(),"allNativeCoordinatorsExcludedAcrossMove":result.is_ok()});
    fs::write(
        root.join("native-summary.json"),
        serde_json::to_vec_pretty(&report).unwrap(),
    )
    .unwrap();
    result.unwrap();
    assert!(elapsed < Duration::from_secs(10));
    let mut moved = paths.clone();
    moved[0] = archive.join(".opencodex-native-main.claim.sqlite");
    external_probe(&moved, false);
    assert_eq!(
        fs::read(archive.join("opencodex-journal.json")).unwrap(),
        owner
    );
    fs::rename(&archive, &context.codex_home).unwrap();
    official_writer_probe(context, selected, root, "released");
    external_probe(&paths, false);
    println!("OFFICIAL_RELOCATION_EVIDENCE {report}");
}
fn lease(f: &Fixture) -> NativeLease {
    acquire_mode(
        &f.context,
        &f.record,
        Instant::now() + Duration::from_secs(10),
        &|_| Ok(()),
        true,
    )
    .unwrap()
}
fn other_paths(f: &Fixture) -> [PathBuf; 4] {
    [
        f.record.paths[1].clone(),
        f.record.paths[2].clone(),
        f.record.paths[3].clone(),
        f.record.paths[1].clone(),
    ]
}
fn claim_paths(path: &Path) -> [PathBuf; 4] {
    std::array::from_fn(|_| path.to_path_buf())
}
#[test]
fn relocation_handoff_keeps_n_l_k_held_and_refuses_permission_in_claim_gap() {
    let f = Fixture::new();
    let original = fs::read(&f.record.paths[0]).unwrap();
    let mut lease = lease(&f);
    external_probe(&f.record.paths, true);
    lease
        .begin_claim_handoff_checked(&|| {
            assert_eq!(lease.claim_mode.get(), ClaimMode::HandoffPending);
            assert!(lease.check().is_err());
            external_probe(&other_paths(&f), true);
            external_probe(&claim_paths(&f.record.paths[0]), false);
            Ok(())
        })
        .unwrap();
    assert!(lease.claim.borrow().is_none());
    external_probe(&f.record.paths, true);
    let archive = f.context.codex_home.with_file_name("retired");
    fs::rename(&f.context.codex_home, &archive).unwrap();
    assert!(lease.check().is_err());
    lease.reanchor_claim(&archive).unwrap();
    let mut paths = f.record.paths.clone();
    paths[0] = archive.join(".opencodex-native-main.claim.sqlite");
    external_probe(&paths, true);
    drop(fs::File::open(&paths[0]).unwrap());
    external_probe(&paths, true);
    lease
        .require_context(
            &ProcessContext {
                runtime: f.context.runtime.clone(),
                working_directory: f.context.user_profile.clone(),
                opencodex_home: f.context.opencodex_home.clone(),
            },
            &f.context.codex_home,
        )
        .unwrap();
    lease.release().unwrap();
    external_probe(&paths, false);
    assert_eq!(fs::read(&paths[0]).unwrap(), original);
    println!(
        "NATIVE_HANDOFF_EVIDENCE {}",
        serde_json::json!({"accepted":false,"gapHasNoPermission":true,"nLKContinuouslyExcludedAtGap":true,"allFourExcludedAfterMove":true,"claimOriginalBytesUnchanged":true,"normalReleaseProbed":true,"noServiceOrWriterAuthority":true})
    );
}
#[test]
fn relocation_is_opt_in_and_partial_acquisition_does_not_release_live_other_locks_early() {
    let f = Fixture::new();
    let ordinary = acquire(
        &f.context,
        &f.record,
        Instant::now() + Duration::from_secs(5),
        &|_| Ok(()),
    )
    .unwrap();
    assert!(ordinary.begin_claim_handoff().is_err());
    ordinary.check().unwrap();
    external_probe(&f.record.paths, true);
    drop(ordinary);
    external_probe(&f.record.paths, false);
    assert!(acquire_mode(
        &f.context,
        &f.record,
        Instant::now() + Duration::from_secs(5),
        &|i| if i == 2 {
            Err(fail("synthetic_partial"))
        } else {
            Ok(())
        },
        true
    )
    .is_err());
    external_probe(&f.record.paths, false);
}
#[test]
fn relocation_failed_handoff_stays_pending_with_n_l_k_excluded_until_cleanup() {
    let f = Fixture::new();
    let mut lease = lease(&f);
    assert!(lease
        .begin_claim_handoff_checked(&|| Err(fail("synthetic_handoff_gap")))
        .is_err());
    assert!(lease.check().is_err());
    assert!(lease.begin_claim_handoff().is_err());
    external_probe(&other_paths(&f), true);
    external_probe(&claim_paths(&f.record.paths[0]), false);
    assert!(lease.release().is_err());
    external_probe(&f.record.paths, false);
}
const HOLDER: &str="infrastructure::official_client_coordination::native::tests::relocation::handoff_foreign_claim_holder";
#[test]
fn handoff_foreign_claim_holder() {
    let Some(path) = std::env::var_os("OCXD_HANDOFF_FOREIGN_CLAIM") else {
        return;
    };
    let c = rusqlite::Connection::open(path).unwrap();
    c.busy_timeout(Duration::ZERO).unwrap();
    c.execute_batch("BEGIN EXCLUSIVE").unwrap();
    fs::write(
        std::env::var_os("OCXD_HANDOFF_FOREIGN_MARKER").unwrap(),
        b"held",
    )
    .unwrap();
    let mut release = [0u8; 1];
    std::io::stdin().read_exact(&mut release).unwrap();
    assert_eq!(release, [1]);
    c.execute_batch("ROLLBACK").unwrap();
    c.close().unwrap();
}
#[test]
fn relocation_foreign_claim_contender_does_not_get_reclassified_as_our_handoff() {
    let f = Fixture::new();
    let mut lease = lease(&f);
    let holder = RefCell::new(None);
    let marker = f.context.control_root.join("foreign-claim-held");
    assert!(lease
        .begin_claim_handoff_checked(&|| {
            let mut child = Command::new(std::env::current_exe().unwrap())
                .args(["--exact", HOLDER, "--nocapture", "--test-threads=1"])
                .env("OCXD_HANDOFF_FOREIGN_CLAIM", &f.record.paths[0])
                .env("OCXD_HANDOFF_FOREIGN_MARKER", &marker)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            let until = Instant::now() + Duration::from_secs(5);
            while !marker.exists() {
                assert!(child.try_wait().unwrap().is_none());
                assert!(Instant::now() < until);
                thread::sleep(Duration::from_millis(2));
            }
            *holder.borrow_mut() = Some(child);
            Ok(())
        })
        .is_err());
    assert_eq!(lease.claim_mode.get(), ClaimMode::HandoffPending);
    assert!(lease.check().is_err());
    external_probe(&f.record.paths, true);
    let mut child = holder.borrow_mut().take().unwrap();
    child.stdin.take().unwrap().write_all(&[1]).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(lease.release().is_err());
    external_probe(&f.record.paths, false);
}
#[test]
fn relocation_reappeared_user_root_or_unknown_archive_is_not_adopted() {
    let f = Fixture::new();
    let mut lease = lease(&f);
    lease.begin_claim_handoff().unwrap();
    let archive = f.context.codex_home.with_file_name("retired");
    fs::rename(&f.context.codex_home, &archive).unwrap();
    let other = Fixture::new();
    assert!(lease.reanchor_claim(&other.context.codex_home).is_err());
    lease.reanchor_claim(&archive).unwrap();
    fs::DirBuilder::new()
        .mode(0o700)
        .create(&f.context.codex_home)
        .unwrap();
    fs::write(f.context.codex_home.join("outside"), b"user").unwrap();
    assert!(lease.check().is_err());
    assert!(lease.reanchor_claim(&archive).is_err());
    external_probe(&other_paths(&f), true);
    assert_eq!(
        fs::read(f.context.codex_home.join("outside")).unwrap(),
        b"user"
    );
    lease.release().unwrap();
    external_probe(&other_paths(&f), false);
}
#[test]
fn relocation_does_not_skip_other_sqlite_vfs_or_owner_checks() {
    for case in ["vfs", "owner"] {
        let f = Fixture::new();
        let lease = lease(&f);
        lease.begin_claim_handoff().unwrap();
        if case == "vfs" {
            let path = &f.record.paths[1];
            fs::rename(path, path.with_extension("original")).unwrap();
            fs::write(path, b"outside").unwrap();
        } else {
            fs::write(
                f.context.codex_home.join("opencodex-journal.json"),
                b"outside",
            )
            .unwrap();
        }
        assert!(lease.check().is_err());
        external_probe(&claim_paths(&f.record.paths[0]), true);
        drop(lease);
        external_probe(&claim_paths(&f.record.paths[0]), false);
    }
}
#[test]
fn relocation_expired_lease_keeps_all_coordinators_until_owner_unwinds() {
    let f = Fixture::new();
    let mut lease = acquire_mode(
        &f.context,
        &f.record,
        Instant::now() + Duration::from_millis(100),
        &|_| Ok(()),
        true,
    )
    .unwrap();
    lease.begin_claim_handoff().unwrap();
    thread::sleep(Duration::from_millis(120));
    assert!(lease.check().is_err());
    assert!(lease.reanchor_claim(&f.context.codex_home).is_err());
    external_probe(&f.record.paths, true);
    assert!(lease.release().is_err());
    external_probe(&f.record.paths, false);
}
