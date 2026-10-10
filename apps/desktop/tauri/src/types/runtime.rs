//! 运行来源、托管安装与卸载的前后端 DTO（IMP Track B · B3 / B4 / B5）。
//!
//! 只做投影与输入解析；真实行为在 `modules::runtime`，命令层只编排。

use serde::{Deserialize, Serialize};

use crate::modules::runtime::install::{InstallSourceKind, ProxyConfig, ProxyScheme};
use crate::modules::runtime::uninstall::{UninstallPlan, UninstallScope};
use crate::modules::runtime::{InstallHistoryEntry, RuntimeSourceKind};

/// 当前运行来源（设置 → 安装配置卡片的第一行事实）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSourceDto {
    pub kind: RuntimeSourceKind,
    pub path: Option<String>,
    pub version: Option<String>,
    pub resolved_at: Option<String>,
    /// 路径是否落在数据根内（界面据此标注「数据根内 / 数据根外」）。
    pub inside_data_root: bool,
    /// 托管入口路径；非托管来源时为 `None`。
    pub managed_entry: Option<String>,
    /// **安装记录登记的**托管前缀（`FZ-51`）；自定义前缀安装时为该自定义目录。
    /// 卸载与「重装同版本」必须用它定位，不能假设等于数据根默认位置。
    pub managed_prefix: String,
    /// 托管前缀的**默认落点** `<数据根>/runtime/opencodex`；「恢复默认」用它，
    /// 不受自定义前缀影响。
    pub default_prefix: String,
    /// 用户显式指定的路径（用于「恢复自动发现」的可用态判断）。
    pub explicit_path: Option<String>,
    pub history: Vec<InstallHistoryEntry>,
}

/// 安装请求（弹窗 ①②③ 步的合并提交）。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInstallRequestDto {
    /// 自定义前缀；缺省用默认 `<数据根>/runtime/opencodex`。
    #[serde(default)]
    pub prefix: Option<String>,
    /// `registry` | `offline`。
    pub source: String,
    /// 联网安装的版本；缺省 `latest`。
    #[serde(default)]
    pub version: Option<String>,
    /// 离线安装的 `.tgz` 路径。
    #[serde(default)]
    pub offline_path: Option<String>,
    #[serde(default)]
    pub proxy_scheme: Option<String>,
    #[serde(default)]
    pub proxy_host: Option<String>,
    #[serde(default)]
    pub proxy_username: Option<String>,
    #[serde(default)]
    pub proxy_secret: Option<String>,
    /// 是否已在二次确认框里允许执行安装脚本（默认 `false`）。
    #[serde(default)]
    pub allow_scripts: bool,
}

impl RuntimeInstallRequestDto {
    /// 代理：三项都缺省＝直连；给了协议但没给地址＝不合法（由校验层报错）。
    pub fn proxy(&self) -> Option<ProxyConfig> {
        let scheme = self.proxy_scheme.as_deref().and_then(ProxyScheme::parse)?;
        let host = self.proxy_host.clone().unwrap_or_default();
        let mut proxy = ProxyConfig::new(scheme, host);
        if let (Some(username), Some(secret)) =
            (self.proxy_username.clone(), self.proxy_secret.clone())
        {
            if !username.is_empty() {
                proxy = proxy.with_credential(username, secret);
            }
        }
        Some(proxy)
    }

    pub fn prefix_path(&self) -> Option<std::path::PathBuf> {
        self.prefix
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(std::path::PathBuf::from)
    }

    pub fn offline_archive(&self) -> Option<std::path::PathBuf> {
        self.offline_path
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(std::path::PathBuf::from)
    }

    /// `source` 字段的受控解析；不接受未知取值。
    pub fn source_kind(&self) -> Option<InstallSourceKind> {
        match self.source.trim().to_ascii_lowercase().as_str() {
            "registry" => Some(InstallSourceKind::Registry),
            "offline" => Some(InstallSourceKind::Offline),
            _ => None,
        }
    }
}

