(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const rootPath = 'C:\\Users\\15119\\.codex\\worktrees\\0110-windows-gates\\opencodex-desktop\\test\\runtime\\windows0110\\custom-root'
  const before = await invoke('get_data_root_config')
  const initialized = await invoke('initialize_data_root', { request: { rootPath } })
  return { before, rootPath, initialized,
    scope: 'initializes a sibling test root only, before reference switch' }
})()
