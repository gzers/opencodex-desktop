use super::*;
#[test]
fn wl2_r7_preparation_read_native_actual_slot_v2_maps_all_original_journal_prefixes_pending() {
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
        let rows = original.read_v2_retained_native(end).unwrap();
        let mut slot =
            crate::infrastructure::windows_client_originals024::v3::CaptureSlot::default();
        let session = uuid::Uuid::new_v4();
        let host=crate::infrastructure::windows_official_lifecycle024::original_host_instance_for_process();
        slot.component_enroll_journal_prefix_for_test(
            hash::sha256_hex(b"actual retained component scope DATA, not productOwner"),
            session,
            host,
            original,
        )
        .unwrap();
        let (reference, detail) = slot.component_read_v2_preparation_for_test(end).unwrap();
        let (again, _) = slot.component_read_v2_preparation_for_test(end).unwrap();
        assert_eq!(reference, again);
        assert_eq!(reference["kind"], "preparation");
        assert_eq!(reference["ownerInstanceId"], host.to_string());
        assert_eq!(reference["source"]["state"], "unobserved");
        assert_eq!(detail["sessionId"], session.to_string());
        assert_eq!(detail["objectMembership"]["nativeJournalRead"], rows);
        assert_eq!(reference["components"][4]["state"], "retained");
        assert_eq!(
            reference["components"][4]["objects"]
                .as_array()
                .unwrap()
                .len(),
            rows["objects"].as_array().unwrap().len() + 1
        );
        let expected_stage = match fault {
            8 => "currentPublished",
            4 => "journalPublished",
            _ => "handleRetained",
        };
        assert_eq!(
            reference["lastStage"], expected_stage,
            "actual enrolled journal stage{fault}"
        );
        assert!(reference["components"][4]["objects"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| row["securityDigest"].is_null()));
        assert!(
            reference.get("manifestDigest").is_none()
                && reference.get("storagePlanDigest").is_none()
        );
        assert!(!serde_json::to_string(&reference)
            .unwrap()
            .contains(&path.to_string_lossy().to_string()));
        assert_eq!(detail["runtimeAuthority"], false);
        println!("r7 preparationread actualslot fault{fault}: exact originaljournal nativeobjects+session/host -> strictV2 {expected_stage}, missing Source/all8baseline staysPending/no nativefactory");
        drop(slot);
        drop(scene);
    }
}
#[test]
fn wl2_r7_preparation_read_native_every_original_initializer_prefix_preserves_pending_and_bytes() {
    // One DATA session is deliberately reused only to compare stable V2 object
    // labels across independent native fixtures; never a native Owner/session
    // factory or authority to combine their objects into one recovery.
    let data_session = uuid::Uuid::new_v4();
    let mut aggregate_id = None;
    let mut manifest_id = None;
    let mut current_id = None;
    for fault in [1, 2, 3, 4, 5, 6, 7, 8, 10, 11] {
        let scene = Scene::new();
        let (mut original, path) = scene.prefix(fault);
        let end = Instant::now() + Duration::from_secs(10);
        let (enrolled, _) = original.v2_native_objects(data_session, end).unwrap();
        let enrolled = serde_json::to_value(&enrolled[0]).unwrap();
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
        let (objects, again) = original.v2_native_objects(data_session, end).unwrap();
        assert_eq!(objects.len(), rows.len() + 1);
        assert_eq!(again, data);
        let encoded = serde_json::to_value(&objects).unwrap();
        let aggregate = encoded.as_array().unwrap().last().unwrap();
        assert_eq!(aggregate["identityDigest"], enrolled["identityDigest"]);
        assert_eq!(
            aggregate["pathDigest"], enrolled["pathDigest"],
            "same original directory path must stay stable at initializer fault {fault}"
        );
        let observed_aggregate = aggregate["objectId"].clone();
        if let Some(expected) = &aggregate_id {
            assert_eq!(
                *expected, observed_aggregate,
                "same aggregate DATA label must not drift with enrolled file count"
            );
        } else {
            aggregate_id = Some(observed_aggregate);
        }
        for (row, object) in rows.iter().zip(encoded.as_array().unwrap()) {
            let target = if row["objectSlot"] == "manifest" {
                &mut manifest_id
            } else {
                &mut current_id
            };
            if let Some(expected) = target.as_ref() {
                assert_eq!(*expected, object["objectId"]);
            } else {
                *target = Some(object["objectId"].clone());
            }
        }
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
fn wl2_r7_preparation_read_native_completed_journal_rejects_late_directory_stream() {
    let scene = Scene::new();
    let (mut original, path) = scene.prefix(8);
    let end = Instant::now() + Duration::from_secs(10);
    assert!(original
        .initialize(
            &uuid::Uuid::new_v4().to_string(),
            &"c".repeat(64),
            &uuid::Uuid::new_v4().to_string(),
            end,
        )
        .is_err());
    let before = original.read_v2_retained_native(end).unwrap();
    assert_eq!(before["directory"]["fullInitialJournal"], true);
    assert_eq!(before["objects"].as_array().unwrap().len(), 2);
    super::super::read::after_rows_for_test(|directory| {
        std::fs::write(
            PathBuf::from(format!("{}:fixture-stream", directory.display())),
            b"synthetic stream",
        )
        .unwrap();
    });
    let result = original.read_v2_retained_native(end);
    let error = result.expect_err("completed journal must reject a late directory stream");
    assert_eq!(error.kind(), io::ErrorKind::InvalidData);
    assert!(original.failed && !original.ready);
    assert!(original.take_complete().is_err());
    assert!(original.completed.is_some());
    assert_eq!(
        std::fs::read(PathBuf::from(format!("{}:fixture-stream", path.display()))).unwrap(),
        b"synthetic stream"
    );
    for name in ["manifest.json", "current.json"] {
        assert!(OpenOptions::new().write(true).open(path.join(name)).is_err());
    }
    println!("completed journal late directory stream: refused {:?}, both original handles retained", error.kind());
    drop(original);
    drop(scene);
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
