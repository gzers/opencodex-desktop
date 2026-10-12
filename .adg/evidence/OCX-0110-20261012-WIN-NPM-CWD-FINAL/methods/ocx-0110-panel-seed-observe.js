(async () => {
  const operation = window.__ocxPanelSeedOperation
  if (!operation) throw Error('No retained seed installation handle')
  const invoke = window.__TAURI_INTERNALS__.invoke
  return { operation, source: await invoke('runtime_source'), protection: await invoke('runtime_protection_status'),
    scope: 'observe the existing native installation; this script never restarts it' }
})()
