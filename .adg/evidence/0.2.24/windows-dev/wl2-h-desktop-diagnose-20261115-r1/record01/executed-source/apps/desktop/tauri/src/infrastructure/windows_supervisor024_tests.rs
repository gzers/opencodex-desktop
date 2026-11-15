use super::*;
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
