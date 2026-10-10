//! 数据根命令：只做显式路径编排，不推导用户目录。

use crate::errors::AppResult;
use crate::modules::data_root::{
    initialize as initialize_module, load_runtime_config, resolve_opencodex_home,
    set_opencodex_home, switch_reference, validate_structure, DataRootRuntimeConfig,
    DataRootSwitchBlocked, DataRootSwitchMode, OpenCodexHomeMode, StructureValidation,
};
use serde::Serialize;

use crate::errors::AppError;
use crate::state::{SharedDataRoot, SharedDataRootAnchor, SharedProcessContext};
use tauri::Manager;

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataRootRequest {
    pub root_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataRootInitialization {
    pub created: bool,
    pub structure_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataRootSwitchRequest {
    pub target_path: String,
    pub mode: DataRootSwitchMode,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenCodexHomeRequest {
    pub mode: OpenCodexHomeMode,
    #[serde(default)]
    pub external_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataRootConfigDto {
    /// Saved binding, which can differ from immutable startup services.
    pub active_data_root: String,
    pub opencodex_home_mode: OpenCodexHomeMode,
    pub opencodex_home: String,
    pub current_data_root: String,
    pub current_opencodex_home: String,
    pub runtime_active: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DataRootSwitchResult {
    pub status: DataRootSwitchStatus,
    pub blocked: Option<DataRootSwitchBlocked>,
    pub config: Option<DataRootConfigDto>,
    /// Binding replacement may have happened; old writers must stay frozen.
    pub reconciliation_required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DataRootSwitchStatus {
    Applied,
    RestartRequired,
    Blocked,
}

#[tauri::command]
pub async fn initialize_data_root(
    request: DataRootRequest,
    app: tauri::AppHandle,
) -> AppResult<DataRootInitialization> {
    let root = std::path::PathBuf::from(request.root_path);
    crate::commands::run_blocking("initialize data root", move || {
        let sandbox = app.state::<crate::state::SharedSandboxBoundary>();
        crate::modules::test_sandbox::check_boundary(sandbox.0.as_deref(), &root)
            .map_err(|_| AppError::NotConfigured)?;
        let created = initialize_module(&root)?;
        Ok(DataRootInitialization {
            created,
            structure_version: crate::modules::data_root::STRUCTURE_VERSION.to_string(),
        })
    })
    .await
}

#[tauri::command]
pub async fn validate_data_root_structure(root_path: String) -> AppResult<StructureValidation> {
    crate::commands::run_readonly("validate data root", move || {
        validate_structure(std::path::Path::new(&root_path))
    })
    .await
}

#[tauri::command]
pub fn get_data_root_config(
    data_root: tauri::State<'_, SharedDataRoot>,
    anchor: tauri::State<'_, SharedDataRootAnchor>,
    context: tauri::State<'_, SharedProcessContext>,
) -> AppResult<DataRootConfigDto> {
    data_root_config_with_paths(&anchor.0, &data_root.0, &context.opencodex_home)
}

#[tauri::command]
pub async fn switch_data_root(
    request: DataRootSwitchRequest,
    anchor: tauri::State<'_, SharedDataRootAnchor>,
    app: tauri::AppHandle,
) -> AppResult<DataRootSwitchResult> {
    let root = anchor.0.clone();
    let target = std::path::PathBuf::from(&request.target_path);
    let mode = request.mode;
    let result = crate::commands::run_readonly("switch data root", move || {
        change_binding(&app, |binding| {
            let sandbox = app.state::<crate::state::SharedSandboxBoundary>();
            crate::modules::test_sandbox::check_boundary(sandbox.0.as_deref(), &target)
                .map_err(|_| AppError::NotConfigured)?;
            if mode != DataRootSwitchMode::MigrateData {
                return switch_data_root_with_paths(&root, &target, mode);
            }
            let runtime = app.state::<SharedDataRoot>();
            let context = app.state::<SharedProcessContext>();
            let instances = app.state::<crate::state::InstanceState>();
            // Acquire and reserve before the irreversible binding commit.
            let mut locks = instances
                .locks
                .lock()
                .map_err(|_| AppError::NotConfigured)?;
            migrate_binding(
                MigrationPaths {
                    anchor: &root,
                    source: &runtime.0,
                    home: &context.opencodex_home,
                    target: &target,
                    boundary: sandbox.0.as_deref(),
                },
                &crate::infrastructure::storage_writers::global(),
                binding,
                &mut locks,
            )
        })
    })
    .await?;
    Ok(result)
}

#[tauri::command]
pub async fn set_opencodex_home_config(
    request: OpenCodexHomeRequest,
    anchor: tauri::State<'_, SharedDataRootAnchor>,
    app: tauri::AppHandle,
) -> AppResult<DataRootSwitchResult> {
    let root = anchor.0.clone();
    let external_path = request.external_path.map(std::path::PathBuf::from);
    let mode = request.mode;
    crate::commands::run_readonly("set opencodex home", move || {
        change_binding(&app, |_| {
            if mode == OpenCodexHomeMode::External {
                let sandbox = app.state::<crate::state::SharedSandboxBoundary>();
                crate::modules::test_sandbox::check_boundary(
                    sandbox.0.as_deref(),
                    external_path.as_deref().ok_or(AppError::NotConfigured)?,
                )
                .map_err(|_| AppError::NotConfigured)?;
            }
            set_opencodex_home_with_paths(&root, mode, external_path.as_deref())
        })
    })
    .await
}

pub fn data_root_config_with_paths(
    data_root: &std::path::Path,
    runtime_root: &std::path::Path,
    runtime_home: &std::path::Path,
) -> AppResult<DataRootConfigDto> {
    let config = load_runtime_config(data_root)?;
    Ok(project_config(&config, runtime_root, runtime_home))
}

fn project_config(
    config: &DataRootRuntimeConfig,
    runtime_root: &std::path::Path,
    runtime_home: &std::path::Path,
) -> DataRootConfigDto {
    let opencodex_home = resolve_opencodex_home(config);
    DataRootConfigDto {
        active_data_root: config.active_data_root.display().to_string(),
        opencodex_home_mode: config.opencodex_home_mode,
        opencodex_home: opencodex_home.display().to_string(),
        current_data_root: runtime_root.display().to_string(),
        current_opencodex_home: runtime_home.display().to_string(),
        runtime_active: config.active_data_root == runtime_root && opencodex_home == runtime_home,
    }
}

pub fn switch_data_root_with_paths(
    current: &std::path::Path,
    target: &std::path::Path,
    mode: DataRootSwitchMode,
) -> AppResult<DataRootSwitchResult> {
    let before = load_runtime_config(current)?;
    apply_config_or_blocked(
        switch_reference(current, target, mode),
        DataRootSwitchBlocked::Corrupted,
        &before,
    )
}

pub fn set_opencodex_home_with_paths(
    current: &std::path::Path,
    mode: OpenCodexHomeMode,
    external_path: Option<&std::path::Path>,
) -> AppResult<DataRootSwitchResult> {
    let before = load_runtime_config(current)?;
    apply_config_or_blocked(
        set_opencodex_home(current, mode, external_path),
        DataRootSwitchBlocked::Nested,
        &before,
    )
}

// Busy work is refused, not cancelled/drained. Saved bindings latch writers
// closed until restart; immutable startup services are never rebound in place.
fn change_binding(
    app: &tauri::AppHandle,
    change: impl FnOnce(
        &mut Option<crate::infrastructure::storage_writers::BindingSave>,
    ) -> AppResult<DataRootSwitchResult>,
) -> AppResult<DataRootSwitchResult> {
    let gate = crate::infrastructure::storage_writers::global();
    let Some(binding) = gate.freeze()? else {
        return Ok(blocked_running());
    };
    let mut binding = Some(binding);
    let mutation = app.state::<crate::state::SharedRuntimeInstall>();
    let Some(_lease) = mutation.acquire() else {
        return Ok(blocked_running());
    };
    {
        let collector = app.state::<crate::state::SharedStatusCollector>();
        let mut collector = collector.lock().map_err(|_| AppError::NotConfigured)?;
        if collector.refresh().is_err() || !binding_runtime_is_idle(collector.matrix().runtime) {
            return Ok(blocked_running());
        }
    }
    let root = app.state::<SharedDataRoot>();
    // Same lock order as preference writes: filesystem transaction, status.
    let _transaction = crate::modules::backup::manager::acquire_preferences_transaction(&root.0)?;
    let status = app.state::<crate::commands::update::SharedUpdateStatus>();
    let status = status.lock().map_err(|_| AppError::NotConfigured)?;
    if status.installing || status.pending_restart.is_some() {
        return Ok(blocked_running());
    }
    let context = app.state::<SharedProcessContext>();
    let mut result = change(&mut binding)?;
    if let Some(config) = result.config.as_mut() {
        config.current_data_root = root.0.display().to_string();
        config.current_opencodex_home = context.opencodex_home.display().to_string();
        config.runtime_active = config.active_data_root == config.current_data_root
            && config.opencodex_home == config.current_opencodex_home;
        if !config.runtime_active {
            if let Some(binding) = binding.take() {
                binding.commit();
            }
        }
    }
    Ok(result)
}

fn binding_runtime_is_idle(runtime: crate::types::status::RuntimeState) -> bool {
    matches!(
        runtime,
        crate::types::status::RuntimeState::Stopped | crate::types::status::RuntimeState::NotFound
    )
}

struct MigrationPaths<'a> {
    anchor: &'a std::path::Path,
    source: &'a std::path::Path,
    home: &'a std::path::Path,
    target: &'a std::path::Path,
    boundary: Option<&'a std::path::Path>,
}

fn migration_error(detail: impl Into<String>) -> AppError {
    AppError::FileSystem {
        operation: "migrate data root".into(),
        detail: detail.into(),
    }
}

fn migrate_binding(
    paths: MigrationPaths<'_>,
    gate: &std::sync::Arc<crate::infrastructure::storage_writers::WriterGate>,
    binding: &mut Option<crate::infrastructure::storage_writers::BindingSave>,
    locks: &mut Vec<crate::modules::instance::AppInstanceLock>,
) -> AppResult<DataRootSwitchResult> {
    use std::path::Component;
    if !binding.as_ref().is_some_and(|save| save.permits(gate)) {
        return Err(migration_error("迁移缺少有效写入冻结。"));
    }
    let canonical = |path: &std::path::Path| {
        path.canonicalize()
            .map_err(|e| migration_error(e.to_string()))
    };
    let anchor = canonical(paths.anchor)?;
    let source = canonical(paths.source)?;
    for root in [&anchor, &source] {
        let lock_path = canonical(&root.join("manager-state/app.lock"))?;
        if !locks
            .iter()
            .any(|lock| canonical(lock.lock_path()).is_ok_and(|held| held == lock_path))
        {
            return Err(migration_error("迁移缺少启动锚点或活动数据目录的实例锁。"));
        }
    }
    if !paths.target.is_absolute()
        || paths
            .target
            .components()
            .any(|c| matches!(c, Component::ParentDir))
    {
        return Err(migration_error("目标必须是规范的绝对路径。"));
    }
    let parent = canonical(
        paths
            .target
            .parent()
            .ok_or_else(|| migration_error("目标缺少父目录。"))?,
    )?;
    let target = parent.join(
        paths
            .target
            .file_name()
            .ok_or_else(|| migration_error("目标缺少目录名。"))?,
    );
    if let Some(boundary) = paths.boundary {
        if !target.starts_with(canonical(boundary)?) {
            return Err(migration_error("目标超出测试沙箱边界，未创建迁移目标。"));
        }
    }
    // Actual startup HOME wins over metadata at an intermediate binding root.
    let inside = paths.home == source.join("opencodex-home");
    let config = DataRootRuntimeConfig {
        active_data_root: source,
        opencodex_home_mode: if inside {
            OpenCodexHomeMode::Inside
        } else {
            OpenCodexHomeMode::External
        },
        opencodex_home_path: (!inside).then(|| paths.home.to_path_buf()),
    };
    locks
        .try_reserve(1)
        .map_err(|e| migration_error(e.to_string()))?;
    let mut copy = crate::modules::data_root::migration::copy_verified(&config, &target)?;
    copy.prepare_runtime()?;
    let committed = copy.prepare_activation(&anchor)?.commit()?;
    // No fallible locking/allocation/I/O between commit and retaining the lock.
    // Uncertain commits MUST latch too; returning an ordinary error is unsafe.
    binding.take().expect("checked migration freeze").commit();
    locks.push(committed.target_lock);
    Ok(DataRootSwitchResult {
        status: DataRootSwitchStatus::RestartRequired,
        blocked: None,
        config: Some(project_config(&committed.config, paths.source, paths.home)),
        reconciliation_required: committed.reconciliation_required,
    })
}

fn blocked_running() -> DataRootSwitchResult {
    DataRootSwitchResult {
        status: DataRootSwitchStatus::Blocked,
        blocked: Some(DataRootSwitchBlocked::Running),
        config: None,
        reconciliation_required: false,
    }
}

fn apply_config_or_blocked(
    result: Result<DataRootRuntimeConfig, crate::errors::AppError>,
    blocked: DataRootSwitchBlocked,
    before: &DataRootRuntimeConfig,
) -> AppResult<DataRootSwitchResult> {
    match result {
        Ok(config) => Ok(DataRootSwitchResult {
            status: DataRootSwitchStatus::RestartRequired,
            blocked: None,
            config: Some(project_config(
                &config,
                &before.active_data_root,
                &resolve_opencodex_home(before),
            )),
            reconciliation_required: false,
        }),
        Err(_) => Ok(DataRootSwitchResult {
            status: DataRootSwitchStatus::Blocked,
            blocked: Some(blocked),
            config: None,
            reconciliation_required: false,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::storage_writers::WriterGate;
    use crate::modules::{data_root, instance::AppInstanceLock};
    use crate::types::status::RuntimeState;
    use std::{fs, path::PathBuf, sync::Arc};

    struct MigrationFixture {
        _temp: tempfile::TempDir,
        anchor: PathBuf,
        source: PathBuf,
        target: PathBuf,
        home: PathBuf,
        locks: Vec<AppInstanceLock>,
        gate: Arc<WriterGate>,
    }

    impl MigrationFixture {
        fn new(external: bool) -> Self {
            let temp = tempfile::tempdir().unwrap();
            let anchor = temp.path().join("anchor");
            let source = temp.path().join("source");
            initialize_module(&anchor).unwrap();
            initialize_module(&source).unwrap();
            switch_reference(&anchor, &source, DataRootSwitchMode::ReferenceOnly).unwrap();
            if external {
                let home = temp.path().join("external-home");
                fs::create_dir(&home).unwrap();
                fs::write(home.join("private-config"), b"external untouched").unwrap();
                set_opencodex_home(&anchor, OpenCodexHomeMode::External, Some(&home)).unwrap();
            }
            fs::write(source.join("manager-state/preferences.json"), b"original").unwrap();
            let (config, locks) =
                data_root::bootstrap::resolve_locked_with_boundary(&anchor, None).unwrap();
            let home = resolve_opencodex_home(&config);
            Self {
                anchor: anchor.canonicalize().unwrap(),
                source: config.active_data_root,
                target: temp
                    .path()
                    .canonicalize()
                    .unwrap()
                    .join("target 中文 space"),
                home,
                locks,
                _temp: temp,
                gate: Arc::new(WriterGate::default()),
            }
        }
        fn migrate(
            &mut self,
            binding: &mut Option<crate::infrastructure::storage_writers::BindingSave>,
            boundary: Option<&std::path::Path>,
        ) -> AppResult<DataRootSwitchResult> {
            migrate_binding(
                MigrationPaths {
                    anchor: &self.anchor,
                    source: &self.source,
                    home: &self.home,
                    target: &self.target,
                    boundary,
                },
                &self.gate,
                binding,
                &mut self.locks,
            )
        }
    }

    #[test]
    fn migration_command_retains_locks_and_freeze_until_locked_restart() {
        for external in [false, true] {
            let mut f = MigrationFixture::new(external);
            let mut binding = f.gate.freeze().unwrap();
            let result = f.migrate(&mut binding, None).unwrap();
            assert!(binding.is_none());
            assert!(f.gate.admit().is_err());
            assert_eq!(f.locks.len(), 3);
            assert!(matches!(
                AppInstanceLock::acquire(&f.target),
                Err(AppError::InstanceLockConflict)
            ));
            assert_eq!(result.status, DataRootSwitchStatus::RestartRequired);
            assert!(!result.reconciliation_required);
            let wire = serde_json::to_value(&result).unwrap();
            assert_eq!(wire["reconciliationRequired"], false);
            let config = result.config.unwrap();
            assert!(!config.runtime_active);
            assert_eq!(config.current_data_root, f.source.display().to_string());
            assert_eq!(config.current_opencodex_home, f.home.display().to_string());
            assert_eq!(
                fs::read(f.source.join("manager-state/preferences.json")).unwrap(),
                b"original"
            );
            f.locks.clear();
            let (restarted, _locks) =
                data_root::bootstrap::resolve_locked_with_boundary(&f.anchor, None).unwrap();
            assert_eq!(restarted.active_data_root, f.target);
            assert_eq!(
                resolve_opencodex_home(&restarted),
                if external {
                    f.home.clone()
                } else {
                    f.target.join("opencodex-home")
                }
            );
            assert_eq!(
                fs::read(f.target.join("manager-state/preferences.json")).unwrap(),
                b"original"
            );
            if external {
                assert_eq!(
                    fs::read(f.home.join("private-config")).unwrap(),
                    b"external untouched"
                );
                assert!(!f.target.join("opencodex-home/private-config").exists());
            }
        }
    }

    #[test]
    fn explicit_sandbox_container_supports_migration_and_locked_restart() {
        for external in [false, true] {
            let mut f = MigrationFixture::new(external);
            let boundary = f._temp.path().canonicalize().unwrap();
            crate::modules::test_sandbox::check_boundary(Some(&boundary), &f.target).unwrap();
            let mut binding = f.gate.freeze().unwrap();
            let result = f.migrate(&mut binding, Some(&boundary)).unwrap();
            assert_eq!(result.status, DataRootSwitchStatus::RestartRequired);
            assert!(binding.is_none());
            assert!(f.gate.admit().is_err());
            assert!(matches!(
                AppInstanceLock::acquire(&f.target),
                Err(AppError::InstanceLockConflict)
            ));
            f.locks.clear();
            let (restarted, locks) =
                data_root::bootstrap::resolve_locked_with_boundary(&f.anchor, Some(&boundary))
                    .unwrap();
            assert_eq!(restarted.active_data_root, f.target);
            assert_eq!(locks.len(), 2);
            assert_eq!(
                fs::read(f.target.join("manager-state/preferences.json")).unwrap(),
                b"original"
            );
            assert_eq!(
                resolve_opencodex_home(&restarted),
                if external {
                    f.home.clone()
                } else {
                    f.target.join("opencodex-home")
                }
            );
            if external {
                assert_eq!(
                    fs::read(f.home.join("private-config")).unwrap(),
                    b"external untouched"
                );
            }
            let outside = tempfile::tempdir().unwrap();
            let denied = outside.path().join("daily-root");
            assert!(
                crate::modules::test_sandbox::check_boundary(Some(&boundary), &denied).is_err()
            );
            assert!(!denied.exists());
        }
    }

    #[test]
    fn sandbox_escape_is_denied_before_reserving_target() {
        let mut f = MigrationFixture::new(false);
        let before = fs::read(data_root::runtime_config_path(&f.anchor)).unwrap();
        let mut binding = f.gate.freeze().unwrap();
        let boundary = f.anchor.clone();
        assert!(f.migrate(&mut binding, Some(&boundary)).is_err());
        assert!(!f.target.exists());
        assert_eq!(
            fs::read(data_root::runtime_config_path(&f.anchor)).unwrap(),
            before
        );
        drop(binding);
        assert!(f.gate.admit().is_ok());
    }

    #[test]
    fn migration_requires_matching_freeze_and_both_startup_locks() {
        for missing_lock in [true, false] {
            let mut f = MigrationFixture::new(false);
            let before = fs::read(data_root::runtime_config_path(&f.anchor)).unwrap();
            let wrong_gate = Arc::new(WriterGate::default());
            let mut binding = if missing_lock {
                f.locks.pop();
                f.gate.freeze().unwrap()
            } else {
                wrong_gate.freeze().unwrap()
            };
            assert!(f.migrate(&mut binding, None).is_err());
            assert!(!f.target.exists());
            assert_eq!(
                fs::read(data_root::runtime_config_path(&f.anchor)).unwrap(),
                before
            );
            drop(binding);
            assert!(f.gate.admit().is_ok());
        }
    }

    #[test]
    fn preparation_failure_keeps_old_binding_and_quarantines_partial_copy() {
        let mut f = MigrationFixture::new(false);
        fs::write(
            f.source.join("manager-state/runtime.json"),
            b"invalid runtime metadata",
        )
        .unwrap();
        let before = fs::read(data_root::runtime_config_path(&f.anchor)).unwrap();
        let mut binding = f.gate.freeze().unwrap();
        assert!(f.migrate(&mut binding, None).is_err());
        assert_eq!(f.locks.len(), 2);
        assert_eq!(
            fs::read(data_root::runtime_config_path(&f.anchor)).unwrap(),
            before
        );
        assert!(f
            .target
            .join(data_root::migration::UNPUBLISHED_MARKER)
            .is_file());
        assert!(initialize_module(&f.target).is_err());
        assert!(data_root::bootstrap::resolve(&f.target).is_err());
        drop(binding);
        assert!(f.gate.admit().is_ok());
    }

    #[test]
    fn only_observed_idle_runtime_allows_binding_save() {
        for runtime in [
            RuntimeState::Loading,
            RuntimeState::StartingFailed,
            RuntimeState::ExternalTakeover,
            RuntimeState::AtRisk,
            RuntimeState::Unreachable,
            RuntimeState::Pending,
            RuntimeState::Starting,
            RuntimeState::Stopping,
            RuntimeState::Running,
        ] {
            assert!(!binding_runtime_is_idle(runtime), "{runtime:?}");
        }
        assert!(binding_runtime_is_idle(RuntimeState::Stopped));
        assert!(binding_runtime_is_idle(RuntimeState::NotFound));
        assert_eq!(
            blocked_running().blocked,
            Some(DataRootSwitchBlocked::Running)
        );
    }

    #[test]
    fn home_only_save_stays_pending_until_actual_startup_home_matches() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("data");
        initialize_module(&root).unwrap();
        let actual_home = root.join("opencodex-home");
        let external = temp.path().join("external-home");
        let result =
            set_opencodex_home_with_paths(&root, OpenCodexHomeMode::External, Some(&external))
                .unwrap();
        let config = result.config.unwrap();
        assert_eq!(result.status, DataRootSwitchStatus::RestartRequired);
        assert!(!config.runtime_active);
        assert_eq!(
            config.current_opencodex_home,
            actual_home.display().to_string()
        );
        let reread = data_root_config_with_paths(&root, &root, &actual_home).unwrap();
        assert!(!reread.runtime_active);
        assert_eq!(reread.opencodex_home, external.display().to_string());
        assert!(
            data_root_config_with_paths(&root, &root, &external)
                .unwrap()
                .runtime_active
        );
        let wire = serde_json::to_value(reread).unwrap();
        assert!(wire.get("currentDataRoot").is_some());
        assert!(wire.get("currentOpencodexHome").is_some());
    }

    #[test]
    fn reference_switch_preserves_external_home_without_copying() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("source");
        let target = temp.path().join("target");
        let external = temp.path().join("external-home");
        initialize_module(&root).unwrap();
        initialize_module(&target).unwrap();
        set_opencodex_home_with_paths(&root, OpenCodexHomeMode::External, Some(&external)).unwrap();
        std::fs::write(root.join("manager-state/source-only"), b"untouched").unwrap();
        let result =
            switch_data_root_with_paths(&root, &target, DataRootSwitchMode::ReferenceOnly).unwrap();
        let config = result.config.unwrap();
        assert_eq!(config.current_data_root, root.display().to_string());
        assert_eq!(config.active_data_root, target.display().to_string());
        assert_eq!(config.opencodex_home, external.display().to_string());
        assert!(!config.runtime_active);
        assert!(!target.join("manager-state/source-only").exists());
    }
}
