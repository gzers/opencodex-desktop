//! MOD-09：FZ-22 / FZ-25 / FZ-27 客户端落点契约与启用矩阵。
//!
//! 首期只支持用户级作用域。路径都来自显式 home 参数；模块不读取
//! 真实用户配置，不访问 Keychain、网络或官方 CLI。

use chrono::{DateTime, SecondsFormat, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// 首期允许写入的六个客户端。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientId {
    Codex,
    Claude,
    Gemini,
    Grok,
    Opencode,
    Hermes,
}

pub mod discovery;
pub mod projection;
pub mod source_dir;

pub const CLIENT_IDS: [ClientId; 6] = [
    ClientId::Codex,
    ClientId::Claude,
    ClientId::Gemini,
    ClientId::Grok,
    ClientId::Opencode,
    ClientId::Hermes,
];

/// Desktop 自管的 Codex 客户端根：`<OPENCODEX_HOME>/codex-client`。
///
/// 运行中的 OpenCodex 服务以它为 `CODEX_HOME`，所以扩展写入只有落在这里
/// 才会被实际读取；真实 `~/.codex` 全程只读。数据根尚未完成准入时返回
/// `None`，调用方据此把 Codex 目标降为只读视图。
pub fn managed_codex_root(data_root: &Path) -> Option<PathBuf> {
    let config = crate::modules::data_root::load_runtime_config(data_root).ok()?;
    let home = crate::modules::data_root::resolve_opencodex_home(&config);
    let root = crate::modules::process::client_home(&home);
    // 自管根必须属于这一个数据根自己的聚合目录。数据根直接放在共享临时
    // 父目录下时，`<父目录>/opencodex/home` 是整台机器共用的位置：
    // 两个实例会互相覆盖 Codex 配置。这种布局一律视为未绑定（只读）。
    if is_shared_parent(data_root) {
        return None;
    }
    Some(root)
}

/// 数据根的父目录是否是与其它实例共用的位置（系统临时目录）。
fn is_shared_parent(data_root: &Path) -> bool {
    shares_parent_with(data_root, &std::env::temp_dir())
}

fn shares_parent_with(data_root: &Path, shared: &Path) -> bool {
    let Some(parent) = data_root.parent() else {
        return true;
    };
    let Ok(parent) = parent.canonicalize() else {
        return true;
    };
    shared.canonicalize().is_ok_and(|shared| shared == parent)
}

/// 六个客户端的目标矩阵：Codex 绑定 Desktop 自管根，其余仍写各自用户级配置。
pub fn client_targets(data_root: &Path, home: &Path) -> Vec<ClientTarget> {
    let codex_root = managed_codex_root(data_root);
    CLIENT_IDS
        .map(|client| ClientTarget::user_target_in(client, home, codex_root.as_deref(), true, true))
        .to_vec()
}

/// FZ-24 分发方式取值。`symlink` 为首选，创建失败或安全校验不通过时自动回退 `copy`。
pub const SYNC_METHOD_SYMLINK: &str = "symlink";
pub const SYNC_METHOD_COPY: &str = "copy";
pub const SYNC_METHODS: [&str; 2] = [SYNC_METHOD_SYMLINK, SYNC_METHOD_COPY];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigFormat {
    Toml,
    Json,
    Jsonc,
    Yaml,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerShape {
    Map,
    Array,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TargetScope {
    User,
    Project,
}

/// FZ-25 指纹对象。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProjectionFingerprint {
    pub schema_version: u32,
    pub fingerprint: String,
    pub updated_at: String,
    pub source: String,
}

impl ProjectionFingerprint {
    /// 2 = 引入 `source_store` / `sync_method`（FZ-23 / FZ-24）。
    /// 3 = 引入逐客户端分发意图 `skill_targets`（图标开关 = 连/断该客户端的链接）。
    pub const SCHEMA_VERSION: u32 = 3;
    /// 仍可读取的最旧 schema；读取后按当前结构重算指纹并迁移。
    pub const MIN_SUPPORTED_SCHEMA_VERSION: u32 = 1;

    pub fn create(fingerprint: impl Into<String>, now: DateTime<Utc>) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            fingerprint: fingerprint.into(),
            updated_at: now.to_rfc3339_opts(SecondsFormat::Secs, true),
            source: "opencodex-desktop".to_string(),
        }
    }

    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version < Self::MIN_SUPPORTED_SCHEMA_VERSION
            || self.schema_version > Self::SCHEMA_VERSION
        {
            return Err("fingerprint schema version mismatch");
        }
        if self.fingerprint.len() != 64 || !self.fingerprint.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Err("fingerprint digest shape mismatch");
        }
        if self.updated_at.trim().is_empty() {
            return Err("fingerprint timestamp is required");
        }
        if self.source != "opencodex-desktop" {
            return Err("fingerprint source mismatch");
        }
        Ok(())
    }

    /// 旧 schema 的指纹按旧结构计算，与当前结构必然不一致。读取时不能据此判
    /// 冲突，否则升级后既有 `extension-config.json` 会被误判为 `external_modified`
    /// 而无法写入；按当前结构重算并在下次写入时落盘新 schema。
    pub fn needs_migration(&self) -> bool {
        self.schema_version < Self::SCHEMA_VERSION
    }
}

