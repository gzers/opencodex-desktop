(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const results = []
  for (const [command, args] of [
    ['list_preferences_backups', undefined],
    ['preview_preferences_backup_cleanup', undefined],
    ['restore_preferences_backup', { id: '../junction-target/sentinel.txt' }],
    ['open_backup_file', { id: '../junction-target/sentinel.txt' }],
  ]) {
    let rejected = false
    try { await invoke(command, args) } catch { rejected = true }
    results.push({ command, rejected })
    if (!rejected) throw Error(command + ' accepted a link/escape fixture')
  }
  return { results, scope: 'real native rejection with an isolated directory junction and traversal ids; no deletion through the junction' }
})()
