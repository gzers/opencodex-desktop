//! 托管卸载的两级判定（IMP `FZ-51`）。
//!
//! 一级（`Managed`）只删**安装记录登记的托管前缀**与稳定入口，删前必须能证明
//! 「这个目录确实是本产品装的」；二级（`Official`）**本产品不删任何文件**，
//! 只停代理、备份、二次确认，然后引导用户执行官方 `ocx uninstall`，
//! 并在之后只读展示官方命令留下的结果（掩码后）。
//!
//! 判定的严格程度是刻意的：卸载是本域唯一会**递归删除目录**的动作，
//! 宁可拒绝一次合法卸载，也不能删错一次。

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::{Deserialize, Serialize};

use super::install::{InstallSourceKind, Masker, RuntimeManifest};
use super::{now_rfc3339, MANAGED_ENTRY_RELATIVE, MANIFEST_FILENAME, OFFICIAL_PACKAGE};
use crate::errors::AppError;
use crate::infrastructure::hash::sha256_hex;
use crate::infrastructure::locking::TargetFileLock;
use crate::modules::backup::{backup_file, BackupAction};

/// 官方卸载记录文件名（`OPENCODEX_HOME` 下，由官方命令写出）。
pub const OFFICIAL_UNINSTALL_RECORD: &str = ".opencodex-uninstall.json";

/// 二次确认口令（保留：完整卸载仍要求逐字输入，见 `UI规范` §18.4）。
pub const OFFICIAL_CONFIRMATION_PHRASE: &str = "uninstall";

/// 官方包名：**硬编码**，任何移除命令都不得接受用户改写（`T-I11`）。
pub const REMOVABLE_PACKAGE: &str = OFFICIAL_PACKAGE;

/// 卸载范围（`FZ-51`，Revision 11 重定义）。
///
/// 轴由「托管 / 官方」改为「只删包体 / 连运行态一起删」：因为卸载现在覆盖
/// **任意已解析来源**（托管 / 用户指定 / 自动发现），归属不再等于范围。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UninstallScope {
    /// 仅移除包体与入口：不触碰 service / shim / config / `OPENCODEX_HOME`。
    Body,
    /// 完整卸载：官方 `ocx uninstall` → 移除包体与入口 → 可选清空 `OPENCODEX_HOME`。
    ///
    /// 官方命令必须先跑：外部 npm 全局来源的入口就是 `ocx`，先卸包会让官方命令无从执行。
    Full,
}

/// 卸载被拒绝的稳定原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UninstallError {
    /// 没有 `.runtime-manifest.json`，无法证明这个目录是本产品装的。
    NoManifest,
    /// 清单里的包名不是官方包。
    PackageMismatch { found: String },
    /// 清单里的版本与当前包体不一致（可能被外部工具改写）。
    ExternallyModified { expected: String, found: String },
    /// 目标前缀包含符号链接，或解析后不是记录路径。
    TargetMismatch { detail: String },
    /// 前缀正被其它操作占用。
    Locked,
    /// 缺少显式确认。
    ConfirmationRequired,
    /// 备份失败：勾选即承诺成功，故**中止**卸载（`T-I13`）。
    BackupFailed { detail: String },
    /// 外部命令执行失败（spawn 失败或非零退出）。
    CommandFailed {
        step: String,
        code: i32,
        detail: String,
    },
    /// 其它文件系统失败。
    FileSystem { operation: String, detail: String },
}

impl UninstallError {
    pub fn code(&self) -> &'static str {
        match self {
            UninstallError::NoManifest => "uninstall_no_manifest",
            UninstallError::PackageMismatch { .. } => "uninstall_package_mismatch",
            UninstallError::ExternallyModified { .. } => "uninstall_externally_modified",
            UninstallError::TargetMismatch { .. } => "uninstall_target_mismatch",
            UninstallError::Locked => "uninstall_locked",
            UninstallError::ConfirmationRequired => "uninstall_confirmation_required",
            UninstallError::BackupFailed { .. } => "uninstall_backup_failed",
            UninstallError::CommandFailed { .. } => "uninstall_command_failed",
            UninstallError::FileSystem { .. } => "uninstall_filesystem",
        }
    }

    pub fn message(&self) -> String {
        match self {
            UninstallError::NoManifest => {
                "该目录没有安装记录，无法确认由本应用安装，已拒绝删除".to_string()
            }
            UninstallError::PackageMismatch { found } => {
                format!("安装记录不是官方包 {OFFICIAL_PACKAGE}（记录为 {found}）")
            }
            UninstallError::ExternallyModified { expected, found } => format!(
                "前缀已被外部改写（记录 {expected}，实际 {found}），可能已被其它工具接管，已拒绝删除"
            ),
            UninstallError::TargetMismatch { detail } => {
                format!("安装位置校验未通过（{detail}），已拒绝删除")
            }
            UninstallError::Locked => "安装位置正被其它操作占用，请稍后重试".to_string(),
            UninstallError::ConfirmationRequired => "需要显式确认后才能卸载".to_string(),
            UninstallError::BackupFailed { detail } => {
                format!("备份失败，已中止卸载（未删除任何文件）：{detail}")
            }
            UninstallError::CommandFailed { step, code, detail } => {
                format!("执行 {step} 失败（退出码 {code}）：{detail}")
            }
            UninstallError::FileSystem { operation, detail } => {
                format!("文件操作失败（{operation}）：{detail}")
            }
        }
    }
}

impl From<AppError> for UninstallError {
    fn from(value: AppError) -> Self {
        match value {
            AppError::TargetLockTimeout { .. } => UninstallError::Locked,
            other => UninstallError::FileSystem {
                operation: "managed runtime uninstall".to_string(),
                detail: other.to_string(),
            },
        }
    }
}

impl From<UninstallError> for AppError {
    fn from(value: UninstallError) -> Self {
        AppError::RuntimeManaged {
            code: value.code().to_string(),
            detail: value.message(),
        }
    }
}

/// 一级卸载的结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedUninstallOutcome {
    pub target: PathBuf,
    pub entry: PathBuf,
    pub version: String,
    /// 备份 id；界面据此提供「重装同版本」出口。
    pub backup_id: String,
    /// 备份文件的落点（便于用户自行找回）。
    pub backup_directory: PathBuf,
    pub removed_prefix: bool,
    pub removed_entry: bool,
}

/// 校验通过的一级卸载目标：只删这个路径，不多删一个字。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedTarget {
    prefix: PathBuf,
    manifest: RuntimeManifest,
}

/// 证明「这个前缀确实是本产品装的」。
///
/// 四道检查缺一不可：清单存在、包名匹配、目标非符号链接且真实路径与记录一致、
/// 包内 `package.json` 的版本与清单一致（防外部改写）。
pub fn verify_managed_target(prefix: &Path) -> Result<VerifiedTarget, UninstallError> {
    // 只检查目标自身是不是符号链接：安装时已确保父目录没有符号链接，
    // 而这里若跟着符号链接走，删除动作就会落到别人身上（T-I8）。
    if std::fs::symlink_metadata(prefix)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(UninstallError::TargetMismatch {
            detail: "安装位置本身是符号链接".to_string(),
        });
    }
    let manifest_path = prefix.join(MANIFEST_FILENAME);
    let raw = std::fs::read(&manifest_path).map_err(|_| UninstallError::NoManifest)?;
    let manifest: RuntimeManifest =
        serde_json::from_slice(&raw).map_err(|_| UninstallError::NoManifest)?;
    if manifest.package != OFFICIAL_PACKAGE {
        return Err(UninstallError::PackageMismatch {
            found: manifest.package.clone(),
        });
    }

    let package_json = prefix
        .join(super::install::PACKAGE_SUBPATH)
        .join("package.json");
    let raw = std::fs::read(&package_json).map_err(|error| UninstallError::FileSystem {
        operation: "read managed package.json".to_string(),
        detail: error.to_string(),
    })?;
    let json: serde_json::Value =
        serde_json::from_slice(&raw).map_err(|error| UninstallError::FileSystem {
            operation: "parse managed package.json".to_string(),
            detail: error.to_string(),
        })?;
    let name = json
        .get("name")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    if name != OFFICIAL_PACKAGE {
        return Err(UninstallError::PackageMismatch {
            found: name.to_string(),
        });
    }
    let version = json
        .get("version")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    if version != manifest.version {
        return Err(UninstallError::ExternallyModified {
            expected: manifest.version.clone(),
            found: version.to_string(),
        });
    }

    Ok(VerifiedTarget {
        prefix: prefix.to_path_buf(),
        manifest,
    })
}

