//! Windows keeps the system caption/buttons/hit testing, with one caption row.
//! DWM owns Mica; the WebView never imitates it with CSS blur. Other platforms
//! retain their existing window and application-menu path.
use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Appearance {
    pub platform: &'static str,
    pub material: &'static str,
    pub reason: &'static str,
    pub theme: &'static str,
    pub build: u32,
    pub transparency: bool,
    pub high_contrast: bool,
    pub composition: bool,
    pub active: bool,
    pub native_caption: bool,
    pub backdrop_attribute: Option<i32>,
    pub dwm_result: Option<i32>,
}

/// Native material availability does not depend on the user's animation tier.
#[cfg(any(windows, test))]
fn material_reason(
    build: u32,
    transparency: bool,
    contrast: bool,
    composition: bool,
    active: bool,
) -> &'static str {
    if build < 22000 {
        "windows-10"
    } else if !transparency {
        "transparency-disabled"
    } else if contrast {
        "high-contrast"
    } else if !composition {
        "composition-unavailable"
    } else if !active {
        "inactive"
    } else {
        "mica"
    }
}

pub fn install(app: &tauri::AppHandle) {
    #[cfg(windows)]
    native::install(app);
    #[cfg(not(windows))]
    let _ = app;
}

pub fn refresh(app: &tauri::AppHandle) {
    #[cfg(windows)]
    native::refresh(app);
    #[cfg(not(windows))]
    let _ = app;
}

pub fn snapshot(app: &tauri::AppHandle) -> Option<Appearance> {
    #[cfg(windows)]
    {
        native::snapshot(app)
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        None
    }
}

