(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const scene = window.__ocxEaeW2Lock
  if (!scene?.preparedPhase || scene.operation) throw Error('Exactly one prepared phase required')
  const phase = scene.preparedPhase
  const operations = {
    create: ['create_preferences_backup', undefined],
    list: ['list_preferences_backups', undefined],
    pin: ['set_preferences_backup_pinned', { id: scene.sourceBackup, pinned: true }],
    policy: ['save_preferences_backup_policy', { policy: scene.policy }],
    restore: ['restore_preferences_backup', { id: scene.sourceBackup }],
    cleanup: ['cleanup_preferences_backups', { preview: window.__ocxEaeW2CleanupPreview }],
  }
  const [command, args] = operations[phase]
  if (phase === 'cleanup' && !args.preview) throw Error('Actual owned cleanup preview absent')
  const operation = { phase, command, status: 'pending', startedAtUtc: new Date().toISOString(),
    startedAtMs: performance.now(), finishedAtMs: null }
  scene.operation = operation
  try {
    operation.result = await invoke(command, args)
    operation.status = 'completed'
  } catch (error) {
    operation.error = error
    operation.status = 'rejected'
  } finally {
    operation.finishedAtMs = performance.now()
    operation.finishedAtUtc = new Date().toISOString()
    scene.records.push(structuredClone(operation))
    scene.preparedPhase = null
    scene.phaseIndex++
  }
  return { operation, nativeBridgeUnchanged: invoke === window.__TAURI_INTERNALS__.invoke,
    scope: 'actual one native command, transparent pending/result record, no mocked IPC or UI inputs' }
})()
