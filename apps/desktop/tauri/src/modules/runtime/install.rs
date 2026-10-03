//! 托管安装内核（IMP `FZ-47` / `FZ-49` / `FZ-50`）。
//!
//! 一次安装要同时满足三件事：**能装上**、**装错不落地**、**凭据不泄漏**。
//! 因此顺序被固定为：
//!
//! 1. 目标前缀校验（`FZ-47` 写路径口径）→ `<前缀>.lock` 跨进程锁；
//! 2. 装进 `<前缀>.new-<随机>` 临时区（离线解包 / 联网走受控 npm）；
//! 3. 回读 `package.json` 校验 `name` / `version` / 入口可执行；
//! 4. 写 `.runtime-manifest.json`；
//! 5. **原子替换**前缀，再写稳定入口 `<数据根>/runtime/bin/ocx`。
//!
//! 任一步失败都会清理临时区并**保持现有前缀与运行来源不变**（`FZ-47`）。
//!
//! 本模块不读 PATH、不读 `~/.npmrc`、不写系统代理、不落盘任何凭据；
//! 含凭据的代理只写临时 `--userconfig`（`0600`，用完即删）。

use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{RecvTimeoutError, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::archive::{self, ArchiveRejection};
use super::paths::{self, PathRejection};
use super::{
    now_rfc3339, InstallHistoryEntry, MANAGED_ENTRY_RELATIVE, MANIFEST_FILENAME, OFFICIAL_PACKAGE,
};
use crate::errors::AppError;
use crate::infrastructure::atomic_write::atomic_write;
use crate::infrastructure::locking::TargetFileLock;

/// 包体在前缀内的落点（`node_modules/@bitkyc08/opencodex`）。
pub const PACKAGE_SUBPATH: &str = "node_modules/@bitkyc08/opencodex";

/// 私有项目 `package.json` 的文件名（前缀根）。
pub const PROJECT_PACKAGE_FILENAME: &str = "package.json";

/// 默认版本通道（`FZ-50`：版本默认 `latest`，可显式指定具体版本）。
pub const DEFAULT_VERSION: &str = "latest";

/// 空闲多久没有输出就提示「可能卡住」（`FZ-50`）。
pub const IDLE_HINT: Duration = Duration::from_secs(120);

/// 受控 npm 调用的最小 PATH 兜底；不继承调用方的 PATH。
const FALLBACK_PATH: &str = "/usr/bin:/bin:/usr/sbin:/sbin";

// ---------------------------------------------------------------------------
// 代理（`FZ-50`）
// ---------------------------------------------------------------------------

/// 代理协议。SOCKS5 一律用 `socks5h://`（让 DNS 也走代理，避免本地泄漏）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyScheme {
    Http,
    Socks5h,
}

impl ProxyScheme {
    pub fn as_str(self) -> &'static str {
        match self {
            ProxyScheme::Http => "http",
            ProxyScheme::Socks5h => "socks5h",
        }
    }

    /// 界面与 IPC 输入的解析；不接受其它协议。
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "http" => Some(ProxyScheme::Http),
            "socks5" | "socks5h" => Some(ProxyScheme::Socks5h),
            _ => None,
        }
    }
}

/// 代理凭据；**只进临时 `--userconfig`，绝不进 argv**（`FZ-50`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyCredential {
    pub username: String,
    pub secret: String,
}

/// 一次安装使用的代理配置；只在本次调用生效，不落盘。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyConfig {
    pub scheme: ProxyScheme,
    /// `host` 或 `host:port`。
    pub host: String,
    pub credential: Option<ProxyCredential>,
}

impl ProxyConfig {
    pub fn new(scheme: ProxyScheme, host: impl Into<String>) -> Self {
        Self {
            scheme,
            host: host.into(),
            credential: None,
        }
    }

    pub fn with_credential(
        mut self,
        username: impl Into<String>,
        secret: impl Into<String>,
    ) -> Self {
        self.credential = Some(ProxyCredential {
            username: username.into(),
            secret: secret.into(),
        });
        self
    }

    pub fn has_credential(&self) -> bool {
        self.credential.is_some()
    }

    /// 无凭据形式：`<scheme>://host[:port]`。
    pub fn url_without_credential(&self) -> String {
        format!("{}://{}", self.scheme.as_str(), self.host)
    }

    /// 界面与日志统一展示的掩码形式：`<scheme>://***@host[:port]`。
    pub fn masked(&self) -> String {
        format!("{}://***@{}", self.scheme.as_str(), self.host)
    }

    /// 写入临时 `--userconfig` 的含凭据形式；无凭据时返回 `None`。
    pub fn url_with_credential(&self) -> Option<String> {
        self.credential.as_ref().map(|credential| {
            format!(
                "{}://{}:{}@{}",
                self.scheme.as_str(),
                credential.username,
                credential.secret,
                self.host
            )
        })
    }

    /// 校验：协议已固定，地址必须非空且不含空白与 `@`（避免把凭据顺手写进地址）。
    pub fn validate(&self) -> Result<(), InstallError> {
        let host = self.host.trim();
        if host.is_empty() {
            return Err(InstallError::BadProxy {
                reason: "代理地址不能为空".to_string(),
            });
        }
        if host.contains(['@', ' ', '\t', '\n']) {
            return Err(InstallError::BadProxy {
                reason: "代理地址不能包含空白或 `@`；凭据请单独填写".to_string(),
            });
        }
        Ok(())
    }
}

/// 把 URL 里的凭据替换成 `***`（npm 输出落日志前的兜底掩码）。
pub fn strip_url_credentials(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if input[index..].starts_with("://") {
            output.push_str("://");
            index += 3;
            let start = index;
            let mut end = index;
            while end < bytes.len()
                && !matches!(
                    bytes[end],
                    b'/' | b' ' | b'\t' | b'\n' | b'\r' | b'"' | b'\'' | b','
                )
            {
                end += 1;
            }
            let authority = &input[start..end];
            match authority.rfind('@') {
                Some(at) => {
                    output.push_str("***");
                    output.push_str(&authority[at..]);
                }
                None => output.push_str(authority),
            }
            index = end;
        } else {
            let ch = input[index..].chars().next().unwrap_or('\u{fffd}');
            output.push(ch);
            index += ch.len_utf8();
        }
    }
    output
}

/// 日志与界面共用的掩码器：先做通用 URL 凭据替换，再逐字抹掉已知秘密。
#[derive(Debug, Clone, Default)]
pub struct Masker {
    secrets: Vec<String>,
}

impl Masker {
    pub fn new(secrets: Vec<String>) -> Self {
        Self {
            secrets: secrets
                .into_iter()
                .filter(|secret| !secret.is_empty())
                .collect(),
        }
    }

    pub fn apply(&self, line: &str) -> String {
        let mut masked = strip_url_credentials(line);
        for secret in &self.secrets {
            if masked.contains(secret.as_str()) {
                masked = masked.replace(secret.as_str(), "***");
            }
        }
        masked
    }
}

// ---------------------------------------------------------------------------
// 进度事件（B3 的载荷类型；事件推送接线在命令层）
// ---------------------------------------------------------------------------

/// 安装阶段；界面按它决定进度条文案与是否显示命令行明细。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallPhase {
    /// 校验安装位置与并发行。
    Preparing,
    /// 联网：向 registry 取包（npm pack）。
    Downloading,
    /// 联网：装进私有前缀（npm install）。
    Installing,
    /// 离线：校验离线包。
    Validating,
    /// 离线：展开到私有前缀。
    Extracting,
    /// 回读校验入口与版本。
    Verifying,
    /// 原子替换前缀并写入口。
    Activating,
    /// 终态成功。
    Done,
    /// 终态失败（会回滚，不改现有来源）。
    Failed,
}

impl InstallPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            InstallPhase::Preparing => "preparing",
            InstallPhase::Downloading => "downloading",
            InstallPhase::Installing => "installing",
            InstallPhase::Validating => "validating",
            InstallPhase::Extracting => "extracting",
            InstallPhase::Verifying => "verifying",
            InstallPhase::Activating => "activating",
            InstallPhase::Done => "done",
            InstallPhase::Failed => "failed",
        }
    }

    /// 是否属于「联网安装的命令行明细」阶段（`UI规范` §18.2）。
    pub fn shows_command_lines(self) -> bool {
        matches!(self, InstallPhase::Downloading | InstallPhase::Installing)
    }
}

/// 一条进度事件（`FZ-48`：**不落盘**，只在运行期推送）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InstallProgress {
    pub phase: InstallPhase,
    pub percent: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<String>,
}

/// 进度出口；命令层实现它并把事件转发成 Tauri 事件。
pub trait InstallProgressSink: Send + Sync {
    fn emit(&self, progress: InstallProgress);
}

/// 什么都不做的出口（非交互安装使用）。
pub struct NullProgressSink;

impl InstallProgressSink for NullProgressSink {
    fn emit(&self, _progress: InstallProgress) {}
}

/// 收集进度到内存，供测试断言阶段顺序与掩码效果。
#[derive(Debug, Default)]
pub struct RecordingProgressSink {
    events: std::sync::Mutex<Vec<InstallProgress>>,
}

impl RecordingProgressSink {
    pub fn events(&self) -> Vec<InstallProgress> {
        self.events
            .lock()
            .map(|events| events.clone())
            .unwrap_or_default()
    }

    pub fn lines(&self) -> Vec<String> {
        self.events()
            .into_iter()
            .filter_map(|event| event.line)
            .collect()
    }
}

impl InstallProgressSink for RecordingProgressSink {
    fn emit(&self, progress: InstallProgress) {
        if let Ok(mut events) = self.events.lock() {
            events.push(progress);
        }
    }
}

// ---------------------------------------------------------------------------
// 取消
// ---------------------------------------------------------------------------

/// 跨线程的取消标志；`FZ-50` 要求取消后终止子进程并清理临时前缀。
#[derive(Debug, Clone, Default)]
pub struct CancelFlag(Arc<AtomicBool>);

impl CancelFlag {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::SeqCst)
    }
}

// ---------------------------------------------------------------------------
// 落地清单（`FZ-47`）
// ---------------------------------------------------------------------------

/// 安装源种类（清单里的 `source` 字段）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallSourceKind {
    Registry,
    Offline,
}

impl InstallSourceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            InstallSourceKind::Registry => "registry",
            InstallSourceKind::Offline => "offline",
        }
    }
}

/// `<前缀>/.runtime-manifest.json`；一级卸载的判定依据（`FZ-51`）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RuntimeManifest {
    pub package: String,
    pub version: String,
    pub tarball_sha256: String,
    pub installed_at: String,
    pub npm_path: String,
    pub scripts_enabled: bool,
    pub source: InstallSourceKind,
}

// ---------------------------------------------------------------------------
// 输入 / 输出
// ---------------------------------------------------------------------------

