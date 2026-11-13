use super::*;
#[test]
fn child_pin_inherits_same_authenticated_parent_and_remains_valid_after_parent_handle_drops() {
    use std::os::unix::fs::DirBuilderExt;
    let temp = tempfile::tempdir().unwrap();
    let path = temp
        .path()
        .canonicalize()
        .unwrap()
        .join("parent 中文 space");
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&path)
        .unwrap();
    let parent = PinnedDirectory::open(&path).unwrap();
    let child = parent.child_private("leaf", true).unwrap();
    let reference =
        observation_metrics::child_reference(|| parent.child_private("leaf", false)).unwrap();
    assert_eq!(child.path(), reference.path());
    assert!(child.identity() == reference.identity());
    assert!(
        protection(child.file(), &stat(child.file()).unwrap()).unwrap()
            == protection(reference.file(), &stat(reference.file()).unwrap()).unwrap()
    );
    assert_eq!(child.chain.len(), parent.chain.len() + 1);
    for ((a, id, security), (b, oldid, oldsecurity)) in child.chain.iter().zip(&parent.chain) {
        assert!(id == oldid && security == oldsecurity);
        assert_ne!(a.as_raw_fd(), b.as_raw_fd());
        let flags = unsafe { libc::fcntl(a.as_raw_fd(), libc::F_GETFD) };
        assert!(flags >= 0);
        assert_ne!(flags & libc::FD_CLOEXEC, 0);
    }
    drop(parent);
    child.private().unwrap();
    reference.private().unwrap();
}
#[test]
fn child_pin_keeps_original_final_parent_check_after_child_security_was_read() {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    for change in ["mode", "rename", "xattr"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = root.join("parent");
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .unwrap();
        let parent = PinnedDirectory::open(&path).unwrap();
        let reached = std::cell::Cell::new(false);
        let result = parent.child_private_checked("child", true, &|point| {
            if point == "child-chain-checked" {
                reached.set(true);
                match change {
                    "mode" => {
                        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
                            .unwrap()
                    }
                    "rename" => {
                        std::fs::rename(&path, root.join("retained-parent")).unwrap();
                        std::fs::DirBuilder::new()
                            .mode(0o700)
                            .create(&path)
                            .unwrap();
                    }
                    "xattr" => {
                        let p = CString::new(path.as_os_str().as_bytes()).unwrap();
                        let n = CString::new("user.ocxd-child-tail").unwrap();
                        assert_eq!(
                            unsafe {
                                libc::setxattr(
                                    p.as_ptr(),
                                    n.as_ptr(),
                                    b"x".as_ptr().cast(),
                                    1,
                                    0,
                                    0,
                                )
                            },
                            0
                        );
                    }
                    _ => unreachable!(),
                }
            }
            Ok(())
        });
        assert!(reached.get());
        assert!(result.is_err(), "{change}");
    }
}
#[test]
fn child_pin_refuses_parent_or_child_changes_at_both_acquisition_boundaries() {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    for stage in ["parent-admitted", "parent-handles-cloned", "child-opened"] {
        for case in [
            "parent-rename",
            "parent-symlink",
            "parent-mode",
            "parent-xattr",
            "child-replace",
            "child-mode",
        ] {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().canonicalize().unwrap();
            let parent_path = root.join("parent");
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(&parent_path)
                .unwrap();
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(parent_path.join("child"))
                .unwrap();
            let parent = PinnedDirectory::open(&parent_path).unwrap();
            let reached = std::cell::Cell::new(false);
            let result = parent.child_private_checked("child", false, &|point| {
                if point == stage {
                    reached.set(true);
                    match case {
                        "parent-rename" => {
                            std::fs::rename(&parent_path, root.join("retained-parent")).unwrap();
                            std::fs::DirBuilder::new()
                                .mode(0o700)
                                .create(&parent_path)
                                .unwrap();
                            std::fs::DirBuilder::new()
                                .mode(0o700)
                                .create(parent_path.join("child"))
                                .unwrap();
                        }
                        "parent-symlink" => {
                            std::fs::rename(&parent_path, root.join("retained-parent")).unwrap();
                            std::os::unix::fs::symlink(root.join("retained-parent"), &parent_path)
                                .unwrap();
                        }
                        "parent-mode" => std::fs::set_permissions(
                            &parent_path,
                            std::fs::Permissions::from_mode(0o755),
                        )
                        .unwrap(),
                        "parent-xattr" => {
                            let p = CString::new(parent_path.as_os_str().as_bytes()).unwrap();
                            let n = CString::new("user.ocxd-child-pin").unwrap();
                            assert_eq!(
                                unsafe {
                                    libc::setxattr(
                                        p.as_ptr(),
                                        n.as_ptr(),
                                        b"x".as_ptr().cast(),
                                        1,
                                        0,
                                        0,
                                    )
                                },
                                0
                            );
                        }
                        "child-replace" => {
                            // Before the child was observed, an otherwise valid new
                            // directory is not a stale child pin. Test replacement
                            // ONLY after that original child fd was captured.
                            if stage == "child-opened" {
                                std::fs::rename(
                                    parent_path.join("child"),
                                    parent_path.join("retained-child"),
                                )
                                .unwrap();
                                std::fs::DirBuilder::new()
                                    .mode(0o700)
                                    .create(parent_path.join("child"))
                                    .unwrap();
                            } else {
                                std::fs::set_permissions(
                                    parent_path.join("child"),
                                    std::fs::Permissions::from_mode(0o755),
                                )
                                .unwrap();
                            }
                        }
                        "child-mode" => std::fs::set_permissions(
                            parent_path.join("child"),
                            std::fs::Permissions::from_mode(0o755),
                        )
                        .unwrap(),
                        _ => unreachable!(),
                    }
                }
                Ok(())
            });
            assert!(reached.get());
            assert!(result.is_err(), "{stage}/{case}");
        }
    }
}
#[test]
fn child_pin_keeps_existing_unsafe_objects_and_literal_component_rules() {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().canonicalize().unwrap();
    let path = root.join("parent");
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&path)
        .unwrap();
    let parent = PinnedDirectory::open(&path).unwrap();
    for bad in [".", "..", "a/b", "/absolute", ""] {
        assert!(parent.child_private(bad, false).is_err());
    }
    let outside = root.join("outside");
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&outside)
        .unwrap();
    std::fs::write(outside.join("keep"), b"outside bytes").unwrap();
    std::os::unix::fs::symlink(&outside, path.join("linked")).unwrap();
    assert!(parent.child_private("linked", true).is_err());
    assert!(std::fs::symlink_metadata(path.join("linked"))
        .unwrap()
        .file_type()
        .is_symlink());
    std::fs::write(path.join("file"), b"keep file").unwrap();
    assert!(parent.child_private("file", true).is_err());
    assert_eq!(std::fs::read(path.join("file")).unwrap(), b"keep file");
    std::fs::DirBuilder::new()
        .mode(0o755)
        .create(path.join("unsafe"))
        .unwrap();
    assert!(parent.child_private("unsafe", true).is_err());
    assert_eq!(
        std::fs::metadata(path.join("unsafe"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o755
    );
    assert_eq!(
        std::fs::read(outside.join("keep")).unwrap(),
        b"outside bytes"
    );
    let literal = parent.child_private("back\\slash", true).unwrap();
    assert_eq!(literal.path(), path.join("back\\slash"));
    literal.private().unwrap();
}
#[test]
fn pinned_set_rejects_conflicting_inode_or_protection_in_either_input_order() {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    for change in ["inode", "mode"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let path = root.join("source");
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&path)
            .unwrap();
        let old = PinnedDirectory::open(&path).unwrap();
        if change == "inode" {
            std::fs::rename(&path, root.join("retained")).unwrap();
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(&path)
                .unwrap();
        } else {
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let current = PinnedDirectory::open(&path).unwrap();
        for pins in [[&old, &current], [&current, &old]] {
            let error = PinnedDirectory::recheck_set(&pins).unwrap_err();
            assert_eq!(
                error.to_string(),
                "directory union has conflicting source pins"
            );
        }
    }
}
#[test]
fn pinned_set_checks_shared_ancestors_again_after_first_pass_mutation() {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    for change in ["mode", "rename", "symlink", "xattr"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let parent = root.join("shared");
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&parent)
            .unwrap();
        for n in ["one", "two"] {
            std::fs::DirBuilder::new()
                .mode(0o700)
                .create(parent.join(n))
                .unwrap();
        }
        let one = PinnedDirectory::open(&parent.join("one")).unwrap();
        let two = PinnedDirectory::open(&parent.join("two")).unwrap();
        let touched = std::cell::Cell::new(false);
        let boundary = parent
            .components()
            .filter(|c| matches!(c, Component::Normal(_)))
            .count();
        let result = PinnedDirectory::recheck_set_checked(&[&one, &two], &|i| {
            if i == boundary && !touched.replace(true) {
                match change {
                    "mode" => {
                        std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o755))
                            .unwrap()
                    }
                    "rename" => {
                        std::fs::rename(&parent, root.join("retained-shared")).unwrap();
                        std::fs::DirBuilder::new()
                            .mode(0o700)
                            .create(&parent)
                            .unwrap();
                    }
                    "symlink" => {
                        std::fs::rename(&parent, root.join("retained-shared")).unwrap();
                        std::os::unix::fs::symlink(root.join("retained-shared"), &parent).unwrap();
                    }
                    "xattr" => {
                        let p = CString::new(parent.as_os_str().as_bytes()).unwrap();
                        let name = CString::new("user.ocxd-prefix-union").unwrap();
                        assert_eq!(
                            unsafe {
                                libc::setxattr(
                                    p.as_ptr(),
                                    name.as_ptr(),
                                    b"x".as_ptr().cast(),
                                    1,
                                    0,
                                    0,
                                )
                            },
                            0
                        );
                    }
                    _ => unreachable!(),
                }
            }
            Ok(())
        });
        assert!(touched.get());
        assert!(result.is_err(), "{change}");
    }
}
#[test]
fn pinned_set_preserves_literal_paths_unique_branches_and_reobserves_every_call() {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    let temp = tempfile::tempdir().unwrap();
    let parent = temp.path().canonicalize().unwrap().join("中文 space");
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&parent)
        .unwrap();
    for n in ["back\\slash", "other"] {
        std::fs::DirBuilder::new()
            .mode(0o700)
            .create(parent.join(n))
            .unwrap();
    }
    let root = PinnedDirectory::open(&parent).unwrap();
    let one = PinnedDirectory::open(&parent.join("back\\slash")).unwrap();
    let two = PinnedDirectory::open(&parent.join("other")).unwrap();
    for pins in [[&root, &one, &two, &one], [&two, &one, &root, &root]] {
        PinnedDirectory::recheck_set(&pins).unwrap();
    }
    assert!(PinnedDirectory::open(&parent.join(".")).is_err());
    assert!(PinnedDirectory::open(Path::new("relative")).is_err());
    std::fs::set_permissions(two.path(), std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(PinnedDirectory::recheck_set(&[&root, &one, &two]).is_err());
}
#[test]
fn pinned_directory_recheck_refuses_xattr_change_after_ancestor_attribute_read() {
    use std::os::unix::fs::DirBuilderExt;
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap().join("owned");
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .unwrap();
    let leaf = root.join("leaf");
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&leaf)
        .unwrap();
    let pin = PinnedDirectory::open(&leaf).unwrap();
    let ancestor = pin.chain.len() - 2;
    let key = CString::new("user.ocxd024-in-scan").unwrap();
    let value = b"changed during scan";
    assert!(pin
        .recheck_checked(&|i| {
            if i == ancestor
                && unsafe {
                    libc::fsetxattr(
                        pin.chain[i].0.as_raw_fd(),
                        key.as_ptr(),
                        value.as_ptr().cast(),
                        value.len(),
                        0,
                        0,
                    )
                } != 0
            {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        })
        .is_err());
    assert!(xattrs(&pin.chain[ancestor].0)
        .unwrap()
        .iter()
        .any(|(k, v)| k == b"user.ocxd024-in-scan" && v == value));
}
#[test]
fn pinned_directory_recheck_retains_inode_attributes_and_rejects_nofollow_replacement() {
    use std::os::unix::fs::{symlink, DirBuilderExt, PermissionsExt};
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap().join("owned");
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .unwrap();
    let leaf = root.join("leaf");
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&leaf)
        .unwrap();
    let pin = PinnedDirectory::open(&leaf).unwrap();
    pin.recheck().unwrap();
    std::fs::write(leaf.join("allowed-content"), b"same directory").unwrap();
    pin.recheck().unwrap();
    std::fs::set_permissions(&leaf, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(pin.recheck().is_err());
    std::fs::set_permissions(&leaf, std::fs::Permissions::from_mode(0o700)).unwrap();
    pin.recheck().unwrap();
    let retained = root.join("retained");
    std::fs::rename(&leaf, &retained).unwrap();
    symlink(&retained, &leaf).unwrap();
    assert!(pin.recheck().is_err());
    std::fs::remove_file(&leaf).unwrap();
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&leaf)
        .unwrap();
    assert!(pin.recheck().is_err());
    assert_eq!(
        std::fs::read(retained.join("allowed-content")).unwrap(),
        b"same directory"
    );
}
#[test]
fn pinned_directory_recheck_rejects_ancestor_replacement_during_attribute_scan() {
    use std::os::unix::fs::DirBuilderExt;
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path().canonicalize().unwrap().join("owned");
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&root)
        .unwrap();
    let leaf = root.join("leaf");
    std::fs::DirBuilder::new()
        .mode(0o700)
        .create(&leaf)
        .unwrap();
    let pin = PinnedDirectory::open(&leaf).unwrap();
    let last = pin.chain.len() - 1;
    let retained = root.with_file_name("retained-owned");
    assert!(pin
        .recheck_checked(&|i| {
            if i == last {
                std::fs::rename(&root, &retained)?;
                std::fs::DirBuilder::new().mode(0o700).create(&root)?;
                std::fs::DirBuilder::new()
                    .mode(0o700)
                    .create(root.join("leaf"))?;
            }
            Ok(())
        })
        .is_err());
    assert!(retained.join("leaf").is_dir());
    assert!(leaf.is_dir());
}
use std::{
    fs,
    os::unix::fs::{symlink, OpenOptionsExt, PermissionsExt},
};