/// 安装结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInstallOutcomeDto {
    pub package: String,
    pub version: String,
    pub target: String,
    pub entry: String,
    pub tarball_sha256: String,
    pub source: InstallSourceKind,
    pub scripts_enabled: bool,
    pub npm_path: String,
    pub installed_at: String,
    pub proxy_used: bool,
    /// 代理进程在运行，需重启才生效（`FZ-48`）。
    pub needs_restart: bool,
    /// 安装已落地，但保护备份仍待完整性核对。
    pub protection_reconciliation_pending: bool,
}

/// 离线包预检结果（拖拽区的「已选 / 校验失败 / 拒绝」三态）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OfflinePackagePreviewDto {
    pub ok: bool,
    pub file_name: String,
    pub file_size: u64,
    /// 包内声明的版本；校验失败时为 `None`。
    pub version: Option<String>,
    /// SHA-256 前缀（界面上只显示前 16 位，避免撑破一行）。
    pub sha256_prefix: Option<String>,
    /// 拒绝原因的稳定标识；通过时为 `None`。
    pub rejection_code: Option<String>,
    pub message: String,
}

/// 卸载请求（Revision 11）：范围与选项；确认由界面勾选后回传。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeUninstallRequestDto {
    /// `body`（仅包体与入口）| `full`（完整卸载）。
    pub scope: String,
    /// 卸载前自动生成备份：**勾选即承诺成功**，失败则中止。
    #[serde(default)]
    pub auto_backup: bool,
    /// 是否清空 `OPENCODEX_HOME` 的非自有残留。
    #[serde(default)]
    pub clean_data: bool,
    /// 显式确认（界面勾选「我已阅读清单并确认继续」后回传）。
    #[serde(default)]
    pub confirmation: Option<String>,
}

impl RuntimeUninstallRequestDto {
    pub fn scope_kind(&self) -> Option<UninstallScope> {
        match self.scope.trim().to_ascii_lowercase().as_str() {
            "body" => Some(UninstallScope::Body),
            "full" => Some(UninstallScope::Full),
            _ => None,
        }
    }

    pub fn confirmed(&self) -> bool {
        self.confirmation
            .as_deref()
            .map(str::trim)
            .is_some_and(|value| !value.is_empty())
    }
}

/// 卸载方案（只读）：界面据此渲染「将移除的对象」与残留候选。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeUninstallPlanDto {
    pub source_kind: RuntimeSourceKind,
    pub source_path: Option<String>,
    /// 包体是否由本产品托管（决定由谁移除）。
    pub managed: bool,
    /// 已检测到的可复用备份 id（无则 `None`）。
    pub backup_id: Option<String>,
    /// 移除命令是否可代执行；`false` 时界面必须显示原因并禁止确认。
    pub removable: bool,
    pub reason: Option<String>,
    pub remove_objects: Vec<String>,
    pub runtime_objects: Vec<String>,
    pub data_objects: Vec<String>,
    pub residue_candidates: Vec<String>,
    pub official_command: Option<String>,
    pub external_command: Option<String>,
}

impl RuntimeUninstallPlanDto {
    pub fn from_plan(plan: &UninstallPlan, ocx: Option<&std::path::Path>) -> Self {
        let reason = match &plan.owner {
            crate::modules::runtime::uninstall::BodyOwner::External(
                crate::modules::runtime::uninstall::ExternalRemoval::Unsupported { reason },
            ) => Some(reason.clone()),
            _ => None,
        };
        Self {
            source_kind: plan.source_kind,
            source_path: plan.source_path.clone(),
            managed: plan.owner.is_managed(),
            backup_id: plan.backup_id.clone(),
            removable: reason.is_none(),
            reason,
            remove_objects: plan.remove_objects.clone(),
            runtime_objects: plan.runtime_objects.clone(),
            data_objects: plan.data_objects.clone(),
            residue_candidates: plan.residue_candidates.clone(),
            official_command: Some(plan.official_command(ocx)),
            external_command: plan.external_command(),
        }
    }
}

/// 卸载的一步结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeUninstallStepDto {
    pub name: String,
    /// `ok` | `skipped` | `failed`。
    pub status: String,
    pub detail: Option<String>,
}

