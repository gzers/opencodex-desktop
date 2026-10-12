(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const context = window.__ocxEaeRestoreEventContext
  if (!context?.id) throw Error('Expected prepared final-source backup')
  const notes = async () => (await invoke('list_notifications')).items
    .map(({ id, resolved, resolvedAt }) => ({ id, resolved, resolvedAt }))
  const beforePreferences = await invoke('get_preferences')
  let rejected = false
  try { await invoke('restore_preferences_backup', { id: context.id }) }
  catch { rejected = true }
  if (!rejected) throw Error('Temporarily damaged payload was not rejected')
  const afterFailure = await notes()
  const afterFailurePreferences = await invoke('get_preferences')
  if (JSON.stringify(beforePreferences) !== JSON.stringify(afterFailurePreferences)) {
    throw Error('Rejected restore altered active preferences')
  }
  // This is a real successful Backup action, not another restore candidate.
  const unrelated = await invoke('create_preferences_backup')
  if (unrelated.cleanupError || unrelated.removedIds.length) throw Error('Unrelated native backup failed or pruned records')
  return {
    backupId: context.id, rejected,
    failedRestorePreferenceDtoUnchanged: true,
    afterFailure,
    successfulOtherAction: unrelated,
    afterSuccessfulOtherAction: await notes(),
    scope: 'real Restore failure then real Backup success; no alternate restore candidate and no injected/manual notification resolution',
  }
})()
