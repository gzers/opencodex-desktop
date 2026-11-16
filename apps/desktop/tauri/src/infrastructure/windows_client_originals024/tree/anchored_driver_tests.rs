use super::*;
use crate::infrastructure::windows_client_originals024::{
    binding::Binding,
    v3::{FileAction, Journal, StoragePlan},
};
use native::RetainedPeer;
pub(in crate::infrastructure::windows_client_originals024::tree::anchored) struct DataScene(
    PathBuf,
);
impl Drop for DataScene {
    fn drop(&mut self) {
        println!("r4-retained-synthetic-fixture={}", self.0.display());
    }
}
type Setup = (
    Fixture,
    DataScene,
    [PathBuf; 4],
    NativeLease,
    [AnchoredTree; 2],
    Journal,
    Instant,
);
pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn setup() -> Setup {
    setup_with_claim(false, false)
}
pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn setup_initialized_claim(
) -> Setup {
    setup_with_claim(true, false)
}
pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn setup_artifacts(
) -> Setup {
    setup_with_claim(false, true)
}
fn setup_with_claim(initialized: bool, artifacts: bool) -> Setup {
    let f = Fixture::new();
    let end = end();
    let (root, custody, paths) = prepare(&f);
    if artifacts {
        for(name,bytes)in [("opencodex-catalog.json",b"{\"models\":[]}".as_slice()),("models_cache.json",b"{\"models\":[],\"fetched_at\":\"2000-01-01T00:00:00Z\",\"client_version\":\"0.0.0\"}".as_slice()),("opencodex-catalog-audit.jsonl",b"".as_slice())]{drop(PrivateRecord::create(&root.join(name),bytes,1024).unwrap());}
    }
    if initialized {
        // Fixture-only BEFORE capture. Empty SQLite first-use may create a
        // transient journal during BEGIN EXCLUSIVE, which correctly triggers
        // unknown-sidecar refusal instead of exercising the raw lock busy path.
        // No production initialization, schema/PRAGMA or official data touched.
        let c = rusqlite::Connection::open(&paths[0]).unwrap();
        c.execute_batch("PRAGMA journal_mode=DELETE; CREATE TABLE synthetic_lease_busy(value INTEGER); INSERT INTO synthetic_lease_busy VALUES(1)").unwrap();
        c.close().unwrap();
    }
    let grok = root.with_file_name(".grok");
    drop(create_private_directory(&grok).unwrap());
    drop(
        PrivateRecord::create(
            &grok.join("settings.json"),
            b"synthetic Grok original",
            1024,
        )
        .unwrap(),
    );
    let grok_custody = root.with_file_name("grok-custody");
    drop(create_private_directory(&grok_custody).unwrap());
    let data_path = PathBuf::from(
        "D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/test/windows024-native/target",
    )
    .join(format!("r6-driver-{}", uuid::Uuid::new_v4()));
    let data = create_private_directory(&data_path).unwrap();
    let scene = DataScene(data_path);
    let pins = [
        PinnedDirectory::open_private(root.parent().unwrap()).unwrap(),
        data,
        create_private_directory(&f.path().join("H")).unwrap(),
        create_private_directory(&f.path().join("I")).unwrap(),
    ];
    assert_ne!(pins[0].identity().volume, pins[1].identity().volume);
    let directory = create_private_directory(&pins[1].path().join("session")).unwrap();
    let peer = RetainedPeer::current().unwrap();
    let binding = Binding::from_native(
        &peer,
        &pins,
        hash::sha256_hex(b"synthetic source not official admission"),
        hash::sha256_hex(b"synthetic source current"),
    )
    .unwrap();
    let mut lease = NativeLease::acquire_synthetic(&paths, end).unwrap();
    let mut claim = lease.anchored_claim(&root, end).unwrap();
    let mut trees = [
        AnchoredTree::capture(&root, &custody, &mut claim, &root, end).unwrap(),
        AnchoredTree::capture(&grok, &grok_custody, &mut claim, &root, end).unwrap(),
    ];
    let plan = StoragePlan::from_native_data(
        trees.iter().map(AnchoredTree::plan).collect(),
        pins[0].path(),
        pins[0].identity(),
    )
    .unwrap();
    let mut journal = Journal::create(
        directory,
        pins,
        peer,
        binding,
        plan,
        claim.epoch(),
        &claim.key(),
        &uuid::Uuid::new_v4().to_string(),
        end,
    )
    .unwrap();
    for tree in &mut trees {
        tree.journal_snapshot_anchors(&mut journal, &mut claim, end)
            .unwrap();
    }
    drop(claim);
    (f, scene, paths, lease, trees, journal, end)
}
#[test]
fn wl2_r6_native_node_driver_binds_durable_intents_actual_generations_and_original_tree_restore() {
    let (mut f, _data, paths, mut lease, mut trees, mut journal, end) = setup();
    let root = trees[0].root.clone();
    let mut claim = lease.anchored_claim(&root, end).unwrap();
    let plans = trees.iter().map(AnchoredTree::plan).collect::<Vec<_>>();
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
    bun_claim(&paths[0], end, true);
    for action in [
        FileAction::MoveOriginal,
        FileAction::CreateWork,
        FileAction::RetireWork,
        FileAction::RestoreOriginal,
    ] {
        for (r, i) in &files {
            trees[*r]
                .journal_step(*i, action, &mut journal, &mut claim, end)
                .unwrap();
            compete(&paths[0], true);
            println!(
                "r6-native-journal-prefix root={r} node={i} action={action:?} journal={:?}",
                journal.native_progress_for_test()
            );
        }
    }
    bun_claim(&paths[0], end, true);
    assert_eq!(
        trees.iter().map(AnchoredTree::plan).collect::<Vec<_>>(),
        plans
    );
    journal.check(end).unwrap();
    assert_eq!(journal.native_progress_for_test(), (30, false));
    drop(claim);
    lease.release(end).unwrap();
    for tree in &mut trees {
        tree.verify_after_release(end).unwrap();
    }
    bun_claim(&paths[0], end, false);
    println!("r6 native driver actual three-file Codex+Grok whole trees restore original IDs/bytes/all8/children;30 canonical events include6 anchor snapshots and24 ordered native file intent/results; explicit lease release+stable proof, no production phase/Closed/successor or release-event authority");
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_native_node_driver_interruptions_keep_exact_native_prefix_and_durable_pending() {
    for fault in [4, 3, 5, 6] {
        let (mut f, _data, _paths, mut lease, mut trees, mut journal, end) = setup();
        let root = trees[0].root.clone();
        let i = trees[0]
            .nodes
            .iter()
            .position(|n| n.kind == Kind::File)
            .unwrap();
        let original = trees[0].root.join(&trees[0].nodes[i].original.relative);
        let mut claim = lease.anchored_claim(&root, end).unwrap();
        journal.inject_driver_fault(fault);
        let error = trees[0]
            .journal_step(i, FileAction::MoveOriginal, &mut journal, &mut claim, end)
            .err()
            .unwrap();
        assert!(trees[0].poisoned);
        assert_eq!(journal.native_progress_for_test(), (7, true));
        assert!(journal.check(end).is_err());
        assert!(trees[0]
            .journal_step(i, FileAction::MoveOriginal, &mut journal, &mut claim, end)
            .is_err());
        if fault == 4 {
            assert_eq!(trees[0].nodes[i].state, State::Original);
            assert!(original.exists());
        } else {
            assert_eq!(trees[0].nodes[i].state, State::Moved);
            assert!(!original.exists());
            assert!(trees[0].original_location(i).exists());
        }
        println!("r6-native-journal-interruption fault={fault} originalPresent={} nativeState={:?} journal={:?} error={error}",original.exists(),trees[0].nodes[i].state,journal.native_progress_for_test());
        drop(claim);
        lease.release(end).unwrap();
        drop(journal);
        drop(trees);
        f.retain();
    }
}
#[test]
fn wl2_r6_native_node_driver_new_lease_epoch_cannot_inherit_previous_permission() {
    let (mut f, _data, paths, lease, mut trees, mut journal, end) = setup();
    lease.release(end).unwrap();
    let root = trees[0].root.clone();
    let i = trees[0]
        .nodes
        .iter()
        .position(|n| n.kind == Kind::File)
        .unwrap();
    let mut new_lease = NativeLease::acquire_synthetic(&paths, end).unwrap();
    let mut claim = new_lease.anchored_claim(&root, end).unwrap();
    assert!(trees[0]
        .journal_step(i, FileAction::MoveOriginal, &mut journal, &mut claim, end)
        .is_err());
    assert_eq!(journal.native_progress_for_test(), (6, true));
    assert_eq!(trees[0].nodes[i].state, State::Original);
    println!("r6 new actual lease same paths/files/token cannot reuse previous epoch; original retained, no native write/current advance; production reacquisition requires explicit phase/release protocol");
    drop(claim);
    new_lease.release(end).unwrap();
    drop(journal);
    drop(trees);
    f.retain();
}
#[test]
fn wl2_r6_native_node_driver_work_create_empty_partial_and_preseal_keep_exact_new_handle() {
    for fault in [1, 2, 3] {
        let (mut f, _data, _paths, mut lease, mut trees, mut journal, end) = setup();
        let root = trees[0].root.clone();
        let i = trees[0]
            .nodes
            .iter()
            .position(|n| n.kind == Kind::File)
            .unwrap();
        let original = trees[0].root.join(&trees[0].nodes[i].original.relative);
        let mut claim = lease.anchored_claim(&root, end).unwrap();
        trees[0]
            .journal_step(i, FileAction::MoveOriginal, &mut journal, &mut claim, end)
            .unwrap();
        trees[0].create_fault = fault;
        let error = trees[0]
            .journal_step(i, FileAction::CreateWork, &mut journal, &mut claim, end)
            .err()
            .unwrap();
        assert!(trees[0].poisoned);
        assert_eq!(journal.native_progress_for_test(), (9, true));
        let node = &trees[0].nodes[i];
        assert_eq!(node.state, State::Moved);
        assert!(node.work.is_none());
        let pending = node
            .pending_work
            .as_ref()
            .expect("actual created object must remain owned across failure");
        let identity = id(pending, false).unwrap();
        assert_ne!(identity, node.original.id);
        let security = native_stamp(pending, end).unwrap();
        let bytes = std::fs::read(&original).unwrap();
        assert_eq!(pending.metadata().unwrap().len() as usize, bytes.len());
        assert_eq!(
            bytes.len(),
            match fault {
                1 => 0,
                2 => 1,
                _ => node.original.bytes.len(),
            }
        );
        assert_eq!(
            std::fs::read(trees[0].original_location(i)).unwrap(),
            node.original.bytes
        );
        let e = OpenOptions::new()
            .write(true)
            .open(&original)
            .err()
            .unwrap();
        assert_eq!(e.raw_os_error(), Some(32));
        println!(
            "r6-work-created-pending {}",
            serde_json::json!({"fault":fault,"volume":identity.volume,"fileId":identity.index.to_string(),"security":security,"bytes":bytes.len(),"sha256":hash::sha256_hex(&bytes),"originalCustodyPresent":true,"rawOtherWriter":32,"error":error.to_string(),"complete":false})
        );
        assert!(journal.check(end).is_err());
        drop(claim);
        lease.release(end).unwrap();
        drop(journal);
        drop(trees);
        f.retain();
    }
}
