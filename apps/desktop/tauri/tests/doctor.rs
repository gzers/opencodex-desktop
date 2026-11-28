//! Doctor 只读契约集成测试：来源契约、脱敏、截断、失败保留和命令投影。

use std::sync::{Arc, Mutex};
use std::time::Duration;

use opencodex_desktop_lib::commands::doctor::run_doctor_with_source;
use opencodex_desktop_lib::errors::AppError;
use opencodex_desktop_lib::infrastructure::official_cli_source::OfficialDoctorSource;
use opencodex_desktop_lib::infrastructure::runtime_executable::FixedRuntimeExecutable;
use opencodex_desktop_lib::modules::doctor::{
    doctor_timeout, DoctorError, DoctorOutput, DoctorReport, DoctorSource, DOCTOR_MAX_LINES,
};
use opencodex_desktop_lib::modules::process::EnvironmentPolicy;
use opencodex_desktop_lib::types::doctor::{DoctorDto, DoctorModeDto};

#[derive(Debug)]
struct FixtureDoctorSource {
    stdout: Vec<u8>,
    error: Option<DoctorError>,
}

impl FixtureDoctorSource {
    #[expect(dead_code)]
    fn success(stdout: &str) -> Self {
        Self {
            stdout: stdout.as_bytes().to_vec(),
            error: None,
        }
    }

    fn failure(error: DoctorError) -> Self {
        Self {
            stdout: Vec::new(),
            error: Some(error),
        }
    }
}

impl DoctorSource for FixtureDoctorSource {
    fn run(&self) -> Result<DoctorOutput, DoctorError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        Ok(DoctorOutput {
            stdout: self.stdout.clone(),
        })
    }
}

fn shared(source: FixtureDoctorSource) -> Arc<Mutex<FixtureDoctorSource>> {
    Arc::new(Mutex::new(source))
}

#[test]
fn doctor_report_sanitizes_frozen_secret_shapes() {
    let output = DoctorOutput {
        stdout: b"provider key: api_key=abcd1234\nS3 URI: s3://private-bucket/path\ncall https://user:token@api.example.com/v1?secret=value ok\n"
            .to_vec(),
    };
    let report = DoctorReport::from_output(&output).expect("report");
    assert_eq!(
        report.lines,
        vec![
            "provider key: api_key=[REDACTED]",
            "S3 URI: [REDACTED]",
            "call https://api.example.com/v1 ok",
        ]
    );
    assert_eq!(
        report.mode,
        opencodex_desktop_lib::modules::doctor::DoctorMode::ReadOnly
    );
    assert!(!report.truncated);
}

#[test]
fn doctor_command_maps_timeout_to_explicit_error() {
    let source = shared(FixtureDoctorSource::failure(DoctorError::Timeout));
    let error = run_doctor_with_source(&source).expect_err("timeout");
    assert!(matches!(error, AppError::Timeout), "{error:?}");
}

#[test]
fn doctor_command_maps_unreachable_and_parse_to_not_configured() {
    for error in [DoctorError::Unreachable, DoctorError::Parse] {
        let source = shared(FixtureDoctorSource::failure(error));
        assert!(matches!(
            run_doctor_with_source(&source),
            Err(AppError::NotConfigured)
        ));
    }
}

#[test]
fn doctor_dto_is_camel_case_and_read_only() {
    let report = DoctorReport::from_output(&DoctorOutput {
        stdout: b"ok\n".to_vec(),
    })
    .expect("report");
    let payload = serde_json::to_value(DoctorDto::from(report)).expect("dto");
    assert_eq!(payload["mode"], "readOnly");
    assert_eq!(payload["lines"][0], "ok");
    assert_eq!(payload["truncated"], false);
}

#[test]
fn doctor_mode_dto_is_read_only_only() {
    let payload = serde_json::to_string(&DoctorModeDto::ReadOnly).expect("mode");
    assert_eq!(payload, "\"readOnly\"");
}

fn official_source(root: &std::path::Path, executable: std::path::PathBuf) -> OfficialDoctorSource {
    OfficialDoctorSource::new(
        FixedRuntimeExecutable::resolved(executable),
        root,
        EnvironmentPolicy {
            opencodex_home: root.join("opencodex-home"),
            ..Default::default()
        },
    )
}

fn lines(output: DoctorOutput) -> Vec<String> {
    DoctorReport::from_output(&output).expect("report").lines
}

#[test]
fn official_source_reports_missing_directory_and_program() {
    let root = tempfile::tempdir().expect("fixture root");
    let source = official_source(root.path(), root.path().join("missing-ocx"));
    let local = lines(source.run().unwrap());
    assert_eq!(local[0], "OpenCodex 程序：未发现");
    assert_eq!(local[1], "诊断工作目录：不可用");
    assert!(lines(source.run_network().unwrap())[0].contains("诊断工作目录不可用"));
    std::fs::create_dir(root.path().join("cache")).unwrap();
    assert!(lines(source.run_network().unwrap())[0].contains("未发现可用的 OpenCodex 程序"));
    assert!(!root.path().join("opencodex-home").exists());
}

#[cfg(unix)]
fn shell_fixture(root: &std::path::Path, body: &str) -> OfficialDoctorSource {
    use std::os::unix::fs::PermissionsExt;
    let executable = root.join("fixture-ocx");
    std::fs::write(&executable, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    std::fs::create_dir(root.join("cache")).unwrap();
    official_source(root, executable)
}

#[cfg(unix)]
#[test]
fn only_explicit_network_diagnostics_run_the_program_in_the_checked_cache() {
    let root = tempfile::tempdir().unwrap();
    let source = shell_fixture(
        root.path(),
        r#"
        test "$1" = doctor || exit 7
        test "${PWD##*/}" = cache || exit 8
        test "${OPENCODEX_HOME##*/}" = opencodex-home || exit 9
        : > executed
        echo 'doctor fixture'
    "#,
    );
    assert_eq!(lines(source.run().unwrap())[1], "诊断工作目录：可用");
    let marker = root.path().join("cache/executed");
    assert!(!marker.exists());
    assert_eq!(lines(source.run_network().unwrap()), ["doctor fixture"]);
    assert!(marker.is_file());
}

#[cfg(unix)]
#[test]
fn network_diagnostics_report_the_timeout_stage_and_owned_process_exit() {
    std::thread::scope(|scope| {
        let root_timeout = scope.spawn(|| {
            assert_network_timeout("exec /bin/sleep 30", "等待诊断进程退出超时");
        });
        assert_network_timeout("/bin/sleep 30 & exit 0", "等待读完诊断输出超时");
        root_timeout.join().unwrap();
    });
}

#[cfg(unix)]
fn assert_network_timeout(script: &str, stage: &str) {
    let root = tempfile::tempdir().unwrap();
    let source = shell_fixture(root.path(), script);
    let started = std::time::Instant::now();
    let report = lines(source.run_network().unwrap());
    assert!(report[0].contains(stage), "{report:?}");
    assert_eq!(report[1], "受控进程已退出。");
    assert!(started.elapsed() >= doctor_timeout());
    assert!(started.elapsed() < doctor_timeout() + Duration::from_secs(3));
}

#[test]
fn frozen_timeout_has_no_writable_repair_flags() {
    // 防回归：只读来源只允许 doctor 子命令；写入型修复参数不存在。
    assert_eq!(doctor_timeout(), Duration::from_secs(5));
    assert_eq!(DOCTOR_MAX_LINES, 400);
}