/// 安装源。
#[derive(Debug, Clone)]
pub enum InstallSource {
    /// 联网安装官方包；`version` 为 `latest` 或具体 semver。
    Registry {
        version: String,
        proxy: Option<ProxyConfig>,
    },
    /// 离线导入一个已下载的 `.tgz`。
    Offline { archive: PathBuf },
}

/// 一次安装请求；`prefix` 由命令层解析并做过初筛，这里仍会**再校验一次**。
#[derive(Debug, Clone)]
pub struct InstallRequest {
    pub prefix: PathBuf,
    pub source: InstallSource,
    /// 用户是否已在二次确认框里显式允许执行安装脚本（`FZ-50`）。
    pub allow_scripts: bool,
    /// 已发现的 npm 绝对路径；联网安装必需。
    pub npm_path: Option<PathBuf>,
    /// 已发现的 node 绝对路径；用于生成入口启动器。
    pub node_path: Option<PathBuf>,
}

/// 一次成功安装的事实。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallOutcome {
    pub package: String,
    pub version: String,
    pub target: PathBuf,
    pub entry: PathBuf,
    pub tarball_sha256: String,
    pub source: InstallSourceKind,
    pub scripts_enabled: bool,
    pub npm_path: String,
    pub installed_at: String,
    pub proxy_used: bool,
}

impl InstallOutcome {
    /// 代理进程正在运行时，新来源要等重启才生效（`FZ-48`）。
    pub fn restart_required(&self, proxy_running: bool) -> bool {
        proxy_running
    }

    pub fn history_entry(&self, result: &str, reason: Option<String>) -> InstallHistoryEntry {
        InstallHistoryEntry {
            action: "install".to_string(),
            result: result.to_string(),
            package: self.package.clone(),
            version: self.version.clone(),
            target: self.target.to_string_lossy().into_owned(),
            at: now_rfc3339(),
            reason,
        }
    }
}

// ---------------------------------------------------------------------------
// 错误
// ---------------------------------------------------------------------------

/// 安装失败的稳定分类；`code()` 是界面与日志共用的标识。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallError {
    /// 目标前缀被写路径校验拒绝。
    Target(PathRejection),
    /// 前缀被其它写者占用。
    Locked,
    /// 联网安装但找不到 npm。
    NpmMissing,
    /// 代理参数不合法。
    BadProxy { reason: String },
    /// 离线包不合规。
    Archive(ArchiveRejection),
    /// npm 以非零退出码结束。
    NpmFailed { exit_code: i32, detail: String },
    /// 包内 `name` 与官方包不一致。
    PackageNameMismatch { found: String },
    /// 包内 `version` 与预期不一致。
    VersionMismatch { expected: String, found: String },
    /// 入口缺失 / 不可执行，且用户未允许执行安装脚本。
    ScriptsRequired,
    /// 用户取消。
    Cancelled,
    /// 其它文件系统失败。
    FileSystem { operation: String, detail: String },
}

impl InstallError {
    pub fn code(&self) -> &'static str {
        match self {
            InstallError::Target(rejection) => rejection.code(),
            InstallError::Locked => "prefix_locked",
            InstallError::NpmMissing => "npm_missing",
            InstallError::BadProxy { .. } => "bad_proxy",
            InstallError::Archive(rejection) => rejection.code(),
            InstallError::NpmFailed { .. } => "npm_failed",
            InstallError::PackageNameMismatch { .. } => "package_name_mismatch",
            InstallError::VersionMismatch { .. } => "version_mismatch",
            InstallError::ScriptsRequired => "scripts_required",
            InstallError::Cancelled => "cancelled",
            InstallError::FileSystem { .. } => "filesystem",
        }
    }

    /// 用户可读的一句话；界面在此基础上补出口（如「去允许安装脚本」）。
    pub fn message(&self) -> String {
        match self {
            InstallError::Target(rejection) => match rejection {
                PathRejection::NotAbsolute => "安装位置必须是绝对路径".to_string(),
                PathRejection::Missing => "安装位置的父目录不存在".to_string(),
                PathRejection::NotRegularFile => "安装位置不是普通目录".to_string(),
                PathRejection::NotExecutable => "安装位置不可执行".to_string(),
                PathRejection::SymlinkComponent => "安装位置包含符号链接，已拒绝".to_string(),
                PathRejection::SystemProtected => "安装位置落在系统保护目录内，已拒绝".to_string(),
                PathRejection::NotDirectory => "安装位置不是目录".to_string(),
                PathRejection::NotWritable => "安装位置不可写".to_string(),
            },
            InstallError::Locked => "安装位置正被其它操作占用，请稍后重试".to_string(),
            InstallError::NpmMissing => "未发现可用的 npm；联网安装需要本机 npm".to_string(),
            InstallError::BadProxy { reason } => format!("代理设置不合法：{reason}"),
            InstallError::Archive(rejection) => {
                format!("离线包校验未通过：{}", rejection.code())
            }
            InstallError::NpmFailed { exit_code, .. } => {
                format!("npm 安装以退出码 {exit_code} 结束")
            }
            InstallError::PackageNameMismatch { found } => {
                format!("包名不是官方包 {OFFICIAL_PACKAGE}（实际 {found}）")
            }
            InstallError::VersionMismatch { expected, found } => {
                format!("版本不一致：期望 {expected}，实际 {found}")
            }
            InstallError::ScriptsRequired => {
                "该包需要执行安装脚本才能生成入口；需要你的二次确认".to_string()
            }
            InstallError::Cancelled => "安装已取消".to_string(),
            InstallError::FileSystem { operation, detail } => {
                format!("文件操作失败（{operation}）：{detail}")
            }
        }
    }
}

impl From<PathRejection> for InstallError {
    fn from(value: PathRejection) -> Self {
        InstallError::Target(value)
    }
}

impl From<ArchiveRejection> for InstallError {
    fn from(value: ArchiveRejection) -> Self {
        InstallError::Archive(value)
    }
}

impl From<AppError> for InstallError {
    fn from(value: AppError) -> Self {
        match value {
            AppError::TargetLockTimeout { .. } => InstallError::Locked,
            other => InstallError::FileSystem {
                operation: "runtime install".to_string(),
                detail: other.to_string(),
            },
        }
    }
}

impl From<InstallError> for AppError {
    fn from(value: InstallError) -> Self {
        AppError::RuntimeManaged {
            code: value.code().to_string(),
            detail: value.message(),
        }
    }
}

// ---------------------------------------------------------------------------
// npm 调用边界
// ---------------------------------------------------------------------------

/// `npm pack` 的输入。
#[derive(Debug, Clone)]
pub struct NpmPackRequest {
    pub npm_path: PathBuf,
    pub node_dir: Option<PathBuf>,
    pub version: String,
    pub proxy: Option<ProxyConfig>,
    /// 下载物的落地目录（临时区）。
    pub work_dir: PathBuf,
}

/// `npm install --prefix <prefix> @bitkyc08/opencodex@<version>` 的输入。
#[derive(Debug, Clone)]
pub struct NpmInstallRequest {
    pub npm_path: PathBuf,
    pub node_dir: Option<PathBuf>,
    pub prefix: PathBuf,
    /// 精确版本（不是 `latest`），保证前缀内容与校验过的 tarball 一致。
    pub version: String,
    pub proxy: Option<ProxyConfig>,
    /// 为真时**不**传 `--ignore-scripts`（会执行包内代码）。
    pub scripts_enabled: bool,
    /// 为真时传 `--offline`：离线导入只用 npm 缓存解析依赖，绝不联网（`FZ-49`）。
    pub offline: bool,
}

/// npm 的观测输出。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpmOutcome {
    pub exit_code: i32,
}

/// npm 调用边界；测试用假实现替换，生产用 [`SystemNpmRunner`]。
pub trait NpmRunner: Send + Sync {
    fn pack(
        &self,
        request: &NpmPackRequest,
        sink: &dyn InstallProgressSink,
        cancel: &CancelFlag,
    ) -> Result<PathBuf, InstallError>;

    fn install_package(
        &self,
        request: &NpmInstallRequest,
        sink: &dyn InstallProgressSink,
        cancel: &CancelFlag,
    ) -> Result<NpmOutcome, InstallError>;
}

/// 入口版本探针：装上之后**真的跑一次**入口读版本（`FZ-47` 的「版本可读」）。
///
/// 这一步不能省：官方包的口子入口在归档里是存在的，但它依赖 `bun` 等依赖
/// （由安装脚本 / 依赖安装准备）。只检查「文件存在且可执行」会把一个装上就跑不起来的
/// 前缀判成成功——安装脚本边界也就永远不会被触发。
pub trait VersionProbe: Send + Sync {
    /// 运行入口一次读取版本；读不到返回 `None`。
    fn probe(&self, node: Option<&Path>, script: &Path) -> Option<String>;
}

/// 真实探针：用绝对 node（有则用）跑 `<入口> --version`，带超时。
#[derive(Debug, Clone)]
pub struct RealVersionProbe {
    pub timeout: Duration,
}

impl Default for RealVersionProbe {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
        }
    }
}

impl VersionProbe for RealVersionProbe {
    fn probe(&self, node: Option<&Path>, script: &Path) -> Option<String> {
        let mut command = match node {
            Some(node) => {
                let mut command = Command::new(node);
                command.arg(script);
                command
            }
            None => Command::new(script),
        };
        command
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        let mut child = command.spawn().ok()?;
        let started = Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if started.elapsed() >= self.timeout {
                        // 探针是短命的自有子进程，超时必须收口；这与 FZ-10 约束的
                        // 官方生命周期子进程（只发 SIGTERM、不 SIGKILL）不是同一类。
                        let _ = child.kill();
                        let _ = child.wait();
                        return None;
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(_) => return None,
            }
        }
        let mut output = Vec::new();
        child.stdout.take()?.read_to_end(&mut output).ok()?;
        let text = String::from_utf8_lossy(&output);
        extract_semver(&text)
    }
}

/// 固定回复的探针（测试用），避免单测真的起子进程。
#[derive(Debug, Clone, Default)]
pub struct FixedVersionProbe {
    pub version: Option<String>,
}

impl VersionProbe for FixedVersionProbe {
    fn probe(&self, _node: Option<&Path>, _script: &Path) -> Option<String> {
        self.version.clone()
    }
}

/// 从任意输出里取出第一个 semver；取不到返回 `None`。
pub fn extract_semver(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index].is_ascii_digit() {
            let start = index;
            let mut end = index;
            while end < bytes.len() && (bytes[end].is_ascii_digit() || bytes[end] == b'.') {
                end += 1;
            }
            let candidate = &text[start..end];
            let parts: Vec<&str> = candidate.split('.').collect();
            if parts.len() >= 3
                && parts.iter().all(|part| !part.is_empty())
                && parts
                    .iter()
                    .all(|part| part.chars().all(|ch| ch.is_ascii_digit()))
            {
                return Some(candidate.to_string());
            }
            index = end.max(index + 1);
        } else {
            index += 1;
        }
    }
    None
}

