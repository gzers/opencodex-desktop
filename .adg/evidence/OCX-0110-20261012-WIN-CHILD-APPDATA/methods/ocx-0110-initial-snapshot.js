(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const pinia = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia
  return {
    about: await invoke('app_about'),
    roots: await invoke('get_data_root_config'),
    discovery: await invoke('discover_environment', { request: null }),
    foreground: await invoke('app_foreground'),
    appearance: await invoke('window_appearance'),
    preferences: await invoke('get_preferences'),
    updateStatus: await invoke('get_update_status'),
    schedule: await invoke('update_schedule_plan'),
    bodyText: document.body.innerText,
    dpr: devicePixelRatio, visibility: document.visibilityState,
    focus: document.hasFocus(), stores: pinia ? [...pinia._s.keys()] : [],
    effects: pinia?._s.get('effects')?.strategy,
    treeTables: document.querySelectorAll('.ui-tree-table').length,
  }
})()
