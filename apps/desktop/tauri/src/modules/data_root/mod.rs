//! FZ-01 ~ FZ-03 数据根初始化与结构校验。
//!
//! 模块只作用于显式传入的数据根目录；测试使用临时路径。
//! 不扫描用户目录、不迁移数据、不启动代理、不访问 Keychain。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::errors::AppError;

pub const STRUCTURE_VERSION: &str = "1";
pub const METADATA_RELATIVE_PATH: &str = "manager-state/data-root.json";
pub const STRUCTURE_LOCK_RELATIVE_PATH: &str = "manager-state/structure.lock";

pub const PARTITIONS: [(&str, &str); 7] = [
    ("manager state", "manager-state"),
    ("opencodex home", "opencodex-home"),
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
    Running,
    Nested,
    FutureVersion,
    Corrupted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StructureValidation {
    Valid,
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
    std::fs::create_dir_all(root).map_err(|error| AppError::FileSystem {
        operation: "create data root".to_string(),
        detail: error.to_string(),
    })?;
    let paths = DataRootPaths::new(root.to_path_buf());

    // 旧行为会把结构版本连同运行期配置一起覆盖掉；这里先自愈，避免用户
    // 切换数据根后直接起不来。
    repair_missing_structure_version(root)?;

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
        ensure_partitions(root)?;
        return Ok(false);
    }

    create_partitions(root)?;
    write_metadata_and_lock(&paths)?;

    Ok(true)
}

/// 幂等补建既有数据根的缺失分区；不改动已有内容。
pub fn ensure_partitions(root: &Path) -> Result<(), AppError> {
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
            std::fs::create_dir(&path).map_err(|error| AppError::FileSystem {
                operation: "ensure data root partition".to_string(),
                detail: error.to_string(),
            })?;
            set_private_directory(&path)?;
        }
    }
    Ok(())
}

