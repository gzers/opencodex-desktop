(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const scene = window.__ocxEaeN10PinScene
  if (scene?.created.length !== 14) throw Error('Fourteen actual owned records required')
  const ids = scene.created.map(result => result.backup.backupId)
  const same = (left, right) => JSON.stringify([...left].sort()) === JSON.stringify([...right].sort())
  const output = { ownedIds: ids, newestId: ids[13], before: await invoke('list_preferences_backups') }
  output.beforePinPreview = await invoke('preview_preferences_backup_cleanup')
  if (!same(output.beforePinPreview.candidateIds, ids.slice(0, 4))) throw Error('N10 did not retain exactly newest ten ordinary old records')
  await invoke('set_preferences_backup_pinned', { id: ids[13], pinned: true })
  scene.newestPinned = true
  output.afterNewestPin = await invoke('list_preferences_backups')
  output.afterPinPreview = await invoke('preview_preferences_backup_cleanup')
  if (!same(output.afterPinPreview.candidateIds, ids.slice(0, 3))) throw Error('Newest pin incorrectly occupied one of the ten ordinary slots')
  try {
    await invoke('cleanup_preferences_backups', { preview: output.beforePinPreview })
    throw Error('Stale preview accepted newest pin change')
  } catch (error) {
    if (String(error).includes('Stale preview accepted')) throw error
    output.stalePreviewRejected = true
    output.stalePreviewError = error
  }
  output.afterRejected = await invoke('list_preferences_backups')
  if (JSON.stringify(output.afterRejected) !== JSON.stringify(output.afterNewestPin)) throw Error('Stale cleanup changed inventory')
  output.freshPreview = await invoke('preview_preferences_backup_cleanup')
  if (!same(output.freshPreview.candidateIds, ids.slice(0, 3))) throw Error('Non-owned fresh candidate')
  output.removed = await invoke('cleanup_preferences_backups', { preview: output.freshPreview })
  if (!same(output.removed, ids.slice(0, 3))) throw Error('Cleanup differs from exactly owned three candidates')
  output.after = await invoke('list_preferences_backups')
  const owned = output.after.filter(item => ids.includes(item.id))
  output.retainedOwned = owned
  output.ordinaryOwnedRetained = owned.filter(item => !item.pinned && !item.protected).length
  output.pinnedOwnedRetained = owned.filter(item => item.pinned).length
  if (owned.length !== 11 || output.ordinaryOwnedRetained !== 10 || output.pinnedOwnedRetained !== 1
      || !owned.find(item => item.id === ids[13])?.pinned) throw Error('Expected ten ordinary plus one newest pinned record')
  if (scene.before.some(item => !output.after.some(value => value.id === item.id))) throw Error('Cleanup removed an original record')
  output.finalPreview = await invoke('preview_preferences_backup_cleanup')
  if (output.finalPreview.candidateIds.length) throw Error('Unexpected remaining candidate')
  return output
})()
