use super::*;
use crate::modules::model_config::config_file::v2::native_import::backups::UpgradeRequest;
use crate::modules::runtime::program_selection::tests::Fixture as SelectedFixture;
use serde_json::json;
use std::os::unix::fs::PermissionsExt;

fn write(path: &Path, bytes: &[u8], mode: u32) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, bytes).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
}
struct Npm {
    archive: PathBuf,
    escape: bool,
    tamper_official: bool,
}
impl NpmRunner for Npm {
    fn pack(
        &self,
        _: &NpmPackRequest,
        _: &dyn InstallProgressSink,
        _: &CancelFlag,
    ) -> Result<PathBuf, InstallError> {
        Ok(self.archive.clone())
    }
    fn install_package(
        &self,
        request: &NpmInstallRequest,
        _: &dyn InstallProgressSink,
        _: &CancelFlag,
    ) -> Result<NpmOutcome, InstallError> {
        assert!(!request.scripts_enabled);
        archive::extract(&self.archive, None, &request.prefix.join(PACKAGE_SUBPATH))?;
        write(
            &request.prefix.join("node_modules/bun/package.json"),
            br#"{"name":"bun","version":"1.4.4"}"#,
            0o644,
        );
        write(
            &request
                .prefix
                .join("node_modules/@oven/bun-darwin-aarch64/package.json"),
            br#"{"name":"@oven/bun-darwin-aarch64","version":"1.4.4"}"#,
            0o644,
        );
        write(
            &request
                .prefix
                .join("node_modules/@oven/bun-darwin-aarch64/bin/bun"),
            b"#!/bin/sh
exit 98
",
            0o755,
        );
        if self.tamper_official {
            write(
                &request.prefix.join(PACKAGE_SUBPATH).join("bin/ocx.mjs"),
                b"modified official code",
                0o755,
            );
        }
        if self.escape {
            std::os::unix::fs::symlink(
                "../../../../outside",
                request.prefix.join("node_modules/escape"),
            )
            .unwrap();
        }
        Ok(NpmOutcome { exit_code: 0 })
    }
}
struct Fixture {
    source: SelectedFixture,
    _data: tempfile::TempDir,
    data: PathBuf,
    home: PathBuf,
    archive: PathBuf,
    current: SelectedProgram,
    config: Vec<u8>,
}
impl Fixture {
    fn new() -> Self {
        let source = SelectedFixture::new();
        let data = tempfile::tempdir().unwrap();
        let data_root = data.path().canonicalize().unwrap().join("desktop");
        let home = data.path().canonicalize().unwrap().join("home");
        std::fs::create_dir_all(data_root.join("manager-state")).unwrap();
        std::fs::create_dir(&home).unwrap();
        let config=br#"{"port":41234,"providers":{"P":{"apiKey":"fixture-key","models":["model"]}},"preserve":true}"#.to_vec();
        std::fs::write(home.join("config.json"), &config).unwrap();
        let current = program_selection::read(
            &source.root,
            program_layout::host_target().unwrap().1,
            "darwin-arm64",
        )
        .unwrap()
        .unwrap();
        let archive = data.path().join("opencodex-3.0.0.tgz");
        let gzip = flate2::write::GzEncoder::new(
            std::fs::File::create(&archive).unwrap(),
            flate2::Compression::fast(),
        );
        let mut tar = tar::Builder::new(gzip);
        for (path, bytes, mode) in [
            (
                "package/package.json",
                serde_json::to_vec(
                    &json!({"name":OFFICIAL_PACKAGE,"version":"3.0.0","bin":{"ocx":"bin/ocx.mjs"}}),
                )
                .unwrap(),
                0o644,
            ),
            (
                "package/bin/ocx.mjs",
                b"throw Error('synthetic: never executed');".to_vec(),
                0o755,
            ),
        ] {
            let mut h = tar::Header::new_gnu();
            h.set_size(bytes.len() as u64);
            h.set_mode(mode);
            h.set_cksum();
            tar.append_data(&mut h, path, bytes.as_slice()).unwrap();
        }
        tar.into_inner().unwrap().finish().unwrap();
        Self {
            source,
            _data: data,
            data: data_root,
            home,
            archive,
            current,
            config,
        }
    }
    fn request(&self, offline: bool) -> InstallRequest {
        InstallRequest {
            prefix: self.current.paths.package_root.clone(),
            source: if offline {
                InstallSource::Offline {
                    archive: self.archive.clone(),
                }
            } else {
                InstallSource::Registry {
                    version: "3.0.0".into(),
                    proxy: None,
                }
            },
            allow_scripts: false,
            node_path: Some(self.current.paths.node.clone()),
            npm_path: Some(self.current.paths.npm.clone()),
        }
    }
    fn old(&self) {
        assert_eq!(
            program_selection::read(
                &self.source.root,
                program_layout::host_target().unwrap().1,
                "darwin-arm64"
            )
            .unwrap()
            .unwrap()
            .selection,
            self.current.selection
        );
        assert_eq!(
            std::fs::read(self.home.join("config.json")).unwrap(),
            self.config
        );
    }
}

