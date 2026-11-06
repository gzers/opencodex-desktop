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

fn retained_ref(n: u128) -> schema_v2::RefV2 {
    schema_v2::RefV2 {
        template_id: uuid::Uuid::from_u128(0x550e8400_e29b_41d4_a716_446655440000 + n).to_string(),
        revision_id: uuid::Uuid::from_u128(0x660e8400_e29b_41d4_a716_446655440000 + n).to_string(),
        digest: schema_v2::digest(&n.to_le_bytes()),
    }
}
fn retained_policy(
    global: &[schema_v2::RefV2],
    channel: &[schema_v2::RefV2],
    manual: Option<&schema_v2::RefV2>,
    auto: Option<&schema_v2::RefV2>,
) -> serde_json::Value {
    serde_json::json!({
        "schemaVersion": 2, "templateRefs": global, "defaultChannelId": null,
        "pendingRemovals": [], "recoveryId": null,
        "channels": [{"id": "550e8400-e29b-41d4-a716-446655440000", "label": "retained channel",
            "templateRefs": channel, "models": [
                {"rawModelId": "manual-model", "binding": manual.map(|reference| serde_json::json!({"kind":"manual", "reference":reference}))},
                {"rawModelId": "auto-model", "binding": auto.map(|reference| serde_json::json!({"kind":"auto_read", "reference":reference}))}
            ]}]
    })
}
fn retained_draft(policy: serde_json::Value, baseline: serde_json::Value) -> serde_json::Value {
    serde_json::json!({"schemaVersion":2, "base":{"dtoVersion":2}, "policy":policy, "observedBaseline":baseline})
}
fn retained_write(
    path: &str,
    before: Option<serde_json::Value>,
    after: Option<serde_json::Value>,
) -> WriteV2 {
    WriteV2 {
        target: TargetV2::Manager(path.into()),
        before: before.map(|v| serde_json::to_vec(&v).unwrap()),
        after: after.map(|v| serde_json::to_vec(&v).unwrap()),
    }
}
// Produce real, protected exact-image history without changing business targets.
// Historical scans intentionally do not require a pending marker or live CAS.
fn retained_history(f: &Fixture, writes: Vec<WriteV2>, phase: Phase) -> PathBuf {
    let c = Context::open(&f.root, &f.home).unwrap();
    let base = c
        .manager()
        .unwrap()
        .child(TRANSACTIONS, true, true)
        .unwrap()
        .unwrap();
    let id = uuid::Uuid::new_v4().to_string();
    let dir = base.child(&id, true, true).unwrap().unwrap();
    let rollback_count = if matches!(phase, Phase::RollingBack | Phase::RolledBack) {
        writes.len()
    } else {
        0
    };
    let journal = Journal {
        version: 2,
        plan: Plan {
            protected_restore: false,
            id,
            binding: c.binding().unwrap(),
            entries: writes
                .iter()
                .map(|w| Entry {
                    target: w.target.clone(),
                    before: image(&w.before),
                    after: image(&w.after),
                })
                .collect(),
        },
        phase,
        rollback_count,
    };
    for (i, w) in writes.iter().enumerate() {
        for (before, bytes) in [(true, &w.before), (false, &w.after)] {
            if let Some(bytes) = bytes {
                dir.create(&image_name(i, before), bytes).unwrap();
            }
        }
    }
    save_journal(&dir, &journal).unwrap();
    dir.path.clone()
}
fn retained_refs(f: &Fixture) -> Result<Vec<schema_v2::RefV2>> {
    retained_template_refs(&f.root, &f.home)
}
fn retained_rewrite(dir: &Path, edit: impl FnOnce(&mut Journal)) {
    let dir = Dir::anchor(dir).unwrap();
    let bytes = dir.read(JOURNAL, MAX_JOURNAL, true).unwrap().unwrap();
    let mut j: Journal = decode(&bytes).unwrap();
    edit(&mut j);
    save_journal(&dir, &j).unwrap();
}

