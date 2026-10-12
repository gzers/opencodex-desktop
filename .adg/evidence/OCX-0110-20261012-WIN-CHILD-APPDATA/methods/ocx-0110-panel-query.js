(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const before = await invoke('official_remote_cache')
  let query
  try { query = { ok: true, value: await invoke('official_remote_latest', { trigger: 'user' }) } }
  catch (error) { query = { ok: false, error } }
  return { before, query, cache: await invoke('official_remote_cache'), schedule: await invoke('update_schedule_plan'),
    discovery: await invoke('discover_environment', { request: null }), notifications: await invoke('list_notifications') }
})()