pub fn validate_structure(root: &Path) -> Result<StructureValidation, AppError> {
    if !root.is_dir() {
        return Err(AppError::FileSystem {
            operation: "validate data root".to_string(),
            detail: format!("{} is not a directory", root.display()),
        });
    }
    let metadata = root.join(METADATA_RELATIVE_PATH);
    if !metadata.is_file() {
        return Ok(StructureValidation::Corrupted);
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

pub fn load_runtime_config(data_root: &Path) -> Result<DataRootRuntimeConfig, AppError> {
    if !data_root.is_absolute() {
        return Err(AppError::NotConfigured);
    }
    let path = runtime_config_path(data_root);
    let bytes = std::fs::read(&path).map_err(|error| AppError::FileSystem {
        operation: "read data root runtime config".to_string(),
        detail: error.to_string(),
    })?;
    let payload: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| AppError::FileSystem {
            operation: "parse data root runtime config".to_string(),
            detail: "invalid runtime config".to_string(),
        })?;
    if payload.get("structure_version")
        == Some(&serde_json::Value::String(STRUCTURE_VERSION.to_string()))
        && payload.get("active_data_root").is_none()
    {
        return Ok(DataRootRuntimeConfig {
            active_data_root: data_root.to_path_buf(),
            opencodex_home_mode: OpenCodexHomeMode::Inside,
            opencodex_home_path: None,
        });
    }
    let config: DataRootRuntimeConfig =
        serde_json::from_value(payload).map_err(|_| AppError::FileSystem {
            operation: "parse data root runtime config".to_string(),
            detail: "invalid runtime config".to_string(),
        })?;
    if !config.active_data_root.is_absolute() {
        return Err(AppError::FileSystem {
            operation: "validate active data root".to_string(),
            detail: "active data root must be an absolute path".to_string(),
        });
    }
    match config.opencodex_home_mode {
        OpenCodexHomeMode::Inside => Ok(config),
        OpenCodexHomeMode::External => match config.opencodex_home_path {
            Some(ref path) if path.is_absolute() => Ok(config),
            _ => Err(AppError::FileSystem {
                operation: "validate OPENCODEX_HOME".to_string(),
                detail: "external OPENCODEX_HOME requires an absolute path".to_string(),
            }),
        },
    }
}

pub fn save_runtime_config(
    data_root: &Path,
    value: &DataRootRuntimeConfig,
) -> Result<(), AppError> {
    if !data_root.is_absolute() || !value.active_data_root.is_absolute() {
        return Err(AppError::NotConfigured);
    }
    if let OpenCodexHomeMode::External = value.opencodex_home_mode {
        if !value
            .opencodex_home_path
            .as_ref()
            .is_some_and(|path| path.is_absolute())
        {
            return Err(AppError::FileSystem {
                operation: "save OPENCODEX_HOME".to_string(),
                detail: "external OPENCODEX_HOME requires an absolute path".to_string(),
            });
        }
    }
    // `manager-state/data-root.json` 同时被当作结构元数据与运行期配置读取：
    // `initialize` 按 `DataRootMetadata` 解析它，`load_runtime_config` 按
    // `DataRootRuntimeConfig` 解析它。此前这里只写运行期字段，把
    // `structure_version` 整个覆盖掉，切换数据根后应用再也起不来
    // （`parse structure metadata; invalid structure metadata`）。
    // 因此写回时必须保留结构版本，让两种读者都能解析同一份文档。
    let payload = merged_metadata_payload(value)?;
    crate::infrastructure::atomic_write::atomic_write(
        &runtime_config_path(data_root),
        &payload,
        0o600,
    )
}

/// 生成「结构版本 + 运行期配置」的合并文档。
fn merged_metadata_payload(value: &DataRootRuntimeConfig) -> Result<Vec<u8>, AppError> {
    let mut document = serde_json::Map::new();
    document.insert(
        "structure_version".to_string(),
        serde_json::Value::String(STRUCTURE_VERSION.to_string()),
    );
    let runtime = serde_json::to_value(value).map_err(|error| AppError::AtomicWrite {
        reason: error.to_string(),
    })?;
    let serde_json::Value::Object(fields) = runtime else {
        return Err(AppError::AtomicWrite {
            reason: "runtime config is not an object".to_string(),
        });
    };
    for (key, value) in fields {
        document.insert(key, value);
    }
    serde_json::to_vec_pretty(&serde_json::Value::Object(document)).map_err(|error| {
        AppError::AtomicWrite {
            reason: error.to_string(),
        }
    })
}

/// 修复被旧行为覆盖掉结构版本的数据根：文件里只剩运行期配置。
///
/// 返回 `true` 表示确实修复过。只认「合法 JSON 且带 `active_data_root`、
/// 缺 `structure_version`」这一种形态，其余异常仍按损坏处理。
pub fn repair_missing_structure_version(data_root: &Path) -> Result<bool, AppError> {
    let paths = DataRootPaths::new(data_root.to_path_buf());
    let Ok(bytes) = std::fs::read(&paths.metadata) else {
        return Ok(false);
    };
    let Ok(payload) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return Ok(false);
    };
    if payload.get("structure_version").is_some() || payload.get("active_data_root").is_none() {
        return Ok(false);
    }
    let Ok(config) = serde_json::from_value::<DataRootRuntimeConfig>(payload) else {
        return Ok(false);
    };
    crate::infrastructure::atomic_write::atomic_write(
        &paths.metadata,
        &merged_metadata_payload(&config)?,
        0o600,
    )?;
    Ok(true)
}

pub fn switch_reference(
    current: &Path,
    target: &Path,
    mode: DataRootSwitchMode,
) -> Result<DataRootRuntimeConfig, AppError> {
    if mode == DataRootSwitchMode::MigrateData {
        return Err(AppError::FileSystem {
            operation: "switch data root".to_string(),
            detail: "data migration is not enabled in this task".to_string(),
        });
    }
    // 「切回数据根自身」是合法的重置操作，不该被嵌套校验挡住；
    // 否则用户一旦切走就再也回不到管理器自己的数据根。
    let canonical_current = current
        .canonicalize()
        .unwrap_or_else(|_| current.to_path_buf());
    let canonical_target = target
        .canonicalize()
        .unwrap_or_else(|_| target.to_path_buf());
    if canonical_current != canonical_target && is_nested(current, target) {
        return Err(AppError::FileSystem {
            operation: "switch data root".to_string(),
            detail: "current and target data roots cannot be nested".to_string(),
        });
    }
    if validate_structure(target)? != StructureValidation::Valid {
        return Err(AppError::FileSystem {
            operation: "switch data root".to_string(),
            detail: "target data root is not current version 1".to_string(),
        });
    }
    let config = DataRootRuntimeConfig {
        active_data_root: target.to_path_buf(),
        opencodex_home_mode: OpenCodexHomeMode::Inside,
        opencodex_home_path: None,
    };
    save_runtime_config(current, &config)?;
    Ok(config)
}

