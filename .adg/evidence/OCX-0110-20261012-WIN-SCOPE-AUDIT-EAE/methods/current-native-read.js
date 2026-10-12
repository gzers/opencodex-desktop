(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const preferences = await invoke('get_preferences')
  const root = await invoke('get_data_root_config')
  const source = await invoke('runtime_source')
  return {
    application: await invoke('app_about'),
    root,
    runtime: {
      kind: source.kind,
      version: source.version,
      executable: source.executable,
      resolvedPath: source.resolvedPath,
      activeKind: source.activeKind,
    },
    preferences: {
      theme: preferences.theme,
      automaticCheck: preferences.appUpdateAutoCheck,
      channel: preferences.appUpdateChannel,
      autoPanel: preferences.autoPanel,
      launchWithCodex: preferences.launchWithCodex,
    },
    appearance: await invoke('window_appearance'),
    backupPolicy: await invoke('preferences_backup_policy'),
    foreground: await invoke('app_foreground'),
    route: location.hash,
    scope: 'read-only acceptance audit of existing lifecycle; no check, installation, backup mutation or route input',
  }
})()
