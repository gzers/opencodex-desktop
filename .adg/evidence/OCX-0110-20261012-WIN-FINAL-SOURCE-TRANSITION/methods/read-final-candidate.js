(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const pinia = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia
  const store = pinia?._s.get('preferences')
  if (!store) throw Error('Final candidate preferences store unavailable')
  const before = await invoke('get_preferences')
  if (before.themeNeedsImport || before.appUpdateAutoCheck) throw Error('Unexpected import or auto-check state')
  await store.load()
  const after = await invoke('get_preferences')
  const backups = await invoke('list_preferences_backups')
  const notifications = await invoke('list_notifications')
  return {
    about: await invoke('app_about'),
    roots: await invoke('get_data_root_config'),
    preferenceReadbackMatches: JSON.stringify(before) === JSON.stringify(after),
    storeError: store.error,
    theme: after.theme,
    themeNeedsImport: after.themeNeedsImport,
    automaticCheck: after.appUpdateAutoCheck,
    channel: after.appUpdateChannel,
    originalPin: backups.find(item => item.id === 'bk_20261012044452_a921f503'),
    notifications: notifications.items.map(({ id, resolved, resolvedAt }) => ({ id, resolved, resolvedAt })),
    appearance: await invoke('window_appearance'),
    foreground: await invoke('app_foreground'),
    dpr: devicePixelRatio,
    route: location.hash,
    scope: 'actual final main WebView/native readback and production preference reload; no update check or installation',
  }
})()
