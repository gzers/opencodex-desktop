use super::*;
use std::{fs, os::unix::fs::PermissionsExt};

pub(crate) struct Fixture {
    _temp: tempfile::TempDir,
    pub(crate) root: PathBuf,
    pub(crate) paths: RuntimeVersionPaths,
    image: ImageManifest,
}
impl Fixture {
    pub(crate) fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let paths =
            RuntimeVersionPaths::new(&root.join("runtime/versions"), "2.99.0", Platform::Macos)
                .unwrap();
        let package = paths.package_root.join(install::PACKAGE_SUBPATH);
        let script = package.join("bin/ocx.mjs");
        let package_json = serde_json::to_vec(&serde_json::json!({"name":OFFICIAL_PACKAGE,"version":"2.99.0","bin":{"ocx":"bin/ocx.mjs"}})).unwrap();
        for (path, bytes, mode) in [
            (package.join("package.json"), package_json.as_slice(), 0o600),
            (
                script.clone(),
                b"#!/usr/bin/env node\nthrow Error('do not run');\n",
                0o755,
            ),
            (paths.node.clone(), b"#!/bin/sh\nexit 93\n", 0o755),
            (paths.npm.clone(), b"throw Error('do not run npm');", 0o600),
            (paths.bun.clone(), b"#!/bin/sh\nexit 94\n", 0o755),
            (
                paths.package_root.join("node_modules/bun/bin/bun.exe"),
                b"#!/bin/sh\nexit 94\n",
                0o755,
            ),
        ] {
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(&path, bytes).unwrap();
            fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
        }
        let entry = root.join("runtime/bin/ocx-runtime");
        fs::create_dir_all(entry.parent().unwrap()).unwrap();
        fs::write(&entry, install::entry_script(Some(&paths.node), &script)).unwrap();
        fs::set_permissions(&entry, fs::Permissions::from_mode(0o755)).unwrap();
        let image = ImageManifest {
            schema_version: 1,
            tree_sha256: super::super::program_image::content_digest(&paths.version_root).unwrap(),
            package: OFFICIAL_PACKAGE.into(),
            version: "2.99.0".into(),
            platform: "darwin-arm64".into(),
            official_archive_sha256: hash::sha256_hex(b"pinned synthetic archive identity"),
            bin: "bin/ocx.mjs".into(),
            package_sha256: hash::sha256_file(&package.join("package.json")).unwrap(),
            bin_sha256: hash::sha256_file(&script).unwrap(),
            node_sha256: hash::sha256_file(&paths.node).unwrap(),
            npm_sha256: hash::sha256_file(&paths.npm).unwrap(),
            bun_sha256: hash::sha256_file(&paths.bun).unwrap(),
            bun_launcher_sha256: hash::sha256_file(
                &paths.package_root.join("node_modules/bun/bin/bun.exe"),
            )
            .unwrap(),
        };
        let f = Self {
            _temp: temp,
            root,
            paths,
            image,
        };
        f.publish();
        f
    }
    fn publish(&self) {
        let bytes = serde_json::to_vec(&self.image).unwrap();
        fs::write(self.paths.version_root.join(IMAGE_MANIFEST), &bytes).unwrap();
        let selection = CurrentSelection {
            schema_version: 1,
            version: self.image.version.clone(),
            platform: self.image.platform.clone(),
            image_sha256: hash::sha256_hex(&bytes),
        };
        fs::write(
            self.root.join("runtime/current.json"),
            serde_json::to_vec(&selection).unwrap(),
        )
        .unwrap();
    }
    fn read(&self) -> AppResult<Option<SelectedProgram>> {
        read(&self.root, Platform::Macos, "darwin-arm64")
    }
}

#[test]
fn absent_program_root_is_not_created_or_adopted() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().canonicalize().unwrap().join("not-installed");
    assert_eq!(read(&root, Platform::Macos, "darwin-arm64").unwrap(), None);
    assert!(!root.exists());
    let alias = t.path().canonicalize().unwrap().join("alias");
    std::os::unix::fs::symlink(t.path(), &alias).unwrap();
    assert!(read(&alias.join("missing"), Platform::Macos, "darwin-arm64").is_err());
}

#[test]
fn reads_one_complete_image_without_executing_any_program() {
    let f = Fixture::new();
    let selected = f.read().unwrap().unwrap();
    assert_eq!(selected.paths, f.paths);
    assert_eq!(selected.selection.version, "2.99.0");
    assert_eq!(
        selected.script,
        f.paths
            .package_root
            .join(install::PACKAGE_SUBPATH)
            .join("bin/ocx.mjs")
    );
}

#[test]
fn missing_selection_never_adopts_existing_flat_runtime() {
    let f = Fixture::new();
    fs::remove_file(f.root.join("runtime/current.json")).unwrap();
    fs::create_dir_all(f.root.join("runtime/opencodex")).unwrap();
    fs::write(f.root.join("runtime/opencodex/old-sentinel"), b"never use").unwrap();
    assert_eq!(f.read().unwrap(), None);
    assert_eq!(
        fs::read(f.root.join("runtime/opencodex/old-sentinel")).unwrap(),
        b"never use"
    );
}

#[test]
fn corrupt_or_duplicate_selection_is_unknown_not_uninstalled() {
    let f = Fixture::new();
    let current = f.root.join("runtime/current.json");
    for bytes in [b"{".as_slice(), br#"{"schemaVersion":1,"schemaVersion":1,"version":"2.99.0","platform":"darwin-arm64","imageSha256":"bad"}"#] {
        fs::write(&current, bytes).unwrap();
        assert!(f.read().is_err());
        assert_eq!(fs::read(&current).unwrap(), bytes);
    }
}

#[test]
fn every_critical_program_file_and_stable_launcher_are_bound() {
    for which in 0..7 {
        let f = Fixture::new();
        let package = f.paths.package_root.join(install::PACKAGE_SUBPATH);
        let target = [
            f.paths.node.clone(),
            f.paths.npm.clone(),
            f.paths.bun.clone(),
            package.join("package.json"),
            package.join("bin/ocx.mjs"),
            f.root.join("runtime/bin/ocx-runtime"),
            f.paths.package_root.join("node_modules/bun/bin/bun.exe"),
        ][which]
            .clone();
        fs::write(&target, b"changed").unwrap();
        assert!(f.read().is_err(), "{}", target.display());
        assert_eq!(fs::read(&target).unwrap(), b"changed");
    }
}

#[test]
fn manifest_mismatch_and_other_platform_do_not_select_a_fallback() {
    let mut f = Fixture::new();
    fs::write(f.paths.version_root.join(IMAGE_MANIFEST), b"{}").unwrap();
    assert!(f.read().is_err());
    f.image.platform = "win32-x64".into();
    f.publish();
    assert!(f.read().is_err());
    f.image.platform = "darwin-arm64".into();
    f.image.bin = "../foreign.js".into();
    f.publish();
    assert!(f.read().is_err());
}

#[test]
fn links_cannot_redirect_the_selection_or_its_runtime_files() {
    for which in 0..2 {
        let f = Fixture::new();
        let target = if which == 0 {
            f.root.join("runtime/current.json")
        } else {
            f.paths.node.clone()
        };
        let outside = f.root.join("other-sentinel");
        fs::write(&outside, fs::read(&target).unwrap()).unwrap();
        fs::remove_file(&target).unwrap();
        std::os::unix::fs::symlink(&outside, &target).unwrap();
        assert!(f.read().is_err());
        assert!(outside.exists());
    }
}
