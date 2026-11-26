//! P0: measure the actual original complete scanner, not selected-file checks.
use super::*;
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
struct Scene(Option<tempfile::TempDir>);
impl Scene {
    fn new() -> Self {
        Self(Some(
            tempfile::Builder::new()
                .prefix("ocx024-r7-image-")
                .tempdir()
                .unwrap(),
        ))
    }
    fn root(&self) -> PathBuf {
        self.0.as_ref().unwrap().path().canonicalize().unwrap()
    }
}
impl Drop for Scene {
    fn drop(&mut self) {
        if std::thread::panicking() {
            if let Some(scene) = self.0.take() {
                eprintln!(
                    "failed owned scanner fixture retained: {}",
                    scene.keep().display()
                );
            }
        } else if let Some(scene) = self.0.take() {
            let path = scene.path().to_path_buf();
            scene.close().unwrap_or_else(|e| {
                panic!(
                    "owned successful scanner fixture cleanup failed; path={} raw={:?} {e}",
                    path.display(),
                    e.raw_os_error()
                )
            });
        }
    }
}
fn scan(
    root: &Path,
    workers: usize,
    ordinal: usize,
) -> (BTreeMap<String, Entry>, serde_json::Value) {
    cost::begin();
    let started = Instant::now();
    let result = finish_inventory(root, collect_inventory(root).unwrap(), workers);
    let elapsed = started.elapsed();
    let metrics = cost::end();
    let rows = result.unwrap();
    let bytes = rows
        .values()
        .map(|r| match r {
            Entry::File { bytes, .. } => *bytes,
            _ => 0,
        })
        .sum::<u64>();
    let report = serde_json::json!({"ordinal":ordinal,"workers":workers,"source":"same actual program_image inventory section","entries":rows.len(),"fileBytes":bytes,"treeSha256":inventory_digest(&rows).unwrap(),"elapsedUs":elapsed.as_micros(),"stages":metrics,"metricNote":"worker nanoseconds sum overlapping spans; join wait includes worker I/O, not pure CPU cost","nativeOwner":false,"officialExecuted":false,"productionBudgetAccepted":false});
    (rows, report)
}
fn populate(root: &Path) {
    fs::create_dir(root.join("empty")).unwrap();
    fs::create_dir(root.join("nested")).unwrap();
    fs::write(root.join("one"), b"abcdef").unwrap();
    fs::write(root.join("nested/two"), b"same length content").unwrap();
    fs::write(
        root.join("nested/.runtime-image.json"),
        b"included ordinary bytes",
    )
    .unwrap();
    fs::write(
        root.join(".runtime-image.json"),
        b"root manifest excluded only",
    )
    .unwrap();
}
#[test]
fn r7_image_baseline_worker1_default4_and_serial_reference_cover_every_byte_and_empty_directory() {
    let scene = Scene::new();
    let root = scene.root();
    populate(&root);
    let reference = inventory_serial(&root).unwrap();
    let (one, one_cost) = scan(&root, 1, 0);
    let (four, four_cost) = scan(&root, 4, 1);
    assert_eq!(one, reference);
    assert_eq!(four, reference);
    assert_eq!(four.len(), 5);
    assert_eq!(four.get("empty"), Some(&Entry::Directory));
    assert!(four.contains_key("nested/.runtime-image.json"));
    assert!(!four.contains_key(".runtime-image.json"));
    println!(
        "R7_SCAN_BASELINE {}",
        serde_json::json!({"samples":[one_cost,four_cost],"referenceDigest":inventory_digest(&reference).unwrap()})
    );
}
#[test]
fn r7_image_same_length_byte_change_add_delete_empty_directory_and_hardlink_are_not_cached() {
    for case in [
        "same-length-byte",
        "add",
        "delete",
        "empty-directory",
        "hardlink",
    ] {
        let scene = Scene::new();
        let root = scene.root();
        populate(&root);
        let before = content_digest(&root).unwrap();
        match case {
            "same-length-byte" => fs::write(root.join("one"), b"abcdeg").unwrap(),
            "add" => fs::write(root.join("new"), b"new").unwrap(),
            "delete" => fs::remove_file(root.join("one")).unwrap(),
            "empty-directory" => fs::create_dir(root.join("another-empty")).unwrap(),
            "hardlink" => fs::hard_link(root.join("one"), root.join("linked")).unwrap(),
            _ => unreachable!(),
        }
        if case == "hardlink" {
            assert!(content_digest(&root).is_err());
        } else {
            assert_ne!(content_digest(&root).unwrap(), before, "{case}");
        }
    }
}
#[test]
fn r7_image_sparse_file_bound_refuses_without_reading_payload() {
    let scene = Scene::new();
    let root = scene.root();
    let path = root.join("oversize");
    let file = fs::File::create(&path).unwrap();
    file.set_len(MAX_FILE + 1).unwrap();
    drop(file);
    assert!(inventory(&root).is_err());
    assert_eq!(fs::metadata(path).unwrap().len(), MAX_FILE + 1);
}
#[test]
fn r7_image_worker_failure_joins_every_started_worker_before_returning_error() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Condvar, Mutex,
    };
    let scene = Scene::new();
    let root = scene.root();
    for n in 0..16 {
        fs::write(root.join(format!("file-{n}")), vec![b'a'; 65536]).unwrap();
    }
    let draft = collect_inventory(&root).unwrap();
    let state = Arc::new((Mutex::new(0usize), Condvar::new()));
    let done = AtomicUsize::new(0);
    let result = finish_inventory_checked(&root, draft, 4, &|_| {
        let (lock, wake) = &*state;
        let mut arrived = lock.lock().unwrap();
        *arrived += 1;
        let ordinal = *arrived;
        wake.notify_all();
        let (arrived, timed) = wake
            .wait_timeout_while(arrived, Duration::from_secs(2), |n| *n < 4)
            .unwrap();
        if timed.timed_out() || *arrived < 4 {
            return Err(fail());
        }
        drop(arrived);
        done.fetch_add(1, Ordering::SeqCst);
        if ordinal == 1 {
            Err(fail())
        } else {
            Ok(())
        }
    });
    assert!(result.is_err());
    assert!(done.load(Ordering::SeqCst) >= 4);
    let joined = done.load(Ordering::SeqCst);
    // A scoped join has already completed. No worker may increment after return.
    std::thread::yield_now();
    assert_eq!(done.load(Ordering::SeqCst), joined);
}
#[test]
#[ignore = "explicit fixed copied image only; no product/Owner/official launch"]
fn r7_image_fixed_copy_stage_metrics_worker1_and_default4() {
    let root = PathBuf::from(
        std::env::var_os("OCXD024_R7_FIXED_SCAN_ROOT").expect("explicit copied image root"),
    );
    let root = root.canonicalize().unwrap();
    assert!(root
        .to_string_lossy()
        .contains("r6-source-scan-cost-d57c32d10af74ab888fcde86e6ec97cd"));
    assert_eq!(root.file_name().unwrap(), "2.99.0");
    let raw = fs::read(root.join(".runtime-image.json")).unwrap();
    assert_eq!(
        hash::sha256_hex(&raw),
        "ab9314d3df4475481736245580b24a4d1a6fa52e8030512d9afc9340b6d6c3b6"
    );
    let mut samples = Vec::new();
    for (ordinal, workers) in [1, 4].into_iter().enumerate() {
        let (rows, report) = scan(&root, workers, ordinal);
        assert_eq!(
            inventory_digest(&rows).unwrap(),
            "306e50d0620aa99b61200ec4c0af8c200fb2f12f3b45ee2068dd85dc35f5fd55"
        );
        assert_eq!(fs::read(root.join(".runtime-image.json")).unwrap(), raw);
        samples.push(report);
    }
    println!(
        "R7_FIXED_SCAN_METRICS {}",
        serde_json::json!({"samples":samples,"readOnlyCopiedFixture":true,"productionAccepted":false})
    );
}

