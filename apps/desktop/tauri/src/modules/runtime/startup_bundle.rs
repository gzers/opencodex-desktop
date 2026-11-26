//! First-start provisioning on a worker, sharing the installer/storage
//! boundary. Admission is acquired before scheduling so startup reconciliation
//! cannot race it and silently prevent the first install.
use super::RuntimeHandle;
use crate::{
    errors::{AppError, AppResult},
    state::SharedRuntimeInstall,
};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::Arc,
};

#[path = "bundle_download.rs"]
mod bundle_download;

pub(crate) struct Retry {
    expected: String,
    transfer: Option<bundle_download::Transfer>,
}

impl Retry {
    pub(crate) fn new(expected: String, confirmed: bool, app: tauri::AppHandle) -> Self {
        Self {
            expected,
            transfer: confirmed.then(|| bundle_download::github(app)),
        }
    }
}

pub(crate) fn start(
    installation: PathBuf,
    desktop: PathBuf,
    resources: PathBuf,
    descriptor_pin: &'static str,
    runtime: Arc<RuntimeHandle>,
    install: SharedRuntimeInstall,
    ready: impl FnOnce() + Send + 'static,
) -> AppResult<()> {
    launch(
        installation,
        desktop,
        resources,
        descriptor_pin,
        runtime,
        install,
        (None, ready),
    )
}
/// Explicit retry is bound to the last observed failed attempt; a second click
/// cannot join another install or silently turn repair/removal into first supply.
pub(crate) fn retry(
    desktop: PathBuf,
    resources: PathBuf,
    pin: &'static str,
    runtime: Arc<RuntimeHandle>,
    install: SharedRuntimeInstall,
    request: Retry,
    ready: impl FnOnce() + Send + 'static,
) -> AppResult<()> {
    launch(
        runtime.installation_root().into(),
        desktop,
        resources,
        pin,
        runtime,
        install,
        (Some(request), ready),
    )
}
fn retryable(i: &std::path::Path) -> bool {
    let runtime = i.join("runtime");
    for name in [
        "current.json",
        ".selection-pending.json",
        super::program_removal::PENDING,
        super::program_removal::REMOVED,
        super::program_reinstall::PENDING,
        super::program_repair::PENDING,
    ] {
        if !matches!(std::fs::symlink_metadata(runtime.join(name)),Err(e) if e.kind()==std::io::ErrorKind::NotFound)
        {
            return false;
        }
    }
    match std::fs::read_dir(runtime.join("versions")) {
        Ok(mut rows) => rows.next().is_none(),
        Err(e) => e.kind() == std::io::ErrorKind::NotFound,
    }
}
fn category(e: &AppError) -> super::preparation::Failure {
    use super::preparation::Failure;
    match e {
        AppError::RuntimeManaged { code, .. }
            if code.contains("pending") || code.contains("unconfirmed") =>
        {
            Failure::RecoveryRequired
        }
        AppError::RuntimeManaged { code, .. } if code == "bundle_archive_missing" => {
            Failure::MissingArchive
        }
        AppError::RuntimeManaged { code, .. }
            if code.contains("invalid")
                || matches!(
                    code.as_str(),
                    "bundle_descriptor_digest" | "bundle_archive_digest"
                ) =>
        {
            Failure::Integrity
        }
        AppError::RuntimeManaged { code, .. } if code == "install_in_progress" => Failure::Busy,
        _ => Failure::Unavailable,
    }
}

fn supply(
    installation: &Path,
    desktop: &Path,
    resources: &Path,
    pin: &str,
    transfer: Option<bundle_download::Transfer>,
    observe: &dyn Fn(super::preparation::Phase),
) -> AppResult<()> {
    use super::{bundled, preparation::Phase};

    let (descriptor, asset) = bundled::startup_asset(installation, resources, pin)?;
    let archive = resources.join(&asset.name);
    let provision = |archive: &Path| {
        bundled::versioned::provision_observed(
            installation,
            desktop,
            archive,
            &descriptor,
            pin,
            observe,
        )?;
        Ok(())
    };
    match fs::symlink_metadata(&archive) {
        Ok(_) => {
            bundled::check_startup_archive(resources, &archive)?;
            provision(&archive)
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            let transfer = transfer.ok_or_else(|| AppError::RuntimeManaged {
                code: "bundle_archive_missing".into(),
                detail: "固定运行资源归档缺失，需要确认后下载".into(),
            })?;
            bundled::roots(installation, desktop)?;
            observe(Phase::Downloading);
            bundle_download::with_verified_archive(desktop, &asset, transfer, provision)
        }
        Err(e) => Err(AppError::RuntimeManaged {
            code: "bundle_archive_unavailable".into(),
            detail: format!("无法检查本地运行归档：{e}"),
        }),
    }
}

