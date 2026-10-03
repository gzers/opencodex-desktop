//! 设置页工作台命令：受控路径投影、显式打开、外链与内置文档。
//!
//! 路径不来自前端信任输入；前端传入路径仍必须命中后端白名单。打开只
//! 调用系统相关动作，不读取或改写目标内容。

use std::path::{Path, PathBuf};

use tauri::State;

use crate::errors::{AppError, AppResult};
use crate::modules::extensions::ClientId;
use crate::state::{SharedDataRoot, SharedHomeDir};
use crate::types::workspace::{
    AgentPathDto, AgentPathKind, ManagedPathDto, ManagedPathKey, ManagedPathTargetsDto,
    OpenResultDto,
};

#[tauri::command]
pub fn get_managed_path_targets(
    data_root: State<'_, SharedDataRoot>,
    home: State<'_, SharedHomeDir>,
) -> AppResult<ManagedPathTargetsDto> {
    Ok(managed_path_targets_with_roots(&data_root.0, &home.0))
}

#[tauri::command]
pub fn open_managed_path(
    path: String,
    data_root: State<'_, SharedDataRoot>,
    home: State<'_, SharedHomeDir>,
) -> AppResult<OpenResultDto> {
    open_managed_path_with_roots(&data_root.0, &home.0, &path)
}

#[tauri::command]
pub fn open_external_link(url: String) -> AppResult<OpenResultDto> {
    open_external_link_with_url(&url)
}

#[tauri::command]
pub fn open_local_document(document_id: String) -> AppResult<OpenResultDto> {
    open_local_document_with_id(&document_id)
}

pub fn managed_path_targets_with_roots(data_root: &Path, home: &Path) -> ManagedPathTargetsDto {
    let managed = [
        (
            ManagedPathKey::ManagerState,
            "manager-state",
            "管理器设置目录",
        ),
        (ManagedPathKey::Backups, "backups", "备份目录"),
        (ManagedPathKey::Logs, "logs", "日志目录"),
        (ManagedPathKey::Exports, "exports", "导出目录"),
        (ManagedPathKey::Cache, "cache", "缓存目录"),
        (ManagedPathKey::SyncState, "sync-state", "同步状态目录"),
    ]
    .into_iter()
    .map(|(key, relative, label)| ManagedPathDto {
        key,
        path: data_root.join(relative).display().to_string(),
        label: label.to_string(),
    })
    .collect();

    // FZ-23：源目录是配置值——默认为 Agent Skills 共享目录
    // （`<主目录>/.agents/skills`），也可以是用户配置的自定义目录。
    // 管理器写入区仍在数据根内，仅用于导入 / 恢复 / 备份 / 导出。
    let extension_config = crate::modules::extensions::projection::load_config_lenient(data_root);
    let preferred_skills =
        crate::modules::extensions::source_dir::configured_source_dir(&extension_config, home);
    let mut agent = vec![AgentPathDto {
        kind: AgentPathKind::SkillsSource,
        client: "agents".to_string(),
        label: "Skills 源目录".to_string(),
        path: preferred_skills.display().to_string(),
    }];
    for client in [
        ClientId::Codex,
        ClientId::Claude,
        ClientId::Gemini,
        ClientId::Grok,
        ClientId::Opencode,
        ClientId::Hermes,
    ] {
        let target =
            crate::modules::extensions::ClientTarget::user_target(client, home, true, true);
        agent.push(AgentPathDto {
            kind: AgentPathKind::Skills,
            client: serde_json::to_value(target.client_id)
                .expect("client id")
                .as_str()
                .unwrap_or_default()
                .to_string(),
            label: format!("{} Skills", target.display_name),
            path: target.skills_dir.display().to_string(),
        });
        agent.push(AgentPathDto {
            kind: AgentPathKind::Mcp,
            client: serde_json::to_value(target.client_id)
                .expect("client id")
                .as_str()
                .unwrap_or_default()
                .to_string(),
            label: format!("{} MCP 配置", target.display_name),
            path: target.mcp_config_path.display().to_string(),
        });
    }
    ManagedPathTargetsDto { managed, agent }
}

