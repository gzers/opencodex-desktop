(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const preferences = await invoke('get_preferences')
  const backups = await invoke('list_preferences_backups')
  return {
    preferences: {
      keepProxyOnClose: preferences.keepProxyOnClose,
      automaticCheck: preferences.appUpdateAutoCheck,
      channel: preferences.appUpdateChannel,
      theme: preferences.theme,
      themeNeedsImport: preferences.themeNeedsImport,
    },
    originalPin: backups.find(item => item.id === 'bk_20261012044452_a921f503'),
    appearance: await invoke('window_appearance'),
    foreground: await invoke('app_foreground'),
    documentVisibility: document.visibilityState,
    dpr: devicePixelRatio,
    route: location.hash,
    scope: 'read-only production WebView in actual retained administrator-token candidate; token evidence collected separately from real process handle',
  }
})()
