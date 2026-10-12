(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  if (window.__ocxEaeRetention) throw Error('Retention scene already exists')
  const policy = await invoke('preferences_backup_policy')
  const preferences = await invoke('get_preferences')
  if (policy.mode !== 'manual' || policy.keep_recent !== 10 || policy.keep_days !== 30
      || preferences.appUpdateChannel !== 'stable' || !preferences.appUpdateAutoCheck) {
    throw Error('Original policy or live-cycle preference differs')
  }
  const before = await invoke('list_preferences_backups')
  const originalPin = before.find(item => item.id === 'bk_20261012044452_a921f503')
  if (!originalPin?.pinned) throw Error('Original user pin missing')
  if ((await invoke('preview_preferences_backup_cleanup')).candidateIds.length) {
    throw Error('Preexisting cleanup candidates: no scene mutation allowed')
  }
  window.__ocxEaeRetention = { originalPolicy: policy, before, originalPin, phases: {} }
  const created = []
  for (let i = 0; i < 3; i++) {
    const result = await invoke('create_preferences_backup')
    created.push(result)
    window.__ocxEaeRetention.phases.manual = created
    if (result.removedIds.length || result.cleanupError) throw Error('Unexpected creation cleanup')
  }
  const after = await invoke('list_preferences_backups')
  const preview = await invoke('preview_preferences_backup_cleanup')
  if (preview.candidateIds.length) throw Error('D30 failed to retain recent fixture records')
  return { before, created, after, preview, policy,
    recentManualCount: after.filter(item => item.action === 'manual-preferences').length,
    mouseKeyboardUiAccepted: false, syntheticDatesYetApplied: false }
})()
