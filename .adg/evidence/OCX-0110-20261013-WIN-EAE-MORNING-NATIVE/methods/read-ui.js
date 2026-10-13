(async () => {
  const invoke = window.__TAURI_INTERNALS__.invoke
  const pinia = document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia
  const preferences = pinia?._s.get('preferences')
  const effects = pinia?._s.get('effects')
  const updates = pinia?._s.get('updates')
  const rect = element => {
    if (!element) return null
    const r = element.getBoundingClientRect()
    return { left: r.left, top: r.top, width: r.width, height: r.height, right: r.right, bottom: r.bottom }
  }
  const active = document.activeElement
  const dialogs = [...document.querySelectorAll('[role=dialog]')].map(dialog => ({
    role: dialog.getAttribute('role'), modal: dialog.getAttribute('aria-modal'),
    title: document.getElementById(dialog.getAttribute('aria-labelledby'))?.textContent,
    activeFocusInside: dialog.contains(active), rectangle: rect(dialog),
    header: rect(dialog.querySelector('.update-center-head')),
    body: rect(dialog.querySelector('.update-center-body')),
    footer: rect(dialog.querySelector('.update-center-foot')),
    bodyScrollTop: dialog.querySelector('.update-center-body')?.scrollTop,
    buttons: [...dialog.querySelectorAll('button')].map(button => ({
      label: button.getAttribute('aria-label') || button.textContent?.trim(),
      disabled: button.disabled, expanded: button.getAttribute('aria-expanded'),
      focused: button === active, rectangle: rect(button),
    })),
    notes: [...dialog.querySelectorAll('.update-notes')].map(note => ({
      label: note.getAttribute('aria-label'), text: note.textContent,
      tabIndex: note.tabIndex, scrollTop: note.scrollTop,
      scrollHeight: note.scrollHeight, clientHeight: note.clientHeight,
      overflowY: getComputedStyle(note).overflowY, rectangle: rect(note),
    })),
    checkboxes: [...dialog.querySelectorAll('input[type=checkbox]')].map(input => ({
      label: input.parentElement?.textContent?.trim(), checked: input.checked, disabled: input.disabled,
    })),
    progress: [...dialog.querySelectorAll('progress')].map(element => ({
      label: element.getAttribute('aria-label'), hasValue: element.hasAttribute('value'),
      value: element.value, max: element.max,
    })),
    versions: [...dialog.querySelectorAll('.update-versions')].map(element => element.textContent?.trim()),
    results: [...dialog.querySelectorAll('.update-object-head .tag')].map(element => element.textContent?.trim()),
    hints: [...dialog.querySelectorAll('.update-hint')].map(element => element.textContent?.trim()),
  }))
  return {
    atUtc: new Date().toISOString(), atMs: performance.now(), route: location.hash,
    foreground: await invoke('app_foreground'), documentVisibility: document.visibilityState,
    dpr: devicePixelRatio, viewport: { width: innerWidth, height: innerHeight },
    activeElement: { tag: active?.tagName, label: active?.getAttribute('aria-label') || active?.textContent?.trim()?.slice(0, 150),
      className: active?.className, inUpdateDialog: active?.closest('.update-center') !== null },
    manager: await invoke('get_update_status'), panelCache: await invoke('official_remote_cache'),
    uiPreferences: { channel: preferences?.data?.appUpdateChannel, automaticCheck: preferences?.data?.appUpdateAutoCheck,
      theme: preferences?.data?.theme, error: preferences?.error },
    effects: { visible: effects?.visible, foreground: effects?.foreground },
    busy: { manager: updates?.appUpdateBusy, panel: updates?.officialUpdateBusy },
    dialogs,
    scope: 'read-only real DOM geometry/focus and cached native statuses; no route/focus/scroll/input, check, install or response injection',
  }
})()
