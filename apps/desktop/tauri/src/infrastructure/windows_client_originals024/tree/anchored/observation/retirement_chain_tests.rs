use super::super::super::{
    segments::release_trees,
    tests::{compete, driver::setup},
};
use super::*;
use crate::infrastructure::windows_client_originals024::{
    coordination::NativeLease,
    v3::{FileAction, Journal},
};

fn key(root: &Path, relative: &str) -> String {
    hash::sha256_hex(
        format!(
            "{}/{}",
            root.file_name().unwrap().to_str().unwrap(),
            relative
        )
        .as_bytes(),
    )
}
fn prepare(
    trees: &mut [AnchoredTree; 2],
    journal: &mut Journal,
    lease: &mut Option<NativeLease>,
    end: Instant,
) {
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    let files = trees
        .iter()
        .enumerate()
        .flat_map(|(r, t)| {
            t.nodes
                .iter()
                .enumerate()
                .filter_map(move |(i, n)| (n.kind == Kind::File).then_some((r, i)))
        })
        .collect::<Vec<_>>();
    for action in [FileAction::MoveOriginal, FileAction::CreateWork] {
        for (r, i) in &files {
            trees[*r]
                .journal_step(*i, action, journal, &mut claim, end)
                .unwrap();
        }
    }
    open_trees(trees, journal, &mut claim, end).unwrap();
    drop(claim);
    release_trees(trees, journal, lease, end).unwrap();
    for (name, bytes) in [
        ("opencodex-catalog.json", b"{\"models\":[]}".as_slice()),
        ("models_cache.json", b"synthetic current cache".as_slice()),
        (
            "opencodex-catalog-audit.jsonl",
            b"synthetic audit\n".as_slice(),
        ),
    ] {
        drop(PrivateRecord::create(&root.join(name), bytes, 1024).unwrap());
    }
    for purpose in ["ready", "pre-stop", "stopped"] {
        if purpose == "pre-stop" {
            journal.component_stop_scope_for_test(end).unwrap();
        }
        if purpose == "stopped" {
            let path = root.join("synthetic-current-replacement.tmp");
            drop(
                PrivateRecord::create(
                    &path,
                    b"{\"models\":[{\"slug\":\"synthetic-current\"}]}",
                    1024,
                )
                .unwrap(),
            );
            std::fs::rename(&path, root.join("opencodex-catalog.json")).unwrap();
        }
        assert!(lease.is_none());
        let plans = trees.each_ref().map(|t| t.plan());
        *lease = Some(
            journal
                .component_reacquire_same_window_for_test(&plans, end)
                .unwrap(),
        );
        let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
        observe_trees(trees, journal, &mut claim, end).unwrap();
        let context = journal
            .component_begin_phase_for_test(purpose, &mut claim, end)
            .unwrap();
        super::super::snapshots::create_snapshots(trees, journal, &context, &mut claim, false, end)
            .unwrap();
        journal
            .component_record_phase_for_test(
                purpose,
                super::super::snapshots::witness(trees, end).unwrap(),
                &mut claim,
                end,
            )
            .unwrap();
        if purpose != "stopped" {
            super::super::series::resume_trees(trees, journal, &mut claim, end).unwrap();
            drop(claim);
            release_trees(trees, journal, lease, end).unwrap();
        }
        println!(
            "r6 recovery native canonical component phase={purpose} remainingMs={}",
            end.saturating_duration_since(Instant::now()).as_millis()
        );
    }
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    for t in trees {
        t.capture_retirement(&mut claim, end).unwrap();
    }
}
fn keys(trees: &[AnchoredTree; 2]) -> (Vec<String>, Vec<String>) {
    (
        trees
            .iter()
            .flat_map(|t| {
                t.retirement.as_ref().unwrap().items.iter().map(|i| {
                    key(
                        &t.root,
                        &t.retirement.as_ref().unwrap().frame.nodes[i.frame_index]
                            .entry
                            .relative,
                    )
                })
            })
            .collect(),
        trees
            .iter()
            .flat_map(|t| {
                t.nodes
                    .iter()
                    .filter(|n| n.kind == Kind::File)
                    .map(|n| key(&t.root, &n.original.relative))
            })
            .collect(),
    )
}
/// Post-failure forensic data. This does not clear poison, re-open mutation
/// permission, change the cutoff, or use those observations as recovery proof.
fn positions(trees: &[AnchoredTree; 2]) -> serde_json::Value {
    let mut rows = Vec::new();
    for t in trees {
        for n in &t.nodes {
            let path = if n.kind == Kind::File && n.state != State::Restored {
                t.custody
                    .path()
                    .join(format!("original-{}", key(&t.root, &n.original.relative)))
            } else {
                t.root.join(&n.original.relative)
            };
            let file = open_node(&path, n.kind, true).unwrap();
            assert_eq!(id(&file, n.kind == Kind::Directory).unwrap(), n.original.id);
            rows.push(
                serde_json::json!({"kind":"original","relative":n.original.relative,"path":path,
                "fileId":n.original.id.index.to_string(),"state":format!("{:?}",n.state)}),
            );
        }
        let r = t.retirement.as_ref().unwrap();
        for item in &r.items {
            let n = &r.frame.nodes[item.frame_index].entry;
            let path = if item.retired {
                item.target.clone()
            } else {
                t.root.join(&n.relative)
            };
            let file = open_node(&path, Kind::File, true).unwrap();
            assert_eq!(id(&file, false).unwrap(), n.id);
            let bytes = std::fs::read(&path).unwrap();
            assert_eq!(bytes, n.bytes);
            rows.push(serde_json::json!({"kind":"current","relative":n.relative,"path":path,
                "fileId":n.id.index.to_string(),"retired":item.retired,"bytes":bytes.len(),"sha256":hash::sha256_hex(&bytes)}));
        }
    }
    serde_json::json!({"postFailureForensicOnly":true,"rows":rows})
}
#[test]
fn wl2_r6_recovery_chain_native_all_current_and_original_nodes_use_same_canonical_chain() {
    let (mut f, _data, paths, held, mut trees, mut journal, end) = setup();
    let mut lease = Some(held);
    prepare(&mut trees, &mut journal, &mut lease, end);
    let originals = trees.each_ref().map(|t| t.plan());
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    let (current, original) = keys(&trees);
    assert_eq!(current.len(), 6);
    assert_eq!(original.len(), 3);
    journal
        .component_prepare_recovery_for_test(
            recovery_witness(&mut trees, &mut claim, end).unwrap(),
            &mut claim,
            end,
        )
        .unwrap();
    for (retire, nodes) in [(true, &current), (false, &original)] {
        for node in nodes {
            journal
                .component_execute_recovery_for_test(
                    recovery_witness(&mut trees, &mut claim, end).unwrap(),
                    node,
                    retire,
                    &mut claim,
                    end,
                )
                .unwrap();
            compete(&paths[0], true);
        }
    }
    recovery_witness(&mut trees, &mut claim, end)
        .unwrap()
        .restored(&mut claim, end)
        .unwrap();
    assert_eq!(trees.each_ref().map(|t| t.plan()), originals);
    let state = journal.component_recovery_state_for_test().unwrap();
    assert_eq!(state["retiredCount"], 6);
    assert_eq!(state["restoredCount"], 3);
    assert!(state["pending"].is_null());
    println!(
        "r6-recovery-chain-native-success={}",
        serde_json::json!({"durable":journal.component_recovery_trace_for_test().unwrap(),"positions":positions(&trees)})
    );
    drop(claim);
    lease.take().unwrap().release(end).unwrap();
    for t in &mut trees {
        t.readback_retirement_after_release(end).unwrap();
    }
    println!("r6 recovery native component canonical plan/6 current retirement/3 exact original restore intent-result-current with actual same handles and all8/children/archives preserved; ordinary claim competitor busy at every node, explicit raw lease release readback done; canonical final release/history and actual Source/Stop/content/Closed not qualified");
    drop(journal);
    drop(trees);
    f.retain();
}
fn interrupted(fault: u8, restore: bool) {
    let (mut f, _data, _paths, held, mut trees, mut journal, end) = setup();
    let mut lease = Some(held);
    prepare(&mut trees, &mut journal, &mut lease, end);
    let root = trees[0].root.clone();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    let (current, original) = keys(&trees);
    if matches!(fault, 1 | 2 | 44) {
        journal.inject_driver_fault(fault);
        assert!(journal
            .component_prepare_recovery_for_test(
                recovery_witness(&mut trees, &mut claim, end).unwrap(),
                &mut claim,
                end
            )
            .is_err());
    } else {
        journal
            .component_prepare_recovery_for_test(
                recovery_witness(&mut trees, &mut claim, end).unwrap(),
                &mut claim,
                end,
            )
            .unwrap();
        if restore {
            for node in &current {
                journal
                    .component_execute_recovery_for_test(
                        recovery_witness(&mut trees, &mut claim, end).unwrap(),
                        node,
                        true,
                        &mut claim,
                        end,
                    )
                    .unwrap();
            }
        }
        journal.inject_driver_fault(fault);
        let node = if restore { &original[0] } else { &current[0] };
        assert!(journal
            .component_execute_recovery_for_test(
                recovery_witness(&mut trees, &mut claim, end).unwrap(),
                node,
                !restore,
                &mut claim,
                end
            )
            .is_err());
    }
    assert!(journal.native_progress_for_test().1);
    assert!(trees.iter().all(|t| t.poisoned));
    let counts = trees
        .iter()
        .map(|t| {
            t.retirement
                .as_ref()
                .unwrap()
                .items
                .iter()
                .filter(|i| i.retired)
                .count()
        })
        .sum::<usize>();
    let restored = trees
        .iter()
        .flat_map(|t| &t.nodes)
        .filter(|n| n.kind == Kind::File && n.state == State::Restored)
        .count();
    let moved = !matches!(fault, 1 | 2 | 40 | 44);
    assert_eq!(
        counts,
        if restore {
            current.len()
        } else {
            usize::from(moved)
        }
    );
    assert_eq!(restored, usize::from(restore && moved));
    let trace = journal.component_recovery_trace_for_test().unwrap();
    assert_eq!(
        trace["currentActualBytes"].as_u64() == Some(1),
        matches!(fault, 2 | 43)
    );
    let events = trace["immutableRecoveryEvents"].as_array().unwrap();
    let last = events.last().unwrap();
    let sequence = trace["sequence"].as_u64().unwrap();
    if matches!(fault, 1 | 2) {
        assert_eq!(events.len(), 1);
        assert_eq!(last["operation"], "RecoveryPlanIntent");
        assert!(trace["summary"].is_null());
        assert_eq!(last["sequence"].as_u64(), Some(sequence + 1));
    } else if fault == 44 {
        assert_eq!(events.len(), 1);
        assert_eq!(last["operation"], "RecoveryPlanIntent");
        assert_eq!(trace["summary"]["planVerified"], false);
        assert_eq!(last["sequence"].as_u64(), Some(sequence));
    } else {
        let complete = matches!(fault, 42 | 43);
        let operation = match (restore, complete) {
            (false, false) => "WorkRetireIntent",
            (false, true) => "WorkRetiredVerified",
            (true, false) => "OriginalRestoreIntent",
            (true, true) => "OriginalRestoredVerified",
        };
        assert_eq!(last["operation"], operation);
        assert_eq!(
            last["sequence"].as_u64(),
            Some(sequence + u64::from(complete))
        );
        assert_eq!(
            trace["summary"]["retiredCount"],
            if restore {
                serde_json::json!(current.len())
            } else {
                serde_json::json!(0)
            }
        );
        assert_eq!(trace["summary"]["restoredCount"], 0);
        assert!(!trace["summary"]["pending"].is_null());
    }
    if !matches!(fault, 2 | 43) {
        assert_eq!(trace["currentActual"]["sequence"].as_u64(), Some(sequence));
        assert_eq!(trace["currentActual"]["recovery"], trace["summary"]);
    }
    assert!(
        journal.check(end).is_err(),
        "poison must never auto-catch-up"
    );
    assert_eq!(
        journal.component_recovery_trace_for_test().unwrap(),
        trace,
        "refused check must not rewrite current or phase records"
    );
    println!(
        "r6-recovery-chain-native-interruption={}",
        serde_json::json!({"fault":fault,"restore":restore,"currentRetired":counts,"originalRestored":restored,"durable":trace,"positions":positions(&trees)})
    );
    drop(claim);
    lease.take().unwrap().release(end).unwrap();
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_recovery_chain_native_interrupted_plan_preserves_immutable_and_partial_current() {
    for fault in [1, 2, 44] {
        interrupted(fault, false);
    }
}
#[test]
fn wl2_r6_recovery_chain_native_interrupted_retirement_preserves_exact_native_prefix() {
    for fault in [40, 41, 42, 43] {
        interrupted(fault, false);
    }
}
#[test]
fn wl2_r6_recovery_chain_native_interrupted_original_restore_preserves_exact_native_prefix() {
    for fault in [40, 41, 42, 43] {
        interrupted(fault, true);
    }
}
