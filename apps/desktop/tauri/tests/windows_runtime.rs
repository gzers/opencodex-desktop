#![cfg(windows)]
//! 使用真实 Node 和隔离脚本验证 Windows batch、环境与退出码，而非仅检查字符串。

use opencodex_desktop_lib::infrastructure::{
    official_cli_source::OfficialDoctorSource, official_version_source::OfficialCliVersionSource,
    platform, process_runner::ControlledProcessRunner, runtime_executable::FixedRuntimeExecutable,
    status_source::OfficialStatusSource,
};
use opencodex_desktop_lib::modules::{
    about::OfficialVersionSource,
    doctor::DoctorSource,
    process::{EnvironmentPolicy, LifecycleAction, LifecycleResult, ProcessCommand, ProcessRunner},
    runtime::{
        archive,
        install::{
            CancelFlag, InstallProgress, InstallProgressSink, InstallRequest, InstallSource,
            NpmInstallRequest, NpmOutcome, NpmPackRequest, NpmRunner, RealVersionProbe,
            RuntimeInstaller, SystemNpmRunner, VersionProbe,
        },
    },
    status::StatusSource,
};
use std::path::{Path, PathBuf};

fn node() -> PathBuf {
    let path = std::env::var_os("OPENCODEX_TEST_NODE")
        .map(PathBuf::from)
        .expect("Windows runtime tests require OPENCODEX_TEST_NODE (absolute node.exe path)");
    assert!(path.is_absolute() && path.is_file(), "invalid test Node");
    path
}