/// 残留核验的一项；`unknown` **不得**被当成已清除。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeUninstallResidueDto {
    pub path: String,
    /// `cleared` | `present` | `unknown`。
    pub status: String,
}

/// 卸载结果（Revision 11）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeUninstallResultDto {
    pub scope: UninstallScope,
    pub source_kind: RuntimeSourceKind,
    pub source_path: Option<String>,
    pub backup_id: Option<String>,
    pub backup_directory: Option<String>,
    pub steps: Vec<RuntimeUninstallStepDto>,
    pub residue: Vec<RuntimeUninstallResidueDto>,
    /// 官方命令（掩码后）的输出行。
    pub official_output: Vec<String>,
    /// 卸载后是否需要重启才生效。
    pub needs_restart: bool,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> RuntimeInstallRequestDto {
        RuntimeInstallRequestDto {
            prefix: None,
            source: "registry".to_string(),
            version: None,
            offline_path: None,
            proxy_scheme: None,
            proxy_host: None,
            proxy_username: None,
            proxy_secret: None,
            allow_scripts: false,
        }
    }

    #[test]
    fn proxy_is_absent_without_a_scheme() {
        assert!(request().proxy().is_none());
    }

    #[test]
    fn proxy_without_credentials_stays_credential_free() {
        let mut value = request();
        value.proxy_scheme = Some("socks5".to_string());
        value.proxy_host = Some("127.0.0.1:1080".to_string());
        let proxy = value.proxy().expect("proxy");
        assert_eq!(proxy.scheme, ProxyScheme::Socks5h);
        assert!(!proxy.has_credential());
        assert_eq!(proxy.masked(), "socks5h://***@127.0.0.1:1080");
    }

    #[test]
    fn proxy_with_credentials_carries_them_only_into_the_config_form() {
        let mut value = request();
        value.proxy_scheme = Some("http".to_string());
        value.proxy_host = Some("host:8080".to_string());
        value.proxy_username = Some("alice".to_string());
        value.proxy_secret = Some("s3cret".to_string());
        let proxy = value.proxy().expect("proxy");
        assert_eq!(proxy.url_without_credential(), "http://host:8080");
        assert_eq!(
            proxy.url_with_credential().as_deref(),
            Some("http://alice:s3cret@host:8080")
        );
        assert!(!proxy.url_without_credential().contains("s3cret"));
    }

    #[test]
    fn source_and_scope_parsing_is_closed_world() {
        let mut value = request();
        assert_eq!(value.source_kind(), Some(InstallSourceKind::Registry));
        value.source = "offline".to_string();
        assert_eq!(value.source_kind(), Some(InstallSourceKind::Offline));
        value.source = "whatever".to_string();
        assert_eq!(value.source_kind(), None);

        let uninstall = RuntimeUninstallRequestDto {
            scope: "body".to_string(),
            auto_backup: true,
            clean_data: false,
            confirmation: Some("confirmed".to_string()),
        };
        assert_eq!(uninstall.scope_kind(), Some(UninstallScope::Body));
        assert!(uninstall.confirmed());
        let full = RuntimeUninstallRequestDto {
            scope: "FULL".to_string(),
            auto_backup: false,
            clean_data: true,
            confirmation: None,
        };
        assert_eq!(full.scope_kind(), Some(UninstallScope::Full));
        assert!(!full.confirmed());
        let bad = RuntimeUninstallRequestDto {
            scope: "nope".to_string(),
            auto_backup: false,
            clean_data: false,
            confirmation: Some("   ".to_string()),
        };
        assert_eq!(bad.scope_kind(), None);
        assert!(!bad.confirmed());
    }

    #[test]
    fn blank_offline_path_is_not_an_archive() {
        let mut value = request();
        value.offline_path = Some("  ".to_string());
        assert!(value.offline_archive().is_none());
        value.offline_path = Some("/tmp/opencodex-0.3.1.tgz".to_string());
        assert_eq!(
            value.offline_archive().as_deref(),
            Some(std::path::Path::new("/tmp/opencodex-0.3.1.tgz"))
        );
    }
}
