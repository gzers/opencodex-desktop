//! Windows actual-files regression for the installed launcher/resolver boundary.
use super::*;

#[test]
fn installed_native_separator_launcher_resolves_from_equivalent_mixed_root() {
    let task = tempfile::tempdir_in(crate::modules::test_sandbox::temporary_parent()).unwrap();
    let root = task.path().join("Desktop 中文 Spaces");
    let prefix = root.join("runtime/opencodex");
    let stored_prefix = paths::validate_install_target(&prefix, None).unwrap();
    let (node, _) = managed_dependency_paths(&root);
    let entry = root.join(MANAGED_ENTRY_RELATIVE);
    let package = stored_prefix.join(PACKAGE_SUBPATH);
    let bin = "bin/ocx.mjs";
    for path in [
        node.parent().unwrap(),
        entry.parent().unwrap(),
        &package.join("bin"),
    ] {
        std::fs::create_dir_all(path).unwrap();
    }
    std::fs::write(&node, b"isolated node identity fixture; never executed").unwrap();
    std::fs::write(package.join(bin), b"throw new Error('never executed')").unwrap();
    std::fs::write(
        package.join("package.json"),
        serde_json::to_vec(&serde_json::json!({
            "name": OFFICIAL_PACKAGE, "version": "2.91.0", "bin": {"ocx": bin}
        }))
        .unwrap(),
    )
    .unwrap();
    write_entry(&entry, Some(&node), &package.join(bin)).unwrap();
    let proof = super::super::protection::activation_proof(
        &stored_prefix,
        &uuid::Uuid::new_v4().to_string(),
        bin,
        &node,
    )
    .unwrap();
    let manifest = RuntimeManifest {
        activation: Some(proof),
        package: OFFICIAL_PACKAGE.into(),
        version: "2.91.0".into(),
        tarball_sha256: "a".repeat(64),
        installed_at: now_rfc3339(),
        npm_path: "explicit fixture".into(),
        scripts_enabled: false,
        source: InstallSourceKind::Registry,
    };
    std::fs::write(
        stored_prefix.join(MANIFEST_FILENAME),
        serde_json::to_vec(&manifest).unwrap(),
    )
    .unwrap();
    // The writer receives normalized storage paths, while fixed-I resolution
    // joins the same constants onto the original ordinary Windows root.
    validate_managed_installation(&root, Some(&node)).unwrap();
    let d = task.path().join("D");
    std::fs::create_dir_all(&d).unwrap();
    let handle = super::super::RuntimeHandle::initialize_managed(&d, &root);
    assert_eq!(handle.kind(), super::super::RuntimeSourceKind::Managed);
    // Changed launcher bytes are still refused; formatting equivalence must
    // never remove the activation proof or exact generated-entry check.
    let bytes = std::fs::read(&entry).unwrap();
    std::fs::write(&entry, [bytes.as_slice(), b"tampered"].concat()).unwrap();
    assert!(validate_managed_installation(&root, Some(&node)).is_err());
    assert_eq!(
        handle.refresh().kind,
        super::super::RuntimeSourceKind::Unresolved
    );
}

#[test]
fn windows_launcher_separator_equivalence_keeps_percent_quotes_and_argument_boundary() {
    let native = Path::new(r"C:\Desktop 中文\runtime\node\node.exe");
    let mixed = Path::new("C:/Desktop 中文/runtime/node/node.exe");
    let script = Path::new(r"C:\Desktop 100%\runtime\opencodex\bin\ocx.mjs");
    let mixed_script = Path::new("C:/Desktop 100%/runtime/opencodex/bin/ocx.mjs");
    let expected = entry_script(Some(native), script);
    assert_eq!(entry_script(Some(mixed), mixed_script), expected);
    assert!(expected.contains("100%%"));
    assert!(expected.contains("setlocal DisableDelayedExpansion\r\n"));
    assert!(expected.ends_with(" %*\r\nexit /b %errorlevel%\r\n"));
}
