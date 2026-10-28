//! 平台工具发现仅检查受控候选，不搜索宿主 PATH、不执行探测命令。

use crate::types::discovery::EnvironmentPaths;
use std::path::{Path, PathBuf};

pub fn default_paths() -> Option<EnvironmentPaths> {
    let home = super::platform::home_dir()?;
    Some(paths_for_home(&home))
}

pub fn paths_for_home(home: &Path) -> EnvironmentPaths {
    let directories = candidate_dirs(home);
    EnvironmentPaths::new(
        select_tool(&directories, tool_names("node")),
        select_tool(&directories, tool_names("npm")),
        select_tool(&directories, tool_names("ocx")),
    )
}

fn tool_names(tool: &str) -> Vec<String> {
    #[cfg(windows)]
    {
        vec![format!("{tool}.exe"), format!("{tool}.cmd")]
    }
    #[cfg(not(windows))]
    {
        vec![tool.to_string()]
    }
}

/// 测试用：暴露平台可执行文件名规则，避免测试里写死无扩展名的名字。
#[cfg(test)]
pub fn tool_names_for_test(tool: &str) -> Vec<String> {
    tool_names(tool)
}

fn select_tool(directories: &[PathBuf], names: Vec<String>) -> PathBuf {
    directories
        .iter()
        .flat_map(|directory| names.iter().map(|name| directory.join(name)))
        .find(|path| path.is_file())
        .unwrap_or_else(|| directories[0].join(&names[0]))
}

pub fn candidate_dirs(home: &Path) -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        let mut directories = windows_home_candidates(home);
        // An explicit sandbox may discover only tools provisioned in its HOME.
        // Host NVM variables and Known Folders can contain the daily ocx launcher.
        if crate::modules::test_sandbox::enabled() {
            return directories;
        }
        let known_roaming = dirs::data_dir();
        let known_local = dirs::data_local_dir();
        directories = windows_declared_candidates(
            home,
            |key| std::env::var_os(key).map(PathBuf::from),
            known_roaming.as_deref(),
            known_local.as_deref(),
        );
        if let Some(prefix) = npm_prefix_from_npmrc(home) {
            directories.push(prefix);
        }
        directories
    }
    #[cfg(not(windows))]
    {
        let mut directories = unix_home_candidates(home);
        // 自定义 npm 前缀：`npm config set prefix` 写在 ~/.npmrc 里。
        // 只读配置文件，不执行 npm，符合本模块“不搜索 PATH、不跑探测命令”的约束。
        if let Some(prefix) = npm_prefix_from_npmrc(home) {
            directories.push(prefix.join("bin"));
        }
        // nvm 的 node 版本目录逐层扫描（数量有上限，避免大目录拖慢发现）。
        directories.extend(nvm_version_dirs(home));
        // Homebrew / 系统前缀放在最后：它们存在但优先级低于用户级包管理器。
        directories.extend(unix_system_candidates());
        directories
    }
}

/// 非 Windows 常见安装位置：先用户级包管理器，再 Homebrew / 系统前缀。
/// 顺序即优先级；`select_tool` 取第一个存在的可执行文件。
#[cfg(not(windows))]
fn unix_home_candidates(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".local/bin"),
        home.join(".npm-global/bin"),
        home.join(".volta/bin"),
        home.join("Library/pnpm"),
        home.join(".local/share/pnpm"),
        home.join(".bun/bin"),
    ]
}

/// Homebrew 与 npm 默认前缀在 macOS 上属于系统级位置；单独列出便于测试与非 macOS 排除。
#[cfg(target_os = "macos")]
fn unix_system_candidates() -> Vec<PathBuf> {
    vec![
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
    ]
}

#[cfg(all(not(windows), not(target_os = "macos")))]
fn unix_system_candidates() -> Vec<PathBuf> {
    vec![PathBuf::from("/usr/local/bin"), PathBuf::from("/usr/bin")]
}

