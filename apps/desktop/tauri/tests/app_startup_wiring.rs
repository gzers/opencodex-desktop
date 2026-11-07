#![cfg(unix)]
use opencodex_desktop_lib::infrastructure::runtime_executable::FixedRuntimeExecutable;
use opencodex_desktop_lib::infrastructure::status_source::OfficialStatusSource;
use opencodex_desktop_lib::modules::data_root::{validate_structure, StructureValidation};
use opencodex_desktop_lib::modules::process::EnvironmentPolicy;
use opencodex_desktop_lib::modules::status::StatusCollector;
use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;

#[test]
fn default_data_root_initializer_matches_v2_desktop_partition_structure() {
    let temp = tempfile::tempdir().expect("create temporary home fixture");
    let user = temp.path().canonicalize().unwrap().join("profile");
    let layout = opencodex_desktop_lib::modules::data_root::layout::Layout::new(
        &user,
        &temp.path().canonicalize().unwrap().join("program"),
        opencodex_desktop_lib::modules::data_root::layout::Platform::Unix,
    )
    .unwrap();
    let data_root = layout.desktop_root.clone();

    let created = opencodex_desktop_lib::modules::data_root::initialize(&data_root)
        .expect("default data root should initialize");
    assert!(created);

    let structure = validate_structure(&data_root).expect("validate default data root");
    assert_eq!(structure, StructureValidation::Valid);
    for name in [
        "manager-state",
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
    assert_eq!(
        layout.opencodex_home,
        user.join(".codex-desktop/opencodex/home")
    );
    assert!(!data_root.join("opencodex-home").exists());
    let metadata: serde_json::Value = serde_json::from_slice(
        &std::fs::read(data_root.join("manager-state/data-root.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(metadata["structure_version"], "2");
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
