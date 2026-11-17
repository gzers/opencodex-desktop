use super::*;
use serde_json::{json, Value};
fn reference() -> Value {
    json!({"kind":"canonical","manifestDigest":"a".repeat(64),"bindingDigest":"b".repeat(64),"storagePlanDigest":"c".repeat(64),"failedPrefixDigest":"d".repeat(64)})
}
fn preparation(complete: bool) -> Value {
    let components=["source","custody","codex","grok","journal"].into_iter().enumerate().map(|(i,role)|{
        if complete {json!({"role":role,"state":"retained","prefixDigest":format!("{i:064x}"),"objects":[{"objectId":format!("{:064x}",i+10),"role":role,"originalStage":"handleRetained","pathDigest":format!("{:064x}",i+20),"identityDigest":format!("{:064x}",i+30),"securityDigest":format!("{:064x}",i+40),"contentDigest":format!("{:064x}",i+50)}]})}
        else if i==1{json!({"role":role,"state":"retained","prefixDigest":"e".repeat(64),"objects":[{"objectId":"f".repeat(64),"role":role,"originalStage":"nativeCreated","pathDigest":"a".repeat(64),"identityDigest":null,"securityDigest":null,"contentDigest":null}]})}
        else{json!({"role":role,"state":"unobserved","prefixDigest":null,"objects":[]})}
    }).collect::<Vec<_>>();
    json!({"kind":"preparation","ownerInstanceId":"44444444-4444-4444-4444-444444444444","source":if complete{json!({"state":"captured","bindingDigest":"b".repeat(64),"imageDigest":"c".repeat(64)})}else{json!({"state":"unobserved"})},"intentPrefixDigest":"a".repeat(64),"nativeStagePrefixDigest":"b".repeat(64),"lastStage":"handleRetained","components":components})
}
fn record_with(reference: Value) -> Value {
    json!({"schema":SCHEMA,"sessionId":"11111111-1111-1111-1111-111111111111","failedActionId":"22222222-2222-2222-2222-222222222222","recoveryActionId":"33333333-3333-3333-3333-333333333333","securityPolicyDigest":"e".repeat(64),"reference":reference,"sequence":1,"previousRecordDigest":null,"startedQpc":"1000","deadlineQpc":"11000","qpcFrequency":"1000","event":{"kind":"AbortRecoveryIntent"}})
}
fn encoded_data(v: &Value) -> Vec<u8> {
    serde_json::to_vec(v).unwrap()
}
fn raw_chain(reference: Value, events: &[Value]) -> Vec<Vec<u8>> {
    let mut result: Vec<Vec<u8>> = Vec::new();
    for (i, event) in events.iter().enumerate() {
        let mut r = record_with(reference.clone());
        r["event"] = event.clone();
        r["sequence"] = json!(i + 1);
        if let Some(previous) = result.last() {
            r["previousRecordDigest"] = json!(crate::infrastructure::hash::sha256_hex(previous));
        }
        result.push(encoded_data(&r));
    }
    result
}
fn quiescent() -> Value {
    json!({"kind":"LauncherQuiesced","pid":123,"creationFiletime":"42","forced":true,"rawExit":242,"jobTotal":1,"jobActive":0})
}
fn terminal(nodes: u32) -> Value {
    json!({"kind":"AbortRestoreVerified","originalNodes":nodes,"coordinationObjects":3,"normalStop":false,"closed":false,"publicStartOpen":false})
}
fn prep_events(reference: &Value) -> Vec<Value> {
    let mut result = vec![json!({"kind":"AbortRecoveryIntent"}), quiescent()];
    for object in reference["components"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|c| c["objects"].as_array().unwrap())
    {
        for kind in ["PreparationRetainIntent", "PreparationRetainedVerified"] {
            result.push(json!({"kind":kind,"objectId":object["objectId"],"role":object["role"],"originalStage":object["originalStage"],"identityDigest":object["identityDigest"],"retainedMembershipDigest":"f".repeat(64)}));
        }
    }
    result
}
fn record() -> Value {
    record_with(reference())
}
fn chain(events: &[Value]) -> Vec<Vec<u8>> {
    raw_chain(reference(), events)
}
fn events() -> Vec<Value> {
    vec![
        json!({"kind":"AbortRecoveryIntent"}),
        json!({"kind":"LauncherQuiesced","pid":123,"creationFiletime":"42","forced":true,"rawExit":242,"jobTotal":1,"jobActive":0}),
        json!({"kind":"WorkRetireIntent","nodeId":"f".repeat(64),"workFileId":"10","parentId":"20"}),
        json!({"kind":"WorkRetiredVerified","nodeId":"f".repeat(64),"workFileId":"10","parentId":"20"}),
        json!({"kind":"OriginalReturnIntent","nodeId":"f".repeat(64),"originalFileId":"30","parentId":"20"}),
        json!({"kind":"OriginalReturnedVerified","nodeId":"f".repeat(64),"originalFileId":"30","parentId":"20"}),
        json!({"kind":"AbortRestoreVerified","originalNodes":10,"coordinationObjects":3,"normalStop":false,"closed":false,"publicStartOpen":false}),
    ]
}

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
    fn initialized(&self, end: Instant) -> AttachmentStorageV2 {
        let mut original = AttachmentStorageV2::default();
        original.initialize(self.directory(), end).unwrap();
        original
    }
}
impl Drop for Scene {
    fn drop(&mut self) {
        if let Some(temp) = self.0.take() {
            if std::thread::panicking() {
                println!(
                    "r7-v2-attachment-failed-fixture-retained={}",
                    temp.keep().display()
                );
            } else {
                let path = temp.path().to_path_buf();
                temp.close().unwrap();
                assert!(!path.exists());
                println!("r7-v2-attachment-cleanup-absent={}", path.display());
            }
        }
    }
}
fn end() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
#[test]
fn wl2_r7_v2_storage_native_preparation_pending_and_complete_declared_members_remain_data() {
    for mode in ["complete-data", "pending-data", "unknown-terminal-refused"] {
        let scene = Scene::new();
        let end = end();
        let mut original = scene.initialized(end);
        let complete = mode == "complete-data";
        let reference = preparation(complete);
        let mut events = if complete {
            prep_events(&reference)
        } else {
            vec![json!({"kind":"AbortRecoveryIntent"}), quiescent()]
        };
        for bytes in raw_chain(reference.clone(), &events) {
            original.append_data(&bytes, end).unwrap();
        }
        let prefix = original.read_data(end).unwrap();
        events.push(if mode == "pending-data" {
            json!({"kind":"AbortRecoveryPending","stage":"nativeCreated","rawError":null})
        } else {
            terminal(if complete { 2 } else { 0 })
        });
        let full = raw_chain(reference, &events);
        let result = original.append_data(full.last().unwrap(), end);
        if mode == "unknown-terminal-refused" {
            assert!(result.is_err());
            assert!(original.failed && original.pending_file.is_none());
            assert_eq!(original.files.len(), prefix.len());
            assert!(!scene.path().join("records/v2-record-000003.json").exists());
            for (stored, previous) in original.files.iter().zip(&prefix) {
                assert_eq!(std::fs::read(&stored.path).unwrap(), *previous);
            }
        } else {
            result.unwrap();
            assert_eq!(original.read_data(end).unwrap(), full);
            assert_eq!(original.files.len(), if complete { 13 } else { 3 });
        }
        println!("r7-v2 native DATA persistence {mode}; synthetic declared IDs are DATA only, no original Owner/claim/quiescence/restore constructed");
        drop(original);
        drop(scene);
    }
}
#[test]
fn wl2_r7_v2_storage_native_complete_strict_chain_exact_original_files_and_no_authority() {
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
fn wl2_r7_v2_storage_native_every_empty_half_full_or_published_failure_keeps_exact_prefix() {
    let raw = encoded_data(&record());
    for stage in 1..=5 {
        let scene = Scene::new();
        let end = end();
        let mut original = scene.initialized(end);
        original.fault = stage;
        assert!(original.append_data(&raw, end).is_err());
        assert!(original.failed && !original.ready);
        let path = scene.path().join("records/v2-record-000001.json");
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
fn wl2_r7_v2_storage_native_cross_scope_replay_unknown_and_wrong_sequence_refuse_before_write() {
    let full = chain(&events());
    for variant in [
        "replay",
        "scope",
        "unknown",
        "gap",
        "v1",
        "reference",
        "preparation",
    ] {
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
            "v1" => {
                v["schema"] = json!("windows-before-resume-recovery-v1");
                encoded_data(&v)
            }
            "reference" => {
                v["reference"]["manifestDigest"] = json!("0".repeat(64));
                encoded_data(&v)
            }
            "preparation" => {
                v["reference"] = preparation(false);
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
        assert!(!scene.path().join("records/v2-record-000002.json").exists());
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
fn wl2_r7_v2_storage_native_member_file_security_directory_security_or_ads_drift_refuse() {
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
                    Some(|directory| change_dacl(&directory.join("v2-record-000001.json")));
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
fn wl2_r7_v2_storage_native_expired_renewed_or_existing_namespace_never_adopts() {
    for variant in ["expired-init", "renewed-read", "expired-append", "existing"] {
        let scene = Scene::new();
        let directory = scene.directory();
        let end = end();
        let mut original = AttachmentStorageV2::default();
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
        assert!(!scene.path().join("records/v2-record-000001.json").exists());
        drop(original);
        drop(scene);
    }
}
