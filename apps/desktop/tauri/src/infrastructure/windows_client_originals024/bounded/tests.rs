use super::*;
#[test]
fn wl2_r4_policy_is_fixed_no_full_or_partial_conversion() {
    let fixed = Policy::fixed();
    assert_eq!(REQUIRED, [1, 2, 4, 16, 32, 64, 128, 256]);
    assert_eq!(REQUIRED.iter().fold(0, |a, x| a | x), 0x1f7);
    let original = serde_json::to_value(&fixed).unwrap();
    for key in [
        "scopeId",
        "queryAccess",
        "includedMask",
        "requiredProjections",
        "excludedAssurance",
    ] {
        let mut bad = original.clone();
        bad.as_object_mut().unwrap().remove(key);
        assert!(serde_json::from_value::<Policy>(bad).is_err());
    }
    for key in ["scopeId", "excludedAssurance"] {
        let mut bad = original.clone();
        bad[key] = serde_json::json!("full-v1");
        assert!(serde_json::from_value::<Policy>(bad)
            .unwrap()
            .validate()
            .is_err());
    }
    for masks in [
        vec![1, 2, 4],
        vec![1, 2, 4, 16, 32, 64, 128, 128],
        vec![2, 1, 4, 16, 32, 64, 128, 256],
        vec![1, 2, 4, 16, 32, 64, 128, 256, 8],
    ] {
        let mut bad = original.clone();
        bad["requiredProjections"] = serde_json::json!(masks);
        assert!(serde_json::from_value::<Policy>(bad)
            .unwrap()
            .validate()
            .is_err());
    }
    let raw = serde_json::to_string(&fixed).unwrap();
    let duplicate = raw.replacen('{', "{\"queryAccess\":131072,", 1);
    assert!(serde_json::from_str::<Policy>(&duplicate).is_err());
    assert_eq!(fixed.digest().unwrap(), Policy::fixed().digest().unwrap());
}
fn empty_sd() -> Vec<u8> {
    let mut b = vec![0u8; 20];
    b[0] = 1;
    b[2..4].copy_from_slice(&0x8000u16.to_le_bytes());
    b
}
#[test]
fn wl2_r4_projection_structures_refuse_unknown_bounds_and_compare_only_covered_control() {
    let b = empty_sd();
    assert_eq!(project(&b, 4).unwrap().state, "absent");
    let mut null = b.clone();
    null[2..4].copy_from_slice(&0x8004u16.to_le_bytes());
    assert_eq!(project(&null, 4).unwrap().state, "null");
    let mut empty = null.clone();
    empty[16..20].copy_from_slice(&20u32.to_le_bytes());
    empty.extend_from_slice(&[2, 0, 8, 0, 0, 0, 0, 0]);
    assert_eq!(project(&empty, 4).unwrap().state, "empty");
    let mut audit_control = empty.clone();
    audit_control[2..4].copy_from_slice(&0xa834u16.to_le_bytes());
    assert!(project(&audit_control, 4).unwrap() == project(&empty, 4).unwrap());
    let mut dacl_control = empty.clone();
    dacl_control[2..4].copy_from_slice(&0x9004u16.to_le_bytes());
    assert!(project(&dacl_control, 4).unwrap() != project(&empty, 4).unwrap());
    for mask in [0, 8, 0x10000, 0x1f7] {
        assert!(project(&empty, mask).is_err());
    }
    for bad in [
        vec![],
        vec![0; 20],
        {
            let mut x = empty.clone();
            x[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
            x
        },
        {
            let mut x = empty.clone();
            x[20] = 99;
            x
        },
    ] {
        assert!(project(&bad, 4).is_err());
    }
}
#[test]
fn wl2_r4_native_all_projection_queries_and_ordinary_token() {
    let token = current_identity().unwrap();
    assert!(!token.elevated);
    let t = tempfile::tempdir().unwrap();
    let path = t.path().join("synthetic-bounded-security");
    std::fs::write(&path, b"synthetic no credential").unwrap();
    let file = OpenOptions::new()
        .access_mode(BOUNDED_SECURITY_QUERY_ACCESS)
        .share_mode(1)
        .open(&path)
        .unwrap();
    let deadline = Instant::now() + std::time::Duration::from_secs(5);
    let mut failed = Vec::new();
    for mask in REQUIRED.into_iter().chain([BOUNDED_SECURITY_INFORMATION]) {
        let access = required_access(mask);
        match query(&file, mask) {
            Ok(raw) => {
                let parsed: Result<Vec<_>, _> = if mask == BOUNDED_SECURITY_INFORMATION {
                    REQUIRED.iter().map(|p| project(&raw, *p)).collect()
                } else {
                    project(&raw, mask).map(|x| vec![x])
                };
                match parsed {Ok(p)=>println!("r4native mask=0x{mask:x} access=0x{access:x} raw0 states={:?} sdHash={} fullSecurityProven=false audit=excluded-unknown",p.iter().map(|x|(&x.mask,&x.state,x.aces.len())).collect::<Vec<_>>(),hash::sha256_hex(&raw)),Err(e)=>{println!("r4native mask=0x{mask:x} structureUnknown {e}");failed.push(mask);}}
            }
            Err(e) => {
                println!("r4native {e}");
                failed.push(mask);
            }
        }
    }
    assert!(
        failed.is_empty(),
        "required masks refused {failed:?}; no fallback"
    );
    drop(file);
    match BoundedAccessV1::capture(&path, false, deadline) {
        Ok(proof) => {
            proof.recheck(deadline).unwrap();
            println!(
                "r4native ordinaryToken sid={} session={} elevated={} {}",
                token.sid,
                token.session_id,
                token.elevated,
                proof.diagnostic()
            );
        }
        Err(e) => panic!("r4native capture requiredUnknown: {e}"),
    }
}

#[test]
fn wl2_r4_mixed_policy_aces_do_not_expand_sacl_global_assurance() {
    let absent = project(&empty_sd(), 32).unwrap();
    let mut combined = empty_sd();
    combined[2..4].copy_from_slice(&0x8010u16.to_le_bytes());
    combined[12..16].copy_from_slice(&20u32.to_le_bytes());
    // A native-layout mandatory label ACE with S-1-16-4096, mask NO_WRITE_UP.
    let mut ace = vec![17, 0, 20, 0, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 16];
    ace.extend_from_slice(&4096u32.to_le_bytes());
    combined.extend_from_slice(&[2, 0, 28, 0, 1, 0, 0, 0]);
    combined.extend_from_slice(&ace);
    let resource = project(&combined, 32).unwrap();
    assert!(same_covered_component(&resource, &absent));
    let label = project(&combined, 16).unwrap();
    assert_eq!(label.aces.len(), 1);
    label.validate().unwrap();
    assert!(!same_covered_component(
        &label,
        &project(&empty_sd(), 16).unwrap()
    ));
    let mut changed = combined.clone();
    *changed.last_mut().unwrap() = 1;
    assert!(!same_covered_component(
        &label,
        &project(&changed, 16).unwrap()
    ));
}
#[test]
fn wl2_r4_native_explicit_low_integrity_fixture_is_read_and_policy_bound() {
    use std::os::windows::{ffi::OsStrExt, io::FromRawHandle};
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
            text: *const u16,
            revision: u32,
            sd: *mut *mut std::ffi::c_void,
            size: *mut u32,
        ) -> i32;
    }
    let token = current_identity().unwrap();
    assert!(!token.elevated);
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("new-low-label-only");
    let sddl = format!(
        "O:{}D:P(A;;FA;;;{})(A;;FA;;;SY)S:(ML;;NW;;;LW)",
        token.sid, token.sid
    );
    let text: Vec<u16> = sddl.encode_utf16().chain(Some(0)).collect();
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
    struct Local(*mut std::ffi::c_void);
    impl Drop for Local {
        fn drop(&mut self) {
            unsafe {
                LocalFree(self.0);
            }
        }
    }
    let _sd = Local(sd);
    let sa = windows_sys::Win32::Security::SECURITY_ATTRIBUTES {
        nLength: std::mem::size_of::<windows_sys::Win32::Security::SECURITY_ATTRIBUTES>() as u32,
        lpSecurityDescriptor: sd,
        bInheritHandle: 0,
    };
    let name: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let h = unsafe {
        windows_sys::Win32::Storage::FileSystem::CreateFileW(
            name.as_ptr(),
            0x20000,
            1,
            &sa,
            1,
            0x200000,
            std::ptr::null_mut(),
        )
    };
    assert_ne!(
        h,
        windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE,
        "new task fixture creation native={:?}",
        io::Error::last_os_error()
    );
    drop(unsafe { File::from_raw_handle(h) });
    let deadline = Instant::now() + std::time::Duration::from_secs(5);
    let captured = BoundedAccessV1::capture(&path, false, deadline).unwrap();
    let label = captured.projections.iter().find(|p| p.mask == 16).unwrap();
    assert!(
        !label.aces.is_empty(),
        "explicit label fixture must provide nonempty native positive"
    );
    captured.recheck(deadline).unwrap();
    captured.snapshot_data().unwrap().validate().unwrap();
    println!("r4native explicit file-label positive: {} ; resource/CAP/trust/filter populated positives remain unverified",captured.diagnostic());
    drop(captured);
}
