// Smoke verification for the final Windows EXE, using WebView2's loopback CDP.
// Requires Node >=24. No external browser/automation package is needed.
import assert from 'node:assert/strict'
import { writeFile } from 'node:fs/promises'
import { join } from 'node:path'
import { describeError, discoverMainTarget } from './windows-cdp.mjs'
const [endpoint, output] = process.argv.slice(2)
assert(endpoint && output, 'Usage: node windows-webview.mjs <loopback CDP endpoint> <output directory>')
assert.equal(new URL(endpoint).hostname, '127.0.0.1')
const diagnostics = {}
const saveDiagnostics = value => writeFile(join(output, 'cdp-diagnostics.json'), JSON.stringify(value, null, 2) + '\n')
let ws
let id = 0
const pending = new Map()
function call(method, params = {}) {
  if (ws.readyState !== WebSocket.OPEN) return Promise.reject(Error('CDP WebSocket is not open'))
  const current = ++id
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => { pending.delete(current); reject(Error(`CDP timeout: ${method}`)) }, 20_000)
    pending.set(current, { resolve: value => { clearTimeout(timer); resolve(value) }, reject: error => { clearTimeout(timer); reject(error) } })
    try { ws.send(JSON.stringify({ id: current, method, params })) }
    catch (error) { pending.delete(current); clearTimeout(timer); reject(error) }
  })
}
try {
  const target = await discoverMainTarget(endpoint, { diagnostics, saveDiagnostics })
  diagnostics.phase = 'websocket'
  const socketUrl = new URL(target.webSocketDebuggerUrl)
  assert.equal(socketUrl.protocol, 'ws:')
  assert(['127.0.0.1', 'localhost', '[::1]'].includes(socketUrl.hostname), 'CDP WebSocket must remain on loopback')
  assert.equal(socketUrl.port, new URL(endpoint).port)
  ws = new WebSocket(target.webSocketDebuggerUrl)
  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(Error('CDP WebSocket open timeout')), 5000)
    ws.addEventListener('open', () => { clearTimeout(timer); resolve() }, { once: true })
    ws.addEventListener('error', () => { clearTimeout(timer); reject(Error('CDP WebSocket open failed')) }, { once: true })
  })
  ws.addEventListener('message', event => {
    let value
    try { value = JSON.parse(event.data) }
    catch (error) {
      for (const request of pending.values()) request.reject(Error('Malformed CDP WebSocket message: ' + error.message))
      pending.clear()
      ws.close()
      return
    }
    if (!value.id) return
    const request = pending.get(value.id)
    if (!request) return
    pending.delete(value.id)
    value.error ? request.reject(Error(value.error.message)) : request.resolve(value.result)
  })
  ws.addEventListener('close', () => {
    for (const request of pending.values()) request.reject(Error('CDP WebSocket closed'))
    pending.clear()
  })
  diagnostics.phase = 'render'
  let documentState
  for (let attempt = 0; attempt < 80; attempt++) {
    const result = await call('Runtime.evaluate', {
      expression: `({text: document.body?.innerText ?? '', mounted: !!document.querySelector('#app')?.children.length, bridge: !!window.__TAURI_INTERNALS__?.invoke})`,
      returnByValue: true,
    })
    assert(!result.exceptionDetails, JSON.stringify(result.exceptionDetails))
    documentState = result.result.value
    if (documentState?.mounted && documentState.bridge && documentState.text.length > 30) break
    await new Promise(resolve => setTimeout(resolve, 250))
  }
  assert(documentState?.mounted && documentState.bridge && documentState.text.length > 30, 'Vue main UI did not render')
  diagnostics.phase = 'invoke'
  const invoke = await call('Runtime.evaluate', {
    expression: `(async () => {
      const invoke = window.__TAURI_INTERNALS__.invoke;
      return {
        about: await invoke('app_about'),
        discovery: await invoke('discover_environment', {request: null}),
        preferences: await invoke('get_preferences'),
        dataRoot: await invoke('get_data_root_config')
      };
    })()`,
    awaitPromise: true,
    returnByValue: true,
  })
  assert(!invoke.exceptionDetails, JSON.stringify(invoke.exceptionDetails))
  const facts = invoke.result.value
  assert.equal(facts.about.platform, 'Windows')
  assert.equal(facts.about.version, '0.1.8')
  assert.equal(facts.preferences.cliEnabled, false)
  assert(facts.dataRoot, 'Application data root did not initialize')
  diagnostics.phase = 'screenshot'
  const screenshot = await call('Page.captureScreenshot', { format: 'png' })
  await writeFile(join(output, 'main-window.png'), Buffer.from(screenshot.data, 'base64'))
  await writeFile(join(output, 'webview.json'), JSON.stringify({
    result: 'pass', mainUiMounted: true, bodyTextLength: documentState.text.length,
    about: facts.about, discovery: facts.discovery, cliEnabled: facts.preferences.cliEnabled,
    sandboxDataRootInitialized: true,
  }, null, 2) + '\n')
  diagnostics.result = 'pass'
  diagnostics.phase = 'complete'
  await saveDiagnostics(diagnostics)
  console.log('PASS: Windows 0.1.8 rendered, Tauri commands succeeded, sandbox initialized.')
} catch (error) {
  diagnostics.result = 'fail'
  diagnostics.error = describeError(error)
  await saveDiagnostics(diagnostics)
  throw error
} finally { ws?.close() }