/// 真实 npm 适配器：受控环境 + 可选代理 + 临时 `--userconfig`。
#[derive(Debug, Default)]
pub struct SystemNpmRunner {
    /// npm 需要的 `HOME`（缓存与日志）；缺失时使用进程默认 HOME。
    pub home: Option<PathBuf>,
}

impl SystemNpmRunner {
    pub fn new(home: Option<PathBuf>) -> Self {
        Self { home }
    }

    /// 受控命令：清空继承环境，只给 node / npm 目录与系统最小 PATH；
    /// **不注入应用持有的任何凭据环境变量**，也不继承 `npm_config_*`（`FZ-50`）。
    fn base_command(&self, npm_path: &Path, node_dir: Option<&Path>) -> Command {
        let mut command = Command::new(npm_path);
        command.env_clear();
        let path = match node_dir {
            Some(dir) if dir.is_absolute() => {
                format!("{}:{FALLBACK_PATH}", dir.to_string_lossy())
            }
            _ => FALLBACK_PATH.to_string(),
        };
        command.env("PATH", path);
        command.env("LANG", "C.UTF-8");
        command.env("LC_ALL", "C.UTF-8");
        command.env("npm_config_update_notifier", "false");
        command.env("npm_config_fund", "false");
        command.env("npm_config_audit", "false");
        command.env("npm_config_progress", "false");
        if let Some(home) = self.home.as_ref() {
            command.env("HOME", home);
            command.env("npm_config_cache", home.join(".npm"));
        }
        command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
        command
    }

    /// 代理相关参数；含凭据时改写临时 `--userconfig`（`0600`），凭据不进 argv。
    fn proxy_arguments(
        &self,
        proxy: &ProxyConfig,
        work_dir: &Path,
    ) -> Result<(Vec<std::ffi::OsString>, Option<TempUserconfig>), InstallError> {
        if !proxy.has_credential() {
            return Ok((
                vec![
                    std::ffi::OsString::from("--proxy"),
                    std::ffi::OsString::from(proxy.url_without_credential()),
                ],
                None,
            ));
        }
        let userconfig = TempUserconfig::create(work_dir, proxy)?;
        Ok((
            vec![
                std::ffi::OsString::from("--userconfig"),
                userconfig.path().as_os_str().to_os_string(),
            ],
            Some(userconfig),
        ))
    }
}

impl NpmRunner for SystemNpmRunner {
    fn pack(
        &self,
        request: &NpmPackRequest,
        sink: &dyn InstallProgressSink,
        cancel: &CancelFlag,
    ) -> Result<PathBuf, InstallError> {
        std::fs::create_dir_all(&request.work_dir).map_err(|error| InstallError::FileSystem {
            operation: "create npm download directory".to_string(),
            detail: error.to_string(),
        })?;
        let (proxy_args, _userconfig) = match request.proxy.as_ref() {
            Some(proxy) => self.proxy_arguments(proxy, &request.work_dir)?,
            None => (Vec::new(), None),
        };
        let masker = masker_for(request.proxy.as_ref());
        let spec = format!("{OFFICIAL_PACKAGE}@{}", request.version);

        let mut command = self.base_command(&request.npm_path, request.node_dir.as_deref());
        command
            .arg("pack")
            .arg(&spec)
            .arg("--pack-destination")
            .arg(&request.work_dir)
            .arg("--loglevel")
            .arg("info");
        command.args(&proxy_args);

        let (exit_code, tail) = run_streaming(
            command,
            sink,
            cancel,
            InstallPhase::Downloading,
            5,
            20,
            &masker,
        )?;
        if exit_code != 0 {
            return Err(InstallError::NpmFailed {
                exit_code,
                detail: npm_reason(&tail, "npm pack 未成功"),
            });
        }
        newest_tarball(&request.work_dir).ok_or_else(|| InstallError::NpmFailed {
            exit_code,
            detail: "npm pack 没有产出 tarball".to_string(),
        })
    }

    fn install_package(
        &self,
        request: &NpmInstallRequest,
        sink: &dyn InstallProgressSink,
        cancel: &CancelFlag,
    ) -> Result<NpmOutcome, InstallError> {
        std::fs::create_dir_all(&request.prefix).map_err(|error| InstallError::FileSystem {
            operation: "create install prefix".to_string(),
            detail: error.to_string(),
        })?;
        let (proxy_args, _userconfig) = match request.proxy.as_ref() {
            Some(proxy) => self.proxy_arguments(proxy, &request.prefix)?,
            None => (Vec::new(), None),
        };
        let masker = masker_for(request.proxy.as_ref());
        let spec = format!("{OFFICIAL_PACKAGE}@{}", request.version);

        let mut command = self.base_command(&request.npm_path, request.node_dir.as_deref());
        command
            .arg("install")
            .arg("--prefix")
            .arg(&request.prefix)
            .arg("--no-audit")
            .arg("--no-fund")
            .arg("--loglevel")
            .arg("info");
        if !request.scripts_enabled {
            command.arg("--ignore-scripts");
        }
        if request.offline {
            // 离线导入只用缓存解析依赖，绝不联网（`FZ-49`：离线包不联网）。
            command.arg("--offline");
        }
        command.args(&proxy_args);
        command.arg(&spec);

        let (exit_code, tail) = run_streaming(
            command,
            sink,
            cancel,
            InstallPhase::Installing,
            45,
            40,
            &masker,
        )?;
        if exit_code != 0 {
            return Err(InstallError::NpmFailed {
                exit_code,
                detail: npm_reason(&tail, "npm install 未成功"),
            });
        }
        Ok(NpmOutcome { exit_code })
    }
}

/// 把 npm 末尾输出压成一句可读原因（已掩码）；全空时回落到兜底文案。
fn npm_reason(tail: &[String], fallback: &str) -> String {
    let is_error = |line: &&String| {
        let trimmed = line.trim();
        trimmed.contains("npm error") || trimmed.contains("npm ERR")
    };
    // 优先给出 `npm error code XXX`（最可操作），否则任一错误行，最后才用末尾行兜底。
    if let Some(line) = tail
        .iter()
        .find(|line| line.trim().contains("npm error code"))
    {
        return line.trim().chars().take(300).collect();
    }
    if let Some(line) = tail.iter().find(is_error) {
        return line.trim().chars().take(300).collect();
    }
    tail.last()
        .map(|line| line.trim().chars().take(300).collect::<String>())
        .filter(|line| !line.is_empty())
        .unwrap_or_else(|| fallback.to_string())
}

fn masker_for(proxy: Option<&ProxyConfig>) -> Masker {
    match proxy {
        Some(proxy) => Masker::new(
            proxy
                .credential
                .as_ref()
                .map(|credential| vec![credential.secret.clone()])
                .unwrap_or_default(),
        ),
        None => Masker::new(Vec::new()),
    }
}

/// 临时 `--userconfig`：Unix `0600`，内容只含 `proxy` / `https-proxy` 两行；
/// **Drop 即删**，成功、失败与取消都走同一条清理路径（`FZ-50`）。
#[derive(Debug)]
struct TempUserconfig {
    path: PathBuf,
}

