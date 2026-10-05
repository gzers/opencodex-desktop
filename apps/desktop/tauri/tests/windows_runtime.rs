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
    runtime::install::{
        CancelFlag, InstallProgress, InstallProgressSink, NpmInstallRequest, NpmRunner,
        RealVersionProbe, SystemNpmRunner, VersionProbe,
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

struct Sink;
impl InstallProgressSink for Sink {
    fn emit(&self, _: InstallProgress) {}
}

#[test]
fn windows_npm_cmd_runner_passes_paths_and_controlled_environment() {
    let root = tempfile::Builder::new()
        .prefix("npm 中文 with spaces ")
        .tempdir()
        .unwrap();
    let evidence = root.path().join("arguments.json");
    let script = format!(
        "require('fs').writeFileSync({}, JSON.stringify({{argv:process.argv.slice(2), profile:process.env.USERPROFILE, path:process.env.PATH}}));",
        serde_json::to_string(&evidence.to_string_lossy()).unwrap(),
    );
    let npm = write_launcher(root.path(), &script);
    let runner = SystemNpmRunner::new(Some(root.path().to_owned()));
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
    assert_eq!(evidence["profile"], root.path().to_string_lossy().as_ref());
    assert!(
        std::env::split_paths(std::ffi::OsStr::new(evidence["path"].as_str().unwrap()))
            .any(|path| Some(path.as_path()) == node().parent())
    );
}