struct Fixture {
    _temp: tempfile::TempDir,
    root: PathBuf,
    data: PathBuf,
    client: PathBuf,
}
impl Fixture {
    fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let data = root.join("desktop");
        let client = root.join("client 中文 space");
        fs::create_dir(&data).unwrap();
        fs::set_permissions(&data, fs::Permissions::from_mode(0o700)).unwrap();
        fs::create_dir(&client).unwrap();
        fs::set_permissions(&client, fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(client.join(NAMES[0]), b"original catalog bytes").unwrap();
        fs::set_permissions(client.join(NAMES[0]), fs::Permissions::from_mode(0o640)).unwrap();
        Self {
            _temp: temp,
            root,
            data,
            client,
        }
    }
    fn journal(&self, id: &str) -> PathBuf {
        self.data.join("manager-state/client-originals").join(id)
    }
}

#[test]
fn native_snapshot_retains_exact_original_bytes_and_metadata_without_source_writes() {
    let f = Fixture::new();
    let dir = PinnedDirectory::open(&f.client).unwrap();
    let before = Capture::read(&dir).unwrap();
    assert_eq!(
        before.images()[0].bytes(),
        Some(b"original catalog bytes".as_slice())
    );
    assert_eq!(before.images()[1], Image::Absent);
    assert_eq!(Capture::read(&dir).unwrap(), before);
    assert!(!format!("{before:?}").contains("original catalog bytes"));
    let originals = Originals::begin(&f.data, &f.client).unwrap();
    assert_eq!(originals.read(Phase::Original).unwrap(), before);
    let raw = fs::read(f.journal(originals.attempt()).join("original.json")).unwrap();
    assert_eq!(digest(&raw), originals.original_sha256());
    assert_eq!(
        fs::metadata(f.client.join(NAMES[0]))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o640
    );
    assert_eq!(
        fs::metadata(f.journal(originals.attempt()).join("original.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o400
    );
}

#[test]
fn original_frame_binds_client_tree_file_hashes_without_recording_their_content_bytes() {
    let f = Fixture::new();
    fs::create_dir(f.client.join("config")).unwrap();
    fs::write(
        f.client.join("config/config.toml"),
        b"token = 'tree-secret'\n",
    )
    .unwrap();
    fs::create_dir(f.client.join("agents")).unwrap();
    fs::write(f.client.join("agents/AGENTS.md"), b"private instruction\n").unwrap();
    let originals = Originals::begin(&f.data, &f.client).unwrap();
    let raw = fs::read(f.journal(originals.attempt()).join("original.json")).unwrap();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&raw).unwrap()["schemaVersion"],
        4
    );
    let tree = serde_json::from_slice::<serde_json::Value>(&raw).unwrap()["tree"].clone();
    assert!(tree["entries"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == "config/config.toml"));
    assert!(tree["entries"]
        .as_array()
        .unwrap()
        .iter()
        .any(|entry| entry["path"] == "agents/AGENTS.md"));
    assert!(!raw
        .windows(b"tree-secret".len())
        .any(|bytes| bytes == b"tree-secret"));
    assert!(!raw
        .windows(b"private instruction".len())
        .any(|bytes| bytes == b"private instruction"));
    assert!(
        TreeSnapshot::read(&PinnedDirectory::open(&f.client).unwrap())
            .unwrap()
            .entry_count()
            >= 4
    );
}

