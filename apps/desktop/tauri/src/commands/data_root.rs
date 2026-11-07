//! 数据根命令：只做显式路径编排，不推导用户目录。

use crate::errors::AppResult;
use crate::modules::data_root::{
    initialize as initialize_module, resolve_opencodex_home, set_opencodex_home, switch_reference,
    validate_structure, DataRootRuntimeConfig, DataRootSwitchBlocked, DataRootSwitchMode,
    OpenCodexHomeMode, StructureValidation,
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
    pub aggregate_root: String,
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
        let runtime = app.state::<std::sync::Arc<crate::modules::runtime::RuntimeHandle>>();
        let created =
            initialize_with_paths(&root, sandbox.0.as_deref(), runtime.installation_root())?;
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
        validate_structure(&std::path::Path::new(&root_path).join("desktop"))
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
    if request.mode == DataRootSwitchMode::MigrateData {
        return Ok(blocked_result(DataRootSwitchBlocked::Unsupported));
    }
    let root = anchor.0.clone();
    let target = std::path::PathBuf::from(&request.target_path);
    let mode = request.mode;
    let result = crate::commands::run_readonly("switch data root", move || {
        change_binding(&app, |_| {
            let sandbox = app.state::<crate::state::SharedSandboxBoundary>();
            let runtime = app.state::<std::sync::Arc<crate::modules::runtime::RuntimeHandle>>();
            validate_reference_target(&target, sandbox.0.as_deref(), runtime.installation_root())?;
            switch_data_root_with_paths(&root, &target, mode)
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
            if let Some(home) = external_path
                .as_deref()
                .filter(|_| mode == OpenCodexHomeMode::External)
            {
                check_installation_separation(&app, home)?;
            }
            set_opencodex_home_with_paths(&root, mode, external_path.as_deref())
        })
    })
    .await
}

fn initialize_with_paths(
    root: &std::path::Path,
    boundary: Option<&std::path::Path>,
    install: &std::path::Path,
) -> AppResult<bool> {
    use crate::modules::data_root::{layout, startup_v2};
    let layout = layout::Layout::with_aggregate_root(root, install, layout::Platform::Unix)?;
    for path in [
        &layout.aggregate_root,
        &layout.desktop_root,
        &layout.opencodex_home,
        &layout.desktop_root.join("manager-state"),
        &layout.desktop_root.join("manager-state/app.lock"),
    ] {
        crate::modules::test_sandbox::check_boundary(boundary, path)
            .map_err(|_| AppError::NotConfigured)?;
        startup_v2::check_owner(path)?;
        layout::Layout::with_aggregate_root(path, install, layout::Platform::Unix)?;
    }
    crate::modules::data_root::check_partitions(&layout.desktop_root)?;
    let _lock = crate::modules::instance::AppInstanceLock::acquire(&layout.desktop_root)?;
    initialize_module(&layout.desktop_root)
}

pub fn data_root_config_with_paths(
    data_root: &std::path::Path,
    runtime_root: &std::path::Path,
    runtime_home: &std::path::Path,
) -> AppResult<DataRootConfigDto> {
    let config = crate::modules::data_root::startup_v2::selected_config(data_root)?;
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
        aggregate_root: config
            .active_data_root
            .parent()
            .expect("D has aggregate parent")
            .display()
            .to_string(),
        opencodex_home_mode: config.opencodex_home_mode,
        opencodex_home: opencodex_home.display().to_string(),
        current_data_root: runtime_root.display().to_string(),
        current_opencodex_home: runtime_home.display().to_string(),
        runtime_active: crate::modules::data_root::runtime_binding_matches(
            config,
            runtime_root,
            runtime_home,
        ),
    }
}

