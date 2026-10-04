//! 应用自更新领域模型与错误投影。
//!
//! 领域层只维护通道、当前版本、签名校验结论和失败保留语义；
//! 真实网络、下载与安装由 Tauri updater 基础设施执行。

use serde::{Deserialize, Serialize};

pub const CHANNELS: [&str; 2] = ["stable", "beta"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateChannel {
    Stable,
    Beta,
}

impl UpdateChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "stable" => Some(Self::Stable),
            "beta" => Some(Self::Beta),
            _ => None,
        }
    }

    /// 通道端点（U-05）：单一来源是固化运行策略的 updates.desktop.channels，
    /// 领域层不再另写一份假地址。检查、安装与后台调度都从这里取。
    pub fn endpoint(&self) -> &'static str {
        crate::modules::runtime_defaults::desktop_update_endpoint(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UpdateStatus {
    pub channel: UpdateChannel,
    pub current_version: String,
    pub available_version: Option<String>,
    pub last_checked_at: Option<String>,
    pub signature_verified: Option<bool>,
    pub error: Option<String>,
}

impl UpdateStatus {
    pub fn pending(current_version: impl Into<String>) -> Self {
        Self {
            channel: UpdateChannel::Stable,
            current_version: current_version.into(),
            available_version: None,
            last_checked_at: None,
            signature_verified: None,
            error: None,
        }
    }

    /// 用偏好里的「更新通道与频率」初始化通道；`manual` 仍落在稳定通道，
    /// 只是不参与自动检查频率。
    pub fn with_channel(mut self, channel: UpdateChannel) -> Self {
        self.channel = channel;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateFailureClass {
    Network,
    Signature,
    Install,
    Cancelled,
    Internal,
}

impl UpdateFailureClass {
    pub fn message(&self) -> &'static str {
        match self {
            Self::Network => "更新下载或连接失败；已保留当前版本。",
            Self::Signature => "更新签名校验失败；已终止安装并保留当前版本。",
            Self::Install => "更新安装失败；已保留当前版本。",
            Self::Cancelled => "更新已取消；已保留当前版本。",
            Self::Internal => "更新内部错误；已保留当前版本。",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channels_match_frozen_contract() {
        assert_eq!(CHANNELS, ["stable", "beta"]);
        assert_eq!(UpdateChannel::parse("stable"), Some(UpdateChannel::Stable));
        assert_eq!(UpdateChannel::parse("beta"), Some(UpdateChannel::Beta));
        assert_eq!(UpdateChannel::parse("nightly"), None);
        assert!(UpdateChannel::Stable.endpoint().starts_with("https://"));
        assert!(UpdateChannel::Stable.endpoint().contains("stable"));
        assert!(UpdateChannel::Beta.endpoint().contains("beta"));
    }

    #[test]
    fn failures_never_replace_current_version() {
        assert!(UpdateFailureClass::Signature
            .message()
            .contains("已终止安装并保留当前版本"));
        assert!(UpdateFailureClass::Network
            .message()
            .contains("已保留当前版本"));
        assert!(UpdateFailureClass::Install
            .message()
            .contains("已保留当前版本"));
    }
}