#[test]
fn complete_tree_capture_refuses_symlink_without_following_it() {
    let f = Fixture::new();
    let outside = f.root.join("outside-secret");
    fs::write(&outside, b"outside").unwrap();
    symlink(&outside, f.client.join("linked-secret")).unwrap();
    assert!(Originals::begin(&f.data, &f.client).is_err());
    assert_eq!(fs::read(&outside).unwrap(), b"outside");
}

#[test]
fn old_frames_and_tree_references_with_invalid_inventory_are_not_backfilled_or_adopted() {
    let f = Fixture::new();
    let originals = Originals::begin(&f.data, &f.client).unwrap();
    let path = f.journal(originals.attempt()).join("original.json");
    let before = fs::read(&path).unwrap();
    let value: serde_json::Value = serde_json::from_slice(&before).unwrap();
    let mut old = value.clone();
    old["schemaVersion"] = 1.into();
    old.as_object_mut().unwrap().remove("tree");
    assert!(decode(&encode(&old).unwrap()).is_err());
    let mut inventory_only = value.clone();
    inventory_only["schemaVersion"] = 2.into();
    assert!(decode(&encode(&inventory_only).unwrap()).is_err());
    let mut codex_only = value.clone();
    codex_only["schemaVersion"] = 3.into();
    codex_only.as_object_mut().unwrap().remove("grok");
    codex_only.as_object_mut().unwrap().remove("profileRoot");
    assert!(decode(&encode(&codex_only).unwrap()).is_err());
    let mut invalid = value;
    invalid["tree"]["totalBytes"] = 0.into();
    let bytes = encode(&invalid).unwrap();
    // Independently supplied hashes cannot make structurally invalid tree
    // evidence valid. This test supplies its own synthetic bad reference only.
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, &bytes).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(Originals::open(
        &f.data,
        &f.client,
        originals.attempt(),
        &digest(&bytes),
        &[]
    )
    .is_err());
    assert_eq!(fs::read(&path).unwrap(), bytes);
}

