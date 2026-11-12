use super::*;
use std::{
    os::unix::fs::{symlink, PermissionsExt},
    sync::atomic::{AtomicU64, AtomicUsize, Ordering},
};
fn fixture() -> tempfile::TempDir {
    let t = tempfile::tempdir().unwrap();
    let root = t.path();
    fs::create_dir(root.join("empty")).unwrap();
    fs::create_dir(root.join("nested")).unwrap();
    for i in 0..64 {
        let path = if i % 2 == 0 {
            root.join(format!("file-{i:02}"))
        } else {
            root.join("nested").join(format!("file-{i:02}"))
        };
        fs::write(&path, vec![i as u8; 1000 + i]).unwrap();
        fs::set_permissions(
            path,
            fs::Permissions::from_mode(if i % 3 == 0 { 0o755 } else { 0o644 }),
        )
        .unwrap();
    }
    fs::write(
        root.join(super::super::program_selection::IMAGE_MANIFEST),
        b"self manifest excluded",
    )
    .unwrap();
    fs::write(
        root.join("nested")
            .join(super::super::program_selection::IMAGE_MANIFEST),
        b"ordinary nested bytes",
    )
    .unwrap();
    t
}
#[test]
fn bounded_image_readers_match_serial_every_row_digest_empty_directory_and_executable_bit() {
    let t = fixture();
    let root = t.path().canonicalize().unwrap();
    let serial = inventory_serial(&root).unwrap();
    for workers in [1, 4, usize::MAX] {
        let parallel = finish_inventory(&root, collect_inventory(&root).unwrap(), workers).unwrap();
        assert_eq!(parallel, serial);
        assert_eq!(
            inventory_digest(&parallel).unwrap(),
            inventory_digest(&serial).unwrap()
        );
    }
    assert!(matches!(serial["empty"], Entry::Directory));
    assert!(!serial.contains_key(super::super::program_selection::IMAGE_MANIFEST));
    assert!(serial.contains_key("nested/.runtime-image.json"));
}
#[test]
fn delayed_image_jobs_refuse_retargeted_ancestor_before_reading_foreign_payload() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().canonicalize().unwrap();
    fs::create_dir(root.join("sub")).unwrap();
    fs::write(root.join("sub/one"), b"original").unwrap();
    let draft = collect_inventory(&root).unwrap();
    let outside = tempfile::tempdir().unwrap();
    fs::write(
        outside.path().join("one"),
        b"foreign payload never authorized",
    )
    .unwrap();
    fs::rename(root.join("sub"), root.join("retained")).unwrap();
    symlink(outside.path(), root.join("sub")).unwrap();
    cost::begin();
    assert!(finish_inventory(&root, draft, 4).is_err());
    let measured = cost::end();
    assert!(
        !measured.stages.contains_key("file-total"),
        "ancestor must refuse before file_entry, not only after a foreign read"
    );
    assert_eq!(
        fs::read(outside.path().join("one")).unwrap(),
        b"foreign payload never authorized"
    );
}
#[test]
fn finished_image_inventory_refuses_directory_change_during_reader_work_without_partial_success() {
    let t = fixture();
    let root = t.path().canonicalize().unwrap();
    let draft = collect_inventory(&root).unwrap();
    let changed = std::sync::atomic::AtomicBool::new(false);
    assert!(finish_inventory_checked(&root, draft, 4, &|_| {
        if !changed.swap(true, Ordering::SeqCst) {
            fs::write(root.join("new-user-file"), b"keep added file").unwrap();
        }
        Ok(())
    })
    .is_err());
    assert!(changed.load(Ordering::SeqCst));
    assert_eq!(
        fs::read(root.join("new-user-file")).unwrap(),
        b"keep added file"
    );
}
#[test]
fn shared_image_read_budget_is_reserved_before_hashing_even_with_competing_readers() {
    let t = tempfile::tempdir().unwrap();
    let root = t.path().canonicalize().unwrap();
    let path = root.join("small");
    fs::write(&path, [b'x'; 1024]).unwrap();
    let used = AtomicU64::new(MAX_BYTES - 2048);
    let reads = std::thread::scope(|scope| {
        let mut hs = Vec::new();
        for _ in 0..4 {
            let used = &used;
            let path = &path;
            hs.push(scope.spawn(move || {
                cost::begin();
                let result = file_entry_budgeted(path, Some(used));
                let cost = cost::end();
                (result, cost)
            }));
        }
        hs.into_iter()
            .map(|h| h.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert_eq!(reads.iter().filter(|(r, _)| r.is_ok()).count(), 2);
    assert_eq!(used.load(Ordering::Acquire), MAX_BYTES);
    for (result, metrics) in reads {
        if result.is_err() {
            assert!(
                !metrics.stages.contains_key("file-read-hash"),
                "failed reservation cannot perform content IO"
            );
        }
    }
    assert_eq!(fs::metadata(&path).unwrap().len(), 1024);
}
#[test]
fn budgeted_image_reader_keeps_eof_growth_probe_before_any_later_size_restore() {
    let t = tempfile::tempdir().unwrap();
    let path = t.path().join("one");
    fs::write(&path, b"x").unwrap();
    let original_time = fs::metadata(&path).unwrap().modified().unwrap();
    let grew = std::sync::atomic::AtomicBool::new(false);
    let restored = std::sync::atomic::AtomicBool::new(false);
    let used = AtomicU64::new(0);
    let result = file_entry_checked(&path, Some(&used), &|phase| {
        if phase == "admitted" {
            fs::write(&path, b"xy").unwrap();
            grew.store(true, Ordering::SeqCst);
        }
        if phase == "read-complete" {
            fs::write(&path, b"x").unwrap();
            fs::File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_times(fs::FileTimes::new().set_modified(original_time))
                .unwrap();
            restored.store(true, Ordering::SeqCst);
        }
        Ok(())
    });
    assert!(result.is_err());
    assert!(grew.load(Ordering::SeqCst));
    assert!(
        !restored.load(Ordering::SeqCst),
        "growth must refuse on the extra EOF byte before post-read metadata can be restored"
    );
    assert_eq!(used.load(Ordering::SeqCst), 1);
    assert_eq!(fs::read(&path).unwrap(), b"xy");
}
#[test]
fn bounded_image_workers_join_and_refuse_panic_or_failure_before_caller_cleanup() {
    for panic in [false, true] {
        let t = fixture();
        let root = t.path().canonicalize().unwrap();
        let draft = collect_inventory(&root).unwrap();
        let active = AtomicUsize::new(0);
        let peak = AtomicUsize::new(0);
        let jobs = AtomicUsize::new(0);
        let result = finish_inventory_checked(&root, draft, usize::MAX, &|_| {
            struct Exit<'a>(&'a AtomicUsize);
            impl Drop for Exit<'_> {
                fn drop(&mut self) {
                    self.0.fetch_sub(1, Ordering::SeqCst);
                }
            }
            let now = active.fetch_add(1, Ordering::SeqCst) + 1;
            let _exit = Exit(&active);
            peak.fetch_max(now, Ordering::SeqCst);
            let job = jobs.fetch_add(1, Ordering::SeqCst);
            std::thread::sleep(std::time::Duration::from_millis(3));
            if job == 0 {
                if panic {
                    panic!("owned worker fault injection");
                } else {
                    return Err(fail());
                }
            }
            Ok(())
        });
        assert!(result.is_err());
        assert!(jobs.load(Ordering::SeqCst) > 0);
        assert_eq!(
            active.load(Ordering::SeqCst),
            0,
            "all callbacks/IO scopes must have unwound before caller may clean"
        );
        assert!(peak.load(Ordering::SeqCst) <= MAX_WORKERS);
        assert!(root.join("nested/file-01").exists());
    }
}
#[test]
fn parallel_image_name_collision_links_and_late_sparse_growth_never_return_an_inventory() {
    for case in ["collision", "symlink", "hardlink", "growth"] {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().canonicalize().unwrap();
        fs::create_dir(root.join("a")).unwrap();
        fs::write(root.join("a/b"), b"one").unwrap();
        if case == "collision" {
            fs::write(root.join("a\\b"), b"different normalized path").unwrap();
            assert!(inventory(&root).is_err());
            continue;
        }
        let draft = collect_inventory(&root).unwrap();
        match case {
            "symlink" => {
                fs::rename(root.join("a/b"), root.join("retained")).unwrap();
                symlink(root.join("retained"), root.join("a/b")).unwrap();
            }
            "hardlink" => fs::hard_link(root.join("a/b"), root.join("retained")).unwrap(),
            "growth" => {
                fs::OpenOptions::new()
                    .write(true)
                    .open(root.join("a/b"))
                    .unwrap()
                    .set_len(MAX_FILE + 1)
                    .unwrap();
            }
            _ => unreachable!(),
        }
        assert!(finish_inventory(&root, draft, 4).is_err());
        assert!(root.join("a/b").exists());
    }
}
