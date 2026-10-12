(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const scene = window.__ocxEaeProtectionScene
  if (!scene?.failedGuardId) throw Error('Failed guard scene absent')
  const id = scene.created[0].backup.backupId
  const result = await invoke('restore_preferences_backup', { id })
  const inventory = await invoke('list_preferences_backups')
  const notifications = (await invoke('list_notifications')).items.map(({ id, resolved, resolvedAt }) => ({ id, resolved, resolvedAt }))
  const preferencesUnchanged = JSON.stringify(await invoke('get_preferences')) === JSON.stringify(scene.originalPreferences)
  if (!preferencesUnchanged || result.refreshRequired || result.protectionReconciliationPending) throw Error('Repaired restore not fully committed')
  const failedGuard = inventory.find(item => item.id === scene.failedGuardId)
  const successfulGuard = inventory.find(item => item.id === result.protectionBackupId)
  if (!failedGuard?.protected || successfulGuard?.protected) throw Error('Old failed guard must not be released by a different transaction')
  const store = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia?._s.get('preferences')
  await store?.load()
  return { candidateId: id, actualRestore: result, inventory, notifications, preferencesDtoExactlyPreserved: true,
    failedGuard, successfulGuard, storeTheme: store?.data?.theme, storeError: store?.error,
    nativeBridgeUnchanged: invoke === window.__TAURI_INTERNALS__.invoke,
    scope: 'real Restore after returning to earlier fingerprint; source generation must not resurrect old UUID' }
})()
