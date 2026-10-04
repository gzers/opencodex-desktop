//! 业务模块聚合层。新增模块先声明边界，再在独立 TASK 授权下实现。

pub mod about;
pub mod backup;
pub mod codex_shim;
pub mod config_migration;
pub mod container;
pub mod data_root;
pub mod discovery;
pub mod doctor;
pub mod extensions;
pub mod instance;
pub mod ipc;
pub mod logs;
pub mod migration;
pub mod notifications;
pub mod preferences;
pub mod process;
pub mod runtime;
pub mod runtime_defaults;
pub mod skills;
pub mod status;
pub mod sync;
pub mod test_sandbox;
pub mod tray;
pub mod update;
pub mod write;

pub fn __marker() {}
