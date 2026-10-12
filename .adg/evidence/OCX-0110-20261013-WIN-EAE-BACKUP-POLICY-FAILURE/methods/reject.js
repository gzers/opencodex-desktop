(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const scene = window.__ocxEaePolicyFailureScene
  if (!scene) throw Error('Prepared scene absent')
  const before = await invoke('list_preferences_backups')
  const output = { before }
  try {
    await invoke('create_preferences_backup')
    throw Error('Malformed policy unexpectedly admitted a backup')
  } catch (error) {
    if (String(error).includes('Malformed policy unexpectedly')) throw error
    output.backupRejected = true
    output.backupError = error
  }
  output.after = await invoke('list_preferences_backups')
  if (JSON.stringify(before) !== JSON.stringify(output.after)) throw Error('Failed creation mutated backup inventory')
  output.previewAfterFailure = await invoke('preview_preferences_backup_cleanup')
  if (JSON.stringify([...output.previewAfterFailure.candidateIds].sort()) !==
      JSON.stringify(scene.created.map(item => item.backup.backupId).sort())) throw Error('Failed backup rotated owned candidates')
  try {
    await invoke('preferences_backup_policy')
    throw Error('Malformed policy silently fell back to manual')
  } catch (error) {
    if (String(error).includes('silently fell back')) throw error
    output.policyReadRejected = true
    output.policyReadError = error
  }
  output.notifications = (await invoke('list_notifications')).items.map(({ id, resolved, resolvedAt }) => ({ id, resolved, resolvedAt }))
  output.preferencesDtoExactlyPreserved = JSON.stringify(await invoke('get_preferences')) === JSON.stringify(scene.preferences)
  if (!output.preferencesDtoExactlyPreserved) throw Error('Failure changed live-cycle preferences')
  return output
})()
