use super::*;
use crate::modules::runtime::{
    program_activation::tests::Fixture,
    program_selection::{ImageManifest, IMAGE_MANIFEST},
};
struct Test {
    program: Fixture,
    h: PathBuf,
    config: Vec<u8>,
    marker: Vec<u8>,
    ledger: Vec<(String, Vec<u8>)>,
}
impl Test {
    fn new() -> Self {
        let program = Fixture::new(true);
        let i = &program.i;
        let d = &program.d;
        let h = d.with_file_name("opencodex-home");
        fs::create_dir(&h).unwrap();
        let config =
            br#"{ "port":41234,"providers":{"fixture":{"apiKey":"synthetic-only"}},"keep":true }"#
                .to_vec();
        fs::write(h.join("config.json"), &config).unwrap();
        for version in ["2.99.0", "3.0.0"] {
            let (s, m) = selection(i, version);
            program_catalog::register(i, &s, &m.official_archive_sha256).unwrap();
        }
        let ledger = fs::read_dir(i.join("runtime/image-admissions"))
            .unwrap()
            .map(|r| {
                let p = r.unwrap().path();
                (
                    p.file_name().unwrap().to_str().unwrap().into(),
                    fs::read(p).unwrap(),
                )
            })
            .collect();
        fs::write(i.join("runtime/bin/other-tool"), b"not application owned").unwrap();
        let p = program_removal::prepare_registered(i, d, &h).unwrap();
        p.execute(
            &p.preview().unwrap().confirmation_digest,
            || Ok(()),
            || Ok(()),
        )
        .unwrap();
        let marker = fs::read(i.join("runtime").join(program_removal::REMOVED)).unwrap();
        Self {
            program,
            h,
            config,
            marker,
            ledger,
        }
    }
    fn candidate(&self) -> (PathBuf, CurrentSelection, String) {
        let stage = tempfile::Builder::new()
            .prefix(".test-verified-stage-")
            .tempdir_in(&self.program.i)
            .unwrap()
            .keep();
        let s = Fixture::image(&stage, "2.99.0");
        let (s2, m) = selection(&stage, &s.version);
        assert_eq!(s, s2);
        (
            stage.join("runtime/versions").join(&s.version),
            s,
            m.official_archive_sha256,
        )
    }
    fn run(&self, interrupt: Option<&str>) -> AppResult<Outcome> {
        let (staged, selection, source) = self.candidate();
        let stage = staged.ancestors().nth(3).unwrap().to_path_buf();
        let lock = Mutation::acquire(&self.program.i).unwrap();
        let result = install(
            Candidate {
                program: &self.program.i,
                desktop: &self.program.d,
                home: &self.h,
                staged_image: &staged,
                selection: &selection,
                descriptor_pin: &"a".repeat(64),
                source_sha256: &source,
            },
            &lock,
            || Ok(()),
            |at| {
                if interrupt == Some(at) {
                    Err(fail("injected reinstall interruption"))
                } else {
                    Ok(())
                }
            },
        );
        fs::remove_dir_all(stage).unwrap();
        result
    }
    fn preserved(&self) {
        assert_eq!(fs::read(self.h.join("config.json")).unwrap(), self.config);
        assert_eq!(
            fs::read(self.program.i.join("runtime/bin/other-tool")).unwrap(),
            b"not application owned"
        );
    }
    fn finished(&self, id: &str) {
        let (target, platform) = program_layout::host_target().unwrap();
        let selected = program_selection::read(&self.program.i, platform, target)
            .unwrap()
            .unwrap();
        assert_eq!(selected.selection.version, "2.99.0");
        let all = program_catalog::inspect(&self.program.i).unwrap();
        assert_eq!(all.selections, vec![selected.selection]);
        let history = self.program.i.join("runtime/reinstall-history").join(id);
        assert_eq!(fs::read(history.join("removed.json")).unwrap(), self.marker);
        for (name, bytes) in &self.ledger {
            assert_eq!(
                fs::read(history.join("image-admissions").join(name)).unwrap(),
                *bytes
            )
        }
        assert!(self
            .program
            .d
            .join("manager-state/runtime-reinstalls")
            .join(format!("{id}.json"))
            .is_file());
        assert!(!self.program.i.join("runtime").join(PENDING).exists());
        assert!(!self
            .program
            .i
            .join("runtime")
            .join(program_removal::REMOVED)
            .exists());
        self.preserved();
        let history = RuntimeStore::new(&self.program.d).load().history;
        assert_eq!(history.last().unwrap().action, "install");
    }
}
fn selection(root: &Path, version: &str) -> (CurrentSelection, ImageManifest) {
    let b = fs::read(
        root.join("runtime/versions")
            .join(version)
            .join(IMAGE_MANIFEST),
    )
    .unwrap();
    let m: ImageManifest = serde_json::from_slice(&b).unwrap();
    (
        CurrentSelection {
            schema_version: 1,
            version: version.into(),
            platform: m.platform.clone(),
            image_sha256: hash::sha256_hex(&b),
        },
        m,
    )
}
#[test]
fn explicit_reinstall_archives_old_admissions_and_preserves_config_and_unrelated_tools() {
    let f = Test::new();
    let out = f.run(None).unwrap();
    assert!(out.configuration_preserved);
    f.finished(&out.operation_id);
    assert!(
        f.run(None).is_err(),
        "already installed is not another removal authority"
    );
    f.preserved();
}
#[test]
fn every_reinstall_checkpoint_continues_the_same_operation_and_ordinary_supply_never_resumes_it() {
    for phase in [
        "prepared",
        "marker_archived",
        "catalog_archived",
        "image",
        "admission",
        "activation_prepared",
        "activation_journal",
        "entry",
        "current",
        "record",
        "activated",
        "receipt",
    ] {
        let f = Test::new();
        assert!(f.run(Some(phase)).is_err(), "{phase}");
        let (j, original) = load(&f.program.i, &f.program.d, &f.h).unwrap().unwrap();
        let (target, platform) = program_layout::host_target().unwrap();
        assert!(program_selection::read(&f.program.i, platform, target).is_err());
        assert!(bundled::versioned::provision(
            &f.program.i,
            &f.program.d,
            Path::new("/absent-resource"),
            b"bad",
            "bad"
        )
        .is_err());
        assert_eq!(read(&f.program.i, &j.pending()).unwrap().unwrap(), original);
        f.preserved();
        let outcome = f.run(None).unwrap_or_else(|e| panic!("{phase}: {e:?}"));
        assert_eq!(outcome.operation_id, j.operation_id);
        f.finished(&outcome.operation_id);
    }
}
#[test]
fn continuation_refuses_changed_source_image_evidence_or_context_and_keeps_unknown_objects() {
    for kind in [
        "source",
        "image",
        "marker",
        "catalog",
        "activation",
        "other-home",
        "unknown-version",
    ] {
        let f = Test::new();
        let phase = if kind == "source" || kind == "activation" {
            "activation_prepared"
        } else {
            "image"
        };
        assert!(f.run(Some(phase)).is_err());
        let (j, original) = load(&f.program.i, &f.program.d, &f.h).unwrap().unwrap();
        let path = match kind {
            "source" => RuntimeStore::new(&f.program.d).path().to_path_buf(),
            "image" => f.program.i.join("runtime/versions/2.99.0/user-added"),
            "marker" => j.history().join("removed.json"),
            "catalog" => j.history().join("image-admissions").join(&f.ledger[0].0),
            "activation" => j.history().join("activation.json"),
            _ => f.program.i.join("runtime/versions/unregistered"),
        };
        if kind == "unknown-version" {
            fs::create_dir(&path).unwrap()
        } else if kind != "other-home" {
            fs::write(&path, b"external bytes must remain").unwrap()
        }
        if kind == "other-home" {
            let other = f.h.with_file_name("other-home");
            fs::create_dir(&other).unwrap();
            assert!(preflight(&f.program.i, &f.program.d, &other).is_err());
        } else {
            assert!(f.run(None).is_err(), "{kind}");
            if kind != "unknown-version" {
                assert_eq!(fs::read(path).unwrap(), b"external bytes must remain")
            }
        }
        assert_eq!(read(&f.program.i, &j.pending()).unwrap().unwrap(), original);
        f.preserved();
    }
}
#[test]
fn unconfirmed_occupancy_or_failed_configuration_backup_do_not_begin_reinstall() {
    for blocked in ["occupied", "backup"] {
        let f = Test::new();
        let (staged, selection, source) = f.candidate();
        if blocked == "backup" {
            let dir = f.program.d.join("backups");
            fs::remove_dir_all(&dir).unwrap();
            fs::write(dir, b"blocked").unwrap()
        }
        let lock = Mutation::acquire(&f.program.i).unwrap();
        assert!(install(
            Candidate {
                program: &f.program.i,
                desktop: &f.program.d,
                home: &f.h,
                staged_image: &staged,
                selection: &selection,
                descriptor_pin: &"a".repeat(64),
                source_sha256: &source
            },
            &lock,
            || if blocked == "occupied" {
                Err(fail("unknown native ownership"))
            } else {
                Ok(())
            },
            |_| Ok(())
        )
        .is_err());
        assert!(!f.program.i.join("runtime").join(PENDING).exists());
        assert!(program_removal::intentionally_removed(&f.program.i, &f.program.d).unwrap());
        f.preserved();
    }
}

