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
    ready: impl FnOnce() + Send + 'static,
) -> AppResult<()> {
    if runtime.uses_versioned_programs() {
        super::program_reinstall::require_idle(&installation)?;
        super::program_repair::require_idle(&installation)?;
    }
    if runtime.uses_versioned_programs()
        && super::program_removal::intentionally_removed(&installation, &desktop)?
    {
        ready();
        return Ok(());
    }
    if runtime.uses_versioned_programs() && matches!(runtime.selected_program(), Ok(Some(_))) {
        // Initial source admission already verified this image. A normal
        // startup does not rewrite or re-unpack a readable installed program.
        ready();
        return Ok(());
    }
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
            if runtime.uses_versioned_programs() {
                super::bundled::versioned::provision(
                    &installation,
                    &desktop,
                    &resources.join(name),
                    &descriptor,
                    descriptor_pin,
                )?;
            } else {
                super::bundled::provision(
                    &installation,
                    &desktop,
                    &resources.join(name),
                    &descriptor,
                    descriptor_pin,
                )?;
            }
            runtime.refresh();
            ready();
            Ok(())
        })();
        let message = match &result {
            Ok(()) => "integrated runtime provisioned and verified".to_string(),
            // This worker reads only the pinned local distribution and managed
            // installation metadata. Preserve its actual bounded failure reason
            // so first-start failures can be diagnosed without touching secrets.
            Err(error) => format!(
                "integrated runtime provisioning failed; installation retained for recovery: {error}"
            ),
        };
        let _ =
            crate::infrastructure::runtime_log::RuntimeLog::new(&desktop).append_event(&message);
    });
    Ok(())
}

#[cfg(all(test, target_os = "macos", target_arch = "aarch64"))]
mod tests {
    use super::*;
    #[test]
    #[ignore = "requires the explicitly pinned complete platform bundle"]
    fn real_first_start_worker_holds_install_lease_and_announces_refreshed_runtime() {
        let archive = PathBuf::from(
            std::env::var_os("OCXD_TEST_BUNDLE_ARCHIVE").expect("explicit bundle archive"),
        );
        let descriptor = PathBuf::from(
            std::env::var_os("OCXD_TEST_BUNDLE_DESCRIPTOR").expect("explicit descriptor"),
        );
        assert!(archive.is_absolute() && descriptor.is_absolute());
        let root = tempfile::Builder::new()
            .prefix("ocxd023-startup-bundle-real-")
            .tempdir()
            .unwrap();
        let canonical_root = root.path().canonicalize().unwrap();
        let resources = canonical_root.join("resources");
        let installation = canonical_root.join("program I");
        let desktop = canonical_root.join("user U/desktop");
        let home = canonical_root.join("user U/opencodex/home");
        for dir in [&resources, &installation, &desktop, &home] {
            std::fs::create_dir_all(dir).unwrap();
        }
        let bytes = std::fs::read(descriptor).unwrap();
        let pin = crate::infrastructure::hash::sha256_hex(&bytes);
        assert_eq!(
            pin,
            "88a96e98e3070baa05c7d2ccb402ba70f491f25ca4a93505f8927b8eb0b8b152"
        );
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let name = value["archiveName"].as_str().unwrap();
        std::fs::copy(archive, resources.join(name)).unwrap();
        std::fs::write(resources.join("runtime-bundle.manifest.json"), bytes).unwrap();
        let config = home.join("config.json");
        std::fs::write(&config, b"{\"providers\":{},\"preserve\":true}").unwrap();
        let before = std::fs::read(&config).unwrap();
        let runtime = RuntimeHandle::initialize_managed(&desktop, &installation);
        let install = SharedRuntimeInstall::new();
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        start(
            installation.clone(),
            desktop,
            resources,
            "88a96e98e3070baa05c7d2ccb402ba70f491f25ca4a93505f8927b8eb0b8b152",
            runtime.clone(),
            install.clone(),
            move || {
                send.send(()).unwrap();
            },
        )
        .unwrap();
        assert!(
            install.acquire().is_none(),
            "worker must already own installation before scheduling"
        );
        receive
            .recv_timeout(std::time::Duration::from_secs(600))
            .expect("actual provision worker callback");
        assert!(
            crate::modules::runtime::install::validate_managed_installation(
                &installation,
                Some(&installation.join("runtime/node/bin/node")),
            )
            .is_ok()
        );
        assert_eq!(std::fs::read(config).unwrap(), before);
        assert!(installation.join("runtime/bin/ocx").is_file());
        assert_eq!(
            runtime.executable(),
            Some(installation.join("runtime/bin/ocx"))
        );
        assert_eq!(
            runtime.kind(),
            crate::modules::runtime::RuntimeSourceKind::Managed
        );
    }
}
