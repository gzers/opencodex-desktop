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
