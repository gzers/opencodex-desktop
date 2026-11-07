//! Fixed event/time fixtures, isolated roots; no config reads or clock guesses.
use super::*;
use crate::infrastructure::locking::{lock_path_for, TargetFileLock};
use std::{path::Path, time::Duration};
const EVENT: &str = "11111111-1111-4111-8111-111111111111";
const SECOND: &str = "22222222-2222-4222-8222-222222222222";
const THIRD: &str = "33333333-3333-4333-8333-333333333333";
fn state(value: &str) -> SaveState {
    SaveState::from([("model:stable-id".into(), value.into())])
}
fn bytes(root: &Path) -> Option<Vec<u8>> {
    std::fs::read(root.join(FILE)).ok()
}
fn install(root: &Path, contents: &[u8]) {
    std::fs::create_dir_all(root.join("manager-state")).unwrap();
    std::fs::write(root.join(FILE), contents).unwrap();
}
fn planned(
    root: &Path,
    before: &SaveState,
    after: &SaveState,
    time: &str,
    event: &str,
) -> ModelResult<PlannedVersionsV2> {
    let _lock = lock_versions(root)?;
    plan_save_at(root, before, after, time, event)
}
fn versions(contents: &[u8]) -> BTreeMap<String, Version> {
    serde_json::from_slice(contents).unwrap()
}
#[test]
fn plan_is_byte_identical_to_old_write_across_save_replay_and_clock_rollback() {
    let old = tempfile::tempdir().unwrap();
    let planned_root = tempfile::tempdir().unwrap();
    let empty = SaveState::new();
    let a = state("a");
    let b = state("b");
    let c = state("c");
    let d = state("d");
    let e = state("e");
    let events = [
        (&empty, &a, "2026-11-06T12:00:00Z", EVENT),
        (&empty, &a, "2026-11-06T12:00:00Z", EVENT),
        (&a, &b, "2026-11-06T10:00:00Z", SECOND),
        (&b, &c, "2026-11-06T11:00:00Z", THIRD),
        (
            &c,
            &d,
            "2026-11-06T12:00:00Z",
            "44444444-4444-4444-8444-444444444444",
        ),
        (
            &d,
            &e,
            "2026-11-06T12:00:01Z",
            "55555555-5555-4555-8555-555555555555",
        ),
    ];
    for (before, after, time, event) in events {
        let original = bytes(planned_root.path());
        let plan = planned(planned_root.path(), before, after, time, event).unwrap();
        assert_eq!(plan.before, original);
        assert_eq!(bytes(planned_root.path()), original);
        record_at_validated(old.path(), before, after, time, event, || Ok(())).unwrap();
        assert_eq!(bytes(old.path()).as_deref(), Some(plan.after.as_slice()));
        install(planned_root.path(), &plan.after);
    }
    assert_eq!(
        observe(old.path(), &e).unwrap(),
        observe(planned_root.path(), &e).unwrap()
    );
    let record = &observe(planned_root.path(), &e).unwrap()[0];
    assert_eq!(record.time_kind, TimeKind::Saved);
    assert_eq!(record.revision, 5);
    assert_eq!(record.saved_at, "2026-11-06T12:00:01Z");
}
#[test]
fn no_op_does_not_create_a_time_increment_revision_or_normalize_old_bytes() {
    let root = tempfile::tempdir().unwrap();
    let same = state("first-observation");
    let plan = planned(root.path(), &same, &same, "2026-11-06T12:00:00Z", EVENT).unwrap();
    assert_eq!(plan.before, None);
    assert_eq!(plan.after, b"{}");
    assert_eq!(bytes(root.path()), None);
    assert_eq!(
        observe(root.path(), &same).unwrap()[0].time_kind,
        TimeKind::Unknown
    );
    let old=br#"{ "model:stable-id": { "digest":"legacy", "revision": 7, "saved_at":"2026-11-06T01:00:00Z" } }"#;
    install(root.path(), old);
    let same = state("legacy");
    let plan = planned(root.path(), &same, &same, "2026-11-06T13:00:00Z", SECOND).unwrap();
    assert_eq!(plan.before.as_deref(), Some(old.as_slice()));
    assert_eq!(plan.after, old);
    assert_eq!(bytes(root.path()).unwrap(), old);
    let actual = observe(root.path(), &same).unwrap();
    assert_eq!(actual[0].revision, 7);
    assert_eq!(actual[0].time_kind, TimeKind::Unknown);
    assert!(actual[0].instant().is_none());
}
#[test]
fn event_replay_is_frozen_and_conflicts_match_the_old_api() {
    let root = tempfile::tempdir().unwrap();
    let empty = SaveState::new();
    let a = state("a");
    let b = state("b");
    let time = "2026-11-06T12:00:00Z";
    let first = planned(root.path(), &empty, &a, time, EVENT).unwrap();
    install(root.path(), &first.after);
    let replay = planned(root.path(), &empty, &a, time, EVENT).unwrap();
    assert_eq!(replay.after, first.after);
    assert_eq!(versions(&replay.after)["model:stable-id"].revision, 1);
    for (after, time) in [(&b, time), (&a, "2026-11-06T12:00:01Z")] {
        let new_error = planned(root.path(), &empty, after, time, EVENT).unwrap_err();
        let old_error =
            record_at_validated(root.path(), &empty, after, time, EVENT, || Ok(())).unwrap_err();
        assert_eq!(new_error.code, old_error.code);
        assert_eq!(new_error.code, "content_version_event_conflict");
        assert_eq!(bytes(root.path()).unwrap(), first.after);
    }
    // Restoring previously saved bytes after an external change is a new event.
    let restored = planned(root.path(), &b, &a, time, SECOND).unwrap();
    assert_eq!(versions(&restored.after)["model:stable-id"].revision, 2);
    assert_eq!(bytes(root.path()).unwrap(), first.after);
}
#[test]
fn deletion_and_recreation_keep_clock_floor_and_original_deletion_time_rules() {
    let old = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let empty = SaveState::new();
    let a = state("a");
    let b = state("b");
    let c = state("c");
    let events = [
        (&empty, &a, "2026-11-06T12:00:00Z", EVENT),
        (&a, &empty, "2026-11-06T13:00:00Z", SECOND),
        (&a, &empty, "2026-11-06T13:00:00Z", SECOND),
        (&empty, &b, "2026-11-06T11:00:00Z", THIRD),
        (
            &b,
            &c,
            "2026-11-06T12:00:00Z",
            "44444444-4444-4444-8444-444444444444",
        ),
    ];
    for (before, after, time, event) in events {
        let plan = planned(root.path(), before, after, time, event).unwrap();
        record_at_validated(old.path(), before, after, time, event, || Ok(())).unwrap();
        assert_eq!(bytes(old.path()).as_deref(), Some(plan.after.as_slice()));
        install(root.path(), &plan.after);
        let record = &versions(&plan.after)["model:stable-id"];
        assert_eq!(
            record.trusted_floor.as_deref(),
            Some("2026-11-06T12:00:00+00:00")
        );
        if after.is_empty() {
            assert_eq!(record.saved_at, "2026-11-06T12:00:00Z");
            assert_eq!(record.content_digest, "");
            assert_eq!(record.revision, 2);
        }
    }
    let record = &observe(root.path(), &c).unwrap()[0];
    assert_eq!(record.revision, 4);
    assert_eq!(record.time_kind, TimeKind::ClockUnreliable);
}
#[test]
fn strict_planning_rejects_nested_or_scope_duplicates_and_invalid_types_without_writes() {
    let invalid = [
        br#"{"scope":{},"scope":{}}"#.as_slice(),
        br#"{"scope":{"contentDigest":"a","contentDigest":"b","savedAt":"2026-11-06T00:00:00Z"}}"#
            .as_slice(),
        br#"{"scope":{"contentDigest":false,"savedAt":"time"}}"#.as_slice(),
        br#"[]"#.as_slice(),
        br#"{"scope":{"digest":"a","saved_at":"old","future":true}}"#.as_slice(),
    ];
    for raw in invalid {
        let root = tempfile::tempdir().unwrap();
        install(root.path(), raw);
        assert!(planned(
            root.path(),
            &SaveState::new(),
            &state("a"),
            "2026-11-06T12:00:00Z",
            EVENT
        )
        .is_err());
        assert_eq!(bytes(root.path()).unwrap(), raw);
    }
    let root = tempfile::tempdir().unwrap();
    let raw = br#"{"scope":{},"scope":{}}"#;
    install(root.path(), raw);
    assert_eq!(
        planned(
            root.path(),
            &state("a"),
            &state("a"),
            "2026-11-06T12:00:00Z",
            EVENT
        )
        .unwrap_err()
        .code,
        "model_json_duplicate_key"
    );
    assert_eq!(bytes(root.path()).unwrap(), raw);
}
#[test]
fn version_lock_is_the_old_target_lock_and_planning_does_not_reenter_it_or_write() {
    let root = tempfile::tempdir().unwrap();
    let guard = lock_versions(root.path()).unwrap();
    assert!(root
        .path()
        .join("manager-state/saved-content-versions.json.lock")
        .exists());
    assert!(
        TargetFileLock::try_lock_with_timeout(&root.path().join(FILE), Duration::ZERO).is_err()
    );
    let before = std::fs::read_dir(root.path().join("manager-state"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    let first = plan_save_at(
        root.path(),
        &SaveState::new(),
        &state("a"),
        "2026-11-06T12:00:00Z",
        EVENT,
    )
    .unwrap();
    let again = plan_save_at(
        root.path(),
        &SaveState::new(),
        &state("a"),
        "2026-11-06T12:00:00Z",
        EVENT,
    )
    .unwrap();
    assert_eq!(first, again);
    assert_eq!(bytes(root.path()), None);
    let after = std::fs::read_dir(root.path().join("manager-state"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect::<Vec<_>>();
    assert_eq!(before, after);
    drop(guard);
    TargetFileLock::try_lock_with_timeout(&root.path().join(FILE), Duration::ZERO).unwrap();
    assert_eq!(
        lock_path_for(&root.path().join(FILE)),
        root.path()
            .join("manager-state/saved-content-versions.json.lock")
    );
}
#[test]
fn legacy_scope_alias_is_normalized_only_when_a_real_change_is_planned() {
    let old = tempfile::tempdir().unwrap();
    let root = tempfile::tempdir().unwrap();
    let raw =
        br#"{"model:stable-id":{"digest":"a","revision":3,"saved_at":"2026-11-06T01:00:00Z"}}"#;
    install(old.path(), raw);
    install(root.path(), raw);
    let plan = planned(
        root.path(),
        &state("a"),
        &state("b"),
        "2026-11-06T02:00:00Z",
        EVENT,
    )
    .unwrap();
    record_at_validated(
        old.path(),
        &state("a"),
        &state("b"),
        "2026-11-06T02:00:00Z",
        EVENT,
        || Ok(()),
    )
    .unwrap();
    assert_eq!(Some(&plan.after), bytes(old.path()).as_ref());
    assert_eq!(bytes(root.path()).unwrap(), raw);
    let record = &versions(&plan.after)["model:stable-id"];
    assert_eq!(record.scope_key, "model:stable-id");
    assert_eq!(record.revision, 4);
    assert_eq!(record.time_kind, TimeKind::Saved);
}
#[test]
fn event_time_validation_and_old_callback_lock_order_remain_unchanged() {
    let root = tempfile::tempdir().unwrap();
    for (time, event) in [("invalid", EVENT), ("2026-11-06T12:00:00Z", "invalid")] {
        assert!(planned(root.path(), &SaveState::new(), &state("a"), time, event).is_err());
        assert_eq!(bytes(root.path()), None);
    }
    let mut called = false;
    record_at_validated(
        root.path(),
        &SaveState::new(),
        &state("a"),
        "2026-11-06T12:00:00Z",
        EVENT,
        || {
            called = true;
            assert!(
                TargetFileLock::try_lock_with_timeout(&root.path().join(FILE), Duration::ZERO)
                    .is_err()
            );
            Ok(())
        },
    )
    .unwrap();
    assert!(called);
    let saved = bytes(root.path());
    assert!(record_at_validated(
        root.path(),
        &state("a"),
        &state("b"),
        "2026-11-06T13:00:00Z",
        SECOND,
        || Err(invalid("fixture_rejected", "test"))
    )
    .is_err());
    assert_eq!(bytes(root.path()), saved);
    // Original no-op path calls validation directly and creates no version lock.
    let empty = tempfile::tempdir().unwrap();
    record_at_validated(
        empty.path(),
        &state("a"),
        &state("a"),
        "2026-11-06T12:00:00Z",
        THIRD,
        || {
            assert!(!empty.path().join("manager-state").exists());
            Ok(())
        },
    )
    .unwrap();
    assert!(!empty.path().join("manager-state").exists());
}
