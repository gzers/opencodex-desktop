use super::*;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

fn tgz(path: &Path, script: &[u8]) {
    let gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    let mut tar = tar::Builder::new(gzip);
    for (name, body) in [
        (
            "package/package.json",
            br#"{"name":"@bitkyc08/opencodex","version":"1.2.3","bin":{"ocx":"bin/ocx.js"}}"#
                .as_slice(),
        ),
        ("package/bin/ocx.js", script),
    ] {
        let mut header = tar::Header::new_gnu();
        header.set_size(body.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        tar.append_data(&mut header, name, body).unwrap();
    }
    std::fs::write(path, tar.into_inner().unwrap().finish().unwrap()).unwrap();
}
fn fixture_pin(path: &Path) -> RecoverySourcePin {
    // A synthetic distribution authority for test bytes only, never production.
    RecoverySourcePin::from_trusted_metadata(
        "1.2.3".into(),
        archive::inspect(path, None).unwrap().sha256,
    )
    .unwrap()
}
struct Source {
    path: PathBuf,
    pin: RecoverySourcePin,
    receiver: SharedOfflineDrop,
    copies: Mutex<Vec<PathBuf>>,
}
impl SourceResolver for Source {
    fn resolve(
        &self,
        request: &RuntimeRecoveryPrepareRequest,
        _: &RecoveryContext,
    ) -> AppResult<ResolvedRecoverySource> {
        let result = match request.source {
            RecoverySourceKind::Bundled => {
                ResolvedRecoverySource::copy_pinned(&self.path, &self.pin)
            }
            RecoverySourceKind::Offline => ResolvedRecoverySource::from_staged_offline(
                &self.receiver,
                request.staged_archive_id.as_deref().unwrap(),
                &self.pin,
            ),
        }?;
        self.copies
            .lock()
            .unwrap()
            .push(result.source.archive.clone());
        Ok(result)
    }
}
struct Proof {
    h: PathBuf,
    state: Mutex<ServiceOwnerProof>,
    stops: AtomicUsize,
}
impl ServiceOwnerProofProvider for Proof {
    fn query(&self, context: &ProcessContext) -> AppResult<ServiceOwnerProof> {
        assert_eq!(context.opencodex_home, self.h);
        Ok(self.state.lock().unwrap().clone())
    }
    fn stop_owned(&self, context: &ProcessContext, identity: &str, pid: u32) -> AppResult<()> {
        assert_eq!(context.opencodex_home, self.h);
        let mut state = self.state.lock().unwrap();
        if *state
            != (ServiceOwnerProof::Owned {
                identity: identity.into(),
                pid,
            })
        {
            return Err(error(
                "fixture_owner_changed",
                "fixture PID creation identity changed",
            ));
        }
        self.stops.fetch_add(1, Ordering::SeqCst);
        *state = ServiceOwnerProof::Stopped;
        Ok(())
    }
}

#[test]
fn source_pin_and_unknown_staged_id_fail_closed_and_owned_copy_drops() {
    let root = tempfile::tempdir().unwrap();
    let archive = root.path().join("opencodex-1.2.3.tgz");
    tgz(&archive, b"console.log('1.2.3')");
    let pin = fixture_pin(&archive);
    let copy = ResolvedRecoverySource::copy_pinned(&archive, &pin).unwrap();
    let copied = copy.source.archive.clone();
    assert!(copied.exists());
    drop(copy);
    assert!(!copied.exists());
    assert!(archive.exists());
    let bad = RecoverySourcePin::from_trusted_metadata("1.2.3".into(), "0".repeat(64)).unwrap();
    assert!(
        matches!(ResolvedRecoverySource::copy_pinned(&archive, &bad), Err(AppError::RuntimeManaged {code, ..}) if code == "recovery_source_pin_mismatch")
    );
    let receiver = SharedOfflineDrop::default();
    assert!(
        matches!(ResolvedRecoverySource::from_staged_offline(&receiver, &uuid::Uuid::new_v4().to_string(), &pin), Err(AppError::FileSystem {operation, ..}) if operation == "finish staged recovery archive")
    );
    let bytes = std::fs::read(&archive).unwrap();
    let mut drop = receiver.0.lock().unwrap();
    let id = drop
        .begin(root.path(), "opencodex-1.2.3.tgz", bytes.len() as u64)
        .unwrap();
    drop.append(&id, 0, &bytes).unwrap();
    let staged = drop.finish(&id).unwrap();
    std::mem::drop(drop);
    let copy = ResolvedRecoverySource::from_staged_offline(&receiver, &id, &pin).unwrap();
    assert!(!staged.exists());
    assert!(copy.source.archive.exists());
    assert!(ResolvedRecoverySource::from_staged_offline(&receiver, &id, &pin).is_err());
}

#[test]
#[cfg(any(
    all(target_os = "macos", target_arch = "aarch64"),
    all(windows, target_arch = "x86_64")
))]
fn fixed_resource_resolver_authenticates_descriptor_and_both_bundled_and_staged_sources() {
    use crate::commands::runtime_recovery_v2_sources::FixedResourceSourceResolver;
    let root = tempfile::tempdir().unwrap();
    let resources = root.path().join("runtime-bundle");
    let d = root.path().join("D");
    let i = root.path().join("I");
    let h = root.path().join("H");
    for path in [&resources, &d, &i, &h] {
        std::fs::create_dir_all(path).unwrap();
    }
    let original = root.path().join("opencodex-1.2.3.tgz");
    tgz(&original, b"console.log('1.2.3')");
    let source_pin = fixture_pin(&original);
    let archive = resources.join("maintained-runtime.tgz");
    std::fs::copy(&original, &archive).unwrap();
    let descriptor = serde_json::to_vec(&serde_json::json!({
        "schemaVersion":1,
        "platform": if cfg!(windows) {"win32-x64"} else {"darwin-arm64"},
        "runtimeVersion":"1.2.3",
        "runtimeArchiveSha256":source_pin.sha256,
        "entries":[], "archiveName":"whole-runtime.tar.gz"
    }))
    .unwrap();
    std::fs::write(resources.join("runtime-bundle.manifest.json"), &descriptor).unwrap();
    let descriptor_pin = crate::infrastructure::hash::sha256_hex(&descriptor);
    let receiver = SharedOfflineDrop::default();
    let runtime = RuntimeHandle::initialize_managed(&d, &i);
    let context = RecoveryContext::new(
        runtime.clone(),
        d.clone(),
        root.path().into(),
        ProcessContext {
            runtime,
            working_directory: d.clone(),
            opencodex_home: h,
        },
    )
    .unwrap();
    let resolver = FixedResourceSourceResolver::new(
        resources.clone(),
        receiver.clone(),
        Some(descriptor_pin.clone()),
        Some(source_pin.sha256.clone()),
        Some("1.2.3".into()),
    )
    .unwrap();
    let request = RuntimeRecoveryPrepareRequest {
        source: RecoverySourceKind::Bundled,
        staged_archive_id: None,
    };
    let copy = resolver.resolve(&request, &context).unwrap();
    assert_eq!(copy.source.expected_sha256, source_pin.sha256);
    let copied = copy.source.archive.clone();
    drop(copy);
    assert!(!copied.exists());
    // Missing build pins fail explicitly; a fully staged/pinned build above
    // has a working bundled path rather than a permanent bundled veto.
    let descriptor_only = FixedResourceSourceResolver::new(
        resources.clone(),
        receiver.clone(),
        Some(descriptor_pin.clone()),
        None,
        Some("1.2.3".into()),
    )
    .unwrap();
    assert!(
        matches!(descriptor_only.resolve(&request, &context), Err(AppError::RuntimeManaged {code,..}) if code == "recovery_source_pin_missing")
    );
    let bytes = std::fs::read(&archive).unwrap();
    let id = {
        let mut stage = receiver.0.lock().unwrap();
        let id = stage
            .begin(&d, "opencodex-1.2.3.tgz", bytes.len() as u64)
            .unwrap();
        stage.append(&id, 0, &bytes).unwrap();
        id
    };
    resolver
        .resolve(
            &RuntimeRecoveryPrepareRequest {
                source: RecoverySourceKind::Offline,
                staged_archive_id: Some(id.clone()),
            },
            &context,
        )
        .unwrap();
    assert!(receiver.0.lock().unwrap().finish(&id).is_err());
    let inconsistent = FixedResourceSourceResolver::new(
        resources.clone(),
        receiver.clone(),
        Some(descriptor_pin.clone()),
        Some("0".repeat(64)),
        Some("1.2.3".into()),
    )
    .unwrap();
    assert!(
        matches!(inconsistent.resolve(&request, &context), Err(AppError::RuntimeManaged {code,..}) if code == "recovery_source_pin_mismatch")
    );
    let wrong_version = FixedResourceSourceResolver::new(
        resources.clone(),
        receiver.clone(),
        Some(descriptor_pin),
        Some(source_pin.sha256.clone()),
        Some("1.2.4".into()),
    )
    .unwrap();
    assert!(
        matches!(wrong_version.resolve(&request, &context), Err(AppError::RuntimeManaged {code,..}) if code == "recovery_source_version_pin_mismatch")
    );
    let missing =
        FixedResourceSourceResolver::new(resources.clone(), receiver, None, None, None).unwrap();
    assert!(
        matches!(missing.resolve(&request, &context), Err(AppError::RuntimeManaged {code,..}) if code == "recovery_resource_pin_missing")
    );
    tgz(&archive, b"same filename but untrusted replacement");
    assert!(
        matches!(resolver.resolve(&request, &context), Err(AppError::RuntimeManaged {code,..}) if code == "recovery_source_pin_mismatch")
    );
    std::fs::write(
        resources.join("runtime-bundle.manifest.json"),
        b"forged adjacent descriptor",
    )
    .unwrap();
    assert!(
        matches!(resolver.resolve(&request, &context), Err(AppError::RuntimeManaged {code,..}) if code == "recovery_descriptor_pin_mismatch")
    );
}

