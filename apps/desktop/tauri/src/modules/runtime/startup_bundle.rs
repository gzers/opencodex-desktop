//! Offline first-start provisioning on a worker, sharing the installer/storage
//! boundary. Admission is acquired before scheduling so startup reconciliation
//! cannot race it and silently prevent the first install.
use super::RuntimeHandle;
use crate::{
    errors::{AppError, AppResult},
    state::SharedRuntimeInstall,
};
use std::{path::PathBuf, sync::Arc};

pub(crate) fn start(
    installation: PathBuf,
    desktop: PathBuf,
    resources: PathBuf,
    descriptor_pin: &'static str,
    runtime: Arc<RuntimeHandle>,
    install: SharedRuntimeInstall,
) -> AppResult<()> {
    let storage = crate::infrastructure::storage_writers::global().admit()?;
    let lease = install.acquire().ok_or(AppError::RuntimeManaged {
        code: "install_in_progress".into(),
        detail: "已有运行程序操作正在进行，未开始集成供应".into(),
    })?;
    tauri::async_runtime::spawn_blocking(move || {
        let _storage = storage;
        let _lease = lease;
        let result = (|| -> AppResult<()> {
            if super::protection::PanelProtection::load(&desktop)?.is_some() {
                let guard =
                    crate::modules::backup::manager::acquire_preferences_transaction(&desktop)?;
                super::protection::reconcile_pending(&desktop, &guard)?;
            }
            let path = resources.join("runtime-bundle.manifest.json");
            crate::modules::backup::safety::check_path(&resources, &path, false)?;
            let descriptor = crate::modules::backup::safety::read_file(&path, 64 * 1024 * 1024)?;
            let value: serde_json::Value =
                serde_json::from_slice(&descriptor).map_err(|_| AppError::NotConfigured)?;
            let name = value["archiveName"]
                .as_str()
                .filter(|s| {
                    !s.is_empty() && *s != "." && *s != ".." && !s.contains(['/', '\\', ':'])
                })
                .ok_or(AppError::NotConfigured)?;
            super::bundled::provision(
                &installation,
                &desktop,
                &resources.join(name),
                &descriptor,
                descriptor_pin,
            )?;
            runtime.refresh();
            Ok(())
        })();
        let message = match &result {
            Ok(()) => "integrated runtime provisioned and verified",
            Err(_) => "integrated runtime provisioning failed; installation retained for recovery",
        };
        let _ = crate::infrastructure::runtime_log::RuntimeLog::new(&desktop).append_event(message);
    });
    Ok(())
}