#[test]
fn registry_and_offline_update_preserve_old_image_and_config_and_atomically_activate_candidate() {
    for offline in [false, true] {
        let f = Fixture::new();
        let npm = Npm {
            archive: f.archive.clone(),
            escape: false,
            tamper_official: false,
        };
        let sink = RecordingProgressSink::default();
        let probe = FixedVersionProbe {
            version: Some("3.0.0".into()),
        };
        let mut installer = RuntimeInstaller::new(
            &f.data,
            &npm,
            None,
            &sink,
            &probe,
            Some(f.current.paths.node.clone()),
        );
        installer.config_backup = Some(UpgradeRequest {
            home: f.home.clone(),
            program_source: None,
        });
        let outcome = installer
            .install_versioned(&f.source.root, &f.current, &f.request(offline), || Ok(()))
            .unwrap();
        assert!(outcome.changed);
        assert_eq!(outcome.installation.version, "3.0.0");
        let next = program_selection::read(
            &f.source.root,
            program_layout::host_target().unwrap().1,
            "darwin-arm64",
        )
        .unwrap()
        .unwrap();
        assert_eq!(next.selection.version, "3.0.0");
        assert!(f.current.paths.version_root.exists());
        assert_eq!(std::fs::read(f.home.join("config.json")).unwrap(), f.config);
        assert_eq!(
            std::fs::read(next.paths.bun).unwrap(),
            std::fs::read(next.paths.package_root.join("node_modules/bun/bin/bun.exe")).unwrap()
        );
        assert_eq!(
            crate::modules::runtime::RuntimeStore::new(&f.data)
                .load()
                .history
                .last()
                .unwrap()
                .version,
            "3.0.0"
        );
        assert_eq!(
            crate::modules::model_config::config_file::v2::native_import::list_backups(
                &f.data, &f.home
            )
            .unwrap()
            .len(),
            1
        );
        assert!(!f.source.root.join("runtime/opencodex").exists());
        assert!(!f
            .source
            .root
            .join("runtime/.selection-pending.json")
            .exists());
        assert!(!std::fs::read_dir(&f.source.root)
            .unwrap()
            .flatten()
            .any(|e| e
                .file_name()
                .to_string_lossy()
                .starts_with(".runtime-install-stage-")));
    }
}

#[test]
fn candidate_errors_cancel_unknown_service_and_failed_backup_never_switch_current() {
    for scenario in [
        "escape",
        "cancel",
        "owner",
        "backup",
        "collision",
        "version",
        "tamper",
    ] {
        let f = Fixture::new();
        let npm = Npm {
            archive: f.archive.clone(),
            escape: scenario == "escape",
            tamper_official: scenario == "tamper",
        };
        let sink = RecordingProgressSink::default();
        let probe = FixedVersionProbe {
            version: Some("3.0.0".into()),
        };
        let mut installer = RuntimeInstaller::new(
            &f.data,
            &npm,
            None,
            &sink,
            &probe,
            Some(f.current.paths.node.clone()),
        );
        installer.config_backup = Some(UpgradeRequest {
            home: f.home.clone(),
            program_source: None,
        });
        if scenario == "cancel" {
            installer.cancel.cancel();
        }
        if scenario == "backup" {
            std::fs::write(f.data.join("backups"), b"blocked").unwrap();
        }
        if scenario == "collision" {
            write(
                &f.source.root.join("runtime/versions/3.0.0/foreign"),
                b"keep",
                0o600,
            );
        }
        let mut request = f.request(false);
        if scenario == "version" {
            request.source = InstallSource::Registry {
                version: "4.0.0".into(),
                proxy: None,
            };
        }
        assert!(
            installer
                .install_versioned(&f.source.root, &f.current, &request, || {
                    if scenario == "owner" {
                        Err(AppError::NotConfigured)
                    } else {
                        Ok(())
                    }
                })
                .is_err(),
            "{scenario}"
        );
        f.old();
        assert!(!f
            .source
            .root
            .join("runtime/.selection-pending.json")
            .exists());
        assert!(!std::fs::read_dir(&f.source.root)
            .unwrap()
            .flatten()
            .any(|e| e
                .file_name()
                .to_string_lossy()
                .starts_with(".runtime-install-stage-")));
        if scenario == "collision" {
            assert_eq!(
                std::fs::read(f.source.root.join("runtime/versions/3.0.0/foreign")).unwrap(),
                b"keep"
            );
        }
    }
}