struct AttestationFixture {
    stop: Arc<AtomicBool>,
    join: Option<std::thread::JoinHandle<()>>,
}
impl AttestationFixture {
    fn start(h: &Path, pid: u32) -> Self {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        use hmac::{Hmac, Mac};
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let port = listener.local_addr().unwrap().port();
        let secret = "x".repeat(43);
        std::fs::write(
            h.join("runtime-port.json"),
            serde_json::to_vec(
                &serde_json::json!({"pid":pid,"port":port,"attestationSecret":secret}),
            )
            .unwrap(),
        )
        .unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let end = stop.clone();
        let join = std::thread::spawn(move || {
            while !end.load(Ordering::SeqCst) {
                let (mut stream, _) = match listener.accept() {
                    Ok(value) => value,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(5));
                        continue;
                    }
                    Err(e) => panic!("fixture HTTP accept: {e}"),
                };
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(10)))
                    .unwrap();
                let mut request = Vec::new();
                let mut buffer = [0; 2048];
                while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                    let n = stream.read(&mut buffer).unwrap();
                    if n == 0 {
                        break;
                    }
                    request.extend_from_slice(&buffer[..n]);
                }
                let request = String::from_utf8(request).unwrap();
                assert!(request.starts_with("GET /healthz "));
                let challenge = request
                    .lines()
                    .find_map(|line| {
                        let (key, value) = line.split_once(':')?;
                        key.eq_ignore_ascii_case("x-opencodex-attestation-challenge")
                            .then(|| value.trim())
                    })
                    .unwrap();
                let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret.as_bytes()).unwrap();
                mac.update(
                    format!("opencodex-local-management-v1\n{challenge}\n{pid}\n{port}").as_bytes(),
                );
                let proof = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
                let body = serde_json::json!({"service":"opencodex","pid":pid}).to_string();
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nx-opencodex-attestation-proof: {proof}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
            }
        });
        Self {
            stop,
            join: Some(join),
        }
    }
}
impl Drop for AttestationFixture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        let result = self.join.take().unwrap().join();
        if !std::thread::panicking() {
            result.unwrap();
        }
    }
}

