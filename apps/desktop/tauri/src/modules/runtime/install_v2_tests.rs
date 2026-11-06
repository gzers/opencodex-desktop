//! Native npm/Node fixtures. Explicit executable paths, isolated HOME/cache/H.
//! No network, real user config, upstream inference or WebDAV access.
#![cfg(unix)]
use super::*;
use std::sync::{atomic::AtomicUsize, Mutex};

struct Service(Mutex<RecoveryServiceState>);
impl Service {
    fn stopped() -> Self {
        Self(Mutex::new(RecoveryServiceState::Stopped))
    }
}
impl RecoveryServiceBoundary for Service {
    fn snapshot(&self) -> Result<RecoveryServiceState, InstallError> {
        Ok(self.0.lock().unwrap().clone())
    }
    fn stop_owned(&self, identity: &str) -> Result<(), InstallError> {
        let mut state = self.0.lock().unwrap();
        if *state
            != (RecoveryServiceState::Owned {
                identity: identity.into(),
            })
        {
            return Err(recovery_error("foreign_service", "ownership changed"));
        }
        *state = RecoveryServiceState::Stopped;
        Ok(())
    }
}

struct IsolatedProbe {
    home: PathBuf,
    h: PathBuf,
    calls: AtomicUsize,
    fail_at: AtomicUsize,
}
impl VersionProbe for IsolatedProbe {
    fn probe(&self, node: Option<&Path>, script: &Path) -> Option<String> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        let output = Command::new(node?)
            .arg(script)
            .arg("--version")
            .env_clear()
            .env("HOME", &self.home)
            .env("OPENCODEX_HOME", &self.h)
            .env("PATH", node?.parent()?)
            .stdin(Stdio::null())
            .output()
            .ok()?;
        if call == self.fail_at.load(Ordering::SeqCst) || !output.status.success() {
            return None;
        }
        extract_semver(&String::from_utf8_lossy(&output.stdout))
    }
}

