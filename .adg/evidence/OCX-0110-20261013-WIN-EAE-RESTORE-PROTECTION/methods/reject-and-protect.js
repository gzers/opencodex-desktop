(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const scene = window.__ocxEaeProtectionScene
  if (scene?.created.length !== 2) throw Error('Two owned records required')
  const damagedId = scene.created[0].backup.backupId
  const healthyId = scene.created[1].backup.backupId
  const expectedIds = [damagedId, healthyId].sort()
  const output = { damagedId, healthyId, before: await invoke('list_preferences_backups') }
  const notes = async () => (await invoke('list_notifications')).items.map(({ id, resolved, resolvedAt }) => ({ id, resolved, resolvedAt }))
  output.beforePreview = await invoke('preview_preferences_backup_cleanup')
  if (JSON.stringify([...output.beforePreview.candidateIds].sort()) !== JSON.stringify(expectedIds)) throw Error('Non-owned candidate')
  try {
    await invoke('cleanup_preferences_backups', { preview: output.beforePreview })
    throw Error('Cleanup accepted damaged payload')
  } catch (error) {
    if (String(error).includes('Cleanup accepted')) throw error
    output.cleanupRejected = true
    output.cleanupError = error
  }
  output.afterCleanup = await invoke('list_preferences_backups')
  if (JSON.stringify(output.afterCleanup) !== JSON.stringify(output.before)) throw Error('Cleanup deleted a verified first candidate before checking the damaged second')
  try {
    await invoke('restore_preferences_backup', { id: damagedId })
    throw Error('Restore accepted damaged payload')
  } catch (error) {
    if (String(error).includes('Restore accepted')) throw error
    output.restoreRejected = true
    output.restoreError = error
  }
  output.afterRestore = await invoke('list_preferences_backups')
  const beforeIds = new Set(output.before.map(item => item.id))
  const guards = output.afterRestore.filter(item => !beforeIds.has(item.id))
  if (guards.length !== 1 || guards[0].action !== 'restore-protection' || !guards[0].protected) throw Error('Expected one active restore guard')
  output.failedGuard = guards[0]
  scene.failedGuardId = guards[0].id
  const referenced = output.afterRestore.find(item => item.id === damagedId)
  if (!referenced?.protected || referenced.protectionReason !== 'referenced-by-protection') throw Error('Failed restore source unprotected')
  output.afterRestorePreview = await invoke('preview_preferences_backup_cleanup')
  if (JSON.stringify(output.afterRestorePreview.candidateIds) !== JSON.stringify([healthyId])) throw Error('Source/guard linkage did not affect preview')
  try {
    await invoke('set_preferences_backup_pinned', { id: guards[0].id, pinned: false })
    throw Error('Unpin released active guard')
  } catch (error) {
    if (String(error).includes('Unpin released')) throw error
    output.activeGuardUnpinRejected = true
    output.activeGuardUnpinError = error
  }
  try {
    await invoke('cleanup_preferences_backups', { preview: output.beforePreview })
    throw Error('Old preview accepted changed guard inventory')
  } catch (error) {
    if (String(error).includes('Old preview accepted')) throw error
    output.stalePreviewRejected = true
    output.stalePreviewError = error
  }
  output.finalInventory = await invoke('list_preferences_backups')
  if (JSON.stringify(output.finalInventory) !== JSON.stringify(output.afterRestore)) throw Error('Rejected guard/cleanup command changed inventory')
  output.afterFailureNotifications = await notes()
  output.preferencesDtoExactlyPreserved = JSON.stringify(await invoke('get_preferences')) === JSON.stringify(scene.originalPreferences)
  if (!output.preferencesDtoExactlyPreserved) throw Error('Failed restore altered preferences')
  return output
})()
