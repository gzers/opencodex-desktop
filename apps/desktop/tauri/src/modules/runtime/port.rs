//! Explicit per-HOME port configuration. Only config.port is projected into the
//! official document; mode and a crash-recovery intent belong to Desktop.
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{atomic_write::atomic_write, locking::TargetFileLock},
    modules::backup::safety,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    net::{Ipv4Addr, TcpListener},
    path::{Path, PathBuf},
};
use zeroize::Zeroizing;

const SETTINGS: &str = "manager-state/runtime-port-settings.json";
const INTENT: &str = "manager-state/runtime-port-save.json";

struct ConfigMutation(rusqlite::Connection);
impl ConfigMutation {
    fn acquire(home: &Path) -> AppResult<Self> {
        let path = home.join("config-mutation.sqlite");
        for suffix in ["", "-journal", "-wal", "-shm"] {
            safety::check_path(
                home,
                &home.join(format!("config-mutation.sqlite{suffix}")),
                true,
            )?;
        }
        let file = crate::infrastructure::platform::open_or_create_private_file(&path, 0o600)
            .map_err(|_| error("port_config_lock", "配置事务锁无法打开，未保存端口"))?;
        safety::require_single_link(&file)?;
        let connection = rusqlite::Connection::open(&path)
            .map_err(|_| error("port_config_lock", "配置事务数据库无法打开，未保存端口"))?;
        connection
            .busy_timeout(crate::infrastructure::locking::TARGET_LOCK_TIMEOUT)
            .map_err(|_| error("port_config_lock", "配置事务等待策略不可用"))?;
        connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(|_| error("port_config_lock", "另一个配置事务正在进行，请稍后重试"))?;
        Ok(Self(connection))
    }
    fn commit(&self) -> AppResult<()> {
        // Exact maintained-runtime generation contract, not an independent clock.
        self.0.execute_batch("CREATE TABLE IF NOT EXISTS config_generation (singleton INTEGER PRIMARY KEY CHECK (singleton = 1), value INTEGER NOT NULL CHECK (value >= 0)); INSERT OR IGNORE INTO config_generation (singleton,value) VALUES (1,0);")
            .map_err(|_|error("port_save_unconfirmed","配置代次未能核验，保存意图已保留"))?;
        let generation: i64 = self
            .0
            .query_row(
                "SELECT value FROM config_generation WHERE singleton=1",
                [],
                |r| r.get(0),
            )
            .map_err(|_| error("port_save_unconfirmed", "配置代次无效，保存意图已保留"))?;
        if !(0..9_007_199_254_740_991).contains(&generation) {
            return Err(error(
                "port_save_unconfirmed",
                "配置代次越界，未宣称保存成功",
            ));
        }
        if self
            .0
            .execute(
                "UPDATE config_generation SET value=value+1 WHERE singleton=1 AND value=?",
                [generation],
            )
            .map_err(|_| error("port_save_unconfirmed", "配置代次写入未完成"))?
            != 1
        {
            return Err(error(
                "port_save_unconfirmed",
                "配置代次已变化，未宣称保存成功",
            ));
        }
        self.0
            .execute_batch("COMMIT")
            .map_err(|_| error("port_save_unconfirmed", "配置事务结果待核对"))?;
        Ok(())
    }
}
impl Drop for ConfigMutation {
    fn drop(&mut self) {
        let _ = self.0.execute_batch("ROLLBACK");
    }
}
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Intent {
    schema_version: u32,
    desired: Saved,
    before_digest: String,
    after_digest: String,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub mode: Mode,
    pub port: Option<u16>,
    pub revision: String,
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
fn read(root: &Path, relative: &str, limit: u64) -> AppResult<Zeroizing<Vec<u8>>> {
    let path = root.join(relative);
    safety::check_path(root, &path, true)?;
    match std::fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Zeroizing::new(vec![])),
        Err(_) => Err(error("port_read", "端口配置无法安全读取，原文件已保留")),
        Ok(_) => Ok(Zeroizing::new(safety::read_file(&path, limit)?)),
    }
}
fn config(home: &Path) -> AppResult<(Value, Zeroizing<Vec<u8>>)> {
    let bytes = read(home, "config.json", 16 * 1024 * 1024)?;
    // Initializing a genuinely missing document is allowed; damaged documents are never empty.
    let value = if bytes.is_empty() && !home.join("config.json").exists() {
        Value::Object(Default::default())
    } else {
        serde_json::from_slice(&bytes)
            .map_err(|_| error("port_config_invalid", "渠道配置格式错误，未写入端口"))?
    };
    if !value.is_object() {
        return Err(error("port_config_invalid", "渠道配置不是对象，未写入端口"));
    }
    Ok((value, bytes))
}
fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> AppResult<Option<T>> {
    if bytes.is_empty() {
        Ok(None)
    } else {
        serde_json::from_slice(bytes)
            .map(Some)
            .map_err(|_| error("port_settings_invalid", "端口设置记录损坏，未覆盖原记录"))
    }
}
fn validate_saved(value: &Saved, home: &Path) -> AppResult<()> {
    if value.schema_version != 1 || value.home != home || value.port == 0 {
        return Err(error(
            "port_settings_invalid",
            "端口设置结构、HOME或数值不一致",
        ));
    }
    Ok(())
}
fn revision(config: &[u8], settings: &[u8], intent: &[u8]) -> String {
    hash(format!("{}:{}:{}", hash(config), hash(settings), hash(intent)).as_bytes())
}
pub fn load(root: &Path, home: &Path) -> AppResult<Snapshot> {
    if !root.is_absolute() || !home.is_absolute() {
        return Err(error("port_context", "端口配置需要当前用户的明确目录"));
    }
    let (config, bytes) = config(home)?;
    let settings = read(root, SETTINGS, 4096)?;
    let intent = read(root, INTENT, 8192)?;
    let saved: Option<Saved> = decode(&settings)?;
    let pending: Option<Intent> = decode(&intent)?;
    if let Some(s) = &saved {
        validate_saved(s, home)?;
    }
    if let Some(i) = &pending {
        if i.schema_version != 1 {
            return Err(error("port_settings_invalid", "端口保存记录结构暂不支持"));
        }
        validate_saved(&i.desired, home)?;
    }
    let port = match config.get("port") {
        None => None,
        Some(v) => Some(
            v.as_u64()
                .filter(|n| *n <= u16::MAX as u64)
                .ok_or_else(|| error("port_config_invalid", "当前配置端口无效，原文件已保留"))?
                as u16,
        )
        .filter(|p| *p != 0),
    };
    // A completed intent can remain if the final removal was interrupted. Only
    // exact matching readbacks make it complete; a changed document stays pending.
    let incomplete = pending.is_some();
    Ok(Snapshot {
        mode: saved.as_ref().filter(|s| Some(s.port) == port).map_or(
            if port.is_some() {
                Mode::Fixed
            } else {
                Mode::Auto
            },
            |s| s.mode,
        ),
        port,
        revision: revision(&bytes, &settings, &intent),
        reconciliation_required: incomplete,
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
    if !root.is_absolute() || !home.is_absolute() {
        return Err(error("port_context", "端口配置需要当前用户的明确目录"));
    }
    let mutation = ConfigMutation::acquire(home)?;
    safety::check_path(home, &home.join("config.json.lock"), true)?;
    let _lock = TargetFileLock::lock(&home.join("config.json"))?;
    let settings_path = root.join(SETTINGS);
    safety::check_path(root, &settings_path, true)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&settings_path),
        true,
    )?;
    safety::check_path(root, &root.join(INTENT), true)?;
    let _settings = TargetFileLock::lock(&settings_path)?;
    let before = load(root, home)?;
    if before.revision != request.expected_revision {
        return Err(error("port_stale", "端口或渠道配置已变化，请刷新后重试"));
    }
    if before.reconciliation_required {
        return Err(error(
            "port_save_unconfirmed",
            "上次端口保存未确认，请先核对原操作",
        ));
    }
    // Keep the reservation through the durable write. It is only a check, not
    // ownership of a future listener; startup must bind/recheck again.
    let listener = checked_listener(request.mode, request.port)?;
    let port = listener
        .local_addr()
        .map_err(|_| error("port_check_failed", "无法读取已检查端口"))?
        .port();
    let (mut value, bytes) = config(home)?;
    value["port"] = Value::from(port);
    let next = Zeroizing::new(
        serde_json::to_vec_pretty(&value)
            .map_err(|_| error("port_config_invalid", "端口配置序列化失败"))?,
    );
    let desired = Saved {
        schema_version: 1,
        home: home.to_path_buf(),
        mode: request.mode,
        port,
    };
    let intent = Intent {
        schema_version: 1,
        desired: desired.clone(),
        before_digest: hash(&bytes),
        after_digest: hash(&next),
    };
    atomic_write(
        &root.join(INTENT),
        &serde_json::to_vec(&intent).map_err(|_| AppError::NotConfigured)?,
        0o600,
    )?;
    // Re-read directly before replacement; do not overwrite a concurrent external edit.
    if read(home, "config.json", 16 * 1024 * 1024)?.as_slice() != bytes.as_slice() {
        return Err(error(
            "port_save_unconfirmed",
            "配置在端口检查后变化，已保留保存意图供核对",
        ));
    }
    atomic_write(&home.join("config.json"), &next, 0o600)?;
    if read(home, "config.json", 16 * 1024 * 1024)?.as_slice() != next.as_slice() {
        return Err(error(
            "port_save_unconfirmed",
            "端口写后读回不一致，未标记保存成功",
        ));
    }
    atomic_write(
        &settings_path,
        &serde_json::to_vec(&desired).map_err(|_| AppError::NotConfigured)?,
        0o600,
    )?;
    let verified = load(root, home)?;
    let settings_read: Option<Saved> = decode(&read(root, SETTINGS, 4096)?)?;
    if settings_read.as_ref() != Some(&desired) || verified.port != Some(port) {
        return Err(error("port_save_unconfirmed", "端口设置尚未核验一致"));
    }
    mutation.commit()?;
    std::fs::remove_file(root.join(INTENT))
        .map_err(|_| error("port_save_unconfirmed", "端口已写入，完成记录待核对"))?;
    drop(listener);
    load(root, home)
}

