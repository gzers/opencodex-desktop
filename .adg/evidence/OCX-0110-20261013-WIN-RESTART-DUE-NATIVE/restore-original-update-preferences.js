(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const pinia = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia
  const preferences = pinia?._s.get('preferences')
  if (!preferences) throw Error('Production preference store is unavailable')
  const actual = await invoke('get_preferences')
  const status = await invoke('get_update_status')
  const updates = pinia?._s.get('updates')
  if (actual.appUpdateChannel !== 'stable' || !actual.appUpdateAutoCheck ||
      status.pendingRestart || updates?.appUpdateBusy || updates?.officialUpdateBusy ||
      updates?.officialRemoteLoading) {
    throw Error('Restore only the observed idle stable automatic session')
  }
  // Original Oct12 initialization recorded stable/manual; preserve every other
  // current preference and the successful next_due records.
  await invoke('save_preferences', { preferences: { ...actual,
    appUpdateChannel: 'stable', appUpdateAutoCheck: false } })
  await preferences.load()
  const after = await invoke('get_preferences')
  return {
    atUtc: new Date().toISOString(), originalPair: { channel: 'stable', automaticCheck: false },
    actualPair: { channel: after.appUpdateChannel, automaticCheck: after.appUpdateAutoCheck },
    restored: after.appUpdateChannel === 'stable' && !after.appUpdateAutoCheck,
    scope: 'owned test cleanup after observed due terminal; restore original update preference pair only',
    explicitUpdateQueryIssued: false, channelSwitched: false, scheduleDeadlineRewritten: false,
    applicationStopped: false, continuous24hLifecycleAccepted: false, fullCyclesAccepted: false,
  }
})()
