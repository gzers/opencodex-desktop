(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const context = window.__ocxEaeRestoreEventContext
  if (!context?.id) throw Error('Prepared final-source restore candidate absent')
  const result = await invoke('restore_preferences_backup', { id: context.id })
  const preferences = await invoke('get_preferences')
  const inventory = await invoke('list_preferences_backups')
  const notifications = await invoke('list_notifications')
  const store = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia?._s.get('preferences')
  await store?.load()
  return {
    backupId: context.id, actualRestore: result,
    notifications: notifications.items.map(({ id, resolved, resolvedAt }) => ({ id, resolved, resolvedAt })),
    protection: inventory.find(item => item.id === result.protectionBackupId),
    originalUserPin: inventory.find(item => item.id === 'bk_20261012044452_a921f503'),
    preferences: { channel: preferences.appUpdateChannel, automaticCheck: preferences.appUpdateAutoCheck, theme: preferences.theme },
    storeError: store?.error, storeTheme: store?.data?.theme,
    invokeBridgeUnchanged: invoke === window.__TAURI_INTERNALS__.invoke,
    scope: 'actual matching final-source restore after repair of original fixture bytes; no event or notification mutation command',
  }
})()
