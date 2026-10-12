(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const scene = window.__ocxEaeProtectionScene
  if (!scene?.failedGuardId) throw Error('Failed guard scene absent')
  const id = scene.created[1].backup.backupId
  const result = await invoke('restore_preferences_backup', { id })
  const inventory = await invoke('list_preferences_backups')
  const notifications = (await invoke('list_notifications')).items.map(({ id, resolved, resolvedAt }) => ({ id, resolved, resolvedAt }))
  const preferencesUnchanged = JSON.stringify(await invoke('get_preferences')) === JSON.stringify(scene.originalPreferences)
  if (!preferencesUnchanged || result.refreshRequired || result.protectionReconciliationPending) throw Error('Healthy restore not fully committed')
  const failedGuard = inventory.find(item => item.id === scene.failedGuardId)
  const successfulGuard = inventory.find(item => item.id === result.protectionBackupId)
  if (!failedGuard?.protected || successfulGuard?.protected) throw Error('Old failure protection or new successful guard state changed incorrectly')
  return { candidateId: id, actualRestore: result, inventory, notifications, preferencesDtoExactlyPreserved: true,
    failedGuard, successfulGuard, scope: 'real successful Restore for another candidate, not manual notification resolution' }
})()
