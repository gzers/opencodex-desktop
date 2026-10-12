(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  if (window.__ocxEaeProtectionScene) throw Error('Scene must not be restarted')
  const preferences = await invoke('get_preferences')
  const policy = await invoke('preferences_backup_policy')
  if (policy.mode !== 'manual' || preferences.theme !== 'system'
      || preferences.appUpdateChannel !== 'stable' || !preferences.appUpdateAutoCheck) throw Error('Cycle or policy precondition differs')
  const inventory = await invoke('list_preferences_backups')
  const notifications = (await invoke('list_notifications')).items.map(({ id, resolved, resolvedAt }) => ({ id, resolved, resolvedAt }))
  if ((await invoke('preview_preferences_backup_cleanup')).candidateIds.length) throw Error('Preexisting candidates cannot be touched')
  const scene = { originalPreferences: preferences, inventory, notifications, created: [] }
  window.__ocxEaeProtectionScene = scene
  for (let index = 0; index < 2; index++) {
    const result = await invoke('create_preferences_backup')
    scene.created.push(result)
    if (result.cleanupError || result.removedIds.length) throw Error('Creation pruned preexisting records')
  }
  return { inventory, notifications, policy, created: scene.created, currentPreferences: preferences,
    scope: 'actual final-desktop native creation, no UI input, no clock or schedule mutation' }
})()