struct Fixture {
    temp: tempfile::TempDir,
    i: PathBuf,
    d: PathBuf,
    h: PathBuf,
    home: PathBuf,
    npm_path: PathBuf,
    node: PathBuf,
    archive: PathBuf,
    runner: SystemNpmRunner,
    probe: IsolatedProbe,
    sink: RecordingProgressSink,
    log: PathBuf,
}
impl Fixture {
    fn new(label: &str) -> Self {
        let temp = tempfile::Builder::new()
            .prefix("ocxd023-runtime-recovery-")
            .tempdir_in("/tmp")
            .unwrap();
        // /tmp is the explicit test anchor; /private is correctly system-protected.
        let base = temp.path().to_path_buf();
        let i = base.join("Desktop 安装 I");
        let d = base.join("U/desktop");
        let h = base.join("U/opencodex/home");
        let home = base.join("os-user");
        for path in [&i, &h, &home] {
            std::fs::create_dir_all(path).unwrap();
        }
        std::fs::write(
            h.join("config.json"),
            r#"{"providers":{"渠道 保留":{"apiKey":"synthetic-only"}}}"#.as_bytes(),
        )
        .unwrap();
        crate::modules::data_root::initialize(&d).unwrap();
        crate::modules::preferences::PreferencesStore::new(&d)
            .save(&crate::modules::preferences::Preferences::default())
            .unwrap();
        let supplied_node = std::fs::canonicalize(PathBuf::from(
            std::env::var_os("OPENCODEX_TEST_NODE").expect("set explicit OPENCODEX_TEST_NODE"),
        ))
        .unwrap();
        let supplied_npm = std::fs::canonicalize(PathBuf::from(
            std::env::var_os("OPENCODEX_TEST_NPM").expect("set explicit OPENCODEX_TEST_NPM"),
        ))
        .unwrap();
        let (node, npm_path) = managed_dependency_paths(&i);
        std::fs::create_dir_all(node.parent().unwrap()).unwrap();
        std::fs::copy(supplied_node, &node).unwrap();
        fn copy_tree(source: &Path, target: &Path) {
            std::fs::create_dir_all(target).unwrap();
            for entry in std::fs::read_dir(source).unwrap() {
                let entry = entry.unwrap();
                let from = entry.path();
                let to = target.join(entry.file_name());
                if from.is_dir() {
                    copy_tree(&from, &to);
                } else {
                    std::fs::copy(from, to).unwrap();
                }
            }
        }
        copy_tree(
            supplied_npm.parent().unwrap().parent().unwrap(),
            npm_path.parent().unwrap().parent().unwrap(),
        );
        let package = base.join("synthetic-package");
        std::fs::create_dir_all(package.join("bin")).unwrap();
        std::fs::write(package.join("package.json"), serde_json::to_vec(&serde_json::json!({"name":OFFICIAL_PACKAGE,"version":"1.2.3","bin":{"ocx":"bin/ocx.js"},"files":["bin"]})).unwrap()).unwrap();
        std::fs::write(
            package.join("bin/ocx.js"),
            "#!/usr/bin/env node\nconsole.log('opencodex 1.2.3');\n",
        )
        .unwrap();
        crate::infrastructure::platform::set_mode(&package.join("bin/ocx.js"), 0o755).unwrap();
        let log = PathBuf::from(format!(
            "/tmp/ocxd023-runtime-recovery-{label}-{}.log",
            uuid::Uuid::new_v4()
        ));
        let packed = Command::new(&node)
            .arg(&npm_path)
            .args(["pack", "--ignore-scripts", "--offline", "--json"])
            .current_dir(&package)
            .env_clear()
            .env("HOME", &home)
            .env(
                "PATH",
                crate::infrastructure::platform::controlled_path(
                    [
                        node.parent().unwrap().to_path_buf(),
                        npm_path.parent().unwrap().to_path_buf(),
                    ],
                    None,
                ),
            )
            .env("npm_config_cache", d.join("cache/npm"))
            .env("npm_config_audit", "false")
            .env("npm_config_update_notifier", "false")
            .output()
            .unwrap();
        std::fs::write(&log, [&packed.stdout[..], &packed.stderr[..]].concat()).unwrap();
        assert!(
            packed.status.success(),
            "npm pack failed; {}",
            log.display()
        );
        let packed_json: serde_json::Value = serde_json::from_slice(&packed.stdout).unwrap();
        let archive = package.join(packed_json[0]["filename"].as_str().unwrap());
        let runner = SystemNpmRunner::new(Some(home.clone()), d.join("cache/npm"));
        let probe = IsolatedProbe {
            home: home.clone(),
            h: h.clone(),
            calls: AtomicUsize::new(0),
            fail_at: AtomicUsize::new(0),
        };
        Self {
            temp,
            i,
            d,
            h,
            home,
            npm_path,
            node,
            archive,
            runner,
            probe,
            sink: RecordingProgressSink::default(),
            log,
        }
    }
    fn installer(&self) -> RuntimeInstaller<'_> {
        let mut installer = RuntimeInstaller::new(
            &self.d,
            &self.runner,
            Some(self.temp.path().to_path_buf()),
            &self.sink,
            &self.probe,
            Some(self.node.clone()),
        );
        installer.installation_root = Some(self.i.clone());
        installer
    }
    fn prefix(&self) -> PathBuf {
        self.i.join(super::super::MANAGED_PREFIX_RELATIVE)
    }
    fn entry(&self) -> PathBuf {
        self.i.join(MANAGED_ENTRY_RELATIVE)
    }
    fn request(&self) -> InstallRequest {
        InstallRequest {
            prefix: self.prefix(),
            source: InstallSource::Offline {
                archive: self.archive.clone(),
            },
            allow_scripts: false,
            npm_path: Some(self.npm_path.clone()),
            node_path: Some(self.node.clone()),
        }
    }
    fn source(&self) -> RuntimeRecoverySource {
        RuntimeRecoverySource {
            archive: self.archive.clone(),
            expected_sha256: archive::inspect(&self.archive, None).unwrap().sha256,
            expected_version: "1.2.3".into(),
        }
    }
    fn install(&self) {
        use crate::modules::{
            backup::manager,
            runtime::protection::{self, PanelProtection, ProtectionState},
        };
        let config = self.preserved_config();
        let guard = manager::begin_preferences_protection(&self.d, true).unwrap();
        let receipt = PanelProtection::arm_for_source(
            &self.d,
            &guard,
            &self.prefix(),
            "1.2.3",
            Some(&self.node),
            InstallSourceKind::Offline,
        )
        .unwrap()
        .unwrap();
        let mut installer = self.installer();
        installer.attempt_id = Some(receipt.attempt_id());
        installer.install(&self.request()).unwrap();
        assert_eq!(protection::status(&self.d).state, ProtectionState::Pending);
        // Drop the original objects and reopen from D. A good D launcher must
        // never mask a changed I launcher, even with exactly the expected bytes.
        drop(receipt);
        drop(guard);
        let good_entry = std::fs::read(self.entry()).unwrap();
        let decoy = self.d.join(MANAGED_ENTRY_RELATIVE);
        std::fs::create_dir_all(decoy.parent().unwrap()).unwrap();
        std::fs::write(&decoy, &good_entry).unwrap();
        crate::infrastructure::platform::set_mode(&decoy, 0o755).unwrap();
        std::fs::write(self.entry(), b"tampered actual I launcher").unwrap();
        let mut reopened = PanelProtection::load(&self.d).unwrap().unwrap();
        let guard = manager::acquire_preferences_transaction(&self.d).unwrap();
        assert!(reopened.reconcile(&self.d, &guard).is_err());
        assert_eq!(protection::status(&self.d).state, ProtectionState::Pending);
        std::fs::write(self.entry(), good_entry).unwrap();
        std::fs::remove_dir_all(self.d.join("runtime")).unwrap();
        drop(guard);
        let mut reopened = PanelProtection::load(&self.d).unwrap().unwrap();
        let guard = manager::acquire_preferences_transaction(&self.d).unwrap();
        reopened.reconcile(&self.d, &guard).unwrap();
        assert_eq!(protection::status(&self.d).state, ProtectionState::None);
        assert_eq!(self.preserved_config(), config);
        let mut log = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.log)
            .unwrap();
        writeln!(log, "protection reopen: corrupt I + valid D refused; restored I + missing D verified; state remained in D; HOME unchanged").unwrap();
    }
    fn mismatch(&self) {
        let package = self.prefix().join(PACKAGE_SUBPATH).join("package.json");
        let mut json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&package).unwrap()).unwrap();
        json["version"] = "9.9.9".into();
        std::fs::write(package, serde_json::to_vec(&json).unwrap()).unwrap();
        std::fs::write(
            self.prefix().join(PACKAGE_SUBPATH).join("bin/ocx.js"),
            "console.log('opencodex 9.9.9')\n",
        )
        .unwrap();
    }
    fn prepare(&self, service: &dyn RecoveryServiceBoundary) -> RuntimeRecoveryOperation {
        self.installer()
            .prepare_recovery(self.source(), self.npm_path.clone(), service)
            .unwrap()
    }
    fn preserved_config(&self) -> Vec<u8> {
        std::fs::read(self.h.join("config.json")).unwrap()
    }
    fn assert_consistent(&self) {
        validate_managed_installation(&self.i, Some(&self.node)).unwrap();
        let manifest: RuntimeManifest =
            serde_json::from_slice(&std::fs::read(self.prefix().join(MANIFEST_FILENAME)).unwrap())
                .unwrap();
        let record: super::super::RuntimeSourceRecord =
            serde_json::from_slice(&std::fs::read(recovery_record_path(&self.d)).unwrap()).unwrap();
        assert_eq!(manifest.version, "1.2.3");
        assert_eq!(manifest.tarball_sha256, self.source().expected_sha256);
        assert_eq!(record.source, super::super::RuntimeSourceKind::Managed);
        assert_eq!(
            record.path.as_deref().map(Path::new),
            Some(self.entry().as_path())
        );
        assert_eq!(record.resolved_version.as_deref(), Some("1.2.3"));
        let output = Command::new(self.entry())
            .arg("--version")
            .env_clear()
            .env("HOME", &self.home)
            .env("OPENCODEX_HOME", &self.h)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            extract_semver(&String::from_utf8_lossy(&output.stdout)).as_deref(),
            Some("1.2.3")
        );
        assert!(!self.d.join("runtime").exists());
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&self.log)
            .unwrap();
        writeln!(
            file,
            "\nfixture={}\nprogress={:#?}\n",
            self.temp.path().display(),
            self.sink.events()
        )
        .unwrap();
        eprintln!("npm fixture evidence: {}", self.log.display());
    }
}

