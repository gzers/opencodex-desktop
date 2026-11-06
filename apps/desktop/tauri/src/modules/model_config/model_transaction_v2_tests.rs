use super::*;
use std::fs;

const POLICY: &str = "manager-state/model-policy.json";
const OLD_CONFIG: &[u8] = b"  {\"credential\":\"synthetic-original\"}\n";
const NEW_CONFIG: &[u8] = b"{\"credential\":\"synthetic-next\"}\r\n";
const OLD_POLICY: &[u8] = b"policy original\n";
const NEW_POLICY: &[u8] = b"policy next\n";
const OLD_DRAFT: &[u8] = b"draft original\n";
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    home: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("root");
        let home = temp.path().join("home");
        fs::create_dir_all(root.join("manager-state")).unwrap();
        fs::create_dir(&home).unwrap();
        fs::write(home.join("config.json"), OLD_CONFIG).unwrap();
        fs::write(root.join(POLICY), OLD_POLICY).unwrap();
        fs::write(root.join(DRAFT), OLD_DRAFT).unwrap();
        Self {
            _temp: temp,
            root,
            home,
        }
    }
    fn writes(&self) -> Vec<WriteV2> {
        vec![
            WriteV2 {
                target: TargetV2::OfficialConfig,
                before: Some(OLD_CONFIG.to_vec()),
                after: Some(NEW_CONFIG.to_vec()),
            },
            WriteV2 {
                target: TargetV2::Manager(POLICY.into()),
                before: Some(OLD_POLICY.to_vec()),
                after: Some(NEW_POLICY.to_vec()),
            },
            WriteV2 {
                target: TargetV2::Manager(DRAFT.into()),
                before: Some(OLD_DRAFT.to_vec()),
                after: None,
            },
        ]
    }
    fn commit(&self) -> Result<()> {
        commit(&self.root, &self.home, self.writes())
    }
    fn recover(&self) -> Result<bool> {
        recover(&self.root, &self.home)
    }
    fn pending(&self) -> PathBuf {
        self.root.join("manager-state").join(PENDING)
    }
    fn dir(&self) -> PathBuf {
        let marker: Marker = decode(&fs::read(self.pending()).unwrap()).unwrap();
        self.root
            .join("manager-state")
            .join(TRANSACTIONS)
            .join(marker.id)
    }
    fn before(&self) {
        assert!(fs::read(self.home.join("config.json")).unwrap() == OLD_CONFIG);
        assert!(fs::read(self.root.join(POLICY)).unwrap() == OLD_POLICY);
        assert!(fs::read(self.root.join(DRAFT)).unwrap() == OLD_DRAFT);
    }
    fn after(&self) {
        assert!(fs::read(self.home.join("config.json")).unwrap() == NEW_CONFIG);
        assert!(fs::read(self.root.join(POLICY)).unwrap() == NEW_POLICY);
        assert!(!self.root.join(DRAFT).exists());
    }
    fn interrupted(&self) {
        let _hook = hook(|e| {
            if e == TestEvent::AfterApply(0) {
                TestAction::Interrupt
            } else {
                TestAction::Continue
            }
        });
        code(self.commit(), "model_transaction_test_interrupted");
        assert!(self.pending().exists());
    }
    fn rewrite_journal(&self, edit: impl FnOnce(&mut Journal)) {
        let dir = self.dir();
        let mut j: Journal = decode(&fs::read(dir.join(JOURNAL)).unwrap()).unwrap();
        edit(&mut j);
        let mut m: Marker = decode(&fs::read(self.pending()).unwrap()).unwrap();
        m.plan_digest = checksum(&j.plan).unwrap();
        fs::write(dir.join(JOURNAL), envelope(&j).unwrap()).unwrap();
        fs::write(self.pending(), envelope(&m).unwrap()).unwrap();
    }
}
struct Hook;
impl Drop for Hook {
    fn drop(&mut self) {
        TEST_HOOK.with(|h| *h.borrow_mut() = None);
    }
}
fn hook(f: impl FnMut(TestEvent) -> TestAction + 'static) -> Hook {
    TEST_HOOK.with(|h| *h.borrow_mut() = Some(Box::new(f)));
    Hook
}
fn code<T>(result: Result<T>, expected: &str) {
    match result {
        Ok(_) => panic!("expected stable transaction error"),
        Err(err) => {
            assert_eq!(err.code, expected);
            let text = serde_json::to_string(&err).unwrap();
            assert!(
                !text.contains("synthetic-")
                    && !text.contains("config.json")
                    && !text.contains("/root")
            );
        }
    }
}

