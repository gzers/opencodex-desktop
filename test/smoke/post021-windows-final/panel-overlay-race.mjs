import assert from 'node:assert/strict'
import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { join } from 'node:path'
import { connect } from './cdp-session.mjs'
import { assertModelSandbox, modelConfigPath } from '../windows-model-ui.mjs'

const [endpoint, sandbox, output] = process.argv.slice(2)
assert(endpoint && sandbox && output, 'usage: panel-overlay-race.mjs ENDPOINT SANDBOX OUTPUT')
await mkdir(output, { recursive: true })
const session = await connect(endpoint)
const report = {
  result: 'fail',
  fixture: 'Five real native Hide/public-close/Show races; original payload and callback values preserved; no delayed response or mock',
  cases: [],
}
try {
  const facts = { about: await session.invoke('app_about'), dataRoot: await session.invoke('get_data_root_config') }
  assertModelSandbox(facts, sandbox)
  const config = modelConfigPath(facts, sandbox), before = await readFile(config)
  assert(await session.invoke('app_foreground'), 'native foreground required')
  const run = async () => {
    const pinia = document.querySelector('#app').__vue_app__.config.globalProperties.$pinia
    document.querySelector('.notification-panel.show button[aria-label="收起通知中心到铃铛"]')?.click()
    await new Promise(resolve => setTimeout(resolve, 300))
    if (location.hash !== '#panel' || !pinia._s.get('panel').loaded || pinia._s.get('panel').loadTrace.outcome !== 'visible-ready') {
      throw Error('Begin with an actually loaded native panel; loading/retry is a separate scenario')
    }
    const original = window.chrome.webview.postMessage, originalFetch = window.fetch
    const trace = window.__panelTransportTrace = [], cases = []
    let serial = 0, closeNext = false
    function submit(request, transport) {
      const id = ++serial
      const begin = { id, event: 'native-submit', at: performance.now(), action: request?.action, reload: request?.reload, transport }
      trace.push(begin)
      return begin
    }
    function closePending(begin) {
      if (begin.action !== 'hide' || !closeNext) return
      closeNext = false
      queueMicrotask(() => {
        const button = document.querySelector('.notification-panel.show button[aria-label="收起通知中心到铃铛"]')
        if (!button) { trace.push({ id: begin.id, event: 'missing-public-close', at: performance.now() }); return }
        button.click()
        trace.push({ id: begin.id, event: 'public-close', at: performance.now() })
      })
    }
    window.fetch = function (url, options) {
      if (!String(url).includes('/sync_embedded_panel')) return originalFetch.call(this, url, options)
      const payload = JSON.parse(options.body), begin = submit(payload.request, 'custom-protocol-fetch')
      const result = originalFetch.call(this, url, options)
      closePending(begin)
      result.then(response => response.clone().json().then(value => {
        trace.push({ id: begin.id, event: 'native-result', at: performance.now(), value, status: response.status })
      })).catch(error => trace.push({ id: begin.id, event: 'transport-error', at: performance.now(), message: String(error) }))
      return result
    }
    window.chrome.webview.postMessage = function (raw) {
      const message = JSON.parse(raw)
      if (message.cmd !== 'sync_embedded_panel') return original.call(this, raw)
      const begin = submit(message.payload.request, 'postMessage'), id = begin.id
      // Observe the actual one-shot callbacks without changing their values,
      // delivery time, registration identity, or the native request payload.
      for (const [kind, key] of [['native-result', 'callback'], ['native-error', 'error']]) {
        const callback = window.__TAURI_INTERNALS__.callbacks.get(message[key])
        if (callback) window.__TAURI_INTERNALS__.callbacks.set(message[key], value => {
          trace.push({ id, event: kind, at: performance.now(), value })
          return callback(value)
        })
      }
      const result = original.call(this, raw)
      closePending(begin)
      return result
    }
    try {
      for (let index = 0; index < 5; index++) {
        const from = trace.length
        closeNext = true
        pinia._s.get('notifications').panelOpen = true
        await new Promise(resolve => setTimeout(resolve, 500))
        const events = trace.slice(from)
        const hide = events.find(event => event.event === 'native-submit' && event.action === 'hide')
        const close = events.find(event => event.event === 'public-close')
        const end = events.find(event => event.event === 'native-result' && event.id === hide?.id)
        const show = events.find(event => event.event === 'native-submit' && event.action === 'show')
        const shown = events.find(event => event.event === 'native-result' && event.id === show?.id)
        cases.push({ index, hide, close, end, show, shown,
          notificationOpen: pinia._s.get('notifications').panelOpen,
          loaded: pinia._s.get('panel').loaded,
          trace: JSON.parse(JSON.stringify(pinia._s.get('panel').loadTrace)),
        })
      }
      return { cases, transport: trace }
    } finally { window.chrome.webview.postMessage = original; window.fetch = originalFetch; closeNext = false }
  }
  const result = await session.evaluate('(' + run.toString() + ')()')
  report.cases = result.cases; report.transport = result.transport
  for (const row of result.cases) {
    assert(row.hide && row.close && row.end && row.show && row.shown, 'incomplete real IPC observations')
    assert(row.hide.at <= row.close.at && row.close.at < row.end.at, 'close must precede actual Hide response')
    assert(row.shown.value.visible && row.shown.value.ready && !row.notificationOpen && row.loaded, 'restored native Show must be visible-ready')
  }
  assert((await readFile(config)).equals(before), 'config bytes changed')
  assert(await session.invoke('app_foreground'), 'native foreground lost')
  report.result = 'pass'
} catch (error) {
  report.error = { message: error.message, stack: error.stack }; process.exitCode = 1
  report.transport = await session.evaluate('window.__panelTransportTrace').catch(() => null)
} finally {
  await writeFile(join(output, 'panel-hide-race.json'), JSON.stringify(report, null, 2) + String.fromCharCode(10))
  session.close()
}
