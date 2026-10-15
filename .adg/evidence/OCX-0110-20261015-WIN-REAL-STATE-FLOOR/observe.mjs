import assert from 'node:assert/strict'
import { readFile, writeFile } from 'node:fs/promises'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { dirname, join } from 'node:path'
const output = dirname(fileURLToPath(import.meta.url))
const code = 'C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop'
const { discoverMainTarget } = await import(pathToFileURL(join(code, 'test/smoke/windows-cdp.mjs')))
const launch = JSON.parse((await readFile(join(output, 'state-floor-retry-launch.json'), 'utf8')).replace(/^\uFEFF/, ''))
const target = await discoverMainTarget(`http://127.0.0.1:${launch.port}`, { timeoutMs: 20000 })
assert.equal(new URL(target.webSocketDebuggerUrl).port, String(launch.port))
const ws = new WebSocket(target.webSocketDebuggerUrl)
await new Promise((resolve, reject) => {
  const timer = setTimeout(() => reject(Error('WebSocket open deadline')), 5000)
  ws.addEventListener('open', () => { clearTimeout(timer); resolve() }, { once: true })
  ws.addEventListener('error', () => { clearTimeout(timer); reject(Error('WebSocket open failed')) }, { once: true })
})
let id = 0
const pending = new Map()
const records = new Map()
const events = []
const commands = new Set(['update_schedule_plan', 'check_for_update', 'official_remote_latest'])
function call(method, params = {}) {
  const current = ++id
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => { pending.delete(current); reject(Error(`CDP deadline ${method}`)) }, 10000)
    pending.set(current, { resolve, reject, timer })
    ws.send(JSON.stringify({ id: current, method, params }))
  })
}
ws.addEventListener('message', event => {
  const value = JSON.parse(event.data)
  if (value.id) {
    const item = pending.get(value.id)
    if (!item) return
    pending.delete(value.id); clearTimeout(item.timer)
    value.error ? item.reject(Error(value.error.message)) : item.resolve(value.result)
    return
  }
  const p = value.params
  if (value.method === 'Network.requestWillBeSent') {
    let url
    try { url = new URL(p.request.url) } catch { return }
    const command = url.pathname.replace(/^\//, '')
    if (!commands.has(command)) return
    if (!['ipc:', 'http:', 'https:'].includes(url.protocol) ||
        !['localhost', 'ipc.localhost'].includes(url.hostname)) return
    const record = { command, requestId: p.requestId, startedAtUtc: new Date(p.wallTime * 1000).toISOString(),
      startedMonotonicSeconds: p.timestamp, transport: url.protocol + '//' + url.hostname }
    records.set(p.requestId, record); events.push(record)
  } else if (value.method === 'Network.responseReceived' && records.has(p.requestId)) {
    const record = records.get(p.requestId)
    record.responseMonotonicSeconds = p.timestamp
    record.httpStatus = p.response.status
  } else if (value.method === 'Network.loadingFinished' && records.has(p.requestId)) {
    const record = records.get(p.requestId)
    record.finishedMonotonicSeconds = p.timestamp
    record.finishedObservedAtUtc = new Date().toISOString()
    void call('Network.getResponseBody', { requestId: p.requestId }).then(body => {
      let text = body.base64Encoded ? Buffer.from(body.body, 'base64').toString('utf8') : body.body
      if (text.length > 16384) throw Error('IPC response exceeds bounded observation')
      record.responseText = text
      record.status = record.httpStatus >= 400 ? 'rejected' : 'fulfilled'
    }).catch(error => { record.readError = String(error) })
  } else if (value.method === 'Network.loadingFailed' && records.has(p.requestId)) {
    const record = records.get(p.requestId)
    record.status = 'transport-failed'; record.failureMonotonicSeconds = p.timestamp
  }
})
async function evaluate(expression) {
  const result = await call('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true })
  if (result.exceptionDetails) throw Error(JSON.stringify(result.exceptionDetails))
  return result.result.value
}
let repaired = false
const started = performance.now()
try {
  await call('Network.enable', { maxTotalBufferSize: 2 * 1024 * 1024, maxResourceBufferSize: 65536 })
  const initial = await evaluate(`({atUtc:new Date().toISOString(),nowPerf:performance.now(),
    visible:document.visibilityState,hasFocus:document.hasFocus(),bridgeInvokeWritable:
    Object.getOwnPropertyDescriptor(window.__TAURI_INTERNALS__,'invoke').writable})`)
  await writeFile(join(output, 'network-instrumentation.json'), JSON.stringify({
    ...initial, mode: 'passive Network domain, selected IPC commands only',
    bridgeReplaced: false, frontendOrNativeResultStubbed: false, requestInterceptionEnabled: false,
  }, null, 2) + '\n', { flag: 'wx' })
  let lastWake = -1
  let firstFailure
  let complete = false
  while (performance.now() - started < 1950_000) {
    const clock = await evaluate(`({atUtc:new Date().toISOString(),nowPerf:performance.now(),
      documentVisible:document.visibilityState,hasFocus:document.hasFocus()})`)
    await writeFile(join(output, 'ipc-observation.json'), JSON.stringify({ clock, events }, null, 2) + '\n')
    firstFailure ??= events.find(event => event.command === 'update_schedule_plan' && event.status === 'rejected')
    if (!firstFailure && performance.now() - started > 90000) throw Error('No real rejected schedule IPC observed within 90 seconds')
    if (firstFailure) {
      const elapsed = Date.now() - Date.parse(firstFailure.finishedObservedAtUtc)
      if (!repaired && elapsed >= 10000) {
        const now = Math.floor(Date.now() / 1000)
        const state = { schema: 1, entries: Object.fromEntries(
          ['manager-stable', 'manager-beta', 'panel-latest'].map(key => [key, {
            next_due: now + 18 * 3600, failures: 0, last_success: now,
          }])) }
        await writeFile(join(launch.dataRoot, 'manager-state/update-schedule.json'), JSON.stringify(state))
        await writeFile(join(output, 'schedule-repair.json'), JSON.stringify({
          atUtc: new Date().toISOString(), elapsedSinceRealErrorObservedMs: elapsed, state,
          oldClockChanged: false, frontendTimerChanged: false,
        }, null, 2) + '\n', { flag: 'wx' })
        repaired = true
      }
      const stage = Math.floor(elapsed / 60000)
      if (stage !== lastWake && stage <= 29) {
        await evaluate(`(() => { window.dispatchEvent(new Event('online')); return true; })()`)
        lastWake = stage
      }
      const retry = events.find(event => event.command === 'update_schedule_plan' &&
        event.startedMonotonicSeconds > firstFailure.startedMonotonicSeconds)
      if (retry?.status) {
        const elapsedMs = (retry.startedMonotonicSeconds - firstFailure.finishedMonotonicSeconds) * 1000
        const result = { atUtc: new Date().toISOString(), firstFailure, retry,
          realMonotonicRetryGapMs: elapsedMs, expectedMinimumMs: 1800000,
          noEarlyPlannerCalls: elapsedMs >= 1800000, repaired,
          retryActuallyFulfilled: retry.status === 'fulfilled',
          queryEvents: events.filter(event => event.command !== 'update_schedule_plan'),
          bridgeReplaced: false, frontendOrNativeResultStubbed: false,
          systemClockOrTimerChanged: false, syntheticOnlineWakeEvents: true,
          passiveRealIpcObservation: true, fullSchedulerGatePassed: false }
        await writeFile(join(output, 'result.json'), JSON.stringify(result, null, 2) + '\n', { flag: 'wx' })
        assert(result.noEarlyPlannerCalls)
        assert(result.retryActuallyFulfilled)
        assert.equal(result.queryEvents.length, 0)
        complete = true
        break
      }
      await writeFile(join(output, 'progress.json'), JSON.stringify({
        atUtc: clock.atUtc, phase: 'real-30-minute-floor', elapsedSinceRealErrorObservedMs: elapsed,
        dueAtUtcEstimate: new Date(Date.now() + Math.max(0, 1800000 - elapsed)).toISOString(),
        observedPlanCalls: events.filter(event => event.command === 'update_schedule_plan').length,
        queryCalls: events.filter(event => event.command !== 'update_schedule_plan').length,
        repaired, documentVisible: clock.documentVisible, hasFocus: clock.hasFocus,
      }, null, 2) + '\n')
    }
    await new Promise(resolve => setTimeout(resolve, 5000))
  }
  assert(complete, 'No actual planner retry within bounded real interval')
} catch (error) {
  await writeFile(join(output, 'network-failure.original.json'), JSON.stringify({
    atUtc: new Date().toISOString(), error: String(error), repaired, events, fullSchedulerGatePassed: false,
  }, null, 2) + '\n')
  process.exitCode = 1
} finally { ws.close() }
