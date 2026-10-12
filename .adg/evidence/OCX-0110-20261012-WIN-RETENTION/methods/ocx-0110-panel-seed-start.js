(async () => {
  const internal = window.__TAURI_INTERNALS__
  const invoke = internal.invoke
  if (window.__ocxPanelSeedOperation) throw Error('Seed operation already exists; inspect it instead of restarting')
  const source = await invoke('runtime_source')
  if (source.kind !== 'unresolved') throw Error('Refusing to replace an existing runtime during seed installation')
  const operation = {
    purpose: 'official 2.82.0 baseline for a real 2.83.0 update in the isolated manager',
    requestedVersion: '2.82.0', startedAtUtc: new Date().toISOString(),
    state: 'running', events: [], originalSource: source, installerScriptsAllowed: false,
  }
  const handler = internal.transformCallback(event => {
    operation.events.push({ receivedAtUtc: new Date().toISOString(), payload: event.payload })
  }, false)
  const eventId = await invoke('plugin:event|listen', {
    event: 'runtime-install-progress', target: { kind: 'Any' }, handler,
  })
  window.__ocxPanelSeedOperation = operation
  window.__ocxPanelSeedPromise = invoke('install_runtime', { request: {
    prefix: null, source: 'registry', version: '2.82.0', offlinePath: null,
    proxyScheme: null, proxyHost: null, proxyUsername: null, proxySecret: null,
    allowScripts: false,
  } }).then(value => { operation.state = 'succeeded'; operation.result = value })
    .catch(error => { operation.state = 'failed'; operation.error = error })
    .finally(async () => {
      operation.finishedAtUtc = new Date().toISOString()
      await invoke('plugin:event|unlisten', { event: 'runtime-install-progress', eventId })
      internal.unregisterCallback(handler)
      operation.listenerRemoved = true
    })
  return { started: true, requestedVersion: operation.requestedVersion, startedAtUtc: operation.startedAtUtc,
    scope: 'one real managed install in this sole isolated GUI; no global npm installation' }
})()
