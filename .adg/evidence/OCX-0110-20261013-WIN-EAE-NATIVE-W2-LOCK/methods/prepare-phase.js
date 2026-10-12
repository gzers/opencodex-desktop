(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const scene = window.__ocxEaeW2Lock
  if (!scene || scene.operation?.status === 'pending') throw Error('Scene absent or previous operation not settled')
  const phases = ['create', 'list', 'pin', 'policy', 'restore', 'cleanup']
  const phase = phases[scene.phaseIndex]
  if (!phase || scene.preparedPhase) throw Error('Unexpected phase sequence')
  if (JSON.stringify(await invoke('get_preferences')) !== JSON.stringify(scene.preferences)) throw Error('Original preferences not retained before phase')
  if (phase === 'restore') {
    await invoke('save_preferences', { preferences: { ...scene.preferences, theme: 'light' } })
    if ((await invoke('get_preferences')).theme !== 'light') throw Error('Different actual restore payload not prepared')
  }
  const created = []
  if (phase === 'cleanup') {
    for (let index = 0; index < 3; index++) {
      const result = await invoke('create_preferences_backup')
      scene.cleanupCreated.push(result)
      created.push(result)
      if (result.removedIds.length || result.cleanupError) throw Error('Cleanup fixture creation pruned inventory')
    }
  }
  scene.operation = null
  scene.preparedPhase = phase
  return { phase, created, currentPreferences: await invoke('get_preferences'),
    originalPreferences: scene.preferences, originalPolicy: scene.policy,
    sourceBackupId: scene.sourceBackup, inventory: await invoke('list_preferences_backups') }
})()
