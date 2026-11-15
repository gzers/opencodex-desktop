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
