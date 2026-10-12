(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const before = await invoke('list_notifications')
  const failure = before.items.find(item => item.id.startsWith('preferences-backup-failed') && !item.resolved)
  if (!failure) throw Error('Captured backup failure is absent')
  const backup = await invoke('create_preferences_backup')
  const after = await invoke('list_notifications')
  const recovered = after.items.find(item => item.id === failure.id)
  if (!recovered?.resolved) throw Error('Exact backup recovery did not resolve captured failure')
  return { backup, failureId: failure.id, recovered, after,
    scope: 'real native exact retry after restoring test-only saved policy' }
})()
