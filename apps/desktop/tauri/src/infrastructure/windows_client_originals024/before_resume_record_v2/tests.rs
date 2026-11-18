use super::*;
use serde_json::{json, Value};
#[test]
fn wl2_r7_bound_preparation_data_encoder_rejects_canonical_origin_clock_and_previous_substitution()
{
    let fields = [
        "11111111-1111-1111-1111-111111111111",
        "22222222-2222-2222-2222-222222222222",
        "33333333-3333-3333-3333-333333333333",
    ]
    .map(String::from);
    let policy = "e".repeat(64);
    let clock = ["1000", "11000", "1000"].map(String::from);
    let refdata: ReferenceV2 = serde_json::from_value(reference()).unwrap();
    assert!(RecordV2::initial_preparation_data(
        [
            fields[0].clone(),
            fields[1].clone(),
            fields[2].clone(),
            policy.clone()
        ],
        refdata,
        clock.clone()
    )
    .is_err());
    for variant in ["same-action", "nil-session", "bad-policy", "long-clock"] {
        let refdata: ReferenceV2 = serde_json::from_value(preparation(false)).unwrap();
        let mut input = [
            fields[0].clone(),
            fields[1].clone(),
            fields[2].clone(),
            policy.clone(),
        ];
        let mut time = clock.clone();
        match variant {
            "same-action" => input[2] = input[1].clone(),
            "nil-session" => input[0] = uuid::Uuid::nil().to_string(),
            "bad-policy" => input[3] = "unknown".into(),
            _ => time[1] = "11001".into(),
        }
        assert!(
            RecordV2::initial_preparation_data(input, refdata, time).is_err(),
            "{variant}"
        );
    }
    let refdata: ReferenceV2 = serde_json::from_value(preparation(false)).unwrap();
    let record = RecordV2::initial_preparation_data(
        [
            fields[0].clone(),
            fields[1].clone(),
            fields[2].clone(),
            policy,
        ],
        refdata,
        clock,
    )
    .unwrap();
    let raw = record.encode_data().unwrap();
    let mut wrong: Value = serde_json::from_slice(&raw).unwrap();
    wrong["sessionId"] = json!(uuid::Uuid::new_v4().to_string());
    assert!(record
        .next_data(
            &bytes(&wrong),
            EventV2::AbortRecoveryPending {
                stage: Stage::Registered,
                raw_error: None
            }
        )
        .is_err());
    let next = record
        .next_data(
            &raw,
            EventV2::AbortRecoveryPending {
                stage: Stage::Registered,
                raw_error: None,
            },
        )
        .unwrap();
    let next_raw = next.encode_data().unwrap();
    validate_chain_v2(&[raw, next_raw]).unwrap();
}
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
fn record(reference: Value) -> Value {
    json!({"schema":SCHEMA,"sessionId":"11111111-1111-1111-1111-111111111111","failedActionId":"22222222-2222-2222-2222-222222222222","recoveryActionId":"33333333-3333-3333-3333-333333333333","securityPolicyDigest":"e".repeat(64),"reference":reference,"sequence":1,"previousRecordDigest":null,"startedQpc":"1000","deadlineQpc":"11000","qpcFrequency":"1000","event":{"kind":"AbortRecoveryIntent"}})
}
fn bytes(v: &Value) -> Vec<u8> {
    serde_json::to_vec(v).unwrap()
}
fn raw_chain(reference: Value, events: &[Value]) -> Vec<Vec<u8>> {
    let mut result: Vec<Vec<u8>> = Vec::new();
    for (i, event) in events.iter().enumerate() {
        let mut r = record(reference.clone());
        r["event"] = event.clone();
        r["sequence"] = json!(i + 1);
        if let Some(previous) = result.last() {
            r["previousRecordDigest"] = json!(crate::infrastructure::hash::sha256_hex(previous));
        }
        result.push(bytes(&r));
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
#[test]
fn wl2_r7_v2_reference_strict_canonical_and_unverified_preparation_are_separate_data_variants() {
    for value in [record(reference()), record(preparation(false))] {
        let r = RecordV2::decode(&bytes(&value)).unwrap();
        validate_chain_v2(&[bytes(&value)]).unwrap();
        assert_eq!(
            r.reference.pending(),
            value["reference"]["kind"] == "preparation"
        );
    }
    let reference = preparation(false);
    let mut events = vec![json!({"kind":"AbortRecoveryIntent"}), quiescent()];
    events.push(json!({"kind":"AbortRecoveryPending","stage":"nativeCreated","rawError":null}));
    validate_chain_v2(&raw_chain(reference.clone(), &events)).unwrap();
    events[2] = terminal(0);
    assert!(validate_chain_v2(&raw_chain(reference, &events)).is_err());
}
#[test]
fn wl2_r7_v2_reference_every_topfield_required_nullable_duplicate_version_and_clock_fail_closed() {
    let good = record(reference());
    for field in good.as_object().unwrap().keys() {
        let mut bad = good.clone();
        bad.as_object_mut().unwrap().remove(field);
        assert!(RecordV2::decode(&bytes(&bad)).is_err(), "missing{field}");
    }
    for (field, value) in [
        ("schema", json!("windows-before-resume-recovery-v1")),
        ("schema", json!("windows-client-originals-v3")),
        ("sessionId", json!("00000000-0000-0000-0000-000000000000")),
        ("failedActionId", good["recoveryActionId"].clone()),
        ("sequence", json!(0)),
        ("startedQpc", json!("01000")),
        ("deadlineQpc", json!("11001")),
        ("qpcFrequency", json!(u64::MAX.to_string())),
        ("unknown", json!(false)),
    ] {
        let mut bad = good.clone();
        bad[field] = value;
        assert!(RecordV2::decode(&bytes(&bad)).is_err(), "{field}");
    }
    let dup = String::from_utf8(bytes(&good)).unwrap().replacen(
        '{',
        "{\"schema\":\"windows-before-resume-recovery-v2\",",
        1,
    );
    assert!(RecordV2::decode(dup.as_bytes()).is_err());
    assert!(RecordV2::decode(&vec![b' '; 65537]).is_err());
}
#[test]
fn wl2_r7_v2_reference_preparation_strict_fields_component_order_duplicates_and_manifest_forgery_refuse(
) {
    let p = preparation(true);
    for key in p.as_object().unwrap().keys() {
        let mut bad = p.clone();
        bad.as_object_mut().unwrap().remove(key);
        assert!(
            RecordV2::decode(&bytes(&record(bad))).is_err(),
            "reference{key}"
        );
    }
    for (key, value) in [
        ("manifestDigest", json!("a".repeat(64))),
        ("storagePlanDigest", json!("c".repeat(64))),
        ("lastStage", json!("arbitrary-diagnostic-text")),
        ("ownerInstanceId", json!("UNKNOWN")),
    ] {
        let mut bad = p.clone();
        bad[key] = value;
        assert!(RecordV2::decode(&bytes(&record(bad))).is_err(), "prep{key}");
    }
    for index in 0..5 {
        for key in p["components"][index].as_object().unwrap().keys() {
            let mut bad = p.clone();
            bad["components"][index]
                .as_object_mut()
                .unwrap()
                .remove(key);
            assert!(
                RecordV2::decode(&bytes(&record(bad))).is_err(),
                "componentmissing{key}"
            );
        }
        for key in p["components"][index]["objects"][0]
            .as_object()
            .unwrap()
            .keys()
        {
            let mut bad = p.clone();
            bad["components"][index]["objects"][0]
                .as_object_mut()
                .unwrap()
                .remove(key);
            assert!(
                RecordV2::decode(&bytes(&record(bad))).is_err(),
                "objectmissing{key}"
            );
        }
    }
    for case in [
        "order",
        "role",
        "duplicate-id",
        "duplicate-path",
        "missingrole",
        "unknownobject",
        "emptyretained",
    ] {
        let mut bad = p.clone();
        match case {
            "order" => bad["components"].as_array_mut().unwrap().swap(0, 1),
            "role" => bad["components"][0]["role"] = json!("unknown"),
            "duplicate-id" => {
                bad["components"][1]["objects"][0]["objectId"] =
                    bad["components"][0]["objects"][0]["objectId"].clone()
            }
            "duplicate-path" => {
                let mut o = bad["components"][0]["objects"][0].clone();
                o["objectId"] = json!("9".repeat(64));
                bad["components"][0]["objects"]
                    .as_array_mut()
                    .unwrap()
                    .push(o);
            }
            "missingrole" => {
                bad["components"].as_array_mut().unwrap().pop();
            }
            "unknownobject" => bad["components"][0]["objects"][0]["unknown"] = true.into(),
            _ => bad["components"][0]["objects"] = json!([]),
        }
        assert!(RecordV2::decode(&bytes(&record(bad))).is_err(), "{case}");
    }
}
#[test]
fn wl2_r7_v2_reference_source_variants_required_and_unobserved_never_becomes_absence_or_success() {
    let mut p = preparation(true);
    for source in [
        json!({"state":"captured"}),
        json!({"state":"captured","bindingDigest":"a".repeat(64),"imageDigest":"b".repeat(64),"extra":null}),
        json!({"state":"observedAbsent","parentIdentityDigest":"a".repeat(64)}),
        json!({"state":"unobserved","bindingDigest":"a".repeat(64)}),
    ] {
        p["source"] = source;
        assert!(RecordV2::decode(&bytes(&record(p.clone()))).is_err());
    }
    p = preparation(true);
    p["source"] = json!({"state":"unobserved"});
    let mut events = prep_events(&p);
    events.push(terminal(2));
    assert!(validate_chain_v2(&raw_chain(p.clone(), &events)).is_err());
    p = preparation(true);
    p["source"] = json!({"state":"observedAbsent","parentIdentityDigest":"a".repeat(64),"observationDigest":"b".repeat(64)});
    p["components"][0] = json!({"role":"source","state":"verifiedAbsent","prefixDigest":"a".repeat(64),"objects":[]});
    RecordV2::decode(&bytes(&record(p.clone()))).unwrap();
    assert!(RecordV2::decode(&bytes(&{
        let mut r = record(p);
        r["event"] = terminal(0);
        r
    }))
    .is_err());
}
#[test]
fn wl2_r7_v2_reference_preparation_events_match_exact_declared_objects_but_never_imply_restored() {
    let p = preparation(true);
    let good = prep_events(&p);
    validate_chain_v2(&raw_chain(p.clone(), &good)).unwrap();
    for case in [
        "missing-intent",
        "wrong-id",
        "wrong-role",
        "wrong-stage",
        "wrong-membership",
        "replay",
    ] {
        let mut bad = good.clone();
        match case {
            "missing-intent" => {
                bad.remove(2);
            }
            "wrong-id" => bad[3]["objectId"] = json!("9".repeat(64)),
            "wrong-role" => bad[3]["role"] = json!("custody"),
            "wrong-stage" => bad[3]["originalStage"] = json!("currentPublished"),
            "wrong-membership" => bad[3]["retainedMembershipDigest"] = json!("e".repeat(64)),
            _ => bad.insert(4, bad[2].clone()),
        }
        assert!(
            validate_chain_v2(&raw_chain(p.clone(), &bad)).is_err(),
            "{case}"
        );
    }
    let mut complete = good.clone();
    complete.push(terminal(2));
    validate_chain_v2(&raw_chain(p.clone(), &complete)).unwrap();
    let mut hidden = complete.clone();
    *hidden.last_mut().unwrap() = terminal(0);
    assert!(validate_chain_v2(&raw_chain(p.clone(), &hidden)).is_err());
    let mut lost = complete;
    lost.drain(2..4);
    assert!(
        validate_chain_v2(&raw_chain(p.clone(), &lost)).is_err(),
        "unaccounted retained object not terminal"
    );
    let mut no_complete = p;
    no_complete["components"][0]["objects"][0]["securityDigest"] = Value::Null;
    let mut events = prep_events(&no_complete);
    events.push(terminal(2));
    assert!(validate_chain_v2(&raw_chain(no_complete, &events)).is_err());
}
#[test]
fn wl2_r7_v2_reference_canonical_chain_strict_nodes_forced_falseclosed_and_cross_action_replay_refuse(
) {
    let p = reference();
    let node = "f".repeat(64);
    let events = vec![
        json!({"kind":"AbortRecoveryIntent"}),
        quiescent(),
        json!({"kind":"WorkRetireIntent","nodeId":node,"workFileId":"10","parentId":"20"}),
        json!({"kind":"WorkRetiredVerified","nodeId":node,"workFileId":"10","parentId":"20"}),
        json!({"kind":"OriginalReturnIntent","nodeId":node,"originalFileId":"30","parentId":"20"}),
        json!({"kind":"OriginalReturnedVerified","nodeId":node,"originalFileId":"30","parentId":"20"}),
        terminal(10),
    ];
    let good = raw_chain(p.clone(), &events);
    validate_chain_v2(&good).unwrap();
    for name in [
        "sessionId",
        "failedActionId",
        "recoveryActionId",
        "previousRecordDigest",
        "deadlineQpc",
    ] {
        let mut bad = good.clone();
        let mut v: Value = serde_json::from_slice(&bad[1]).unwrap();
        v[name] = if name == "deadlineQpc" {
            json!("10999")
        } else if name.ends_with("Id") {
            json!("44444444-4444-4444-4444-444444444444")
        } else {
            json!("9".repeat(64))
        };
        bad[1] = bytes(&v);
        assert!(validate_chain_v2(&bad).is_err());
    }
    for name in ["normalStop", "closed", "publicStartOpen"] {
        let mut bad = events.clone();
        bad.last_mut().unwrap()[name] = true.into();
        assert!(validate_chain_v2(&raw_chain(p.clone(), &bad)).is_err());
    }
    let mut bad = events;
    bad[1]["forced"] = false.into();
    assert!(validate_chain_v2(&raw_chain(p, &bad)).is_err());
}
#[test]
fn wl2_r7_v2_reference_original_formats_are_not_translated_or_used_as_fallback() {
    let old = json!({"schema":"windows-before-resume-recovery-v1","sessionId":"11111111-1111-1111-1111-111111111111","failedActionId":"22222222-2222-2222-2222-222222222222","manifestDigest":"a".repeat(64),"bindingDigest":"b".repeat(64),"storagePlanDigest":"c".repeat(64),"securityPolicyDigest":"d".repeat(64),"failedPrefixDigest":"e".repeat(64),"recoveryActionId":"33333333-3333-3333-3333-333333333333","sequence":1,"previousRecordDigest":null,"startedQpc":"1000","deadlineQpc":"11000","qpcFrequency":"1000","event":{"kind":"AbortRecoveryIntent"}});
    crate::infrastructure::windows_client_originals024::before_resume_record::Record::decode(
        &bytes(&old),
    )
    .unwrap();
    assert!(RecordV2::decode(&bytes(&old)).is_err());
    assert!(
        crate::infrastructure::windows_client_originals024::before_resume_record::Record::decode(
            &bytes(&record(reference()))
        )
        .is_err()
    );
    let mut forged = old;
    forged["schema"] = json!(SCHEMA);
    assert!(
        RecordV2::decode(&bytes(&forged)).is_err(),
        "schema replacement is not conversion"
    );
}
#[test]
fn wl2_r7_v2_reference_zero_client_nodes_requires_explicit_complete_source_absence_and_all_retained_objects(
) {
    let mut p = preparation(true);
    for i in [2, 3, 4] {
        let role = p["components"][i]["role"].clone();
        p["components"][i] = json!({"role":role,"state":"verifiedAbsent","prefixDigest":format!("{i:064x}"),"objects":[]});
    }
    let mut events = prep_events(&p);
    events.push(terminal(0));
    validate_chain_v2(&raw_chain(p.clone(), &events)).unwrap();
    // Pure DATA consistency only: actual initial native set/zero mutation/all
    // locks/source proof is still required by a future original Owner factory.
    p["components"][4]["state"] = json!("unobserved");
    p["components"][4]["prefixDigest"] = Value::Null;
    assert!(validate_chain_v2(&raw_chain(p, &events)).is_err());
}
#[test]
fn wl2_r7_v2_reference_object_count_record_size_unknown_events_and_pending_terminal_suffix_refuse()
{
    let mut p = preparation(true);
    let original = p["components"][0]["objects"][0].clone();
    for i in 0..513 {
        let mut node = original.clone();
        node["objectId"] = json!(format!("{:064x}", i + 1000));
        node["pathDigest"] = json!(format!("{:064x}", i + 2000));
        p["components"][0]["objects"]
            .as_array_mut()
            .unwrap()
            .push(node);
    }
    assert!(RecordV2::decode(&bytes(&record(p))).is_err());
    for kind in ["Closed", "Stopped", "Ready", "NativePredecessors"] {
        let mut r = record(reference());
        r["event"] = json!({"kind":kind});
        assert!(RecordV2::decode(&bytes(&r)).is_err());
    }
    let events = vec![
        json!({"kind":"AbortRecoveryIntent"}),
        json!({"kind":"AbortRecoveryPending","stage":"registered","rawError":null}),
        quiescent(),
    ];
    assert!(validate_chain_v2(&raw_chain(preparation(false), &events)).is_err());
}