#[test]
fn service_boundary_requires_h_attestation_and_rechecks_identity_before_fake_stop() {
    let root = tempfile::tempdir().unwrap();
    let context = ProcessContext {
        runtime: crate::infrastructure::runtime_executable::FixedRuntimeExecutable::unresolved(),
        working_directory: root.path().into(),
        opencodex_home: root.path().into(),
    };
    let proof = Proof {
        h: root.path().into(),
        state: Mutex::new(ServiceOwnerProof::Unknown),
        stops: AtomicUsize::new(0),
    };
    let boundary = ServiceBoundary {
        context: &context,
        proof: &proof,
    };
    assert_eq!(boundary.snapshot().unwrap(), RecoveryServiceState::Unknown);
    assert!(boundary.stop_owned("synthetic-uid:pid:creation").is_err());
    *proof.state.lock().unwrap() = ServiceOwnerProof::Owned {
        identity: "synthetic-uid:pid:creation".into(),
        pid: 4242,
    };
    assert!(
        boundary.snapshot().is_err(),
        "no H record cannot authorize Owned"
    );
    let _server = AttestationFixture::start(root.path(), 4242);
    assert_eq!(
        boundary.snapshot().unwrap(),
        RecoveryServiceState::Owned {
            identity: "synthetic-uid:pid:creation".into()
        }
    );
    assert!(boundary.stop_owned("old-creation-identity").is_err());
    assert_eq!(proof.stops.load(Ordering::SeqCst), 0);
    boundary.stop_owned("synthetic-uid:pid:creation").unwrap();
    assert_eq!(proof.stops.load(Ordering::SeqCst), 1);
    assert_eq!(boundary.snapshot().unwrap(), RecoveryServiceState::Stopped);
    *proof.state.lock().unwrap() = ServiceOwnerProof::Owned {
        identity: "synthetic-uid:pid:new-creation".into(),
        pid: 4243,
    };
    assert!(
        boundary.snapshot().is_err(),
        "a healthy but different PID cannot authorize stop"
    );
    assert_eq!(proof.stops.load(Ordering::SeqCst), 1);
}

