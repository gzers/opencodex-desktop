(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const policy = await invoke('preferences_backup_policy')
  if (policy.mode !== 'manual') throw Error('Preserve existing policy; manual expected')
  const preferences = await invoke('get_preferences')
  if (preferences.appUpdateChannel !== 'stable' || preferences.appUpdateAutoCheck) throw Error('Unexpected update state')
  const before = await invoke('list_notifications')
  const a = await invoke('create_preferences_backup')
  const b = await invoke('create_preferences_backup')
  if (a.cleanupError || b.cleanupError || a.removedIds.length || b.removedIds.length) throw Error('Unexpected backup cleanup')
  window.__ocxRestoreEventScope = { a: a.backup.backupId, b: b.backup.backupId }
  return { ...window.__ocxRestoreEventScope, policy, before, created: [a, b], source: 'two actual native backups of current canonical isolated preferences' }
})()
