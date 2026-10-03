//! 安装发现模块：按 Node.js → npm → ocx 的冻结顺序做受控只读检查。
//!
//! 模块只检查显式传入的可执行路径，不搜索 PATH、不执行安装、
//! 不写用户配置、不访问 Keychain。生产路径解析留在命令编排层。

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// 单个前置组件的检查结果。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CheckResult {
    pub found: bool,
    pub path: Option<String>,
    pub version: Option<String>,
}

impl CheckResult {
    fn missing() -> Self {
        Self {
            found: false,
            path: None,
            version: None,
        }
    }

    fn not_checked() -> Self {
        Self::missing()
    }

    fn found(path: &Path, version: Option<String>) -> Self {
        Self {
            found: true,
            path: Some(path.to_string_lossy().into_owned()),
            version,
        }
    }
}

/// 环境门禁结果；`ready` 表示三项都已发现。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentGate {
    Ready,
    MissingNode,
    MissingNpm,
    MissingOcx,
}

/// 三个可执行文件的受控解析路径。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentPaths {
    pub node: PathBuf,
    pub npm: PathBuf,
    pub ocx: PathBuf,
}

impl EnvironmentPaths {
    pub fn new(node: PathBuf, npm: PathBuf, ocx: PathBuf) -> Self {
        Self { node, npm, ocx }
    }
}

/// 一次完整发现的可观察输出。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EnvironmentReport {
    pub node: CheckResult,
    pub npm: CheckResult,
    pub ocx: CheckResult,
    pub gate: EnvironmentGate,
    pub short_circuited: bool,
    pub brew_found: bool,
}

/// 发现门禁；输入必须是已解析的受控路径。
#[derive(Debug, Clone)]
pub struct EnvironmentDiscovery {
    paths: EnvironmentPaths,
}

impl EnvironmentDiscovery {
    pub fn new(paths: EnvironmentPaths) -> Self {
        Self { paths }
    }

    pub fn discover(&self) -> EnvironmentReport {
        let node = if self.is_executable_file(&self.paths.node) {
            CheckResult::found(&self.paths.node, None)
        } else {
            CheckResult::missing()
        };

        if !node.found {
            return EnvironmentReport {
                brew_found: self.is_brew_available(),
                node,
                npm: CheckResult::not_checked(),
                ocx: CheckResult::not_checked(),
                gate: EnvironmentGate::MissingNode,
                short_circuited: true,
            };
        }

        let npm = if self.is_executable_file(&self.paths.npm) {
            CheckResult::found(&self.paths.npm, None)
        } else {
            CheckResult::missing()
        };

        if !npm.found {
            return EnvironmentReport {
                brew_found: false,
                node,
                npm,
                ocx: CheckResult::not_checked(),
                gate: EnvironmentGate::MissingNpm,
                short_circuited: true,
            };
        }

        let ocx_found = self.is_executable_file(&self.paths.ocx);
        let ocx = if ocx_found {
            CheckResult::found(&self.paths.ocx, None)
        } else {
            CheckResult::missing()
        };

        let gate = if ocx_found {
            EnvironmentGate::Ready
        } else {
            EnvironmentGate::MissingOcx
        };

        EnvironmentReport {
            brew_found: false,
            node,
            npm,
            ocx,
            gate,
            short_circuited: false,
        }
    }

    /// 缺 Node.js 时才读取 Homebrew 固定探测点；不搜索 PATH。
    fn is_brew_available(&self) -> bool {
        std::env::var_os("HOME")
            .filter(|home| !home.is_empty())
            .map(|home| Path::new(&home).join(".brew/bin/brew").is_file())
            .unwrap_or(false)
    }

    fn is_executable_file(&self, path: &Path) -> bool {
        path.is_file()
            && std::fs::metadata(path)
                .map(|meta| is_executable(&meta))
                .unwrap_or(false)
    }
}

#[cfg(unix)]
fn is_executable(metadata: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o111 != 0
}

#[cfg(not(unix))]
fn is_executable(_metadata: &std::fs::Metadata) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;

    struct TempEnvironment {
        #[allow(dead_code)]
        root: tempfile::TempDir,
        node: PathBuf,
        npm: PathBuf,
        ocx: PathBuf,
    }

    fn write_executable(path: &Path) {
        std::fs::write(path, b"#!/bin/sh\nexit 0\n").expect("write fixture");
        std::fs::set_permissions(path, Permissions::from_mode(0o755)).expect("chmod fixture");
    }

    fn environment() -> TempEnvironment {
        let root = tempfile::tempdir().expect("create temporary fixture root");
        let node = root.path().join("node");
        let npm = root.path().join("npm");
        let ocx = root.path().join("ocx");
        TempEnvironment {
            root,
            node,
            npm,
            ocx,
        }
    }

    fn discovery(environment: &TempEnvironment) -> EnvironmentDiscovery {
        EnvironmentDiscovery::new(EnvironmentPaths::new(
            environment.node.clone(),
            environment.npm.clone(),
            environment.ocx.clone(),
        ))
    }

    #[test]
    fn all_present_is_ready_and_does_not_short_circuit() {
        let env = environment();
        write_executable(&env.node);
        write_executable(&env.npm);
        write_executable(&env.ocx);

        let report = discovery(&env).discover();

        assert_eq!(report.gate, EnvironmentGate::Ready);
        assert!(!report.short_circuited);
        assert!(report.node.found && report.npm.found && report.ocx.found);
    }

    #[test]
    fn missing_node_marks_followups_not_checked() {
        let env = environment();
        write_executable(&env.npm);
        write_executable(&env.ocx);

        let report = discovery(&env).discover();

        assert_eq!(report.gate, EnvironmentGate::MissingNode);
        assert!(report.short_circuited);
        assert!(!report.node.found);
        assert!(!report.npm.found);
        assert!(!report.ocx.found);
    }

    #[test]
    fn missing_npm_marks_ocx_not_checked() {
        let env = environment();
        write_executable(&env.node);
        write_executable(&env.ocx);

        let report = discovery(&env).discover();

        assert_eq!(report.gate, EnvironmentGate::MissingNpm);
        assert!(report.short_circuited);
        assert!(report.node.found);
        assert!(!report.npm.found);
        assert!(!report.ocx.found);
    }

    #[test]
    fn missing_ocx_keeps_ready_dependencies_visible() {
        let env = environment();
        write_executable(&env.node);
        write_executable(&env.npm);

        let report = discovery(&env).discover();

        assert_eq!(report.gate, EnvironmentGate::MissingOcx);
        assert!(!report.short_circuited);
        assert!(report.node.found);
        assert!(report.npm.found);
        assert!(!report.ocx.found);
    }

    #[test]
    fn non_executable_file_is_treated_as_missing() {
        let env = environment();
        std::fs::write(&env.node, b"not executable").expect("write fixture");
        write_executable(&env.npm);
        write_executable(&env.ocx);

        let report = discovery(&env).discover();

        assert_eq!(report.gate, EnvironmentGate::MissingNode);
        assert!(report.short_circuited);
    }
}
