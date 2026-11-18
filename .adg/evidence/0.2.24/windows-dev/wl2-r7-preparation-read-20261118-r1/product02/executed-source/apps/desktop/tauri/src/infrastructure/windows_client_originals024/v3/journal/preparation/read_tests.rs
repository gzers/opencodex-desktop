use super::*;
#[test]
fn wl2_r7_preparation_read_native_every_original_initializer_prefix_preserves_pending_and_bytes() {
    for fault in [1, 2, 3, 4, 5, 6, 7, 8, 10, 11] {
        let scene = Scene::new();
        let (mut original, path) = scene.prefix(fault);
        let end = Instant::now() + Duration::from_secs(10);
        assert!(original
            .initialize(
                &uuid::Uuid::new_v4().to_string(),
                &"c".repeat(64),
                &uuid::Uuid::new_v4().to_string(),
                end
            )
            .is_err());
        let before = std::fs::read_dir(&path)
            .unwrap()
            .map(|p| {
                let p = p.unwrap().path();
                (
                    p.file_name().unwrap().to_os_string(),
                    std::fs::read(&p).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        let failed = original.failed;
        let ready = original.ready;
        let started = original.initialization_started;
        let data = original.read_v2_retained_native(end).unwrap();
        assert_eq!(data["pending"], true);
        assert_eq!(data["runtimeAuthority"], false);
        let rows = data["objects"].as_array().unwrap();
        assert_eq!(rows.len(), before.len());
        if fault == 2 {
            assert_eq!(rows[0]["writeState"], "empty");
            assert_eq!(rows[0]["originalSecurityBaselineVerified"], false);
        }
        if fault == 3 {
            assert_eq!(rows[0]["writeState"], "completeBytes");
            assert_eq!(rows[0]["originalSecurityBaselineVerified"], false);
        }
        if fault == 10 {
            assert_eq!(rows[0]["writeState"], "partialBytes");
            assert_eq!(rows[0]["originalSecurityBaselineVerified"], false);
        }
        if fault == 11 {
            assert_eq!(rows[1]["writeState"], "partialBytes");
            assert_eq!(rows[1]["originalSecurityBaselineVerified"], false);
        }
        let session = uuid::Uuid::new_v4();
        let (objects, again) = original.v2_native_objects(session, end).unwrap();
        assert_eq!(objects.len(), rows.len() + 1);
        assert_eq!(again, data);
        let encoded = serde_json::to_value(&objects).unwrap();
        assert!(encoded
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["securityDigest"].is_null()));
        if matches!(fault, 2 | 3 | 10) {
            assert!(encoded[0]["securityDigest"].is_null());
        }
        assert_eq!(
            (
                original.failed,
                original.ready,
                original.initialization_started
            ),
            (failed, ready, started)
        );
        assert!(original.take_complete().is_err());
        let after = std::fs::read_dir(&path)
            .unwrap()
            .map(|p| {
                let p = p.unwrap().path();
                (
                    p.file_name().unwrap().to_os_string(),
                    std::fs::read(&p).unwrap(),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(before, after);
        assert!(!serde_json::to_string(&data)
            .unwrap()
            .contains(&path.to_string_lossy().to_string()));
        for (file, _) in &before {
            assert!(OpenOptions::new()
                .write(true)
                .open(path.join(file))
                .is_err());
            assert!(std::fs::remove_file(path.join(file)).is_err());
        }
        println!("r7 preparationread actualjournal fault{fault}: retained files={} originalbytes/IDs/all8/ADS/memberbeforeafter; rawprefix Pending unchanged, no Source/Owner/restore permission",rows.len());
        drop(original);
        drop(scene);
    }
}
#[test]
fn wl2_r7_preparation_read_native_mismatch_member_security_ads_and_cutoff_refuse() {
    for mode in [
        "member",
        "file-security",
        "directory-security",
        "ads",
        "wrong-prefix",
        "expired",
        "unregistered",
        "late-member",
        "late-file-security",
    ] {
        let scene = Scene::new();
        let (mut original, path) = scene.prefix(3);
        let end = Instant::now() + Duration::from_secs(10);
        if mode != "unregistered" {
            assert!(original
                .initialize(
                    &uuid::Uuid::new_v4().to_string(),
                    &"c".repeat(64),
                    &uuid::Uuid::new_v4().to_string(),
                    end
                )
                .is_err());
        }
        match mode {
            "member" => std::fs::write(path.join("foreign.json"), b"preserve unknown").unwrap(),
            "file-security" => change_dacl(&path.join("manifest.json")),
            "directory-security" => change_dacl(&path),
            "ads" => std::fs::write(
                PathBuf::from(format!("{}:unknown", path.display())),
                b"retained unknown ADS",
            )
            .unwrap(),
            "wrong-prefix" => {
                original
                    .manifest_open
                    .as_mut()
                    .unwrap()
                    .seek(SeekFrom::Start(0))
                    .unwrap();
                original
                    .manifest_open
                    .as_mut()
                    .unwrap()
                    .write_all(b"!")
                    .unwrap();
                original.manifest_open.as_ref().unwrap().sync_all().unwrap();
            }
            "unregistered" => {
                original.attempted = false;
            }
            "late-member" => super::super::read::after_rows_for_test(|directory| {
                std::fs::write(directory.join("late-member.json"), b"preserve unknown").unwrap();
            }),
            "late-file-security" => super::super::read::after_rows_for_test(|directory| {
                change_dacl(&directory.join("manifest.json"))
            }),
            _ => {}
        }
        let read_end = if mode == "expired" {
            Instant::now()
        } else {
            end
        };
        let result = original.read_v2_retained_native(read_end);
        assert!(result.is_err(), "{mode}");
        assert!(original.failed && !original.ready);
        assert!(original.take_complete().is_err());
        println!("r7 preparationread actualjournal refuses {mode}: raw={:?} kind={:?}, original resources retained/no repair",result.as_ref().unwrap_err().raw_os_error(),result.as_ref().unwrap_err().kind());
        drop(original);
        drop(scene);
    }
}
#[test]
fn wl2_r7_preparation_read_native_expired_start_prefix_reads_without_mutation_reactivation() {
    let scene = Scene::new();
    let (mut original, path) = scene.prefix(10);
    let old_end = Instant::now() + Duration::from_millis(500);
    assert!(original
        .initialize(
            &uuid::Uuid::new_v4().to_string(),
            &"c".repeat(64),
            &uuid::Uuid::new_v4().to_string(),
            old_end
        )
        .is_err());
    let before = std::fs::read(path.join("manifest.json")).unwrap();
    while Instant::now() < old_end {
        std::thread::sleep(Duration::from_millis(1));
    }
    let read_end = Instant::now() + Duration::from_secs(5);
    let observed = original.read_v2_retained_native(read_end).unwrap();
    assert_eq!(observed["objects"][0]["writeState"], "partialBytes");
    assert_eq!(observed["pending"], true);
    assert!(original.failed && !original.ready && original.initialization_started);
    assert!(original.take_complete().is_err());
    assert!(original
        .initialize(
            &uuid::Uuid::new_v4().to_string(),
            &"c".repeat(64),
            &uuid::Uuid::new_v4().to_string(),
            read_end
        )
        .is_err());
    assert_eq!(std::fs::read(path.join("manifest.json")).unwrap(), before);
    assert!(!path.join("current.json").exists());
    println!("r7 preparationread expired originalStart: same actual halfmanifest bytes read in caller5, no oldmutationreactivation/newcurrent/nativeproof");
    drop(original);
    drop(scene);
}
fn change_dacl(path: &Path) {
    use std::{ffi::c_void, os::windows::ffi::OsStrExt};
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
            text: *const u16,
            revision: u32,
            descriptor: *mut *mut c_void,
            size: *mut u32,
        ) -> i32;
        fn SetFileSecurityW(path: *const u16, information: u32, descriptor: *const c_void) -> i32;
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
    assert_ne!(
        result, 0,
        "only exact own synthetic negative directory/file"
    );
}
