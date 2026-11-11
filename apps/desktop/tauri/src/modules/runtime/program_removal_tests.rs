use super::*;
use crate::modules::runtime::program_activation::tests::Fixture;
use crate::modules::runtime::program_selection::tests::Fixture as ProgramFixture;
struct Test {
    program: ProgramFixture,
    _user: tempfile::TempDir,
    d: PathBuf,
    h: PathBuf,
    old: CurrentSelection,
    new: CurrentSelection,
    config: Vec<u8>,
}
impl Test {
    fn new() -> Self {
        let program = ProgramFixture::new();
        let user = tempfile::tempdir().unwrap();
        let base = user.path().canonicalize().unwrap();
        let d = base.join("desktop");
        let h = base.join("opencodex-home");
        fs::create_dir_all(d.join("manager-state")).unwrap();
        fs::create_dir(&h).unwrap();
        let config =
            br#"{ "port":41234,"providers":{"test":{"apiKey":"fixture-only"}},"keep":true }"#
                .to_vec();
        fs::write(h.join("config.json"), &config).unwrap();
        let (target, platform) = program_layout::host_target().unwrap();
        let old = program_selection::read(&program.root, platform, target)
            .unwrap()
            .unwrap()
            .selection;
        let new = Fixture::image(&program.root, "3.0.0");
        RuntimeStore::new(&d)
            .save(&RuntimeSourceRecord {
                source: RuntimeSourceKind::Managed,
                path: Some(entry(&program.root).unwrap().to_string_lossy().into_owned()),
                resolved_version: Some(old.version.clone()),
                ..Default::default()
            })
            .unwrap();
        Self {
            program,
            _user: user,
            d,
            h,
            old,
            new,
            config,
        }
    }
    fn plan(&self) -> Prepared {
        prepare(
            &self.program.root,
            &self.d,
            &self.h,
            &[self.old.clone(), self.new.clone()],
        )
        .unwrap()
    }
    fn unchanged(&self) {
        assert_eq!(fs::read(self.h.join("config.json")).unwrap(), self.config);
        assert!(self.program.paths.version_root.is_dir());
        assert!(current(&self.program.root).is_file());
    }
    fn finished(&self) {
        assert_eq!(fs::read(self.h.join("config.json")).unwrap(), self.config);
        assert_eq!(
            fs::read_dir(self.program.root.join("runtime/versions"))
                .unwrap()
                .count(),
            0
        );
        assert!(!current(&self.program.root).exists());
        assert!(!entry(&self.program.root).unwrap().exists());
        assert!(!pending(&self.program.root).exists());
        assert!(intentionally_removed(&self.program.root, &self.d).unwrap());
        let (target, platform) = program_layout::host_target().unwrap();
        assert!(
            program_selection::read(&self.program.root, platform, target)
                .unwrap()
                .is_none()
        );
        let record = RuntimeStore::new(&self.d).load();
        assert_eq!(record.source, RuntimeSourceKind::Unresolved);
        assert_eq!(record.history.last().unwrap().action, "uninstall");
    }
}
#[test]
fn exact_named_images_removed_and_user_configuration_backups_and_unrelated_objects_kept() {
    let f = Test::new();
    let unrelated = f.program.root.join("runtime/bin/other-tool");
    fs::write(&unrelated, b"another program").unwrap();
    let plan = f.plan();
    let preview = plan.preview().unwrap();
    assert_eq!(preview.versions, vec!["2.99.0", "3.0.0"]);
    assert!(preview.files > 0);
    assert!(!f.d.join("backups").exists());
    let revoked = std::cell::Cell::new(false);
    let outcome = plan
        .execute(
            &preview.confirmation_digest,
            || Ok(()),
            || {
                revoked.set(true);
                Ok(())
            },
        )
        .unwrap();
    assert!(revoked.get());
    assert!(outcome.configuration_preserved);
    f.finished();
    assert_eq!(fs::read(unrelated).unwrap(), b"another program");
    let backups = crate::modules::backup::safety::scan(&f.d.join("backups")).unwrap();
    assert_eq!(backups.len(), 1);
    assert_eq!(
        fs::read(backups[0].directory.join("config.json")).unwrap(),
        f.config
    );
}
#[test]
fn unknown_version_changed_file_confirmation_and_revoke_or_backup_failures_do_not_remove_code() {
    for case in ["version", "file", "digest", "owner", "terminal", "backup"] {
        let f = Test::new();
        let plan = f.plan();
        let p = plan.preview().unwrap();
        if case == "version" {
            fs::create_dir(f.program.root.join("runtime/versions/unknown")).unwrap();
        }
        if case == "file" {
            fs::write(
                f.program.paths.version_root.join("added-user-file"),
                b"external",
            )
            .unwrap();
        }
        if case == "backup" {
            fs::write(f.d.join("backups"), b"blocked").unwrap();
        }
        let digest = if case == "digest" {
            "wrong"
        } else {
            &p.confirmation_digest
        };
        assert!(
            plan.execute(
                digest,
                || if case == "owner" {
                    Err(fail("program still running"))
                } else {
                    Ok(())
                },
                || if case == "terminal" {
                    Err(fail("unowned terminal"))
                } else {
                    Ok(())
                }
            )
            .is_err(),
            "{case}"
        );
        f.unchanged();
        assert!(!pending(&f.program.root).exists());
        assert!(!removed(&f.program.root).exists());
    }
}
#[test]
fn every_removal_phase_has_a_continuable_original_operation_without_recreating_deleted_files() {
    for phase in [
        "prepared",
        "metadata",
        "file",
        "directory",
        "image",
        "record",
        "marker",
    ] {
        let f = Test::new();
        let plan = f.plan();
        let p = plan.preview().unwrap();
        let result = plan.execute_inner(
            &p.confirmation_digest,
            || Ok(()),
            || Ok(()),
            |at| {
                if at == phase {
                    Err(fail("injected interruption"))
                } else {
                    Ok(())
                }
            },
        );
        assert!(result.is_err(), "{phase}");
        assert!(pending(&f.program.root).exists());
        let (target, platform) = program_layout::host_target().unwrap();
        assert!(program_selection::read(&f.program.root, platform, target).is_err());
        let outcome = resume(
            &f.program.root,
            &f.d,
            &f.h,
            &p.confirmation_digest,
            || Ok(()),
            || Ok(()),
        )
        .unwrap();
        assert_eq!(outcome.operation_id, p.operation_id);
        f.finished();
    }
}
#[test]
fn continuation_refuses_unknown_surviving_content_or_a_foreign_data_root_and_keeps_it() {
    for kind in ["add", "modify", "other-data", "reappear-entry"] {
        let f = Test::new();
        let plan = f.plan();
        let p = plan.preview().unwrap();
        assert!(plan
            .execute_inner(
                &p.confirmation_digest,
                || Ok(()),
                || Ok(()),
                |phase| if phase == "metadata" {
                    Err(fail("interrupt"))
                } else {
                    Ok(())
                }
            )
            .is_err());
        let sentinel = if kind == "modify" {
            f.program.paths.npm.clone()
        } else {
            f.program.paths.version_root.join("new-foreign-file")
        };
        if kind == "add" || kind == "modify" {
            fs::write(&sentinel, b"external data retained").unwrap();
        }
        if kind == "reappear-entry" {
            fs::write(entry(&f.program.root).unwrap(), b"foreign replacement").unwrap();
        }
        let other = f.d.with_file_name("different-data");
        fs::create_dir(&other).unwrap();
        let data = if kind == "other-data" { &other } else { &f.d };
        let pending_before = fs::read(pending(&f.program.root)).unwrap();
        assert!(resume(
            &f.program.root,
            data,
            &f.h,
            &p.confirmation_digest,
            || Ok(()),
            || Ok(())
        )
        .is_err());
        assert_eq!(fs::read(pending(&f.program.root)).unwrap(), pending_before);
        assert!(f.program.paths.version_root.is_dir());
        if kind == "add" || kind == "modify" {
            assert_eq!(fs::read(sentinel).unwrap(), b"external data retained");
        }
        if kind == "other-data" {
            assert_eq!(fs::read_dir(other).unwrap().count(), 0);
        }
    }
}
#[test]
fn normal_startup_after_explicit_removal_does_not_resupply_or_require_bundle_files() {
    let f = Test::new();
    let plan = f.plan();
    plan.execute(
        &plan.preview().unwrap().confirmation_digest,
        || Ok(()),
        || Ok(()),
    )
    .unwrap();
    let handle = super::super::RuntimeHandle::initialize_versioned(&f.d, &f.program.root);
    let called = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let signal = called.clone();
    super::super::startup_bundle::start(
        f.program.root.clone(),
        f.d.clone(),
        f.program.root.join("no-such-resources"),
        "not-used",
        handle,
        crate::state::SharedRuntimeInstall::new(),
        move || signal.store(true, std::sync::atomic::Ordering::SeqCst),
    )
    .unwrap();
    assert!(called.load(std::sync::atomic::Ordering::SeqCst));
    f.finished();
}