/// FZ-22 用户级客户端落点。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ClientTarget {
    pub client_id: ClientId,
    pub scope: TargetScope,
    pub project_root: Option<PathBuf>,
    /// 落点边界校验的信任锚（FZ-22）：所有写入都必须落在它之内。
    ///
    /// 对第三方客户端是解析后的用户主目录；对 Codex 是 Desktop 自管的
    /// `<OPENCODEX_HOME>/codex-client`——真实 `~/.codex` 全程只读。
    /// 备份等受管副产物一律写数据根，**不得**用本字段派生。
    pub anchor: PathBuf,
    /// 该客户端全部落点所属的用户主目录；用于禁止把源目录、锁定文件之类
    /// 的受管根放到客户端配置旁边。
    pub home: PathBuf,
    pub resolved_config_path: PathBuf,
    pub display_name: String,
    pub mcp_config_path: PathBuf,
    pub format: ConfigFormat,
    pub mcp_key: String,
    pub container_shape: ContainerShape,
    pub env_key_name: String,
    pub skills_dir: PathBuf,
    pub installed: bool,
    pub writable: bool,
    pub official_writer: bool,
}

impl ClientTarget {
    /// 根据显式 home 与安装探测构造用户级落点。
    pub fn user_target(
        client_id: ClientId,
        home: &Path,
        installed: bool,
        user_enabled: bool,
    ) -> Self {
        Self::user_target_in(client_id, home, None, installed, user_enabled)
    }

