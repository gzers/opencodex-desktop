use super::*;
use std::{path::PathBuf, time::Duration};
struct Scene(Option<tempfile::TempDir>);
impl Scene {
    fn new() -> Self {
        Self(Some(
            tempfile::Builder::new()
                .prefix("ocx024-r7-partial-native-")
                .tempdir()
                .unwrap(),
        ))
    }
    fn path(&self) -> &Path {
        self.0.as_ref().unwrap().path()
    }
    fn private_tree(&self) -> PathBuf {
        let root = self.path().join(".codex");
        drop(native::create_private_directory(&root).unwrap());
        for name in ANCHORED_CODEX_STABLE_FILES {
            drop(native::PrivateRecord::create(&root.join(name), b"", 1024).unwrap());
        }
        drop(
            native::PrivateRecord::create(
                &root.join("config.toml"),
                b"synthetic existing original bytes",
                1024,
            )
            .unwrap(),
        );
        root
    }
}
impl Drop for Scene {
    fn drop(&mut self) {
        if let Some(scene) = self.0.take() {
            if std::thread::panicking() {
                eprintln!(
                    "failed own partial native scene retained: {}",
                    scene.keep().display()
                );
            } else {
                scene.close().unwrap();
            }
        }
    }
}
#[test]
fn r7_partial_actual_collect_keeps_pending_invalid_coordinator_handle_and_completed_original_nodes()
{
    let scene = Scene::new();
    let root = scene.private_tree();
    let owner = root.join(ANCHORED_CODEX_STABLE_FILES[1]);
    std::fs::write(&owner, b"invalid original SQLite fixture").unwrap();
    let mut nodes = Vec::new();
    let mut pending = None;
    let mut pending_locator = None;
    let end = Instant::now() + Duration::from_secs(5);
    assert!(collect(
        &root,
        "",
        true,
        &mut nodes,
        &mut 0,
        &mut pending,
        &mut pending_locator,
        end
    )
    .is_err());
    assert!(!nodes.is_empty() && pending.is_some());
    assert_eq!(
        pending_locator,
        Some((ANCHORED_CODEX_STABLE_FILES[1].into(), Kind::Coordinator))
    );
    assert!(nodes.iter().all(|n| n.state == State::Original));
    let retained_id = id(pending.as_ref().unwrap(), false).unwrap();
    let file = OpenOptions::new()
        .access_mode(0x20080)
        .share_mode(7)
        .open(&owner)
        .unwrap();
    assert_eq!(id(&file, false).unwrap(), retained_id);
    drop(file);
    assert!(std::fs::remove_file(&owner).is_err());
    assert_eq!(
        std::fs::read(root.join("config.toml")).unwrap(),
        b"synthetic existing original bytes"
    );
    drop(pending);
    drop(nodes);
    assert_eq!(
        std::fs::read(&owner).unwrap(),
        b"invalid original SQLite fixture"
    );
    println!("actual product collect/stamp/open/id/security/stream code retains pending original coordinator + completed nodes; no claim/admission/move/restore authority asserted");
}
#[test]
fn r7_partial_actual_collect_unknown_stream_retains_deny_write_delete_original_and_does_not_move() {
    let scene = Scene::new();
    let root = scene.private_tree();
    let file = root.join("config.toml");
    let ads = PathBuf::from(format!("{}:r7-owned", file.display()));
    std::fs::write(&ads, b"unknown original stream").unwrap();
    let mut nodes = Vec::new();
    let mut pending = None;
    let mut pending_locator = None;
    assert!(collect(
        &root,
        "",
        true,
        &mut nodes,
        &mut 0,
        &mut pending,
        &mut pending_locator,
        Instant::now() + Duration::from_secs(5)
    )
    .is_err());
    assert!(pending.is_some());
    assert_eq!(pending_locator, Some(("config.toml".into(), Kind::File)));
    assert!(nodes.iter().all(|n| n.state == State::Original));
    assert!(std::fs::OpenOptions::new().write(true).open(&file).is_err());
    assert!(std::fs::remove_file(&file).is_err());
    drop(pending);
    drop(nodes);
    assert_eq!(
        std::fs::read(&file).unwrap(),
        b"synthetic existing original bytes"
    );
    assert_eq!(std::fs::read(&ads).unwrap(), b"unknown original stream");
}
#[test]
fn r7_partial_actual_collect_success_retains_all_native_ids8_and_bytes_without_pending_slot() {
    let scene = Scene::new();
    let root = scene.private_tree();
    let end = Instant::now() + Duration::from_secs(5);
    let mut nodes = Vec::new();
    let mut pending = None;
    let mut pending_locator = None;
    let mut bytes = 0;
    collect(
        &root,
        "",
        true,
        &mut nodes,
        &mut bytes,
        &mut pending,
        &mut pending_locator,
        end,
    )
    .unwrap();
    assert_eq!(nodes.len(), 4);
    assert_eq!(bytes, b"synthetic existing original bytes".len());
    assert!(pending.is_none());
    assert!(pending_locator.is_none());
    for node in &nodes {
        assert_eq!(node.state, State::Original);
        assert_eq!(
            id(&node.file, node.kind == Kind::Directory).unwrap(),
            node.original.id
        );
        let actual = native_stamp(&node.file, end).unwrap();
        assert!(actual == node.original.security);
    }
    assert!(std::fs::OpenOptions::new()
        .write(true)
        .open(root.join("config.toml"))
        .is_err());
    drop(nodes);
    std::fs::write(
        root.join("config.toml"),
        b"only own observation handles released",
    )
    .unwrap();
}
