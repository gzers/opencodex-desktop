use super::*;
use std::{
    cell::Cell,
    fs,
    os::unix::fs::{symlink, PermissionsExt},
};
struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    client: PathBuf,
    vault: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::Builder::new()
            .prefix("ocxd024-tree-archive-")
            .tempdir()
            .unwrap();
        let root = temp.path().canonicalize().unwrap();
        let client = root.join("client");
        let vault = root.join("vault");
        for p in [&client, &vault] {
            fs::create_dir(p).unwrap();
            fs::set_permissions(p, fs::Permissions::from_mode(0o700)).unwrap();
        }
        Self {
            _temp: temp,
            root,
            client,
            vault,
        }
    }
    fn pins(&self) -> (PinnedDirectory, PinnedDirectory) {
        (
            PinnedDirectory::open(&self.client).unwrap(),
            PinnedDirectory::open(&self.vault).unwrap(),
        )
    }
}
#[test]
fn complete_tree_bytes_are_private_deduplicated_and_read_from_the_original_after_source_changes() {
    let f = Fixture::new();
    fs::create_dir(f.client.join("agents")).unwrap();
    fs::write(f.client.join("auth.json"), b"synthetic-credential").unwrap();
    fs::write(f.client.join("agents/AGENTS.md"), b"synthetic-credential").unwrap();
    fs::write(f.client.join("empty"), b"").unwrap();
    let (root, vault) = f.pins();
    let tree = TreeSnapshot::read(&root).unwrap();
    tree.archive(&root, &vault).unwrap();
    let entries = fs::read_dir(&f.vault).unwrap().count();
    assert_eq!(entries, 2, "same content has one immutable blob");
    tree.archive(&root, &vault).unwrap();
    assert_eq!(fs::read_dir(&f.vault).unwrap().count(), entries);
    for entry in fs::read_dir(&f.vault).unwrap() {
        let m = entry.unwrap().metadata().unwrap();
        assert_eq!(m.permissions().mode() & 0o777, 0o400);
    }
    fs::write(f.client.join("auth.json"), b"later input").unwrap();
    assert_eq!(
        tree.archived_file(&vault, "auth.json").unwrap().as_slice(),
        b"synthetic-credential"
    );
    assert!(tree.archive(&root, &vault).is_err());
    assert_eq!(
        fs::read(f.client.join("auth.json")).unwrap(),
        b"later input"
    );
    assert!(tree.archived_file(&vault, "../auth.json").is_err());
    assert!(tree.archived_file(&vault, "unknown").is_err());
}
#[test]
fn missing_partial_unsafe_or_redirected_content_never_gets_repaired_from_current_files() {
    for kind in ["missing", "partial", "mode", "symlink"] {
        let f = Fixture::new();
        let bytes = b"original source";
        fs::write(f.client.join("config.toml"), bytes).unwrap();
        let (root, vault) = f.pins();
        let tree = TreeSnapshot::read(&root).unwrap();
        tree.archive(&root, &vault).unwrap();
        let target = f.vault.join(blob_name(&digest(bytes)).unwrap());
        match kind {
            "missing" => fs::remove_file(&target).unwrap(),
            "partial" => {
                fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
                fs::write(&target, b"{interrupted").unwrap();
                fs::set_permissions(&target, fs::Permissions::from_mode(0o400)).unwrap();
            }
            "mode" => fs::set_permissions(&target, fs::Permissions::from_mode(0o644)).unwrap(),
            _ => {
                fs::remove_file(&target).unwrap();
                fs::write(f.root.join("outside"), bytes).unwrap();
                symlink(f.root.join("outside"), &target).unwrap();
            }
        }
        assert!(tree.verify_archive(&vault).is_err(), "{kind}");
        assert!(tree.archived_file(&vault, "config.toml").is_err(), "{kind}");
        // Reopening never creates bytes. Existing bad objects never heal.
        if kind != "missing" {
            assert!(tree.archive(&root, &vault).is_err(), "{kind}");
        }
        assert_eq!(fs::read(f.client.join("config.toml")).unwrap(), bytes);
        if kind == "missing" {
            assert!(!target.exists());
        }
        if kind == "partial" {
            assert_eq!(fs::read(&target).unwrap(), b"{interrupted");
        }
        if kind == "mode" {
            assert_eq!(
                fs::metadata(&target).unwrap().permissions().mode() & 0o777,
                0o644
            );
        }
    }
}
#[test]
fn changed_original_during_copy_retains_the_created_blob_but_refuses_a_new_archive() {
    let f = Fixture::new();
    let original = b"old original";
    fs::write(f.client.join("config.toml"), original).unwrap();
    let (root, vault) = f.pins();
    let tree = TreeSnapshot::read(&root).unwrap();
    let once = Cell::new(false);
    let result = tree.archive_checked(&root, &vault, &|point| {
        if point == "after_blob" && !once.replace(true) {
            fs::write(f.client.join("config.toml"), b"new input must stay")?;
        }
        Ok(())
    });
    assert!(result.is_err());
    assert!(once.get());
    assert_eq!(
        fs::read(f.vault.join(blob_name(&digest(original)).unwrap())).unwrap(),
        original
    );
    assert_eq!(
        fs::read(f.client.join("config.toml")).unwrap(),
        b"new input must stay"
    );
    assert!(tree.archive(&root, &vault).is_err());
}
#[test]
fn actual_process_exit_after_durable_blob_preserves_it_for_same_independent_tree() {
    const CHILD: &str = "OCXD024_TREE_BLOB_CRASH";
    const TEST:&str="infrastructure::macos_client_originals::tree::archive::tests::actual_process_exit_after_durable_blob_preserves_it_for_same_independent_tree";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        assert!(
            root.starts_with("/private/tmp")
                && root
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("ocxd024-tree-archive-")
        );
        let client = PinnedDirectory::open(&root.join("client")).unwrap();
        let vault = PinnedDirectory::open(&root.join("vault")).unwrap();
        let tree = TreeSnapshot::read(&client).unwrap();
        let _ = tree.archive_checked(&client, &vault, &|point| {
            if point == "after_blob" {
                std::process::exit(91);
            }
            Ok(())
        });
        panic!("crash point not reached");
    }
    let f = Fixture::new();
    fs::write(f.client.join("config.toml"), b"native original").unwrap();
    let (root, vault) = f.pins();
    let tree = TreeSnapshot::read(&root).unwrap();
    let independent = digest(&encode(&tree).unwrap());
    let p = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", TEST, "--nocapture"])
        .env(CHILD, &f.root)
        .output()
        .unwrap();
    assert_eq!(
        p.status.code(),
        Some(91),
        "{}",
        String::from_utf8_lossy(&p.stderr)
    );
    assert_eq!(digest(&encode(&tree).unwrap()), independent);
    tree.verify_archive(&vault).unwrap();
    tree.archive(&root, &vault).unwrap();
    assert_eq!(
        tree.archived_file(&vault, "config.toml")
            .unwrap()
            .as_slice(),
        b"native original"
    );
    println!("TREE_BYTE_ARCHIVE_CRASH exit=91 sameIndependentTree=true durableBlobVerified=true sourceUnchanged=true");
}
