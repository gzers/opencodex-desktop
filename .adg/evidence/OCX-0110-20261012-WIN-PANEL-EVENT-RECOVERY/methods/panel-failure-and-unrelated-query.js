(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const updates = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia?._s.get('updates')
  if (!updates || updates.officialRemoteLoading || updates.officialUpdateBusy) throw Error('Panel store not idle')
  const notifications = async () => {
    const value = await invoke('list_notifications')
    return { aggregate: value.aggregate, items: value.items.map(({ detail, ...item }) => item) }
  }
  const before = await notifications()
  const remoteBefore = JSON.parse(JSON.stringify(updates.officialRemote))
  await updates.loadRemoteLatest('user')
  const afterFailure = await notifications()
  const failureState = {
    error: updates.officialRemoteError,
    loading: updates.officialRemoteLoading,
    remote: JSON.parse(JSON.stringify(updates.officialRemote)),
    cache: await invoke('official_remote_cache'),
  }
  let manager
  try {
    manager = await invoke('check_for_update', { trigger: 'user' })
  } catch (error) {
    manager = { invocationError: String(error) }
  }
  return {
    before, remoteBefore, afterFailure, failureState, unrelatedManagerQuery: manager,
    afterUnrelatedManagerQuery: await notifications(),
    nativeInvokeUnchanged: invoke === window.__TAURI_INTERNALS__.invoke,
    scope: 'real npm query failure using temporary isolated HOME registry config; manager check is actual source query; no package lifecycle scripts or event injection',
  }
})()
