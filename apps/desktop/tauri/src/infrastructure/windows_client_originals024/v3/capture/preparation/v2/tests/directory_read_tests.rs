use super::*;
use crate::infrastructure::windows_client_originals024::v3::capture::preparation::directory_read;
fn anchor() -> PartialCaptureAnchor {
    PartialCaptureAnchor::new(
        crate::infrastructure::hash::sha256_hex(
            b"actualdirectory preparation fixture, not nativeOwner",
        ),
        uuid::Uuid::new_v4(),
        crate::infrastructure::windows_official_lifecycle024::original_host_instance_for_process(),
    )
}
#[test]
fn wl2_r7_directory_read_native_all_create_prefixes_use_original_handles_without_absence_or_baseline_grant(
) {
    for fault in 1..=5 {
        let scene = Scene::new();
        let path = scene.path().join("actual-new-directory");
        let mut original = native::PrivateDirectoryPreparation::default();
        original.inject_directory_read_fixture_fault(fault).unwrap();
        let end = Instant::now() + Duration::from_secs(10);
        assert!(original.create(&path, end).is_err());
        let state = original.retained_stage_data();
        let first = directory_read::read(&original, end).unwrap();
        let again = directory_read::read(&original, end).unwrap();
        assert_eq!(first, again);
        assert_eq!(first["created"], fault >= 2);
        assert_eq!(first["terminalObserved"], fault >= 3);
        assert_eq!(first["pending"], true);
        assert_eq!(first["verifiedAbsent"], false);
        assert_eq!(first["securityDigest"], serde_json::Value::Null);
        assert_eq!(first["originalSecurityBaselineVerified"], false);
        if fault >= 3 {
            assert!(first["identityDigest"].is_string());
            assert!(first["contentDigest"].is_string());
            assert!(first["observedSecurityDigest"].is_string());
        } else {
            assert!(first["identityDigest"].is_null() && first["contentDigest"].is_null());
        }
        assert_eq!(original.retained_stage_data(), state);
        assert!(original.take_complete().is_err());
        let mut prefix = anchor();
        prefix.intend(path.clone()).unwrap();
        prefix.directory_prefixes.push(original);
        let (reference, details) = prefix.v2_reference_data(end).unwrap();
        assert!(reference.pending());
        let data = serde_json::to_value(reference).unwrap();
        assert_eq!(
            data["components"][1]["objects"][0]["identityDigest"],
            first["identityDigest"]
        );
        assert_eq!(
            data["components"][1]["objects"][0]["securityDigest"],
            serde_json::Value::Null
        );
        assert_eq!(
            details["objectMembership"]["nativeDirectoryReads"][0],
            first
        );
        assert_eq!(
            data["lastStage"],
            if fault >= 3 {
                "handleRetained"
            } else if fault >= 2 {
                "nativeCreated"
            } else {
                "intentPublished"
            }
        );
        assert!(!serde_json::to_string(&details)
            .unwrap()
            .contains(&path.to_string_lossy().to_string()));
        println!("r7 directoryread actualCreate fault{fault}: exact originalparent/terminal resources; currentSDdiagnostic only, no missingbaseline/Absent/recovery factory");
        drop(prefix);
        drop(scene);
    }
}
#[test]
fn wl2_r7_directory_read_native_unopened_created_path_never_adopts_replacement_or_unknown_members()
{
    let scene = Scene::new();
    let path = scene.path().join("created-nohandle");
    let old = scene.path().join("external-original");
    let end = Instant::now() + Duration::from_secs(10);
    let mut original = native::PrivateDirectoryPreparation::default();
    original.inject_directory_read_fixture_fault(2).unwrap();
    assert!(original.create(&path, end).is_err());
    std::fs::rename(&path, &old).unwrap();
    let replacement = native::create_private_directory(&path).unwrap();
    std::fs::write(path.join("do-not-adopt.bin"), b"foreign bytes preserved").unwrap();
    let data = directory_read::read(&original, end).unwrap();
    assert_eq!(data["created"], true);
    assert_eq!(data["terminalObserved"], false);
    assert_eq!(data["currentPathObservedWithoutOriginalHandle"], false);
    assert!(data["identityDigest"].is_null() && data["contentDigest"].is_null());
    assert!(original.take_complete().is_err());
    assert_eq!(
        std::fs::read(path.join("do-not-adopt.bin")).unwrap(),
        b"foreign bytes preserved"
    );
    assert!(old.is_dir());
    println!("r7 directoryread nativeunopened created path replaced: no terminalopen/memberenumeration/Absent or inferred permit; originalcreation gap remainsUnknown");
    drop(original);
    drop(replacement);
    drop(scene);
}
#[test]
fn wl2_r7_directory_read_native_transfer_existing_and_expired_reads_cannot_reactivate_creation() {
    let scene = Scene::new();
    let path = scene.path().join("transferred");
    let mut original = native::PrivateDirectoryPreparation::default();
    let end = Instant::now() + Duration::from_secs(10);
    original.create(&path, end).unwrap();
    let before = directory_read::read(&original, end).unwrap();
    assert_eq!(before["terminalObserved"], true);
    let pin = original.take_complete().unwrap();
    std::fs::write(
        path.join("later-consumer.bin"),
        b"actual later consumer bytes",
    )
    .unwrap();
    let data = directory_read::read(&original, end).unwrap();
    assert_eq!(data["transferred"], true);
    assert_eq!(data["terminalObserved"], false);
    assert!(data["identityDigest"].is_null());
    assert!(original.take_complete().is_err());
    assert_eq!(
        directory_read::read(&original, Instant::now())
            .unwrap_err()
            .kind(),
        io::ErrorKind::TimedOut
    );
    drop(pin);
    drop(original);
    let existing = scene.path().join("existing");
    let old = native::create_private_directory(&existing).unwrap();
    std::fs::write(existing.join("old.bin"), b"preserve").unwrap();
    let mut refused = native::PrivateDirectoryPreparation::default();
    assert!(refused.create(&existing, end).is_err());
    let read = directory_read::read(&refused, end).unwrap();
    assert_eq!(read["created"], false);
    assert_eq!(read["verifiedAbsent"], false);
    assert!(read["identityDigest"].is_null());
    assert_eq!(
        std::fs::read(existing.join("old.bin")).unwrap(),
        b"preserve"
    );
    println!("r7 directoryread transferred/existing/expired native prefix staysPending; no reopen/adopt/renew original creation");
    drop(refused);
    drop(old);
    drop(scene);
}
#[test]
fn wl2_r7_directory_read_native_unknown_and_late_member_security_or_ads_refuse() {
    for mode in ["member", "security", "ads", "late-member", "late-security"] {
        let scene = Scene::new();
        let path = scene.path().join("owned-new-empty");
        let mut original = native::PrivateDirectoryPreparation::default();
        original.inject_directory_read_fixture_fault(3).unwrap();
        let end = Instant::now() + Duration::from_secs(10);
        assert!(original.create(&path, end).is_err());
        match mode {
            "member" => std::fs::write(path.join("foreign.bin"), b"preserve foreign").unwrap(),
            "security" => change_dacl(&path),
            "ads" => std::fs::write(
                std::path::PathBuf::from(format!("{}:unknown", path.display())),
                b"unknown stream",
            )
            .unwrap(),
            "late-member" => directory_read::after_directory_for_test(|p| {
                std::fs::write(p.join("late.bin"), b"late foreign").unwrap()
            }),
            "late-security" => directory_read::after_directory_for_test(change_dacl),
            _ => unreachable!(),
        }
        let result = directory_read::read(&original, end);
        assert!(result.is_err(), "{mode}");
        assert!(original.take_complete().is_err());
        println!("r7 directoryread native refuses {mode}: raw={:?} kind={:?}; exact own prefix/resources retained, no repair",result.as_ref().unwrap_err().raw_os_error(),result.as_ref().unwrap_err().kind());
        drop(original);
        drop(scene);
    }
}
#[test]
fn wl2_r7_directory_read_native_stable_custody_labels_do_not_shift_when_first_role_arrives() {
    let scene = Scene::new();
    let mut prefix = anchor();
    let end = Instant::now() + Duration::from_secs(10);
    let grok = native::create_private_directory(&scene.path().join("grok-custody")).unwrap();
    prefix.custody[1] = Some(grok);
    let path = scene.path().join("directory-prefix");
    prefix.intend(path.clone()).unwrap();
    let pin = prefix.create_directory(&path, end).unwrap();
    prefix.pins.push(pin);
    let (first, _) = prefix.v2_reference_data(end).unwrap();
    let first = serde_json::to_value(first).unwrap();
    let grok_id = first["components"][1]["objects"][0]["objectId"].clone();
    let directory_id = first["components"][1]["objects"][1]["objectId"].clone();
    let source_pin = first["components"][0]["objects"][0]["objectId"].clone();
    prefix.custody[0] =
        Some(native::create_private_directory(&scene.path().join("codex-custody")).unwrap());
    let (second, _) = prefix.v2_reference_data(end).unwrap();
    assert!(second.pending());
    let second = serde_json::to_value(second).unwrap();
    assert_eq!(second["components"][1]["objects"][1]["objectId"], grok_id);
    assert_eq!(
        second["components"][1]["objects"][2]["objectId"],
        directory_id
    );
    assert_eq!(
        second["components"][0]["objects"][0]["objectId"],
        source_pin
    );
    assert_ne!(grok_id, directory_id);
    assert_ne!(second["components"][1]["objects"][0]["objectId"], grok_id);
    println!("r7 directoryread actualcustody/nativeprefix transfer stablelabels: codex0/grok1/directory16+/sourcepins16+; missing slot never compacts earlier IDs; no nativeOwner constructed");
    drop(prefix);
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
    let path = path
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
    let result = unsafe { SetFileSecurityW(path.as_ptr(), 4, sd) };
    unsafe {
        windows_sys::Win32::Foundation::LocalFree(sd);
    }
    assert_ne!(result, 0);
}
