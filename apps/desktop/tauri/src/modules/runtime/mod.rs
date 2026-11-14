//! 运行来源解析层（IMP `FZ-47` / `FZ-48`，`TASK` 见 Track B · B1）。
//!
//! 本模块回答一个问题：**当前用哪个 `ocx`**。答案唯一，并且被
//! 发现、状态采集、Doctor、版本检查、启停与 IPC **共用**——不允许各自算一份。
//!
//! 优先级（`FZ-48`）：用户显式指定 > 托管安装 > 自动发现候选。**不读取 PATH**。

pub mod archive;
pub mod bundled;
pub mod client_artifact_policy;
pub mod install;
pub mod official_lifecycle024;
pub(crate) mod offline_drop;
pub mod paths;
pub mod port;
pub(crate) mod preparation;
pub mod program_activation;
pub mod program_catalog;
pub mod program_image;
pub mod program_layout;
pub(crate) mod program_reinstall;
pub mod program_removal;
pub(crate) mod program_repair;
pub mod program_selection;
pub(crate) mod protection;
pub(crate) mod startup_bundle;
pub mod terminal_cli;
pub mod uninstall;
pub mod version_facts;
pub mod windows_lifecycle024_contract;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::errors::AppError;
use paths::PathRejection;

/// 只允许官方包；硬编码，不接受用户改写（`FZ-47` / `FZ-50`）。
pub const OFFICIAL_PACKAGE: &str = "@bitkyc08/opencodex";

/// `runtime.json` 结构版本；未知版本只读降级、不写回（`FZ-48`）。
pub const RUNTIME_SCHEMA_VERSION: u32 = 1;

/// 安装 / 卸载历史最多保留的条数（`FZ-48`）。
pub const MAX_HISTORY: usize = 50;

/// 托管私有前缀相对数据根的位置（`FZ-47`）。
pub const MANAGED_PREFIX_RELATIVE: &str = "runtime/opencodex";

/// 托管安装的稳定入口相对数据根的位置（`FZ-47`）。
#[cfg(not(windows))]
pub const MANAGED_ENTRY_RELATIVE: &str = "runtime/bin/ocx";
#[cfg(windows)]
pub const MANAGED_ENTRY_RELATIVE: &str = "runtime/bin/ocx.cmd";

/// 前缀内的落地清单文件名（`FZ-47`）。
pub const MANIFEST_FILENAME: &str = ".runtime-manifest.json";

/// 运行来源的种类（`FZ-48`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeSourceKind {
    /// 用户在设置中显式指定。
    Explicit,
    /// 管理器托管安装（`<数据根>/runtime/bin/ocx`）。
    Managed,
    /// 自动发现候选（`~/.local/bin`、Homebrew、npm 前缀推导等）。
    Discovered,
    /// 一个都没解析出来。
    Unresolved,
}

impl RuntimeSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            RuntimeSourceKind::Explicit => "explicit",
            RuntimeSourceKind::Managed => "managed",
            RuntimeSourceKind::Discovered => "discovered",
            RuntimeSourceKind::Unresolved => "unresolved",
        }
    }
}

/// 安装 / 卸载历史条目（`FZ-48`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InstallHistoryEntry {
    pub action: String,
    pub result: String,
    pub package: String,
    pub version: String,
    pub target: String,
    pub at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// `manager-state/runtime.json` 的内存形态（`FZ-48`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeSourceRecord {
    pub schema_version: u32,
    pub source: RuntimeSourceKind,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub resolved_version: Option<String>,
    #[serde(default)]
    pub resolved_at: Option<String>,
    #[serde(default)]
    pub history: Vec<InstallHistoryEntry>,
}

impl Default for RuntimeSourceRecord {
    fn default() -> Self {
        Self {
            schema_version: RUNTIME_SCHEMA_VERSION,
            source: RuntimeSourceKind::Unresolved,
            path: None,
            resolved_version: None,
            resolved_at: None,
            history: Vec::new(),
        }
    }
}

impl RuntimeSourceRecord {
    /// 读取时把历史裁到上限，避免手改或旧版本写出的超长文件把内存撑大。
    fn normalized(mut self) -> Self {
        if self.history.len() > MAX_HISTORY {
            let keep_from = self.history.len() - MAX_HISTORY;
            self.history.drain(..keep_from);
        }
        self
    }

    /// 追加历史，保留最近 `MAX_HISTORY` 条。
    pub fn push_history(&mut self, entry: InstallHistoryEntry) {
        self.history.push(entry);
        if self.history.len() > MAX_HISTORY {
            let drop = self.history.len() - MAX_HISTORY;
            self.history.drain(..drop);
        }
    }

    /// **安装记录登记的托管前缀**（`FZ-51` 的口径）。
    ///
    /// `FZ-47` 允许把包体装到自定义前缀（选择器 / 文本框给出），此时前缀不再等于
    /// `<数据根>/runtime/opencodex`；卸载与「重装同版本」必须按**记录**定位，否则会
    /// 指向一个不存在的默认目录（真机：自定义前缀安装后，卸载弹窗显示的是默认前缀）。
    /// 取最近一条**尚未被成功卸载**的成功安装 `target`；没有可用记录时回落到数据根默认位置。
    pub fn installed_prefix(&self, data_root: &Path) -> PathBuf {
        // 从最近一条往回扫：先记下「之后被成功卸载过」的前缀，再取第一条尚未被
        // 卸载的成功安装目标。否则一级卸载后仍会指向一个已经被删掉的目录
        // （真机：自定义前缀卸载后，安装弹窗又预填回那个已删除的自定义目录）。
        let mut removed: Vec<&str> = Vec::new();
        for entry in self.history.iter().rev() {
            if entry.target.is_empty() {
                continue;
            }
            match (entry.action.as_str(), entry.result.as_str()) {
                ("uninstall", "succeeded") => removed.push(entry.target.as_str()),
                ("install", "succeeded") if !removed.contains(&entry.target.as_str()) => {
                    return PathBuf::from(&entry.target);
                }
                _ => {}
            }
        }
        data_root.join(MANAGED_PREFIX_RELATIVE)
    }
}

