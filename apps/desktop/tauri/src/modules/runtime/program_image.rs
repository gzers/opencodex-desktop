//! Bounded complete image inventory for publication, reuse, repair and removal.
//! It is separate from high-frequency current-selection freshness checks.
use crate::{
    errors::{AppError, AppResult},
    infrastructure::hash,
    modules::backup::safety,
};
use std::{collections::BTreeMap, fs, io::Read, path::Path};
const MAX_ENTRIES: usize = 100_000;
const MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_FILE: u64 = 512 * 1024 * 1024;
#[cfg(test)]
mod cost {
    use std::{cell::RefCell, time::Instant};
    #[derive(Default, serde::Serialize)]
    pub(super) struct Metrics {
        pub(super) stages: std::collections::BTreeMap<&'static str, (u64, u128)>,
    }
    thread_local! {static ACTIVE:RefCell<Option<Metrics>>=const{RefCell::new(None)};}
    pub(super) fn begin() {
        ACTIVE.with(|v| {
            assert!(v.borrow().is_none());
            *v.borrow_mut() = Some(Metrics::default());
        });
    }
    pub(super) fn end() -> Metrics {
        ACTIVE.with(|v| v.borrow_mut().take().expect("active cost observation"))
    }
    pub(super) fn active() -> bool {
        ACTIVE.with(|v| v.borrow().is_some())
    }
    pub(super) fn merge(metrics: Metrics) {
        ACTIVE.with(|v| {
            if let Some(current) = v.borrow_mut().as_mut() {
                for (name, (count, nanos)) in metrics.stages {
                    let value = current.stages.entry(name).or_default();
                    value.0 += count;
                    value.1 += nanos;
                }
            }
        });
    }
    pub(super) struct Worker<'a> {
        enabled: bool,
        results: &'a std::sync::Mutex<Vec<Metrics>>,
    }
    impl<'a> Worker<'a> {
        pub(super) fn new(enabled: bool, results: &'a std::sync::Mutex<Vec<Metrics>>) -> Self {
            if enabled {
                begin();
            }
            Self { enabled, results }
        }
    }
    impl Drop for Worker<'_> {
        fn drop(&mut self) {
            if self.enabled {
                if let Ok(mut results) = self.results.lock() {
                    results.push(end());
                }
            }
        }
    }
    pub(super) struct Span(&'static str, Instant);
    pub(super) fn span(label: &'static str) -> Span {
        Span(label, Instant::now())
    }
    impl Drop for Span {
        fn drop(&mut self) {
            ACTIVE.with(|v| {
                if let Some(m) = v.borrow_mut().as_mut() {
                    let stage = m.stages.entry(self.0).or_default();
                    stage.0 += 1;
                    stage.1 += self.1.elapsed().as_nanos();
                }
            });
        }
    }
}
fn fail() -> AppError {
    AppError::RuntimeManaged {
        code: "runtime_image_unconfirmed".into(),
        detail: "运行版本文件树无法核对，原目录保留".into(),
    }
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub(crate) enum Entry {
    Directory,
    File {
        bytes: u64,
        sha256: String,
        executable: bool,
    },
}
/// Excludes only the root image manifest to avoid a self-referential hash. The
/// manifest itself is bound by CurrentSelection or the exact activation receipt.
pub fn content_digest(root: &Path) -> AppResult<String> {
    let rows = inventory(root)?;
    if rows.is_empty() {
        return Err(fail());
    }
    inventory_digest(&rows)
}
pub(crate) fn inventory_digest(rows: &BTreeMap<String, Entry>) -> AppResult<String> {
    let bytes = serde_json::to_vec(rows).map_err(|_| fail())?;
    Ok(hash::sha256_hex(&bytes))
}
const MAX_WORKERS: usize = 4;
struct InventoryDraft {
    rows: BTreeMap<String, Entry>,
    jobs: Vec<(String, std::path::PathBuf)>,
    directories: Vec<(std::path::PathBuf, fs::Metadata)>,
}
fn same_directory(before: &fs::Metadata, after: &fs::Metadata) -> AppResult<bool> {
    if !after.is_dir()
        || before.modified().map_err(|_| fail())? != after.modified().map_err(|_| fail())?
    {
        return Ok(false);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if before.dev() != after.dev()
            || before.ino() != after.ino()
            || before.mode() != after.mode()
            || before.uid() != after.uid()
            || before.gid() != after.gid()
        {
            return Ok(false);
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if before.creation_time() != after.creation_time()
            || before.file_attributes() != after.file_attributes()
        {
            return Ok(false);
        }
    }
    Ok(true)
}
fn collect_inventory(root: &Path) -> AppResult<InventoryDraft> {
    if !super::program_layout::clean_absolute(root) || !safety::inspect(root)?.is_dir() {
        return Err(fail());
    }
    let mut draft = InventoryDraft {
        rows: BTreeMap::new(),
        jobs: Vec::new(),
        directories: Vec::new(),
    };
    let mut seen = std::collections::BTreeSet::new();
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    let mut count = 0usize;
    let mut bytes = 0u64;
    while let Some((directory, depth)) = stack.pop() {
        if depth > 64 {
            return Err(fail());
        }
        #[cfg(test)]
        let directory_cost = cost::span("directory-path-admission");
        safety::check_path(root, &directory, false)?;
        let before = safety::inspect(&directory)?;
        if !before.is_dir() {
            return Err(fail());
        }
        #[cfg(test)]
        drop(directory_cost);
        for item in fs::read_dir(&directory).map_err(|_| fail())? {
            count += 1;
            if count > MAX_ENTRIES {
                return Err(fail());
            }
            let path = item.map_err(|_| fail())?.path();
            #[cfg(test)]
            let inspect_cost = cost::span("entry-inspect-name");
            let meta = safety::inspect(&path)?;
            let relative = path.strip_prefix(root).map_err(|_| fail())?;
            let name = relative
                .to_str()
                .ok_or_else(fail)?
                .replace(char::from(92), "/");
            #[cfg(test)]
            drop(inspect_cost);
            if depth == 0 && name == super::program_selection::IMAGE_MANIFEST {
                if !meta.is_file() {
                    return Err(fail());
                }
                continue;
            }
            if !seen.insert(name.clone()) {
                return Err(fail());
            }
            if meta.is_dir() {
                draft.rows.insert(name, Entry::Directory);
                stack.push((path, depth + 1));
            } else {
                if meta.len() > MAX_FILE {
                    return Err(fail());
                }
                bytes = bytes
                    .checked_add(meta.len())
                    .filter(|v| *v <= MAX_BYTES)
                    .ok_or_else(fail)?;
                draft.jobs.push((name, path));
            }
        }
        let after = safety::inspect(&directory)?;
        if !same_directory(&before, &after)? {
            return Err(fail());
        }
        draft.directories.push((directory, before));
    }
    Ok(draft)
}
fn finish_inventory(
    root: &Path,
    draft: InventoryDraft,
    workers: usize,
) -> AppResult<BTreeMap<String, Entry>> {
    finish_inventory_checked(root, draft, workers, &|_| Ok(()))
}
fn finish_inventory_checked(
    root: &Path,
    mut draft: InventoryDraft,
    workers: usize,
    checkpoint: &(impl Fn(&Path) -> AppResult<()> + Sync),
) -> AppResult<BTreeMap<String, Entry>> {
    use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
    let workers = workers.clamp(1, MAX_WORKERS).min(draft.jobs.len().max(1));
    let next = AtomicUsize::new(0);
    let cancelled = AtomicBool::new(false);
    let reserved_bytes = AtomicU64::new(0);
    #[cfg(test)]
    let measure = cost::active();
    #[cfg(test)]
    let worker_metrics = std::sync::Mutex::new(Vec::new());
    let jobs = &draft.jobs;
    let completed = std::thread::scope(|scope| -> AppResult<Vec<(String, Entry)>> {
        let mut handles = Vec::with_capacity(workers);
        let mut error = None;
        for _ in 0..workers {
            let next = &next;
            let cancelled = &cancelled;
            let reserved_bytes = &reserved_bytes;
            #[cfg(test)]
            let worker_metrics = &worker_metrics;
            let created = std::thread::Builder::new()
                .name("ocxd-image-read".into())
                .spawn_scoped(scope, move || -> AppResult<Vec<(String, Entry)>> {
                    struct CancelOnUnwind<'a>(&'a AtomicBool);
                    impl Drop for CancelOnUnwind<'_> {
                        fn drop(&mut self) {
                            if std::thread::panicking() {
                                self.0.store(true, Ordering::Release);
                            }
                        }
                    }
                    let _unwind = CancelOnUnwind(cancelled);
                    #[cfg(test)]
                    let _cost = cost::Worker::new(measure, worker_metrics);
                    let mut rows = Vec::new();
                    while !cancelled.load(Ordering::Acquire) {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some((name, path)) = jobs.get(i) else {
                            break;
                        };
                        let result = checkpoint(path)
                            .and_then(|()| safety::check_path(root, path, false))
                            .and_then(|()| file_entry_budgeted(path, Some(reserved_bytes)));
                        match result {
                            Ok(entry) => rows.push((name.clone(), entry)),
                            Err(e) => {
                                cancelled.store(true, Ordering::Release);
                                return Err(e);
                            }
                        }
                    }
                    Ok(rows)
                });
            match created {
                Ok(handle) => handles.push(handle),
                Err(_) => {
                    cancelled.store(true, Ordering::Release);
                    error = Some(fail());
                    break;
                }
            }
        }
        // Always join EVERY owner before returning any failure. No detached IO
        // continues into cleanup and no successful subset is returned.
        let mut rows = Vec::with_capacity(jobs.len());
        for h in handles {
            match h.join() {
                Ok(Ok(mut part)) => rows.append(&mut part),
                Ok(Err(e)) => {
                    if error.is_none() {
                        error = Some(e);
                    }
                }
                Err(_) => {
                    cancelled.store(true, Ordering::Release);
                    if error.is_none() {
                        error = Some(fail());
                    }
                }
            }
        }
        if let Some(error) = error {
            return Err(error);
        }
        if rows.len() != jobs.len() {
            return Err(fail());
        }
        Ok(rows)
    });
    #[cfg(test)]
    if let Ok(metrics) = worker_metrics.into_inner() {
        for metrics in metrics {
            cost::merge(metrics);
        }
    }
    let mut total = 0u64;
    for (name, entry) in completed? {
        let Entry::File { bytes, .. } = &entry else {
            return Err(fail());
        };
        total = total
            .checked_add(*bytes)
            .filter(|v| *v <= MAX_BYTES)
            .ok_or_else(fail)?;
        if draft.rows.insert(name, entry).is_some() {
            return Err(fail());
        }
    }
    // Keep the enumeration-end check and also fence all original directory
    // paths/identities/mtimes AFTER the scoped byte readers finish.
    for (directory, before) in &draft.directories {
        safety::check_path(root, directory, false)?;
        let after = safety::inspect(directory)?;
        if !same_directory(before, &after)? {
            return Err(fail());
        }
    }
    Ok(draft.rows)
}
pub(crate) fn inventory(root: &Path) -> AppResult<BTreeMap<String, Entry>> {
    finish_inventory(root, collect_inventory(root)?, MAX_WORKERS)
}
#[cfg(test)]
fn inventory_serial(root: &Path) -> AppResult<BTreeMap<String, Entry>> {
    if !super::program_layout::clean_absolute(root) || !safety::inspect(root)?.is_dir() {
        return Err(fail());
    }
    let mut rows = BTreeMap::new();
    let mut stack = vec![(root.to_path_buf(), 0usize)];
    let mut count = 0usize;
    let mut total = 0u64;
    while let Some((directory, depth)) = stack.pop() {
        #[cfg(test)]
        let directory_cost = cost::span("directory-path-admission");
        if depth > 64 {
            return Err(fail());
        }
        safety::check_path(root, &directory, false)?;
        let directory_before = fs::metadata(&directory).map_err(|_| fail())?;
        #[cfg(test)]
        drop(directory_cost);
        for item in fs::read_dir(&directory).map_err(|_| fail())? {
            let item = item.map_err(|_| fail())?;
            count += 1;
            if count > MAX_ENTRIES {
                return Err(fail());
            }
            let path = item.path();
            #[cfg(test)]
            let inspect_cost = cost::span("entry-inspect-name");
            let meta = safety::inspect(&path)?;
            let relative = path.strip_prefix(root).map_err(|_| fail())?;
            let name = relative
                .to_str()
                .ok_or_else(fail)?
                .replace(char::from(92), "/");
            #[cfg(test)]
            drop(inspect_cost);
            if depth == 0 && name == super::program_selection::IMAGE_MANIFEST {
                if !meta.is_file() {
                    return Err(fail());
                }
                continue;
            }
            if rows.contains_key(&name) {
                return Err(fail());
            }
            let entry = if meta.is_dir() {
                stack.push((path, depth + 1));
                Entry::Directory
            } else {
                let entry = file_entry(&path)?;
                let Entry::File { bytes, .. } = &entry else {
                    return Err(fail());
                };
                total = total
                    .checked_add(*bytes)
                    .filter(|n| *n <= MAX_BYTES)
                    .ok_or_else(fail)?;
                entry
            };
            rows.insert(name, entry);
        }
        let directory_after = fs::metadata(&directory).map_err(|_| fail())?;
        if directory_before.modified().map_err(|_| fail())?
            != directory_after.modified().map_err(|_| fail())?
        {
            return Err(fail());
        }
    }
    Ok(rows)
}
pub(crate) fn file_entry(path: &Path) -> AppResult<Entry> {
    file_entry_budgeted(path, None)
}
fn file_entry_budgeted(
    path: &Path,
    reserved: Option<&std::sync::atomic::AtomicU64>,
) -> AppResult<Entry> {
    file_entry_checked(path, reserved, &|_| Ok(()))
}
fn file_entry_checked(
    path: &Path,
    reserved: Option<&std::sync::atomic::AtomicU64>,
    checkpoint: &impl Fn(&str) -> AppResult<()>,
) -> AppResult<Entry> {
    #[cfg(test)]
    let total_cost = cost::span("file-total");
    #[cfg(test)]
    let admission_cost = cost::span("file-admission-open");
    if !safety::inspect(path)?.is_file() {
        return Err(fail());
    }
    let file = safety::open_file(path, false)?;
    safety::require_single_link(&file)?;
    let before = file.metadata().map_err(|_| fail())?;
    if before.len() > MAX_FILE {
        return Err(fail());
    }
    // Reserve this exact length before ANY read. A job grown after traversal
    // cannot exceed the original aggregate2GiB bound. Failed reservations are
    // not reused; the whole observation refuses. Keep the original EOF growth
    // probe: a corrupt in-flight file may read ONE extra byte before refusing,
    // so failure-only probe overshoot is bounded by MAX_WORKERS, not another
    // entire file. Valid inventory bytes still cannot exceed MAX_BYTES.
    if let Some(total) = reserved {
        total
            .fetch_update(
                std::sync::atomic::Ordering::AcqRel,
                std::sync::atomic::Ordering::Acquire,
                |n| n.checked_add(before.len()).filter(|v| *v <= MAX_BYTES),
            )
            .map_err(|_| fail())?;
    }
    #[cfg(test)]
    drop(admission_cost);
    checkpoint("admitted")?;
    #[cfg(test)]
    let hash_cost = cost::span("file-read-hash");
    let limit = if reserved.is_some() {
        before.len().checked_add(1).ok_or_else(fail)?
    } else {
        MAX_FILE + 1
    };
    let mut read = file.take(limit);
    let mut digest = sha2::Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut bytes = 0u64;
    use sha2::Digest;
    loop {
        let n = read.read(&mut buffer).map_err(|_| fail())?;
        if n == 0 {
            break;
        }
        bytes += n as u64;
        if bytes > before.len() {
            return Err(fail());
        }
        digest.update(&buffer[..n]);
    }
    checkpoint("read-complete")?;
    let after = read.get_ref().metadata().map_err(|_| fail())?;
    #[cfg(test)]
    drop(hash_cost);
    safety::require_single_link(read.get_ref())?;
    if bytes != before.len()
        || after.len() != before.len()
        || after.modified().map_err(|_| fail())? != before.modified().map_err(|_| fail())?
    {
        return Err(fail());
    }
    #[cfg(unix)]
    let executable = {
        use std::os::unix::fs::PermissionsExt;
        if after.permissions().mode() != before.permissions().mode() {
            return Err(fail());
        }
        before.permissions().mode() & 0o111 != 0
    };
    #[cfg(not(unix))]
    let executable = false;
    #[cfg(test)]
    drop(total_cost);
    Ok(Entry::File {
        bytes,
        sha256: format!("{:x}", digest.finalize()),
        executable,
    })
}
pub fn verify(selected: &super::program_selection::SelectedProgram) -> AppResult<()> {
    if content_digest(&selected.paths.version_root)? != selected.image.tree_sha256 {
        return Err(fail());
    }
    let manifest_path = selected
        .paths
        .version_root
        .join(super::program_selection::IMAGE_MANIFEST);
    safety::check_path(&selected.paths.version_root, &manifest_path, false)?;
    let file = safety::open_file(&manifest_path, false)?;
    safety::require_single_link(&file)?;
    let mut manifest = Vec::new();
    file.take(64 * 1024 + 1)
        .read_to_end(&mut manifest)
        .map_err(|_| fail())?;
    if manifest.len() > 64 * 1024 {
        return Err(fail());
    }
    if hash::sha256_hex(&manifest) != selected.selection.image_sha256 {
        return Err(fail());
    }
    Ok(())
}

