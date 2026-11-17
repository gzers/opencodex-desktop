use super::super::tests::{bytes as encoded_data, chain, events, record};
use super::*;
use std::time::Duration;
struct Scene(Option<tempfile::TempDir>);
impl Scene {
    fn new() -> Self {
        Self(Some(tempfile::tempdir().unwrap()))
    }
    fn path(&self) -> &std::path::Path {
        self.0.as_ref().unwrap().path()
    }
    fn directory(&self) -> PinnedDirectory {
        native::create_private_directory(&self.path().join("records")).unwrap()
    }
    fn initialized(&self, end: Instant) -> AttachmentStorage {
        let mut original = AttachmentStorage::default();
        original.initialize(self.directory(), end).unwrap();
        original
    }
}
impl Drop for Scene {
    fn drop(&mut self) {
        if let Some(temp) = self.0.take() {
            if std::thread::panicking() {
                println!(
                    "r7-attachment-failed-fixture-retained={}",
                    temp.keep().display()
                );
            } else {
                let path = temp.path().to_path_buf();
                temp.close().unwrap();
                assert!(!path.exists());
                println!("r7-attachment-cleanup-absent={}", path.display());
            }
        }
    }
}
fn end() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
#[test]
fn wl2_r7_attachment_storage_native_complete_strict_chain_exact_original_files_and_no_authority() {
    let scene = Scene::new();
    let end = end();
    let mut original = scene.initialized(end);
    let raw = chain(&events());
    for bytes in &raw {
        original.append_data(bytes, end).unwrap();
    }
    assert_eq!(original.read_data(end).unwrap(), raw);
    assert_eq!(original.files.len(), 7);
    for stored in &original.files {
        assert_eq!(std::fs::read(&stored.path).unwrap(), stored.raw);
        assert_eq!(native_id(&stored.file, false).unwrap(), stored.id);
        assert!(stamp(&stored.file, end).unwrap() == stored.security);
        assert!(OpenOptions::new().write(true).open(&stored.path).is_err());
        assert!(std::fs::remove_file(&stored.path).is_err());
    }
    assert!(std::fs::rename(
        original.directory.as_ref().unwrap().path(),
        scene.path().join("replaced")
    )
    .is_err());
    let mut extra = events();
    extra.push(serde_json::json!({"kind":"AbortRecoveryPending","stage":"cannot-after-terminal","rawError":null}));
    assert!(original
        .append_data(chain(&extra).last().unwrap(), end)
        .is_err());
    assert!(original.failed && original.files.len() == 7);
    println!("r7 actual native attachment bytes/ID/all8/ADS/complete directory verified; terminal suffix rejected, immutable files retained; DATA never grants restore/Stop/Closed");
    drop(original);
    drop(scene);
}
#[test]
fn wl2_r7_attachment_storage_native_every_empty_half_full_or_published_failure_keeps_exact_prefix()
{
    let raw = encoded_data(&record());
    for stage in 1..=5 {
        let scene = Scene::new();
        let end = end();
        let mut original = scene.initialized(end);
        original.fault = stage;
        assert!(original.append_data(&raw, end).is_err());
        assert!(original.failed && !original.ready);
        let path = scene.path().join("records/record-000001.json");
        assert_eq!(path.exists(), stage >= 2);
        if stage <= 4 {
            assert!(original.pending_raw.is_some() && original.pending_path.is_some());
        }
        if stage == 2 {
            assert!(original.pending_file.is_some());
            assert_eq!(std::fs::metadata(&path).unwrap().len(), 0);
        }
        if stage == 3 {
            assert_eq!(std::fs::read(&path).unwrap(), raw[..raw.len() / 2]);
        }
        if stage == 4 || stage == 5 {
            assert_eq!(std::fs::read(&path).unwrap(), raw);
        }
        if stage == 5 {
            assert!(original.pending_file.is_none());
            assert_eq!(original.files.len(), 1);
        }
        if stage >= 2 {
            let file = if stage == 5 {
                &original.files[0].file
            } else {
                original.pending_file.as_ref().unwrap()
            };
            let id = native_id(file, false).unwrap();
            assert!(OpenOptions::new().write(true).open(&path).is_err());
            assert!(std::fs::remove_file(&path).is_err());
            assert!(original.append_data(&raw, end).is_err());
            let file = if stage == 5 {
                &original.files[0].file
            } else {
                original.pending_file.as_ref().unwrap()
            };
            assert_eq!(native_id(file, false).unwrap(), id);
        } else {
            assert!(original.append_data(&raw, end).is_err());
        }
        assert!(original.read_data(end).is_err());
        println!("r7 actual native attachment stage{stage}: original pending/written resources and exact bytes kept, replay/read-success refused");
        drop(original);
        drop(scene);
    }
}
#[test]
fn wl2_r7_attachment_storage_native_cross_scope_replay_unknown_and_wrong_sequence_refuse_before_write(
) {
    let full = chain(&events());
    for variant in ["replay", "scope", "unknown", "gap"] {
        let scene = Scene::new();
        let end = end();
        let mut original = scene.initialized(end);
        original.append_data(&full[0], end).unwrap();
        let mut v: serde_json::Value = serde_json::from_slice(&full[1]).unwrap();
        let raw = match variant {
            "replay" => full[0].clone(),
            "scope" => {
                v["recoveryActionId"] = "44444444-4444-4444-4444-444444444444".into();
                encoded_data(&v)
            }
            "unknown" => {
                v["event"]["unexpected"] = true.into();
                encoded_data(&v)
            }
            _ => {
                v["sequence"] = 3.into();
                encoded_data(&v)
            }
        };
        assert!(original.append_data(&raw, end).is_err());
        assert_eq!(original.files.len(), 1);
        assert!(original.pending_file.is_none());
        assert!(!scene.path().join("records/record-000002.json").exists());
        assert_eq!(std::fs::read(&original.files[0].path).unwrap(), full[0]);
        drop(original);
        drop(scene);
    }
}
fn change_dacl(path: &std::path::Path) {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
            text: *const u16,
            revision: u32,
            descriptor: *mut *mut std::ffi::c_void,
            size: *mut u32,
        ) -> i32;
        fn SetFileSecurityW(
            path: *const u16,
            information: u32,
            descriptor: *const std::ffi::c_void,
        ) -> i32;
    }
    let text = "D:P(A;;FA;;;OW)(A;;FA;;;SY)"
        .encode_utf16()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let name = path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let mut sd = std::ptr::null_mut();
    assert_ne!(
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text.as_ptr(),
                1,
                &mut sd,
                std::ptr::null_mut(),
            )
        },
        0
    );
    let result = unsafe { SetFileSecurityW(name.as_ptr(), 4, sd) };
    unsafe {
        windows_sys::Win32::Foundation::LocalFree(sd);
    }
    assert_ne!(result, 0, "only new own fixture DACL change");
}
#[test]
fn wl2_r7_attachment_storage_native_member_file_security_directory_security_or_ads_drift_refuse() {
    for variant in [
        "member",
        "file-security",
        "directory-security",
        "ads",
        "late-file-security",
        "late-member",
    ] {
        let scene = Scene::new();
        let end = end();
        let mut original = scene.initialized(end);
        original.append_data(&encoded_data(&record()), end).unwrap();
        let path = original.files[0].path.clone();
        match variant {
            "member" => std::fs::write(scene.path().join("records/foreign.json"), b"{}").unwrap(),
            "file-security" => change_dacl(&path),
            "directory-security" => change_dacl(&scene.path().join("records")),
            "late-file-security" => {
                original.after_read =
                    Some(|directory| change_dacl(&directory.join("record-000001.json")));
            }
            "late-member" => {
                original.after_read = Some(|directory| {
                    std::fs::write(directory.join("late-foreign.json"), b"{}").unwrap();
                });
            }
            _ => {
                let stream = PathBuf::from(format!(
                    "{}:unknown-own-stream",
                    scene.path().join("records").display()
                ));
                std::fs::write(stream, b"fixture unknown stream").unwrap();
            }
        }
        assert!(original.read_data(end).is_err(), "{variant}");
        assert!(original.failed);
        assert_eq!(std::fs::read(path).unwrap(), encoded_data(&record()));
        println!("r7 actual native attachment refuses {variant}; original file/content retained");
        drop(original);
        drop(scene);
    }
}
#[test]
fn wl2_r7_attachment_storage_native_expired_renewed_or_existing_namespace_never_adopts() {
    for variant in ["expired-init", "renewed-read", "expired-append", "existing"] {
        let scene = Scene::new();
        let directory = scene.directory();
        let end = end();
        let mut original = AttachmentStorage::default();
        if variant == "existing" {
            std::fs::write(
                directory.path().join("old-record.json"),
                b"preserve existing",
            )
            .unwrap();
        }
        if variant == "expired-init" {
            assert_eq!(
                original
                    .initialize(directory, Instant::now())
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::TimedOut
            );
            assert!(original.directory.is_some());
        } else if variant == "existing" {
            assert!(original.initialize(directory, end).is_err());
            assert_eq!(
                std::fs::read(scene.path().join("records/old-record.json")).unwrap(),
                b"preserve existing"
            );
        } else {
            original.initialize(directory, end).unwrap();
            if variant == "renewed-read" {
                assert!(original.read_data(end + Duration::from_nanos(1)).is_err());
            } else {
                assert!(original
                    .append_data(&encoded_data(&record()), Instant::now())
                    .is_err());
            }
            assert!(original.append_data(&encoded_data(&record()), end).is_err());
            assert!(original.files.is_empty());
        }
        assert!(original.failed);
        assert!(!scene.path().join("records/record-000001.json").exists());
        drop(original);
        drop(scene);
    }
}
