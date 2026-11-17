use super::*;
use crate::infrastructure::windows_official_runtime024::{create_private_directory, PrivateRecord};
use std::{
    fs::OpenOptions,
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    time::Duration,
};
struct Scene(Option<tempfile::TempDir>);
impl Drop for Scene {
    fn drop(&mut self) {
        if let Some(root) = self.0.take() {
            if thread::panicking() {
                println!("r7-canonical-trait-failed-root={}", root.keep().display());
            } else {
                let path = root.path().to_path_buf();
                root.close().unwrap();
                assert!(!path.exists());
                println!("r7-canonical-trait-cleanup-absent={}", path.display());
            }
        }
    }
}
fn busy(path: &std::path::Path) {
    use windows_sys::Win32::{
        Storage::FileSystem::{LockFileEx, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY},
        System::IO::OVERLAPPED,
    };
    let file = OpenOptions::new()
        .read(true)
        .share_mode(3)
        .open(path)
        .unwrap();
    let mut overlap: OVERLAPPED = unsafe { std::mem::zeroed() };
    overlap.Anonymous.Anonymous.Offset = 0x40000000;
    assert_eq!(
        unsafe {
            LockFileEx(
                file.as_raw_handle(),
                LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
                0,
                512,
                0,
                &mut overlap,
            )
        },
        0
    );
    assert_eq!(io::Error::last_os_error().raw_os_error(), Some(33));
}
#[test]
fn wl2_r7_canonical_trait_native_stage_borrows_keep_same_actual_claim_nlk_until_explicit_release() {
    let scene = Scene(Some(tempfile::tempdir().unwrap()));
    let root = scene.0.as_ref().unwrap().path();
    let client = root.join(".codex");
    drop(create_private_directory(&client).unwrap());
    drop(create_private_directory(&root.join("coordination")).unwrap());
    let paths = [
        client.join(".opencodex-native-main.claim.sqlite"),
        root.join("coordination/n.sqlite"),
        root.join("coordination/l.sqlite"),
        root.join("coordination/k.sqlite"),
    ];
    for path in &paths {
        drop(PrivateRecord::create(path, b"", 16).unwrap());
    }
    let end = Instant::now() + Duration::from_secs(10);
    let mut local = WorkerLocal::new();
    local
        .preparation
        .prepare_fixture_paths(&paths, end)
        .unwrap();
    local.lease = Some(local.preparation.take_complete().unwrap());
    let first = {
        let mut claim = local.lease().unwrap().anchored_claim(&client, end).unwrap();
        claim.check(&client, end).unwrap();
        (claim.epoch().to_owned(), claim.key())
    };
    for stage in ["prepared", "quiescent", "return"] {
        for path in &paths {
            busy(path);
        }
        let mut claim = local.lease().unwrap().anchored_claim(&client, end).unwrap();
        claim.check(&client, end).unwrap();
        assert_eq!((claim.epoch().to_owned(), claim.key()), first);
        drop(claim);
        for path in &paths {
            busy(path);
        }
        println!("r7 canonicaltrait actuallease {stage}: borrowedclaim dropped only, sameepoch/key and all4actualranges remain raw33; no originalOwner constructed");
    }
    local.release(end).unwrap();
    local.released_for_transfer(end).unwrap();
    assert!(local.lease.is_none() && local.released.is_some());
    drop(local);
    drop(scene);
}
#[test]
fn wl2_r7_canonical_trait_original_metadata_thread_and_deadline_substitutions_refuse() {
    let now = Instant::now();
    let end = now + Duration::from_secs(1);
    let current = thread::current().id();
    let binding = Binding {
        session: uuid::Uuid::new_v4().to_string(),
        action: uuid::Uuid::new_v4().to_string(),
        scope: "same native owner scope DATA".into(),
        end,
        nonce: uuid::Uuid::new_v4(),
        thread: current,
    };
    binding
        .check_metadata(
            &binding.session,
            &binding.action,
            &binding.scope,
            end,
            current,
        )
        .unwrap();
    assert!(binding
        .check_metadata("foreign", &binding.action, &binding.scope, end, current)
        .is_err());
    assert!(binding
        .check_metadata(&binding.session, "foreign", &binding.scope, end, current)
        .is_err());
    assert!(binding
        .check_metadata(&binding.session, &binding.action, "foreign", end, current)
        .is_err());
    assert!(binding
        .check_metadata(
            &binding.session,
            &binding.action,
            &binding.scope,
            end + Duration::from_nanos(1),
            current
        )
        .is_err());
    let foreign = thread::spawn(|| thread::current().id()).join().unwrap();
    assert!(binding
        .check_metadata(
            &binding.session,
            &binding.action,
            &binding.scope,
            end,
            foreign
        )
        .is_err());
    assert!(binding
        .check_metadata(
            &binding.session,
            &binding.action,
            &binding.scope,
            Instant::now(),
            current
        )
        .is_err());
    println!("r7 canonicaltrait metadata guard only, no nativeproof created; crosssession/action/scope/end/thread/cutoff refused");
}
#[test]
fn wl2_r7_canonical_trait_bound_types_implement_unchanged_contract_without_clone_or_serde() {
    fn implements<
        T: BeforeFirstResumeRecovery<
            Owner = Owner,
            PreparedAbort = PreparedCanonicalAbort,
            QuiescentLauncher = QuiescentCanonicalLauncher,
            AbortRestored = RestoredCanonicalOriginals,
        >,
    >() {
    }
    implements::<WorkerLocal>();
    trait AmbiguousClone<A> {
        fn verify() {}
    }
    impl<T: ?Sized> AmbiguousClone<()> for T {}
    impl<T: ?Sized + Clone> AmbiguousClone<u8> for T {}
    let _ = <PreparedCanonicalAbort as AmbiguousClone<_>>::verify;
    let _ = <QuiescentCanonicalLauncher as AmbiguousClone<_>>::verify;
    let _ = <RestoredCanonicalOriginals as AmbiguousClone<_>>::verify;
    trait AmbiguousSerialize<A> {
        fn verify() {}
    }
    impl<T: ?Sized> AmbiguousSerialize<()> for T {}
    impl<T: ?Sized + serde::Serialize> AmbiguousSerialize<u8> for T {}
    let _ = <PreparedCanonicalAbort as AmbiguousSerialize<_>>::verify;
    let _ = <QuiescentCanonicalLauncher as AmbiguousSerialize<_>>::verify;
    let _ = <RestoredCanonicalOriginals as AmbiguousSerialize<_>>::verify;
    trait AmbiguousSend<A> {
        fn verify() {}
    }
    impl<T: ?Sized> AmbiguousSend<()> for T {}
    impl<T: ?Sized + Send> AmbiguousSend<u8> for T {}
    let _ = <WorkerLocal as AmbiguousSend<_>>::verify;
}
