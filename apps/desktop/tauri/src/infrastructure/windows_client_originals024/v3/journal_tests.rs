use super::*;
use crate::infrastructure::windows_official_runtime024::create_private_directory;
use std::time::Duration;
struct CrossData(PathBuf);
impl Drop for CrossData {
    fn drop(&mut self) {
        println!("r4-retained-synthetic-fixture={}", self.0.display());
    }
}
fn setup() -> (
    super::super::super::tree::tests::Fixture,
    CrossData,
    Journal,
) {
    let (f, plan, profile, parent) = super::super::tests::plan();
    let path = PathBuf::from(
        "D:/Users/15119/WorkSpace/Code/tools/opencodex-desktop/test/windows024-native/target",
    )
    .join(format!("r6-v3-{}", uuid::Uuid::new_v4()));
    let data = create_private_directory(&path).unwrap();
    let retained = CrossData(path);
    assert_ne!(
        data.identity().volume,
        parent.volume,
        "test requires actual distinct native volumes; no letter-only claim"
    );
    let pins = [
        PinnedDirectory::open_private(&profile).unwrap(),
        data,
        create_private_directory(&f.path().join("H")).unwrap(),
        create_private_directory(&f.path().join("I")).unwrap(),
    ];
    let directory = create_private_directory(&pins[1].path().join("session")).unwrap();
    let peer = RetainedPeer::current().unwrap();
    let binding = Binding::from_native(
        &peer,
        &pins,
        hash::sha256_hex(b"synthetic source, not official admission"),
        hash::sha256_hex(b"synthetic current"),
    )
    .unwrap();
    let end = Instant::now() + Duration::from_secs(10);
    let journal = Journal::create(
        directory,
        pins,
        peer,
        binding,
        plan,
        &uuid::Uuid::new_v4().to_string(),
        &"c".repeat(64),
        &uuid::Uuid::new_v4().to_string(),
        end,
    )
    .unwrap();
    (f, retained, journal)
}
#[test]
fn wl2_r6_capture_canonical_data_namespace_binds_native_session_leaf_and_rejects_other_descendants()
{
    let (mut f, _data, journal) = setup();
    let end = journal.deadline;
    let data = journal.pins[1].path();
    let official = create_private_directory(&data.join("official024")).unwrap();
    let windows = create_private_directory(&official.path().join("windows")).unwrap();
    let session = uuid::Uuid::new_v4().to_string();
    let directory = create_private_directory(&windows.path().join(&session)).unwrap();
    assert_eq!(
        session_namespace(&directory, &journal.pins[1]).unwrap(),
        Some(session.clone())
    );
    let peer = RetainedPeer::current().unwrap();
    let pins = [
        PinnedDirectory::open(journal.pins[0].path()).unwrap(),
        PinnedDirectory::open_private(data).unwrap(),
        PinnedDirectory::open(journal.pins[2].path()).unwrap(),
        PinnedDirectory::open(journal.pins[3].path()).unwrap(),
    ];
    let binding = Binding::from_native(
        &peer,
        &pins,
        hash::sha256_hex(b"synthetic namespace source, not production admission"),
        hash::sha256_hex(b"synthetic namespace current"),
    )
    .unwrap();
    let plan: StoragePlan =
        serde_json::from_slice(&encoded(&journal.manifest.storage_plan).unwrap()).unwrap();
    let mut canonical = Journal::create(
        directory,
        pins,
        peer,
        binding,
        plan,
        &uuid::Uuid::new_v4().to_string(),
        &"c".repeat(64),
        &uuid::Uuid::new_v4().to_string(),
        end,
    )
    .unwrap();
    canonical.check(end).unwrap();
    assert_eq!(canonical.manifest.session_id, session);
    assert_eq!(canonical.progress.session, session);
    let wrong = create_private_directory(&data.join("wrong-parent")).unwrap();
    let leaf =
        create_private_directory(&wrong.path().join(uuid::Uuid::new_v4().to_string())).unwrap();
    assert!(session_namespace(&leaf, &journal.pins[1]).is_err());
    let invalid = create_private_directory(&windows.path().join("not-a-session")).unwrap();
    assert!(session_namespace(&invalid, &journal.pins[1]).is_err());
    println!("r6 actual native D/official024/windows/canonicalUUID journal uses same UUID in immutable manifest/current; wrong descendant/nonUUID refuse, direct D/session remains component-only, actual Source create factory still requires production namespace and original Owner");
    drop(canonical);
    drop(invalid);
    drop(leaf);
    drop(wrong);
    drop(windows);
    drop(official);
    drop(journal);
    f.retain();
}
#[test]
fn wl2_r6_v3_native_journal_manifest_event_current_and_other_writer_refusal() {
    let (mut f, _data, mut journal) = setup();
    let end = journal.deadline;
    let node = journal.manifest.storage_plan.roots[0]
        .nodes
        .iter()
        .find(|n| n.kind == Kind::CustodiedFile)
        .unwrap();
    let event = super::super::tests::event(
        &journal.manifest.storage_plan,
        &journal.progress,
        node,
        Operation::NodeMoveIntent,
        "999",
    );
    let old_id = journal.current_id;
    let directory = journal.directory.path().to_path_buf();
    journal.append(event, end).unwrap();
    journal.check(end).unwrap();
    assert_eq!(journal.progress.sequence, 1);
    assert_eq!(journal.current_id, old_id);
    let current: Current =
        serde_json::from_slice(&std::fs::read(directory.join("current.json")).unwrap()).unwrap();
    assert_eq!(current.schema, ANCHORED_CURRENT_SCHEMA_V3);
    assert_eq!(current.sequence, 1);
    let e = OpenOptions::new()
        .read(true)
        .write(true)
        .open(directory.join("current.json"))
        .err()
        .unwrap();
    assert_eq!(e.raw_os_error(), Some(32));
    assert!(directory.join("event-000001.json").exists());
    assert!(!journal
        .progress
        .complete_files(&journal.manifest.storage_plan));
    println!("r6 native manifest/event immutable sync/readback and current same-ID CAS pass; actual D separate volume; second current writer raw32. This is data journal only, original tree not mutated, no recovery capability.");
    drop(journal);
    f.retain();
}
#[test]
fn wl2_r6_v3_native_journal_event_or_partial_current_failure_poison_without_catchup() {
    for fault in [1, 2] {
        let (mut f, _data, mut journal) = setup();
        let end = journal.deadline;
        let directory = journal.directory.path().to_path_buf();
        let node = journal.manifest.storage_plan.roots[0]
            .nodes
            .iter()
            .find(|n| n.kind == Kind::CustodiedFile)
            .unwrap();
        let event = super::super::tests::event(
            &journal.manifest.storage_plan,
            &journal.progress,
            node,
            Operation::NodeMoveIntent,
            "999",
        );
        journal.fault = fault;
        let error = journal.append(event, end).err().unwrap();
        println!("r6 durable injected fault={fault}: {error}");
        assert!(journal.poisoned);
        assert_eq!(journal.progress.sequence, 0);
        assert!(directory.join("event-000001.json").exists());
        assert!(journal.check(end).is_err());
        journal.fault = 0;
        assert!(
            journal.check(end).is_err(),
            "no automatic repair of partial current"
        );
        let bytes = std::fs::read(directory.join("current.json")).unwrap();
        if fault == 1 {
            let c: Current = serde_json::from_slice(&bytes).unwrap();
            assert_eq!(c.sequence, 0);
        } else {
            assert_eq!(bytes, b"{");
        }
        drop(journal);
        f.retain();
    }
}
#[test]
fn wl2_r6_v3_native_journal_unknown_node_and_late_deadline_poison() {
    for foreign in [false, true] {
        let (mut f, _data, mut journal) = setup();
        let end = journal.deadline;
        if foreign {
            std::fs::create_dir(journal.directory.path().join("unknown-empty")).unwrap();
            assert!(journal.check(end).is_err());
        } else {
            assert!(journal.check(end + Duration::from_millis(1)).is_err());
        }
        assert!(journal.poisoned);
        assert!(journal.check(end).is_err());
        assert_eq!(journal.progress.sequence, 0);
        drop(journal);
        f.retain();
    }
}