/// 一级卸载：备份 → 删前缀与入口 → 调用方刷新运行来源并写历史。
pub fn uninstall_managed(
    data_root: &Path,
    prefix: &Path,
    entry: &Path,
) -> Result<ManagedUninstallOutcome, UninstallError> {
    let verified = verify_managed_target(prefix)?;
    let _lock = TargetFileLock::lock(prefix).map_err(UninstallError::from)?;

    let now = chrono::Utc::now();
    let mut backup_id = String::new();
    let mut backup_directory = data_root.to_path_buf();

    // 备份口径：运行来源记录 + 前缀内的 `package.json` / `package-lock.json`
    // + 落地清单 + 锁文件；**不备份包体**（可按记录重装同版本）。
    let mut records: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    let runtime_record = data_root
        .join("manager-state")
        .join(super::RUNTIME_SOURCE_FILENAME);
    if let Ok(bytes) = std::fs::read(&runtime_record) {
        records.push((runtime_record, bytes));
    }
    for name in [
        super::install::PROJECT_PACKAGE_FILENAME.to_string(),
        "package-lock.json".to_string(),
        MANIFEST_FILENAME.to_string(),
    ] {
        let path = prefix.join(&name);
        if let Ok(bytes) = std::fs::read(&path) {
            records.push((path, bytes));
        }
    }
    let lock_path = crate::infrastructure::locking::lock_path_for(prefix);
    if let Ok(bytes) = std::fs::read(&lock_path) {
        records.push((lock_path, bytes));
    }

    for (path, bytes) in &records {
        let note = if backup_id.is_empty() {
            None
        } else {
            Some(format!("同一事务的附加文件（主备份 {backup_id}）"))
        };
        let record = backup_file(
            data_root,
            BackupAction::RuntimeUninstall,
            path,
            bytes,
            now,
            note,
        )
        .map_err(UninstallError::from)?;
        if backup_id.is_empty() {
            backup_id = record.manifest.backup_id.clone();
            backup_directory = record.directory.clone();
        }
    }

    let removed_prefix = verified.prefix.exists();
    if removed_prefix {
        std::fs::remove_dir_all(&verified.prefix).map_err(|error| UninstallError::FileSystem {
            operation: "remove managed prefix".to_string(),
            detail: error.to_string(),
        })?;
    }
    let removed_entry = entry.exists();
    if removed_entry {
        std::fs::remove_file(entry).map_err(|error| UninstallError::FileSystem {
            operation: "remove managed entry".to_string(),
            detail: error.to_string(),
        })?;
    }

    Ok(ManagedUninstallOutcome {
        target: verified.prefix,
        entry: entry.to_path_buf(),
        version: verified.manifest.version,
        backup_id,
        backup_directory,
        removed_prefix,
        removed_entry,
    })
}

/// 官方卸载「将移除的对象」清单：只读官方记录文件，不改写、不删除。
///
/// 官方把待移除路径写进 `OPENCODEX_HOME/config.json.managed-backup-*` 与
/// `.opencodex-uninstall.json`；两者都可能不存在（尚未执行过官方卸载）。
pub fn official_uninstall_objects(opencodex_home: &Path) -> Vec<String> {
    let mut objects = Vec::new();
    let Ok(entries) = std::fs::read_dir(opencodex_home) else {
        return objects;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name == OFFICIAL_UNINSTALL_RECORD || name.starts_with("config.json.managed-backup-") {
            objects.push(path.to_string_lossy().into_owned());
        }
    }
    objects.sort();
    objects
}

/// 只读展示官方卸载留下的（掩码后）结果；文件不存在时返回空。
///
/// `FZ-51` 的二级卸载「仅展示官方输出的结果与退出码」——本产品不执行该命令，
/// 因此这里读的是官方自己写下的记录。
pub fn official_uninstall_observation(opencodex_home: &Path) -> Vec<String> {
    let path = opencodex_home.join(OFFICIAL_UNINSTALL_RECORD);
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let masker = Masker::default();
    raw.lines()
        .filter(|line| !line.trim().is_empty())
        .take(200)
        .map(|line| masker.apply(line))
        .collect()
}

fn read_official_record_paths(opencodex_home: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let record = opencodex_home.join(OFFICIAL_UNINSTALL_RECORD);
    if record.is_file() {
        paths.push(record);
    }
    let Ok(entries) = std::fs::read_dir(opencodex_home) else {
        return paths;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|value| value.to_str()) {
            if name.starts_with("config.json.managed-backup-") {
                paths.push(path);
            }
        }
    }
    paths.sort();
    paths
}

fn shell_quote(path: &Path) -> String {
    format!("'{}'", path.to_string_lossy().replace('\'', "'\\''"))
}

/// 卸载历史条目（动作固定 `uninstall`）。
pub fn history_entry(
    result: &str,
    version: &str,
    target: &Path,
    reason: Option<String>,
) -> super::InstallHistoryEntry {
    super::InstallHistoryEntry {
        action: "uninstall".to_string(),
        result: result.to_string(),
        package: OFFICIAL_PACKAGE.to_string(),
        version: version.to_string(),
        target: target.to_string_lossy().into_owned(),
        at: now_rfc3339(),
        reason,
    }
}

/// 运行来源回退后的分类摘要，供界面在卸载完成后解释「现在用哪个 ocx」。
pub fn source_summary(kind: super::RuntimeSourceKind) -> &'static str {
    match kind {
        super::RuntimeSourceKind::Explicit => "用户指定",
        super::RuntimeSourceKind::Managed => "托管安装",
        super::RuntimeSourceKind::Discovered => "自动发现",
        super::RuntimeSourceKind::Unresolved => "未解析",
    }
}

/// 备份内容摘要：让界面能说清「备份了什么」，而不是只给一个 id。
pub fn backup_digest(files: &[String]) -> String {
    let mut hasher_input = files.join("\n");
    if hasher_input.is_empty() {
        hasher_input.push_str("(no files)");
    }
    sha256_hex(hasher_input.as_bytes())[..16].to_string()
}

/// 一级卸载的稳定入口路径（`<数据根>/runtime/bin/ocx`）。
pub fn managed_entry(data_root: &Path) -> PathBuf {
    data_root.join(MANAGED_ENTRY_RELATIVE)
}

/// 一级卸载登记前缀（默认 `<数据根>/runtime/opencodex`）。
pub fn managed_prefix(data_root: &Path) -> PathBuf {
    data_root.join(super::MANAGED_PREFIX_RELATIVE)
}

/// 清单里的来源种类只用于展示（离线安装也允许一级卸载）。
pub fn manifest_source_label(source: InstallSourceKind) -> &'static str {
    match source {
        InstallSourceKind::Registry => "registry",
        InstallSourceKind::Offline => "offline",
    }
}

// ── Revision 11：任意已解析来源的统一卸载（由应用执行；`T-I11` ~ `T-I15`） ──

/// 外部包体的移除动作。**命令与包名硬编码**，不接受用户改写（`T-I11`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalRemoval {
    /// npm 全局前缀下的安装：`npm uninstall -g @bitkyc08/opencodex`。
    NpmGlobal { prefix: PathBuf, entry: PathBuf },
    /// 用户指定位置里的包目录：只删「包目录 + bin 入口」两处。
    PackageDir {
        package_dir: PathBuf,
        entry: PathBuf,
    },
    /// 形态无法确认：**拒绝代删**，只展示路径与原因（`T-I11`）。
    Unsupported { reason: String },
}