#[test]
fn retained_refs_return_sorted_exact_global_channel_manual_auto_and_draft_images() {
    let f = Fixture::new();
    let refs: Vec<_> = (1..=9).map(retained_ref).collect();
    let mut same_ids_new_digest = refs[0].clone();
    same_ids_new_digest.digest = schema_v2::digest(b"different exact revision content");
    let before = retained_policy(
        &[refs[3].clone(), refs[0].clone()],
        &[refs[2].clone()],
        Some(&refs[1]),
        Some(&refs[4]),
    );
    let after = retained_policy(
        &[same_ids_new_digest.clone(), refs[0].clone()],
        &[],
        None,
        Some(&refs[5]),
    );
    let draft = retained_draft(
        retained_policy(&[refs[6].clone()], &[], None, Some(&refs[7])),
        retained_policy(&[refs[8].clone()], &[], None, None),
    );
    retained_history(
        &f,
        vec![
            retained_write(POLICY, Some(before), Some(after)),
            retained_write(DRAFT, Some(draft), None),
        ],
        Phase::Committed,
    );
    let mut expected = refs;
    expected.push(same_ids_new_digest);
    expected.sort();
    assert_eq!(retained_refs(&f).unwrap(), expected);
    assert!(!f.pending().exists());
    f.before();
}
#[test]
fn retained_refs_ignore_unadopted_library_revisions_parents_and_decoy_nested_refs() {
    let f = Fixture::new();
    let adopted = retained_ref(1);
    let unrelated = retained_ref(2);
    let library = serde_json::json!({"schemaVersion":2,"templates":[{"id":unrelated.template_id,"revisions":[{"id":unrelated.revision_id,"digest":unrelated.digest,"baseRevision":retained_ref(3).revision_id}]}]});
    retained_history(
        &f,
        vec![retained_write(
            "manager-state/model-family-templates.json",
            Some(library.clone()),
            Some(library),
        )],
        Phase::Committed,
    );
    assert!(retained_refs(&f).unwrap().is_empty());
    let mut policy = retained_policy(std::slice::from_ref(&adopted), &[], None, None);
    policy["channels"][0]["models"][0]["source"] = serde_json::json!({"publicContent":{"reference":unrelated,"templateRefs":[retained_ref(4)]}});
    retained_history(
        &f,
        vec![retained_write(POLICY, None, Some(policy))],
        Phase::Committed,
    );
    assert_eq!(retained_refs(&f).unwrap(), vec![adopted]);
}
#[test]
fn retained_refs_protect_both_images_in_every_valid_phase() {
    for phase in [
        Phase::Applying,
        Phase::Committed,
        Phase::RollingBack,
        Phase::RolledBack,
    ] {
        let f = Fixture::new();
        let a = retained_ref(1);
        let b = retained_ref(2);
        retained_history(
            &f,
            vec![retained_write(
                POLICY,
                Some(retained_policy(std::slice::from_ref(&a), &[], None, None)),
                Some(retained_policy(std::slice::from_ref(&b), &[], None, None)),
            )],
            phase,
        );
        assert_eq!(retained_refs(&f).unwrap(), vec![a, b]);
        f.before();
        assert!(!f.pending().exists());
    }
}
#[test]
fn retained_refs_are_available_after_real_commit_rollback_and_pre_marker_interruption() {
    for state in 0..3 {
        let f = Fixture::new();
        let a = retained_ref(1);
        let b = retained_ref(2);
        let before = retained_policy(std::slice::from_ref(&a), &[], None, None);
        let after = retained_policy(std::slice::from_ref(&b), &[], None, None);
        fs::write(f.root.join(POLICY), serde_json::to_vec(&before).unwrap()).unwrap();
        let writes = vec![retained_write(POLICY, Some(before), Some(after))];
        let h = hook(move |e| match (state, e) {
            (1, TestEvent::AfterApply(0)) => TestAction::Fail,
            (2, TestEvent::Prepared) => TestAction::Interrupt,
            _ => TestAction::Continue,
        });
        let result = commit(&f.root, &f.home, writes);
        drop(h);
        match state {
            0 => result.unwrap(),
            1 => code(result, "model_transaction_write"),
            _ => code(result, "model_transaction_test_interrupted"),
        }
        assert_eq!(retained_refs(&f).unwrap(), vec![a, b]);
        assert!(!f.pending().exists());
    }
}
#[test]
fn retained_refs_missing_directories_return_empty_with_zero_creates_or_modification() {
    let f = Fixture::new();
    let manager = f.root.join("manager-state");
    let before = fs::metadata(&manager).unwrap().modified().unwrap();
    assert!(retained_refs(&f).unwrap().is_empty());
    assert_eq!(fs::metadata(&manager).unwrap().modified().unwrap(), before);
    assert!(!manager.join(TRANSACTIONS).exists());
    f.before();
    fs::remove_dir_all(&manager).unwrap();
    let before = fs::metadata(&f.root).unwrap().modified().unwrap();
    assert!(retained_refs(&f).unwrap().is_empty());
    assert!(!manager.exists());
    assert_eq!(fs::metadata(&f.root).unwrap().modified().unwrap(), before);
}
#[test]
fn retained_refs_reject_checksum_corruption_unknown_versions_and_missing_journal() {
    for variant in 0..3 {
        let f = Fixture::new();
        let dir = retained_history(
            &f,
            vec![retained_write(
                POLICY,
                None,
                Some(retained_policy(&[retained_ref(1)], &[], None, None)),
            )],
            Phase::Committed,
        );
        match variant {
            0 => {
                let path = dir.join(JOURNAL);
                let mut v: serde_json::Value =
                    serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                v["value"]["phase"] = "applying".into();
                fs::write(path, serde_json::to_vec(&v).unwrap()).unwrap();
                code(retained_refs(&f), "model_transaction_checksum");
            }
            1 => {
                retained_rewrite(&dir, |j| j.version = 3);
                code(retained_refs(&f), "model_transaction_version");
            }
            _ => {
                fs::remove_file(dir.join(JOURNAL)).unwrap();
                code(retained_refs(&f), "model_transaction_journal_invalid");
            }
        }
        f.before();
        assert!(dir.exists());
    }
}
#[test]
fn retained_refs_reject_unknown_payload_versions_duplicate_json_and_invalid_typed_refs() {
    for variant in 0..7 {
        let f = Fixture::new();
        let mut policy = retained_policy(&[retained_ref(1)], &[], None, None);
        match variant {
            0 => policy["schemaVersion"] = 1.into(),
            1 => policy["templateRefs"][0]["revisionId"] = "../invalid".into(),
            2 => policy["templateRefs"][0]["digest"] = "invalid".into(),
            3 => policy["templateRefs"][0]["extra"] = true.into(),
            4 => {
                policy["channels"][0]["models"][0]["binding"] =
                    serde_json::json!({"kind":"future_binding","reference":retained_ref(2)})
            }
            5 => policy["channels"] = serde_json::Value::Null,
            _ => (),
        }
        let mut bytes = serde_json::to_vec(&policy).unwrap();
        if variant == 6 {
            bytes = String::from_utf8(bytes)
                .unwrap()
                .replace(
                    "\"schemaVersion\":2",
                    "\"schemaVersion\":2,\"schemaVersion\":2",
                )
                .into_bytes();
        }
        retained_history(
            &f,
            vec![WriteV2 {
                target: TargetV2::Manager(POLICY.into()),
                before: None,
                after: Some(bytes),
            }],
            Phase::Committed,
        );
        code(
            retained_refs(&f),
            match variant {
                0 => "model_transaction_version",
                5 | 6 => "model_transaction_payload_invalid",
                _ => "model_transaction_reference_invalid",
            },
        );
        f.before();
    }
    let f = Fixture::new();
    let mut draft = retained_draft(
        retained_policy(&[], &[], None, None),
        serde_json::Value::Null,
    );
    draft["base"]["dtoVersion"] = 3.into();
    retained_history(
        &f,
        vec![retained_write(DRAFT, Some(draft), None)],
        Phase::RolledBack,
    );
    code(retained_refs(&f), "model_transaction_version");
}
#[test]
fn retained_refs_verify_exact_payload_digests_and_sizes_without_modifying_evidence() {
    for variant in 0..3 {
        let f = Fixture::new();
        let dir = retained_history(
            &f,
            vec![retained_write(
                POLICY,
                None,
                Some(retained_policy(&[retained_ref(1)], &[], None, None)),
            )],
            Phase::Committed,
        );
        let path = dir.join("0.after");
        match variant {
            0 => fs::write(&path, b"corrupt").unwrap(),
            1 => fs::remove_file(&path).unwrap(),
            _ => retained_rewrite(&dir, |j| {
                j.plan.entries[0].after.as_mut().unwrap().bytes += 1
            }),
        }
        code(retained_refs(&f), "model_transaction_backup_invalid");
        f.before();
        assert!(dir.exists());
    }
}
#[test]
fn retained_refs_wrong_home_root_copy_or_directory_id_binding_is_not_empty_success() {
    let f = Fixture::new();
    let dir = retained_history(
        &f,
        vec![retained_write(
            POLICY,
            None,
            Some(retained_policy(&[retained_ref(1)], &[], None, None)),
        )],
        Phase::Committed,
    );
    let other = Fixture::new();
    code(
        retained_template_refs(&f.root, &other.home),
        "model_transaction_binding",
    );
    retained_rewrite(&dir, |j| {
        j.plan.binding.root = Context::open(&other.root, &other.home)
            .unwrap()
            .binding()
            .unwrap()
            .root
    });
    code(retained_refs(&f), "model_transaction_binding");
    let f = Fixture::new();
    let dir = retained_history(
        &f,
        vec![retained_write(
            POLICY,
            None,
            Some(retained_policy(&[], &[], None, None)),
        )],
        Phase::Committed,
    );
    retained_rewrite(&dir, |j| j.plan.id = uuid::Uuid::new_v4().to_string());
    code(retained_refs(&f), "model_transaction_journal_invalid");
}
#[test]
fn retained_refs_cap_is_complete_at_256_and_errors_on_the_257th_without_pagination() {
    let f = Fixture::new();
    for i in 0..MAX_RETAINED_TRANSACTIONS {
        retained_history(
            &f,
            vec![retained_write(
                POLICY,
                None,
                Some(retained_policy(&[retained_ref(i as u128)], &[], None, None)),
            )],
            Phase::Committed,
        );
    }
    assert_eq!(retained_refs(&f).unwrap().len(), MAX_RETAINED_TRANSACTIONS);
    retained_history(
        &f,
        vec![retained_write(
            POLICY,
            None,
            Some(retained_policy(&[retained_ref(1000)], &[], None, None)),
        )],
        Phase::Committed,
    );
    code(retained_refs(&f), "model_limit");
    f.before();
}
#[test]
fn retained_refs_cumulative_metadata_budget_includes_journals_and_all_selected_images() {
    let f = Fixture::new();
    for _ in 0..2 {
        let mut bytes =
            serde_json::to_vec(&retained_policy(&[retained_ref(1)], &[], None, None)).unwrap();
        bytes.resize(MAX_RETAINED_METADATA / 2, b' ');
        retained_history(
            &f,
            vec![WriteV2 {
                target: TargetV2::Manager(POLICY.into()),
                before: None,
                after: Some(bytes),
            }],
            Phase::Committed,
        );
    }
    code(retained_refs(&f), "model_limit");
    f.before();
}
#[test]
fn retained_refs_bound_typed_reference_arrays_and_do_not_return_a_partial_prefix() {
    let f = Fixture::new();
    let mut policy = retained_policy(&[retained_ref(1)], &[], None, None);
    policy["templateRefs"] =
        serde_json::to_value(vec![retained_ref(1); MAX_REF_ARRAY + 1]).unwrap();
    retained_history(
        &f,
        vec![retained_write(POLICY, None, Some(policy))],
        Phase::Committed,
    );
    code(retained_refs(&f), "model_limit");
}
#[cfg(unix)]
#[test]
fn retained_refs_reject_unsafe_directory_names_files_symlinks_and_public_directories() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    for variant in 0..6 {
        let f = Fixture::new();
        let dir = retained_history(
            &f,
            vec![retained_write(
                POLICY,
                None,
                Some(retained_policy(&[retained_ref(1)], &[], None, None)),
            )],
            Phase::Committed,
        );
        let base = dir.parent().unwrap();
        match variant {
            0 => fs::create_dir(base.join("not-a-uuid")).unwrap(),
            1 => fs::write(
                base.join(uuid::Uuid::new_v4().to_string()),
                b"not a directory",
            )
            .unwrap(),
            2 => symlink(&dir, base.join(uuid::Uuid::new_v4().to_string())).unwrap(),
            3 => {
                let path = dir.join(JOURNAL);
                let outside = f._temp.path().join("outside-journal");
                fs::rename(&path, &outside).unwrap();
                symlink(outside, path).unwrap();
            }
            4 => {
                let path = dir.join("0.after");
                let outside = f._temp.path().join("outside-image");
                fs::rename(&path, &outside).unwrap();
                symlink(outside, path).unwrap();
            }
            _ => fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).unwrap(),
        }
        assert!(retained_refs(&f).is_err());
        f.before();
        assert!(dir.exists());
    }
    let f = Fixture::new();
    let actual = f._temp.path().join("outside-history");
    fs::create_dir(&actual).unwrap();
    symlink(&actual, f.root.join("manager-state").join(TRANSACTIONS)).unwrap();
    assert!(retained_refs(&f).is_err());
    assert_eq!(fs::read_dir(actual).unwrap().count(), 0);
}

