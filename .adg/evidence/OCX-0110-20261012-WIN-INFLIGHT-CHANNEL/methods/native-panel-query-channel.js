(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const pinia = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia
  const updates = pinia?._s.get('updates')
  const preferences = pinia?._s.get('preferences')
  if (!updates || !preferences) throw Error('Production stores are not mounted')
  const initialPreferences = await invoke('get_preferences')
  if (initialPreferences.appUpdateChannel !== 'stable' || initialPreferences.appUpdateAutoCheck) {
    throw Error('Expected stable channel and disabled automatic checks')
  }
  if (updates.officialRemoteLoading || updates.officialUpdateBusy) throw Error('Existing panel operation')
  const trace = {
    startedAtUtc: new Date().toISOString(),
    nativeInvokeUnchanged: true,
    originalChannel: initialPreferences.appUpdateChannel,
    cacheBefore: await invoke('official_remote_cache'),
    transitions: [],
    snapshots: [],
    source: 'production Pinia loadRemoteLatest and native set_update_channel; actual npm metadata source; no response or clock injection',
    cleanup: null,
  }
  const state = () => ({
    atMs: performance.now(),
    atUtc: new Date().toISOString(),
    loading: updates.officialRemoteLoading,
    error: updates.officialRemoteError,
    remote: JSON.parse(JSON.stringify(updates.officialRemote)),
  })
  const unsubscribe = updates.$subscribe(() => trace.transitions.push(state()), { flush: 'sync', detached: true })
  let finished = false
  let query
  try {
    trace.queryStartedAtMs = performance.now()
    query = updates.loadRemoteLatest('user').finally(() => {
      finished = true
      trace.queryCompletedAtMs = performance.now()
      trace.queryFinalState = state()
    })
    await new Promise(resolve => setTimeout(resolve, 100))
    trace.queryStillPendingBeforeSwitch = !finished && updates.officialRemoteLoading
    if (!trace.queryStillPendingBeforeSwitch) throw Error('No observed overlap; do not count in-flight acceptance')
    trace.switchStartedAtMs = performance.now()
    trace.switched = await invoke('set_update_channel', { channel: 'beta' })
    trace.switchCompletedAtMs = performance.now()
    trace.queryStillPendingAfterSwitch = !finished && updates.officialRemoteLoading
    const switchedPreferences = await invoke('get_preferences')
    trace.snapshots.push({
      phase: 'native beta switch completed',
      ...state(),
      nativeCache: await invoke('official_remote_cache'),
      nativePreferences: {
        channel: switchedPreferences.appUpdateChannel,
        automaticCheck: switchedPreferences.appUpdateAutoCheck,
      },
    })
    await query
    trace.cacheAfterQuery = await invoke('official_remote_cache')
    trace.nativeManagerAfterQuery = await invoke('get_update_status')
  } catch (error) {
    trace.observationError = String(error)
    if (query) await query
  } finally {
    try {
      trace.restoredStatus = await invoke('set_update_channel', { channel: initialPreferences.appUpdateChannel })
      await preferences.load()
      const after = await invoke('get_preferences')
      trace.cleanup = {
        originalChannelRestored: after.appUpdateChannel === initialPreferences.appUpdateChannel,
        automaticCheckStillDisabled: after.appUpdateAutoCheck === false,
        nativeInvokeUnchanged: window.__TAURI_INTERNALS__.invoke === invoke,
        preferencesStoreChannel: preferences.data?.appUpdateChannel,
        preferenceStoreError: preferences.error,
      }
    } finally {
      unsubscribe()
    }
  }
  trace.finishedAtUtc = new Date().toISOString()
  trace.scope = 'one actual in-flight panel metadata query across stable to beta; no installer, manager query, panel installation or full event matrix'
  return trace
})()
