//! 「当前用哪个 `ocx`」的读取接口（IMP `FZ-48`）。
//!
//! 状态采集、Doctor、版本检查、启停与 IPC 都只依赖这个 trait，因此它们**必然**
//! 共用同一份解析结果：实现方是 `modules::runtime::RuntimeHandle` 这个共享句柄，
//! 安装 / 卸载完成后 `refresh` 一次，所有消费者下一次读取即拿到新路径。
//!
//! trait 放在 infrastructure 是为了让基础设施层不反向依赖业务模块；
//! 具体实现由 `modules::runtime` 提供（模块层实现基础设施 trait 是允许的方向）。

use std::path::PathBuf;
use std::sync::Arc;

pub trait RuntimeExecutableProvider: Send + Sync + std::fmt::Debug {
    /// 当前已解析出的 `ocx` 绝对路径；未解析时为 `None`。
    fn executable(&self) -> Option<PathBuf>;
}

/// 命令层与基础设施层共享的只读句柄。
pub type SharedRuntimeExecutable = Arc<dyn RuntimeExecutableProvider>;

/// 固定值的来源实现：不参与解析，永远返回同一路径（或未解析）。
///
/// 用于 fixture、单元测试，以及明确知道路径、不需要来源解析的场景
/// （例如扩展管理里的外部 CLI 探测）。
#[derive(Debug, Clone)]
pub struct FixedRuntimeExecutable(pub Option<PathBuf>);

impl FixedRuntimeExecutable {
    pub fn new(path: Option<PathBuf>) -> Self {
        Self(path)
    }

    pub fn resolved(path: impl Into<PathBuf>) -> SharedRuntimeExecutable {
        Arc::new(Self(Some(path.into())))
    }

    pub fn unresolved() -> SharedRuntimeExecutable {
        Arc::new(Self(None))
    }
}

impl RuntimeExecutableProvider for FixedRuntimeExecutable {
    fn executable(&self) -> Option<PathBuf> {
        self.0.clone()
    }
}
