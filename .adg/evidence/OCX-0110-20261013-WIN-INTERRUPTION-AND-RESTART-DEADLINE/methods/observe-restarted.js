(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const pinia = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia
  if (!pinia) throw Error('Production stores unavailable')
  const prefs = await invoke('get_preferences')
  const manager = await invoke('get_update_status')
  const panelCache = await invoke('official_remote_cache')
  const updates = pinia._s.get('updates')
  const effects = pinia._s.get('effects')
  return {
    atUtc: new Date().toISOString(),
    currentProcessElapsedMs: performance.now(),
    route: location.hash,
    preferences: {
      channel: prefs.appUpdateChannel,
      automaticCheck: prefs.appUpdateAutoCheck,
      intervalSeconds: prefs.appUpdateCheckIntervalSeconds,
    },
    manager,
    panelCache,
    foreground: await invoke('app_foreground'),
    documentVisibility: document.visibilityState,
    updateStore: updates ? {
      panelLoading: updates.officialRemoteLoading,
      panelError: updates.officialRemoteError,
      panelRemote: JSON.parse(JSON.stringify(updates.officialRemote)),
      managerBusy: updates.appUpdateBusy,
      panelBusy: updates.officialUpdateBusy,
    } : null,
    effects: effects ? { foreground: effects.foreground, visible: effects.visible } : null,
    originalInMemoryTracePresent: !!window.__ocxRealStablePanelCycle,
    scope: 'read-only restarted process snapshot; no query/check/save, listener injection, routing, focus, clock or timer changes',
    originalContinuousLifecycleProven: false,
    automaticTriggerProven: false,
    rawNetworkOrQueryCountsProven: false,
    fullCyclesAccepted: false,
  }
})()
