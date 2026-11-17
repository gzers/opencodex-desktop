use super::*;
use serde_json::{json, Value};
pub(super) fn record() -> Value {
    json!({"schema":SCHEMA,"sessionId":"11111111-1111-1111-1111-111111111111","failedActionId":"22222222-2222-2222-2222-222222222222","manifestDigest":"a".repeat(64),"bindingDigest":"b".repeat(64),"storagePlanDigest":"c".repeat(64),"securityPolicyDigest":"d".repeat(64),"failedPrefixDigest":"e".repeat(64),"recoveryActionId":"33333333-3333-3333-3333-333333333333","sequence":1,"previousRecordDigest":null,"startedQpc":"1000","deadlineQpc":"11000","qpcFrequency":"1000","event":{"kind":"AbortRecoveryIntent"}})
}
pub(super) fn bytes(v: &Value) -> Vec<u8> {
    serde_json::to_vec(v).unwrap()
}
pub(super) fn chain(events: &[Value]) -> Vec<Vec<u8>> {
    let mut out: Vec<Vec<u8>> = Vec::new();
    for (i, event) in events.iter().enumerate() {
        let mut r = record();
        r["event"] = event.clone();
        r["sequence"] = json!(i + 1);
        if let Some(previous) = out.last() {
            r["previousRecordDigest"] = json!(crate::infrastructure::hash::sha256_hex(previous));
        }
        out.push(bytes(&r));
    }
    out
}
pub(super) fn events() -> Vec<Value> {
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
#[test]
fn r7_abort_attachment_strict_fields_versions_duplicates_and_timing() {
    let good = record();
    Record::decode(&bytes(&good)).unwrap();
    for key in good.as_object().unwrap().keys() {
        let mut bad = good.clone();
        bad.as_object_mut().unwrap().remove(key);
        // serde Option otherwise accepts omission: exact field check must still
        // distinguish absent previousRecordDigest from an explicit first null.
        assert!(Record::decode(&bytes(&bad)).is_err(), "missing {key}");
    }
    for (key, value) in [
        ("schema", json!("windows-client-originals-v3")),
        ("sessionId", json!("00000000-0000-0000-0000-000000000000")),
        ("failedPrefixDigest", json!("F".repeat(64))),
        ("startedQpc", json!("01000")),
        ("deadlineQpc", json!("11001")),
        ("deadlineQpc", json!("1000")),
        ("qpcFrequency", json!("0")),
        ("sequence", json!(0)),
        ("normalStop", json!(true)),
        ("event", json!({"kind":"Closed"})),
        ("recoveryActionId", good["failedActionId"].clone()),
    ] {
        let mut bad = good.clone();
        bad[key] = value;
        assert!(Record::decode(&bytes(&bad)).is_err(), "{key}");
    }
    let mut bad = good.clone();
    bad["event"]["unexpected"] = json!(false);
    assert!(Record::decode(&bytes(&bad)).is_err());
    let text = String::from_utf8(bytes(&good)).unwrap().replacen(
        '{',
        "{\"schema\":\"windows-before-resume-recovery-v1\",",
        1,
    );
    assert!(Record::decode(text.as_bytes()).is_err());
    assert!(Record::decode(&vec![b' '; 65537]).is_err());
}
#[test]
fn r7_abort_attachment_chain_rejects_cross_scope_replay_gaps_and_false_closed() {
    let good = chain(&events());
    validate_chain(&good).unwrap();
    for key in [
        "sessionId",
        "failedActionId",
        "manifestDigest",
        "bindingDigest",
        "storagePlanDigest",
        "securityPolicyDigest",
        "failedPrefixDigest",
        "recoveryActionId",
        "startedQpc",
        "deadlineQpc",
        "qpcFrequency",
        "sequence",
        "previousRecordDigest",
    ] {
        let mut bad = good.clone();
        let mut row: Value = serde_json::from_slice(&bad[1]).unwrap();
        row[key] = match key {
            "sequence" => json!(3),
            "startedQpc" => json!("999"),
            "deadlineQpc" => json!("10999"),
            "qpcFrequency" => json!("1001"),
            k if k.ends_with("Id") => json!("44444444-4444-4444-4444-444444444444"),
            _ => json!("0".repeat(64)),
        };
        bad[1] = bytes(&row);
        assert!(validate_chain(&bad).is_err(), "{key}");
    }
    let mut replay = good.clone();
    replay.push(good[6].clone());
    assert!(validate_chain(&replay).is_err());
    for key in ["normalStop", "closed", "publicStartOpen"] {
        let mut bad = events();
        bad[6][key] = json!(true);
        assert!(validate_chain(&chain(&bad)).is_err());
    }
}
#[test]
fn r7_abort_attachment_prefix_rejects_missing_quiescence_reordered_or_wrong_native_ids() {
    let good = events();
    for remove in [1, 2, 3, 4, 5] {
        let mut bad = good.clone();
        bad.remove(remove);
        assert!(validate_chain(&chain(&bad)).is_err(), "remove{remove}");
    }
    for (row, key, value) in [
        (3, "workFileId", "11"),
        (3, "parentId", "21"),
        (4, "originalFileId", "10"),
        (5, "originalFileId", "31"),
        (5, "parentId", "21"),
    ] {
        let mut bad = good.clone();
        bad[row][key] = json!(value);
        assert!(validate_chain(&chain(&bad)).is_err(), "{row}/{key}");
    }
    for (key, value) in [
        ("forced", json!(false)),
        ("rawExit", json!(0)),
        ("jobTotal", json!(2)),
        ("jobActive", json!(1)),
    ] {
        let mut bad = good.clone();
        bad[1][key] = value;
        assert!(validate_chain(&chain(&bad)).is_err());
    }
    let pending = json!({"kind":"AbortRecoveryPending","stage":"source-timeout","rawError":1460});
    for end in 1..good.len() {
        let mut partial = good[..end].to_vec();
        partial.push(pending.clone());
        validate_chain(&chain(&partial)).unwrap();
        partial.push(pending.clone());
        assert!(validate_chain(&chain(&partial)).is_err());
    }
}

#[test]
fn r7_abort_attachment_event_fields_are_required_and_unknowns_and_duplicate_nodes_refuse() {
    let baseline = events();
    for (i, event) in baseline.iter().enumerate() {
        for key in event.as_object().unwrap().keys() {
            let mut bad = baseline.clone();
            bad[i].as_object_mut().unwrap().remove(key);
            assert!(
                validate_chain(&chain(&bad)).is_err(),
                "event{i} missing{key}"
            );
        }
        let mut bad = baseline.clone();
        bad[i]["futureField"] = json!(true);
        assert!(validate_chain(&chain(&bad)).is_err(), "event{i} unknown");
    }
    let mut repeated = baseline.clone();
    repeated.insert(4, baseline[2].clone());
    assert!(validate_chain(&chain(&repeated)).is_err());
    let mut wrong_order = baseline.clone();
    wrong_order.swap(4, 5);
    assert!(validate_chain(&chain(&wrong_order)).is_err());
    let mut nil = record();
    nil["qpcFrequency"] = json!(u64::MAX.to_string());
    assert!(Record::decode(&bytes(&nil)).is_err());
    let mut pending = baseline[..3].to_vec();
    pending.push(json!({"kind":"AbortRecoveryPending","stage":"lock-busy","rawError":null}));
    validate_chain(&chain(&pending)).unwrap();
    pending
        .last_mut()
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("rawError");
    assert!(validate_chain(&chain(&pending)).is_err());
}
