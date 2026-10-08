//! Read-only native-window diagnostics; only the local main WebView may read it.
#[tauri::command]
pub fn window_appearance(
    app: tauri::AppHandle,
    webview: tauri::Webview,
) -> Result<Option<crate::infrastructure::window_appearance::Appearance>, String> {
    if webview.label() != "main" {
        return Err("native window is available only to the main view".into());
    }
    Ok(crate::infrastructure::window_appearance::snapshot(&app))
}
