(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const before = await invoke('list_preferences_backups')
  const manual = before.filter(item => item.action === 'manual-preferences')
  if (manual.length > 14) throw Error('Unexpected existing test backup population')
  const created = []
  for (let index = manual.length; index < 14; index++) {
    created.push(await invoke('create_preferences_backup'))
  }
  const after = await invoke('list_preferences_backups')
  const afterManual = after.filter(item => item.action === 'manual-preferences')
  if (afterManual.length !== 14) throw Error('Recent native backups were unexpectedly rotated')
  const preview = await invoke('preview_preferences_backup_cleanup')
  if (preview.candidateIds.length) throw Error('Recent backups violate N10 OR D30 retention')
  return { before, created, after, preview, recentManualCount: afterManual.length,
    scope: 'real native backup creation and preview; all newly created records within D30' }
})()
