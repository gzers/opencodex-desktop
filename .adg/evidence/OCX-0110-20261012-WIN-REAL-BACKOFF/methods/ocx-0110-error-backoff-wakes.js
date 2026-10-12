(async () => {
  const probe = window.__ocxErrorBackoffProbe
  if (!probe || probe.status !== 'waiting-real-floor') throw Error('Real backoff is not active')
  const pinia = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia
  const routes = pinia._s.get('routes')
  for (let index=0; index<100; index++) {
    routes.go(index % 2 ? 'overview' : 'settings')
    window.dispatchEvent(new Event('online'))
    probe.sourceEventsGenerated += 2
    await new Promise(resolve => setTimeout(resolve, 5))
  }
  await new Promise(resolve => setTimeout(resolve, 200))
  if (probe.planCalls.length !== 1 || probe.queries.length) throw Error('Queued route/online wakes bypassed the real error floor')
  return {
    planCalls: probe.planCalls, queries: probe.queries, sourceEventsGenerated: probe.sourceEventsGenerated,
    elapsedSinceFailureMs: performance.now() - probe.firstFailureAtMs,
    scope: 'actual production route watcher and generated online events after native error; no claim of a physical network reconnect',
  }
})()