#[test]
fn r7_image_pinned_full_inventory_matches_reference_workers1_and4() {
    let scene = Scene::new();
    let root = scene.root();
    populate(&root);
    let reference = inventory_serial(&root).unwrap();
    for workers in [1, 4] {
        let pinned = windows::inventory_checked(&root, None, workers, &|_| Ok(())).unwrap();
        assert_eq!(pinned, reference);
        assert_eq!(
            inventory_digest(&pinned).unwrap(),
            inventory_digest(&reference).unwrap()
        );
    }
}
#[test]
fn r7_image_pinned_objects_deny_content_write_delete_and_same_name_replacement() {
    let scene = Scene::new();
    let root = scene.root();
    populate(&root);
    let pinned = windows::inventory_checked(&root, None, 4, &|stage| {
        if stage == "all-objects-pinned" {
            assert!(fs::OpenOptions::new()
                .write(true)
                .open(root.join("one"))
                .is_err());
            assert!(fs::remove_file(root.join("one")).is_err());
            assert!(fs::rename(root.join("one"), root.join("retired")).is_err());
            assert!(fs::rename(root.join("nested"), root.join("substituted")).is_err());
            assert!(fs::rename(&root, root.with_extension("replaced")).is_err());
        }
        Ok(())
    })
    .unwrap();
    assert_eq!(pinned, inventory_serial(&root).unwrap());
    fs::write(root.join("one"), b"change").unwrap();
    assert_ne!(windows::inventory(&root, None).unwrap(), pinned);
}
#[test]
fn r7_image_stale_enumeration_type_refuses_both_replacement_directions() {
    use std::sync::atomic::{AtomicBool, Ordering};

    for initially_directory in [false, true] {
        let scene = Scene::new();
        let root = scene.root();
        let leaf = root.join("child");
        if initially_directory {
            fs::create_dir(&leaf).unwrap();
        } else {
            fs::write(&leaf, b"original file").unwrap();
        }
        let replaced = AtomicBool::new(false);
        let result = windows::inventory_checked(&root, None, 4, &|stage| {
            if stage == "root-children-listed" {
                assert!(!replaced.swap(true, Ordering::AcqRel));
                if initially_directory {
                    fs::remove_dir(&leaf).unwrap();
                    fs::write(&leaf, b"replacement file").unwrap();
                } else {
                    fs::remove_file(&leaf).unwrap();
                    fs::create_dir(&leaf).unwrap();
                }
            }
            assert_ne!(stage, "all-objects-pinned");
            Ok(())
        });
        assert!(replaced.load(Ordering::Acquire));
        assert!(
            matches!(result, Err(crate::errors::AppError::RuntimeManaged { ref code, .. }) if code == "runtime_image_unconfirmed")
        );
        assert_eq!(
            windows::inventory(&root, None).unwrap(),
            inventory_serial(&root).unwrap()
        );
    }
}

