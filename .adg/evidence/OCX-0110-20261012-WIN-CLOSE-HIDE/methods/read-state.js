(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const preferences = await invoke('get_preferences')
  return {
    keepProxyOnClose: preferences.keepProxyOnClose,
    automaticCheck: preferences.appUpdateAutoCheck,
    channel: preferences.appUpdateChannel,
    foreground: await invoke('app_foreground'),
    appearance: await invoke('window_appearance'),
    documentVisibility: document.visibilityState,
    route: location.hash,
    tray: await invoke('tray_state'),
    scope: 'read-only native and production WebView state; no window mutation',
  }
})()
