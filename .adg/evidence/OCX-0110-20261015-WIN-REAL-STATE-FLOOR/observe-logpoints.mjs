import assert from 'node:assert/strict'
import { readFile, writeFile } from 'node:fs/promises'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { dirname, join } from 'node:path'
const output = dirname(fileURLToPath(import.meta.url))
const code = 'C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop'
const { discoverMainTarget } = await import(pathToFileURL(join(code, 'test/smoke/windows-cdp.mjs')))
const launch = JSON.parse((await readFile(join(output, 'state-floor-retry-launch.json'), 'utf8')).replace(/^\uFEFF/, ''))
const target = await discoverMainTarget(`http://127.0.0.1:${launch.port}`, { timeoutMs: 20000 })
const ws = new WebSocket(target.webSocketDebuggerUrl)
await new Promise((resolve, reject) => {
  const timer = setTimeout(() => reject(Error('WebSocket open deadline')), 5000)
  ws.addEventListener('open', () => { clearTimeout(timer); resolve() }, { once: true })
  ws.addEventListener('error', () => { clearTimeout(timer); reject(Error('WebSocket open failed')) }, { once: true })
})
let id = 0
let pauses = 0
const pending = new Map()
const breakpoints = []
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
  if (value.method === 'Debugger.paused') { pauses++; void call('Debugger.resume'); return }
  if (!value.id) return
  const item = pending.get(value.id)
  if (!item) return
  pending.delete(value.id); clearTimeout(item.timer)
  value.error ? item.reject(Error(value.error.message)) : item.resolve(value.result)
})
async function evaluate(expression, returnByValue = true) {
  const result = await call('Runtime.evaluate', { expression, returnByValue, awaitPromise: true })
  if (result.exceptionDetails) throw Error(JSON.stringify(result.exceptionDetails))
  return returnByValue ? result.result.value : result.result
}
let repaired = false
try {
  // The previous passive Network observer could not see Windows postMessage IPC.
  // Reload only this isolated WebView to establish a known scheduler start.
  await call('Page.reload', { ignoreCache: false })
  let ready = false
  for (let attempt = 0; attempt < 80; attempt++) {
    try { ready = await evaluate(`!!window.__TAURI_INTERNALS__?.postMessage && !!document.querySelector('#app')?.children.length`) }
    catch {}
    if (ready) break
    await new Promise(resolve => setTimeout(resolve, 100))
  }
  assert(ready, 'Reloaded isolated UI not ready')
  await call('Debugger.enable')
  const initial = await evaluate(`(() => {
    window.__ocxFloorLog = { events: [], installedAt:new Date().toISOString(),installedPerf:performance.now() };
    return {atUtc:new Date().toISOString(),nowPerf:performance.now(),visible:document.visibilityState,hasFocus:document.hasFocus()};
  })()`)
  assert(initial.nowPerf < 20000, 'Observer installed too late for the real startup grace')
  const post = await evaluate('window.__TAURI_INTERNALS__.postMessage', false)
  const callbacks = await evaluate('window.__TAURI_INTERNALS__.runCallback', false)
  const startCondition = `(() => { try {
    const m=arguments[0],s=window.__ocxFloorLog;
    if (m && ['update_schedule_plan','check_for_update','official_remote_latest'].includes(m.cmd)) {
      s.events.push({command:m.cmd,callback:m.callback,error:m.error,startedAtUtc:new Date().toISOString(),startPerf:performance.now()});
    }
  } catch {} return false; })()`
  const endCondition = `(() => { try {
    const id=arguments[0],s=window.__ocxFloorLog;
    const e=s.events.findLast(x=>!x.status&&(x.callback===id||x.error===id));
    if(e){e.status=e.error===id?'rejected':'fulfilled';e.endPerf=performance.now();e.finishedAtUtc=new Date().toISOString();
      if(e.command==='update_schedule_plan'&&e.status==='fulfilled'){e.plan=arguments[1];}
      if(e.status==='rejected'){e.errorKind=typeof arguments[1]==='object'?arguments[1]?.code:typeof arguments[1];}}
  } catch {} return false; })()`
  for (const [fn, condition] of [[post, startCondition], [callbacks, endCondition]]) {
    const result = await call('Debugger.setBreakpointOnFunctionCall', { objectId: fn.objectId, condition })
    breakpoints.push(result.breakpointId)
  }
  await writeFile(join(output, 'logpoint-method.json'), JSON.stringify({
    ...initial, schemaVersion: 1, scope: 'isolated WebView reload, production scheduler/native IPC unchanged',
    method: 'conditional function-call logpoints always returning false; original functions remain immutable',
    postCondition: startCondition, callbackCondition: endCondition, clockOrTimerChanged: false,
    stubbedResult: false, nativeBridgeReplaced: false, expectedDebuggerPauses: 0,
    priorNetworkMethodCouldNotObserveWindowsPostMessage: true,
  }, null, 2) + '\n', { flag: 'wx' })
  const started = performance.now()
  let firstFailure
  let lastWake = -1
  let complete = false
  while (performance.now() - started < 1950000) {
    const observation = await evaluate(`({atUtc:new Date().toISOString(),nowPerf:performance.now(),
      documentVisible:document.visibilityState,hasFocus:document.hasFocus(),events:window.__ocxFloorLog.events})`)
    await writeFile(join(output, 'logpoint-observation.json'), JSON.stringify({ ...observation, pauses }, null, 2) + '\n')
    assert.equal(pauses, 0, 'Debugger paused; timing experiment is not admissible')
    firstFailure ??= observation.events.find(event => event.command === 'update_schedule_plan' && event.status === 'rejected')
    if (!firstFailure && performance.now() - started > 90000) throw Error('No rejected real planner IPC observed within 90 seconds')
    if (firstFailure) {
      const elapsed = observation.nowPerf - firstFailure.endPerf
      if (!repaired && elapsed >= 10000) {
        const now = Math.floor(Date.now() / 1000)
        const state = { schema: 1, entries: Object.fromEntries(
          ['manager-stable', 'manager-beta', 'panel-latest'].map(key => [key, {
            next_due: now + 18 * 3600, failures: 0, last_success: now,
          }])) }
        await writeFile(join(launch.dataRoot, 'manager-state/update-schedule.json'), JSON.stringify(state))
        await writeFile(join(output, 'real-state-repair.json'), JSON.stringify({
          atUtc: new Date().toISOString(), elapsedSinceRealFailureMs: elapsed, state,
          frontendTimerChanged: false, nextDeadlinesAreFixtureOnly: true,
        }, null, 2) + '\n', { flag: 'wx' })
        repaired = true
      }
      const stage = Math.floor(elapsed / 60000)
      if (stage !== lastWake && stage <= 29) {
        await evaluate(`(() => { window.dispatchEvent(new Event('online')); return true; })()`)
        lastWake = stage
      }
      const retry = observation.events.find(event => event.command === 'update_schedule_plan' &&
        event.startPerf > firstFailure.startPerf)
      if (retry?.status) {
        const gap = retry.startPerf - firstFailure.endPerf
        const result = { atUtc: new Date().toISOString(), firstFailure, retry,
          realMonotonicRetryGapMs: gap, minimumMs: 1800000,
          noEarlyPlannerCalls: gap >= 1800000, realRetryFulfilled: retry.status === 'fulfilled',
          repaired, pauses, queryEvents: observation.events.filter(event => event.command !== 'update_schedule_plan'),
          frontendOrNativeResultStubbed: false, nativeBridgeReplaced: false, systemClockOrTimersChanged: false,
          syntheticOnlineWakesUsed: true, fullSchedulerGatePassed: false }
        await writeFile(join(output, 'logpoint-result.json'), JSON.stringify(result, null, 2) + '\n', { flag: 'wx' })
        assert(result.noEarlyPlannerCalls && result.realRetryFulfilled)
        assert.equal(result.queryEvents.length, 0)
        complete = true
        break
      }
      await writeFile(join(output, 'logpoint-progress.json'), JSON.stringify({
        atUtc: observation.atUtc, elapsedSinceRealFailureMs: elapsed,
        dueAtUtcEstimate: new Date(Date.now() + Math.max(0, 1800000 - elapsed)).toISOString(),
        observedPlanCalls: observation.events.filter(event => event.command === 'update_schedule_plan').length,
        queryCalls: observation.events.filter(event => event.command !== 'update_schedule_plan').length,
        pauses, repaired, documentVisible: observation.documentVisible, hasFocus: observation.hasFocus,
      }, null, 2) + '\n')
    }
    await new Promise(resolve => setTimeout(resolve, 5000))
  }
  assert(complete, 'No terminal planner retry within bounded interval')
} catch (error) {
  await writeFile(join(output, 'logpoint-failure.original.json'), JSON.stringify({
    atUtc: new Date().toISOString(), error: String(error), repaired, pauses, fullSchedulerGatePassed: false,
  }, null, 2) + '\n')
  process.exitCode = 1
} finally {
  for (const breakpointId of breakpoints) {
    try { await call('Debugger.removeBreakpoint', { breakpointId }) } catch {}
  }
  try { await call('Debugger.disable') } catch {}
  ws.close()
}
