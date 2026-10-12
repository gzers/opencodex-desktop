(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  if (window.__ocxEaeN10PinScene) throw Error('Existing N10 scene must not be restarted')
  const preferences = await invoke('get_preferences')
  const policy = await invoke('preferences_backup_policy')
  if (policy.mode !== 'manual' || policy.keep_recent !== 10 || policy.keep_days !== 30
      || preferences.theme !== 'system' || preferences.appUpdateChannel !== 'stable' || !preferences.appUpdateAutoCheck) throw Error('Policy/cycle precondition differs')
  const before = await invoke('list_preferences_backups')
  const originalPin = before.find(item => item.id === 'bk_20261012044452_a921f503')
  if (!originalPin?.pinned || (await invoke('preview_preferences_backup_cleanup')).candidateIds.length) throw Error('Original pin or clean preview precondition differs')
  const ordinaryOriginal = before.filter(item => item.action === 'manual-preferences' && !item.pinned && !item.protected)
  if (ordinaryOriginal.length > 40) throw Error('Original temporary-pin population outside bounded scene')
  const scene = { originalPreferences: preferences, policy, before, ordinaryOriginal, temporaryOriginalPins: [], created: [], newestPinned: false }
  window.__ocxEaeN10PinScene = scene
  for (const item of ordinaryOriginal) {
    await invoke('set_preferences_backup_pinned', { id: item.id, pinned: true })
    scene.temporaryOriginalPins.push(item.id)
  }
  const afterOriginalPins = await invoke('list_preferences_backups')
  if (afterOriginalPins.some(item => item.action === 'manual-preferences' && !item.pinned && !item.protected)) throw Error('Original records still occupy ordinary N10 slots')
  for (let index = 0; index < 14; index++) {
    const result = await invoke('create_preferences_backup')
    scene.created.push(result)
    if (result.removedIds.length || result.cleanupError) throw Error('Manual creation pruned records')
  }
  const recent = await invoke('list_preferences_backups')
  const preview = await invoke('preview_preferences_backup_cleanup')
  if (preview.candidateIds.length) throw Error('D30 failed to retain new recent records')
  return { before, ordinaryOriginal, temporaryOriginalPins: scene.temporaryOriginalPins,
    afterOriginalPins, created: scene.created, recent, preview, policy,
    scope: 'temporary real native pins isolate the fourteen new owned N10 records; no original dates or payloads changed' }
})()
