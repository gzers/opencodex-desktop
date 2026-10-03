#![cfg(unix)]
use opencodex_desktop_lib::infrastructure::runtime_executable::FixedRuntimeExecutable;
use opencodex_desktop_lib::infrastructure::status_source::OfficialStatusSource;
use opencodex_desktop_lib::modules::data_root::{validate_structure, StructureValidation};
use opencodex_desktop_lib::modules::process::EnvironmentPolicy;
use opencodex_desktop_lib::modules::status::StatusCollector;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;

#[test]
fn default_data_root_initializer_matches_fz02_structure() {
    let temp = tempfile::tempdir().expect("create temporary home fixture");
    let data_root = temp.path().join("OpenCodex Desktop");

    let created = opencodex_desktop_lib::modules::data_root::initialize(&data_root)
        .expect("default data root should initialize");
    assert!(created);

    let structure = validate_structure(&data_root).expect("validate default data root");
    assert_eq!(structure, StructureValidation::Valid);
    for name in [
        "manager-state",
        "opencodex-home",
        "backups",
        "logs",
        "exports",
        "cache",
        "sync-state",
    ] {
        assert!(
            data_root.join(name).is_dir(),
            "{name} partition should exist"
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn controlled_status_source_uses_explicit_fixture_without_path_search() {
    let temp = tempfile::tempdir().expect("create temporary home fixture");
    let executable = temp.path().join("ocx");
    std::fs::write(
        &executable,
        b"#!/bin/sh\necho '{\"status\":\"stopped\",\"dataRoot\":\"/tmp/fixture\",\"startup\":{\"protection\":\"none\",\"rebootSafe\":false}}'\n",
    )
    .expect("write fixture");
    std::fs::set_permissions(&executable, Permissions::from_mode(0o755)).expect("chmod fixture");
    let home = temp.path().join("home");
    std::fs::create_dir_all(&home).expect("create home");
    let data_root = temp.path().join("OpenCodex Desktop");
    std::fs::create_dir_all(&data_root).expect("create data root fixture");

    let source = OfficialStatusSource::new(
        FixedRuntimeExecutable::resolved(executable),
        home,
        EnvironmentPolicy {
            opencodex_home: data_root.join("opencodex-home"),
            ..Default::default()
        },
    );
    let mut collector = StatusCollector::new(source);
    let mapped = collector.refresh().expect("controlled status refresh");
    assert_eq!(
        mapped.runtime,
        opencodex_desktop_lib::types::status::RuntimeState::Stopped
    );
}
