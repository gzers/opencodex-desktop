// Bounded complete image inventory for publication, reuse, repair and removal.
// It is separate from high-frequency current-selection freshness checks.
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
    thread_local! {static PATH_REFERENCE:std::cell::Cell<bool>=const{std::cell::Cell::new(false)};}
    pub(super) fn path_reference_enabled() -> bool {
        PATH_REFERENCE.with(|v| v.get())
    }
    pub(super) fn path_reference<T>(action: impl FnOnce() -> T) -> T {
        struct Reset;
        impl Drop for Reset {
            fn drop(&mut self) {
                PATH_REFERENCE.with(|v| v.set(false));
            }
        }
        PATH_REFERENCE.with(|v| {
            assert!(!v.get());
            v.set(true);
        });
        let _reset = Reset;
        action()
    }
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
        pub(super) fn new(
            enabled: bool,
            results: &'a std::sync::Mutex<Vec<Metrics>>,
            path_reference: bool,
        ) -> Self {
            PATH_REFERENCE.with(|v| v.set(path_reference));
            if enabled {
                begin();
            }
            Self { enabled, results }
        }
    }
    impl Drop for Worker<'_> {
        fn drop(&mut self) {
            PATH_REFERENCE.with(|v| v.set(false));
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
#[cfg(windows)]
#[path = "program_image_windows.rs"]
mod windows;
/// Inventory requires existing paths only. Each component gets the same native
/// symlink/reparse/type admission once, before any content read. Do not turn
/// this into a cached directory grant or use it for create/missing paths.
fn check_inventory_path(root: &Path, path: &Path) -> AppResult<()> {
    #[cfg(test)]
    let _boundary = cost::span("existing-path-boundary");
    #[cfg(test)]
    if cost::path_reference_enabled() {
        return safety::check_path(root, path, false);
    }
    if !root.is_absolute() || !safety::inspect(root)?.is_dir() {
        return Err(fail());
    }
    let relative = path.strip_prefix(root).map_err(|_| fail())?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        if !matches!(component, std::path::Component::Normal(_)) {
            return Err(fail());
        }
        current.push(component);
        safety::inspect(&current)?;
    }
    Ok(())
}
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
    #[cfg(test)]
    let _cost = cost::span("inventory-enumeration-total");
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
        check_inventory_path(root, &directory)?;
        let before = safety::inspect(&directory)?;
        if !before.is_dir() {
            return Err(fail());
        }
        #[cfg(test)]
        drop(directory_cost);
        #[cfg(test)]
        let enumeration_cost = cost::span("directory-enumeration");
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
        #[cfg(test)]
        drop(enumeration_cost);
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
    #[cfg(test)]
    let _cost = cost::span("inventory-read-and-fence-total");
    use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
    let workers = workers.clamp(1, MAX_WORKERS).min(draft.jobs.len().max(1));
    let next = AtomicUsize::new(0);
    let cancelled = AtomicBool::new(false);
    let reserved_bytes = AtomicU64::new(0);
    #[cfg(test)]
    let measure = cost::active();
    #[cfg(test)]
    let path_reference = cost::path_reference_enabled();
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
                    let _cost = cost::Worker::new(measure, worker_metrics, path_reference);
                    let mut rows = Vec::new();
                    while !cancelled.load(Ordering::Acquire) {
                        let i = next.fetch_add(1, Ordering::Relaxed);
                        let Some((name, path)) = jobs.get(i) else {
                            break;
                        };
                        let result = checkpoint(path)
                            .and_then(|()| check_inventory_path(root, path))
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
            #[cfg(test)]
            let _join_cost = cost::span("worker-join-wait");
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
        #[cfg(test)]
        let _cost = cost::span("directory-final-fence");
        check_inventory_path(root, directory)?;
        let after = safety::inspect(directory)?;
        if !same_directory(before, &after)? {
            return Err(fail());
        }
    }
    Ok(draft.rows)
}
pub(crate) fn inventory(root: &Path) -> AppResult<BTreeMap<String, Entry>> {
    #[cfg(test)]
    let _cost = cost::span("inventory-total");
    #[cfg(windows)]
    {
        windows::inventory(root, None)
    }
    #[cfg(not(windows))]
    {
        finish_inventory(root, collect_inventory(root)?, MAX_WORKERS)
    }
}
#[cfg(windows)]
pub(crate) fn inventory_until(
    root: &Path,
    end: std::time::Instant,
) -> AppResult<BTreeMap<String, Entry>> {
    windows::inventory(root, Some(end))
}
#[cfg(all(test, windows))]
#[path = "program_image_windows_tests.rs"]
mod windows_tests;
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
fn verify_inventory(
    root: &Path,
    expected_tree: &str,
    expected_manifest: &str,
    end: Option<std::time::Instant>,
) -> AppResult<()> {
    let rows = {
        #[cfg(windows)]
        {
            match end {
                Some(end) => inventory_until(root, end)?,
                None => inventory(root)?,
            }
        }
        #[cfg(not(windows))]
        {
            inventory(root)?
        }
    };
    if rows.is_empty() || inventory_digest(&rows)? != expected_tree {
        return Err(fail());
    }
    let manifest_path = root.join(super::program_selection::IMAGE_MANIFEST);
    safety::check_path(root, &manifest_path, false)?;
    let file = safety::open_file(&manifest_path, false)?;
    safety::require_single_link(&file)?;
    let mut manifest = Vec::new();
    file.take(64 * 1024 + 1)
        .read_to_end(&mut manifest)
        .map_err(|_| fail())?;
    if manifest.len() > 64 * 1024 {
        return Err(fail());
    }
    if hash::sha256_hex(&manifest) != expected_manifest {
        return Err(fail());
    }
    if end.is_some_and(|end| std::time::Instant::now() >= end) {
        return Err(fail());
    }
    Ok(())
}
