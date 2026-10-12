(() => {
  const trace = window.__ocxTimerBackoffTrace
  if (!trace) throw Error('Retained verified recorder is absent')
  const record = trace.records[0]
  if (!record) throw Error('Actual prior-floor timer has not been recorded')
  const source = trace.sources.find(item => item.text.includes(record.callbackText))
  if (!source) throw Error('Timer callback is absent from the running native app source')
  const positions = []
  let index = source.text.indexOf(record.callbackText)
  while (index >= 0) {
    const tail = source.text.slice(index, index + 3500)
    const error = tail.search(/performance\.now\(\)\s*\+\s*(?:18e5|1800000)/)
    const arm = source.text.slice(Math.max(0, index - 160), index + record.callbackText.length + 160)
    positions.push({
      offset: index, arm, errorWithin3500Characters: error,
      errorSnippet: error >= 0 ? tail.slice(Math.max(0, error - 80), error + 180) : null,
    })
    index = source.text.indexOf(record.callbackText, index + record.callbackText.length)
  }
  return {
    record: { id: record.id, delayMs: record.delayMs, scheduledAtUtc: record.scheduledAtUtc, callbackText: record.callbackText },
    source: { url: source.url, sha256: source.sha256 }, matches: positions,
    timerBridgeStillInstalled: window.setTimeout !== trace.originalSetTimeout,
    nativeInvokeUnchanged: window.__TAURI_INTERNALS__.invoke === trace.originalInvoke,
    scope: 'read-only attribution diagnostic before waiting for a fresh error; no timer restarts or source modification',
  }
})()