/// Explicit recovery only adopts an exact completed config write from its own
/// durable intent. A third-party edit or the before-image never becomes success.
pub fn reconcile(root: &Path, home: &Path) -> AppResult<Snapshot> {
    if !root.is_absolute() || !home.is_absolute() {
        return Err(error("port_context", "端口配置需要当前用户的明确目录"));
    }
    let mutation = ConfigMutation::acquire(home)?;
    safety::check_path(home, &home.join("config.json.lock"), true)?;
    let _lock = TargetFileLock::lock(&home.join("config.json"))?;
    let path = root.join(SETTINGS);
    safety::check_path(root, &path, true)?;
    safety::check_path(
        root,
        &crate::infrastructure::locking::lock_path_for(&path),
        true,
    )?;
    let _settings = TargetFileLock::lock(&path)?;
    let bytes = read(root, INTENT, 8192)?;
    let Some(intent): Option<Intent> = decode(&bytes)? else {
        return load(root, home);
    };
    validate_saved(&intent.desired, home)?;
    let (_, current) = config(home)?;
    if intent.schema_version == 1 && hash(&current) == intent.before_digest {
        std::fs::remove_file(root.join(INTENT))
            .map_err(|_| error("port_save_unconfirmed", "原操作未写入，清理保存意图失败"))?;
        return load(root, home);
    }
    if intent.schema_version != 1 || hash(&current) != intent.after_digest {
        return Err(error(
            "port_save_unconfirmed",
            "原端口操作结果不一致，未重写渠道配置",
        ));
    }
    atomic_write(
        &path,
        &serde_json::to_vec(&intent.desired).map_err(|_| AppError::NotConfigured)?,
        0o600,
    )?;
    let verified: Option<Saved> = decode(&read(root, SETTINGS, 4096)?)?;
    if verified.as_ref() != Some(&intent.desired) {
        return Err(error("port_save_unconfirmed", "端口完成记录仍未核验"));
    }
    mutation.commit()?;
    std::fs::remove_file(root.join(INTENT))
        .map_err(|_| error("port_save_unconfirmed", "原端口结果已核验，完成记录未收口"))?;
    load(root, home)
}