#[test]
fn windows_sandbox_home_discovery_and_pipe_use_the_same_explicit_identity() {
    let fixture = tempfile::tempdir().unwrap();
    let sandbox = fixture.path().join("sandbox");
    let daily = fixture.path().join("daily");
    std::fs::create_dir_all(&sandbox).unwrap();
    std::fs::create_dir_all(daily.join(".local/bin")).unwrap();
    std::fs::write(daily.join(".local/bin/ocx.cmd"), b"daily launcher fixture").unwrap();
    for (root, expected) in [
        (Some(sandbox.as_os_str()), "isolated"),
        (Some(std::ffi::OsStr::new("relative")), "invalid"),
        (None, "invalid"),
    ] {
        let mut child = std::process::Command::new(std::env::current_exe().unwrap());
        child.args([
            "--exact",
            "windows_explicit_sandbox_child",
            "--ignored",
            "--nocapture",
        ]);
        child
            .env("OPENCODEX_SANDBOX", "1")
            .env("OPENCODEX_SANDBOX_TEST_EXPECT", expected)
            .env("HOME", &daily)
            .env("USERPROFILE", &daily)
            .env("NVM_SYMLINK", daily.join(".local/bin"))
            .env("NODE_HOME", daily.join(".local/bin"))
            .env("NVM_HOME", daily.join(".local/bin"));
        match root {
            Some(root) => {
                child.env("OPENCODEX_SANDBOX_ROOT", root);
            }
            None => {
                child.env_remove("OPENCODEX_SANDBOX_ROOT");
            }
        }
        let output = child.output().unwrap();
        assert!(
            output.status.success(),
            "child rejected {expected}: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
#[ignore = "child entry explicitly executed by the isolated-environment parent"]
fn windows_explicit_sandbox_child() {
    use opencodex_desktop_lib::infrastructure::discovery_paths;
    use opencodex_desktop_lib::modules::{ipc::endpoint, test_sandbox};
    assert!(test_sandbox::enabled());
    if std::env::var("OPENCODEX_SANDBOX_TEST_EXPECT").unwrap() == "invalid" {
        assert!(test_sandbox::resolve_data_root(&std::env::temp_dir()).is_err());
        assert!(platform::home_dir().is_none());
        assert!(discovery_paths::default_paths().is_none());
        assert!(endpoint::pipe_name().is_err());
        return;
    }
    let root = test_sandbox::sandbox_root().unwrap();
    assert_eq!(
        test_sandbox::resolve_data_root(&std::env::temp_dir()).unwrap(),
        root
    );
    let home = platform::home_dir().unwrap();
    assert_eq!(home, root.join("home"));
    let paths = discovery_paths::default_paths().unwrap();
    assert!(paths.node.starts_with(&home));
    assert!(paths.npm.starts_with(&home));
    assert!(paths.ocx.starts_with(&home));
    assert!(!paths.ocx.exists());
    assert!(discovery_paths::runtime_candidates(&home)
        .iter()
        .all(|path| path.starts_with(&home)));
    assert!(endpoint::pipe_name()
        .unwrap()
        .starts_with(r"\\.\pipe\OpenCodeXDesktop-Sandbox-"));
    // The resolved sandbox HOME must not be mistaken for the daily profile
    // when child processes receive APPDATA/LOCALAPPDATA.
    let mut command = std::process::Command::new(node());
    command.env_clear();
    platform::apply_user_environment(&mut command, Some(home.as_os_str()));
    command.args(["-e", "console.log(JSON.stringify(process.env))"]);
    let output = command.output().unwrap();
    assert!(output.status.success());
    let environment: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        environment["APPDATA"],
        home.join("AppData/Roaming").to_string_lossy().as_ref()
    );
    assert_eq!(
        environment["LOCALAPPDATA"],
        home.join("AppData/Local").to_string_lossy().as_ref()
    );
    assert_eq!(environment["HOME"], home.to_string_lossy().as_ref());
    assert_eq!(environment["USERPROFILE"], home.to_string_lossy().as_ref());
    assert!(environment.get("OPENCODEX_TEST_NODE").is_none());
    assert!(environment.get("NVM_SYMLINK").is_none());
}

fn write_launcher(root: &Path, script: &str) -> PathBuf {
    let js = root.join("入口 script.js");
    std::fs::write(&js, script).unwrap();
    let entry = root.join("ocx.cmd");
    std::fs::write(&entry, format!(
        "@echo off\r\n\"%SystemRoot%\\System32\\chcp.com\" 65001 >nul\r\nsetlocal DisableDelayedExpansion\r\n\"{}\" \"{}\" %*\r\nexit /b %errorlevel%\r\n",
        node().display(), js.display(),
    )).unwrap();
    entry
}

fn environment(root: &Path) -> EnvironmentPolicy {
    EnvironmentPolicy {
        home: Some(root.as_os_str().to_owned()),
        path: Some(platform::controlled_path(
            node().parent().map(Path::to_path_buf),
            None,
        )),
        opencodex_home: root.to_owned(),
        ..Default::default()
    }
}

#[test]
fn windows_user_environment_is_explicit_and_does_not_inherit_credentials() {
    let root = tempfile::tempdir().unwrap();
    let mut command = std::process::Command::new(node());
    command.env_clear();
    platform::apply_user_environment(&mut command, Some(root.path().as_os_str()));
    command.args(["-e", "console.log(JSON.stringify(process.env))"]);
    let output = command.output().unwrap();
    assert!(output.status.success());
    let env: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(env["USERPROFILE"], root.path().to_string_lossy().as_ref());
    assert_eq!(env["HOME"], root.path().to_string_lossy().as_ref());
    assert!(env["SystemRoot"].is_string());
    assert!(env["APPDATA"]
        .as_str()
        .unwrap()
        .starts_with(root.path().to_str().unwrap()));
    assert!(env.get("OPENCODEX_TEST_NODE").is_none());
    assert!(env.get("HTTPS_PROXY").is_none());
}

#[test]
fn windows_cmd_sources_support_spaces_unicode_and_official_output() {
    let root = tempfile::Builder::new()
        .prefix("ocx 测试 with spaces ")
        .tempdir()
        .unwrap();
    let entry = write_launcher(
        root.path(),
        r#"
if (process.env.USERPROFILE !== process.env.HOME || !process.env.SystemRoot) process.exit(9);
switch (process.argv[2]) {
case '--version': console.log('OpenCodex v2.77.0'); break;
case 'doctor': console.log('doctor fixture'); break;
case 'status': console.log(JSON.stringify({status:'stopped'})); break;
default: process.exit(7);
}
"#,
    );
    let runtime = FixedRuntimeExecutable::resolved(&entry);
    let env = environment(root.path());
    let version = OfficialCliVersionSource::new(runtime.clone(), root.path(), env.clone())
        .run()
        .unwrap();
    assert!(String::from_utf8(version.stdout)
        .unwrap()
        .contains("2.77.0"));
    let doctor = OfficialDoctorSource::new(runtime.clone(), root.path(), env.clone())
        .run()
        .unwrap();
    assert!(String::from_utf8(doctor.stdout)
        .unwrap()
        .contains("doctor fixture"));
    let source = OfficialStatusSource::new(runtime, root.path(), env);
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let status = runtime.block_on(async { source.fetch() }).unwrap();
    assert_eq!(status["status"], "stopped");
}

#[test]
fn windows_activation_waits_for_a_short_lived_directory_handle() {
    use std::os::windows::fs::OpenOptionsExt;
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("temporary");
    let target = root.path().join("activated");
    std::fs::create_dir(&source).unwrap();
    let handle = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(0x0200_0000)
        .share_mode(0x1 | 0x2)
        .open(&source)
        .unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(150));
        drop(handle);
    });
    platform::rename(&source, &target).unwrap();
    release.join().unwrap();
    assert!(target.is_dir() && !source.exists());
}

