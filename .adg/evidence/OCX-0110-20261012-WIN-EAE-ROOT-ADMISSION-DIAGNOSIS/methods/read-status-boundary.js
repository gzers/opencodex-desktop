(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const before = await invoke('get_data_root_config')
  const runtime = await invoke('runtime_source')
  const snapshot = await invoke('get_status_snapshot')
  const preferences = await invoke('get_preferences')
  const after = await invoke('get_data_root_config')
  return {
    runtime: { kind: runtime.kind, version: runtime.version },
    statusProjection: {
      matrix: snapshot.matrix, source: snapshot.source,
      hasProcessId: snapshot.pid !== null, hasPort: snapshot.port !== null,
      health: snapshot.facts?.health,
    },
    beforeRoot: before,
    afterRoot: after,
    rootConfigUnchanged: JSON.stringify(before) === JSON.stringify(after),
    preferences: { channel: preferences.appUpdateChannel, automaticCheck: preferences.appUpdateAutoCheck },
    dataRootSwitchInvoked: false,
    manualUpdateCheckInvoked: false,
    versionProbeInvoked: false,
    statusRefreshRequested: true,
    nativeCommandSuccessNotEquivalentToCollectorRefreshSuccess: true,
    scope: 'read-only root/source and real status refresh/projection; no attempted root transaction or inferred admission',
  }
})()
