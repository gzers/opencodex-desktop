// Passive WebView observation. This never launches the product, navigates,
// invokes a production command, replaces the IPC bridge, or changes timers.
import assert from 'node:assert/strict'
import { createHash } from 'node:crypto'
import { appendFile, mkdir, readFile, writeFile } from 'node:fs/promises'
import { dirname, isAbsolute, join, relative, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { discoverMainTarget } from './windows-cdp.mjs'

const key = '__ocxWindowsSchedulerObservation'
const commands = ['update_schedule_plan', 'check_for_update', 'official_remote_latest']
const admittedImages = {
  de6862d40389d527527b9db8ef52e186a58b625f: '626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608',
  '8e69ff6db69ea13b8cb29feb9f7a1abacd202a7b': '870cb30c62c0cf563edacf7fe30a35d3b66cbffbb574beeb6b354ed2c7fab15a',
}

export function requireObservationWindow(authorization, now = Date.now()) {
  assert.equal(authorization.origin, 'human', 'Current human authorization required')
  assert.equal(authorization.explicitHumanStart, true, 'Explicit human start required')
  assert.equal(authorization.automaticContinuation, false, 'Automatic continuation is not human approval')
  const start = Date.parse(authorization.authorizedAtUtc)
  const end = Date.parse(authorization.expiresAtUtc)
  assert(Number.isFinite(start) && Number.isFinite(end) && end > start &&
    end - start <= 30 * 60_000 && start <= now && now < end, 'Human window expired or invalid')
  const close = Math.min(start + 25 * 60_000, end)
  assert(now < close, 'Normal cleanup reservation has started')
  return close
}

export function observationExpressions({ maximumRecords = 100_000 } = {}) {
  assert(Number.isSafeInteger(maximumRecords) && maximumRecords >= 1 && maximumRecords <= 100_000)
  const state = `window[${JSON.stringify(key)}]`
  const install = `(() => {
    if (${state}) throw Error('An observation already exists; never silently replace it');
    const bridge = window.__TAURI_INTERNALS__;
    if (typeof bridge?.postMessage !== 'function' || typeof bridge?.runCallback !== 'function')
      throw Error('Original native IPC bridge unavailable');
    const s = { installedAtUtc:new Date().toISOString(), installedPerf:performance.now(),
      events:[], frames:[], longTasks:[], routes:[], dropped:0, errors:0, active:true,nextCall:0,
      pending:[],
      postMessage:bridge.postMessage, runCallback:bridge.runCallback,
      originalRaf:window.requestAnimationFrame, originalSetTimeout:window.setTimeout,
      originalInvoke:bridge.invoke, maximumRecords:${maximumRecords},
      initialVisibility:document.visibilityState, initialFocus:document.hasFocus(),
      initialRoute:location.hash, initialTimeOrigin:performance.timeOrigin,
      initialSupportedEntries:[...PerformanceObserver.supportedEntryTypes] };
    s.append = (list, row) => {
      if (s.events.length+s.frames.length+s.longTasks.length+s.routes.length >= s.maximumRecords)
        { s.dropped++; return false; }
      list.push(row);return true;
    };
    Object.defineProperty(window, ${JSON.stringify(key)}, { value:s, configurable:true });
    if (!s.initialSupportedEntries.includes('longtask')) throw Error('Long-task observer unsupported');
    s.observer = new PerformanceObserver(list => {
      for (const e of list.getEntries()) s.append(s.longTasks,
        {startMs:e.startTime,durationMs:e.duration,name:e.name});
    });
    s.observer.observe({type:'longtask', buffered:false});
    let previous=null;
    s.tick = time => {
      if (!s.active) return;
      s.append(s.frames,{atMs:time,intervalMs:previous===null?null:time-previous,
        visible:document.visibilityState,focused:document.hasFocus()});
      previous=time;
      s.raf=s.originalRaf.call(window,s.tick);
    };
    s.route = event => s.append(s.routes,{type:event.type,atMs:performance.now(),hash:location.hash});
    window.addEventListener('hashchange',s.route);
    window.addEventListener('popstate',s.route);
    s.raf=s.originalRaf.call(window,s.tick);
    return {installedAtUtc:s.installedAtUtc,installedPerf:s.installedPerf,
      initialTimeOrigin:s.initialTimeOrigin,initialVisibility:s.initialVisibility,
      initialFocus:s.initialFocus,initialRoute:s.initialRoute,
      initialSupportedEntries:s.initialSupportedEntries};
  })()`
  // Always-false function-call conditions record actual calls without pausing.
  // Do not serialize payloads, results, URL queries, source credentials or tokens.
  const startCondition = `(() => { try {
    const s=${state},m=arguments[0];
    if (s?.active && m && ${JSON.stringify(commands)}.includes(m.cmd)) {
      const e={phase:'begin',callId:++s.nextCall,command:m.cmd,callback:m.callback,error:m.error,
        trigger:typeof m.payload?.trigger==='string'?m.payload.trigger:null,
        atMs:performance.now(),atUtc:new Date().toISOString()};
      if(s.pending.length>=s.maximumRecords){s.dropped++;}
      else if(s.append(s.events,e))s.pending.push(e);
    }
  } catch { if (${state}) ${state}.errors++; } return false; })()`
  const endCondition = `(() => { try {
    const s=${state},id=arguments[0];
    if (s?.active) {
      const index=s.pending.findLastIndex(x=>x.callback===id||x.error===id),e=s.pending[index];
      if(e){s.pending.splice(index,1);s.append(s.events,{phase:'end',callId:e.callId,command:e.command,callback:id,
        status:e.error===id?'rejected':'fulfilled',atMs:performance.now(),atUtc:new Date().toISOString()});}
    }
  } catch { if (${state}) ${state}.errors++; } return false; })()`
  const snapshot = `(() => {
    const s=${state};
    if(!s)throw Error('Observation context lost');
    const delta={};
    for(const name of ['events','frames','longTasks','routes']){
      delta[name]=s[name];s[name]=[];
    }
    return {installedAtUtc:s.installedAtUtc,installedPerf:s.installedPerf,armedPerf:s.armedPerf,nowPerf:performance.now(),
      timeOrigin:performance.timeOrigin,...delta,
      dropped:s.dropped,errors:s.errors,visibility:document.visibilityState,focused:document.hasFocus(),
      bridgeUnchanged:s.postMessage===window.__TAURI_INTERNALS__.postMessage &&
        s.runCallback===window.__TAURI_INTERNALS__.runCallback && s.originalInvoke===window.__TAURI_INTERNALS__.invoke,
      clockAndRafUnchanged:s.originalRaf===window.requestAnimationFrame &&
        s.originalSetTimeout===window.setTimeout};
  })()`
  const cleanup = `(() => {
    const s=${state};if(!s)return false;
    s.active=false;cancelAnimationFrame(s.raf);s.observer?.disconnect();
    window.removeEventListener('hashchange',s.route);window.removeEventListener('popstate',s.route);
    delete ${state};return true;
  })()`
  return { install, startCondition, endCondition, snapshot, cleanup, state }
}

export function summarizeObservation(snapshot, { pauses = 0, includeFrameStatistics = true } = {}) {
  assert.equal(snapshot.dropped, 0, 'Observation overflow; zero counts are not admissible')
  assert.equal(snapshot.errors, 0, 'Observer condition error')
  assert.equal(pauses, 0, 'Debugger pause invalidates this observation')
  assert.equal(snapshot.bridgeUnchanged, true, 'Native IPC bridge changed')
  assert.equal(snapshot.clockAndRafUnchanged, true, 'Application timer or RAF changed')
  const begin = snapshot.events.filter(event => event.phase === 'begin')
  const finished = new Set(snapshot.events.filter(event => event.phase === 'end').map(event => event.callId))
  const queryCalls = begin.filter(event => event.command !== 'update_schedule_plan')
  const foregroundFrames = snapshot.frames.filter(frame =>
    frame.intervalMs !== null && frame.visible === 'visible' && frame.focused)
  const intervals = includeFrameStatistics ? foregroundFrames.map(frame => frame.intervalMs).sort((a, b) => a - b) : []
  const p95 = intervals.length ? intervals[Math.ceil(intervals.length * .95) - 1] : null
  return {
    observedStartPerf: snapshot.armedPerf, installedPerf: snapshot.installedPerf, observedEndPerf: snapshot.nowPerf,
    plannerCalls: begin.filter(event => event.command === 'update_schedule_plan').length,
    queryCalls: queryCalls.length,
    queryCommands: Object.fromEntries(commands.slice(1).map(command =>
      [command, queryCalls.filter(event => event.command === command).length])),
    pendingObservedCalls: begin.filter(event => !finished.has(event.callId)).length,
    foregroundFrameIntervals: foregroundFrames.length, foregroundRafIntervalP95Ms: p95,
    frameStatisticsDeferred: !includeFrameStatistics,
    longTasks: snapshot.longTasks.length,
    mainThreadTasksOver50Ms: snapshot.longTasks.filter(task => task.durationMs > 50).length,
    rawRouteEvents: snapshot.routes.length,
    bridgeReplaced: false, clockOrRafReplaced: false, resultStubbed: false,
    httpRequestOrByteCountClaimed: false, subprocessCountClaimed: false,
    intervalBeforeObserverCovered: false, fullPerformanceGatePassed: false,
  }
}

export class ObservationCdp {
  constructor(socket) {
    this.socket = socket
    this.sequence = 0
    this.pending = new Map()
    this.pauses = 0
    socket.addEventListener('message', event => {
      const message = JSON.parse(event.data)
      if (message.method === 'Debugger.paused') {
        this.pauses++
        // Release an unexpected pause immediately, then reject the measurement.
        void this.call('Debugger.resume').catch(() => {})
      }
      const pending = this.pending.get(message.id)
      if (!pending) return
      this.pending.delete(message.id)
      clearTimeout(pending.timer)
      message.error ? pending.reject(Error(message.error.message)) : pending.resolve(message.result)
    })
    socket.addEventListener('close', () => {
      for (const pending of this.pending.values()) {
        clearTimeout(pending.timer); pending.reject(Error('Observation target disconnected'))
      }
      this.pending.clear()
    })
  }
  static async open(url) {
    const endpoint = new URL(url)
    assert(endpoint.protocol === 'ws:' && endpoint.hostname === '127.0.0.1' && endpoint.port &&
      !endpoint.username && !endpoint.password, 'Exact loopback WebSocket required')
    const socket = new WebSocket(url)
    await new Promise((resolveOpen, reject) => {
      const timer = setTimeout(() => { socket.close(); reject(Error('CDP connect deadline')) }, 5000)
      socket.addEventListener('open', () => { clearTimeout(timer); resolveOpen() }, { once: true })
      socket.addEventListener('error', () => { clearTimeout(timer); reject(Error('CDP connect failed')) }, { once: true })
    })
    return new ObservationCdp(socket)
  }
  call(method, params = {}) {
    const id = ++this.sequence
    return new Promise((resolveCall, reject) => {
      const timer = setTimeout(() => { this.pending.delete(id); reject(Error(`CDP deadline ${method}`)) }, 10_000)
      this.pending.set(id, { resolve: resolveCall, reject, timer })
      try { this.socket.send(JSON.stringify({ id, method, params })) }
      catch (error) { clearTimeout(timer); this.pending.delete(id); reject(error) }
    })
  }
  async evaluate(expression, returnByValue = true) {
    const result = await this.call('Runtime.evaluate', { expression, returnByValue, awaitPromise: true })
    assert(!result.exceptionDetails, JSON.stringify(result.exceptionDetails))
    return returnByValue ? result.result.value : result.result
  }
}

export async function attachSchedulerObserver(client, options) {
  const expressions = observationExpressions(options)
  const breakpoints = []
  try {
    const installation = await client.evaluate(expressions.install)
    await client.call('Debugger.enable')
    for (const [member, condition] of [
      ['postMessage', expressions.startCondition], ['runCallback', expressions.endCondition],
    ]) {
      const remote = await client.evaluate(`window.__TAURI_INTERNALS__.${member}`, false)
      assert(remote.objectId && remote.type === 'function', 'Native bridge is not a function')
      const result = await client.call('Debugger.setBreakpointOnFunctionCall', { objectId: remote.objectId, condition })
      breakpoints.push(result.breakpointId)
    }
    installation.armedPerf = await client.evaluate(`(${expressions.state}.armedPerf=performance.now())`)
    const accumulated = { events: [], frames: [], longTasks: [], routes: [] }
    const snapshot = async ({ includeFrameStatistics = true } = {}) => {
      const delta = await client.evaluate(expressions.snapshot)
      for (const name of ['events', 'frames', 'longTasks', 'routes']) accumulated[name].push(...delta[name])
      const raw = { ...delta, ...accumulated }
      return { raw, delta, summary: summarizeObservation(raw, { pauses: client.pauses, includeFrameStatistics }) }
    }
    return { installation, snapshot, cleanup: async () => {
      const errors = []
      for (const breakpointId of breakpoints) {
        try { await client.call('Debugger.removeBreakpoint', { breakpointId }) } catch (error) { errors.push(String(error)) }
      }
      try { await client.evaluate(expressions.cleanup) } catch (error) { errors.push(String(error)) }
      try { await client.call('Debugger.disable') } catch (error) { errors.push(String(error)) }
      return { observerRemoved: errors.length === 0, errors, pauses: client.pauses }
    } }
  } catch (error) {
    for (const breakpointId of breakpoints) {
      try { await client.call('Debugger.removeBreakpoint', { breakpointId }) } catch {}
    }
    try { await client.evaluate(expressions.cleanup) } catch {}
    try { await client.call('Debugger.disable') } catch {}
    throw error
  }
}

async function observePlan(path) {
  const plan = JSON.parse((await readFile(path, 'utf8')).replace(/^\uFEFF/, ''))
  const closeAt = requireObservationWindow(plan.authorization)
  const repo = resolve(dirname(fileURLToPath(import.meta.url)), '../..')
  const output = resolve(plan.outputDirectory)
  const owned = relative(join(repo, 'test/runtime'), output)
  assert(owned && !owned.startsWith('..') && !isAbsolute(owned), 'Fresh task runtime output required')
  assert(plan.binding?.exeSha256 && admittedImages[plan.binding?.source] === plan.binding.exeSha256,
  'Admitted product binding required')
  // Binding is supplied by the retained-handle native controller. CDP cannot
  // independently prove a PID, token or executable hash; report that boundary.
  assert(plan.binding.retainedHandleIdentityFile && plan.stopFile, 'Native identity and explicit stop endpoints required')
  const nativeBytes = await readFile(plan.binding.retainedHandleIdentityFile)
  assert.equal(createHash('sha256').update(nativeBytes).digest('hex'), plan.binding.identitySha256,
    'Native listener/product binding bytes differ')
  const native = JSON.parse(nativeBytes.toString('utf8').replace(/^\uFEFF/, ''))
  assert(native.schemaVersion === 1 && native.source === plan.binding.source &&
    native.exeSha256 === plan.binding.exeSha256 && native.retainedProductHandleVerified === true &&
    native.loopbackListenerVerified === true && native.listenerIsOwnedWebViewDescendant === true &&
    native.nativeProductCount === 1 && native.product.elevated === false &&
    native.product.elevationType === 3 && native.debugPort === Number(new URL(plan.endpoint).port),
  'Exact ordinary product/listener evidence required')
  assert(Date.parse(native.observedAtUtc) >= Date.parse(plan.authorization.authorizedAtUtc) &&
    Date.parse(native.product.creationTimeUtc) >= Date.parse(plan.authorization.authorizedAtUtc),
  'Native binding predates the fresh human window')
  assert.equal(createHash('sha256').update(await readFile(native.product.executable)).digest('hex'),
    plan.binding.exeSha256, 'Actual product executable digest differs')
  await mkdir(output)
  await writeFile(join(output, 'plan.original.json'), JSON.stringify(plan, null, 2) + '\n', { flag: 'wx' })
  let client
  let observation
  let lastRaw
  let finished = false
  try {
    const target = await discoverMainTarget(plan.endpoint)
    assert.equal(new URL(target.webSocketDebuggerUrl).port, new URL(plan.endpoint).port)
    client = await ObservationCdp.open(target.webSocketDebuggerUrl)
    observation = await attachSchedulerObserver(client)
    await writeFile(join(output, 'observer-ready.json'), JSON.stringify({
      ...observation.installation, targetId: target.id,
      source: plan.binding.source, nativeBindingReceiptSha256: plan.binding.identitySha256,
      nativeIdentityVerifiedLiveByThisObserver: false,
    }, null, 2) + '\n', { flag: 'wx' })
    for (;;) {
      const record = await observation.snapshot({ includeFrameStatistics: false })
      lastRaw = record.raw
      // Stream deltas rather than repeatedly serializing every historical frame
      // inside the measured WebView. Summarization runs in this external process.
      await appendFile(join(output, 'observations.original.jsonl'), JSON.stringify(record.delta) + '\n')
      await writeFile(join(output, 'observation-summary.json'), JSON.stringify(record.summary, null, 2) + '\n')
      if (Date.now() >= closeAt) break
      try { await readFile(plan.stopFile); break } catch (error) { if (error.code !== 'ENOENT') throw error }
      await new Promise(resolveDelay => setTimeout(resolveDelay, 1000))
    }
    finished = true
  } catch (error) {
    await writeFile(join(output, 'failure.original.json'), JSON.stringify({
      error: String(error), pauses: client?.pauses ?? 0, fullPerformanceGatePassed: false,
    }, null, 2) + '\n', { flag: 'wx' })
    throw error
  } finally {
    const cleanup = observation ? await observation.cleanup() : { observerInstalled: false }
    await writeFile(join(output, 'observer-cleanup.json'), JSON.stringify(cleanup, null, 2) + '\n', { flag: 'wx' })
    client?.socket.close()
    if (finished && lastRaw) {
      await writeFile(join(output, 'observation-summary.json'),
        JSON.stringify(summarizeObservation(lastRaw, { pauses: client.pauses }), null, 2) + '\n')
    }
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  if (process.argv[2] === '--observe' && process.argv[3]) {
    await observePlan(process.argv[3])
  } else {
    console.log(JSON.stringify({ status: 'prepared-only', productStarted: false,
      nativeUiDriven: false, etwSessionStarted: false, authorizationRequiredForObserve: true }))
  }
}
