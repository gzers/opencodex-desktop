(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  if (window.__ocxEaeW2Lock) throw Error('Existing native lock scene cannot be restarted')
  const preferences = await invoke('get_preferences')
  const policy = await invoke('preferences_backup_policy')
  if (preferences.theme !== 'system' || preferences.appUpdateChannel !== 'stable'
      || !preferences.appUpdateAutoCheck || policy.mode !== 'manual') throw Error('Cycle or policy precondition differs')
  const inventory = await invoke('list_preferences_backups')
  if ((await invoke('preview_preferences_backup_cleanup')).candidateIds.length) throw Error('Preexisting cleanup candidates')
  const created = await invoke('create_preferences_backup')
  if (created.removedIds.length || created.cleanupError) throw Error('Setup pruned existing records')
  window.__ocxEaeW2Lock = {
    preferences, policy, inventory, sourceBackup: created.backup.backupId,
    phaseIndex: 0, preparedPhase: null, operation: null, cleanupCreated: [], records: [],
  }
  return { preferences, policy, inventory, created, phases: ['create', 'list', 'pin', 'policy', 'restore', 'cleanup'],
    scope: 'actual existing final desktop, one new owned backup; no UI input, second desktop or schedule mutation' }
})()