/// `runtime.json` 的读写（分区键 `manager state`）。
#[derive(Debug, Clone)]
pub struct RuntimeStore {
    path: PathBuf,
}

impl RuntimeStore {
    pub fn new(data_root: &Path) -> Self {
        Self {
            path: data_root
                .join("manager-state")
                .join(RUNTIME_SOURCE_FILENAME),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// 读：文件缺失或内容损坏都降级为 `Unresolved` + 空历史，绝不 panic。
    ///
    /// 契约要求「未知版本只读降级、不写回」，因此这里只做读取归一化，
    /// 不尝试修补文件——真正的写回只发生在终态（`save`）。
    pub fn load(&self) -> RuntimeSourceRecord {
        let Ok(raw) = std::fs::read(&self.path) else {
            return RuntimeSourceRecord::default();
        };
        let Ok(record) = serde_json::from_slice::<RuntimeSourceRecord>(&raw) else {
            return RuntimeSourceRecord::default();
        };
        if record.schema_version != RUNTIME_SCHEMA_VERSION {
            return RuntimeSourceRecord::default();
        }
        record.normalized()
    }

    /// 写：原子替换，Unix `0600`（本文件不含凭据，但统一按管理器状态文件对待）。
    pub fn save(&self, record: &RuntimeSourceRecord) -> Result<(), AppError> {
        let payload = serde_json::to_vec_pretty(record).map_err(|error| AppError::FileSystem {
            operation: "serialize runtime source record".to_string(),
            detail: error.to_string(),
        })?;
        crate::infrastructure::atomic_write::atomic_write(&self.path, &payload, 0o600)
    }

    fn load_for_update(&self) -> Result<RuntimeSourceRecord, AppError> {
        match std::fs::symlink_metadata(&self.path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok(RuntimeSourceRecord::default())
            }
            Err(_) => return Err(AppError::NotConfigured),
            Ok(_) => {}
        }
        let root = self
            .path
            .parent()
            .and_then(Path::parent)
            .ok_or(AppError::NotConfigured)?;
        crate::modules::backup::safety::check_path(root, &self.path, false)?;
        let raw = crate::modules::backup::safety::read_file(&self.path, 1024 * 1024)?;
        let record: RuntimeSourceRecord =
            serde_json::from_slice(&raw).map_err(|_| AppError::RuntimeManaged {
                code: "runtime_history_invalid".into(),
                detail: "运行程序历史记录无效，原件保留；未将其作为当前执行来源".into(),
            })?;
        if record.schema_version != RUNTIME_SCHEMA_VERSION {
            return Err(AppError::RuntimeManaged {
                code: "runtime_history_invalid".into(),
                detail: "运行程序历史记录版本未知，原件保留".into(),
            });
        }
        Ok(record.normalized())
    }
}

/// `runtime.json` 的文件名。
pub const RUNTIME_SOURCE_FILENAME: &str = "runtime.json";

/// 一次解析的结果：用哪个 `ocx`，以及它是**哪一类**来源。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeResolution {
    pub kind: RuntimeSourceKind,
    pub path: Option<PathBuf>,
}

impl RuntimeResolution {
    pub fn unresolved() -> Self {
        Self {
            kind: RuntimeSourceKind::Unresolved,
            path: None,
        }
    }

    pub fn is_resolved(&self) -> bool {
        self.path.is_some()
    }
}

/// 解析输入。
#[derive(Debug, Clone)]
pub struct RuntimeResolver {
    managed_entry: PathBuf,
    discovered: Vec<PathBuf>,
    managed_only: bool,
    versioned: bool,
}

#[derive(Debug, Clone)]
struct RuntimeObservation {
    resolution: RuntimeResolution,
    selected: Option<program_selection::SelectedProgram>,
    problem: Option<String>,
}
impl RuntimeObservation {
    fn legacy(resolution: RuntimeResolution) -> Self {
        Self {
            resolution,
            selected: None,
            problem: None,
        }
    }
}

impl RuntimeResolver {
    /// `data_root` 提供托管安装的固定入口位置；`discovered` 按优先级从高到低给出。
    pub fn new(data_root: &Path, discovered: Vec<PathBuf>) -> Self {
        Self {
            managed_entry: data_root.join(MANAGED_ENTRY_RELATIVE),
            discovered,
            managed_only: false,
            versioned: false,
        }
    }

    /// 显式指定是否可用（读路径校验：跟随符号链接、不限制系统目录）。
    pub fn validate_explicit(path: &Path) -> Result<PathBuf, PathRejection> {
        paths::validate_executable(path)
    }

