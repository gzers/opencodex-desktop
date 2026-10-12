(async () => {
  const selectors = ['html', 'body', '#app', '.app-window', '.desktop-stage', '.app-shell', 'main.main']
  return {
    innerWidth, innerHeight, outerWidth, outerHeight, dpr: devicePixelRatio,
    documentWidth: document.documentElement.clientWidth,
    documentHeight: document.documentElement.clientHeight,
    elements: selectors.map(selector => {
      const element = document.querySelector(selector)
      if (!element) return { selector, present: false }
      const style = getComputedStyle(element)
      return {
        selector, box: element.getBoundingClientRect().toJSON(),
        cssWidth: style.width, maxWidth: style.maxWidth, minWidth: style.minWidth,
        cssHeight: style.height, zoom: style.zoom, display: style.display,
      }
    }),
    foreground: await window.__TAURI_INTERNALS__.invoke('app_foreground'),
    appearance: await window.__TAURI_INTERNALS__.invoke('window_appearance'),
    scope: 'read-only actual WebView viewport and layout after native caption input',
  }
})()