pub fn switch_data_root_with_paths(
    current: &std::path::Path,
    target: &std::path::Path,
    mode: DataRootSwitchMode,
) -> AppResult<DataRootSwitchResult> {
    if mode == DataRootSwitchMode::MigrateData {
        return Ok(blocked_result(DataRootSwitchBlocked::Unsupported));
    }
    let before = crate::modules::data_root::startup_v2::selected_config(current)?;
    if mode == DataRootSwitchMode::ReferenceOnly {
        for aggregate in [
            current.parent().ok_or(AppError::NotConfigured)?,
            before
                .active_data_root
                .parent()
                .ok_or(AppError::NotConfigured)?,
        ] {
            if !crate::modules::data_root::same_runtime_directory(aggregate, target)
                && crate::modules::data_root::is_nested(aggregate, target)
            {
                return Ok(blocked_result(DataRootSwitchBlocked::Nested));
            }
        }
        match validate_structure(&target.join("desktop")) {
            Ok(StructureValidation::FutureVersion) => {
                return Ok(blocked_result(DataRootSwitchBlocked::FutureVersion))
            }
            Ok(StructureValidation::Valid) => {}
            _ => return Ok(blocked_result(DataRootSwitchBlocked::Corrupted)),
        }
    }
    apply_config_or_blocked(
        switch_reference(current, target, mode),
        DataRootSwitchBlocked::Corrupted,
        &before,
        || crate::modules::data_root::startup_v2::selected_config(current),
    )
}

pub fn set_opencodex_home_with_paths(
    current: &std::path::Path,
    mode: OpenCodexHomeMode,
    external_path: Option<&std::path::Path>,
) -> AppResult<DataRootSwitchResult> {
    let before = crate::modules::data_root::startup_v2::selected_config(current)?;
    if mode == OpenCodexHomeMode::External
        && external_path.is_some_and(|home| crate::modules::data_root::is_nested(current, home))
    {
        return Ok(blocked_result(DataRootSwitchBlocked::Nested));
    }
    apply_config_or_blocked(
        set_opencodex_home(&before.active_data_root, mode, external_path),
        DataRootSwitchBlocked::Nested,
        &before,
        || crate::modules::data_root::startup_v2::selected_config(current),
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
        let verified = collector.refresh().is_ok();
        if let Some(reason) = binding_runtime_blocked(verified, collector.matrix().runtime) {
            return Ok(blocked_result(reason));
        }
    }
    let root = app.state::<SharedDataRoot>();
    // Same lock order as preference writes: filesystem transaction, status.
    let _transaction = crate::modules::backup::manager::acquire_preferences_transaction(&root.0)?;
    crate::modules::runtime::protection::ensure_binding_ready(&root.0)?;
    let status = app.state::<crate::commands::update::SharedUpdateStatus>();
    let status = status.lock().map_err(|_| AppError::NotConfigured)?;
    if status.installing || status.pending_restart.is_some() {
        return Ok(blocked_running());
    }
    let context = app.state::<SharedProcessContext>();
    let anchor = app.state::<SharedDataRootAnchor>();
    let instances = app.state::<crate::state::InstanceState>();
    let locks = instances
        .locks
        .lock()
        .map_err(|_| AppError::NotConfigured)?;
    require_current_locks(&locks, &anchor.0, &root.0)?;
    let mut result = change(&mut binding)?;
    finish_binding_change(&mut result, &root.0, &context.opencodex_home, &mut binding);
    Ok(result)
}

pub(crate) fn require_current_locks(
    locks: &[crate::modules::instance::AppInstanceLock],
    anchor: &std::path::Path,
    current: &std::path::Path,
) -> AppResult<()> {
    for required in [
        anchor.join("manager-startup.lock"),
        current.join("manager-state/app.lock"),
    ] {
        if !locks.iter().any(|held| {
            crate::modules::data_root::same_runtime_directory(held.lock_path(), &required)
        }) {
            return Err(AppError::NotConfigured);
        }
    }
    Ok(())
}

fn finish_binding_change(
    result: &mut DataRootSwitchResult,
    root: &std::path::Path,
    home: &std::path::Path,
    binding: &mut Option<crate::infrastructure::storage_writers::BindingSave>,
) {
    if let Some(config) = result.config.as_mut() {
        config.current_data_root = root.display().to_string();
        config.current_opencodex_home = home.display().to_string();
        config.runtime_active = crate::modules::data_root::same_runtime_directory(
            std::path::Path::new(&config.active_data_root),
            root,
        ) && crate::modules::data_root::same_runtime_directory(
            std::path::Path::new(&config.opencodex_home),
            home,
        );
        if config.runtime_active && !result.reconciliation_required {
            result.status = DataRootSwitchStatus::Applied;
        }
        if !config.runtime_active || result.reconciliation_required {
            if let Some(binding) = binding.take() {
                binding.commit();
            }
        }
    } else if result.reconciliation_required {
        if let Some(binding) = binding.take() {
            binding.commit();
        }
    }
}