#[test]
fn windows_stop_observes_actual_zero_and_nonzero_exit_codes() {
    for (code, expected) in [(0, LifecycleResult::Stopped), (7, LifecycleResult::Failed)] {
        let root = tempfile::Builder::new()
            .prefix("ocx stop 中文 ")
            .tempdir()
            .unwrap();
        let entry = write_launcher(root.path(), &format!("process.exit({code});"));
        let command = ProcessCommand {
            action: LifecycleAction::Stop,
            executable: entry,
            working_directory: root.path().to_owned(),
            environment: environment(root.path()),
        };
        assert_eq!(
            ControlledProcessRunner::new(root.path())
                .execute(&command)
                .unwrap(),
            expected
        );
    }
}

#[test]
fn windows_version_probe_executes_node_script_in_a_spaced_path() {
    let root = tempfile::Builder::new()
        .prefix("ocx probe 中文 ")
        .tempdir()
        .unwrap();
    let js = root.path().join("入口.js");
    std::fs::write(&js, "console.log('2.77.0');").unwrap();
    assert_eq!(
        RealVersionProbe::default()
            .probe(Some(&node()), &js)
            .as_deref(),
        Some("2.77.0")
    );
}

#[test]
fn windows_npm_receives_safe_paths_without_retargeting_validated_storage() {
    use opencodex_desktop_lib::modules::runtime::install::NpmPackRequest;
    let root = tempfile::Builder::new()
        .prefix("npm canonical 中文 ")
        .tempdir()
        .unwrap();
    let evidence = root.path().join("canonical-arguments.json");
    let script = format!(
        r#"const fs=require('fs'),p=require('path'),argv=process.argv.slice(2);
const option=argv[0]==='pack'?'--pack-destination':'--prefix';
const target=argv[argv.indexOf(option)+1],cache=process.env.npm_config_cache;
if(target.startsWith('\\\\?\\')||cache.startsWith('\\\\?\\'))process.exit(9);
fs.mkdirSync(target,{{recursive:true}});
if(argv[0]==='pack')fs.writeFileSync(p.join(target,'fixture.tgz'),'bounded fixture');
fs.writeFileSync({},JSON.stringify({{argv,cache,cwd:process.cwd(),targetReal:fs.realpathSync(target),cacheReal:fs.realpathSync(cache)}}));"#,
        serde_json::to_string(&evidence.to_string_lossy()).unwrap()
    );
    let npm = write_launcher(root.path(), &script);
    let cache = root.path().join("cache/npm");
    let prefix = root.path().join("安装 prefix");
    let download = root.path().join("下载");
    for directory in [&cache, &prefix, &download] {
        std::fs::create_dir_all(directory).unwrap();
    }
    let canonical_prefix = prefix.canonicalize().unwrap();
    let canonical_download = download.canonicalize().unwrap();
    let canonical_cache = cache.canonicalize().unwrap();
    assert!(canonical_prefix.to_string_lossy().starts_with(r"\\?\"));
    let runner = SystemNpmRunner::new(Some(root.path().to_owned()), canonical_cache.clone());
    runner
        .install_package(
            &NpmInstallRequest {
                npm_path: npm.clone(),
                node_dir: node().parent().map(Path::to_path_buf),
                prefix: canonical_prefix.clone(),
                version: "2.77.0".into(),
                proxy: None,
                scripts_enabled: false,
                offline: true,
            },
            &Sink,
            &CancelFlag::new(),
        )
        .unwrap();
    let observed: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&evidence).unwrap()).unwrap();
    assert_eq!(
        PathBuf::from(observed["targetReal"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        canonical_prefix
    );
    assert_eq!(
        PathBuf::from(observed["cacheReal"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        canonical_cache
    );
    assert_eq!(
        PathBuf::from(observed["cwd"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        canonical_prefix
    );
    let archive = runner
        .pack(
            &NpmPackRequest {
                npm_path: npm,
                node_dir: node().parent().map(Path::to_path_buf),
                work_dir: canonical_download.clone(),
                version: "2.77.0".into(),
                proxy: None,
            },
            &Sink,
            &CancelFlag::new(),
        )
        .unwrap();
    assert_eq!(archive, canonical_download.join("fixture.tgz"));
    let observed: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&evidence).unwrap()).unwrap();
    assert_eq!(
        PathBuf::from(observed["cwd"].as_str().unwrap())
            .canonicalize()
            .unwrap(),
        canonical_download
    );
    assert_eq!(
        dunce::simplified(Path::new(r"\\?\C:\CON")),
        Path::new(r"\\?\C:\CON")
    );
    assert_eq!(
        dunce::simplified(Path::new(r"\\?\C:\directory\trailing.")),
        Path::new(r"\\?\C:\directory\trailing.")
    );
}

struct Sink;
impl InstallProgressSink for Sink {
    fn emit(&self, _: InstallProgress) {}
}

struct RuntimeFixtureNpm {
    archive: PathBuf,
}

impl NpmRunner for RuntimeFixtureNpm {
    fn pack(
        &self,
        _request: &NpmPackRequest,
        _sink: &dyn InstallProgressSink,
        _cancel: &CancelFlag,
    ) -> Result<PathBuf, opencodex_desktop_lib::modules::runtime::install::InstallError> {
        Ok(self.archive.clone())
    }

    fn install_package(
        &self,
        request: &NpmInstallRequest,
        _sink: &dyn InstallProgressSink,
        _cancel: &CancelFlag,
    ) -> Result<NpmOutcome, opencodex_desktop_lib::modules::runtime::install::InstallError> {
        archive::extract(
            &self.archive,
            None,
            &request.prefix.join("node_modules/@bitkyc08/opencodex"),
        )?;
        std::fs::write(request.prefix.join("package-lock.json"), b"{}").map_err(|error| {
            opencodex_desktop_lib::modules::runtime::install::InstallError::FileSystem {
                operation: "fixture package lock".into(),
                detail: error.to_string(),
            }
        })?;
        Ok(NpmOutcome { exit_code: 0 })
    }
}

struct RuntimeFixtureProbe;
impl VersionProbe for RuntimeFixtureProbe {
    fn probe(&self, _node: Option<&Path>, _script: &Path) -> Option<String> {
        Some("0.3.1".to_string())
    }
}

fn runtime_fixture_archive(root: &Path) -> PathBuf {
    use flate2::{write::GzEncoder, Compression};
    let archive = root.join("opencodex-0.3.1.tgz");
    let file = std::fs::File::create(&archive).unwrap();
    let mut tar = tar::Builder::new(GzEncoder::new(file, Compression::fast()));
    let package = br#"{"name":"@bitkyc08/opencodex","version":"0.3.1","bin":{"ocx":"bin/ocx.js"}}"#;
    let script = b"console.log('0.3.1');";
    for (path, bytes, mode) in [
        ("package/package.json", package.as_slice(), 0o644),
        ("package/bin/ocx.js", script.as_slice(), 0o755),
    ] {
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(mode);
        header.set_cksum();
        tar.append_data(&mut header, path, bytes).unwrap();
    }
    tar.into_inner().unwrap().finish().unwrap();
    archive
}

#[test]
fn windows_managed_install_rejects_unowned_targets_and_restores_old_package_on_entry_failure() {
    let root = tempfile::tempdir().unwrap();
    let data_root = root.path().join("data root 中文");
    let prefix = data_root.join("runtime/opencodex");
    let archive = runtime_fixture_archive(root.path());
    let npm = RuntimeFixtureNpm {
        archive: archive.clone(),
    };
    let installer = |prefix: PathBuf| {
        let request = InstallRequest {
            prefix,
            source: InstallSource::Offline {
                archive: archive.clone(),
            },
            allow_scripts: false,
            npm_path: Some(node()),
            node_path: Some(node()),
        };
        RuntimeInstaller::new(
            &data_root,
            &npm,
            Some(root.path().to_path_buf()),
            &Sink,
            &RuntimeFixtureProbe,
            Some(node()),
        )
        .install(&request)
    };
    let first = installer(prefix.clone()).expect("initial managed install");
    std::fs::write(prefix.join("old-sentinel"), b"old").unwrap();

    use std::os::windows::fs::OpenOptionsExt;
    let entry = data_root.join("runtime/bin/ocx.cmd");
    let held_entry = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .share_mode(0x1 | 0x2)
        .open(&entry)
        .expect("hold the launcher without delete sharing");
    let error = installer(prefix.clone()).expect_err("launcher activation must fail");
    assert_eq!(error.code(), "filesystem");
    assert_eq!(std::fs::read(prefix.join("old-sentinel")).unwrap(), b"old");
    assert!(first.target.join(".runtime-manifest.json").is_file());
    drop(held_entry);

    let unowned = data_root.join("runtime/unowned");
    std::fs::create_dir_all(&unowned).unwrap();
    std::fs::write(unowned.join("user-file"), b"keep").unwrap();
    let error = installer(unowned.clone()).expect_err("unowned nonempty target must fail");
    assert_eq!(error.code(), "filesystem");
    assert_eq!(std::fs::read(unowned.join("user-file")).unwrap(), b"keep");
}

#[test]
fn windows_managed_install_accepts_the_extended_form_of_a_local_prefix() {
    let root = tempfile::tempdir().unwrap();
    let data_root = root.path().join("data root 中文");
    let ordinary = data_root.join("runtime/opencodex");
    let extended = PathBuf::from(format!(r"\\?\{}", ordinary.display()));
    let archive = runtime_fixture_archive(root.path());
    let npm = RuntimeFixtureNpm {
        archive: archive.clone(),
    };
    let request = InstallRequest {
        prefix: extended.clone(),
        source: InstallSource::Offline { archive },
        allow_scripts: false,
        npm_path: Some(node()),
        node_path: Some(node()),
    };
    let outcome = RuntimeInstaller::new(
        &data_root,
        &npm,
        Some(root.path().to_path_buf()),
        &Sink,
        &RuntimeFixtureProbe,
        Some(node()),
    )
    .install(&request)
    .expect("extended local path should retain its filesystem identity");
    assert!(outcome.target.exists());
    assert!(ordinary.join(".runtime-manifest.json").is_file());
    assert!(data_root.join("runtime/bin/ocx.cmd").is_file());
}

#[test]
fn windows_real_version_probe_requires_zero_exit() {
    let root = tempfile::tempdir().unwrap();
    let script = root.path().join("probe.js");
    std::fs::write(&script, b"console.log('2.77.0'); process.exitCode=7;").unwrap();
    assert_eq!(
        RealVersionProbe::default().probe(Some(&node()), &script),
        None,
        "version output from a failed process must not be accepted"
    );
}

#[test]
fn windows_npm_cmd_runner_passes_paths_and_controlled_environment() {
    let root = tempfile::Builder::new()
        .prefix("npm 中文 with spaces ")
        .tempdir()
        .unwrap();
    let evidence = root.path().join("arguments.json");
    let script = format!(
        "require('fs').writeFileSync({}, JSON.stringify({{argv:process.argv.slice(2), cwd:process.cwd(), profile:process.env.USERPROFILE, path:process.env.PATH}}));",
        serde_json::to_string(&evidence.to_string_lossy()).unwrap(),
    );
    let npm = write_launcher(root.path(), &script);
    let runner = SystemNpmRunner::new(Some(root.path().to_owned()), root.path().join("cache/npm"));
    runner
        .install_package(
            &NpmInstallRequest {
                npm_path: npm,
                node_dir: node().parent().map(Path::to_path_buf),
                prefix: root.path().join("安装 prefix"),
                version: "2.77.0".to_owned(),
                proxy: None,
                scripts_enabled: false,
                offline: true,
            },
            &Sink,
            &CancelFlag::new(),
        )
        .unwrap();
    let evidence: serde_json::Value =
        serde_json::from_slice(&std::fs::read(evidence).unwrap()).unwrap();
    let argv: Vec<String> = serde_json::from_value(evidence["argv"].clone()).unwrap();
    assert!(argv.contains(
        &root
            .path()
            .join("安装 prefix")
            .to_string_lossy()
            .into_owned()
    ));
    assert!(argv.contains(&"--offline".to_owned()));
    assert!(argv.contains(&"--ignore-scripts".to_owned()));
    assert_eq!(
        PathBuf::from(evidence["cwd"].as_str().unwrap()),
        root.path().join("安装 prefix")
    );
    assert_eq!(evidence["profile"], root.path().to_string_lossy().as_ref());
    assert!(
        std::env::split_paths(std::ffi::OsStr::new(evidence["path"].as_str().unwrap()))
            .any(|path| Some(path.as_path()) == node().parent())
    );
}

#[test]
#[ignore = "explicit CI official registry install; no daily profile or generation"]
fn windows_official_managed_registry_upgrade_and_offline_reinstall() {
    use opencodex_desktop_lib::modules::runtime::install::NullProgressSink;
    use opencodex_desktop_lib::modules::runtime::{RuntimeStore, MANAGED_PREFIX_RELATIVE};
    assert_eq!(std::env::var("OPENCODEX_MANAGED_LIVE").as_deref(), Ok("1"));
    let root = tempfile::Builder::new()
        .prefix("official managed 中文 ")
        .tempdir()
        .unwrap();
    let data = root.path().join("data");
    std::fs::create_dir_all(&data).unwrap();
    let data = data.canonicalize().unwrap();
    let prefix = data.join(MANAGED_PREFIX_RELATIVE);
    let npm =
        PathBuf::from(std::env::var_os("OPENCODEX_TEST_NPM").expect("explicit npm.cmd required"));
    assert!(npm.is_absolute() && npm.is_file());
    let runner = SystemNpmRunner::new(Some(root.path().to_path_buf()), data.join("cache/npm"));
    let probe = RealVersionProbe::default();
    let mut verified = Vec::new();
    for version in ["2.90.0", "2.91.0"] {
        let request = InstallRequest {
            prefix: prefix.clone(),
            source: InstallSource::Registry {
                version: version.into(),
                proxy: None,
            },
            allow_scripts: true,
            npm_path: Some(npm.clone()),
            node_path: Some(node()),
        };
        let outcome = RuntimeInstaller::new(
            &data,
            &runner,
            Some(root.path().to_path_buf()),
            &NullProgressSink,
            &probe,
            Some(node()),
        )
        .install(&request)
        .expect("actual official managed registry install");
        assert_eq!(outcome.version, version);
        assert!(outcome.entry.is_file());
        assert_eq!(
            probe
                .probe(
                    Some(&node()),
                    &prefix.join("node_modules/@bitkyc08/opencodex/bin/ocx.mjs")
                )
                .as_deref(),
            Some(version)
        );
        let record = RuntimeStore::new(&data).load();
        assert_eq!(record.installed_prefix(&data), prefix);
        assert_eq!(record.resolved_version.as_deref(), Some(version));
        verified.push(version.to_string());
    }
    let archive = runner
        .pack(
            &NpmPackRequest {
                npm_path: npm.clone(),
                node_dir: node().parent().map(Path::to_path_buf),
                version: "2.91.0".into(),
                proxy: None,
                work_dir: data.join("download"),
            },
            &NullProgressSink,
            &CancelFlag::new(),
        )
        .unwrap();
    let request = InstallRequest {
        prefix: prefix.clone(),
        source: InstallSource::Offline { archive },
        allow_scripts: true,
        npm_path: Some(npm),
        node_path: Some(node()),
    };
    let outcome = RuntimeInstaller::new(
        &data,
        &runner,
        Some(root.path().to_path_buf()),
        &NullProgressSink,
        &probe,
        Some(node()),
    )
    .install(&request)
    .expect("actual npm offline reinstall from warmed isolated cache");
    assert_eq!(outcome.version, "2.91.0");
    assert_eq!(
        RuntimeStore::new(&data)
            .load()
            .history
            .iter()
            .filter(|e| e.action == "install" && e.result == "succeeded")
            .count(),
        3
    );
    if let Some(path) = std::env::var_os("OPENCODEX_MANAGED_LIVE_REPORT") {
        std::fs::write(path,serde_json::to_vec_pretty(&serde_json::json!({"result":"pass","packages":verified,"officialRegistry":true,"realNpm":true,"realVersionProbe":true,"extendedCanonicalDataRoot":true,"upgrade":true,"offlineReinstall":true,"sourceRecordReadback":true,"dailyConfigTouched":false,"generationRequests":0})).unwrap()).unwrap();
    }
}

#[test]
fn windows_source_registration_failure_rolls_back_package_and_entry_then_retries() {
    use opencodex_desktop_lib::modules::runtime::RuntimeStore;
    use std::os::windows::fs::OpenOptionsExt;
    let root = tempfile::tempdir().unwrap();
    let data = root.path().join("data");
    let archive = runtime_fixture_archive(root.path());
    let npm = RuntimeFixtureNpm {
        archive: archive.clone(),
    };
    let request = InstallRequest {
        prefix: data.join("runtime/opencodex"),
        source: InstallSource::Offline { archive },
        allow_scripts: false,
        npm_path: Some(node()),
        node_path: Some(node()),
    };
    let run = || {
        RuntimeInstaller::new(
            &data,
            &npm,
            Some(root.path().to_path_buf()),
            &Sink,
            &RuntimeFixtureProbe,
            Some(node()),
        )
        .install(&request)
    };
    let first = run().unwrap();
    std::fs::write(first.target.join("keep-old"), b"old").unwrap();
    let entry = std::fs::read(&first.entry).unwrap();
    let store = RuntimeStore::new(&data);
    let record = std::fs::read(store.path()).unwrap();
    let held = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(0x1 | 0x2)
        .open(store.path())
        .unwrap();
    assert!(run().is_err());
    assert_eq!(
        std::fs::read(first.target.join("keep-old")).unwrap(),
        b"old"
    );
    assert_eq!(std::fs::read(&first.entry).unwrap(), entry);
    assert_eq!(std::fs::read(store.path()).unwrap(), record);
    drop(held);
    let next = run().unwrap();
    assert!(!next.target.join("keep-old").exists());
    assert_eq!(
        store
            .load()
            .history
            .iter()
            .filter(|e| e.result == "succeeded")
            .count(),
        2
    );
}