/// 包体归属：决定谁移除（托管由应用直接删；外部由应用执行固定命令）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BodyOwner {
    Managed { prefix: PathBuf, entry: PathBuf },
    External(ExternalRemoval),
}

impl BodyOwner {
    pub fn is_managed(&self) -> bool {
        matches!(self, BodyOwner::Managed { .. })
    }
}

/// 非自有残留：**显式类别**（`T-I12`），不做通配、不递归猜测。
pub const RESIDUE_DIR_NAMES: &[&str] = &["integrations", "lab"];
pub const RESIDUE_EXACT_NAMES: &[&str] = &[
    "routing-history.sqlite",
    "routing-history.sqlite-shm",
    "routing-history.sqlite-wal",
    "codex-quota-cache.json",
];
pub const RESIDUE_PREFIX_NAMES: &[&str] = &[
    "catalog-backup-",
    "config.json.invalid-",
    "config.json.pre-",
];

/// 卸载方案（只读）：界面据此展示「将移除的对象」与残留候选。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallPlan {
    pub source_kind: super::RuntimeSourceKind,
    pub source_path: Option<String>,
    pub owner: BodyOwner,
    pub backup_id: Option<String>,
    pub remove_objects: Vec<String>,
    pub runtime_objects: Vec<String>,
    pub data_objects: Vec<String>,
    pub residue_candidates: Vec<String>,
}

impl UninstallPlan {
    /// 完整卸载要执行的官方命令（只读展示用）。
    pub fn official_command(&self, ocx: Option<&Path>) -> String {
        match ocx {
            Some(path) => format!("{} uninstall", shell_quote(path)),
            None => "ocx uninstall".to_string(),
        }
    }

    /// 外部包体的固定移除命令（只读展示用；`None` 表示无需/不能代执行）。
    pub fn external_command(&self) -> Option<String> {
        match &self.owner {
            BodyOwner::Managed { .. } => None,
            BodyOwner::External(ExternalRemoval::NpmGlobal { .. }) => {
                Some(format!("npm uninstall -g {REMOVABLE_PACKAGE}"))
            }
            BodyOwner::External(ExternalRemoval::PackageDir { package_dir, .. }) => {
                Some(format!("rm -rf {}", shell_quote(package_dir)))
            }
            BodyOwner::External(ExternalRemoval::Unsupported { .. }) => None,
        }
    }
}

/// 计算当前运行来源的卸载方案；**纯只读**，不做任何删除。
pub fn plan_uninstall(
    data_root: &Path,
    opencodex_home: &Path,
    kind: super::RuntimeSourceKind,
    path: Option<&Path>,
) -> UninstallPlan {
    let owner = classify_body_owner(data_root, kind, path);
    let mut remove_objects: Vec<String> = Vec::new();
    match &owner {
        BodyOwner::Managed { prefix, entry } => {
            remove_objects.push(prefix.to_string_lossy().into_owned());
            remove_objects.push(entry.to_string_lossy().into_owned());
        }
        BodyOwner::External(ExternalRemoval::NpmGlobal { entry, .. }) => {
            remove_objects.push(format!("npm 全局包 {REMOVABLE_PACKAGE}"));
            remove_objects.push(entry.to_string_lossy().into_owned());
        }
        BodyOwner::External(ExternalRemoval::PackageDir { package_dir, entry }) => {
            remove_objects.push(package_dir.to_string_lossy().into_owned());
            remove_objects.push(entry.to_string_lossy().into_owned());
        }
        BodyOwner::External(ExternalRemoval::Unsupported { .. }) => {}
    }
    let runtime_objects = vec![
        "service · 官方 launchd/systemd/WinSW 注册".to_string(),
        "codex autostart shim · 还原原生启动器".to_string(),
        "shell hook / 系统环境变量跟踪（macOS）".to_string(),
        "客户端 config · 恢复原生 Codex".to_string(),
    ];
    let data_objects = official_uninstall_objects(opencodex_home);
    let residue_candidates = residue_candidates(opencodex_home);
    UninstallPlan {
        source_kind: kind,
        source_path: path.map(|value| value.to_string_lossy().into_owned()),
        owner,
        backup_id: latest_uninstall_backup(data_root),
        remove_objects,
        runtime_objects,
        data_objects,
        residue_candidates,
    }
}

/// 判定包体归属；判定不出来一律 `Unsupported`（宁可不删，`T-I11`）。
fn classify_body_owner(
    data_root: &Path,
    kind: super::RuntimeSourceKind,
    path: Option<&Path>,
) -> BodyOwner {
    if kind == super::RuntimeSourceKind::Managed {
        return BodyOwner::Managed {
            prefix: managed_prefix(data_root),
            entry: managed_entry(data_root),
        };
    }
    let Some(path) = path else {
        return BodyOwner::External(ExternalRemoval::Unsupported {
            reason: "没有可用的安装路径".to_string(),
        });
    };
    let Some(package_dir) = package_dir_for(path) else {
        return BodyOwner::External(ExternalRemoval::Unsupported {
            reason: format!(
                "无法确认 {} 属于 {REMOVABLE_PACKAGE} 的包目录，已拒绝代删",
                path.display()
            ),
        });
    };
    match npm_prefix_for(&package_dir) {
        Some(prefix) => BodyOwner::External(ExternalRemoval::NpmGlobal {
            prefix,
            entry: path.to_path_buf(),
        }),
        None => BodyOwner::External(ExternalRemoval::PackageDir {
            package_dir,
            entry: path.to_path_buf(),
        }),
    }
}

/// 从 `ocx` 入口回溯它的包目录（`…/node_modules/@bitkyc08/opencodex`）。
fn package_dir_for(entry: &Path) -> Option<PathBuf> {
    let resolved = std::fs::canonicalize(entry).ok()?;
    for ancestor in resolved.ancestors() {
        let name = ancestor.file_name().and_then(|value| value.to_str())?;
        if name != "opencodex" {
            continue;
        }
        let scope = ancestor.parent()?;
        if scope.file_name().and_then(|value| value.to_str()) != Some("@bitkyc08") {
            continue;
        }
        let modules = scope.parent()?;
        if modules.file_name().and_then(|value| value.to_str()) != Some("node_modules") {
            continue;
        }
        return Some(ancestor.to_path_buf());
    }
    None
}

/// npm 前缀：`<prefix>/lib/node_modules/...` 或 `<prefix>/node_modules/...`。
fn npm_prefix_for(package_dir: &Path) -> Option<PathBuf> {
    let modules = package_dir.parent()?.parent()?;
    if modules.file_name().and_then(|value| value.to_str()) != Some("node_modules") {
        return None;
    }
    let base = modules.parent()?;
    if base.file_name().and_then(|value| value.to_str()) == Some("lib") {
        return base.parent().map(Path::to_path_buf);
    }
    Some(base.to_path_buf())
}

/// 最近一次可复用的卸载备份（`backups/<Y>/<M>/runtime-uninstall/<bk_*>`）。
pub fn latest_uninstall_backup(data_root: &Path) -> Option<String> {
    let root = data_root.join("backups");
    let mut found: Vec<String> = Vec::new();
    for year in std::fs::read_dir(&root).ok()?.flatten() {
        for month in std::fs::read_dir(year.path()).ok()?.flatten() {
            let dir = month.path().join("runtime-uninstall");
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.starts_with("bk_") {
                        found.push(name.to_string());
                    }
                }
            }
        }
    }
    found.sort();
    found.pop()
}

/// `OPENCODEX_HOME` 里「非官方自有」的残留候选（只读列举，`T-I12`）。
pub fn residue_candidates(opencodex_home: &Path) -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(opencodex_home) else {
        return out;
    };
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if is_residue_name(&name) {
            out.push(entry.path().to_string_lossy().into_owned());
        }
    }
    out.sort();
    out
}

fn is_residue_name(name: &str) -> bool {
    RESIDUE_DIR_NAMES.contains(&name)
        || RESIDUE_EXACT_NAMES.contains(&name)
        || RESIDUE_PREFIX_NAMES
            .iter()
            .any(|prefix| name.starts_with(prefix))
}

