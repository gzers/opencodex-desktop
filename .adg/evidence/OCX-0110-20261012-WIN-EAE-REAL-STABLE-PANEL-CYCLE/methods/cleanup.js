(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const trace = window.__ocxRealStablePanelCycle
  if (!trace) throw Error('Real cycle marker absent; do not silently modify another run')
  const current = await invoke('get_preferences')
  await invoke('save_preferences', { preferences: { ...current,
    appUpdateChannel: trace.originalChannel, appUpdateAutoCheck: trace.originalAutomaticCheck } })
  const store = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia?._s.get('preferences')
  await store?.load()
  const actual = await invoke('get_preferences')
  trace.status = 'observation-ended-original-update-preferences-restored'
  return {
    channel: actual.appUpdateChannel, automaticCheck: actual.appUpdateAutoCheck,
    originalUpdatePreferencesRestored: actual.appUpdateChannel === trace.originalChannel
      && actual.appUpdateAutoCheck === trace.originalAutomaticCheck,
    sourceEvents: trace.events,
    scope: 'restore original update preferences only after monitor terminal decision; no cycle success inferred',
  }
})()
