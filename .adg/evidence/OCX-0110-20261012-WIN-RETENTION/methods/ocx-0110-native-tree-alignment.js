(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const pinia = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia
  pinia._s.get('routes').go('settings', { section: 'backup' })
  let table
  for (let attempt = 0; attempt < 100; attempt++) {
    await new Promise(resolve => setTimeout(resolve, 50))
    table = document.querySelector('.ui-tree-table')
    if (table?.querySelectorAll('tbody .tree-name').length >= 3
        && !table.hasAttribute('aria-busy')) break
  }
  if (!table || table.querySelectorAll('tbody .tree-name').length < 3) throw Error('Real backup tree did not load')
  table.scrollIntoView({ block: 'center', inline: 'nearest' })
  await new Promise(resolve => setTimeout(resolve, 100))
  const rows = [...table.querySelectorAll('tbody tr')].map(row => {
    const label = row.querySelector('.tree-label')
    const lineCenter = label.getBoundingClientRect().top + parseFloat(getComputedStyle(label).lineHeight) / 2
    const controls = [...row.querySelectorAll('.tree-toggle,.tree-icon,.tree-check,td:last-child button')].map(control => {
      const rectangle = control.getBoundingClientRect()
      return { className: control.className, label: control.getAttribute('aria-label'),
        height: rectangle.height, center: rectangle.top + rectangle.height / 2,
        offsetFromFirstLine: rectangle.top + rectangle.height / 2 - lineCenter }
    })
    const purpose = row.querySelector('.tree-cell:not(.tree-cell-action)')
    const purposeCenter = purpose.getBoundingClientRect().top
      + parseFloat(getComputedStyle(purpose).paddingTop) + parseFloat(getComputedStyle(purpose).lineHeight) / 2
    if (controls.some(control => Math.abs(control.offsetFromFirstLine) > 1)
        || Math.abs(purposeCenter - lineCenter) > 1) throw Error('Actual WebView first-line alignment failed')
    if (controls.filter(control => control.className.includes('btn')).some(control => control.height < 34)) {
      throw Error('Actual action hit height was reduced')
    }
    return { label: label.textContent, lineCenter, purposeCenter, controls }
  })
  const backups = await invoke('list_preferences_backups')
  const originalPin = backups.find(backup => backup.id === 'bk_20261012044452_a921f503')
  if (!originalPin?.pinned) throw Error('Pinned record was lost across candidate restart')
  return { rows, dpr: devicePixelRatio, documentVisibility: document.visibilityState,
    foreground: await invoke('app_foreground'), theme: document.documentElement.dataset.theme,
    pinSurvivedRestart: true, backupCount: backups.length,
    scope: 'production backup tree in exact candidate WebView; physical DPI recorded separately; no keyboard/foreground acceptance' }
})()
