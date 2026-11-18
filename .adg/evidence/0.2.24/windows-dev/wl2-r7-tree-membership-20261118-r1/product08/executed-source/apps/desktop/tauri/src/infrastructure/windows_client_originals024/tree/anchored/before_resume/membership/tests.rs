use super::*;
use crate::infrastructure::windows_client_originals024::{
    coordination::NativeLease, tree::tests::Fixture,
};
use std::time::Duration;
struct Scene {
    fixture: Option<Fixture>,
    root: PathBuf,
    custody: PathBuf,
    paths: [PathBuf; 4],
}
impl Scene {
    fn new() -> Self {
        let fixture = Fixture::new();
        let (root, custody, paths) = super::super::super::tests::prepare(&fixture);
        Self {
            fixture: Some(fixture),
            root,
            custody,
            paths,
        }
    }
}
impl Drop for Scene {
    fn drop(&mut self) {
        if let Some(fixture) = self.fixture.take() {
            let path = fixture.path().to_path_buf();
            drop(fixture);
            if !std::thread::panicking() {
                assert!(!path.exists());
                println!("r7-tree-membership-cleanup-absent={}", path.display());
            }
        }
    }
}
fn end() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
fn original_id(session: uuid::Uuid, relative: &str) -> String {
    projection_digest(&(session.to_string(), Role::Codex, "original", relative)).unwrap()
}
fn work_id(session: uuid::Uuid, relative: &str) -> String {
    projection_digest(&(session.to_string(), Role::Codex, "work", relative)).unwrap()
}
#[test]
fn wl2_r7_tree_membership_native_failed_open_keeps_intent_and_never_reopens_after_conflict_gone() {
    let scene = Scene::new();
    let end = end();
    let path = scene.root.join("config.toml");
    let held = OpenOptions::new()
        .read(true)
        .share_mode(0)
        .open(&path)
        .unwrap();
    let held_id = id(&held, false).unwrap();
    let mut lease = NativeLease::acquire_synthetic(&scene.paths, end).unwrap();
    let mut claim = lease.anchored_claim(&scene.root, end).unwrap();
    let mut prefix = None;
    let error = AnchoredTree::capture_retained(
        &scene.root,
        &scene.custody,
        &mut claim,
        &scene.root,
        end,
        &mut prefix,
    )
    .err()
    .unwrap();
    assert_eq!(error.raw_os_error(), Some(32));
    let tree = prefix.as_mut().unwrap();
    assert!(tree.pending_capture.is_none());
    assert_eq!(
        tree.pending_capture_locator.as_ref().unwrap().0,
        "config.toml"
    );
    let session = uuid::Uuid::new_v4();
    let (first, _) = tree
        .v2_retained_object_data(session, Role::Codex, end)
        .unwrap();
    let pending = first
        .iter()
        .find(|row| row.object_id == original_id(session, "config.toml"))
        .unwrap();
    assert_eq!(pending.original_stage, Stage::IntentPublished);
    assert!(
        pending.identity_digest.is_none()
            && pending.security_digest.is_none()
            && pending.content_digest.is_none()
    );
    assert_eq!(id(&held, false).unwrap(), held_id);
    drop(held);
    let (again, _) = tree
        .v2_retained_object_data(session, Role::Codex, end)
        .unwrap();
    assert!(
        first == again,
        "removing unrelated conflict must not reopen or upgrade original capture intent"
    );
    assert!(tree.pending_capture.is_none() && tree.poisoned);
    assert!(tree.read_before_resume_prefix(&mut claim, end).is_err());
    assert!(AnchoredTree::capture_retained(
        &scene.root,
        &scene.custody,
        &mut claim,
        &scene.root,
        end,
        &mut prefix
    )
    .is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"synthetic original config");
    println!("r7 treemembership actualopen raw32: exact pendingrelative retained before open; even afteractualconflictclose no nativehandle/identity/Absent/retry adopted");
    drop(prefix);
    drop(claim);
    lease.release(end).unwrap();
    drop(scene);
}
#[test]
fn wl2_r7_tree_membership_native_partial_capture_retains_exact_locator_and_missing_baseline() {
    let scene = Scene::new();
    let path = scene.root.join(ANCHORED_CODEX_STABLE_FILES[1]);
    std::fs::write(&path, b"invalid original SQLite DATA").unwrap();
    let end = end();
    let mut lease = NativeLease::acquire_synthetic(&scene.paths, end).unwrap();
    let mut claim = lease.anchored_claim(&scene.root, end).unwrap();
    let mut prefix = None;
    assert!(AnchoredTree::capture_retained(
        &scene.root,
        &scene.custody,
        &mut claim,
        &scene.root,
        end,
        &mut prefix
    )
    .is_err());
    let tree = prefix.as_mut().unwrap();
    let (relative, kind) = tree.pending_capture_locator.as_ref().unwrap();
    assert_eq!(relative, ANCHORED_CODEX_STABLE_FILES[1]);
    assert_eq!(*kind, Kind::Coordinator);
    let identity = id(tree.pending_capture.as_ref().unwrap(), false).unwrap();
    let session = uuid::Uuid::new_v4();
    let (data, errors) = tree
        .v2_retained_object_data(session, Role::Codex, end)
        .unwrap();
    assert!(errors.is_empty());
    assert_eq!(data.len(), tree.nodes.len() + 3);
    let pending = data
        .iter()
        .find(|row| row.object_id == original_id(session, ANCHORED_CODEX_STABLE_FILES[1]))
        .unwrap();
    assert_eq!(
        pending.identity_digest.as_ref().unwrap(),
        &projection_digest(&(identity.volume, identity.index.to_string())).unwrap()
    );
    assert!(pending.security_digest.is_none() && pending.content_digest.is_some());
    assert!(tree.poisoned);
    assert!(tree.read_before_resume_prefix(&mut claim, end).is_err());
    assert!(std::fs::remove_file(&path).is_err());
    assert_eq!(
        std::fs::read(&path).unwrap(),
        b"invalid original SQLite DATA"
    );
    println!("r7 treemembership actualpartialcapture exact pendinglocator+originalhandle ID/content observed, missingSD baseline null, poisoned oldread/mutation stillrefuses");
    drop(prefix);
    drop(claim);
    lease.release(end).unwrap();
    drop(scene);
}
#[test]
fn wl2_r7_tree_membership_native_working_and_every_pending_creator_list_each_actual_original_object(
) {
    for fault in [0, 1, 2, 3] {
        let scene = Scene::new();
        let end = end();
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, end).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, end).unwrap();
        let mut tree =
            AnchoredTree::capture(&scene.root, &scene.custody, &mut claim, &scene.root, end)
                .unwrap();
        let session = uuid::Uuid::new_v4();
        let (before, _) = tree
            .v2_retained_object_data(session, Role::Codex, end)
            .unwrap();
        let keys = before
            .iter()
            .map(|row| row.object_id.clone())
            .collect::<std::collections::BTreeSet<_>>();
        let i = tree
            .nodes
            .iter()
            .position(|node| node.kind == Kind::File)
            .unwrap();
        let relative = tree.nodes[i].original.relative.clone();
        let original = id(&tree.nodes[i].file, false).unwrap();
        tree.step(i, State::Moved, &mut claim, &scene.root, end)
            .unwrap();
        tree.create_fault = fault;
        let result = tree.step(i, State::Working, &mut claim, &scene.root, end);
        assert_eq!(result.is_err(), fault != 0);
        let actual = if fault == 0 {
            &tree.nodes[i].work.as_ref().unwrap().0
        } else {
            tree.nodes[i].pending_work.as_ref().unwrap()
        };
        let actual_id = id(actual, false).unwrap();
        assert_ne!(actual_id, original);
        let (after, _) = tree
            .v2_retained_object_data(session, Role::Codex, end)
            .unwrap();
        assert_eq!(after.len(), before.len() + 1);
        assert_eq!(
            after
                .iter()
                .filter(|row| row.object_id != work_id(session, &relative))
                .map(|row| row.object_id.clone())
                .collect::<std::collections::BTreeSet<_>>(),
            keys
        );
        let work = after
            .iter()
            .find(|row| row.object_id == work_id(session, &relative))
            .unwrap();
        assert_eq!(
            work.identity_digest.as_ref().unwrap(),
            &projection_digest(&(actual_id.volume, actual_id.index.to_string())).unwrap()
        );
        assert_eq!(work.security_digest.is_none(), fault != 0);
        let contents = std::fs::read(tree.work_location(i)).unwrap();
        assert_eq!(
            work.content_digest.as_ref().unwrap(),
            &hash::sha256_hex(&contents)
        );
        if fault != 0 {
            assert!(tree.poisoned);
            assert!(tree.read_before_resume_prefix(&mut claim, end).is_err());
        }
        assert_eq!(id(&tree.nodes[i].file, false).unwrap(), original);
        assert!(std::fs::remove_file(tree.work_location(i)).is_err());
        println!("r7 treemembership actualwork fault{fault}: sameoriginal/nativeworkdistinctIDs, each retainedcreator is member, bytes={} missingbaselinePending/no completedintent inferred",contents.len());
        drop(tree);
        drop(claim);
        lease.release(end).unwrap();
        drop(scene);
    }
}
#[test]
fn wl2_r7_tree_membership_native_complete_capture_sort_and_later_work_do_not_rekey_originals() {
    let scene = Scene::new();
    let end = end();
    let mut lease = NativeLease::acquire_synthetic(&scene.paths, end).unwrap();
    let mut claim = lease.anchored_claim(&scene.root, end).unwrap();
    let mut tree =
        AnchoredTree::capture(&scene.root, &scene.custody, &mut claim, &scene.root, end).unwrap();
    let session = uuid::Uuid::new_v4();
    let (before, _) = tree
        .v2_retained_object_data(session, Role::Codex, end)
        .unwrap();
    let i = tree
        .nodes
        .iter()
        .position(|node| node.kind == Kind::File)
        .unwrap();
    let relative = tree.nodes[i].original.relative.clone();
    tree.nodes.swap(1, 2);
    let (reordered, _) = tree
        .v2_retained_object_data(session, Role::Codex, end)
        .unwrap();
    assert!(
        before == reordered,
        "same complete object set must not depend on node vector order"
    );
    tree.nodes
        .sort_by(|a, b| a.original.relative.cmp(&b.original.relative));
    let i = tree
        .nodes
        .iter()
        .position(|node| node.original.relative == relative)
        .unwrap();
    tree.step(i, State::Moved, &mut claim, &scene.root, end)
        .unwrap();
    tree.step(i, State::Working, &mut claim, &scene.root, end)
        .unwrap();
    let (after, _) = tree
        .v2_retained_object_data(session, Role::Codex, end)
        .unwrap();
    for row in before {
        let next = after
            .iter()
            .find(|item| item.object_id == row.object_id)
            .unwrap();
        assert_eq!(next.identity_digest, row.identity_digest);
        assert_eq!(next.path_digest, row.path_digest);
    }
    println!("r7 treemembership actualnative original IDs stable across nodeorder/move/work and originalparent/custody stablekeys; bounded DATA no nativeOwnergrant");
    drop(tree);
    drop(claim);
    lease.release(end).unwrap();
    drop(scene);
}
#[test]
fn wl2_r7_tree_membership_native_unknown_bytes_sd_ads_members_and_deadline_refuse() {
    for mode in [
        "work-bytes",
        "pending-bytes",
        "security",
        "stream",
        "member",
        "custody-member",
        "late-member",
        "late-security",
        "expired",
    ] {
        let scene = Scene::new();
        let end = end();
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, end).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, end).unwrap();
        let mut tree =
            AnchoredTree::capture(&scene.root, &scene.custody, &mut claim, &scene.root, end)
                .unwrap();
        let i = tree
            .nodes
            .iter()
            .position(|node| node.kind == Kind::File)
            .unwrap();
        match mode {
            "work-bytes" | "pending-bytes" => {
                tree.step(i, State::Moved, &mut claim, &scene.root, end)
                    .unwrap();
                tree.create_fault = if mode == "pending-bytes" { 2 } else { 0 };
                let _ = tree.step(i, State::Working, &mut claim, &scene.root, end);
                let file = if mode == "pending-bytes" {
                    tree.nodes[i].pending_work.as_mut().unwrap()
                } else {
                    &mut tree.nodes[i].work.as_mut().unwrap().0
                };
                file.seek(SeekFrom::Start(0)).unwrap();
                file.write_all(b"!").unwrap();
                file.sync_all().unwrap();
            }
            "security" => changed_dacl(&tree.original_location(i)),
            "stream" => {
                let path = tree.root.join(".opencodex-native-main.owner.sqlite");
                std::fs::write(
                    PathBuf::from(format!("{}:unknown", path.display())),
                    b"unknown stream",
                )
                .unwrap();
            }
            "member" => std::fs::write(tree.root.join("foreign.bin"), b"preserve").unwrap(),
            "custody-member" => {
                std::fs::write(tree.custody.path().join("foreign.bin"), b"preserve").unwrap()
            }
            "late-member" => after_for_test(|root| {
                std::fs::write(root.join("late.bin"), b"late foreign").unwrap()
            }),
            "late-security" => after_for_test(|root| changed_dacl(&root.join("config.toml"))),
            _ => {}
        }
        let read_end = if mode == "expired" {
            Instant::now()
        } else {
            end
        };
        let result = tree.v2_retained_object_data(uuid::Uuid::new_v4(), Role::Codex, read_end);
        assert!(result.is_err(), "{mode}");
        assert!(tree.poisoned);
        let error = result.err().unwrap();
        println!("r7 treemembership native refuses {mode}: raw={:?} kind={:?}, sameactualresources retained/no repair or terminalgrant",error.raw_os_error(),error.kind());
        drop(tree);
        drop(claim);
        lease.release(end).unwrap();
        drop(scene);
    }
}
fn changed_dacl(path: &Path) {
    use std::{ffi::c_void, os::windows::ffi::OsStrExt};
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
            text: *const u16,
            revision: u32,
            descriptor: *mut *mut c_void,
            size: *mut u32,
        ) -> i32;
        fn SetFileSecurityW(path: *const u16, information: u32, descriptor: *const c_void) -> i32;
    }
    let text = "D:P(A;;FA;;;OW)(A;;FA;;;SY)"
        .encode_utf16()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let path = path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let mut sd = std::ptr::null_mut();
    assert_ne!(
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text.as_ptr(),
                1,
                &mut sd,
                std::ptr::null_mut(),
            )
        },
        0
    );
    let result = unsafe { SetFileSecurityW(path.as_ptr(), 4, sd) };
    unsafe {
        windows_sys::Win32::Foundation::LocalFree(sd);
    }
    assert_ne!(result, 0);
}