#[test]
fn actual_multifile_success_deletes_draft_and_keeps_exact_private_originals() {
    let f = Fixture::new();
    f.commit().unwrap();
    f.after();
    assert!(!f.pending().exists());
    assert!(!f.recover().unwrap());
    let base = f.root.join("manager-state").join(TRANSACTIONS);
    let dirs: Vec<_> = fs::read_dir(&base)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(dirs.len(), 1);
    assert!(uuid4(dirs[0].file_name().unwrap().to_str().unwrap()));
    assert!(fs::read(dirs[0].join("0.before")).unwrap() == OLD_CONFIG);
    assert!(fs::read(dirs[0].join("0.after")).unwrap() == NEW_CONFIG);
    assert!(fs::read(dirs[0].join("2.before")).unwrap() == OLD_DRAFT);
    assert!(!dirs[0].join("2.after").exists());
    let journal = fs::read(dirs[0].join(JOURNAL)).unwrap();
    assert!(!journal.windows(10).any(|w| w == b"synthetic-"));
    assert!(!String::from_utf8(journal)
        .unwrap()
        .contains(f.root.to_str().unwrap()));
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for d in [&base, &dirs[0]] {
            assert_eq!(fs::metadata(d).unwrap().permissions().mode() & 0o777, 0o700);
        }
        for e in fs::read_dir(&dirs[0]).unwrap() {
            assert_eq!(
                e.unwrap().metadata().unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        for p in [f.home.join("config.json"), f.root.join(POLICY)] {
            assert_eq!(fs::metadata(p).unwrap().permissions().mode() & 0o777, 0o600);
        }
    }
}
#[test]
fn create_absent_objects_and_distinguish_absent_from_empty() {
    let f = Fixture::new();
    let path = "manager-state/model-protocol-mappings.json";
    commit(
        &f.root,
        &f.home,
        vec![WriteV2 {
            target: TargetV2::Manager(path.into()),
            before: None,
            after: Some(vec![]),
        }],
    )
    .unwrap();
    assert!(fs::read(f.root.join(path)).unwrap().is_empty());
    code(
        commit(
            &f.root,
            &f.home,
            vec![WriteV2 {
                target: TargetV2::Manager(path.into()),
                before: None,
                after: Some(b"next".to_vec()),
            }],
        ),
        "model_transaction_cas",
    );
    f.before();
}
#[test]
fn initial_cas_has_no_business_or_preparation_side_effects() {
    let f = Fixture::new();
    let mut writes = f.writes();
    writes[1].before = None;
    code(commit(&f.root, &f.home, writes), "model_transaction_cas");
    f.before();
    assert!(!f.root.join("manager-state").join(TRANSACTIONS).exists());
    assert!(!f.pending().exists());
}
#[test]
fn confirmation_after_preparation_rechecks_every_before_without_business_writes() {
    let f = Fixture::new();
    let path = f.root.join(POLICY);
    let _hook = hook(move |e| {
        if e == TestEvent::Prepared {
            fs::write(&path, b"external-policy").unwrap();
        }
        TestAction::Continue
    });
    code(f.commit(), "model_transaction_cas");
    assert!(fs::read(f.home.join("config.json")).unwrap() == OLD_CONFIG);
    assert!(fs::read(f.root.join(POLICY)).unwrap() == b"external-policy");
    assert!(fs::read(f.root.join(DRAFT)).unwrap() == OLD_DRAFT);
    assert!(!f.pending().exists());
}
#[test]
fn intermediate_interrupt_recovers_original_plan_then_is_idempotent() {
    let f = Fixture::new();
    f.interrupted();
    assert!(fs::read(f.home.join("config.json")).unwrap() == NEW_CONFIG);
    assert!(fs::read(f.root.join(POLICY)).unwrap() == OLD_POLICY);
    code(f.commit(), "model_transaction_pending");
    assert!(f.recover().unwrap());
    f.after();
    assert!(!f.recover().unwrap());
}
#[test]
fn ordinary_write_failure_compensates_prior_writes() {
    let f = Fixture::new();
    let _hook = hook(|e| {
        if e == TestEvent::BeforeApply(1) {
            TestAction::Fail
        } else {
            TestAction::Continue
        }
    });
    code(f.commit(), "model_transaction_write");
    f.before();
    assert!(!f.pending().exists());
}
#[test]
fn deletion_failure_compensates_all_preceding_objects() {
    let f = Fixture::new();
    let _hook = hook(|e| {
        if e == TestEvent::AfterApply(2) {
            TestAction::Fail
        } else {
            TestAction::Continue
        }
    });
    code(f.commit(), "model_transaction_write");
    f.before();
    assert!(!f.pending().exists());
}
#[test]
fn creation_failure_compensation_removes_new_object() {
    let f = Fixture::new();
    let path = "manager-state/model-execution-source-ids.json";
    let writes = vec![
        WriteV2 {
            target: TargetV2::Manager(path.into()),
            before: None,
            after: Some(b"ids".to_vec()),
        },
        WriteV2 {
            target: TargetV2::OfficialConfig,
            before: Some(OLD_CONFIG.to_vec()),
            after: Some(NEW_CONFIG.to_vec()),
        },
    ];
    let _hook = hook(|e| {
        if e == TestEvent::BeforeApply(1) {
            TestAction::Fail
        } else {
            TestAction::Continue
        }
    });
    code(commit(&f.root, &f.home, writes), "model_transaction_write");
    assert!(!f.root.join(path).exists());
    f.before();
}
#[test]
fn per_file_cas_preserves_external_edit_and_compensates_other_objects() {
    let f = Fixture::new();
    let path = f.root.join(POLICY);
    let h = hook(move |e| {
        if e == TestEvent::BeforeApply(1) {
            fs::write(&path, b"external-policy").unwrap();
        }
        TestAction::Continue
    });
    code(f.commit(), "model_transaction_compensation_conflict");
    drop(h);
    assert!(fs::read(f.home.join("config.json")).unwrap() == OLD_CONFIG);
    assert!(fs::read(f.root.join(POLICY)).unwrap() == b"external-policy");
    assert!(f.pending().exists());
    code(f.recover(), "model_transaction_external_change");
    fs::write(f.root.join(POLICY), OLD_POLICY).unwrap();
    assert!(f.recover().unwrap());
    f.before();
}
#[test]
fn external_change_to_written_object_is_preserved_on_compensation() {
    let f = Fixture::new();
    let path = f.home.join("config.json");
    let h = hook(move |e| {
        if e == TestEvent::BeforeApply(1) {
            fs::write(&path, b"external-config").unwrap();
            TestAction::Fail
        } else {
            TestAction::Continue
        }
    });
    code(f.commit(), "model_transaction_compensation_conflict");
    drop(h);
    assert!(fs::read(f.home.join("config.json")).unwrap() == b"external-config");
    assert!(fs::read(f.root.join(POLICY)).unwrap() == OLD_POLICY);
    assert!(f.pending().exists());
    code(f.recover(), "model_transaction_external_change");
}
#[test]
fn recovery_preflights_all_targets_before_replaying_any() {
    let f = Fixture::new();
    f.interrupted();
    fs::write(f.root.join(DRAFT), b"external-draft").unwrap();
    code(f.recover(), "model_transaction_external_change");
    assert!(fs::read(f.root.join(POLICY)).unwrap() == OLD_POLICY);
    assert!(fs::read(f.root.join(DRAFT)).unwrap() == b"external-draft");
    assert!(f.pending().exists());
}
#[test]
fn interrupted_compensation_recovers_in_rollback_direction() {
    let f = Fixture::new();
    let h = hook(|e| match e {
        TestEvent::BeforeApply(2) => TestAction::Fail,
        TestEvent::BeforeCompensate(0) => TestAction::Interrupt,
        _ => TestAction::Continue,
    });
    code(f.commit(), "model_transaction_compensation_failed");
    drop(h);
    assert!(fs::read(f.home.join("config.json")).unwrap() == NEW_CONFIG);
    assert!(fs::read(f.root.join(POLICY)).unwrap() == OLD_POLICY);
    let j: Journal = decode(&fs::read(f.dir().join(JOURNAL)).unwrap()).unwrap();
    assert!(j.phase == Phase::RollingBack);
    assert!(f.recover().unwrap());
    f.before();
}
#[test]
fn journal_checksum_tamper_rejected_without_target_writes() {
    let f = Fixture::new();
    f.interrupted();
    let path = f.dir().join(JOURNAL);
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["value"]["phase"] = "committed".into();
    fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    code(f.recover(), "model_transaction_checksum");
    assert!(fs::read(f.root.join(POLICY)).unwrap() == OLD_POLICY);
    assert!(f.pending().exists());
}
#[test]
fn backup_tamper_rejected_even_for_an_already_applied_object() {
    let f = Fixture::new();
    f.interrupted();
    fs::write(f.dir().join("0.before"), b"tampered").unwrap();
    code(f.recover(), "model_transaction_backup_invalid");
    assert!(fs::read(f.root.join(POLICY)).unwrap() == OLD_POLICY);
    assert!(f.pending().exists());
}
#[test]
fn duplicate_nested_journal_keys_rejected_by_strict_parser() {
    let f = Fixture::new();
    f.interrupted();
    let path = f.dir().join(JOURNAL);
    let text = String::from_utf8(fs::read(&path).unwrap()).unwrap();
    let text = text.replace("\"version\":2", "\"version\":2,\"version\":2");
    fs::write(path, text).unwrap();
    code(f.recover(), "model_transaction_journal_invalid");
}
#[test]
fn duplicate_marker_keys_and_unknown_fields_rejected() {
    let f = Fixture::new();
    f.interrupted();
    let saved = fs::read(f.pending()).unwrap();
    let text = String::from_utf8(saved.clone())
        .unwrap()
        .replace("\"version\":2", "\"version\":2,\"version\":2");
    fs::write(f.pending(), text).unwrap();
    code(f.recover(), "model_transaction_journal_invalid");
    let mut value: serde_json::Value = serde_json::from_slice(&saved).unwrap();
    value["extra"] = true.into();
    fs::write(f.pending(), serde_json::to_vec(&value).unwrap()).unwrap();
    code(f.recover(), "model_transaction_journal_invalid");
}
#[test]
fn unsupported_versions_and_non_uuid4_markers_rejected() {
    let f = Fixture::new();
    f.interrupted();
    let saved = fs::read(f.pending()).unwrap();
    let mut m: Marker = decode(&saved).unwrap();
    m.version = 99;
    fs::write(f.pending(), envelope(&m).unwrap()).unwrap();
    code(f.recover(), "model_transaction_version");
    m.version = 2;
    for id in [
        "../escape",
        "550e8400-e29b-11d4-a716-446655440000",
        "550E8400-E29B-41D4-A716-446655440000",
    ] {
        m.id = id.into();
        fs::write(f.pending(), envelope(&m).unwrap()).unwrap();
        code(f.recover(), "model_transaction_journal_invalid");
    }
    fs::write(f.pending(), saved).unwrap();
    f.rewrite_journal(|j| j.version = 99);
    code(f.recover(), "model_transaction_version");
}
#[test]
fn wrong_home_and_copied_root_marker_refuse_recovery() {
    let f = Fixture::new();
    f.interrupted();
    let other = Fixture::new();
    code(recover(&f.root, &other.home), "model_transaction_binding");
    Dir::anchor(&other.root.join("manager-state"))
        .unwrap()
        .create(PENDING, &fs::read(f.pending()).unwrap())
        .unwrap();
    code(other.recover(), "model_transaction_binding");
    other.before();
    assert!(fs::read(f.root.join(POLICY)).unwrap() == OLD_POLICY);
}
#[test]
fn recovery_budget_is_checked_before_loading_any_backup() {
    let f = Fixture::new();
    f.interrupted();
    f.rewrite_journal(|j| {
        for e in &mut j.plan.entries {
            for image in [&mut e.before, &mut e.after].into_iter().flatten() {
                image.bytes = MAX_IMAGE;
            }
        }
    });
    code(f.recover(), "model_transaction_limit");
    assert!(fs::read(f.root.join(POLICY)).unwrap() == OLD_POLICY);
}
#[test]
fn oversized_images_and_too_many_targets_are_rejected_before_preparation() {
    let f = Fixture::new();
    let w = WriteV2 {
        target: TargetV2::OfficialConfig,
        before: Some(OLD_CONFIG.to_vec()),
        after: Some(vec![0; MAX_IMAGE + 1]),
    };
    code(commit(&f.root, &f.home, vec![w]), "model_transaction_limit");
    let writes = (0..9)
        .map(|_| WriteV2 {
            target: TargetV2::OfficialConfig,
            before: None,
            after: Some(vec![]),
        })
        .collect();
    code(commit(&f.root, &f.home, writes), "model_transaction_limit");
    f.before();
    assert!(!f.pending().exists());
}
#[test]
fn arbitrary_paths_duplicate_targets_and_non_draft_deletion_are_rejected() {
    let f = Fixture::new();
    for path in [
        "../model-policy.json",
        "manager-state/../config.json",
        "/tmp/model-policy.json",
        "model-policy.json",
        "manager-state/unknown.json",
        "manager-state\\model-policy.json",
    ] {
        code(
            commit(
                &f.root,
                &f.home,
                vec![WriteV2 {
                    target: TargetV2::Manager(path.into()),
                    before: None,
                    after: Some(vec![]),
                }],
            ),
            "model_transaction_target_invalid",
        );
    }
    let mut writes = f.writes();
    writes.push(WriteV2 {
        target: TargetV2::OfficialConfig,
        before: None,
        after: Some(vec![]),
    });
    code(
        commit(&f.root, &f.home, writes),
        "model_transaction_target_invalid",
    );
    code(
        commit(
            &f.root,
            &f.home,
            vec![WriteV2 {
                target: TargetV2::OfficialConfig,
                before: Some(OLD_CONFIG.to_vec()),
                after: None,
            }],
        ),
        "model_transaction_target_invalid",
    );
    f.before();
}
#[test]
fn no_pending_means_no_historical_or_old_root_scanning() {
    let f = Fixture::new();
    let history = f
        .root
        .join("manager-state")
        .join(TRANSACTIONS)
        .join("not-a-uuid");
    fs::create_dir_all(&history).unwrap();
    fs::write(history.join(JOURNAL), b"corrupt historical journal").unwrap();
    assert!(!f.recover().unwrap());
    f.before();
}
#[cfg(unix)]
#[test]
fn symlink_targets_and_directory_paths_are_rejected() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let external = f._temp.path().join("external");
    fs::write(&external, OLD_CONFIG).unwrap();
    fs::remove_file(f.home.join("config.json")).unwrap();
    symlink(&external, f.home.join("config.json")).unwrap();
    assert!(f.commit().is_err());
    assert!(fs::read(&external).unwrap() == OLD_CONFIG);
    assert!(!f.pending().exists());
    fs::remove_file(f.home.join("config.json")).unwrap();
    fs::write(f.home.join("config.json"), OLD_CONFIG).unwrap();
    fs::remove_file(f.root.join(POLICY)).unwrap();
    symlink("missing", f.root.join(POLICY)).unwrap();
    assert!(f.commit().is_err());
    let alias = f._temp.path().join("alias");
    symlink(&f.root, &alias).unwrap();
    code(
        commit(&alias, &f.home, f.writes()),
        "model_transaction_path_invalid",
    );
    let alias_home = f._temp.path().join("alias-home");
    symlink(&f.home, &alias_home).unwrap();
    code(
        recover(&f.root, &alias_home),
        "model_transaction_path_invalid",
    );
}
#[cfg(unix)]
#[test]
fn symlink_manager_directory_cannot_redirect_a_transaction() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let actual = f.root.join("actual-manager");
    fs::rename(f.root.join("manager-state"), &actual).unwrap();
    symlink(&actual, f.root.join("manager-state")).unwrap();
    assert!(f.commit().is_err());
    assert!(fs::read(actual.join("model-policy.json")).unwrap() == OLD_POLICY);
    assert!(fs::read(f.home.join("config.json")).unwrap() == OLD_CONFIG);
}
#[cfg(unix)]
#[test]
fn symlink_private_backups_and_journal_never_followed() {
    use std::os::unix::fs::symlink;
    for name in [JOURNAL, "0.before"] {
        let f = Fixture::new();
        f.interrupted();
        let dir = f.dir();
        let path = dir.join(name);
        let outside = f._temp.path().join("outside");
        fs::rename(&path, &outside).unwrap();
        symlink(&outside, &path).unwrap();
        assert!(f.recover().is_err());
        assert!(fs::read(f.root.join(POLICY)).unwrap() == OLD_POLICY);
        assert!(f.pending().exists());
    }
}
#[cfg(unix)]
#[test]
fn linked_targets_and_public_private_files_are_rejected() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    fs::hard_link(f.home.join("config.json"), f._temp.path().join("linked")).unwrap();
    code(f.commit(), "model_transaction_path_invalid");
    f.before();
    let f = Fixture::new();
    f.interrupted();
    fs::set_permissions(f.dir().join("0.before"), fs::Permissions::from_mode(0o644)).unwrap();
    code(f.recover(), "model_transaction_permissions");
}
#[cfg(unix)]
#[test]
fn home_directory_replacement_after_preparation_does_not_redirect_writes() {
    let f = Fixture::new();
    let home = f.home.clone();
    let moved = f._temp.path().join("moved-home");
    let _hook = hook(move |e| {
        if e == TestEvent::Prepared {
            fs::rename(&home, &moved).unwrap();
            fs::create_dir(&home).unwrap();
            fs::write(home.join("config.json"), OLD_CONFIG).unwrap();
        }
        TestAction::Continue
    });
    code(f.commit(), "model_transaction_binding");
    assert!(fs::read(f.home.join("config.json")).unwrap() == OLD_CONFIG);
    assert!(!f.pending().exists());
}

