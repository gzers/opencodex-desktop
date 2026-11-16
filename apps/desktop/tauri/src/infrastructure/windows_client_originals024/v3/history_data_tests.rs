use super::super::{recovery::tests::released, tests::plan};
use super::*;
fn references(plan: &StoragePlan, context: &HistoryContextData) -> [HistoryReference; 2] {
    std::array::from_fn(|i| {
        let root = &plan.roots[i];
        let id = (u64::MAX - 4000 - i as u64 * 2).to_string();
        let meta = (u64::MAX - 4001 - i as u64 * 2).to_string();
        let security = |id: &str| {
            let mut s = serde_json::to_value(&root.nodes[0].security).unwrap();
            s["fileId"] = id.into();
            serde_json::from_value(s).unwrap()
        };
        HistoryReference {
            schema: "windows-anchored-original-history-reference-v3".into(),
            root_id: root.nodes[0].root_id.clone(),
            directory: Path::new(&root.root)
                .parent()
                .unwrap()
                .join(format!(
                    ".ocxd024-history-{}-{}",
                    context.session_id,
                    if i == 0 { "codex" } else { "grok" }
                ))
                .to_str()
                .unwrap()
                .into(),
            volume: root.custody_volume,
            file_id: id.clone(),
            security: security(&id),
            metadata_id: meta.clone(),
            metadata_security: security(&meta),
            metadata_sha256: "f".repeat(64),
            frame_digest: context.original_frames[i].clone(),
            context: serde_json::from_slice(&encoded(context).unwrap()).unwrap(),
            runtime_authority: false,
        }
    })
}
pub(in crate::infrastructure::windows_client_originals024::v3) fn linked(
    p: &StoragePlan,
) -> Progress {
    let mut g = released(p);
    let context = g.history_context(p).unwrap();
    g.apply_history(
        p,
        &g.history_event(HistoryOperation::OriginalHistoryIntent, &context, None)
            .unwrap(),
    )
    .unwrap();
    g.apply_history(
        p,
        &g.history_event(
            HistoryOperation::OriginalHistoryLinkedVerified,
            &context,
            Some(references(p, &context)),
        )
        .unwrap(),
    )
    .unwrap();
    g
}
#[test]
fn wl2_r6_history_data_original_reference_requires_exact_release_scope_both_roots_and_no_text_closed(
) {
    let (mut f, p, _, _) = plan();
    let mut g = released(&p);
    let context = g.history_context(&p).unwrap();
    assert!(g
        .apply_history(
            &p,
            &g.history_event(
                HistoryOperation::OriginalHistoryLinkedVerified,
                &context,
                Some(references(&p, &context))
            )
            .unwrap()
        )
        .is_err());
    g.apply_entry(
        &p,
        &segments::ChainEntry::History(
            g.history_event(HistoryOperation::OriginalHistoryIntent, &context, None)
                .unwrap(),
        ),
    )
    .unwrap();
    let good = serde_json::to_value(
        g.history_event(
            HistoryOperation::OriginalHistoryLinkedVerified,
            &context,
            Some(references(&p, &context)),
        )
        .unwrap(),
    )
    .unwrap();
    for pointer in [
        "/context/leaseEpoch",
        "/context/claimKey",
        "/context/deadlineScope",
        "/context/releasedEventDigest",
        "/context/recoveryPlanDigest",
        "/references/0/rootId",
        "/references/0/directory",
        "/references/0/frameDigest",
        "/references/0/metadataId",
        "/references/1/context/bindingDigest",
    ] {
        let mut bad = good.clone();
        *bad.pointer_mut(pointer).unwrap() = "foreign".into();
        let tip = g.tip.clone();
        assert!(
            g.apply_history(&p, &serde_json::from_value(bad).unwrap())
                .is_err(),
            "{pointer}"
        );
        assert_eq!(g.tip, tip);
        assert!(g
            .history_summary()
            .unwrap()
            .unwrap()
            .linked_references
            .is_none());
    }
    let mut bad = good.clone();
    bad["references"][0]["runtimeAuthority"] = true.into();
    assert!(g
        .apply_history(&p, &serde_json::from_value(bad).unwrap())
        .is_err());
    for (volume, id) in g.recovery_reserved_identities().unwrap() {
        let mut bad = good.clone();
        bad["references"][0]["volume"] = volume.into();
        bad["references"][0]["fileId"] = id.clone().into();
        bad["references"][0]["security"]["volume"] = volume.into();
        bad["references"][0]["security"]["fileId"] = id.into();
        assert!(
            g.apply_history(&p, &serde_json::from_value(bad).unwrap())
                .is_err(),
            "archive copy cannot alias original/current/work/snapshot/parent/custody identity"
        );
    }
    let mut bad = good.clone();
    bad["references"].as_array_mut().unwrap().swap(0, 1);
    assert!(g
        .apply_history(&p, &serde_json::from_value(bad).unwrap())
        .is_err());
    let mut bad = good.clone();
    bad["Closed"] = true.into();
    assert!(serde_json::from_value::<segments::ChainEntry>(bad).is_err());
    g.apply_history(&p, &serde_json::from_value(good).unwrap())
        .unwrap();
    assert!(g
        .history_summary()
        .unwrap()
        .unwrap()
        .linked_references
        .is_some());
    assert!(g.history_context(&p).is_err());
    assert!(g
        .apply_history(
            &p,
            &g.history_event(
                HistoryOperation::OriginalHistoryLinkedVerified,
                &context,
                Some(references(&p, &context))
            )
            .unwrap()
        )
        .is_err());
    println!("r6 original history DATA exact released-action/source/manifest/plan/claim/recovery/both original frames binds intent then immutable references; malformed/missing/replay/textClosed reject, no native history/Closed factory");
    f.retain();
}
#[test]
fn wl2_r6_history_data_unreleased_or_incomplete_and_context_changes_cannot_prepare_archive() {
    let (mut f, p, _, _) = plan();
    let mut g = released(&p);
    let context = g.history_context(&p).unwrap();
    let good = serde_json::to_value(
        g.history_event(HistoryOperation::OriginalHistoryIntent, &context, None)
            .unwrap(),
    )
    .unwrap();
    g.lease_state = segments::LeaseState::ReleasePrepared;
    assert!(g.history_context(&p).is_err());
    assert!(g
        .apply_history(&p, &serde_json::from_value(good.clone()).unwrap())
        .is_err());
    g.lease_state = segments::LeaseState::Released;
    for field in [
        "manifestDigest",
        "bindingDigest",
        "storagePlanDigest",
        "releaseObjectsDigest",
        "originalFrames",
        "historyIntentSequence",
    ] {
        let mut bad = good.clone();
        bad["context"][field] = match field {
            "originalFrames" => serde_json::json!(["f".repeat(64), "e".repeat(64)]),
            "historyIntentSequence" => 0.into(),
            _ => "foreign".into(),
        };
        assert!(
            g.apply_history(&p, &serde_json::from_value(bad).unwrap())
                .is_err(),
            "{field}"
        );
        assert!(g.history.is_none());
    }
    println!("r6 original history DATA requires original full-node/anchor/explicit release and exact native scope locators before archive intent; no deadline reset or copy permit from caller JSON");
    f.retain();
}
#[test]
fn wl2_r6_handles_data_native_history_references_must_equal_original_canonical_link() {
    let (mut f, p, _, _) = plan();
    let g = linked(&p);
    let context = &g.history.as_ref().unwrap().context;
    let refs = references(&p, context).map(|r| serde_json::to_value(r).unwrap());
    g.verify_original_history_references(&p, refs.clone())
        .unwrap();
    let mut bad = refs.clone();
    bad[0]["metadataSha256"] = "a".repeat(64).into();
    assert!(
        g.verify_original_history_references(&p, bad).is_err(),
        "valid-form foreign metadata digest must not substitute actual linked original archive"
    );
    let mut bad = refs.clone();
    bad.swap(0, 1);
    assert!(g.verify_original_history_references(&p, bad).is_err());
    let mut bad = refs;
    bad[1]["context"]["releasedEventDigest"] = "a".repeat(64).into();
    assert!(g.verify_original_history_references(&p, bad).is_err());
    println!("r6 original-handle native history DATA join requires exact BOTH canonical linked references, not merely matching shape/scope/original frame; valid-form substituted archive hash/cross-root/release-context refuses");
    f.retain();
}
