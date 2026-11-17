use super::query::*;
use crate::infrastructure::{hash, windows_official_runtime024 as native};
use std::{
    fs::OpenOptions,
    os::windows::fs::OpenOptionsExt,
    time::{Duration, Instant},
};

#[test]
fn r7_query_uses_fixed_product_policy_and_refuses_missing_unknown_duplicate_or_old_fields() {
    let source = include_str!("../../../../../../../apps/desktop/tauri/src/modules/runtime/windows_lifecycle024_contract.rs");
    for literal in [
        "pub const BOUNDED_SECURITY_SCOPE: &str = \"windows-bounded-access-v1\";",
        "pub const BOUNDED_SECURITY_QUERY_ACCESS: u32 = 0x0002_0000;",
        "pub const BOUNDED_SECURITY_INFORMATION: u32 = 0x0000_01f7;",
        "audit-sacl-and-unrequested-security-unknown",
    ] {
        assert!(source.contains(literal));
    }
    let fixed = Policy::fixed();
    assert_eq!(REQUIRED, [1, 2, 4, 16, 32, 64, 128, 256]);
    assert_eq!(
        fixed.digest().unwrap(),
        "688b50b892f40096c8add9263783bfec9102573f9a791a6883ad261d14437f07"
    );
    let value = serde_json::to_value(&fixed).unwrap();
    for key in [
        "scopeId",
        "queryAccess",
        "includedMask",
        "requiredProjections",
        "excludedAssurance",
    ] {
        let mut bad = value.clone();
        bad.as_object_mut().unwrap().remove(key);
        assert!(serde_json::from_value::<Policy>(bad).is_err());
    }
    for masks in [
        vec![1, 2, 4],
        vec![1, 2, 4, 16, 32, 64, 128, 128],
        vec![1, 2, 4, 16, 32, 64, 128, 256, 8],
    ] {
        let mut bad = value.clone();
        bad["requiredProjections"] = serde_json::json!(masks);
        assert!(serde_json::from_value::<Policy>(bad)
            .unwrap()
            .validate()
            .is_err());
    }
    let mut old = value.clone();
    old["scopeId"] = serde_json::json!("full-v1");
    assert!(serde_json::from_value::<Policy>(old)
        .unwrap()
        .validate()
        .is_err());
    let mut unknown = value.clone();
    unknown["allowPartial"] = serde_json::json!(true);
    assert!(serde_json::from_value::<Policy>(unknown).is_err());
    let duplicate =
        serde_json::to_string(&fixed)
            .unwrap()
            .replacen('{', "{\"queryAccess\":131072,", 1);
    assert!(serde_json::from_str::<Policy>(&duplicate).is_err());
}

#[test]
fn r7_same_native_handle_all_eight_repeated_queries_and_expiry_refusal() {
    let token = native::current_identity().unwrap();
    assert!(!token.elevated);
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("synthetic-original.txt");
    std::fs::write(&path, b"synthetic credential-free exact original\n").unwrap();
    let file = OpenOptions::new()
        .access_mode(0x80020000)
        .share_mode(7)
        .open(&path)
        .unwrap();
    let end = Instant::now() + Duration::from_secs(5);
    let a = native_stamp(&file, end).unwrap();
    let b = native_stamp(&file, end).unwrap();
    assert!(a == b);
    a.validate().unwrap();
    let value = serde_json::to_value(&a).unwrap();
    assert_eq!(value["projections"].as_array().unwrap().len(), 8);
    assert_eq!(value["auditSaclAssurance"], "excluded-unknown");
    assert_eq!(value["fullSecurityProven"], false);
    for mask in REQUIRED {
        assert_eq!(required_access(mask), 0x20000);
        let native_raw = query(&file, mask).unwrap();
        let individual = project(&native_raw, mask).unwrap();
        let combined = project(&query(&file, 0x1f7).unwrap(), mask).unwrap();
        assert!(same_covered_component(&individual, &combined));
    }
    let expired = native_stamp(&file, Instant::now() - Duration::from_millis(1))
        .err()
        .unwrap();
    assert_eq!(expired.api, "bounded-original-deadline");
    println!("same actual File: all8 individual/combined/repeated native queries; fixed policy; expired refused; ordinary SID={} type={} (controller fixture, not marenben acceptance)", token.sid, token.elevation_type);
}

#[test]
fn r7_native_exact_object_rename_roundtrip_preserves_projection_and_detects_replacement() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("original.txt");
    let bytes = b"synthetic same-volume original\n";
    std::fs::write(&path, bytes).unwrap();
    let read = |p: &std::path::Path| {
        OpenOptions::new()
            .access_mode(0x80020000)
            .share_mode(7)
            .open(p)
            .unwrap()
    };
    let before = native_stamp(&read(&path), Instant::now() + Duration::from_secs(5)).unwrap();
    let custody = native::create_private_directory(&root.path().join("custody")).unwrap();
    let end = Instant::now() + Duration::from_secs(10);
    let mut retained = native::RetainedFileMove::open(&path, end).unwrap();
    let exact_id = retained.identity();
    retained
        .move_into(custody.path(), "held-original", end)
        .unwrap();
    assert!(!path.exists());
    let held = read(&custody.path().join("held-original"));
    let moved = native_stamp(&held, end).unwrap();
    assert!(before == moved);
    retained.restore(end).unwrap();
    assert_eq!(retained.identity(), exact_id);
    assert!(native_stamp(&read(&path), end).unwrap() == before);
    assert_eq!(std::fs::read(&path).unwrap(), bytes);
    drop(held);
    drop(retained);
    // A new file with identical bytes is not the same native original.
    std::fs::rename(&path, root.path().join("retained-original.txt")).unwrap();
    std::fs::write(&path, bytes).unwrap();
    let replacement = native_stamp(&read(&path), end).unwrap();
    assert!(before != replacement);
    assert_eq!(hash::sha256_file(&path).unwrap(), hash::sha256_hex(bytes));
}