#[test]
fn phase_archive_reopens_original_config_and_agent_bytes_and_refuses_missing_blobs_without_repair()
{
    let f = Fixture::new();
    fs::create_dir(f.client.join("agents")).unwrap();
    fs::write(f.client.join("config.toml"), b"model='native'\n").unwrap();
    fs::write(
        f.client.join("agents/AGENTS.md"),
        b"original private instructions",
    )
    .unwrap();
    let originals = Originals::begin(&f.data, &f.client).unwrap();
    let attempt = originals.attempt().to_owned();
    let original = originals.original_sha256().to_owned();
    fs::write(f.client.join("config.toml"), b"model='routed'\n").unwrap();
    let ready = originals.seal(Phase::Ready).unwrap();
    let reopened = Originals::open(
        &f.data,
        &f.client,
        &attempt,
        &original,
        &[(Phase::Ready, ready)],
    )
    .unwrap();
    assert_eq!(
        reopened
            .archived_tree_file(Phase::Original, "config.toml")
            .unwrap()
            .as_slice(),
        b"model='native'\n"
    );
    assert_eq!(
        reopened
            .archived_tree_file(Phase::Ready, "config.toml")
            .unwrap()
            .as_slice(),
        b"model='routed'\n"
    );
    assert_eq!(
        reopened
            .archived_tree_file(Phase::Original, "agents/AGENTS.md")
            .unwrap()
            .as_slice(),
        b"original private instructions"
    );
    let missing = f.journal(&attempt).join(format!(
        "tree-content-{}.blob",
        digest(b"original private instructions")
    ));
    fs::remove_file(&missing).unwrap();
    assert!(Originals::open(&f.data, &f.client, &attempt, &original, &[]).is_err());
    assert!(reopened.seal(Phase::PreStop).is_err());
    assert!(!missing.exists());
    assert_eq!(
        fs::read(f.client.join("config.toml")).unwrap(),
        b"model='routed'\n"
    );
}

