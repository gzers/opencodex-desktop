use super::*;
fn snapshot(id: &str) -> serde_json::Value {
    let entries=super::super::bounded::REQUIRED.into_iter().map(|mask|{
        let structure=serde_json::json!({"mask":mask,"revision":1,"control":0,"state":"absent","sid":null,"aclRevision":null,"aces":[]});
        // Hash the typed canonical serializer, not JSON map order.
        let typed:super::super::bounded::Projection=serde_json::from_value(structure.clone()).unwrap();
        serde_json::json!({"requestedMask":mask,"nativeStatus":0,"structure":structure,"structureDigest":hash::sha256_hex(&serde_json::to_vec(&typed).unwrap())})
    }).collect::<Vec<_>>();
    serde_json::json!({"scopeId":"windows-bounded-access-v1","securityPolicyDigest":Policy::fixed().digest().unwrap(),"volume":1,"fileId":id,"projections":entries,"auditSaclAssurance":"excluded-unknown","fullSecurityProven":false})
}
pub(super) fn manifest_value() -> serde_json::Value {
    let sid = current_identity().unwrap().sid;
    serde_json::json!({"schema":"windows-client-originals-v2","documentKind":"manifest","sessionId":uuid::Uuid::new_v4().to_string(),
        "binding":{"sid":sid,"profile":{"path":"C:/fixture/Profile","volume":1,"fileId":"1"},"data":{"path":"D:/fixture/D","volume":2,"fileId":"2"},"home":{"path":"C:/fixture/H","volume":1,"fileId":"3"},"installation":{"path":"C:/fixture/I","volume":1,"fileId":"4"},"sourceDigest":"a".repeat(64),"currentDigest":"b".repeat(64),"owner":{"pid":1234,"creationFiletime":"123456789"}},
        "securityPolicy":Policy::fixed(),"securityPolicyDigest":Policy::fixed().digest().unwrap(),
        "nodes":([".codex",".grok"].map(|relative|serde_json::json!({"relative":relative,"kind":"Absent","identity":null,"parentIdentity":{"volume":1,"fileId":"1"},"securityProjectionSnapshot":null,"parentSecurityProjectionSnapshot":snapshot("1"),"length":null,"contentDigest":null,"children":[],"custodyIdentity":null})))})
}
pub(super) fn bundle(m: serde_json::Value) -> (Vec<u8>, Vec<Vec<u8>>, Vec<u8>) {
    let manifest = serde_json::to_vec(&m).unwrap();
    let parsed: Manifest = json(&manifest, 1024 * 1024).unwrap();
    let mut phases: Vec<Vec<u8>> = Vec::new();
    for (i, p) in [Phase::Captured, Phase::Ready].into_iter().enumerate() {
        let binding: Binding = serde_json::from_value(m["binding"].clone()).unwrap();
        let s = Stage {
            schema: ORIGINALS_RECORD_SCHEMA_V2.into(),
            document_kind: "phase".into(),
            session_id: parsed.session_id.clone(),
            sequence: i as u32 + 1,
            previous_digest: phases.last().map_or(String::new(), |b| hash::sha256_hex(b)),
            previous_record_id: if i == 0 {
                None
            } else {
                Some("phase-00000001.json".into())
            },
            binding,
            phase: p,
            originals_manifest_digest: hash::sha256_hex(&manifest),
            security_policy_digest: parsed.security_policy_digest.clone(),
        };
        phases.push(serde_json::to_vec(&s).unwrap());
    }
    let tip = check_prefix(&manifest, &phases, &current_identity().unwrap().sid).unwrap();
    let current = serde_json::to_vec(&current_for(&tip)).unwrap();
    (manifest, phases, current)
}
#[test]
fn wl2_r4_v2_policy_digest_scope_schema_and_all8_projection_set_reject_downgrade() {
    let original = manifest_value();
    let sid = current_identity().unwrap().sid;
    let good: Manifest = serde_json::from_value(original.clone()).unwrap();
    check_manifest(&good, &sid).unwrap();
    for path in ["schema", "securityPolicy", "securityPolicyDigest"] {
        let mut bad = original.clone();
        bad.as_object_mut().unwrap().remove(path);
        assert!(serde_json::from_value::<Manifest>(bad).is_err());
    }
    for value in [
        "windows-client-originals-v1",
        "PartialDiagnostic",
        "windows-client-current-v2",
    ] {
        let mut bad = original.clone();
        bad["schema"] = serde_json::json!(value);
        assert!(check_manifest(&serde_json::from_value(bad).unwrap(), &sid).is_err());
    }
    for field in ["scopeId", "excludedAssurance"] {
        let mut bad = original.clone();
        bad["securityPolicy"][field] = serde_json::json!("full-v1");
        assert!(check_manifest(&serde_json::from_value(bad).unwrap(), &sid).is_err());
    }
    for changes in [
        serde_json::json!([1, 2, 4]),
        serde_json::json!([1, 2, 4, 16, 32, 64, 128, 128]),
        serde_json::json!([2, 1, 4, 16, 32, 64, 128, 256]),
    ] {
        let mut bad = original.clone();
        bad["securityPolicy"]["requiredProjections"] = changes;
        assert!(check_manifest(&serde_json::from_value(bad).unwrap(), &sid).is_err());
    }
    for value in [serde_json::json!(true), serde_json::json!("absent")] {
        let mut bad = original.clone();
        if value.is_boolean() {
            bad["nodes"][0]["parentSecurityProjectionSnapshot"]["fullSecurityProven"] = value;
        } else {
            bad["nodes"][0]["parentSecurityProjectionSnapshot"]["auditSaclAssurance"] = value;
        }
        assert!(check_manifest(&serde_json::from_value(bad).unwrap(), &sid).is_err());
    }
    for field in ["nativeStatus", "requestedMask"] {
        let mut bad = original.clone();
        bad["nodes"][0]["parentSecurityProjectionSnapshot"]["projections"][0][field] =
            serde_json::json!(5);
        assert!(check_manifest(&serde_json::from_value(bad).unwrap(), &sid).is_err());
    }
}
#[test]
fn wl2_r4_v2_native_private_records_remain_pending_and_current_mismatch_is_not_authority() {
    let (m, records, c) = bundle(manifest_value());
    let meta: Manifest = json(&m, 1024 * 1024).unwrap();
    let t = tempfile::tempdir().unwrap();
    let pin = crate::infrastructure::windows_official_runtime024::create_private_directory(
        &t.path().join(&meta.session_id),
    )
    .unwrap();
    let manifest =
        PrivateRecord::create(&pin.path().join("manifest.json"), &m, 1024 * 1024).unwrap();
    let mut handles = Vec::new();
    for (i, b) in records.iter().enumerate() {
        handles.push(
            PrivateRecord::create(
                &pin.path().join(format!("phase-{:08}.json", i + 1)),
                b,
                65536,
            )
            .unwrap(),
        );
    }
    let pending = read_pending(pin.path()).unwrap();
    assert!(!pending.current_matches && !pending.terminal_record_only);
    drop(pending);
    let current = PrivateRecord::create(&pin.path().join("current.json"), &c, 4096).unwrap();
    let pending = read_pending(pin.path()).unwrap();
    assert!(pending.current_matches && !pending.terminal_record_only);
    drop(pending);
    drop(current);
    let mut wrong: serde_json::Value = serde_json::from_slice(&c).unwrap();
    wrong["securityPolicyDigest"] = serde_json::json!("f".repeat(64));
    std::fs::write(
        pin.path().join("current.json"),
        serde_json::to_vec(&wrong).unwrap(),
    )
    .unwrap();
    let pending = read_pending(pin.path()).unwrap();
    assert!(!pending.current_matches);
    drop(pending);
    std::fs::write(pin.path().join("unknown-authority.json"), b"{}").unwrap();
    assert!(read_pending(pin.path()).is_err());
    println!("v2 native-private records: policy-bound pending only; missing/stale current keeps Required; no Full/partial conversion or Closed eligibility");
    drop(handles);
    drop(manifest);
    drop(pin);
}
#[test]
fn wl2_r4_v2_phase_current_policy_digest_replay_and_unknown_fields_refuse() {
    let (m, good, c) = bundle(manifest_value());
    let sid = current_identity().unwrap().sid;
    for field in [
        "schema",
        "securityPolicyDigest",
        "originalsManifestDigest",
        "sessionId",
        "previousDigest",
    ] {
        let mut bad = good.clone();
        let mut value: serde_json::Value = serde_json::from_slice(&bad[1]).unwrap();
        value[field] = serde_json::json!("changed");
        bad[1] = serde_json::to_vec(&value).unwrap();
        assert!(check_prefix(&m, &bad, &sid).is_err());
    }
    assert!(check_prefix(&m, &[good[1].clone()], &sid).is_err());
    assert!(check_prefix(&m, &[good[0].clone(), good[0].clone()], &sid).is_err());
    let raw = String::from_utf8(c.clone()).unwrap();
    for b in [
        raw.replacen('{', "{\"securityPolicyDigest\":\"duplicate\",", 1),
        raw.replacen('{', "{\"restoreAuthority\":true,", 1),
    ] {
        assert!(json::<Current>(b.as_bytes(), 4096).is_err());
    }
    assert!(json::<Current>(b"{", 4096).is_err());
    let mut old: serde_json::Value = serde_json::from_slice(&c).unwrap();
    old["schema"] = serde_json::json!("windows-client-current-v1");
    let expected = current_for(&check_prefix(&m, &good, &sid).unwrap());
    assert!(json::<Current>(&serde_json::to_vec(&old).unwrap(), 4096).unwrap() != expected);
}
