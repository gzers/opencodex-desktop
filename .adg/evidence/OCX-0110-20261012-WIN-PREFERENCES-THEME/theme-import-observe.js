(async () => {
  const preferences = await window.__TAURI_INTERNALS__.invoke('get_preferences')
  return {
    theme: preferences.theme, themeNeedsImport: preferences.themeNeedsImport,
    schemaVersion: preferences.schemaVersion, scope: 'read-only actual native preference DTO'
  }
})()