#[test]
fn sequential_frames_reopen_only_with_original_and_independently_retained_phase_digests() {
    let f = Fixture::new();
    let originals = Originals::begin(&f.data, &f.client).unwrap();
    fs::write(f.client.join(NAMES[1]), b"ready cache").unwrap();
    let ready = originals.seal(Phase::Ready).unwrap();
    let pre = originals.seal(Phase::PreStop).unwrap();
    fs::remove_file(f.client.join(NAMES[1])).unwrap();
    let stopped = originals.seal(Phase::Stopped).unwrap();
    let reopened = Originals::open(
        &f.data,
        &f.client,
        originals.attempt(),
        originals.original_sha256(),
        &[
            (Phase::Ready, ready.clone()),
            (Phase::PreStop, pre),
            (Phase::Stopped, stopped),
        ],
    )
    .unwrap();
    assert_eq!(
        reopened.read(Phase::Ready).unwrap().images()[1].bytes(),
        Some(b"ready cache".as_slice())
    );
    assert_eq!(
        reopened.read(Phase::Stopped).unwrap().images()[1],
        Image::Absent
    );
    let original_only = Originals::open(
        &f.data,
        &f.client,
        originals.attempt(),
        originals.original_sha256(),
        &[],
    )
    .unwrap();
    assert!(original_only.read(Phase::Ready).is_err());
    assert!(Originals::open(
        &f.data,
        &f.client,
        originals.attempt(),
        originals.original_sha256(),
        &[(Phase::Ready, "a".repeat(64))]
    )
    .is_err());
    assert!(reopened.seal(Phase::Ready).is_err());
    assert_eq!(
        originals.read(Phase::Original).unwrap().images()[0].bytes(),
        Some(b"original catalog bytes".as_slice())
    );
}

