(async () => {
  const table = document.querySelector('.ui-tree-table')
  if (!table || table.getAttribute('aria-busy') === 'true') throw Error('Actual production tree not ready')
  const rows = [...table.querySelectorAll('tbody tr')].map(row => {
    const label = row.querySelector('.tree-label')
    if (!label) throw Error('Tree label missing')
    const rectangle = label.getBoundingClientRect()
    const labelCenter = rectangle.top + parseFloat(getComputedStyle(label).lineHeight) / 2
    const controls = [...row.querySelectorAll('.tree-toggle,.tree-icon,.tree-check,td:last-child button')].map(control => {
      const r = control.getBoundingClientRect()
      return {
        className: control.className, label: control.getAttribute('aria-label'),
        height: r.height, firstLineOffset: r.top + r.height / 2 - labelCenter,
      }
    })
    const purpose = row.querySelector('.tree-cell:not(.tree-cell-action)')
    if (!purpose) throw Error('Purpose column missing')
    const p = purpose.getBoundingClientRect()
    const purposeCenter = p.top + parseFloat(getComputedStyle(purpose).paddingTop)
      + parseFloat(getComputedStyle(purpose).lineHeight) / 2
    return {
      label: label.textContent, labelCenter, purposeCenter,
      purposeFirstLineOffset: purposeCenter - labelCenter,
      labelHeight: rectangle.height, controls,
    }
  })
  const pin = (await window.__TAURI_INTERNALS__.invoke('list_preferences_backups'))
    .find(item => item.id === 'bk_20261012044452_a921f503')
  return {
    route: location.hash, rows, rowCount: rows.length,
    checkControlCount: table.querySelectorAll('.tree-check').length,
    dpr: devicePixelRatio, theme: document.documentElement.dataset.theme,
    documentVisibility: document.visibilityState,
    foreground: await window.__TAURI_INTERNALS__.invoke('app_foreground'),
    originalPinRetained: !!pin?.pinned,
    programmaticFocusOrRouteOrScrollPerformed: false,
    scope: 'read-only actual production geometry after real input; no emulation, route/focus/scroll changes or checkbox matrix claim',
  }
})()
