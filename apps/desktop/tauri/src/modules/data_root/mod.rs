//! FZ-01 ~ FZ-03 数据根初始化与结构校验。
//!
//! 模块只作用于显式传入的数据根目录；测试使用临时路径。
//! 不扫描用户目录、不迁移数据、不启动代理、不访问 Keychain。

pub mod bootstrap;
pub mod layout;
pub mod migration;
pub(crate) mod removal_gate;
pub(crate) mod startup_v2;
#[cfg(any(windows, test))]
pub mod uninstall;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::errors::AppError;

pub const STRUCTURE_VERSION: &str = "2";
pub const METADATA_RELATIVE_PATH: &str = "manager-state/data-root.json";
pub const STRUCTURE_LOCK_RELATIVE_PATH: &str = "manager-state/structure.lock";

pub const PARTITIONS: [(&str, &str); 6] = [
    ("manager state", "manager-state"),
    ("backups", "backups"),
    ("logs", "logs"),
    ("exports", "exports"),
    ("cache", "cache"),
    ("sync state", "sync-state"),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DataRootMetadata {
    pub structure_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DataRootRuntimeConfig {
    pub active_data_root: PathBuf,
    pub opencodex_home_mode: OpenCodexHomeMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opencodex_home_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenCodexHomeMode {
    Inside,
    External,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataRootSwitchMode {
    ReferenceOnly,
    MigrateData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DataRootSwitchBlocked {
    Unsupported,
    Running,
    RuntimeUnverified,
    Nested,
    FutureVersion,
    Corrupted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StructureValidation {
    Valid,
    Uninitialized,
    FutureVersion,
    Corrupted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataRootPaths {
    pub root: PathBuf,
    pub metadata: PathBuf,
    pub structure_lock: PathBuf,
}

impl DataRootPaths {
    pub fn new(root: PathBuf) -> Self {
        Self {
            metadata: root.join(METADATA_RELATIVE_PATH),
            structure_lock: root.join(STRUCTURE_LOCK_RELATIVE_PATH),
            root,
        }
    }
}

/// 初始化或引用一个显式数据根；返回 true 表示本次实际创建。
pub fn initialize(root: &Path) -> Result<bool, AppError> {
    startup_v2::checked_path(root)?;
    startup_v2::check_owner(root)?;
    check_partitions(root)?;
    migration::require_published(root)?;
    crate::infrastructure::platform::create_private_dir_all(root, 0o700).map_err(|error| {
        AppError::FileSystem {
            operation: "create data root".to_string(),
            detail: error.to_string(),
        }
    })?;
    let paths = DataRootPaths::new(root.to_path_buf());

    // No automatic repair or import of legacy metadata in v2.

    if metadata_matches_version(&paths.metadata)? {
        if !paths
            .metadata
            .parent()
            .is_some_and(|parent| parent.is_dir())
        {
            return Err(AppError::FileSystem {
                operation: "validate manager state".to_string(),
                detail: "manager-state directory is missing".to_string(),
            });
        }
        set_private_directory(root)?;
        ensure_partitions(root)?;
        return Ok(false);
    }

    if std::fs::read_dir(root)
        .map_err(|e| AppError::FileSystem {
            operation: "inspect v2 root".into(),
            detail: e.to_string(),
        })?
        .any(|entry| match entry {
            Ok(entry) if entry.file_name() == "manager-state" => std::fs::read_dir(entry.path())
                .map(|mut files| {
                    files.any(|f| f.map(|f| f.file_name() != "app.lock").unwrap_or(true))
                })
                .unwrap_or(true),
            Ok(entry) => !["manager-startup.lock", "manager-location.json"]
                .iter()
                .any(|name| entry.file_name() == *name),
            Err(_) => true,
        })
    {
        return Err(AppError::FileSystem {
            operation: "initialize v2 root".into(),
            detail: "nonempty unrecognized directory; original contents retained".into(),
        });
    }

    set_private_directory(root)?;
    let existing: Vec<_> = PARTITIONS
        .iter()
        .filter_map(|(_, name)| {
            let path = root.join(name);
            path.exists().then_some(path)
        })
        .collect();
    create_partitions(root)?;
    if let Err(error) = write_metadata_and_lock(&paths) {
        // Only remove empty partitions created by this initialization.
        for (_, name) in PARTITIONS {
            let path = root.join(name);
            if !existing.contains(&path) {
                let _ = std::fs::remove_dir(path);
            }
        }
        return Err(error);
    }

    Ok(true)
}

/// 幂等补建既有数据根的缺失分区；不改动已有内容。
pub fn ensure_partitions(root: &Path) -> Result<(), AppError> {
    migration::require_published(root)?;
    check_partitions(root)?;
    for (_, name) in PARTITIONS {
        let path = root.join(name);
        if path.is_dir() {
            set_private_directory(&path)?;
        } else if path.exists() {
            return Err(AppError::FileSystem {
                operation: "ensure data root partition".to_string(),
                detail: "partition target is not a directory".to_string(),
            });
        } else {
            crate::infrastructure::platform::create_private_dir_all(&path, 0o700).map_err(
                |error| AppError::FileSystem {
                    operation: "ensure data root partition".to_string(),
                    detail: error.to_string(),
                },
            )?;
            set_private_directory(&path)?;
        }
    }
    Ok(())
}

pub(crate) fn check_partitions(root: &Path) -> Result<(), AppError> {
    for path in PARTITIONS.iter().map(|(_, name)| root.join(name)).chain([
        root.join(METADATA_RELATIVE_PATH),
        root.join(STRUCTURE_LOCK_RELATIVE_PATH),
    ]) {
        match std::fs::symlink_metadata(&path) {
            Ok(meta) => {
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if meta.file_attributes() & 0x400 != 0 {
                        return Err(AppError::NotConfigured);
                    }
                }
                if meta.file_type().is_symlink() {
                    return Err(AppError::NotConfigured);
                }
                startup_v2::check_owner(&path)?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                return Err(AppError::FileSystem {
                    operation: "inspect v2 partitions".into(),
                    detail: e.to_string(),
                })
            }
        }
    }
    Ok(())
}

pub fn validate_structure(root: &Path) -> Result<StructureValidation, AppError> {
    migration::require_published(root)?;
    if !root.is_dir() {
        return Err(AppError::FileSystem {
            operation: "validate data root".to_string(),
            detail: format!("{} is not a directory", root.display()),
        });
    }
    check_partitions(root)?;
    let metadata = root.join(METADATA_RELATIVE_PATH);
    if !metadata.is_file() {
        let mut entries = std::fs::read_dir(root).map_err(|error| AppError::FileSystem {
            operation: "inspect data root".to_string(),
            detail: error.to_string(),
        })?;
        return match entries.next() {
            None => Ok(StructureValidation::Uninitialized),
            Some(Ok(_)) => Ok(StructureValidation::Corrupted),
            Some(Err(error)) => Err(AppError::FileSystem {
                operation: "inspect data root entry".to_string(),
                detail: error.to_string(),
            }),
        };
    }
    if root.join("manager-state").is_dir() {
        read_and_validate_metadata(&metadata)
    } else {
        Ok(StructureValidation::Corrupted)
    }
}

pub fn runtime_config_path(data_root: &Path) -> PathBuf {
    data_root.join(METADATA_RELATIVE_PATH)
}

/// Metadata belongs only to this D. The fixed location file is read separately.
pub fn load_runtime_config(data_root: &Path) -> Result<DataRootRuntimeConfig, AppError> {
    startup_v2::checked_path(data_root)?;
    check_partitions(data_root)?;
    let document = read_metadata_document(&runtime_config_path(data_root))?;
    if document.structure_version != STRUCTURE_VERSION {
        return Err(AppError::NotConfigured);
    }
    let config = DataRootRuntimeConfig {
        active_data_root: data_root.into(),
        opencodex_home_mode: document.opencodex_home_mode,
        opencodex_home_path: document.opencodex_home_path,
    };
    validate_home_config(&config)?;
    Ok(config)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetadataDocument {
    structure_version: String,
    opencodex_home_mode: OpenCodexHomeMode,
    #[serde(skip_serializing_if = "Option::is_none")]
    opencodex_home_path: Option<PathBuf>,
}

fn read_metadata_document(path: &Path) -> Result<MetadataDocument, AppError> {
    use std::io::Read;
    let mut bytes = Vec::new();
    let file = std::fs::File::open(path).map_err(|e| AppError::FileSystem {
        operation: "read v2 directory metadata".into(),
        detail: e.to_string(),
    })?;
    file.take(65537)
        .read_to_end(&mut bytes)
        .map_err(|e| AppError::FileSystem {
            operation: "read v2 directory metadata".into(),
            detail: e.to_string(),
        })?;
    if bytes.len() > 65536 {
        return Err(AppError::NotConfigured);
    }
    serde_json::from_slice(&bytes).map_err(|_| AppError::NotConfigured)
}

fn validate_home_config(config: &DataRootRuntimeConfig) -> Result<(), AppError> {
    startup_v2::checked_path(&config.active_data_root)?;
    if config.active_data_root.parent().is_none() {
        return Err(AppError::NotConfigured);
    }
    match (
        config.opencodex_home_mode,
        config.opencodex_home_path.as_deref(),
    ) {
        (OpenCodexHomeMode::Inside, None) => {
            startup_v2::check_owner(&resolve_opencodex_home(config))?;
            if is_nested(&config.active_data_root, &resolve_opencodex_home(config)) {
                return Err(AppError::NotConfigured);
            }
            Ok(())
        }
        (OpenCodexHomeMode::External, Some(home)) => {
            startup_v2::checked_path(home)?;
            startup_v2::check_owner(home)?;
            if is_nested(&config.active_data_root, home) {
                return Err(AppError::NotConfigured);
            }
            Ok(())
        }
        _ => Err(AppError::NotConfigured),
    }
}

pub fn save_runtime_config(
    data_root: &Path,
    value: &DataRootRuntimeConfig,
) -> Result<(), AppError> {
    startup_v2::check_owner(data_root)?;
    check_partitions(data_root)?;
    load_runtime_config(data_root)?;
    if !same_runtime_directory(data_root, &value.active_data_root) {
        return Err(AppError::NotConfigured);
    }
    validate_home_config(value)?;
    let path = runtime_config_path(data_root);
    crate::infrastructure::atomic_write::atomic_write(
        &path,
        &merged_metadata_payload(value)?,
        0o600,
    )?;
    startup_v2::sync_parent(&path)?;
    if load_runtime_config(data_root)? != *value {
        return Err(AppError::NotConfigured);
    }
    Ok(())
}

fn merged_metadata_payload(value: &DataRootRuntimeConfig) -> Result<Vec<u8>, AppError> {
    validate_home_config(value)?;
    serde_json::to_vec_pretty(&MetadataDocument {
        structure_version: STRUCTURE_VERSION.into(),
        opencodex_home_mode: value.opencodex_home_mode,
        opencodex_home_path: value.opencodex_home_path.clone(),
    })
    .map_err(|e| AppError::AtomicWrite {
        reason: e.to_string(),
    })
}

/// The anchor is fixed current-user D; target is aggregate U.
/// Selecting a root never copies or rewrites either root's HOME settings.
pub fn switch_reference(
    anchor: &Path,
    target: &Path,
    mode: DataRootSwitchMode,
) -> Result<DataRootRuntimeConfig, AppError> {
    if mode == DataRootSwitchMode::MigrateData {
        return Err(migration::unsupported());
    }

    let before = startup_v2::selected_config(anchor)?;
    let target = startup_v2::checked_path(target)?;
    let current = before
        .active_data_root
        .parent()
        .ok_or(AppError::NotConfigured)?;
    let fixed = anchor.parent().ok_or(AppError::NotConfigured)?;
    for root in [current, fixed] {
        if !same_runtime_directory(root, &target) && is_nested(root, &target) {
            return Err(AppError::NotConfigured);
        }
    }
    let desktop = target.join("desktop");
    if validate_structure(&desktop)? != StructureValidation::Valid {
        return Err(AppError::NotConfigured);
    }
    startup_v2::check_owner(&target)?;
    startup_v2::check_owner(&desktop)?;
    let config = load_runtime_config(&desktop)?;
    if is_nested(anchor, &resolve_opencodex_home(&config)) {
        return Err(AppError::NotConfigured);
    }
    let _target_lock = if !same_runtime_directory(&desktop, &before.active_data_root) {
        Some(crate::modules::instance::AppInstanceLock::acquire(
            &desktop,
        )?)
    } else {
        None
    };
    startup_v2::set_location(anchor, &target)?;
    Ok(config)
}

/// Edit one explicitly supplied D, never an active pointer.
pub fn set_opencodex_home(
    current: &Path,
    mode: OpenCodexHomeMode,
    external_path: Option<&Path>,
) -> Result<DataRootRuntimeConfig, AppError> {
    let mut config = load_runtime_config(current)?;
    config.opencodex_home_mode = mode;
    config.opencodex_home_path = match mode {
        OpenCodexHomeMode::Inside => None,
        OpenCodexHomeMode::External => Some(startup_v2::checked_path(
            external_path.ok_or(AppError::NotConfigured)?,
        )?),
    };
    save_runtime_config(current, &config)?;
    Ok(config)
}

pub fn resolve_opencodex_home(config: &DataRootRuntimeConfig) -> PathBuf {
    match config.opencodex_home_mode {
        OpenCodexHomeMode::Inside => config
            .active_data_root
            .parent()
            .expect("desktop has an aggregate parent")
            .join("opencodex/home"),
        OpenCodexHomeMode::External => config
            .opencodex_home_path
            .clone()
            .expect("validated external path"),
    }
}

/// Compare resolved directory identities, not display spelling (for example
/// macOS /tmp and /private/tmp). Resolution failure must not erase a pending
/// binding change. This does not replace boundary validation or instance locks.
pub fn runtime_binding_matches(
    config: &DataRootRuntimeConfig,
    runtime_root: &Path,
    runtime_home: &Path,
) -> bool {
    same_runtime_directory(&config.active_data_root, runtime_root)
        && same_runtime_directory(&resolve_opencodex_home(config), runtime_home)
}

pub fn same_runtime_directory(saved: &Path, current: &Path) -> bool {
    saved == current
        || matches!((startup_v2::checked_path(saved), startup_v2::checked_path(current)),
            (Ok(saved), Ok(current)) if saved == current)
}

pub(crate) fn is_nested(left: &Path, right: &Path) -> bool {
    match (
        startup_v2::checked_path(left),
        startup_v2::checked_path(right),
    ) {
        (Ok(left), Ok(right)) => left.starts_with(&right) || right.starts_with(&left),
        _ => true,
    }
}

fn metadata_matches_version(path: &Path) -> Result<bool, AppError> {
    if !path.is_file() {
        return Ok(false);
    }
    let validation = read_and_validate_metadata(path)?;
    if validation == StructureValidation::FutureVersion {
        return Err(AppError::FileSystem {
            operation: "validate structure version".to_string(),
            detail: "data root structure version is newer than supported version 2".to_string(),
        });
    }
    if validation == StructureValidation::Corrupted {
        return Err(AppError::FileSystem {
            operation: "validate data root".to_string(),
            detail: "data root metadata is missing or corrupted".to_string(),
        });
    }
    Ok(true)
}

fn create_partitions(root: &Path) -> Result<(), AppError> {
    let mut created = Vec::new();
    let result = PARTITIONS.iter().try_for_each(|(_, name)| {
        let path = root.join(name);
        // 已存在的分区目录（例如上一次初始化中途中断留下的）只补权限，不再当作失败。
        // 此前这里对每个分区直接 `create_dir`，只要有一个同名目录已存在就返回
        // 「create data root partition; File exists」，导致整个应用启动失败。
        if path.is_dir() {
            set_private_directory(&path)?;
            return Ok(());
        }
        if path.exists() {
            return Err(AppError::FileSystem {
                operation: "create data root partition".to_string(),
                detail: "partition target is not a directory".to_string(),
            });
        }
        crate::infrastructure::platform::create_private_dir_all(&path, 0o700).map_err(|error| {
            AppError::FileSystem {
                operation: "create data root partition".to_string(),
                detail: error.to_string(),
            }
        })?;
        set_private_directory(&path)?;
        created.push(path);
        Ok(())
    });

    if let Err(error) = result {
        rollback_created(created);
        return Err(error);
    }
    Ok(())
}

fn write_metadata_and_lock(paths: &DataRootPaths) -> Result<(), AppError> {
    let payload = merged_metadata_payload(&DataRootRuntimeConfig {
        active_data_root: paths.root.clone(),
        opencodex_home_mode: OpenCodexHomeMode::Inside,
        opencodex_home_path: None,
    })?;
    crate::infrastructure::atomic_write::atomic_write(&paths.metadata, &payload, 0o600)?;
    if let Err(error) =
        crate::infrastructure::atomic_write::atomic_write(&paths.structure_lock, b"locked", 0o600)
    {
        let _ = std::fs::remove_file(&paths.metadata);
        return Err(error);
    }
    Ok(())
}

fn read_and_validate_metadata(path: &Path) -> Result<StructureValidation, AppError> {
    let metadata = match read_metadata_document(path) {
        Ok(document) => document,
        Err(AppError::NotConfigured) => return Ok(StructureValidation::Corrupted),
        Err(error) => return Err(error),
    };
    match metadata.structure_version.parse::<u64>() {
        Ok(2) if metadata.structure_version == STRUCTURE_VERSION => {
            let root = path
                .parent()
                .and_then(Path::parent)
                .ok_or(AppError::NotConfigured)?;
            Ok(
                if validate_home_config(&DataRootRuntimeConfig {
                    active_data_root: root.into(),
                    opencodex_home_mode: metadata.opencodex_home_mode,
                    opencodex_home_path: metadata.opencodex_home_path,
                })
                .is_ok()
                {
                    StructureValidation::Valid
                } else {
                    StructureValidation::Corrupted
                },
            )
        }
        Ok(version) if version > 2 => Ok(StructureValidation::FutureVersion),
        _ => Ok(StructureValidation::Corrupted),
    }
}

fn rollback_created(created: Vec<PathBuf>) {
    for path in created {
        let _ = std::fs::remove_dir(path);
    }
}

fn set_private_directory(path: &Path) -> Result<(), AppError> {
    crate::infrastructure::platform::set_mode(path, 0o700).map_err(|error| AppError::FileSystem {
        operation: "set partition permissions".to_string(),
        detail: error.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn document(version: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "structure_version": version, "opencodex_home_mode": "inside"
        }))
        .unwrap()
    }

    #[test]
    fn initializes_private_v2_partitions_and_home_metadata_idempotently() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("aggregate/desktop");
        assert!(initialize(&root).unwrap());
        let bytes = fs::read(root.join(METADATA_RELATIVE_PATH)).unwrap();
        assert!(!initialize(&root).unwrap());
        assert_eq!(fs::read(root.join(METADATA_RELATIVE_PATH)).unwrap(), bytes);
        assert_eq!(
            validate_structure(&root).unwrap(),
            StructureValidation::Valid
        );
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            value,
            serde_json::json!({"structure_version":"2", "opencodex_home_mode":"inside"})
        );
        let config = load_runtime_config(&root).unwrap();
        assert_eq!(config.active_data_root, root);
        assert_eq!(
            resolve_opencodex_home(&config),
            t.path().join("aggregate/opencodex/home")
        );
        for (_, name) in PARTITIONS {
            assert!(root.join(name).is_dir());
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                assert_eq!(
                    fs::metadata(root.join(name)).unwrap().permissions().mode() & 0o777,
                    0o700
                );
            }
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(root.join(METADATA_RELATIVE_PATH))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn unknown_nonempty_root_is_preserved_instead_of_imported_or_repaired() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("desktop");
        fs::create_dir_all(root.join("manager-state")).unwrap();
        fs::write(root.join("manager-state/keep"), b"keep").unwrap();
        assert!(initialize(&root).is_err());
        assert_eq!(fs::read(root.join("manager-state/keep")).unwrap(), b"keep");
        assert!(!root.join("cache").exists());
        assert_eq!(
            validate_structure(&root).unwrap(),
            StructureValidation::Corrupted
        );
    }

    #[test]
    fn empty_root_is_uninitialized_and_cannot_be_selected() {
        let t = tempfile::tempdir().unwrap();
        assert_eq!(
            validate_structure(t.path()).unwrap(),
            StructureValidation::Uninitialized
        );
        let source = tempfile::tempdir().unwrap();
        initialize(&source.path().join("desktop")).unwrap();
        assert!(switch_reference(
            &source.path().join("desktop"),
            t.path(),
            DataRootSwitchMode::ReferenceOnly
        )
        .is_err());
        assert_eq!(fs::read_dir(t.path()).unwrap().count(), 0);
    }

    #[test]
    fn old_future_and_corrupt_metadata_fail_without_writes() {
        for bytes in [document("1"), document("3"), document("10"), b"{broken".to_vec(),
            br#"{"active_data_root":"/old","opencodex_home_mode":"inside"}"#.to_vec(),
            br#"{"structure_version":"2","opencodex_home_mode":"inside","active_data_root":"/old"}"#.to_vec(),
            br#"{"structure_version":"2"}"#.to_vec()] {
            let t = tempfile::tempdir().unwrap();
            let root = t.path().join("desktop");
            fs::create_dir_all(root.join("manager-state")).unwrap();
            fs::write(root.join(METADATA_RELATIVE_PATH), &bytes).unwrap();
            assert!(initialize(&root).is_err());
            assert!(load_runtime_config(&root).is_err());
            assert_eq!(fs::read(root.join(METADATA_RELATIVE_PATH)).unwrap(), bytes);
            assert!(!root.join("cache").exists());
        }
    }

    #[test]
    fn version_comparison_is_numeric_and_v2_is_current() {
        for (version, expected) in [
            ("1", StructureValidation::Corrupted),
            ("2", StructureValidation::Valid),
            ("3", StructureValidation::FutureVersion),
            ("10", StructureValidation::FutureVersion),
        ] {
            let t = tempfile::tempdir().unwrap();
            fs::create_dir(t.path().join("manager-state")).unwrap();
            fs::write(t.path().join(METADATA_RELATIVE_PATH), document(version)).unwrap();
            assert_eq!(validate_structure(t.path()).unwrap(), expected);
        }
    }

    #[test]
    fn external_home_validation_precedes_metadata_write() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("desktop");
        initialize(&root).unwrap();
        let before = fs::read(runtime_config_path(&root)).unwrap();
        for path in [
            None,
            Some(Path::new("relative")),
            Some(root.as_path()),
            Some(root.join("home").as_path()),
            Some(t.path()),
            Some(t.path().join("missing/../escape").as_path()),
        ] {
            assert!(set_opencodex_home(&root, OpenCodexHomeMode::External, path).is_err());
            assert_eq!(fs::read(runtime_config_path(&root)).unwrap(), before);
        }
        let external = t.path().join("external/new/home");
        let config =
            set_opencodex_home(&root, OpenCodexHomeMode::External, Some(&external)).unwrap();
        assert_eq!(
            resolve_opencodex_home(&config),
            startup_v2::checked_path(&external).unwrap()
        );
        assert!(!external.exists());
        let inside = set_opencodex_home(&root, OpenCodexHomeMode::Inside, None).unwrap();
        assert_eq!(inside.opencodex_home_path, None);
        assert_eq!(fs::read(runtime_config_path(&root)).unwrap(), before);
    }

    #[test]
    fn metadata_cannot_bind_a_different_root() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("desktop");
        initialize(&root).unwrap();
        let before = fs::read(runtime_config_path(&root)).unwrap();
        let mut config = load_runtime_config(&root).unwrap();
        config.active_data_root = t.path().join("different/desktop");
        assert!(save_runtime_config(&root, &config).is_err());
        assert_eq!(fs::read(runtime_config_path(&root)).unwrap(), before);
    }

    #[cfg(unix)]
    #[test]
    fn linked_partitions_cannot_modify_a_different_root() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("desktop");
        initialize(&root).unwrap();
        let outside = t.path().join("outside");
        fs::create_dir(&outside).unwrap();
        fs::remove_dir(root.join("cache")).unwrap();
        std::os::unix::fs::symlink(&outside, root.join("cache")).unwrap();
        let before = fs::metadata(&outside).unwrap().permissions();
        assert!(initialize(&root).is_err());
        assert_eq!(fs::metadata(&outside).unwrap().permissions(), before);
        assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
    }
}
