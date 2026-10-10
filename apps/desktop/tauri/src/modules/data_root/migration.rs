//! Copy/verification kernel. Does not change bindings or remove the old root.
//!
//! The command layer must freeze every writer and own the source instance lock
//! before calling this module. The returned target lock must survive until exit.
//! Activation remains separate from copying and requires runtime preparation,
//! the startup anchor lock, and a writer freeze through process exit.

pub mod activation;

use super::{DataRootRuntimeConfig, OpenCodexHomeMode, PARTITIONS};
use crate::errors::{AppError, AppResult};
use crate::infrastructure::hash::sha256_file;
use crate::modules::instance::AppInstanceLock;
use serde::Serialize;
use std::fs::{self, File, OpenOptions};
use std::path::{Component, Path, PathBuf};

const MAX_ENTRIES: usize = 1_000_000;
const MAX_DEPTH: usize = 128;
pub const RECEIPT_PATH: &str = "manager-state/data-root-migration.json";
pub const UNPUBLISHED_MARKER: &str = ".data-root-migration-incomplete";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Entry {
    Directory {
        path: PathBuf,
    },
    File {
        path: PathBuf,
        bytes: u64,
        sha256: String,
    },
    Link {
        path: PathBuf,
        target: PathBuf,
        directory: bool,
    },
}

