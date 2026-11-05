// Explicit sandbox-only native WebView2 diagnostics. Launch the desktop EXE with
// OPENCODEX_SANDBOX=1 and OPENCODEX_WINDOWS_SMOKE_CDP_PORT first. No headless
// browser is started. Fixtures change this isolated renderer and preferences only.
import assert from 'node:assert/strict'
import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'
import { discoverMainTarget } from '../../../../test/smoke/windows-cdp.mjs'
import { pngStripeContrast } from '../../../../test/smoke/windows-png.mjs'

const [endpoint, output, phase = 'inspect', sandboxRoot, expectation = 'fixed'] = process.argv.slice(2)
assert(endpoint && output && sandboxRoot, 'usage: node qa/windows-materials.mjs http://127.0.0.1:PORT OUTPUT PHASE EXPECTED_SANDBOX_ROOT')
await mkdir(output, { recursive: true })
const target = await discoverMainTarget(endpoint)
const socket = new WebSocket(target.webSocketDebuggerUrl)
await new Promise((resolve, reject) => { socket.addEventListener('open', resolve, { once: true }); socket.addEventListener('error', reject, { once: true }) })
let serial = 0
const pending = new Map()
socket.addEventListener('message', event => {
  const message = JSON.parse(event.data), request = pending.get(message.id)
  if (request) { pending.delete(message.id); message.error ? request.reject(Error(message.error.message)) : request.resolve(message.result) }
})
function call(method, params = {}) {
  return new Promise((resolve, reject) => {
    const id = ++serial, timer = setTimeout(() => { pending.delete(id); reject(Error('timeout: ' + method)) }, 45000)
    pending.set(id, { resolve: value => { clearTimeout(timer); resolve(value) }, reject: error => { clearTimeout(timer); reject(error) } })
    socket.send(JSON.stringify({ id, method, params }))
  })
}
async function evaluate(fn, ...args) {
  const value = await call('Runtime.evaluate', { expression: '(' + fn.toString() + ')(' + args.map(arg => JSON.stringify(arg)).join(',') + ')', awaitPromise: true, returnByValue: true })
  assert(!value.exceptionDetails, JSON.stringify(value.exceptionDetails))
  return value.result.value
}
async function capture(name) {
  const bytes = Buffer.from((await call('Page.captureScreenshot', { format: 'png', captureBeyondViewport: false })).data, 'base64')
  await writeFile(join(output, name + '.png'), bytes)
  return bytes
}
async function configure(theme, tier, scale = 100) {
  return evaluate(async (theme, tier, scale) => {
    const p = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia, prefs = p._s.get('preferences')
    if (!await prefs.save({ ...prefs.data, theme, visualEffects: tier, interfaceScale: scale, appUpdateAutoCheck: false })) throw Error('isolated preference save failed')
    p._s.get('theme').apply(theme)
    await new Promise(resolve => setTimeout(resolve, 220))
    return { ...document.documentElement.dataset, scale: prefs.data.interfaceScale }
  }, theme, tier, scale)
}
async function settle() { await new Promise(resolve => setTimeout(resolve, 250)) }
async function removeFixtures() {
  await evaluate(() => {
    for (const node of document.querySelectorAll('[data-windows-materials-fixture]')) node.remove()
  })
}
async function stripes() {
  await evaluate(() => {
    document.querySelector('#windows-materials-stripes')?.remove()
    const layer = document.createElement('div')
    layer.id = 'windows-materials-stripes'; layer.dataset.windowsMaterialsFixture = 'stripes'
    layer.style.cssText = 'position:absolute;inset:0;z-index:-1;pointer-events:none;background:repeating-linear-gradient(90deg,#050505 0 2px,#fafafa 2px 4px)'
    document.querySelector('.app-window').prepend(layer)
  })
}
async function openConnection() {
  await evaluate(async () => {
    const p = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia
    p._s.get('routes').go('sync', { tab: 'webdav' })
    p._s.get('sync').openConnection()
    await new Promise(resolve => setTimeout(resolve, 250))
  })
}
async function closeConnection() {
  await evaluate(() => document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('sync').connectionOpen = false)
  await settle()
}
async function dialogProbe(name) {
  const metrics = await evaluate(() => {
    const node = document.querySelector('dialog.sync-connection-dialog[open]')
    if (!node) throw Error('real connection dialog is absent')
    const box = node.getBoundingClientRect(), style = getComputedStyle(node)
    return { rect: box.toJSON(), sample: { x: box.x + 20, y: box.y + box.height - 170, width: box.width - 40, height: 90 }, blur: style.backdropFilter, background: style.backgroundColor,
      font: style.fontSize, titleFont: getComputedStyle(node.querySelector('h2')).fontSize, zoom: style.zoom, dpr: devicePixelRatio,
      focusInside: node.contains(document.activeElement), native: node.matches(':modal'), root: { ...document.documentElement.dataset } }
  })
  metrics.contrast = pngStripeContrast(await capture(name), metrics.sample, metrics.dpr)
  return metrics
}
async function gallery() {
  await closeConnection()
  // Load the actual lazy production stylesheet before sampling its scoped classes.
  await evaluate(async () => {
    const p = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia
    p._s.get('routes').go('models'); await new Promise(resolve => setTimeout(resolve, 300))
  })
  return evaluate(() => {
    document.querySelector('#windows-materials-gallery')?.remove()
    const host = document.createElement('div')
    host.id = 'windows-materials-gallery'; host.dataset.windowsMaterialsFixture = 'gallery'
    host.style.cssText = 'position:fixed;inset:12px;z-index:9999;display:grid;grid-template-columns:repeat(4,minmax(0,1fr));grid-auto-rows:140px;gap:12px;padding:12px;background:var(--panel);overflow:auto'
    const cases = [
      ['notification', 'notification-panel show'], ['toast', 'toast show'], ['task', 'process-progress'], ['menu', 'select-menu select-menu-portal'],
      ['panel', 'card'], ['row', 'setting-row'], ['button', 'btn'], ['tag', 'tag'],
      ['selector', 'select-trigger'], ['sticky', 'sticky-page-tabs'], ['detail-head', 'detail-workspace-head'], ['detail-collapse', 'detail-collapse'],
      ['overlay-card', 'card-overlay'], ['environment', 'env-gate'], ['agent-panel', 'model-agent-panel'], ['task-control', 'notif-ctl'],
    ]
    const rules = []
    const collect = items => { for (const rule of items) { if (rule.selectorText) rules.push(rule.selectorText); else if (rule.cssRules) collect(rule.cssRules) } }
    for (const sheet of document.styleSheets) collect(sheet.cssRules)
    for (const [name, classes] of cases) {
      const cell = document.createElement('div')
      cell.style.cssText = 'position:relative;min-width:0;background:repeating-linear-gradient(90deg,#050505 0 2px,#fafafa 2px 4px)'
      // Task controls inherit from a real task surface. A test-only inner stripe
      // in that backdrop root makes their own six/eight-pixel filter measurable.
      if (name === 'task-control') cell.className = 'process-progress'
      const node = document.createElement(['button', 'selector', 'task-control'].includes(name) ? 'button' : 'div')
      node.className = classes; node.dataset.materialProbe = name
      if (name === 'sticky') node.dataset.stuck = 'true'
      for (const selector of rules) {
        if (selector.includes('.' + classes.split(' ')[0])) {
          for (const [, scope] of selector.matchAll(/\[(data-v-[a-z0-9]+)\]/g)) node.setAttribute(scope, '')
        }
      }
      node.style.cssText = 'position:absolute;inset:0;display:block;width:100%;height:100%;min-width:0;max-width:none;max-height:none;margin:0;box-sizing:border-box;opacity:1;transform:none'
      const label = document.createElement('span')
      label.textContent = name; label.style.cssText = 'position:absolute;left:12px;top:12px;font:13px sans-serif;color:var(--text)'
      node.append(label); cell.append(node); host.append(cell)
    }
    document.querySelector('.app-window').append(host)
    return cases.map(([name]) => name)
  })
}
async function galleryProbe(name) {
  const metrics = await evaluate(() => [...document.querySelectorAll('[data-material-probe]')].map(node => {
    const style = getComputedStyle(node)
    return { name: node.dataset.materialProbe, rect: node.getBoundingClientRect().toJSON(), blur: style.backdropFilter, background: style.backgroundColor, dpr: devicePixelRatio }
  }))
  const pixels = await capture(name)
  for (const item of metrics) item.contrast = pngStripeContrast(pixels, item.rect, item.dpr)
  return metrics
}
async function injectUnsupportedFallback() {
  return evaluate(() => {
    const bodies = []
    const collect = rules => { for (const rule of rules) {
      if (rule instanceof CSSSupportsRule && /^not\b/.test(rule.conditionText) && rule.conditionText.includes('backdrop-filter')) {
        bodies.push([...rule.cssRules].map(child => child.cssText).join('\n'))
      } else if (rule.cssRules) collect(rule.cssRules)
    } }
    for (const sheet of document.styleSheets) collect(sheet.cssRules)
    if (!bodies.length) throw Error('production unsupported-filter fallback not found')
    const style = document.createElement('style')
    style.dataset.windowsMaterialsFixture = 'unsupported-filter-injection'
    style.textContent = bodies.join('\n'); document.documentElement.append(style)
    // The control probe adds an artificial stripe to its task backdrop root to
    // measure the control's own filter. During fallback that parent must instead
    // use the actual production task fill, which becomes opaque through tokens.
    document.querySelector('[data-material-probe="task-control"]')?.parentElement.style.removeProperty('background')
    // Start existing background transitions before the caller's settling delay;
    // the first computed-style read must not start them at screenshot time.
    void document.body.offsetHeight
    return { mode: 'explicit isolated injection of production @supports-not branch bodies', branches: bodies.length, root: { ...document.documentElement.dataset }, css: style.textContent }
  })
}
async function visibleSurfaces(selectors) {
  return evaluate(selectors => Object.fromEntries(selectors.map(selector => [selector,
    [...document.querySelectorAll(selector)].filter(node => {
      const box = node.getBoundingClientRect()
      return box.width > 0 && box.height > 0 && box.bottom > 0 && box.right > 0 && box.top < innerHeight && box.left < innerWidth
    }).map(node => {
      const style = getComputedStyle(node)
      return { classes: node.className, rect: node.getBoundingClientRect().toJSON(), blur: style.backdropFilter, background: style.backgroundColor,
        font: style.fontSize, parent: node.parentElement.tagName + '.' + node.parentElement.className, nativeDialog: node instanceof HTMLDialogElement && node.matches(':modal') }
    })])), selectors)
}
async function productionScenes(theme, tier) {
  await removeFixtures(); await closeConnection(); await configure(theme, tier)
  await evaluate(async () => {
    const p = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia
    p._s.get('routes').go('settings', { section: 'general' })
    await new Promise(resolve => setTimeout(resolve, 350))
    p._s.get('notifications').panelOpen = true
    p._s.get('feedback').showToast('隔离材质验证 · 未执行同步或重启', { durationMs: 60000 })
    p._s.get('operation-tasks').apply({ id: 'c0000000-0000-4000-8000-000000000021', kind: 'node_measurement', phase: 'running' })
    p._s.get('model-save-feedback').showSaved()
    await new Promise(resolve => setTimeout(resolve, 350))
  })
  const feedback = await visibleSurfaces(['.setting-row', '.notification-panel', '.toast', '.process-progress', '.process-progress .notif-ctl'])
  for (const [selector, matches] of Object.entries(feedback)) assert(matches.length, 'missing actual production surface: ' + selector)
  await capture(theme + '-' + tier + '-settings-feedback')
  await evaluate(async () => {
    const p = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia
    p._s.get('notifications').panelOpen = false; p._s.get('feedback').clearToast()
    p._s.get('operation-tasks').tasks = []; p._s.get('model-save-feedback').dismiss()
    p._s.get('routes').go('models')
    const models = p._s.get('model-config'); await models.load()
    models.view.tab = 'templates'; models.edit(models.snapshot.library.templates[0])
    models.view.templatePosition = models.draftSession; models.view.templateEditor = true
    await new Promise(resolve => setTimeout(resolve, 300))
    document.querySelector('.main').scrollTop = 180
    await new Promise(resolve => setTimeout(resolve, 180))
  })
  const detail = await visibleSurfaces(['.detail-workspace-head', '.detail-collapse', '.sticky-page-tabs', '.select-trigger'])
  for (const [selector, matches] of Object.entries(detail)) assert(matches.length, 'missing actual detail surface: ' + selector)
  await capture(theme + '-' + tier + '-detail-sticky')
  await evaluate(async () => {
    const trigger = [...document.querySelectorAll('.select-trigger')].find(node => !node.disabled && node.getClientRects().length)
    if (!trigger) throw Error('no actual select trigger')
    trigger.click(); await new Promise(resolve => setTimeout(resolve, 180))
  })
  const menu = await visibleSurfaces(['.select-menu-portal'])
  assert(menu['.select-menu-portal'].length, 'actual teleported menu did not open')
  await capture(theme + '-' + tier + '-select-portal')
  await evaluate(() => {
    const menu = document.querySelector('.select-menu-portal')
    if (menu) menu.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }))
  })
  return { theme, tier, feedback, detail, menu, fixtures: 'Toast/task/save notice are synthetic in-memory signals; the displayed components and handlers are production. No backend save, measurement, sync or restart is called.' }
}
async function clickText(text, within = 'body') {
  await evaluate(async (text, within) => {
    const button = [...document.querySelector(within).querySelectorAll('button')].find(node => !node.disabled && node.getClientRects().length && node.textContent.trim() === text)
    if (!button) throw Error('missing production action: ' + text)
    button.click(); await new Promise(resolve => setTimeout(resolve, 400))
  }, text, within)
}
async function captureDialog(name) {
  const surfaces = await visibleSurfaces(['dialog[open]'])
  assert(surfaces['dialog[open]'].length, 'actual public dialog did not open: ' + name)
  for (const surface of surfaces['dialog[open]']) {
    assert(surface.nativeDialog && surface.blur.includes('blur(8px)'), 'production host did not use the native public glass dialog: ' + name)
    assert.equal(surface.font, '13px', 'dialog body typography changed')
  }
  await capture(name)
  const closed = await evaluate(async () => {
    const dialogs = [...document.querySelectorAll('dialog[open]')]
    dialogs.at(-1).querySelector('.ui-dialog-header .modal-close').click()
    await new Promise(resolve => setTimeout(resolve, 250))
    return document.querySelectorAll('dialog[open]').length
  })
  assert.equal(closed, surfaces['dialog[open]'].length - 1, 'public X did not close exactly its layer')
  return { name, surfaces }
}
async function performanceSample(scroll) {
  return evaluate(async scroll => {
    if (!await window.__TAURI_INTERNALS__.invoke('app_foreground')) throw Error('native foreground required before performance sample; reconnect/unlock RDP first')
    const p = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia, effects = p._s.get('effects')
    const main = document.querySelector('.main'), intervals = [], longTasks = [], callbackCosts = [], dispatchDelays = [], scrollPositions = [], originalRaf = requestAnimationFrame
    const observer = new PerformanceObserver(list => longTasks.push(...list.getEntries().map(entry => ({ start: entry.startTime, duration: entry.duration }))))
    observer.observe({ type: 'longtask' })
    const canvas = document.querySelector('.motion-mesh-host canvas'), gl = canvas?.getContext('webgl'), draw = gl?.drawArrays
    let draws = 0, frame = 0, last = 0, lastScrollSample = -Infinity, active = true
    if (gl) gl.drawArrays = function (...args) { draws++; return draw.apply(this, args) }
    const start = performance.now()
    const tick = time => {
      if (!active) return
      const entered = performance.now()
      dispatchDelays.push(Math.max(0, entered - time))
      if (last) intervals.push(time - last)
      last = time
      if (scroll) main.scrollTop = Math.abs(Math.sin((time - start) / 1800)) * 900
      // Sample real movement once per second; a per-frame geometry read would
      // perturb the layout/composition cost that this probe is measuring.
      if (scroll && time - lastScrollSample >= 1000) { scrollPositions.push(main.scrollTop); lastScrollSample = time }
      callbackCosts.push(performance.now() - entered)
      frame = originalRaf(tick)
    }
    frame = originalRaf(tick)
    try {
      await new Promise(resolve => setTimeout(resolve, 30000))
      const sorted = [...intervals].sort((a, b) => a - b), B = 1000 / 60
      const distribution = values => {
        const ordered = [...values].sort((a, b) => a - b)
        return { samples: ordered.length, p50: ordered[Math.floor(ordered.length * .5)] ?? null, p95: ordered[Math.floor(ordered.length * .95)] ?? null, max: ordered.length ? ordered.at(-1) : null }
      }
      return { durationMs: performance.now() - start, scroll, B, samples: sorted.length, p50: sorted[Math.floor(sorted.length * .5)], p95: sorted[Math.floor(sorted.length * .95)],
        max: Math.max(...sorted), over6B: sorted.filter(value => value > 6 * B).length, longTasks, draws,
        diagnostics: { callbackCostMs: distribution(callbackCosts), callbackDispatchDelayMs: distribution(dispatchDelays),
          actualScrollRange: scrollPositions.length ? { min: Math.min(...scrollPositions), max: Math.max(...scrollPositions), available: main.scrollHeight - main.clientHeight } : null,
          note: 'Callback cost excludes later style/layout/paint/GPU work; dispatch delay alone cannot identify the bottleneck. Acceptance still uses the original frame-interval budget.' },
        strategy: JSON.parse(JSON.stringify(effects.strategy)), foreground: await window.__TAURI_INTERNALS__.invoke('app_foreground'), intervals }
    } finally { active = false; cancelAnimationFrame(frame); observer.disconnect(); if (gl) gl.drawArrays = draw }
  }, scroll)
}
const results = { phase, expectation, target: { id: target.id, url: target.url }, cases: [] }
try {
  results.binding = await evaluate(() => window.__TAURI_INTERNALS__.invoke('get_data_root_config'))
  const normalize = path => {
    const normalized = path.replaceAll('\\', '/').replace(/\/$/, '').toLowerCase()
    return normalized.startsWith('//?/') ? normalized.slice(4) : normalized
  }
  for (const key of ['activeDataRoot', 'currentDataRoot', 'opencodexHome', 'currentOpencodexHome']) {
    const path = normalize(results.binding[key]), expected = normalize(sandboxRoot)
    assert(path === expected || path.startsWith(expected + '/'), 'renderer binding escapes expected sandbox: ' + key)
  }
  results.environment = await evaluate(async () => {
    const canvas = document.createElement('canvas'), gl = canvas.getContext('webgl'), debug = gl?.getExtension('WEBGL_debug_renderer_info')
    const gpu = debug ? { vendor: gl.getParameter(debug.UNMASKED_VENDOR_WEBGL), renderer: gl.getParameter(debug.UNMASKED_RENDERER_WEBGL) } : null
    gl?.getExtension('WEBGL_lose_context')?.loseContext()
    return { about: await window.__TAURI_INTERNALS__.invoke('app_about'), native: await window.__TAURI_INTERNALS__.invoke('window_appearance'),
      ua: navigator.userAgent, dpr: devicePixelRatio, viewport: { width: innerWidth, height: innerHeight }, gpu,
      supports: CSS.supports('backdrop-filter', 'blur(1px)'), webkitSupports: CSS.supports('-webkit-backdrop-filter', 'blur(1px)'), root: { ...document.documentElement.dataset } }
  })
  if (phase === 'dialog' || phase === 'dialog-matrix') {
    await removeFixtures(); await stripes(); await openConnection()
    for (const theme of phase === 'dialog' ? ['dark'] : ['light', 'dark']) for (const tier of phase === 'dialog' ? ['high'] : ['high', 'mid', 'low']) {
      await configure(theme, tier)
      results.cases.push({ theme, tier, ...await dialogProbe(theme + '-' + tier + '-connection') })
    }
  } else if (phase === 'gallery-matrix') {
    await removeFixtures(); await configure('light', 'high'); await gallery()
    for (const theme of ['light', 'dark']) for (const tier of ['high', 'mid', 'low']) {
      await configure(theme, tier)
      results.cases.push({ theme, tier, surfaces: await galleryProbe(theme + '-' + tier + '-materials') })
    }
  } else if (phase === 'fallback') {
    await removeFixtures(); await configure('dark', 'high'); await gallery()
    results.injection = await injectUnsupportedFallback(); await settle()
    results.cases.push({ theme: 'dark', tier: 'high', surfaces: await galleryProbe('dark-high-injected-fallback') })
    await removeFixtures(); await stripes(); await openConnection()
    results.dialogInjection = await injectUnsupportedFallback(); await settle()
    results.cases.push({ theme: 'dark', tier: 'high', ...await dialogProbe('dark-high-connection-injected-fallback') })
  } else if (phase === 'production-matrix') {
    for (const theme of ['light', 'dark']) for (const tier of ['high', 'mid', 'low']) results.cases.push(await productionScenes(theme, tier))
  } else if (phase === 'hosts') {
    await removeFixtures(); await closeConnection(); await configure('dark', 'high')
    await evaluate(() => document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('modal').open({ title: '隔离查看窗口', body: '<p>只验证公共材质与退出，不执行操作。</p>', mode: 'view' }))
    await settle(); results.cases.push(await captureDialog('app-modal-view'))
    await evaluate(async () => { const p = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia; p._s.get('model-config').leave(); p._s.get('routes').go('sync', { tab: 'file' }); await new Promise(resolve => setTimeout(resolve, 400)) })
    await clickText('导入配置'); results.cases.push(await captureDialog('file-import'))
    await clickText('导出配置'); results.cases.push(await captureDialog('file-export-preview'))
    await evaluate(async () => {
      const p = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia
      p._s.get('routes').go('models'); p._s.get('model-config').view.tab = 'channels'; await new Promise(resolve => setTimeout(resolve, 300))
    })
    await clickText('新增渠道'); results.cases.push(await captureDialog('new-channel'))
    await evaluate(async () => {
      const models = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('model-config')
      const channel = { id: 'materials-fixture', label: '材质验收渠道', templateRefs: [], modelRefs: {}, connection: { providerId: 'materials-fixture', protocol: 'openai-chat', nodes: [{ id: 'one', name: '隔离节点', url: 'https://materials.invalid' }], activeNodeId: 'one', credentialRef: null, enabled: false, allowPrivateNetwork: false }, selection: { mode: 'none' }, modelOrder: [], discovered: [], defaults: {}, overrides: {}, familyOverrides: {} }
      models.channelDraft = { ...JSON.parse(JSON.stringify(models.snapshot.policy)), channels: [channel], channelOrder: [channel.id] }
      await new Promise(resolve => setTimeout(resolve, 250))
      const trigger = document.querySelector('.channel-node-detect')
      if (!trigger) throw Error('synthetic channel did not expose the actual node viewer')
      trigger.click(); await new Promise(resolve => setTimeout(resolve, 250))
    })
    results.cases.push({ ...await captureDialog('node-detection-view'), fixture: 'One in-memory disabled channel at materials.invalid; no measurement command is invoked.' })
    await evaluate(async () => {
      const p = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia, models = p._s.get('model-config')
      models.channelDraft = JSON.parse(JSON.stringify(models.snapshot.policy))
      p._s.get('runtime').requestInstallModal('registry'); await new Promise(resolve => setTimeout(resolve, 300))
    })
    results.cases.push(await captureDialog('runtime-install-view'))
    await evaluate(async () => { document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('routes').go('settings', { section: 'upgrade' }); await new Promise(resolve => setTimeout(resolve, 350)) })
    await clickText('查看管理器更新'); results.cases.push(await captureDialog('update-center-view'))
  } else if (phase === 'performance-modal') {
    assert(await evaluate(() => window.__TAURI_INTERNALS__.invoke('app_foreground')), 'native foreground required before performance setup; reconnect/unlock RDP first')
    await removeFixtures(); await closeConnection(); await configure('dark', 'high')
    await evaluate(async () => {
      const p = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia
      p._s.get('feedback').clearToast(); p._s.get('operation-tasks').tasks = []; p._s.get('model-save-feedback').dismiss(); p._s.get('notifications').panelOpen = false
      p._s.get('routes').go('overview'); await new Promise(resolve => setTimeout(resolve, 1200))
      p._s.get('sync').openConnection(); await new Promise(resolve => setTimeout(resolve, 500))
    })
    results.cases.push({ name: 'modal-overview-high', ...await performanceSample(false) })
    for (const result of results.cases) { assert(result.foreground, 'performance sample lost native foreground'); assert(result.p95 <= 2 * result.B, 'existing modal p95 <= 2B budget failed'); assert.equal(result.over6B, 0, 'long modal frame requires analysis') }
  } else if (phase === 'performance') {
    assert(await evaluate(() => window.__TAURI_INTERNALS__.invoke('app_foreground')), 'native foreground required before performance setup; reconnect/unlock RDP first')
    await removeFixtures(); await closeConnection(); await configure('dark', 'high')
    await evaluate(async () => {
      const p = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia
      p._s.get('feedback').clearToast(); p._s.get('operation-tasks').tasks = []; p._s.get('model-save-feedback').dismiss(); p._s.get('notifications').panelOpen = false
      p._s.get('routes').go('overview'); await new Promise(resolve => setTimeout(resolve, 1200))
    })
    results.cases.push({ name: 'static-overview-high', ...await performanceSample(false) })
    await evaluate(async () => {
      const p = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia, models = p._s.get('model-config')
      p._s.get('routes').go('models'); await models.load(); models.view.tab = 'templates'; models.edit(models.snapshot.library.templates[0]); models.view.templatePosition = models.draftSession; models.view.templateEditor = true
      await new Promise(resolve => setTimeout(resolve, 1200))
    })
    results.cases.push({ name: 'scroll-detail-high', ...await performanceSample(true) })
    for (const result of results.cases) { assert(result.foreground, 'performance sample lost native foreground'); assert(result.p95 <= 2 * result.B, 'existing p95 <= 2B budget failed'); assert.equal(result.over6B, 0, 'long frame requires analysis') }
  } else if (phase === 'scale') {
    await removeFixtures(); await openConnection()
    for (const scale of [50, 100, 200]) for (const tier of ['high', 'low']) {
      await configure('dark', tier, scale)
      const scene = await visibleSurfaces(['dialog.sync-connection-dialog[open]'])
      const panel = scene['dialog.sync-connection-dialog[open]'][0]
      assert(panel && panel.rect.x >= 0 && panel.rect.y >= 0 && panel.rect.right <= results.environment.viewport.width && panel.rect.bottom <= results.environment.viewport.height, 'scaled dialog clipped')
      results.cases.push({ scale, tier, panel }); await capture('scale-' + scale + '-' + tier)
    }
    await configure('dark', 'high', 100)
  } else if (phase === 'hub') {
    await removeFixtures(); await closeConnection(); await configure('dark', 'high'); await stripes()
    const source = await readFile(new URL('../../tauri/assets/panel-hub.js', import.meta.url), 'utf8')
    const mounted = await call('Runtime.evaluate', { expression: source, awaitPromise: true, returnByValue: true })
    assert(!mounted.exceptionDetails, JSON.stringify(mounted.exceptionDetails))
    await settle()
    await evaluate(() => {
      const root = document.querySelector('#ocxd-panel-hub'); root.dataset.windowsMaterialsFixture = 'hub'
      root.classList.add('open')
    })
    for (const theme of ['light', 'dark']) for (const tier of ['high', 'mid', 'low']) {
      await configure(theme, tier)
      await evaluate((theme, tier) => window.dispatchEvent(new CustomEvent('ocxd-panel-state', { detail: { theme, effects: tier, scale: 100 } })), theme, tier)
      await settle()
      const surfaces = await visibleSurfaces(['#ocxd-panel-hub .hub-menu', '#ocxd-panel-hub .hub-toggle'])
      const menu = surfaces['#ocxd-panel-hub .hub-menu'][0]
      assert(menu, 'native renderer did not mount the exact panel-hub source')
      const bytes = await capture(theme + '-' + tier + '-hub')
      menu.contrast = pngStripeContrast(bytes, menu.rect, results.environment.dpr)
      results.cases.push({ theme, tier, surfaces, fixture: 'Exact injected overlay source in the native manager WebView2, not a running official child panel.' })
    }
    await configure('dark', 'high')
    await evaluate(() => window.dispatchEvent(new CustomEvent('ocxd-panel-state', { detail: { theme: 'dark', effects: 'high', scale: 100 } })))
    results.injection = await injectUnsupportedFallback(); await settle()
    const surfaces = await visibleSurfaces(['#ocxd-panel-hub .hub-menu', '#ocxd-panel-hub .hub-toggle'])
    surfaces['#ocxd-panel-hub .hub-menu'][0].contrast = pngStripeContrast(await capture('dark-high-hub-fallback'), surfaces['#ocxd-panel-hub .hub-menu'][0].rect, results.environment.dpr)
    results.cases.push({ theme: 'dark', tier: 'high', fallback: true, surfaces })
  } else if (phase === 'restore') {
    await removeFixtures(); await configure('dark', 'high'); await openConnection()
    await capture('connection-normal')
  } else if (phase !== 'inspect') throw Error('unknown diagnostic phase: ' + phase)
  if (expectation === 'fixed') {
    const filter = (surface, solid, label) => assert(solid ? surface.blur === 'none' : surface.blur !== 'none', label + ': unexpected filter ' + surface.blur)
    const pixel = (surface, label) => assert(surface.contrast <= 1, label + ': sharp stripes remain (' + surface.contrast + ')')
    for (const scene of results.cases) {
      const solid = scene.tier === 'low' || scene.fallback || phase === 'fallback'
      if (scene.contrast !== undefined) { filter(scene, solid, phase); pixel(scene, phase) }
      if (Array.isArray(scene.surfaces)) for (const surface of scene.surfaces) { filter(surface, solid, surface.name); pixel(surface, surface.name) }
      if (phase === 'production-matrix') for (const group of [scene.feedback, scene.detail, scene.menu]) {
        for (const [selector, matches] of Object.entries(group)) for (const surface of matches) filter(surface, solid, selector)
      }
      if (phase === 'hub') for (const [selector, matches] of Object.entries(scene.surfaces)) for (const surface of matches) {
        filter(surface, solid, selector); if (surface.contrast !== undefined) pixel(surface, selector)
      }
    }
  }
  results.result = expectation === 'fixed' ? 'passed' : 'captured-baseline'
  await writeFile(join(output, phase + '.json'), JSON.stringify(results, null, 2) + '\n')
  console.log(JSON.stringify({ phase, cases: results.cases.length, environment: results.environment }, null, 2))
} catch (error) {
  await writeFile(join(output, phase + '-failed.json'), JSON.stringify({ ...results, error: { message: error.message, stack: error.stack } }, null, 2) + '\n')
  throw error
} finally { socket.close() }