fn launch(
    installation: PathBuf,
    desktop: PathBuf,
    resources: PathBuf,
    descriptor_pin: &'static str,
    runtime: Arc<RuntimeHandle>,
    install: SharedRuntimeInstall,
    control: (Option<Retry>, impl FnOnce() + Send + 'static),
) -> AppResult<()> {
    use super::preparation::Phase;
    let (request, ready) = control;
    let (expected, transfer) = request.map_or((None, None), |request| {
        (Some(request.expected), request.transfer)
    });
    let id = runtime
        .preparation
        .begin(expected.as_deref(), transfer.is_some())?;
    let admission = (|| -> AppResult<Option<_>> {
        super::program_reinstall::require_idle(&installation)?;
        super::program_repair::require_idle(&installation)?;
        if super::program_removal::intentionally_removed(&installation, &desktop)? {
            runtime
                .preparation
                .finish(&id, Phase::Removed, None, false)?;
            return Ok(None);
        }
        if runtime.selected_program()?.is_some() {
            runtime
                .preparation
                .finish(&id, Phase::Supplied, None, false)?;
            return Ok(None);
        }
        if !retryable(&installation) {
            return Err(AppError::RuntimeManaged {
                code: "preparation_recovery_required".into(),
                detail: "已有程序事务或版本需要核对，未重新供应".into(),
            });
        }
        let storage = crate::infrastructure::storage_writers::global().admit()?;
        let lease = install.acquire().ok_or(AppError::RuntimeManaged {
            code: "install_in_progress".into(),
            detail: "已有运行程序操作正在进行".into(),
        })?;
        Ok(Some((storage, lease)))
    })();
    let (storage, lease) = match admission {
        Ok(Some(value)) => value,
        Ok(None) => {
            ready();
            return Ok(());
        }
        Err(e) => {
            let _ = runtime
                .preparation
                .finish(&id, Phase::Review, Some(category(&e)), false);
            return Err(e);
        }
    };
    tauri::async_runtime::spawn_blocking(move || {
        let _storage = storage;
        let _lease = lease;
        let result = (|| -> AppResult<()> {
            runtime.preparation.stage(&id, Phase::Preparing)?;
            if super::protection::PanelProtection::load(&desktop)?.is_some() {
                let guard =
                    crate::modules::backup::manager::acquire_preferences_transaction(&desktop)?;
                super::protection::reconcile_pending(&desktop, &guard)?;
            }
            supply(
                &installation,
                &desktop,
                &resources,
                descriptor_pin,
                transfer,
                &|phase| {
                    if phase != Phase::Preparing {
                        let _ = runtime.preparation.stage(&id, phase);
                    }
                },
            )?;
            runtime.observe_current();
            if runtime.selected_program()?.is_none() {
                return Err(AppError::NotConfigured);
            }
            Ok(())
        })();
        let message = match &result {
            Ok(()) => "integrated runtime provisioned and verified".to_string(),
            Err(e) => format!("integrated runtime provisioning failed; original retained: {e}"),
        };
        let _ =
            crate::infrastructure::runtime_log::RuntimeLog::new(&desktop).append_event(&message);
        // A terminal observation must not invite a retry while this worker still
        // owns installation. Drop actual leases before exposing retryable state.
        drop(_lease);
        drop(_storage);
        match result {
            Ok(()) => {
                let _ = runtime
                    .preparation
                    .finish(&id, Phase::Supplied, None, false);
                ready();
            }
            Err(e) => {
                let can_retry = retryable(&installation);
                let _ = runtime.preparation.finish(
                    &id,
                    if can_retry {
                        Phase::Failed
                    } else {
                        Phase::Review
                    },
                    Some(category(&e)),
                    can_retry,
                );
            }
        }
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
            "e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3"
        );
        let value: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let name = value["archiveName"].as_str().unwrap();
        std::fs::copy(archive, resources.join(name)).unwrap();
        std::fs::write(resources.join("runtime-bundle.manifest.json"), bytes).unwrap();
        let config = home.join("config.json");
        std::fs::write(&config, b"{\"providers\":{},\"preserve\":true}").unwrap();
        let before = std::fs::read(&config).unwrap();
        let runtime = RuntimeHandle::initialize_versioned(&desktop, &installation);
        let install = SharedRuntimeInstall::new();
        let (send, receive) = std::sync::mpsc::sync_channel(1);
        start(
            installation.clone(),
            desktop,
            resources,
            "e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3",
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
        assert!(runtime.selected_program().unwrap().is_some());
        assert_eq!(std::fs::read(config).unwrap(), before);
        assert!(installation.join("runtime/bin/ocx-runtime").is_file());
        assert_eq!(
            runtime.executable(),
            Some(installation.join("runtime/bin/ocx-runtime"))
        );
        assert_eq!(
            runtime.kind(),
            crate::modules::runtime::RuntimeSourceKind::Managed
        );
        assert_eq!(
            runtime.preparation.snapshot().unwrap().phase,
            crate::modules::runtime::preparation::Phase::Supplied
        );
        assert!(!runtime.preparation.snapshot().unwrap().retryable);
        assert!(
            install.acquire().is_some(),
            "completed status releases the actual installation lease"
        );
        println!(
            "REAL_PREPARATION {}",
            serde_json::json!({"version":"2.99.0","phase":"supplied","configurationPreserved":true,"installationLeaseReleased":true,"serviceStarted":false})
        );
    }
}

#[cfg(all(test, target_os = "macos", target_arch = "aarch64"))]
#[path = "startup_bundle_state_tests.rs"]
mod state_tests;
