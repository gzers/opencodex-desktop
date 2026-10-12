(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const before = await invoke('list_preferences_backups')
  const originalPin = before.find(item => item.id === 'bk_20261012044452_a921f503')
  if (!originalPin?.pinned) throw Error('Original pinned backup missing')
  const preview = await invoke('preview_preferences_backup_cleanup')
  if (preview.candidateIds.length !== 3 || preview.candidateIds.includes(originalPin.id)) {
    throw Error('Historical date fixture did not yield the expected three ordinary candidates')
  }
  const extraPinId = preview.candidateIds[0]
  await invoke('set_preferences_backup_pinned', { id: extraPinId, pinned: true })
  let staleRejected = false
  try { await invoke('cleanup_preferences_backups', { preview }) }
  catch { staleRejected = true }
  if (!staleRejected) throw Error('Stale preview was accepted after a pin change')
  const afterRejected = await invoke('list_preferences_backups')
  if (afterRejected.length !== before.length) throw Error('Stale preview deleted test backups')
  const extraPinned = afterRejected.find(item => item.id === extraPinId)
  if (!extraPinned?.pinned) throw Error('Additional pin was not preserved')
  await invoke('set_preferences_backup_pinned', { id: extraPinId, pinned: false })
  const fresh = await invoke('preview_preferences_backup_cleanup')
  const removed = await invoke('cleanup_preferences_backups', { preview: fresh })
  const after = await invoke('list_preferences_backups')
  if (JSON.stringify([...removed].sort()) !== JSON.stringify([...fresh.candidateIds].sort())) {
    throw Error('Actual removals differ from fresh preview')
  }
  const remainingManual = after.filter(item => item.action === 'manual-preferences')
  if (remainingManual.length !== 11
      || remainingManual.filter(item => !item.pinned).length !== 10
      || !remainingManual.find(item => item.id === originalPin.id)?.pinned) {
    throw Error('Pinned record occupied a recent-record slot or was deleted')
  }
  const finalPreview = await invoke('preview_preferences_backup_cleanup')
  if (finalPreview.candidateIds.length) throw Error('Cleanup left unexpected ordinary candidates')
  return {
    before, preview, extraPinId, staleRejected, afterRejected, fresh, removed,
    after, finalPreview, ordinaryManualRetained: 10, originalPinRetained: true,
    scope: 'real native cleanup using explicitly synthetic historical manifest dates; not elapsed D30 or long-cycle observation',
  }
})()