    /// 解析优先级：显式指定 > 托管安装 > 自动发现候选。
    ///
    /// `explicit` 只有在**通过可执行校验**时才生效；校验失败即静默回落到下一档，
    /// 避免一个失效的手填路径把整个应用卡成「未解析」。
    pub fn resolve(&self, explicit: Option<&Path>) -> RuntimeResolution {
        if self.versioned {
            return self.observe(explicit).resolution;
        }
        if self.managed_only {
            let root = self
                .managed_entry
                .parent()
                .and_then(Path::parent)
                .and_then(Path::parent);
            if let Some(root) = root {
                let node = root.join(if cfg!(windows) {
                    "runtime/node/node.exe"
                } else {
                    "runtime/node/bin/node"
                });
                if install::validate_managed_installation(root, Some(&node)).is_ok() {
                    return RuntimeResolution {
                        kind: RuntimeSourceKind::Managed,
                        path: Some(self.managed_entry.clone()),
                    };
                }
            }
            return RuntimeResolution::unresolved();
        }
        if !self.managed_only {
            if let Some(path) = explicit {
                if let Ok(valid) = paths::validate_executable(path) {
                    return RuntimeResolution {
                        kind: RuntimeSourceKind::Explicit,
                        path: Some(valid),
                    };
                }
            }
        }
        if let Ok(valid) = paths::validate_executable(&self.managed_entry) {
            return RuntimeResolution {
                kind: RuntimeSourceKind::Managed,
                path: Some(valid),
            };
        }
        if !self.managed_only {
            for candidate in &self.discovered {
                if let Ok(valid) = paths::validate_executable(candidate) {
                    return RuntimeResolution {
                        kind: RuntimeSourceKind::Discovered,
                        path: Some(valid),
                    };
                }
            }
        }
        RuntimeResolution::unresolved()
    }

    fn observe(&self, explicit: Option<&Path>) -> RuntimeObservation {
        if !self.versioned {
            return RuntimeObservation::legacy(self.resolve(explicit));
        }
        let result = (|| {
            let root = self
                .managed_entry
                .parent()
                .and_then(Path::parent)
                .and_then(Path::parent)
                .ok_or(AppError::NotConfigured)?;
            let (target, platform) = program_layout::host_target()?;
            program_selection::read(root, platform, target)
        })();
        match result {
            Ok(Some(selected)) => RuntimeObservation {
                resolution: RuntimeResolution {
                    kind: RuntimeSourceKind::Managed,
                    path: Some(selected.stable_entry.clone()),
                },
                selected: Some(selected),
                problem: None,
            },
            Ok(None) => RuntimeObservation::legacy(RuntimeResolution::unresolved()),
            Err(error) => RuntimeObservation {
                resolution: RuntimeResolution::unresolved(),
                selected: None,
                problem: Some(error.to_string()),
            },
        }
    }
}

/// 可重解析的共享句柄（`FZ-48`：发现 / 状态 / Doctor / 版本 / 启停 / IPC 共用同一份结果）。
///
/// 句柄本身是 `Arc`，可同时注入多个消费者；`refresh` 在安装 / 卸载终态后调用，
/// 之后所有消费者下一次读取都会拿到新结果。
#[derive(Debug)]
pub struct RuntimeHandle {
    store: RuntimeStore,
    resolver: RuntimeResolver,
    explicit: Mutex<Option<PathBuf>>,
    current: Mutex<RuntimeObservation>,
    pub(crate) preparation: preparation::Preparation,
}

impl RuntimeHandle {
    /// 建立句柄并完成首次解析；首次解析结果**立即落盘**，让外部工具也能读到当前来源。
    pub fn initialize(data_root: &Path, discovered: Vec<PathBuf>) -> Arc<Self> {
        let store = RuntimeStore::new(data_root);
        let explicit = match store.load() {
            record if record.source == RuntimeSourceKind::Explicit => {
                record.path.map(PathBuf::from)
            }
            _ => None,
        };
        let resolver = RuntimeResolver::new(data_root, discovered);
        let resolution = resolver.resolve(explicit.as_deref());
        let handle = Arc::new(Self {
            preparation: Default::default(),
            store,
            resolver,
            explicit: Mutex::new(explicit),
            current: Mutex::new(RuntimeObservation::legacy(resolution)),
        });
        let _ = handle.persist();
        handle
    }

    /// v2 installed program source is independent from manager state and legacy discovery.
    pub fn initialize_managed(data_root: &Path, installation_root: &Path) -> Arc<Self> {
        Self::initialize_program(data_root, installation_root, false)
    }

    /// Production startup selects only the current version record. Older flat
    /// fixtures keep their explicit constructor; this path never falls back.
    pub fn initialize_versioned(data_root: &Path, installation_root: &Path) -> Arc<Self> {
        Self::initialize_program(data_root, installation_root, true)
    }

    fn initialize_program(
        data_root: &Path,
        installation_root: &Path,
        versioned: bool,
    ) -> Arc<Self> {
        let store = RuntimeStore::new(data_root);
        let resolver = RuntimeResolver {
            managed_entry: installation_root.join(if versioned {
                program_layout::runtime_entry_relative(if cfg!(windows) {
                    crate::modules::data_root::layout::Platform::Windows
                } else if cfg!(target_os = "macos") {
                    crate::modules::data_root::layout::Platform::Macos
                } else {
                    crate::modules::data_root::layout::Platform::Unix
                })
            } else {
                MANAGED_ENTRY_RELATIVE
            }),
            discovered: vec![],
            managed_only: true,
            versioned,
        };
        let current = resolver.observe(None);
        let handle = Arc::new(Self {
            preparation: Default::default(),
            store,
            resolver,
            explicit: Mutex::new(None),
            current: Mutex::new(current),
        });
        let _ = handle.persist();
        handle
    }
    pub fn installation_root(&self) -> &Path {
        self.resolver
            .managed_entry
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .expect("managed entry under installation/runtime/bin")
    }