#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
#[test]
fn actual_owned_terminal_revoke_is_part_of_removal_and_foreign_replacement_blocks_it() {
    use crate::modules::runtime::terminal_cli::{self, tests::Fixture as Terminal, Action};
    for foreign in [false, true] {
        let f = Terminal::new();
        let c = &f.c;
        let config = br#"{"port":41234,"providers":{}}"#;
        fs::write(c.opencodex_home.join("config.json"), config).unwrap();
        f.change(Action::Enable);
        let selected = program_selection::read(
            &c.program_root,
            program_layout::host_target().unwrap().1,
            "darwin-arm64",
        )
        .unwrap()
        .unwrap();
        let plan = prepare(
            &c.program_root,
            &c.desktop_root,
            &c.opencodex_home,
            std::slice::from_ref(&selected.selection),
        )
        .unwrap();
        let command = c.user_profile.join(".local/bin/ocx");
        if foreign {
            fs::write(&command, b"foreign replacement").unwrap();
        }
        let result = plan.execute(
            &plan.preview().unwrap().confirmation_digest,
            || Ok(()),
            || {
                let preview = terminal_cli::preview(c, Action::Disable, "/bin/zsh", &[])?;
                terminal_cli::apply(
                    c,
                    Action::Disable,
                    "/bin/zsh",
                    &[],
                    &preview.confirmation_digest,
                )
                .map(|_| ())
            },
        );
        assert_eq!(
            fs::read(c.opencodex_home.join("config.json")).unwrap(),
            config
        );
        if foreign {
            assert!(result.is_err());
            assert_eq!(fs::read(command).unwrap(), b"foreign replacement");
            assert!(selected.paths.version_root.exists());
            assert!(current(&c.program_root).exists());
            assert!(!pending(&c.program_root).exists());
        } else {
            result.unwrap();
            assert!(!command.exists());
            assert!(!c.program_root.join("runtime/bin/ocx").exists());
            assert!(!c.user_profile.join(".zshrc").exists());
            assert!(intentionally_removed(&c.program_root, &c.desktop_root).unwrap());
        }
    }
}

