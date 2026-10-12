(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const pinia = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia
  if (!pinia) throw Error('Merged candidate has no mounted preference stores')
  const preferences = await invoke('get_preferences')
  if (preferences.themeNeedsImport !== false) {
    throw Error('Explicit schema-2 appearance still incorrectly requests legacy theme import')
  }
  const store = pinia._s.get('preferences')
  for (let attempt = 0; attempt < 100 && (!store?.data || store.loading); attempt++) {
    await new Promise(resolve => setTimeout(resolve, 50))
  }
  if (!store?.data || store.error) throw Error('Native preferences have not loaded successfully')
  const notificationsBefore = await invoke('list_notifications')
  const snapshotBefore = JSON.parse(JSON.stringify(store.data))
  await store.load()
  const reloaded = await invoke('get_preferences')
  if (reloaded.themeNeedsImport !== false
      || reloaded.theme !== preferences.theme
      || store.data.theme !== preferences.theme
      || store.error) {
    throw Error('Explicit native preference reload altered the stored theme')
  }
  const backups = await invoke('list_preferences_backups')
  const originalPin = backups.find(backup => backup.id === 'bk_20261012044452_a921f503')
  if (!originalPin?.pinned) throw Error('Original user pin was lost across merged candidate startup')
  const cacheTheme = localStorage.getItem('ocx-theme')
  if (cacheTheme !== preferences.theme) throw Error('Native explicit theme is not the cache projection')
  return {
    preferences, snapshotBefore, reloaded, storeData: JSON.parse(JSON.stringify(store.data)),
    storeError: store.error, themeCache: cacheTheme,
    notificationsBefore, notificationsAfter: await invoke('list_notifications'),
    roots: await invoke('get_data_root_config'), originalPin, backupCount: backups.length,
    foreground: await invoke('app_foreground'), dpr: devicePixelRatio,
    scope: 'exact merged production WebView native DTO and store reload; filesystem hashes checked separately; no legacy document or script mutation',
  }
})()
