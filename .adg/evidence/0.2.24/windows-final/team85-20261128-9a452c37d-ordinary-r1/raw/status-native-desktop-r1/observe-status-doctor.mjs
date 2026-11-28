import assert from 'node:assert/strict'
import { closeSync, openSync, writeFileSync, existsSync } from 'node:fs'
import { join } from 'node:path'
import { setTimeout as delay } from 'node:timers/promises'
import { discoverMainTarget } from '../observer/test/smoke/windows-cdp.mjs'

const [endpoint, sandbox, reportPath] = process.argv.slice(2)
const traceName = '__ocxTeam85NativeStatus'
const report = { accepted: false, sandbox, source: '1dac7c065413347ebc56b6123a8cf10c517449aa', startedUtc: new Date().toISOString(), firstError: null }
let lastKnownFrame = null
let firstUnknownFrame = null
const pending = new Map()
const breakpoints = []
let socket
let nextId = 0
let installed = false

function call(method, params = {}, timeoutMs = 1500) {
  const id = ++nextId
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      pending.delete(id)
      reject(Error(method + ' timed out'))
    }, timeoutMs)
    pending.set(id, message => {
      clearTimeout(timer)
      if (message.error) reject(Error(method + ': ' + message.error.message))
      else resolve(message.result)
    })
    socket.send(JSON.stringify({ id, method, params }))
  })
}

async function evaluate(expression, awaitPromise = false) {
  const result = await call('Runtime.evaluate', { expression, awaitPromise, returnByValue: true }, awaitPromise ? 12000 : 1500)
  assert(!result.exceptionDetails, result.exceptionDetails?.text)
  return result.result.value
}

function installTrace(key) {
  if (globalThis[key]) throw Error('Trace already installed')
  const bridge = window.__TAURI_INTERNALS__
  const tracked = ['app_about', 'runtime_preparation_status', 'discover_environment', 'get_status_snapshot']
  const ids = new Map()
  globalThis[key] = {
    sent: [], replies: [], invoke: bridge.invoke, ipc: bridge.ipc, runCallback: bridge.runCallback,
    send(input) {
      if (!tracked.includes(input?.cmd) || ids.has(input.callback)) return
      const row = { command: input.cmd, id: input.callback, at: performance.now(), origin: this.probe ?? 'ui' }
      this.sent.push(row)
      ids.set(input.callback, { row, ok: true })
      ids.set(input.error, { row, ok: false })
      row.errorId = input.error
    },
    receive(id, data) {
      const found = ids.get(id)
      if (!found) return
      const { row, ok } = found
      ids.delete(row.id)
      ids.delete(row.errorId)
      const at = performance.now()
      this.replies.push({ command: row.command, id: row.id, origin: row.origin, at, elapsedMs: at - row.at, ok,
        runtime: data?.matrix?.runtime ?? null, failure: data?.status_failure ?? null,
        canStart: data?.can_start ?? null, code: data?.code ?? null,
        phase: data?.phase ?? null, attemptId: data?.attemptId ?? null, gate: data?.gate ?? null })
    },
  }
  return true
}

function sample(key) {
  const trace = globalThis[key]
  const bridge = window.__TAURI_INTERNALS__
  const buttons = [...document.querySelectorAll('button')].filter(button => button.getClientRects().length)
  const labels = buttons.map(button => button.textContent.trim())
  const body = document.body?.innerText ?? ''
  return {
    unchanged: trace.invoke === bridge.invoke && trace.ipc === bridge.ipc && trace.runCallback === bridge.runCallback,
    sent: trace.sent, replies: trace.replies, at: performance.now(),
    unknown: body.includes('运行状态未知'), loading: body.includes('等待首次观测'),
    preparing: body.includes('正在处理本地运行资源'),
    retry: labels.includes('重试状态'), diagnostics: labels.includes('查看诊断'),
    start: buttons.filter(button => /^启动(?: OpenCodex)?$/.test(button.textContent.trim())).map(button => ({
      disabled: button.disabled || button.getAttribute('aria-disabled') === 'true',
    })),
  }
}

async function waitForSample(predicate, timeoutMs) {
  const deadline = performance.now() + timeoutMs
  while (performance.now() < deadline) {
    const row = await evaluate('(' + sample.toString() + ')(' + JSON.stringify(traceName) + ')')
    report.lastView = row
    if (row.unknown && row.retry && row.diagnostics && !row.loading && !row.preparing) {
      firstUnknownFrame ??= { at: row.at, previousAt: lastKnownFrame }
    } else {
      lastKnownFrame = row.at
      firstUnknownFrame = null
    }
    assert(row.unchanged, 'Tauri bridge changed during tracing')
    if (predicate(row)) return row
    await delay(100)
  }
  throw Error('Observation deadline expired')
}

function clickRetry() {
  const button = [...document.querySelectorAll('button')].find(button =>
    button.textContent.trim() === '重试状态' && button.getClientRects().length && !button.disabled)
  if (!button) return false
  button.click()
  return true
}