#[test]
#[ignore = "requires the explicitly pinned official 2.99 complete bundle; no service launch"]
fn real_official_bundle_remove_reinstall_and_read_actual_version_preserves_configuration() {
    let archive =
        PathBuf::from(std::env::var_os("OCXD_TEST_BUNDLE_ARCHIVE").expect("explicit bundle"));
    let descriptor = fs::read(PathBuf::from(
        std::env::var_os("OCXD_TEST_BUNDLE_DESCRIPTOR").expect("explicit descriptor"),
    ))
    .unwrap();
    let pin = std::env::var("OCXD_TEST_BUNDLE_PIN").expect("explicit pin");
    assert_eq!(
        pin,
        "e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3"
    );
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let i = root.join("program I");
    let d = root.join("user/desktop");
    let h = root.join("user/opencodex/home");
    for dir in [&i, &d, &h] {
        fs::create_dir_all(dir).unwrap()
    }
    let config =
        br#"{ "port":41234,"providers":{"fixture":{"apiKey":"synthetic-only"}},"keep":true }"#;
    fs::write(h.join("config.json"), config).unwrap();
    let started = std::time::Instant::now();
    bundled::versioned::provision(&i, &d, &archive, &descriptor, &pin).unwrap();
    let original_catalog = program_catalog::revision(&i).unwrap();
    let p = program_removal::prepare_registered(&i, &d, &h).unwrap();
    p.execute(
        &p.preview().unwrap().confirmation_digest,
        || Ok(()),
        || Ok(()),
    )
    .unwrap();
    assert!(program_removal::intentionally_removed(&i, &d).unwrap());
    let marker = fs::read(i.join("runtime").join(program_removal::REMOVED)).unwrap();
    let r = review(&i, &d, &h, &pin, "2.99.0").unwrap();
    assert!(!r.resume);
    let installed = bundled::versioned::reinstall(
        &i,
        &d,
        bundled::versioned::ReinstallRequest {
            home: &h,
            confirmation_digest: &r.confirmation_digest,
        },
        &archive,
        &descriptor,
        &pin,
        || Ok(()),
    )
    .unwrap();
    assert!(matches!(installed,bundled::ProvisionResult::Installed{version} if version=="2.99.0"));
    let selected =
        program_selection::read(&i, program_layout::host_target().unwrap().1, "darwin-arm64")
            .unwrap()
            .unwrap();
    program_image::verify(&selected).unwrap();
    assert_eq!(
        program_catalog::inspect(&i).unwrap().selections,
        vec![selected.selection.clone()]
    );
    let version = std::process::Command::new(&selected.paths.node)
        .arg(&selected.script)
        .arg("--version")
        .env("HOME", root.join("user"))
        .env("USERPROFILE", root.join("user"))
        .env("OPENCODEX_HOME", &h)
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8_lossy(&version.stdout).trim(),
        "opencodex 2.99.0"
    );
    assert_eq!(fs::read(h.join("config.json")).unwrap(), config);
    let history = fs::read_dir(i.join("runtime/reinstall-history"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert_eq!(fs::read(history.join("removed.json")).unwrap(), marker);
    assert_eq!(
        program_catalog::archived_revision(&i, &history.join("image-admissions")).unwrap(),
        original_catalog
    );
    assert!(!i.join("runtime/bin/ocx").exists());
    assert!(!i.join("runtime").join(PENDING).exists());
    println!(
        "REAL_OFFICIAL_REINSTALL {}",
        serde_json::json!({"version":"2.99.0","elapsedMs":started.elapsed().as_millis(),"configurationPreserved":true,"oldEvidencePreserved":true,"actualVersionCommand":true,"terminalAutomaticallyEnabled":false,"serviceProof":"injected isolated fixture","serviceStarted":false})
    );
}

#[test]
fn preview_is_readonly_and_original_context_pin_and_prepared_source_cannot_be_replaced() {
    let f = Test::new();
    let before = fs::read_dir(f.program.d.join("backups")).unwrap().count();
    let preview = review(&f.program.i, &f.program.d, &f.h, &"a".repeat(64), "2.99.0").unwrap();
    assert!(!preview.resume);
    assert_eq!(preview.version, "2.99.0");
    assert_eq!(
        before,
        fs::read_dir(f.program.d.join("backups")).unwrap().count()
    );
    assert!(!f.program.i.join("runtime").join(PENDING).exists());
    assert!(f.run(Some("prepared")).is_err());
    let r = review(&f.program.i, &f.program.d, &f.h, &"a".repeat(64), "2.99.0").unwrap();
    assert!(r.resume);
    assert_ne!(r.confirmation_digest, preview.confirmation_digest);
    assert!(review(&f.program.i, &f.program.d, &f.h, &"b".repeat(64), "2.99.0").is_err());
    assert!(review(&f.program.i, &f.program.d, &f.h, &"a".repeat(64), "3.0.0").is_err());
    assert!(
        crate::modules::runtime::program_activation::inspect_pending(&f.program.i, &f.program.d)
            .unwrap()
            .is_none()
    );
    f.preserved();
}
