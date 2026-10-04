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
        let visible = app.get_window("main").is_some_and(|window| {
            window.is_visible().unwrap_or(false) && !window.is_minimized().unwrap_or(true)
        });
        let foreground = active && visible;
        if self.foreground.swap(foreground, Ordering::AcqRel) != foreground {
            let _ = app.emit("app-foreground-changed", foreground);
            self.wake.notify_one();
        }
    }
}
#[tauri::command]
pub fn app_foreground(app: tauri::AppHandle, webview: tauri::Webview) -> bool {
    webview.label() == "main" && app.state::<AppActivity>().foreground()
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
#[cfg(not(target_os = "macos"))]
pub fn install(app: &tauri::AppHandle) {
    let active = app
        .get_window("main")
        .is_some_and(|window| window.is_focused().unwrap_or(false));
    app.state::<AppActivity>().publish(app, active);
}
