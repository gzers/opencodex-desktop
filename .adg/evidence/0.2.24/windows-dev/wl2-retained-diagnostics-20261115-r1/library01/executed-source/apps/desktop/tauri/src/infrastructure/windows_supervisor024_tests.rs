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
    let cmd =
        std::path::PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join("System32/cmd.exe");
    let output = std::process::Command::new(cmd)
        .args(["/d", "/c", "mklink", "/J"])
        .arg(&link)
        .arg(&foreign)
        .output()
        .unwrap();
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
