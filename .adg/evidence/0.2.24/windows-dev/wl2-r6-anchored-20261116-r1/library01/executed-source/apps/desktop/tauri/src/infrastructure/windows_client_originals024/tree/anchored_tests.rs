use super::super::super::coordination::NativeLease;
use super::super::tests::Fixture;
use super::*;
use rusqlite::Connection;
use std::time::Duration;
fn end() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
fn prepare(f: &Fixture) -> (PathBuf, PathBuf, [PathBuf; 4]) {
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
fn compete(path: &Path, blocked: bool) {
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
    compete(&paths[0], false);
    println!("r6 exact original directory/two stable file IDs/all8 and complete children retained; original files restored by original handles; work generations retired; lease explicitly released; component only, Start/Closed=false");
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
