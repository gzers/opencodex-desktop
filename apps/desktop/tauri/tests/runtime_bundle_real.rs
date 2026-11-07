//! Explicit artifact acceptance. Temporary I/D/H only; never starts a service.
#![cfg(feature = "integration-test")]
use opencodex_desktop_lib::modules::runtime::{
    self,
    bundled::{self, ProvisionResult},
};
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

#[test]
#[ignore = "requires an explicitly supplied pinned complete platform bundle"]
fn complete_bundle_provisions_and_runs_readonly_without_external_node_or_bun() {
    let archive =
        PathBuf::from(std::env::var_os("OCXD_RUNTIME_BUNDLE_ARCHIVE").expect("explicit archive"));
    let descriptor =
        std::fs::read(std::env::var_os("OCXD_RUNTIME_BUNDLE_MANIFEST").expect("explicit manifest"))
            .unwrap();
    let pin =
        std::env::var("OCXD_RUNTIME_BUNDLE_MANIFEST_SHA256").expect("pinned descriptor digest");
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let install = root.join("program/OpenCodeX Desktop");
    let data = root.join("profile/.codex-desktop/desktop");
    let home = root.join("profile/.codex-desktop/opencodex/home");
    std::fs::create_dir_all(&install).unwrap();
    std::fs::create_dir_all(data.join("manager-state")).unwrap();
    std::fs::create_dir_all(&home).unwrap();
    let sentinel = br#"{"providers":{},"unrelated":{"keep":true}}"#;
    std::fs::write(home.join("config.json"), sentinel).unwrap();
    let version = bundled::verify_bundle(&archive, &descriptor, &pin)
        .unwrap()
        .version;
    assert!(matches!(
        bundled::provision(&install, &data, &archive, &descriptor, &pin).unwrap(),
        ProvisionResult::Installed { .. }
    ));
    let (node, _) = runtime::install::managed_dependency_paths(&install);
    runtime::install::validate_managed_installation(&install, Some(&node)).unwrap();
    assert!(matches!(
        bundled::provision(&install, &data, &archive, &descriptor, &pin).unwrap(),
        ProvisionResult::AlreadyInstalled { .. }
    ));
    for args in [vec!["--version"], vec!["config", "validate"]] {
        let output = root.join(if args.len() == 1 {
            "version.out"
        } else {
            "validation.out"
        });
        let file = std::fs::File::create(&output).unwrap();
        let mut command = Command::new(install.join(runtime::MANAGED_ENTRY_RELATIVE));
        command
            .args(&args)
            .env_clear()
            .env("HOME", root.join("profile"))
            .env("OPENCODEX_HOME", &home)
            .env("XDG_CONFIG_HOME", root.join("profile/.config"))
            .env("XDG_CACHE_HOME", root.join("profile/.cache"))
            .env(
                "PATH",
                opencodex_desktop_lib::infrastructure::platform::controlled_path(
                    Vec::<PathBuf>::new(),
                    None,
                ),
            )
            .env("NO_PROXY", "*")
            .env("CI", "1")
            .current_dir(&home)
            .stdin(Stdio::null())
            .stdout(Stdio::from(file.try_clone().unwrap()))
            .stderr(Stdio::from(file));
        let mut child = command.spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if Instant::now() > deadline {
                let _ = child.kill();
                let _ = child.wait();
                panic!("readonly bundled command exceeded deadline");
            }
            std::thread::sleep(Duration::from_millis(25));
        };
        let observed = std::fs::read_to_string(&output).unwrap();
        assert!(status.success(), "readonly command failed: {observed}");
        if args.len() == 1 {
            assert!(
                observed.contains(&version),
                "version differs from pinned bundle"
            );
        }
    }
    assert_eq!(std::fs::read(home.join("config.json")).unwrap(), sentinel);
}
