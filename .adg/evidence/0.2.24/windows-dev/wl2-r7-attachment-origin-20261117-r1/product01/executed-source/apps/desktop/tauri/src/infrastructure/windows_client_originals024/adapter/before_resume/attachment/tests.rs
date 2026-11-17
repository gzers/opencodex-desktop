use super::*;
fn prefix() -> PreparedAbortPrefix {
    let observed = serde_json::json!({"canonical":{"sessionId":"11111111-1111-1111-1111-111111111111","manifestDigest":"a".repeat(64),"bindingDigest":"b".repeat(64),"storagePlanDigest":"c".repeat(64),"securityPolicyDigest":"d".repeat(64)},"runtimeAuthority":false});
    PreparedAbortPrefix {
        session: uuid::Uuid::new_v4(),
        scope: "synthetic DATA comparison not original Owner authority".into(),
        action: "33333333-3333-3333-3333-333333333333".into(),
        end: Instant::now() + Duration::from_secs(10),
        epoch: "epoch".into(),
        claim: "claim".into(),
        digest: data_digest(&observed).unwrap(),
        observed,
    }
}
fn clock() -> [String; 3] {
    ["1000", "11000", "1000"].map(String::from)
}
#[test]
fn wl2_r7_attachment_origin_data_projection_preserves_canonical_uuid_all_hashes_failed_action_and_fixed_clock(
) {
    let p = prefix();
    let record = initial_data(&p, "22222222-2222-2222-2222-222222222222".into(), clock()).unwrap();
    let raw = record.encode_data().unwrap();
    let actual: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(actual["sessionId"], p.observed["canonical"]["sessionId"]);
    assert_ne!(actual["sessionId"], p.session.to_string());
    for name in [
        "manifestDigest",
        "bindingDigest",
        "storagePlanDigest",
        "securityPolicyDigest",
    ] {
        assert_eq!(actual[name], p.observed["canonical"][name]);
    }
    assert_eq!(actual["failedPrefixDigest"], p.digest);
    assert_eq!(actual["recoveryActionId"], p.action);
    assert_eq!(
        actual["failedActionId"],
        "22222222-2222-2222-2222-222222222222"
    );
    assert_eq!(actual["startedQpc"], "1000");
    assert_eq!(actual["deadlineQpc"], "11000");
    assert_eq!(actual["qpcFrequency"], "1000");
    assert_eq!(actual["event"]["kind"], "AbortRecoveryIntent");
    let next = record
        .next_data(
            &raw,
            Event::LauncherQuiesced {
                pid: 123,
                creation_filetime: "42".into(),
                forced: true,
                raw_exit: 242,
                job_total: 1,
                job_active: 0,
            },
        )
        .unwrap();
    let next_raw = next.encode_data().unwrap();
    crate::infrastructure::windows_client_originals024::before_resume_record::validate_chain(&[
        raw.clone(),
        next_raw.clone(),
    ])
    .unwrap();
    let n: serde_json::Value = serde_json::from_slice(&next_raw).unwrap();
    for name in [
        "sessionId",
        "failedActionId",
        "recoveryActionId",
        "startedQpc",
        "deadlineQpc",
        "qpcFrequency",
    ] {
        assert_eq!(n[name], actual[name]);
    }
    assert_eq!(n["previousRecordDigest"], hash::sha256_hex(&raw));
    println!("r7 exact v1 producer DATA mapping uses canonical UUID not adapter locator and same action/clock; component comparison grants no native permission");
}
#[test]
fn wl2_r7_attachment_origin_missing_invalid_canonical_or_same_failed_action_never_fills_placeholders(
) {
    for name in [
        "sessionId",
        "manifestDigest",
        "bindingDigest",
        "storagePlanDigest",
        "securityPolicyDigest",
    ] {
        let mut p = prefix();
        p.observed["canonical"]
            .as_object_mut()
            .unwrap()
            .remove(name);
        assert!(initial_data(&p, "22222222-2222-2222-2222-222222222222".into(), clock()).is_err());
    }
    let mut p = prefix();
    assert!(initial_data(&p, p.action.clone(), clock()).is_err());
    p.observed["canonical"]["manifestDigest"] = "UNKNOWN".into();
    assert!(initial_data(&p, "22222222-2222-2222-2222-222222222222".into(), clock()).is_err());
}
#[test]
fn wl2_r7_attachment_origin_pending_original_directory_pin_retained_without_record_authority() {
    let scene = tempfile::tempdir().unwrap();
    let path = scene.path().join("attachment");
    let directory =
        crate::infrastructure::windows_official_runtime024::create_private_directory(&path)
            .unwrap();
    let id = directory.identity();
    let p = prefix();
    let producer = BoundAttachment::pending(&p, directory);
    assert!(producer.failed && !producer.attempted && producer.last.is_none());
    assert_eq!(producer.pending_directory.as_ref().unwrap().identity(), id);
    assert!(std::fs::rename(&path, scene.path().join("replacement")).is_err());
    drop(producer);
    let root = scene.path().to_path_buf();
    scene.close().unwrap();
    assert!(!root.exists());
    println!("r7-attachment-origin-cleanup-absent={}", root.display());
}
