use super::super::super::coordination::NativeLease;
use super::super::tests::Fixture;
use super::*;
use rusqlite::Connection;
use std::time::Duration;
#[path = "anchored_driver_tests.rs"]
pub(super) mod driver;
fn end() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
pub(super) fn prepare(f: &Fixture) -> (PathBuf, PathBuf, [PathBuf; 4]) {
    let profile = create_private_directory(&f.path().join("Profile")).unwrap();
    let root = profile.path().join(".codex");
    drop(create_private_directory(&root).unwrap());
    drop(create_private_directory(&root.join("nested")).unwrap());
    drop(create_private_directory(&root.join("empty")).unwrap());
    for (name, bytes) in [
        ("config.toml", b"synthetic original config".as_slice()),
        (
            "nested/auth.json",
            b"synthetic credential-free auth".as_slice(),
        ),
    ] {
        drop(PrivateRecord::create(&root.join(name), bytes, 1024).unwrap());
    }
    for name in ANCHORED_CODEX_STABLE_FILES {
        drop(PrivateRecord::create(&root.join(name), b"", 16).unwrap());
    }
    let coordination = create_private_directory(&f.path().join("coordination")).unwrap();
    let paths = std::array::from_fn(|i| {
        if i == 0 {
            root.join(ANCHORED_CODEX_STABLE_FILES[0])
        } else {
            coordination.path().join(format!("{i}.sqlite"))
        }
    });
    for path in paths.iter().skip(1) {
        drop(PrivateRecord::create(path, b"", 16).unwrap());
    }
    let custody = profile.path().join("custody");
    drop(create_private_directory(&custody).unwrap());
    (root, custody, paths)
}
#[test]
fn wl2_r7_partial_capture_native_invalid_coordinator_keeps_completed_nodes_and_pending_handle() {
    let f = Fixture::new();
    let (root, custody, paths) = prepare(&f);
    let owner = root.join(ANCHORED_CODEX_STABLE_FILES[1]);
    std::fs::write(&owner, b"invalid SQLite original fixture").unwrap();
    let end = end();
    let mut lease = NativeLease::acquire_synthetic(&paths, end).unwrap();
    let mut claim = lease.anchored_claim(&root, end).unwrap();
    let mut retained = None;
    assert!(
        AnchoredTree::capture_retained(&root, &custody, &mut claim, &root, end, &mut retained)
            .is_err()
    );
    let tree = retained
        .as_mut()
        .expect("partial enrolled before fallible node collection");
    assert!(tree.poisoned);
    assert!(tree.pending_capture.is_some());
    assert!(!tree.nodes.is_empty());
    assert!(tree.nodes.iter().all(|n| n.state == State::Original));
    let pending_id=id(tree.pending_capture.as_ref().unwrap(),false).unwrap();
    let fresh=OpenOptions::new().access_mode(0x20080).share_mode(7).open(&owner).unwrap();
    assert_eq!(id(&fresh,false).unwrap(),pending_id);drop(fresh);
    assert!(std::fs::remove_file(&owner).is_err());
    assert!(
        AnchoredTree::capture_retained(&root, &custody, &mut claim, &root, end, &mut retained)
            .is_err(),
        "failed original prefix cannot be replaced by retry"
    );
    assert_eq!(
        std::fs::read(root.join("config.toml")).unwrap(),
        b"synthetic original config"
    );
    assert!(names(&custody).unwrap().is_empty());
    drop(retained);
    drop(claim);
    lease.release(end).unwrap();
    assert_eq!(
        std::fs::read(&owner).unwrap(),
        b"invalid SQLite original fixture"
    );
    println!("r7 partial original tree retains actual completed native nodes + pending invalid coordinator handle; no move/work/phase/restore/Closed or replacement of failed prefix");
}
#[test]
fn wl2_r7_partial_capture_native_unknown_stream_retains_prefix_and_never_moves_original() {
    let f = Fixture::new();
    let (root, custody, paths) = prepare(&f);
    let file = root.join("config.toml");
    let ads = PathBuf::from(format!("{}:r7-owned-stream", file.display()));
    std::fs::write(&ads, b"unknown stream fixture").unwrap();
    let end = end();
    let mut lease = NativeLease::acquire_synthetic(&paths, end).unwrap();
    let mut claim = lease.anchored_claim(&root, end).unwrap();
    let mut retained = None;
    assert!(
        AnchoredTree::capture_retained(&root, &custody, &mut claim, &root, end, &mut retained)
            .is_err()
    );
    let tree = retained.as_ref().unwrap();
    assert!(tree.poisoned && tree.pending_capture.is_some());
    assert!(tree.nodes.iter().all(|n| n.state == State::Original));
    assert!(std::fs::OpenOptions::new().write(true).open(&file).is_err());
    assert!(names(&custody).unwrap().is_empty());
    assert_eq!(std::fs::read(&file).unwrap(), b"synthetic original config");
    drop(retained);
    drop(claim);
    lease.release(end).unwrap();
    assert_eq!(std::fs::read(&ads).unwrap(), b"unknown stream fixture");
}
#[test]
fn wl2_r7_partial_capture_native_success_transfers_same_complete_tree_and_refuses_reenrollment() {
    let f = Fixture::new();
    let (root, custody, paths) = prepare(&f);
    let end = end();
    let mut lease = NativeLease::acquire_synthetic(&paths, end).unwrap();
    let mut claim = lease.anchored_claim(&root, end).unwrap();
    let mut retained = None;
    AnchoredTree::capture_retained(&root, &custody, &mut claim, &root, end, &mut retained).unwrap();
    let original = retained
        .as_ref()
        .unwrap()
        .nodes
        .iter()
        .map(|n| n.original.id)
        .collect::<Vec<_>>();
    assert!(!retained.as_ref().unwrap().poisoned);
    assert!(retained.as_ref().unwrap().pending_capture.is_none());
    assert!(
        AnchoredTree::capture_retained(&root, &custody, &mut claim, &root, end, &mut retained)
            .is_err()
    );
    assert_eq!(
        retained
            .as_ref()
            .unwrap()
            .nodes
            .iter()
            .map(|n| n.original.id)
            .collect::<Vec<_>>(),
        original
    );
    retained
        .as_mut()
        .unwrap()
        .verify(&mut claim, &root, end)
        .unwrap();
    drop(retained);
    drop(claim);
    lease.release(end).unwrap();
}
pub(super) fn compete(path: &Path, blocked: bool) {
    let c = Connection::open(path).unwrap();
    c.busy_timeout(Duration::ZERO).unwrap();
    let result = c.execute_batch("BEGIN EXCLUSIVE");
    if blocked {
        assert!(
            matches!(result,Err(rusqlite::Error::SqliteFailure(code,_)) if code.code==rusqlite::ErrorCode::DatabaseBusy)
        );
    } else {
        result.unwrap();
        c.execute_batch("ROLLBACK").unwrap();
    }
    c.close().unwrap();
}
pub(super) fn bun_claim(path: &Path, deadline: Instant, blocked: bool) {
    use std::{
        os::windows::process::CommandExt,
        process::{Command, Stdio},
    };
    let bun=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../test/windows024-native/target/installer-w1/attempt01/stage/platform-bundle/runtime-bundle/runtime/bun/bun.exe");
    let _pin = OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&bun)
        .unwrap();
    assert_eq!(
        hash::sha256_file(&bun).unwrap(),
        "cb1d8e6564e655e49d30cead359750a1cbbd12f42651caa4b62244d7bd93c244"
    );
    let script = r#"import { Database } from 'bun:sqlite'; let db; try { db=new Database(process.argv.at(-1),{create:false}); db.exec('PRAGMA busy_timeout=0; BEGIN EXCLUSIVE; ROLLBACK'); console.log('claim-acquired'); } catch(e) { console.log(JSON.stringify({code:e?.code,errno:e?.errno})); process.exitCode=e?.code==='SQLITE_BUSY'?73:74; } finally { db?.close(); }"#;
    let mut child = Command::new(&bun)
        .arg("--eval")
        .arg(script)
        .arg(path)
        .creation_flags(0x08000000)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let status = loop {
        if let Some(s) = child.try_wait().unwrap() {
            break s;
        }
        assert!(
            Instant::now() < deadline,
            "Bun claim child exceeded original action deadline; no force or renewed timer"
        );
        std::thread::sleep(Duration::from_millis(5));
    };
    let mut out = String::new();
    let mut err = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut out)
        .unwrap();
    child
        .stderr
        .take()
        .unwrap()
        .read_to_string(&mut err)
        .unwrap();
    println!(
        "r6 fixed-Bun claim-only held={blocked} raw={:?} stdout={out} stderr={err}",
        status.code()
    );
    assert_eq!(status.code(), Some(if blocked { 73 } else { 0 }));
}
#[test]
fn wl2_r6_anchored_existing_tree_continuous_claim_node_roundtrip() {
    let mut f = Fixture::new();
    let (root, custody, paths) = prepare(&f);
    let deadline = end();
    let mut lease = NativeLease::acquire_synthetic(&paths, deadline).unwrap();
    let mut claim = lease.anchored_claim(&root, deadline).unwrap();
    println!("r6-epoch={} key={}", claim.epoch(), claim.key());
    let mut tree = AnchoredTree::capture(&root, &custody, &mut claim, &root, deadline).unwrap();
    let plan = tree.plan();
    assert_eq!(tree.nodes.len(), 7);
    let files = tree
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(i, n)| (n.kind == Kind::File).then_some(i))
        .collect::<Vec<_>>();
    bun_claim(&paths[0], deadline, true);
    for operation in [
        State::Moved,
        State::Working,
        State::Retired,
        State::Restored,
    ] {
        for i in &files {
            compete(&paths[0], true);
            tree.step(*i, operation, &mut claim, &root, deadline)
                .unwrap();
            bun_claim(&paths[0], deadline, true);
            compete(&paths[0], true);
            println!("r6-native-prefix node={} state={operation:?}", tree.key(*i));
        }
    }
    assert_eq!(tree.plan(), plan);
    for n in &tree.nodes {
        if n.kind == Kind::Directory {
            assert_eq!(
                names(&root.join(&n.original.relative)).unwrap(),
                n.original.children
            );
        }
    }
    tree.verify(&mut claim, &root, deadline).unwrap();
    drop(claim);
    lease.release(deadline).unwrap();
    tree.verify_after_release(deadline).unwrap();
    compete(&paths[0], false);
    bun_claim(&paths[0], deadline, false);
    println!("r6 exact original directory/two stable file IDs/all8 and complete children retained; original files restored by original handles; work generations retired; lease explicitly released; component only, Start/Closed=false");
    drop(tree);
    f.retain();
}
#[test]
fn wl2_r6_anchored_grok_names_are_ordinary_and_large_work_file_keeps_original_ids() {
    let mut f = Fixture::new();
    let (codex, _, paths) = prepare(&f);
    let deadline = end();
    let grok = codex.with_file_name(".grok");
    drop(create_private_directory(&grok).unwrap());
    let large = vec![0x73; 1280 * 1024];
    let p = grok.join(ANCHORED_CODEX_STABLE_FILES[0]);
    drop(PrivateRecord::create(&p, b"", 16).unwrap());
    std::fs::write(&p, &large).unwrap();
    let custody = grok.with_file_name("grok-custody");
    drop(create_private_directory(&custody).unwrap());
    let mut lease = NativeLease::acquire_synthetic(&paths, deadline).unwrap();
    let mut claim = lease.anchored_claim(&codex, deadline).unwrap();
    let mut tree = AnchoredTree::capture(&grok, &custody, &mut claim, &codex, deadline).unwrap();
    assert_eq!(tree.nodes[1].kind, Kind::File);
    for s in [
        State::Moved,
        State::Working,
        State::Retired,
        State::Restored,
    ] {
        tree.step(1, s, &mut claim, &codex, deadline).unwrap();
    }
    assert_eq!(std::fs::read(&p).unwrap(), large);
    drop(claim);
    lease.release(deadline).unwrap();
    drop(tree);
    f.retain();
}
#[test]
fn wl2_r6_anchored_every_native_prefix_retains_handles_without_resume() {
    for prefix in 0..=8 {
        let mut f = Fixture::new();
        let (root, custody, paths) = prepare(&f);
        let deadline = end();
        let mut lease = NativeLease::acquire_synthetic(&paths, deadline).unwrap();
        let mut claim = lease.anchored_claim(&root, deadline).unwrap();
        let mut tree = AnchoredTree::capture(&root, &custody, &mut claim, &root, deadline).unwrap();
        let files = tree
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(i, n)| (n.kind == Kind::File).then_some(i))
            .collect::<Vec<_>>();
        let operations = [
            State::Moved,
            State::Working,
            State::Retired,
            State::Restored,
        ]
        .into_iter()
        .flat_map(|s| files.iter().map(move |i| (*i, s)))
        .take(prefix);
        for (i, s) in operations {
            tree.step(i, s, &mut claim, &root, deadline).unwrap();
        }
        tree.verify(&mut claim, &root, deadline).unwrap();
        compete(&paths[0], true);
        println!("r6 interrupted-native-prefix={prefix} exact native locations preserved; no JSON reconstruction or Ready/Closed");
        drop(claim);
        lease.release(deadline).unwrap();
        drop(tree);
        f.retain();
    }
}
#[test]
fn wl2_r6_anchored_unknown_empty_directory_and_original_edit_refuse_without_overwrite() {
    let mut f = Fixture::new();
    let (root, custody, paths) = prepare(&f);
    let deadline = end();
    let mut lease = NativeLease::acquire_synthetic(&paths, deadline).unwrap();
    let mut claim = lease.anchored_claim(&root, deadline).unwrap();
    let mut tree = AnchoredTree::capture(&root, &custody, &mut claim, &root, deadline).unwrap();
    let i = tree
        .nodes
        .iter()
        .position(|n| n.kind == Kind::File)
        .unwrap();
    tree.step(i, State::Moved, &mut claim, &root, deadline)
        .unwrap();
    std::fs::create_dir(root.join("foreign-empty")).unwrap();
    assert!(tree
        .step(i, State::Working, &mut claim, &root, deadline)
        .is_err());
    assert!(tree.poisoned);
    assert!(tree.original_location(i).exists());
    assert!(root.join("foreign-empty").exists());
    assert!(tree.verify(&mut claim, &root, deadline).is_err());
    drop(claim);
    lease.release(deadline).unwrap();
    drop(tree);
    f.retain();
}
#[test]
fn wl2_r6_anchored_absent_root_wrong_claim_open_file_and_sidecar_admission_refuse() {
    for case in 0..4 {
        let mut f = Fixture::new();
        let (root, custody, paths) = prepare(&f);
        let deadline = end();
        let mut lease = NativeLease::acquire_synthetic(&paths, deadline).unwrap();
        if case == 0 {
            assert!(lease
                .anchored_claim(&root.with_file_name(".grok"), deadline)
                .is_err());
        } else {
            let mut claim = lease.anchored_claim(&root, deadline).unwrap();
            if case == 1 {
                assert!(AnchoredTree::capture(
                    &root.with_file_name(".grok"),
                    &custody,
                    &mut claim,
                    &root,
                    deadline
                )
                .is_err());
            }
            if case == 2 {
                let blocker = OpenOptions::new()
                    .read(true)
                    .share_mode(1 | 2)
                    .open(root.join("nested/auth.json"))
                    .unwrap();
                let e = AnchoredTree::capture(&root, &custody, &mut claim, &root, deadline)
                    .err()
                    .unwrap();
                assert_eq!(e.raw_os_error(), Some(32));
                println!(
                    "r6 ordinary file open admission raw={:?}, original retained",
                    e.raw_os_error()
                );
                drop(blocker);
            }
            if case == 3 {
                std::fs::write(
                    root.join(format!("{}-wal", ANCHORED_CODEX_STABLE_FILES[1])),
                    b"synthetic unknown sidecar",
                )
                .unwrap();
                assert!(
                    AnchoredTree::capture(&root, &custody, &mut claim, &root, deadline).is_err()
                );
            }
        }
        // Wrong-claim validation poisons that lease, whose explicit release
        // must continue to report failure while releasing only owned ranges.
        if case == 0 {
            assert!(lease.release(deadline).is_err());
        } else {
            lease.release(deadline).unwrap();
        }
        assert!(std::fs::read_dir(&custody).unwrap().next().is_none());
        f.retain();
    }
}
