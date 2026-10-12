(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const backups = await invoke('list_preferences_backups')
  const files = await invoke('backup_files')
  const table = document.querySelector('.ui-tree-table') ?? document.querySelector('table[aria-label="备份文件与目录"]')
  const geometry = table ? [...table.querySelectorAll('tr')].map(row => ({
    text: row.innerText,
    rectangle: row.getBoundingClientRect().toJSON(),
    controls: [...row.querySelectorAll('button,input,svg')].map(control => ({
      tag: control.tagName, label: control.getAttribute('aria-label') ?? control.textContent,
      expanded: control.getAttribute('aria-expanded'),
      rectangle: control.getBoundingClientRect().toJSON(),
    })),
  })) : []
  return {
    backups, files, policy: await invoke('preferences_backup_policy'),
    preferences: await invoke('get_preferences'),
    notifications: await invoke('list_notifications'),
    geometry, dpr: devicePixelRatio, foreground: await invoke('app_foreground'),
  }
})()