#[test]
#[ignore = "native npm integration: set absolute OPENCODEX_TEST_NODE and OPENCODEX_TEST_NPM; run --ignored"]
fn v2_real_npm_mismatch_recovery_keeps_old_package_home_and_reinstalls() {
    let f = Fixture::new("mismatch");
    f.install();
    f.mismatch();
    let config = f.preserved_config();
    let before = recovery_tree_digest(&f.prefix()).unwrap();
    assert!(f.installer().install(&f.request()).is_err());
    let service = Service::stopped();
    let whole_before =
        recovery_tree_digest(&std::fs::canonicalize(f.temp.path()).unwrap()).unwrap();
    let mut operation = f.prepare(&service);
    assert_eq!(
        whole_before,
        recovery_tree_digest(&std::fs::canonicalize(f.temp.path()).unwrap()).unwrap(),
        "prepare must be read-only"
    );
    assert_eq!(operation.preview.package_version.as_deref(), Some("9.9.9"));
    assert_eq!(operation.preview.manifest_version.as_deref(), Some("1.2.3"));
    let id = operation.preview.operation_id.clone();
    assert!(f
        .installer()
        .apply_recovery(&mut operation, "wrong-id", &service)
        .is_err());
    f.installer()
        .apply_recovery(&mut operation, &id, &service)
        .unwrap();
    assert_eq!(
        recovery_tree_digest(&operation.preview.retained_prefix).unwrap(),
        before
    );
    assert_eq!(
        std::fs::read(operation.preview.metadata_backup.join("entry.before")).unwrap(),
        operation.entry_before.clone().unwrap()
    );
    assert_eq!(
        std::fs::read(
            operation
                .preview
                .metadata_backup
                .join("runtime.before.json")
        )
        .unwrap(),
        operation.record_before.clone().unwrap()
    );
    assert_eq!(
        RuntimeRecoveryOperation::read_status(&f.d, &id).unwrap(),
        operation.status()
    );
    assert_eq!(operation.status.phase, RuntimeRecoveryPhase::Succeeded);
    assert!(operation.status.restart_required);
    assert!(operation.cancel().is_err());
    f.assert_consistent();
    assert_eq!(f.preserved_config(), config);
    f.installer().install(&f.request()).unwrap();
    f.assert_consistent();
    assert_eq!(f.preserved_config(), config);
    assert_eq!(
        recovery_tree_digest(&operation.preview.retained_prefix).unwrap(),
        before
    );
}

