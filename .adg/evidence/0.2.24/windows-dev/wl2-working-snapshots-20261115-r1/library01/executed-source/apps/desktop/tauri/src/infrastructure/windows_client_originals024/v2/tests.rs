use super::*;
fn progress_prefix(steps: &[(Phase, TreeRecord)]) -> (Vec<u8>, Vec<Vec<u8>>) {
    let (manifest, old, _) = bundle(manifest_value());
    let mut records = vec![old[0].clone()];
    for (phase, tree_record) in steps {
        let mut stage: Stage = json(&old[0], 65536).unwrap();
        stage.sequence = records.len() as u32 + 1;
        stage.previous_digest = hash::sha256_hex(records.last().unwrap());
        stage.previous_record_id = Some(format!("phase-{:08}.json", stage.sequence - 1));
        stage.phase = *phase;
        stage.tree_record = *tree_record;
        stage.working_record = working_record_for(
            *phase,
            &stage.binding,
            &stage.session_id,
            &stage.security_policy_digest,
        );
        records.push(serde_json::to_vec(&stage).unwrap());
    }
    (manifest, records)
}
#[test]
fn wl2_r4_working_reference_missing_wrong_phase_and_cross_session_refuse() {
    let mut steps = Vec::new();
    for root in [TreeRoot::Codex, TreeRoot::Grok] {
        for action in [TreeAction::MoveToCustody, TreeAction::CreateWorkingCopy] {
            steps.push((Phase::Captured, TreeRecord::Intent { root, action }));
            steps.push((Phase::Captured, TreeRecord::Completed { root, action }));
        }
    }
    steps.push((Phase::Ready, TreeRecord::None));
    let (manifest, records) = progress_prefix(&steps);
    let sid = current_identity().unwrap().sid;
    assert!(check_prefix(&manifest, &records, &sid).is_ok());
    for change in ["missing", "none", "phase", "session", "digest", "root"] {
        let mut wrong = records.clone();
        let mut value: serde_json::Value = serde_json::from_slice(wrong.last().unwrap()).unwrap();
        match change {
            "missing" => {
                value.as_object_mut().unwrap().remove("workingRecord");
            }
            "none" => value["workingRecord"] = serde_json::json!("None"),
            "phase" => value["workingRecord"]["Snapshot"]["phase"] = serde_json::json!("PreStop"),
            "session" => {
                value["workingRecord"]["Snapshot"]["sessionId"] =
                    serde_json::json!(uuid::Uuid::new_v4().to_string())
            }
            "digest" => {
                value["workingRecord"]["Snapshot"]["bindingDigest"] =
                    serde_json::json!("f".repeat(64))
            }
            "root" => {
                value["workingRecord"]["Snapshot"]["roots"][0]["root"] =
                    serde_json::json!(".foreign")
            }
            _ => unreachable!(),
        }
        *wrong.last_mut().unwrap() = serde_json::to_vec(&value).unwrap();
        assert!(check_prefix(&manifest, &wrong, &sid).is_err(), "{change}");
    }
    println!("working snapshot locator is mandatory and fixed to phase/session/binding/policy/root set; pure data remains PendingOnly, no native archive or phase permit reconstructed");
}
#[test]
fn wl2_r4_unperformed_roots_cannot_cross_ready_or_closed_in_v2_records() {
    let sid = current_identity().unwrap().sid;
    let (manifest, records) = progress_prefix(&[(Phase::Ready, TreeRecord::None)]);
    assert!(
        check_prefix(&manifest, &records, &sid).is_err(),
        "pure Ready text without either actual recorded root step must refuse"
    );
    let mut steps = Vec::new();
    for root in [TreeRoot::Codex, TreeRoot::Grok] {
        for action in [TreeAction::MoveToCustody, TreeAction::CreateWorkingCopy] {
            steps.push((Phase::Captured, TreeRecord::Intent { root, action }));
            steps.push((Phase::Captured, TreeRecord::Completed { root, action }));
        }
    }
    steps.extend(
        [
            Phase::Ready,
            Phase::PreparedRestore,
            Phase::OfficialTeardown,
        ]
        .map(|phase| (phase, TreeRecord::None)),
    );
    let mut premature = steps.clone();
    premature.push((Phase::Restored, TreeRecord::None));
    let (manifest, records) = progress_prefix(&premature);
    assert!(check_prefix(&manifest, &records, &sid).is_err());
    for root in [TreeRoot::Codex, TreeRoot::Grok] {
        let action = TreeAction::RestoreOriginal;
        steps.push((Phase::OfficialTeardown, TreeRecord::Intent { root, action }));
        steps.push((
            Phase::OfficialTeardown,
            TreeRecord::Completed { root, action },
        ));
        if root == TreeRoot::Codex {
            let mut partial = steps.clone();
            partial.push((Phase::Restored, TreeRecord::None));
            let (manifest, records) = progress_prefix(&partial);
            assert!(check_prefix(&manifest, &records, &sid).is_err());
        }
    }
    steps.extend([Phase::Restored, Phase::Closed].map(|phase| (phase, TreeRecord::None)));
    let (manifest, records) = progress_prefix(&steps);
    let tip = check_prefix(&manifest, &records, &sid).unwrap();
    assert!(tip.terminal_record_only);
    // Even a complete syntactic chain is pending locator data, not retained
    // native process/root custody or an adapter ClosedEligibility.
    println!("canonical two-root history required before Ready/Restored; full synthetic Closed prefix remains data-only, no production eligibility");
}
#[test]
fn wl2_r4_tree_progress_refuses_gap_duplicate_wrong_root_and_unfinished_phase() {
    use TreeAction::{CreateWorkingCopy, MoveToCustody, RestoreOriginal};
    use TreeRoot::{Codex, Grok};
    let intent = |root, action| TreeRecord::Intent { root, action };
    let done = |root, action| TreeRecord::Completed { root, action };
    let c = Phase::Captured;
    let invalid_prefixes = vec![
        vec![(c, done(Codex, MoveToCustody))],
        vec![(c, intent(Codex, CreateWorkingCopy))],
        vec![
            (c, intent(Codex, MoveToCustody)),
            (c, done(Grok, MoveToCustody)),
        ],
        vec![
            (c, intent(Codex, MoveToCustody)),
            (c, intent(Grok, MoveToCustody)),
        ],
        vec![
            (c, intent(Codex, MoveToCustody)),
            (c, done(Codex, MoveToCustody)),
            (c, intent(Codex, MoveToCustody)),
        ],
        vec![
            (c, intent(Codex, MoveToCustody)),
            (Phase::Ready, TreeRecord::None),
        ],
        vec![
            (c, intent(Codex, MoveToCustody)),
            (c, done(Codex, MoveToCustody)),
            (Phase::Ready, TreeRecord::None),
        ],
        vec![(c, intent(Codex, RestoreOriginal))],
    ];
    for steps in invalid_prefixes {
        let (manifest, records) = progress_prefix(&steps);
        assert!(
            check_prefix(&manifest, &records, &current_identity().unwrap().sid).is_err(),
            "invalid progress {steps:?}"
        );
    }
    let (manifest, partial) = progress_prefix(&[(c, intent(Codex, MoveToCustody))]);
    let tip = check_prefix(&manifest, &partial, &current_identity().unwrap().sid).unwrap();
    assert_eq!(tip.tree_record, intent(Codex, MoveToCustody));
    assert!(!tip.terminal_record_only);
    let mut old: serde_json::Value = serde_json::from_slice(&partial[0]).unwrap();
    old.as_object_mut().unwrap().remove("treeRecord");
    assert!(json::<Stage>(&serde_json::to_vec(&old).unwrap(), 65536).is_err());
    old["treeRecord"] =
        serde_json::json!({"Intent":{"root":"Codex","action":"CopyOriginalAcrossVolumes"}});
    assert!(json::<Stage>(&serde_json::to_vec(&old).unwrap(), 65536).is_err());
}
#[test]
fn wl2_r4_tree_progress_abort_restore_is_pending_data_not_closed_permission() {
    let mut steps = Vec::new();
    for root in [TreeRoot::Codex, TreeRoot::Grok] {
        for action in [TreeAction::MoveToCustody, TreeAction::CreateWorkingCopy] {
            steps.push((Phase::Captured, TreeRecord::Intent { root, action }));
            steps.push((Phase::Captured, TreeRecord::Completed { root, action }));
        }
    }
    steps.push((Phase::Aborted, TreeRecord::None));
    for root in [TreeRoot::Codex, TreeRoot::Grok] {
        let action = TreeAction::RestoreOriginal;
        steps.push((Phase::Aborted, TreeRecord::Intent { root, action }));
        steps.push((Phase::Aborted, TreeRecord::Completed { root, action }));
    }
    let (manifest, records) = progress_prefix(&steps);
    let tip = check_prefix(&manifest, &records, &current_identity().unwrap().sid).unwrap();
    assert_eq!(tip.sequence, 14);
    assert!(tip.phase == Phase::Aborted && !tip.terminal_record_only);
    steps.push((Phase::Closed, TreeRecord::None));
    let (manifest, records) = progress_prefix(&steps);
    assert!(check_prefix(&manifest, &records, &current_identity().unwrap().sid).is_err());
}
fn snapshot(id: &str) -> serde_json::Value {
    let entries=super::super::bounded::REQUIRED.into_iter().map(|mask|{
        let structure=serde_json::json!({"mask":mask,"revision":1,"control":0,"state":"absent","sid":null,"aclRevision":null,"aces":[]});
        // Hash the typed canonical serializer, not JSON map order.
        let typed:super::super::bounded::Projection=serde_json::from_value(structure.clone()).unwrap();
        serde_json::json!({"requestedMask":mask,"nativeStatus":0,"structure":structure,"structureDigest":hash::sha256_hex(&serde_json::to_vec(&typed).unwrap())})
    }).collect::<Vec<_>>();
    serde_json::json!({"scopeId":"windows-bounded-access-v1","securityPolicyDigest":Policy::fixed().digest().unwrap(),"volume":1,"fileId":id,"projections":entries,"auditSaclAssurance":"excluded-unknown","fullSecurityProven":false})
}
fn working_record_for(
    phase: Phase,
    binding: &Binding,
    session: &str,
    policy: &str,
) -> WorkingRecord {
    let phase = match phase {
        Phase::Ready => SnapshotPhase::Ready,
        Phase::PreparedRestore => SnapshotPhase::PreStop,
        _ => return WorkingRecord::None,
    };
    let object = |id: &str| serde_json::json!({"volume":1,"fileId":id,"security":snapshot(id)});
    let value = serde_json::json!({"schema":"windows-client-working-snapshot-v2","phase":phase,
        "sessionId":session,"bindingDigest":hash::sha256_hex(&serde_json::to_vec(binding).unwrap()),
        "securityPolicyDigest":policy,"directory":object("6"),
        "roots":[{"root":".codex","directory":object("7"),"metadata":object("8"),"metadataSha256":"a".repeat(64)},
        {"root":".grok","directory":object("9"),"metadata":object("10"),"metadataSha256":"b".repeat(64)}]});
    // Synthetic locator data only. This cannot construct a WorkingArchive or
    // a native RootTree/ReadySession from the recorded ids and hashes.
    WorkingRecord::Snapshot(serde_json::from_value(value).unwrap())
}
pub(super) fn manifest_value() -> serde_json::Value {
    let sid = current_identity().unwrap().sid;
    let session_id = uuid::Uuid::new_v4().to_string();
    serde_json::json!({"schema":"windows-client-originals-v2","documentKind":"manifest","sessionId":session_id,
        "binding":{"sid":sid,"profile":{"path":"C:/fixture/Profile","volume":1,"fileId":"1"},"data":{"path":"D:/fixture/D","volume":2,"fileId":"2"},"home":{"path":"C:/fixture/H","volume":1,"fileId":"3"},"installation":{"path":"C:/fixture/I","volume":1,"fileId":"4"},"sourceDigest":"a".repeat(64),"currentDigest":"b".repeat(64),"owner":{"pid":1234,"creationFiletime":"123456789"}},
        "securityPolicy":Policy::fixed(),"securityPolicyDigest":Policy::fixed().digest().unwrap(),
        "custody":{"path":format!("C:/fixture/Profile/.ocxd024-originals-{session_id}"),"identity":{"volume":1,"fileId":"5"},"securityProjectionSnapshot":snapshot("5")},
        "nodes":([".codex",".grok"].map(|relative|serde_json::json!({"relative":relative,"kind":"Absent","identity":null,"parentIdentity":{"volume":1,"fileId":"1"},"securityProjectionSnapshot":null,"parentSecurityProjectionSnapshot":snapshot("1"),"length":null,"contentDigest":null,"children":[],"custodyIdentity":null})))})
}
pub(super) fn bundle(m: serde_json::Value) -> (Vec<u8>, Vec<Vec<u8>>, Vec<u8>) {
    let manifest = serde_json::to_vec(&m).unwrap();
    let parsed: Manifest = json(&manifest, 1024 * 1024).unwrap();
    let mut phases: Vec<Vec<u8>> = Vec::new();
    // Data-only fixtures have no native root work. A legal Aborted prefix can
    // exercise schemas/CAS without inventing a Ready history.
    for (i, p) in [Phase::Captured, Phase::Aborted].into_iter().enumerate() {
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
            tree_record: TreeRecord::None,
            working_record: WorkingRecord::None,
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