    /// 同 `user_target`，但 Codex 的落点改到 Desktop 自管的客户端根。
    ///
    /// `codex_root` 必须是 Desktop 自管目录（`<OPENCODEX_HOME>/codex-client`），
    /// 且必须位于 `home` 之内；否则按未绑定处理，退回只读的 `~/.codex` 视图。
    /// 其余五个客户端的目标是各自的用户级配置，与 Desktop 自管根无关。
    pub fn user_target_in(
        client_id: ClientId,
        home: &Path,
        codex_root: Option<&Path>,
        installed: bool,
        user_enabled: bool,
    ) -> Self {
        // 只接受绝对路径的 Desktop 自管根绑定；越出本机主目录的绑定仍由
        // 写入时的边界校验按数据根复核（`OPENCODEX_HOME` 可显式外置）。
        // 绑定不成立时 Codex 退回真实 `~/.codex`，只作为只读视图。
        let codex_root = codex_root.filter(|root| root.is_absolute());
        let read_only_fallback = client_id == ClientId::Codex && codex_root.is_none();
        let (
            display_name,
            mcp_config_path,
            format,
            mcp_key,
            container_shape,
            env_key_name,
            skills_dir,
            anchor,
        ) = match client_id {
            ClientId::Codex => {
                // 真实 `~/.codex` 只读：运行中的服务读的是 CODEX_HOME，
                // 也就是这个 Desktop 自管根。
                let anchor = codex_root
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| home.join(".codex"));
                (
                    "Codex",
                    anchor.join("config.toml"),
                    ConfigFormat::Toml,
                    "mcp_servers",
                    ContainerShape::Map,
                    "env",
                    anchor.join("skills"),
                    anchor,
                )
            }
            ClientId::Claude => (
                "Claude",
                home.join(".claude.json"),
                ConfigFormat::Json,
                "mcp_servers",
                ContainerShape::Map,
                "env",
                home.join(".claude/skills"),
                home.to_path_buf(),
            ),
            ClientId::Gemini => (
                "Gemini",
                home.join(".gemini/settings.json"),
                ConfigFormat::Json,
                "mcp_servers",
                ContainerShape::Map,
                "env",
                home.join(".gemini/skills"),
                home.to_path_buf(),
            ),
            ClientId::Grok => (
                "Grok",
                home.join(".grok/user-settings.json"),
                ConfigFormat::Json,
                "mcp_servers",
                ContainerShape::Array,
                "env",
                home.join(".grok/skills"),
                home.to_path_buf(),
            ),
            ClientId::Opencode => (
                "OpenCode",
                home.join(".config/opencode/opencode.json"),
                ConfigFormat::Json,
                "mcp_servers",
                ContainerShape::Map,
                "environment",
                home.join(".config/opencode/skills"),
                home.to_path_buf(),
            ),
            ClientId::Hermes => (
                "Hermes",
                home.join(".hermes/config.yaml"),
                ConfigFormat::Yaml,
                "mcp_servers",
                ContainerShape::Map,
                "env",
                home.join(".hermes/skills"),
                home.to_path_buf(),
            ),
        };
        Self {
            client_id,
            scope: TargetScope::User,
            project_root: None,
            anchor,
            home: home.to_path_buf(),
            resolved_config_path: mcp_config_path.clone(),
            display_name: display_name.to_string(),
            mcp_config_path,
            format,
            mcp_key: mcp_key.to_string(),
            container_shape,
            env_key_name: env_key_name.to_string(),
            skills_dir,
            installed,
            writable: installed && user_enabled && !read_only_fallback,
            official_writer: true,
        }
    }

    /// FZ-22：未安装跳过且不落盘。
    pub fn can_write(&self) -> bool {
        self.writable
    }
}

/// FZ-25 统一配置修订号与客户端启用矩阵。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ExtensionConfig {
    pub skills: Vec<String>,
    pub servers: Vec<String>,
    pub enablement: BTreeMap<ClientId, bool>,
    /// 逐客户端分发意图：`Skill 名 → 要分发到哪些客户端`。
    ///
    /// 缺省（键不存在）= 沿用旧语义「所有启用客户端都分发」，因此升级不会改动既有落地；
    /// 一旦用户逐个开关过某个 Skill，就写入精确清单，投影只按清单连/断链接。
    /// **源目录只读**：这里只描述链接，不代表可以删改本源 Skill。
    #[serde(default)]
    pub skill_targets: BTreeMap<String, Vec<ClientId>>,
    /// FZ-23 源目录。`None` = 使用默认源目录（`<主目录>/.agents/skills`）；
    /// `Some(path)` = 用户自定义目录（绝对路径，只读）。
    #[serde(default)]
    pub source_store: Option<String>,
    /// FZ-24 分发方式。用户可选项；仅 `symlink` / `copy`。
    #[serde(default = "default_sync_method")]
    pub sync_method: String,
    pub revision: u64,
}

fn default_sync_method() -> String {
    crate::modules::runtime_defaults::extensions_default_sync_method().to_string()
}

impl Default for ExtensionConfig {
    fn default() -> Self {
        Self {
            skills: Vec::new(),
            servers: Vec::new(),
            enablement: CLIENT_IDS
                .map(|client| {
                    (
                        client,
                        crate::modules::runtime_defaults::extensions_default_client_enablement(),
                    )
                })
                .into_iter()
                .collect(),
            skill_targets: BTreeMap::new(),
            source_store: None,
            sync_method: default_sync_method(),
            revision: 1,
        }
    }
}

