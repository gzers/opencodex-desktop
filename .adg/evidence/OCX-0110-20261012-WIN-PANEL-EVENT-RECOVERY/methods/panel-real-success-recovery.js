(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const updates = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia?._s.get('updates')
  if (!updates || updates.officialRemoteLoading || updates.officialUpdateBusy) throw Error('Panel store not idle')
  await updates.loadRemoteLatest('user')
  const value = await invoke('list_notifications')
  const preferences = await invoke('get_preferences')
  return {
    error: updates.officialRemoteError,
    loading: updates.officialRemoteLoading,
    remote: JSON.parse(JSON.stringify(updates.officialRemote)),
    cache: await invoke('official_remote_cache'),
    notifications: { aggregate: value.aggregate, items: value.items.map(({ detail, ...item }) => item) },
    nativeInvokeUnchanged: invoke === window.__TAURI_INTERNALS__.invoke,
    preferences: { channel: preferences.appUpdateChannel, automaticCheck: preferences.appUpdateAutoCheck },
    scope: 'actual panel metadata success after removal of controlled local registry failure; no notification mutation command or response injection',
  }
})()
