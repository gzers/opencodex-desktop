import assert from 'node:assert/strict'
import test from 'node:test'
import { setTimeout as delay } from 'node:timers/promises'
import { runInNewContext } from 'node:vm'
import { observePreparation, readPreparationFrame } from './windows024-preparation-observer.mjs'

const firstAttempt = '9a3c59cf-6e03-45b7-8944-c82a8949f0e0'
const secondAttempt = 'e59d3400-249c-4f29-a4f4-79ac79cf7104'
const immediateTerminal = { pollMs: 1, settleMs: 0 }

function frame(phase, revision, signals = {}, attemptId = firstAttempt) {
  const preparation = { phase, revision, attemptId, failedPhase: null, failure: null }
  return { preparation, localHintVisible: false, stepsVisible: false, ...signals }
}

test('records changes and a separate terminal outcome for the sampled attempt', async () => {
  const progress = frame('extracting', 2, { localHintVisible: true, stepsVisible: true })
  const input = [frame('idle', 0, {}, null), progress, progress, frame('supplied', 3)]
  const rows = []
  await observePreparation(() => input.shift(), row => rows.push(row), immediateTerminal)
  assert.deepEqual(rows.map(row => [row.kind, row.phase]), [
    ['sample', 'idle'], ['sample', 'extracting'], ['sample', 'supplied'], ['end', 'supplied'],
  ])
  assert.equal(rows.at(-1).attemptId, firstAttempt)
  assert.equal(rows.at(-1).localProgressObserved, true)
  assert(rows.every((row, index) => !index || row.elapsedMs >= rows[index - 1].elapsedMs))
})

test('terminal-only samples do not claim visible local progress', async () => {
  const rows = []
  await observePreparation(() => frame('supplied', 6), row => rows.push(row), immediateTerminal)
  assert.equal(rows.at(-1).localProgressObserved, false)
})

test('a new terminal attempt gets its own settle window and progress evidence', async () => {
  let reads = 0
  const rows = []
  const failed = frame('failed', 2)
  failed.preparation.failure = 'missing_archive'
  failed.preparation.failedPhase = 'checking'
  const supplied = frame('supplied', 4, {}, secondAttempt)
  await observePreparation(async () => {
    ++reads
    if (reads === 1) return frame('checking', 1, { localHintVisible: true, stepsVisible: true })
    if (reads === 2) return failed
    await delay(35)
    return supplied
  }, row => rows.push(row), { pollMs: 1, settleMs: 25 })
  assert.equal(reads, 4)
  assert.equal(rows.find(row => row.phase === 'failed').failure, 'missing_archive')
  assert.equal(rows.at(-1).attemptId, secondAttempt)
  assert.equal(rows.at(-1).localProgressObserved, false)
})

test('retains samples on deadline and refuses a terminal frame received too late', async () => {
  const rows = []
  let reads = 0
  await assert.rejects(observePreparation(async remainingMs => {
    assert(remainingMs > 0 && remainingMs <= 100)
    if (++reads === 1) return frame('checking', 1)
    await delay(remainingMs + 10)
    return frame('supplied', 2)
  }, row => rows.push(row), { timeoutMs: 100, ...immediateTerminal }), /within its own deadline/)
  assert.deepEqual(rows.map(row => row.phase), ['checking', 'observation_timeout'])
  assert.equal(rows.at(-1).attemptId, firstAttempt)
})

test('stops at unknown phases and invalid attempt identities', async () => {
  for (const [sample, error] of [
    [frame('unknown', 1), /Unknown preparation phase/],
    [frame('checking', 1, {}, 'invalid'), /Invalid preparation attempt ID/],
  ]) {
    const rows = []
    await assert.rejects(observePreparation(() => sample, row => rows.push(row), immediateTerminal), error)
    assert.deepEqual(rows, [])
  }
})

test('the serialized WebView frame reads only preparation status and visible card elements', async () => {
  const commands = []
  const localHint = { getClientRects: () => [1], textContent: '正在处理本地运行资源，无需联网。' }
  const steps = { getClientRects: () => [1] }
  const card = {
    getClientRects: () => [1],
    querySelectorAll(selector) {
      assert.equal(selector, 'p')
      return [localHint]
    },
    querySelector(selector) {
      assert.equal(selector, 'ol[aria-label="内置环境准备阶段"]')
      return steps
    },
  }
  const preparation = frame('verifying', 3).preparation
  const context = {
    window: { __TAURI_INTERNALS__: { invoke(command) {
      commands.push(command)
      return { ...preparation, futureField: 'must not leave the page' }
    } } },
    document: { querySelectorAll(selector) {
      assert.equal(selector, '[data-testid="integrated-preparation"]')
      return [card]
    } },
    getComputedStyle: element => ({ visibility: element === steps ? 'hidden' : 'visible' }),
  }
  const result = await runInNewContext(
    '(' + readPreparationFrame.toString() + ')()', context,
  )
  assert.deepEqual(JSON.parse(JSON.stringify(result)), { preparation, localHintVisible: true, stepsVisible: false })
  assert.deepEqual(commands, ['runtime_preparation_status'])
})