#[test]
#[ignore = "native npm integration: set absolute OPENCODEX_TEST_NODE and OPENCODEX_TEST_NPM; run --ignored"]
fn v2_real_npm_corrupted_manifest_package_and_source_recover_with_positive_provenance() {
    let f = Fixture::new("corruption");
    f.install();
    let service = Service::stopped();
    let config = f.preserved_config();
    for target in [
        f.prefix().join(MANIFEST_FILENAME),
        f.prefix().join(PACKAGE_SUBPATH).join("package.json"),
        recovery_record_path(&f.d),
    ] {
        std::fs::write(&target, b"{corrupted").unwrap();
        assert!(f.installer().install(&f.request()).is_err());
        let mut operation = f.prepare(&service);
        let id = operation.preview.operation_id.clone();
        let before = recovery_tree_digest(&f.prefix()).unwrap();
        f.installer()
            .apply_recovery(&mut operation, &id, &service)
            .unwrap();
        assert_eq!(
            recovery_tree_digest(&operation.preview.retained_prefix).unwrap(),
            before
        );
        f.assert_consistent();
        assert_eq!(f.preserved_config(), config);
    }
}

#[test]
#[ignore = "native npm integration: set absolute OPENCODEX_TEST_NODE and OPENCODEX_TEST_NPM; run --ignored"]
fn v2_real_npm_activation_failure_rolls_back_exactly_and_new_preview_retries() {
    let f = Fixture::new("rollback-retry");
    f.install();
    f.mismatch();
    let service = Service::stopped();
    let config = f.preserved_config();
    let before = recovery_tree_digest(&f.prefix()).unwrap();
    let entry_before = std::fs::read(f.entry()).unwrap();
    let record_before = std::fs::read(recovery_record_path(&f.d)).unwrap();
    let mut operation = f.prepare(&service);
    let id = operation.preview.operation_id.clone();
    // Offline fill probe, staging verification, then real final-prefix probe.
    f.probe
        .fail_at
        .store(f.probe.calls.load(Ordering::SeqCst) + 3, Ordering::SeqCst);
    assert!(f
        .installer()
        .apply_recovery(&mut operation, &id, &service)
        .is_err());
    assert_eq!(operation.status.phase, RuntimeRecoveryPhase::Failed);
    assert_eq!(
        RuntimeRecoveryOperation::read_status(&f.d, &id).unwrap(),
        operation.status()
    );
    assert_eq!(recovery_tree_digest(&f.prefix()).unwrap(), before);
    assert_eq!(std::fs::read(f.entry()).unwrap(), entry_before);
    assert_eq!(
        std::fs::read(recovery_record_path(&f.d)).unwrap(),
        record_before
    );
    assert!(!operation.preview.retained_prefix.exists());
    assert!(operation.preview.metadata_backup.exists());
    f.probe.fail_at.store(0, Ordering::SeqCst);
    let mut retry = f.prepare(&service);
    let retry_id = retry.preview.operation_id.clone();
    assert_ne!(id, retry_id);
    f.installer()
        .apply_recovery(&mut retry, &retry_id, &service)
        .unwrap();
    f.assert_consistent();
    assert_eq!(f.preserved_config(), config);
}

