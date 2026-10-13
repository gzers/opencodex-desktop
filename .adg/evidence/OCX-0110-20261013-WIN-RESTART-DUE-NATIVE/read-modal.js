(() => {
  const dialog = document.querySelector('[role="dialog"][aria-modal="true"]')
  const active = document.activeElement
  return {
    atUtc: new Date().toISOString(), atMs: performance.now(), route: location.hash,
    dialogOpen: !!dialog, title: dialog?.querySelector('h3')?.textContent ?? null,
    focus: { tag: active?.tagName, text: active?.textContent?.trim().slice(0, 100),
      ariaLabel: active?.getAttribute('aria-label'), insideDialog: !!dialog?.contains(active),
      isDialog: active === dialog },
    buttons: dialog ? Array.from(dialog.querySelectorAll('button')).map(button => ({
      text: button.textContent?.trim(), ariaLabel: button.getAttribute('aria-label'),
      disabled: button.disabled,
    })) : [],
    notes: dialog ? Array.from(dialog.querySelectorAll('.update-notes')).map(note => ({
      text: note.textContent, clientHeight: note.clientHeight, scrollHeight: note.scrollHeight,
    })) : [],
    progressElements: dialog?.querySelectorAll('progress').length ?? 0,
    scope: 'DOM metrics of UI reached exclusively through native computer-use inputs; no focus or routing mutation',
  }
})()