pub fn set_opencodex_home(
    current: &Path,
    mode: OpenCodexHomeMode,
    external_path: Option<&Path>,
) -> Result<DataRootRuntimeConfig, AppError> {
    let mut config = load_runtime_config(current)?;
    let resolved_path: Option<&Path> = match mode {
        OpenCodexHomeMode::Inside => None,
        OpenCodexHomeMode::External => Some(external_path.ok_or_else(|| AppError::NotConfigured)?),
    };
    if let Some(path) = resolved_path {
        if is_nested(current, path) || *path == current.join("opencodex-home") {
            return Err(AppError::FileSystem {
                operation: "set OPENCODEX_HOME".to_string(),
                detail: "OPENCODEX_HOME cannot be nested in the manager data root".to_string(),
            });
        }
    }
    config.active_data_root = current.to_path_buf();
    config.opencodex_home_mode = mode;
    config.opencodex_home_path = resolved_path.map(Path::to_path_buf);
    save_runtime_config(current, &config)?;
    Ok(config)
}

pub fn resolve_opencodex_home(config: &DataRootRuntimeConfig) -> PathBuf {
    match config.opencodex_home_mode {
        OpenCodexHomeMode::Inside => config.active_data_root.join("opencodex-home"),
        OpenCodexHomeMode::External => config
            .opencodex_home_path
            .clone()
            .expect("validated external path"),
    }
}

fn is_nested(left: &Path, right: &Path) -> bool {
    let left = left.canonicalize().unwrap_or_else(|_| left.to_path_buf());
    let right = match right.canonicalize() {
        Ok(path) => path,
        Err(_) => {
            let parent = right
                .parent()
                .and_then(|parent| parent.canonicalize().ok())
                .unwrap_or_else(|| right.to_path_buf());
            match right.file_name() {
                Some(name) => parent.join(name),
                None => right.to_path_buf(),
            }
        }
    };
    if left.starts_with(&right) || right.starts_with(&left) {
        return true;
    }
    left.starts_with(&right)
        || left.join("opencodex-home").starts_with(&right)
        || right.join("opencodex-home").starts_with(&left)
}

