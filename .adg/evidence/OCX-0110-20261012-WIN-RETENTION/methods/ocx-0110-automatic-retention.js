(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  if ((await invoke('preferences_backup_policy')).mode !== 'automatic') throw Error('Automatic cleanup was not explicitly enabled')
  const before = await invoke('list_preferences_backups')
  const preview = await invoke('preview_preferences_backup_cleanup')
  if (preview.candidateIds.length !== 3) throw Error('Expected three old manual records before new backup')
  const created = await invoke('create_preferences_backup')
  const after = await invoke('list_preferences_backups')
  const manual = after.filter(item => item.action === 'manual-preferences')
  if (created.cleanupError || created.removedIds.length !== 4
      || manual.length !== 11 || manual.filter(item => !item.pinned).length !== 10) {
    throw Error('Automatic cleanup after successful backup violates N10 plus pins')
  }
  if (!manual.find(item => item.id === 'bk_20261012044452_a921f503')?.pinned) throw Error('Automatic cleanup deleted original pin')
  const successful = manual.find(item => item.id === created.backup.backupId)
  if (!successful) throw Error('Newly successful backup was removed')
  await invoke('save_preferences_backup_policy', { policy: { schema_version: 1, mode: 'manual', keep_recent: 10, keep_days: 30 } })
  return { before, preview, created, after, newlySuccessfulBackupRetained: true,
    originalPinRetained: true, restoredPolicy: await invoke('preferences_backup_policy'),
    scope: 'native automatic cleanup triggered by real new backup success against synthetic old dates; not long-time observation' }
})()