pub fn open_managed_path_with_roots(
    data_root: &Path,
    home: &Path,
    requested_path: &str,
) -> AppResult<OpenResultDto> {
    let request = PathBuf::from(requested_path);
    let targets = managed_path_targets_with_roots(data_root, home);
    if let Some(allowed) = targets
        .managed
        .iter()
        .map(|item| PathBuf::from(&item.path))
        .chain(targets.agent.iter().map(|item| PathBuf::from(&item.path)))
        .find(|path| path == &request)
    {
        if !allowed.exists() {
            return Ok(OpenResultDto { opened: false });
        }
        return open_path(&allowed).map(|_| OpenResultDto { opened: true });
    }
    // 数据根、用户主目录与真实 OPENCODEX_HOME 都允许打开。OPENCODEX_HOME 由
    // 数据根运行配置解析：inside 模式为 `<数据根>/opencodex-home`，external 模式为
    // 配置中的外部路径。此前只把 OS 主目录当作 OPENCODEX_HOME，导致前端传入的
    // 真实 `opencodex-home` 路径未命中白名单而静默失败（概览与安装配置的「打开」）。
    let resolved_opencodex_home = crate::modules::data_root::load_runtime_config(data_root)
        .ok()
        .map(|config| crate::modules::data_root::resolve_opencodex_home(&config));
    let direct_roots = [
        Some(data_root.to_path_buf()),
        Some(home.to_path_buf()),
        resolved_opencodex_home,
    ];
    for root in direct_roots.into_iter().flatten() {
        if request == root && root.is_dir() {
            return open_path(&root).map(|_| OpenResultDto { opened: true });
        }
    }
    Err(AppError::NotConfigured)
}

pub fn open_external_link_with_url(url: &str) -> AppResult<OpenResultDto> {
    let allowed = [
        "https://opencodex.dev",
        "https://github.com/lidge-jun/OpenCodex",
        "https://github.com/lidge-jun/OpenCodex/issues",
        "https://github.com/lidge-jun/OpenCodex/blob/main/LICENSE",
        "https://github.com/gzers/opencodex-desktop",
        "https://github.com/gzers/opencodex-desktop/issues",
    ];
    let is_local_panel = is_local_panel_url(url);
    if !allowed.contains(&url) && !is_local_panel {
        return Err(AppError::NotConfigured);
    }
    open_url(url).map(|_| OpenResultDto { opened: true })
}

/// 官方面板失败回退只允许运行期发现的 `http://127.0.0.1:<port>`；
/// 不接受路径、凭据、非本机主机或其他协议。
fn is_local_panel_url(url: &str) -> bool {
    let prefix = "http://127.0.0.1:";
    if !url.starts_with(prefix) {
        return false;
    }
    let rest = &url[prefix.len()..];
    let (port, path) = rest.split_once('/').unwrap_or((rest, ""));
    match (port, path) {
        (_, "web") => !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) && port != "web",
        (_, "") => {
            !port.is_empty() && !rest.ends_with('/') && port.chars().all(|c| c.is_ascii_digit())
        }
        _ => false,
    }
}

pub fn open_local_document_with_id(document_id: &str) -> AppResult<OpenResultDto> {
    let target = match document_id {
        "license" => project_document(&["LICENSE"]),
        "third-party" => {
            project_document(&["docs/04-项目资料/04-品牌与图标素材/01-原始素材/LICENSE"])
        }
        _ => {
            return Err(AppError::NotFound {
                entity: "local document".to_string(),
            })
        }
    };
    if !target.exists() {
        return Ok(OpenResultDto { opened: false });
    }
    open_path(&target).map(|_| OpenResultDto { opened: true })
}

fn project_document(relative: &[&str]) -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // 目录布局为 `<repo>/apps/desktop/tauri`，仓库根在清单目录之上 3 层
    // （`ancestors()` 第 0 项是清单目录本身，见 IMP-03 目录迁移）。
    let project_root = manifest_dir
        .ancestors()
        .nth(3)
        .expect("cargo manifest lives at apps/desktop/tauri inside the project root");
    let mut path = project_root.to_path_buf();
    for segment in relative {
        path.push(segment);
    }
    path
}