impl TempUserconfig {
    fn create(directory: &Path, proxy: &ProxyConfig) -> Result<Self, InstallError> {
        let url = proxy.url_with_credential().ok_or(InstallError::BadProxy {
            reason: "缺少代理凭据".to_string(),
        })?;
        std::fs::create_dir_all(directory).map_err(|error| InstallError::FileSystem {
            operation: "create proxy config directory".to_string(),
            detail: error.to_string(),
        })?;
        let path = directory.join(format!(".npmrc-proxy-{}", random_suffix()));
        let content = format!("proxy={url}\nhttps-proxy={url}\n");
        atomic_write(&path, content.as_bytes(), 0o600).map_err(InstallError::from)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempUserconfig {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

/// 跑一条子进程命令，逐行推送输出（已掩码），并按需响应取消与空闲提示。
///
/// 返回 `(退出码, 末尾若干行)`：末尾行用于把 npm 的真实原因带进失败分类
/// （例如离线导入时的 `ENOTCACHED`），而不是只报一个退出码。
fn run_streaming(
    mut command: Command,
    sink: &dyn InstallProgressSink,
    cancel: &CancelFlag,
    phase: InstallPhase,
    base_percent: u8,
    span: u8,
    masker: &Masker,
) -> Result<(i32, Vec<String>), InstallError> {
    let mut child = command.spawn().map_err(|error| InstallError::FileSystem {
        operation: "spawn npm".to_string(),
        detail: error.to_string(),
    })?;
    let (sender, receiver) = std::sync::mpsc::channel::<String>();
    if let Some(stdout) = child.stdout.take() {
        spawn_line_reader(stdout, sender.clone());
    }
    if let Some(stderr) = child.stderr.take() {
        spawn_line_reader(stderr, sender.clone());
    }
    drop(sender);

    let mut seen = 0u32;
    let mut tail: Vec<String> = Vec::new();
    let mut last_output = Instant::now();
    loop {
        if cancel.is_cancelled() {
            let _ = child.kill();
            let _ = child.wait();
            return Err(InstallError::Cancelled);
        }
        match receiver.recv_timeout(Duration::from_millis(150)) {
            Ok(line) => {
                seen = seen.saturating_add(1);
                last_output = Instant::now();
                let masked = masker.apply(&line);
                if !masked.trim().is_empty() {
                    tail.push(masked.clone());
                    if tail.len() > 12 {
                        tail.remove(0);
                    }
                }
                sink.emit(InstallProgress {
                    phase,
                    percent: percent_within(base_percent, span, seen),
                    line: Some(masked),
                });
            }
            Err(RecvTimeoutError::Timeout) => {
                if last_output.elapsed() >= IDLE_HINT {
                    last_output = Instant::now();
                    sink.emit(InstallProgress {
                        phase,
                        percent: percent_within(base_percent, span, seen),
                        line: Some("已 120 秒没有输出，命令可能卡住".to_string()),
                    });
                }
            }
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }
    let status = child.wait().map_err(|error| InstallError::FileSystem {
        operation: "wait for npm".to_string(),
        detail: error.to_string(),
    })?;
    Ok((status.code().unwrap_or(-1), tail))
}

fn spawn_line_reader<R: std::io::Read + Send + 'static>(reader: R, sender: Sender<String>) {
    std::thread::spawn(move || {
        for line in BufReader::new(reader).lines() {
            match line {
                Ok(line) => {
                    if sender.send(line).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });
}

/// 把行号粗略映射进 `[base, base + span]`：行数越多越接近上限，
/// 但永远不越界（真实百分比由阶段推进表达）。
fn percent_within(base: u8, span: u8, seen: u32) -> u8 {
    let step = seen.min(u32::from(span));
    base.saturating_add(step as u8)
}

fn newest_tarball(directory: &Path) -> Option<PathBuf> {
    let entries = std::fs::read_dir(directory).ok()?;
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("tgz") {
            continue;
        }
        let modified = entry
            .metadata()
            .and_then(|metadata| metadata.modified())
            .unwrap_or(std::time::UNIX_EPOCH);
        if best
            .as_ref()
            .map(|(time, _)| modified >= *time)
            .unwrap_or(true)
        {
            best = Some((modified, path));
        }
    }
    best.map(|(_, path)| path)
}

/// 6 字节随机十六进制；用于临时区与临时配置文件名。
fn random_suffix() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 6];
    rand::rng().fill_bytes(&mut bytes);
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

// ---------------------------------------------------------------------------
// 安装执行
// ---------------------------------------------------------------------------

/// 临时区在 Drop 时递归清理；落位成功后 `disarm`，避免误删已生效的前缀。
#[derive(Debug)]
struct TempPrefixGuard {
    path: PathBuf,
    armed: bool,
}

impl TempPrefixGuard {
    fn new(path: PathBuf) -> Self {
        Self { path, armed: true }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for TempPrefixGuard {
    fn drop(&mut self) {
        if self.armed {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

/// 回读校验通过后的包事实。
#[derive(Debug, Clone, PartialEq, Eq)]
struct VerifiedPackage {
    version: String,
    /// 相对包根的口子入口（已去掉前导 `./`）。
    bin_entry: String,
    /// 入口的绝对路径（已确认存在且可执行）。
    bin_path: PathBuf,
}

/// 装填阶段（离线 / 联网）产出的事实。
#[derive(Debug, Clone)]
struct Filled {
    tarball_sha256: String,
    scripts_enabled: bool,
    source: InstallSourceKind,
    npm_path: String,
}

/// 托管安装执行器。所有平台交互都经注入的边界，便于隔离测试。
pub struct RuntimeInstaller<'a> {
    pub data_root: &'a Path,
    pub npm: &'a dyn NpmRunner,
    /// 用于写路径校验的符号链接检查范围（通常是用户主目录）。
    pub home: Option<PathBuf>,
    pub sink: &'a dyn InstallProgressSink,
    pub cancel: CancelFlag,
    /// 入口版本探针（`FZ-47` 的「版本可读」检查）。
    pub probe: &'a dyn VersionProbe,
    /// 已发现的 node 绝对路径；探针与入口启动器都用它。
    pub node_path: Option<PathBuf>,
}

impl<'a> RuntimeInstaller<'a> {
    pub fn new(
        data_root: &'a Path,
        npm: &'a dyn NpmRunner,
        home: Option<PathBuf>,
        sink: &'a dyn InstallProgressSink,
        probe: &'a dyn VersionProbe,
        node_path: Option<PathBuf>,
    ) -> Self {
        Self {
            data_root,
            npm,
            home,
            sink,
            cancel: CancelFlag::new(),
            probe,
            node_path,
        }
    }

    fn node(&self) -> Option<&Path> {
        self.node_path.as_deref()
    }

    /// 跑一次 npm 安装；遇到 `ENOTEMPTY` 就清理并**重试一次**。
    ///
    /// 真实链路里在**冷缓存**下必现：npm 把平台包（如 `@oven/bun-darwin-arm64`）
    /// 重命名进 `node_modules` 时撞上同名目录，直接以 `ENOTEMPTY` 失败。
    /// 重试前把上一次的产物移走，npm 面对干净前缀即可成功。重试次数固定为 1，
    /// 不做无限重试，失败仍按真实原因上报。
    fn install_with_retry(
        &self,
        temp: &Path,
        request: &NpmInstallRequest,
    ) -> Result<(), InstallError> {
        match self.npm.install_package(request, self.sink, &self.cancel) {
            Ok(_) => Ok(()),
            Err(InstallError::NpmFailed { ref detail, .. }) if detail.contains("ENOTEMPTY") => {
                self.emit(
                    InstallPhase::Installing,
                    40,
                    Some(
                        "npm 在重命名依赖时撞到同名目录（ENOTEMPTY），已清理并重试一次".to_string(),
                    ),
                );
                Self::reset_install_output(temp)?;
                self.npm
                    .install_package(request, self.sink, &self.cancel)
                    .map(|_| ())
            }
            Err(other) => Err(other),
        }
    }

    /// 带脚本重跑前清掉上一次的安装产物。
    ///
    /// 真实链路里这里踩过 npm `ENOTEMPTY`：第二次 `npm install` 要在已有
    /// `node_modules` 上重命名目录，npm 会直接失败。重跑是「重新装一遍」，
    /// 所以先把上一次的产物移走，让 npm 面对一个干净的前缀。
    fn reset_install_output(prefix: &Path) -> Result<(), InstallError> {
        let node_modules = prefix.join("node_modules");
        if node_modules.exists() {
            std::fs::remove_dir_all(&node_modules).map_err(|error| InstallError::FileSystem {
                operation: "reset node_modules before scripts retry".to_string(),
                detail: error.to_string(),
            })?;
        }
        for name in [PROJECT_PACKAGE_FILENAME, "package-lock.json"] {
            let path = prefix.join(name);
            if path.is_file() {
                let _ = std::fs::remove_file(path);
            }
        }
        Ok(())
    }

    /// 执行一次安装。失败时**不改动现有前缀与运行来源**。
    pub fn install(&self, request: &InstallRequest) -> Result<InstallOutcome, InstallError> {
        self.emit(InstallPhase::Preparing, 2, None);
        self.check_cancel()?;
        ensure_parent_directory(&request.prefix)?;
        let prefix = paths::validate_install_target(&request.prefix, self.home.as_deref())?;
        let _lock = TargetFileLock::lock(&prefix).map_err(InstallError::from)?;

        let temp = sibling_with_tag(&prefix, "new")?;
        let mut guard = TempPrefixGuard::new(temp.clone());
        std::fs::create_dir_all(&temp).map_err(|error| InstallError::FileSystem {
            operation: "create temporary prefix".to_string(),
            detail: error.to_string(),
        })?;

        let filled = match &request.source {
            InstallSource::Offline { archive } => self.fill_offline(request, &temp, archive)?,
            InstallSource::Registry { version, proxy } => {
                self.fill_registry(request, &temp, version, proxy.as_ref())?
            }
        };
        self.check_cancel()?;

        let expected = match &request.source {
            InstallSource::Registry { version, .. } if version != DEFAULT_VERSION => {
                Some(version.as_str())
            }
            _ => None,
        };
        self.emit(InstallPhase::Verifying, 88, None);
        let verified = verify_package(&temp, expected, self.probe, self.node())?;

        let installed_at = now_rfc3339();
        let manifest = RuntimeManifest {
            package: OFFICIAL_PACKAGE.to_string(),
            version: verified.version.clone(),
            tarball_sha256: filled.tarball_sha256.clone(),
            installed_at: installed_at.clone(),
            npm_path: filled.npm_path.clone(),
            scripts_enabled: filled.scripts_enabled,
            source: filled.source,
        };
        self.write_manifest(&temp, &manifest)?;

        self.emit(InstallPhase::Activating, 95, None);
        atomic_replace_prefix(&temp, &prefix)?;
        guard.disarm();

        // 入口必须指向**落位后**的最终路径：临时区名字里带随机后缀，
        // 若把临时路径写进启动器，替换之后入口就指向一个已经不存在的目录
        // （真实链路里表现为入口 `MODULE_NOT_FOUND`）。
        let entry = self.data_root.join(MANAGED_ENTRY_RELATIVE);
        let final_bin = prefix.join(PACKAGE_SUBPATH).join(&verified.bin_entry);
        write_entry(
            &entry,
            request.node_path.as_deref().or(self.node()),
            &final_bin,
        )?;

        self.emit(InstallPhase::Done, 100, None);
        Ok(InstallOutcome {
            package: OFFICIAL_PACKAGE.to_string(),
            version: verified.version,
            target: prefix,
            entry,
            tarball_sha256: filled.tarball_sha256,
            source: filled.source,
            scripts_enabled: filled.scripts_enabled,
            npm_path: filled.npm_path,
            installed_at,
            proxy_used: matches!(
                &request.source,
                InstallSource::Registry { proxy: Some(_), .. }
            ),
        })
    }

    fn fill_offline(
        &self,
        request: &InstallRequest,
        temp: &Path,
        archive: &Path,
    ) -> Result<Filled, InstallError> {
        self.emit(InstallPhase::Validating, 8, None);
        // 先做 `FZ-49` 校验：读遍每一条，越界 / 链接 / 超限一律整体拒绝，
        // **在任何展开动作之前**就把不合格的归档挡掉。
        let inspection = archive::inspect(archive, None)?;
        self.check_cancel()?;
        self.emit(
            InstallPhase::Extracting,
            25,
            Some(format!("展开离线包（{} 个条目）", inspection.entries)),
        );
        // 加固展开（逐条规范化 + 拒绝链接条目 + 保留可执行位）。
        let package_root = temp.join(PACKAGE_SUBPATH);
        archive::extract(archive, None, &package_root)?;
        write_project_package(temp, &inspection.version)?;

        // 官方包**不含** node_modules（依赖由 registry 提供），所以光展开是跑不起来的。
        // 因此离线导入仍需 npm 补齐依赖与入口链接，但一律走 `--offline`：
        // 只读本机缓存，绝不联网（用户的离线前提不被破坏）。
        // npm 缺失时直接阻断，而不是交出一个装上就跑不起来的前缀。
        let npm_path = request.npm_path.clone().ok_or(InstallError::NpmMissing)?;
        let node_dir = self
            .node_path
            .as_ref()
            .and_then(|node| node.parent().map(Path::to_path_buf));

        self.install_with_retry(
            temp,
            &NpmInstallRequest {
                npm_path: npm_path.clone(),
                node_dir: node_dir.clone(),
                prefix: temp.to_path_buf(),
                // 本地归档路径即安装源；`--offline` 保证只用缓存。
                version: archive.to_string_lossy().into_owned(),
                proxy: None,
                scripts_enabled: false,
                offline: true,
            },
        )?;
        self.check_cancel()?;

        let mut scripts_enabled = false;
        if let Err(error) = verify_package(temp, Some(&inspection.version), self.probe, self.node())
        {
            if error != InstallError::ScriptsRequired || !request.allow_scripts {
                return Err(error);
            }
            scripts_enabled = true;
            self.emit(
                InstallPhase::Installing,
                62,
                Some("按你的确认带安装脚本重跑一次".to_string()),
            );
            Self::reset_install_output(temp)?;
            self.npm.install_package(
                &NpmInstallRequest {
                    npm_path: npm_path.clone(),
                    node_dir,
                    prefix: temp.to_path_buf(),
                    version: archive.to_string_lossy().into_owned(),
                    proxy: None,
                    scripts_enabled: true,
                    offline: true,
                },
                self.sink,
                &self.cancel,
            )?;
            self.check_cancel()?;
        }

        Ok(Filled {
            tarball_sha256: inspection.sha256,
            scripts_enabled,
            source: InstallSourceKind::Offline,
            npm_path: npm_path.to_string_lossy().into_owned(),
        })
    }

    fn fill_registry(
        &self,
        request: &InstallRequest,
        temp: &Path,
        version: &str,
        proxy: Option<&ProxyConfig>,
    ) -> Result<Filled, InstallError> {
        let npm_path = request.npm_path.clone().ok_or(InstallError::NpmMissing)?;
        if let Some(proxy) = proxy {
            proxy.validate()?;
        }
        let node_dir = request
            .node_path
            .as_ref()
            .and_then(|node| node.parent().map(Path::to_path_buf));

        let downloads = temp.join(".downloads");
        let tarball = self.npm.pack(
            &NpmPackRequest {
                npm_path: npm_path.clone(),
                node_dir: node_dir.clone(),
                version: version.to_string(),
                proxy: proxy.cloned(),
                work_dir: downloads,
            },
            self.sink,
            &self.cancel,
        )?;
        self.check_cancel()?;

        // 下载物同样过一遍 `FZ-49` 校验：包名、版本与入口都以此为准。
        let inspection = archive::inspect(&tarball, None)?;

        self.emit(InstallPhase::Installing, 45, None);
        self.install_with_retry(
            temp,
            &NpmInstallRequest {
                npm_path: npm_path.clone(),
                node_dir: node_dir.clone(),
                prefix: temp.to_path_buf(),
                version: inspection.version.clone(),
                proxy: proxy.cloned(),
                scripts_enabled: false,
                offline: false,
            },
        )?;
        self.check_cancel()?;

        let mut scripts_enabled = false;
        if let Err(error) = verify_package(temp, Some(&inspection.version), self.probe, self.node())
        {
            if error != InstallError::ScriptsRequired || !request.allow_scripts {
                return Err(error);
            }
            // 用户显式允许后才带脚本重跑一次（`FZ-50`）。高风险：会执行包内代码。
            scripts_enabled = true;
            self.emit(
                InstallPhase::Installing,
                62,
                Some("按你的确认带安装脚本重跑一次".to_string()),
            );
            Self::reset_install_output(temp)?;
            self.npm.install_package(
                &NpmInstallRequest {
                    npm_path: npm_path.clone(),
                    node_dir,
                    prefix: temp.to_path_buf(),
                    version: inspection.version.clone(),
                    proxy: proxy.cloned(),
                    scripts_enabled: true,
                    offline: false,
                },
                self.sink,
                &self.cancel,
            )?;
            self.check_cancel()?;
        }

        Ok(Filled {
            tarball_sha256: inspection.sha256,
            scripts_enabled,
            source: InstallSourceKind::Registry,
            npm_path: npm_path.to_string_lossy().into_owned(),
        })
    }

    fn write_manifest(
        &self,
        prefix: &Path,
        manifest: &RuntimeManifest,
    ) -> Result<(), InstallError> {
        let payload =
            serde_json::to_vec_pretty(manifest).map_err(|error| InstallError::FileSystem {
                operation: "serialize runtime manifest".to_string(),
                detail: error.to_string(),
            })?;
        atomic_write(&prefix.join(MANIFEST_FILENAME), &payload, 0o644).map_err(InstallError::from)
    }

    fn emit(&self, phase: InstallPhase, percent: u8, line: Option<String>) {
        self.sink.emit(InstallProgress {
            phase,
            percent,
            line,
        });
    }

    fn check_cancel(&self) -> Result<(), InstallError> {
        if self.cancel.is_cancelled() {
            return Err(InstallError::Cancelled);
        }
        Ok(())
    }
}

/// 回读 `node_modules/@bitkyc08/opencodex/package.json` 校验包事实，
/// 并用探针**真的跑一次入口读版本**（`FZ-47` 的「入口可执行 + 版本可读」）。
fn verify_package(
    prefix: &Path,
    expected: Option<&str>,
    probe: &dyn VersionProbe,
    node: Option<&Path>,
) -> Result<VerifiedPackage, InstallError> {
    let package_root = prefix.join(PACKAGE_SUBPATH);
    let raw = std::fs::read(package_root.join("package.json")).map_err(|error| {
        InstallError::FileSystem {
            operation: "read installed package.json".to_string(),
            detail: error.to_string(),
        }
    })?;
    let json: serde_json::Value =
        serde_json::from_slice(&raw).map_err(|error| InstallError::FileSystem {
            operation: "parse installed package.json".to_string(),
            detail: error.to_string(),
        })?;

    let name = json
        .get("name")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    if name != OFFICIAL_PACKAGE {
        return Err(InstallError::PackageNameMismatch { found: name });
    }
    let version = json
        .get("version")
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string();
    if version.is_empty() {
        return Err(InstallError::VersionMismatch {
            expected: expected.unwrap_or(DEFAULT_VERSION).to_string(),
            found: "(缺失)".to_string(),
        });
    }
    if let Some(expected) = expected {
        if version != expected {
            return Err(InstallError::VersionMismatch {
                expected: expected.to_string(),
                found: version,
            });
        }
    }

    let bin_entry = archive::resolve_bin(&json).ok_or(InstallError::ScriptsRequired)?;
    let relative = normalize_bin(&bin_entry);
    if relative.is_empty() {
        return Err(InstallError::ScriptsRequired);
    }
    let bin_path = package_root.join(&relative);
    if !is_executable_file(&bin_path) {
        // 入口缺失 / 不可执行：可能是包内脚本尚未运行（`FZ-50`）。
        return Err(InstallError::ScriptsRequired);
    }

    // 「版本可读」：真的跑一次入口。装上却跑不起来的前缀不能算装好——
    // 官方包的 bun 依赖就是在这一步暴露出来的（安装脚本未跑时入口会说缺依赖）。
    let probed = probe.probe(node, &bin_path);
    let Some(probed) = probed else {
        return Err(InstallError::ScriptsRequired);
    };
    if let Some(expected) = expected {
        if probed != expected {
            return Err(InstallError::VersionMismatch {
                expected: expected.to_string(),
                found: probed,
            });
        }
    }

    Ok(VerifiedPackage {
        version: probed,
        bin_entry: relative,
        bin_path,
    })
}

/// 去掉 `bin` 里的前导 `./`，并拒绝试图逃出包根的写法。
fn normalize_bin(entry: &str) -> String {
    let trimmed = entry.trim().trim_start_matches("./");
    if trimmed.is_empty() || trimmed.starts_with('/') || trimmed.split('/').any(|part| part == "..")
    {
        return String::new();
    }
    trimmed.to_string()
}

fn is_executable_file(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        std::fs::metadata(path)
            .map(|metadata| metadata.is_file())
            .unwrap_or(false)
    }
}

/// 前缀根的私有项目 `package.json`：让前缀等价一个可重装的 npm 项目。
fn write_project_package(prefix: &Path, version: &str) -> Result<(), InstallError> {
    let payload = serde_json::json!({
        "name": "opencodex-managed",
        "private": true,
        "version": "0.0.0",
        "dependencies": { OFFICIAL_PACKAGE: version },
    });
    let bytes = serde_json::to_vec_pretty(&payload).map_err(|error| InstallError::FileSystem {
        operation: "serialize project package.json".to_string(),
        detail: error.to_string(),
    })?;
    atomic_write(&prefix.join(PROJECT_PACKAGE_FILENAME), &bytes, 0o644).map_err(InstallError::from)
}

/// 稳定入口 `<数据根>/runtime/bin/ocx`：绝对 node + 包内入口的启动器。
///
/// 不指向 `node_modules/.bin`（`FZ-47`），也不依赖 PATH。
fn write_entry(entry: &Path, node: Option<&Path>, script: &Path) -> Result<(), InstallError> {
    let content = entry_script(node, script);
    if let Some(parent) = entry.parent() {
        std::fs::create_dir_all(parent).map_err(|error| InstallError::FileSystem {
            operation: "create runtime bin directory".to_string(),
            detail: error.to_string(),
        })?;
    }
    atomic_write(entry, content.as_bytes(), 0o755).map_err(InstallError::from)
}

fn entry_script(node: Option<&Path>, script: &Path) -> String {
    let script = shell_quote(script);
    match node {
        Some(node) => format!(
            "#!/bin/sh\n# OpenCodex 托管入口：由 OpenCodeX 桌面管理器生成，卸载托管包体时一并移除。\nexec {} {} \"$@\"\n",
            shell_quote(node),
            script
        ),
        None => format!(
            "#!/bin/sh\n# OpenCodex 托管入口：由桌面管理器生成；未记录 node 绝对路径，回落 env node。\nexec env node {} \"$@\"\n",
            script
        ),
    }
}

fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}

/// 原子落位：现有前缀先改名让位，新前缀就位后再删旧；失败则把旧前缀改回来。
fn atomic_replace_prefix(temp: &Path, prefix: &Path) -> Result<(), InstallError> {
    if !prefix.exists() {
        return std::fs::rename(temp, prefix).map_err(|error| InstallError::FileSystem {
            operation: "activate temporary prefix".to_string(),
            detail: error.to_string(),
        });
    }
    let retired = sibling_with_tag(prefix, "old")?;
    std::fs::rename(prefix, &retired).map_err(|error| InstallError::FileSystem {
        operation: "retire existing prefix".to_string(),
        detail: error.to_string(),
    })?;
    match std::fs::rename(temp, prefix) {
        Ok(()) => {
            let _ = std::fs::remove_dir_all(&retired);
            Ok(())
        }
        Err(error) => {
            // 回滚：旧前缀必须回到原位，否则用户会「装了个空的」。
            let _ = std::fs::rename(&retired, prefix);
            Err(InstallError::FileSystem {
                operation: "activate temporary prefix".to_string(),
                detail: error.to_string(),
            })
        }
    }
}

/// `<父目录>` 必须存在；不存在就建出来（默认前缀在数据根内，属我们的地盘）。
fn ensure_parent_directory(prefix: &Path) -> Result<(), InstallError> {
    let parent = prefix.parent().ok_or(InstallError::FileSystem {
        operation: "resolve install parent".to_string(),
        detail: "安装位置没有父目录".to_string(),
    })?;
    std::fs::create_dir_all(parent).map_err(|error| InstallError::FileSystem {
        operation: "create install parent directory".to_string(),
        detail: error.to_string(),
    })
}

/// `<目标>.<tag>-<随机>`；与目标同父目录，保证 `rename` 是原子的。
fn sibling_with_tag(target: &Path, tag: &str) -> Result<PathBuf, InstallError> {
    let name = target.file_name().ok_or(InstallError::FileSystem {
        operation: "resolve prefix name".to_string(),
        detail: "安装位置没有目录名".to_string(),
    })?;
    let mut file_name = name.to_os_string();
    file_name.push(format!(".{tag}-{}", random_suffix()));
    Ok(target.with_file_name(file_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::runtime::archive::{self, ArchiveRejection};
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::Mutex;

    // ---- 测试夹具：造官方离线包 ----

    fn official_json(version: &str) -> serde_json::Value {
        serde_json::json!({
            "name": OFFICIAL_PACKAGE,
            "version": version,
            "bin": { "ocx": "bin/ocx.js" },
        })
    }

    fn build_tgz(
        dir: &Path,
        file_name: &str,
        package_json: serde_json::Value,
        extra: &[(&str, &[u8])],
    ) -> PathBuf {
        let path = dir.join(file_name);
        let file = std::fs::File::create(&path).expect("create tgz");
        let encoder = GzEncoder::new(file, Compression::fast());
        let mut builder = tar::Builder::new(encoder);
        let payload = serde_json::to_vec_pretty(&package_json).expect("serialize package.json");
        let mut header = tar::Header::new_gnu();
        header.set_size(payload.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "package/package.json", payload.as_slice())
            .expect("append package.json");
        for (name, body) in extra {
            let mut header = tar::Header::new_gnu();
            header.set_size(body.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            builder
                .append_data(&mut header, *name, *body)
                .expect("append entry");
        }
        builder
            .into_inner()
            .expect("finish tar")
            .finish()
            .expect("finish gzip");
        path
    }

    fn official_package(dir: &Path, version: &str) -> PathBuf {
        build_tgz(
            dir,
            &format!("bitkyc08-opencodex-{version}.tgz"),
            official_json(version),
            &[(
                "package/bin/ocx.js",
                b"#!/usr/bin/env node\nconsole.log('ocx')\n",
            )],
        )
    }

    /// 单测里的安装器：版本探针固定回复，避免真的起子进程。
    fn installer<'a>(
        data_root: &'a Path,
        npm: &'a dyn NpmRunner,
        sink: &'a dyn InstallProgressSink,
        home: Option<PathBuf>,
        probe: &'a dyn VersionProbe,
    ) -> RuntimeInstaller<'a> {
        RuntimeInstaller::new(data_root, npm, home, sink, probe, None)
    }

    /// 只在「第二次探针」才成功的探针，用来复现官方包的真实形态：
    /// 入口文件在归档里就有，但依赖未装好时**跑不起来**。
    struct GatedProbe {
        calls: Mutex<usize>,
    }

    impl GatedProbe {
        fn new() -> Self {
            Self {
                calls: Mutex::new(0),
            }
        }
    }

    impl VersionProbe for GatedProbe {
        fn probe(&self, _node: Option<&Path>, _script: &Path) -> Option<String> {
            let mut calls = self.calls.lock().expect("probe lock");
            *calls += 1;
            if *calls >= 2 {
                Some("0.3.1".to_string())
            } else {
                None
            }
        }
    }

    fn prefix_of(data_root: &Path) -> PathBuf {
        data_root.join(super::super::MANAGED_PREFIX_RELATIVE)
    }

    // ---- 假 npm ----

    struct FakeNpm {
        tarball: PathBuf,
        /// 模拟「入口由安装脚本生成」：不带脚本时删除入口。
        require_scripts: bool,
        calls: Mutex<Vec<bool>>,
    }

    impl FakeNpm {
        fn new(tarball: PathBuf) -> Self {
            Self {
                tarball,
                require_scripts: false,
                calls: Mutex::new(Vec::new()),
            }
        }

        fn calls(&self) -> Vec<bool> {
            self.calls
                .lock()
                .map(|calls| calls.clone())
                .unwrap_or_default()
        }
    }

    impl NpmRunner for FakeNpm {
        fn pack(
            &self,
            request: &NpmPackRequest,
            sink: &dyn InstallProgressSink,
            _cancel: &CancelFlag,
        ) -> Result<PathBuf, InstallError> {
            std::fs::create_dir_all(&request.work_dir).map_err(|error| {
                InstallError::FileSystem {
                    operation: "fake pack".to_string(),
                    detail: error.to_string(),
                }
            })?;
            sink.emit(InstallProgress {
                phase: InstallPhase::Downloading,
                percent: 10,
                line: Some(format!("fake npm pack {}", request.version)),
            });
            let name = self
                .tarball
                .file_name()
                .ok_or(InstallError::NpmMissing)?
                .to_os_string();
            let dest = request.work_dir.join(name);
            std::fs::copy(&self.tarball, &dest).map_err(|error| InstallError::FileSystem {
                operation: "fake pack copy".to_string(),
                detail: error.to_string(),
            })?;
            Ok(dest)
        }

        fn install_package(
            &self,
            request: &NpmInstallRequest,
            sink: &dyn InstallProgressSink,
            _cancel: &CancelFlag,
        ) -> Result<NpmOutcome, InstallError> {
            if let Ok(mut calls) = self.calls.lock() {
                calls.push(request.scripts_enabled);
            }
            sink.emit(InstallProgress {
                phase: InstallPhase::Installing,
                percent: 50,
                line: Some(format!("fake npm install {}", request.version)),
            });
            let package_root = request.prefix.join(PACKAGE_SUBPATH);
            archive::extract(&self.tarball, None, &package_root)?;
            if self.require_scripts && !request.scripts_enabled {
                let _ = std::fs::remove_file(package_root.join("bin/ocx.js"));
            }
            std::fs::write(request.prefix.join("package-lock.json"), b"{}").map_err(|error| {
                InstallError::FileSystem {
                    operation: "fake lock".to_string(),
                    detail: error.to_string(),
                }
            })?;
            Ok(NpmOutcome { exit_code: 0 })
        }
    }

    fn offline_request(prefix: PathBuf, archive: PathBuf, npm_path: PathBuf) -> InstallRequest {
        InstallRequest {
            prefix,
            source: InstallSource::Offline { archive },
            allow_scripts: false,
            npm_path: Some(npm_path),
            node_path: None,
        }
    }

    fn registry_request(prefix: PathBuf, version: &str, npm: &Path) -> InstallRequest {
        InstallRequest {
            prefix,
            source: InstallSource::Registry {
                version: version.to_string(),
                proxy: None,
            },
            allow_scripts: false,
            npm_path: Some(npm.to_path_buf()),
            node_path: None,
        }
    }

    // ---- 掩码与代理 ----

    #[test]
    fn strip_url_credentials_masks_userinfo_only() {
        assert_eq!(
            strip_url_credentials("proxy=http://alice:s3cret@host:8080/path"),
            "proxy=http://***@host:8080/path"
        );
        assert_eq!(
            strip_url_credentials("see https://example.com/a@b for details"),
            "see https://example.com/a@b for details"
        );
        assert_eq!(strip_url_credentials("no url here"), "no url here");
    }

    #[test]
    fn masker_also_removes_known_secret() {
        let masker = Masker::new(vec!["s3cret".to_string()]);
        assert_eq!(masker.apply("token=s3cret done"), "token=*** done");
        assert_eq!(
            masker.apply("http://alice:s3cret@host:1/x"),
            "http://***@host:1/x"
        );
    }

    #[test]
    fn proxy_forms_match_the_frozen_shape() {
        let plain = ProxyConfig::new(ProxyScheme::Http, "127.0.0.1:7890");
        assert_eq!(plain.url_without_credential(), "http://127.0.0.1:7890");
        assert_eq!(plain.masked(), "http://***@127.0.0.1:7890");
        assert!(plain.url_with_credential().is_none());
        assert!(plain.validate().is_ok());

        let socks = ProxyConfig::new(ProxyScheme::Socks5h, "127.0.0.1:1080")
            .with_credential("alice", "s3cret");
        assert_eq!(socks.url_without_credential(), "socks5h://127.0.0.1:1080");
        assert_eq!(
            socks.url_with_credential().as_deref(),
            Some("socks5h://alice:s3cret@127.0.0.1:1080")
        );
        assert!(socks.has_credential());
    }

    #[test]
    fn proxy_validation_rejects_empty_and_credential_hosts() {
        assert!(matches!(
            ProxyConfig::new(ProxyScheme::Http, "  ").validate(),
            Err(InstallError::BadProxy { .. })
        ));
        assert!(matches!(
            ProxyConfig::new(ProxyScheme::Http, "host:1\nx").validate(),
            Err(InstallError::BadProxy { .. })
        ));
        assert_eq!(ProxyScheme::parse("socks5"), Some(ProxyScheme::Socks5h));
        assert_eq!(ProxyScheme::parse("HTTP"), Some(ProxyScheme::Http));
        assert_eq!(ProxyScheme::parse("ftp"), None);
    }

    // ---- 临时 userconfig ----

    #[test]
    fn credential_proxy_never_reaches_the_command_line() {
        let dir = tempfile::tempdir().expect("temp");
        let runner = SystemNpmRunner::new(None);

        // 无凭据：argv 里是 `--proxy <scheme>://host:port>`，本来就不含凭据。
        let plain = ProxyConfig::new(ProxyScheme::Http, "host:8080");
        let (args, userconfig) = runner.proxy_arguments(&plain, dir.path()).expect("plain");
        assert_eq!(
            args,
            vec![
                std::ffi::OsString::from("--proxy"),
                std::ffi::OsString::from("http://host:8080"),
            ]
        );
        assert!(userconfig.is_none());

        // 含凭据：argv 只出现 `--userconfig <路径>`；用户名与口令都**不进** argv（`FZ-50`）。
        let secret =
            ProxyConfig::new(ProxyScheme::Socks5h, "host:1080").with_credential("alice", "s3cret");
        let (args, userconfig) = runner
            .proxy_arguments(&secret, dir.path())
            .expect("credential");
        let userconfig = userconfig.expect("含凭据必须改走临时 userconfig");
        let joined = args
            .iter()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(
            joined,
            format!("--userconfig {}", userconfig.path().display())
        );
        assert!(!joined.contains("s3cret"));
        assert!(!joined.contains("alice"));
        assert!(!joined.contains("--proxy"));
    }

    #[test]
    fn temp_userconfig_is_0600_and_removed_on_drop() {
        let dir = tempfile::tempdir().expect("temp");
        let proxy = ProxyConfig::new(ProxyScheme::Http, "host:8080").with_credential("a", "b");
        let path = {
            let config = TempUserconfig::create(dir.path(), &proxy).expect("create");
            let path = config.path().to_path_buf();
            let mode = std::fs::metadata(&path)
                .expect("metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
            let content = std::fs::read_to_string(&path).expect("read");
            assert!(content.contains("proxy=http://a:b@host:8080"));
            assert!(content.contains("https-proxy=http://a:b@host:8080"));
            path
        };
        assert!(!path.exists(), "临时 npmrc 必须用完即删");
    }

    // ---- 离线安装 ----

    #[test]
    fn offline_install_lands_prefix_manifest_and_entry() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let archive = official_package(root.path(), "0.3.1");
        let node = root.path().join("node");
        write_executable(&node);
        let npm_path = root.path().join("npm");
        write_executable(&npm_path);
        let sink = RecordingProgressSink::default();
        let npm = FakeNpm::new(archive.clone());
        let probe = FixedVersionProbe {
            version: Some("0.3.1".to_string()),
        };

        let mut request = offline_request(prefix_of(&data_root), archive, npm_path.clone());
        request.node_path = Some(node.clone());
        let outcome = installer(
            &data_root,
            &npm,
            &sink,
            Some(root.path().to_path_buf()),
            &probe,
        )
        .install(&request)
        .expect("offline install");

        assert_eq!(outcome.package, OFFICIAL_PACKAGE);
        assert_eq!(outcome.version, "0.3.1");
        assert_eq!(outcome.source, InstallSourceKind::Offline);
        assert!(!outcome.scripts_enabled);
        assert_eq!(outcome.tarball_sha256.len(), 64);
        // 官方包不含依赖，离线导入同样要经 npm（离线模式）补齐，因此记录了 npm 路径。
        assert_eq!(outcome.npm_path, npm_path.to_string_lossy());
        assert_eq!(outcome.target, prefix_of(&data_root));
        assert_eq!(
            outcome.entry,
            data_root.join(super::super::MANAGED_ENTRY_RELATIVE)
        );

        let prefix = prefix_of(&data_root);
        assert!(prefix
            .join("node_modules/@bitkyc08/opencodex/package.json")
            .is_file());
        assert!(prefix
            .join("node_modules/@bitkyc08/opencodex/bin/ocx.js")
            .is_file());
        assert!(prefix.join(PROJECT_PACKAGE_FILENAME).is_file());

        let manifest: RuntimeManifest = serde_json::from_slice(
            &std::fs::read(prefix.join(MANIFEST_FILENAME)).expect("manifest"),
        )
        .expect("parse manifest");
        assert_eq!(manifest.package, OFFICIAL_PACKAGE);
        assert_eq!(manifest.version, "0.3.1");
        assert_eq!(manifest.source, InstallSourceKind::Offline);
        assert!(!manifest.scripts_enabled);
        assert_eq!(manifest.tarball_sha256, outcome.tarball_sha256);

        let entry = outcome.entry;
        let content = std::fs::read_to_string(&entry).expect("entry");
        assert!(content.starts_with("#!/bin/sh"));
        assert!(
            content.contains(&format!("'{}'", node.to_string_lossy())),
            "入口必须用绝对 node 路径：{content}"
        );
        let mode = std::fs::metadata(&entry)
            .expect("metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o755);

        // 临时区不能留下残渣。
        let leftovers: Vec<_> = std::fs::read_dir(prefix.parent().expect("runtime dir"))
            .expect("read dir")
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".new-") || name.contains(".old-"))
            .collect();
        assert!(leftovers.is_empty(), "临时区未清理：{leftovers:?}");

        // 进度事件覆盖阶段顺序，且不落盘（这里只断言内存事件）。
        let phases: Vec<InstallPhase> = sink.events().into_iter().map(|e| e.phase).collect();
        assert!(phases.contains(&InstallPhase::Validating));
        assert!(phases.contains(&InstallPhase::Extracting));
        assert!(phases.contains(&InstallPhase::Activating));
        assert_eq!(phases.last(), Some(&InstallPhase::Done));
    }

    #[test]
    fn offline_install_rejects_bad_archive_and_keeps_existing_prefix() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let prefix = prefix_of(&data_root);
        std::fs::create_dir_all(&prefix).expect("mkdir prefix");
        std::fs::write(prefix.join("sentinel.txt"), b"keep me").expect("sentinel");

        let bad = root.path().join("opencodex-0.3.1.tgz");
        std::fs::write(&bad, b"not a gzip stream").expect("write");

        let sink = NullProgressSink;
        let npm = FakeNpm::new(official_package(root.path(), "0.3.1"));
        let probe = FixedVersionProbe {
            version: Some("0.3.1".to_string()),
        };
        let npm_path = root.path().join("npm");
        write_executable(&npm_path);
        let error = installer(
            &data_root,
            &npm,
            &sink,
            Some(root.path().to_path_buf()),
            &probe,
        )
        .install(&offline_request(prefix.clone(), bad, npm_path))
        .expect_err("bad archive must be rejected");
        assert_eq!(error, InstallError::Archive(ArchiveRejection::NotGzip));

        assert!(
            prefix.join("sentinel.txt").is_file(),
            "失败不得改动现有前缀"
        );
        assert!(!data_root
            .join(super::super::MANAGED_ENTRY_RELATIVE)
            .exists());
    }

    #[test]
    fn offline_install_rejects_non_official_package_name() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let mut json = official_json("0.3.1");
        json["name"] = serde_json::json!("evil-package");
        let archive = build_tgz(
            root.path(),
            "bitkyc08-opencodex-0.3.1.tgz",
            json,
            &[("package/bin/ocx.js", b"#!/usr/bin/env node\n")],
        );
        let sink = NullProgressSink;
        let npm = FakeNpm::new(archive.clone());
        let probe = FixedVersionProbe {
            version: Some("0.3.1".to_string()),
        };
        let npm_path = root.path().join("npm");
        write_executable(&npm_path);
        let error = installer(
            &data_root,
            &npm,
            &sink,
            Some(root.path().to_path_buf()),
            &probe,
        )
        .install(&offline_request(prefix_of(&data_root), archive, npm_path))
        .expect_err("非官方包必须拒绝");
        assert!(matches!(error, InstallError::Archive(_)));
        assert!(!prefix_of(&data_root).exists());
    }

    /// 回归：稳定入口必须指向**最终**前缀，不能指向 `<前缀>.new-<随机>`。
    /// 真实链路里这条写错会让装好的入口直接 `MODULE_NOT_FOUND`。
    #[test]
    fn stable_entry_points_at_the_final_prefix_not_the_temporary_one() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let archive = official_package(root.path(), "0.3.1");
        let sink = NullProgressSink;
        let npm = FakeNpm::new(archive.clone());
        let probe = FixedVersionProbe {
            version: Some("0.3.1".to_string()),
        };
        let npm_path = root.path().join("npm");
        write_executable(&npm_path);
        let node = root.path().join("node");
        write_executable(&node);

        let mut request = offline_request(prefix_of(&data_root), archive, npm_path);
        request.node_path = Some(node.clone());
        let outcome = installer(
            &data_root,
            &npm,
            &sink,
            Some(root.path().to_path_buf()),
            &probe,
        )
        .install(&request)
        .expect("install");

        let content = std::fs::read_to_string(&outcome.entry).expect("entry");
        assert!(
            content.contains(&format!(
                "{}/node_modules/@bitkyc08/opencodex/bin/ocx.js",
                outcome.target.display()
            )),
            "入口必须指向最终前缀：{content}"
        );
        assert!(!content.contains(".new-"), "入口不得引用临时区：{content}");
    }

    #[test]
    fn successful_install_replaces_previous_prefix() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let prefix = prefix_of(&data_root);
        std::fs::create_dir_all(&prefix).expect("mkdir prefix");
        std::fs::write(prefix.join("stale.txt"), b"old").expect("stale");

        let archive = official_package(root.path(), "0.3.1");
        let sink = NullProgressSink;
        let npm = FakeNpm::new(archive.clone());
        let probe = FixedVersionProbe {
            version: Some("0.3.1".to_string()),
        };
        let npm_path = root.path().join("npm");
        write_executable(&npm_path);
        installer(
            &data_root,
            &npm,
            &sink,
            Some(root.path().to_path_buf()),
            &probe,
        )
        .install(&offline_request(prefix.clone(), archive, npm_path))
        .expect("install");

        assert!(
            !prefix.join("stale.txt").exists(),
            "原子替换后旧前缀内容不应残留"
        );
        assert!(prefix.join(MANIFEST_FILENAME).is_file());
    }

    #[test]
    fn install_target_rejections_surface_system_protection() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let archive = official_package(root.path(), "0.3.1");
        let sink = NullProgressSink;
        let npm = FakeNpm::new(archive.clone());
        let probe = FixedVersionProbe {
            version: Some("0.3.1".to_string()),
        };
        let npm_path = root.path().join("npm");
        write_executable(&npm_path);
        let error = installer(
            &data_root,
            &npm,
            &sink,
            Some(root.path().to_path_buf()),
            &probe,
        )
        .install(&offline_request(
            PathBuf::from("/usr/lib/opencodex"),
            archive,
            npm_path,
        ))
        .expect_err("系统保护目录必须拒绝");
        assert_eq!(error, InstallError::Target(PathRejection::SystemProtected));
        assert_eq!(error.code(), "system_protected");
    }

    // ---- 联网安装 ----

    #[test]
    fn registry_install_uses_npm_and_records_provenance() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let archive = official_package(root.path(), "0.3.1");
        let npm_path = root.path().join("npm");
        write_executable(&npm_path);
        let sink = RecordingProgressSink::default();
        let npm = FakeNpm::new(archive);
        let probe = FixedVersionProbe {
            version: Some("0.3.1".to_string()),
        };

        let outcome = installer(
            &data_root,
            &npm,
            &sink,
            Some(root.path().to_path_buf()),
            &probe,
        )
        .install(&registry_request(
            prefix_of(&data_root),
            DEFAULT_VERSION,
            &npm_path,
        ))
        .expect("registry install");

        assert_eq!(outcome.version, "0.3.1");
        assert_eq!(outcome.source, InstallSourceKind::Registry);
        assert_eq!(outcome.npm_path, npm_path.to_string_lossy());
        assert!(!outcome.scripts_enabled);
        assert!(!outcome.proxy_used);
        assert_eq!(
            npm.calls(),
            vec![false],
            "默认必须以 --ignore-scripts 跑一次"
        );

        let manifest: RuntimeManifest = serde_json::from_slice(
            &std::fs::read(prefix_of(&data_root).join(MANIFEST_FILENAME)).expect("manifest"),
        )
        .expect("parse manifest");
        assert_eq!(manifest.source, InstallSourceKind::Registry);
        assert_eq!(manifest.npm_path, npm_path.to_string_lossy());
    }