#[test]
fn one_fresh_prefix_validates_each_frame_once_and_rejects_any_later_corruption() {
    let f = Fixture::new();
    let originals = Originals::begin(&f.data, &f.client).unwrap();
    let mut references = Vec::new();
    for phase in [Phase::Ready, Phase::PreStop, Phase::Stopped] {
        fs::write(
            f.client.join("config.toml"),
            format!("unique-{}", phase.index()),
        )
        .unwrap();
        references.push((phase, originals.seal(phase).unwrap()));
    }
    let observed = std::cell::RefCell::new(Vec::new());
    let frames = originals
        .frames_through_checked(Phase::Stopped, &|phase| {
            observed.borrow_mut().push(phase);
            Ok(())
        })
        .unwrap();
    assert_eq!(
        *observed.borrow(),
        vec![
            Phase::Original,
            Phase::Ready,
            Phase::PreStop,
            Phase::Stopped
        ]
    );
    assert_eq!(frames.len(), 4);
    for phase in [
        Phase::Original,
        Phase::Ready,
        Phase::PreStop,
        Phase::Stopped,
    ] {
        let path = originals.directory.path().join(phase.filename());
        let raw = fs::read(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, b"{}").unwrap();
        assert!(
            originals.frames_through(Phase::Stopped).is_err(),
            "phase {} not revalidated",
            phase.index()
        );
        assert!(Originals::open(
            &f.data,
            &f.client,
            originals.attempt(),
            originals.original_sha256(),
            &references
        )
        .is_err());
        assert_eq!(fs::read(&path).unwrap(), b"{}");
        fs::write(&path, &raw).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
        assert!(originals.frames_through(Phase::Stopped).is_ok());
    }
    // Permuted independent references still require the full highest prefix,
    // never skip the final phase because the input's last element is earlier.
    references.swap(0, 2);
    Originals::open(
        &f.data,
        &f.client,
        originals.attempt(),
        originals.original_sha256(),
        &references,
    )
    .unwrap();
    for content in [
        b"unique-1".as_slice(),
        b"unique-2".as_slice(),
        b"unique-3".as_slice(),
    ] {
        let path = originals
            .directory
            .path()
            .join(format!("tree-content-{}.blob", digest(content)));
        let raw = fs::read(&path).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        fs::write(&path, b"altered archive").unwrap();
        assert!(originals.frames_through(Phase::Stopped).is_err());
        assert!(Originals::open(
            &f.data,
            &f.client,
            originals.attempt(),
            originals.original_sha256(),
            &references
        )
        .is_err());
        fs::write(&path, &raw).unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    }
    assert!(originals.frames_through(Phase::Stopped).is_ok());
}
#[test]
fn fresh_prefix_per_frame_context_rechecks_xattr_mode_and_replaced_root() {
    for case in ["xattr", "mode", "replacement"] {
        let f = Fixture::new();
        let originals = Originals::begin(&f.data, &f.client).unwrap();
        for phase in [Phase::Ready, Phase::PreStop, Phase::Stopped] {
            originals.seal(phase).unwrap();
        }
        let observed = std::cell::Cell::new(0usize);
        assert!(originals
            .frames_through_checked(Phase::Stopped, &|phase| {
                observed.set(observed.get() + 1);
                if phase == Phase::Original {
                    match case {
                        "xattr" => {
                            let name = CString::new("user.ocxd024-prefix-boundary").unwrap();
                            let bytes = b"must refuse";
                            assert_eq!(
                                unsafe {
                                    libc::fsetxattr(
                                        originals.client.file().as_raw_fd(),
                                        name.as_ptr(),
                                        bytes.as_ptr().cast(),
                                        bytes.len(),
                                        0,
                                        0,
                                    )
                                },
                                0,
                                "xattr injection must succeed independently"
                            );
                        }
                        "mode" => fs::set_permissions(&f.client, fs::Permissions::from_mode(0o755))
                            .unwrap(),
                        "replacement" => {
                            fs::rename(&f.client, f.root.join("retained-client")).unwrap();
                            fs::create_dir(&f.client).unwrap();
                            fs::set_permissions(&f.client, fs::Permissions::from_mode(0o700))
                                .unwrap();
                        }
                        _ => unreachable!(),
                    }
                }
                Ok(())
            })
            .is_err());
        assert_eq!(
            observed.get(),
            1,
            "next frame must refuse after a successful context mutation"
        );
        if case == "replacement" {
            assert_eq!(
                fs::read(f.root.join("retained-client").join(NAMES[0])).unwrap(),
                b"original catalog bytes"
            );
        }
    }
}
#[test]
fn fresh_prefix_per_frame_dynamic_check_refuses_permission_change_before_planned_restore() {
    let f = Fixture::new();
    let originals = Originals::begin(&f.data, &f.client).unwrap();
    for phase in [Phase::Ready, Phase::PreStop, Phase::Stopped] {
        originals.seal(phase).unwrap();
    }
    let changed = std::cell::Cell::new(false);
    let restored = std::cell::Cell::new(false);
    assert!(originals
        .frames_through_checked(Phase::Stopped, &|phase| {
            if phase == Phase::Original {
                fs::set_permissions(&f.client, fs::Permissions::from_mode(0o755)).unwrap();
                changed.set(true);
            }
            if phase == Phase::PreStop {
                fs::set_permissions(&f.client, fs::Permissions::from_mode(0o700)).unwrap();
                restored.set(true);
            }
            Ok(())
        })
        .is_err());
    assert!(changed.get());
    assert!(
        !restored.get(),
        "must refuse at Ready before an altered context can be restored later"
    );
    assert_eq!(
        fs::metadata(&f.client).unwrap().permissions().mode() & 0o7777,
        0o755
    );
}

#[test]
fn skipped_phase_and_second_original_are_never_a_new_baseline() {
    let f = Fixture::new();
    let originals = Originals::begin(&f.data, &f.client).unwrap();
    let path = f.journal(originals.attempt()).join("original.json");
    let bytes = fs::read(&path).unwrap();
    assert!(originals.seal(Phase::Original).is_err());
    assert!(originals.seal(Phase::PreStop).is_err());
    assert_eq!(fs::read(path).unwrap(), bytes);
    assert!(!f
        .journal(originals.attempt())
        .join("pre-stop.json")
        .exists());
}

#[test]
fn same_bytes_replaced_inode_is_distinct_and_source_aliases_are_refused() {
    let f = Fixture::new();
    let dir = PinnedDirectory::open(&f.client).unwrap();
    let before = Capture::read(&dir).unwrap();
    let p = f.client.join(NAMES[0]);
    let old = f.root.join("saved");
    fs::rename(&p, &old).unwrap();
    fs::write(&p, b"original catalog bytes").unwrap();
    assert_ne!(Capture::read(&dir).unwrap(), before);
    fs::remove_file(&p).unwrap();
    fs::hard_link(&old, &p).unwrap();
    assert!(Capture::read(&dir).is_err());
    fs::remove_file(&p).unwrap();
    symlink(&old, &p).unwrap();
    assert!(Capture::read(&dir).is_err());
    assert_eq!(fs::read(old).unwrap(), b"original catalog bytes");
}

#[test]
fn replaced_parent_is_refused_even_if_its_contents_are_identical() {
    let f = Fixture::new();
    let originals = Originals::begin(&f.data, &f.client).unwrap();
    let moved = f.root.join("original-client");
    fs::rename(&f.client, &moved).unwrap();
    fs::create_dir(&f.client).unwrap();
    fs::copy(moved.join(NAMES[0]), f.client.join(NAMES[0])).unwrap();
    assert!(originals.seal(Phase::Ready).is_err());
    assert!(originals.read(Phase::Original).is_err());
    assert!(Originals::open(
        &f.data,
        &f.client,
        originals.attempt(),
        originals.original_sha256(),
        &[]
    )
    .is_err());
}