    /// 当前解析结果（不做 I/O）。
    pub fn current(&self) -> RuntimeResolution {
        self.current
            .lock()
            .map(|value| value.resolution.clone())
            .unwrap_or_else(|_| RuntimeResolution::unresolved())
    }
    pub fn uses_versioned_programs(&self) -> bool {
        self.resolver.versioned
    }

    /// Cached full admission plus bounded record/entry freshness. High-frequency
    /// status never hashes Node/Bun again; action/source refresh owns that check.
    pub fn source_problem(&self) -> Option<String> {
        self.selected_program().err().map(|e| e.to_string())
    }
    pub fn selected_program(&self) -> Result<Option<program_selection::SelectedProgram>, AppError> {
        let current = self.current.lock().map_err(|_| AppError::NotConfigured)?;
        if let Some(detail) = &current.problem {
            return Err(AppError::RuntimeManaged {
                code: "runtime_source_unconfirmed".into(),
                detail: detail.clone(),
            });
        }
        if let Some(selected) = &current.selected {
            program_selection::verify_current(self.installation_root(), selected)?;
        }
        Ok(current.selected.clone())
    }

    /// 当前可执行文件路径；未解析时为 `None`。
    pub fn executable(&self) -> Option<PathBuf> {
        if self.source_problem().is_some() {
            return None;
        }
        self.current().path
    }

    pub fn kind(&self) -> RuntimeSourceKind {
        self.current().kind
    }

    pub fn store(&self) -> &RuntimeStore {
        &self.store
    }

    /// 用户显式指定的路径（未指定时为 `None`）。
    pub fn explicit_path(&self) -> Option<PathBuf> {
        self.explicit
            .lock()
            .map(|value| value.clone())
            .unwrap_or_default()
    }

    /// 托管入口路径（`FZ-47`），与当前是否解析到它无关。
    pub fn resolver_managed_only(&self) -> bool {
        self.resolver.managed_only
    }
    pub fn managed_entry(&self) -> PathBuf {
        self.resolver.managed_entry.clone()
    }

    /// 记录并持久化最新的已知版本（由版本检查在读取成功后回填）。
    pub fn record_resolved_version(&self, version: &str) {
        if self.uses_versioned_programs()
            && self
                .selected_program()
                .ok()
                .flatten()
                .is_none_or(|p| p.selection.version != version)
        {
            return;
        }
        let mut record = if self.uses_versioned_programs() {
            let Ok(record) = self.store.load_for_update() else {
                return;
            };
            record
        } else {
            self.store.load()
        };
        let resolution = self.current();
        record.schema_version = RUNTIME_SCHEMA_VERSION;
        record.source = resolution.kind;
        record.path = resolution
            .path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned());
        record.resolved_version = Some(version.to_string());
        record.resolved_at = Some(now_rfc3339());
        let _ = self.store.save(&record);
    }

    /// 追加一条安装 / 卸载历史（只在**终态**调用）。
    pub fn record_history(&self, entry: InstallHistoryEntry) -> Result<(), AppError> {
        let mut record = if self.uses_versioned_programs() {
            self.store.load_for_update()?
        } else {
            self.store.load()
        };
        record.push_history(entry);
        self.store.save(&record)
    }

    /// 重新解析并落盘。安装 / 卸载完成、或用户改动显式指定后调用。
    pub fn refresh(&self) -> RuntimeResolution {
        let resolution = self.observe_current();
        let _ = self.persist();
        resolution
    }

    /// Explicit source/version query re-admits actual programs without writing
    /// history or changing the user's selected program.
    pub fn observe_current(&self) -> RuntimeResolution {
        let explicit = self
            .explicit
            .lock()
            .map(|value| value.clone())
            .unwrap_or_default();
        let observation = self.resolver.observe(explicit.as_deref());
        let resolution = observation.resolution.clone();
        if let Ok(mut current) = self.current.lock() {
            *current = observation;
        }
        resolution
    }

    /// 设置用户显式指定的 `ocx` 路径；`None` 表示回到「托管 > 发现」。
    ///
    /// 传入的路径必须通过可执行校验，否则会被忽略（调用方在 B5 负责给用户提示）。
    pub fn set_explicit(&self, path: Option<PathBuf>) -> RuntimeResolution {
        if self.resolver.managed_only {
            return self.refresh();
        }
        let accepted = match path {
            Some(candidate) if RuntimeResolver::validate_explicit(&candidate).is_ok() => {
                Some(candidate)
            }
            Some(_) => self
                .explicit
                .lock()
                .map(|value| value.clone())
                .unwrap_or_default(),
            None => None,
        };
        if let Ok(mut value) = self.explicit.lock() {
            *value = accepted;
        }
        self.refresh()
    }

    fn persist(&self) -> Result<(), AppError> {
        // Unknown source cannot erase a prior recorded installation/history.
        // That record remains historical, never an executable fallback.
        let selected = self.selected_program()?;
        // A missing version selection is an observation, not a new install or
        // uninstall transaction. In particular, do not rewrite the exact source
        // after-image retained by a completed explicit removal.
        if self.uses_versioned_programs() && selected.is_none() {
            return Ok(());
        }
        let resolution = self.current();
        let mut record = if self.uses_versioned_programs() {
            self.store.load_for_update()?
        } else {
            self.store.load()
        };
        record.schema_version = RUNTIME_SCHEMA_VERSION;
        record.source = resolution.kind;
        let resolved_path = resolution
            .path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned());
        // 已记录的版本属于被读过的那个来源；来源换到别的路径（或未解析）后，
        // 旧版本不再是「当前来源的版本」，必须清掉，否则卡片会把上一个来源的
        // 版本挂在新的来源上（`FZ-48`：`resolved_version` 描述当前来源）。
        if record.path != resolved_path {
            record.resolved_version = None;
        }
        if self.uses_versioned_programs() {
            record.resolved_version = selected.map(|p| p.selection.version);
        }
        record.path = resolved_path;
        record.resolved_at = Some(now_rfc3339());
        self.store.save(&record)
    }
}

