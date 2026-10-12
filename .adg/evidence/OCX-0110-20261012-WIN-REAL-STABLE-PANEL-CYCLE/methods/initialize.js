(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const pinia = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia
  const preferences = pinia?._s.get('preferences')
  const updates = pinia?._s.get('updates')
  if (!preferences || !updates) throw Error('Production stores unavailable')
  const original = await invoke('get_preferences')
  if (original.appUpdateChannel !== 'stable' || original.appUpdateAutoCheck) throw Error('Expected original manual stable mode')
  if (updates.officialRemoteLoading || updates.officialUpdateBusy || updates.appUpdateBusy) throw Error('Update domain not idle')
  const trace = {
    startedAtUtc: new Date().toISOString(), startedAtMs: performance.now(),
    originalChannel: original.appUpdateChannel, originalAutomaticCheck: original.appUpdateAutoCheck,
    events: [], status: 'preparing-real-cycle',
    businessResponsesInjected: false, clocksChanged: false, timersWrapped: false,
    manualQueryUsedOnlyAtInitialization: true, fullRealCycleAccepted: false,
  }
  window.__ocxRealStablePanelCycle = trace
  try {
    trace.managerQueryStartedAtUtc = new Date().toISOString()
    trace.managerQueryStartedAtMs = performance.now()
    trace.managerInitialQuery = await invoke('check_for_update', { trigger: 'user' })
    trace.managerQueryCompletedAtMs = performance.now()
    trace.managerQueryCompletedAtUtc = new Date().toISOString()
    if (!['up_to_date', 'available'].includes(trace.managerInitialQuery.status)
        || trace.managerInitialQuery.update.error) throw Error('Successful stable query baseline unavailable')
    trace.panelQueryStartedAtMs = performance.now()
    await updates.loadRemoteLatest('user')
    trace.panelQueryCompletedAtMs = performance.now()
    trace.panelQueryCompletedAtUtc = new Date().toISOString()
    if (updates.officialRemoteError || !updates.officialRemote) throw Error('Successful panel query baseline unavailable')
    trace.panelInitialRemote = JSON.parse(JSON.stringify(updates.officialRemote))
    await invoke('save_preferences', { preferences: { ...original, appUpdateAutoCheck: true } })
    await preferences.load()
    const actual = await invoke('get_preferences')
    if (!actual.appUpdateAutoCheck || actual.appUpdateChannel !== 'stable') throw Error('Automatic stable mode not committed')
    trace.enabledAtMs = performance.now()
    trace.enabledAtUtc = new Date().toISOString()
    trace.foregroundAtStart = await invoke('app_foreground')
    trace.status = 'observing-real-stable-and-panel-twenty-four-hour-deadlines'
    updates.$subscribe(() => {
      trace.events.push({
        atMs: performance.now(), atUtc: new Date().toISOString(),
        panelLoading: updates.officialRemoteLoading, panelError: updates.officialRemoteError,
        panelRemote: JSON.parse(JSON.stringify(updates.officialRemote)),
      })
    }, { flush: 'sync', detached: true })
  } catch (error) {
    trace.preparationError = String(error)
    await invoke('save_preferences', { preferences: original })
    await preferences.load()
    trace.status = 'preparation-failed-original-preferences-restored'
  }
  return trace
})()
