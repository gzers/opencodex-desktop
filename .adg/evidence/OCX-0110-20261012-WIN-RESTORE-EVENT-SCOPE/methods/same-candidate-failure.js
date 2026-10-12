(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const context = window.__ocxRestoreEventScope
  if (!context?.a) throw Error('Missing prepared backup A')
  const before = await invoke('list_notifications')
  let rejected = false
  try { await invoke('restore_preferences_backup', { id: context.a }) } catch { rejected = true }
  return { ...context, rejected, before, afterFailure: await invoke('list_notifications'),
    scope: 'new real failure of current A candidate without intervening restore candidate' }
})()
