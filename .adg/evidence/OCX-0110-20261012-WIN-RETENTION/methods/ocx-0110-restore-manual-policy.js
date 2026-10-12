(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const current = await invoke('preferences_backup_policy')
  if (current.mode !== 'manual') await invoke('save_preferences_backup_policy', { policy: { ...current, mode: 'manual' } })
  return { policy: await invoke('preferences_backup_policy'), scope: 'restore test cleanup preference to manual' }
})()
