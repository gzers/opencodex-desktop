//! User-selected native config import; authorization comes from the actual
//! application context and an opaque, single-use selection, not DTO paths.
use super::{
    runtime_recovery_v2::{ServiceOwnerProof, ServiceOwnerProofProvider},
    runtime_recovery_v2_owner::SystemServiceOwnerProofProvider,
};
use crate::{
    errors::{AppError, AppResult},
    modules::{
        model_config::{
            config_file::v2::native_import,
            schema_v2::{self, Result},
        },
        runtime::{program_selection::SelectedProgram, RuntimeHandle},
    },
    state::{SharedDataRoot, SharedProcessContext, SharedRuntimeInstall},
};
use serde::Serialize;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::io::AsyncReadExt;
use zeroize::Zeroizing;

#[derive(Default)]
pub struct Selections(Mutex<Option<Prepared>>);

struct Prepared {
    id: String,
    selected_at: Instant,
    runtime: SelectedProgram,
    selection: native_import::Selection,
}

fn remember(
    selections: &Selections,
    runtime: SelectedProgram,
    selection: native_import::Selection,
) -> Result<String> {
    let id = uuid::Uuid::new_v4().to_string();
    *selections
        .0
        .lock()
        .map_err(|_| schema_v2::error("native_import_stale"))? = Some(Prepared {
        id: id.clone(),
        selected_at: Instant::now(),
        runtime,
        selection,
    });
    Ok(id)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackSelection {
    pub id: String,
    pub preview: native_import::RollbackPreview,
}

#[tauri::command]
pub async fn model_v2_native_rollback_list(
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
) -> Result<Vec<native_import::BackupSummary>> {
    let data = data.0.clone();
    let home = context.opencodex_home.clone();
    tauri::async_runtime::spawn_blocking(move || native_import::list_backups(&data, &home))
        .await
        .map_err(|_| schema_v2::error("model_storage"))?
}

#[tauri::command]
pub async fn model_v2_native_rollback_select(
    backup_id: String,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    selections: tauri::State<'_, Arc<Selections>>,
) -> Result<RollbackSelection> {
    let data = data.0.clone();
    let home = context.opencodex_home.clone();
    let runtime = runtime.inner().clone();
    let selections = selections.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let runtime = program(&runtime)?;
        let (selection, preview) =
            native_import::prepare_rollback(&data, &home, &backup_id, |bytes| {
                validate(&runtime, bytes)
            })?;
        Ok(RollbackSelection {
            id: remember(&selections, runtime, selection)?,
            preview,
        })
    })
    .await
    .map_err(|_| schema_v2::error("model_storage"))?
}

fn program(handle: &RuntimeHandle) -> Result<SelectedProgram> {
    handle.observe_current();
    handle
        .selected_program()
        .ok()
        .flatten()
        .filter(|p| p.selection.version == "2.99.0")
        .ok_or_else(|| schema_v2::error("native_import_runtime_unavailable"))
}

