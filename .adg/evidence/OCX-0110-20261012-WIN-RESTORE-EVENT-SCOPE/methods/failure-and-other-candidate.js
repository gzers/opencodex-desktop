(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const context = window.__ocxRestoreEventScope
  if (!context?.a || !context?.b) throw Error('Missing prepared backup IDs')
  let rejected = false
  try { await invoke('restore_preferences_backup', { id: context.a }) } catch { rejected = true }
  const afterFailure = await invoke('list_notifications')
  if (!rejected) throw Error('Corrupt isolated backup was not rejected')
  const otherRestore = await invoke('restore_preferences_backup', { id: context.b })
  const afterOtherCandidate = await invoke('list_notifications')
  return { ...context, rejected, afterFailure, otherRestore, afterOtherCandidate,
    scope: 'actual restore integrity failure followed by real successful restore of another backup ID; no notification mutation or event injection' }
})()
