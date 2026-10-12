(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const preferences = await invoke('get_preferences')
  const policy = await invoke('preferences_backup_policy')
  if (preferences.theme !== 'system' || preferences.appUpdateChannel !== 'stable'
      || preferences.appUpdateAutoCheck !== true || policy.mode !== 'manual') {
    throw Error('Expected current final-source cycle state')
  }
  const notifications = await invoke('list_notifications')
  const created = await invoke('create_preferences_backup')
  if (created.cleanupError || created.removedIds.length) throw Error('Unexpected pruning during isolated preparation')
  window.__ocxEaeRestoreEventContext = Object.freeze({ id: created.backup.backupId })
  return {
    backupId: created.backup.backupId,
    created,
    beforeNotifications: notifications.items.map(({ id, resolved, resolvedAt }) => ({ id, resolved, resolvedAt })),
    preferences: { channel: preferences.appUpdateChannel, automaticCheck: preferences.appUpdateAutoCheck, theme: preferences.theme },
    scope: 'new real native backup of current final-source cycle preferences; no notification mutation',
  }
})()
