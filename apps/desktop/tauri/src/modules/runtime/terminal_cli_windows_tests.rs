//! Memory registry + owned filesystem tests of the production transaction engine.
//! No Windows API, daily PATH or installed executable is used on this host.
use super::*;
use std::cell::{Cell, RefCell};
use std::os::unix::fs::PermissionsExt;
struct Memory {
    value: RefCell<Option<PathValue>>,
    writes: Cell<usize>,
    notifies: Cell<usize>,
    fail_notify: Cell<bool>,
    external_after_write: Cell<bool>,
}
impl Memory {
    fn new() -> Self {
        Self {
            value: RefCell::new(None),
            writes: Cell::new(0),
            notifies: Cell::new(0),
            fail_notify: Cell::new(false),
            external_after_write: Cell::new(false),
        }
    }
}
impl Registry for Memory {
    fn read(&self) -> AppResult<Option<PathValue>> {
        Ok(self.value.borrow().clone())
    }
    fn replace(&self, before: &Option<PathValue>, after: &Option<PathValue>) -> AppResult<()> {
        if &*self.value.borrow() != before {
            return Err(fail("synthetic expected conflict"));
        }
        *self.value.borrow_mut() = after.clone();
        self.writes.set(self.writes.get() + 1);
        if self.external_after_write.get() {
            *self.value.borrow_mut() = Some(PathValue::encode(1, "outside-change")?);
            return Err(fail("synthetic postimage mismatch"));
        }
        Ok(())
    }
    fn notify(&self) -> AppResult<()> {
        self.notifies.set(self.notifies.get() + 1);
        if self.fail_notify.get() {
            Err(fail("synthetic notify refusal"))
        } else {
            Ok(())
        }
    }
}
fn fixture() -> (super::super::tests::Fixture, Memory) {
    let f = super::super::tests::Fixture::new();
    fs::write(console(&f.c), b"owned console fixture, never executed").unwrap();
    fs::set_permissions(console(&f.c), fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(command(&f.c), b"native entry fixture, never executed").unwrap();
    write_source(&f.c);
    (f, Memory::new())
}
fn write_source(c: &Context) {
    let source = InstallSource {
        schema_version: 1,
        package: "opencodex-desktop".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        entry_kind: ENTRY_KIND.into(),
        files: ["ocx.exe", "ocxd.exe"]
            .into_iter()
            .map(|n| {
                (
                    n.into(),
                    hash::sha256_file(&c.program_root.join(n)).unwrap(),
                )
            })
            .collect(),
    };
    fs::write(c.program_root.join(ENTRY_SOURCE), encode(&source).unwrap()).unwrap();
}
fn execute(c: &Context, action: Action, r: &Memory) {
    let p = plan(c, action, &[], r).unwrap().1;
    apply_with(c, action, &[], &p.confirmation_digest, r, &|_| Ok(())).unwrap();
}
#[test]
fn windows_policy_transaction_enables_disables_and_keeps_unrelated_files() {
    let (f, r) = fixture();
    let unrelated = f.c.user_profile.join("keep.txt");
    fs::write(&unrelated, b"keep").unwrap();
    let p = inspect_with(&f.c, &[], &r).unwrap();
    assert!(!p.enabled && !p.pending);
    assert!(command(&f.c).exists());
    execute(&f.c, Action::Enable, &r);
    assert_eq!(
        fs::read(command(&f.c)).unwrap(),
        b"native entry fixture, never executed"
    );
    require_forwarding(&f.c).unwrap();
    assert!(installed(&f.c).unwrap().is_some());
    assert_eq!(r.writes.get(), 1);
    assert_eq!(inspect_with(&f.c, &[], &r).unwrap().action, Action::Disable);
    let removal = removal_with(&f.c, &[], &r).unwrap().unwrap();
    assert_eq!(removal.action, Action::Disable);
    execute(&f.c, Action::Disable, &r);
    assert_eq!(r.read().unwrap(), None);
    assert!(command(&f.c).exists() && !f.c.record().exists() && !f.c.pending().exists());
    assert_eq!(fs::read(unrelated).unwrap(), b"keep");
    assert!(removal_with(&f.c, &[], &r).unwrap().is_none());
}
#[test]
fn every_published_boundary_resumes_same_action_without_new_baseline() {
    for action in [Action::Enable, Action::Disable] {
        for at in ["pending", "path", "command", "record", "notified"] {
            let (f, r) = fixture();
            if action == Action::Disable {
                execute(&f.c, Action::Enable, &r);
            }
            let p = plan(&f.c, action, &[], &r).unwrap().1;
            let result = apply_with(&f.c, action, &[], &p.confirmation_digest, &r, &|point| {
                if point == at {
                    Err(fail("synthetic crash boundary"))
                } else {
                    Ok(())
                }
            });
            assert!(result.is_err(), "{action:?}/{at}");
            assert!(f.c.pending().exists());
            let raw = fs::read(f.c.pending()).unwrap();
            let preview = inspect_with(&f.c, &[], &r).unwrap();
            assert!(preview.pending);
            assert_eq!(preview.action, action);
            let opposite = if action == Action::Enable {
                Action::Disable
            } else {
                Action::Enable
            };
            assert!(plan(&f.c, opposite, &[], &r).is_err());
            assert_eq!(fs::read(f.c.pending()).unwrap(), raw);
            apply_with(&f.c, action, &[], &preview.confirmation_digest, &r, &|_| {
                Ok(())
            })
            .unwrap();
            assert!(!f.c.pending().exists());
            assert!(command(&f.c).exists());
            assert_eq!(f.c.record().exists(), action == Action::Enable);
        }
    }
}
#[test]
fn changed_confirmation_command_record_and_path_are_never_adopted() {
    let (f, r) = fixture();
    let p = plan(&f.c, Action::Enable, &[], &r).unwrap().1;
    *r.value.borrow_mut() = Some(PathValue::encode(1, "external-before").unwrap());
    assert!(apply_with(
        &f.c,
        Action::Enable,
        &[],
        &p.confirmation_digest,
        &r,
        &|_| Ok(())
    )
    .is_err());
    assert!(!f.c.pending().exists());
    assert_eq!(r.writes.get(), 0);
    for object in ["command", "record", "path", "pending"] {
        let (f, r) = fixture();
        let p = plan(&f.c, Action::Enable, &[], &r).unwrap().1;
        assert!(apply_with(
            &f.c,
            Action::Enable,
            &[],
            &p.confirmation_digest,
            &r,
            &|point| if point == "command" {
                Err(fail("synthetic interruption"))
            } else {
                Ok(())
            }
        )
        .is_err());
        let original = fs::read(f.c.pending()).unwrap();
        match object {
            "command" => fs::write(command(&f.c), b"foreign command").unwrap(),
            "record" => fs::write(f.c.record(), b"foreign record").unwrap(),
            "path" => *r.value.borrow_mut() = Some(PathValue::encode(1, "outside-after").unwrap()),
            "pending" => {
                let mut t: Transaction = decode(&original).unwrap();
                t.path_after = Some(PathValue::encode(1, "arbitrary-target").unwrap());
                fs::write(f.c.pending(), encode(&t).unwrap()).unwrap();
            }
            _ => unreachable!(),
        };
        let writes = r.writes.get();
        assert!(inspect_with(&f.c, &[], &r).is_err(), "{object}");
        assert_eq!(r.writes.get(), writes);
        if object != "pending" {
            assert_eq!(fs::read(f.c.pending()).unwrap(), original);
        }
        assert!(f.c.pending().exists());
    }
}
#[test]
fn notify_failure_and_external_postwrite_conflict_remain_pending() {
    let (f, r) = fixture();
    r.fail_notify.set(true);
    let p = plan(&f.c, Action::Enable, &[], &r).unwrap().1;
    assert!(apply_with(
        &f.c,
        Action::Enable,
        &[],
        &p.confirmation_digest,
        &r,
        &|_| Ok(())
    )
    .is_err());
    assert_eq!(r.writes.get(), 1);
    assert!(f.c.pending().exists());
    r.fail_notify.set(false);
    execute(&f.c, Action::Enable, &r);
    assert_eq!(r.writes.get(), 1);
    assert_eq!(r.notifies.get(), 2);
    let (f, r) = fixture();
    r.external_after_write.set(true);
    let p = plan(&f.c, Action::Enable, &[], &r).unwrap().1;
    assert!(apply_with(
        &f.c,
        Action::Enable,
        &[],
        &p.confirmation_digest,
        &r,
        &|_| Ok(())
    )
    .is_err());
    assert!(f.c.pending().exists());
    assert!(inspect_with(&f.c, &[], &r).is_err());
    assert_eq!(r.read().unwrap().unwrap().text().unwrap(), "outside-change");
    assert!(!f.c.record().exists());
}
#[test]
fn preexisting_path_and_later_user_edits_are_preserved() {
    let (f, r) = fixture();
    let text = directory(&f.c).unwrap();
    let before = Some(PathValue::encode(1, &text).unwrap());
    *r.value.borrow_mut() = before.clone();
    execute(&f.c, Action::Enable, &r);
    execute(&f.c, Action::Disable, &r);
    assert_eq!(r.read().unwrap(), before);
    assert_eq!(r.writes.get(), 0);
    let (f, r) = fixture();
    execute(&f.c, Action::Enable, &r);
    let text = format!(
        "{};{};",
        directory(&f.c).unwrap(),
        f.c.user_profile.display()
    );
    *r.value.borrow_mut() = Some(PathValue::encode(1, &text).unwrap());
    execute(&f.c, Action::Disable, &r);
    assert_eq!(
        r.read().unwrap().unwrap().text().unwrap(),
        format!("{};", f.c.user_profile.display())
    );
}
#[test]
fn conflicts_forged_context_and_helper_changes_refuse() {
    let (f, r) = fixture();
    let other = f.c.user_profile.join("other-bin");
    fs::create_dir(&other).unwrap();
    fs::write(other.join("ocx.exe"), b"outside").unwrap();
    let p = plan(&f.c, Action::Enable, std::slice::from_ref(&other), &r)
        .unwrap()
        .1;
    assert_eq!(p.conflicts, vec![other.join("ocx.exe")]);
    assert!(apply_with(
        &f.c,
        Action::Enable,
        &[other],
        &p.confirmation_digest,
        &r,
        &|_| Ok(())
    )
    .is_err());
    assert!(!f.c.pending().exists());
    let p = plan(&f.c, Action::Enable, &[], &r).unwrap().1;
    fs::write(console(&f.c), b"changed helper").unwrap();
    assert!(apply_with(
        &f.c,
        Action::Enable,
        &[],
        &p.confirmation_digest,
        &r,
        &|_| Ok(())
    )
    .is_err());
    assert_eq!(r.writes.get(), 0);
    let mut wrong = f.c.clone();
    wrong.opencodex_home = f.c.user_profile.join("another-home");
    assert!(inspect_with(&wrong, &[], &r).is_err());
}

#[test]
fn current_facts_are_rechecked_after_full_image_readback() {
    for object in ["helper", "path", "command", "pending"] {
        let (f, r) = fixture();
        let p = plan(&f.c, Action::Enable, &[], &r).unwrap().1;
        let result = apply_with(
            &f.c,
            Action::Enable,
            &[],
            &p.confirmation_digest,
            &r,
            &|point| {
                if point == "image-readback" {
                    match object {
                        "helper" => fs::write(console(&f.c), b"later helper").unwrap(),
                        "path" => {
                            *r.value.borrow_mut() =
                                Some(PathValue::encode(1, "later path").unwrap())
                        }
                        "command" => fs::write(command(&f.c), b"later command").unwrap(),
                        "pending" => fs::write(f.c.pending(), b"later pending").unwrap(),
                        _ => unreachable!(),
                    }
                }
                Ok(())
            },
        );
        assert!(result.is_err(), "{object}");
        assert!(f.c.pending().exists());
        match object {
            "helper" => assert_eq!(fs::read(console(&f.c)).unwrap(), b"later helper"),
            "path" => assert_eq!(r.read().unwrap().unwrap().text().unwrap(), "later path"),
            "command" => assert_eq!(fs::read(command(&f.c)).unwrap(), b"later command"),
            "pending" => assert_eq!(fs::read(f.c.pending()).unwrap(), b"later pending"),
            _ => unreachable!(),
        }
    }
}

#[test]
fn pristine_installer_is_readonly_and_does_not_guess_a_lost_binding() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let profile = root.join("profile");
    let program = root.join("program");
    fs::create_dir(&profile).unwrap();
    fs::create_dir(&program).unwrap();
    assert!(pristine_installer(&profile, &program).unwrap());
    assert!(!profile.join(".codex-desktop").exists());
    assert!(!program.join("runtime").exists());
    fs::create_dir_all(program.join("runtime/bin")).unwrap();
    fs::write(program.join("runtime/bin/ocx.cmd"), b"unknown entry").unwrap();
    assert!(!pristine_installer(&profile, &program).unwrap());
    assert_eq!(
        fs::read(program.join("runtime/bin/ocx.cmd")).unwrap(),
        b"unknown entry"
    );
}
