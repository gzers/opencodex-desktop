#![cfg(windows)]
#[path = "../../apps/desktop/tauri/src/infrastructure/windows_official_runtime024/mod.rs"]
pub mod backend;
pub use backend as windows_official_runtime024;
#[path = "../../apps/desktop/tauri/src/infrastructure/windows_cli024.rs"]
pub mod windows_cli024;

#[cfg(test)]
mod tests {
    use super::backend::*;
    use std::{
        fs, io,
        path::PathBuf,
        sync::atomic::{AtomicU64, Ordering},
    };
    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "ocxd024-native-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&p).unwrap();
            Self(p)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).expect("remove exact new test directory");
        }
    }
    #[test]
    fn real_token_user_is_observed() {
        let a = current_identity().unwrap();
        assert!(a.sid.starts_with("S-1-5-21-"));
        assert!((1..=3).contains(&a.elevation_type));
        assert_eq!(a, current_identity().unwrap());
    }
    #[test]
    fn private_record_pins_bytes_and_generation() {
        let f = Fixture::new();
        let root = f.0.join("private");
        let dir = create_private_directory(&root).unwrap();
        let path = root.join("record");
        let mut record = PrivateRecord::create(&path, b"synthetic", 64).unwrap();
        assert_eq!(record.read().unwrap(), b"synthetic");
        assert!(fs::write(&path, b"forbidden").is_err());
        assert!(fs::rename(&path, root.join("replaced")).is_err());
        assert!(fs::rename(&root, f.0.join("moved")).is_err());
        assert!(PrivateRecord::create(&path, b"overwrite", 64).is_err());
        let id = record.identity();
        drop(record);
        let mut read = PrivateRecord::open(&path, 64).unwrap();
        assert_eq!(id, read.identity());
        assert_eq!(read.read().unwrap(), b"synthetic");
        drop(read);
        drop(dir);
    }
    #[test]
    fn wl2_h_published_record_allows_independent_readers_while_denies_mutations() {
        let f = Fixture::new();
        let dir = create_private_directory(&f.0.join("private")).unwrap();
        let p = dir.path().join("rendezvous");
        let mut creator =
            PrivateRecord::create(&p, b"nonsecret locator bound by native peer", 128).unwrap();
        let mut reader = PrivateRecord::open(&p, 128).expect(
            "a manager must read the live published locator without relaxing immutable sharing",
        );
        assert_eq!(creator.identity(), reader.identity());
        assert_eq!(creator.read().unwrap(), reader.read().unwrap());
        assert!(fs::write(&p, b"bad replacement").is_err());
        assert!(fs::rename(&p, dir.path().join("other")).is_err());
        assert!(fs::remove_file(&p).is_err());
        drop(creator);
        assert!(fs::write(&p, b"still forbidden while independent reader retained").is_err());
        assert_eq!(
            reader.read().unwrap(),
            b"nonsecret locator bound by native peer"
        );
        drop(reader);
        drop(dir);
    }
    #[test]
    fn wl2_h_original_writer_share_mode_reproduces_native_conflict() {
        use std::os::windows::fs::OpenOptionsExt;
        let f = Fixture::new();
        let p = f.0.join("unsealed-fixture");
        let writer = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .share_mode(1)
            .open(&p)
            .unwrap();
        let failure = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&p)
            .unwrap_err();
        assert_eq!(failure.raw_os_error(), Some(32));
        // Diagnostic collection is read-only and may acknowledge the existing
        // writer. This is not how immutable PrivateRecord is sealed/published.
        let diagnostic = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(3)
            .open(&p)
            .unwrap();
        drop(diagnostic);
        drop(writer);
    }
    #[test]
    fn wl2_h_locator_snapshot_is_data_not_mutation_or_native_authority() {
        let f = Fixture::new();
        let dir = create_private_directory(&f.0.join("private")).unwrap();
        let p = dir.path().join("locator");
        let original = PrivateRecord::create(&p, b"nonsecret candidate address", 128).unwrap();
        assert_eq!(
            PrivateRecord::locator_snapshot(&p, 128).unwrap(),
            b"nonsecret candidate address"
        );
        assert!(fs::write(&p, b"cannot mutate").is_err());
        assert!(PrivateRecord::locator_snapshot(&p, 1).is_err());
        assert!(PrivateRecord::locator_snapshot(&p, 65537).is_err());
        // Snapshot only returns bytes, not PrivateRecord/RetainedPeer/Owner.
        drop(original);
        use std::os::windows::fs::OpenOptionsExt;
        let old_creator = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .share_mode(1)
            .open(&p)
            .unwrap();
        assert_eq!(
            PrivateRecord::open(&p, 128).err().unwrap().raw_os_error(),
            Some(32)
        );
        assert_eq!(
            PrivateRecord::locator_snapshot(&p, 128).unwrap(),
            b"nonsecret candidate address"
        );
        assert!(std::fs::OpenOptions::new()
            .write(true)
            .share_mode(3)
            .open(&p)
            .is_err());
        drop(old_creator);
        drop(dir);
    }
    #[test]
    fn linked_or_broad_objects_are_refused_without_repair() {
        let f = Fixture::new();
        let broad = f.0.join("broad");
        fs::create_dir(&broad).unwrap();
        fs::write(broad.join("old"), b"keep").unwrap();
        assert!(PinnedDirectory::open_private(&broad).is_err());
        assert!(PrivateRecord::open(&broad.join("old"), 64).is_err());
        assert_eq!(fs::read(broad.join("old")).unwrap(), b"keep");
        let root = f.0.join("private");
        let dir = create_private_directory(&root).unwrap();
        let path = root.join("record");
        drop(PrivateRecord::create(&path, b"keep", 64).unwrap());
        fs::hard_link(&path, root.join("hard")).unwrap();
        assert!(PrivateRecord::open(&path, 64).is_err());
        drop(dir);
    }
    #[test]
    fn unsafe_names_and_bounds_fail_before_creation() {
        let f = Fixture::new();
        let dir = create_private_directory(&f.0.join("private")).unwrap();
        for name in ["CON", "stream:secret", "trailing.", "bad ", "sub/../out"] {
            assert!(PrivateRecord::create(&dir.path().join(name), b"x", 64).is_err());
        }
        assert!(PrivateRecord::create(&dir.path().join("large"), b"123", 2).is_err());
        assert!(!dir.path().join("large").exists());
        assert_eq!(
            PrivateRecord::open(&dir.path().join("missing"), 64)
                .err()
                .unwrap()
                .kind(),
            io::ErrorKind::NotFound
        );
    }
}
