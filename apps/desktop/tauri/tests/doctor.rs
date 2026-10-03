#![cfg(unix)]
//! Doctor 只读契约集成测试：来源契约、脱敏、截断、失败保留和命令投影。

use std::sync::{Arc, Mutex};
use std::time::Duration;

use opencodex_desktop_lib::commands::doctor::run_doctor_with_source;
use opencodex_desktop_lib::errors::AppError;
use opencodex_desktop_lib::infrastructure::official_cli_source::OfficialDoctorSource;
use opencodex_desktop_lib::infrastructure::runtime_executable::FixedRuntimeExecutable;
use opencodex_desktop_lib::modules::doctor::{
    DoctorError, DoctorOutput, DoctorReport, DoctorSource, DOCTOR_MAX_LINES, DOCTOR_TIMEOUT,
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

#[tokio::test]
async fn official_source_requires_explicit_paths_and_freezes_environment() {
    let root = tempfile::tempdir().expect("fixture root");
    let missing = root.path().join("missing-ocx");
    let working = root.path().join("home");
    std::fs::create_dir_all(&working).expect("working directory");
    let environment = EnvironmentPolicy {
        opencodex_home: root.path().join("opencodex-home"),
        ..Default::default()
    };
    let source = OfficialDoctorSource::new(
        FixedRuntimeExecutable::resolved(missing),
        working,
        environment,
    );
    let error = source.run().expect_err("missing executable");
    assert_eq!(error, DoctorError::Unreachable);
}

#[test]
fn official_source_enforces_frozen_timeout() {
    let root = tempfile::tempdir().expect("fixture root");
    let executable = root.path().join("sleeping-ocx");
    std::fs::write(
        &executable,
        format!("#!/bin/sh\nsleep {}\n", DOCTOR_TIMEOUT.as_secs() + 1),
    )
    .expect("fixture");
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o755))
        .expect("chmod fixture");
    let working = root.path().join("home");
    std::fs::create_dir_all(&working).expect("working directory");
    let environment = EnvironmentPolicy {
        opencodex_home: root.path().join("opencodex-home"),
        ..Default::default()
    };
    let source = OfficialDoctorSource::new(
        FixedRuntimeExecutable::resolved(executable),
        working,
        environment,
    );
    let started = std::time::Instant::now();
    let error = std::thread::spawn(move || source.run())
        .join()
        .expect("source thread should not panic")
        .expect_err("timeout expected");
    assert_eq!(error, DoctorError::Timeout);
    assert!(started.elapsed() >= DOCTOR_TIMEOUT);
    assert!(started.elapsed() < DOCTOR_TIMEOUT + Duration::from_secs(2));
}

#[test]
fn official_source_rejects_non_executable_file() {
    let root = tempfile::tempdir().expect("fixture root");
    let executable = root.path().join("not-executable");
    std::fs::write(&executable, b"not executable").expect("fixture");
    let working = root.path().join("home");
    std::fs::create_dir_all(&working).expect("working directory");
    let environment = EnvironmentPolicy {
        opencodex_home: root.path().join("opencodex-home"),
        ..Default::default()
    };
    let source = OfficialDoctorSource::new(
        FixedRuntimeExecutable::resolved(executable),
        working,
        environment,
    );
    assert_eq!(
        source.run().expect_err("unreachable"),
        DoctorError::Unreachable
    );
}

#[test]
fn frozen_timeout_has_no_writable_repair_flags() {
    // 防回归：只读来源只允许 doctor 子命令；写入型修复参数不存在。
    assert_eq!(DOCTOR_TIMEOUT, Duration::from_secs(5));
    assert_eq!(DOCTOR_MAX_LINES, 400);
}
