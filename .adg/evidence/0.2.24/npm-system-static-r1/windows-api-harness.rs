pub mod errors {
 #[derive(Debug)] pub enum AppError {RuntimeManaged {code:String,detail:String}}
 pub type AppResult<T> = Result<T,AppError>;
}
pub mod modules {
 pub mod preferences {pub enum ProxyPolicy {None,System,Manual(String)}}
 pub mod process {#[derive(Default)] pub struct EnvironmentPolicy {pub http_proxy:Option<std::ffi::OsString>,pub https_proxy:Option<std::ffi::OsString>,pub no_proxy:Option<std::ffi::OsString>}}
}
#[path="/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes/apps/desktop/tauri/src/modules/network_defaults/npm_proxy.rs"]
pub mod production_adapter;