#[test]
fn staged_write_failure_cleans_temporary_file_and_compensates() {
    let f = Fixture::new();
    let mut armed = false;
    let h = hook(move |event| {
        if event == TestEvent::AfterApply(0) {
            armed = true;
        }
        if armed && event == TestEvent::BeforePublish {
            armed = false;
            TestAction::Fail
        } else {
            TestAction::Continue
        }
    });
    code(f.commit(), "model_transaction_write");
    drop(h);
    f.before();
    assert!(!f.pending().exists());
    for path in [&f.home, &f.root.join("manager-state")] {
        assert!(fs::read_dir(path).unwrap().all(|e| !e
            .unwrap()
            .file_name()
            .to_string_lossy()
            .ends_with(".tmp")));
    }
}
#[test]
fn ordinary_commit_requires_before_even_if_external_content_equals_after() {
    let f = Fixture::new();
    let path = f.root.join(POLICY);
    let h = hook(move |event| {
        if event == TestEvent::BeforeApply(1) {
            fs::write(&path, NEW_POLICY).unwrap();
        }
        TestAction::Continue
    });
    code(f.commit(), "model_transaction_cas");
    drop(h);
    f.before();
    assert!(!f.pending().exists());
}
#[test]
fn crash_after_all_target_writes_still_recovers_from_fixed_marker() {
    let f = Fixture::new();
    let h = hook(|event| {
        if event == TestEvent::AfterApply(2) {
            TestAction::Interrupt
        } else {
            TestAction::Continue
        }
    });
    code(f.commit(), "model_transaction_test_interrupted");
    drop(h);
    f.after();
    assert!(f.recover().unwrap());
    f.after();
    assert!(!f.pending().exists());
}
#[test]
fn crash_after_marker_before_first_write_replays_all_targets() {
    let f = Fixture::new();
    let h = hook(|event| {
        if event == TestEvent::BeforeApply(0) {
            TestAction::Interrupt
        } else {
            TestAction::Continue
        }
    });
    code(f.commit(), "model_transaction_test_interrupted");
    drop(h);
    f.before();
    assert!(f.recover().unwrap());
    f.after();
}
#[test]
fn duplicate_targets_and_unknown_targets_inside_checksummed_journal_rejected() {
    let f = Fixture::new();
    f.interrupted();
    f.rewrite_journal(|j| j.plan.entries[1].target = TargetV2::OfficialConfig);
    code(f.recover(), "model_transaction_target_invalid");
    let f = Fixture::new();
    f.interrupted();
    f.rewrite_journal(|j| {
        j.plan.entries[1].target = TargetV2::Manager("manager-state/../elsewhere".into())
    });
    code(f.recover(), "model_transaction_target_invalid");
}
#[test]
fn missing_required_backup_or_unexpected_absent_image_refuses_recovery() {
    let f = Fixture::new();
    f.interrupted();
    fs::remove_file(f.dir().join("1.after")).unwrap();
    code(f.recover(), "model_transaction_backup_invalid");
    let f = Fixture::new();
    f.interrupted();
    let c = Dir::anchor(&f.dir()).unwrap();
    c.create("2.after", b"unexpected").unwrap();
    code(f.recover(), "model_transaction_backup_invalid");
}
#[test]
fn oversized_pending_and_journal_files_are_bounded() {
    let f = Fixture::new();
    f.interrupted();
    fs::write(f.dir().join(JOURNAL), vec![0; MAX_JOURNAL + 1]).unwrap();
    code(f.recover(), "model_transaction_limit");
    let f = Fixture::new();
    f.interrupted();
    fs::write(f.pending(), vec![0; MAX_MARKER + 1]).unwrap();
    code(f.recover(), "model_transaction_limit");
}
#[cfg(unix)]
#[test]
fn pending_and_transaction_directory_symlinks_are_rejected() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    f.interrupted();
    let outside = f._temp.path().join("outside-marker");
    fs::rename(f.pending(), &outside).unwrap();
    symlink(&outside, f.pending()).unwrap();
    assert!(f.recover().is_err());
    let f = Fixture::new();
    f.interrupted();
    let base = f.root.join("manager-state").join(TRANSACTIONS);
    let outside = f._temp.path().join("outside-transactions");
    fs::rename(&base, &outside).unwrap();
    symlink(&outside, &base).unwrap();
    assert!(f.recover().is_err());
    assert!(fs::read(f.root.join(POLICY)).unwrap() == OLD_POLICY);
}

