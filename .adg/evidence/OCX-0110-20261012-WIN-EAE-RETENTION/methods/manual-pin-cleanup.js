(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const scene = window.__ocxEaeRetention
  if (!scene?.phases.manual?.length) throw Error('Owned manual IDs unavailable')
  const ids = scene.phases.manual.map(result => result.backup.backupId)
  const sameIds = (left, right) => JSON.stringify([...left].sort()) === JSON.stringify([...right].sort())
  const output = { ownedIds: ids, scope: 'actual native commands, synthetic dates for three newly owned records' }
  let pinned = false
  try {
    const before = await invoke('list_preferences_backups')
    const preview = await invoke('preview_preferences_backup_cleanup')
    output.before = before
    output.preview = preview
    if (!sameIds(preview.candidateIds, ids)) throw Error('Cleanup includes preexisting records')
    await invoke('set_preferences_backup_pinned', { id: ids[0], pinned: true })
    pinned = true
    output.pinnedInventory = await invoke('list_preferences_backups')
    output.pinnedPreview = await invoke('preview_preferences_backup_cleanup')
    if (!sameIds(output.pinnedPreview.candidateIds, ids.slice(1))) throw Error('Pin occupied a recent slot or remained a candidate')
    try {
      await invoke('cleanup_preferences_backups', { preview })
      output.stalePreviewAccepted = true
    } catch (error) {
      output.stalePreviewAccepted = false
      output.staleError = error
    }
    output.afterRejected = await invoke('list_preferences_backups')
    if (output.stalePreviewAccepted || output.afterRejected.length !== before.length) throw Error('Stale cleanup mutated inventory')
  } catch (error) {
    output.error = String(error)
  } finally {
    if (pinned) await invoke('set_preferences_backup_pinned', { id: ids[0], pinned: false })
  }
  if (!output.error) {
    output.freshPreview = await invoke('preview_preferences_backup_cleanup')
    if (!sameIds(output.freshPreview.candidateIds, ids)) throw Error('Fresh cleanup includes non-owned records')
    output.removed = await invoke('cleanup_preferences_backups', { preview: output.freshPreview })
    output.after = await invoke('list_preferences_backups')
    output.finalPreview = await invoke('preview_preferences_backup_cleanup')
    if (!sameIds(output.removed, ids) || output.finalPreview.candidateIds.length) throw Error('Fresh cleanup differs from preview')
    if (!output.after.find(item => item.id === scene.originalPin.id)?.pinned) throw Error('Original pin changed')
    if (JSON.stringify(output.after) !== JSON.stringify(scene.before)) throw Error('Preexisting DTO inventory changed')
    output.preexistingInventoryExactlyPreserved = true
  }
  return output
})()