try {
  const target = await discoverMainTarget(endpoint, { pollIntervalMs: 50 })
  const ws = new URL(target.webSocketDebuggerUrl)
  assert.equal(ws.protocol, 'ws:')
  assert(['127.0.0.1', 'localhost'].includes(ws.hostname))
  assert.equal(ws.port, new URL(endpoint).port)
  socket = new WebSocket(ws)
  await new Promise((resolve, reject) => {
    const timer = setTimeout(() => reject(Error('CDP open timed out')), 5000)
    socket.addEventListener('open', () => { clearTimeout(timer); resolve() }, { once: true })
    socket.addEventListener('error', () => { clearTimeout(timer); reject(Error('CDP connection failed')) }, { once: true })
  })
  socket.addEventListener('message', ({ data }) => {
    const message = JSON.parse(data)
    const complete = pending.get(message.id)
    pending.delete(message.id)
    complete?.(message)
  })
  const key = JSON.stringify(traceName)
  const current = join(sandbox, 'program', 'runtime', 'current.json')
  const failures = report.failures = []
  async function record(stage, work) {
    try { await work() } catch (error) {
      failures.push({ stage, error: error.message })
      report.firstError ??= stage + ': ' + error.message
      process.exitCode = 1
    }
  }
  await record('traceSetup', async () => {
    const deadline = performance.now() + 5000
    while (!(await evaluate("typeof window.__TAURI_INTERNALS__?.ipc === 'function'"))) {
      assert(performance.now() < deadline, 'The Tauri bridge did not initialize')
      await delay(50)
    }
    await call('Debugger.enable')
    assert(await evaluate('(' + installTrace.toString() + ')(' + key + ')'))
    installed = true
    for (const [name, expression] of [['ipc', 'send(message)'], ['runCallback', 'receive(id,data)']]) {
      const { result, exceptionDetails } = await call('Runtime.evaluate', {
        expression: 'window.__TAURI_INTERNALS__.' + name, objectGroup: traceName,
      })
      assert(!exceptionDetails && result?.type === 'function' && result.objectId, 'Missing real bridge function ' + name)
      const resultOfBreakpoint = await call('Debugger.setBreakpointOnFunctionCall', {
        objectId: result.objectId, condition: '(globalThis[' + key + '].' + expression + ',false)',
      })
      breakpoints.push(resultOfBreakpoint.breakpointId)
    }
    report.attached = await waitForSample(() => true, 2000)
    const about = await evaluate("window.__TAURI_INTERNALS__.invoke('app_about').then(()=>true)", true)
    assert.equal(about, true)
    await waitForSample(row => row.replies.some(reply => reply.command === 'app_about' && reply.ok), 2000)
    report.sentinelVerified = true
  })
  await record('environmentReady', async () => {
    const observed = await waitForSample(row => existsSync(current) && row.replies.some(reply =>
      reply.origin === 'ui' && reply.command === 'runtime_preparation_status' && reply.ok && reply.phase === 'supplied'), 400000)
    const supplied = observed.replies.find(reply => reply.origin === 'ui' && reply.command === 'runtime_preparation_status' &&
      reply.ok && reply.phase === 'supplied')
    report.preparation = { ...supplied, observedUtc: new Date().toISOString() }
  })
  if (report.preparation && report.sentinelVerified) {
    await record('firstStatus', async () => {
      const first = await waitForSample(row => {
        const ready = row.replies.find(reply => reply.origin === 'ui' && reply.command === 'discover_environment' &&
          reply.ok && reply.gate === 'ready' && row.sent.some(request => request.id === reply.id && request.at >= report.preparation.at))
        const status = ready && row.sent.find(request => request.origin === 'ui' && request.command === 'get_status_snapshot' && request.at >= ready.at)
        if (!status || row.loading || row.preparing) return false
        const reply = row.replies.find(candidate => candidate.id === status.id) ?? null
        const confirmed = reply?.ok && !reply.failure && reply.elapsedMs <= 5000 &&
          reply.runtime && !['loading', 'unreachable', 'unknown'].includes(reply.runtime)
        report.automatic = { ready, status, reply, uiObservedElapsedMs: row.at - status.at }
        return confirmed || (row.unknown && row.retry && row.diagnostics)
      }, 12000)
      report.first = first
      if (!first.unknown) return
      const { status } = report.automatic
      assert(first.start.every(button => button.disabled), 'Start is available while Status is unknown')
      assert(firstUnknownFrame && firstUnknownFrame.at >= status.at, 'No fresh unknown view sampled after Status')
      const upper = firstUnknownFrame.at - status.at
      const lower = Math.max(0, (firstUnknownFrame.previousAt ?? status.at) - status.at)
      report.automatic.unknownObservedElapsedMs = upper
      report.automatic.unknownElapsedLowerBoundMs = lower
      report.automatic.unknownBudget = upper <= 5000 ? 'within' : lower >= 5000 ? 'over' : 'inconclusive'
      assert(lower < 5000, 'UI still lacked the unknown state at or after 5 seconds')
    })
    await record('statusRetry', async () => {
      if (!report.lastView?.retry) return
      const beforeRetry = await evaluate('performance.now()')
      assert(await evaluate('(' + clickRetry.toString() + ')()'))
      report.retry = await waitForSample(row => row.replies.some(reply => reply.command === 'get_status_snapshot' &&
        row.sent.some(request => request.id === reply.id && request.at >= beforeRetry)), 9000)
      report.retryReplies = report.retry.replies.filter(reply => reply.command === 'get_status_snapshot' &&
        report.retry.sent.some(request => request.id === reply.id && request.at >= beforeRetry))
    })
  }
} catch (error) {
  report.firstError = error.message
  process.exitCode = 1
} finally {
  for (const breakpointId of breakpoints) {
    await call('Debugger.removeBreakpoint', { breakpointId }).catch(() => {})
  }
  if (installed) {
    await evaluate('delete globalThis[' + JSON.stringify(traceName) + ']').catch(() => {})
    await call('Runtime.releaseObjectGroup', { objectGroup: traceName }).catch(() => {})
  }
  socket?.close()
  report.finishedUtc = new Date().toISOString()
  const fd = openSync(reportPath, 'wx')
  try { writeFileSync(fd, JSON.stringify(report, null, 2) + '\n') } finally { closeSync(fd) }
}