#[test]
fn registered_preparation_binds_ledger_and_continuation_uses_original_admissions_after_partial_deletion(
) {
    let f = Test::new();
    let (target, platform) = program_layout::host_target().unwrap();
    for selected in [&f.old, &f.new] {
        let image =
            program_selection::read_image(&f.program.root, platform, target, selected).unwrap();
        super::super::program_catalog::register(
            &f.program.root,
            selected,
            &image.image.official_archive_sha256,
        )
        .unwrap();
    }
    let plan = prepare_registered(&f.program.root, &f.d, &f.h).unwrap();
    let preview = plan.preview().unwrap();
    let path = fs::read_dir(f.program.root.join("runtime/image-admissions"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let original = fs::read(&path).unwrap();
    let mut changed = original.clone();
    changed.push(b' ');
    fs::write(&path, &changed).unwrap();
    assert!(plan
        .execute(&preview.confirmation_digest, || Ok(()), || Ok(()))
        .is_err());
    f.unchanged();
    assert!(!pending(&f.program.root).exists());
    fs::write(&path, &original).unwrap();
    assert!(plan
        .execute_inner(
            &preview.confirmation_digest,
            || Ok(()),
            || Ok(()),
            |phase| if phase == "file" {
                Err(fail("interrupt"))
            } else {
                Ok(())
            }
        )
        .is_err());
    assert!(super::super::program_catalog::inspect(&f.program.root).is_err());
    assert_eq!(
        super::super::program_catalog::revision(&f.program.root).unwrap(),
        plan.0.catalog_revision.as_ref().unwrap().as_str()
    );
    resume(
        &f.program.root,
        &f.d,
        &f.h,
        &preview.confirmation_digest,
        || Ok(()),
        || Ok(()),
    )
    .unwrap();
    f.finished();
    assert_eq!(
        fs::read(path).unwrap(),
        original,
        "source admissions retained as evidence"
    );
}
