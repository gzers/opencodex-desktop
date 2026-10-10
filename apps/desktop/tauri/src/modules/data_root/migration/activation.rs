//! One binding commit point, with a still-marked target until locked startup
//! verification. This module never switches live services or deletes the source.
use super::*;
use crate::infrastructure::atomic_write::atomic_write;
use crate::infrastructure::hash::sha256_hex;
use serde::{Deserialize, Serialize};

pub const VERIFICATION_PATH: &str = "manager-state/data-root-migration-startup.json";
const MAX_MARKER_BYTES: u64 = 16 * 1024;
const MAX_RECEIPT_BYTES: u64 = 512 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadyMarker {
    schema_version: u32,
    phase: String,
    anchor: PathBuf,
    source: PathBuf,
    target: PathBuf,
    binding_sha256: String,
    receipt_sha256: String,
    inventory_sha256: String,
    external_home: bool,
}

/// Owning this value also owns the target instance lock. The caller must already
/// own every startup binding lock and freeze all writers. Dropping it before
/// commit leaves the old binding unchanged and the target unstartable.
pub struct PendingActivation {
    copy: VerifiedCopy,
    marker: ReadyMarker,
    old_binding: Vec<u8>,
    new_binding: Vec<u8>,
    pub config: DataRootRuntimeConfig,
}

/// A successful atomic replace is irreversible within this operation. Even if
/// subsequent durability/readback fails, return the retained lock and require
/// restart reconciliation; never resume old live writers as an ordinary failure.
pub struct ActivationCommit {
    pub target_lock: AppInstanceLock,
    pub config: DataRootRuntimeConfig,
    pub reconciliation_required: bool,
}

pub(in crate::modules::data_root) struct StartupPermit(Option<ReadyMarker>);

impl VerifiedCopy {
    pub fn prepare_activation(self, anchor: &Path) -> AppResult<PendingActivation> {
        if self.receipt.phase != "runtime_prepared" {
            return Err(failure("activation requires prepared runtime paths"));
        }
        let anchor = anchor.canonicalize().map_err(io)?;
        if !anchor.is_absolute() || anchor == self.receipt.target {
            return Err(failure(
                "activation requires a distinct absolute startup anchor",
            ));
        }
        let config = super::super::load_runtime_config(&self.receipt.target)?;
        if config.active_data_root != self.receipt.target {
            return Err(failure("prepared target is not self bound"));
        }
        let old_binding = read_regular(&super::super::runtime_config_path(&anchor), 65536)?;
        check_anchor_source(&anchor, &self.receipt.source, &config)?;
        let new_binding = super::super::merged_metadata_payload(&config)?;
        let external_home = config.opencodex_home_mode == OpenCodexHomeMode::External;
        let raw_receipt =
            serde_json::to_vec_pretty(&self.receipt).map_err(|e| failure(e.to_string()))?;
        if raw_receipt.len() as u64 > MAX_RECEIPT_BYTES
            || regular_digest(&self.receipt.target.join(RECEIPT_PATH), MAX_RECEIPT_BYTES)?
                != sha256_hex(&raw_receipt)
            || inventory(&self.receipt.source, external_home)? != self.receipt.entries
            || inventory(&self.receipt.target, external_home)? != self.receipt.prepared_entries
        {
            return Err(failure(
                "copy receipt or inventory changed before activation",
            ));
        }
        // File fsync alone does not persist newly populated directory entries
        // or runtime launcher replacements. Flush children before their parents
        // before publishing a ready marker. Non-Unix durability still requires
        // platform verification; sync_directory makes no such claim there.
        for entry in self.receipt.prepared_entries.iter().rev() {
            if let Entry::Directory { path } = entry {
                sync_directory(&self.receipt.target.join(path))?;
            }
        }
        sync_directory(&self.receipt.target)?;
        sync_directory(self.receipt.target.parent().expect("target parent"))?;
        let marker = ReadyMarker {
            schema_version: 1,
            phase: "awaiting_binding_commit".into(),
            anchor,
            source: self.receipt.source.clone(),
            target: self.receipt.target.clone(),
            binding_sha256: sha256_hex(&new_binding),
            receipt_sha256: sha256_hex(&raw_receipt),
            inventory_sha256: inventory_digest(&self.receipt.prepared_entries)?,
            external_home,
        };
        let marker_path = marker.target.join(UNPUBLISHED_MARKER);
        // Refuse an externally removed or substituted marker, including links.
        read_regular(&marker_path, MAX_MARKER_BYTES)?;
        write_verified(
            &marker_path,
            &serde_json::to_vec(&marker).map_err(|e| failure(e.to_string()))?,
        )?;
        Ok(PendingActivation {
            copy: self,
            marker,
            old_binding,
            new_binding,
            config,
        })
    }
}

