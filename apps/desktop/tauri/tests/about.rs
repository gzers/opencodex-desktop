#![cfg(unix)]
//! 关于页契约集成测试：应用元数据、官方版本来源与命令投影。

use std::sync::{Arc, Mutex};

use opencodex_desktop_lib::commands::about::official_project_facts_with_source;
use opencodex_desktop_lib::errors::AppError;
use opencodex_desktop_lib::infrastructure::official_version_source::OfficialCliVersionSource;
use opencodex_desktop_lib::infrastructure::runtime_executable::FixedRuntimeExecutable;
use opencodex_desktop_lib::modules::about::{
    official_version_timeout, OfficialProjectFacts, OfficialVersionError, OfficialVersionOutput,
    OfficialVersionSource, OFFICIAL_VERSION_MAX_CHARS,
};
use opencodex_desktop_lib::modules::process::EnvironmentPolicy;
use opencodex_desktop_lib::types::about::OfficialProjectDto;

#[derive(Debug)]
struct FixtureSource {
    result: Result<OfficialVersionOutput, OfficialVersionError>,
}

impl OfficialVersionSource for FixtureSource {
    fn run(&self) -> Result<OfficialVersionOutput, OfficialVersionError> {
        self.result.clone()
    }
}

fn shared(source: FixtureSource) -> Arc<Mutex<FixtureSource>> {
    Arc::new(Mutex::new(source))
}

#[test]
fn official_version_output_is_sanitized_and_bounded() {
    let output = OfficialVersionOutput {
        stdout: b"OpenCodex 2.50.0\nnext-line\n".to_vec(),
    };
    let facts = OfficialProjectFacts::from_output(&output).expect("facts");
    assert_eq!(facts.display_name, "OpenCodex");
    assert_eq!(facts.version.as_deref(), Some("2.50.0"));
    assert_eq!(facts.raw_version, "OpenCodex 2.50.0");
    assert!(!facts.truncated);
}

#[test]
fn official_command_maps_errors_and_preserves_explicit_contract() {
    let source = shared(FixtureSource {
        result: Err(OfficialVersionError::Timeout),
    });
    assert!(matches!(
        official_project_facts_with_source(&source),
        Err(AppError::Timeout)
    ));
    let source = shared(FixtureSource {
        result: Err(OfficialVersionError::Unreachable),
    });
    assert!(matches!(
        official_project_facts_with_source(&source),
        Err(AppError::NotConfigured)
    ));
}

#[test]
fn official_project_dto_is_camel_case() {
    let facts = OfficialProjectFacts::from_output(&OfficialVersionOutput {
        stdout: b"OpenCodex 2.50.0\n".to_vec(),
    })
    .expect("facts");
    let payload = serde_json::to_value(OfficialProjectDto::from(facts)).expect("dto");
    assert_eq!(payload["displayName"], "OpenCodex");
    assert_eq!(payload["version"], "2.50.0");
    assert_eq!(payload["truncated"], false);
}

#[tokio::test]
async fn official_cli_source_requires_explicit_paths_and_freezes_environment() {
    let root = tempfile::tempdir().expect("fixture root");
    let missing = root.path().join("missing-ocx");
    let working = root.path().join("home");
    std::fs::create_dir_all(&working).expect("working directory");
    let environment = EnvironmentPolicy {
        opencodex_home: root.path().join("opencodex-home"),
        ..Default::default()
    };
    let source = OfficialCliVersionSource::new(
        FixedRuntimeExecutable::resolved(missing),
        working,
        environment,
    );
    assert_eq!(
        source.run().expect_err("missing executable"),
        OfficialVersionError::Unreachable
    );
}

#[test]
fn official_cli_source_enforces_frozen_timeout() {
    let root = tempfile::tempdir().expect("fixture root");
    let executable = root.path().join("sleeping-ocx");
    std::fs::write(
        &executable,
        format!(
            "#!/bin/sh\nsleep {}\n",
            official_version_timeout().as_secs() + 1
        ),
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
    let source = OfficialCliVersionSource::new(
        FixedRuntimeExecutable::resolved(executable),
        working,
        environment,
    );
    let started = std::time::Instant::now();
    let error = std::thread::spawn(move || source.run())
        .join()
        .expect("source thread should not panic")
        .expect_err("timeout expected");
    assert_eq!(error, OfficialVersionError::Timeout);
    assert!(started.elapsed() >= official_version_timeout());
    assert!(started.elapsed() < official_version_timeout() + std::time::Duration::from_secs(2));
}

#[test]
fn frozen_version_contract_has_no_writable_flags() {
    assert_eq!(
        official_version_timeout(),
        std::time::Duration::from_secs(2)
    );
    assert_eq!(OFFICIAL_VERSION_MAX_CHARS, 240);
}