#[test]
fn retained_refs_collect_draft_after_as_well_as_before_without_following_arbitrary_ref_fields() {
    let f = Fixture::new();
    let a = retained_ref(1);
    let b = retained_ref(2);
    let before = retained_draft(
        retained_policy(std::slice::from_ref(&a), &[], None, None),
        serde_json::Value::Null,
    );
    let mut after = retained_draft(
        retained_policy(&[], &[], Some(&b), None),
        serde_json::Value::Null,
    );
    after["base"]["libraryDigest"] = retained_ref(3).digest.into();
    after["credentialEdits"] = serde_json::json!({"public-ref":{"reference":retained_ref(4)}});
    retained_history(
        &f,
        vec![retained_write(DRAFT, Some(before), Some(after))],
        Phase::Applying,
    );
    assert_eq!(retained_refs(&f).unwrap(), vec![a, b]);
    f.before();
}
#[test]
fn retained_refs_bound_journal_reads_and_reject_oversized_unexpected_images() {
    let f = Fixture::new();
    let dir = retained_history(
        &f,
        vec![retained_write(
            POLICY,
            None,
            Some(retained_policy(&[retained_ref(1)], &[], None, None)),
        )],
        Phase::Committed,
    );
    let unexpected = Dir::anchor(&dir).unwrap();
    unexpected
        .create("0.before", b"unexpected image bytes")
        .unwrap();
    code(retained_refs(&f), "model_transaction_backup_invalid");
    let f = Fixture::new();
    let dir = retained_history(
        &f,
        vec![retained_write(
            POLICY,
            None,
            Some(retained_policy(&[], &[], None, None)),
        )],
        Phase::Committed,
    );
    fs::write(dir.join(JOURNAL), vec![b' '; MAX_JOURNAL + 1]).unwrap();
    code(retained_refs(&f), "model_limit");
}

