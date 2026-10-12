(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const scene = window.__ocxEaeN10PinScene
  if (!scene) throw Error('Original temporary-pin journal unavailable')
  const restoredOriginalPinIds = []
  for (const id of [...scene.temporaryOriginalPins].reverse()) {
    await invoke('set_preferences_backup_pinned', { id, pinned: false })
    restoredOriginalPinIds.push(id)
    scene.temporaryOriginalPins = scene.temporaryOriginalPins.filter(value => value !== id)
  }
  if (scene.newestPinned) {
    await invoke('set_preferences_backup_pinned', { id: scene.created[13].backup.backupId, pinned: false })
    scene.newestPinned = false
  }
  const after = await invoke('list_preferences_backups')
  const originalIds = new Set(scene.before.map(item => item.id))
  const originalAfter = after.filter(item => originalIds.has(item.id))
  if (JSON.stringify(originalAfter) !== JSON.stringify(scene.before)) throw Error('Original inventory DTO did not restore exactly')
  const current = await invoke('get_preferences')
  if (JSON.stringify(current) !== JSON.stringify(scene.originalPreferences)) throw Error('Cycle preferences changed')
  const policy = await invoke('preferences_backup_policy')
  if (JSON.stringify(policy) !== JSON.stringify(scene.policy)) throw Error('Original policy changed')
  return { restoredOriginalPinIds, originalInventoryExactlyRestored: true, after, policy,
    newestTemporaryPinRemoved: !scene.newestPinned, currentPreferences: current,
    finalPreview: await invoke('preview_preferences_backup_cleanup'),
    nativeBridgeUnchanged: invoke === window.__TAURI_INTERNALS__.invoke }
})()
