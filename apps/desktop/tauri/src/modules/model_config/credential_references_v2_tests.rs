use super::*;
use serde_json::json;
use std::cell::{Cell, RefCell};
#[derive(Default)]
struct Mock {
    values: RefCell<BTreeMap<String, Zeroizing<String>>>,
    reads: RefCell<Vec<String>>,
    creates: Cell<usize>,
    removes: RefCell<Vec<String>>,
    fail_read: RefCell<Option<String>>,
    fail_create: Cell<usize>,
    create_then_fail: Cell<bool>,
    fail_remove: Cell<bool>,
}
impl Mock {
    fn put(&self, a: &str, v: &str) {
        self.values
            .borrow_mut()
            .insert(a.into(), Zeroizing::new(v.into()));
    }
    fn has(&self, a: &str) -> bool {
        self.values.borrow().contains_key(a)
    }
    fn same(&self, a: &str, b: &str) -> bool {
        self.values.borrow().get(a) == self.values.borrow().get(b)
    }
}
impl CredentialVaultV2 for Mock {
    fn read(&self, a: &str) -> Result<Option<Zeroizing<String>>> {
        self.reads.borrow_mut().push(a.into());
        if self.fail_read.borrow().as_deref() == Some(a) {
            return Err(fault("mock_read"));
        }
        Ok(self.values.borrow().get(a).cloned())
    }
    fn create(&self, a: &str, v: &str) -> Result<()> {
        let n = self.creates.get() + 1;
        self.creates.set(n);
        if self.has(a) {
            return Err(fault("mock_collision"));
        }
        if self.fail_create.get() == n {
            if self.create_then_fail.get() {
                self.put(a, v);
            }
            return Err(fault("mock_create"));
        }
        self.put(a, v);
        Ok(())
    }
    fn remove_if_digest(&self, a: &str, d: &str) -> Result<()> {
        if self.fail_remove.get() {
            return Err(fault("mock_remove"));
        }
        if let Some(v) = self.values.borrow().get(a) {
            if sha256(v.as_bytes()) != d {
                return Err(fault("mock_changed"));
            }
        }
        self.values.borrow_mut().remove(a);
        self.removes.borrow_mut().push(a.into());
        Ok(())
    }
}
struct Fixture {
    _tmp: tempfile::TempDir,
    root: PathBuf,
    home: PathBuf,
    original: Value,
    next: Value,
    vault: Mock,
}
impl Fixture {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let base = tmp.path().canonicalize().unwrap();
        let root = base.join("data");
        let home = base.join("home");
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir(&home).unwrap();
        let row = json!({"name":"Old","baseUrl":"https://fixture.test/v1","adapter":"openai-chat","authMode":"key",
            "apiKey":"keychain:Old/b","apiKeyPool":[{"id":"11111111","key":"keychain:Old/a","label":"first","addedAt":2},
                {"id":"22222222","key":"keychain:Old/b"},{"id":"33333333","key":"$OCXD_UNDEFINED_FIXTURE"}],
            "headers":{"Authorization":"keychain:Old/header","X-Env":concat!("$","{OCXD_UNDEFINED_HEADER}")},
            "proxy":{"auth":{"password":"keychain:Old/proxy"}},"mcpServers":{"tool":{"env":{"TOKEN":"keychain:Old/mcp"}}},
            "desktopExecutor":{"env":{"TOKEN":"keychain:Old/executor"}}});
        let original = json!({"providers":{"Old":row},"other":"keychain:unselected/no-read"});
        let mut next = original.clone();
        let mut row = next["providers"]
            .as_object_mut()
            .unwrap()
            .remove("Old")
            .unwrap();
        row["name"] = json!("New");
        next["providers"]["New"] = row;
        std::fs::write(
            home.join("config.json"),
            serde_json::to_vec(&original).unwrap(),
        )
        .unwrap();
        let vault = Mock::default();
        for s in ["a", "b", "header", "proxy", "mcp", "executor"] {
            vault.put(&format!("Old/{s}"), &format!("fixture-{s}-SECRET"));
        }
        Self {
            _tmp: tmp,
            root,
            home,
            original,
            next,
            vault,
        }
    }
    fn prepare(&mut self) -> Result<ReferencePlanV2> {
        prepare(
            &self.root,
            &self.home,
            &self.original,
            &mut self.next,
            &[mapping("Old", "New")],
            &self.vault,
        )
    }
    fn pending(&self) -> bool {
        self.root.join("manager-state").join(PENDING).exists()
    }
    fn journal(&self) -> Journal {
        let c = Context::open(&self.root, &self.home).unwrap();
        let m = c.marker().unwrap().unwrap();
        decode(
            &c.operation(&m.id, false)
                .unwrap()
                .read(JOURNAL, MAX_IMAGE, true)
                .unwrap()
                .unwrap(),
        )
        .unwrap()
    }
    fn no_config_write(&self) {
        let a: Value =
            serde_json::from_slice(&std::fs::read(self.home.join("config.json")).unwrap()).unwrap();
        assert!(a == self.original);
    }
}
fn mapping(a: &str, b: &str) -> ReferenceSourceV2 {
    ReferenceSourceV2 {
        source_provider_id: a.into(),
        target_provider_id: b.into(),
    }
}
thread_local! {static FAIL_POINT: RefCell<Option<(&'static str,usize,bool)>>=const {RefCell::new(None)};}
fn hook(point: &str) -> Result<()> {
    FAIL_POINT.with(|p| {
        let mut p = p.borrow_mut();
        if let Some((at, left, crash)) = p.as_mut() {
            if point == *at {
                *left -= 1;
                if *left == 0 {
                    let crash = *crash;
                    *p = None;
                    return Err(fault(if crash {
                        "credential_references_test_interrupted"
                    } else {
                        "mock_checkpoint"
                    }));
                }
            }
        }
        Ok(())
    })
}
struct Inject;
impl Inject {
    fn at(at: &'static str, n: usize, crash: bool) -> Self {
        FAIL_POINT.with(|p| *p.borrow_mut() = Some((at, n, crash)));
        HOOK.with(|h| *h.borrow_mut() = Some(hook));
        Self
    }
}
impl Drop for Inject {
    fn drop(&mut self) {
        HOOK.with(|h| *h.borrow_mut() = None);
        FAIL_POINT.with(|p| *p.borrow_mut() = None);
    }
}
#[test]
fn rename_preserves_active_pool_slots_labels_nested_credentials_and_env() {
    let mut f = Fixture::new();
    let p = f.prepare().unwrap();
    assert_eq!(f.next["providers"]["New"]["apiKey"], "keychain:New/b");
    assert_eq!(
        f.next["providers"]["New"]["apiKeyPool"][0]["id"],
        "11111111"
    );
    assert_eq!(
        f.next["providers"]["New"]["apiKeyPool"][0]["label"],
        "first"
    );
    assert_eq!(f.next["providers"]["New"]["apiKeyPool"][0]["addedAt"], 2);
    assert_eq!(
        f.next["providers"]["New"]["apiKeyPool"][2]["key"],
        "$OCXD_UNDEFINED_FIXTURE"
    );
    assert_eq!(
        f.next["providers"]["New"]["headers"]["X-Env"],
        concat!("$", "{OCXD_UNDEFINED_HEADER}")
    );
    for s in ["a", "b", "header", "proxy", "mcp", "executor"] {
        assert!(f.vault.same(&format!("Old/{s}"), &format!("New/{s}")));
    }
    assert_eq!(f.vault.creates.get(), 6);
    f.no_config_write();
    finish(&f.root, &f.home, &p, &f.next, &f.vault).unwrap();
    finish(&f.root, &f.home, &p, &f.next, &f.vault).unwrap();
    assert!(!f.pending());
    assert!(f.vault.removes.borrow().is_empty());
    assert!(!f
        .vault
        .reads
        .borrow()
        .iter()
        .any(|a| a.contains("unselected")));
}
#[test]
fn dual_protocol_and_copy_preserve_source() {
    let mut f = Fixture::new();
    let row = f.original["providers"]["Old"].clone();
    f.next["providers"]["Old"] = row.clone();
    f.next["providers"]["Responses"] = row;
    f.next["providers"]["Responses"]["adapter"] = json!("openai-responses");
    let p = prepare(
        &f.root,
        &f.home,
        &f.original,
        &mut f.next,
        &[mapping("Old", "New"), mapping("Old", "Responses")],
        &f.vault,
    )
    .unwrap();
    for s in ["a", "b", "header", "proxy", "mcp", "executor"] {
        assert!(f.vault.same(&format!("Old/{s}"), &format!("New/{s}")));
        assert!(f.vault.same(&format!("Old/{s}"), &format!("Responses/{s}")));
    }
    assert!(f.next["providers"]["Old"] == f.original["providers"]["Old"]);
    finish(&f.root, &f.home, &p, &f.next, &f.vault).unwrap();
    assert_eq!(f.vault.creates.get(), 12);
}
#[test]
fn bare_account_plaintext_and_nonprivate_values_survive() {
    let mut f = Fixture::new();
    f.vault.put("Old", "bare-SECRET");
    f.original["providers"]["Old"] = json!({"baseUrl":"https://fixture.test","apiKey":"keychain:Old","headers":{"Plain":"literal-SECRET"}});
    f.next = json!({"providers":{"New":f.original["providers"]["Old"]},"nonprivate":"keychain:Old/untouched"});
    let p = f.prepare().unwrap();
    assert_eq!(f.next["providers"]["New"]["apiKey"], "keychain:New");
    assert_eq!(f.next["nonprivate"], "keychain:Old/untouched");
    assert_eq!(
        f.next["providers"]["New"]["headers"]["Plain"],
        "literal-SECRET"
    );
    finish(&f.root, &f.home, &p, &f.next, &f.vault).unwrap();
    assert!(f.vault.same("Old", "New"));
}
#[test]
fn foreign_refs_membership_and_direction_fail_before_all_vault_reads() {
    for mode in 0..9 {
        let mut f = Fixture::new();
        match mode {
            0 => f.next["providers"]["New"]["apiKey"] = json!("keychain:Other/x"),
            1 => {
                f.original["providers"]["Old"]["headers"]["X-Foreign"] = json!("keychain:Oldish/x")
            }
            2 => f.next["providers"]["New"]["apiKey"] = json!("keychain:Old/not-in-source"),
            3 => f.next["providers"]["New"]["baseUrl"] = json!("https://changed.test"),
            4 => f.next["providers"]["New"]["authMode"] = json!("oauth"),
            5 => f.original["providers"]["Old"]["authMode"] = json!("forward"),
            6 => f.next["providers"]["New"]["apiKeyTransport"] = json!("query"),
            7 => {
                f.original["providers"]
                    .as_object_mut()
                    .unwrap()
                    .remove("Old");
            }
            _ => f.next["providers"]["New"]["apiKey"] = json!("keychain:Old/\ninvalid"),
        }
        let before = f.next.clone();
        assert!(f.prepare().is_err(), "case {mode}");
        assert!(f.next == before);
        assert!(f.vault.reads.borrow().is_empty());
        assert_eq!(f.vault.creates.get(), 0);
        assert!(!f.pending());
    }
    let mut f = Fixture::new();
    assert!(prepare(
        &f.root,
        &f.home,
        &f.original,
        &mut f.next,
        &[mapping("Old", "New"), mapping("openai", "New")],
        &f.vault
    )
    .is_err());
    assert!(f.vault.reads.borrow().is_empty());
}
#[test]
fn collision_writes_nothing_and_preexisting_equal_target_is_never_removed() {
    let mut f = Fixture::new();
    f.vault.put("New/a", "different-SECRET");
    assert!(f.prepare().is_err());
    assert_eq!(f.vault.creates.get(), 0);
    assert!(!f.pending());
    assert!(f
        .vault
        .reads
        .borrow()
        .iter()
        .all(|a| a.starts_with("Old/") || a.starts_with("New/")));
    f.vault.put("New/a", "fixture-a-SECRET");
    let p = f.prepare().unwrap();
    abort(&f.root, &f.home, &p, &f.original, &f.vault).unwrap();
    abort(&f.root, &f.home, &p, &f.original, &f.vault).unwrap();
    assert!(f.vault.has("New/a"));
    assert!(f.vault.has("Old/a"));
    assert!(!f.vault.has("New/b"));
    assert!(!f
        .vault
        .removes
        .borrow()
        .iter()
        .any(|a| a == "New/a" || a.starts_with("Old/")));
}
#[test]
fn partial_failure_compensates_receipts_and_next_is_unchanged() {
    let mut f = Fixture::new();
    f.vault.fail_create.set(3);
    let before = f.next.clone();
    assert!(f.prepare().is_err());
    assert!(f.next == before);
    assert!(!f.pending());
    assert!(!f.vault.has("New/a"));
    assert!(!f.vault.has("New/b"));
    assert_eq!(f.vault.removes.borrow().len(), 2);
    assert!(f.vault.has("Old/a"));
    f.no_config_write();
}
#[test]
fn create_error_after_write_retains_unknown_ownership_and_same_operation() {
    let mut f = Fixture::new();
    f.vault.fail_create.set(2);
    f.vault.create_then_fail.set(true);
    assert!(f.prepare().is_err());
    assert!(f.pending());
    let id = f.journal().spec.id;
    assert!(f.vault.has("New/b"));
    assert!(!f.vault.has("New/a"));
    for _ in 0..2 {
        assert!(recover(&f.root, &f.home, &f.original, &f.vault).is_err());
        assert_eq!(f.journal().spec.id, id);
    }
    assert!(f.vault.read("Old/b").unwrap().is_some());
    let creates = f.vault.creates.get();
    assert!(f.prepare().is_err());
    assert_eq!(f.journal().spec.id, id);
    assert_eq!(f.vault.creates.get(), creates);
}
#[test]
fn crash_before_create_and_after_receipt_recovers_by_compensation() {
    for (point, n) in [
        ("prepared", 1),
        ("before_create", 1),
        ("receipt", 2),
        ("ready", 1),
    ] {
        let mut f = Fixture::new();
        let before = f.next.clone();
        {
            let _i = Inject::at(point, n, true);
            assert!(f.prepare().is_err());
        }
        assert!(f.pending());
        assert!(f.next == before);
        assert!(recover(&f.root, &f.home, &f.original, &f.vault).unwrap());
        assert!(!recover(&f.root, &f.home, &f.original, &f.vault).unwrap());
        assert!(!f.pending());
        assert!(!f.vault.has("New/a"));
        assert!(f.vault.has("Old/a"));
        f.no_config_write();
    }
}
#[test]
fn crash_after_create_before_receipt_never_claims_cleanup_success() {
    let mut f = Fixture::new();
    {
        let _i = Inject::at("after_create", 1, true);
        assert!(f.prepare().is_err());
    }
    let id = f.journal().spec.id;
    for _ in 0..2 {
        assert!(recover(&f.root, &f.home, &f.original, &f.vault).is_err());
    }
    assert!(f.vault.has("New/a"));
    assert_eq!(f.journal().spec.id, id);
    assert!(f.vault.removes.borrow().is_empty());
}
#[test]
fn expected_actual_and_finished_marker_crash_are_idempotent() {
    for crash in [false, true] {
        let mut f = Fixture::new();
        let p = f.prepare().unwrap();
        if crash {
            let _i = Inject::at("finished", 1, true);
            assert!(finish(&f.root, &f.home, &p, &f.next, &f.vault).is_err());
        }
        assert!(recover(&f.root, &f.home, &f.next, &f.vault).unwrap());
        finish(&f.root, &f.home, &p, &f.next, &f.vault).unwrap();
        assert!(!f.pending());
        assert!(f.vault.removes.borrow().is_empty());
        assert!(f.vault.has("Old/a"));
    }
}
#[test]
fn externally_referenced_targets_anywhere_are_preserved_until_unreferenced() {
    let mut f = Fixture::new();
    let p = f.prepare().unwrap();
    let mut ext = f.original.clone();
    ext["otherExtension"] = json!({"hold":["keychain:New/b"]});
    assert!(abort(&f.root, &f.home, &p, &ext, &f.vault).is_err());
    assert!(f.vault.has("New/b"));
    assert!(!f.vault.has("New/a"));
    assert!(f.pending());
    assert!(recover(&f.root, &f.home, &ext, &f.vault).is_err());
    abort(&f.root, &f.home, &p, &f.original, &f.vault).unwrap();
    assert!(!f.vault.has("New/b"));
    assert!(f.vault.has("Old/b"));
}
#[test]
fn external_replacement_loses_ownership_even_if_the_old_value_returns() {
    let mut f = Fixture::new();
    let p = f.prepare().unwrap();
    f.vault.put("New/a", "external-SECRET");
    assert!(abort(&f.root, &f.home, &p, &f.original, &f.vault).is_err());
    f.vault.put("New/a", "fixture-a-SECRET");
    for _ in 0..2 {
        assert!(abort(&f.root, &f.home, &p, &f.original, &f.vault).is_err());
    }
    assert!(f.vault.has("New/a"));
    assert_eq!(f.journal().spec.id, p.spec.id);
    f.vault.values.borrow_mut().remove("New/a");
    abort(&f.root, &f.home, &p, &f.original, &f.vault).unwrap();
    assert!(!f.pending());
    assert!(f.vault.has("Old/a"));
}
#[test]
fn removal_error_is_uncertain_and_retried_without_claiming_ownership() {
    let mut f = Fixture::new();
    let p = f.prepare().unwrap();
    f.vault.fail_remove.set(true);
    assert!(abort(&f.root, &f.home, &p, &f.original, &f.vault).is_err());
    f.vault.fail_remove.set(false);
    assert!(abort(&f.root, &f.home, &p, &f.original, &f.vault).is_err());
    assert!(f.vault.has("New/a"));
    assert_eq!(f.journal().spec.id, p.spec.id);
    f.vault.values.borrow_mut().remove("New/a");
    abort(&f.root, &f.home, &p, &f.original, &f.vault).unwrap();
}
#[test]
fn finish_readback_failure_and_changed_actual_keep_journal_and_secrets() {
    let mut f = Fixture::new();
    let p = f.prepare().unwrap();
    *f.vault.fail_read.borrow_mut() = Some("New/b".into());
    for _ in 0..2 {
        assert!(finish(&f.root, &f.home, &p, &f.next, &f.vault).is_err());
        assert!(f.pending());
    }
    *f.vault.fail_read.borrow_mut() = None;
    let mut changed = f.next.clone();
    changed["providers"]["New"]["baseUrl"] = json!("https://other.test");
    assert!(finish(&f.root, &f.home, &p, &changed, &f.vault).is_err());
    assert!(f.vault.has("Old/a"));
    assert!(f.vault.has("New/a"));
    finish(&f.root, &f.home, &p, &f.next, &f.vault).unwrap();
}
#[test]
fn crash_during_abort_never_deletes_external_replacement() {
    let mut f = Fixture::new();
    let p = f.prepare().unwrap();
    {
        let _i = Inject::at("after_remove", 1, true);
        assert!(abort(&f.root, &f.home, &p, &f.original, &f.vault).is_err());
    }
    f.vault.put("New/a", "fixture-a-SECRET");
    assert!(recover(&f.root, &f.home, &f.original, &f.vault).is_err());
    assert!(f.vault.has("New/a"));
    assert!(f.vault.has("Old/a"));
}
#[test]
fn limits_fail_without_copying() {
    let mut f = Fixture::new();
    f.next["providers"]["New"]["headers"] = json!({"many":vec!["keychain:Old/a";513]});
    assert!(f.prepare().is_err());
    assert!(f.vault.reads.borrow().is_empty());
    let mut f = Fixture::new();
    f.vault.put("Old/a", &"x".repeat(MAX_SECRET + 1));
    assert!(f.prepare().is_err());
    assert_eq!(f.vault.creates.get(), 0);
    let mut f = Fixture::new();
    let s: Vec<_> = (0..33).map(|_| mapping("Old", "New")).collect();
    assert!(prepare(&f.root, &f.home, &f.original, &mut f.next, &s, &f.vault).is_err());
    assert!(f.vault.reads.borrow().is_empty());
}
#[test]
fn metadata_is_private_and_contains_no_plaintext() {
    let mut f = Fixture::new();
    let p = f.prepare().unwrap();
    let c = Context::open(&f.root, &f.home).unwrap();
    let dir = c.operation(&p.spec.id, false).unwrap();
    for b in [
        c.manager.read(PENDING, MAX_IMAGE, true).unwrap().unwrap(),
        dir.read(JOURNAL, MAX_IMAGE, true).unwrap().unwrap(),
    ] {
        assert!(!String::from_utf8_lossy(&b).contains("SECRET"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(dir.path.join(JOURNAL))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            std::fs::metadata(&dir.path).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
    abort(&f.root, &f.home, &p, &f.original, &f.vault).unwrap();
}
#[test]
fn wrong_home_and_tampered_record_fail_before_vault_reads() {
    let mut f = Fixture::new();
    let p = f.prepare().unwrap();
    f.vault.reads.borrow_mut().clear();
    let other = f.root.join("other");
    std::fs::create_dir(&other).unwrap();
    assert!(finish(&f.root, &other, &p, &f.next, &f.vault).is_err());
    assert!(recover(&f.root, &other, &f.next, &f.vault).is_err());
    assert!(f.vault.reads.borrow().is_empty());
    let c = Context::open(&f.root, &f.home).unwrap();
    let dir = c.operation(&p.spec.id, false).unwrap();
    let mut j = f.journal();
    j.spec.copies[0].from = "Foreign/a".into();
    let b = dir.read(JOURNAL, MAX_IMAGE, true).unwrap().unwrap();
    dir.replace(JOURNAL, Some(&b), Some(&envelope(&j).unwrap()), true)
        .unwrap();
    assert!(recover(&f.root, &f.home, &f.next, &f.vault).is_err());
    assert!(f.vault.reads.borrow().is_empty());
}
#[cfg(unix)]
#[test]
fn symlink_root_home_parent_marker_and_hardlink_record_fail_closed() {
    use std::os::unix::fs::symlink;
    for mode in 0..5 {
        let mut f = Fixture::new();
        if mode == 0 {
            let link = f.root.with_file_name("link");
            symlink(&f.root, &link).unwrap();
            f.root = link;
            assert!(f.prepare().is_err());
        } else if mode == 1 {
            symlink(&f.home, f.root.join("manager-state")).unwrap();
            assert!(f.prepare().is_err());
        } else if mode == 2 {
            let link = f.home.with_file_name("link");
            symlink(&f.home, &link).unwrap();
            f.home = link;
            assert!(f.prepare().is_err());
        } else {
            let p = f.prepare().unwrap();
            f.vault.reads.borrow_mut().clear();
            let c = Context::open(&f.root, &f.home).unwrap();
            let op = c.operation(&p.spec.id, false).unwrap();
            if mode == 3 {
                let marker = c.manager.path.join(PENDING);
                std::fs::remove_file(&marker).unwrap();
                symlink(op.path.join(JOURNAL), marker).unwrap();
            } else {
                std::fs::hard_link(op.path.join(JOURNAL), f.home.join("link")).unwrap();
            }
            assert!(recover(&f.root, &f.home, &f.next, &f.vault).is_err());
            assert!(f.vault.reads.borrow().is_empty());
        }
    }
}
#[test]
fn plain_and_env_only_can_change_url_without_vault_reads() {
    let mut f = Fixture::new();
    f.original =
        json!({"providers":{"Old":{"apiKey":"$ENV_UNDEFINED","headers":{"X":"plain-SECRET"}}}});
    f.next = json!({"providers":{"New":{"apiKey":"$ENV_UNDEFINED","headers":{"X":"plain-SECRET"},"baseUrl":"https://changed.test"}}});
    let p = f.prepare().unwrap();
    assert!(f.vault.reads.borrow().is_empty());
    finish(&f.root, &f.home, &p, &f.next, &f.vault).unwrap();
    assert_eq!(f.next["providers"]["New"]["apiKey"], "$ENV_UNDEFINED");
}
#[test]
fn recovery_completes_exact_original_plan_without_new_namespaces_or_uuid() {
    for point in ["prepared", "before_create", "after_create", "receipt"] {
        let mut f = Fixture::new();
        let mut expected = f.next.clone();
        rewrite(&mut expected, &[mapping("Old", "New")], false).unwrap();
        {
            let _i = Inject::at(point, 1, true);
            assert!(f.prepare().is_err());
        }
        let id = f.journal().spec.id;
        assert!(recover(&f.root, &f.home, &expected, &f.vault).unwrap());
        assert_eq!(f.vault.creates.get(), 6);
        let ctx = Context::open(&f.root, &f.home).unwrap();
        let dir = ctx.operation(&id, false).unwrap();
        let j: Journal = decode(&dir.read(JOURNAL, MAX_IMAGE, true).unwrap().unwrap()).unwrap();
        assert_eq!(j.spec.id, id);
        assert!(j.phase == Phase::Finished);
        for s in ["a", "b", "header", "proxy", "mcp", "executor"] {
            assert!(f.vault.same(&format!("Old/{s}"), &format!("New/{s}")));
        }
        assert!(f.vault.removes.borrow().is_empty());
        f.no_config_write();
    }
}
#[test]
fn resumed_copies_verify_source_digest_and_all_target_collisions_before_writing() {
    for source_changed in [true, false] {
        let mut f = Fixture::new();
        let mut expected = f.next.clone();
        rewrite(&mut expected, &[mapping("Old", "New")], false).unwrap();
        {
            let _i = Inject::at("prepared", 1, true);
            assert!(f.prepare().is_err());
        }
        let id = f.journal().spec.id;
        f.vault.put(
            if source_changed {
                "Old/proxy"
            } else {
                "New/proxy"
            },
            "external-SECRET",
        );
        for _ in 0..2 {
            assert!(recover(&f.root, &f.home, &expected, &f.vault).is_err());
        }
        assert_eq!(f.vault.creates.get(), 0);
        assert_eq!(f.journal().spec.id, id);
        assert!(f.pending());
    }
}
#[test]
fn previously_receipted_but_later_deleted_target_is_not_recreated() {
    let mut f = Fixture::new();
    let p = f.prepare().unwrap();
    f.vault.values.borrow_mut().remove("New/a");
    assert!(finish(&f.root, &f.home, &p, &f.next, &f.vault).is_err());
    assert_eq!(f.vault.creates.get(), 6);
    assert!(f.pending());
    assert!(f.vault.has("Old/a"));
}
#[test]
fn same_provider_mapping_never_claims_source_creation_or_removes_it() {
    let mut f = Fixture::new();
    f.next = f.original.clone();
    let p = prepare(
        &f.root,
        &f.home,
        &f.original,
        &mut f.next,
        &[mapping("Old", "Old")],
        &f.vault,
    )
    .unwrap();
    abort(&f.root, &f.home, &p, &f.original, &f.vault).unwrap();
    assert_eq!(f.vault.creates.get(), 0);
    assert!(f.vault.has("Old/a"));
    assert!(f.vault.removes.borrow().is_empty());
}
#[test]
fn more_than_thirty_two_distinct_providers_refuses_before_reading() {
    let mut f = Fixture::new();
    let row = f.original["providers"]["Old"].clone();
    let mut sources = Vec::new();
    for i in 0..17 {
        let a = format!("Source{i}");
        let b = format!("Target{i}");
        f.original["providers"][&a] = row.clone();
        f.next["providers"][&b] = row.clone();
        // ENV-only avoids an earlier account-ownership failure.
        f.original["providers"][&a] = json!({"apiKey":"$ENV"});
        f.next["providers"][&b] = json!({"apiKey":"$ENV"});
        sources.push(mapping(&a, &b));
    }
    assert!(prepare(
        &f.root,
        &f.home,
        &f.original,
        &mut f.next,
        &sources,
        &f.vault
    )
    .is_err());
    assert!(f.vault.reads.borrow().is_empty());
    assert_eq!(f.vault.creates.get(), 0);
}
#[cfg(unix)]
#[test]
fn parent_components_and_oversized_or_unprotected_metadata_refuse_without_vault_reads() {
    use std::os::unix::fs::PermissionsExt;
    let mut f = Fixture::new();
    let bad = f.root.join("..").join("data");
    assert!(prepare(
        &bad,
        &f.home,
        &f.original,
        &mut f.next,
        &[mapping("Old", "New")],
        &f.vault
    )
    .is_err());
    let p = f.prepare().unwrap();
    f.vault.reads.borrow_mut().clear();
    let c = Context::open(&f.root, &f.home).unwrap();
    let dir = c.operation(&p.spec.id, false).unwrap();
    let file = dir.path.join(JOURNAL);
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(recover(&f.root, &f.home, &f.next, &f.vault).is_err());
    assert!(f.vault.reads.borrow().is_empty());
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();
    std::fs::write(file, vec![b'x'; MAX_IMAGE + 1]).unwrap();
    assert!(recover(&f.root, &f.home, &f.next, &f.vault).is_err());
    assert!(f.vault.reads.borrow().is_empty());
}
#[test]
fn complete_256_slot_pool_and_nonfirst_active_slot_are_preserved() {
    let mut f = Fixture::new();
    let mut slots = Vec::new();
    for i in 0..256 {
        let account = format!("Old/{i:08x}");
        f.vault.put(&account, &format!("slot-{i}-SECRET"));
        slots.push(json!({"id":format!("{i:08x}"),"key":format!("keychain:{account}")}));
    }
    f.original = json!({"providers":{"Old":{"baseUrl":"https://fixture.test","apiKey":"keychain:Old/000000ff","apiKeyPool":slots}}});
    f.next = json!({"providers":{"New":f.original["providers"]["Old"]}});
    let p = f.prepare().unwrap();
    assert_eq!(f.vault.creates.get(), 256);
    assert_eq!(
        f.next["providers"]["New"]["apiKey"],
        "keychain:New/000000ff"
    );
    assert_eq!(
        f.next["providers"]["New"]["apiKeyPool"]
            .as_array()
            .unwrap()
            .len(),
        256
    );
    finish(&f.root, &f.home, &p, &f.next, &f.vault).unwrap();
    for i in 0..256 {
        assert!(f
            .vault
            .same(&format!("Old/{i:08x}"), &format!("New/{i:08x}")));
    }
}
thread_local! {static EXTERNAL_FILE: RefCell<Option<(PathBuf,Vec<u8>)>>=const {RefCell::new(None)};}
fn external_change_hook(point: &str) -> Result<()> {
    if point == "receipt" {
        EXTERNAL_FILE.with(|s| {
            let (path, bytes) = s.borrow_mut().take().unwrap();
            std::fs::write(path, bytes).unwrap();
        });
        return Err(fault("mock_external_change"));
    }
    Ok(())
}
#[test]
fn prepare_failure_compensates_against_actual_file_not_stale_original() {
    let mut f = Fixture::new();
    let mut external = f.original.clone();
    external["external"] = json!("keychain:New/a");
    EXTERNAL_FILE.with(|s| {
        *s.borrow_mut() = Some((
            f.home.join("config.json"),
            serde_json::to_vec(&external).unwrap(),
        ))
    });
    {
        let _reset = Inject;
        HOOK.with(|h| *h.borrow_mut() = Some(external_change_hook));
        assert!(f.prepare().is_err());
    }
    assert!(f.vault.has("New/a"));
    assert!(f.vault.has("Old/a"));
    assert!(f.pending());
    assert!(f.vault.removes.borrow().is_empty());
    assert!(recover(&f.root, &f.home, &external, &f.vault).is_err());
    let j = f.journal();
    let p = ReferencePlanV2 { spec: j.spec };
    abort(&f.root, &f.home, &p, &f.original, &f.vault).unwrap();
    assert!(!f.vault.has("New/a"));
}
