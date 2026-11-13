use super::*;
use crate::modules::runtime::program_activation::tests::Fixture;
struct Test {
    f: Fixture,
    h: PathBuf,
    s: CurrentSelection,
    source: String,
    record: Vec<u8>,
    config: Vec<u8>,
    staged: PathBuf,
    entry_after: Vec<u8>,
}
impl Test {
    fn new(damage: &str) -> Self {
        let f = Fixture::new(true);
        let h = f.d.with_file_name("home");
        fs::create_dir(&h).unwrap();
        let config =
            br#"{"port":41234,"providers":{"sample":{"apiKey":"synthetic-only"}}}"#.to_vec();
        fs::write(h.join("config.json"), &config).unwrap();
        let (target, platform) = program_layout::host_target().unwrap();
        let selected = program_selection::read(&f.i, platform, target)
            .unwrap()
            .unwrap();
        let s = selected.selection.clone();
        let source = selected.image.official_archive_sha256.clone();
        for version in ["2.99.0", "3.0.0"] {
            let b = fs::read(
                f.i.join("runtime/versions")
                    .join(version)
                    .join(IMAGE_MANIFEST),
            )
            .unwrap();
            let m: program_selection::ImageManifest = decode(&b).unwrap();
            program_catalog::register(
                &f.i,
                &CurrentSelection {
                    schema_version: 1,
                    version: version.into(),
                    platform: target.into(),
                    image_sha256: hash::sha256_hex(&b),
                },
                &m.official_archive_sha256,
            )
            .unwrap();
        }
        let stage = tempfile::Builder::new()
            .prefix(".repair-test-stage-")
            .tempdir_in(&f.i)
            .unwrap()
            .keep();
        let copy = Fixture::image(&stage, "2.99.0");
        assert_eq!(copy, s);
        let staged = stage.join("runtime/versions/2.99.0");
        let entry_after =
            install::entry_script(Some(&selected.paths.node), &selected.script).into_bytes();
        match damage {
            "file" => fs::write(&selected.paths.node, b"damaged node, preserve original").unwrap(),
            "missing-file" => fs::remove_file(&selected.paths.npm).unwrap(),
            "manifest" => fs::write(
                selected.paths.version_root.join(IMAGE_MANIFEST),
                b"damaged manifest, preserve original",
            )
            .unwrap(),
            "extra" => fs::write(
                selected.paths.version_root.join("user-added"),
                b"user original retained",
            )
            .unwrap(),
            "entry" => fs::write(entry(&f.i).unwrap(), b"damaged entry original").unwrap(),
            "missing-image" => fs::remove_dir_all(selected.paths.version_root).unwrap(),
            "healthy" => {}
            _ => panic!(),
        }
        let record = fs::read(RuntimeStore::new(&f.d).path()).unwrap();
        Self {
            f,
            h,
            s,
            source,
            record,
            config,
            staged,
            entry_after,
        }
    }
    fn preview(&self) -> Review {
        review(
            &self.f.i,
            &self.f.d,
            &self.h,
            &"a".repeat(64),
            "2.99.0",
            &self.source,
        )
        .unwrap()
    }
    fn run(&self, confirmation: &str, phase: Option<&str>) -> AppResult<Outcome> {
        let lock = Mutation::acquire(&self.f.i).unwrap();
        execute(
            Candidate {
                program: &self.f.i,
                desktop: &self.f.d,
                home: &self.h,
                image: &self.staged,
                selection: &self.s,
                source: &self.source,
                pin: &"a".repeat(64),
                confirmation,
                entry_after: &self.entry_after,
            },
            &lock,
            || Ok(()),
            |at| {
                if phase == Some(at) {
                    Err(fail("injected repair interruption"))
                } else {
                    Ok(())
                }
            },
        )
    }
    fn kept(&self) {
        assert_eq!(fs::read(self.h.join("config.json")).unwrap(), self.config);
        assert_eq!(
            fs::read(RuntimeStore::new(&self.f.d).path()).unwrap(),
            self.record
        );
    }
    fn done(&self, id: &str) {
        self.kept();
        let selected = program_selection::read(
            &self.f.i,
            program_layout::host_target().unwrap().1,
            &self.s.platform,
        )
        .unwrap()
        .unwrap();
        assert_eq!(selected.selection, self.s);
        program_image::verify(&selected).unwrap();
        assert!(!self.f.i.join("runtime").join(PENDING).exists());
        assert!(self
            .f
            .d
            .join("manager-state/runtime-repairs")
            .join(format!("{id}.json"))
            .is_file());
    }
}
#[test]
fn same_admitted_image_repairs_damaged_missing_or_extra_content_and_keeps_originals() {
    for damage in [
        "file",
        "missing-file",
        "manifest",
        "extra",
        "entry",
        "missing-image",
    ] {
        let t = Test::new(damage);
        let r = t.preview();
        let original = snapshot(&t.f.i, &t.f.i.join("runtime/versions/2.99.0")).unwrap();
        let result = t
            .run(&r.confirmation_digest, None)
            .unwrap_or_else(|e| panic!("{damage}: {e:?}"));
        t.done(&result.operation_id);
        if original.present {
            assert_eq!(
                snapshot(
                    &t.f.i,
                    &t.f.i
                        .join("runtime/repair-history")
                        .join(result.operation_id)
                        .join("image")
                )
                .unwrap(),
                original
            );
        }
    }
}
#[test]
fn every_repair_interruption_preserves_original_operation_and_can_continue_without_new_baseline() {
    for phase in [
        "prepared",
        "entry_archived",
        "image_archived",
        "image_published",
        "entry",
        "receipt",
    ] {
        let t = Test::new("file");
        let r = t.preview();
        assert!(t.run(&r.confirmation_digest, Some(phase)).is_err());
        let (j, b) = load(&t.f.i, &t.f.d, &t.h).unwrap().unwrap();
        assert!(program_selection::read(
            &t.f.i,
            program_layout::host_target().unwrap().1,
            &t.s.platform
        )
        .is_err());
        let r = t.preview();
        assert!(r.resume);
        assert_eq!(read(&t.f.i, &j.pending(), LIMIT).unwrap().unwrap(), b);
        let out = t
            .run(&r.confirmation_digest, None)
            .unwrap_or_else(|e| panic!("{phase}: {e:?}"));
        assert_eq!(out.operation_id, j.operation_id);
        t.done(&out.operation_id);
    }
}
#[test]
fn unregistered_healthy_or_different_source_and_external_drift_are_not_repair_permission() {
    let healthy = Test::new("healthy");
    assert!(review(
        &healthy.f.i,
        &healthy.f.d,
        &healthy.h,
        &"a".repeat(64),
        "2.99.0",
        &healthy.source
    )
    .is_err());
    healthy.kept();
    for kind in [
        "source",
        "version",
        "ledger",
        "current",
        "confirmation",
        "drift",
        "symlink",
    ] {
        let t = Test::new("file");
        let r = t.preview();
        if kind == "source" {
            assert!(review(
                &t.f.i,
                &t.f.d,
                &t.h,
                &"a".repeat(64),
                "2.99.0",
                &"b".repeat(64)
            )
            .is_err());
            continue;
        }
        if kind == "version" {
            assert!(review(&t.f.i, &t.f.d, &t.h, &"a".repeat(64), "4.0.0", &t.source).is_err());
            continue;
        }
        if kind == "ledger" {
            let path = fs::read_dir(t.f.i.join("runtime/image-admissions"))
                .unwrap()
                .next()
                .unwrap()
                .unwrap()
                .path();
            fs::write(path, b"foreign ledger").unwrap();
        }
        if kind == "current" {
            fs::write(t.f.i.join("runtime/current.json"), b"foreign current").unwrap();
        }
        if kind == "drift" {
            fs::write(
                t.f.i.join("runtime/versions/2.99.0/new-external"),
                b"keep new external",
            )
            .unwrap();
        }
        if kind == "symlink" {
            std::os::unix::fs::symlink("/outside", t.f.i.join("runtime/versions/2.99.0/link"))
                .unwrap();
        }
        assert!(
            t.run(
                if kind == "confirmation" {
                    "wrong"
                } else {
                    &r.confirmation_digest
                },
                None
            )
            .is_err(),
            "{kind}"
        );
        assert!(!t.f.i.join("runtime").join(PENDING).exists());
        t.kept();
    }
}
#[test]
fn continuation_refuses_changes_to_original_archive_source_or_new_image() {
    for kind in ["archive", "record", "image", "entry"] {
        let t = Test::new("file");
        let r = t.preview();
        assert!(t
            .run(&r.confirmation_digest, Some("image_published"))
            .is_err());
        let (j, b) = load(&t.f.i, &t.f.d, &t.h).unwrap().unwrap();
        let path = match kind {
            "archive" => j.history().join("image/external"),
            "record" => RuntimeStore::new(&t.f.d).path().into(),
            "image" => t.f.i.join("runtime/versions/2.99.0/external"),
            _ => entry(&t.f.i).unwrap(),
        };
        fs::write(&path, b"external original must remain").unwrap();
        assert!(t.run(&r.confirmation_digest, None).is_err(), "{kind}");
        assert_eq!(fs::read(path).unwrap(), b"external original must remain");
        assert_eq!(read(&t.f.i, &j.pending(), LIMIT).unwrap().unwrap(), b);
        assert_eq!(fs::read(t.h.join("config.json")).unwrap(), t.config);
    }
}

