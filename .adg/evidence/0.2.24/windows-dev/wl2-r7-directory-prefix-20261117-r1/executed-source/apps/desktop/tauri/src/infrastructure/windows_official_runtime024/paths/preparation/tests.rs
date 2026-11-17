use super::*;
use std::time::Duration;
struct Scene(Option<tempfile::TempDir>);
impl Scene {
    fn new() -> Self {
        Self(Some(tempfile::tempdir().unwrap()))
    }
    fn path(&self) -> &Path {
        self.0.as_ref().unwrap().path()
    }
}
impl Drop for Scene {
    fn drop(&mut self) {
        if let Some(temp) = self.0.take() {
            if std::thread::panicking() {
                println!(
                    "r7-directory-failed-fixture-retained={}",
                    temp.keep().display()
                );
            } else {
                let path = temp.path().to_path_buf();
                temp.close().unwrap();
                assert!(!path.exists());
                println!("r7-directory-cleanup-absent={}", path.display());
            }
        }
    }
}
fn end() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
#[test]
fn wl2_r7_directory_prefix_native_every_failure_keeps_exact_created_prefix_and_refuses_retry() {
    for stage in 1..=5 {
        let scene = Scene::new();
        let path = scene.path().join("original-preparation");
        let mut prefix = PrivateDirectoryPreparation::default();
        prefix.fault = stage;
        assert!(prefix.create(&path, end()).is_err());
        assert!(prefix.attempted && prefix.failed && !prefix.ready);
        assert!(prefix.take_complete().is_err());
        assert_eq!(prefix.created, stage >= 2);
        assert_eq!(path.exists(), stage >= 2);
        if stage == 2 {
            assert!(prefix.parent.is_some());
            assert!(prefix.pending.is_none() && prefix.complete.is_none());
        }
        if stage == 3 || stage == 4 {
            let original = prefix.pending.as_ref().unwrap();
            identity(original, true).unwrap();
            assert!(std::fs::rename(&path, scene.path().join("replacement")).is_err());
            assert!(std::fs::remove_dir(&path).is_err());
        }
        if stage == 5 {
            let original = prefix.complete.as_ref().unwrap();
            original.recheck().unwrap();
            assert!(prefix.parent.is_none() && prefix.pending.is_none());
            assert!(std::fs::rename(&path, scene.path().join("replacement")).is_err());
        }
        assert!(prefix.create(&path, end()).is_err());
        assert!(prefix.create(&scene.path().join("renewed"), end()).is_err());
        assert!(!scene.path().join("renewed").exists());
        assert_eq!(path.exists(), stage >= 2);
        println!("r7 actual directory stage{stage}: original parent/create fact/native-open prefix retained; incomplete refuses take/retry");
        drop(prefix);
        drop(scene);
    }
}
#[test]
fn wl2_r7_directory_prefix_native_complete_transfers_same_original_handles_without_reopen() {
    let scene = Scene::new();
    let path = scene.path().join("actual-private-directory");
    let mut prefix = PrivateDirectoryPreparation::default();
    prefix.create(&path, end()).unwrap();
    assert!(prefix.created && prefix.ready && !prefix.failed);
    let complete = prefix.complete.as_ref().unwrap();
    let original_handle = complete.handle();
    let original_ids = complete.owned_file_identities().unwrap();
    let pin = prefix.take_complete().unwrap();
    assert_eq!(pin.handle(), original_handle);
    assert_eq!(pin.owned_file_identities().unwrap(), original_ids);
    assert_eq!(pin.identity(), *original_ids.last().unwrap());
    assert!(prefix.take_complete().is_err());
    assert!(std::fs::rename(&path, scene.path().join("must-not-rename")).is_err());
    verify_private(pin.handle(), &current_identity().unwrap().sid, true).unwrap();
    drop(pin);
    drop(prefix);
    drop(scene);
}
#[test]
fn wl2_r7_directory_prefix_native_expired_and_existing_name_refuse_without_mutation() {
    let scene = Scene::new();
    let path = scene.path().join("expired");
    let mut prefix = PrivateDirectoryPreparation::default();
    assert_eq!(
        prefix.create(&path, Instant::now()).unwrap_err().kind(),
        io::ErrorKind::TimedOut
    );
    assert!(!prefix.created && !path.exists());
    assert!(prefix.take_complete().is_err());
    drop(prefix);
    let existing = scene.path().join("existing");
    let old = create_private_directory(&existing).unwrap();
    std::fs::write(
        existing.join("preserve.bin"),
        b"exact original existing bytes",
    )
    .unwrap();
    let id = old.identity();
    let mut prefix = PrivateDirectoryPreparation::default();
    assert!(prefix.create(&existing, end()).is_err());
    assert!(!prefix.created && prefix.pending.is_none() && prefix.complete.is_none());
    assert!(prefix.parent.is_some() && prefix.failed);
    assert!(prefix.take_complete().is_err());
    assert_eq!(old.identity(), id);
    old.recheck().unwrap();
    assert_eq!(
        std::fs::read(existing.join("preserve.bin")).unwrap(),
        b"exact original existing bytes"
    );
    drop(prefix);
    drop(old);
    drop(scene);
}
#[test]
fn wl2_r7_directory_prefix_native_unopened_create_never_adopts_replacement_or_proves_identity() {
    let scene = Scene::new();
    let path = scene.path().join("created-without-terminal-open");
    let moved = scene.path().join("external-retained-original");
    let mut prefix = PrivateDirectoryPreparation::default();
    prefix.fault = 2;
    assert!(prefix.create(&path, end()).is_err());
    assert!(prefix.created && prefix.pending.is_none() && prefix.complete.is_none());
    // Win32 create returned no handle. An external change is possible here;
    // it is never converted into an identity or restoration grant by retry.
    std::fs::rename(&path, &moved).unwrap();
    let replacement = create_private_directory(&path).unwrap();
    std::fs::write(path.join("foreign.bin"), b"do not adopt or repair").unwrap();
    assert!(prefix.take_complete().is_err());
    assert!(prefix.create(&path, end()).is_err());
    assert!(prefix.pending.is_none() && prefix.complete.is_none());
    assert_eq!(
        std::fs::read(path.join("foreign.bin")).unwrap(),
        b"do not adopt or repair"
    );
    assert!(moved.exists());
    drop(replacement);
    drop(prefix);
    drop(scene);
}
