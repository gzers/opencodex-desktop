(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const preferencesStore = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia?._s.get('preferences')
  if (!preferencesStore) throw Error('Production preference store not mounted')
  const original = await invoke('get_preferences')
  const policy = await invoke('preferences_backup_policy')
  if (original.theme !== 'system' || original.appUpdateAutoCheck || policy.mode !== 'manual') {
    throw Error('Unexpected administrator fixture state')
  }
  const output = {
    startedAtUtc: new Date().toISOString(),
    originalPreferenceSummary: {
      theme: original.theme, automaticCheck: original.appUpdateAutoCheck, channel: original.appUpdateChannel,
    },
    cleanup: null,
    scope: 'actual administrator process native backup, temporary different-theme save and restore; no synthetic UI input, installer or package scripts',
  }
  let pinEstablished = false
  let backupId = null
  try {
    const created = await invoke('create_preferences_backup')
    if (created.cleanupError || created.removedIds.length) throw Error('Unexpected cleanup while preparing backup')
    backupId = created.backup.backupId
    output.createdBackupId = backupId
    await invoke('set_preferences_backup_pinned', { id: backupId, pinned: true })
    pinEstablished = true
    const light = await invoke('save_preferences', { preferences: { ...original, theme: 'light' } })
    if (light.theme !== 'light') throw Error('Different theme save did not commit')
    output.changedTheme = light.theme
    output.beforeRestorePin = (await invoke('list_preferences_backups')).find(item => item.id === backupId)
    output.restored = await invoke('restore_preferences_backup', { id: backupId })
    const restored = await invoke('get_preferences')
    output.restoredTheme = restored.theme
    output.restoredPreferenceDtoMatches = JSON.stringify(restored) === JSON.stringify(original)
    if (!output.restoredPreferenceDtoMatches) throw Error('Restore differed from original DTO')
    output.pinAfterRestore = (await invoke('list_preferences_backups')).find(item => item.id === backupId)
    output.protectionAfterRestore = (await invoke('list_preferences_backups'))
      .find(item => item.id === output.restored.protectionBackupId)
  } catch (error) {
    output.observationError = String(error)
  } finally {
    const current = await invoke('get_preferences')
    if (JSON.stringify(current) !== JSON.stringify(original)) {
      await invoke('save_preferences', { preferences: original })
      output.fallbackPreferenceSaveUsed = true
    }
    if (pinEstablished) await invoke('set_preferences_backup_pinned', { id: backupId, pinned: false })
    await preferencesStore.load()
    const after = await invoke('get_preferences')
    const backups = await invoke('list_preferences_backups')
    output.cleanup = {
      preferenceDtoRestored: JSON.stringify(after) === JSON.stringify(original),
      temporaryPinRemoved: !backups.find(item => item.id === backupId)?.pinned,
      originalUserPinRetained: !!backups.find(item => item.id === 'bk_20261012044452_a921f503')?.pinned,
      productionStoreError: preferencesStore.error,
      productionStoreTheme: preferencesStore.data?.theme,
      automaticChecksDisabled: after.appUpdateAutoCheck === false,
      nativeInvokeUnchanged: invoke === window.__TAURI_INTERNALS__.invoke,
    }
  }
  output.finishedAtUtc = new Date().toISOString()
  return output
})()
