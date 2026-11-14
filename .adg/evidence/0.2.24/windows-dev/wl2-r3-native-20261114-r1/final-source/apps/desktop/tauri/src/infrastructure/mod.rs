// 平台相关实现入口。平台差异隔离在这一层，业务模块不得直接依赖具体平台 API。

pub mod atomic_write;
pub mod codex_shim_source;
pub mod credentials;
pub mod discovery_paths;
pub mod hash;
pub mod keychain;
pub mod locking;
#[cfg(target_os = "macos")]
pub mod macos_client_originals;
#[cfg(target_os = "macos")]
pub mod macos_process_exit;
#[cfg(target_os = "macos")]
pub mod macos_process_pair;
#[cfg(target_os = "macos")]
pub mod macos_tcp_listeners;
pub mod official_cli_source;
#[cfg(target_os = "macos")]
pub mod official_client_coordination;
pub mod official_version_source;
pub mod platform;
pub mod process_runner;
mod process_start_output;
pub mod runtime_executable;
pub mod runtime_log;
pub mod status_source;
pub(crate) mod storage_writers;
pub mod tray_controller;
pub mod webdav_client;
pub mod webdav_config_v2;

pub mod app_activity;
pub mod window_appearance;
#[cfg(windows)]
pub mod windows_cli024;
#[cfg(windows)]
pub(crate) mod windows_official_runtime024;
#[cfg(windows)]
pub(crate) mod windows_private_create;
#[cfg(all(windows, test))]
pub(crate) mod windows_query_fixture;
#[cfg(windows)]
pub(crate) mod windows_query_job;
#[cfg(any(windows, test))]
pub(crate) mod windows_smoke;
#[cfg(any(windows, test))]
pub(crate) mod windows_startup_root;
#[cfg(any(windows, test))]
pub(crate) mod windows_tray_icon;

pub(crate) mod attested_status;
#[cfg(windows)]
mod windows_client_originals024;
#[cfg(windows)]
pub(crate) mod windows_official_lifecycle024;
#[cfg(all(windows, test))]
mod windows_owner_host024;
#[cfg(windows)]
mod windows_supervisor024;
#[cfg(windows)]
mod windows_diagnostic_roles024;

#[cfg(windows)]
pub(crate) mod windows_task_scheduler;

#[cfg(any(windows, test))]
pub(crate) mod runtime_lifetime;