#[test]
fn ipc_rejects_source_paths_digests_and_operations() {
    for value in [
        serde_json::json!({"source":"bundled","path":"/tmp/anything.tgz"}),
        serde_json::json!({"source":"bundled","expectedSha256":"0".repeat(64)}),
        serde_json::json!({"source":"bundled","expectedVersion":"1.2.3"}),
        serde_json::json!({"source":"registry","url":"https://example.test/a"}),
        serde_json::json!({"source":"offline","operation":{}}),
    ] {
        assert!(serde_json::from_value::<RuntimeRecoveryPrepareRequest>(value).is_err());
    }
    for value in [
        serde_json::json!({"operationId":"x","confirmationDigest":"y","operation":{}}),
        serde_json::json!({"operationId":"x","confirmationDigest":"y","source":{}}),
    ] {
        assert!(serde_json::from_value::<RuntimeRecoveryApplyRequest>(value).is_err());
    }
    for value in [
        serde_json::json!({"source":"offline"}),
        serde_json::json!({"source":"offline","stagedArchiveId":"../../other"}),
        serde_json::json!({"source":"bundled","stagedArchiveId":uuid::Uuid::new_v4().to_string()}),
    ] {
        assert!(
            serde_json::from_value::<RuntimeRecoveryPrepareRequest>(value)
                .unwrap()
                .validate()
                .is_err()
        );
    }
}

#[test]
fn file_errors_keep_the_actual_operation() {
    assert!(
        matches!(install_error(InstallError::FileSystem { operation:"rename actual entry".into(),detail:"fixture".into() }), AppError::FileSystem { operation, .. } if operation == "rename actual entry")
    );
}

#[cfg(unix)]
mod native {
    use super::*;
    use crate::modules::runtime::install::{InstallRequest, InstallSource};
    use std::{io::Write, sync::mpsc, time::Duration};