#[test]
fn r7_image_pinned_full_membership_detects_extra_file_and_directory_in_final_fence() {
    for directory in [false, true] {
        let scene = Scene::new();
        let root = scene.root();
        populate(&root);
        let observed = windows::inventory_checked(&root, None, 4, &|stage| {
            if stage == "before-final-fence" {
                if directory {
                    fs::create_dir(root.join("new-empty")).unwrap();
                } else {
                    fs::write(root.join("new-file"), b"owned extra").unwrap();
                }
            }
            Ok(())
        });
        assert!(observed.is_err());
    }
}
#[test]
fn r7_image_pinned_ads_hardlink_sparse_and_expired_deadline_are_refused() {
    for case in ["ADS", "hardlink", "sparse", "expired"] {
        let scene = Scene::new();
        let root = scene.root();
        populate(&root);
        match case {
            "ADS" => fs::write(root.join("one:r7-test"), b"alternate stream").unwrap(),
            "hardlink" => fs::hard_link(root.join("one"), root.join("linked")).unwrap(),
            "sparse" => {
                fs::File::create(root.join("oversize"))
                    .unwrap()
                    .set_len(MAX_FILE + 1)
                    .unwrap();
            }
            _ => {}
        }
        let end = if case == "expired" {
            Some(Instant::now() - Duration::from_millis(1))
        } else {
            None
        };
        assert!(windows::inventory(&root, end).is_err(), "{case}");
    }
}
#[test]
fn r7_image_pinned_refuses_directory_stream_added_after_payload_reads() {
    let scene = Scene::new();
    let root = scene.root();
    populate(&root);
    let reference = inventory_serial(&root).unwrap();
    assert_eq!(windows::inventory(&root, None).unwrap(), reference);

    let stream = root.join("empty:r7-late-stream");
    let read_jobs = std::sync::atomic::AtomicUsize::new(0);
    let result = windows::inventory_checked(&root, None, 4, &|stage| {
        if stage == "before-file-read" {
            read_jobs.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        if stage == "before-final-fence" {
            assert_eq!(read_jobs.load(std::sync::atomic::Ordering::Relaxed), 3);
            fs::write(&stream, b"late directory stream").unwrap();
        }
        Ok(())
    });
    assert_eq!(fs::read(&stream).unwrap(), b"late directory stream");
    assert!(matches!(
        result,
        Err(AppError::RuntimeManaged { code, .. }) if code == "runtime_image_unconfirmed"
    ));
    assert_eq!(fs::read_dir(root.join("empty")).unwrap().count(), 0);
    fs::remove_file(&stream).unwrap();
    assert_eq!(windows::inventory(&root, None).unwrap(), reference);
    println!("R7_LATE_DIRECTORY_STREAM root={} reads=3", root.display());
}
#[test]
fn r7_image_pinned_worker_error_and_panic_join_all_started_readers() {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Condvar, Mutex,
    };
    for panic_worker in [false, true] {
        let scene = Scene::new();
        let root = scene.root();
        for i in 0..16 {
            fs::write(root.join(format!("file-{i}")), vec![b'a'; 65536]).unwrap();
        }
        let state = (Mutex::new(0usize), Condvar::new());
        let finished = AtomicUsize::new(0);
        let result = windows::inventory_checked(&root, None, 4, &|stage| {
            if stage != "before-file-read" {
                return Ok(());
            }
            let mut n = state.0.lock().unwrap();
            *n += 1;
            let i = *n;
            state.1.notify_all();
            let (n, timed) = state
                .1
                .wait_timeout_while(n, Duration::from_secs(2), |n| *n < 4)
                .unwrap();
            assert!(!timed.timed_out());
            drop(n);
            finished.fetch_add(1, Ordering::SeqCst);
            if i == 1 {
                if panic_worker {
                    panic!("owned injected worker failure");
                }
                return Err(fail());
            }
            Ok(())
        });
        assert!(result.is_err());
        assert!(finished.load(Ordering::SeqCst) >= 4);
        fs::write(
            root.join("file-0"),
            b"pins all released after readers joined",
        )
        .unwrap();
    }
}

