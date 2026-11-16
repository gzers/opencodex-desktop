use super::super::super::{
    segments::release_trees,
    tests::{compete, driver::setup_artifacts},
};
use super::*;
use crate::infrastructure::windows_client_originals024::{
    coordination::NativeLease, v3::FileAction,
};
use crate::modules::runtime::client_artifact_policy::{Action, Artifact, Assessment};
fn work(
    trees: &mut [AnchoredTree; 2],
    journal: &mut Journal,
    lease: &mut NativeLease,
    end: Instant,
) {
    let root = trees[0].root.clone();
    let mut claim = lease.anchored_claim(&root, end).unwrap();
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
}
fn catalog(root: &Path, model: bool) {
    let rows = if model {
        r#"[{"slug":"provider/routed"}]"#
    } else {
        "[]"
    };
    std::fs::write(
        root.join("opencodex-catalog.json"),
        format!("{{\"models\":{rows}}}"),
    )
    .unwrap();
    std::fs::write(root.join("models_cache.json"),format!("{{\"models\":{rows},\"fetched_at\":\"2000-01-01T00:00:00Z\",\"client_version\":\"0.0.0\"}}")).unwrap();
}
fn run_native_series(change_middle: bool) {
    let (mut f, _data, paths, held, mut trees, mut journal, end) = setup_artifacts();
    let mut lease = Some(held);
    work(&mut trees, &mut journal, lease.as_mut().unwrap(), end);
    let original = trees[0].plan();
    let root = trees[0].root.clone();
    release_trees(&mut trees, &mut journal, &mut lease, end).unwrap();
    catalog(&root, true);
    reacquire_for_observation(&mut trees, &mut journal, &mut lease, None, end).unwrap();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    observe_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
    super::super::snapshots::link_observation(&mut trees, &mut journal, &mut claim, false, end)
        .unwrap();
    let first = journal.observed_sequence_for_test();
    assert!(
        journal
            .observation_series_context([first, first], &mut claim, end)
            .is_err(),
        "missing independent intermediate frame cannot synthesize PreStop"
    );
    resume_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
    drop(claim);
    release_trees(&mut trees, &mut journal, &mut lease, end).unwrap();
    if change_middle {
        catalog(&root, false);
    }
    reacquire_for_observation(&mut trees, &mut journal, &mut lease, None, end).unwrap();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    observe_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
    super::super::snapshots::link_observation(&mut trees, &mut journal, &mut claim, false, end)
        .unwrap();
    let second = journal.observed_sequence_for_test();
    assert!(second > first);
    resume_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
    drop(claim);
    release_trees(&mut trees, &mut journal, &mut lease, end).unwrap();
    catalog(&root, false);
    reacquire_for_observation(&mut trees, &mut journal, &mut lease, None, end).unwrap();
    let mut claim = lease.as_mut().unwrap().anchored_claim(&root, end).unwrap();
    observe_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
    super::super::snapshots::link_observation(&mut trees, &mut journal, &mut claim, false, end)
        .unwrap();
    let context = journal
        .observation_series_context([first, second], &mut claim, end)
        .unwrap();
    let routes = vec!["provider/routed".to_string()];
    let now = chrono::Utc::now();
    let assessment = trees[0].assess_observation_content(
        &context,
        Some(&routes),
        &[123],
        "synthetic-home",
        now,
        now,
        &mut claim,
        end,
    );
    if change_middle {
        assert!(assessment
            .as_ref()
            .err()
            .unwrap()
            .to_string()
            .contains("ChangedSinceReady"));
        assert!(trees[0].poisoned);
    } else {
        assert_eq!(
            assessment.as_ref().unwrap(),
            &Assessment([
                (Artifact::Catalog, Action::AlreadyOriginal),
                (Artifact::ModelsCache, Action::AlreadyOriginal),
                (Artifact::CatalogAudit, Action::AlreadyOriginal)
            ])
        );
    }
    assert_eq!(
        trees[0].plan(),
        original,
        "actual original plan/baseline must never adopt changed work"
    );
    assert!(trees.iter().all(|tree| tree.snapshot_history.len() == 2));
    assert_eq!(journal.snapshot_state_for_test().2, "Preparing");
    assert_eq!(journal.work_mode_for_test(), "Observed");
    compete(&paths[0], true);
    journal.check(end).unwrap();
    println!("r6-native-series context={} sequences={:?} policy={assessment:?} remainingMs={}; actual separate frames and same-volume immutable archives, old work generations persist; two matching routed content frames and actual final original content accepted by unchanged shared policy as DATA only; PID/routes/audit are explicit synthetic inputs, not actual Source/Ready/NormalExit, no restore/Closed authority",context.binding(0).unwrap(),context.sequences(),end.saturating_duration_since(Instant::now()).as_millis());
    drop(claim);
    lease.take().unwrap().release(end).unwrap();
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_series_three_native_observations_shared_policy_and_original_baseline_survive_reopen() {
    run_native_series(false);
}
#[test]
fn wl2_r6_series_actual_between_observations_change_remains_policy_refusal_and_preserves_custody() {
    run_native_series(true);
}
#[test]
fn wl2_r6_series_resume_native_or_result_failure_keeps_exact_linked_snapshot_prefix() {
    for fault in [24, 25, 26, 27] {
        let (mut f, _data, paths, mut lease, mut trees, mut journal, end) = setup_artifacts();
        work(&mut trees, &mut journal, &mut lease, end);
        let root = trees[0].root.clone();
        let mut claim = lease.anchored_claim(&root, end).unwrap();
        observe_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
        super::super::snapshots::link_observation(&mut trees, &mut journal, &mut claim, false, end)
            .unwrap();
        journal.inject_driver_fault(fault);
        let error = resume_trees(&mut trees, &mut journal, &mut claim, end)
            .err()
            .unwrap();
        assert!(trees.iter().all(|t| t.poisoned));
        assert_eq!(journal.work_mode_for_test(), "Resuming");
        assert!(journal.check(end).is_err());
        let histories = trees
            .iter()
            .map(|t| t.snapshot_history.len())
            .collect::<Vec<_>>();
        assert_eq!(histories, if fault == 24 { vec![0, 0] } else { vec![1, 1] });
        compete(&paths[0], true);
        println!("r6-series-resume-fault={fault} histories={histories:?} nativeModes={:?} canonical=Resuming error={error}; exact work descriptors/archive objects/original custody retained, no reentry or auto-result",trees.each_ref().map(|t|t.work_mode));
        drop(claim);
        lease.release(end).unwrap();
        drop(journal);
        drop(trees);
        f.retain();
    }
}
#[test]
fn wl2_r6_series_unlinked_or_invalid_old_archive_refuses_reopen_without_adoption() {
    for linked in [false, true] {
        let (mut f, _data, _paths, mut lease, mut trees, mut journal, end) = setup_artifacts();
        work(&mut trees, &mut journal, &mut lease, end);
        let root = trees[0].root.clone();
        let mut claim = lease.anchored_claim(&root, end).unwrap();
        observe_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
        if linked {
            super::super::snapshots::link_observation(
                &mut trees,
                &mut journal,
                &mut claim,
                false,
                end,
            )
            .unwrap();
            resume_trees(&mut trees, &mut journal, &mut claim, end).unwrap();
            let reference = trees[0].snapshot_history[0].archive.reference(end).unwrap();
            let directory = PathBuf::from(reference["directory"].as_str().unwrap());
            drop(create_private_directory(&directory.join("foreign-empty")).unwrap());
            assert!(trees[0].verify_custody_anchors(end).is_err());
            assert!(observe_trees(&mut trees, &mut journal, &mut claim, end).is_err());
        } else {
            assert!(resume_trees(&mut trees, &mut journal, &mut claim, end).is_err());
            assert!(trees[0].observed.is_some());
        }
        println!("r6-series-refusal linked={linked}; missing two-root native link or unknown retained archive child never yields reopen permission; actual immutable/original objects preserved");
        drop(claim);
        lease.release(end).unwrap();
        drop(journal);
        drop(trees);
        f.retain();
    }
}