fn validate(program: &SelectedProgram, bytes: &[u8]) -> AppResult<()> {
    use std::{io::Write, process::Stdio};
    let scratch = tempfile::tempdir().map_err(|_| AppError::NotConfigured)?;
    let root = scratch.path();
    let input = root.join("selected.json");
    let mut file = crate::infrastructure::platform::create_new_private_file(&input, 0o600)
        .map_err(|_| AppError::NotConfigured)?;
    file.write_all(bytes).map_err(|_| AppError::NotConfigured)?;
    drop(file);
    tauri::async_runtime::block_on(async {
        let mut command = tokio::process::Command::new(&program.paths.node);
        command
            .arg(&program.script)
            .args(["config", "validate"])
            .arg(&input)
            .arg("--json")
            .env_clear()
            .env("PATH", "")
            .env("OCX_NO_DESKTOP_HANDOFF", "1")
            .current_dir(root)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        for name in [
            "HOME",
            "USERPROFILE",
            "OPENCODEX_HOME",
            "CODEX_HOME",
            "CLAUDE_CONFIG_DIR",
            "XDG_CONFIG_HOME",
            "XDG_CACHE_HOME",
            "TMPDIR",
            "TEMP",
        ] {
            command.env(name, root);
        }
        #[cfg(windows)]
        if let Some(dir) = std::env::var_os("SystemRoot") {
            command.env("SystemRoot", dir);
        }
        #[cfg(unix)]
        command.process_group(0);
        #[cfg(windows)]
        let job = crate::infrastructure::windows_query_job::OwnedQueryJob::new(&mut command)
            .map_err(|_| AppError::NotConfigured)?;
        let mut child = command.spawn().map_err(|_| AppError::NotConfigured)?;
        #[cfg(windows)]
        if job.assign_and_resume(&child).is_err() {
            let _ = child.kill().await;
            let _ = child.wait().await;
            return Err(AppError::NotConfigured);
        }
        #[cfg(unix)]
        let pid = child.id();
        let mut pipe = child
            .stdout
            .take()
            .ok_or(AppError::NotConfigured)?
            .take(16385);
        let mut output = Zeroizing::new(Vec::new());
        let observed = tokio::time::timeout(Duration::from_secs(10), async {
            let (status, _) = tokio::try_join!(child.wait(), pipe.read_to_end(&mut output))?;
            Ok::<_, std::io::Error>(status)
        })
        .await;
        #[cfg(windows)]
        let family_done = job.terminate_and_wait().await.is_ok();
        #[cfg(unix)]
        if let Some(pid) = pid {
            // Own process group only. No unrelated service or user process.
            unsafe {
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
        }
        let _ = child.kill().await;
        let _ = child.wait().await;
        #[cfg(windows)]
        if !family_done {
            return Err(AppError::NotConfigured);
        }
        let status = observed
            .map_err(|_| AppError::Timeout)?
            .map_err(|_| AppError::NotConfigured)?;
        if !status.success()
            || output.len() > 16384
            || serde_json::from_slice::<serde_json::Value>(&output)
                .ok()
                .and_then(|v| v["ok"].as_bool())
                != Some(true)
        {
            return Err(AppError::NotConfigured);
        }
        Ok(())
    })
}

#[tauri::command]
pub async fn model_v2_native_import_select(
    path: PathBuf,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    selections: tauri::State<'_, Arc<Selections>>,
) -> Result<String> {
    let data = data.0.clone();
    let home = context.opencodex_home.clone();
    let runtime = runtime.inner().clone();
    let selections = selections.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let runtime = program(&runtime)?;
        let selection =
            native_import::prepare(&data, &home, &path, |bytes| validate(&runtime, bytes))?;
        remember(&selections, runtime, selection)
    })
    .await
    .map_err(|_| schema_v2::error("model_storage"))?
}

#[tauri::command]
pub async fn model_v2_native_import_apply(
    id: String,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    selections: tauri::State<'_, Arc<Selections>>,
    lease: tauri::State<'_, SharedRuntimeInstall>,
) -> Result<native_import::Outcome> {
    apply_selection(id, false, data, context, runtime, selections, lease).await
}

#[tauri::command]
pub async fn model_v2_native_rollback_apply(
    id: String,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    selections: tauri::State<'_, Arc<Selections>>,
    lease: tauri::State<'_, SharedRuntimeInstall>,
) -> Result<native_import::Outcome> {
    apply_selection(id, true, data, context, runtime, selections, lease).await
}