#[cfg(windows)]
mod native {
    use super::*;
    use std::sync::{Mutex, OnceLock};
    use tauri::Manager;
    use windows_sys::Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        Graphics::Dwm::*,
        System::Registry::{RegGetValueW, HKEY_CURRENT_USER, RRF_RT_REG_DWORD},
        UI::{
            Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW},
            Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
            WindowsAndMessaging::*,
        },
    };

    static APP: OnceLock<tauri::AppHandle> = OnceLock::new();
    const SUBCLASS: usize = 0x4f435841;
    #[derive(Default)]
    pub struct State(Mutex<Option<Appearance>>);

    #[repr(C)]
    struct OsVersion {
        size: u32,
        major: u32,
        minor: u32,
        build: u32,
        platform: u32,
        service: [u16; 128],
    }
    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn RtlGetVersion(info: *mut OsVersion) -> i32;
    }

    fn build() -> u32 {
        let mut info = OsVersion {
            size: std::mem::size_of::<OsVersion>() as u32,
            major: 0,
            minor: 0,
            build: 0,
            platform: 0,
            service: [0; 128],
        };
        if unsafe { RtlGetVersion(&mut info) } >= 0 {
            info.build
        } else {
            0
        }
    }
    fn setting(name: &str, default: u32) -> u32 {
        let path: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize\0"
            .encode_utf16()
            .collect();
        let name: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();
        let mut value = default;
        let mut size = 4;
        let result = unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                path.as_ptr(),
                name.as_ptr(),
                RRF_RT_REG_DWORD,
                std::ptr::null_mut(),
                (&mut value as *mut u32).cast(),
                &mut size,
            )
        };
        if result == 0 {
            value
        } else {
            default
        }
    }
    fn attribute(hwnd: HWND, name: u32, value: i32) -> i32 {
        unsafe { DwmSetWindowAttribute(hwnd, name, (&value as *const i32).cast(), 4) }
    }
    fn read_attribute(hwnd: HWND, name: u32) -> Option<i32> {
        let mut value = 0;
        let result =
            unsafe { DwmGetWindowAttribute(hwnd, name, (&mut value as *mut i32).cast(), 4) };
        (result >= 0).then_some(value)
    }
    pub(super) fn snapshot(app: &tauri::AppHandle) -> Option<Appearance> {
        app.try_state::<State>()?.0.lock().ok()?.clone()
    }
    pub(super) fn refresh(app: &tauri::AppHandle) {
        let app = app.clone();
        // DWM and the native caption are updated on the window's UI thread.
        let queued = app.clone();
        let _ = app.run_on_main_thread(move || apply(&queued));
    }
    fn apply(app: &tauri::AppHandle) {
        let Some(window) = app.get_webview_window("main") else {
            return;
        };
        let Ok(handle) = window.hwnd() else { return };
        let hwnd = handle.0 as HWND;
        let preferences = app
            .try_state::<crate::state::SharedDataRoot>()
            .and_then(|root| {
                crate::modules::preferences::PreferencesStore::new(&root.0)
                    .load()
                    .ok()
            });
        let setting_theme = preferences
            .as_ref()
            .map_or("system", |prefs| prefs.theme.as_str());
        let dark = match setting_theme {
            "dark" => true,
            "light" => false,
            _ => setting("AppsUseLightTheme", 1) == 0,
        };
        let transparency = setting("EnableTransparency", 1) != 0;
        let mut contrast = HIGHCONTRASTW {
            cbSize: std::mem::size_of::<HIGHCONTRASTW>() as u32,
            ..Default::default()
        };
        let mut composed = 0;
        let high_contrast = unsafe {
            SystemParametersInfoW(
                SPI_GETHIGHCONTRAST,
                contrast.cbSize,
                (&mut contrast as *mut HIGHCONTRASTW).cast(),
                0,
            )
        } != 0
            && contrast.dwFlags & HCF_HIGHCONTRASTON != 0;
        let composition = unsafe { DwmIsCompositionEnabled(&mut composed) } >= 0 && composed != 0;
        let active = unsafe {
            GetAncestor(GetForegroundWindow(), GA_ROOT) == hwnd
                && IsWindowVisible(hwnd) != 0
                && IsIconic(hwnd) == 0
        };
        let build = build();
        let mut state = Appearance {
            platform: "Windows",
            material: "solid",
            reason: material_reason(build, transparency, high_contrast, composition, active),
            theme: if dark { "dark" } else { "light" },
            build,
            transparency,
            high_contrast,
            composition,
            active,
            native_caption: true,
            backdrop_attribute: None,
            dwm_result: None,
        };
        // Avoid redundant compositor writes while still allowing policy/theme events.
        if snapshot(app).as_ref().is_some_and(|previous| {
            previous.theme == state.theme
                && previous.reason == state.reason
                && previous.transparency == transparency
                && previous.high_contrast == high_contrast
                && previous.composition == composition
                && previous.active == active
        }) {
            return;
        }
        let _ = window.set_theme(Some(if dark {
            tauri::Theme::Dark
        } else {
            tauri::Theme::Light
        }));
        let _ = attribute(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE as u32, i32::from(dark));
        let backdrop = if state.reason == "mica" {
            DWMSBT_MAINWINDOW
        } else {
            DWMSBT_NONE
        };
        if build >= 22621 {
            let result = attribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE as u32, backdrop);
            state.dwm_result = Some(result);
            state.backdrop_attribute = read_attribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE as u32);
            if state.reason == "mica"
                && result >= 0
                && state.backdrop_attribute == Some(DWMSBT_MAINWINDOW)
            {
                state.material = "mica";
            } else if state.reason == "mica" {
                state.reason = "dwm-unavailable";
            }
        } else if build >= 22000 {
            // Same bounded compatibility attribute used by Tauri's window-vibrancy
            // on initial Win11. It is not attempted on Win10.
            let result = attribute(hwnd, 1029, i32::from(state.reason == "mica"));
            state.dwm_result = Some(result);
            if state.reason == "mica" && result >= 0 {
                state.material = "mica";
            } else if state.reason == "mica" {
                state.reason = "dwm-unavailable";
            }
        }
        if state.material != "mica" && build >= 22621 {
            let _ = attribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE as u32, DWMSBT_NONE);
            state.backdrop_attribute = read_attribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE as u32);
        }
        let color = if dark { 0x002c2420 } else { 0x00f5ede8 }; // COLORREF BGR
                                                                // A user accent-colored system caption would form a separate blue strip.
                                                                // Keep the native caption palette continuous with the product shell;
                                                                // the client Mica remains the real DWM backdrop, not a CSS tint imitation.
        let _ = attribute(hwnd, DWMWA_CAPTION_COLOR as u32, color);
        let margins = windows_sys::Win32::UI::Controls::MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyTopHeight: -1,
            cyBottomHeight: -1,
        };
        if state.material == "mica" {
            let extended = unsafe { DwmExtendFrameIntoClientArea(hwnd, &margins) };
            if extended < 0 {
                state.material = "solid";
                state.reason = "frame-extension-failed";
                state.dwm_result = Some(extended);
                let _ = attribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE as u32, DWMSBT_NONE);
            }
        } else {
            let margins = windows_sys::Win32::UI::Controls::MARGINS::default();
            unsafe { DwmExtendFrameIntoClientArea(hwnd, &margins) };
        }
        let color = if state.material == "mica" {
            tauri::utils::config::Color(0, 0, 0, 0)
        } else if dark {
            tauri::utils::config::Color(32, 36, 44, 255)
        } else {
            tauri::utils::config::Color(232, 237, 245, 255)
        };
        if let Err(error) = window.set_background_color(Some(color)) {
            eprintln!("Native window background failed: {error}");
            state.material = "solid";
            state.reason = "webview-background-unavailable";
            let _ = attribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE as u32, DWMSBT_NONE);
            state.backdrop_attribute = read_attribute(hwnd, DWMWA_SYSTEMBACKDROP_TYPE as u32);
            let fallback = if dark {
                tauri::utils::config::Color(32, 36, 44, 255)
            } else {
                tauri::utils::config::Color(232, 237, 245, 255)
            };
            let _ = window.set_background_color(Some(fallback));
        }
        if let Some(store) = app.try_state::<State>() {
            if let Ok(mut stored) = store.0.lock() {
                *stored = Some(state.clone());
            }
        }
        let _ = crate::commands::event_delivery::emit_signal_to(
            app,
            "main",
            "native-window-appearance",
            crate::modules::notifications::registry::Job::Appearance,
            crate::modules::notifications::registry::Trigger::NativeCallback,
            crate::modules::notifications::registry::Channel::Local,
            &state,
        );
        if let Some(root) = app.try_state::<crate::state::SharedDataRoot>() {
            let _ = crate::infrastructure::runtime_log::RuntimeLog::new(&root.0).append_event(
                &format!("window appearance: theme={} material={} reason={} build={} active={} transparency={} dwm={:?}",
                    state.theme, state.material, state.reason, state.build, state.active, state.transparency, state.dwm_result));
        }
    }
    unsafe extern "system" fn message(
        hwnd: HWND,
        msg: u32,
        wp: WPARAM,
        lp: LPARAM,
        id: usize,
        _: usize,
    ) -> LRESULT {
        let result = unsafe { DefSubclassProc(hwnd, msg, wp, lp) };
        if msg == WM_NCDESTROY {
            unsafe { RemoveWindowSubclass(hwnd, Some(message), id) };
        } else if matches!(
            msg,
            WM_SETTINGCHANGE | WM_THEMECHANGED | WM_DWMCOMPOSITIONCHANGED | WM_ACTIVATE | WM_SIZE
        ) {
            if let Some(app) = APP.get() {
                refresh(app);
            }
        }
        result
    }
    pub(super) fn install(app: &tauri::AppHandle) {
        app.manage(State::default());
        let Some(window) = app.get_window("main") else {
            return;
        };
        let _ = window.set_title("OpenCodeX Desktop");
        let Ok(handle) = window.hwnd() else { return };
        if APP.set(app.clone()).is_ok()
            && unsafe { SetWindowSubclass(handle.0 as HWND, Some(message), SUBCLASS, 0) } == 0
        {
            eprintln!("Native appearance setting-change listener unavailable");
        }
        apply(app);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_policy_fallback_is_independent_of_animation_quality() {
        assert_eq!(
            material_reason(19045, true, false, true, true),
            "windows-10"
        );
        assert_eq!(
            material_reason(26100, false, false, true, true),
            "transparency-disabled"
        );
        assert_eq!(
            material_reason(26100, true, true, true, true),
            "high-contrast"
        );
        assert_eq!(
            material_reason(26100, true, false, false, true),
            "composition-unavailable"
        );
        assert_eq!(material_reason(26100, true, false, true, false), "inactive");
        assert_eq!(material_reason(26100, true, false, true, true), "mica");
    }
}