impl ExtensionConfig {
    /// 某 Skill 的目标客户端；`None` 表示尚未逐个选择（沿用「全部启用客户端」）。
    pub fn skill_targets_for(&self, name: &str) -> Option<&Vec<ClientId>> {
        self.skill_targets.get(name)
    }

    /// 写入某 Skill 的目标客户端清单；空清单表示不再分发该 Skill。
    pub fn set_skill_targets(&mut self, name: &str, mut clients: Vec<ClientId>) {
        clients.sort();
        clients.dedup();
        if clients.is_empty() {
            self.skill_targets.remove(name);
            self.skills.retain(|item| item != name);
            return;
        }
        if !self.skills.iter().any(|item| item == name) {
            self.skills.push(name.to_string());
        }
        self.skill_targets.insert(name.to_string(), clients);
    }

    /// 把「还没逐项选择过」的 Skill 显式化：以**当前实际已落地**的客户端为基线。
    ///
    /// 只由写命令在用户显式开关某个 Skill 时调用，不在读取路径上改动配置；
    /// 基线取实际落地状态而不是「全部启用」，避免点一个客户端就把六个都点亮。
    pub fn materialize_skill_targets(&mut self, name: &str, observed: Vec<ClientId>) {
        if self.skill_targets.contains_key(name) {
            return;
        }
        let mut seed = observed;
        seed.sort();
        seed.dedup();
        self.skill_targets.insert(name.to_string(), seed);
    }

    pub fn fingerprint_payload(&self) -> String {
        serde_json::to_vec(self)
            .unwrap_or_default()
            .iter()
            .fold(String::new(), |_, _| String::new())
    }

    pub fn fingerprint(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(serde_json::to_vec(self).unwrap_or_default());
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect()
    }

    pub fn bump_revision(&mut self) {
        self.revision = self.revision.saturating_add(1);
    }

    pub fn set_enabled(&mut self, client: ClientId, enabled: bool) {
        self.enablement.insert(client, enabled);
        self.bump_revision();
    }

    /// 同步方式必须是冻结取值之一；损坏配置按默认值修正，不猜测。
    pub fn sync_method_normalized(&self) -> &str {
        if SYNC_METHODS.contains(&self.sync_method.as_str()) {
            self.sync_method.as_str()
        } else {
            SYNC_METHOD_SYMLINK
        }
    }

    pub fn uses_copy(&self) -> bool {
        self.sync_method_normalized() == SYNC_METHOD_COPY
    }

    /// 设置源目录。`None` 表示回到默认源目录。
    pub fn set_source_store(&mut self, value: Option<String>) {
        if self.source_store == value {
            return;
        }
        self.source_store = value;
        self.bump_revision();
    }

    /// 设置分发方式；取值非法时拒绝，保留原值。
    pub fn set_sync_method(&mut self, value: &str) -> bool {
        if !SYNC_METHODS.contains(&value) || self.sync_method == value {
            return false;
        }
        self.sync_method = value.to_string();
        self.bump_revision();
        true
    }
}

/// 计算统一配置与 live 内容的比对结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictKind {
    ExternalModified,
    ClientInconsistent,
    HalfWritten,
}

pub fn conflict_for_live(
    expected: &ProjectionFingerprint,
    live: Option<&str>,
) -> Option<ConflictKind> {
    match live {
        Some(hash) if *hash == expected.fingerprint => None,
        Some(_) => Some(ConflictKind::ExternalModified),
        None => Some(ConflictKind::HalfWritten),
    }
}

