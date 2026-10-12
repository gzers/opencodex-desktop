(async () => {
  const internal = window.__TAURI_INTERNALS__
  const invoke = internal.invoke
  if (window.__ocxErrorBackoffProbe) throw Error('Existing real backoff probe must be observed, not restarted')
  const pinia = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia
  const prefs = pinia._s.get('preferences')
  const effects = pinia._s.get('effects')
  const updates = pinia._s.get('updates')
  const roots = pinia._s.get('data-root')
  if (!await invoke('app_foreground') || document.visibilityState !== 'visible'
      || !effects.foreground || !effects.visible) throw Error('Actual foreground precondition is not satisfied')
  if (updates.appUpdateBusy || updates.officialUpdateBusy || roots.switching) throw Error('Existing native transaction is busy')
  const originalPreferences = await invoke('get_preferences')
  if (originalPreferences.appUpdateAutoCheck) throw Error('Initial manual mode must be retained until probe is armed')
  const probe = {
    startedAtUtc: new Date().toISOString(), startedAtMs: performance.now(),
    originalInvoke: invoke, originalPreferences, planCalls: [], queries: [],
    status: 'armed', sourceEventsGenerated: 0, realClock: true, requiredFloorMs: 1_800_000,
  }
  internal.invoke = function(command, args, options) {
    const record = command === 'update_schedule_plan'
      ? { calledAtUtc: new Date().toISOString(), calledAtMs: performance.now(), outcome: 'pending' }
      : null
    if (record) probe.planCalls.push(record)
    if (command === 'check_for_update' || command === 'official_remote_latest') {
      probe.queries.push({ command, calledAtUtc: new Date().toISOString(), calledAtMs: performance.now(),
        trigger: args?.trigger ?? null })
    }
    const result = invoke.call(internal, command, args, options)
    if (!record) return result
    return Promise.resolve(result).then(value => {
      record.completedAtMs = performance.now(); record.outcome = 'fulfilled'
      record.targetCount = value.targets?.length ?? null
      return value
    }, error => {
      record.completedAtMs = performance.now(); record.outcome = 'rejected'
      record.code = error?.code ?? null
      throw error
    })
  }
  window.__ocxErrorBackoffProbe = probe
  if (!await prefs.save({ ...originalPreferences, appUpdateAutoCheck: true })) throw Error('Automatic scheduling preference could not be enabled')
  for (let attempt=0; attempt<100 && !probe.planCalls.some(call => call.outcome === 'rejected'); attempt++) {
    await new Promise(resolve => setTimeout(resolve, 50))
  }
  const failed = probe.planCalls.find(call => call.outcome === 'rejected')
  if (!failed) throw Error('Real invalid schedule did not reach the production scheduler')
  probe.firstFailureAtMs = failed.completedAtMs
  probe.status = 'waiting-real-floor'
  return {
    startedAtUtc: probe.startedAtUtc, firstFailureAtMs: probe.firstFailureAtMs,
    planCalls: probe.planCalls, queries: probe.queries, requiredFloorMs: probe.requiredFloorMs,
    foreground: await invoke('app_foreground'),
    scope: 'actual native schedule JSON failure observed through a transparent invoke recorder; real production timer and real performance.now retained',
  }
})()