struct PublishedProbe {
    program_root: PathBuf,
    reject: bool,
}
impl VersionProbe for PublishedProbe {
    fn probe(&self, _: Option<&Path>, script: &Path) -> Option<String> {
        if self.reject && script.starts_with(self.program_root.join("runtime/versions/3.0.0")) {
            None
        } else {
            Some("3.0.0".into())
        }
    }
}
#[test]
fn retry_reuses_only_the_full_identical_candidate_after_pre_activation_failure() {
    let f = Fixture::new();
    let npm = Npm {
        archive: f.archive.clone(),
        escape: false,
        tamper_official: false,
    };
    let sink = RecordingProgressSink::default();
    let failing = PublishedProbe {
        program_root: f.source.root.clone(),
        reject: true,
    };
    let mut installer = RuntimeInstaller::new(
        &f.data,
        &npm,
        None,
        &sink,
        &failing,
        Some(f.current.paths.node.clone()),
    );
    installer.config_backup = Some(UpgradeRequest {
        home: f.home.clone(),
        program_source: None,
    });
    assert!(installer
        .install_versioned(&f.source.root, &f.current, &f.request(false), || Ok(()))
        .is_err());
    f.old();
    let published = f.source.root.join("runtime/versions/3.0.0");
    assert!(published.exists());
    let before = tree_digest(&published).unwrap();
    let good = PublishedProbe {
        program_root: f.source.root.clone(),
        reject: false,
    };
    let mut installer = RuntimeInstaller::new(
        &f.data,
        &npm,
        None,
        &sink,
        &good,
        Some(f.current.paths.node.clone()),
    );
    installer.config_backup = Some(UpgradeRequest {
        home: f.home.clone(),
        program_source: None,
    });
    installer
        .install_versioned(&f.source.root, &f.current, &f.request(false), || Ok(()))
        .unwrap();
    assert_eq!(tree_digest(&published).unwrap(), before);
    assert_eq!(
        crate::modules::runtime::RuntimeStore::new(&f.data)
            .load()
            .history
            .len(),
        1
    );
}

#[test]
fn service_appearing_during_activation_retains_the_journal_and_allows_explicit_rollback() {
    let f = Fixture::new();
    let npm = Npm {
        archive: f.archive.clone(),
        escape: false,
        tamper_official: false,
    };
    let sink = RecordingProgressSink::default();
    let probe = FixedVersionProbe {
        version: Some("3.0.0".into()),
    };
    let mut installer = RuntimeInstaller::new(
        &f.data,
        &npm,
        None,
        &sink,
        &probe,
        Some(f.current.paths.node.clone()),
    );
    installer.config_backup = Some(UpgradeRequest {
        home: f.home.clone(),
        program_source: None,
    });
    let checks = std::cell::Cell::new(0);
    let result = installer.install_versioned(&f.source.root, &f.current, &f.request(false), || {
        checks.set(checks.get() + 1);
        if checks.get() >= 5 {
            Err(AppError::NotConfigured)
        } else {
            Ok(())
        }
    });
    assert!(result.is_err());
    assert!(f
        .source
        .root
        .join("runtime/.selection-pending.json")
        .is_file());
    assert!(program_selection::read(
        &f.source.root,
        program_layout::host_target().unwrap().1,
        "darwin-arm64"
    )
    .is_err());
    let mutation = Mutation::acquire(&f.source.root).unwrap();
    let pending = mutation.pending_summary(&f.data).unwrap();
    mutation
        .recover(
            &f.data,
            &pending.attempt_id,
            crate::modules::runtime::program_activation::Decision::Rollback,
            || Ok(()),
        )
        .unwrap();
    f.old();
    assert!(f.source.root.join("runtime/versions/3.0.0").is_dir());
    assert!(crate::modules::runtime::RuntimeStore::new(&f.data)
        .load()
        .history
        .is_empty());
}

