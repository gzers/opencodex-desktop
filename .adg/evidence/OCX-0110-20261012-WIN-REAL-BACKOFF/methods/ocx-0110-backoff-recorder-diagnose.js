(() => {
  const invoke = Object.getOwnPropertyDescriptor(window.__TAURI_INTERNALS__, 'invoke')
  const timeout = Object.getOwnPropertyDescriptor(window, 'setTimeout')
  const probe = window.__ocxErrorBackoffProbe
  return {
    invokeDescriptor: { writable: invoke?.writable, configurable: invoke?.configurable, enumerable: invoke?.enumerable },
    setTimeoutDescriptor: { writable: timeout?.writable, configurable: timeout?.configurable },
    originalInvokeRestored: probe ? window.__TAURI_INTERNALS__.invoke === probe.originalInvoke : null,
    recorderActuallyInstalled: probe ? window.__TAURI_INTERNALS__.invoke !== probe.originalInvoke : null,
    retainedProbe: probe ? { status: probe.status, startedAtUtc: probe.startedAtUtc, planCalls: probe.planCalls, queries: probe.queries } : null,
    scope: 'read-only diagnostic of failed recorder installation; no security property override or timer change',
  }
})()
