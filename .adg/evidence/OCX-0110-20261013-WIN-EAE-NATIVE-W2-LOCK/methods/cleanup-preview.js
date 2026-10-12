(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const scene = window.__ocxEaeW2Lock
  if (scene?.preparedPhase !== 'cleanup' || scene.cleanupCreated.length !== 3) throw Error('Owned cleanup phase absent')
  const preview = await invoke('preview_preferences_backup_cleanup')
  const ids = scene.cleanupCreated.map(item => item.backup.backupId)
  if (JSON.stringify([...preview.candidateIds].sort()) !== JSON.stringify([...ids].sort())) throw Error('Non-owned cleanup candidate')
  window.__ocxEaeW2CleanupPreview = preview
  return { preview, ownedIds: ids }
})()
