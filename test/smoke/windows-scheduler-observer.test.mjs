import assert from 'node:assert/strict'
import { fork } from 'node:child_process'
import { once } from 'node:events'
import test from 'node:test'
import { fileURLToPath } from 'node:url'
import {
  attachSchedulerObserver, ObservationCdp, requireObservationWindow, summarizeObservation,
} from './windows-scheduler-observer.mjs'

test('expired, future and automatic windows cannot authorize observation', () => {
  const start = Date.parse('2026-10-18T00:00:00Z')
  const valid = { origin: 'human', explicitHumanStart: true, automaticContinuation: false,
    authorizedAtUtc: new Date(start).toISOString(), expiresAtUtc: new Date(start + 30 * 60_000).toISOString() }
  assert.equal(requireObservationWindow(valid, start + 1000), start + 25 * 60_000)
  for (const [authorization, now] of [
    [{ ...valid, automaticContinuation: true }, start + 1000],
    [{ ...valid, explicitHumanStart: false }, start + 1000],
    [{ ...valid, origin: 'automation' }, start + 1000],
    [valid, start - 1], [valid, start + 25 * 60_000], [valid, start + 30 * 60_000],
    [{ ...valid, expiresAtUtc: new Date(start + 31 * 60_000).toISOString() }, start + 1000],
  ]) assert.throws(() => requireObservationWindow(authorization, now))
})

test('a zero count with lost records, changed hooks or debugger pauses is rejected', () => {
  const snapshot = { events: [], frames: [], longTasks: [], routes: [], dropped: 0, errors: 0,
    bridgeUnchanged: true, clockAndRafUnchanged: true, installedPerf: 100, armedPerf: 110, nowPerf: 200 }
  assert.equal(summarizeObservation(snapshot).queryCalls, 0)
  for (const changed of [
    { dropped: 1 }, { errors: 1 }, { bridgeUnchanged: false }, { clockAndRafUnchanged: false },
  ]) assert.throws(() => summarizeObservation({ ...snapshot, ...changed }))
  assert.throws(() => summarizeObservation(snapshot, { pauses: 1 }))
  const sample = summarizeObservation({ ...snapshot,
    frames: [
      { intervalMs: 1, visible: 'hidden', focused: true },
      ...Array.from({ length: 20 }, (_, index) => ({ intervalMs: index + 1, visible: 'visible', focused: true })),
    ],
    longTasks: [{ durationMs: 50 }, { durationMs: 50.1 }],
  })
  assert.equal(sample.foregroundRafIntervalP95Ms, 19)
  assert.equal(sample.mainThreadTasksOver50Ms, 1)
  assert.equal(sample.httpRequestOrByteCountClaimed, false)
  assert.equal(sample.subprocessCountClaimed, false)
})

async function inspectorTarget(t) {
  const child = fork(fileURLToPath(new URL('./fixtures/windows-observer-target.mjs', import.meta.url)), {
    silent: true, windowsHide: true, execArgv: ['--inspect=127.0.0.1:0'], env: { ...process.env, NODE_OPTIONS: '' },
  })
  let client
  const exited = once(child, 'exit')
  t.after(async () => {
    client?.socket.close()
    if (child.exitCode === null) child.send('exit')
    const [code, signal] = await exited
    assert.equal(code, 0)
    assert.equal(signal, null)
  })
  const endpoint = new Promise((resolve, reject) => {
    let text = ''
    const timer = setTimeout(() => reject(Error('Isolated inspector did not become ready')), 10_000)
    child.stderr.on('data', chunk => {
      text += chunk
      const match = /Debugger listening on (ws:\/\/127\.0\.0\.1:\d+\/[^\s]+)/.exec(text)
      if (match) { clearTimeout(timer); resolve(match[1]) }
    })
  })
  await once(child, 'message')
  client = await ObservationCdp.open(await endpoint)
  return { child, client }
}

test('real V8 function-call logpoints capture sub-millisecond calls without replacing the frozen bridge', async t => {
  const { child, client } = await inspectorTarget(t)
  const observer = await attachSchedulerObserver(client)
  const before = await observer.snapshot()
  assert.equal(before.summary.queryCalls, 0)
  const complete = once(child, 'message')
  child.send('calls')
  assert.deepEqual(await complete, ['calls-complete', undefined])
  const { raw, summary } = await observer.snapshot()
  assert.equal(summary.queryCalls, 26)
  assert.equal(summary.plannerCalls, 1)
  assert.equal(summary.queryCommands.check_for_update, 25)
  assert.equal(summary.queryCommands.official_remote_latest, 1)
  assert.equal(summary.pendingObservedCalls, 0)
  assert.equal(raw.events.filter(event => event.phase === 'end' && event.status === 'rejected').length, 1)
  assert.equal(raw.events.filter(event => event.phase === 'begin' && event.trigger === 'automatic').length, 25)
  assert.equal(JSON.stringify(raw).includes('NOT_TO_BE_SERIALIZED'), false)
  assert.equal(raw.bridgeUnchanged, true)
  assert.equal(client.pauses, 0)
  assert.deepEqual(await observer.cleanup(), { observerRemoved: true, errors: [], pauses: 0 })
  assert.equal(await client.evaluate(`Object.getOwnPropertyDescriptor(window.__TAURI_INTERNALS__,'postMessage').writable`), false)
  assert.equal(await client.evaluate(`typeof window.__ocxWindowsSchedulerObservation`), 'undefined')
})

test('real logpoint overflow fails closed and still removes observation', async t => {
  const { child, client } = await inspectorTarget(t)
  const observer = await attachSchedulerObserver(client, { maximumRecords: 3 })
  const complete = once(child, 'message')
  child.send('calls')
  await complete
  await assert.rejects(observer.snapshot(), /overflow/)
  assert.equal((await observer.cleanup()).observerRemoved, true)
  assert.equal(client.pauses, 0)
})

test('asynchronous call completion survives drained batches without duplicating old records', async t => {
  const { child, client } = await inspectorTarget(t)
  const observer = await attachSchedulerObserver(client)
  let complete = once(child, 'message')
  child.send('begin-only')
  await complete
  const first = await observer.snapshot()
  assert.equal(first.summary.queryCalls, 1)
  assert.equal(first.summary.pendingObservedCalls, 1)
  assert.equal(first.delta.events.length, 1)
  complete = once(child, 'message')
  child.send('end-only')
  await complete
  const second = await observer.snapshot()
  assert.equal(second.summary.queryCalls, 1)
  assert.equal(second.summary.pendingObservedCalls, 0)
  assert.equal(second.raw.events.length, 2)
  assert.equal(second.delta.events.length, 1)
  assert.equal(second.delta.events[0].phase, 'end')
  assert.equal((await observer.snapshot()).delta.events.length, 0)
  assert.equal((await observer.cleanup()).observerRemoved, true)
})

test('a closed target rejects new commands immediately without dangling cleanup deadlines', async t => {
  const { client } = await inspectorTarget(t)
  const closed = new Promise(resolve => client.socket.addEventListener('close', resolve, { once: true }))
  client.socket.close()
  await closed
  await assert.rejects(client.call('Debugger.disable'), /disconnected/)
  assert.equal(client.pending.size, 0)
})