#[test]
fn eight_whitelisted_targets_can_participate_in_one_transaction() {
    let f = Fixture::new();
    let mut writes = vec![WriteV2 {
        target: TargetV2::OfficialConfig,
        before: Some(OLD_CONFIG.to_vec()),
        after: Some(NEW_CONFIG.to_vec()),
    }];
    for &name in &MANAGERS[..MAX_FILES - 1] {
        let before = fs::read(f.root.join(name)).ok();
        writes.push(WriteV2 {
            target: TargetV2::Manager(name.into()),
            before,
            after: Some(b"eight-object-image".to_vec()),
        });
    }
    assert_eq!(writes.len(), MAX_FILES);
    commit(&f.root, &f.home, writes).unwrap();
    for &name in &MANAGERS[..MAX_FILES - 1] {
        assert!(fs::read(f.root.join(name)).unwrap() == b"eight-object-image");
    }
    assert!(!f.pending().exists());
}
#[cfg(unix)]
#[test]
fn public_transaction_directories_are_rejected_without_business_writes() {
    use std::os::unix::fs::PermissionsExt;
    let f = Fixture::new();
    let base = f.root.join("manager-state").join(TRANSACTIONS);
    fs::create_dir(&base).unwrap();
    fs::set_permissions(&base, fs::Permissions::from_mode(0o755)).unwrap();
    code(f.commit(), "model_transaction_permissions");
    f.before();
    assert!(!f.pending().exists());
}