#[test]
#[ignore = "explicit complete fixed owned image reference/new implementation measurement"]
fn r7_image_fixed_pinned_reference_same_full_bytes_and_digest() {
    let root =
        PathBuf::from(std::env::var_os("OCXD024_R7_FIXED_SCAN_ROOT").expect("explicit fixed copy"));
    let root = root.canonicalize().unwrap();
    assert!(root
        .to_string_lossy()
        .contains("r6-source-scan-cost-d57c32d10af74ab888fcde86e6ec97cd"));
    let expected = "306e50d0620aa99b61200ec4c0af8c200fb2f12f3b45ee2068dd85dc35f5fd55";
    let mut reports = Vec::new();
    let mut reference = None;
    for (ordinal, (native, workers)) in [(false, 4), (true, 4), (false, 1), (true, 1)]
        .into_iter()
        .enumerate()
    {
        cost::begin();
        let started = Instant::now();
        let result = if native {
            windows::inventory_checked(&root, None, workers, &|_| Ok(()))
        } else {
            finish_inventory(&root, collect_inventory(&root).unwrap(), workers)
        };
        let elapsed = started.elapsed();
        let metrics = cost::end();
        let rows = result.unwrap();
        let digest = inventory_digest(&rows).unwrap();
        assert_eq!(digest, expected);
        if let Some(reference) = &reference {
            assert_eq!(&rows, reference);
        } else {
            reference = Some(rows.clone());
        }
        let bytes = rows
            .values()
            .map(|r| match r {
                Entry::File { bytes, .. } => *bytes,
                _ => 0,
            })
            .sum::<u64>();
        reports.push(serde_json::json!({"ordinal":ordinal,"implementation":if native{"native-pinned"}else{"reference"},"workers":workers,"elapsedUs":elapsed.as_micros(),"entries":rows.len(),"fileBytes":bytes,"treeSha256":digest,"metrics":metrics,"exactInventoryEqual":true,"productionAccepted":false}));
    }
    println!(
        "R7_PINNED_FIXED_SCAN {}",
        serde_json::json!({"samples":reports,"sameOriginalCompleteBytes":true,"productionBudgetAccepted":false})
    );
}