impl Entry {
    fn path(&self) -> &Path {
        match self {
            Self::Directory { path } | Self::File { path, .. } | Self::Link { path, .. } => path,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CopyReceipt {
    pub schema_version: u32,
    /// "verified_copy" is not a committed binding or successful restart.
    pub phase: &'static str,
    pub source: PathBuf,
    pub target: PathBuf,
    pub excluded: Vec<&'static str>,
    pub bytes: u64,
    pub entries: Vec<Entry>,
    /// Inventory after rewriting only the target self-binding, excluding this
    /// receipt and the unpublished marker. Runtime relocation is still pending.
    pub prepared_entries: Vec<Entry>,
}

pub struct VerifiedCopy {
    pub receipt: CopyReceipt,
    /// Prevent another instance from repairing or starting the unpublished copy.
    pub target_lock: AppInstanceLock,
}

impl VerifiedCopy {
    /// Prepare path-bearing runtime files without publishing the target. The
    /// caller still owns the source lock and writer freeze throughout this step.
    pub fn prepare_runtime(&mut self) -> AppResult<()> {
        if self.receipt.phase != "verified_copy" {
            return Err(failure(
                "runtime preparation requires a fresh verified copy",
            ));
        }
        let target = &self.receipt.target;
        if !fs::symlink_metadata(target.join(UNPUBLISHED_MARKER))
            .map_err(io)?
            .is_file()
        {
            return Err(failure("unpublished marker must be a regular file"));
        }
        let external_home = self
            .receipt
            .excluded
            .iter()
            .any(|path| path.starts_with("opencodex-home"));
        let original_receipt =
            serde_json::to_vec_pretty(&self.receipt).map_err(|e| failure(e.to_string()))?;
        if fs::read(target.join(RECEIPT_PATH)).map_err(io)? != original_receipt
            || inventory(&self.receipt.source, external_home)? != self.receipt.entries
            || inventory(target, external_home)? != self.receipt.prepared_entries
        {
            return Err(failure(
                "copy receipt or inventory changed before runtime preparation",
            ));
        }
        crate::modules::runtime::relocation::prepare(&self.receipt.source, target)?;
        let prepared = inventory(target, external_home)?;
        let unchanged = |entries: &[Entry]| {
            entries
                .iter()
                .filter(|entry| {
                    entry.path() != Path::new("manager-state/runtime.json")
                        && entry.path()
                            != Path::new(crate::modules::runtime::MANAGED_ENTRY_RELATIVE)
                })
                .cloned()
                .collect::<Vec<_>>()
        };
        if inventory(&self.receipt.source, external_home)? != self.receipt.entries
            || unchanged(&prepared) != unchanged(&self.receipt.prepared_entries)
        {
            return Err(failure("unexpected file change during runtime preparation"));
        }
        self.receipt.prepared_entries = prepared;
        self.receipt.phase = "runtime_prepared";
        let raw = serde_json::to_vec_pretty(&self.receipt).map_err(|e| failure(e.to_string()))?;
        crate::infrastructure::atomic_write::atomic_write(&target.join(RECEIPT_PATH), &raw, 0o600)?;
        if fs::read(target.join(RECEIPT_PATH)).map_err(io)? != raw {
            return Err(failure("runtime preparation receipt failed readback"));
        }
        Ok(())
    }
}

fn failure(message: impl Into<String>) -> AppError {
    AppError::FileSystem {
        operation: "copy data root".into(),
        detail: message.into(),
    }
}

fn io(error: std::io::Error) -> AppError {
    failure(error.to_string())
}

/// Marker presence of ANY type blocks startup; a malformed marker is not a
/// license to repair a partially copied tree. Only a later binding transaction
/// may remove it after runtime relocation and final verification.
pub fn require_published(root: &Path) -> AppResult<()> {
    match fs::symlink_metadata(root.join(UNPUBLISHED_MARKER)) {
        Ok(_) => Err(failure(
            "data root migration is incomplete or uncommitted; old binding retained",
        )),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(io(e)),
    }
}

fn reserve_unpublished(target: &Path) -> AppResult<()> {
    let parent = target
        .parent()
        .ok_or_else(|| failure("target has no parent"))?;
    let staging = parent.join(format!(".ocx-migration-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&staging).map_err(io)?;
    super::set_private_directory(&staging)?;
    crate::infrastructure::atomic_write::atomic_write(
        &staging.join(UNPUBLISHED_MARKER),
        b"unpublished verified-copy staging; not an active data root",
        0o600,
    )?;
    sync_directory(&staging)?;
    // Never fall back to a replacing rename. Existing empty directories and
    // dangling symlinks are user-owned targets too.
    rename_exclusive(&staging, target)?;
    sync_directory(parent)
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> AppResult<()> {
    File::open(path).map_err(io)?.sync_all().map_err(io)
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> AppResult<()> {
    Ok(())
}

#[cfg(any(target_os = "macos", target_os = "linux"))]
fn rename_exclusive(source: &Path, target: &Path) -> AppResult<()> {
    use std::os::unix::ffi::OsStrExt;
    let source = std::ffi::CString::new(source.as_os_str().as_bytes())
        .map_err(|_| failure("invalid staging path"))?;
    let target = std::ffi::CString::new(target.as_os_str().as_bytes())
        .map_err(|_| failure("invalid target path"))?;
    #[cfg(target_os = "macos")]
    // SAFETY: both NUL-terminated paths remain alive for this filesystem call.
    let result = unsafe { libc::renamex_np(source.as_ptr(), target.as_ptr(), libc::RENAME_EXCL) };
    #[cfg(target_os = "linux")]
    // SAFETY: both NUL-terminated paths remain alive; flags forbid replacement.
    let result = unsafe {
        libc::renameat2(
            libc::AT_FDCWD,
            source.as_ptr(),
            libc::AT_FDCWD,
            target.as_ptr(),
            libc::RENAME_NOREPLACE,
        )
    };
    if result == 0 {
        Ok(())
    } else {
        Err(io(std::io::Error::last_os_error()))
    }
}

#[cfg(windows)]
fn rename_exclusive(source: &Path, target: &Path) -> AppResult<()> {
    use std::os::windows::ffi::OsStrExt;
    let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let target: Vec<u16> = target.as_os_str().encode_wide().chain(Some(0)).collect();
    if source[..source.len() - 1].contains(&0) || target[..target.len() - 1].contains(&0) {
        return Err(failure("invalid path"));
    }
    // SAFETY: paths are NUL-terminated and live for this call. No REPLACE_EXISTING
    // or COPY_ALLOWED flags: the sibling directory move cannot replace a target.
    let result = unsafe {
        windows_sys::Win32::Storage::FileSystem::MoveFileExW(
            source.as_ptr(),
            target.as_ptr(),
            windows_sys::Win32::Storage::FileSystem::MOVEFILE_WRITE_THROUGH,
        )
    };
    if result != 0 {
        Ok(())
    } else {
        Err(io(std::io::Error::last_os_error()))
    }
}

#[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
fn rename_exclusive(_source: &Path, _target: &Path) -> AppResult<()> {
    Err(failure(
        "exclusive data root reservation is unsupported on this platform",
    ))
}

fn excluded(path: &Path, external_home: bool) -> bool {
    [
        "manager-state/app.lock",
        ".backup-w2.lock",
        "cache/ipc/opencodex.ipc",
        RECEIPT_PATH,
        activation::VERIFICATION_PATH,
        UNPUBLISHED_MARKER,
    ]
    .iter()
    .any(|excluded| path == Path::new(excluded))
        || (external_home && path.starts_with("opencodex-home"))
}

fn allowed_partition(path: &Path) -> bool {
    PARTITIONS.iter().any(|(_, name)| path == Path::new(name)) || path == Path::new("runtime")
}

fn inventory(root: &Path, external_home: bool) -> AppResult<Vec<Entry>> {
    let mut entries = Vec::new();
    visit(root, Path::new(""), external_home, &mut entries, 0)?;
    entries.sort_by(|a, b| a.path().cmp(b.path()));
    Ok(entries)
}

fn visit(
    root: &Path,
    relative: &Path,
    external_home: bool,
    entries: &mut Vec<Entry>,
    depth: usize,
) -> AppResult<()> {
    if depth > MAX_DEPTH {
        return Err(failure("directory depth exceeds migration limit"));
    }
    for child in fs::read_dir(root.join(relative)).map_err(io)? {
        let child = child.map_err(io)?;
        let path = relative.join(child.file_name());
        if excluded(&path, external_home) {
            continue;
        }
        if relative.as_os_str().is_empty() && !allowed_partition(&path) {
            return Err(failure(
                "unregistered root entry requires an explicit migration scope",
            ));
        }
        if entries.len() >= MAX_ENTRIES {
            return Err(failure("entry count exceeds migration limit"));
        }
        let absolute = root.join(&path);
        let metadata = fs::symlink_metadata(&absolute).map_err(io)?;
        if metadata.file_type().is_symlink() {
            let resolved = absolute.canonicalize().map_err(io)?;
            if !resolved.starts_with(root)
                || (external_home && resolved.starts_with(root.join("opencodex-home")))
            {
                return Err(failure("symbolic link escapes migration scope"));
            }
            // Store the resolved root-relative target, never the old absolute root.
            let target = resolved
                .strip_prefix(root)
                .map_err(|_| failure("invalid link target"))?
                .to_path_buf();
            if excluded(&target, external_home) {
                return Err(failure("link points to an excluded runtime file"));
            }
            entries.push(Entry::Link {
                path,
                target,
                directory: resolved.is_dir(),
            });
        } else if metadata.is_dir() {
            entries.push(Entry::Directory { path: path.clone() });
            visit(root, &path, external_home, entries, depth + 1)?;
        } else if metadata.is_file() {
            entries.push(Entry::File {
                path,
                bytes: metadata.len(),
                sha256: sha256_file(&absolute).map_err(io)?,
            });
        } else {
            return Err(failure("unsupported special file in migration scope"));
        }
    }
    Ok(())
}

/// Requires a NEW final directory under an existing parent. Never merges a copy
/// into an existing tree, and never changes source bytes or its binding.
/// On failure the reserved target may remain incomplete; it is not activated.
pub fn copy_verified(config: &DataRootRuntimeConfig, target: &Path) -> AppResult<VerifiedCopy> {
    copy_with(config, target, |_, _| Ok(()))
}

fn copy_with(
    config: &DataRootRuntimeConfig,
    target: &Path,
    after_copy: impl FnOnce(&Path, &Path) -> AppResult<()>,
) -> AppResult<VerifiedCopy> {
    if !target.is_absolute()
        || target
            .components()
            .any(|c| matches!(c, Component::ParentDir))
    {
        return Err(failure("target must be an absolute normalized path"));
    }
    let source = config.active_data_root.canonicalize().map_err(io)?;
    if super::validate_structure(&source)? != super::StructureValidation::Valid {
        return Err(failure("source structure is unsupported or corrupted"));
    }
    let parent = target
        .parent()
        .ok_or_else(|| failure("target has no parent"))?
        .canonicalize()
        .map_err(io)?;
    let target = parent.join(
        target
            .file_name()
            .ok_or_else(|| failure("target has no name"))?,
    );
    if target.starts_with(&source) || source.starts_with(&target) {
        return Err(failure("source and target cannot be nested"));
    }
    if let Some(home) = config
        .opencodex_home_path
        .as_ref()
        .filter(|_| config.opencodex_home_mode == OpenCodexHomeMode::External)
    {
        let home = resolve_future_path(home)?;
        if home.starts_with(&target)
            || target.starts_with(&home)
            || home.starts_with(&source)
            || source.starts_with(&home)
        {
            return Err(failure("external HOME intersects the migration roots"));
        }
    }
    let external_home = config.opencodex_home_mode == OpenCodexHomeMode::External;
    let entries = inventory(&source, external_home)?;
    let bytes = entries.iter().try_fold(0u64, |sum, entry| match entry {
        Entry::File { bytes, .. } => sum
            .checked_add(*bytes)
            .ok_or_else(|| failure("copy size overflow")),
        _ => Ok(sum),
    })?;
    // Publish an already-marked reservation atomically. There is never a visible
    // target without its durable marker, including the mkdir/lock crash window.
    reserve_unpublished(&target)?;
    let target_lock = AppInstanceLock::acquire(&target)?;
    for entry in &entries {
        let destination = target.join(entry.path());
        match entry {
            Entry::Directory { .. } => {
                if destination == target.join("manager-state") {
                    continue;
                }
                fs::create_dir(&destination).map_err(io)?;
                super::set_private_directory(&destination)?;
            }
            Entry::File { .. } => copy_file(&source.join(entry.path()), &destination)?,
            Entry::Link { .. } => {} // All referents are copied before making links.
        }
    }
    for entry in &entries {
        if let Entry::Link {
            path,
            target: relative_target,
            directory,
        } = entry
        {
            create_link(
                &target.join(relative_target),
                &target.join(path),
                *directory,
            )?;
        }
    }
    after_copy(&source, &target)?;
    // Re-scan both sides: changed bytes, extra/missing files and link changes are
    // failures even if the individual copy syscall itself succeeded.
    if inventory(&source, external_home)? != entries
        || inventory(&target, external_home)? != entries
    {
        return Err(failure(
            "source changed or copied inventory failed verification; old binding retained",
        ));
    }
    // Give the unpublished copy a self binding. Never leave a chain back to the
    // old directory. The original anchor is deliberately untouched here.
    let mut copied_config = config.clone();
    copied_config.active_data_root = target.clone();
    super::save_runtime_config(&target, &copied_config)?;
    if super::load_runtime_config(&target)? != copied_config {
        return Err(failure("copied binding failed readback"));
    }
    let prepared_entries = inventory(&target, external_home)?;
    let mut excluded_paths = vec![
        "manager-state/app.lock",
        ".backup-w2.lock",
        "cache/ipc/opencodex.ipc",
        RECEIPT_PATH,
        activation::VERIFICATION_PATH,
        UNPUBLISHED_MARKER,
    ];
    if external_home {
        excluded_paths.push("opencodex-home (dormant internal HOME; external HOME unchanged)");
    }
    let receipt = CopyReceipt {
        schema_version: 1,
        phase: "verified_copy",
        source,
        target: target.clone(),
        excluded: excluded_paths,
        bytes,
        entries,
        prepared_entries,
    };
    let raw = serde_json::to_vec_pretty(&receipt).map_err(|e| failure(e.to_string()))?;
    crate::infrastructure::atomic_write::atomic_write(&target.join(RECEIPT_PATH), &raw, 0o600)?;
    if fs::read(target.join(RECEIPT_PATH)).map_err(io)? != raw {
        return Err(failure("copy receipt failed readback"));
    }
    Ok(VerifiedCopy {
        receipt,
        target_lock,
    })
}

fn resolve_future_path(path: &Path) -> AppResult<PathBuf> {
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(failure("invalid external HOME path"));
    }
    let mut existing = path;
    while !existing.try_exists().map_err(io)? {
        existing = existing
            .parent()
            .ok_or_else(|| failure("external HOME has no existing ancestor"))?;
    }
    Ok(existing.canonicalize().map_err(io)?.join(
        path.strip_prefix(existing)
            .map_err(|_| failure("invalid HOME ancestor"))?,
    ))
}

fn copy_file(source: &Path, destination: &Path) -> AppResult<()> {
    if !fs::symlink_metadata(source).map_err(io)?.is_file() {
        return Err(failure("source type changed"));
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let input = options.open(source).map_err(io)?;
    let metadata = input.metadata().map_err(io)?;
    if !metadata.is_file() {
        return Err(failure("source type changed during open"));
    }
    use std::io::Read;
    let mut bounded = input.take(
        metadata
            .len()
            .checked_add(1)
            .ok_or_else(|| failure("file size overflow"))?,
    );
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(io)?;
    if std::io::copy(&mut bounded, &mut output).map_err(io)? != metadata.len() {
        return Err(failure("source length changed during copy"));
    }
    fs::set_permissions(destination, metadata.permissions()).map_err(io)?;
    output.sync_all().map_err(io)
}

#[cfg(unix)]
fn create_link(target: &Path, link: &Path, _directory: bool) -> AppResult<()> {
    std::os::unix::fs::symlink(target, link).map_err(io)
}

#[cfg(windows)]
fn create_link(target: &Path, link: &Path, directory: bool) -> AppResult<()> {
    if directory {
        std::os::windows::fs::symlink_dir(target, link).map_err(io)
    } else {
        std::os::windows::fs::symlink_file(target, link).map_err(io)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> (tempfile::TempDir, DataRootRuntimeConfig, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let source = temp.path().join("source");
        super::super::initialize(&source).unwrap();
        fs::write(
            source.join("manager-state/preferences.json"),
            b"{ \"schema_version\": 1 }",
        )
        .unwrap();
        fs::create_dir_all(source.join("runtime/bin")).unwrap();
        fs::write(source.join("runtime/bin/fixture"), b"runtime payload").unwrap();
        let config = super::super::load_runtime_config(&source).unwrap();
        let target = temp.path().join("target 中文 space");
        (temp, config, target)
    }

    #[test]
    fn verified_copy_keeps_old_binding_bytes_and_locks_target() {
        let (_temp, config, target) = fixture();
        let source_binding = fs::read(
            config
                .active_data_root
                .join(super::super::METADATA_RELATIVE_PATH),
        )
        .unwrap();
        let copied = copy_verified(&config, &target).unwrap();
        assert_eq!(copied.receipt.phase, "verified_copy");
        assert!(copied.receipt.bytes > 0);
        assert_eq!(
            fs::read(
                config
                    .active_data_root
                    .join(super::super::METADATA_RELATIVE_PATH)
            )
            .unwrap(),
            source_binding
        );
        assert_eq!(
            fs::read(target.join("runtime/bin/fixture")).unwrap(),
            b"runtime payload"
        );
        assert_eq!(
            super::super::load_runtime_config(&target)
                .unwrap()
                .active_data_root,
            target.canonicalize().unwrap()
        );
        assert!(matches!(
            AppInstanceLock::acquire(&target),
            Err(AppError::InstanceLockConflict)
        ));
        drop(copied);
        assert!(AppInstanceLock::acquire(&target).is_ok());
        assert!(require_published(&target).is_err());
        assert!(super::super::bootstrap::resolve(&target).is_err());
    }

    #[test]
    fn existing_empty_target_is_never_merged() {
        let (_temp, config, target) = fixture();
        fs::create_dir(&target).unwrap();
        assert!(copy_verified(&config, &target).is_err());
        assert_eq!(fs::read_dir(&target).unwrap().count(), 0);
    }

    #[test]
    fn external_home_and_dormant_internal_home_are_not_copied() {
        let (temp, mut config, target) = fixture();
        let home = temp.path().join("external-home");
        fs::create_dir(&home).unwrap();
        fs::write(home.join("keep"), b"external").unwrap();
        fs::write(
            config.active_data_root.join("opencodex-home/dormant"),
            b"stale",
        )
        .unwrap();
        config.opencodex_home_mode = OpenCodexHomeMode::External;
        config.opencodex_home_path = Some(home.clone());
        let copied = copy_verified(&config, &target).unwrap();
        assert!(!target.join("opencodex-home").exists());
        assert_eq!(
            super::super::load_runtime_config(&target)
                .unwrap()
                .opencodex_home_path,
            Some(home.clone())
        );
        assert_eq!(fs::read(home.join("keep")).unwrap(), b"external");
        assert!(copied
            .receipt
            .entries
            .iter()
            .all(|entry| !entry.path().starts_with("opencodex-home")));
        assert!(copied
            .receipt
            .excluded
            .iter()
            .any(|path| path.starts_with("opencodex-home")));
    }

    #[test]
    fn transient_locks_are_excluded_but_structure_metadata_is_copied() {
        let (_temp, config, target) = fixture();
        fs::write(config.active_data_root.join(".backup-w2.lock"), b"lock").unwrap();
        fs::write(
            config.active_data_root.join("manager-state/app.lock"),
            b"old process",
        )
        .unwrap();
        let copied = copy_verified(&config, &target).unwrap();
        assert!(!target.join(".backup-w2.lock").exists());
        assert!(target.join("manager-state/structure.lock").exists());
        assert!(copied
            .receipt
            .entries
            .iter()
            .all(|entry| entry.path() != Path::new("manager-state/app.lock")));
    }

    #[test]
    fn nested_unknown_and_relative_targets_fail_before_reservation() {
        let (_temp, config, target) = fixture();
        assert!(copy_verified(&config, &config.active_data_root.join("nested")).is_err());
        assert!(copy_verified(&config, Path::new("relative")).is_err());
        fs::write(config.active_data_root.join("unregistered"), b"keep").unwrap();
        assert!(copy_verified(&config, &target).is_err());
        assert!(!target.exists());
    }

    #[test]
    fn damaged_copy_and_changed_source_never_update_source_binding() {
        for source_changes in [false, true] {
            let (_temp, config, target) = fixture();
            let before = fs::read(
                config
                    .active_data_root
                    .join(super::super::METADATA_RELATIVE_PATH),
            )
            .unwrap();
            let result = copy_with(&config, &target, |source, target| {
                let root = if source_changes { source } else { target };
                fs::write(root.join("runtime/bin/fixture"), b"changed").map_err(io)
            });
            assert!(result.is_err());
            assert_eq!(
                fs::read(
                    config
                        .active_data_root
                        .join(super::super::METADATA_RELATIVE_PATH)
                )
                .unwrap(),
                before
            );
            assert!(!target.join(RECEIPT_PATH).exists());
        }
    }

    #[test]
    fn interrupted_copy_keeps_old_root_and_no_verified_receipt() {
        let (_temp, config, target) = fixture();
        let result = copy_with(&config, &target, |_, _| Err(failure("injected disk full")));
        assert!(result.is_err());
        assert_eq!(
            super::super::load_runtime_config(&config.active_data_root).unwrap(),
            config
        );
        assert!(!target.join(RECEIPT_PATH).exists());
        assert!(config.active_data_root.join("runtime/bin/fixture").exists());
    }

    #[test]
    fn failed_copy_cannot_start_be_repaired_or_referenced_after_lock_release() {
        let (_temp, config, target) = fixture();
        assert!(copy_with(&config, &target, |_, _| Err(failure(
            "injected copy interruption"
        )))
        .is_err());
        fs::remove_dir(target.join("cache")).unwrap();
        let metadata = fs::read(target.join(super::super::METADATA_RELATIVE_PATH)).unwrap();
        let binding = fs::read(
            config
                .active_data_root
                .join(super::super::METADATA_RELATIVE_PATH),
        )
        .unwrap();
        let lock = fs::read(target.join("manager-state/app.lock")).unwrap();
        assert!(super::super::initialize(&target).is_err());
        assert!(super::super::ensure_partitions(&target).is_err());
        assert!(super::super::validate_structure(&target).is_err());
        assert!(super::super::bootstrap::resolve(&target).is_err());
        assert!(super::super::switch_reference(
            &config.active_data_root,
            &target,
            super::super::DataRootSwitchMode::ReferenceOnly
        )
        .is_err());
        assert!(!target.join("cache").exists());
        assert_eq!(
            fs::read(target.join(super::super::METADATA_RELATIVE_PATH)).unwrap(),
            metadata
        );
        assert_eq!(
            fs::read(target.join("manager-state/app.lock")).unwrap(),
            lock
        );
        assert_eq!(
            fs::read(
                config
                    .active_data_root
                    .join(super::super::METADATA_RELATIVE_PATH)
            )
            .unwrap(),
            binding
        );
    }

    #[test]
    fn reservation_blocks_startup_even_before_any_partition_exists() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("target");
        reserve_unpublished(&target).unwrap();
        assert!(super::super::bootstrap::resolve(&target).is_err());
        assert!(super::super::initialize(&target).is_err());
        assert_eq!(fs::read_dir(&target).unwrap().count(), 1);
    }

    #[test]
    fn receipt_distinguishes_verified_source_from_prepared_self_binding() {
        let (_temp, config, target) = fixture();
        let copy = copy_verified(&config, &target).unwrap();
        assert_eq!(
            inventory(&target, false).unwrap(),
            copy.receipt.prepared_entries
        );
        assert_ne!(copy.receipt.entries, copy.receipt.prepared_entries);
        let without_binding = |entries: &[Entry]| {
            entries
                .iter()
                .filter(|e| e.path() != Path::new(super::super::METADATA_RELATIVE_PATH))
                .cloned()
                .collect::<Vec<_>>()
        };
        assert_eq!(
            without_binding(&copy.receipt.entries),
            without_binding(&copy.receipt.prepared_entries)
        );
    }

    #[cfg(unix)]
    #[test]
    fn broken_marker_blocks_repair_and_existing_dangling_target_is_preserved() {
        use std::os::unix::fs::symlink;
        let (temp, config, target) = fixture();
        symlink(temp.path().join("missing"), &target).unwrap();
        assert!(copy_verified(&config, &target).is_err());
        assert!(fs::symlink_metadata(&target)
            .unwrap()
            .file_type()
            .is_symlink());
        let root = temp.path().join("partial");
        fs::create_dir(&root).unwrap();
        symlink(
            temp.path().join("missing-marker"),
            root.join(UNPUBLISHED_MARKER),
        )
        .unwrap();
        assert!(super::super::initialize(&root).is_err());
        assert!(!root.join("manager-state").exists());
    }

    #[cfg(unix)]
    #[test]
    fn internal_links_are_relocated_and_executability_preserved() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let (_temp, config, target) = fixture();
        let payload = config.active_data_root.join("runtime/bin/fixture");
        fs::set_permissions(&payload, fs::Permissions::from_mode(0o755)).unwrap();
        symlink(
            "fixture",
            config.active_data_root.join("runtime/bin/relative"),
        )
        .unwrap();
        symlink(
            &payload,
            config.active_data_root.join("runtime/bin/absolute"),
        )
        .unwrap();
        let _copy = copy_verified(&config, &target).unwrap();
        for name in ["relative", "absolute"] {
            assert_eq!(
                target
                    .join("runtime/bin")
                    .join(name)
                    .canonicalize()
                    .unwrap(),
                target.join("runtime/bin/fixture").canonicalize().unwrap()
            );
        }
        assert_eq!(
            fs::metadata(target.join("runtime/bin/fixture"))
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o755
        );
    }

    #[cfg(unix)]
    #[test]
    fn escaping_and_broken_links_fail_without_reading_external_payload() {
        use std::os::unix::fs::symlink;
        for existing in [false, true] {
            let (temp, config, target) = fixture();
            let outside = temp.path().join("outside");
            if existing {
                fs::write(&outside, b"private").unwrap();
            }
            symlink(&outside, config.active_data_root.join("manager-state/link")).unwrap();
            assert!(copy_verified(&config, &target).is_err());
            assert!(!target.exists());
        }
    }
}