/// RFC 3339（UTC，秒精度）。
pub fn now_rfc3339() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// 让共享句柄直接充当基础设施层的可执行文件来源（`FZ-48` 的「共用同一份解析结果」）。
impl crate::infrastructure::runtime_executable::RuntimeExecutableProvider for RuntimeHandle {
    fn executable(&self) -> Option<PathBuf> {
        RuntimeHandle::executable(self)
    }
    fn managed_installation_root(&self) -> Option<PathBuf> {
        self.resolver_managed_only()
            .then(|| self.installation_root().to_path_buf())
    }
    fn source_problem(&self) -> Option<String> {
        RuntimeHandle::source_problem(self)
    }
    fn observed_program_identity(
        &self,
    ) -> crate::errors::AppResult<install::ManagedRuntimeIdentity> {
        if !self.uses_versioned_programs() {
            return install::resolve_managed_installation(self.installation_root())
                .map_err(Into::into);
        }
        let selected = self.selected_program()?.ok_or(AppError::NotConfigured)?;
        Ok(install::ManagedRuntimeIdentity {
            installation_root: self.installation_root().into(),
            prefix: selected.paths.package_root,
            entry: selected.stable_entry,
            node: selected.paths.node,
            package_version: selected.selection.version,
        })
    }
    fn managed_program_identity(
        &self,
    ) -> crate::errors::AppResult<install::ManagedRuntimeIdentity> {
        if !self.uses_versioned_programs() {
            return install::resolve_managed_installation(self.installation_root())
                .map_err(Into::into);
        }
        let (target, platform) = program_layout::host_target()?;
        let selected = program_selection::read(self.installation_root(), platform, target)?
            .ok_or(AppError::NotConfigured)?;
        Ok(install::ManagedRuntimeIdentity {
            installation_root: self.installation_root().into(),
            prefix: selected.paths.package_root,
            entry: selected.stable_entry,
            node: selected.paths.node,
            package_version: selected.selection.version,
        })
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    #[test]
    fn versioned_source_uses_actual_image_and_preserves_history_on_unknown() {
        let f = program_selection::tests::Fixture::new();
        let state = tempfile::tempdir().unwrap();
        let root = state.path().canonicalize().unwrap().join("desktop");
        let handle = RuntimeHandle::initialize_versioned(&root, &f.root);
        assert_eq!(handle.kind(), RuntimeSourceKind::Managed);
        assert_eq!(
            handle.executable(),
            Some(f.root.join("runtime/bin/ocx-runtime"))
        );
        assert_eq!(handle.selected_program().unwrap().unwrap().paths, f.paths);
        assert_eq!(
            handle.store.load().resolved_version.as_deref(),
            Some("2.99.0")
        );
        let record = std::fs::read(handle.store.path()).unwrap();
        handle.record_resolved_version("9.9.9");
        assert_eq!(std::fs::read(handle.store.path()).unwrap(), record);
        std::fs::write(f.root.join("runtime/current.json"), b"{").unwrap();
        assert!(handle.executable().is_none());
        assert!(handle.source_problem().is_some());
        assert_eq!(std::fs::read(handle.store.path()).unwrap(), record);
        assert_eq!(handle.refresh().kind, RuntimeSourceKind::Unresolved);
        assert!(handle.source_problem().is_some());
        assert!(handle.selected_program().is_err());
        assert_eq!(std::fs::read(handle.store.path()).unwrap(), record);
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    #[test]
    fn versioned_source_never_uses_flat_or_explicit_fallback() {
        let t = tempfile::tempdir().unwrap();
        let root = t.path().canonicalize().unwrap();
        let program = root.join("program");
        let data = root.join("desktop");
        let handle = RuntimeHandle::initialize_versioned(&data, &program);
        assert!(handle.source_problem().is_none());
        assert_eq!(handle.kind(), RuntimeSourceKind::Unresolved);
        write_executable(&program.join("runtime/bin/ocx"));
        let external = root.join("external/ocx");
        write_executable(&external);
        assert_eq!(
            handle.set_explicit(Some(external)).kind,
            RuntimeSourceKind::Unresolved
        );
        assert!(handle.source_problem().is_none());
        assert!(handle.selected_program().unwrap().is_none());
    }

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    #[test]
    fn damaged_history_cannot_override_source_or_be_erased_by_observation() {
        let f = program_selection::tests::Fixture::new();
        let t = tempfile::tempdir().unwrap();
        let data = t.path().canonicalize().unwrap().join("desktop");
        let store = RuntimeStore::new(&data);
        std::fs::create_dir_all(store.path().parent().unwrap()).unwrap();
        std::fs::write(store.path(), b"{unreadable history").unwrap();
        let before = std::fs::read(store.path()).unwrap();
        let handle = RuntimeHandle::initialize_versioned(&data, &f.root);
        assert_eq!(handle.kind(), RuntimeSourceKind::Managed);
        assert_eq!(
            handle
                .selected_program()
                .unwrap()
                .unwrap()
                .selection
                .version,
            "2.99.0"
        );
        handle.observe_current();
        handle.refresh();
        handle.record_resolved_version("2.99.0");
        assert_eq!(std::fs::read(store.path()).unwrap(), before);
    }

    fn write_executable(path: &Path) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("mkdir fixture");
        }
        std::fs::write(path, b"#!/bin/sh\nexit 0\n").expect("write fixture");
        std::fs::set_permissions(path, Permissions::from_mode(0o755)).expect("chmod fixture");
    }