const VERSIONS: &str = "manager-state/saved-content-versions.json";
const OLD_VERSIONS: &[u8] = b"  {\"version\":1}\r\n";
const NEW_VERSIONS: &[u8] = b"{\"version\":2}\n";

#[test]
fn versions_file_participates_in_six_file_interrupted_save_and_exact_recovery() {
    for interrupted_after in [0, 2, 4] {
        let f = Fixture::new();
        fs::write(f.root.join(VERSIONS), OLD_VERSIONS).unwrap();
        let mut writes = f.writes();
        writes.insert(
            2,
            WriteV2 {
                target: TargetV2::Manager(VERSIONS.into()),
                before: Some(OLD_VERSIONS.to_vec()),
                after: Some(NEW_VERSIONS.to_vec()),
            },
        );
        for path in [
            "manager-state/model-materialization-ledger.json",
            "manager-state/model-protocol-mappings.json",
        ] {
            writes.push(WriteV2 {
                target: TargetV2::Manager(path.into()),
                before: None,
                after: Some(b"fixed-image".to_vec()),
            });
        }
        assert_eq!(writes.len(), 6);
        let h = hook(move |e| {
            if e == TestEvent::AfterApply(interrupted_after) {
                TestAction::Interrupt
            } else {
                TestAction::Continue
            }
        });
        code(
            commit(&f.root, &f.home, writes),
            "model_transaction_test_interrupted",
        );
        drop(h);
        let expected = if interrupted_after < 2 {
            OLD_VERSIONS
        } else {
            NEW_VERSIONS
        };
        assert!(fs::read(f.root.join(VERSIONS)).unwrap() == expected);
        let dir = f.dir();
        assert!(f.recover().unwrap());
        f.after();
        assert!(fs::read(f.root.join(VERSIONS)).unwrap() == NEW_VERSIONS);
        assert!(fs::read(dir.join("2.before")).unwrap() == OLD_VERSIONS);
        for path in [
            "manager-state/model-materialization-ledger.json",
            "manager-state/model-protocol-mappings.json",
        ] {
            assert!(fs::read(f.root.join(path)).unwrap() == b"fixed-image");
        }
        assert!(!f.pending().exists());
        assert!(!f.recover().unwrap());
    }
}
#[test]
fn versions_file_stale_before_rejects_save_without_preparation() {
    let f = Fixture::new();
    fs::write(f.root.join(VERSIONS), OLD_VERSIONS).unwrap();
    let mut writes = f.writes();
    writes.push(WriteV2 {
        target: TargetV2::Manager(VERSIONS.into()),
        before: None,
        after: Some(NEW_VERSIONS.to_vec()),
    });
    code(commit(&f.root, &f.home, writes), "model_transaction_cas");
    f.before();
    assert!(fs::read(f.root.join(VERSIONS)).unwrap() == OLD_VERSIONS);
    assert!(!f.root.join("manager-state").join(TRANSACTIONS).exists());
}
#[test]
fn versions_file_per_file_cas_preserves_external_edit_and_compensates_config() {
    let f = Fixture::new();
    fs::write(f.root.join(VERSIONS), OLD_VERSIONS).unwrap();
    let mut writes = f.writes();
    writes.insert(
        1,
        WriteV2 {
            target: TargetV2::Manager(VERSIONS.into()),
            before: Some(OLD_VERSIONS.to_vec()),
            after: Some(NEW_VERSIONS.to_vec()),
        },
    );
    let path = f.root.join(VERSIONS);
    let h = hook(move |e| {
        if e == TestEvent::BeforeApply(1) {
            fs::write(&path, b"external-versions").unwrap();
        }
        TestAction::Continue
    });
    code(
        commit(&f.root, &f.home, writes),
        "model_transaction_compensation_conflict",
    );
    drop(h);
    f.before();
    assert!(fs::read(f.root.join(VERSIONS)).unwrap() == b"external-versions");
    code(f.recover(), "model_transaction_external_change");
    assert!(fs::read(f.root.join(VERSIONS)).unwrap() == b"external-versions");
    fs::write(f.root.join(VERSIONS), OLD_VERSIONS).unwrap();
    assert!(f.recover().unwrap());
    f.before();
    assert!(fs::read(f.root.join(VERSIONS)).unwrap() == OLD_VERSIONS);
}
#[test]
fn versions_file_deletion_is_rejected_in_commit_and_checksummed_recovery() {
    let f = Fixture::new();
    fs::write(f.root.join(VERSIONS), OLD_VERSIONS).unwrap();
    code(
        commit(
            &f.root,
            &f.home,
            vec![WriteV2 {
                target: TargetV2::Manager(VERSIONS.into()),
                before: Some(OLD_VERSIONS.to_vec()),
                after: None,
            }],
        ),
        "model_transaction_target_invalid",
    );
    f.before();
    assert!(fs::read(f.root.join(VERSIONS)).unwrap() == OLD_VERSIONS);
    assert!(!f.pending().exists());
    f.interrupted();
    f.rewrite_journal(|j| {
        j.plan.entries[2].target = TargetV2::Manager(VERSIONS.into());
    });
    code(f.recover(), "model_transaction_target_invalid");
    assert!(fs::read(f.root.join(VERSIONS)).unwrap() == OLD_VERSIONS);
    assert!(fs::read(f.root.join(DRAFT)).unwrap() == OLD_DRAFT);
    assert!(fs::read(f.root.join(POLICY)).unwrap() == OLD_POLICY);
    assert!(f.pending().exists());
}
