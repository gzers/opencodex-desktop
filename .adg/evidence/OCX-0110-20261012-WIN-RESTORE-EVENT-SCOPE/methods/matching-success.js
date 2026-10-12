(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const context = window.__ocxRestoreEventScope
  if (!context?.a || !context?.b) throw Error('Missing prepared backup IDs')
  const matchingRestore = await invoke('restore_preferences_backup', { id: context.a })
  const notifications = await invoke('list_notifications')
  const backups = await invoke('list_preferences_backups')
  const policy = await invoke('preferences_backup_policy')
  const preferences = await invoke('get_preferences')
  return { ...context, matchingRestore, notifications, backups, policy,
    preferences: { channel: preferences.appUpdateChannel, automaticCheck: preferences.appUpdateAutoCheck, theme: preferences.theme },
    nativeInvokeUnchanged: invoke === window.__TAURI_INTERNALS__.invoke,
    scope: 'actual matching restore after restoring fixture payload bytes; no manual notification resolution' }
})()
