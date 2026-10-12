(async () => {
  const trace = window.__ocxTimerBackoffTrace
  if (!trace?.anchor || trace.status !== 'fresh-native-error-floor-observed') throw Error('No fresh real error floor')
  const pinia = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia
  for (let index=0; index<100; index++) {
    pinia._s.get('routes').go(index % 2 ? 'overview' : 'settings')
    window.dispatchEvent(new Event('online'))
    trace.sourceEvents += 2
    await new Promise(resolve => trace.originalSetTimeout.call(window, resolve, 5))
  }
  const associated = trace.records.filter(record => record.callbackText === trace.anchor.callbackText
    && record.scheduledAtMs >= trace.anchor.scheduledAtMs)
  if (associated.some(record => record.fired)) throw Error('A production retry fired during immediate source wakes')
  if (associated.filter(record => !record.fired && !record.cancelled).length > 1) {
    throw Error('Production scheduler retained multiple live timers')
  }
  return { sourceEvents: trace.sourceEvents, associatedArmCount: associated.length,
    activeTimerCount: associated.filter(record => !record.fired && !record.cancelled).length,
    firedCount: 0, scope: 'actual production route watchers plus generated online events, not physical network reconnect' }
})()
