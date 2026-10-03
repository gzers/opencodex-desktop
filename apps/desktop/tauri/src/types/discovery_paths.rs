//! 平台默认发现路径。首期目标限定 macOS；
//! 后续平台差异必须移动到 infrastructure 适配器，不得散落到业务模块。

pub fn macos_default_paths() -> crate::types::discovery::EnvironmentPaths {
    let home = std::env::var_os("HOME").unwrap_or_default();
    let home = std::path::PathBuf::from(home);
    crate::types::discovery::EnvironmentPaths::new(
        home.join(".local/bin/node"),
        home.join(".local/bin/npm"),
        home.join(".local/bin/ocx"),
    )
}
