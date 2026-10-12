(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const scene = window.__ocxEaePolicyFailureScene
  if (scene?.created.length !== 3) throw Error('Three owned candidate IDs unavailable')
  const before = await invoke('list_preferences_backups')
  const preview = await invoke('preview_preferences_backup_cleanup')
  const ids = scene.created.map(item => item.backup.backupId)
  if (JSON.stringify([...preview.candidateIds].sort()) !== JSON.stringify([...ids].sort())) throw Error('Non-owned cleanup candidate')
  await invoke('save_preferences_backup_policy', { policy: { ...scene.policy, mode: 'automatic' } })
  const after = await invoke('list_preferences_backups')
  if (JSON.stringify(before) !== JSON.stringify(after)) throw Error('Policy save deleted immediately')
  return { before, after, preview, automaticPolicy: await invoke('preferences_backup_policy'),
    policySaveDidNotDelete: true, scope: 'actual explicit automatic opt-in, no backup creation or cleanup call' }
})()
