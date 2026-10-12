(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  if (window.__ocxEaePolicyFailureScene) throw Error('Existing scene must not be restarted')
  const preferences = await invoke('get_preferences')
  const policy = await invoke('preferences_backup_policy')
  if (policy.mode !== 'manual' || policy.keep_recent !== 10 || policy.keep_days !== 30
      || preferences.theme !== 'system' || preferences.appUpdateChannel !== 'stable' || !preferences.appUpdateAutoCheck) throw Error('Policy/cycle precondition differs')
  const inventory = await invoke('list_preferences_backups')
  const notifications = (await invoke('list_notifications')).items.map(({ id, resolved, resolvedAt }) => ({ id, resolved, resolvedAt }))
  if ((await invoke('preview_preferences_backup_cleanup')).candidateIds.length) throw Error('Preexisting cleanup candidates')
  const scene = { preferences, policy, inventory, notifications, created: [] }
  window.__ocxEaePolicyFailureScene = scene
  for (let index = 0; index < 3; index++) {
    const result = await invoke('create_preferences_backup')
    scene.created.push(result)
    if (result.cleanupError || result.removedIds.length) throw Error('Creation pruned inventory')
  }
  return { preferences, policy, inventory, notifications, created: scene.created,
    after: await invoke('list_preferences_backups'), preview: await invoke('preview_preferences_backup_cleanup'),
    scope: 'real new owned native backups only; no source preferences or schedule mutation' }
})()
