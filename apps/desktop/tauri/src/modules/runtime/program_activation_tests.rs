use super::*;
use crate::modules::{
    data_root::layout::Platform,
    runtime::{
        program_layout::RuntimeVersionPaths,
        program_selection::{ImageManifest, IMAGE_MANIFEST},
        OFFICIAL_PACKAGE,
    },
};
use std::os::unix::fs::PermissionsExt;

struct Fixture {
    _temp: tempfile::TempDir,
    i: PathBuf,
    d: PathBuf,
    old: CurrentSelection,
    new: CurrentSelection,
    initial: bool,
}
impl Fixture {
    fn new(initial: bool) -> Self {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let i = base.join("program");
        let d = base.join("desktop");
        fs::create_dir_all(i.join("runtime/bin")).unwrap();
        fs::create_dir_all(d.join("manager-state")).unwrap();
        let old = Self::image(&i, "2.99.0");
        let new = Self::image(&i, "3.0.0");
        let f = Self {
            _temp: temp,
            i,
            d,
            old,
            new,
            initial,
        };
        if initial {
            let selected = f.admit(&f.old);
            fs::write(
                f.entry(),
                install::entry_script(Some(&selected.paths.node), &selected.script),
            )
            .unwrap();
            fs::set_permissions(f.entry(), fs::Permissions::from_mode(0o755)).unwrap();
            fs::write(f.current(), encode(&f.old).unwrap()).unwrap();
            let record = RuntimeSourceRecord {
                source: RuntimeSourceKind::Managed,
                path: Some(f.entry().to_string_lossy().into_owned()),
                resolved_version: Some("2.99.0".into()),
                ..Default::default()
            };
            RuntimeStore::new(&f.d).save(&record).unwrap();
        }
        f
    }
    fn image(root: &Path, version: &str) -> CurrentSelection {
        let (target, platform) = program_layout::host_target().unwrap();
        let paths =
            RuntimeVersionPaths::new(&root.join("runtime/versions"), version, platform).unwrap();
        let package = paths.package_root.join(install::PACKAGE_SUBPATH);
        let script = package.join("bin/ocx.mjs");
        for (file,bytes,mode) in [
            (package.join("package.json"),encode(&serde_json::json!({"name":OFFICIAL_PACKAGE,"version":version,"bin":{"ocx":"bin/ocx.mjs"}})).unwrap(),0o600),
            (script.clone(),b"throw Error('never execute fixture');".to_vec(),0o755),
            (paths.node.clone(),b"#!/bin/sh
exit 97
".to_vec(),0o755),
            (paths.npm.clone(),b"throw Error('never run npm');".to_vec(),0o600),
            (paths.bun.clone(),b"#!/bin/sh
exit 98
".to_vec(),0o755),
            (paths.package_root.join("node_modules/bun/bin/bun.exe"),b"#!/bin/sh
exit 98
".to_vec(),0o755),
        ] {
            fs::create_dir_all(file.parent().unwrap()).unwrap();fs::write(&file,bytes).unwrap();
            fs::set_permissions(&file,fs::Permissions::from_mode(mode)).unwrap();
        }
        let image = ImageManifest {
            schema_version: 1,
            package: OFFICIAL_PACKAGE.into(),
            version: version.into(),
            platform: target.into(),
            official_archive_sha256: hash::sha256_hex(version.as_bytes()),
            bin: "bin/ocx.mjs".into(),
            package_sha256: hash::sha256_file(&package.join("package.json")).unwrap(),
            bin_sha256: hash::sha256_file(&script).unwrap(),
            node_sha256: hash::sha256_file(&paths.node).unwrap(),
            npm_sha256: hash::sha256_file(&paths.npm).unwrap(),
            bun_sha256: hash::sha256_file(&paths.bun).unwrap(),
            bun_launcher_sha256: hash::sha256_file(
                &paths.package_root.join("node_modules/bun/bin/bun.exe"),
            )
            .unwrap(),
        };
        let bytes = encode(&image).unwrap();
        fs::write(paths.version_root.join(IMAGE_MANIFEST), &bytes).unwrap();
        CurrentSelection {
            schema_version: 1,
            version: version.into(),
            platform: target.into(),
            image_sha256: hash::sha256_hex(&bytes),
        }
    }
    fn platform(&self) -> Platform {
        program_layout::host_target().unwrap().1
    }
    fn entry(&self) -> PathBuf {
        self.i
            .join(program_layout::runtime_entry_relative(self.platform()))
    }
    fn current(&self) -> PathBuf {
        self.i.join("runtime/current.json")
    }
    fn record(&self) -> PathBuf {
        RuntimeStore::new(&self.d).path().into()
    }
    fn admit(&self, s: &CurrentSelection) -> SelectedProgram {
        program_selection::read_image(&self.i, self.platform(), &s.platform, s).unwrap()
    }
    fn snapshot(&self) -> Vec<Option<Vec<u8>>> {
        [self.entry(), self.current(), self.record()]
            .map(|p| fs::read(p).ok())
            .to_vec()
    }
    fn verify(&self, expected: Option<&CurrentSelection>) {
        assert_eq!(
            program_selection::read(&self.i, self.platform(), &self.new.platform)
                .unwrap()
                .as_ref()
                .map(|p| &p.selection),
            expected
        );
        assert!(!self.i.join("runtime").join(ACTIVATION_PENDING).exists());
        if let Some(selected) = expected {
            let record: RuntimeSourceRecord = decode(&fs::read(self.record()).unwrap()).unwrap();
            assert!(record_matches(&record, Some(selected), &self.entry()));
        } else {
            assert!(!self.record().exists());
            assert!(!self.entry().exists());
        }
        assert!(self.admit(&self.old).script.is_file());
        assert!(self.admit(&self.new).script.is_file());
    }
}

#[test]
fn changes_one_selection_and_source_mirror_without_mutating_or_deleting_either_image() {
    let f = Fixture::new(true);
    let old = f.admit(&f.old);
    let new = f.admit(&f.new);
    let originals = [fs::read(old.script).unwrap(), fs::read(new.script).unwrap()];
    let mut source: serde_json::Value = decode(&fs::read(f.record()).unwrap()).unwrap();
    source["extraUserFact"] = serde_json::json!({"preserve":true});
    fs::write(f.record(), encode(&source).unwrap()).unwrap();
    let mutation = Mutation::acquire(&f.i).unwrap();
    let selected = mutation
        .activate(&f.d, Some(&f.old), &f.new, || Ok(()))
        .unwrap();
    assert_eq!(selected.selection, f.new);
    f.verify(Some(&f.new));
    let source: serde_json::Value = decode(&fs::read(f.record()).unwrap()).unwrap();
    assert_eq!(
        source["extraUserFact"],
        serde_json::json!({"preserve":true})
    );
    assert_eq!(fs::read(f.admit(&f.old).script).unwrap(), originals[0]);
    assert_eq!(fs::read(f.admit(&f.new).script).unwrap(), originals[1]);
    assert_eq!(
        fs::read_dir(f.d.join("manager-state/runtime-activations"))
            .unwrap()
            .count(),
        1
    );
}

#[test]
fn every_interruption_stage_can_explicitly_finish_or_restore_exact_before_images() {
    for initial in [false, true] {
        for phase in [
            Checkpoint::Journal,
            Checkpoint::Entry,
            Checkpoint::Current,
            Checkpoint::Record,
        ] {
            for decision in [Decision::Complete, Decision::Rollback] {
                let f = Fixture::new(initial);
                let before = f.snapshot();
                let mutation = Mutation::acquire(&f.i).unwrap();
                assert!(mutation
                    .activate_inner(
                        &f.d,
                        f.initial.then_some(&f.old),
                        &f.new,
                        || Ok(()),
                        |now| if now == phase {
                            Err(fail("injected interruption"))
                        } else {
                            Ok(())
                        }
                    )
                    .is_err());
                assert!(program_selection::read(&f.i, f.platform(), &f.new.platform).is_err());
                let pending = mutation.pending_summary(&f.d).unwrap();
                assert_eq!(pending.to, "3.0.0");
                assert_eq!(pending.from.as_deref(), initial.then_some("2.99.0"));
                assert!(mutation
                    .recover(&f.d, "wrong-attempt", decision, || Ok(()))
                    .is_err());
                mutation
                    .recover(&f.d, &pending.attempt_id, decision, || Ok(()))
                    .unwrap();
                let expected = if decision == Decision::Complete {
                    Some(&f.new)
                } else {
                    initial.then_some(&f.old)
                };
                f.verify(expected);
                if decision == Decision::Rollback {
                    assert_eq!(f.snapshot(), before);
                }
            }
        }
    }
}

#[test]
fn wrong_baseline_unknown_source_and_admission_refusal_do_not_begin_a_transaction() {
    let f = Fixture::new(true);
    let before = f.snapshot();
    let mutation = Mutation::acquire(&f.i).unwrap();
    assert!(mutation.activate(&f.d, None, &f.new, || Ok(())).is_err());
    assert!(mutation
        .activate(&f.d, Some(&f.old), &f.new, || Err(fail("owner unknown")))
        .is_err());
    assert_eq!(f.snapshot(), before);
    assert!(!mutation.pending().exists());
    fs::write(f.record(), b"unrecognized source bytes").unwrap();
    let before = f.snapshot();
    assert!(mutation
        .activate(&f.d, Some(&f.old), &f.new, || Ok(()))
        .is_err());
    assert_eq!(f.snapshot(), before);
    assert!(!mutation.pending().exists());
}

#[test]
fn unknown_value_in_any_recovery_target_is_preserved_before_recovery_writes() {
    for which in 0..3 {
        let f = Fixture::new(true);
        let mutation = Mutation::acquire(&f.i).unwrap();
        assert!(mutation
            .activate_inner(
                &f.d,
                Some(&f.old),
                &f.new,
                || Ok(()),
                |phase| if phase == Checkpoint::Entry {
                    Err(fail("injected"))
                } else {
                    Ok(())
                }
            )
            .is_err());
        let id = mutation.pending_summary(&f.d).unwrap().attempt_id;
        fs::write(
            [f.entry(), f.current(), f.record()][which].clone(),
            b"foreign user data",
        )
        .unwrap();
        let before = f.snapshot();
        let pending = fs::read(mutation.pending()).unwrap();
        assert!(mutation
            .recover(&f.d, &id, Decision::Rollback, || Ok(()))
            .is_err());
        assert_eq!(f.snapshot(), before);
        assert_eq!(fs::read(mutation.pending()).unwrap(), pending);
    }
}

#[test]
fn recovery_refuses_other_data_roots_tampered_journal_or_changed_candidate_image() {
    let f = Fixture::new(true);
    let mutation = Mutation::acquire(&f.i).unwrap();
    assert!(mutation
        .activate_inner(
            &f.d,
            Some(&f.old),
            &f.new,
            || Ok(()),
            |phase| if phase == Checkpoint::Journal {
                Err(fail("injected"))
            } else {
                Ok(())
            }
        )
        .is_err());
    let id = mutation.pending_summary(&f.d).unwrap().attempt_id;
    let before = f.snapshot();
    let other = f.d.with_file_name("other-desktop");
    fs::create_dir_all(other.join("manager-state")).unwrap();
    assert!(mutation
        .recover(&other, &id, Decision::Complete, || Ok(()))
        .is_err());
    assert_eq!(
        fs::read_dir(other.join("manager-state")).unwrap().count(),
        0
    );
    let pending = fs::read(mutation.pending()).unwrap();
    let mut changed: Envelope = decode(&pending).unwrap();
    changed.journal.after.version = "4.0.0".into();
    fs::write(mutation.pending(), encode(&changed).unwrap()).unwrap();
    assert!(mutation
        .recover(&f.d, &id, Decision::Complete, || Ok(()))
        .is_err());
    fs::write(mutation.pending(), &pending).unwrap();
    fs::write(f.admit(&f.new).script, b"unknown replacement program").unwrap();
    assert!(mutation
        .recover(&f.d, &id, Decision::Complete, || Ok(()))
        .is_err());
    assert_eq!(f.snapshot(), before);
    assert_eq!(fs::read(mutation.pending()).unwrap(), pending);
}
