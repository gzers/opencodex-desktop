(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const before = await invoke('official_remote_cache')
  const beta = await invoke('set_update_channel', { channel: 'beta' })
  const afterBeta = await invoke('official_remote_cache')
  const stable = await invoke('set_update_channel', { channel: 'stable' })
  const afterStable = await invoke('official_remote_cache')
  if (JSON.stringify(before) !== JSON.stringify(afterBeta)
      || JSON.stringify(before) !== JSON.stringify(afterStable)) throw Error('Manager channel switch changed panel cache')
  return { before, beta, afterBeta, stable, afterStable, panelCachePreserved: true,
    scope: 'real channel transactions; persisted panel schedule checked separately' }
})()
