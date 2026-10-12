(async () => {
  const trace = window.__ocxTimerBackoffTrace
  if (!trace) return { cleaned: false, reason: 'no verified trace' }
  const pinia = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia
  const saved = await pinia._s.get('preferences').save(trace.originalPreferences)
  window.setTimeout = trace.originalSetTimeout
  window.clearTimeout = trace.originalClearTimeout
  if (!saved || window.__TAURI_INTERNALS__.invoke !== trace.originalInvoke) throw Error('Original test preference or bridge identity did not restore')
  trace.status = 'finished'
  return {
    cleaned: true, originalTimersRestored: true, nativeBridgeUnchanged: true,
    actualAutoCheck: (await trace.originalInvoke('get_preferences')).appUpdateAutoCheck,
    sourceWitness: trace.sourceWitness ?? null, anchor: trace.anchor ?? null,
    sourceEvents: trace.sourceEvents, records: trace.records,
    scope: 'all real timer records retained; original preference and standard timer functions restored, no native bridge override',
  }
})()
