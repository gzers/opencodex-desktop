//! 跨命令、模块与测试共享的传输类型。

pub mod about;
pub mod cleanup;
pub mod codex_shim;
pub mod discovery;
pub mod discovery_paths;
pub mod doctor;
pub mod extensions;
pub mod logs;
pub mod migration;
pub mod notifications;
pub mod preferences;
pub mod process_action;
pub mod runtime;
pub mod runtime_status;
pub mod status;
pub mod update;
pub mod upgrade;
pub mod workspace;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppMode {
    Shell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStatus {
    pub mode: AppMode,
    pub ready: bool,
}
