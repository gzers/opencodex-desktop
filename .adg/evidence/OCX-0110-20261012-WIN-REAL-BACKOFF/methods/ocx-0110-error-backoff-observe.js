(async () => {
  const probe = window.__ocxErrorBackoffProbe
  if (!probe || !probe.firstFailureAtMs) throw Error('No retained real backoff probe')
  const invoke = probe.originalInvoke
  return {
    status: probe.status, nowAtMs: performance.now(), firstFailureAtMs: probe.firstFailureAtMs,
    elapsedSinceFailureMs: performance.now() - probe.firstFailureAtMs,
    requiredFloorMs: probe.requiredFloorMs, planCalls: probe.planCalls, queries: probe.queries,
    sourceEventsGenerated: probe.sourceEventsGenerated, startedAtUtc: probe.startedAtUtc,
    observedAtUtc: new Date().toISOString(), foreground: await invoke('app_foreground'),
    visibility: document.visibilityState, realClock: probe.realClock,
    scope: 'read existing recorder only; never invokes update_schedule_plan or restarts the scheduler',
  }
})()
