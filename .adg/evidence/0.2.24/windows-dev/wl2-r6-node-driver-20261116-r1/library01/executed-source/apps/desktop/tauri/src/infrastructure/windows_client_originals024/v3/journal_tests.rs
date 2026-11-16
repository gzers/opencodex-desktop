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
