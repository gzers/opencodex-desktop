//! Per-user Desktop settings for the runtime launch port.
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{atomic_write::atomic_write_before_replace, locking::TargetFileLock},
    modules::backup::safety,
};
use serde::{Deserialize, Serialize};
use std::{
    net::{Ipv4Addr, TcpListener},
    path::{Path, PathBuf},
    time::Instant,
};

const SETTINGS: &str = "manager-state/runtime-port-settings.json";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    Auto,
    Fixed,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Saved {
    schema_version: u32,
    home: PathBuf,
    mode: Mode,
    port: u16,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub mode: Mode,
    pub port: Option<u16>,
    pub revision: String,
    #[serde(skip)]
    pub reconciliation_required: bool,
}
#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SaveRequest {
    pub mode: Mode,
    pub port: Option<u16>,
    pub expected_revision: String,
}
fn error(code: &str, message: &str) -> AppError {
    AppError::RuntimeManaged {
        code: code.into(),
        detail: message.into(),
    }
}
fn hash(bytes: &[u8]) -> String {
    crate::infrastructure::hash::sha256_hex(bytes)
}
fn read_settings(root: &Path) -> AppResult<Option<Vec<u8>>> {
    let path = root.join(SETTINGS);
    safety::check_path(root, &path, true)?;
    match std::fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err(error("port_read", "端口设置无法读取")),
        Ok(_) => Ok(Some(safety::read_file(&path, 4096)?)),
    }
}

pub fn load(root: &Path, home: &Path) -> AppResult<Snapshot> {
    if !root.is_absolute() || !home.is_absolute() {
        return Err(error("port_context", "端口配置需要当前用户的明确目录"));
    }
    let Some(bytes) = read_settings(root)? else {
        return Ok(Snapshot {
            mode: Mode::Auto,
            port: None,
            revision: hash(b""),
            reconciliation_required: false,
        });
    };
    let saved: Saved = serde_json::from_slice(&bytes)
        .map_err(|_| error("port_settings_invalid", "端口设置记录损坏，原记录已保留"))?;
    if saved.schema_version != 1 || saved.home != home || saved.port == 0 {
        return Err(error(
            "port_settings_invalid",
            "端口设置结构、HOME 或数值不一致",
        ));
    }
    Ok(Snapshot {
        mode: saved.mode,
        port: Some(saved.port),
        revision: hash(&bytes),
        reconciliation_required: false,
    })
}

fn checked_listener(mode: Mode, port: Option<u16>) -> AppResult<TcpListener> {
    let port = match mode {
        Mode::Auto => 0,
        Mode::Fixed => port
            .filter(|p| *p != 0)
            .ok_or_else(|| error("port_invalid", "请输入1～65535之间的整数端口"))?,
    };
    TcpListener::bind((Ipv4Addr::LOCALHOST, port)).map_err(|e| match e.kind() {
        std::io::ErrorKind::AddrInUse => error(
            "port_conflict",
            &format!("端口 {port} 已被占用，请调整端口或重新分配"),
        ),
        std::io::ErrorKind::PermissionDenied => error("port_permission", "没有权限监听所选端口"),
        _ => error("port_check_failed", "无法检查空闲端口，原设置已保留"),
    })
}
pub fn save(root: &Path, home: &Path, request: &SaveRequest) -> AppResult<Snapshot> {
    save_with_deadline(root, home, request, None)
}

fn check_deadline(deadline: Option<Instant>) -> AppResult<()> {
    if deadline.is_some_and(|deadline| Instant::now() >= deadline) {
        return Err(AppError::Timeout);
    }
    Ok(())
}

fn save_with_deadline(
    root: &Path,
    home: &Path,
    request: &SaveRequest,
    start_deadline: Option<Instant>,
) -> AppResult<Snapshot> {
    if !root.is_absolute() || !home.is_absolute() {
        return Err(error("port_context", "端口配置需要当前用户的明确目录"));
    }
    let path = root.join(SETTINGS);
    safety::check_path(root, &path, true)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&path),
        true,
    )?;
    let _lock = match start_deadline {
        Some(deadline) => TargetFileLock::lock_until(&path, deadline)?,
        None => TargetFileLock::lock(&path)?,
    };
    let before = load(root, home)?;
    if before.revision != request.expected_revision {
        return Err(error("port_stale", "端口设置已变化，请刷新后重试"));
    }
    // The reservation lasts through the write. Startup checks the port again.
    let listener = checked_listener(request.mode, request.port)?;
    let port = listener
        .local_addr()
        .map_err(|_| error("port_check_failed", "无法读取已检查端口"))?
        .port();
    let desired = Saved {
        schema_version: 1,
        home: home.to_path_buf(),
        mode: request.mode,
        port,
    };
    let bytes = serde_json::to_vec(&desired).map_err(|_| AppError::NotConfigured)?;
    check_deadline(start_deadline)?;
    atomic_write_before_replace(&path, &bytes, 0o600, || check_deadline(start_deadline))?;
    let verified = load(root, home)?;
    if verified.revision != hash(&bytes) {
        return Err(error(
            "port_save_unconfirmed",
            "端口设置写入后未能核验，请刷新设置",
        ));
    }
    Ok(verified)
}

