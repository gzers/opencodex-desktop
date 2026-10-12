(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const trace = window.__ocxRealStablePanelCycle
  if (!trace) throw Error('Real cycle marker absent; do not restart automatically')
  const preferences = await invoke('get_preferences')
  const effects = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia?._s.get('effects')
  return {
    atUtc: new Date().toISOString(), atMs: performance.now(),
    manager: await invoke('get_update_status'), panelCache: await invoke('official_remote_cache'),
    preferences: { channel: preferences.appUpdateChannel, automaticCheck: preferences.appUpdateAutoCheck },
    foreground: await invoke('app_foreground'), documentVisibility: document.visibilityState,
    effects: effects ? { foreground: effects.foreground, visible: effects.visible } : null,
    trace: { status: trace.status, startedAtUtc: trace.startedAtUtc, enabledAtUtc: trace.enabledAtUtc,
      managerQueryStartedAtMs: trace.managerQueryStartedAtMs,
      managerQueryCompletedAtMs: trace.managerQueryCompletedAtMs,
      panelQueryStartedAtMs: trace.panelQueryStartedAtMs, panelQueryCompletedAtMs: trace.panelQueryCompletedAtMs,
      events: trace.events },
    scope: 'read-only scheduler observation; no manual check, deadline mutation, route event or input',
  }
})()