/// 卸载步骤结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepStatus {
    Ok,
    Skipped,
    Failed,
}

impl StepStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            StepStatus::Ok => "ok",
            StepStatus::Skipped => "skipped",
            StepStatus::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallStep {
    pub name: String,
    pub status: StepStatus,
    pub detail: Option<String>,
}

/// 残留核验三项态：`未知` **不得**被当成「已清除」（`T-I14`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResidueStatus {
    Cleared,
    Present,
    Unknown,
}

impl ResidueStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            ResidueStatus::Cleared => "cleared",
            ResidueStatus::Present => "present",
            ResidueStatus::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResidueItem {
    pub path: String,
    pub status: ResidueStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallOutcome {
    pub steps: Vec<UninstallStep>,
    pub backup_id: Option<String>,
    pub backup_directory: Option<PathBuf>,
    pub residue: Vec<ResidueItem>,
    /// 官方命令（掩码后）的输出行。
    pub official_output: Vec<String>,
}

impl UninstallOutcome {
    /// A zero exit code is not removal evidence. All observed targets must be
    /// absent, and at least one requested step must have actually completed.
    /// Intentionally retained HOME data is not part of residue_check's targets.
    pub fn verified_complete(&self) -> bool {
        self.steps.iter().any(|step| step.status == StepStatus::Ok)
            && self
                .steps
                .iter()
                .all(|step| step.status != StepStatus::Failed)
            && !self.residue.is_empty()
            && self
                .residue
                .iter()
                .all(|item| item.status == ResidueStatus::Cleared)
    }
}

/// 卸载选项（Revision 11）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UninstallOptions {
    pub scope: UninstallScope,
    /// 勾选即承诺成功：备份失败则**中止**卸载（`T-I13`）。
    pub auto_backup: bool,
    /// 是否清空 `OPENCODEX_HOME` 的非自有残留。
    pub clean_data: bool,
    /// 显式确认（界面勾选清单后回传）。
    pub confirmed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutcome {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// 外部命令执行面：真实实现走子进程，测试用桩（不触网、不执行真命令）。
pub trait UninstallCommands {
    fn ocx_uninstall(&self, ocx: &Path) -> Result<CommandOutcome, UninstallError>;
    fn npm_uninstall_global(&self, npm: &Path) -> Result<CommandOutcome, UninstallError>;
}

/// 真实命令执行：受控环境（`env_clear` + 最小 PATH + `HOME`），不继承凭据环境变量（`FZ-50`）。
#[derive(Debug, Default)]
pub struct SystemUninstallCommands {
    pub home: Option<PathBuf>,
}

impl SystemUninstallCommands {
    fn harden(&self, command: &mut Command, bin_dir: Option<&Path>) {
        command.env_clear();
        let path =
            crate::infrastructure::platform::controlled_path(bin_dir.map(Path::to_path_buf), None);
        command.env("PATH", path);
        command.env("LANG", "C.UTF-8");
        command.env("LC_ALL", "C.UTF-8");
        command.env("npm_config_update_notifier", "false");
        command.env("npm_config_fund", "false");
        command.env("npm_config_audit", "false");
        crate::infrastructure::platform::apply_user_environment(
            command,
            self.home.as_ref().map(|home| home.as_os_str()),
        );
        command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null());
    }
}

fn run_command(mut command: Command) -> Result<CommandOutcome, UninstallError> {
    let output = command
        .output()
        .map_err(|error| UninstallError::CommandFailed {
            step: "spawn".to_string(),
            code: -1,
            detail: error.to_string(),
        })?;
    Ok(CommandOutcome {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

impl UninstallCommands for SystemUninstallCommands {
    fn ocx_uninstall(&self, ocx: &Path) -> Result<CommandOutcome, UninstallError> {
        let mut command = Command::new(ocx);
        command.arg("uninstall");
        self.harden(&mut command, ocx.parent());
        run_command(command)
    }

    fn npm_uninstall_global(&self, npm: &Path) -> Result<CommandOutcome, UninstallError> {
        let mut command = Command::new(npm);
        command
            .arg("uninstall")
            .arg("-g")
            .arg(REMOVABLE_PACKAGE)
            .arg("--no-audit")
            .arg("--no-fund")
            .arg("--loglevel")
            .arg("info");
        self.harden(&mut command, npm.parent());
        run_command(command)
    }
}

fn step(name: &str, status: StepStatus, detail: Option<String>) -> UninstallStep {
    UninstallStep {
        name: name.to_string(),
        status,
        detail,
    }
}

fn first_line(text: &str) -> String {
    let masker = Masker::default();
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| masker.apply(line))
        .unwrap_or_else(|| "(无输出)".to_string())
}

fn masked_lines(stdout: &str, stderr: &str) -> Vec<String> {
    let masker = Masker::default();
    let mut out: Vec<String> = Vec::new();
    for line in stdout.lines().chain(stderr.lines()) {
        let trimmed = line.trim_end();
        if trimmed.trim().is_empty() {
            continue;
        }
        out.push(masker.apply(trimmed));
        if out.len() >= 200 {
            break;
        }
    }
    out
}

/// 备份将被动到的记录（运行来源记录 + 官方记录文件）；无可备份对象时返回占位 id。
fn backup_uninstall_state(
    data_root: &Path,
    opencodex_home: &Path,
    full: bool,
) -> Result<(String, PathBuf), UninstallError> {
    let now = chrono::Utc::now();
    let mut records: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    let runtime_record = data_root
        .join("manager-state")
        .join(super::RUNTIME_SOURCE_FILENAME);
    if let Ok(bytes) = std::fs::read(&runtime_record) {
        records.push((runtime_record, bytes));
    }
    if full {
        for path in read_official_record_paths(opencodex_home) {
            if let Ok(bytes) = std::fs::read(&path) {
                records.push((path, bytes));
            }
        }
    }
    if records.is_empty() {
        return Ok(("bk_none".to_string(), data_root.join("backups")));
    }
    let mut backup_id = String::new();
    let mut backup_directory = data_root.to_path_buf();
    for (path, bytes) in &records {
        let note = if backup_id.is_empty() {
            Some("卸载前的备份".to_string())
        } else {
            Some(format!("同一事务的附加文件（主备份 {backup_id}）"))
        };
        let record = backup_file(
            data_root,
            BackupAction::RuntimeUninstall,
            path,
            bytes,
            now,
            note,
        )
        .map_err(UninstallError::from)?;
        if backup_id.is_empty() {
            backup_id = record.manifest.backup_id.clone();
            backup_directory = record.directory.clone();
        }
    }
    Ok((backup_id, backup_directory))
}

/// 清空 `OPENCODEX_HOME` 的非自有残留：**显式类别** + 路径必须落在 home 内 + 不跟随符号链接（`T-I12`）。
pub fn clean_opencodex_residue(opencodex_home: &Path) -> Result<usize, UninstallError> {
    let Ok(root) = std::fs::canonicalize(opencodex_home) else {
        return Ok(0);
    };
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Ok(0);
    };
    let mut removed = 0usize;
    for entry in entries.flatten() {
        let Some(name) = entry.file_name().to_str().map(str::to_string) else {
            continue;
        };
        if !is_residue_name(&name) {
            continue;
        }
        let path = entry.path();
        let Ok(metadata) = std::fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_symlink() {
            std::fs::remove_file(&path).map_err(|error| UninstallError::FileSystem {
                operation: format!("remove symlink {}", path.display()),
                detail: error.to_string(),
            })?;
            removed += 1;
            continue;
        }
        if metadata.is_dir() {
            let Ok(canonical) = std::fs::canonicalize(&path) else {
                continue;
            };
            if !canonical.starts_with(&root) || canonical == root {
                return Err(UninstallError::TargetMismatch {
                    detail: format!("残留目录越出 OPENCODEX_HOME：{}", path.display()),
                });
            }
            std::fs::remove_dir_all(&path).map_err(|error| UninstallError::FileSystem {
                operation: format!("remove residue dir {}", path.display()),
                detail: error.to_string(),
            })?;
            removed += 1;
            continue;
        }
        std::fs::remove_file(&path).map_err(|error| UninstallError::FileSystem {
            operation: format!("remove residue file {}", path.display()),
            detail: error.to_string(),
        })?;
        removed += 1;
    }
    Ok(removed)
}

/// 只读残留核验：包体 / 入口 / service plist / `~/.codex` 内 OpenCodex 自有文件。
pub fn residue_check(data_root: &Path, plan: &UninstallPlan, full: bool) -> Vec<ResidueItem> {
    let mut targets: Vec<PathBuf> = Vec::new();
    match &plan.owner {
        BodyOwner::Managed { prefix, entry } => {
            targets.push(prefix.clone());
            targets.push(entry.clone());
        }
        BodyOwner::External(ExternalRemoval::NpmGlobal { prefix, entry }) => {
            targets.push(entry.clone());
            targets.push(
                prefix
                    .join("lib")
                    .join("node_modules")
                    .join("@bitkyc08")
                    .join("opencodex"),
            );
        }
        BodyOwner::External(ExternalRemoval::PackageDir { package_dir, entry }) => {
            targets.push(package_dir.clone());
            targets.push(entry.clone());
        }
        BodyOwner::External(ExternalRemoval::Unsupported { .. }) => {}
    }
    if full {
        if let Some(home) = crate::infrastructure::platform::home_dir() {
            targets.push(
                home.join("Library")
                    .join("LaunchAgents")
                    .join("com.opencodex.proxy.plist"),
            );
            for name in [
                ".opencodex-native-main.claim.sqlite",
                ".opencodex-native-main.owner.sqlite",
            ] {
                targets.push(home.join(".codex").join(name));
            }
        }
    }
    let _ = data_root;
    targets.sort();
    targets.dedup();
    targets
        .into_iter()
        .map(|path| {
            let status = match std::fs::symlink_metadata(&path) {
                Ok(_) => ResidueStatus::Present,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    ResidueStatus::Cleared
                }
                Err(_) => ResidueStatus::Unknown,
            };
            ResidueItem {
                path: path.to_string_lossy().into_owned(),
                status,
            }
        })
        .collect()
}

/// 统一卸载：停代理由调用方先行完成；这里按固定顺序执行并在失败处如实记录（`T-I11` ~ `T-I15`）。
pub fn execute_uninstall(
    data_root: &Path,
    opencodex_home: &Path,
    plan: &UninstallPlan,
    options: &UninstallOptions,
    ocx: Option<&Path>,
    npm: Option<&Path>,
    commands: &dyn UninstallCommands,
) -> Result<UninstallOutcome, UninstallError> {
    if !options.confirmed {
        return Err(UninstallError::ConfirmationRequired);
    }
    let full = options.scope == UninstallScope::Full;
    let mut steps: Vec<UninstallStep> = Vec::new();
    let mut backup_id: Option<String> = None;
    let mut backup_directory: Option<PathBuf> = None;
    let mut official_output: Vec<String> = Vec::new();

    if options.auto_backup {
        match backup_uninstall_state(data_root, opencodex_home, full) {
            Ok((id, dir)) => {
                backup_id = Some(id.clone());
                backup_directory = Some(dir);
                steps.push(step("生成备份", StepStatus::Ok, Some(format!("bk {id}"))));
            }
            Err(error) => {
                steps.push(step("生成备份", StepStatus::Failed, Some(error.message())));
                return Err(UninstallError::BackupFailed {
                    detail: error.message(),
                });
            }
        }
    } else {
        steps.push(step(
            "生成备份",
            StepStatus::Skipped,
            Some("未勾选自动备份；按约定不阻断卸载（卸载后不可恢复）".to_string()),
        ));
    }

    if full {
        match ocx {
            Some(path) => match commands.ocx_uninstall(path) {
                Ok(outcome) => {
                    official_output = masked_lines(&outcome.stdout, &outcome.stderr);
                    if outcome.code == 0 {
                        steps.push(step(
                            "执行官方 ocx uninstall",
                            StepStatus::Ok,
                            Some("官方已清理 service / shim / config 并恢复原生 Codex".to_string()),
                        ));
                    } else {
                        steps.push(step(
                            "执行官方 ocx uninstall",
                            StepStatus::Failed,
                            Some(format!(
                                "退出码 {}；官方输出按掩码展示，未重试、不代删",
                                outcome.code
                            )),
                        ));
                    }
                }
                Err(error) => steps.push(step(
                    "执行官方 ocx uninstall",
                    StepStatus::Failed,
                    Some(error.message()),
                )),
            },
            None => steps.push(step(
                "执行官方 ocx uninstall",
                StepStatus::Failed,
                Some("未解析到 ocx 可执行文件".to_string()),
            )),
        }
    }

    match &plan.owner {
        BodyOwner::Managed { prefix, entry } => match uninstall_managed(data_root, prefix, entry) {
            Ok(outcome) => {
                if backup_id.is_none() && !outcome.backup_id.is_empty() {
                    backup_id = Some(outcome.backup_id.clone());
                    backup_directory = Some(outcome.backup_directory.clone());
                }
                steps.push(step(
                    "移除包体与入口",
                    StepStatus::Ok,
                    Some(format!(
                        "{} · {}",
                        outcome.target.display(),
                        outcome.entry.display()
                    )),
                ));
            }
            Err(error) => steps.push(step(
                "移除包体与入口",
                StepStatus::Failed,
                Some(error.message()),
            )),
        },
        BodyOwner::External(ExternalRemoval::NpmGlobal { prefix, .. }) => match npm {
            Some(npm_path) => match commands.npm_uninstall_global(npm_path) {
                Ok(outcome) if outcome.code == 0 => steps.push(step(
                    "移除包体与入口",
                    StepStatus::Ok,
                    Some(format!(
                        "npm uninstall -g {REMOVABLE_PACKAGE}（前缀 {}）",
                        prefix.display()
                    )),
                )),
                Ok(outcome) => steps.push(step(
                    "移除包体与入口",
                    StepStatus::Failed,
                    Some(format!(
                        "npm 退出码 {}：{}",
                        outcome.code,
                        first_line(&outcome.stderr)
                    )),
                )),
                Err(error) => steps.push(step(
                    "移除包体与入口",
                    StepStatus::Failed,
                    Some(error.message()),
                )),
            },
            None => steps.push(step(
                "移除包体与入口",
                StepStatus::Failed,
                Some("未解析到 npm，无法代执行卸包命令".to_string()),
            )),
        },
        BodyOwner::External(ExternalRemoval::PackageDir { package_dir, entry }) => {
            match remove_external_package_dir(package_dir, entry) {
                Ok(()) => steps.push(step(
                    "移除包体与入口",
                    StepStatus::Ok,
                    Some(format!("{} · {}", package_dir.display(), entry.display())),
                )),
                Err(error) => steps.push(step(
                    "移除包体与入口",
                    StepStatus::Failed,
                    Some(error.message()),
                )),
            }
        }
        BodyOwner::External(ExternalRemoval::Unsupported { reason }) => steps.push(step(
            "移除包体与入口",
            StepStatus::Failed,
            Some(reason.clone()),
        )),
    }

    if full {
        if options.clean_data {
            match clean_opencodex_residue(opencodex_home) {
                Ok(count) => steps.push(step(
                    "清空 OPENCODEX_HOME 非自有残留",
                    StepStatus::Ok,
                    Some(format!("移除 {count} 项")),
                )),
                Err(error) => steps.push(step(
                    "清空 OPENCODEX_HOME 非自有残留",
                    StepStatus::Failed,
                    Some(error.message()),
                )),
            }
        } else {
            steps.push(step(
                "清空 OPENCODEX_HOME 非自有残留",
                StepStatus::Skipped,
                Some("未勾选；残留保留".to_string()),
            ));
        }
    }

    let residue = residue_check(data_root, plan, full);
    Ok(UninstallOutcome {
        steps,
        backup_id,
        backup_directory,
        residue,
        official_output,
    })
}

/// 用户指定位置：只删「包目录 + 入口」，且都必须是 `@bitkyc08/opencodex` 的形态（`T-I11`）。
fn remove_external_package_dir(package_dir: &Path, entry: &Path) -> Result<(), UninstallError> {
    let expected = package_dir.join("package.json");
    let raw = std::fs::read(&expected).map_err(|_| UninstallError::TargetMismatch {
        detail: format!("{} 没有 package.json，已拒绝代删", package_dir.display()),
    })?;
    let json: serde_json::Value =
        serde_json::from_slice(&raw).map_err(|error| UninstallError::FileSystem {
            operation: "parse external package.json".to_string(),
            detail: error.to_string(),
        })?;
    let name = json
        .get("name")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    if name != REMOVABLE_PACKAGE {
        return Err(UninstallError::PackageMismatch {
            found: name.to_string(),
        });
    }
    if std::fs::symlink_metadata(package_dir)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
    {
        return Err(UninstallError::TargetMismatch {
            detail: "包目录本身是符号链接".to_string(),
        });
    }
    if entry.exists() || std::fs::symlink_metadata(entry).is_ok() {
        let inside = std::fs::canonicalize(entry)
            .map(|canonical| canonical.starts_with(package_dir))
            .unwrap_or(false);
        if inside
            || std::fs::symlink_metadata(entry)
                .map(|m| m.file_type().is_symlink())
                .unwrap_or(false)
        {
            std::fs::remove_file(entry).map_err(|error| UninstallError::FileSystem {
                operation: format!("remove entry {}", entry.display()),
                detail: error.to_string(),
            })?;
        }
    }
    std::fs::remove_dir_all(package_dir).map_err(|error| UninstallError::FileSystem {
        operation: format!("remove package dir {}", package_dir.display()),
        detail: error.to_string(),
    })
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::modules::runtime::install::{
        InstallOutcome, InstallRequest, InstallSource, InstallSourceKind, NullProgressSink,
        RuntimeInstaller,
    };
    use crate::modules::runtime::{RuntimeHandle, RuntimeSourceKind};
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::os::unix::fs::PermissionsExt;

    /// 造一个官方离线包。
    fn official_tgz(dir: &Path, version: &str) -> PathBuf {
        let path = dir.join(format!("bitkyc08-opencodex-{version}.tgz"));
        let file = std::fs::File::create(&path).expect("create tgz");
        let encoder = GzEncoder::new(file, Compression::fast());
        let mut builder = tar::Builder::new(encoder);
        let payload = serde_json::to_vec_pretty(&serde_json::json!({
            "name": OFFICIAL_PACKAGE,
            "version": version,
            "bin": { "ocx": "bin/ocx.js" },
        }))
        .expect("serialize");
        let mut header = tar::Header::new_gnu();
        header.set_size(payload.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "package/package.json", payload.as_slice())
            .expect("append");
        let mut header = tar::Header::new_gnu();
        header.set_size(20);
        header.set_mode(0o755);
        header.set_cksum();
        builder
            .append_data(
                &mut header,
                "package/bin/ocx.js",
                &b"#!/usr/bin/env node\n"[..],
            )
            .expect("append");
        builder.into_inner().expect("tar").finish().expect("gz");
        path
    }

    /// 用真实安装内核装一次，得到一个「本产品装的」前缀。
    fn install_managed(data_root: &Path, scratch: &Path) -> InstallOutcome {
        let sink = NullProgressSink;
        let npm = FakeNpm {
            tarball: official_tgz(scratch, "0.3.1"),
            calls: std::sync::Mutex::new(Vec::new()),
        };
        let probe = crate::modules::runtime::install::FixedVersionProbe {
            version: Some("0.3.1".to_string()),
        };
        // 假 npm 的绝对路径：离线导入也要经 npm（离线模式）补齐依赖。
        let npm_path = scratch.join("npm");
        std::fs::write(&npm_path, b"#!/bin/sh\nexit 0\n").expect("npm fixture");
        std::fs::set_permissions(&npm_path, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        let installer = RuntimeInstaller::new(
            data_root,
            &npm,
            Some(scratch.to_path_buf()),
            &sink,
            &probe,
            None,
        );
        let request = InstallRequest {
            prefix: super::managed_prefix(data_root),
            source: InstallSource::Offline {
                archive: npm.tarball.clone(),
            },
            allow_scripts: false,
            npm_path: Some(npm_path),
            node_path: None,
        };
        installer.install(&request).expect("install")
    }

    /// 假 npm：按归档把包装进前缀（离线导入路径不再自己展开）。
    struct FakeNpm {
        tarball: PathBuf,
        calls: std::sync::Mutex<Vec<bool>>,
    }

    impl crate::modules::runtime::install::NpmRunner for FakeNpm {
        fn pack(
            &self,
            _request: &crate::modules::runtime::install::NpmPackRequest,
            _sink: &dyn crate::modules::runtime::install::InstallProgressSink,
            _cancel: &crate::modules::runtime::install::CancelFlag,
        ) -> Result<PathBuf, crate::modules::runtime::install::InstallError> {
            Ok(self.tarball.clone())
        }

        fn install_package(
            &self,
            request: &crate::modules::runtime::install::NpmInstallRequest,
            _sink: &dyn crate::modules::runtime::install::InstallProgressSink,
            _cancel: &crate::modules::runtime::install::CancelFlag,
        ) -> Result<
            crate::modules::runtime::install::NpmOutcome,
            crate::modules::runtime::install::InstallError,
        > {
            if let Ok(mut calls) = self.calls.lock() {
                calls.push(request.scripts_enabled);
            }
            let package_root = request.prefix.join(super::super::install::PACKAGE_SUBPATH);
            crate::modules::runtime::archive::extract(&self.tarball, None, &package_root)
                .map_err(crate::modules::runtime::install::InstallError::from)?;
            Ok(crate::modules::runtime::install::NpmOutcome { exit_code: 0 })
        }
    }

    #[test]
    fn managed_uninstall_removes_only_prefix_and_entry_and_backs_up() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let outcome = install_managed(&data_root, root.path());
        let entry = super::managed_entry(&data_root);
        assert!(outcome.target.is_dir() && entry.is_file());

        // 先写一份运行来源记录，验证它会被备份。
        let handle = RuntimeHandle::initialize(&data_root, Vec::new());
        assert_eq!(handle.kind(), RuntimeSourceKind::Managed);

        let result = uninstall_managed(&data_root, &outcome.target, &entry).expect("uninstall");
        assert_eq!(result.version, "0.3.1");
        assert!(result.removed_prefix && result.removed_entry);
        assert!(!result.backup_id.is_empty(), "必须给出可定位的备份 id");
        assert!(result.backup_directory.is_dir());
        assert!(!outcome.target.exists(), "前缀应被删除");
        assert!(!entry.exists(), "稳定入口应被删除");

        // 备份里能看到运行来源记录。
        let backed_up = std::fs::read_dir(&result.backup_directory)
            .expect("read backup")
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert!(
            backed_up.iter().any(|name| name == "runtime.json"),
            "运行来源记录必须进备份：{backed_up:?}"
        );

        // 卸载后运行来源回退到「未解析」（没有其它候选）。
        let after = handle.refresh();
        assert_eq!(after.kind, RuntimeSourceKind::Unresolved);
    }

    #[test]
    fn managed_uninstall_falls_back_to_discovered_source() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let outcome = install_managed(&data_root, root.path());
        let entry = super::managed_entry(&data_root);

        let discovered = root.path().join("home/.local/bin/ocx");
        std::fs::create_dir_all(discovered.parent().expect("parent")).expect("mkdir");
        std::fs::write(&discovered, b"#!/bin/sh\nexit 0\n").expect("write");
        std::fs::set_permissions(&discovered, std::fs::Permissions::from_mode(0o755))
            .expect("chmod");
        let handle = RuntimeHandle::initialize(&data_root, vec![discovered.clone()]);
        assert_eq!(handle.kind(), RuntimeSourceKind::Managed);

        uninstall_managed(&data_root, &outcome.target, &entry).expect("uninstall");
        let after = handle.refresh();
        assert_eq!(after.kind, RuntimeSourceKind::Discovered);
        assert_eq!(after.path, Some(discovered));
    }

    #[test]
    fn uninstall_refuses_directory_without_manifest() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let prefix = super::managed_prefix(&data_root);
        std::fs::create_dir_all(&prefix).expect("mkdir");
        std::fs::write(prefix.join("important.txt"), b"user data").expect("write");

        let error = uninstall_managed(&data_root, &prefix, &super::managed_entry(&data_root))
            .expect_err("没有安装记录必须拒绝");
        assert_eq!(error, UninstallError::NoManifest);
        assert!(
            prefix.join("important.txt").is_file(),
            "拒绝时不得删任何东西"
        );
    }

    #[test]
    fn uninstall_refuses_when_prefix_was_externally_modified() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let outcome = install_managed(&data_root, root.path());

        // 模拟外部工具换掉了包体版本。
        let package_json = outcome
            .target
            .join(super::super::install::PACKAGE_SUBPATH)
            .join("package.json");
        let mut json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&package_json).expect("read")).expect("parse");
        json["version"] = serde_json::json!("9.9.9");
        std::fs::write(&package_json, serde_json::to_vec(&json).expect("ser")).expect("write");

        let error = uninstall_managed(
            &data_root,
            &outcome.target,
            &super::managed_entry(&data_root),
        )
        .expect_err("外部改写必须拒绝");
        match error {
            UninstallError::ExternallyModified { expected, found } => {
                assert_eq!(expected, "0.3.1");
                assert_eq!(found, "9.9.9");
            }
            other => panic!("expected externally modified, got {other:?}"),
        }
        assert!(outcome.target.is_dir(), "拒绝时不得删除前缀");
    }

    #[test]
    fn uninstall_refuses_symlinked_prefix() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let outcome = install_managed(&data_root, root.path());

        let link = root.path().join("prefix-link");
        std::os::unix::fs::symlink(&outcome.target, &link).expect("symlink");
        let error = uninstall_managed(&data_root, &link, &super::managed_entry(&data_root))
            .expect_err("符号链接必须拒绝");
        assert!(matches!(error, UninstallError::TargetMismatch { .. }));
        assert!(outcome.target.is_dir());
    }

    #[derive(Default)]
    struct StubCommands {
        code: i32,
        stdout: String,
        stderr: String,
        calls: std::cell::RefCell<Vec<String>>,
    }

    impl StubCommands {
        fn ok() -> Self {
            Self {
                code: 0,
                stdout: "✅ service removed\n".to_string(),
                stderr: String::new(),
                calls: std::cell::RefCell::new(Vec::new()),
            }
        }
    }

    impl UninstallCommands for StubCommands {
        fn ocx_uninstall(&self, ocx: &Path) -> Result<CommandOutcome, UninstallError> {
            self.calls
                .borrow_mut()
                .push(format!("ocx {}", ocx.display()));
            Ok(CommandOutcome {
                code: self.code,
                stdout: self.stdout.clone(),
                stderr: self.stderr.clone(),
            })
        }

        fn npm_uninstall_global(&self, npm: &Path) -> Result<CommandOutcome, UninstallError> {
            self.calls
                .borrow_mut()
                .push(format!("npm {}", npm.display()));
            Ok(CommandOutcome {
                code: self.code,
                stdout: self.stdout.clone(),
                stderr: self.stderr.clone(),
            })
        }
    }

    /// 造一个 npm 全局前缀形态的安装：`<prefix>/lib/node_modules/@bitkyc08/opencodex`。
    fn install_npm_global(root: &Path, version: &str) -> (PathBuf, PathBuf) {
        let prefix = root.join("npm");
        let package_dir = prefix
            .join("lib")
            .join("node_modules")
            .join("@bitkyc08")
            .join("opencodex");
        std::fs::create_dir_all(package_dir.join("bin")).expect("mkdir");
        std::fs::write(
            package_dir.join("package.json"),
            serde_json::to_vec(&serde_json::json!({
                "name": OFFICIAL_PACKAGE,
                "version": version,
                "bin": { "ocx": "bin/ocx" },
            }))
            .expect("json"),
        )
        .expect("write package.json");
        let entry = package_dir.join("bin").join("ocx");
        std::fs::write(&entry, b"#!/bin/sh\n").expect("write entry");
        (
            std::fs::canonicalize(&prefix).expect("canonical prefix"),
            std::fs::canonicalize(&entry).expect("canonical entry"),
        )
    }

    #[test]
    fn plan_classifies_body_owner_for_every_resolved_source() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let home = root.path().join("opencodex-home");
        std::fs::create_dir_all(&home).expect("mkdir");

        let managed = plan_uninstall(&data_root, &home, RuntimeSourceKind::Managed, None);
        assert!(managed.owner.is_managed());

        let (prefix, entry) = install_npm_global(root.path(), "2.64.0");
        let plan = plan_uninstall(
            &data_root,
            &home,
            RuntimeSourceKind::Discovered,
            Some(&entry),
        );
        assert_eq!(
            plan.owner,
            BodyOwner::External(ExternalRemoval::NpmGlobal {
                prefix,
                entry: entry.clone()
            })
        );
        assert_eq!(
            plan.external_command().as_deref(),
            Some("npm uninstall -g @bitkyc08/opencodex")
        );

        // 认不出形态 → 明确拒绝代删，而不是猜。
        let stray = root.path().join("stray-ocx");
        std::fs::write(&stray, b"x").expect("write");
        let plan = plan_uninstall(&data_root, &home, RuntimeSourceKind::Explicit, Some(&stray));
        assert!(matches!(
            plan.owner,
            BodyOwner::External(ExternalRemoval::Unsupported { .. })
        ));
        assert!(plan.external_command().is_none());
        assert!(plan.remove_objects.is_empty());
    }

    #[test]
    fn execute_full_uninstall_runs_fixed_commands_and_reports_steps() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let home = root.path().join("opencodex-home");
        std::fs::create_dir_all(&home).expect("mkdir");
        std::fs::write(home.join("routing-history.sqlite"), b"x").expect("write residue");
        std::fs::write(home.join("config.json"), b"{}").expect("write config");
        let (_, entry) = install_npm_global(root.path(), "2.64.0");
        let plan = plan_uninstall(
            &data_root,
            &home,
            RuntimeSourceKind::Discovered,
            Some(&entry),
        );
        let commands = StubCommands::ok();

        let outcome = execute_uninstall(
            &data_root,
            &home,
            &plan,
            &UninstallOptions {
                scope: UninstallScope::Full,
                auto_backup: true,
                clean_data: true,
                confirmed: true,
            },
            Some(Path::new("/usr/local/bin/ocx")),
            Some(Path::new("/usr/local/bin/npm")),
            &commands,
        )
        .expect("execute");

        let names: Vec<&str> = outcome
            .steps
            .iter()
            .map(|step| step.name.as_str())
            .collect();
        assert!(names.contains(&"移除包体与入口"));
        assert!(names.contains(&"执行官方 ocx uninstall"));
        assert!(names.contains(&"清空 OPENCODEX_HOME 非自有残留"));
        assert!(outcome
            .steps
            .iter()
            .all(|step| step.status != StepStatus::Failed));
        let calls = commands.calls.borrow().clone();
        assert!(calls.iter().any(|call| call.starts_with("npm ")));
        assert!(calls.iter().any(|call| call.starts_with("ocx ")));
        // 顺序回归（TASK-160）：官方 `ocx uninstall` 必须在「移除包体与入口」**之前**。
        // 外部 npm 全局来源的入口就是 `ocx` 本身，先跑 `npm uninstall -g` 会把它删掉，
        // 官方命令随后必然失败（真机 history 出现过 `uninstall/failed`）。
        let ocx_index = calls.iter().position(|call| call.starts_with("ocx "));
        let npm_index = calls.iter().position(|call| call.starts_with("npm "));
        assert!(
            matches!((ocx_index, npm_index), (Some(a), Some(b)) if a < b),
            "官方命令必须先于卸包：{calls:?}"
        );
        // 残留：非自有类别被清，官方自有清单原样留着。
        assert!(!home.join("routing-history.sqlite").exists());
        assert!(home.join("config.json").exists());
        assert!(outcome.residue.iter().any(
            |item| item.status == ResidueStatus::Present && !item.path.ends_with("config.json")
        ));
        // The stub's successful exit did not actually delete the package.
        assert!(!outcome.verified_complete());
    }

    #[test]
    fn execute_requires_explicit_confirmation() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let home = root.path().join("opencodex-home");
        std::fs::create_dir_all(&home).expect("mkdir");
        let plan = plan_uninstall(&data_root, &home, RuntimeSourceKind::Managed, None);
        let error = execute_uninstall(
            &data_root,
            &home,
            &plan,
            &UninstallOptions {
                scope: UninstallScope::Body,
                auto_backup: false,
                clean_data: false,
                confirmed: false,
            },
            None,
            None,
            &StubCommands::ok(),
        )
        .expect_err("未确认必须拒绝");
        assert_eq!(error, UninstallError::ConfirmationRequired);
    }

    #[test]
    fn body_scope_never_touches_runtime_or_home() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let home = root.path().join("opencodex-home");
        std::fs::create_dir_all(&home).expect("mkdir");
        std::fs::write(home.join("routing-history.sqlite"), b"x").expect("write residue");
        let (_, entry) = install_npm_global(root.path(), "2.64.0");
        let plan = plan_uninstall(
            &data_root,
            &home,
            RuntimeSourceKind::Discovered,
            Some(&entry),
        );
        let commands = StubCommands::ok();
        let outcome = execute_uninstall(
            &data_root,
            &home,
            &plan,
            &UninstallOptions {
                scope: UninstallScope::Body,
                auto_backup: false,
                clean_data: true,
                confirmed: true,
            },
            Some(Path::new("/usr/local/bin/ocx")),
            Some(Path::new("/usr/local/bin/npm")),
            &commands,
        )
        .expect("execute");
        assert!(!outcome
            .steps
            .iter()
            .any(|step| step.name.contains("ocx uninstall")));
        assert!(commands
            .calls
            .borrow()
            .iter()
            .all(|call| !call.starts_with("ocx ")));
        assert!(home.join("routing-history.sqlite").exists());
        assert!(!outcome.verified_complete());
    }

    #[test]
    fn actual_body_removal_is_verified_without_deleting_retained_home() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let home = root.path().join("opencodex-home");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(home.join("config.json"), b"{}").unwrap();
        let (prefix, entry) = install_npm_global(root.path(), "2.64.0");
        let package_dir = prefix.join("lib/node_modules/@bitkyc08/opencodex");
        let mut plan = plan_uninstall(&data_root, &home, RuntimeSourceKind::Explicit, Some(&entry));
        plan.owner = BodyOwner::External(ExternalRemoval::PackageDir {
            package_dir: package_dir.clone(),
            entry: entry.clone(),
        });
        let outcome = execute_uninstall(
            &data_root,
            &home,
            &plan,
            &UninstallOptions {
                scope: UninstallScope::Body,
                auto_backup: false,
                clean_data: false,
                confirmed: true,
            },
            None,
            None,
            &StubCommands::ok(),
        )
        .unwrap();
        assert!(!package_dir.exists());
        assert!(!entry.exists());
        assert!(home.join("config.json").exists());
        assert!(outcome.verified_complete());
        let mut unknown = outcome.clone();
        unknown.residue[0].status = ResidueStatus::Unknown;
        assert!(!unknown.verified_complete());
        let mut missing_evidence = outcome.clone();
        missing_evidence.residue.clear();
        assert!(!missing_evidence.verified_complete());
        let mut partial = outcome;
        partial
            .steps
            .push(step("官方卸载", StepStatus::Failed, None));
        assert!(!partial.verified_complete());
    }

    #[test]
    fn residue_cleanup_is_explicit_and_stays_inside_home() {
        let root = tempfile::tempdir().expect("temp");
        let home = root.path().join("opencodex-home");
        std::fs::create_dir_all(home.join("integrations")).expect("mkdir");
        std::fs::create_dir_all(home.join("lab")).expect("mkdir");
        std::fs::write(home.join("routing-history.sqlite"), b"x").expect("write");
        std::fs::write(home.join("catalog-backup-abc.json"), b"x").expect("write");
        // 不在显式类别里的文件必须保留。
        std::fs::write(home.join("config.json"), b"{}").expect("write");
        std::fs::write(home.join("keep-me.json"), b"{}").expect("write");

        let removed = clean_opencodex_residue(&home).expect("clean");
        assert_eq!(
            removed, 4,
            "integrations / lab / routing-history / catalog-backup-* 各一"
        );
        assert!(!home.join("integrations").exists());
        assert!(home.join("config.json").exists());
        assert!(home.join("keep-me.json").exists());
    }

    #[test]
    fn residue_check_distinguishes_cleared_from_present() {
        let root = tempfile::tempdir().expect("temp");
        let data_root = root.path().join("data-root");
        let home = root.path().join("opencodex-home");
        std::fs::create_dir_all(&home).expect("mkdir");
        let (_, entry) = install_npm_global(root.path(), "2.64.0");
        let plan = plan_uninstall(
            &data_root,
            &home,
            RuntimeSourceKind::Discovered,
            Some(&entry),
        );
        let present = residue_check(&data_root, &plan, false);
        assert!(present
            .iter()
            .any(|item| item.status == ResidueStatus::Present));
        std::fs::remove_dir_all(entry.parent().expect("bin").parent().expect("package dir"))
            .expect("remove");
        let cleared = residue_check(&data_root, &plan, false);
        assert!(cleared
            .iter()
            .all(|item| item.status == ResidueStatus::Cleared));
    }

    #[test]
    fn official_observation_reads_and_masks_the_official_record() {
        let root = tempfile::tempdir().expect("temp");
        let opencodex_home = root.path().join("opencodex-home");
        std::fs::create_dir_all(&opencodex_home).expect("mkdir");
        assert!(official_uninstall_observation(&opencodex_home).is_empty());

        std::fs::write(
            opencodex_home.join(OFFICIAL_UNINSTALL_RECORD),
            b"proxy http://alice:s3cret@host:8080\nexit_code=0\n",
        )
        .expect("write");
        let lines = official_uninstall_observation(&opencodex_home);
        assert_eq!(lines.len(), 2);
        assert!(
            lines[0].contains("***@host:8080"),
            "凭据必须掩码：{}",
            lines[0]
        );
        assert!(!lines[0].contains("s3cret"));
    }

    #[test]
    fn uninstall_error_maps_to_stable_codes() {
        assert_eq!(UninstallError::NoManifest.code(), "uninstall_no_manifest");
        assert_eq!(
            UninstallError::ExternallyModified {
                expected: "a".into(),
                found: "b".into()
            }
            .code(),
            "uninstall_externally_modified"
        );
        let app: AppError = UninstallError::NoManifest.into();
        assert!(matches!(app, AppError::RuntimeManaged { .. }));
    }

    #[test]
    fn history_entry_is_shaped_like_the_frozen_contract() {
        let entry = history_entry("succeeded", "0.3.1", Path::new("/x/opencodex"), None);
        assert_eq!(entry.action, "uninstall");
        assert_eq!(entry.package, OFFICIAL_PACKAGE);
        assert_eq!(entry.version, "0.3.1");
        assert_eq!(entry.target, "/x/opencodex");
        assert!(entry.reason.is_none());
    }

    #[test]
    fn manifest_source_label_is_stable() {
        assert_eq!(
            manifest_source_label(InstallSourceKind::Registry),
            "registry"
        );
        assert_eq!(manifest_source_label(InstallSourceKind::Offline), "offline");
        assert_eq!(source_summary(RuntimeSourceKind::Managed), "托管安装");
    }
}
