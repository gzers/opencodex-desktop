(async () => {
  const table = document.querySelector('.ui-tree-table')
  const section = table?.closest('.backup-files')
  if (!table || !section) throw Error('Actual backup tree is not available')
  if (window.__ocxTreeNativeRefreshProbe) throw Error('Existing native refresh observer must be inspected')
  const button = section.querySelector('.backup-files-head button')
  if (!button) throw Error('Production refresh button missing')
  const expanded = () => [...table.querySelectorAll('.tree-toggle[aria-expanded=true]')].map(control => control.getAttribute('aria-label'))
  const probe = {
    priorExpanded: expanded(), focusBefore: document.activeElement?.getAttribute('aria-label'),
    startedAtUtc: new Date().toISOString(), clicked: false, status: 'armed', transitions: [],
    scope: 'observe actual user click and aria-busy lifecycle; never invokes refresh, synthetic click or focus',
  }
  let finish
  probe.refreshPromise = new Promise(resolve => { finish = resolve })
  const observer = new MutationObserver(() => {
    const busy = table.getAttribute('aria-busy') === 'true'
    probe.transitions.push({ busy, at: performance.now() })
    if (busy) probe.sawBusy = true
    if (probe.clicked && probe.sawBusy && !busy) {
      probe.focusAfter = document.activeElement?.textContent
      probe.focusAfterLabel = document.activeElement?.getAttribute('aria-label')
      probe.afterExpanded = expanded()
      probe.status = 'finished'
      probe.finishedAtUtc = new Date().toISOString()
      observer.disconnect()
      finish()
    }
  })
  observer.observe(table, { attributes: true, attributeFilter: ['aria-busy'] })
  const listener = () => {
    probe.clicked = true
    probe.focusAtClick = document.activeElement?.textContent
  }
  button.addEventListener('click', listener, { once: true })
  window.__ocxTreeNativeRefreshProbe = probe
  return { status: probe.status, focusBefore: probe.focusBefore, priorExpanded: probe.priorExpanded, listenerBoundToActualRefresh: true }
})()
