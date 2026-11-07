use super::*;

#[test]
fn ordinary_and_canonical_extended_roots_accept_existing_and_absent_children() {
    let temp = tempfile::tempdir().unwrap();
    let canonical = temp.path().canonicalize().unwrap();
    assert!(canonical.to_string_lossy().starts_with(r"\\?\"));
    let ordinary = PathBuf::from(canonical.to_str().unwrap().strip_prefix(r"\\?\").unwrap());
    let existing = canonical.join("existing directory");
    fs::create_dir(&existing).unwrap();
    for root in [&ordinary, &canonical] {
        absolute(&root.join("existing directory")).unwrap();
        absolute(&root.join("absent/child")).unwrap();
        roots(&root.join("program"), &root.join("profile/desktop")).unwrap();
        assert!(roots(&root.join("program"), &root.join("program/nested")).is_err());
    }
}

#[test]
fn complete_root_checks_still_reject_regular_files_and_parent_traversal() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let file = root.join("file");
    fs::write(&file, b"keep").unwrap();
    assert!(absolute(&file).is_err());
    assert!(absolute(&file.join("child")).is_err());
    // Joining onto a verbatim Windows prefix normalizes parent components.
    // Construct the caller spelling directly so the negative input still has ..
    let traversal = PathBuf::from(format!(r"{}\one\..\two", root.display()));
    assert!(traversal.to_str().unwrap().contains(r"\..\"));
    assert!(absolute(&traversal).is_err());
    assert_eq!(fs::read(&file).unwrap(), b"keep");
}

#[test]
fn junction_is_rejected_before_any_linked_directory_is_accepted() {
    let temp = tempfile::tempdir().unwrap();
    let canonical = temp.path().canonicalize().unwrap();
    let ordinary = PathBuf::from(canonical.to_str().unwrap().strip_prefix(r"\\?\").unwrap());
    let target = ordinary.join("target");
    let junction = ordinary.join("junction");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("sentinel"), b"untouched").unwrap();
    // Directory junctions require no developer-mode or symbolic-link privilege.
    // Setup failure is a real test failure, never a skipped negative assertion.
    let output = std::process::Command::new("cmd.exe")
        .args(["/d", "/c", "mklink", "/J"])
        .arg(&junction)
        .arg(&target)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "junction fixture failed: {output:?}"
    );
    let normal_result = absolute(&junction.join("absent"));
    let extended_result = absolute(&canonical.join("junction/absent"));
    fs::remove_dir(&junction).unwrap();
    for result in [normal_result, extended_result] {
        match result {
            Err(AppError::FileSystem { detail, .. }) => {
                assert!(detail.contains("reparse point"), "{detail}");
            }
            other => panic!("expected explicit reparse rejection, got {other:?}"),
        }
    }
    assert_eq!(fs::read(target.join("sentinel")).unwrap(), b"untouched");
}