impl PendingActivation {
    pub fn commit(self) -> AppResult<ActivationCommit> {
        self.commit_with(atomic_write, sync_directory)
    }

    fn commit_with(
        self,
        replace: impl FnOnce(&Path, &[u8], u32) -> AppResult<()>,
        synchronize: impl FnOnce(&Path) -> AppResult<()>,
    ) -> AppResult<ActivationCommit> {
        let anchor_path = super::super::runtime_config_path(&self.marker.anchor);
        if read_regular(&anchor_path, 65536)? != self.old_binding
            || read_marker(&self.marker.target)? != self.marker
            || inventory(&self.copy.receipt.source, self.marker.external_home)?
                != self.copy.receipt.entries
            || inventory(&self.marker.target, self.marker.external_home)?
                != self.copy.receipt.prepared_entries
            || regular_digest(&self.marker.target.join(RECEIPT_PATH), MAX_RECEIPT_BYTES)?
                != self.marker.receipt_sha256
        {
            return Err(failure(
                "binding or migration inventory changed before commit",
            ));
        }
        // Atomic replacement is the sole commit point. An uncertain outcome
        // retains the lock and freeze, too; the caller must surface reconciliation.
        let result = replace(&anchor_path, &self.new_binding, 0o600);
        let readback = read_regular(&anchor_path, 65536);
        if let Err(error) = result {
            if readback.as_ref().is_ok_and(|raw| raw == &self.old_binding) {
                return Err(error);
            }
            return Ok(ActivationCommit {
                target_lock: self.copy.target_lock,
                config: self.config,
                reconciliation_required: true,
            });
        }
        let confirmed = readback.as_ref().is_ok_and(|raw| raw == &self.new_binding)
            && synchronize(anchor_path.parent().expect("binding parent")).is_ok();
        Ok(ActivationCommit {
            target_lock: self.copy.target_lock,
            config: self.config,
            reconciliation_required: !confirmed,
        })
    }
}

/// Pre-lock admission checks only small metadata and the committed anchor. It
/// does not remove markers, repair partitions or hash the whole copied tree.
pub(in crate::modules::data_root) fn admit_startup(root: &Path) -> AppResult<StartupPermit> {
    match fs::symlink_metadata(root.join(UNPUBLISHED_MARKER)) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(StartupPermit(None)),
        Err(e) => Err(io(e)),
        Ok(_) => {
            let marker = read_marker(root)?;
            check_commit(&marker, root)?;
            Ok(StartupPermit(Some(marker)))
        }
    }
}

/// Must run immediately after acquiring this root's instance lock and before
/// initialization or any mutable service. The full inventory is verified once;
/// subsequent normal startups do not repeatedly hash user data.
pub(in crate::modules::data_root) fn complete_startup(
    root: &Path,
    lock: &AppInstanceLock,
    permit: StartupPermit,
) -> AppResult<()> {
    let Some(marker) = permit.0 else {
        return require_published(root);
    };
    let root = root.canonicalize().map_err(io)?;
    if lock.lock_path().canonicalize().map_err(io)?
        != root
            .join("manager-state/app.lock")
            .canonicalize()
            .map_err(io)?
        || read_marker(&root)? != marker
    {
        return Err(failure(
            "startup verification lacks the target lock or marker changed",
        ));
    }
    check_commit(&marker, &root)?;
    if regular_digest(&root.join(RECEIPT_PATH), MAX_RECEIPT_BYTES)? != marker.receipt_sha256
        || inventory_digest(&inventory(&root, marker.external_home)?)? != marker.inventory_sha256
    {
        return Err(failure(
            "committed migration failed locked startup inventory verification",
        ));
    }
    // This is inventory verification, not evidence that the whole application
    // started, the UI passed, or a release was accepted.
    let mut verification = marker.clone();
    verification.phase = "startup_inventory_verified".into();
    write_verified(
        &root.join(VERIFICATION_PATH),
        &serde_json::to_vec(&verification).map_err(|e| failure(e.to_string()))?,
    )?;
    // Recheck the admission binding immediately before making the target normal.
    check_commit(&marker, &root)?;
    fs::remove_file(root.join(UNPUBLISHED_MARKER)).map_err(io)?;
    sync_directory(&root)
}

