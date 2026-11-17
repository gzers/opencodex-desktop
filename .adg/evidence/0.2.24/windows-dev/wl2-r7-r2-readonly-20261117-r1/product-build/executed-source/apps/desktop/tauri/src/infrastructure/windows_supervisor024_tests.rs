use super::*;
#[test]
fn wl2_next_s_missing_nested_staged_leaf_below_junction_is_unknown() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("H");
    let foreign = root.path().join("outside-H");
    std::fs::create_dir(&home).unwrap();
    std::fs::create_dir(&foreign).unwrap();
    assert!(staged(&home)
        .iter()
        .all(|(_, state)| matches!(state, Namespace::Absent { .. })));
    let link = home.join("winsw");
    let powershell = system_test_powershell();
    // Same task-only setup used by windows_private_create's native junction
    // test. Paths are environment values, never interpolated shell code.
    let output = std::process::Command::new(powershell)
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
            "$ErrorActionPreference='Stop'; New-Item -ItemType Junction -Path $env:OCX024_TEST_LINK -Target $env:OCX024_TEST_TARGET | Out-Null"])
        .env("OCX024_TEST_LINK", &link)
        .env("OCX024_TEST_TARGET", &foreign)
        .output().unwrap();
    println!(
        "task junction rawExit={:?}; stdout={}; stderr={}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "task junction creation failed: {:?} {:?}",
        output.stdout,
        output.stderr
    );
    let observed = staged(&home);
    let nested_unknown = observed
        .iter()
        .filter(|(name, state)| {
            name.starts_with("winsw/") && matches!(state, Namespace::Unknown { .. })
        })
        .count();
    let declaration_refused = require_state_absent(&link.join("service-state.json")).is_err();
    // Remove the exact junction itself, never recurse through it or its target.
    std::fs::remove_dir(&link).unwrap();
    assert_eq!(
        nested_unknown, 2,
        "missing leaves below a junction must not become Absent"
    );
    assert!(declaration_refused);
    assert!(foreign.is_dir());
    assert!(staged(&home)
        .iter()
        .all(|(_, state)| matches!(state, Namespace::Absent { .. })));
}
#[test]
fn wl2_next_s_missing_leaf_below_regular_file_parent_is_unknown() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("winsw"), b"not a directory").unwrap();
    let states = staged(root.path());
    assert!(states
        .iter()
        .filter(|(name, _)| name.starts_with("winsw/"))
        .all(|(_, state)| matches!(state, Namespace::Unknown { .. })));
    assert!(require_state_absent(&root.path().join("winsw/service-state.json")).is_err());
}
#[test]
fn wl2_supervisor_errors_never_become_absence() {
    for code in [0, 1, 5, 1058, 1072, 1460, 0x8007_0005, 0x8004_1315] {
        assert!(matches!(
            exact_not_found("SCM", code, 1060),
            Namespace::Unknown { .. }
        ));
        assert!(matches!(
            exact_not_found("task", code, 0x8007_0002),
            Namespace::Unknown { .. }
        ));
    }
    assert!(matches!(
        exact_not_found("SCM", 1060, 1060),
        Namespace::Absent { raw_code: 1060 }
    ));
    assert!(matches!(
        exact_not_found("task", 0x8007_0002, 0x8007_0002),
        Namespace::Absent { .. }
    ));
    assert!(query(Instant::now()).is_err());
}
#[test]
fn wl2_staged_and_native_namespaces_are_independent() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("opencodex-service.cmd"),
        b"synthetic staged conflict",
    )
    .unwrap();
    let rows = staged(dir.path());
    assert_eq!(rows.len(), 5);
    assert!(matches!(rows[1].1, Namespace::Present));
    let facts = query(Instant::now() + std::time::Duration::from_secs(5)).unwrap();
    println!(
        "native namespaces (observations only): {}",
        serde_json::to_string(&facts).unwrap()
    );
    // No absence assertion relies on installed state of the development host.
    assert!(matches!(rows[1].1, Namespace::Present));
}

#[test]
fn wl2_next_s_parent_notfound_and_access_denied_keep_raw_unknown() {
    for raw in [2, 3, 5, 32] {
        let result = path_namespace(Err(PathObservationError {
            leaf: false,
            cause: io::Error::from_raw_os_error(raw),
        }));
        assert_eq!(
            result,
            Namespace::Unknown {
                stage: "exact-staged-parent-pin",
                raw_code: raw as u32
            }
        );
    }
    assert_eq!(
        path_namespace(Err(PathObservationError {
            leaf: true,
            cause: io::Error::from_raw_os_error(2)
        })),
        Namespace::Absent { raw_code: 2 }
    );
    assert_eq!(
        path_namespace(Err(PathObservationError {
            leaf: true,
            cause: io::Error::from_raw_os_error(5)
        })),
        Namespace::Unknown {
            stage: "exact-staged-leaf",
            raw_code: 5
        }
    );
}

fn system_test_powershell() -> std::path::PathBuf {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetSystemDirectoryW(buffer: *mut u16, size: u32) -> u32;
    }
    let mut system = vec![0u16; 32768];
    let n = unsafe { GetSystemDirectoryW(system.as_mut_ptr(), system.len() as u32) } as usize;
    assert!(n > 0 && n < system.len());
    use std::os::windows::ffi::OsStringExt;
    std::path::PathBuf::from(std::ffi::OsString::from_wide(&system[..n]))
        .join("WindowsPowerShell/v1.0/powershell.exe")
}
