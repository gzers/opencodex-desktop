// 平台相关实现入口。平台差异隔离在这一层，业务模块不得直接依赖具体平台 API。

pub mod atomic_write;
pub mod codex_shim_source;
pub mod discovery_paths;
pub mod hash;
pub mod keychain;
pub mod locking;
pub mod official_cli_source;
pub mod official_version_source;
pub mod platform;
pub mod process_runner;
pub mod runtime_executable;
pub mod runtime_log;
pub mod status_source;
pub mod tray_controller;
pub mod webdav_client;

pub mod app_activity;
pub mod window_appearance;
#[cfg(any(windows, test))]
pub(crate) mod windows_smoke;
