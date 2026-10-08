//! One native foreground signal for decorative rendering and status scheduling.
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Emitter, Manager};

#[derive(Default)]
pub struct AppActivity {
    foreground: AtomicBool,
    pub wake: tokio::sync::Notify,
}
impl AppActivity {
    pub fn foreground(&self) -> bool {
        self.foreground.load(Ordering::Acquire)
    }
    pub fn publish(&self, app: &tauri::AppHandle, active: bool) {
        // Windows keyboard focus can remain in a WebView child, or be absent
        // after restore. App activation must follow the foreground root HWND.
        #[cfg(windows)]
        let foreground = {
            let _ = active;
            windows_activity::foreground(app)
        };
        #[cfg(not(windows))]
        let visible = app.get_window("main").is_some_and(|window| {
            window.is_visible().unwrap_or(false) && !window.is_minimized().unwrap_or(true)
        });
        #[cfg(not(windows))]
        let foreground = active && visible;
        if self.foreground.swap(foreground, Ordering::AcqRel) != foreground {
            let _ = app.emit("app-foreground-changed", foreground);
            self.wake.notify_one();
        }
    }
}
#[tauri::command]
pub fn app_foreground(app: tauri::AppHandle, webview: tauri::Webview) -> bool {
    #[cfg(windows)]
    app.state::<AppActivity>().publish(&app, true);
    webview.label() == "main" && app.state::<AppActivity>().foreground()
}

#[cfg(windows)]
mod windows_activity {
    use super::*;
    use std::sync::OnceLock;
    use windows_sys::Win32::{
        Foundation::HWND,
        UI::{
            Accessibility::{SetWinEventHook, HWINEVENTHOOK},
            WindowsAndMessaging::{
                GetAncestor, GetForegroundWindow, IsIconic, IsWindowVisible, EVENT_OBJECT_HIDE,
                EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_MINIMIZEEND,
                EVENT_SYSTEM_MINIMIZESTART, GA_ROOT, OBJID_WINDOW, WINEVENT_OUTOFCONTEXT,
            },
        },
    };
    struct Context {
        app: tauri::AppHandle,
        window: usize,
    }
    static CONTEXT: OnceLock<Context> = OnceLock::new();

    pub(super) fn foreground(app: &tauri::AppHandle) -> bool {
        let Some(window) = app.get_window("main").and_then(|window| window.hwnd().ok()) else {
            return false;
        };
        let hwnd = window.0 as HWND;
        // These are read-only User32 queries, safe from event/command threads.
        unsafe {
            super::windows_foreground(
                IsWindowVisible(hwnd) != 0,
                IsIconic(hwnd) != 0,
                GetAncestor(GetForegroundWindow(), GA_ROOT) == hwnd,
            )
        }
    }
    unsafe extern "system" fn changed(
        _: HWINEVENTHOOK,
        event: u32,
        window: HWND,
        object: i32,
        _: i32,
        _: u32,
        _: u32,
    ) {
        if let Some(context) = CONTEXT.get() {
            if event == EVENT_SYSTEM_FOREGROUND
                || (window as usize == context.window && object == OBJID_WINDOW)
            {
                context
                    .app
                    .state::<AppActivity>()
                    .publish(&context.app, true);
            }
        }
    }
    pub(super) fn install(app: &tauri::AppHandle) {
        let Some(window) = app.get_window("main").and_then(|window| window.hwnd().ok()) else {
            return;
        };
        if CONTEXT
            .set(Context {
                app: app.clone(),
                window: window.0 as usize,
            })
            .is_err()
        {
            return;
        }
        // Setup is on the message-loop thread. Out-of-context hooks do not inject
        // into other processes; three fixed hooks live until process exit.
        for (first, last, process) in [
            (EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND, 0),
            (
                EVENT_SYSTEM_MINIMIZESTART,
                EVENT_SYSTEM_MINIMIZEEND,
                std::process::id(),
            ),
            (EVENT_OBJECT_SHOW, EVENT_OBJECT_HIDE, std::process::id()),
        ] {
            let hook = unsafe {
                SetWinEventHook(
                    first,
                    last,
                    std::ptr::null_mut(),
                    Some(changed),
                    process,
                    0,
                    WINEVENT_OUTOFCONTEXT,
                )
            };
            if hook.is_null() {
                eprintln!("Windows foreground hook installation failed");
            }
        }
        app.state::<AppActivity>().publish(app, true);
    }
}
#[cfg(any(windows, test))]
fn windows_foreground(visible: bool, minimized: bool, matching_root: bool) -> bool {
    visible && !minimized && matching_root
}
#[cfg(windows)]
pub fn install(app: &tauri::AppHandle) {
    windows_activity::install(app);
}

