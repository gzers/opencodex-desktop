(async () => {
  const trace = window.__ocxTimerBackoffTrace
  if (!trace || trace.status !== 'installed-no-fixture') throw Error('Verified recorder is not available')
  const pinia = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia
  if (!await pinia._s.get('preferences').save({ ...trace.originalPreferences, appUpdateAutoCheck: true })) {
    throw Error('Real automatic scheduling preference save failed')
  }
  trace.status = 'awaiting-fresh-thirty-minute-arm'
  return {
    status: trace.status, startedAtUtc: trace.startedAtUtc,
    scope: 'enable the real production scheduler against the isolated invalid file; retain any retry floor from the earlier trial',
  }
})()