    fn fixture() -> (tempfile::TempDir, PathBuf, PathBuf, PathBuf) {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let managed = data_root.join(MANAGED_ENTRY_RELATIVE);
        let discovered = root.path().join("home/.local/bin/ocx");
        (root, data_root, managed, discovered)
    }

    #[test]
    fn managed_v2_never_uses_legacy_or_explicit_programs() {
        let root = tempfile::tempdir().unwrap();
        let data = root.path().join("user/desktop");
        let install = root.path().join("installed");
        let legacy = data.join(MANAGED_ENTRY_RELATIVE);
        let explicit = root.path().join("old/ocx");
        write_executable(&legacy);
        write_executable(&explicit);
        let handle = RuntimeHandle::initialize_managed(&data, &install);
        assert_eq!(handle.kind(), RuntimeSourceKind::Unresolved);
        assert_eq!(
            handle.set_explicit(Some(explicit)).kind,
            RuntimeSourceKind::Unresolved
        );
        assert!(handle.explicit_path().is_none());
        assert_eq!(handle.installation_root(), install);
        let entry = install.join(MANAGED_ENTRY_RELATIVE);
        write_executable(&entry);
        assert_eq!(handle.refresh().kind, RuntimeSourceKind::Unresolved); // executable alone is not install proof
        let prefix = install.join(MANAGED_PREFIX_RELATIVE);
        let package = prefix.join(install::PACKAGE_SUBPATH);
        std::fs::create_dir_all(package.join("bin")).unwrap();
        let node = install.join("runtime/node/bin/node");
        write_executable(&node);
        let script = package.join("bin/ocx.mjs");
        write_executable(&script);
        std::fs::write(package.join("package.json"),serde_json::to_vec(&serde_json::json!({"name":OFFICIAL_PACKAGE,"version":"2.97.0-desktop023.names.1","bin":{"ocx":"bin/ocx.mjs"}})).unwrap()).unwrap();
        std::fs::write(&entry, install::entry_script(Some(&node), &script)).unwrap();
        let proof = protection::activation_proof(
            &prefix,
            &uuid::Uuid::new_v4().to_string(),
            "bin/ocx.mjs",
            &node,
        )
        .unwrap();
        let manifest = install::RuntimeManifest {
            activation: Some(proof),
            package: OFFICIAL_PACKAGE.into(),
            version: "2.97.0-desktop023.names.1".into(),
            tarball_sha256: "a".repeat(64),
            installed_at: "2026-11-06T00:00:00Z".into(),
            npm_path: "fixture".into(),
            scripts_enabled: false,
            source: install::InstallSourceKind::Offline,
        };
        std::fs::write(
            prefix.join(MANIFEST_FILENAME),
            serde_json::to_vec(&manifest).unwrap(),
        )
        .unwrap();
        assert_eq!(handle.refresh().path, Some(entry));
        std::fs::remove_file(handle.managed_entry()).unwrap();
        assert_eq!(handle.refresh().kind, RuntimeSourceKind::Unresolved);
        assert!(legacy.exists());
    }

    #[test]
    fn priority_is_explicit_then_managed_then_discovered() {
        let (root, data_root, managed, discovered) = fixture();
        write_executable(&managed);
        write_executable(&discovered);
        let explicit = root.path().join("explicit/ocx");
        write_executable(&explicit);

        let resolver = RuntimeResolver::new(&data_root, vec![discovered.clone()]);

        let all = resolver.resolve(Some(&explicit));
        assert_eq!(all.kind, RuntimeSourceKind::Explicit);
        assert_eq!(all.path, Some(explicit));

        let without_explicit = resolver.resolve(None);
        assert_eq!(without_explicit.kind, RuntimeSourceKind::Managed);
        assert_eq!(without_explicit.path, Some(managed));
    }

    #[test]
    fn managed_wins_over_discovered_and_falls_back_when_missing() {
        let (_root, data_root, managed, discovered) = fixture();
        write_executable(&discovered);
        let resolver = RuntimeResolver::new(&data_root, vec![discovered.clone()]);

        let discovered_only = resolver.resolve(None);
        assert_eq!(discovered_only.kind, RuntimeSourceKind::Discovered);
        assert_eq!(discovered_only.path, Some(discovered));

        write_executable(&managed);
        let managed_now = resolver.resolve(None);
        assert_eq!(managed_now.kind, RuntimeSourceKind::Managed);
    }

