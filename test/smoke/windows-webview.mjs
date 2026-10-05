// Smoke verification for the final Windows EXE, using WebView2's loopback CDP.
// Requires Node >=24. No external browser/automation package is needed.
import assert from 'node:assert/strict'
import { writeFile } from 'node:fs/promises'
import { join } from 'node:path'
const [endpoint, output] = process.argv.slice(2)
assert(endpoint && output, 'Usage: node windows-webview.mjs <loopback CDP endpoint> <output directory>')
assert.equal(new URL(endpoint).hostname, '127.0.0.1')
let target
for (let attempt = 0; attempt < 80; attempt++) {
  try {
    const targets = await (await fetch(`${endpoint}/json/list`)).json()
    target = targets.find(item => item.type === 'page' && item.webSocketDebuggerUrl)
    if (target) break
  } catch {}
  await new Promise(resolve => setTimeout(resolve, 250))
}
assert(target, 'Main WebView CDP target did not become available')
const ws = new WebSocket(target.webSocketDebuggerUrl)
await new Promise((resolve, reject) => {
  ws.addEventListener('open', resolve, { once: true })
  ws.addEventListener('error', reject, { once: true })
})
let id = 0
const pending = new Map()
ws.addEventListener('message', event => {
  const value = JSON.parse(event.data)
  if (!value.id) return
  const request = pending.get(value.id)
  if (!request) return
  pending.delete(value.id)
  value.error ? request.reject(Error(value.error.message)) : request.resolve(value.result)
})
function call(method, params = {}) {
  const current = ++id
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => { pending.delete(current); reject(Error(`CDP timeout: ${method}`)) }, 20_000)
    pending.set(current, { resolve: value => { clearTimeout(timer); resolve(value) }, reject: error => { clearTimeout(timer); reject(error) } })
    ws.send(JSON.stringify({ id: current, method, params }))
  })
}
try {
  let documentState
  for (let attempt = 0; attempt < 80; attempt++) {
    const result = await call('Runtime.evaluate', {
      expression: `({text: document.body.innerText, mounted: !!document.querySelector('#app')?.children.length, bridge: !!window.__TAURI_INTERNALS__?.invoke})`,
      returnByValue: true,
    })
    documentState = result.result.value
    if (documentState?.mounted && documentState.bridge && documentState.text.length > 30) break
    await new Promise(resolve => setTimeout(resolve, 250))
  }
  assert(documentState?.mounted && documentState.bridge && documentState.text.length > 30, 'Vue main UI did not render')
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
  const screenshot = await call('Page.captureScreenshot', { format: 'png' })
  await writeFile(join(output, 'main-window.png'), Buffer.from(screenshot.data, 'base64'))
  await writeFile(join(output, 'webview.json'), JSON.stringify({
    result: 'pass', mainUiMounted: true, bodyTextLength: documentState.text.length,
    about: facts.about, discovery: facts.discovery, cliEnabled: facts.preferences.cliEnabled,
    sandboxDataRootInitialized: true,
  }, null, 2) + '\n')
  console.log('PASS: Windows 0.1.8 rendered, Tauri commands succeeded, sandbox initialized.')
} finally { ws.close() }
