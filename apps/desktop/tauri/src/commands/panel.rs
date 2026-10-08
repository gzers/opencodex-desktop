//! 官方面板使用主窗口的子 WebView，不使用 iframe、不创建第二个窗口。
//! 仅主壳可调用；URL 来自后端运行快照，官方页面不获得管理器 IPC 权限。

use crate::errors::{AppError, AppResult};
use crate::state::SharedStatusCollector;
use crate::types::status::RuntimeState;
use serde::{Deserialize, Serialize};
use tauri::Manager;

pub const PANEL_VIEW_LABEL: &str = "official-panel-content";
const PANEL_SIDEBAR_WIDTH: f64 = 64.0;
const PANEL_TITLEBAR_HEIGHT: f64 = 28.0;
/// 面板内容页左上角圆角，与主壳 `.main` 的 `--radius`（12px）同源；按界面缩放折算成窗口坐标。
const PANEL_CORNER_RADIUS: f64 = 12.0;
const HUB_SCRIPT: &str = include_str!("../../assets/panel-hub.js");

#[derive(Default)]
pub struct PanelLifetime(pub std::sync::Mutex<PanelIdle>);
#[derive(Default)]
pub struct PanelIdle {
    generation: u64,
    hidden: bool,
    since: Option<std::time::Instant>,
    origin: String,
    path: String,
    fragment: String,
    scroll: f64,
}
impl PanelIdle {
    fn accepts(&self, generation: u64) -> bool {
        self.hidden
            && self.generation == generation
            && self.since.is_some_and(|at| {
                at.elapsed() >= crate::modules::runtime_defaults::panel_idle_timeout()
            })
    }
}
const IDLE_GUARD_SCRIPT: &str = include_str!("../../assets/panel-idle.js");