#[test]
#[ignore = "requires explicitly pinned official2.99 complete bundle; no service started"]
fn real_official_same_version_repair_preserves_damaged_original_and_configuration() {
    let archive =
        PathBuf::from(std::env::var_os("OCXD_TEST_BUNDLE_ARCHIVE").expect("explicit archive"));
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
    let i = root.join("program");
    let d = root.join("user/desktop");
    let h = root.join("user/home");
    for p in [&i, &d, &h] {
        fs::create_dir_all(p).unwrap();
    }
    let config =
        br#"{ "port":41234,"providers":{"sample":{"apiKey":"synthetic-only"}},"keep":true }"#;
    fs::write(h.join("config.json"), config).unwrap();
    let start = std::time::Instant::now();
    bundled::versioned::provision(&i, &d, &archive, &descriptor, &pin).unwrap();
    let selected =
        program_selection::read(&i, program_layout::host_target().unwrap().1, "darwin-arm64")
            .unwrap()
            .unwrap();
    let node = selected.paths.node.clone();
    let original_node = hash::sha256_file(&node).unwrap();
    fs::write(&node, b"damaged-original-node-preserved").unwrap();
    let before = fs::read(RuntimeStore::new(&d).path()).unwrap();
    let r = review(
        &i,
        &d,
        &h,
        &pin,
        "2.99.0",
        &selected.image.official_archive_sha256,
    )
    .unwrap();
    bundled::versioned::repair(
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
    let current =
        program_selection::read(&i, program_layout::host_target().unwrap().1, "darwin-arm64")
            .unwrap()
            .unwrap();
    program_image::verify(&current).unwrap();
    assert_eq!(current.selection, selected.selection);
    assert_eq!(hash::sha256_file(&node).unwrap(), original_node);
    let history = fs::read_dir(i.join("runtime/repair-history"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let rel = node.strip_prefix(&selected.paths.version_root).unwrap();
    assert_eq!(
        fs::read(history.join("image").join(rel)).unwrap(),
        b"damaged-original-node-preserved"
    );
    assert_eq!(fs::read(h.join("config.json")).unwrap(), config);
    assert_eq!(fs::read(RuntimeStore::new(&d).path()).unwrap(), before);
    let output = std::process::Command::new(&current.paths.node)
        .arg(&current.script)
        .arg("--version")
        .env("HOME", root.join("user"))
        .env("OPENCODEX_HOME", &h)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout).trim(),
        "opencodex 2.99.0"
    );
    println!(
        "REAL_OFFICIAL_REPAIR {}",
        serde_json::json!({"version":"2.99.0","elapsedMs":start.elapsed().as_millis(),"configurationPreserved":true,"sourceRecordPreserved":true,"originalDamagedNodeRetained":true,"actualVersionCommand":true,"serviceStarted":false,"serviceUnusedProof":"injected isolated fixture"})
    );
}

#[test]
fn unknown_occupancy_backup_failure_or_configuration_drift_refuse_without_erasing_originals() {
    for kind in ["unknown", "backup", "config-drift"] {
        let t = Test::new("file");
        let r = t.preview();
        let original = snapshot(&t.f.i, &t.f.i.join("runtime/versions/2.99.0")).unwrap();
        if kind == "backup" {
            fs::write(t.f.d.join("backups"), b"blocked").unwrap();
        }
        let lock = Mutation::acquire(&t.f.i).unwrap();
        let result = execute(
            Candidate {
                program: &t.f.i,
                desktop: &t.f.d,
                home: &t.h,
                image: &t.staged,
                selection: &t.s,
                source: &t.source,
                pin: &"a".repeat(64),
                confirmation: &r.confirmation_digest,
                entry_after: &t.entry_after,
            },
            &lock,
            || {
                if kind == "unknown" {
                    Err(fail("native occupancy unknown"))
                } else {
                    Ok(())
                }
            },
            |phase| {
                if kind == "config-drift" && phase == "prepared" {
                    fs::write(t.h.join("config.json"), b"external config must remain").unwrap();
                }
                Ok(())
            },
        );
        assert!(result.is_err(), "{kind}");
        assert_eq!(
            snapshot(&t.f.i, &t.f.i.join("runtime/versions/2.99.0")).unwrap(),
            original
        );
        if kind == "config-drift" {
            assert_eq!(
                fs::read(t.h.join("config.json")).unwrap(),
                b"external config must remain"
            );
        } else {
            t.kept();
            assert!(!t.f.i.join("runtime").join(PENDING).exists());
        }
    }
}
