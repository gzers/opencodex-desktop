use super::*;
use crate::infrastructure::windows_client_originals024::{
    coordination::NativeLease, tree::tests::Fixture,
};
use std::time::Duration;
#[path = "join_tests.rs"]
mod join_tests;
struct Scene {
    fixture: Option<Fixture>,
    root: PathBuf,
    custody: PathBuf,
    paths: [PathBuf; 4],
}
impl Scene {
    fn new() -> Self {
        let fixture = Fixture::new();
        let (root, custody, paths) = super::super::tests::prepare(&fixture);
        drop(PrivateRecord::create(&root.join("third.json"), b"unchanged original", 1024).unwrap());
        Self {
            fixture: Some(fixture),
            root,
            custody,
            paths,
        }
    }
    fn tree(&self, claim: &mut AnchoredClaim<'_>, end: Instant) -> AnchoredTree {
        AnchoredTree::capture(&self.root, &self.custody, claim, &self.root, end).unwrap()
    }
}
impl Drop for Scene {
    fn drop(&mut self) {
        if let Some(fixture) = self.fixture.take() {
            let path = fixture.path().to_path_buf();
            drop(fixture);
            if !std::thread::panicking() {
                assert!(!path.exists(), "fresh tree prefix fixture cleanup");
                println!("r7-tree-prefix-cleanup-absent={}", path.display());
            }
        }
    }
}
fn end() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
#[test]
fn wl2_r7_prefix_read_tree_native_mixed_states_fresh_claim_keeps_poison_and_same_original_objects()
{
    let scene = Scene::new();
    let first = end();
    let mut lease = NativeLease::acquire_synthetic(&scene.paths, first).unwrap();
    let mut claim = lease.anchored_claim(&scene.root, first).unwrap();
    let epoch = claim.epoch().to_owned();
    let mut tree = scene.tree(&mut claim, first);
    let files = tree
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.kind == Kind::File)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    assert_eq!(files.len(), 3);
    tree.step(files[0], State::Moved, &mut claim, &scene.root, first)
        .unwrap();
    tree.step(files[0], State::Working, &mut claim, &scene.root, first)
        .unwrap();
    tree.step(files[1], State::Moved, &mut claim, &scene.root, first)
        .unwrap();
    let old_plan = tree.plan();
    let old_ids = tree
        .nodes
        .iter()
        .map(|n| id(&n.file, n.kind == Kind::Directory).unwrap())
        .collect::<Vec<_>>();
    assert!(tree
        .verify(&mut claim, &scene.root, Instant::now())
        .is_err());
    assert!(tree.poisoned);
    drop(claim);
    assert!(
        lease.release(first).is_err(),
        "expired read claim is failure cleanup, not a normal release receipt"
    );
    let fresh = end();
    let mut lease = NativeLease::acquire_synthetic(&scene.paths, fresh).unwrap();
    let mut claim = lease.anchored_claim(&scene.root, fresh).unwrap();
    assert_ne!(claim.epoch(), epoch);
    let data = tree.read_before_resume_prefix(&mut claim, fresh).unwrap();
    assert_eq!(data["runtimeAuthority"], false);
    assert_eq!(data["poisoned"], true);
    assert_eq!(data["plan"], old_plan);
    for state in ["Original", "Moved", "Working"] {
        assert!(data["nodes"]
            .as_array()
            .unwrap()
            .iter()
            .any(|n| n["state"] == state));
    }
    assert!(tree.verify(&mut claim, &scene.root, fresh).is_err());
    assert!(tree.poisoned);
    assert_eq!(
        tree.nodes
            .iter()
            .map(|n| id(&n.file, n.kind == Kind::Directory).unwrap())
            .collect::<Vec<_>>(),
        old_ids
    );
    assert!(tree
        .step(files[2], State::Moved, &mut claim, &scene.root, fresh)
        .is_err());
    drop(tree);
    drop(claim);
    lease.release(fresh).unwrap();
    drop(scene);
}
fn changed_dacl(path: &Path) {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
            text: *const u16,
            revision: u32,
            descriptor: *mut *mut std::ffi::c_void,
            size: *mut u32,
        ) -> i32;
        fn SetFileSecurityW(
            path: *const u16,
            information: u32,
            descriptor: *const std::ffi::c_void,
        ) -> i32;
    }
    let text = "D:P(A;;FA;;;OW)(A;;FA;;;SY)"
        .encode_utf16()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let name = path
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
    let result = unsafe { SetFileSecurityW(name.as_ptr(), 4, sd) };
    unsafe {
        windows_sys::Win32::Foundation::LocalFree(sd);
    }
    assert_ne!(result, 0, "only new own fixture DACL must actually change");
}
#[test]
fn wl2_r7_prefix_read_tree_native_members_bytes_and_security_drift_refuse() {
    for change in ["client-member", "custody-member", "bytes", "security"] {
        let scene = Scene::new();
        let end = end();
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, end).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, end).unwrap();
        let mut tree = scene.tree(&mut claim, end);
        let i = tree
            .nodes
            .iter()
            .position(|n| n.kind == Kind::File)
            .unwrap();
        if change == "bytes" {
            tree.step(i, State::Moved, &mut claim, &scene.root, end)
                .unwrap();
            tree.step(i, State::Working, &mut claim, &scene.root, end)
                .unwrap();
            let file = &mut tree.nodes[i].work.as_mut().unwrap().0;
            file.seek(SeekFrom::Start(0)).unwrap();
            file.write_all(b"changed own work bytes").unwrap();
            file.sync_all().unwrap();
        } else if change == "security" {
            changed_dacl(&tree.original_location(i));
        } else {
            let dir = if change == "client-member" {
                &scene.root
            } else {
                &scene.custody
            };
            std::fs::write(
                dir.join("foreign-member.bin"),
                b"own fixture drift, do not adopt",
            )
            .unwrap();
        }
        assert!(
            tree.read_before_resume_prefix(&mut claim, end).is_err(),
            "{change}"
        );
        assert!(tree.poisoned);
        println!(
            "r7 actual tree prefix rejects {change}, preserves original native handles and poison"
        );
        drop(tree);
        drop(claim);
        lease.release(end).unwrap();
        drop(scene);
    }
}
#[test]
fn wl2_r7_prefix_read_tree_native_pending_work_open_partial_and_full_unpublished_refuse() {
    for fault in [1, 2, 3] {
        let scene = Scene::new();
        let end = end();
        let mut lease = NativeLease::acquire_synthetic(&scene.paths, end).unwrap();
        let mut claim = lease.anchored_claim(&scene.root, end).unwrap();
        let mut tree = scene.tree(&mut claim, end);
        let i = tree
            .nodes
            .iter()
            .position(|n| n.kind == Kind::File)
            .unwrap();
        tree.step(i, State::Moved, &mut claim, &scene.root, end)
            .unwrap();
        tree.create_fault = fault;
        assert!(tree
            .step(i, State::Working, &mut claim, &scene.root, end)
            .is_err());
        let pending = id(tree.nodes[i].pending_work.as_ref().unwrap(), false).unwrap();
        let original = id(&tree.nodes[i].file, false).unwrap();
        assert!(tree.read_before_resume_prefix(&mut claim, end).is_err());
        assert_eq!(
            id(tree.nodes[i].pending_work.as_ref().unwrap(), false).unwrap(),
            pending
        );
        assert_eq!(id(&tree.nodes[i].file, false).unwrap(), original);
        assert!(tree.poisoned && tree.nodes[i].state == State::Moved);
        assert!(std::fs::remove_file(tree.work_location(i)).is_err());
        drop(tree);
        drop(claim);
        lease.release(end).unwrap();
        drop(scene);
    }
}
#[test]
fn wl2_r7_prefix_read_tree_native_pending_original_capture_remains_unknown() {
    let scene = Scene::new();
    std::fs::write(
        scene.root.join(ANCHORED_CODEX_STABLE_FILES[1]),
        b"invalid actual coordinator",
    )
    .unwrap();
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
    let pending = id(tree.pending_capture.as_ref().unwrap(), false).unwrap();
    assert!(tree.read_before_resume_prefix(&mut claim, end).is_err());
    assert_eq!(
        id(tree.pending_capture.as_ref().unwrap(), false).unwrap(),
        pending
    );
    assert!(tree.poisoned);
    assert!(names(&scene.custody).unwrap().is_empty());
    drop(prefix);
    drop(claim);
    lease.release(end).unwrap();
    drop(scene);
}
#[test]
fn wl2_r7_prefix_read_tree_native_wrong_actual_claim_and_expired_cutoff_refuse() {
    let scene = Scene::new();
    let other = Scene::new();
    let end = end();
    let mut lease = NativeLease::acquire_synthetic(&scene.paths, end).unwrap();
    let mut claim = lease.anchored_claim(&scene.root, end).unwrap();
    let mut tree = scene.tree(&mut claim, end);
    let mut foreign = NativeLease::acquire_synthetic(&other.paths, end).unwrap();
    let mut bad_claim = foreign.anchored_claim(&other.root, end).unwrap();
    assert!(tree.read_before_resume_prefix(&mut bad_claim, end).is_err());
    drop(bad_claim);
    assert!(
        foreign.release(end).is_err(),
        "wrong actual root cannot mint a normal release receipt"
    );
    assert_eq!(
        tree.read_before_resume_prefix(&mut claim, Instant::now())
            .unwrap_err()
            .kind(),
        io::ErrorKind::TimedOut
    );
    assert!(tree.poisoned);
    drop(tree);
    drop(claim);
    lease.release(end).unwrap();
    drop(other);
    drop(scene);
}