#[test]
fn r7_image_pinned_acl_change_during_read_is_refused_at_security_fence() {
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
    let scene = Scene::new();
    let root = scene.root();
    populate(&root);
    // OWNER RIGHTS binds only the actual owner of this new synthetic file;
    // the protected DACL differs from its original inherited permissions.
    let sddl = "D:P(A;;FA;;;OW)(A;;FA;;;SY)"
        .encode_utf16()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let mut descriptor = std::ptr::null_mut();
    assert_ne!(
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                sddl.as_ptr(),
                1,
                &mut descriptor,
                std::ptr::null_mut(),
            )
        },
        0
    );
    struct SD(*mut std::ffi::c_void);
    impl Drop for SD {
        fn drop(&mut self) {
            unsafe {
                windows_sys::Win32::Foundation::LocalFree(self.0);
            }
        }
    }
    let sd = SD(descriptor);
    let length =
        unsafe { windows_sys::Win32::Security::GetSecurityDescriptorLength(sd.0) } as usize;
    assert!((20..=262144).contains(&length));
    let descriptor_data = unsafe { std::slice::from_raw_parts(sd.0.cast::<u8>(), length) }.to_vec();
    drop(sd);
    let name = root
        .join("one")
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let result = windows::inventory_checked(&root, None, 4, &|stage| {
        if stage == "before-final-fence" {
            assert_ne!(
                unsafe { SetFileSecurityW(name.as_ptr(), 4, descriptor_data.as_ptr().cast()) },
                0,
                "own fixture DACL must actually change"
            );
        }
        Ok(())
    });
    assert!(result.is_err());
}
#[test]
fn r7_image_pinned_deadline_expires_without_returning_subset_or_detached_reader() {
    let scene = Scene::new();
    let root = scene.root();
    populate(&root);
    let started = Instant::now();
    let end = started + Duration::from_millis(50);
    let result = windows::inventory_checked(&root, Some(end), 4, &|stage| {
        if stage == "all-objects-pinned" {
            std::thread::sleep(
                end.saturating_duration_since(Instant::now()) + Duration::from_millis(5),
            );
        }
        Ok(())
    });
    assert!(result.is_err());
    assert!(started.elapsed() < Duration::from_secs(1));
    fs::write(
        root.join("one"),
        b"all failed readers joined and original pins released",
    )
    .unwrap();
}
#[test]
fn r7_image_pinned_junction_is_refused_without_reading_target_files() {
    use std::os::windows::fs::MetadataExt;
    let scene = Scene::new();
    let root = scene.root();
    populate(&root);
    let link = root.join("junction");
    let target = root.join("nested");
    use std::os::windows::{fs::OpenOptionsExt, io::AsRawHandle};
    // Native self-owned junction fixture. No shell, policy change or symbolic
    // link privilege: FSCTL_SET_REPARSE_POINT on this newly created directory.
    fs::create_dir(&link).unwrap();
    let print = target
        .to_string_lossy()
        .strip_prefix(r"\\?\")
        .unwrap_or(&target.to_string_lossy())
        .to_owned();
    let substitute = format!(r"\??\{print}").encode_utf16().collect::<Vec<_>>();
    let print = print.encode_utf16().collect::<Vec<_>>();
    let sub_bytes = (substitute.len() * 2) as u16;
    let print_bytes = (print.len() * 2) as u16;
    let mut data = Vec::new();
    data.extend_from_slice(&0xa0000003u32.to_le_bytes());
    data.extend_from_slice(&(8u16 + sub_bytes + 2 + print_bytes + 2).to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&0u16.to_le_bytes());
    data.extend_from_slice(&sub_bytes.to_le_bytes());
    data.extend_from_slice(&(sub_bytes + 2).to_le_bytes());
    data.extend_from_slice(&print_bytes.to_le_bytes());
    for c in substitute
        .into_iter()
        .chain(Some(0))
        .chain(print)
        .chain(Some(0))
    {
        data.extend_from_slice(&c.to_le_bytes());
    }
    let pin = fs::OpenOptions::new()
        .access_mode(0x40000000)
        .share_mode(3)
        .custom_flags(0x2200000)
        .open(&link)
        .unwrap();
    let mut returned = 0;
    let ok = unsafe {
        windows_sys::Win32::System::IO::DeviceIoControl(
            pin.as_raw_handle(),
            589988,
            data.as_ptr().cast(),
            data.len() as u32,
            std::ptr::null_mut(),
            0,
            &mut returned,
            std::ptr::null_mut(),
        )
    };
    assert_ne!(
        ok,
        0,
        "native owned junction setup raw={:?}",
        std::io::Error::last_os_error()
    );
    drop(pin);
    assert_ne!(
        fs::symlink_metadata(&link).unwrap().file_attributes() & 0x400,
        0
    );
    assert!(windows::inventory(&root, None).is_err());
    assert_eq!(
        fs::read(target.join("two")).unwrap(),
        b"same length content"
    );
}
#[test]
fn r7_image_pinned_aggregate2gib_bound_refuses_before_any_payload_read() {
    use std::os::windows::io::AsRawHandle;
    let scene = Scene::new();
    let root = scene.root();
    for i in 0..5 {
        let file = fs::File::create(root.join(format!("sparse-{i}"))).unwrap();
        let mut returned = 0u32;
        let ok = unsafe {
            windows_sys::Win32::System::IO::DeviceIoControl(
                file.as_raw_handle(),
                590020,
                std::ptr::null(),
                0,
                std::ptr::null_mut(),
                0,
                &mut returned,
                std::ptr::null_mut(),
            )
        };
        assert_ne!(ok, 0, "self-owned sparse fixture native setup required");
        file.set_len(MAX_FILE).unwrap();
    }
    let calls = std::sync::atomic::AtomicUsize::new(0);
    let result = windows::inventory_checked(&root, None, 4, &|stage| {
        if stage == "before-file-read" {
            calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        Ok(())
    });
    assert!(result.is_err());
    assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 0);
}
#[test]
fn r7_image_pinned_full_inventory_accepts_long_native_paths() {
    use std::os::windows::ffi::OsStrExt;
    let scene = Scene::new();
    let root = scene.root();
    let component = "n".repeat(180);
    let directory = root.join(&component).join(&component).join(&component);
    let path = directory.join("payload");
    assert!(path.as_os_str().encode_wide().count() > 512);
    fs::create_dir_all(&directory).unwrap();
    let payload = b"read through the long path query";
    fs::write(&path, payload).unwrap();
    let rows = windows::inventory(&root, None).unwrap();
    let key = format!("{component}/{component}/{component}/payload");
    assert_eq!(rows.len(), 4);
    assert_eq!(
        rows[&key],
        Entry::File {
            bytes: payload.len() as u64,
            sha256: hash::sha256_hex(payload),
            executable: false,
        }
    );
}
#[test]
fn r7_image_pinned_depth65_preserves_original64_directory_bound() {
    let scene = Scene::new();
    let root = scene.root();
    let mut path = root.clone();
    for _ in 0..65 {
        path.push("n");
        fs::create_dir(&path).unwrap();
    }
    assert!(windows::inventory(&root, None).is_err());
    assert!(collect_inventory(&root).is_err());
}
#[test]
fn r7_image_pinned_full_verify_still_binds_excluded_root_manifest_and_original_deadline() {
    let scene = Scene::new();
    let root = scene.root();
    populate(&root);
    let tree = inventory_digest(&inventory(&root).unwrap()).unwrap();
    let manifest = hash::sha256_file(&root.join(".runtime-image.json")).unwrap();
    verify_inventory(
        &root,
        &tree,
        &manifest,
        Some(Instant::now() + Duration::from_secs(5)),
    )
    .unwrap();
    assert!(verify_inventory(&root, &tree, &manifest, Some(Instant::now())).is_err());
    fs::write(
        root.join(".runtime-image.json"),
        b"changed excluded root manifest",
    )
    .unwrap();
    assert_eq!(inventory_digest(&inventory(&root).unwrap()).unwrap(), tree);
    assert!(verify_inventory(&root, &tree, &manifest, None).is_err());
    fs::write(root.join("one"), b"abcdeg").unwrap();
    assert!(verify_inventory(&root, &tree, &manifest, None).is_err());
}
#[test]
#[ignore = "explicit fixed copied image Windows entry/full manifest readback under a real unchanged10s test window"]
fn r7_image_fixed_pinned_entry_full_verify_with_original10s_deadline() {
    let root = PathBuf::from(
        std::env::var_os("OCXD024_R7_FIXED_SCAN_ROOT").expect("explicit fixed copied source"),
    );
    let root = root.canonicalize().unwrap();
    assert!(root
        .to_string_lossy()
        .contains("r6-source-scan-cost-d57c32d10af74ab888fcde86e6ec97cd"));
    cost::begin();
    let started = Instant::now();
    let end = started + Duration::from_secs(10);
    let result = verify_inventory(
        &root,
        "306e50d0620aa99b61200ec4c0af8c200fb2f12f3b45ee2068dd85dc35f5fd55",
        "ab9314d3df4475481736245580b24a4d1a6fa52e8030512d9afc9340b6d6c3b6",
        Some(end),
    );
    let elapsed = started.elapsed();
    let metrics = cost::end();
    println!(
        "R7_PINNED_ENTRY_FULL_VERIFY {}",
        serde_json::json!({"passed":result.is_ok(),"elapsedUs":elapsed.as_micros(),"budgetMs":10000,"error":result.as_ref().err().map(|e|e.to_string()),"completeInventoryAndManifestReadback":result.is_ok(),"metrics":metrics,"copiedFixture":true,"officialExecuted":false,"productionRecoveryAccepted":false})
    );
    result.unwrap();
    assert!(elapsed <= Duration::from_secs(10));
}
