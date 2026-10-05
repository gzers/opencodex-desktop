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
        let mut directories = vec![
            home.join(".local/bin"),
            home.join(".nvmd/bin"),
            home.join("AppData/Roaming/npm"),
            home.join("AppData/Local/Programs/nodejs"),
        ];
        for key in ["NVM_SYMLINK", "NODE_HOME", "NVM_HOME"] {
            if let Some(path) = std::env::var_os(key)
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
            {
                directories.push(path);
            }
        }
        if super::platform::home_dir().as_deref() == Some(home) {
            if let Some(path) = dirs::data_dir() {
                directories.push(path.join("npm"));
            }
            if let Some(path) = dirs::data_local_dir() {
                directories.push(path.join("Programs/nodejs"));
            }
        }
        for key in ["ProgramFiles", "ProgramFiles(x86)"] {
            if let Some(path) = std::env::var_os(key)
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
            {
                directories.push(path.join("nodejs"));
            }
        }
        directories
    }
    #[cfg(not(windows))]
    {
        vec![home.join(".local/bin")]
    }
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
        let paths = paths_for_home(root.path());
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