/// 读取 `~/.npmrc` 的 prefix；缺失或非法时返回 None。
fn npm_prefix_from_npmrc(home: &Path) -> Option<PathBuf> {
    let text = std::fs::read_to_string(home.join(".npmrc")).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        let Some(rest) = line.strip_prefix("prefix") else {
            continue;
        };
        let Some(rest) = rest.trim_start().strip_prefix('=') else {
            continue;
        };
        let value = rest
            .trim()
            .trim_matches('"')
            .trim_matches(char::from(39))
            .trim();
        if value.is_empty() {
            continue;
        }
        let path = PathBuf::from(value);
        if path.is_absolute() {
            return Some(path);
        }
    }
    None
}

/// `~/.nvm/versions/node/<version>/bin`，只扫描一层且设上限。
#[cfg(not(windows))]
fn nvm_version_dirs(home: &Path) -> Vec<PathBuf> {
    const MAX_VERSIONS: usize = 16;
    let root = home.join(".nvm/versions/node");
    let Ok(entries) = std::fs::read_dir(&root) else {
        return Vec::new();
    };
    let mut dirs: Vec<PathBuf> = entries
        .flatten()
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.path().join("bin"))
        .take(MAX_VERSIONS)
        .collect();
    dirs.sort();
    dirs.reverse();
    dirs
}

#[cfg(any(windows, test))]
fn windows_home_candidates(home: &Path) -> Vec<PathBuf> {
    vec![
        home.join(".local/bin"),
        home.join(".nvmd/bin"),
        home.join("AppData/Roaming/npm"),
        home.join("AppData/Local/Programs/nodejs"),
    ]
}

/// Explicit declarations precede defaults. Never execute shell profiles or scan PATH.
#[cfg(any(windows, test))]
fn windows_declared_candidates(
    home: &Path,
    env: impl Fn(&str) -> Option<PathBuf>,
    roaming: Option<&Path>,
    local: Option<&Path>,
) -> Vec<PathBuf> {
    let mut result = Vec::new();
    let mut append = |path: PathBuf| {
        if path.is_absolute() && !result.contains(&path) {
            result.push(path);
        }
    };
    for key in [
        "NVM_SYMLINK",
        "NODE_HOME",
        "FNM_MULTISHELL_PATH",
        "PNPM_HOME",
    ] {
        if let Some(path) = env(key) {
            append(path);
        }
    }
    if let Some(path) = env("VOLTA_HOME") {
        append(path.join("bin"));
    }
    let mut roots = vec![home.join(".volta"), home.join(".fnm")];
    for key in ["NVM_HOME", "FNM_DIR"] {
        if let Some(path) = env(key).filter(|p| p.is_absolute()) {
            append(path.clone());
            roots.push(path);
        }
    }
    for base in [roaming, local].into_iter().flatten() {
        append(base.join("npm"));
        append(base.join("pnpm"));
        append(base.join("Volta/bin"));
        append(base.join("Programs/nodejs"));
        roots.push(base.join("fnm"));
    }
    // Per-user conventional tools precede machine-wide fallback installations.
    for path in windows_home_candidates(home) {
        append(path);
    }
    for key in ["ProgramFiles", "ProgramFiles(x86)"] {
        if let Some(path) = env(key) {
            append(path.join("nodejs"));
        }
    }
    for root in roots {
        if !root.is_absolute() {
            continue;
        }
        append(root.join("bin"));
        // Bounded, one-level recognized layouts only; active declarations above win.
        for (directory, leaf) in [
            (root.clone(), ""),
            (root.join("node-versions"), "installation"),
        ] {
            let Ok(entries) = std::fs::read_dir(&directory) else {
                continue;
            };
            let mut count = 0;
            let mut versions = Vec::new();
            for entry in entries {
                count += 1;
                if count > 128 {
                    break;
                }
                let Ok(entry) = entry else {
                    continue;
                };
                let name = entry.file_name().to_string_lossy().into_owned();
                let version = name
                    .trim_start_matches('v')
                    .split('.')
                    .map(str::parse::<u32>)
                    .collect::<Result<Vec<_>, _>>();
                if let Ok(version) = version {
                    if version.len() == 3 && entry.path().is_dir() {
                        versions.push((version, entry.path().join(leaf)));
                    }
                }
            }
            versions.sort_by(|a, b| b.0.cmp(&a.0));
            for (_, path) in versions.into_iter().take(16) {
                append(path);
            }
        }
    }
    result
}

