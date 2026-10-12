(async () => {
  if (window.__ocxTimerBackoffTrace) throw Error('Existing timer trace must be observed instead of replaced')
  const internal = window.__TAURI_INTERNALS__
  const invoke = internal.invoke
  const originalSetTimeout = window.setTimeout
  const originalClearTimeout = window.clearTimeout
  const now = () => performance.now()
  if (!await invoke('app_foreground') || document.visibilityState !== 'visible') throw Error('Real foreground window required')
  const originalPreferences = await invoke('get_preferences')
  if (originalPreferences.appUpdateAutoCheck) throw Error('Expected the retained test preferences to be manual')
  const descriptor = Object.getOwnPropertyDescriptor(window, 'setTimeout')
  if (!descriptor?.writable) throw Error('Timer boundary is not writable; no fixture will be changed')
  const scripts = [...document.querySelectorAll('script[src]')].map(script => script.src)
  const sources = []
  for (const url of scripts) {
    if (new URL(url).origin !== location.origin) throw Error('Production source must be from this same native app')
    const response = await fetch(url)
    if (!response.ok) throw Error('Own production script unavailable')
    const text = await response.text()
    const hash = [...new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(text)))]
      .map(byte => byte.toString(16).padStart(2, '0')).join('')
    sources.push({ url, text, sha256: hash })
  }
  const trace = {
    startedAtUtc: new Date().toISOString(), originalSetTimeout, originalClearTimeout, originalPreferences,
    originalInvoke: invoke, records: [], byHandle: new Map(), sources,
    status: 'installed-no-fixture', clockUnchanged: true, nativeBridgeUnchanged: true, sourceEvents: 0,
  }
  function setTimeoutObserved(callback, delay, ...args) {
    const numericDelay = Number(delay ?? 0)
    let record
    if (typeof callback === 'function' && numericDelay >= 60_000) {
      record = {
        id: trace.records.length, scheduledAtMs: now(), scheduledAtUtc: new Date().toISOString(),
        delayMs: numericDelay, callbackText: callback.toString(), stack: new Error().stack,
        cancelled: false, fired: false,
      }
      trace.records.push(record)
    }
    const delegate = record ? function(...callbackArgs) {
      record.fired = true; record.firedAtMs = now(); record.firedAtUtc = new Date().toISOString()
      return callback.apply(this, callbackArgs)
    } : callback
    const handle = originalSetTimeout.call(window, delegate, delay, ...args)
    if (record) { record.handle = handle; trace.byHandle.set(handle, record) }
    return handle
  }
  function clearTimeoutObserved(handle) {
    const record = trace.byHandle.get(handle)
    if (record && !record.fired) {
      record.cancelled = true; record.cancelledAtMs = now()
    }
    return originalClearTimeout.call(window, handle)
  }
  window.setTimeout = setTimeoutObserved
  window.clearTimeout = clearTimeoutObserved
  if (window.setTimeout !== setTimeoutObserved || window.clearTimeout !== clearTimeoutObserved
      || internal.invoke !== invoke) {
    window.setTimeout = originalSetTimeout; window.clearTimeout = originalClearTimeout
    throw Error('Transparent timer recorder did not install; fixture remains untouched')
  }
  window.__ocxTimerBackoffTrace = trace
  return {
    status: trace.status, recorderInstalled: true, nativeBridgeUnchanged: true,
    clockUnchanged: true, originalAutoCheck: originalPreferences.appUpdateAutoCheck,
    ownScriptSources: sources.map(source => ({ url: source.url, sha256: source.sha256 })),
    scope: 'observe real timer arm/cancel/fire with unchanged delays and callbacks; no Tauri-property override, no response injection',
  }
})()