fn reclaim_if_idle(app: &tauri::AppHandle, target: &tauri::Url) {
    let query: std::collections::HashMap<_, _> = target.query_pairs().collect();
    let Some(generation) = query
        .get("generation")
        .and_then(|value| value.parse::<u64>().ok())
    else {
        return;
    };
    let fragment = query
        .get("fragment")
        .map(|value| value.to_string())
        .unwrap_or_default();
    let path = query
        .get("path")
        .map(|value| value.to_string())
        .unwrap_or_else(|| "/web".into());
    if path.len() > 2048 || !(path == "/web" || path.starts_with("/web/")) {
        return;
    }
    let scroll = query
        .get("scroll")
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value >= 0.0)
        .unwrap_or_default();
    if fragment.len() > 2048 {
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        // Serialize Show/Hide/reclaim with the same lock: a stale timer cannot
        // destroy a panel just reopened by the user.
        let lifetime = app.state::<PanelLifetime>();
        let Ok(mut idle) = lifetime.0.lock() else {
            return;
        };
        if !idle.accepts(generation) {
            return;
        }
        if let Some(view) = app.get_webview(PANEL_VIEW_LABEL) {
            if view.close().is_ok() {
                idle.path = path;
                idle.fragment = fragment;
                idle.scroll = scroll;
            }
        }
    });
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PanelAction {
    Show,
    Layout,
    Hide,
}

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct PanelBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}
impl PanelBounds {
    /// 校验主壳上报的面板区域。
    ///
    /// 坐标是主壳 WebView 的窗口坐标，而主壳整体带 CSS `zoom`（`--ui-zoom`，
    /// 见 `apps/desktop/ui/src/styles/base.css`），因此边界常量必须跟着缩放走：
    /// 侧栏 `--side-w`（面板路由 64px）不除以 zoom，缩放后的实际占位是 `64 * zoom`；
    /// 标题栏按 `calc(28px / zoom)` 反向补偿，缩放后恒为 28 物理像素。
    /// 此前这里按常量 64 判定，界面缩放一旦小于 100%（例如 98%）面板就会被误拒。
    fn validate(&self, width: f64, height: f64, zoom: f64) -> AppResult<()> {
        let sidebar = PANEL_SIDEBAR_WIDTH * zoom;
        let titlebar = PANEL_TITLEBAR_HEIGHT;
        if ![self.x, self.y, self.width, self.height]
            .iter()
            .all(|n| n.is_finite())
            || self.x + 0.5 < sidebar
            || self.y + 0.5 < titlebar
            || self.width < 1.0
            || self.height < 1.0
            || self.x + self.width > width + 1.0
            || self.y + self.height > height + 1.0
        {
            return Err(AppError::NotConfigured);
        }
        Ok(())
    }
    fn rect(self) -> tauri::Rect {
        tauri::Rect {
            position: tauri::LogicalPosition::new(self.x, self.y).into(),
            size: tauri::LogicalSize::new(self.width, self.height).into(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelRequest {
    pub action: PanelAction,
    pub bounds: Option<PanelBounds>,
    #[serde(default)]
    pub reload: bool,
    pub scale: Option<f64>,
    /// 解析后的界面主题（`light` / `dark`）；只用于管理器注入的浮层，不改写官方页面样式。
    pub theme: Option<String>,
    /// 管理器的主题**设置**（`light` / `dark` / `system`）。官方面板自己也用
    /// `localStorage["ocx-theme"]` 决定明暗，这里按它的机制同步，不注入样式。
    pub theme_setting: Option<String>,
    pub effects: Option<String>,
    pub toast: Option<String>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PanelResult {
    pub visible: bool,
    pub panel_url: Option<String>,
}

pub fn validate_panel_url(url: &str) -> AppResult<tauri::Url> {
    let parsed = tauri::Url::parse(url).map_err(|_| AppError::NotConfigured)?;
    if parsed.scheme() != "http"
        || parsed.host_str() != Some("127.0.0.1")
        || parsed.port().filter(|port| *port > 0).is_none()
        || parsed.path() != "/web"
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(AppError::NotConfigured);
    }
    Ok(parsed)
}

/// 官方面板的主题存储键：与官方页面自己使用的键一致（`ocx-theme`）。
const PANEL_THEME_STORAGE_KEY: &str = "ocx-theme";
const PANEL_THEME_LIGHT: &str = "light";
const PANEL_THEME_DARK: &str = "dark";
const PANEL_THEME_SYSTEM: &str = "system";

/// 规范化管理器的主题设置；未知值按 `system`（跟随系统），不向面板注入任意字符串。
fn normalize_theme_setting(value: Option<&str>) -> &'static str {
    match value {
        Some(PANEL_THEME_LIGHT) => PANEL_THEME_LIGHT,
        Some(PANEL_THEME_DARK) => PANEL_THEME_DARK,
        _ => PANEL_THEME_SYSTEM,
    }
}

/// 面板页启动脚本：先按管理器的主题设置写入官方页面自己的存储，再挂快捷浮层。
///
/// 官方页面在首屏前读 `localStorage["ocx-theme"]`（`data-theme` 也由它自己维护），
/// 所以这里只写它自己的键、不注入样式：面板因此在首屏就是正确明暗，不会闪一下再切换。
fn panel_initialization_script(theme_setting: &str) -> String {
    let setting = serde_json::json!(theme_setting).to_string();
    format!(
        "(() => {{ try {{ const key = '{PANEL_THEME_STORAGE_KEY}';\
         if ({setting} === '{PANEL_THEME_SYSTEM}') {{ localStorage.removeItem(key); }}\
         else {{ localStorage.setItem(key, {setting}); }}\
         }} catch (error) {{}} }})();\n{HUB_SCRIPT}"
    )
}

/// 面板圆角半径：与主壳 `.main` 的 `--radius` 同源，按界面缩放折算成窗口坐标。
fn corner_radius_for_scale(scale: f64) -> f64 {
    (PANEL_CORNER_RADIUS * scale / 100.0).max(0.0)
}

/// 官方面板是原生子 WebView，与主 WebView 不是同一合成层：主壳 `.main` 的
/// `border-top-left-radius` + `overflow:hidden` 只能裁剪 DOM，裁不到这块原生视图，
/// 面板左上角因此是方的（把主壳的圆角整个盖住）。
///
/// 这里把同一份圆角直接打到原生视图的 layer 上（随界面缩放同步半径）。wry 会把子
/// WebView 包一层容器视图，容器本身就是方角且不透明，所以容器那一层也必须一起圆，
/// 否则面板角落仍是方的。为免误伤主 WebView，只对类名为 wry 容器（`WryWebViewParent`）
/// 的父视图下钻，最多一层。
fn apply_panel_corner_radius<R: tauri::Runtime>(view: &tauri::Webview<R>, radius: f64) {
    #[cfg(target_os = "macos")]
    {
        use objc2::msg_send;
        use objc2::runtime::{AnyClass, AnyObject, Bool};

        /// 给单个视图的 layer 打左上角圆角。
        unsafe fn round(view: *mut AnyObject, radius: f64) {
            let _: () = msg_send![view, setWantsLayer: Bool::YES];
            let layer: *mut AnyObject = msg_send![view, layer];
            if layer.is_null() {
                return;
            }
            let _: () = msg_send![layer, setCornerRadius: radius];
            let _: () = msg_send![layer, setMasksToBounds: Bool::YES];
            // kCALayerMinXMinYCorner = 1 << 0：只圆面板自身的左上角，其余三角贴窗口边缘保持方角。
            let _: () = msg_send![layer, setMaskedCorners: 1_usize << 0];
        }

        fn class_name(object: *mut AnyObject) -> String {
            unsafe {
                let cls: *const AnyClass = msg_send![object, class];
                if cls.is_null() {
                    return String::new();
                }
                (*cls).name().to_string_lossy().into_owned()
            }
        }

        let radius = radius.max(0.0);
        let _ = view.with_webview(move |platform| unsafe {
            let native = platform.inner() as *mut AnyObject;
            if native.is_null() {
                return;
            }
            round(native, radius);
            let mut current = native;
            for _ in 0..2 {
                let parent: *mut AnyObject = msg_send![current, superview];
                if parent.is_null() {
                    break;
                }
                if !class_name(parent).contains("WryWebViewParent") {
                    break;
                }
                round(parent, radius);
                current = parent;
            }
        });
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (view, radius);
    }
}

fn notify_main<R: tauri::Runtime>(app: &tauri::AppHandle<R>, kind: &str, value: &str) {
    // Values are JSON encoded, never concatenated as executable HTML/JS.
    if let Some(main) = app.get_webview("main") {
        let detail = serde_json::json!({ "kind": kind, "value": value });
        let _ = main.eval(format!(
            "window.dispatchEvent(new CustomEvent('ocxd-panel', {{detail:{detail}}}))"
        ));
    }
}

fn navigation_allowed(url: &tauri::Url, origin: &tauri::Url) -> bool {
    url.origin() == origin.origin()
        && url.username().is_empty()
        && url.password().is_none()
        && (url.path() == "/web" || url.path().starts_with("/web/"))
}

#[tauri::command]
pub async fn sync_embedded_panel(
    app: tauri::AppHandle,
    webview: tauri::Webview,
    collector: tauri::State<'_, SharedStatusCollector>,
    request: PanelRequest,
) -> AppResult<PanelResult> {
    if webview.label() != "main" {
        return Err(AppError::NotConfigured);
    }
    let collector = collector.inner().clone();
    // add_child waits for the native event loop: never call it from that loop.
    tauri::async_runtime::spawn_blocking(move || {
        let lifetime = app.state::<PanelLifetime>();
        let mut idle = lifetime.0.lock().map_err(|_| AppError::NotConfigured)?;
        if request.action == PanelAction::Hide {
            if let Some(view) = app.get_webview(PANEL_VIEW_LABEL) { view.hide()?; }
            if !idle.hidden {
                idle.hidden = true;
                idle.generation = idle.generation.wrapping_add(1);
                idle.since = Some(std::time::Instant::now());
                let generation = idle.generation;
                let timer_app = app.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(crate::modules::runtime_defaults::panel_idle_timeout()).await;
                    let lifetime = timer_app.state::<PanelLifetime>();
                    if !lifetime.0.lock().is_ok_and(|idle| idle.accepts(generation)) { return; }
                    if let Some(view) = timer_app.get_webview(PANEL_VIEW_LABEL) {
                        let _ = view.eval(format!("window.__ocxdTryReclaim?.({generation})"));
                    }
                });
            }
            return Ok(PanelResult { visible: false, panel_url: None });
        }
        let window = app.get_window("main").ok_or(AppError::NotConfigured)?;
        let size = window.inner_size()?.to_logical::<f64>(window.scale_factor()?);
        let scale = request.scale.unwrap_or(100.0);
        if !scale.is_finite() || !(50.0..=200.0).contains(&scale) { return Err(AppError::NotConfigured); }
        let theme_setting = normalize_theme_setting(request.theme_setting.as_deref());
        let bounds = request.bounds.ok_or(AppError::NotConfigured)?;
        bounds.validate(size.width, size.height, scale / 100.0)?;
        if request.action == PanelAction::Layout && app.get_webview(PANEL_VIEW_LABEL).is_none() {
            return Ok(PanelResult { visible: false, panel_url: None });
        }

        if request.action == PanelAction::Show {
            idle.hidden = false;
            idle.since = None;
            idle.generation = idle.generation.wrapping_add(1);
        }
        let url = if request.action == PanelAction::Show {
            let guard = collector.lock().map_err(|_| AppError::NotConfigured)?;
            if guard.matrix().runtime != RuntimeState::Running { return Err(AppError::NotConfigured); }
            Some(validate_panel_url(&format!("http://127.0.0.1:{}/web", guard.port().ok_or(AppError::NotConfigured)?))?)
        } else { None };

        if let Some(url) = url.as_ref() {
            let origin = url.origin().ascii_serialization();
            if idle.origin != origin { idle.origin = origin; idle.path.clear(); idle.fragment.clear(); idle.scroll = 0.0; }
        }
        if let (Some(existing), Some(url)) = (app.get_webview(PANEL_VIEW_LABEL), url.as_ref()) {
            if existing.url()?.origin() != url.origin() { existing.close()?; }
        }
        let view = if let Some(view) = app.get_webview(PANEL_VIEW_LABEL) { view } else {
            let mut url = url.clone().ok_or(AppError::NotConfigured)?;
            if !request.reload {
                if !idle.path.is_empty() { url.set_path(&idle.path); }
                if !idle.fragment.is_empty() { url.set_fragment(Some(&idle.fragment)); }
            }
            let resume_scroll = if request.reload { 0.0 } else { idle.scroll };
            let restore_once = std::sync::atomic::AtomicBool::new(true);
            let origin = url.clone();
            let navigation_app = app.clone();
            let load_app = app.clone();
            let builder = tauri::webview::WebviewBuilder::new(PANEL_VIEW_LABEL, tauri::WebviewUrl::External(url))
                .focused(false)
                .initialization_script(panel_initialization_script(theme_setting))
                .initialization_script(IDLE_GUARD_SCRIPT)
                .on_navigation(move |target| {
                    if target.scheme() == "ocxd-panel" {
                        if target.host_str() == Some("reclaim") { reclaim_if_idle(&navigation_app, target); return false; }
                        if let Some(action @ ("reload" | "browser" | "zoom-in" | "zoom-out")) = target.host_str() {
                            notify_main(&navigation_app, "action", action);
                        }
                        return false;
                    }
                    navigation_allowed(target, &origin)
                })
                .on_new_window(|_, _| tauri::webview::NewWindowResponse::Deny)
                .on_page_load(move |view, payload| {
                    if matches!(payload.event(), tauri::webview::PageLoadEvent::Finished) && restore_once.swap(false, std::sync::atomic::Ordering::AcqRel) { let _ = view.eval(format!("requestAnimationFrame(() => window.scrollTo(0, {resume_scroll}))")); }
                    let status = match payload.event() {
                        tauri::webview::PageLoadEvent::Started => "loading",
                        tauri::webview::PageLoadEvent::Finished => "ready",
                    };
                    notify_main(&load_app, "load", status);
                });
            window.add_child(builder, tauri::LogicalPosition::new(bounds.x, bounds.y),
                tauri::LogicalSize::new(bounds.width, bounds.height))?
        };
        view.set_bounds(bounds.rect())?;
        view.set_zoom(scale / 100.0)?;
        apply_panel_corner_radius(&view, corner_radius_for_scale(scale));
        let detail = panel_state_detail(
            scale,
            request.theme.as_deref(),
            theme_setting,
            request.toast.as_deref(),
        );
        let mut detail = detail;
        detail["effects"] = serde_json::json!(match request.effects.as_deref() {
            Some("mid") => "mid", Some("low") => "low", _ => "high",
        });
        view.eval(format!("window.__ocxdPanelState = {detail}; window.dispatchEvent(new CustomEvent('ocxd-panel-state', {{detail:{detail}}}))"))?;
        if request.action == PanelAction::Show {
            if request.reload { view.reload()?; }
            view.show()?;
        }
        Ok(PanelResult { visible: true, panel_url: url.map(|url| url.to_string()) })
    }).await.map_err(|_| AppError::NotConfigured)?
}

/// 注入浮层需要的状态：缩放读数与主题。
///
/// 主题只驱动管理器注入浮层的玻璃取值；官方页面自身样式保持不变。
/// 未知或缺失的主题按 `light` 处理，不向浮层暴露任意字符串。
fn panel_state_detail(
    scale: f64,
    theme: Option<&str>,
    theme_setting: &str,
    toast: Option<&str>,
) -> serde_json::Value {
    let theme = if theme == Some("dark") {
        "dark"
    } else {
        "light"
    };
    serde_json::json!({
        "scale": scale,
        "theme": theme,
        "themeSetting": theme_setting,
        "toast": toast.unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corner_radius_follows_interface_scale() {
        assert_eq!(corner_radius_for_scale(100.0), 12.0);
        assert!((corner_radius_for_scale(98.0) - 11.76).abs() < 1e-9);
        assert_eq!(corner_radius_for_scale(150.0), 18.0);
        assert_eq!(corner_radius_for_scale(0.0), 0.0);
    }

    #[test]
    fn panel_state_detail_normalizes_theme_and_keeps_scale() {
        let dark = panel_state_detail(120.0, Some("dark"), "dark", None);
        assert_eq!(dark["theme"], "dark");
        assert_eq!(dark["scale"], 120.0);
        assert_eq!(dark["toast"], "");
        assert_eq!(dark["themeSetting"], "dark");

        for unknown in [Some("neon"), None, Some("")] {
            let light = panel_state_detail(100.0, unknown, "system", Some("提示"));
            assert_eq!(light["theme"], "light");
            assert_eq!(light["toast"], "提示");
            assert_eq!(light["themeSetting"], "system");
        }
    }

    // 回归：官方面板用它自己的 `ocx-theme` 存储键决定明暗；管理器只同步这个键，
    // 不改写官方页面样式。未知主题设置一律按「跟随系统」，不向页面注入任意值。
    #[test]
    fn theme_setting_syncs_through_the_official_storage_key() {
        assert_eq!(normalize_theme_setting(Some("dark")), "dark");
        assert_eq!(normalize_theme_setting(Some("light")), "light");
        assert_eq!(normalize_theme_setting(Some("system")), "system");
        assert_eq!(normalize_theme_setting(Some("neon")), "system");
        assert_eq!(normalize_theme_setting(None), "system");

        let dark = panel_initialization_script("dark");
        assert!(dark.contains("ocx-theme"), "{dark}");
        assert!(
            dark.contains(r#"localStorage.setItem(key, "dark")"#),
            "{dark}"
        );
        // 初始化脚本仍须挂载面板快捷浮层。
        assert!(dark.contains("ocxd-panel-hub"), "{dark}");

        let system = panel_initialization_script("system");
        assert!(system.contains("localStorage.removeItem(key)"), "{system}");
        assert!(system.contains("ocxd-panel-hub"), "{system}");
    }

    #[test]
    fn only_runtime_local_panel_urls_are_accepted() {
        assert!(validate_panel_url("http://127.0.0.1:10100/web").is_ok());
        for url in [
            "http://localhost:10100/web",
            "http://127.0.0.1:10100/other",
            "http://127.0.0.1:10100/web?x=1",
            "https://127.0.0.1:10100/web",
            "http://user:pass@127.0.0.1:10100/web",
            "http://127.0.0.1:0/web",
        ] {
            assert!(validate_panel_url(url).is_err(), "{url}");
        }
    }
    #[test]
    fn navigation_cannot_escape_the_panel_origin() {
        let origin = validate_panel_url("http://127.0.0.1:10100/web").unwrap();
        assert!(navigation_allowed(
            &"http://127.0.0.1:10100/web#codex".parse().unwrap(),
            &origin
        ));
        for url in [
            "http://127.0.0.1:10101/web",
            "https://example.com/web",
            "tauri://localhost",
            "http://127.0.0.1:10100/api",
        ] {
            assert!(!navigation_allowed(&url.parse().unwrap(), &origin));
        }
    }
    #[test]
    fn bounds_never_cover_shell_controls_or_escape_window() {
        let valid = PanelBounds {
            x: 64.,
            y: 28.,
            width: 1115.,
            height: 715.,
        };
        assert!(valid.validate(1180., 760., 1.0).is_ok());
        for bad in [
            PanelBounds { x: 0., ..valid },
            PanelBounds { y: 27., ..valid },
            PanelBounds {
                width: f64::NAN,
                ..valid
            },
            PanelBounds {
                width: 1180.,
                ..valid
            },
            PanelBounds {
                height: -1.,
                ..valid
            },
        ] {
            assert!(bad.validate(1180., 760., 1.0).is_err(), "{bad:?}");
        }
    }

    // 回归：界面缩放 <100% 时侧栏按 `--side-w * zoom` 收缩，面板起点随之左移。
    // 此前按常量 64 判定，98% 缩放下面板会被误拒（表现为「启动后仍打不开面板」）。
    #[test]
    fn bounds_follow_shell_zoom_instead_of_fixed_sidebar_width() {
        let at_98 = PanelBounds {
            x: 62.72,
            y: 28.,
            width: 1117.28,
            height: 732.,
        };
        assert!(at_98.validate(1180., 760., 0.98).is_ok());
        assert!(PanelBounds { x: 62., ..at_98 }
            .validate(1180., 760., 0.98)
            .is_err());

        // 放大同样要跟着走：150% 时侧栏实际占 96 物理像素。
        let at_150 = PanelBounds {
            x: 96.,
            y: 28.,
            width: 900.,
            height: 700.,
        };
        assert!(at_150.validate(1180., 760., 1.5).is_ok());
        assert!(PanelBounds { x: 95., ..at_150 }
            .validate(1180., 760., 1.5)
            .is_err());
    }
    #[test]
    fn idle_reclaim_requires_same_generation_hidden_view_and_elapsed_deadline() {
        let mut idle = PanelIdle {
            hidden: true,
            generation: 3,
            since: Some(std::time::Instant::now()),
            ..Default::default()
        };
        assert!(!idle.accepts(3));
        idle.since = Some(
            std::time::Instant::now() - crate::modules::runtime_defaults::panel_idle_timeout(),
        );
        assert!(idle.accepts(3));
        assert!(!idle.accepts(2));
        idle.hidden = false;
        assert!(!idle.accepts(3));
    }
}
