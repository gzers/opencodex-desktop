use super::*;
use crate::infrastructure::windows_client_originals024::before_resume_record_v2::{
    validate_chain_v2, EventV2, RecordV2,
};
#[test]
fn wl2_r7_bound_preparation_native_original_anchor_metadata_substitutions_refuse_without_mutation()
{
    let scene = Scene::new();
    let scope =
        crate::infrastructure::hash::sha256_hex(b"actual component scope DATA not nativeOwner");
    let session = uuid::Uuid::new_v4();
    let host =
        crate::infrastructure::windows_official_lifecycle024::original_host_instance_for_process();
    let mut prefix = PartialCaptureAnchor::new(scope.clone(), session, host);
    let end = Instant::now() + Duration::from_secs(10);
    let path = scene.path().join("bound-original-directory");
    prefix.intend(path.clone()).unwrap();
    let pin = prefix.create_directory(&path, end).unwrap();
    prefix.custody[0] = Some(pin);
    let id = prefix.custody[0].as_ref().unwrap().identity();
    prefix
        .check_original_capture_metadata(session, host, &scope, end)
        .unwrap();
    for (other_session, other_host, other_scope, other_end) in [
        (uuid::Uuid::new_v4(), host, scope.clone(), end),
        (session, uuid::Uuid::new_v4(), scope.clone(), end),
        (session, host, "foreign-scope".into(), end),
        (session, host, scope.clone(), Instant::now()),
        (uuid::Uuid::nil(), host, scope.clone(), end),
    ] {
        assert!(prefix
            .check_original_capture_metadata(other_session, other_host, &other_scope, other_end)
            .is_err());
    }
    let (reference, details) = prefix.v2_reference_data(end).unwrap();
    assert!(reference.pending());
    assert_eq!(details["sessionId"], session.to_string());
    assert_eq!(prefix.custody[0].as_ref().unwrap().identity(), id);
    assert!(prefix.failed);
    println!("r7 boundpreparation actualoriginalanchor metadata rejects wrongcapture/host/scope/expiry; same nativecreatorID/failed state retained; DATAguard not originalOwner permission");
    drop(prefix);
    drop(scene);
}
#[test]
fn wl2_r7_bound_preparation_native_reference_encodes_strict_v2_pending_original_session_without_manifest(
) {
    let scene = Scene::new();
    let session = uuid::Uuid::new_v4();
    let host =
        crate::infrastructure::windows_official_lifecycle024::original_host_instance_for_process();
    let mut prefix = PartialCaptureAnchor::new(
        crate::infrastructure::hash::sha256_hex(
            b"actual native component with explicit DATA action",
        ),
        session,
        host,
    );
    let end = Instant::now() + Duration::from_secs(10);
    let path = scene.path().join("original-pending-directory");
    prefix.intend(path.clone()).unwrap();
    let directory = prefix.create_directory(&path, end).unwrap();
    prefix.custody[0] = Some(directory);
    let (reference, _) = prefix.v2_reference_data(end).unwrap();
    assert!(reference.pending());
    let failed = uuid::Uuid::new_v4().to_string();
    let recovery = uuid::Uuid::new_v4().to_string();
    let policy = crate::infrastructure::windows_client_originals024::bounded::Policy::fixed()
        .digest()
        .unwrap();
    // Record fields below are explicit test DATA. No productOwner, native
    // failedStart/recovery action or QPC authority is constructed by this test.
    let record = RecordV2::initial_preparation_data(
        [
            session.to_string(),
            failed.clone(),
            recovery.clone(),
            policy,
        ],
        reference,
        ["1000", "11000", "1000"].map(String::from),
    )
    .unwrap();
    let raw = record.encode_data().unwrap();
    let data: serde_json::Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(data["sessionId"], session.to_string());
    assert_eq!(data["reference"]["ownerInstanceId"], host.to_string());
    assert_eq!(data["reference"]["source"]["state"], "unobserved");
    assert!(data["reference"].get("manifestDigest").is_none());
    let records = record.preparation_pending_data(Some(32)).unwrap();
    let [intent_raw, pending_raw] = &records;
    assert_eq!(intent_raw, &raw);
    validate_chain_v2(&records).unwrap();
    assert!(RecordV2::decode(pending_raw)
        .unwrap()
        .preparation_pending_data(None)
        .is_err());
    let data: serde_json::Value = serde_json::from_slice(pending_raw).unwrap();
    assert_eq!(data["failedActionId"], failed);
    assert_eq!(data["recoveryActionId"], recovery);
    assert_eq!(
        data["previousRecordDigest"],
        crate::infrastructure::hash::sha256_hex(&raw)
    );
    assert_eq!(data["event"]["rawError"], 32);
    assert_eq!(data["event"]["stage"], data["reference"]["lastStage"]);
    assert!(record
        .next_data(
            &raw,
            EventV2::AbortRestoreVerified {
                original_nodes: 0,
                coordination_objects: 3,
                normal_stop: false,
                closed: false,
                public_start_open: false
            }
        )
        .is_err());
    println!("r7 boundpreparation actualnativeprefix -> independentstrictV2 Intent/PendingDATA retains originalsession and explicittestaction/clock, unknownSource neverrestoreterminal");
    drop(prefix);
    drop(scene);
}
