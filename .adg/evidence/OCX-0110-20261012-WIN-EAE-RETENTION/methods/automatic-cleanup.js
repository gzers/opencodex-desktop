(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const scene = window.__ocxEaeRetention
  if (!scene?.phases.automatic?.length) throw Error('Owned automatic IDs unavailable')
  const ids = scene.phases.automatic.map(result => result.backup.backupId)
  const sameIds = (left, right) => JSON.stringify([...left].sort()) === JSON.stringify([...right].sort())
  const output = { ownedIds: ids, scope: 'actual native opt-in and new successful backup; synthetic old dates, not D30 elapsed' }
  try {
    output.before = await invoke('list_preferences_backups')
    output.preview = await invoke('preview_preferences_backup_cleanup')
    if (!sameIds(output.preview.candidateIds, ids)) throw Error('Non-owned cleanup candidate')
    await invoke('save_preferences_backup_policy', { policy: { ...scene.originalPolicy, mode: 'automatic' } })
    output.automaticPolicy = await invoke('preferences_backup_policy')
    output.afterPolicySave = await invoke('list_preferences_backups')
    output.policySaveDidNotDelete = JSON.stringify(output.before) === JSON.stringify(output.afterPolicySave)
    if (!output.policySaveDidNotDelete) throw Error('Policy change deleted immediately')
    output.created = await invoke('create_preferences_backup')
    if (output.created.cleanupError || !sameIds(output.created.removedIds, ids)) throw Error('Automatic removals differ from owned candidates')
    output.after = await invoke('list_preferences_backups')
    const newlyCreated = output.after.find(item => item.id === output.created.backup.backupId)
    if (!newlyCreated) throw Error('Successful new backup missing')
    const oldOnly = output.after.filter(item => item.id !== newlyCreated.id)
    if (JSON.stringify(oldOnly) !== JSON.stringify(scene.before)) throw Error('Preexisting DTO inventory changed')
    if (!oldOnly.find(item => item.id === scene.originalPin.id)?.pinned) throw Error('Original pin changed')
    output.preexistingInventoryExactlyPreserved = true
    output.newlySuccessfulBackupRetained = true
    output.finalPreview = await invoke('preview_preferences_backup_cleanup')
    if (output.finalPreview.candidateIds.length) throw Error('Unexpected remaining cleanup candidate')
  } catch (error) {
    output.error = String(error)
  } finally {
    await invoke('save_preferences_backup_policy', { policy: scene.originalPolicy })
    output.finalPolicy = await invoke('preferences_backup_policy')
    output.cyclePreferences = await invoke('get_preferences')
    output.originalPolicyRestored = JSON.stringify(output.finalPolicy) === JSON.stringify(scene.originalPolicy)
  }
  return output
})()