#[test]
#[ignore = "native npm integration: set absolute OPENCODEX_TEST_NODE and OPENCODEX_TEST_NPM; run --ignored"]
fn v2_real_npm_refuses_unknown_source_program_and_foreign_service_without_writes() {
    let f = Fixture::new("unknown-refusal");
    f.install();
    f.mismatch();
    let service = Service::stopped();
    let before = recovery_tree_digest(&f.prefix()).unwrap();
    let mut source = f.source();
    source.expected_sha256 = "0".repeat(64);
    assert!(f
        .installer()
        .prepare_recovery(source, f.npm_path.clone(), &service)
        .is_err());
    let unknown = Service(Mutex::new(RecoveryServiceState::Unknown));
    assert!(f
        .installer()
        .prepare_recovery(f.source(), f.npm_path.clone(), &unknown)
        .is_err());
    assert_eq!(recovery_tree_digest(&f.prefix()).unwrap(), before);
    std::fs::remove_file(f.prefix().join(MANIFEST_FILENAME)).unwrap();
    std::fs::remove_file(recovery_record_path(&f.d)).unwrap();
    let unknown_before = recovery_tree_digest(&f.prefix()).unwrap();
    assert!(f
        .installer()
        .prepare_recovery(f.source(), f.npm_path.clone(), &service)
        .is_err());
    assert!(f.installer().install(&f.request()).is_err());
    assert_eq!(recovery_tree_digest(&f.prefix()).unwrap(), unknown_before);
    assert!(!f.d.join("backups/runtime-recovery").exists());
}

#[test]
#[ignore = "native npm integration: set absolute OPENCODEX_TEST_NODE and OPENCODEX_TEST_NPM; run --ignored"]
fn v2_real_npm_preview_changes_cancel_and_service_identity_fail_closed() {
    let f = Fixture::new("stale-cancel-service");
    f.install();
    f.mismatch();
    let service = Service::stopped();
    let mut cancelled = f.prepare(&service);
    let id = cancelled.preview.operation_id.clone();
    cancelled.cancel().unwrap();
    assert!(f
        .installer()
        .apply_recovery(&mut cancelled, &id, &service)
        .is_err());
    assert!(!f.d.join("manager-state/runtime-recovery").exists());
    for path in [
        f.prefix().join("external-change"),
        f.entry(),
        recovery_record_path(&f.d),
        f.archive.clone(),
    ] {
        let previous = std::fs::read(&path).ok();
        let mut op = f.prepare(&service);
        let id = op.preview.operation_id.clone();
        std::fs::write(&path, b"external mutation").unwrap();
        assert!(f
            .installer()
            .apply_recovery(&mut op, &id, &service)
            .is_err());
        assert_eq!(op.status.phase, RuntimeRecoveryPhase::Prepared);
        match previous {
            Some(bytes) => std::fs::write(&path, bytes).unwrap(),
            None => std::fs::remove_file(&path).unwrap(),
        }
    }
    let mut op = f.prepare(&service);
    let id = op.preview.operation_id.clone();
    *service.0.lock().unwrap() = RecoveryServiceState::Owned {
        identity: "foreign-or-changed".into(),
    };
    assert!(f
        .installer()
        .apply_recovery(&mut op, &id, &service)
        .is_err());
    *service.0.lock().unwrap() = RecoveryServiceState::Owned {
        identity: "proven-user-instance-generation-1".into(),
    };
    let mut owned = f.prepare(&service);
    let owned_id = owned.preview.operation_id.clone();
    f.installer()
        .apply_recovery(&mut owned, &owned_id, &service)
        .unwrap();
    assert_eq!(service.snapshot().unwrap(), RecoveryServiceState::Stopped);
    f.assert_consistent();
}