fn read_marker(root: &Path) -> AppResult<ReadyMarker> {
    let raw = read_regular(&root.join(UNPUBLISHED_MARKER), MAX_MARKER_BYTES)?;
    let marker: ReadyMarker = serde_json::from_slice(&raw)
        .map_err(|_| failure("migration marker is incomplete or invalid"))?;
    if marker.schema_version != 1
        || marker.phase != "awaiting_binding_commit"
        || [&marker.anchor, &marker.source, &marker.target]
            .iter()
            .any(|path| {
                !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir))
            })
        || [
            &marker.binding_sha256,
            &marker.receipt_sha256,
            &marker.inventory_sha256,
        ]
        .iter()
        .any(|digest| digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit()))
    {
        return Err(failure("unsupported or malformed activation marker"));
    }
    Ok(marker)
}

fn check_commit(marker: &ReadyMarker, root: &Path) -> AppResult<()> {
    if marker.target != root.canonicalize().map_err(io)?
        || marker.anchor == marker.target
        || regular_digest(&super::super::runtime_config_path(&marker.anchor), 65536)?
            != marker.binding_sha256
    {
        return Err(failure(
            "migration target has no matching committed startup binding",
        ));
    }
    let config = super::super::load_runtime_config(&marker.anchor)?;
    let target = super::super::load_runtime_config(&marker.target)?;
    if config != target
        || config.active_data_root != marker.target
        || (config.opencodex_home_mode == OpenCodexHomeMode::External) != marker.external_home
    {
        return Err(failure("committed binding disagrees with prepared target"));
    }
    Ok(())
}

fn check_anchor_source(
    anchor: &Path,
    source: &Path,
    target: &DataRootRuntimeConfig,
) -> AppResult<()> {
    let mut current = anchor.to_path_buf();
    let mut seen = std::collections::HashSet::new();
    let mut external = None;
    for _ in 0..super::super::bootstrap::MAX_BINDING_DEPTH {
        let canonical = current.canonicalize().map_err(io)?;
        if !seen.insert(canonical.clone()) {
            break;
        }
        require_published(&canonical)?;
        let config = super::super::load_runtime_config(&canonical)?;
        if external.is_none() && config.opencodex_home_mode == OpenCodexHomeMode::External {
            external = config.opencodex_home_path.clone();
        }
        let next = config.active_data_root.canonicalize().map_err(io)?;
        if next == canonical {
            let expected_external = target
                .opencodex_home_path
                .as_ref()
                .filter(|_| target.opencodex_home_mode == OpenCodexHomeMode::External);
            if canonical != source || external.as_ref() != expected_external {
                return Err(failure(
                    "startup chain does not match the copied active root and HOME",
                ));
            }
            return Ok(());
        }
        current = next;
    }
    Err(failure("invalid startup binding chain before migration"))
}

fn read_regular(path: &Path, limit: u64) -> AppResult<Vec<u8>> {
    use std::io::Read;
    let file = open_regular(path, limit)?;
    let mut raw = Vec::new();
    file.take(limit + 1).read_to_end(&mut raw).map_err(io)?;
    if raw.len() as u64 > limit {
        return Err(failure("migration metadata exceeds size limit"));
    }
    Ok(raw)
}

fn open_regular(path: &Path, limit: u64) -> AppResult<File> {
    if !fs::symlink_metadata(path).map_err(io)?.is_file() {
        return Err(failure("migration metadata must be a regular file"));
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path).map_err(io)?;
    let metadata = file.metadata().map_err(io)?;
    if !metadata.is_file() || metadata.len() > limit {
        return Err(failure(
            "migration metadata has invalid type or exceeds size limit",
        ));
    }
    Ok(file)
}