    #[test]
    fn invalid_explicit_falls_back_instead_of_sticking() {
        let (_root, data_root, managed, _discovered) = fixture();
        write_executable(&managed);
        let resolver = RuntimeResolver::new(&data_root, Vec::new());

        // 不存在的显式路径不能把应用钉死在「未解析」。
        let resolution = resolver.resolve(Some(Path::new("/definitely/missing/ocx")));
        assert_eq!(resolution.kind, RuntimeSourceKind::Managed);
    }

    #[test]
    fn nothing_available_is_unresolved() {
        let (_root, data_root, _managed, _discovered) = fixture();
        let resolver = RuntimeResolver::new(&data_root, vec![PathBuf::from("/missing/ocx")]);
        let resolution = resolver.resolve(None);
        assert_eq!(resolution.kind, RuntimeSourceKind::Unresolved);
        assert!(!resolution.is_resolved());
    }

    #[test]
    fn store_round_trips_and_creates_restricted_file() {
        let (_root, data_root, _managed, _discovered) = fixture();
        let store = RuntimeStore::new(&data_root);
        assert_eq!(
            store.path(),
            data_root.join("manager-state").join("runtime.json")
        );

        let mut record = RuntimeSourceRecord {
            source: RuntimeSourceKind::Managed,
            path: Some("/x/ocx".to_string()),
            resolved_version: Some("0.3.1".to_string()),
            resolved_at: Some("2026-09-25T00:00:00+00:00".to_string()),
            ..Default::default()
        };
        record.push_history(InstallHistoryEntry {
            action: "install".to_string(),
            result: "succeeded".to_string(),
            package: OFFICIAL_PACKAGE.to_string(),
            version: "0.3.1".to_string(),
            target: "/x".to_string(),
            at: "2026-09-25T00:00:00+00:00".to_string(),
            reason: None,
        });
        store.save(&record).expect("save");

        assert_eq!(store.load(), record);
        let mode = std::fs::metadata(store.path())
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600, "runtime.json 应为 0600");
    }

    #[test]
    fn store_degrades_on_missing_corrupt_and_unknown_version() {
        let (_root, data_root, _managed, _discovered) = fixture();
        let store = RuntimeStore::new(&data_root);
        assert_eq!(store.load(), RuntimeSourceRecord::default());

        std::fs::create_dir_all(store.path().parent().expect("parent")).expect("mkdir");
        std::fs::write(store.path(), b"{not json").expect("write");
        assert_eq!(store.load(), RuntimeSourceRecord::default());

        std::fs::write(
            store.path(),
            br#"{"schema_version": 99, "source": "managed", "history": []}"#,
        )
        .expect("write");
        assert_eq!(store.load(), RuntimeSourceRecord::default());
    }

    #[test]
    fn history_is_capped_at_the_frozen_limit() {
        let mut record = RuntimeSourceRecord::default();
        for index in 0..(MAX_HISTORY + 10) {
            record.push_history(InstallHistoryEntry {
                action: "install".to_string(),
                result: "succeeded".to_string(),
                package: OFFICIAL_PACKAGE.to_string(),
                version: format!("0.0.{index}"),
                target: "/x".to_string(),
                at: "2026-09-25T00:00:00+00:00".to_string(),
                reason: None,
            });
        }
        assert_eq!(record.history.len(), MAX_HISTORY);
        // 保留的是最近的那些。
        assert!(record
            .history
            .last()
            .expect("last")
            .version
            .ends_with(".59"));
    }

    #[test]
    fn handle_persists_resolution_and_refreshes_after_install() {
        let (_root, data_root, managed, discovered) = fixture();
        write_executable(&discovered);
        let handle = RuntimeHandle::initialize(&data_root, vec![discovered.clone()]);

        assert_eq!(handle.kind(), RuntimeSourceKind::Discovered);
        assert_eq!(handle.executable(), Some(discovered));
        // 首次解析立即落盘，外部工具能读到当前来源。
        let persisted = handle.store().load();
        assert_eq!(persisted.source, RuntimeSourceKind::Discovered);
        assert!(persisted.resolved_at.is_some());

        // 模拟托管安装完成：新入口出现后 refresh 应切到 managed。
        write_executable(&managed);
        let resolution = handle.refresh();
        assert_eq!(resolution.kind, RuntimeSourceKind::Managed);
        assert_eq!(handle.store().load().source, RuntimeSourceKind::Managed);
    }

    #[test]
    fn handle_restores_explicit_choice_across_restart() {
        let (root, data_root, managed, discovered) = fixture();
        write_executable(&discovered);
        write_executable(&managed);
        let explicit = root.path().join("explicit/ocx");
        write_executable(&explicit);

        let handle = RuntimeHandle::initialize(&data_root, vec![discovered]);
        assert_eq!(handle.kind(), RuntimeSourceKind::Managed);
        let chosen = handle.set_explicit(Some(explicit.clone()));
        assert_eq!(chosen.kind, RuntimeSourceKind::Explicit);

        // 重启：新建句柄应从 runtime.json 恢复显式选择。
        let reopened = RuntimeHandle::initialize(&data_root, Vec::new());
        assert_eq!(reopened.kind(), RuntimeSourceKind::Explicit);
        assert_eq!(reopened.executable(), Some(explicit));

        // 清空显式选择后回到托管。
        let cleared = reopened.set_explicit(None);
        assert_eq!(cleared.kind, RuntimeSourceKind::Managed);
    }

    #[test]
    fn set_explicit_ignores_invalid_path_and_keeps_previous_choice() {
        let (root, data_root, managed, _discovered) = fixture();
        write_executable(&managed);
        let explicit = root.path().join("explicit/ocx");
        write_executable(&explicit);

        let handle = RuntimeHandle::initialize(&data_root, Vec::new());
        handle.set_explicit(Some(explicit.clone()));
        let after_invalid = handle.set_explicit(Some(PathBuf::from("/missing/ocx")));
        assert_eq!(after_invalid.kind, RuntimeSourceKind::Explicit);
        assert_eq!(after_invalid.path, Some(explicit));
    }

    #[test]
    fn installed_prefix_comes_from_the_install_record_not_the_default_location() {
        let (_root, data_root, _managed, _discovered) = fixture();
        let mut record = RuntimeSourceRecord::default();
        let default_prefix = data_root.join(MANAGED_PREFIX_RELATIVE);

        // 没有可用的安装记录 → 回落到数据根默认位置。
        assert_eq!(record.installed_prefix(&data_root), default_prefix);
        record.push_history(InstallHistoryEntry {
            action: "install".to_string(),
            result: "failed".to_string(),
            package: OFFICIAL_PACKAGE.to_string(),
            version: "2.66.0".to_string(),
            target: "/custom/never-installed".to_string(),
            at: "2026-09-26T00:00:00+00:00".to_string(),
            reason: Some("boom".to_string()),
        });
        assert_eq!(record.installed_prefix(&data_root), default_prefix);

        // 自定义前缀安装成功后，卸载 / 重装必须按记录定位（FZ-47 / FZ-51）。
        record.push_history(InstallHistoryEntry {
            action: "install".to_string(),
            result: "succeeded".to_string(),
            package: OFFICIAL_PACKAGE.to_string(),
            version: "2.66.0".to_string(),
            target: "/custom/prefix".to_string(),
            at: "2026-09-26T00:00:01+00:00".to_string(),
            reason: None,
        });
        assert_eq!(
            record.installed_prefix(&data_root),
            PathBuf::from("/custom/prefix")
        );

        // 该自定义前缀被成功卸载后，记录不能再指向一个已删除的目录。
        record.push_history(InstallHistoryEntry {
            action: "uninstall".to_string(),
            result: "succeeded".to_string(),
            package: OFFICIAL_PACKAGE.to_string(),
            version: "2.66.0".to_string(),
            target: "/custom/prefix".to_string(),
            at: "2026-09-26T00:00:02+00:00".to_string(),
            reason: None,
        });
        assert_eq!(record.installed_prefix(&data_root), default_prefix);

        // 再装回默认位置后，以最新一条成功安装为准。
        record.push_history(InstallHistoryEntry {
            action: "install".to_string(),
            result: "succeeded".to_string(),
            package: OFFICIAL_PACKAGE.to_string(),
            version: "2.66.0".to_string(),
            target: default_prefix.to_string_lossy().into_owned(),
            at: "2026-09-26T00:00:03+00:00".to_string(),
            reason: None,
        });
        assert_eq!(record.installed_prefix(&data_root), default_prefix);
    }

    #[test]
    fn resolved_version_follows_the_current_source_path() {
        let (_root, data_root, managed, discovered) = fixture();
        write_executable(&discovered);
        let handle = RuntimeHandle::initialize(&data_root, vec![discovered.clone()]);
        handle.record_resolved_version("2.50.0");
        assert_eq!(
            handle.store().load().resolved_version.as_deref(),
            Some("2.50.0")
        );

        // 托管入口出现 → 来源换到另一个路径：旧版本不再属于当前来源，必须清掉。
        write_executable(&managed);
        handle.refresh();
        assert_eq!(handle.store().load().resolved_version, None);

        // 重新读到的版本重新回填。
        handle.record_resolved_version("2.66.0");
        assert_eq!(
            handle.store().load().resolved_version.as_deref(),
            Some("2.66.0")
        );

        // 卸载后来源变未解析：版本同样不能残留。
        std::fs::remove_file(&managed).expect("remove entry");
        std::fs::remove_file(&discovered).expect("remove discovered");
        handle.refresh();
        let record = handle.store().load();
        assert_eq!(record.source, RuntimeSourceKind::Unresolved);
        assert_eq!(record.resolved_version, None);
    }

    #[test]
    fn record_history_keeps_only_terminal_entries_and_caps_length() {
        let (_root, data_root, _managed, _discovered) = fixture();
        let handle = RuntimeHandle::initialize(&data_root, Vec::new());
        for index in 0..3 {
            handle
                .record_history(InstallHistoryEntry {
                    action: "install".to_string(),
                    result: if index == 1 { "failed" } else { "succeeded" }.to_string(),
                    package: OFFICIAL_PACKAGE.to_string(),
                    version: "0.3.1".to_string(),
                    target: "/x".to_string(),
                    at: "2026-09-25T00:00:00+00:00".to_string(),
                    reason: None,
                })
                .expect("record history");
        }
        let record = handle.store().load();
        assert_eq!(record.history.len(), 3);
        assert_eq!(record.history[1].result, "failed");
    }
}