#[test]
#[ignore = "native npm integration: set absolute OPENCODEX_TEST_NODE and OPENCODEX_TEST_NPM; run --ignored"]
fn v2_real_npm_installation_root_is_separate_and_rejects_other_prefixes() {
    let f = Fixture::new("path-separation");
    let mut request = f.request();
    request.prefix = f.d.join("runtime/opencodex");
    assert!(f.installer().install(&request).is_err());
    assert!(!f.d.join("runtime").exists());
    request.prefix = f.temp.path().join("other-prefix");
    assert!(f.installer().install(&request).is_err());
    assert!(!request.prefix.exists());
    f.install();
    f.assert_consistent();
    let calls = f.probe.calls.load(Ordering::SeqCst);
    let identity = resolve_managed_installation(&f.i).unwrap();
    assert_eq!(identity.node, f.node);
    assert_eq!(identity.entry, f.entry());
    let handle = super::super::RuntimeHandle::initialize_managed(&f.d, &f.i);
    assert_eq!(
        crate::commands::runtime::managed_executables(&handle).unwrap(),
        (Some(f.node.clone()), Some(f.npm_path.clone()))
    );
    let cli_before = std::fs::read(&f.npm_path).unwrap();
    std::fs::write(&f.npm_path, "if(process.argv[2] !== 'view') process.exit(31); console.log(JSON.stringify({version:'1.2.3', 'dist.integrity':'sha512-synthetic'}));").unwrap();
    let environment = crate::modules::process::EnvironmentPolicy {
        opencodex_home: f.h.clone(),
        home: Some(f.home.clone().into_os_string()),
        path: Some("/definitely-no-node".into()),
        ..Default::default()
    };
    let remote = crate::commands::runtime::query_runtime_remote_latest(
        &handle,
        &f.home,
        &f.d.join("cache/npm"),
        &f.d.join("logs/runtime-query"),
        &environment,
        "latest",
    )
    .unwrap();
    assert_eq!(remote.version, "1.2.3");
    std::fs::write(&f.npm_path, cli_before).unwrap();
    let moved_node = f.node.with_file_name("node-saved");
    std::fs::rename(&f.node, &moved_node).unwrap();
    assert!(crate::commands::runtime::managed_executables(&handle).is_err());
    assert!(validate_managed_installation(&f.i, Some(&f.node)).is_err());
    std::fs::rename(moved_node, &f.node).unwrap();
    let service = Service::stopped();
    let installer = f.installer();
    let mut operation = crate::commands::runtime::prepare_runtime_recovery(
        &installer,
        f.source(),
        f.npm_path.clone(),
        &service,
    )
    .unwrap();
    crate::commands::runtime::cancel_runtime_recovery(&mut operation).unwrap();
    assert_eq!(
        f.probe.calls.load(Ordering::SeqCst),
        calls,
        "resolver must not execute the package"
    );
    let manifest_path = f.prefix().join(MANIFEST_FILENAME);
    let manifest_bytes = std::fs::read(&manifest_path).unwrap();
    let good: serde_json::Value = serde_json::from_slice(&manifest_bytes).unwrap();
    for bad_node in [
        serde_json::Value::Null,
        serde_json::json!("node"),
        serde_json::json!(f.temp.path().join("absent-fixed-node")),
        serde_json::json!("/bin/sh"),
    ] {
        let mut bad = good.clone();
        bad["activation"]["node"] = bad_node;
        std::fs::write(&manifest_path, serde_json::to_vec(&bad).unwrap()).unwrap();
        assert!(
            resolve_managed_installation(&f.i).is_err(),
            "missing/changed Node must never downgrade to launcher-only admission"
        );
    }
    let mut missing_proof = good;
    missing_proof["activation"] = serde_json::Value::Null;
    std::fs::write(&manifest_path, serde_json::to_vec(&missing_proof).unwrap()).unwrap();
    assert!(resolve_managed_installation(&f.i).is_err());
    std::fs::write(&manifest_path, manifest_bytes).unwrap();
    let mut missing_node = f.installer();
    missing_node.node_path = None;
    let mut request = f.request();
    request.node_path = None;
    assert!(missing_node.install(&request).is_err());
    for path in [
        "../escape.js",
        "./../escape.js",
        "C:\\escape.js",
        "bin\\..\\escape.js",
        ".//absolute.js",
    ] {
        assert!(normalize_bin(path).is_empty(), "{path}");
    }
}
