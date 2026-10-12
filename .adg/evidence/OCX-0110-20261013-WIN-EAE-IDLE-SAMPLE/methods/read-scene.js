(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const pinia = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia
  const updates = pinia?._s.get('updates')
  const roots = pinia?._s.get('data-root')
  const effects = pinia?._s.get('effects')
  const preferences = await invoke('get_preferences')
  return {
    atUtc: new Date().toISOString(), atMs: performance.now(), route: location.hash,
    foreground: await invoke('app_foreground'), documentVisibility: document.visibilityState,
    effects: { foreground: effects?.foreground, visible: effects?.visible, setting: effects?.setting },
    busy: { manager: updates?.appUpdateBusy, panel: updates?.officialUpdateBusy, root: roots?.switching },
    preferences: {
      channel: preferences.appUpdateChannel, automaticCheck: preferences.appUpdateAutoCheck,
      theme: preferences.theme, visualEffects: preferences.visualEffects,
      glowRender: preferences.glowRender, interfaceScale: preferences.interfaceScale,
      autoPanel: preferences.autoPanel, launchWithCodex: preferences.launchWithCodex,
    },
    scope: 'read-only native/store scene observation; no check, route, lifecycle, timer or preference mutation',
  }
})()