pub fn prepare_start(root: &Path, home: &Path) -> AppResult<Snapshot> {
    let current = load(root, home)?;
    if current.reconciliation_required {
        return Err(error(
            "port_save_unconfirmed",
            "端口保存结果待核对，未启动服务",
        ));
    }
    if let Some(port) = current.port {
        match checked_listener(Mode::Fixed, Some(port)) {
            Ok(reservation) => {
                drop(reservation);
                return Ok(current);
            }
            Err(AppError::RuntimeManaged { code, .. })
                if code == "port_conflict" && current.mode == Mode::Auto => {}
            Err(e) => return Err(e),
        }
    }
    save(
        root,
        home,
        &SaveRequest {
            mode: Mode::Auto,
            port: None,
            expected_revision: current.revision,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn runtime_sqlite_mutation_and_generation_are_shared_not_an_independent_lock() {
        let (_t, root, home) = fixture();
        let before = load(&root, &home).unwrap();
        let connection = rusqlite::Connection::open(home.join("config-mutation.sqlite")).unwrap();
        connection.execute_batch("BEGIN IMMEDIATE").unwrap();
        let bytes = std::fs::read(home.join("config.json")).unwrap();
        let result = save(&root, &home, &request(&before));
        assert!(
            matches!(result,Err(AppError::RuntimeManaged{code,..}) if code=="port_config_lock")
        );
        assert_eq!(std::fs::read(home.join("config.json")).unwrap(), bytes);
        assert!(!root.join(INTENT).exists());
        connection.execute_batch("ROLLBACK").unwrap();
        let saved = save(&root, &home, &request(&before)).unwrap();
        assert!(saved.port.is_some());
        let generation: i64 = connection
            .query_row(
                "SELECT value FROM config_generation WHERE singleton=1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(generation, 1);
    }
    fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let base = temp.path().canonicalize().unwrap();
        let root = base.join("desktop");
        let home = base.join("opencodex/home");
        crate::infrastructure::platform::create_private_dir_all(&root, 0o700).unwrap();
        crate::infrastructure::platform::create_private_dir_all(&home, 0o700).unwrap();
        atomic_write(&home.join("config.json"),br#"{"port":0,"providers":{"sample":{"apiKey":"synthetic-only"}},"custom":{"keep":true}}"#,0o600).unwrap();
        (temp, root, home)
    }
    fn request(s: &Snapshot) -> SaveRequest {
        SaveRequest {
            mode: Mode::Auto,
            port: None,
            expected_revision: s.revision.clone(),
        }
    }
    #[test]
    fn auto_port_persists_and_reopens_without_changing_channel_material() {
        let (_t, r, h) = fixture();
        let before = load(&r, &h).unwrap();
        assert_eq!(before.port, None);
        let saved = save(&r, &h, &request(&before)).unwrap();
        assert!(saved.port.is_some());
        assert!(!saved.reconciliation_required);
        let (value, _) = config(&h).unwrap();
        assert_eq!(value["providers"]["sample"]["apiKey"], "synthetic-only");
        assert_eq!(value["custom"]["keep"], true);
        assert_eq!(load(&r, &h).unwrap().port, saved.port);
        assert_eq!(load(&r, &h).unwrap().mode, Mode::Auto);
    }
    #[test]
    fn occupied_fixed_port_and_invalid_zero_leave_documents_untouched() {
        let (_t, r, h) = fixture();
        let before = load(&r, &h).unwrap();
        let bytes = std::fs::read(h.join("config.json")).unwrap();
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        for port in [Some(0), Some(listener.local_addr().unwrap().port()), None] {
            assert!(save(
                &r,
                &h,
                &SaveRequest {
                    mode: Mode::Fixed,
                    port,
                    expected_revision: before.revision.clone()
                }
            )
            .is_err());
            assert_eq!(std::fs::read(h.join("config.json")).unwrap(), bytes);
            assert!(!r.join(SETTINGS).exists());
            assert!(!r.join(INTENT).exists());
        }
    }
    #[test]
    fn stale_request_cannot_overwrite_channel_edit() {
        let (_t, r, h) = fixture();
        let before = load(&r, &h).unwrap();
        atomic_write(&h.join("config.json"), br#"{"port":0,"newer":true}"#, 0o600).unwrap();
        assert!(save(&r, &h, &request(&before)).is_err());
        assert_eq!(config(&h).unwrap().0["newer"], true);
        let saved = save(&r, &h, &request(&load(&r, &h).unwrap())).unwrap();
        assert!(saved.port.is_some());
        atomic_write(
            &h.join("config.json"),
            br#"{"port":12345,"external":true}"#,
            0o600,
        )
        .unwrap();
        let external = load(&r, &h).unwrap();
        assert_eq!(external.mode, Mode::Fixed);
        assert_eq!(external.port, Some(12345));
        assert!(
            !external.reconciliation_required,
            "A complete external edit is not a partial Desktop transaction"
        );
    }
    #[test]
    fn corrupt_or_empty_existing_config_is_not_empty() {
        let (_t, r, h) = fixture();
        for bytes in [b"".as_slice(), b"not-json", b"[]"] {
            atomic_write(&h.join("config.json"), bytes, 0o600).unwrap();
            assert!(load(&r, &h).is_err());
            assert_eq!(std::fs::read(h.join("config.json")).unwrap(), bytes);
        }
    }
    #[test]
    fn automatic_start_reallocates_occupied_port_but_fixed_refuses() {
        let (_t, r, h) = fixture();
        let saved = save(&r, &h, &request(&load(&r, &h).unwrap())).unwrap();
        let busy = TcpListener::bind((Ipv4Addr::LOCALHOST, saved.port.unwrap())).unwrap();
        let next = prepare_start(&r, &h).unwrap();
        assert_ne!(next.port, saved.port);
        assert_eq!(busy.local_addr().unwrap().port(), saved.port.unwrap());
        drop(busy);
        let fixed = save(
            &r,
            &h,
            &SaveRequest {
                mode: Mode::Fixed,
                port: saved.port,
                expected_revision: next.revision,
            },
        )
        .unwrap();
        let _busy = TcpListener::bind((Ipv4Addr::LOCALHOST, fixed.port.unwrap())).unwrap();
        assert!(prepare_start(&r, &h).is_err());
        assert_eq!(load(&r, &h).unwrap().port, fixed.port);
    }
    #[test]
    fn interrupted_projection_needs_explicit_exact_reconciliation() {
        let (_t, r, h) = fixture();
        let saved = save(&r, &h, &request(&load(&r, &h).unwrap())).unwrap();
        let desired: Saved = decode(&read(&r, SETTINGS, 4096).unwrap()).unwrap().unwrap();
        let (_, bytes) = config(&h).unwrap();
        let intent = Intent {
            schema_version: 1,
            desired,
            before_digest: hash(b"before"),
            after_digest: hash(&bytes),
        };
        atomic_write(
            &r.join(INTENT),
            &serde_json::to_vec(&intent).unwrap(),
            0o600,
        )
        .unwrap();
        std::fs::remove_file(r.join(SETTINGS)).unwrap();
        assert!(load(&r, &h).unwrap().reconciliation_required);
        assert!(prepare_start(&r, &h).is_err());
        let recovered = reconcile(&r, &h).unwrap();
        assert_eq!(recovered.port, saved.port);
        assert!(!recovered.reconciliation_required);
    }
    #[test]
    fn recovery_refuses_changed_content_and_other_home() {
        let (_t, r, h) = fixture();
        let saved = save(&r, &h, &request(&load(&r, &h).unwrap())).unwrap();
        assert!(load(&r, &r).is_err());
        let desired: Saved = decode(&read(&r, SETTINGS, 4096).unwrap()).unwrap().unwrap();
        let intent = Intent {
            schema_version: 1,
            desired,
            before_digest: "a".repeat(64),
            after_digest: "b".repeat(64),
        };
        atomic_write(
            &r.join(INTENT),
            &serde_json::to_vec(&intent).unwrap(),
            0o600,
        )
        .unwrap();
        assert!(reconcile(&r, &h).is_err());
        assert_eq!(load(&r, &h).unwrap().port, saved.port);
    }
}
