(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const preferences = await invoke('get_preferences')
  const backups = await invoke('list_preferences_backups')
  const notifications = await invoke('list_notifications')
  return {
    preferences: {
      keepProxyOnClose: preferences.keepProxyOnClose,
      automaticCheck: preferences.appUpdateAutoCheck,
      channel: preferences.appUpdateChannel,
      theme: preferences.theme,
      themeNeedsImport: preferences.themeNeedsImport,
    },
    originalPin: backups.find(item => item.id === 'bk_20261012044452_a921f503'),
    notifications: notifications.items.map(item => ({ id: item.id, resolved: item.resolved, resolvedAt: item.resolvedAt })),
    foreground: await invoke('app_foreground'),
    appearance: await invoke('window_appearance'),
    route: location.hash,
    scope: 'new controlled ordinary session; not a same-HWND restore or a normal-exit acceptance sample',
  }
})()
