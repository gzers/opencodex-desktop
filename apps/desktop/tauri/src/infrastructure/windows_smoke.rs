//! 最终 Windows EXE 首屏测试：仅显式 smoke 请求可以启用 loopback CDP。

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use tauri::utils::config::{Config, WindowConfig};

const PORT_ENV: &str = "OPENCODEX_WINDOWS_SMOKE_CDP_PORT";
// 显式设置参数会覆盖 wry 的默认参数；保留当前 wry 的默认禁用项。
const DEFAULT_ARGS: &str = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection";

pub(crate) struct SmokeWindow {
    pub config: WindowConfig,
    pub data_directory: PathBuf,
}

#[cfg(windows)]
pub(crate) fn from_environment(config: &mut Config) -> Result<Option<SmokeWindow>, String> {
    let port = std::env::var_os(PORT_ENV);
    let sandbox = crate::modules::test_sandbox::sandbox_root();
    let smoke = configure(
        config,
        port.as_deref(),
        crate::modules::test_sandbox::enabled(),
        sandbox.as_deref(),
    )?;
    apply_scale(
        smoke,
        std::env::var_os("OPENCODEX_WINDOWS_SMOKE_SCALE").as_deref(),
    )
}

fn apply_scale(
    mut smoke: Option<SmokeWindow>,
    scale: Option<&OsStr>,
) -> Result<Option<SmokeWindow>, String> {
    if let Some(scale) = scale {
        let value = scale
            .to_str()
            .filter(|value| ["1", "1.25", "1.5", "2"].contains(value))
            .ok_or_else(|| "smoke scale must be 1, 1.25, 1.5 or 2".to_string())?;
        let window = smoke
            .as_mut()
            .ok_or_else(|| "smoke scale requires the sandbox CDP configuration".to_string())?;
        let args = window
            .config
            .additional_browser_args
            .as_mut()
            .expect("validated smoke arguments");
        args.push_str(&format!(" --force-device-scale-factor={value}"));
    }
    Ok(smoke)
}

// 纯配置入口可在 Mac 回归：不修改共享进程环境，也不启动原生 WebView。
fn configure(
    config: &mut Config,
    port: Option<&OsStr>,
    sandbox_enabled: bool,
    sandbox_root: Option<&Path>,
) -> Result<Option<SmokeWindow>, String> {
    let Some(raw_port) = port else {
        return Ok(None);
    };
    let port = raw_port
        .to_str()
        .filter(|value| !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|port| *port != 0)
        .ok_or_else(|| format!("{PORT_ENV} must be an integer between 1 and 65535"))?;
    let root = sandbox_root
        .filter(|root| sandbox_enabled && root.is_absolute())
        .ok_or_else(|| format!("{PORT_ENV} requires an enabled sandbox with an absolute root"))?;
    let main = config
        .app
        .windows
        .iter_mut()
        .find(|window| window.label == "main")
        .ok_or_else(|| "Windows smoke requires a configured main window".to_string())?;
    let mut window = main.clone();
    let base_args = window
        .additional_browser_args
        .as_deref()
        .unwrap_or(DEFAULT_ARGS);
    window.additional_browser_args = Some(format!(
        "{base_args} --remote-debugging-address=127.0.0.1 --remote-debugging-port={port}"
    ));
    // 在 setup 中用 builder 明确传入绝对数据目录和浏览器参数，
    // 不依赖管理员进程可能忽略的 WebView2 环境变量。
    main.create = false;
    Ok(Some(SmokeWindow {
        config: window,
        data_directory: root.join("EBWebView"),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_dpi_is_bounded_and_requires_smoke() {
        let root = tempfile::tempdir().unwrap();
        for scale in ["1", "1.25", "1.5", "2"] {
            let smoke = configure(
                &mut config(),
                Some(OsStr::new("1234")),
                true,
                Some(root.path()),
            )
            .unwrap();
            assert!(apply_scale(smoke, Some(OsStr::new(scale)))
                .unwrap()
                .unwrap()
                .config
                .additional_browser_args
                .unwrap()
                .contains(&format!("--force-device-scale-factor={scale}")));
        }
        assert!(apply_scale(None, Some(OsStr::new("1"))).is_err());
        assert!(apply_scale(None, Some(OsStr::new("1 --disable-gpu"))).is_err());
    }

    fn config() -> Config {
        let mut config = Config::default();
        config.app.windows = vec![WindowConfig {
            label: "main".into(),
            ..Default::default()
        }];
        config
    }

    #[test]
    fn sandbox_without_smoke_port_keeps_normal_window() {
        let mut config = config();
        assert!(configure(&mut config, None, true, None).unwrap().is_none());
        assert!(config.app.windows[0].create);
        assert!(config.app.windows[0].additional_browser_args.is_none());
        assert!(config.app.windows[0].data_directory.is_none());
    }

    #[test]
    fn smoke_requires_valid_port_and_isolated_root() {
        let root = tempfile::tempdir().unwrap();
        for port in ["", "0", "65536", "-1", " 1234", "1234 --foo"] {
            assert!(configure(
                &mut config(),
                Some(OsStr::new(port)),
                true,
                Some(root.path())
            )
            .is_err());
        }
        for (enabled, root) in [
            (false, Some(root.path())),
            (true, None),
            (true, Some(Path::new("relative"))),
        ] {
            assert!(configure(&mut config(), Some(OsStr::new("1234")), enabled, root).is_err());
        }
    }

    #[test]
    fn smoke_configures_only_main_before_automatic_creation() {
        let root = tempfile::tempdir().unwrap();
        let mut config = config();
        config.app.windows.push(WindowConfig {
            label: "other".into(),
            ..Default::default()
        });
        let smoke = configure(
            &mut config,
            Some(OsStr::new("65535")),
            true,
            Some(root.path()),
        )
        .unwrap()
        .unwrap();
        assert!(!config.app.windows[0].create);
        assert!(config.app.windows[1].create);
        assert!(config.app.windows[1].additional_browser_args.is_none());
        assert!(smoke.config.create);
        assert_eq!(smoke.data_directory, root.path().join("EBWebView"));
        assert_eq!(
            smoke.config.additional_browser_args.unwrap(),
            format!(
                "{DEFAULT_ARGS} --remote-debugging-address=127.0.0.1 --remote-debugging-port=65535"
            )
        );
    }

    #[test]
    fn preserves_existing_browser_options_and_rejects_missing_main() {
        let root = tempfile::tempdir().unwrap();
        let mut config = config();
        config.app.windows[0].additional_browser_args = Some("--disable-gpu".into());
        let smoke = configure(&mut config, Some(OsStr::new("1")), true, Some(root.path()))
            .unwrap()
            .unwrap();
        assert!(smoke
            .config
            .additional_browser_args
            .unwrap()
            .starts_with("--disable-gpu "));
        config.app.windows.clear();
        assert!(configure(&mut config, Some(OsStr::new("1")), true, Some(root.path())).is_err());
    }
}