pub fn prepare_start(root: &Path, home: &Path, deadline: Instant) -> AppResult<Snapshot> {
    check_deadline(Some(deadline))?;
    let current = load(root, home)?;
    if let Some(port) = current.port {
        match checked_listener(Mode::Fixed, Some(port)) {
            Ok(reservation) => {
                drop(reservation);
                check_deadline(Some(deadline))?;
                return Ok(current);
            }
            Err(AppError::RuntimeManaged { code, .. })
                if code == "port_conflict" && current.mode == Mode::Auto => {}
            Err(e) => return Err(e),
        }
    }
    save_with_deadline(
        root,
        home,
        &SaveRequest {
            mode: Mode::Auto,
            port: None,
            expected_revision: current.revision,
        },
        Some(deadline),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::atomic_write::atomic_write;
    use std::time::Duration;

    fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let root = base.join("desktop");
        let home = base.join("opencodex/home");
        crate::infrastructure::platform::create_private_dir_all(&root, 0o700).unwrap();
        crate::infrastructure::platform::create_private_dir_all(&home, 0o700).unwrap();
        (temp, root, home)
    }

    #[test]
    fn automatic_port_wait_honors_start_deadline_and_keeps_original_settings() {
        let (_temp, root, home) = fixture();
        let occupied = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let port = occupied.local_addr().unwrap().port();
        let path = root.join(SETTINGS);
        let original = serde_json::to_vec(&Saved {
            schema_version: 1,
            home: home.clone(),
            mode: Mode::Auto,
            port,
        })
        .unwrap();
        atomic_write(&path, &original, 0o600).unwrap();
        let guard = TargetFileLock::lock(&path).unwrap();

        let started = Instant::now();
        let result = prepare_start(&root, &home, started + Duration::from_millis(250));

        assert!(matches!(result, Err(AppError::Timeout)), "{result:?}");
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_eq!(std::fs::read(&path).unwrap(), original);

        drop(guard);
        let saved = prepare_start(&root, &home, Instant::now() + Duration::from_secs(3)).unwrap();
        assert_ne!(saved.port, Some(port));
        assert_eq!(load(&root, &home).unwrap().revision, saved.revision);
    }

    #[test]
    fn auto_is_independent_of_native_config_and_reallocates_on_collision() {
        let (_temp, root, home) = fixture();
        let native = home.join("config.json");
        let first = prepare_start(&root, &home, Instant::now() + Duration::from_secs(5)).unwrap();
        assert!(!native.exists());
        atomic_write(&native, b"malformed native config", 0o600).unwrap();
        assert_eq!(load(&root, &home).unwrap().revision, first.revision);
        let _held = TcpListener::bind((Ipv4Addr::LOCALHOST, first.port.unwrap())).unwrap();
        let second = prepare_start(&root, &home, Instant::now() + Duration::from_secs(5)).unwrap();
        assert_eq!(second.mode, Mode::Auto);
        assert_ne!(second.port, first.port);
        assert_eq!(load(&root, &home).unwrap().port, second.port);
        assert_eq!(std::fs::read(native).unwrap(), b"malformed native config");
        assert!(!home.join("config-mutation.sqlite").exists());
        assert!(!home.join("config.json.lock").exists());
    }

    #[test]
    fn fixed_rejects_invalid_occupied_and_stale_requests() {
        let (_temp, root, home) = fixture();
        let before = load(&root, &home).unwrap();
        let held = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let mut request = SaveRequest {
            mode: Mode::Fixed,
            port: Some(0),
            expected_revision: before.revision,
        };
        assert!(
            matches!(save(&root, &home, &request), Err(AppError::RuntimeManaged { code, .. }) if code == "port_invalid")
        );
        request.port = Some(held.local_addr().unwrap().port());
        assert!(
            matches!(save(&root, &home, &request), Err(AppError::RuntimeManaged { code, .. }) if code == "port_conflict")
        );
        drop(held);
        let saved = save(&root, &home, &request).unwrap();
        assert_eq!(saved.mode, Mode::Fixed);
        assert!(
            matches!(save(&root, &home, &request), Err(AppError::RuntimeManaged { code, .. }) if code == "port_stale")
        );
        let _held = TcpListener::bind((Ipv4Addr::LOCALHOST, saved.port.unwrap())).unwrap();
        assert!(
            matches!(prepare_start(&root, &home, Instant::now() + Duration::from_secs(5)), Err(AppError::RuntimeManaged { code, .. }) if code == "port_conflict")
        );
        assert_eq!(load(&root, &home).unwrap().revision, saved.revision);
    }

    #[test]
    fn damaged_settings_are_preserved() {
        let (_temp, root, home) = fixture();
        let before = load(&root, &home).unwrap();
        let path = root.join(SETTINGS);
        atomic_write(&path, b"", 0o600).unwrap();
        assert!(save(
            &root,
            &home,
            &SaveRequest {
                mode: Mode::Auto,
                port: None,
                expected_revision: before.revision
            }
        )
        .is_err());
        assert_eq!(std::fs::read(path).unwrap(), b"");
    }
}