    #[test]
    fn registry_install_requires_npm_path() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let archive = official_package(root.path(), "0.3.1");
        let sink = NullProgressSink;
        let npm = FakeNpm::new(archive);
        let probe = FixedVersionProbe {
            version: Some("0.3.1".to_string()),
        };
        let mut request = registry_request(
            prefix_of(&data_root),
            DEFAULT_VERSION,
            Path::new("/usr/bin/npm"),
        );
        request.npm_path = None;
        let error = installer(
            &data_root,
            &npm,
            &sink,
            Some(root.path().to_path_buf()),
            &probe,
        )
        .install(&request)
        .expect_err("缺少 npm 必须阻断");
        assert_eq!(error, InstallError::NpmMissing);
    }

    #[test]
    fn scripts_required_then_retried_only_after_explicit_consent() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let archive = official_package(root.path(), "0.3.1");
        let npm_path = root.path().join("npm");
        write_executable(&npm_path);
        let sink = RecordingProgressSink::default();
        let npm = FakeNpm {
            tarball: archive.clone(),
            require_scripts: true,
            calls: Mutex::new(Vec::new()),
        };

        let probe = FixedVersionProbe {
            version: Some("0.3.1".to_string()),
        };
        // 未确认：失败且不改动来源。
        let error = installer(
            &data_root,
            &npm,
            &sink,
            Some(root.path().to_path_buf()),
            &probe,
        )
        .install(&registry_request(
            prefix_of(&data_root),
            DEFAULT_VERSION,
            &npm_path,
        ))
        .expect_err("未确认时必须失败");
        assert_eq!(error, InstallError::ScriptsRequired);
        assert!(!prefix_of(&data_root).exists());
        assert_eq!(npm.calls(), vec![false]);

        // 确认后：带脚本重跑一次并成功。
        let mut request = registry_request(prefix_of(&data_root), DEFAULT_VERSION, &npm_path);
        request.allow_scripts = true;
        let outcome = installer(
            &data_root,
            &npm,
            &sink,
            Some(root.path().to_path_buf()),
            &probe,
        )
        .install(&request)
        .expect("确认后应成功");
        assert!(outcome.scripts_enabled);
        // 两次尝试累积：首次拒绝的 false，第二次「先 false 再带脚本 true」。
        assert_eq!(npm.calls(), vec![false, false, true]);
        let manifest: RuntimeManifest = serde_json::from_slice(
            &std::fs::read(prefix_of(&data_root).join(MANIFEST_FILENAME)).expect("manifest"),
        )
        .expect("parse manifest");
        assert!(manifest.scripts_enabled);
    }

    #[test]
    fn registry_install_rejects_explicit_version_mismatch() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let archive = official_package(root.path(), "0.3.1");
        let npm_path = root.path().join("npm");
        write_executable(&npm_path);
        let sink = NullProgressSink;
        let npm = FakeNpm::new(archive);
        let probe = FixedVersionProbe {
            version: Some("0.3.1".to_string()),
        };
        let error = installer(
            &data_root,
            &npm,
            &sink,
            Some(root.path().to_path_buf()),
            &probe,
        )
        .install(&registry_request(prefix_of(&data_root), "9.9.9", &npm_path))
        .expect_err("显式版本不符必须失败");
        assert!(matches!(error, InstallError::VersionMismatch { .. }));
        assert!(!prefix_of(&data_root).exists());
    }

    #[test]
    fn cancel_before_start_leaves_no_trace() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let archive = official_package(root.path(), "0.3.1");
        let sink = NullProgressSink;
        let npm = FakeNpm::new(archive.clone());
        let probe = FixedVersionProbe {
            version: Some("0.3.1".to_string()),
        };
        let npm_path = root.path().join("npm");
        write_executable(&npm_path);
        let target = installer(
            &data_root,
            &npm,
            &sink,
            Some(root.path().to_path_buf()),
            &probe,
        );
        target.cancel.cancel();
        let error = target
            .install(&offline_request(prefix_of(&data_root), archive, npm_path))
            .expect_err("已取消必须直接失败");
        assert_eq!(error, InstallError::Cancelled);
        assert!(!prefix_of(&data_root).exists());
    }

    // ---- 校验与结果 ----

    #[test]
    fn verify_package_rejects_name_and_version_and_missing_bin() {
        let root = tempfile::tempdir().expect("temp");
        let prefix = root.path().join("prefix");
        let package_root = prefix.join(PACKAGE_SUBPATH);
        std::fs::create_dir_all(package_root.join("bin")).expect("mkdir");
        let probe = FixedVersionProbe {
            version: Some("0.3.1".to_string()),
        };

        let write_json = |value: serde_json::Value| {
            std::fs::write(
                package_root.join("package.json"),
                serde_json::to_vec(&value).expect("serialize"),
            )
            .expect("write");
        };

        write_json(
            serde_json::json!({"name": "other", "version": "0.3.1", "bin": {"ocx": "bin/ocx.js"}}),
        );
        assert!(matches!(
            verify_package(&prefix, None, &probe, None),
            Err(InstallError::PackageNameMismatch { .. })
        ));

        std::fs::write(package_root.join("bin/ocx.js"), b"#!/bin/sh\n").expect("bin");
        std::fs::set_permissions(
            package_root.join("bin/ocx.js"),
            std::fs::Permissions::from_mode(0o755),
        )
        .expect("chmod");
        let probe = FixedVersionProbe {
            version: Some("0.3.1".to_string()),
        };
        write_json(official_json("0.3.1"));
        assert!(matches!(
            verify_package(&prefix, Some("9.9.9"), &probe, None),
            Err(InstallError::VersionMismatch { .. })
        ));
        let verified = verify_package(&prefix, Some("0.3.1"), &probe, None).expect("ok");
        assert_eq!(verified.bin_entry, "bin/ocx.js");
        // 版本以**探针实际读到的**为准，不是 package.json 里的自述。
        assert_eq!(verified.version, "0.3.1");

        std::fs::set_permissions(
            package_root.join("bin/ocx.js"),
            std::fs::Permissions::from_mode(0o644),
        )
        .expect("chmod");
        // 入口不可执行：直接判为「需要安装脚本」，连探针都不必跑。
        assert_eq!(
            verify_package(&prefix, None, &probe, None).unwrap_err(),
            InstallError::ScriptsRequired
        );
    }

    /// 真实世界的官方包形态：入口文件存在且可执行，但依赖未装好时**跑不起来**。
    /// 只检查文件会把这种前缀判成装好；必须靠「版本可读」探针拦下来（`FZ-47`）。
    #[test]
    fn unrunnable_entry_requires_scripts_and_is_retried_after_consent() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let archive = official_package(root.path(), "0.3.1");
        let npm_path = root.path().join("npm");
        write_executable(&npm_path);
        let sink = RecordingProgressSink::default();
        let npm = FakeNpm::new(archive.clone());
        let probe = GatedProbe::new();

        let error = installer(
            &data_root,
            &npm,
            &sink,
            Some(root.path().to_path_buf()),
            &probe,
        )
        .install(&registry_request(
            prefix_of(&data_root),
            DEFAULT_VERSION,
            &npm_path,
        ))
        .expect_err("跑不起来的前缀必须被判为需要脚本");
        assert_eq!(error, InstallError::ScriptsRequired);
        assert!(!prefix_of(&data_root).exists(), "失败不得留下半成品");
        assert_eq!(npm.calls(), vec![false]);

        // 第二次安装用「从第二次起就能读到版本」的探针，模拟脚本跑过之后入口可用的形态。
        let probe2 = GatedProbe {
            calls: Mutex::new(0),
        };
        let mut request = registry_request(prefix_of(&data_root), DEFAULT_VERSION, &npm_path);
        request.allow_scripts = true;
        let outcome = installer(
            &data_root,
            &npm,
            &sink,
            Some(root.path().to_path_buf()),
            &probe2,
        )
        .install(&request)
        .expect("确认后应成功");
        assert!(outcome.scripts_enabled);
        assert_eq!(outcome.version, "0.3.1");
        assert_eq!(npm.calls(), vec![false, false, true]);
    }

    #[test]
    fn extract_semver_reads_the_first_version_token() {
        assert_eq!(
            extract_semver("opencodex 2.50.0").as_deref(),
            Some("2.50.0")
        );
        assert_eq!(
            extract_semver("v1.2.3 (build 99)").as_deref(),
            Some("1.2.3")
        );
        assert_eq!(extract_semver("no version here"), None);
        assert_eq!(extract_semver("2.5"), None);
        assert_eq!(extract_semver(""), None);
    }

    #[test]
    fn npm_reason_prefers_the_error_line() {
        let tail = vec![
            "npm http fetch GET 200".to_string(),
            "npm error code ENOTCACHED".to_string(),
            "npm error request to registry failed".to_string(),
        ];
        assert_eq!(npm_reason(&tail, "fallback"), "npm error code ENOTCACHED");
        assert_eq!(npm_reason(&[], "fallback"), "fallback");
    }

    #[test]
    fn normalize_bin_strips_prefix_and_rejects_escapes() {
        assert_eq!(normalize_bin("./bin/ocx.mjs"), "bin/ocx.mjs");
        assert_eq!(normalize_bin("bin/ocx.js"), "bin/ocx.js");
        assert_eq!(normalize_bin("/etc/passwd"), "");
        assert_eq!(normalize_bin("../evil"), "");
        assert_eq!(normalize_bin("  "), "");
    }

    #[test]
    fn entry_script_falls_back_to_env_node_without_recorded_node() {
        let script =
            Path::new("/data/runtime/opencodex/node_modules/@bitkyc08/opencodex/bin/ocx.mjs");
        let with_node = entry_script(Some(Path::new("/usr/local/bin/node")), script);
        assert!(with_node.contains("exec '/usr/local/bin/node'"));
        let without = entry_script(None, script);
        assert!(without.contains("exec env node"));
        assert!(without.contains("bin/ocx.mjs"));
    }

    #[test]
    fn outcome_history_entry_and_restart_flag() {
        let outcome = InstallOutcome {
            package: OFFICIAL_PACKAGE.to_string(),
            version: "0.3.1".to_string(),
            target: PathBuf::from("/data/runtime/opencodex"),
            entry: PathBuf::from("/data/runtime/bin/ocx"),
            tarball_sha256: "a".repeat(64),
            source: InstallSourceKind::Offline,
            scripts_enabled: false,
            npm_path: String::new(),
            installed_at: "2026-09-25T00:00:00Z".to_string(),
            proxy_used: false,
        };
        let entry = outcome.history_entry("succeeded", None);
        assert_eq!(entry.action, "install");
        assert_eq!(entry.result, "succeeded");
        assert_eq!(entry.package, OFFICIAL_PACKAGE);
        assert_eq!(entry.version, "0.3.1");
        assert_eq!(entry.target, "/data/runtime/opencodex");
        assert!(!outcome.restart_required(false));
        assert!(outcome.restart_required(true));
    }

    #[test]
    fn install_error_maps_to_stable_codes() {
        assert_eq!(InstallError::NpmMissing.code(), "npm_missing");
        assert_eq!(InstallError::ScriptsRequired.code(), "scripts_required");
        assert_eq!(InstallError::Cancelled.code(), "cancelled");
        assert_eq!(
            InstallError::Target(PathRejection::NotWritable).code(),
            "not_writable"
        );
        assert_eq!(InstallError::Locked.code(), "prefix_locked");
        let app: AppError = InstallError::ScriptsRequired.into();
        assert!(matches!(app, AppError::RuntimeManaged { .. }));
        assert!(app.to_string().contains("scripts_required"));
    }

    fn write_executable(path: &Path) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("mkdir fixture");
        }
        std::fs::write(path, b"#!/bin/sh\nexit 0\n").expect("write fixture");
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
    }
}
