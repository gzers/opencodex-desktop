(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const scene = window.__ocxEaeW2Lock
  if (!scene || scene.operation?.status === 'pending') throw Error('Native operation must settle before cleanup')
  const current = await invoke('get_preferences')
  let fallbackPreferencesSaveUsed = false
  if (JSON.stringify(current) !== JSON.stringify(scene.preferences)) {
    await invoke('save_preferences', { preferences: scene.preferences })
    fallbackPreferencesSaveUsed = true
  }
  const inventoryBeforeUnpin = await invoke('list_preferences_backups')
  if (inventoryBeforeUnpin.find(item => item.id === scene.sourceBackup)?.pinned) {
    await invoke('set_preferences_backup_pinned', { id: scene.sourceBackup, pinned: false })
  }
  const store = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia?._s.get('preferences')
  await store?.load()
  const inventory = await invoke('list_preferences_backups')
  const originalIds = new Set(scene.inventory.map(item => item.id))
  if (JSON.stringify(inventory.filter(item => originalIds.has(item.id))) !== JSON.stringify(scene.inventory)) throw Error('Original inventory DTO changed')
  return {
    actualPhases: scene.records, inventory, preferences: await invoke('get_preferences'),
    policy: await invoke('preferences_backup_policy'), sourceBackupId: scene.sourceBackup,
    sourceTemporaryPinRemoved: !inventory.find(item => item.id === scene.sourceBackup)?.pinned,
    originalInventoryExactlyPreserved: true, fallbackPreferencesSaveUsed,
    storeTheme: store?.data?.theme, storeError: store?.error,
    nativeBridgeUnchanged: invoke === window.__TAURI_INTERNALS__.invoke,
    scope: 'post actual-command cleanup; no product restart or manual protection-state rewrite',
  }
})()
