(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const scene = window.__ocxEaePolicyFailureScene
  if (!scene) throw Error('Original policy journal absent')
  await invoke('save_preferences_backup_policy', { policy: scene.policy })
  const preferences = await invoke('get_preferences')
  const policy = await invoke('preferences_backup_policy')
  if (JSON.stringify(policy) !== JSON.stringify(scene.policy) || JSON.stringify(preferences) !== JSON.stringify(scene.preferences)) throw Error('Original policy/preference readback differs')
  return { originalPolicyRestored: true, policy, preferences,
    notifications: (await invoke('list_notifications')).items.map(({ id, resolved, resolvedAt }) => ({ id, resolved, resolvedAt })),
    inventory: await invoke('list_preferences_backups'), preview: await invoke('preview_preferences_backup_cleanup'),
    nativeBridgeUnchanged: invoke === window.__TAURI_INTERNALS__.invoke }
})()