#[cfg(target_os = "macos")]
pub fn install(app: &tauri::AppHandle) {
    use objc2::runtime::{AnyClass, AnyObject, ClassBuilder, NSObject, Sel};
    use objc2::{msg_send, sel, ClassType};
    use std::sync::OnceLock;
    static APP: OnceLock<tauri::AppHandle> = OnceLock::new();
    extern "C" fn activated(_: *mut AnyObject, _: Sel, _: *mut AnyObject) {
        if let Some(app) = APP.get() {
            app.state::<AppActivity>().publish(app, true);
        }
    }
    extern "C" fn deactivated(_: *mut AnyObject, _: Sel, _: *mut AnyObject) {
        if let Some(app) = APP.get() {
            app.state::<AppActivity>().publish(app, false);
        }
    }
    if APP.set(app.clone()).is_err() {
        return;
    }
    // Setup runs on the AppKit thread. The observer is retained for the process
    // lifetime (one NSObject), and notifications execute on the posting AppKit thread.
    unsafe {
        let mut builder =
            ClassBuilder::new(c"OCXDActivityObserver", NSObject::class()).expect("activity class");
        builder.add_method(
            sel!(activated:),
            activated as extern "C" fn(*mut AnyObject, Sel, *mut AnyObject),
        );
        builder.add_method(
            sel!(deactivated:),
            deactivated as extern "C" fn(*mut AnyObject, Sel, *mut AnyObject),
        );
        let class = builder.register();
        let observer: *mut AnyObject = msg_send![class, new];
        let center_class = AnyClass::get(c"NSNotificationCenter").expect("notification center");
        let center: *mut AnyObject = msg_send![center_class, defaultCenter];
        let string_class = AnyClass::get(c"NSString").expect("string class");
        let active: *mut AnyObject = msg_send![string_class, stringWithUTF8String: c"NSApplicationDidBecomeActiveNotification".as_ptr()];
        let inactive: *mut AnyObject = msg_send![string_class, stringWithUTF8String: c"NSApplicationDidResignActiveNotification".as_ptr()];
        let nil = std::ptr::null::<AnyObject>();
        let _: () = msg_send![center, addObserver: observer, selector: sel!(activated:), name: active, object: nil];
        let _: () = msg_send![center, addObserver: observer, selector: sel!(deactivated:), name: inactive, object: nil];
        let app_class = AnyClass::get(c"NSApplication").expect("application class");
        let native: *mut AnyObject = msg_send![app_class, sharedApplication];
        let active: bool = msg_send![native, isActive];
        app.state::<AppActivity>().publish(app, active);
    }
}
#[cfg(not(any(target_os = "macos", windows)))]
pub fn install(app: &tauri::AppHandle) {
    let active = app
        .get_window("main")
        .is_some_and(|window| window.is_focused().unwrap_or(false));
    app.state::<AppActivity>().publish(app, active);
}

#[cfg(test)]
mod tests {
    use super::windows_foreground;

    #[test]
    fn windows_activation_tracks_root_visibility_without_keyboard_focus() {
        assert!(windows_foreground(true, false, true));
        assert!(!windows_foreground(true, true, true));
        assert!(!windows_foreground(false, false, true));
        assert!(!windows_foreground(true, false, false));
    }
}
