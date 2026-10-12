(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const scene = window.__ocxEaeRetention
  if (!scene || (await invoke('preferences_backup_policy')).mode !== 'manual') throw Error('Manual policy not retained')
  const before = await invoke('list_preferences_backups')
  if (JSON.stringify(before) !== JSON.stringify(scene.before)) throw Error('Original inventory changed')
  const created = []
  for (let i = 0; i < 3; i++) {
    const result = await invoke('create_preferences_backup')
    created.push(result)
    scene.phases.automatic = created
    if (result.removedIds.length || result.cleanupError) throw Error('Unexpected manual cleanup')
  }
  const after = await invoke('list_preferences_backups')
  const preview = await invoke('preview_preferences_backup_cleanup')
  if (preview.candidateIds.length) throw Error('Recent records became candidates')
  return { before, created, after, preview, policy: await invoke('preferences_backup_policy') }
})()
