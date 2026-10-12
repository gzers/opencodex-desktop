(async () => {
  const trace = window.__ocxTimerBackoffTrace
  if (!trace) throw Error('No retained timer trace')
  if (!trace.anchor) {
    const anchor = trace.records.find(record => record.delayMs === 1_800_000)
    if (anchor) {
      const source = trace.sources.find(source => source.text.includes(anchor.callbackText))
      if (!source) throw Error('Recorded timer callback is absent from the exact running app source')
      const position = source.text.indexOf(anchor.callbackText)
      const armSnippet = source.text.slice(Math.max(0, position - 160), position + anchor.callbackText.length + 160)
      const tail = source.text.slice(position, position + 3500)
      const errorPosition = tail.search(/performance\.now\(\)\s*\+\s*(?:18e5|1800000)/)
      if (!armSnippet.includes('setTimeout') || !armSnippet.includes('Math.max(100,') || errorPosition < 0) {
        throw Error('Thirty-minute timer cannot be attributed to the production update error branch')
      }
      trace.anchor = anchor
      trace.sourceWitness = {
        url: source.url, sha256: source.sha256, callbackText: anchor.callbackText, armSnippet,
        errorFloorSnippet: tail.slice(Math.max(0, errorPosition - 80), errorPosition + 180),
      }
      trace.status = 'fresh-native-error-floor-observed'
    }
  }
  const associated = trace.anchor
    ? trace.records.filter(record => record.callbackText === trace.anchor.callbackText
      && record.scheduledAtMs >= trace.anchor.scheduledAtMs)
    : []
  const active = associated.filter(record => !record.fired && !record.cancelled)
  const fired = associated.filter(record => record.fired)
  return {
    status: trace.status, anchor: trace.anchor ?? null, sourceWitness: trace.sourceWitness ?? null,
    recordCount: trace.records.length, associatedArmCount: associated.length,
    activeTimerCount: active.length, fired,
    earliestNominalDueMs: associated.length ? Math.min(...associated.map(record => record.scheduledAtMs + record.delayMs)) : null,
    elapsedSinceAnchorMs: trace.anchor ? performance.now() - trace.anchor.scheduledAtMs : null,
    lastRecords: trace.records.slice(-3),
    sourceEvents: trace.sourceEvents,
    foreground: await trace.originalInvoke('app_foreground'), visibility: document.visibilityState,
    observedAtUtc: new Date().toISOString(),
    scope: 'observe existing real timer trace; no scheduler query, clock change or timer restart',
  }
})()
