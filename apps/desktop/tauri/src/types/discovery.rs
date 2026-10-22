//! 安装发现的前后端 DTO 与输入校验。

use crate::modules::discovery::{CheckResult, EnvironmentReport};

pub use crate::modules::discovery::{EnvironmentGate, EnvironmentPaths};

/// Tauri 命令接受的显式路径输入；三项同时存在或同时缺省才有效。
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryRequest {
    #[serde(default)]
    pub node_path: Option<String>,
    #[serde(default)]
    pub npm_path: Option<String>,
    #[serde(default)]
    pub ocx_path: Option<String>,
}

impl DiscoveryRequest {
    pub fn to_environment_paths(&self) -> Option<EnvironmentPaths> {
        match (&self.node_path, &self.npm_path, &self.ocx_path) {
            (Some(node), Some(npm), Some(ocx)) => Some(EnvironmentPaths::new(
                std::path::PathBuf::from(node),
                std::path::PathBuf::from(npm),
                std::path::PathBuf::from(ocx),
            )),
            (None, None, None) => None,
            _ => None,
        }
    }
}

/// 命令层返回给前端的发现报告。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentReportDto {
    pub node: CheckResult,
    pub npm: CheckResult,
    pub ocx: CheckResult,
    pub gate: EnvironmentGate,
    pub short_circuited: bool,
    pub brew_found: bool,
}

impl From<EnvironmentReport> for EnvironmentReportDto {
    fn from(report: EnvironmentReport) -> Self {
        Self {
            node: report.node,
            npm: report.npm,
            ocx: report.ocx,
            gate: report.gate,
            short_circuited: report.short_circuited,
            brew_found: report.brew_found,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_paths_together_are_accepted() {
        let request = DiscoveryRequest {
            node_path: Some("/fixtures/node".to_string()),
            npm_path: Some("/fixtures/npm".to_string()),
            ocx_path: Some("/fixtures/ocx".to_string()),
        };
        let paths = request.to_environment_paths().expect("paths");
        assert_eq!(paths.node, std::path::PathBuf::from("/fixtures/node"));
        assert_eq!(paths.npm, std::path::PathBuf::from("/fixtures/npm"));
        assert_eq!(paths.ocx, std::path::PathBuf::from("/fixtures/ocx"));
    }

    #[test]
    fn partial_paths_are_rejected_as_a_contract_guard() {
        let request = DiscoveryRequest {
            node_path: Some("/fixtures/node".to_string()),
            npm_path: None,
            ocx_path: Some("/fixtures/ocx".to_string()),
        };
        assert!(request.to_environment_paths().is_none());
    }

    #[test]
    fn default_paths_are_resolved_outside_the_discovery_module() {
        let paths = crate::types::discovery_paths::default_paths().expect("platform user home");
        assert!(paths.node.is_absolute());
        assert!(paths.npm.is_absolute());
        assert!(paths.ocx.is_absolute());
        // 工具可能来自任意候选目录（~/.local/bin、Volta、pnpm、nvm、Homebrew…），
        // 不再假定都在 ~/.local/bin。这里只验证两点：父目录属于候选集合；
        // 若该工具在候选里存在，选中的就是第一个存在的那个。
        let home = crate::infrastructure::platform::home_dir().unwrap();
        let candidates = crate::infrastructure::discovery_paths::candidate_dirs(&home);
        for (tool, selected) in [
            ("node", &paths.node),
            ("npm", &paths.npm),
            ("ocx", &paths.ocx),
        ] {
            let parent = selected.parent().expect("parent directory");
            assert!(
                candidates.iter().any(|dir| dir == parent),
                "{tool} 应来自候选目录：{}",
                selected.display()
            );
            let first_existing = candidates
                .iter()
                .map(|dir| dir.join(tool))
                .find(|path| path.is_file());
            if let Some(found) = first_existing {
                assert_eq!(&found, selected, "{tool} 应命中第一个存在的候选");
            }
        }
    }
}
