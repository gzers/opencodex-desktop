import assert from 'node:assert/strict'
import { setTimeout as delay } from 'node:timers/promises'

const localPhases = ['checking', 'preparing', 'verifying', 'extracting', 'assembling', 'activating']
const terminalPhases = ['supplied', 'failed', 'removed', 'review']
const phases = ['idle', 'downloading', ...localPhases, ...terminalPhases]
const failureKinds = ['integrity', 'busy', 'recovery_required', 'unavailable', 'missing_archive']

// Evaluated in the real WebView. Only read the backend snapshot and the rendered preparation card.
export async function readPreparationFrame() {
  const { attemptId, revision, phase, failedPhase, failure } =
    await window.__TAURI_INTERNALS__.invoke('runtime_preparation_status')
  const visible = element => Boolean(element?.getClientRects().length &&
    getComputedStyle(element).visibility === 'visible')
  const cards = Array.from(document.querySelectorAll('[data-testid="integrated-preparation"]')).filter(visible)
  return {
    preparation: { attemptId, revision, phase, failedPhase, failure },
    localHintVisible: cards.some(card => Array.from(card.querySelectorAll('p')).some(
      text => visible(text) && text.textContent.includes('无需联网'),
    )),
    stepsVisible: cards.some(card => visible(card.querySelector('ol[aria-label="内置环境准备阶段"]'))),
  }
}

function projectFrame({ preparation, localHintVisible, stepsVisible }) {
  const { attemptId, revision, phase, failedPhase, failure } = preparation
  assert(phases.includes(phase), 'Unknown preparation phase')
  assert(Number.isSafeInteger(revision) && revision >= 0, 'Invalid preparation revision')
  assert(phase === 'idle' ? attemptId === null : typeof attemptId === 'string' &&
    /^[0-9a-f]{8}(?:-[0-9a-f]{4}){3}-[0-9a-f]{12}$/i.test(attemptId), 'Invalid preparation attempt ID')
  assert(failedPhase === null || phases.includes(failedPhase), 'Invalid failed preparation phase')
  assert(failure === null || failureKinds.includes(failure), 'Unknown preparation failure')
  assert(typeof localHintVisible === 'boolean' && typeof stepsVisible === 'boolean', 'Invalid preparation UI sample')
  return { attemptId, revision, phase, failedPhase, failure, localHintVisible, stepsVisible }
}

export async function observePreparation(readFrame, record, {
  timeoutMs = 300_000, pollMs = 250, settleMs = 1_200,
} = {}) {
  const start = performance.now()
  const deadline = start + timeoutMs
  let previousFrame
  let terminalSince
  let attemptId = null
  let previousPhase
  let localProgressObserved = false
  while (performance.now() < deadline) {
    const raw = await readFrame(Math.max(1, Math.ceil(deadline - performance.now())))
    const sampledAt = performance.now()
    if (sampledAt >= deadline) break
    const frame = projectFrame(raw)
    const elapsedMs = Math.floor(sampledAt - start)
    const serialized = JSON.stringify(frame)
    if (serialized !== previousFrame) {
      await record({ kind: 'sample', elapsedMs, ...frame })
      previousFrame = serialized
    }
    if (attemptId !== frame.attemptId) localProgressObserved = false
    if (attemptId !== frame.attemptId || previousPhase !== frame.phase) terminalSince = undefined
    attemptId = frame.attemptId
    previousPhase = frame.phase
    // This is a co-observation. The UI polls separately and its snapshot may lag.
    localProgressObserved ||= localPhases.includes(frame.phase) && frame.localHintVisible && frame.stepsVisible
    if (terminalPhases.includes(frame.phase)) {
      // The UI polls independently. Leave time to observe its final render after the backend settles.
      terminalSince ??= elapsedMs
      if (elapsedMs - terminalSince >= settleMs) {
        await record({ kind: 'end', elapsedMs, attemptId: frame.attemptId, phase: frame.phase, localProgressObserved })
        return
      }
    }
    const remainingMs = deadline - performance.now()
    if (remainingMs > 0) await delay(Math.min(pollMs, remainingMs))
  }
  await record({ kind: 'end', elapsedMs: Math.floor(performance.now() - start), attemptId,
    phase: 'observation_timeout', localProgressObserved })
  throw Error('Preparation observer did not see a terminal phase within its own deadline')
}