pub fn runtime_candidates(home: &Path) -> Vec<PathBuf> {
    let mut candidates = candidate_dirs(home)
        .iter()
        .flat_map(|directory| {
            tool_names("ocx")
                .iter()
                .map(|name| directory.join(name))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    #[cfg(target_os = "macos")]
    candidates.extend([
        PathBuf::from("/opt/homebrew/bin/ocx"),
        PathBuf::from("/usr/local/bin/ocx"),
    ]);
    candidates.shrink_to_fit();
    candidates
}

pub fn process_path(home: &Path, paths: &EnvironmentPaths) -> std::ffi::OsString {
    let selected = [&paths.node, &paths.npm, &paths.ocx]
        .into_iter()
        .filter_map(|path| path.parent().map(Path::to_path_buf));
    super::platform::controlled_path(selected.chain(candidate_dirs(home)), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolated_windows_candidates_cannot_select_a_daily_launcher() {
        let root = tempfile::tempdir().unwrap();
        let isolated = root.path().join("sandbox/home");
        let daily = root.path().join("daily");
        std::fs::create_dir_all(daily.join(".local/bin")).unwrap();
        std::fs::write(daily.join(".local/bin/ocx.cmd"), b"daily fixture").unwrap();
        let directories = windows_home_candidates(&isolated);
        assert!(directories.iter().all(|path| path.starts_with(&isolated)));
        let missing = select_tool(&directories, vec!["ocx.cmd".into()]);
        assert_eq!(missing, isolated.join(".local/bin/ocx.cmd"));
        assert!(!missing.exists());
        std::fs::create_dir_all(isolated.join(".local/bin")).unwrap();
        std::fs::write(isolated.join(".local/bin/ocx.cmd"), b"isolated fixture").unwrap();
        assert_eq!(
            select_tool(&directories, vec!["ocx.cmd".into()]),
            isolated.join(".local/bin/ocx.cmd")
        );
    }

    #[test]
    fn windows_declared_managers_use_active_roots_and_bound_recognized_versions() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let active = root.path().join("active");
        let volta = root.path().join("volta-custom");
        let fnm = root.path().join("fnm-custom");
        for version in ["v22.1.0", "v24.2.0"] {
            std::fs::create_dir_all(fnm.join("node-versions").join(version).join("installation"))
                .unwrap();
        }
        let values = windows_declared_candidates(
            &home,
            |key| match key {
                "FNM_MULTISHELL_PATH" => Some(active.clone()),
                "VOLTA_HOME" => Some(volta.clone()),
                "FNM_DIR" => Some(fnm.clone()),
                _ => None,
            },
            None,
            None,
        );
        assert_eq!(values.first(), Some(&active));
        assert!(values.contains(&volta.join("bin")));
        assert!(values.contains(&fnm.join("node-versions/v24.2.0/installation")));
        assert!(values.contains(&fnm.join("node-versions/v22.1.0/installation")));
        assert!(!values.contains(&PathBuf::from("relative")));
    }

    #[cfg(not(windows))]
    #[test]
    fn finds_tools_outside_the_default_global_prefix() {
        // 非默认位置：自定义 npm prefix 与 Volta 都要能发现，不能只看 ~/.local/bin。
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let custom = root.path().join("custom-prefix");
        std::fs::create_dir_all(custom.join("bin")).unwrap();
        std::fs::create_dir_all(home.join(".volta/bin")).unwrap();
        std::fs::write(custom.join("bin/ocx"), b"fixture").unwrap();
        std::fs::write(home.join(".volta/bin/node"), b"fixture").unwrap();
        std::fs::create_dir_all(&home).unwrap();
        std::fs::write(
            home.join(".npmrc"),
            format!(
                "prefix = {}
",
                custom.display()
            ),
        )
        .unwrap();
        let paths = paths_for_home(&home);
        assert_eq!(paths.ocx, custom.join("bin/ocx"));
        assert_eq!(paths.node, home.join(".volta/bin/node"));
    }

    #[cfg(not(windows))]
    #[test]
    fn reads_npmrc_prefix_but_ignores_relative_or_commented_values() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().to_path_buf();
        std::fs::write(
            home.join(".npmrc"),
            "# prefix=/ignored
; prefix=/also-ignored
prefix = relative/bin
",
        )
        .unwrap();
        assert_eq!(npm_prefix_from_npmrc(&home), None);
        let absolute = root.path().join("abs");
        std::fs::write(
            home.join(".npmrc"),
            format!("prefix=\"{}\"\n", absolute.display()),
        )
        .unwrap();
        assert_eq!(npm_prefix_from_npmrc(&home), Some(absolute));
    }

    #[cfg(not(windows))]
    #[test]
    fn nvm_versions_are_scanned_newest_first_with_a_cap() {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().to_path_buf();
        for version in ["v18.0.0", "v20.0.0", "v22.0.0"] {
            std::fs::create_dir_all(home.join(".nvm/versions/node").join(version).join("bin"))
                .unwrap();
        }
        let dirs = nvm_version_dirs(&home);
        assert_eq!(dirs.len(), 3);
        assert!(dirs[0].ends_with("v22.0.0/bin"));
    }

    #[test]
    fn discovery_uses_controlled_user_tools_and_returns_missing_candidates() {
        let root = tempfile::tempdir().unwrap();
        let paths = paths_for_home(root.path());
        assert!(paths.node.is_absolute());
        assert!(paths.npm.is_absolute());
        assert!(paths.ocx.is_absolute());
        // Windows runner may also have an allowed system-wide Node installation.
        // Exercise the missing-candidate fallback against an explicit empty set
        // rather than depending on the host's Program Files contents.
        let empty_directory = root.path().join(".local/bin");
        let missing = select_tool(std::slice::from_ref(&empty_directory), tool_names("node"));
        assert_eq!(missing.parent(), Some(empty_directory.as_path()));
        assert!(!missing.exists());
    }

    #[cfg(windows)]
    #[test]
    fn windows_finds_nvmd_and_global_npm_wrappers_without_path_search() {
        let root = tempfile::tempdir().unwrap();
        let node_dir = root.path().join(".nvmd/bin");
        let npm_dir = root.path().join("AppData/Roaming/npm");
        std::fs::create_dir_all(&node_dir).unwrap();
        std::fs::create_dir_all(&npm_dir).unwrap();
        std::fs::write(node_dir.join("node.exe"), b"fixture").unwrap();
        std::fs::write(node_dir.join("npm.cmd"), b"fixture").unwrap();
        std::fs::write(npm_dir.join("ocx.cmd"), b"fixture").unwrap();
        // The fixture supplies its own HOME and no manager declarations; the
        // runner's active NVM_SYMLINK must not impersonate that fixture's tools.
        // Real host declarations are covered by the injected-manager cases and
        // the production default-path native acceptance.
        let directories = windows_declared_candidates(root.path(), |_| None, None, None);
        let paths = EnvironmentPaths::new(
            select_tool(&directories, tool_names("node")),
            select_tool(&directories, tool_names("npm")),
            select_tool(&directories, tool_names("ocx")),
        );
        assert_eq!(paths.node, node_dir.join("node.exe"));
        assert_eq!(paths.npm, node_dir.join("npm.cmd"));
        assert_eq!(paths.ocx, npm_dir.join("ocx.cmd"));
        let path = process_path(root.path(), &paths);
        let directories: Vec<_> = std::env::split_paths(&path).collect();
        assert!(directories.contains(&node_dir));
        assert!(directories.contains(&npm_dir));
        assert!(!path.to_string_lossy().contains("/usr/bin"));
        assert!(runtime_candidates(root.path()).contains(&npm_dir.join("ocx.cmd")));
    }
}
