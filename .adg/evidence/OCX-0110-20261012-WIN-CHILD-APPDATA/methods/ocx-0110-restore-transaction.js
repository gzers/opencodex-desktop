(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const backups = await invoke('list_preferences_backups')
  const candidate = backups.find(backup => backup.id === 'bk_20261012044452_a921f503')
  if (!candidate?.preferencesCandidate) throw Error('Captured backup is absent')
  const before = await invoke('get_preferences')
  if (before.theme !== 'light') throw Error('Different-value restore precondition not met')
  await invoke('set_preferences_backup_pinned', { id: candidate.id, pinned: true })
  const pinned = (await invoke('list_preferences_backups')).find(backup => backup.id === candidate.id)
  if (!pinned?.pinned) throw Error('Pin did not persist')
  const restored = await invoke('restore_preferences_backup', { id: candidate.id })
  const after = await invoke('get_preferences')
  const finalBackups = await invoke('list_preferences_backups')
  const pinAfterRestore = finalBackups.find(backup => backup.id === candidate.id)
  const protection = finalBackups.find(backup => backup.id === restored.protectionBackupId)
  if (after.theme !== 'system') throw Error('Captured original preference not restored')
  if (!pinAfterRestore?.pinned) throw Error('Restore cleared user pin')
  if (protection?.protected) throw Error('Completed restore still holds temporary protection')
  return {
    beforeTheme: before.theme, afterTheme: after.theme,
    pinnedBeforeRestore: pinned.pinned, pinnedAfterRestore: pinAfterRestore.pinned,
    restored, finalBackups, protection, notifications: await invoke('list_notifications'),
    scope: 'real native backup commands in the sole isolated application; UI confirmation pending',
  }
})()
