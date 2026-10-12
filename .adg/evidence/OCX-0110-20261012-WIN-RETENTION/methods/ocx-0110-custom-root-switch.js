(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const before = await invoke('get_data_root_config')
  const targetPath = 'C:\\Users\\15119\\.codex\\worktrees\\0110-windows-gates\\opencodex-desktop\\test\\runtime\\windows0110\\custom-root'
  const switched = await invoke('switch_data_root', { request: { targetPath, mode: 'reference_only' } })
  if (switched.status !== 'restart_required') {
    return { before, switched, admissionAccepted: false, scope: 'real admission; blocked result is not waived' }
  }
  const after = await invoke('get_data_root_config')
  if (after.currentDataRoot !== before.currentDataRoot || after.runtimeActive) {
    throw Error('Pending switch changed running storage identity prematurely')
  }
  let backupRejected = false
  try { await invoke('create_preferences_backup') } catch { backupRejected = true }
  if (!backupRejected) throw Error('Pending reference switch allowed an old-root writer')
  return { before, switched, after, admissionAccepted: true, backupRejected,
    scope: 'real native reference switch and old-root freeze; restart verification pending' }
})()
