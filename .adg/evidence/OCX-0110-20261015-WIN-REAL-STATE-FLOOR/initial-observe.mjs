import assert from 'node:assert/strict'
import { readFile, writeFile, copyFile } from 'node:fs/promises'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { dirname, join } from 'node:path'

const output = dirname(fileURLToPath(import.meta.url))
const code = 'C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop'
const { discoverMainTarget } = await import(pathToFileURL(join(code, 'test/smoke/windows-cdp.mjs')))
const launch = JSON.parse((await readFile(join(output, 'state-floor-launch.json'), 'utf8')).replace(/^\uFEFF/, ''))
assert.equal(launch.source, '02a0d4a8cbae8ef1450908d86b2c75fcad65f04f')
const endpoint = `http://127.0.0.1:${launch.port}`
const target = await discoverMainTarget(endpoint, { timeoutMs: 20000 })
assert(new URL(target.webSocketDebuggerUrl).port === String(launch.port))
const ws = new WebSocket(target.webSocketDebuggerUrl)
await new Promise((resolve, reject) => {
  const timer = setTimeout(() => reject(Error('WebSocket open deadline')), 5000)
  ws.addEventListener('open', () => { clearTimeout(timer); resolve() }, { once: true })
  ws.addEventListener('error', () => { clearTimeout(timer); reject(Error('WebSocket open failed')) }, { once: true })
})
let id = 0
const pending = new Map()
ws.addEventListener('message', event => {
  const value = JSON.parse(event.data)
  if (!value.id) return
  const item = pending.get(value.id)
  if (!item) return
  pending.delete(value.id)
  clearTimeout(item.timer)
  value.error ? item.reject(Error(value.error.message)) : item.resolve(value.result)
})
ws.addEventListener('close', () => {
  for (const item of pending.values()) { clearTimeout(item.timer); item.reject(Error('WebSocket closed')) }
  pending.clear()
})
function call(method, params) {
  const current = ++id
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => { pending.delete(current); reject(Error(`CDP deadline ${method}`)) }, 15000)
    pending.set(current, { resolve, reject, timer })
    ws.send(JSON.stringify({ id: current, method, params }))
  })
}
async function evaluate(expression) {
  const result = await call('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true })
  if (result.exceptionDetails) throw Error(JSON.stringify(result.exceptionDetails))
  return result.result.value
}
let repaired = false
let done = false
const started = performance.now()
try {
  const instrument = await evaluate(`(() => {
    const internals = window.__TAURI_INTERNALS__;
    if (!internals || typeof internals.invoke !== 'function') throw Error('Native bridge missing');
    const descriptor = Object.getOwnPropertyDescriptor(internals, 'invoke');
    if (!descriptor || (!descriptor.writable && !descriptor.configurable)) throw Error('Bridge is not observable without replacement');
    const original = internals.invoke;
    const events = [];
    const state = { installedAt: new Date().toISOString(), installedPerf: performance.now(), events,
      realBridgeForwarded: true, networkOrClockStubbed: false, descriptor: { writable: !!descriptor.writable, configurable: !!descriptor.configurable } };
    const observed = function(command, args, options) {
      const relevant = ['update_schedule_plan', 'check_for_update', 'official_remote_latest'].includes(command);
      const event = relevant ? { command, startedAt: new Date().toISOString(), startPerf: performance.now() } : null;
      if (event) events.push(event);
      let request;
      try { request = original.call(this, command, args, options); }
      catch (error) {
        if (event) { event.status = 'rejected'; event.endPerf = performance.now(); event.finishedAt = new Date().toISOString(); }
        throw error;
      }
      return Promise.resolve(request).then(value => {
        if (event) { event.status = 'fulfilled'; event.endPerf = performance.now(); event.finishedAt = new Date().toISOString();
          if (command === 'update_schedule_plan') event.plan = value; }
        return value;
      }, error => {
        if (event) { event.status = 'rejected'; event.endPerf = performance.now(); event.finishedAt = new Date().toISOString(); }
        throw error;
      });
    };
    Object.defineProperty(internals, 'invoke', { ...descriptor, value: observed });
    window.__ocxRealFloorObservation = state;
    return { installedAt: state.installedAt, installedPerf: state.installedPerf, descriptor: state.descriptor,
      documentVisible: document.visibilityState, hasFocus: document.hasFocus(), invokeReplacedByForwarderOnly: true };
  })()`)
  await writeFile(join(output, 'instrumentation.json'), JSON.stringify(instrument, null, 2) + '\n', { flag: 'wx' })
  await copyFile(join(launch.dataRoot, 'manager-state/update-schedule.json'), join(output, 'schedule-malformed.original.txt'))
  let firstFailure
  let lastWake = -1
  while (performance.now() - started < 1950_000) {
    const observation = await evaluate(`(() => { const s = window.__ocxRealFloorObservation; return {
      atUtc: new Date().toISOString(), nowPerf: performance.now(), documentVisible: document.visibilityState,
      hasFocus: document.hasFocus(), events: s.events, realBridgeForwarded: s.realBridgeForwarded,
      networkOrClockStubbed: s.networkOrClockStubbed }; })()`)
    await writeFile(join(output, 'observation.json'), JSON.stringify(observation, null, 2) + '\n')
    firstFailure ??= observation.events.find(event => event.command === 'update_schedule_plan' && event.status === 'rejected')
    if (firstFailure) {
      const elapsed = observation.nowPerf - firstFailure.endPerf
      if (!repaired && elapsed >= 10_000) {
        // Repair only the owned real schedule, preserving future deadlines for all
        // objects so the eventual real planner has no reason to query a source.
        const future = Math.floor(Date.now() / 1000) + 18 * 3600
        const state = { schema: 1, entries: Object.fromEntries(
          ['manager-stable', 'manager-beta', 'panel-latest'].map(key => [key, {
            next_due: future, failures: 0, last_success: Math.floor(Date.now() / 1000),
          }])) }
        await writeFile(join(launch.dataRoot, 'manager-state/update-schedule.json'), JSON.stringify(state))
        await writeFile(join(output, 'schedule-repair.json'), JSON.stringify({
          atUtc: new Date().toISOString(), realErrorEndPerf: firstFailure.endPerf,
          repairElapsedSinceRealErrorMs: elapsed, state, frontendTimerOrClockChanged: false,
        }, null, 2) + '\n', { flag: 'wx' })
        repaired = true
      }
      const stage = Math.floor(elapsed / 60_000)
      if (stage !== lastWake && stage <= 29) {
        // Exercise the real listener. No coordinator or timer is invoked directly.
        await evaluate(`(() => { window.dispatchEvent(new Event('online')); return true; })()`)
        lastWake = stage
      }
      const later = observation.events.filter(event => event.command === 'update_schedule_plan' && event.startPerf > firstFailure.startPerf)
      if (later.length) {
        const retry = later[0]
        if (retry.status) {
          const floor = retry.startPerf - firstFailure.endPerf
          const result = {
            atUtc: new Date().toISOString(), firstFailure, retry,
            realElapsedBeforeRetryMs: floor, expectedMinimumMs: 1800_000,
            noEarlyPlanCalls: floor >= 1800_000, stateActuallyRepaired: repaired,
            realRetryFulfilled: retry.status === 'fulfilled',
            queryEvents: observation.events.filter(event => event.command !== 'update_schedule_plan'),
            actualSystemClockOrTimerChanged: false, frontendOrNativeResultStubbed: false,
            instrumentedRealProductObservation: true, fullSchedulerGatePassed: false,
          }
          assert(result.noEarlyPlanCalls)
          assert(result.realRetryFulfilled)
          assert.equal(result.queryEvents.length, 0)
          await writeFile(join(output, 'result.json'), JSON.stringify(result, null, 2) + '\n', { flag: 'wx' })
          done = true
          break
        }
      }
      await writeFile(join(output, 'progress.json'), JSON.stringify({
        atUtc: new Date().toISOString(), phase: 'real-30-minute-floor',
        elapsedSinceRealErrorMs: elapsed, dueAtUtcEstimate: new Date(Date.now() + Math.max(0, 1800_000 - elapsed)).toISOString(),
        observedPlanCalls: observation.events.filter(event => event.command === 'update_schedule_plan').length,
        queryCalls: observation.events.filter(event => event.command !== 'update_schedule_plan').length,
        repaired, documentVisible: observation.documentVisible, hasFocus: observation.hasFocus,
      }, null, 2) + '\n')
    }
    await new Promise(resolve => setTimeout(resolve, 5000))
  }
  assert(done, 'No terminal real planner retry within the bounded observation window')
} catch (error) {
  await writeFile(join(output, 'failure.original.json'), JSON.stringify({
    atUtc: new Date().toISOString(), error: String(error), repaired, done, fullSchedulerGatePassed: false,
  }, null, 2) + '\n')
  process.exitCode = 1
} finally { ws.close() }