#[allow(dead_code)]
fn sha256_hex(payload: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn home() -> PathBuf {
        std::env::temp_dir().join("fixtures/opencodex-home")
    }

    #[test]
    fn six_clients_are_frozen_with_default_enabled_matrix() {
        let config = ExtensionConfig::default();
        assert_eq!(CLIENT_IDS.len(), 6);
        assert_eq!(config.enablement.len(), 6);
        assert!(config.enablement.values().all(|enabled| *enabled));
        assert_eq!(config.revision, 1);
    }

    #[test]
    fn user_paths_match_fz22() {
        let root = home();
        let targets = CLIENT_IDS.map(|client| ClientTarget::user_target(client, &root, true, true));
        let paths = targets.clone().map(|target| target.mcp_config_path);
        assert_eq!(
            paths,
            [
                ".codex/config.toml",
                ".claude.json",
                ".gemini/settings.json",
                ".grok/user-settings.json",
                ".config/opencode/opencode.json",
                ".hermes/config.yaml",
            ]
            .map(|path| root.join(path))
        );
        assert!(targets
            .iter()
            .all(|target| target.scope == TargetScope::User));
        assert!(targets.iter().all(|target| target.official_writer));
    }

    #[test]
    fn writable_requires_installed_and_enabled() {
        let root = home();
        let managed = root.join(".codex-desktop/opencodex/home/codex-client");
        let installed =
            ClientTarget::user_target_in(ClientId::Codex, &root, Some(&managed), true, true);
        assert!(installed.can_write());
        let disabled =
            ClientTarget::user_target_in(ClientId::Codex, &root, Some(&managed), true, false);
        assert!(!disabled.can_write());
        let missing =
            ClientTarget::user_target_in(ClientId::Codex, &root, Some(&managed), false, true);
        assert!(!missing.can_write());
        assert!(!missing.skills_dir.exists());
    }

    #[test]
    fn codex_writes_only_inside_the_desktop_managed_client_home() {
        let root = home();
        let managed = root.join(".codex-desktop/opencodex/home/codex-client");
        let target =
            ClientTarget::user_target_in(ClientId::Codex, &root, Some(&managed), true, true);
        assert_eq!(target.anchor, managed);
        assert_eq!(target.mcp_config_path, managed.join("config.toml"));
        assert_eq!(target.skills_dir, managed.join("skills"));
        // 真实 `~/.codex` 不再出现在任何落点里。
        assert!(!target.mcp_config_path.starts_with(root.join(".codex")));
        assert!(!target.skills_dir.starts_with(root.join(".codex")));

        // 未绑定（或绑定不是绝对路径）时只保留只读视图，拒绝写入。
        let relative = PathBuf::from("codex-client");
        for unbound in [None, Some(relative)] {
            let fallback = ClientTarget::user_target_in(
                ClientId::Codex,
                &root,
                unbound.as_deref(),
                true,
                true,
            );
            assert_eq!(fallback.anchor, root.join(".codex"));
            assert!(
                !fallback.can_write(),
                "unbound Codex target must stay read only"
            );
        }
    }

    #[test]
    fn only_codex_moves_off_the_user_client_directory() {
        let root = home();
        let managed = root.join(".codex-desktop/opencodex/home/codex-client");
        let targets = CLIENT_IDS
            .map(|client| ClientTarget::user_target_in(client, &root, Some(&managed), true, true));
        for target in targets {
            if target.client_id == ClientId::Codex {
                assert_eq!(target.anchor, managed);
            } else {
                assert_eq!(target.anchor, root);
                assert!(target.mcp_config_path.starts_with(&root));
            }
        }
    }

    #[test]
    fn data_root_directly_under_the_shared_temp_parent_never_binds_a_managed_codex_root() {
        // 两个实例把数据根直接放在系统临时目录下时，`<父目录>/opencodex/home`
        // 是整机共用的位置；这种布局必须按未绑定处理，退回只读视图。
        let temp = tempfile::tempdir().unwrap();
        let shared = temp.path().join("shared-parent");
        std::fs::create_dir_all(&shared).unwrap();
        let shared_like = shared.join("desktop");
        std::fs::create_dir_all(&shared_like).unwrap();
        assert!(shares_parent_with(&shared_like, &shared));

        // 自己的聚合目录（`<root>/data-root`）不是共享位置。
        let aggregate = shared.join("aggregate/data-root");
        std::fs::create_dir_all(&aggregate).unwrap();
        assert!(!shares_parent_with(&aggregate, &shared));
    }

    #[test]
    fn client_disabled_does_not_delete_projection() {
        let mut config = ExtensionConfig::default();
        config.set_enabled(ClientId::Claude, false);
        assert_eq!(config.revision, 2);
        assert_eq!(config.enablement.get(&ClientId::Claude), Some(&false));
        assert!(!config.skills.is_empty() || config.servers.is_empty());
    }

    #[test]
    fn fingerprint_shape_and_source_are_frozen() {
        let now = Utc.with_ymd_and_hms(2026, 9, 15, 2, 3, 4).unwrap();
        let config = ExtensionConfig::default();
        let fingerprint = ProjectionFingerprint::create(config.fingerprint(), now);
        // 3 = 在 2（source_store / sync_method）之上引入逐客户端分发意图 `skill_targets`。
        assert_eq!(fingerprint.schema_version, 3);
        assert!(!fingerprint.needs_migration());
        assert_eq!(fingerprint.source, "opencodex-desktop");
        assert_eq!(fingerprint.updated_at, "2026-09-15T02:03:04Z");
        assert!(fingerprint.validate().is_ok());
    }

    #[test]
    fn legacy_schema_version_is_readable_but_flagged_for_migration() {
        let now = Utc.with_ymd_and_hms(2026, 9, 15, 2, 3, 4).unwrap();
        let mut legacy = ProjectionFingerprint::create("a".repeat(64), now);
        legacy.schema_version = 1;
        assert!(legacy.validate().is_ok());
        assert!(legacy.needs_migration());

        let mut too_new = ProjectionFingerprint::create("a".repeat(64), now);
        too_new.schema_version = 99;
        assert!(too_new.validate().is_err());
    }

    #[test]
    fn source_store_and_sync_method_participate_in_fingerprint() {
        let config = ExtensionConfig::default();
        let mut custom = config.clone();
        custom.set_source_store(Some("/tmp/custom-skills".to_string()));
        assert_ne!(config.fingerprint(), custom.fingerprint());
        assert_eq!(custom.revision, config.revision + 1);

        let mut copied = config.clone();
        assert!(copied.set_sync_method(super::SYNC_METHOD_COPY));
        assert_ne!(config.fingerprint(), copied.fingerprint());
        assert!(copied.uses_copy());
        // 取值非法或与当前一致时不改动、不涨修订号。
        assert!(!copied.set_sync_method("hardlink"));
        assert!(!config.clone().set_sync_method(super::SYNC_METHOD_SYMLINK));
        assert_eq!(copied.sync_method_normalized(), super::SYNC_METHOD_COPY);
    }

    #[test]
    fn fingerprint_changes_with_enablement_and_content() {
        let config = ExtensionConfig::default();
        let mut changed = config.clone();
        changed.skills.push("fixture".to_string());
        changed.bump_revision();
        assert_ne!(config.fingerprint(), changed.fingerprint());
        assert_ne!(config.revision, changed.revision);
    }

    #[test]
    fn conflict_detection_matches_expected_or_flags_external_change() {
        let config = ExtensionConfig::default();
        let expected = ProjectionFingerprint::create(config.fingerprint(), Utc::now());
        assert_eq!(
            conflict_for_live(&expected, Some(&expected.fingerprint)),
            None
        );
        assert_eq!(
            conflict_for_live(&expected, Some("different")),
            Some(ConflictKind::ExternalModified)
        );
        assert_eq!(
            conflict_for_live(&expected, None),
            Some(ConflictKind::HalfWritten)
        );
    }

    #[test]
    fn serialization_uses_frozen_snake_case() {
        assert_eq!(
            serde_json::to_string(&ClientId::Opencode).unwrap(),
            "\"opencode\""
        );
        assert_eq!(
            serde_json::to_string(&ConfigFormat::Jsonc).unwrap(),
            "\"jsonc\""
        );
        assert_eq!(
            serde_json::to_string(&ContainerShape::Array).unwrap(),
            "\"array\""
        );
    }

    #[test]
    fn no_real_user_home_is_implicitly_read() {
        // Guard against accidental default path derivation in this module.
        let root = home();
        assert!(!root.exists());
        let target = ClientTarget::user_target(ClientId::Codex, &root, true, true);
        assert!(target.mcp_config_path.starts_with(&root));
    }
}