fn binding_runtime_is_idle(runtime: crate::types::status::RuntimeState) -> bool {
    matches!(
        runtime,
        crate::types::status::RuntimeState::Stopped | crate::types::status::RuntimeState::NotFound
    )
}

fn binding_runtime_blocked(
    verified: bool,
    runtime: crate::types::status::RuntimeState,
) -> Option<DataRootSwitchBlocked> {
    if !verified {
        Some(DataRootSwitchBlocked::RuntimeUnverified)
    } else if !binding_runtime_is_idle(runtime) {
        Some(DataRootSwitchBlocked::Running)
    } else {
        None
    }
}

/// Shared by GUI and IPC after acquiring their writer/runtime leases.
pub(crate) fn validate_reference_target(
    target: &std::path::Path,
    boundary: Option<&std::path::Path>,
    install: &std::path::Path,
) -> AppResult<()> {
    use crate::modules::data_root::layout::{Layout, Platform};
    let desktop = target.join("desktop");
    for path in [target, desktop.as_path()] {
        crate::modules::test_sandbox::check_boundary(boundary, path)
            .map_err(|_| AppError::NotConfigured)?;
        Layout::with_aggregate_root(path, install, Platform::Unix)?;
    }
    if let Ok(config) = crate::modules::data_root::load_runtime_config(&desktop) {
        let home = resolve_opencodex_home(&config);
        crate::modules::test_sandbox::check_boundary(boundary, &home)
            .map_err(|_| AppError::NotConfigured)?;
        Layout::with_aggregate_root(&home, install, Platform::Unix)?;
    }
    Ok(())
}

fn check_installation_separation(app: &tauri::AppHandle, root: &std::path::Path) -> AppResult<()> {
    let runtime = app.state::<std::sync::Arc<crate::modules::runtime::RuntimeHandle>>();
    crate::modules::data_root::layout::Layout::with_aggregate_root(
        root,
        runtime.installation_root(),
        crate::modules::data_root::layout::Platform::Unix,
    )?;
    Ok(())
}

fn blocked_running() -> DataRootSwitchResult {
    blocked_result(DataRootSwitchBlocked::Running)
}

fn blocked_result(reason: DataRootSwitchBlocked) -> DataRootSwitchResult {
    DataRootSwitchResult {
        status: DataRootSwitchStatus::Blocked,
        blocked: Some(reason),
        config: None,
        reconciliation_required: false,
    }
}