#[test]
#[ignore = "requires explicit pinned official2.99 bundle and isolated npm registry access"]
fn real_official299_registry_candidate_uses_actual_npm_then_offline_reopen() {
    let archive =
        PathBuf::from(std::env::var_os("OCXD_TEST_BUNDLE_ARCHIVE").expect("bundle archive"));
    let descriptor =
        PathBuf::from(std::env::var_os("OCXD_TEST_BUNDLE_DESCRIPTOR").expect("bundle descriptor"));
    let pin = std::env::var("OCXD_TEST_BUNDLE_PIN").expect("trusted pin");
    assert_eq!(
        pin,
        "e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3"
    );
    let root = tempfile::tempdir().unwrap();
    let base = root.path().canonicalize().unwrap();
    let i = base.join("program");
    let d = base.join("desktop");
    let home = base.join("home");
    let profile = base.join("profile");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir(&profile).unwrap();
    let original = br#"{"port":41234,"providers":{},"fixture":"never launch a service"}"#;
    std::fs::write(home.join("config.json"), original).unwrap();
    crate::modules::runtime::bundled::versioned::provision(
        &i,
        &d,
        &archive,
        &std::fs::read(descriptor).unwrap(),
        &pin,
    )
    .unwrap();
    let current =
        program_selection::read(&i, program_layout::host_target().unwrap().1, "darwin-arm64")
            .unwrap()
            .unwrap();
    let npm = SystemNpmRunner::new(Some(profile), d.join("cache/npm"));
    let sink = RecordingProgressSink::default();
    let probe = RealVersionProbe::default();
    let mut installer = RuntimeInstaller::new(
        &d,
        &npm,
        None,
        &sink,
        &probe,
        Some(current.paths.node.clone()),
    );
    installer.config_backup = Some(UpgradeRequest {
        home: home.clone(),
        program_source: None,
    });
    let host = std::env::var("OCXD_TEST_REGISTRY_PROXY").expect("explicit proxy host:port");
    let request = InstallRequest {
        prefix: current.paths.package_root.clone(),
        source: InstallSource::Registry {
            version: "2.99.0".into(),
            proxy: Some(ProxyConfig::new(ProxyScheme::Http, host)),
        },
        allow_scripts: false,
        node_path: Some(current.paths.node.clone()),
        npm_path: Some(current.paths.npm.clone()),
    };
    let started = std::time::Instant::now();
    let result = installer.install_versioned(&i, &current, &request, || Ok(()));
    for line in sink.lines() {
        println!("registry: {line}");
    }
    let online = result.unwrap();
    assert!(!online.changed);
    assert_eq!(
        program_selection::read(&i, program_layout::host_target().unwrap().1, "darwin-arm64")
            .unwrap()
            .unwrap(),
        current
    );
    let source = current
        .paths
        .package_root
        .join("sources/opencodex-2.99.0.tgz");
    assert_eq!(
        digest(&source).unwrap(),
        "b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c"
    );
    let offline = InstallRequest {
        source: InstallSource::Offline { archive: source },
        ..request
    };
    let outcome = installer
        .install_versioned(&i, &current, &offline, || Ok(()))
        .unwrap();
    assert!(!outcome.changed);
    assert_eq!(std::fs::read(home.join("config.json")).unwrap(), original);
    assert!(!d.join("backups").exists());
    println!(
        "REAL_VERSIONED_INSTALL {}",
        json!({"version":"2.99.0","onlineNoop":!online.changed,"offlineNoop":!outcome.changed,"elapsedMs":started.elapsed().as_millis(),"actualNpm":true,"serviceStopProof":"test fixture injected","homeBytesPreserved":true})
    );
}
