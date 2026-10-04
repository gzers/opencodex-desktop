import { createHash } from 'node:crypto'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createMotionScene } from '@/features/runtime/motion/scene'
import { MOTION_STATES } from '@/features/runtime/motion/states'
import baseline from './fixtures/motion-016.json'

// Golden frames were captured from main 4c5afb51 before the optimization,
// using the same 16ms clock. No production source copy is kept in the fixture.
let pending: Map<number, FrameRequestCallback>
let nextId: number
let now: number
beforeEach(() => {
  pending = new Map(); nextId = 0; now = 1000
  vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => { pending.set(++nextId, cb); return nextId })
  vi.stubGlobal('cancelAnimationFrame', (id: number) => pending.delete(id))
})
afterEach(() => vi.unstubAllGlobals())
function step(count = 1) {
  for (let i = 0; i < count; i++) {
    now += 16
    const callbacks = [...pending.values()]; pending.clear()
    callbacks.forEach(cb => cb(now))
  }
}
function hosts() {
  const ambient = document.createElement('div')
  ambient.innerHTML = '<i class="energy"></i><i class="cloud"></i><i class="cloud"></i><i class="cloud"></i><i class="light-floor"></i>'
  return { ambient, markHost: document.createElement('div') }
}
function snapshot(h: ReturnType<typeof hosts>) { return h.ambient.outerHTML + h.markHost.innerHTML }

describe('unchanged visual frames and stopped background rendering', () => {
  it('matches the pre-optimization output for all nine states and transitions', async () => {
    const actual: Record<string, string[]> = {}
    for (const state of MOTION_STATES) {
      const h = hosts(), scene = createMotionScene(h)
      scene.setState('running', true); step(60)
      scene.setState(state.id)
      const hashes: string[] = []
      for (let frame = 1; frame <= 180; frame++) {
        step()
        if ([1, 10, 60, 180].includes(frame)) hashes.push(createHash('sha256').update(snapshot(h)).digest('hex'))
      }
      actual[state.id] = hashes
      scene.destroy()
      expect(pending.size).toBe(0)
    }
    expect(actual).toEqual(baseline)
  })

  it('pauses both scheduling and phase, and cannot restart after disposal', () => {
    const h = hosts(), scene = createMotionScene(h)
    scene.setState('running', true); step(80)
    const before = snapshot(h)
    scene.setActive(false)
    expect(pending.size).toBe(0)
    now += 300_000; step(5)
    expect(snapshot(h)).toBe(before)
    scene.setActive(true); step()
    expect(snapshot(h)).toBe(before)
    step(10); expect(snapshot(h)).not.toBe(before)
    scene.destroy(); scene.setActive(true)
    expect(pending.size).toBe(0)
  })
})
