import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { win32 } from 'node:path'
import { discoverMainTarget } from 'file:///C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop/test/smoke/windows-cdp.mjs'

const base = 'C:\\Users\\15119\\.codex\\worktrees\\0110-windows-gates\\opencodex-desktop\\test\\runtime\\windows0110'
const session = JSON.parse((await readFile(base + '\\control\\current-session.json', 'utf8')).replace(/^\uFEFF/, ''))
assert.equal(session.status, 'running')
assert(session.sourceCommitMatched)
assert(Date.now() - Date.parse(session.lastLiveAtUtc ?? session.startedAtUtc) < 30_000, 'Supervisor observation is stale')
const expressionFile = process.argv[2]
assert(expressionFile, 'A saved task expression is required')
const target = await discoverMainTarget(session.endpoint)
const socket = new WebSocket(target.webSocketDebuggerUrl)
const pending = new Map()
let nextId = 0
await new Promise((resolve, reject) => {
  const timeout = setTimeout(() => reject(Error('WebSocket connection deadline')), 5000)
  socket.addEventListener('open', () => { clearTimeout(timeout); resolve() }, { once: true })
  socket.addEventListener('error', () => { clearTimeout(timeout); reject(Error('WebSocket failed')) }, { once: true })
})
socket.addEventListener('message', event => {
  const response = JSON.parse(event.data)
  const request = pending.get(response.id)
  if (!request) return
  pending.delete(response.id)
  clearTimeout(request.timeout)
  response.error ? request.reject(Error(response.error.message)) : request.resolve(response.result)
})
socket.addEventListener('close', () => {
  for (const request of pending.values()) { clearTimeout(request.timeout); request.reject(Error('WebSocket closed')) }
  pending.clear()
})
function call(method, params) {
  return new Promise((resolve, reject) => {
    const id = ++nextId
    const timeout = setTimeout(() => { pending.delete(id); reject(Error(method + ' deadline')) }, 45_000)
    pending.set(id, { resolve, reject, timeout })
    socket.send(JSON.stringify({ id, method, params }))
  })
}
async function evaluate(expression) {
  const response = await call('Runtime.evaluate', { expression, awaitPromise: true, returnByValue: true })
  assert(!response.exceptionDetails, JSON.stringify(response.exceptionDetails))
  return response.result.value
}
function normalize(path) {
  return win32.normalize(path.replace(/^\\\\\?\\/, '')).toLowerCase()
}
try {
  const guard = await evaluate(`(async()=>{
    const invoke=window.__TAURI_INTERNALS__.invoke;
    return {about:await invoke('app_about'),root:await invoke('get_data_root_config')};
  })()`)
  assert.equal(guard.about.version, '0.1.10')
  assert.equal(guard.about.platform, 'Windows')
  assert(normalize(guard.root.currentDataRoot).startsWith(normalize(base) + '\\'), 'Application is outside the isolated container')
  const expression = await readFile(expressionFile, 'utf8')
  const result = await evaluate(expression)
  console.log(JSON.stringify({
    collectedAtUtc: new Date().toISOString(), sourceCommit: session.sourceCommit,
    applicationPid: session.applicationPid, executableSha256: session.executableSha256,
    currentDataRoot: guard.root.currentDataRoot, expressionFile, result,
  }, null, 2))
} finally { socket.close() }
