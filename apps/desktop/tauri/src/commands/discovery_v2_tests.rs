#[cfg(unix)]
mod unix {
    use super::super::*;
    use serde_json::json;
    use std::os::unix::fs::PermissionsExt;
    struct Fixture {
        _temp: tempfile::TempDir,
        i: PathBuf,
        d: PathBuf,
        node: PathBuf,
        npm: PathBuf,
        script: PathBuf,
        entry: PathBuf,
    }
    impl Fixture {
        fn new() -> Self {
            let temp = tempfile::tempdir().unwrap();
            let base = temp.path().canonicalize().unwrap();
            let i = base.join("I/Desktop 中文 space");
            let d = base.join("D/desktop");
            std::fs::create_dir_all(d.join("manager-state")).unwrap();
            let (node, npm) = install::managed_dependency_paths(&i);
            let script = i
                .join(runtime::MANAGED_PREFIX_RELATIVE)
                .join(install::PACKAGE_SUBPATH)
                .join("bin/ocx.mjs");
            let entry = i.join(runtime::MANAGED_ENTRY_RELATIVE);
            for file in [&node, &npm, &script, &entry] {
                std::fs::create_dir_all(file.parent().unwrap()).unwrap();
            }
            std::fs::write(&node,"#!/bin/sh\n[ -z \"$PATH\" ] || exit 13\n[ -z \"$DISCOVERY_POISON\" ] || exit 14\n[ \"$HOME\" = \"$OPENCODEX_HOME\" ] || exit 15\n[ \"$1\" = '--version' ] && { echo 'v26.8.1'; exit 0; }\nexec /bin/sh \"$1\" \"$2\"\n").unwrap();
            std::fs::set_permissions(&node, std::fs::Permissions::from_mode(0o755)).unwrap();
            std::fs::write(&npm, "echo '11.19.0'\n").unwrap();
            std::fs::set_permissions(&npm, std::fs::Permissions::from_mode(0o644)).unwrap();
            std::fs::write(&script, "echo 'opencodex 2.97.0-desktop023.names.1'\n").unwrap();
            std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
            let f = Self {
                _temp: temp,
                i,
                d,
                node,
                npm,
                script,
                entry,
            };
            f.seal();
            f
        }
        fn seal(&self) {
            let package = self
                .i
                .join(runtime::MANAGED_PREFIX_RELATIVE)
                .join(install::PACKAGE_SUBPATH)
                .join("package.json");
            std::fs::write(&package,serde_json::to_vec(&json!({"name":runtime::OFFICIAL_PACKAGE,"version":"2.97.0-desktop023.names.1","bin":{"ocx":"bin/ocx.mjs"}})).unwrap()).unwrap();
            std::fs::write(
                &self.entry,
                install::entry_script(Some(&self.node), &self.script),
            )
            .unwrap();
            std::fs::set_permissions(&self.entry, std::fs::Permissions::from_mode(0o755)).unwrap();
            let hash =
                |p: &Path| crate::infrastructure::hash::sha256_hex(&std::fs::read(p).unwrap());
            let manifest = json!({"activation":{"attempt_id":uuid::Uuid::new_v4().to_string(),"bin":"bin/ocx.mjs","node":self.node,"node_sha256":hash(&self.node),"entry_sha256":hash(&self.entry),"package_sha256":hash(&package),"bin_sha256":hash(&self.script)},"package":runtime::OFFICIAL_PACKAGE,"version":"2.97.0-desktop023.names.1","tarball_sha256":"a".repeat(64),"installed_at":"2026-11-07T00:00:00Z","npm_path":self.npm,"scripts_enabled":false,"source":"offline"});
            std::fs::write(
                self.i
                    .join(runtime::MANAGED_PREFIX_RELATIVE)
                    .join(runtime::MANIFEST_FILENAME),
                serde_json::to_vec(&manifest).unwrap(),
            )
            .unwrap();
        }
        fn handle(&self) -> Arc<RuntimeHandle> {
            let h = RuntimeHandle::initialize_managed(&self.d, &self.i);
            assert_eq!(h.kind(), RuntimeSourceKind::Managed);
            h
        }
    }
    #[tokio::test]
    async fn null_uses_actual_managed_proof_fixed_paths_and_real_version_processes() {
        let f = Fixture::new();
        let handle = f.handle();
        let store = std::fs::read(handle.store().path()).unwrap();
        let report = discover(None, handle.clone()).await.unwrap();
        assert_eq!(report.gate, EnvironmentGate::Ready);
        assert_eq!(
            report.node.path,
            Some(f.node.to_string_lossy().into_owned())
        );
        assert_eq!(report.npm.path, Some(f.npm.to_string_lossy().into_owned()));
        assert_eq!(
            report.ocx.path,
            Some(f.entry.to_string_lossy().into_owned())
        );
        assert_eq!(report.node.version.as_deref(), Some("26.8.1"));
        assert_eq!(report.npm.version.as_deref(), Some("11.19.0"));
        assert_eq!(
            report.ocx.version.as_deref(),
            Some("2.97.0-desktop023.names.1")
        );
        assert!(!report.brew_found);
        assert!(!report.short_circuited);
        assert_eq!(std::fs::read(handle.store().path()).unwrap(), store);
        let empty = DiscoveryRequest {
            node_path: None,
            npm_path: None,
            ocx_path: None,
        };
        assert_eq!(discover(Some(empty), handle).await.unwrap(), report);
    }
    #[test]
    fn partial_and_relative_explicit_paths_never_fall_back_to_defaults() {
        for request in [
            DiscoveryRequest {
                node_path: Some("/fixture/node".into()),
                npm_path: None,
                ocx_path: None,
            },
            DiscoveryRequest {
                node_path: Some("relative".into()),
                npm_path: Some("/fixture/npm".into()),
                ocx_path: Some("/fixture/ocx".into()),
            },
            DiscoveryRequest {
                node_path: Some("".into()),
                npm_path: Some("/fixture/npm".into()),
                ocx_path: Some("/fixture/ocx".into()),
            },
            DiscoveryRequest {
                node_path: Some("/fixture/../node".into()),
                npm_path: Some("/fixture/npm".into()),
                ocx_path: Some("/fixture/ocx".into()),
            },
        ] {
            assert!(
                matches!(explicit(Some(request)),Err(AppError::RuntimeManaged{code,..})if code=="discovery_request_invalid")
            );
        }
    }
    #[tokio::test]
    async fn explicit_complete_tools_are_readonly_reports_and_cannot_rebind_the_managed_handle() {
        let f = Fixture::new();
        let handle = f.handle();
        let current = handle.current();
        let store = std::fs::read(handle.store().path()).unwrap();
        let tool = f._temp.path().join("isolated-ocx");
        std::fs::write(&tool, "#!/bin/sh\necho 'opencodex 1.2.3'\n").unwrap();
        std::fs::set_permissions(&tool, std::fs::Permissions::from_mode(0o755)).unwrap();
        let request = DiscoveryRequest {
            node_path: Some(f.node.to_string_lossy().into_owned()),
            npm_path: Some(f.npm.to_string_lossy().into_owned()),
            ocx_path: Some(tool.to_string_lossy().into_owned()),
        };
        let report = discover(Some(request), handle.clone()).await.unwrap();
        assert_eq!(report.gate, EnvironmentGate::Ready);
        assert_eq!(report.ocx.version.as_deref(), Some("1.2.3"));
        assert_eq!(handle.current().path, current.path);
        assert_eq!(handle.kind(), current.kind);
        assert_eq!(std::fs::read(handle.store().path()).unwrap(), store);
    }
    #[tokio::test]
    async fn actual_probe_failure_wrong_version_and_existence_only_are_never_ready() {
        for mode in 0..3 {
            let f = Fixture::new();
            match mode {
                0 => std::fs::write(&f.npm, "exit 8\n").unwrap(),
                1 => std::fs::write(&f.npm, "echo '10.0.0'\n").unwrap(),
                _ => {
                    std::fs::write(&f.script, "exit 4\n").unwrap();
                    f.seal();
                }
            }
            let result = discover(None, f.handle()).await;
            match mode {
                0 => assert!(
                    matches!(result,Err(AppError::RuntimeManaged{code,..}) if code=="discovery_npm_probe_failed")
                ),
                1 => assert!(
                    matches!(result,Err(AppError::RuntimeManaged{code,..})if code=="discovery_runtime_version_mismatch")
                ),
                _ => assert!(
                    matches!(result,Err(AppError::RuntimeManaged{code,..}) if code=="discovery_ocx_probe_failed")
                ),
            }
        }
    }
    #[tokio::test]
    async fn corrupt_missing_pending_and_symlink_sources_cannot_be_ready() {
        let f = Fixture::new();
        let handle = f.handle();
        std::fs::write(&f.node, "changed").unwrap();
        assert!(discover(None, handle).await.is_err());
        let f = Fixture::new();
        let handle = f.handle();
        std::fs::write(f.i.join(".runtime-bundle-pending.json"), "retained pending").unwrap();
        assert!(
            matches!(discover(None,handle).await,Err(AppError::RuntimeManaged{code,..})if code=="discovery_runtime_pending")
        );
        let f = Fixture::new();
        let handle = f.handle();
        std::fs::remove_file(&f.entry).unwrap();
        assert!(discover(None, handle).await.is_err());
        let f = Fixture::new();
        let handle = f.handle();
        let original = f._temp.path().join("original-node");
        std::fs::rename(&f.node, &original).unwrap();
        std::os::unix::fs::symlink(&original, &f.node).unwrap();
        assert!(discover(None, handle).await.is_err());
        let t = tempfile::tempdir().unwrap();
        let missing = RuntimeHandle::initialize_managed(&t.path().join("D"), &t.path().join("I"));
        assert!(
            matches!(discover(None,missing).await,Err(AppError::RuntimeManaged{code,..})if code=="discovery_runtime_unresolved")
        );
    }
    #[tokio::test]
    async fn probe_deadline_and_output_limit_fail_without_hanging_or_invented_version() {
        let temp = tempfile::tempdir().unwrap();
        let script = temp.path().join("tool");
        std::fs::write(&script, "#!/bin/sh\n/bin/sleep 3\n").unwrap();
        std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        let start = std::time::Instant::now();
        assert!(probe(
            &script,
            &["--version".into()],
            temp.path(),
            Duration::from_millis(50)
        )
        .await
        .is_none());
        assert!(start.elapsed() < Duration::from_secs(2));
        std::fs::write(&script,"#!/bin/sh\ni=0; while [ $i -lt 5000 ]; do printf x; i=$((i+1)); done; echo ' 26.8.1'\n").unwrap();
        assert!(probe(&script, &[], temp.path(), Duration::from_secs(2))
            .await
            .is_none());
    }
    #[test]
    fn poisoned_path_and_old_home_are_not_used_by_managed_default() {
        if std::env::var_os("DISCOVERY_TEST_CHILD").is_some() {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                let f = Fixture::new();
                let result = discover(None, f.handle()).await.unwrap();
                assert_eq!(result.gate, EnvironmentGate::Ready);
                assert!(!result.brew_found);
            });
            return;
        }
        let temp = tempfile::tempdir().unwrap();
        let home = temp.path().join("old-home");
        for directory in [".nvm/bin", ".volta/bin", ".brew/bin", ".local/bin"] {
            let dir = home.join(directory);
            std::fs::create_dir_all(&dir).unwrap();
            for tool in ["node", "npm", "ocx", "brew"] {
                let path = dir.join(tool);
                std::fs::write(&path, "#!/bin/sh\nexit 91\n").unwrap();
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
            }
        }
        let output=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","commands::discovery::tests::unix::poisoned_path_and_old_home_are_not_used_by_managed_default","--nocapture"]).env("DISCOVERY_TEST_CHILD","1").env("DISCOVERY_POISON","must-not-reach-probes").env("HOME",&home).env("PATH",home.join(".nvm/bin")).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    }
    #[tokio::test]
    #[ignore = "requires explicitly supplied pinned complete platform runtime bundle"]
    async fn complete_bundle_reports_its_actual_bundled_node_npm_and_ocx_as_ready() {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let d = base.join("D/desktop");
        std::fs::create_dir_all(&d).unwrap();
        let i = if let Some(root) = std::env::var_os("OCXD_TEST_MANAGED_INSTALLATION_ROOT") {
            let root = PathBuf::from(root);
            assert!(root.is_absolute());
            let root = root.canonicalize().unwrap();
            assert!(root.join(runtime::MANAGED_ENTRY_RELATIVE).is_file());
            root
        } else {
            let archive = PathBuf::from(
                std::env::var_os("OCXD_TEST_BUNDLE_ARCHIVE").expect("explicit archive"),
            );
            let descriptor = std::fs::read(
                std::env::var_os("OCXD_TEST_BUNDLE_DESCRIPTOR").expect("explicit descriptor"),
            )
            .unwrap();
            let pin = std::env::var("OCXD_TEST_BUNDLE_DESCRIPTOR_SHA256")
                .expect("trusted descriptor hash");
            let i = base.join("I/Desktop");
            std::fs::create_dir_all(&i).unwrap();
            let result = runtime::bundled::provision(&i, &d, &archive, &descriptor, &pin).unwrap();
            assert!(matches!(
                result,
                runtime::bundled::ProvisionResult::Installed { .. }
            ));
            i
        };
        let handle = RuntimeHandle::initialize_managed(&d, &i);
        let saved = std::fs::read(handle.store().path()).unwrap();
        let report = discover(None, handle.clone()).await.unwrap();
        assert_eq!(report.gate, EnvironmentGate::Ready);
        let (node, npm) = install::managed_dependency_paths(&i);
        assert_eq!(report.node.path, Some(node.to_string_lossy().into_owned()));
        assert_eq!(report.npm.path, Some(npm.to_string_lossy().into_owned()));
        assert_eq!(
            report.ocx.path,
            Some(
                i.join(runtime::MANAGED_ENTRY_RELATIVE)
                    .to_string_lossy()
                    .into_owned()
            )
        );
        assert_eq!(report.node.version.as_deref(), Some("26.8.1"));
        assert_eq!(report.npm.version.as_deref(), Some("11.19.0"));
        assert_eq!(
            report.ocx.version.as_deref(),
            Some("2.97.0-desktop023.names.1")
        );
        assert!(!report.brew_found);
        assert_eq!(saved, std::fs::read(handle.store().path()).unwrap());
        println!(
            "ACTUAL_MANAGED_DISCOVERY {}",
            serde_json::to_string(&report).unwrap()
        );
    }
}