thread_local! {
    static TEST_OPEN_PATHS: std::cell::RefCell<Vec<PathBuf>> =
        const { std::cell::RefCell::new(Vec::new()) };
    static TEST_OPEN_URLS: std::cell::RefCell<Vec<String>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
fn open_path(path: &Path) -> AppResult<()> {
    TEST_OPEN_PATHS.with(|calls| calls.borrow_mut().push(path.to_path_buf()));
    Ok(())
}

#[cfg(test)]
fn open_url(url: &str) -> AppResult<()> {
    TEST_OPEN_URLS.with(|calls| calls.borrow_mut().push(url.to_string()));
    Ok(())
}

#[cfg(test)]
fn recorded_open_paths() -> Vec<PathBuf> {
    TEST_OPEN_PATHS.with(|calls| calls.borrow().clone())
}

#[cfg(test)]
fn recorded_open_urls() -> Vec<String> {
    TEST_OPEN_URLS.with(|calls| calls.borrow().clone())
}

#[cfg(all(target_os = "macos", not(test)))]
fn open_path(path: &Path) -> AppResult<()> {
    std::process::Command::new("/usr/bin/open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| AppError::FileSystem {
            operation: "open managed path".to_string(),
            detail: error.to_string(),
        })
}

#[cfg(all(target_os = "windows", not(test)))]
fn open_path(path: &Path) -> AppResult<()> {
    std::process::Command::new("explorer")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| AppError::FileSystem {
            operation: "open managed path".to_string(),
            detail: error.to_string(),
        })
}

#[cfg(all(unix, not(target_os = "macos"), not(target_os = "windows"), not(test)))]
fn open_path(path: &Path) -> AppResult<()> {
    std::process::Command::new("xdg-open")
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| AppError::FileSystem {
            operation: "open managed path".to_string(),
            detail: error.to_string(),
        })
}

#[cfg(all(
    not(any(target_os = "macos", target_os = "windows")),
    not(all(unix, not(target_os = "macos"), not(target_os = "windows"))),
    not(test)
))]
fn open_path(_path: &Path) -> AppResult<()> {
    Err(AppError::NotConfigured)
}

#[cfg(all(target_os = "macos", not(test)))]
fn open_url(url: &str) -> AppResult<()> {
    std::process::Command::new("/usr/bin/open")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|error| AppError::FileSystem {
            operation: "open external link".to_string(),
            detail: error.to_string(),
        })
}

#[cfg(all(target_os = "windows", not(test)))]
fn open_url(url: &str) -> AppResult<()> {
    std::process::Command::new("cmd")
        .args(["/C", "start", "", url])
        .spawn()
        .map(|_| ())
        .map_err(|error| AppError::FileSystem {
            operation: "open external link".to_string(),
            detail: error.to_string(),
        })
}

#[cfg(all(unix, not(target_os = "macos"), not(target_os = "windows"), not(test)))]
fn open_url(url: &str) -> AppResult<()> {
    std::process::Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|error| AppError::FileSystem {
            operation: "open external link".to_string(),
            detail: error.to_string(),
        })
}

