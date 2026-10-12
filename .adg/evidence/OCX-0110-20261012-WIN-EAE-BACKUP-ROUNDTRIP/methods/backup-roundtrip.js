(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const pinia = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia
  const preferencesStore = pinia?._s.get('preferences')
  if (!preferencesStore) throw Error('Actual production preference store not mounted')
  const original = await invoke('get_preferences')
  const root = await invoke('get_data_root_config')
  const policy = await invoke('preferences_backup_policy')
  if (original.theme !== 'system' || original.themeNeedsImport || original.appUpdateChannel !== 'stable'
      || original.appUpdateAutoCheck !== true || policy.mode !== 'manual') {
    throw Error('Expected final-source cycle preferences and manual backup policy')
  }
  if (!root.currentDataRoot.includes('test\\runtime\\windows0110\\data')) throw Error('Unexpected isolated root')
  const notes = async () => {
    const value = await invoke('list_notifications')
    return value.items.map(({ id, resolved, resolvedAt }) => ({ id, resolved, resolvedAt }))
  }
  const output = {
    startedAtUtc: new Date().toISOString(),
    originalPreferences: {
      theme: original.theme, automaticCheck: original.appUpdateAutoCheck,
      channel: original.appUpdateChannel,
    },
    manualPolicy: policy,
    beforeNotifications: await notes(),
    phases: [],
    cleanup: null,
    scope: 'real final-desktop native commands and production-store readback; RDP disconnected, not a mouse/keyboard UI sample',
  }
  let pinned = false
  let backupId
  const assertCyclePreferences = async phase => {
    const value = await invoke('get_preferences')
    const valid = value.appUpdateAutoCheck === true && value.appUpdateChannel === 'stable'
    output.phases.push({ phase, atUtc: new Date().toISOString(), theme: value.theme,
      automaticCheck: value.appUpdateAutoCheck, channel: value.appUpdateChannel })
    if (!valid) throw Error('Cycle preferences altered during ' + phase)
    return value
  }
  try {
    const created = await invoke('create_preferences_backup')
    if (created.cleanupError || created.removedIds.length) throw Error('Backup unexpectedly pruned existing records')
    backupId = created.backup.backupId
    output.createdBackup = created
    await invoke('set_preferences_backup_pinned', { id: backupId, pinned: true })
    pinned = true
    output.pinBeforeRestore = (await invoke('list_preferences_backups')).find(item => item.id === backupId)
    await invoke('save_preferences', { preferences: { ...original, theme: 'light' } })
    const changed = await assertCyclePreferences('different light value persisted')
    if (changed.theme !== 'light') throw Error('Actual theme save not committed')
    output.changedAppearance = await invoke('window_appearance')
    output.restoreResult = await invoke('restore_preferences_backup', { id: backupId })
    const restored = await assertCyclePreferences('original snapshot restored')
    output.dtoMatchesOriginal = JSON.stringify(restored) === JSON.stringify(original)
    if (!output.dtoMatchesOriginal) throw Error('Restored DTO differs from captured original')
    const inventory = await invoke('list_preferences_backups')
    output.pinAfterRestore = inventory.find(item => item.id === backupId)
    output.protectionAfterRestore = inventory.find(item => item.id === output.restoreResult.protectionBackupId)
    output.afterNotifications = await notes()
  } catch (error) {
    output.observationError = String(error)
  } finally {
    const current = await invoke('get_preferences')
    if (JSON.stringify(current) !== JSON.stringify(original)) {
      await invoke('save_preferences', { preferences: original })
      output.fallbackPreferenceSaveUsed = true
    }
    if (pinned) await invoke('set_preferences_backup_pinned', { id: backupId, pinned: false })
    await preferencesStore.load()
    const final = await assertCyclePreferences('cleanup and production-store reload')
    const inventory = await invoke('list_preferences_backups')
    output.cleanup = {
      originalDtoRestored: JSON.stringify(final) === JSON.stringify(original),
      temporaryPinRemoved: !inventory.find(item => item.id === backupId)?.pinned,
      originalUserPinRetained: !!inventory.find(item => item.id === 'bk_20261012044452_a921f503')?.pinned,
      storeError: preferencesStore.error,
      storeTheme: preferencesStore.data?.theme,
      policy: await invoke('preferences_backup_policy'),
      invokeBridgeUnchanged: window.__TAURI_INTERNALS__.invoke === invoke,
    }
  }
  output.finishedAtUtc = new Date().toISOString()
  return output
})()
