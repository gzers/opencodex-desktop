(async () => {
  const table = document.querySelector('.ui-tree-table')
  if (!table) throw Error('Production backup tree is not mounted')
  const active = document.activeElement
  const rows = [...table.querySelectorAll('tbody tr')].map(row => ({
    label: row.querySelector('.tree-label')?.textContent,
    expanded: row.querySelector('.tree-toggle')?.getAttribute('aria-expanded') ?? null,
    controls: [...row.querySelectorAll('button,input')].map(control => ({
      tag: control.tagName, label: control.getAttribute('aria-label'), text: control.textContent,
      checked: control.getAttribute('aria-checked'), disabled: control.disabled,
    })),
  }))
  return {
    activeElement: { tag: active?.tagName, label: active?.getAttribute('aria-label'),
      text: active?.textContent, inTree: table.contains(active) },
    rows, rowCount: rows.length, route: location.hash,
    foreground: await window.__TAURI_INTERNALS__.invoke('app_foreground'),
    dpr: devicePixelRatio, observedAtUtc: new Date().toISOString(),
    scope: 'read-only actual DOM after native input, no programmatic focus or keyboard dispatch',
  }
})()