#[test]
fn source_directory_permission_change_refuses_without_tightening_it() {
    let f = Fixture::new();
    let originals = Originals::begin(&f.data, &f.client).unwrap();
    fs::set_permissions(&f.client, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(originals.read(Phase::Original).is_err());
    assert!(originals.seal(Phase::Ready).is_err());
    assert_eq!(
        fs::metadata(&f.client).unwrap().permissions().mode() & 0o777,
        0o755
    );
}

#[test]
fn data_and_client_overlap_and_missing_client_directory_are_not_empty_originals() {
    let f = Fixture::new();
    assert!(Originals::begin(&f.client, &f.client).is_err());
    assert!(Originals::begin(&f.data, &f.data.join("nested")).is_err());
    assert!(Originals::begin(&f.data, &f.root.join("does-not-exist")).is_err());
    assert!(!f.data.join("manager-state").exists());
}

#[test]
fn missing_file_is_absent_but_fifo_directory_and_nonreadable_file_are_errors() {
    let f = Fixture::new();
    let dir = PinnedDirectory::open(&f.client).unwrap();
    let target = f.client.join(NAMES[1]);
    assert_eq!(Capture::read(&dir).unwrap().images()[1], Image::Absent);
    fs::create_dir(&target).unwrap();
    assert!(Capture::read(&dir).is_err());
    fs::remove_dir(&target).unwrap();
    let p = CString::new(target.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(p.as_ptr(), 0o600) }, 0);
    assert!(Capture::read(&dir).is_err());
    fs::remove_file(&target).unwrap();
    fs::write(&target, b"private").unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o000)).unwrap();
    assert!(Capture::read(&dir).is_err());
    fs::set_permissions(&target, fs::Permissions::from_mode(0o600)).unwrap();
}

#[test]
fn extended_attributes_are_original_bytes_and_changes_invalidate_snapshots() {
    let f = Fixture::new();
    let target = f.client.join(NAMES[0]);
    let file = File::open(&target).unwrap();
    let name = CString::new("user.ocxd024-test").unwrap();
    let value = b"exact opaque xattr bytes";
    assert_eq!(
        unsafe {
            libc::fsetxattr(
                file.as_raw_fd(),
                name.as_ptr(),
                value.as_ptr().cast(),
                value.len(),
                0,
                0,
            )
        },
        0
    );
    let originals = Originals::begin(&f.data, &f.client).unwrap();
    let captured = originals.read(Phase::Original).unwrap();
    let Image::File { protection, .. } = &captured.images()[0] else {
        panic!("file missing")
    };
    assert!(protection
        .xattrs
        .contains(&(name.as_bytes().to_vec(), value.to_vec())));
    let changed = b"third-party attribute";
    assert_eq!(
        unsafe {
            libc::fsetxattr(
                file.as_raw_fd(),
                name.as_ptr(),
                changed.as_ptr().cast(),
                changed.len(),
                0,
                0,
            )
        },
        0
    );
    assert_ne!(
        Capture::read(&PinnedDirectory::open(&f.client).unwrap()).unwrap(),
        captured
    );
    assert_eq!(originals.read(Phase::Original).unwrap(), captured);
}

#[test]
fn native_acl_is_captured_without_chmod_or_acl_repair() {
    let f = Fixture::new();
    let target = f.client.join(NAMES[0]);
    let uid = unsafe { libc::geteuid() };
    let identity = unsafe { libc::getpwuid(uid) };
    assert!(!identity.is_null());
    let username = unsafe { std::ffi::CStr::from_ptr((*identity).pw_name) }
        .to_str()
        .unwrap();
    let ace = format!("user:{username} allow read");
    let result = std::process::Command::new("/bin/chmod")
        .args(["+a", &ace])
        .arg(&target)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let before = Capture::read(&PinnedDirectory::open(&f.client).unwrap()).unwrap();
    let Image::File { protection, .. } = &before.images()[0] else {
        panic!("file missing")
    };
    assert!(protection.acl.as_ref().is_some_and(|b| !b.is_empty()));
    let originals = Originals::begin(&f.data, &f.client).unwrap();
    assert_eq!(originals.read(Phase::Original).unwrap(), before);
    assert_eq!(
        Capture::read(&PinnedDirectory::open(&f.client).unwrap()).unwrap(),
        before
    );
}

