(async () => {
  const probe = window.__ocxErrorBackoffProbe
  if (!probe) return { cleaned: false, reason: 'probe absent' }
  const internal = window.__TAURI_INTERNALS__
  internal.invoke = probe.originalInvoke
  const pinia = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia
  const saved = await pinia._s.get('preferences').save(probe.originalPreferences)
  if (!saved) throw Error('Original test scheduling preferences could not be restored')
  probe.status = 'finished'
  return {
    cleaned: true, recorderRemoved: true, originalAutoCheckRestored: probe.originalPreferences.appUpdateAutoCheck,
    actualAutoCheck: (await probe.originalInvoke('get_preferences')).appUpdateAutoCheck,
    planCalls: probe.planCalls, queries: probe.queries, sourceEventsGenerated: probe.sourceEventsGenerated,
    scope: 'recorder removed and original isolated scheduling preference restored; real operation evidence retained',
  }
})()
