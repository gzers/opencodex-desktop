//! 应用状态入口。跨命令共享的模块状态集中在这里定义。

use std::sync::{Arc, Mutex};

use crate::infrastructure::status_source::OfficialStatusSource;
use crate::modules::about::OfficialVersionSource;
use crate::modules::codex_shim::CodexShimSource;
use crate::modules::doctor::DoctorSource;
use crate::modules::notifications::NotificationStore;
use crate::modules::process::ControlledProcessRunner;
use crate::modules::status::StatusCollector;
use crate::modules::tray::TrayAction;

#[derive(Debug, Default, Clone)]
pub struct AppState;

/// 命令层共享的状态采集器；真实来源仍由注入边界决定。
pub type SharedStatusCollector = Arc<Mutex<StatusCollector<OfficialStatusSource>>>;

/// 命令层共享的受控进程运行器；保持 trait 边界便于隔离测试。
pub type SharedProcessRunner = Mutex<ControlledProcessRunner>;

/// 命令层共享的通知集合；顶栏与诊断中心消费同一份状态。
pub type SharedNotificationStore = Arc<Mutex<NotificationStore>>;

/// 命令层共享的受控 Doctor 来源；真实子进程由 infrastructure 注入。
pub type SharedDoctorSource = Arc<Mutex<dyn DoctorSource + Send>>;

/// 命令层共享的受控官方版本来源；真实子进程由 infrastructure 注入。
pub type SharedOfficialVersionSource = Arc<Mutex<dyn OfficialVersionSource + Send>>;

/// 命令层共享的受控官方 shim 开关来源；真实子进程由 infrastructure 注入。
pub type SharedCodexShimSource = Arc<Mutex<dyn CodexShimSource + Send>>;

/// 进程动作的受控上下文；运行来源、工作目录与数据根由启动接线统一决定。
///
/// `executable` 不缓存路径：每次动作都向共享句柄现取，这样托管安装 / 卸载
/// 完成后无需重建上下文（`FZ-48`：所有消费者共用同一份解析结果）。
#[derive(Debug, Clone)]
pub struct ProcessContext {
    pub runtime: crate::infrastructure::runtime_executable::SharedRuntimeExecutable,
    pub working_directory: std::path::PathBuf,
    pub opencodex_home: std::path::PathBuf,
}

impl ProcessContext {
    /// 当前 `ocx` 路径；未解析出运行来源时返回 `NotConfigured`，
    /// 由命令层转成界面可解释的失败，而不是拿空路径去 spawn。
    pub fn executable(&self) -> Result<std::path::PathBuf, crate::errors::AppError> {
        self.runtime
            .executable()
            .ok_or(crate::errors::AppError::NotConfigured)
    }
}

pub type SharedProcessContext = ProcessContext;

/// 数据根由启动接线初始化后共享给命令层；路径不进入前端 DTO。
#[derive(Debug, Clone)]
pub struct SharedDataRoot(pub std::path::PathBuf);
/// Stable bootstrap location; active stores always use SharedDataRoot.
pub struct SharedDataRootAnchor(pub std::path::PathBuf);

/// WebDAV 同步的共享运行状态；命令层与后续事件推送消费同一份状态。
pub type SharedSyncStatus = std::sync::Mutex<crate::modules::sync::SyncRun>;

/// 持有单实例锁的状态，确保锁在应用生命周期内不被释放。
#[derive(Debug)]
pub struct SharedHomeDir(pub std::path::PathBuf);

/// 跨层共享的托盘请求队列；菜单事件只发布一次。
#[derive(Debug, Default)]
pub struct SharedTrayRequests(std::sync::Mutex<Vec<TrayAction>>);

impl SharedTrayRequests {
    pub fn new() -> Self {
        Self(std::sync::Mutex::new(Vec::new()))
    }

    pub fn push(&self, action: TrayAction) {
        if let Ok(mut queue) = self.0.lock() {
            queue.push(action);
        }
    }

    pub fn drain(&self) -> Vec<TrayAction> {
        self.0
            .lock()
            .map(|mut queue| std::mem::take(&mut *queue))
            .unwrap_or_default()
    }
}

/// 托管安装 / 卸载的进行态：同一时刻只允许一个写者，并支持取消（`FZ-50`）。
#[derive(Debug, Default, Clone)]
pub struct SharedRuntimeInstall {
    state: std::sync::Arc<Mutex<Option<crate::modules::runtime::install::CancelFlag>>>,
}

/// Lives in the worker, so dropping its IPC observer cannot unlock a still-running write.
pub struct RuntimeMutationLease {
    owner: SharedRuntimeInstall,
    pub cancel: crate::modules::runtime::install::CancelFlag,
}
impl Drop for RuntimeMutationLease {
    fn drop(&mut self) {
        self.owner.finish();
    }
}

impl SharedRuntimeInstall {
    pub fn acquire(&self) -> Option<RuntimeMutationLease> {
        self.begin().map(|cancel| RuntimeMutationLease {
            owner: self.clone(),
            cancel,
        })
    }
    pub fn new() -> Self {
        Self::default()
    }

    /// 开始一次写操作；已有进行中的操作时返回 `None`（拒绝并发安装）。
    pub fn begin(&self) -> Option<crate::modules::runtime::install::CancelFlag> {
        let mut guard = self.state.lock().ok()?;
        if guard.is_some() {
            return None;
        }
        let flag = crate::modules::runtime::install::CancelFlag::new();
        *guard = Some(flag.clone());
        Some(flag)
    }

    pub fn finish(&self) {
        if let Ok(mut guard) = self.state.lock() {
            *guard = None;
        }
    }

    /// 请求取消当前操作；没有进行中的操作时返回 `false`。
    pub fn cancel(&self) -> bool {
        match self.state.lock() {
            Ok(guard) => match guard.as_ref() {
                Some(flag) => {
                    flag.cancel();
                    true
                }
                None => false,
            },
            Err(_) => false,
        }
    }

    pub fn is_running(&self) -> bool {
        self.state
            .lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }
}

pub struct InstanceState {
    pub _locks: Vec<crate::modules::instance::AppInstanceLock>,
}

#[cfg(test)]
mod mutation_tests {
    use super::SharedRuntimeInstall;

    #[test]
    fn lease_retains_shared_reservation_and_cancellation_until_worker_finishes() {
        let shared = SharedRuntimeInstall::new();
        let observer = shared.clone();
        let worker = shared.acquire().unwrap();
        drop(shared);
        assert!(observer.is_running());
        assert!(observer.acquire().is_none());
        assert!(observer.cancel());
        assert!(worker.cancel.is_cancelled());
        drop(worker);
        assert!(!observer.is_running());
        assert!(!observer.cancel());
        assert!(observer.acquire().is_some());
    }
}