fn apply_config_or_blocked(
    result: Result<DataRootRuntimeConfig, crate::errors::AppError>,
    blocked: DataRootSwitchBlocked,
    before: &DataRootRuntimeConfig,
    readback: impl FnOnce() -> AppResult<DataRootRuntimeConfig>,
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
        Err(_) => match readback() {
            Ok(config) if config == *before => Ok(blocked_result(blocked)),
            observed => Ok(DataRootSwitchResult {
                status: DataRootSwitchStatus::RestartRequired,
                blocked: None,
                config: observed.ok().map(|config| {
                    project_config(
                        &config,
                        &before.active_data_root,
                        &resolve_opencodex_home(before),
                    )
                }),
                reconciliation_required: true,
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::storage_writers::WriterGate;
    use crate::modules::data_root::{self, startup_v2};
    use crate::types::status::RuntimeState;
    use std::{fs, sync::Arc};

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
            assert_eq!(
                binding_runtime_blocked(true, runtime),
                Some(DataRootSwitchBlocked::Running)
            );
        }
        for runtime in [RuntimeState::Stopped, RuntimeState::NotFound] {
            assert!(binding_runtime_blocked(true, runtime).is_none());
            assert_eq!(
                binding_runtime_blocked(false, runtime),
                Some(DataRootSwitchBlocked::RuntimeUnverified)
            );
        }
    }

    #[test]
    fn reference_switch_uses_target_metadata_and_fixed_pointer_across_restart() {
        let t = tempfile::tempdir().unwrap();
        let startup = startup_v2::resolve(&t.path().join("user"), None).unwrap();
        let target = t.path().join("selected");
        let target_d = target.join("desktop");
        initialize_module(&target_d).unwrap();
        let external = t.path().join("target-home");
        set_opencodex_home(&target_d, OpenCodexHomeMode::External, Some(&external)).unwrap();
        fs::write(
            startup
                .runtime_config
                .active_data_root
                .join("manager-state/source-only"),
            b"keep",
        )
        .unwrap();
        let bytes = fs::read(data_root::runtime_config_path(&target_d)).unwrap();
        let result = switch_data_root_with_paths(
            &startup.anchor,
            &target,
            DataRootSwitchMode::ReferenceOnly,
        )
        .unwrap();
        let config = result.config.unwrap();
        assert_eq!(result.status, DataRootSwitchStatus::RestartRequired);
        assert!(!config.runtime_active);
        assert!(data_root::same_runtime_directory(
            std::path::Path::new(&config.active_data_root),
            &target_d
        ));
        assert_eq!(
            fs::read(data_root::runtime_config_path(&target_d)).unwrap(),
            bytes
        );
        assert!(!target_d.join("manager-state/source-only").exists());
        assert!(!target_d.join("manager-location.json").exists());
        let runtime_home = resolve_opencodex_home(&startup.runtime_config);
        let saved = data_root_config_with_paths(
            &startup.anchor,
            &startup.runtime_config.active_data_root,
            &runtime_home,
        )
        .unwrap();
        assert!(!saved.runtime_active);
        assert!(data_root::same_runtime_directory(
            std::path::Path::new(&saved.opencodex_home),
            &external
        ));
        let anchor = startup.anchor.clone();
        drop(startup);
        let restart = startup_v2::resolve(&t.path().join("user"), None).unwrap();
        assert_eq!(restart.anchor, anchor);
        assert!(
            data_root_config_with_paths(
                &anchor,
                &restart.runtime_config.active_data_root,
                &resolve_opencodex_home(&restart.runtime_config)
            )
            .unwrap()
            .runtime_active
        );
    }

    #[test]
    fn home_save_edits_selected_root_and_latches_current_writers_until_restart() {
        let t = tempfile::tempdir().unwrap();
        let startup = startup_v2::resolve(&t.path().join("user"), None).unwrap();
        let target = t.path().join("selected");
        initialize_module(&target.join("desktop")).unwrap();
        switch_data_root_with_paths(&startup.anchor, &target, DataRootSwitchMode::ReferenceOnly)
            .unwrap();
        let fixed_bytes = fs::read(data_root::runtime_config_path(&startup.anchor)).unwrap();
        let external = t.path().join("external");
        let mut result = set_opencodex_home_with_paths(
            &startup.anchor,
            OpenCodexHomeMode::External,
            Some(&external),
        )
        .unwrap();
        assert_eq!(
            fs::read(data_root::runtime_config_path(&startup.anchor)).unwrap(),
            fixed_bytes
        );
        let gate = Arc::new(WriterGate::default());
        let mut binding = gate.freeze().unwrap();
        let current_home = resolve_opencodex_home(&startup.runtime_config);
        finish_binding_change(
            &mut result,
            &startup.runtime_config.active_data_root,
            &current_home,
            &mut binding,
        );
        assert!(binding.is_none());
        assert!(gate.admit().is_err());
        let config = result.config.unwrap();
        assert!(!config.runtime_active);
        assert_eq!(
            config.current_data_root,
            startup
                .runtime_config
                .active_data_root
                .display()
                .to_string()
        );
        assert_eq!(
            config.current_opencodex_home,
            current_home.display().to_string()
        );
        let wire = serde_json::to_value(config).unwrap();
        assert!(wire.get("aggregateRoot").is_some());
        assert!(!target.join("desktop/manager-location.json").exists());
    }

    #[test]
    fn migration_request_is_rejected_without_copy_or_pointer_update() {
        let t = tempfile::tempdir().unwrap();
        let startup = startup_v2::resolve(&t.path().join("user"), None).unwrap();
        let pointer = startup.anchor.join("manager-location.json");
        let before = fs::read(&pointer).unwrap();
        let target = t.path().join("uncreated");
        let result =
            switch_data_root_with_paths(&startup.anchor, &target, DataRootSwitchMode::MigrateData)
                .unwrap();
        assert_eq!(result.status, DataRootSwitchStatus::Blocked);
        assert_eq!(result.blocked, Some(DataRootSwitchBlocked::Unsupported));
        assert_eq!(fs::read(pointer).unwrap(), before);
        assert!(!target.exists());
    }

    #[test]
    fn unchanged_save_releases_gate_but_uncertain_save_latches_it() {
        let t = tempfile::tempdir().unwrap();
        let startup = startup_v2::resolve(&t.path().join("user"), None).unwrap();
        let root = &startup.runtime_config.active_data_root;
        let home = resolve_opencodex_home(&startup.runtime_config);
        for uncertain in [false, true] {
            let gate = Arc::new(WriterGate::default());
            let mut binding = gate.freeze().unwrap();
            let mut result = DataRootSwitchResult {
                status: DataRootSwitchStatus::RestartRequired,
                blocked: None,
                config: Some(project_config(&startup.runtime_config, root, &home)),
                reconciliation_required: uncertain,
            };
            finish_binding_change(&mut result, root, &home, &mut binding);
            assert_eq!(
                result.status,
                if uncertain {
                    DataRootSwitchStatus::RestartRequired
                } else {
                    DataRootSwitchStatus::Applied
                }
            );
            drop(binding);
            assert_eq!(gate.admit().is_err(), uncertain);
        }
    }

    #[cfg(unix)]
    #[test]
    fn startup_aliases_preserve_runtime_identity_without_rewriting_metadata() {
        let t = tempfile::tempdir().unwrap();
        let startup = startup_v2::resolve(&t.path().join("user"), None).unwrap();
        let alias = t.path().join("alias");
        std::os::unix::fs::symlink(&startup.anchor, &alias).unwrap();
        let before = fs::read(data_root::runtime_config_path(&startup.anchor)).unwrap();
        let home = resolve_opencodex_home(&startup.runtime_config);
        assert!(
            data_root_config_with_paths(&alias, &startup.runtime_config.active_data_root, &home)
                .unwrap()
                .runtime_active
        );
        assert_eq!(
            fs::read(data_root::runtime_config_path(&startup.anchor)).unwrap(),
            before
        );
    }

    #[test]
    fn binding_changes_require_both_current_service_and_fixed_startup_locks() {
        let t = tempfile::tempdir().unwrap();
        let mut startup = startup_v2::resolve(&t.path().join("user"), None).unwrap();
        require_current_locks(
            &startup.locks,
            &startup.anchor,
            &startup.runtime_config.active_data_root,
        )
        .unwrap();
        let root_lock = startup.locks.pop().unwrap();
        assert!(require_current_locks(
            &startup.locks,
            &startup.anchor,
            &startup.runtime_config.active_data_root
        )
        .is_err());
        assert!(require_current_locks(
            &[root_lock],
            &startup.anchor,
            &startup.runtime_config.active_data_root
        )
        .is_err());
    }

    #[test]
    fn initialize_command_uses_aggregate_semantics_and_refuses_busy_or_nested_targets() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().join("data");
        let install = t.path().join("program");
        assert!(initialize_with_paths(&root, None, &install).unwrap());
        assert!(root.join("desktop/manager-state/data-root.json").is_file());
        assert!(!root.join("manager-state").exists());
        let _busy =
            crate::modules::instance::AppInstanceLock::acquire(&root.join("desktop")).unwrap();
        fs::remove_dir(root.join("desktop/cache")).unwrap();
        assert!(matches!(
            initialize_with_paths(&root, None, &install),
            Err(AppError::InstanceLockConflict)
        ));
        assert!(!root.join("desktop/cache").exists());
        let invalid = install.join("data");
        assert!(initialize_with_paths(&invalid, None, &install).is_err());
        assert!(!invalid.exists());
    }

    #[cfg(unix)]
    #[test]
    fn sandbox_initialization_cannot_follow_a_desktop_link_outside_boundary() {
        let t = tempfile::tempdir().unwrap();
        let sandbox = t.path().canonicalize().unwrap().join("sandbox");
        let outside = t.path().join("outside");
        fs::create_dir(&sandbox).unwrap();
        fs::create_dir(&outside).unwrap();
        let root = sandbox.join("data");
        fs::create_dir(&root).unwrap();
        std::os::unix::fs::symlink(&outside, root.join("desktop")).unwrap();
        assert!(initialize_with_paths(&root, Some(&sandbox), &sandbox.join("program")).is_err());
        assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
    }

    #[test]
    fn uncertain_save_readback_never_reopens_old_writers() {
        let t = tempfile::tempdir().unwrap();
        let startup = startup_v2::resolve(&t.path().join("user"), None).unwrap();
        let before = &startup.runtime_config;
        for readable in [false, true] {
            let gate = Arc::new(WriterGate::default());
            let mut binding = gate.freeze().unwrap();
            let mut changed = before.clone();
            changed.opencodex_home_mode = OpenCodexHomeMode::External;
            changed.opencodex_home_path = Some(t.path().join("external"));
            let mut result = apply_config_or_blocked(
                Err(AppError::NotConfigured),
                DataRootSwitchBlocked::Corrupted,
                before,
                || {
                    if readable {
                        Ok(changed)
                    } else {
                        Err(AppError::NotConfigured)
                    }
                },
            )
            .unwrap();
            assert!(result.reconciliation_required);
            finish_binding_change(
                &mut result,
                &before.active_data_root,
                &resolve_opencodex_home(before),
                &mut binding,
            );
            assert!(binding.is_none());
            assert!(gate.admit().is_err());
        }
        let result = apply_config_or_blocked(
            Err(AppError::NotConfigured),
            DataRootSwitchBlocked::Corrupted,
            before,
            || Ok(before.clone()),
        )
        .unwrap();
        assert_eq!(result.status, DataRootSwitchStatus::Blocked);
        assert!(!result.reconciliation_required);
    }

    #[test]
    fn selected_home_cannot_overlap_fixed_manager_state() {
        let t = tempfile::tempdir().unwrap();
        let startup = startup_v2::resolve(&t.path().join("user"), None).unwrap();
        let target = t.path().join("selected");
        initialize_module(&target.join("desktop")).unwrap();
        switch_data_root_with_paths(&startup.anchor, &target, DataRootSwitchMode::ReferenceOnly)
            .unwrap();
        let before = fs::read(data_root::runtime_config_path(&target.join("desktop"))).unwrap();
        let result = set_opencodex_home_with_paths(
            &startup.anchor,
            OpenCodexHomeMode::External,
            Some(&startup.anchor.join("home")),
        )
        .unwrap();
        assert_eq!(result.blocked, Some(DataRootSwitchBlocked::Nested));
        assert_eq!(
            fs::read(data_root::runtime_config_path(&target.join("desktop"))).unwrap(),
            before
        );
    }

    #[test]
    fn future_and_nested_selection_are_typed_blocks_without_pointer_changes() {
        let t = tempfile::tempdir().unwrap();
        let startup = startup_v2::resolve(&t.path().join("user"), None).unwrap();
        let pointer = startup.anchor.join("manager-location.json");
        let before = fs::read(&pointer).unwrap();
        let future = t.path().join("future");
        initialize_module(&future.join("desktop")).unwrap();
        fs::write(
            future.join("desktop/manager-state/data-root.json"),
            br#"{"structure_version":"3","opencodex_home_mode":"inside"}"#,
        )
        .unwrap();
        assert_eq!(
            switch_data_root_with_paths(
                &startup.anchor,
                &future,
                DataRootSwitchMode::ReferenceOnly
            )
            .unwrap()
            .blocked,
            Some(DataRootSwitchBlocked::FutureVersion)
        );
        let nested = startup.anchor.parent().unwrap().join("nested");
        assert_eq!(
            switch_data_root_with_paths(
                &startup.anchor,
                &nested,
                DataRootSwitchMode::ReferenceOnly
            )
            .unwrap()
            .blocked,
            Some(DataRootSwitchBlocked::Nested)
        );
        assert_eq!(fs::read(pointer).unwrap(), before);
        assert!(!nested.exists());
    }

    #[test]
    fn unsupported_migration_does_not_require_or_probe_a_legacy_anchor() {
        let t = tempfile::tempdir().unwrap();
        let source = t.path().join("not-created/source");
        let target = t.path().join("not-created/target");
        let result =
            switch_data_root_with_paths(&source, &target, DataRootSwitchMode::MigrateData).unwrap();
        assert_eq!(result.blocked, Some(DataRootSwitchBlocked::Unsupported));
        assert_eq!(fs::read_dir(t.path()).unwrap().count(), 0);
    }
}