#[cfg(all(
    not(any(target_os = "macos", target_os = "windows")),
    not(all(unix, not(target_os = "macos"), not(target_os = "windows"))),
    not(test)
))]
fn open_url(_url: &str) -> AppResult<()> {
    Err(AppError::NotConfigured)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn managed_targets_use_frozen_partitions_and_extension_paths() {
        let root = tempfile::tempdir().expect("temp root");
        let home = tempfile::tempdir().expect("temp home");
        crate::modules::data_root::initialize(root.path()).expect("initialize");
        let targets = managed_path_targets_with_roots(root.path(), home.path());
        assert_eq!(targets.managed.len(), 6);
        assert_eq!(
            targets.managed[0].path,
            root.path().join("manager-state").display().to_string()
        );
        assert_eq!(targets.agent.len(), 13);
        // 默认读源 = Agent Skills 共享目录（不再是数据根内的 skills-store）。
        assert_eq!(
            targets.agent[0].path,
            home.path().join(".agents/skills").display().to_string()
        );
        assert_eq!(targets.agent[0].client, "agents");
        assert_eq!(targets.agent[1].client, "codex");
        assert_eq!(
            targets.agent[2].path,
            home.path().join(".codex/config.toml").display().to_string()
        );
        assert_eq!(
            targets.agent[3].path,
            home.path().join(".claude/skills").display().to_string()
        );
    }

    #[test]
    fn open_data_root_and_opencodex_home_are_allowed() {
        let root = tempfile::tempdir().expect("temp root");
        let home = tempfile::tempdir().expect("temp home");
        crate::modules::data_root::initialize(root.path()).expect("initialize");

        let result = open_managed_path_with_roots(
            root.path(),
            home.path(),
            &root.path().display().to_string(),
        )
        .expect("open data root");
        assert!(result.opened);
        assert!(recorded_open_paths().contains(&root.path().to_path_buf()));

        // 真实 OPENCODEX_HOME：inside 模式解析为 `<数据根>/opencodex-home`。
        // 前端传入的正是这个路径；它必须命中白名单，否则按钮会静默失败。
        let opencodex_home = root.path().join("opencodex-home");
        std::fs::create_dir_all(&opencodex_home).expect("create opencodex home");
        let result = open_managed_path_with_roots(
            root.path(),
            home.path(),
            &opencodex_home.display().to_string(),
        )
        .expect("open OPENCODEX_HOME");
        assert!(result.opened);
        assert!(recorded_open_paths().contains(&opencodex_home));

        // 用户主目录本身仍允许打开（退出通道不回退）。
        let result = open_managed_path_with_roots(
            root.path(),
            home.path(),
            &home.path().display().to_string(),
        )
        .expect("open home");
        assert!(result.opened);
    }

    #[test]
    fn open_path_rejects_non_whitelisted_value() {
        let root = tempfile::tempdir().expect("temp root");
        let home = tempfile::tempdir().expect("temp home");
        crate::modules::data_root::initialize(root.path()).expect("initialize");
        let outside = root.path().join("outside");
        fs::create_dir_all(&outside).expect("create outside");
        assert!(matches!(
            open_managed_path_with_roots(root.path(), home.path(), &outside.display().to_string()),
            Err(AppError::NotConfigured)
        ));
    }

    #[test]
    fn open_path_reports_missing_allowed_target_without_platform_call() {
        let root = tempfile::tempdir().expect("temp root");
        let home = tempfile::tempdir().expect("temp home");
        crate::modules::data_root::initialize(root.path()).expect("initialize");
        let exports = root.path().join("exports");
        if exports.exists() {
            std::fs::remove_dir(&exports).expect("remove exports");
        }
        let missing = root.path().join("exports");
        let target = managed_path_targets_with_roots(root.path(), home.path())
            .managed
            .into_iter()
            .find(|item| item.key == ManagedPathKey::Exports)
            .expect("exports target");
        assert_eq!(target.path, missing.display().to_string());
        let result = open_managed_path_with_roots(root.path(), home.path(), &target.path)
            .expect("missing managed path is explicit");
        assert!(!result.opened);
        assert!(recorded_open_paths().is_empty());
    }

    #[test]
    fn external_links_are_frozen_allowlist() {
        assert!(matches!(
            open_external_link_with_url("https://example.com"),
            Err(AppError::NotConfigured)
        ));
        assert!(recorded_open_urls().is_empty());
        let result = open_external_link_with_url("https://opencodex.dev").expect("open url");
        assert!(result.opened);
        assert_eq!(recorded_open_urls(), vec!["https://opencodex.dev"]);
    }

    #[test]
    fn local_panel_browser_fallback_is_runtime_port_allowlist() {
        assert!(is_local_panel_url("http://127.0.0.1:10100"));
        assert!(is_local_panel_url("http://127.0.0.1:10100/web"));

        for url in [
            "http://localhost:10100",
            "http://0.0.0.0:10100",
            "http://127.0.0.1:10100/",
            "http://127.0.0.1:10100/other",
            "http://127.0.0.1:10100/web/secret",
            "http://127.0.0.1:not-a-port/web",
            "https://127.0.0.1:10100",
            "http://127.0.0.1:10100?x=1",
        ] {
            assert!(!is_local_panel_url(url), "must reject {url}");
        }
    }

    #[test]
    fn local_documents_are_frozen() {
        assert!(matches!(
            open_local_document_with_id("unknown"),
            Err(AppError::NotFound { entity }) if entity == "local document"
        ));
        let result = open_local_document_with_id("license").expect("license");
        assert!(result.opened);
        let result = open_local_document_with_id("third-party").expect("third-party");
        assert!(result.opened);
        assert_eq!(
            recorded_open_paths(),
            vec![
                project_document(&["LICENSE"]),
                project_document(&["docs/04-项目资料/04-品牌与图标素材/01-原始素材/LICENSE"]),
            ]
        );
    }
}