fn operation_receipt_path(f: &Fixture, id: &str) -> PathBuf {
    f.root
        .join("manager-state")
        .join(RECEIVE)
        .join(id)
        .join("receipt.json")
}
fn operation_receipt_seed(f: &Fixture, id: &str, bytes: &[u8]) {
    let c = Context::open(&f.root, &f.home).unwrap();
    c.receipt_dir(id, true)
        .unwrap()
        .unwrap()
        .create("receipt.json", bytes)
        .unwrap();
}
fn receive_seven_writes(f: &Fixture, id: &str, receipt_before: Option<Vec<u8>>) -> Vec<WriteV2> {
    let receipt=serde_json::to_vec(&serde_json::json!({"operationId":id,"payloadDigest":schema_v2::digest(b"fixed-payload"),"planDigest":schema_v2::digest(b"fixed-plan"),"configDigest":schema_v2::digest(NEW_CONFIG)})).unwrap();
    let mut writes = vec![WriteV2 {
        target: TargetV2::OfficialConfig,
        before: Some(OLD_CONFIG.to_vec()),
        after: Some(NEW_CONFIG.to_vec()),
    }];
    for name in [
        "manager-state/model-family-templates.json",
        POLICY,
        "manager-state/model-materialization-ledger.json",
        "manager-state/model-protocol-mappings.json",
        VERSIONS,
    ] {
        writes.push(WriteV2 {
            target: TargetV2::Manager(name.into()),
            before: fs::read(f.root.join(name)).ok(),
            after: Some(b"receive-exact-image".to_vec()),
        });
    }
    writes.push(WriteV2 {
        target: TargetV2::ManagerOperationReceipt(id.into()),
        before: receipt_before,
        after: Some(receipt),
    });
    assert_eq!(writes.len(), 7);
    writes
}
#[test]
fn operation_receipt_seven_file_interrupt_recovery_pairs_exact_receipt_with_config() {
    for (cut, existing) in [(0, false), (5, false), (6, true)] {
        let f = Fixture::new();
        let id = uuid::Uuid::new_v4().to_string();
        let before = existing.then(|| b"original receipt image\r\n".to_vec());
        if let Some(bytes) = &before {
            operation_receipt_seed(&f, &id, bytes);
        }
        let writes = receive_seven_writes(&f, &id, before.clone());
        let expected = writes[6].after.as_ref().unwrap().clone();
        let h = hook(move |e| {
            if e == TestEvent::AfterApply(cut) {
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
        let dir = f.dir();
        if cut < 6 {
            assert!(fs::read(operation_receipt_path(&f, &id)).ok() == before);
        }
        assert!(f.recover().unwrap());
        assert!(fs::read(f.home.join("config.json")).unwrap() == NEW_CONFIG);
        assert!(fs::read(operation_receipt_path(&f, &id)).unwrap() == expected);
        assert!(fs::read(dir.join("6.before")).ok() == before);
        assert!(fs::read(dir.join("6.after")).unwrap() == expected);
        assert!(fs::read(dir.join("0.before")).unwrap() == OLD_CONFIG);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let path = operation_receipt_path(&f, &id);
            for directory in [
                path.parent().unwrap(),
                path.parent().unwrap().parent().unwrap(),
            ] {
                assert_eq!(
                    fs::metadata(directory).unwrap().permissions().mode() & 0o777,
                    0o700
                );
            }
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        assert!(!f.pending().exists());
        assert!(!f.recover().unwrap());
    }
    // A fresh receipt must also disappear when a later in-process failure rolls
    // back publication; its creation never makes it a separately committed fact.
    let f = Fixture::new();
    let id = uuid::Uuid::new_v4().to_string();
    let h = hook(|e| {
        if e == TestEvent::AfterApply(6) {
            TestAction::Fail
        } else {
            TestAction::Continue
        }
    });
    code(
        commit(&f.root, &f.home, receive_seven_writes(&f, &id, None)),
        "model_transaction_write",
    );
    drop(h);
    f.before();
    assert!(!operation_receipt_path(&f, &id).exists());
    assert!(!f.pending().exists());
}
#[test]
fn operation_receipt_rejects_uuid_paths_deletion_limits_unknown_variants_and_symlinks() {
    let f = Fixture::new();
    let id = uuid::Uuid::new_v4().to_string();
    for invalid in [
        "../escape".to_string(),
        format!("{id}/receipt.json"),
        format!("manager-state/{RECEIVE}/{id}/receipt.json"),
        format!("{id}\\receipt.json"),
        id.to_uppercase(),
        "550e8400-e29b-11d4-a716-446655440000".into(),
    ] {
        code(
            commit(
                &f.root,
                &f.home,
                vec![WriteV2 {
                    target: TargetV2::ManagerOperationReceipt(invalid),
                    before: None,
                    after: Some(vec![]),
                }],
            ),
            "model_transaction_target_invalid",
        );
    }
    code(
        commit(
            &f.root,
            &f.home,
            vec![WriteV2 {
                target: TargetV2::Manager(format!("manager-state/{RECEIVE}/{id}/receipt.json")),
                before: None,
                after: Some(vec![]),
            }],
        ),
        "model_transaction_target_invalid",
    );
    code(
        commit(
            &f.root,
            &f.home,
            vec![WriteV2 {
                target: TargetV2::ManagerOperationReceipt(id.clone()),
                before: None,
                after: None,
            }],
        ),
        "model_transaction_target_invalid",
    );
    for before in [false, true] {
        let bytes = vec![0; MAX_RECEIPT + 1];
        let w = if before {
            WriteV2 {
                target: TargetV2::ManagerOperationReceipt(id.clone()),
                before: Some(bytes),
                after: Some(vec![]),
            }
        } else {
            WriteV2 {
                target: TargetV2::ManagerOperationReceipt(id.clone()),
                before: None,
                after: Some(bytes),
            }
        };
        code(commit(&f.root, &f.home, vec![w]), "model_transaction_limit");
    }
    assert!(!f.root.join("manager-state").join(RECEIVE).exists());
    f.before();
    let target = TargetV2::ManagerOperationReceipt(id.clone());
    let wire = serde_json::to_value(&target).unwrap();
    assert_eq!(
        wire,
        serde_json::json!({"kind":"manager_operation_receipt","relative":id})
    );
    assert!(serde_json::from_value::<TargetV2>(
        serde_json::json!({"kind":"future_receipt","relative":id})
    )
    .is_err());
    assert!(serde_json::from_value::<TargetV2>(
        serde_json::json!({"kind":"manager_operation_receipt","relative":id,"path":"elsewhere"})
    )
    .is_err());
    // The exact 32 KiB boundary remains accepted; the adapter interprets no body.
    commit(
        &f.root,
        &f.home,
        vec![WriteV2 {
            target,
            before: None,
            after: Some(vec![b' '; MAX_RECEIPT]),
        }],
    )
    .unwrap();
    assert_eq!(
        fs::metadata(operation_receipt_path(&f, &id)).unwrap().len(),
        MAX_RECEIPT as u64
    );
    for (bad_id, deletion) in [(false, false), (true, false), (false, true)] {
        let f = Fixture::new();
        f.interrupted();
        f.rewrite_journal(|j| {
            let index = if deletion { 2 } else { 1 };
            j.plan.entries[index].target = TargetV2::ManagerOperationReceipt(if bad_id {
                "../escape".into()
            } else {
                uuid::Uuid::new_v4().to_string()
            });
            if !deletion && !bad_id {
                j.plan.entries[index].after.as_mut().unwrap().bytes = MAX_RECEIPT + 1;
            }
        });
        code(
            f.recover(),
            if deletion || bad_id {
                "model_transaction_target_invalid"
            } else {
                "model_transaction_limit"
            },
        );
        assert!(fs::read(f.root.join(POLICY)).unwrap() == OLD_POLICY);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        for component in 0..3 {
            let f = Fixture::new();
            let id = uuid::Uuid::new_v4().to_string();
            let outside = f._temp.path().join("outside");
            fs::create_dir(&outside).unwrap();
            if component == 0 {
                symlink(&outside, f.root.join("manager-state").join(RECEIVE)).unwrap();
            } else {
                let c = Context::open(&f.root, &f.home).unwrap();
                let receive = c
                    .manager()
                    .unwrap()
                    .child(RECEIVE, true, true)
                    .unwrap()
                    .unwrap();
                if component == 1 {
                    symlink(&outside, receive.path.join(&id)).unwrap();
                } else {
                    let operation = receive.child(&id, true, true).unwrap().unwrap();
                    fs::write(outside.join("file"), b"outside untouched").unwrap();
                    symlink(outside.join("file"), operation.path.join("receipt.json")).unwrap();
                }
            }
            assert!(commit(&f.root, &f.home, receive_seven_writes(&f, &id, None)).is_err());
            f.before();
            assert!(!f.pending().exists());
            if component == 2 {
                assert!(fs::read(outside.join("file")).unwrap() == b"outside untouched");
            }
        }
    }
}
#[test]
fn operation_receipt_before_cas_and_unknown_after_preserve_external_edits() {
    let f = Fixture::new();
    let id = uuid::Uuid::new_v4().to_string();
    operation_receipt_seed(&f, &id, b"original");
    code(
        commit(&f.root, &f.home, receive_seven_writes(&f, &id, None)),
        "model_transaction_cas",
    );
    f.before();
    assert!(fs::read(operation_receipt_path(&f, &id)).unwrap() == b"original");
    assert!(!f.root.join("manager-state").join(TRANSACTIONS).exists());
    let path = operation_receipt_path(&f, &id);
    let h = hook(move |e| {
        if e == TestEvent::BeforeApply(6) {
            fs::write(&path, b"external receipt").unwrap();
        }
        TestAction::Continue
    });
    code(
        commit(
            &f.root,
            &f.home,
            receive_seven_writes(&f, &id, Some(b"original".to_vec())),
        ),
        "model_transaction_compensation_conflict",
    );
    drop(h);
    f.before();
    assert!(fs::read(operation_receipt_path(&f, &id)).unwrap() == b"external receipt");
    code(f.recover(), "model_transaction_external_change");
    fs::write(operation_receipt_path(&f, &id), b"original").unwrap();
    assert!(f.recover().unwrap());
    f.before();
    // A crash after both images reached disk must not let replay overwrite a
    // subsequent receipt edit or silently reapply config over an external edit.
    let f = Fixture::new();
    let id = uuid::Uuid::new_v4().to_string();
    let writes = receive_seven_writes(&f, &id, None);
    let expected = writes[6].after.clone().unwrap();
    let h = hook(|e| {
        if e == TestEvent::AfterApply(6) {
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
    fs::write(operation_receipt_path(&f, &id), b"external receipt").unwrap();
    code(f.recover(), "model_transaction_external_change");
    assert!(fs::read(f.home.join("config.json")).unwrap() == NEW_CONFIG);
    assert!(fs::read(operation_receipt_path(&f, &id)).unwrap() == b"external receipt");
    fs::write(operation_receipt_path(&f, &id), &expected).unwrap();
    fs::write(f.home.join("config.json"), b"external config").unwrap();
    code(f.recover(), "model_transaction_external_change");
    assert!(fs::read(f.home.join("config.json")).unwrap() == b"external config");
    assert!(fs::read(operation_receipt_path(&f, &id)).unwrap() == expected);
}
#[test]
fn operation_receipt_free_fields_never_become_retained_template_dependencies() {
    let f = Fixture::new();
    let id = uuid::Uuid::new_v4().to_string();
    let adopted = retained_ref(1);
    let unrelated = retained_ref(2);
    let bytes=serde_json::to_vec(&serde_json::json!({"operationId":id,"templateRefs":[unrelated],"policy":retained_policy(&[retained_ref(3)],&[],None,None),"reference":retained_ref(4)})).unwrap();
    retained_history(
        &f,
        vec![WriteV2 {
            target: TargetV2::ManagerOperationReceipt(id),
            before: None,
            after: Some(bytes),
        }],
        Phase::Committed,
    );
    assert!(retained_refs(&f).unwrap().is_empty());
    retained_history(
        &f,
        vec![retained_write(
            POLICY,
            None,
            Some(retained_policy(
                std::slice::from_ref(&adopted),
                &[],
                None,
                None,
            )),
        )],
        Phase::Committed,
    );
    assert_eq!(retained_refs(&f).unwrap(), vec![adopted]);
    f.before();
}

const IDENTITY_MAPPING: &str = "manager-state/sync-identity-mappings-v2.json";
const OLD_IDENTITY_MAPPING: &[u8] = b"fixed original identity mapping\r\n";
const NEW_IDENTITY_MAPPING: &[u8] = b"fixed received identity mapping\n";
fn receive_eight_writes(f: &Fixture, id: &str, mapping_before: Option<Vec<u8>>) -> Vec<WriteV2> {
    let mut writes = receive_seven_writes(f, id, None);
    writes.push(WriteV2 {
        target: TargetV2::Manager(IDENTITY_MAPPING.into()),
        before: mapping_before,
        after: Some(NEW_IDENTITY_MAPPING.to_vec()),
    });
    assert_eq!(writes.len(), MAX_FILES);
    writes
}
#[test]
fn identity_mapping_eight_file_crash_recovers_exact_mapping_and_operation_receipt() {
    for (cut, existing) in [(0, false), (6, false), (7, true)] {
        let f = Fixture::new();
        let id = uuid::Uuid::new_v4().to_string();
        let before = existing.then(|| OLD_IDENTITY_MAPPING.to_vec());
        if let Some(bytes) = &before {
            fs::write(f.root.join(IDENTITY_MAPPING), bytes).unwrap();
        }
        let writes = receive_eight_writes(&f, &id, before.clone());
        let expected_receipt = writes[6].after.clone().unwrap();
        let h = hook(move |e| {
            if e == TestEvent::AfterApply(cut) {
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
        let dir = f.dir();
        if cut < 7 {
            assert!(fs::read(f.root.join(IDENTITY_MAPPING)).ok() == before);
        }
        if cut < 6 {
            assert!(!operation_receipt_path(&f, &id).exists());
        }
        assert!(f.recover().unwrap());
        assert!(fs::read(f.home.join("config.json")).unwrap() == NEW_CONFIG);
        assert!(fs::read(operation_receipt_path(&f, &id)).unwrap() == expected_receipt);
        assert!(fs::read(f.root.join(IDENTITY_MAPPING)).unwrap() == NEW_IDENTITY_MAPPING);
        for name in [
            "manager-state/model-family-templates.json",
            POLICY,
            "manager-state/model-materialization-ledger.json",
            "manager-state/model-protocol-mappings.json",
            VERSIONS,
        ] {
            assert!(fs::read(f.root.join(name)).unwrap() == b"receive-exact-image");
        }
        assert!(fs::read(dir.join("7.before")).ok() == before);
        assert!(fs::read(dir.join("7.after")).unwrap() == NEW_IDENTITY_MAPPING);
        assert!(fs::read(dir.join("6.after")).unwrap() == expected_receipt);
        assert!(!f.pending().exists());
        assert!(!f.recover().unwrap());
    }
}
#[test]
fn identity_mapping_before_cas_preserves_external_edit_and_compensates_receipt() {
    let f = Fixture::new();
    let id = uuid::Uuid::new_v4().to_string();
    fs::write(f.root.join(IDENTITY_MAPPING), OLD_IDENTITY_MAPPING).unwrap();
    code(
        commit(&f.root, &f.home, receive_eight_writes(&f, &id, None)),
        "model_transaction_cas",
    );
    f.before();
    assert!(!operation_receipt_path(&f, &id).exists());
    assert!(!f.root.join("manager-state").join(TRANSACTIONS).exists());
    let path = f.root.join(IDENTITY_MAPPING);
    let h = hook(move |e| {
        if e == TestEvent::BeforeApply(7) {
            fs::write(&path, b"external identity mapping").unwrap();
        }
        TestAction::Continue
    });
    code(
        commit(
            &f.root,
            &f.home,
            receive_eight_writes(&f, &id, Some(OLD_IDENTITY_MAPPING.to_vec())),
        ),
        "model_transaction_compensation_conflict",
    );
    drop(h);
    f.before();
    assert!(!operation_receipt_path(&f, &id).exists());
    assert!(fs::read(f.root.join(IDENTITY_MAPPING)).unwrap() == b"external identity mapping");
    code(f.recover(), "model_transaction_external_change");
    assert!(fs::read(f.root.join(IDENTITY_MAPPING)).unwrap() == b"external identity mapping");
    fs::write(f.root.join(IDENTITY_MAPPING), OLD_IDENTITY_MAPPING).unwrap();
    assert!(f.recover().unwrap());
    f.before();
    assert!(!operation_receipt_path(&f, &id).exists());
    assert!(!f.pending().exists());
}
#[test]
fn identity_mapping_limits_deletion_and_alternate_paths_remain_rejected() {
    let f = Fixture::new();
    code(
        commit(
            &f.root,
            &f.home,
            vec![WriteV2 {
                target: TargetV2::Manager(IDENTITY_MAPPING.into()),
                before: None,
                after: None,
            }],
        ),
        "model_transaction_target_invalid",
    );
    for path in [
        "sync-identity-mappings-v2.json",
        "manager-state/../sync-identity-mappings-v2.json",
        "manager-state/sync-identity-mappings-v2.json/receipt.json",
        "manager-state/other-identity-mappings-v2.json",
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
    for oversized_before in [false, true] {
        let bytes = vec![0; MAX_IMAGE + 1];
        let write = if oversized_before {
            WriteV2 {
                target: TargetV2::Manager(IDENTITY_MAPPING.into()),
                before: Some(bytes),
                after: Some(vec![]),
            }
        } else {
            WriteV2 {
                target: TargetV2::Manager(IDENTITY_MAPPING.into()),
                before: None,
                after: Some(bytes),
            }
        };
        code(
            commit(&f.root, &f.home, vec![write]),
            "model_transaction_limit",
        );
    }
    let id = uuid::Uuid::new_v4().to_string();
    let mut nine = receive_eight_writes(&f, &id, None);
    nine.push(WriteV2 {
        target: TargetV2::Manager("manager-state/model-channel-layout.json".into()),
        before: None,
        after: Some(vec![]),
    });
    code(commit(&f.root, &f.home, nine), "model_transaction_limit");
    f.before();
    assert!(!f.root.join("manager-state").join(TRANSACTIONS).exists());
    assert!(!f.root.join(IDENTITY_MAPPING).exists());
    let h = hook(|e| {
        if e == TestEvent::AfterApply(6) {
            TestAction::Interrupt
        } else {
            TestAction::Continue
        }
    });
    code(
        commit(&f.root, &f.home, receive_eight_writes(&f, &id, None)),
        "model_transaction_test_interrupted",
    );
    drop(h);
    f.rewrite_journal(|j| j.plan.entries[7].after = None);
    code(f.recover(), "model_transaction_target_invalid");
    assert!(!f.root.join(IDENTITY_MAPPING).exists());
    assert!(operation_receipt_path(&f, &id).exists());
    assert!(f.pending().exists());
}

#[test]
fn protected_restore_exact_absence_is_distinct_from_ordinary_delete_and_receipts_remain_forbidden()
{
    let f = Fixture::new();
    let writes = || {
        vec![
            WriteV2 {
                target: TargetV2::OfficialConfig,
                before: Some(OLD_CONFIG.to_vec()),
                after: None,
            },
            WriteV2 {
                target: TargetV2::Manager(POLICY.into()),
                before: Some(OLD_POLICY.to_vec()),
                after: None,
            },
        ]
    };
    code(
        commit(&f.root, &f.home, writes()),
        "model_transaction_target_invalid",
    );
    assert_eq!(fs::read(f.home.join("config.json")).unwrap(), OLD_CONFIG);
    commit_protected_restore(&f.root, &f.home, writes()).unwrap();
    assert!(!f.home.join("config.json").exists());
    assert!(!f.root.join(POLICY).exists());
    let receipt = WriteV2 {
        target: TargetV2::ManagerOperationReceipt("11111111-1111-4111-8111-111111111111".into()),
        before: None,
        after: Some(b"not a restore target".to_vec()),
    };
    code(
        commit_protected_restore(&f.root, &f.home, vec![receipt]),
        "model_transaction_target_invalid",
    );
}
#[test]
fn interrupted_protected_restore_keeps_mode_and_replays_only_the_exact_original_absent_plan() {
    let f = Fixture::new();
    let _guard = hook(|e| {
        if e == TestEvent::AfterApply(0) {
            TestAction::Interrupt
        } else {
            TestAction::Continue
        }
    });
    let writes = vec![
        WriteV2 {
            target: TargetV2::OfficialConfig,
            before: Some(OLD_CONFIG.to_vec()),
            after: None,
        },
        WriteV2 {
            target: TargetV2::Manager(POLICY.into()),
            before: Some(OLD_POLICY.to_vec()),
            after: None,
        },
    ];
    code(
        commit_protected_restore(&f.root, &f.home, writes),
        "model_transaction_test_interrupted",
    );
    drop(_guard);
    let marker: Marker = decode(&fs::read(f.pending()).unwrap()).unwrap();
    let journal: Journal = decode(&fs::read(f.dir().join(JOURNAL)).unwrap()).unwrap();
    assert!(journal.plan.protected_restore);
    assert_eq!(marker.plan_digest, checksum(&journal.plan).unwrap());
    assert!(recover(&f.root, &f.home).unwrap());
    assert!(!f.home.join("config.json").exists());
    assert!(!f.root.join(POLICY).exists());
    assert!(!f.pending().exists());
}
