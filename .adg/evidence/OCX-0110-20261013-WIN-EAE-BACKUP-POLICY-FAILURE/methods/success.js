(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const scene = window.__ocxEaePolicyFailureScene
  if (!scene || (await invoke('preferences_backup_policy')).mode !== 'automatic') throw Error('Exact automatic policy not restored')
  const ids = scene.created.map(item => item.backup.backupId)
  const created = await invoke('create_preferences_backup')
  if (created.cleanupError || JSON.stringify([...created.removedIds].sort()) !== JSON.stringify([...ids].sort())) throw Error('Successful automatic rotation differs from owned candidates')
  const inventory = await invoke('list_preferences_backups')
  const oldInventory = inventory.filter(item => item.id !== created.backup.backupId)
  if (JSON.stringify(oldInventory) !== JSON.stringify(scene.inventory)) throw Error('Preexisting DTO inventory changed')
  const notifications = (await invoke('list_notifications')).items.map(({ id, resolved, resolvedAt }) => ({ id, resolved, resolvedAt }))
  const samePreferences = JSON.stringify(await invoke('get_preferences')) === JSON.stringify(scene.preferences)
  if (!samePreferences) throw Error('Same native preference payload changed')
  return { created, inventory, notifications, preexistingInventoryExactlyPreserved: true,
    preferencesDtoExactlyPreserved: true, finalPreview: await invoke('preview_preferences_backup_cleanup'),
    scope: 'real success after exact policy repair, same unchanged preference payload; no manual event resolution' }
})()