#[test]
fn insecure_existing_data_directory_or_journal_file_is_not_repaired() {
    let f = Fixture::new();
    fs::set_permissions(&f.data, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(Originals::begin(&f.data, &f.client).is_err());
    assert!(!f.data.join("manager-state").exists());
    fs::set_permissions(&f.data, fs::Permissions::from_mode(0o700)).unwrap();
    let originals = Originals::begin(&f.data, &f.client).unwrap();
    let path = f.journal(originals.attempt()).join("original.json");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    let before = fs::read(&path).unwrap();
    assert!(originals.read(Phase::Original).is_err());
    assert_eq!(
        fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn unknown_private_journal_attribute_is_refused_and_preserved() {
    let f = Fixture::new();
    let root = File::open(&f.data).unwrap();
    let key = CString::new("user.unrecognized").unwrap();
    let bytes = b"do not erase";
    assert_eq!(
        unsafe {
            libc::fsetxattr(
                root.as_raw_fd(),
                key.as_ptr(),
                bytes.as_ptr().cast(),
                bytes.len(),
                0,
                0,
            )
        },
        0
    );
    assert!(Originals::begin(&f.data, &f.client).is_err());
    assert!(!f.data.join("manager-state").exists());
    assert_eq!(
        xattrs(&root).unwrap().get(key.as_bytes()),
        Some(&bytes.to_vec())
    );
}

#[test]
fn journal_ancestor_link_does_not_publish_into_an_outside_directory() {
    let f = Fixture::new();
    let outside = f.root.join("outside");
    fs::create_dir(&outside).unwrap();
    symlink(&outside, f.data.join("manager-state")).unwrap();
    assert!(Originals::begin(&f.data, &f.client).is_err());
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
    assert_eq!(
        fs::read(f.client.join(NAMES[0])).unwrap(),
        b"original catalog bytes"
    );
}

#[test]
fn tampered_frame_duplicate_json_and_replaced_original_are_rejected() {
    let f = Fixture::new();
    let originals = Originals::begin(&f.data, &f.client).unwrap();
    let path = f.journal(originals.attempt()).join("original.json");
    let original = fs::read(&path).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, b"{\"schemaVersion\":1,\"schemaVersion\":1}").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o400)).unwrap();
    assert!(originals.read(Phase::Original).is_err());
    assert!(decode(b"{\"schemaVersion\":1,\"schemaVersion\":1}").is_err());
    fs::remove_file(&path).unwrap();
    symlink(f.client.join(NAMES[0]), &path).unwrap();
    assert!(originals.read(Phase::Original).is_err());
    assert_eq!(
        fs::read(f.client.join(NAMES[0])).unwrap(),
        b"original catalog bytes"
    );
    assert!(!original.is_empty());
}

#[test]
fn source_changed_between_capture_and_publication_leaves_no_ready_frame() {
    let f = Fixture::new();
    let originals = Originals::begin(&f.data, &f.client).unwrap();
    assert!(originals
        .seal_checked(Phase::Ready, || {
            fs::write(f.client.join(NAMES[1]), b"outside writer")?;
            Ok(())
        })
        .is_err());
    assert!(!f.journal(originals.attempt()).join("ready.json").exists());
    assert_eq!(
        fs::read(f.client.join(NAMES[1])).unwrap(),
        b"outside writer"
    );
}

#[test]
fn oversize_original_is_not_truncated_or_persisted() {
    let f = Fixture::new();
    let file = File::create(f.client.join(NAMES[1])).unwrap();
    file.set_len(MAX_CONTENT as u64 + 1).unwrap();
    assert!(Originals::begin(&f.data, &f.client).is_err());
    assert!(!f.data.join("manager-state").exists());
}

#[test]
fn interrupted_phase_publication_is_retained_and_never_overwritten_as_empty() {
    let f = Fixture::new();
    let originals = Originals::begin(&f.data, &f.client).unwrap();
    let path = f.journal(originals.attempt()).join("ready.json");
    let mut raw = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o400)
        .open(&path)
        .unwrap();
    raw.write_all(b"{interrupted").unwrap();
    raw.sync_all().unwrap();
    assert!(originals.seal(Phase::Ready).is_err());
    assert!(originals.read(Phase::Ready).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"{interrupted");
    assert!(originals.read(Phase::Original).is_ok());
}

#[test]
fn actual_process_exit_preserves_original_and_explicit_phase_identity() {
    const CHILD: &str = "OCXD024_ORIGINALS_CHILD";
    if let Some(root) = std::env::var_os(CHILD) {
        let root = PathBuf::from(root);
        assert!(root.is_absolute());
        let originals =
            Originals::begin(&root.join("desktop"), &root.join("client 中文 space")).unwrap();
        let ready = originals.seal(Phase::Ready).unwrap();
        let note = serde_json::json!({"attempt":originals.attempt(),"original":originals.original_sha256(),"ready":ready});
        let mut file = File::create(root.join("independent-receipt.json")).unwrap();
        file.write_all(&encode(&note).unwrap()).unwrap();
        file.sync_all().unwrap();
        std::process::exit(91);
    }
    let f = Fixture::new();
    let result=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","infrastructure::macos_client_originals::tests::actual_process_exit_preserves_original_and_explicit_phase_identity","--nocapture"]).env(CHILD,&f.root).output().unwrap();
    assert_eq!(result.status.code(), Some(91));
    let note: serde_json::Value =
        serde_json::from_slice(&fs::read(f.root.join("independent-receipt.json")).unwrap())
            .unwrap();
    let attempt = note["attempt"].as_str().unwrap();
    let original = note["original"].as_str().unwrap();
    let ready = note["ready"].as_str().unwrap();
    let reopened = Originals::open(
        &f.data,
        &f.client,
        attempt,
        original,
        &[(Phase::Ready, ready.into())],
    )
    .unwrap();
    assert!(reopened.read(Phase::Ready).is_ok());
    assert!(reopened.read(Phase::Original).is_ok());
    assert!(reopened.seal(Phase::Ready).is_err());
    let unbound = Originals::open(&f.data, &f.client, attempt, original, &[]).unwrap();
    assert!(unbound.read(Phase::Ready).is_err());
}