async fn apply_selection(
    id: String,
    rollback: bool,
    data: tauri::State<'_, SharedDataRoot>,
    context: tauri::State<'_, SharedProcessContext>,
    runtime: tauri::State<'_, Arc<RuntimeHandle>>,
    selections: tauri::State<'_, Arc<Selections>>,
    lease: tauri::State<'_, SharedRuntimeInstall>,
) -> Result<native_import::Outcome> {
    let admission = crate::infrastructure::storage_writers::global()
        .admit()
        .map_err(|_| schema_v2::error("model_storage"))?;
    let lease = lease
        .acquire()
        .ok_or_else(|| schema_v2::error("native_import_runtime_not_stopped"))?;
    let runtime = runtime.inner().clone();
    let selections = selections.inner().clone();
    let data = data.0.clone();
    let context = context.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let (_lease, _admission) = (lease, admission);
        let prepared = {
            let mut slot = selections
                .0
                .lock()
                .map_err(|_| schema_v2::error("native_import_stale"))?;
            if slot.as_ref().is_none_or(|p| {
                p.id != id
                    || p.selection.is_rollback() != rollback
                    || p.selected_at.elapsed() > Duration::from_secs(600)
            }) {
                return Err(schema_v2::error("native_import_stale"));
            }
            slot.take().unwrap()
        };
        if program(&runtime)? != prepared.runtime {
            return Err(schema_v2::error("native_import_stale"));
        }
        let owner = SystemServiceOwnerProofProvider::new(
            runtime.installation_root().into(),
            context.clone(),
        )
        .map_err(|_| schema_v2::error("native_import_runtime_not_stopped"))?;
        native_import::apply(&data, &context.opencodex_home, prepared.selection, || {
            crate::modules::runtime::protection::ensure_binding_ready(&data)?;
            if owner.query(&context)? == ServiceOwnerProof::Stopped {
                Ok(())
            } else {
                Err(AppError::NotConfigured)
            }
        })
    })
    .await
    .map_err(|_| schema_v2::error("model_storage"))?
}

#[cfg(all(test, target_os = "macos", target_arch = "aarch64"))]
#[test]
#[ignore = "requires pinned, unchanged official 2.99 complete bundle in explicit environment"]
fn real_official299_validates_native_import_then_commits_synthetic_fixture() {
    use std::path::Path;
    let archive =
        PathBuf::from(std::env::var_os("OCXD_TEST_BUNDLE_ARCHIVE").expect("explicit archive"));
    let descriptor = PathBuf::from(
        std::env::var_os("OCXD_TEST_BUNDLE_DESCRIPTOR").expect("explicit descriptor"),
    );
    let pin = std::env::var("OCXD_TEST_BUNDLE_PIN").expect("explicit trusted pin");
    assert!(archive.is_absolute() && descriptor.is_absolute());
    let temp = tempfile::tempdir().unwrap();
    let base = temp.path().canonicalize().unwrap();
    let installation = base.join("program");
    let data = base.join("desktop");
    let home = base.join("home");
    for dir in [&data, &home] {
        std::fs::create_dir_all(dir).unwrap();
    }
    crate::modules::runtime::bundled::versioned::provision(
        &installation,
        &data,
        &archive,
        &std::fs::read(descriptor).unwrap(),
        &pin,
    )
    .unwrap();
    let runtime = RuntimeHandle::initialize_versioned(&data, &installation);
    let p = program(&runtime).unwrap();
    let source = base.join("selected.json");
    let current = home.join("config.json");
    let original = br#"{"port":41234,"providers":{}}"#;
    std::fs::write(&current, original).unwrap();
    // An unknown adapter string can be accepted for upstream extensibility.
    // This malformed official schema section is rejected by the real validator.
    let invalid =
        br#"{"port":51234,"providers":{},"chatgptDesktop":{"appServerShim":"not-a-boolean"}}"#;
    assert!(validate(&p, invalid).is_err());
    std::fs::write(&source, invalid).unwrap();
    assert_eq!(
        native_import::prepare(&data, &home, &source, |raw| validate(&p, raw))
            .err()
            .unwrap()
            .code,
        "native_import_config_invalid"
    );
    assert_eq!(std::fs::read(&current).unwrap(), original);
    assert!(!data.join("backups").exists());
    std::fs::write(&source, br#"{"port":51234,"defaultProvider":"New","providers":{"New":{"adapter":"openai-responses","authMode":"key","baseUrl":"https://fixture.invalid/v1","apiKey":"fixture-only","models":["fixture-model"]}}}"#).unwrap();
    let selected = native_import::prepare(&data, &home, &source, |raw| validate(&p, raw)).unwrap();
    let result = native_import::apply(&data, &home, selected, || Ok(())).unwrap();
    assert!(result.changed && result.backup_id.is_some());
    assert_eq!(result.port, 41234);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&std::fs::read(&current).unwrap()).unwrap()
            ["providers"]["New"]["apiKey"],
        "fixture-only"
    );
    assert!(!Path::new(&home).join("runtime-port.json").exists());
    println!("Official 2.99 config validate accepted selected and effective fixtures; invalid fixture refused; original backup created; no service launched.");
}