fn metadata_matches_version(path: &Path) -> Result<bool, AppError> {
    if !path.is_file() {
        return Ok(false);
    }
    let validation = read_and_validate_metadata(path)?;
    if validation == StructureValidation::FutureVersion {
        return Err(AppError::FileSystem {
            operation: "validate structure version".to_string(),
            detail: "data root structure version is newer than supported version 1".to_string(),
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
        std::fs::create_dir(&path).map_err(|error| AppError::FileSystem {
            operation: "create data root partition".to_string(),
            detail: error.to_string(),
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
    let metadata = DataRootMetadata {
        structure_version: STRUCTURE_VERSION.to_string(),
    };
    let payload = serde_json::to_vec_pretty(&metadata).map_err(|error| AppError::AtomicWrite {
        reason: error.to_string(),
    })?;
    if let Err(error) =
        crate::infrastructure::atomic_write::atomic_write(&paths.metadata, &payload, 0o600)
    {
        rollback_partitions(&paths.root);
        return Err(error);
    }
    if let Err(error) =
        crate::infrastructure::atomic_write::atomic_write(&paths.structure_lock, b"locked", 0o600)
    {
        let _ = std::fs::remove_file(&paths.metadata);
        rollback_partitions(&paths.root);
        return Err(error);
    }
    Ok(())
}

fn read_and_validate_metadata(path: &Path) -> Result<StructureValidation, AppError> {
    let bytes = std::fs::read(path).map_err(|error| AppError::FileSystem {
        operation: "read structure metadata".to_string(),
        detail: error.to_string(),
    })?;
    let metadata: DataRootMetadata =
        serde_json::from_slice(&bytes).map_err(|_| AppError::FileSystem {
            operation: "parse structure metadata".to_string(),
            detail: "invalid structure metadata".to_string(),
        })?;
    match metadata.structure_version.as_str() {
        "1" => Ok(StructureValidation::Valid),
        value if value > "1" => Ok(StructureValidation::FutureVersion),
        _ => Ok(StructureValidation::Corrupted),
    }
}

fn rollback_created(created: Vec<PathBuf>) {
    for path in created {
        let _ = std::fs::remove_dir(path);
    }
}

fn rollback_partitions(root: &Path) {
    for (_, name) in PARTITIONS {
        let path = root.join(name);
        let _ = std::fs::remove_dir_all(path);
    }
}

fn set_private_directory(path: &Path) -> Result<(), AppError> {
    crate::infrastructure::platform::set_mode(path, 0o700).map_err(|error| AppError::FileSystem {
        operation: "set partition permissions".to_string(),
        detail: error.to_string(),
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;

    /// 回归：数据根目录已存在（上一次初始化中断、或用户手工创建）但缺少元数据时，
    /// 初始化必须补齐分区并成功启动，而不是以 `File exists` 让整个应用崩溃。
    #[test]
    fn initialize_recovers_when_partition_directory_already_exists() {
        let temp = tempfile::tempdir().expect("temp parent");
        let root = temp.path().join("data-root");
        // 预先创建其中一个分区目录，模拟中断留下的半成品数据根。
        std::fs::create_dir_all(root.join("manager-state")).expect("pre-create partition");
        std::fs::write(root.join("manager-state").join("keep.txt"), b"keep")
            .expect("seed user data");

        let created = initialize(&root).expect("已存在的分区目录不得阻断启动");
        assert!(created, "缺少元数据时应视为首次初始化");
        assert_eq!(
            validate_structure(&root).expect("validate"),
            StructureValidation::Valid
        );
        // 预先存在的内容必须保留，不能被回滚删掉。
        assert_eq!(
            std::fs::read_to_string(root.join("manager-state").join("keep.txt")).expect("keep"),
            "keep"
        );
    }

    /// 已有的合法数据根重复初始化必须幂等，且不返回「新建」。
    #[test]
    fn initialize_is_idempotent_on_existing_root() {
        let temp = tempfile::tempdir().expect("temp parent");
        let root = temp.path().join("data-root");
        assert!(initialize(&root).expect("first init"));
        assert!(
            !initialize(&root).expect("second init"),
            "第二次不应再视为新建"
        );
        assert_eq!(
            validate_structure(&root).expect("validate"),
            StructureValidation::Valid
        );
    }

    /// 回归：切换数据根引用后，本数据根的结构元数据必须仍然可解析，
    /// 否则「重启后应用起不来」。此前运行期配置会把结构元数据整个覆盖掉。
    #[test]
    fn switching_reference_keeps_structure_metadata_readable() {
        let temp = tempfile::tempdir().expect("temp parent");
        let root = temp.path().join("root-a");
        let target = temp.path().join("root-b");
        initialize(&root).expect("init root");
        initialize(&target).expect("init target");

        switch_reference(&root, &target, DataRootSwitchMode::ReferenceOnly).expect("switch");

        // 模拟重启：同一数据根必须仍能初始化，而不是报结构元数据损坏。
        assert!(!initialize(&root).expect("switching must not break startup"));
        assert_eq!(
            validate_structure(&root).expect("validate"),
            StructureValidation::Valid
        );
        assert_eq!(
            load_runtime_config(&root)
                .expect("runtime config")
                .active_data_root,
            target
        );
    }

    /// 回归：被旧行为覆盖成「只剩运行期配置」的数据根，启动时要能自愈。
    #[test]
    fn clobbered_structure_metadata_is_repaired_on_initialize() {
        let temp = tempfile::tempdir().expect("temp parent");
        let root = temp.path().join("root-a");
        let target = temp.path().join("root-b");
        initialize(&root).expect("init root");
        initialize(&target).expect("init target");
        let clobbered = serde_json::json!({
            "active_data_root": target,
            "opencodex_home_mode": "inside",
        });
        std::fs::write(
            root.join(METADATA_RELATIVE_PATH),
            serde_json::to_vec(&clobbered).expect("json"),
        )
        .expect("write clobbered metadata");

        assert!(!initialize(&root).expect("must self-heal"));
        assert_eq!(
            validate_structure(&root).expect("validate"),
            StructureValidation::Valid
        );
        assert_eq!(
            load_runtime_config(&root)
                .expect("runtime config")
                .active_data_root,
            target
        );
    }

    /// 回归：必须允许「切回数据根自身」，否则用户切走后无法回到管理器自己的数据根，
    /// 也无法把 external OPENCODEX_HOME 之后的引用重置回默认根。
    #[test]
    fn switching_back_to_the_data_root_itself_is_allowed() {
        let temp = tempfile::tempdir().expect("temp parent");
        let root = temp.path().join("root-a");
        let other = temp.path().join("root-b");
        initialize(&root).expect("init root");
        initialize(&other).expect("init other");

        switch_reference(&root, &other, DataRootSwitchMode::ReferenceOnly).expect("switch away");
        switch_reference(&root, &root, DataRootSwitchMode::ReferenceOnly).expect("switch back");

        assert_eq!(
            load_runtime_config(&root)
                .expect("runtime config")
                .active_data_root,
            root
        );
        assert_eq!(
            validate_structure(&root).expect("validate"),
            StructureValidation::Valid
        );
    }

    fn metadata_json(version: &str) -> Vec<u8> {
        serde_json::to_vec(&DataRootMetadata {
            structure_version: version.to_string(),
        })
        .expect("serialize metadata")
    }

    #[test]
    fn initializes_seven_private_partitions_and_metadata() {
        let temp = tempfile::tempdir().expect("create temporary root parent");
        let root = temp.path().join("data-root");

        let created = initialize(&root).expect("initialize data root");
        assert!(created);
        assert_eq!(
            validate_structure(&root).expect("validate"),
            StructureValidation::Valid
        );

        for (_, name) in PARTITIONS {
            let path = root.join(name);
            assert!(path.is_dir(), "{name} should exist");
            let mode = std::fs::metadata(&path).unwrap().permissions().mode();
            assert_eq!(mode & 0o777, 0o700);
        }

        let metadata_path = root.join(METADATA_RELATIVE_PATH);
        assert!(metadata_path.is_file());
        let mode = std::fs::metadata(&metadata_path)
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600);
        assert!(root.join(STRUCTURE_LOCK_RELATIVE_PATH).is_file());
    }

    #[test]
    fn existing_valid_structure_is_referenced_idempotently() {
        let temp = tempfile::tempdir().expect("create temporary root parent");
        let root = temp.path().join("data-root");
        std::fs::create_dir_all(root.join("manager-state")).expect("create manager state");
        std::fs::write(root.join(METADATA_RELATIVE_PATH), metadata_json("1"))
            .expect("write metadata");

        let created = initialize(&root).expect("reference existing root");

        assert!(!created);
        assert_eq!(
            validate_structure(&root).expect("validate"),
            StructureValidation::Valid
        );
        for (_, name) in PARTITIONS {
            assert!(root.join(name).is_dir(), "{name} should be idempotent");
            let mode = std::fs::metadata(root.join(name))
                .unwrap()
                .permissions()
                .mode();
            assert_eq!(mode & 0o777, 0o700);
        }
    }

    #[test]
    fn future_structure_version_is_blocked_before_writes() {
        let temp = tempfile::tempdir().expect("create temporary root parent");
        let root = temp.path().join("data-root");
        std::fs::create_dir_all(root.join("manager-state")).expect("create manager state");
        std::fs::write(root.join(METADATA_RELATIVE_PATH), metadata_json("2"))
            .expect("write metadata");

        let error = initialize(&root).expect_err("future version should fail");

        assert!(error.to_string().contains("newer than supported"));
        for (_, name) in PARTITIONS {
            if name == "manager-state" {
                continue;
            }
            assert!(!root.join(name).exists());
        }
    }

    #[test]
    fn corrupted_or_missing_metadata_is_blocked() {
        let temp = tempfile::tempdir().expect("create temporary root parent");
        let root = temp.path().join("empty");
        std::fs::create_dir(&root).expect("create empty root");
        assert_eq!(
            validate_structure(&root).expect("validate"),
            StructureValidation::Corrupted
        );

        let root = temp.path().join("broken");
        std::fs::create_dir_all(root.join("manager-state")).expect("create manager state");
        std::fs::write(root.join(METADATA_RELATIVE_PATH), b"{broken").expect("write metadata");
        assert!(initialize(&root).is_err());
    }

    #[test]
    fn non_directory_target_is_rejected() {
        let temp = tempfile::tempdir().expect("create temporary parent");
        let target = temp.path().join("not-directory");
        std::fs::write(&target, b"file").expect("write fixture");
        assert!(initialize(&target).is_err());
    }

    #[test]
    fn write_failure_does_not_leave_partial_structure() {
        let temp = tempfile::tempdir().expect("create temporary parent");
        let root = temp.path().join("data-root");
        std::fs::create_dir(&root).expect("create root");
        let metadata_parent = root.join("manager-state");
        std::fs::create_dir(&metadata_parent).expect("create manager state");
        let metadata_path = metadata_parent.join("data-root.json");
        std::fs::write(&metadata_path, b"existing").expect("write metadata");
        std::fs::set_permissions(&metadata_path, Permissions::from_mode(0o444))
            .expect("make read-only");

        let result = initialize(&root);
        std::fs::set_permissions(&metadata_path, Permissions::from_mode(0o644))
            .expect("restore permissions");

        assert!(result.is_err());
        assert!(!root.join("backups").exists());
        assert!(!root.join("logs").exists());
    }

    #[test]
    fn runtime_config_defaults_inside_home_and_rejects_relative_root() {
        let temp = tempfile::tempdir().expect("create temporary root");
        let root = temp.path().join("data-root");
        initialize(&root).expect("initialize data root");
        let config = DataRootRuntimeConfig {
            active_data_root: root.clone(),
            opencodex_home_mode: OpenCodexHomeMode::Inside,
            opencodex_home_path: None,
        };
        save_runtime_config(&root, &config).expect("save runtime config");
        let loaded = load_runtime_config(&root).expect("load runtime config");
        assert_eq!(loaded.active_data_root, root);
        assert_eq!(resolve_opencodex_home(&loaded), root.join("opencodex-home"));

        let relative = temp.path().join("relative");
        std::fs::create_dir(&relative).expect("create relative root");
        assert!(load_runtime_config(&relative).is_err());
    }

    #[test]
    fn external_home_requires_absolute_path_and_rejects_nesting() {
        let temp = tempfile::tempdir().expect("create temporary root");
        let root = temp.path().join("data-root");
        initialize(&root).expect("initialize data root");
        let config = DataRootRuntimeConfig {
            active_data_root: root.clone(),
            opencodex_home_mode: OpenCodexHomeMode::Inside,
            opencodex_home_path: None,
        };
        save_runtime_config(&root, &config).expect("save runtime config");
        assert!(set_opencodex_home(&root, OpenCodexHomeMode::External, None).is_err());
        assert!(
            set_opencodex_home(&root, OpenCodexHomeMode::External, Some(&root.join("home")))
                .is_err()
        );
        let external = temp.path().join("external-home");
        std::fs::create_dir(&external).expect("create external home");
        let updated = set_opencodex_home(&root, OpenCodexHomeMode::External, Some(&external))
            .expect("set external");
        assert_eq!(resolve_opencodex_home(&updated), external);
        let loaded = load_runtime_config(&root).expect("load runtime config");
        assert_eq!(loaded.opencodex_home_mode, OpenCodexHomeMode::External);
    }

    #[test]
    fn reference_switch_requires_valid_non_nested_target() {
        let temp = tempfile::tempdir().expect("create temporary parent");
        let current = temp.path().join("current");
        initialize(&current).expect("initialize current");
        assert!(switch_reference(
            &current,
            &current.join("nested"),
            DataRootSwitchMode::ReferenceOnly
        )
        .is_err());

        let future = temp.path().join("future");
        std::fs::create_dir_all(future.join("manager-state")).expect("create future");
        std::fs::write(future.join(METADATA_RELATIVE_PATH), metadata_json("2"))
            .expect("write future");
        assert!(switch_reference(&current, &future, DataRootSwitchMode::ReferenceOnly).is_err());

        let target = temp.path().join("target");
        initialize(&target).expect("initialize target");
        let config = switch_reference(&current, &target, DataRootSwitchMode::ReferenceOnly)
            .expect("switch reference");
        assert_eq!(config.active_data_root, target);
        assert!(switch_reference(&current, &target, DataRootSwitchMode::MigrateData).is_err());
    }
}
