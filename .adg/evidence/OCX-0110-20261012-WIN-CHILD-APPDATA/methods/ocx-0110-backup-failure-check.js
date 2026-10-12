(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const before = await invoke('list_notifications')
  let result
  try {
    await invoke('create_preferences_backup')
    throw Error('Invalid saved policy did not block backup')
  } catch (error) {
    if (String(error).includes('did not block')) throw error
    result = { rejected: true, error }
  }
  const failed = await invoke('list_notifications')
  const failure = failed.items.find(item => item.id.startsWith('preferences-backup-failed') && !item.resolved)
  if (!failure) throw Error('Native failure event missing')
  const check = await invoke('check_for_update', { trigger: 'user' })
  const afterCheck = await invoke('list_notifications')
  const same = afterCheck.items.find(item => item.id === failure.id)
  if (!same || same.resolved) throw Error('Manager update query cleared backup failure')
  return { before, result, failed, check, afterCheck, failureId: failure.id,
    scope: 'native command failure delivery and unrelated update query isolation; no UI input' }
})()