fn regular_digest(path: &Path, limit: u64) -> AppResult<String> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut file = open_regular(path, limit)?.take(limit + 1);
    let mut hash = Sha256::new();
    let mut buf = [0u8; 65536];
    let mut bytes = 0u64;
    loop {
        let n = file.read(&mut buf).map_err(io)?;
        if n == 0 {
            break;
        }
        bytes += n as u64;
        if bytes > limit {
            return Err(failure("migration metadata grew beyond size limit"));
        }
        hash.update(&buf[..n]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn inventory_digest(entries: &[Entry]) -> AppResult<String> {
    // Same deterministic representation before commit and during locked startup.
    Ok(sha256_hex(
        &serde_json::to_vec(entries).map_err(|e| failure(e.to_string()))?,
    ))
}

fn write_verified(path: &Path, raw: &[u8]) -> AppResult<()> {
    atomic_write(path, raw, 0o600)?;
    if read_regular(path, MAX_MARKER_BYTES)? != raw {
        return Err(failure("activation metadata failed readback"));
    }
    sync_directory(path.parent().expect("activation metadata parent"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::data_root::{self, bootstrap, DataRootSwitchMode, OpenCodexHomeMode};

    struct Fixture {
        _temp: tempfile::TempDir,
        anchor: PathBuf,
        source: PathBuf,
        target: PathBuf,
        config: DataRootRuntimeConfig,
        _locks: Vec<AppInstanceLock>,
    }
    impl Fixture {
        fn new(chain: bool, external: bool) -> Self {
            let temp = tempfile::tempdir().unwrap();
            let anchor = temp.path().join("anchor");
            let source = if chain {
                temp.path().join("source")
            } else {
                anchor.clone()
            };
            data_root::initialize(&anchor).unwrap();
            if chain {
                data_root::initialize(&source).unwrap();
                data_root::switch_reference(&anchor, &source, DataRootSwitchMode::ReferenceOnly)
                    .unwrap();
            }
            if external {
                let home = temp.path().join("external 中文 HOME");
                fs::create_dir(&home).unwrap();
                fs::write(home.join("keep"), b"external original").unwrap();
                // The effective HOME can come from a chain member, not the anchor.
                data_root::set_opencodex_home(&source, OpenCodexHomeMode::External, Some(&home))
                    .unwrap();
            }
            fs::write(
                source.join("manager-state/preferences.json"),
                b"original preferences",
            )
            .unwrap();
            let (config, locks) = bootstrap::resolve_locked_with_boundary(&anchor, None).unwrap();
            let target = temp.path().join("target 中文 space");
            Self {
                _temp: temp,
                anchor,
                source,
                target,
                config,
                _locks: locks,
            }
        }
        fn prepare(&self) -> PendingActivation {
            let mut copy = copy_verified(&self.config, &self.target).unwrap();
            copy.prepare_runtime().unwrap();
            copy.prepare_activation(&self.anchor).unwrap()
        }
    }

    #[test]
    fn ready_copy_crash_retains_old_binding_and_denies_target_startup() {
        let f = Fixture::new(true, false);
        let before = fs::read(data_root::runtime_config_path(&f.anchor)).unwrap();
        let pending = f.prepare();
        assert!(admit_startup(&f.target).is_err());
        assert!(data_root::initialize(&f.target).is_err());
        drop(pending); // Crash after ready marker, before anchor commit.
        assert_eq!(
            fs::read(data_root::runtime_config_path(&f.anchor)).unwrap(),
            before
        );
        assert!(bootstrap::resolve(&f.target).is_err());
        drop(f._locks);
        let config = bootstrap::resolve(&f.anchor).unwrap();
        assert_eq!(config.active_data_root, f.source.canonicalize().unwrap());
        assert!(f.target.join(UNPUBLISHED_MARKER).exists());
    }

    #[test]
    fn commit_retains_target_lock_then_locked_restart_enables_target_once() {
        let f = Fixture::new(false, false);
        let committed = f.prepare().commit().unwrap();
        assert!(!committed.reconciliation_required);
        assert!(f.target.join(UNPUBLISHED_MARKER).is_file());
        assert!(!f.target.join(VERIFICATION_PATH).exists());
        assert!(admit_startup(&f.target).is_ok());
        assert!(data_root::validate_structure(&f.target).is_err());
        assert!(matches!(
            bootstrap::resolve(&f.target),
            Err(AppError::InstanceLockConflict)
        ));
        drop(committed);
        drop(f._locks);
        let (config, locks) = bootstrap::resolve_locked_with_boundary(&f.anchor, None).unwrap();
        assert_eq!(config.active_data_root, f.target.canonicalize().unwrap());
        assert!(!f.target.join(UNPUBLISHED_MARKER).exists());
        let verified: ReadyMarker =
            serde_json::from_slice(&fs::read(f.target.join(VERIFICATION_PATH)).unwrap()).unwrap();
        assert_eq!(verified.phase, "startup_inventory_verified");
        assert_eq!(
            fs::read(f.source.join("manager-state/preferences.json")).unwrap(),
            b"original preferences"
        );
        // Normal mutation after the first successful inventory verification does
        // not force re-hashing or reject the root at the next regular startup.
        fs::write(
            f.target.join("manager-state/preferences.json"),
            b"new preferences",
        )
        .unwrap();
        drop(locks);
        assert!(bootstrap::resolve(&f.anchor).is_ok());
    }

    #[test]
    fn crash_immediately_after_anchor_replace_is_recovered_on_restart() {
        let f = Fixture::new(true, true);
        let pending = f.prepare();
        // Emulate process loss at the commit point, without post-write checks.
        atomic_write(
            &data_root::runtime_config_path(&f.anchor),
            &pending.new_binding,
            0o600,
        )
        .unwrap();
        drop(pending);
        drop(f._locks);
        let config = bootstrap::resolve(&f.anchor).unwrap();
        assert_eq!(config.active_data_root, f.target.canonicalize().unwrap());
        assert_eq!(
            data_root::resolve_opencodex_home(&config)
                .canonicalize()
                .unwrap(),
            data_root::resolve_opencodex_home(&f.config)
                .canonicalize()
                .unwrap()
        );
        assert_eq!(
            fs::read(data_root::resolve_opencodex_home(&config).join("keep")).unwrap(),
            b"external original"
        );
        assert_eq!(
            data_root::load_runtime_config(&f.source)
                .unwrap()
                .active_data_root
                .canonicalize()
                .unwrap(),
            f.source.canonicalize().unwrap()
        );
        assert!(data_root::validate_structure(&f.target).is_ok());
    }

    #[test]
    fn uncertain_replace_or_failed_post_commit_sync_retains_lock_for_reconciliation() {
        for fail_replace in [false, true] {
            let f = Fixture::new(true, false);
            let committed = f
                .prepare()
                .commit_with(
                    |path, raw, mode| {
                        atomic_write(path, raw, mode)?;
                        if fail_replace {
                            Err(failure("injected uncertain replacement"))
                        } else {
                            Ok(())
                        }
                    },
                    |_| Err(failure("injected directory sync failure")),
                )
                .unwrap();
            assert!(committed.reconciliation_required);
            assert!(matches!(
                AppInstanceLock::acquire(&f.target),
                Err(AppError::InstanceLockConflict)
            ));
            assert!(f.target.join(UNPUBLISHED_MARKER).exists());
            drop(committed);
            drop(f._locks);
            assert_eq!(
                bootstrap::resolve(&f.anchor).unwrap().active_data_root,
                f.target.canonicalize().unwrap()
            );
        }
    }

    #[test]
    fn replace_failure_with_confirmed_old_binding_can_return_ordinary_error() {
        let f = Fixture::new(true, false);
        let before = fs::read(data_root::runtime_config_path(&f.anchor)).unwrap();
        assert!(f
            .prepare()
            .commit_with(
                |_, _, _| Err(failure("injected failure before replacement")),
                |_| panic!("must not sync an unchanged binding"),
            )
            .is_err());
        assert_eq!(
            fs::read(data_root::runtime_config_path(&f.anchor)).unwrap(),
            before
        );
        assert!(admit_startup(&f.target).is_err());
        drop(f._locks);
        assert_eq!(
            bootstrap::resolve(&f.anchor).unwrap().active_data_root,
            f.source.canonicalize().unwrap()
        );
    }

    #[test]
    fn source_change_or_anchor_change_before_commit_cannot_replace_binding() {
        for change_anchor in [false, true] {
            let f = Fixture::new(true, false);
            let pending = f.prepare();
            if change_anchor {
                fs::write(data_root::runtime_config_path(&f.anchor), b"outside writer").unwrap();
            } else {
                fs::write(
                    f.source.join("manager-state/preferences.json"),
                    b"outside writer",
                )
                .unwrap();
            }
            let before = fs::read(data_root::runtime_config_path(&f.anchor)).unwrap();
            assert!(pending.commit().is_err());
            assert_eq!(
                fs::read(data_root::runtime_config_path(&f.anchor)).unwrap(),
                before
            );
            assert!(bootstrap::resolve(&f.target).is_err());
        }
    }

    #[test]
    fn receipt_or_target_tampering_blocks_restart_without_partition_repair() {
        for tamper in ["payload", "receipt", "partition", "extra"] {
            let f = Fixture::new(true, false);
            let committed = f.prepare().commit().unwrap();
            drop(committed);
            drop(f._locks);
            match tamper {
                "payload" => {
                    fs::write(f.target.join("manager-state/preferences.json"), b"damaged").unwrap()
                }
                "receipt" => fs::write(f.target.join(RECEIPT_PATH), b"damaged").unwrap(),
                "partition" => fs::remove_dir(f.target.join("cache")).unwrap(),
                "extra" => fs::write(f.target.join("logs/extra"), b"unexpected").unwrap(),
                _ => unreachable!(),
            }
            assert!(bootstrap::resolve(&f.anchor).is_err(), "{tamper}");
            assert!(f.target.join(UNPUBLISHED_MARKER).exists());
            assert!(!f.target.join(VERIFICATION_PATH).exists());
            if tamper == "partition" {
                assert!(!f.target.join("cache").exists());
            }
            assert_eq!(
                fs::read(f.source.join("manager-state/preferences.json")).unwrap(),
                b"original preferences"
            );
        }
    }

    #[test]
    fn restart_record_write_failure_keeps_marker_for_retry() {
        let f = Fixture::new(true, false);
        drop(f.prepare().commit().unwrap());
        drop(f._locks);
        fs::create_dir(f.target.join(VERIFICATION_PATH)).unwrap();
        assert!(bootstrap::resolve(&f.anchor).is_err());
        assert!(f.target.join(UNPUBLISHED_MARKER).exists());
        fs::remove_dir(f.target.join(VERIFICATION_PATH)).unwrap();
        assert!(bootstrap::resolve(&f.anchor).is_ok());
        assert!(!f.target.join(UNPUBLISHED_MARKER).exists());
    }

    #[test]
    fn malformed_oversized_or_uncommitted_marker_is_never_a_startup_license() {
        for tamper in ["schema", "phase", "root", "hash", "oversize", "empty"] {
            let f = Fixture::new(true, false);
            let pending = f.prepare();
            let mut marker = pending.marker.clone();
            drop(pending);
            match tamper {
                "schema" => marker.schema_version = 9,
                "phase" => marker.phase = "done".into(),
                "root" => marker.target = f.source.clone(),
                "hash" => marker.binding_sha256 = "invalid".into(),
                _ => (),
            }
            let raw = match tamper {
                "oversize" => vec![b'x'; MAX_MARKER_BYTES as usize + 1],
                "empty" => vec![],
                _ => serde_json::to_vec(&marker).unwrap(),
            };
            fs::write(f.target.join(UNPUBLISHED_MARKER), raw).unwrap();
            assert!(bootstrap::resolve(&f.target).is_err(), "{tamper}");
        }
    }

    #[test]
    fn wrong_effective_home_and_unprepared_runtime_cannot_be_activated() {
        let f = Fixture::new(true, true);
        let copied = copy_verified(&f.config, &f.target).unwrap();
        assert!(copied.prepare_activation(&f.anchor).is_err());
        assert!(admit_startup(&f.target).is_err());
        let second = f.target.with_file_name("second");
        let mut wrong = f.config.clone();
        wrong.opencodex_home_mode = OpenCodexHomeMode::Inside;
        wrong.opencodex_home_path = None;
        let mut copy = copy_verified(&wrong, &second).unwrap();
        copy.prepare_runtime().unwrap();
        assert!(copy.prepare_activation(&f.anchor).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn marker_and_binding_symlinks_are_rejected() {
        use std::os::unix::fs::symlink;
        for binding in [false, true] {
            let f = Fixture::new(true, false);
            drop(f.prepare().commit().unwrap());
            drop(f._locks);
            let path = if binding {
                data_root::runtime_config_path(&f.anchor)
            } else {
                f.target.join(UNPUBLISHED_MARKER)
            };
            let saved = path.with_extension("saved");
            fs::rename(&path, &saved).unwrap();
            symlink(&saved, &path).unwrap();
            assert!(bootstrap::resolve(&f.target).is_err());
        }
    }

    #[test]
    fn ready_binding_changed_after_commit_refuses_target_without_lock_repair() {
        let f = Fixture::new(true, false);
        drop(f.prepare().commit().unwrap());
        drop(f._locks);
        data_root::switch_reference(&f.anchor, &f.source, DataRootSwitchMode::ReferenceOnly)
            .unwrap();
        fs::remove_file(f.target.join("manager-state/app.lock")).unwrap();
        assert!(bootstrap::resolve(&f.target).is_err());
        assert!(!f.target.join("manager-state/app.lock").exists());
        assert_eq!(
            bootstrap::resolve(&f.anchor).unwrap().active_data_root,
            f.source.canonicalize().unwrap()
        );
    }
}
