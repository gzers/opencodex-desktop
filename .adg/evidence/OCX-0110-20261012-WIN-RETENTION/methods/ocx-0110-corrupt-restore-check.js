(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const before = await invoke('get_preferences')
  const recordsBefore = await invoke('list_preferences_backups')
  let rejected = false
  try { await invoke('restore_preferences_backup', { id: 'bk_20261012044452_a921f503' }) }
  catch { rejected = true }
  if (!rejected) throw Error('Corrupt backup payload was restored')
  const after = await invoke('get_preferences')
  const recordsAfter = await invoke('list_preferences_backups')
  if (JSON.stringify(before) !== JSON.stringify(after)) throw Error('Corrupt restore changed active preferences')
  const pinned = recordsAfter.find(item => item.id === 'bk_20261012044452_a921f503')
  if (!pinned?.pinned) throw Error('Corrupt restore removed user pin')
  return { rejected, before, after, recordsBefore, recordsAfter, pinRetained: true,
    notifications: await invoke('list_notifications'),
    scope: 'real native restore rejection of a temporary corrupted test payload' }
})()
