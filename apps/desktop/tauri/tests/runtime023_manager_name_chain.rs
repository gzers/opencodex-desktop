//! AC-P22-02: real manager rename -> provisioned artifact -> installed catalog.
//! Explicit complete-bundle acceptance; temporary I/D/H/CODEX_HOME only.
#![cfg(feature = "integration-test")]
use opencodex_desktop_lib::modules::{
    model_config::config_file::v2::{self, ChannelIntentV2, ReadRequestV2, SaveRequestV2},
    runtime::{
        self,
        bundled::{self, ProvisionResult},
    },
};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{Duration, Instant},
};

fn input(primary: &str, legacy: &str) -> std::ffi::OsString {
    std::env::var_os(primary)
        .or_else(|| std::env::var_os(legacy))
        .unwrap_or_else(|| panic!("explicit {primary} required"))
}
fn run(command: &mut Command, output: &Path) {
    let file = std::fs::File::create(output).unwrap();
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(file.try_clone().unwrap()))
        .stderr(Stdio::from(file));
    let mut child = command.spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(65);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() > deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("artifact consumer exceeded deadline");
        }
        std::thread::sleep(Duration::from_millis(25));
    };
    let observed = std::fs::read_to_string(output).unwrap();
    print!("{observed}");
    assert!(
        status.success(),
        "artifact consumer failed; output above retained in test log"
    );
}
#[cfg(unix)]
struct OwnCatalogLock(Vec<PathBuf>);
#[cfg(unix)]
impl Drop for OwnCatalogLock {
    fn drop(&mut self) {
        for path in &self.0 {
            if let Ok(metadata) = std::fs::symlink_metadata(path) {
                if metadata.is_file() && !metadata.file_type().is_symlink() {
                    let _ = std::fs::remove_file(path);
                }
            }
        }
        eprintln!(
            "CLEANUP_OWN_CATALOG_LOCK filesRemaining={}",
            self.0.iter().filter(|p| p.exists()).count()
        );
    }
}
#[test]
#[ignore = "requires explicit pinned full platform bundle and OS network sandbox"]
fn manager_renames_reach_the_same_installed_runtime_and_own_codex_catalog() {
    assert_eq!(
        std::env::consts::OS,
        "macos",
        "this acceptance requires macOS network sandbox; native Windows remains separate"
    );
    let archive = PathBuf::from(input(
        "OCXD_TEST_BUNDLE_ARCHIVE",
        "OCXD_RUNTIME_BUNDLE_ARCHIVE",
    ));
    let descriptor_path = PathBuf::from(input(
        "OCXD_TEST_BUNDLE_DESCRIPTOR",
        "OCXD_RUNTIME_BUNDLE_MANIFEST",
    ));
    let descriptor = std::fs::read(&descriptor_path).unwrap();
    let pin = input(
        "OCXD_TEST_BUNDLE_DESCRIPTOR_SHA256",
        "OCXD_RUNTIME_BUNDLE_MANIFEST_SHA256",
    )
    .into_string()
    .unwrap();
    eprintln!("CHAIN_STAGE verify_pinned_complete_bundle");
    let verified = bundled::verify_bundle(&archive, &descriptor, &pin).unwrap();
    let policy: Value = serde_json::from_slice(include_bytes!(
        "../../../../.github/runtime-platforms-023.json"
    ))
    .unwrap();
    let bundle: Value = serde_json::from_slice(&descriptor).unwrap();
    assert_eq!(bundle["runtimeVersion"], verified.version);
    assert_eq!(bundle["runtimeArchiveSha256"], policy["runtime"]["sha256"]);
    assert_eq!(bundle["nodeVersion"], policy["node"]["version"]);

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let install = root.join("I/program with spaces");
    let d = root.join("D/desktop");
    let h = root.join("H/opencodex");
    let codex = root.join("CODEX_HOME");
    let profile = root.join("profile");
    for p in [
        &install,
        &d,
        &h,
        &codex,
        &profile,
        &root.join("empty-path"),
        &root.join("tmp"),
    ] {
        std::fs::create_dir_all(p).unwrap();
    }
    let unknown = json!({"future":{"nested":["保持",42],"active":false}});
    let untouched = json!({"name":"Unselected","adapter":"openai-chat","authMode":"none","baseUrl":"https://unselected.invalid/v1","liveModels":false,"models":["unselected/raw"],"selectedModels":[],"defaultAliases":false,"futureProvider":unknown});
    let ids = vec!["model-1", "vendor/original-model"];
    let initial = json!({"port":4096,"defaultProvider":"original","defaultModelAliases":false,"futureRoot":unknown,"providers":{"original":{"name":"original","adapter":"openai-chat","authMode":"none","baseUrl":"https://unused.invalid/v1","liveModels":false,"models":ids,"selectedModels":ids,"defaultAliases":false,"futureProvider":unknown},"untouched":untouched}});
    std::fs::write(h.join("config.json"), serde_json::to_vec(&initial).unwrap()).unwrap();
    std::fs::write(
        codex.join("catalog.json"),
        serde_json::to_vec(&json!({"futureCatalog":unknown,"models":[]})).unwrap(),
    )
    .unwrap();
    std::fs::write(
        codex.join("config.toml"),
        format!(
            "model_catalog_json = {}\n",
            serde_json::to_string(&codex.join("catalog.json").to_string_lossy()).unwrap()
        ),
    )
    .unwrap();
    let script = root.join("runtime023-manager-name-chain.mjs");
    std::fs::write(
        &script,
        include_bytes!("../../../../test/smoke/runtime023-manager-name-chain.mjs"),
    )
    .unwrap();
    let sandbox = root.join("offline.sb");
    let (node, npm) = runtime::install::managed_dependency_paths(&install);
    let bun = install.join("runtime/bun/bun");
    // The delivered runtime keys K under trusted /tmp by the exact canonical
    // CODEX_HOME hash, intentionally independent of TMPDIR. Permit only this
    // random test home's lock and SQLite sidecars; never a daily home's lock.
    #[cfg(unix)]
    let (catalog_lock, lock_policy, _lock_cleanup) = {
        use sha2::{Digest, Sha256};
        use std::os::unix::fs::MetadataExt;
        let uid = std::fs::metadata(&root).unwrap().uid();
        let coordinator = std::fs::canonicalize("/tmp")
            .unwrap()
            .join(format!("opencodex-runtime-v1-{uid}"));
        let directory = coordinator.join("catalog-write-locks");
        let lock = directory.join(format!(
            "{:x}.sqlite",
            Sha256::digest(codex.to_str().unwrap().as_bytes())
        ));
        let paths = ["", "-wal", "-shm", "-journal"]
            .map(|suffix| PathBuf::from(format!("{}{suffix}", lock.display())))
            .to_vec();
        for path in &paths {
            assert!(
                std::fs::symlink_metadata(path).is_err(),
                "fresh test lock must not reuse any existing object"
            );
        }
        let literals = paths
            .iter()
            .map(|p| {
                format!(
                    "(literal {})",
                    serde_json::to_string(&p.to_string_lossy()).unwrap()
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        let policy = format!(
            "(allow file-write* {literals})\n(allow file-write-create (literal {}) (literal {}))",
            serde_json::to_string(&coordinator.to_string_lossy()).unwrap(),
            serde_json::to_string(&directory.to_string_lossy()).unwrap()
        );
        (lock, policy, OwnCatalogLock(paths))
    };
    #[cfg(not(unix))]
    let (catalog_lock, lock_policy) = (PathBuf::new(), String::new());
    let deny_homes = std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|p| {
            [".codex", ".opencodex", ".claude"]
                .map(|name| {
                    format!(
                        "(deny file-read* (subpath {}))",
                        serde_json::to_string(&p.join(name).to_string_lossy()).unwrap()
                    )
                })
                .join("\n")
        })
        .unwrap_or_default();
    std::fs::write(&sandbox,format!("(version 1)\n(allow default)\n(deny network*)\n(deny process-exec)\n(allow process-exec (literal {}) (literal {}))\n(deny file-write*)\n(allow file-write* (subpath {}) (literal \"/dev/null\"))\n{deny_homes}\n{lock_policy}\n",serde_json::to_string(&node.to_string_lossy()).unwrap(),serde_json::to_string(&bun.to_string_lossy()).unwrap(),serde_json::to_string(&root.to_string_lossy()).unwrap())+&format!("(deny file-write* (subpath {}))\n",serde_json::to_string(&install.to_string_lossy()).unwrap())).unwrap();
    let mut channel_id = None;
    let mut previous_provider = "original".to_string();
    for (index, label) in ["RenamedEnglish", "中文渠道", "Team Space", "中文 / Team"]
        .into_iter()
        .enumerate()
    {
        let snapshot = v2::snapshot(&d, &h).unwrap();
        let begun = v2::begin_draft(
            &d,
            &h,
            ReadRequestV2 {
                expected_source: snapshot.source_digest,
                expected_library: snapshot.library_digest,
                expected_draft: snapshot.draft.as_ref().map(|draft| draft.base.clone()),
            },
        )
        .unwrap();
        let draft = begun.draft.unwrap();
        let channel = draft
            .policy
            .channels
            .iter()
            .find(|c| c.connection.provider_id == previous_provider)
            .unwrap();
        if let Some(id) = &channel_id {
            assert_eq!(id, &channel.id);
        } else {
            channel_id = Some(channel.id.clone());
        }
        let before: Value =
            serde_json::from_slice(&std::fs::read(h.join("config.json")).unwrap()).unwrap();
        let saved = v2::save(
            &d,
            &h,
            SaveRequestV2 {
                expected: draft.base.clone(),
                edits: vec![],
                channel_intents: vec![ChannelIntentV2::Rename {
                    channel_id: channel.id.clone(),
                    name: label.into(),
                }],
                scope_channel_id: Some(channel.id.clone()),
            },
        )
        .unwrap();
        let channel = saved
            .saved_policy
            .channels
            .iter()
            .find(|c| Some(&c.id) == channel_id.as_ref())
            .unwrap();
        let provider = &channel.connection.provider_id;
        assert!(provider
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_')));
        assert_eq!(
            channel
                .models
                .iter()
                .map(|m| m.raw_model_id.as_str())
                .collect::<Vec<_>>(),
            ids
        );
        let actual: Value =
            serde_json::from_slice(&std::fs::read(h.join("config.json")).unwrap()).unwrap();
        assert_eq!(
            actual["providers"]["untouched"],
            before["providers"]["untouched"]
        );
        assert_eq!(actual["futureRoot"], unknown);
        assert_eq!(actual["providers"][provider]["futureProvider"], unknown);
        if provider != &previous_provider {
            assert!(actual["providers"].get(&previous_provider).is_none());
        }
        eprintln!("CHAIN_STAGE manager_save label={label} provider={provider}");
        if index == 0 {
            eprintln!("CHAIN_STAGE provision_complete_bundle");
            assert!(matches!(
                bundled::provision(&install, &d, &archive, &descriptor, &pin).unwrap(),
                ProvisionResult::Installed { .. }
            ));
            runtime::install::validate_managed_installation(&install, Some(&node)).unwrap();
        }
        eprintln!("CHAIN_STAGE installed_consumer label={label}");
        assert!(node.is_file() && npm.is_file() && bun.is_file());
        let input = root.join(format!("step-{index}.json"));
        std::fs::write(&input,serde_json::to_vec(&json!({"root":root,"install":install,"catalogLock":catalog_lock,"label":label,"channelId":channel.id,"providerId":provider,"rawIds":ids,"unknown":unknown,"untouched":actual["providers"]["untouched"],"nodeVersion":bundle["nodeVersion"],"npmVersion":policy["node"]["npmVersion"],"bunVersion":bundle["bunVersion"],"runtimeVersion":verified.version})).unwrap()).unwrap();
        let manager_bytes = std::fs::read(h.join("config.json")).unwrap();
        let mut command = Command::new("/usr/bin/sandbox-exec");
        command
            .arg("-f")
            .arg(&sandbox)
            .arg(&node)
            .arg(&script)
            .arg("node")
            .arg(&input)
            .env_clear()
            .env("HOME", &profile)
            .env("USERPROFILE", &profile)
            .env("OPENCODEX_HOME", &h)
            .env("CODEX_HOME", &codex)
            .env("CLAUDE_CONFIG_DIR", root.join("claude"))
            .env("PATH", root.join("empty-path"))
            .env("TMPDIR", root.join("tmp"))
            .env("TEMP", root.join("tmp"))
            .env("XDG_CONFIG_HOME", profile.join(".config"))
            .env("XDG_CACHE_HOME", profile.join(".cache"))
            .env("npm_config_userconfig", root.join("empty.npmrc"))
            .env("npm_config_globalconfig", root.join("global.npmrc"))
            .env("npm_config_update_notifier", "false")
            .env("OCX_NO_DESKTOP_HANDOFF", "1")
            .env("OCX_TEST_HOME_GUARD", "1")
            .env("CODEX_CLI_PATH", root.join("empty-path/no-codex"))
            .env("CI", "1")
            .current_dir(&h);
        run(&mut command, &root.join(format!("step-{index}.log")));
        assert_eq!(manager_bytes, std::fs::read(h.join("config.json")).unwrap());
        previous_provider = provider.clone();
    }
    println!(
        "MANAGER_NAME_CHAIN_RESULT {}",
        json!({"labels":4,"runtimeVersion":verified.version,"descriptorSha256":pin,"archiveSha256":bundle["archiveSha256"],"network":"OS sandbox deny","temporaryRoots":"I/D/H/CODEX_HOME in one TempDir","dailyConfig":"untouched"})
    );
}
