use super::*;
use crate::infrastructure::windows_client_originals024::tree::tests::Fixture;
use native::{create_private_directory, PrivateRecord};
use rusqlite::Connection;
use std::time::Duration;
struct Scene {
    fixture: Option<Fixture>,
    paths: [PathBuf; 4],
}
impl Scene {
    fn new() -> Self {
        let fixture = Fixture::new();
        let client = create_private_directory(&fixture.path().join(".codex")).unwrap();
        let coordination = create_private_directory(&fixture.path().join("coordination")).unwrap();
        let paths = std::array::from_fn(|i| {
            if i == 0 {
                client.path().join(".opencodex-native-main.claim.sqlite")
            } else {
                coordination.path().join(format!("{i}.sqlite"))
            }
        });
        for p in &paths {
            drop(PrivateRecord::create(p, b"", 16).unwrap());
            let c = Connection::open(p).unwrap();
            c.execute_batch("PRAGMA journal_mode=DELETE; CREATE TABLE synthetic(value INTEGER); INSERT INTO synthetic VALUES(1)").unwrap();
            c.close().unwrap();
        }
        Self {
            fixture: Some(fixture),
            paths,
        }
    }
}
impl Drop for Scene {
    fn drop(&mut self) {
        if let Some(f) = self.fixture.take() {
            let path = f.path().to_path_buf();
            drop(f);
            if !std::thread::panicking() {
                assert!(!path.exists());
                println!("r7-lease-prefix-cleanup-absent={}", path.display());
            }
        }
    }
}
fn end() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
fn images(paths: &[PathBuf; 4]) -> Vec<(FileIdentity, SnapshotData, Vec<u8>)> {
    paths
        .iter()
        .map(|p| {
            let file = open(p).unwrap();
            (
                file_id(&file).unwrap(),
                native_stamp(&file, end()).unwrap(),
                std::fs::read(p).unwrap(),
            )
        })
        .collect()
}
fn actual_ranges_block(held: &Held) {
    for r in &held.ranges {
        let f = open(&held.pin.path).unwrap();
        let mut overlap: OVERLAPPED = unsafe { std::mem::zeroed() };
        overlap.Anonymous.Anonymous.Offset = r.offset;
        let raw = unsafe {
            LockFileEx(
                f.as_raw_handle(),
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0,
                r.length,
                0,
                &mut overlap,
            )
        };
        assert_eq!(
            raw, 0,
            "actual exact retained range must block foreign exclusive description"
        );
        assert_eq!(io::Error::last_os_error().raw_os_error(), Some(33));
    }
}
fn cleanup_own_failed(prefix: &mut LeasePreparation) {
    assert!(prefix.failed && !prefix.ready);
    if let Some(lease) = &mut prefix.lease {
        assert!(!lease.active);
        for held in lease.held.iter_mut().rev() {
            held.release().unwrap();
            assert!(held.ranges.is_empty());
        }
    }
    // Fixture-only explicit release is not a valid lease/normal release receipt.
    assert!(prefix.take_complete().is_err());
}
#[test]
fn wl2_r7_lease_prefix_native_missing_reserved_range_revokes_authority_and_keeps_claim() {
    let scene = Scene::new();
    let deadline = end();
    let mut prefix = LeasePreparation::default();
    prefix.prepare_fixture_paths(&scene.paths, deadline).unwrap();
    let mut lease = prefix.take_complete().unwrap();
    lease.check(deadline).unwrap();

    // A successful OS unlock changes a single SQLite role without changing the
    // pinned files, original bytes, or the original claim and other ranges.
    lease.held[2].unlock(BASE + 1).unwrap();
    for held in &lease.held {
        actual_ranges_block(held);
    }
    assert!(lease.check(deadline).is_err());
    assert!(!lease.active);
    assert!(lease
        .anchored_claim(scene.paths[0].parent().unwrap(), deadline)
        .is_err());
    actual_ranges_block(&lease.held[0]);

    // Explicit fixture-only cleanup after verifying the surviving ranges.
    for held in lease.held.iter_mut().rev() {
        held.release().unwrap();
    }
}
#[test]
fn wl2_r7_lease_prefix_native_each_partial_open_and_range_failure_retains_exact_owned_resources() {
    for fault in [1, 2, 3, 4, 5, 6, 7, 8, 20, 21, 22] {
        let scene = Scene::new();
        let before = images(&scene.paths);
        let mut prefix = LeasePreparation::default();
        prefix.fault = fault;
        assert!(prefix.acquire(&scene.paths, None, end()).is_err());
        assert!(prefix.attempted && prefix.failed && !prefix.ready);
        assert!(prefix.take_complete().is_err());
        let lease = prefix.lease.as_ref().unwrap();
        let actual = lease
            .held
            .iter()
            .map(|h| {
                (
                    file_id(&h.pin.file).unwrap(),
                    h.ranges
                        .iter()
                        .map(|r| (r.offset, r.length))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        for held in &lease.held {
            actual_ranges_block(held);
        }
        match fault {
            1 => assert!(lease.held.is_empty() && prefix.pending.file.is_none()),
            2 | 3 => {
                assert!(prefix.pending.file.is_some());
                assert_eq!(
                    file_id(prefix.pending.file.as_ref().unwrap()).unwrap(),
                    before[0].0
                );
            }
            4 => assert!(prefix.pending.complete.is_some()),
            5 => assert_eq!(lease.held[0].ranges.len(), 0),
            6 | 7 => assert_eq!(lease.held[0].ranges.len(), 1),
            8 => assert_eq!(lease.held.len(), 4),
            20 => assert_eq!(
                lease.held[1]
                    .ranges
                    .iter()
                    .map(|r| (r.offset, r.length))
                    .collect::<Vec<_>>(),
                vec![(BASE, 1)]
            ),
            21 => assert_eq!(
                lease.held[1]
                    .ranges
                    .iter()
                    .map(|r| (r.offset, r.length))
                    .collect::<Vec<_>>(),
                vec![(BASE, 1), (BASE + 2, 510)]
            ),
            22 => assert_eq!(
                lease.held[1]
                    .ranges
                    .iter()
                    .map(|r| (r.offset, r.length))
                    .collect::<Vec<_>>(),
                vec![(BASE + 2, 510)]
            ),
            _ => unreachable!(),
        }
        assert!(prefix.acquire(&scene.paths, None, end()).is_err());
        assert_eq!(
            prefix
                .lease
                .as_ref()
                .unwrap()
                .held
                .iter()
                .map(|h| (
                    file_id(&h.pin.file).unwrap(),
                    h.ranges
                        .iter()
                        .map(|r| (r.offset, r.length))
                        .collect::<Vec<_>>()
                ))
                .collect::<Vec<_>>(),
            actual
        );
        cleanup_own_failed(&mut prefix);
        drop(prefix);
        assert!(images(&scene.paths) == before);
        println!("r7 actual retained lease stage{fault}: own original handles/ranges kept, native competitor refused held ranges; no retry/take/normal release proof; explicit own fixture cleanup");
        drop(scene);
    }
}
#[test]
fn wl2_r7_lease_prefix_native_real_sqlite_busy_each_role_keeps_foreign_transaction_and_own_prefix()
{
    for role in 0..4 {
        let scene = Scene::new();
        let before = images(&scene.paths);
        let c = Connection::open(&scene.paths[role]).unwrap();
        c.busy_timeout(Duration::ZERO).unwrap();
        c.execute_batch("BEGIN IMMEDIATE").unwrap();
        let mut prefix = LeasePreparation::default();
        let error = prefix.acquire(&scene.paths, None, end()).unwrap_err();
        assert_eq!(error.raw_os_error(), Some(33));
        assert!(prefix.failed && !prefix.ready);
        let lease = prefix.lease.as_ref().unwrap();
        assert_eq!(lease.held.len(), role + 1);
        for held in &lease.held {
            actual_ranges_block(held);
        }
        assert!(!c.is_autocommit());
        assert!(prefix.take_complete().is_err());
        cleanup_own_failed(&mut prefix);
        assert!(!c.is_autocommit());
        c.execute_batch("ROLLBACK").unwrap();
        c.close().unwrap();
        drop(prefix);
        assert!(images(&scene.paths) == before);
        for p in &scene.paths {
            let c = Connection::open(p).unwrap();
            c.busy_timeout(Duration::ZERO).unwrap();
            c.execute_batch("BEGIN EXCLUSIVE; ROLLBACK").unwrap();
            c.close().unwrap();
        }
        println!("r7 real SQLite busy role{role} raw33: original foreign transaction remains, exact own partial ranges retained until explicit fixture cleanup");
        drop(scene);
    }
}
#[test]
fn wl2_r7_lease_prefix_native_invalid_bytes_or_ads_keep_original_open_and_preceding_ranges() {
    for change in ["bytes", "ads"] {
        let scene = Scene::new();
        if change == "bytes" {
            std::fs::write(&scene.paths[2], b"unknown original database bytes").unwrap();
        } else {
            std::fs::write(
                PathBuf::from(format!("{}:unknown-r7", scene.paths[2].display())),
                b"own unknown stream",
            )
            .unwrap();
        }
        let raw = std::fs::read(&scene.paths[2]).unwrap();
        let original = file_id(&open(&scene.paths[2]).unwrap()).unwrap();
        let mut prefix = LeasePreparation::default();
        assert!(prefix.acquire(&scene.paths, None, end()).is_err());
        assert!(prefix.failed && prefix.pending.file.is_some());
        assert_eq!(prefix.lease.as_ref().unwrap().held.len(), 2);
        assert_eq!(
            file_id(prefix.pending.file.as_ref().unwrap()).unwrap(),
            original
        );
        for held in &prefix.lease.as_ref().unwrap().held {
            actual_ranges_block(held);
        }
        assert!(std::fs::remove_file(&scene.paths[2]).is_err());
        assert!(prefix.take_complete().is_err());
        cleanup_own_failed(&mut prefix);
        drop(prefix);
        assert_eq!(std::fs::read(&scene.paths[2]).unwrap(), raw);
        drop(scene);
    }
}
#[test]
fn wl2_r7_lease_prefix_native_success_transfers_same_files_ranges_once_and_expiry_never_retries() {
    let scene = Scene::new();
    let before = images(&scene.paths);
    let end = end();
    let mut prefix = LeasePreparation::default();
    prefix.acquire(&scene.paths, None, end).unwrap();
    let pointers = prefix
        .lease
        .as_ref()
        .unwrap()
        .held
        .iter()
        .map(|h| h.pin.file.as_raw_handle())
        .collect::<Vec<_>>();
    let epoch = prefix.lease.as_ref().unwrap().epoch.clone();
    let mut lease = prefix.take_complete().unwrap();
    assert!(prefix.take_complete().is_err());
    assert!(prefix.acquire(&scene.paths, None, end).is_err());
    assert_eq!(
        lease
            .held
            .iter()
            .map(|h| h.pin.file.as_raw_handle())
            .collect::<Vec<_>>(),
        pointers
    );
    assert_eq!(lease.epoch, epoch);
    let root = scene.paths[0].parent().unwrap();
    let mut claim = lease.anchored_claim(root, end).unwrap();
    claim.check(root, end).unwrap();
    drop(claim);
    for held in &lease.held {
        actual_ranges_block(held);
    }
    lease.release(end).unwrap();
    drop(prefix);
    assert!(images(&scene.paths) == before);
    let mut expired = LeasePreparation::default();
    assert_eq!(
        expired
            .acquire(&scene.paths, None, Instant::now())
            .unwrap_err()
            .kind(),
        io::ErrorKind::TimedOut
    );
    assert!(expired.acquire(&scene.paths, None, end).is_err());
    assert!(expired.take_complete().is_err());
    let mut duplicate = LeasePreparation::default();
    let paths = std::array::from_fn(|_| scene.paths[0].clone());
    assert!(duplicate.acquire(&paths, None, end).is_err());
    assert!(duplicate.lease.is_none());
    drop(expired);
    drop(duplicate);
    drop(scene);
}
#[test]
fn wl2_r7_lease_prefix_type_boundary_stays_non_send_and_non_sync_without_unsafe_impl() {
    trait AmbiguousSend<A> {
        fn verify() {}
    }
    impl<T: ?Sized> AmbiguousSend<()> for T {}
    impl<T: ?Sized + Send> AmbiguousSend<u8> for T {}
    let _ = <LeasePreparation as AmbiguousSend<_>>::verify;
    let _ = <NativeLease as AmbiguousSend<_>>::verify;
    trait AmbiguousSync<A> {
        fn verify() {}
    }
    impl<T: ?Sized> AmbiguousSync<()> for T {}
    impl<T: ?Sized + Sync> AmbiguousSync<u8> for T {}
    let _ = <LeasePreparation as AmbiguousSync<_>>::verify;
    let _ = <NativeLease as AmbiguousSync<_>>::verify;
}
