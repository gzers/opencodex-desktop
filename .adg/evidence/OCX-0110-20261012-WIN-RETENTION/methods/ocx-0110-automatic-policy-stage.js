(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const originalPolicy = await invoke('preferences_backup_policy')
  const before = await invoke('list_preferences_backups')
  if (before.filter(item => item.action === 'manual-preferences').length !== 14) throw Error('Unexpected automatic-test population')
  await invoke('save_preferences_backup_policy', { policy: { ...originalPolicy, mode: 'automatic' } })
  const after = await invoke('list_preferences_backups')
  if (JSON.stringify(before) !== JSON.stringify(after)) throw Error('Saving policy immediately changed backups')
  return { originalPolicy, automaticPolicy: await invoke('preferences_backup_policy'), before, after,
    policySaveDidNotDelete: true, scope: 'real native opt-in policy save; no automatic cleanup before new backup success' }
})()