    struct ControlledProof {
        inner: Proof,
        calls: AtomicUsize,
        block_at: AtomicUsize,
        entered: mpsc::Sender<()>,
        release: Mutex<mpsc::Receiver<()>>,
    }
    impl ServiceOwnerProofProvider for ControlledProof {
        fn query(&self, context: &ProcessContext) -> AppResult<ServiceOwnerProof> {
            let n = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
            if n == self.block_at.load(Ordering::SeqCst) {
                self.entered.send(()).unwrap();
                self.release
                    .lock()
                    .unwrap()
                    .recv_timeout(Duration::from_secs(30))
                    .unwrap();
            }
            self.inner.query(context)
        }
        fn stop_owned(&self, context: &ProcessContext, identity: &str, pid: u32) -> AppResult<()> {
            self.inner.stop_owned(context, identity, pid)
        }
    }
    fn snapshot(root: &Path) -> String {
        fn visit(root: &Path, path: &Path, bytes: &mut Vec<u8>) {
            let mut entries = std::fs::read_dir(path)
                .unwrap()
                .map(|e| e.unwrap().path())
                .collect::<Vec<_>>();
            entries.sort();
            for p in entries {
                bytes.extend_from_slice(p.strip_prefix(root).unwrap().to_string_lossy().as_bytes());
                let m = std::fs::symlink_metadata(&p).unwrap();
                if m.file_type().is_symlink() {
                    bytes.extend_from_slice(
                        std::fs::read_link(&p).unwrap().to_string_lossy().as_bytes(),
                    );
                } else if m.is_dir() {
                    visit(root, &p, bytes);
                } else {
                    bytes.extend_from_slice(
                        crate::infrastructure::hash::sha256_file(&p)
                            .unwrap()
                            .as_bytes(),
                    );
                }
            }
        }
        let mut bytes = Vec::new();
        visit(root, root, &mut bytes);
        crate::infrastructure::hash::sha256_hex(&bytes)
    }
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
    struct Fixture {
        root: tempfile::TempDir,
        recovery: SharedRuntimeRecoveryV2,
        source: Arc<Source>,
        proof: Arc<ControlledProof>,
        entered: mpsc::Receiver<()>,
        release: mpsc::Sender<()>,
        updater: super::super::super::update::SharedUpdateStatus,
        log: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let root = tempfile::Builder::new()
                .prefix("ocxd023-runtime-recovery-ipc-fixture-")
                .tempdir_in("/tmp")
                .unwrap();
            let i = root.path().join("Desktop I");
            let d = root.path().join("user/desktop");
            let h = root.path().join("user/opencodex/home");
            let home = root.path().join("os-user");
            for path in [&i, &h, &home] {
                std::fs::create_dir_all(path).unwrap();
            }
            std::fs::write(
                h.join("config.json"),
                br#"{"fixtureKey":"synthetic-never-daily"}"#,
            )
            .unwrap();
            std::fs::write(h.join("keychain.json"), b"synthetic preserved key bytes").unwrap();
            crate::modules::data_root::initialize(&d).unwrap();
            crate::modules::preferences::PreferencesStore::new(&d)
                .save(&crate::modules::preferences::Preferences::default())
                .unwrap();
            let supplied_node = std::fs::canonicalize(
                std::env::var_os("OPENCODEX_TEST_NODE").expect("explicit test Node required"),
            )
            .unwrap();
            let supplied_npm = std::fs::canonicalize(
                std::env::var_os("OPENCODEX_TEST_NPM").expect("explicit test npm6 cli required"),
            )
            .unwrap();
            let npm_json: serde_json::Value = serde_json::from_slice(
                &std::fs::read(
                    supplied_npm
                        .parent()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .join("package.json"),
                )
                .unwrap(),
            )
            .unwrap();
            let expected_major = std::env::var("OPENCODEX_TEST_NPM_MAJOR").unwrap_or_else(|_| "6".into());
            assert!(["6", "11"].contains(&expected_major.as_str()), "explicit supported fixture npm major");
            assert_eq!(npm_json["version"].as_str().unwrap().split('.').next(), Some(expected_major.as_str()),
                "native fixture must use the explicitly selected npm major");
            let (node, npm) = crate::modules::runtime::install::managed_dependency_paths(&i);
            std::fs::create_dir_all(node.parent().unwrap()).unwrap();
            std::fs::copy(supplied_node, &node).unwrap();
            copy_tree(
                supplied_npm.parent().unwrap().parent().unwrap(),
                npm.parent().unwrap().parent().unwrap(),
            );
            let archive = root.path().join("opencodex-1.2.3.tgz");
            tgz(
                &archive,
                b"#!/usr/bin/env node\nconsole.log('opencodex 1.2.3');\n",
            );
            let pin = fixture_pin(&archive);
            let runtime = RuntimeHandle::initialize_managed(&d, &i);
            let process = ProcessContext {
                runtime: runtime.clone(),
                working_directory: d.clone(),
                opencodex_home: h.clone(),
            };
            let runner = SystemNpmRunner::new(Some(home.clone()), d.join("cache/npm"));
            let probe = RealVersionProbe::default();
            let mut installer = RuntimeInstaller::new(
                &d,
                &runner,
                Some(home.clone()),
                &NullProgressSink,
                &probe,
                Some(node.clone()),
            );
            installer.installation_root = Some(i.clone());
            installer
                .install(&InstallRequest {
                    prefix: i.join("runtime/opencodex"),
                    source: InstallSource::Offline {
                        archive: archive.clone(),
                    },
                    allow_scripts: false,
                    npm_path: Some(npm),
                    node_path: Some(node),
                })
                .unwrap();
            runtime.refresh();
            let package = i.join("runtime/opencodex/node_modules/@bitkyc08/opencodex/package.json");
            let mut json: serde_json::Value =
                serde_json::from_slice(&std::fs::read(&package).unwrap()).unwrap();
            json["version"] = "9.9.9".into();
            std::fs::write(package, serde_json::to_vec(&json).unwrap()).unwrap();
            let source = Arc::new(Source {
                path: archive,
                pin,
                receiver: SharedOfflineDrop::default(),
                copies: Mutex::new(Vec::new()),
            });
            let (entered_tx, entered) = mpsc::channel();
            let (release, release_rx) = mpsc::channel();
            let proof = Arc::new(ControlledProof {
                inner: Proof {
                    h,
                    state: Mutex::new(ServiceOwnerProof::Stopped),
                    stops: AtomicUsize::new(0),
                },
                calls: AtomicUsize::new(0),
                block_at: AtomicUsize::new(usize::MAX),
                entered: entered_tx,
                release: Mutex::new(release_rx),
            });
            let context = RecoveryContext::new(runtime, d, home, process).unwrap();
            let mut recovery = SharedRuntimeRecoveryV2::new(
                context,
                SharedRuntimeInstall::new(),
                source.clone(),
                proof.clone(),
            );
            // Keep the fixture gate isolated from concurrent app binding tests.
            Arc::get_mut(&mut recovery.0).unwrap().gate =
                Arc::new(crate::infrastructure::storage_writers::WriterGate::default());
            let log = PathBuf::from(format!(
                "/tmp/ocxd023-runtime-recovery-ipc-native-{}.log",
                uuid::Uuid::new_v4()
            ));
            std::fs::write(
                &log,
                b"synthetic I/D/H/npm6; no daily service/config access\n",
            )
            .unwrap();
            Self {
                root,
                recovery,
                source,
                proof,
                entered,
                release,
                updater: Arc::new(Mutex::new(crate::modules::update::UpdateStatus::pending(
                    "0.2.23",
                ))),
                log,
            }
        }
        fn prepare(&self) -> RuntimeRecoveryV2Status {
            self.recovery
                .prepare_sync(RuntimeRecoveryPrepareRequest {
                    source: RecoverySourceKind::Bundled,
                    staged_archive_id: None,
                })
                .unwrap()
        }
        fn apply_request(preview: &RuntimeRecoveryV2Status) -> RuntimeRecoveryApplyRequest {
            RuntimeRecoveryApplyRequest {
                operation_id: preview.status.preview.operation_id.clone(),
                confirmation_digest: preview.confirmation_digest.clone(),
            }
        }
        fn reopened(&self) -> SharedRuntimeRecoveryV2 {
            SharedRuntimeRecoveryV2::new(
                self.recovery.0.context.clone(),
                self.recovery.0.install.clone(),
                self.source.clone(),
                self.proof.clone(),
            )
        }
        fn note(&self, note: &str) {
            writeln!(
                std::fs::OpenOptions::new()
                    .append(true)
                    .open(&self.log)
                    .unwrap(),
                "{note}"
            )
            .unwrap();
        }
    }

    #[test]
    #[ignore = "isolated native npm6 fixture: set absolute OPENCODEX_TEST_NODE/OPENCODEX_TEST_NPM; run --ignored"]
    fn facade_real_core_preview_refusal_leases_apply_failure_and_reopen() {
        let f = Fixture::new();
        let ctx = &f.recovery.0.context;
        let h_before = snapshot(&ctx.process.opencodex_home);
        let whole = snapshot(f.root.path());
        let preview = f.prepare();
        assert_eq!(
            snapshot(f.root.path()),
            whole,
            "prepare writes no business files"
        );
        let id = preview.status.preview.operation_id.clone();
        let copy = f.source.copies.lock().unwrap().last().unwrap().clone();
        assert_eq!(
            f.recovery.cancel_sync(&id).unwrap().status.phase,
            RuntimeRecoveryPhase::Cancelled
        );
        assert!(!copy.exists());
        assert!(f
            .recovery
            .apply_sync(Fixture::apply_request(&preview), &f.updater)
            .is_err());
        assert!(f
            .reopened()
            .apply_sync(Fixture::apply_request(&preview), &f.updater)
            .is_err());
        assert_eq!(snapshot(f.root.path()), whole);
        f.note(
            "preview read-only; cancel destroyed private copy; restart cannot revive a capability",
        );

        let wrong_confirmation = f.prepare();
        let mut wrong = Fixture::apply_request(&wrong_confirmation);
        wrong.confirmation_digest = "0".repeat(64);
        assert!(
            matches!(f.recovery.apply_sync(wrong, &f.updater), Err(AppError::RuntimeManaged {code, ..}) if code == "recovery_confirmation_required")
        );
        assert!(!wrong_confirmation.status.preview.selected_archive.exists());
        assert_eq!(snapshot(f.root.path()), whole);
        let superseded = f.prepare();
        let replacement = f.prepare();
        assert!(!superseded.status.preview.selected_archive.exists());
        assert_eq!(
            f.recovery
                .status_sync(&superseded.status.preview.operation_id)
                .unwrap()
                .status
                .phase,
            RuntimeRecoveryPhase::Cancelled
        );
        f.recovery
            .cancel_sync(&replacement.status.preview.operation_id)
            .unwrap();

        *f.proof.inner.state.lock().unwrap() = ServiceOwnerProof::Unknown;
        assert!(
            matches!(f.recovery.prepare_sync(RuntimeRecoveryPrepareRequest { source: RecoverySourceKind::Bundled, staged_archive_id: None }), Err(AppError::RuntimeManaged {code, ..}) if code == "recovery_service_unknown")
        );
        assert_eq!(snapshot(f.root.path()), whole);
        assert_eq!(f.proof.inner.stops.load(Ordering::SeqCst), 0);
        assert!(!ctx.data_root.join("backups/runtime-recovery").exists());
        *f.proof.inner.state.lock().unwrap() = ServiceOwnerProof::Stopped;

        let preview = f.prepare();
        *f.proof.inner.state.lock().unwrap() = ServiceOwnerProof::Unknown;
        assert!(
            matches!(f.recovery.apply_sync(Fixture::apply_request(&preview), &f.updater), Err(AppError::RuntimeManaged {code, ..}) if code == "recovery_service_unknown")
        );
        assert_eq!(snapshot(f.root.path()), whole);
        assert!(!preview.status.preview.selected_archive.exists());
        *f.proof.inner.state.lock().unwrap() = ServiceOwnerProof::Stopped;

        let preview = f.prepare();
        f.updater.lock().unwrap().installing = true;
        assert!(
            matches!(f.recovery.apply_sync(Fixture::apply_request(&preview), &f.updater), Err(AppError::RuntimeManaged {code, ..}) if code == "desktop_update_in_progress")
        );
        assert!(!preview.status.preview.selected_archive.exists());
        assert!(!f.recovery.0.install.is_running());
        f.updater.lock().unwrap().installing = false;

        let preview = f.prepare();
        let entry_before = std::fs::read(&preview.status.preview.entry).unwrap();
        std::fs::write(&preview.status.preview.entry, b"changed baseline fixture").unwrap();
        let changed = snapshot(f.root.path());
        assert!(
            matches!(f.recovery.apply_sync(Fixture::apply_request(&preview), &f.updater), Err(AppError::RuntimeManaged {code, ..}) if code == "recovery_snapshot_changed")
        );
        assert_eq!(
            snapshot(f.root.path()),
            changed,
            "baseline refusal precedes backup writes"
        );
        assert!(!preview.status.preview.selected_archive.exists());
        std::fs::write(&preview.status.preview.entry, entry_before).unwrap();

        let preview = f.prepare();
        let normal_installer = f.recovery.0.install.acquire().unwrap();
        assert!(
            matches!(f.recovery.apply_sync(Fixture::apply_request(&preview), &f.updater), Err(AppError::RuntimeManaged {code, ..}) if code == "install_in_progress")
        );
        assert!(!preview.status.preview.selected_archive.exists());
        assert!(f.recovery.0.install.is_running());
        drop(normal_installer);
        let preview = f.prepare();
        let frozen = f.recovery.0.gate.freeze().unwrap().unwrap();
        assert!(f
            .recovery
            .apply_sync(Fixture::apply_request(&preview), &f.updater)
            .is_err());
        assert!(!preview.status.preview.selected_archive.exists());
        drop(frozen);
        f.note("unknown owner and changed baseline refused without stop/backup; shared install and storage gate block recovery");

        let preview = f.prepare();
        let id = preview.status.preview.operation_id.clone();
        let old_prefix = snapshot(&preview.status.preview.prefix);
        // Query 1: explicit owner admission; 2: fresh preview; 3: core
        // admission; 4: after Applying + backup.
        f.proof
            .block_at
            .store(f.proof.calls.load(Ordering::SeqCst) + 4, Ordering::SeqCst);
        let worker = f.recovery.clone();
        let updater = f.updater.clone();
        let request = Fixture::apply_request(&preview);
        let applying = std::thread::spawn(move || worker.apply_sync(request, &updater));
        f.entered.recv_timeout(Duration::from_secs(30)).unwrap();
        assert!(f.recovery.0.install.is_running());
        assert!(f.recovery.0.install.acquire().is_none());
        assert!(f.recovery.0.gate.freeze().unwrap().is_none());
        assert_eq!(
            f.recovery.status_sync(&id).unwrap().status.phase,
            RuntimeRecoveryPhase::Applying
        );
        assert_eq!(
            f.reopened().status_sync(&id).unwrap().status.phase,
            RuntimeRecoveryPhase::Applying
        );
        assert!(f.recovery.cancel_sync(&id).is_err());
        assert!(f
            .recovery
            .apply_sync(Fixture::apply_request(&preview), &f.updater)
            .is_err());
        assert!(preview
            .status
            .preview
            .metadata_backup
            .join("entry.before")
            .exists());
        f.release.send(()).unwrap();
        let done = applying.join().unwrap().unwrap();
        assert_eq!(done.status.phase, RuntimeRecoveryPhase::Succeeded);
        assert_eq!(
            snapshot(&preview.status.preview.retained_prefix),
            old_prefix
        );
        assert!(!preview.status.preview.selected_archive.exists());
        assert!(!f.recovery.0.install.is_running());
        assert_eq!(f.reopened().status_sync(&id).unwrap().status, done.status);
        assert!(f
            .reopened()
            .apply_sync(Fixture::apply_request(&preview), &f.updater)
            .is_err());
        assert_eq!(snapshot(&ctx.process.opencodex_home), h_before);
        assert_eq!(
            protection::status(&ctx.data_root).state,
            protection::ProtectionState::None
        );
        f.note("actual core/npm6 success retained old prefix and metadata; Applying poll + cancel refusal; all leases held; journal reopen read-only; H/key unchanged");

        // A real core failure after Applying; existing backup is never overwritten.
        let preview = f.prepare();
        let id = preview.status.preview.operation_id.clone();
        let program = snapshot(ctx.installation_root());
        std::fs::create_dir_all(&preview.status.preview.metadata_backup).unwrap();
        let sentinel = preview
            .status
            .preview
            .metadata_backup
            .join("foreign-backup-sentinel");
        std::fs::write(&sentinel, b"keep exact backup").unwrap();
        assert!(f
            .recovery
            .apply_sync(Fixture::apply_request(&preview), &f.updater)
            .is_err());
        assert_eq!(
            f.recovery.status_sync(&id).unwrap().status.phase,
            RuntimeRecoveryPhase::Failed
        );
        assert_eq!(
            f.reopened().status_sync(&id).unwrap().status.phase,
            RuntimeRecoveryPhase::Failed
        );
        assert_eq!(snapshot(ctx.installation_root()), program);
        assert_eq!(std::fs::read(sentinel).unwrap(), b"keep exact backup");
        assert_eq!(snapshot(&ctx.process.opencodex_home), h_before);
        assert!(!preview.status.preview.selected_archive.exists());
        assert!(!f.recovery.0.install.is_running());
        f.note("actual failed core journal survives reopen; original I/H/key/backup preserved; owned source cleaned");
        eprintln!("runtime recovery IPC evidence: {}", f.log.display());
    }
}
