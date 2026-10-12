(async () => {
  const probe = window.__ocxTreeNativeRefreshProbe
  if (!probe?.clicked) throw Error('Actual native refresh click has not occurred')
  await Promise.race([probe.refreshPromise, new Promise((_, reject) => setTimeout(() => reject(Error('Native refresh lifecycle deadline')), 5000))])
  if (probe.status !== 'finished') throw Error('Production refresh did not finish')
  if (JSON.stringify(probe.priorExpanded) !== JSON.stringify(probe.afterExpanded)) throw Error('Production refresh reset user expansion')
  if (probe.focusAfter.trim() !== '刷新') throw Error('External refresh button lost focus after actual async refresh')
  return {
    clicked: true, status: probe.status, startedAtUtc: probe.startedAtUtc, finishedAtUtc: probe.finishedAtUtc,
    priorExpanded: probe.priorExpanded, afterExpanded: probe.afterExpanded,
    focusAtClick: probe.focusAtClick, focusAfter: probe.focusAfter,
    sawBusy: probe.sawBusy, transitions: probe.transitions,
    foreground: await window.__TAURI_INTERNALS__.invoke('app_foreground'),
    scope: probe.scope,
  }
})()
