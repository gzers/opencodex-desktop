//! Explicit isolated regression for the prerelease version reported by the
//! installed entry. It never starts OpenCodex, Codex or a real sync.
use std::path::PathBuf;

use opencodex_desktop_lib::modules::runtime::{
    archive::declared_version,
    install::{
        InstallRequest, InstallSource, NullProgressSink, RealVersionProbe, RuntimeInstaller,
        SystemNpmRunner, VersionProbe,
    },
    uninstall::{managed_prefix, uninstall_managed},
    RuntimeHandle, RuntimeSourceKind,
};

#[test]
#[ignore = "requires explicit local candidate archive, Node/npm, and public registry access"]
fn actual_managed_prerelease_install_preserves_full_version_and_survives_failure() {
    let archive =
        PathBuf::from(std::env::var_os("OCX_TEST_PRERELEASE_ARCHIVE").expect("explicit archive"));
    let node = PathBuf::from(std::env::var_os("OCX_TEST_RUNTIME_NODE").expect("explicit Node"));
    let npm = PathBuf::from(std::env::var_os("OCX_TEST_RUNTIME_NPM").expect("explicit npm"));
    assert!(archive.is_absolute() && node.is_absolute() && npm.is_absolute());
    let expected = declared_version(&archive).expect("explicit prerelease archive filename");
    assert!(expected.starts_with("2.97.0-post021.ownerquery."));
    let root = tempfile::Builder::new()
        .prefix("ocx-prerelease 安装 ")
        .tempdir()
        .unwrap();
    let home = root.path().join("home");
    std::fs::create_dir_all(&home).unwrap();
    let data = root.path().join("data");
    let cache = root.path().join("npm-cache");
    let runner = SystemNpmRunner::new(Some(home.clone()), cache);
    let probe = RealVersionProbe::default();
    let sink = NullProgressSink;
    let installer = RuntimeInstaller::new(
        &data,
        &runner,
        Some(home),
        &sink,
        &probe,
        Some(node.clone()),
    );
    let prefix = managed_prefix(&data);
    // Warm this test's empty npm cache through the real production installer.
    // No user's prefix, source record, npm config or runtime is reused.
    let stable = installer
        .install(&InstallRequest {
            prefix: prefix.clone(),
            source: InstallSource::Registry {
                version: "2.97.0".into(),
                proxy: None,
            },
            allow_scripts: false,
            npm_path: Some(npm.clone()),
            node_path: Some(node.clone()),
        })
        .expect("stable install must populate only the isolated test prefix/cache");
    assert_eq!(stable.version, "2.97.0");
    assert_eq!(probe.probe(None, &stable.entry).as_deref(), Some("2.97.0"));
    let candidate = installer
        .install(&InstallRequest {
            prefix: prefix.clone(),
            source: InstallSource::Offline { archive },
            allow_scripts: false,
            npm_path: Some(npm),
            node_path: Some(node),
        })
        .expect("the full prerelease version must match the archive manifest");
    assert_eq!(candidate.version, expected);
    assert!(!candidate.scripts_enabled);
    assert_eq!(
        probe.probe(None, &candidate.entry).as_deref(),
        Some(expected.as_str())
    );
    let handle = RuntimeHandle::initialize(&data, Vec::new());
    let resolution = handle.refresh();
    assert_eq!(resolution.kind, RuntimeSourceKind::Managed);
    assert_eq!(
        handle.store().load().resolved_version.as_deref(),
        Some(expected.as_str())
    );
    let entry_before = std::fs::read(&candidate.entry).unwrap();
    let package_path = prefix.join("node_modules/@bitkyc08/opencodex/package.json");
    let package_before = std::fs::read(&package_path).unwrap();
    let bad = installer.install(&InstallRequest {
        prefix: prefix.clone(),
        source: InstallSource::Registry {
            version: "0.0.0-post021-intentionally-absent".into(),
            proxy: None,
        },
        allow_scripts: false,
        npm_path: Some(PathBuf::from(
            std::env::var_os("OCX_TEST_RUNTIME_NPM").unwrap(),
        )),
        node_path: Some(PathBuf::from(
            std::env::var_os("OCX_TEST_RUNTIME_NODE").unwrap(),
        )),
    });
    assert!(bad.is_err());
    assert_eq!(
        probe.probe(None, &candidate.entry).as_deref(),
        Some(expected.as_str())
    );
    assert_eq!(std::fs::read(&candidate.entry).unwrap(), entry_before);
    assert_eq!(std::fs::read(package_path).unwrap(), package_before);
    let after = handle.refresh();
    assert_eq!(after.kind, RuntimeSourceKind::Managed);
    assert_eq!(after.path, resolution.path);
    assert_eq!(
        handle.store().load().resolved_version.as_deref(),
        Some(expected.as_str())
    );
    uninstall_managed(&data, &prefix, &candidate.entry).unwrap();
    println!("PASS: real managed stable→prerelease/offline entry/source version; failed install preserved candidate; isolated uninstall");
}