#[cfg(all(test, unix))]
#[path = "program_image_parallel_tests.rs"]
mod parallel_tests;
#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::modules::data_root::layout::Platform;
    use crate::modules::runtime::program_selection::{self, tests::Fixture};
    use std::os::unix::fs::PermissionsExt;
    #[test]
    #[ignore = "explicit owned official2.99 image read-only cost observation; no service actions"]
    fn official299_complete_image_inventory_stage_cost() {
        let root = std::path::PathBuf::from(
            std::env::var_os("OCXD024_IMAGE_SCAN_FIXTURE_ROOT").expect("new owned fixture"),
        );
        assert!(
            root.is_absolute()
                && root.canonicalize().unwrap() == root
                && root.starts_with("/private/tmp")
                && root
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with("ocxd024-versioned-lifecycle-")
        );
        assert_eq!(
            std::env::var_os("HOME"),
            Some(root.join("profile").into_os_string())
        );
        let descriptor = fs::read(root.join("resources/runtime-bundle.manifest.json")).unwrap();
        let pin = hash::sha256_hex(&descriptor);
        assert_eq!(
            pin,
            "e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3"
        );
        let program = root.join("program 中文 space");
        let data = root.join("profile/desktop");
        if !program.exists() {
            super::super::bundled::versioned::provision(
                &program,
                &data,
                &root.join("resources/runtime-bundle-0.2.24-darwin-arm64.tar.gz"),
                &descriptor,
                &pin,
            )
            .unwrap();
        }
        let selected = program_selection::read(&program, Platform::Macos, "darwin-arm64")
            .unwrap()
            .unwrap();
        assert_eq!(
            selected.image.official_archive_sha256,
            "b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c"
        );
        if std::env::var_os("OCXD024_IMAGE_SCAN_COMPARE").is_some() {
            let serial_run = || {
                cost::begin();
                let start = std::time::Instant::now();
                let rows = inventory_serial(&selected.paths.version_root).unwrap();
                let elapsed = start.elapsed().as_micros();
                let metrics = cost::end();
                (rows, elapsed, metrics)
            };
            let parallel_run = || {
                cost::begin();
                let start = std::time::Instant::now();
                let rows = inventory(&selected.paths.version_root).unwrap();
                let elapsed = start.elapsed().as_micros();
                let metrics = cost::end();
                (rows, elapsed, metrics)
            };
            let parallel_first = std::env::var_os("OCXD024_IMAGE_SCAN_PARALLEL_FIRST").is_some();
            let ((serial, serial_us, serial_metrics), (parallel, parallel_us, parallel_metrics)) =
                if parallel_first {
                    let parallel = parallel_run();
                    (serial_run(), parallel)
                } else {
                    let serial = serial_run();
                    (serial, parallel_run())
                };
            assert_eq!(serial, parallel);
            assert_eq!(
                inventory_digest(&parallel).unwrap(),
                selected.image.tree_sha256
            );
            let expected_files = parallel
                .values()
                .filter(|v| matches!(v, Entry::File { .. }))
                .count() as u64;
            assert_eq!(
                parallel_metrics.stages["file-read-hash"].0, expected_files,
                "all worker measurements must be joined, not lost thread-local samples"
            );
            let report = serde_json::json!({"accepted":false,"scope":"same owned exact official image, old sequential reference versus bounded4 current full inventory; no Stop acceptance","entries":parallel.len(),"fileCount":expected_files,"serialUs":serial_us,"parallelUs":parallel_us,"parallelFirst":parallel_first,"serialMetrics":serial_metrics,"parallelMetrics":parallel_metrics,"treeSha256":selected.image.tree_sha256,"rowsExactlyEqual":true});
            fs::write(
                root.join("image-scan-comparison.json"),
                serde_json::to_vec_pretty(&report).unwrap(),
            )
            .unwrap();
            println!("IMAGE_SCAN_COMPARISON {report}");
            return;
        }
        cost::begin();
        let started = std::time::Instant::now();
        let rows = inventory(&selected.paths.version_root).unwrap();
        let elapsed = started.elapsed();
        let metrics = cost::end();
        let digest = inventory_digest(&rows).unwrap();
        assert_eq!(digest, selected.image.tree_sha256);
        let bytes: u64 = rows
            .values()
            .map(|r| match r {
                Entry::File { bytes, .. } => *bytes,
                _ => 0,
            })
            .sum();
        let report = serde_json::json!({"accepted":false,"scope":"actual official image inventory only; no service or Stop acceptance","entries":rows.len(),"fileBytes":bytes,"elapsedUs":elapsed.as_micros(),"metrics":metrics,"treeSha256":digest,"officialArchiveSha256":selected.image.official_archive_sha256,"versionRoot":selected.paths.version_root});
        fs::write(
            root.join("image-scan-baseline.json"),
            serde_json::to_vec_pretty(&report).unwrap(),
        )
        .unwrap();
        println!("IMAGE_SCAN_COST {report}");
    }
    #[test]
    fn action_time_inventory_detects_noncritical_changes_without_reading_other_directories() {
        for case in [
            "added",
            "empty_dir",
            "changed",
            "executable",
            "symlink",
            "hardlink",
        ] {
            let f = Fixture::new();
            let selected = program_selection::read(&f.root, Platform::Macos, "darwin-arm64")
                .unwrap()
                .unwrap();
            verify(&selected).unwrap();
            let extra = f.paths.version_root.join("added-file");
            match case {
                "added" => fs::write(extra, b"user file that must not be deleted").unwrap(),
                "empty_dir" => {
                    fs::create_dir(f.paths.version_root.join("empty-user-directory")).unwrap()
                }
                "changed" => fs::write(&f.paths.npm, b"different npm module").unwrap(),
                "executable" => {
                    fs::set_permissions(&f.paths.npm, fs::Permissions::from_mode(0o755)).unwrap()
                }
                "symlink" => {
                    std::os::unix::fs::symlink(f.root.join("never-read-missing"), extra).unwrap()
                }
                "hardlink" => fs::hard_link(&f.paths.npm, extra).unwrap(),
                _ => unreachable!(),
            }
            assert!(verify(&selected).is_err(), "{case}");
            assert!(f.paths.version_root.exists());
        }
    }
    #[test]
    fn seal_excludes_only_root_manifest_and_still_binds_it_through_selection() {
        let f = Fixture::new();
        let selected = program_selection::read(&f.root, Platform::Macos, "darwin-arm64")
            .unwrap()
            .unwrap();
        let before = content_digest(&f.paths.version_root).unwrap();
        fs::write(
            f.paths.version_root.join(program_selection::IMAGE_MANIFEST),
            b"changed manifest",
        )
        .unwrap();
        assert_eq!(content_digest(&f.paths.version_root).unwrap(), before);
        assert!(verify(&selected).is_err());
        fs::write(
            f.paths.package_root.join(program_selection::IMAGE_MANIFEST),
            b"nested manifest is ordinary content",
        )
        .unwrap();
        assert_ne!(content_digest(&f.paths.version_root).unwrap(), before);
    }
    #[test]
    fn rejects_sparse_oversize_without_allocating_payload_and_preserves_file() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().canonicalize().unwrap();
        let path = root.join("oversize");
        let file = fs::File::create(&path).unwrap();
        file.set_len(MAX_FILE + 1).unwrap();
        assert!(content_digest(&root).is_err());
        assert_eq!(fs::metadata(path).unwrap().len(), MAX_FILE + 1);
    }
}
