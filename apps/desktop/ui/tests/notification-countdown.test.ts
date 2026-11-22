import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createCountdown } from '@/app/feedback/countdown'
import { BELL_PULSE_MS, COLLAPSE_MS, isLeaving, leave, pulseBell } from '@/app/feedback/motion'

beforeEach(() => { vi.useFakeTimers(); document.body.innerHTML = '' })
afterEach(() => { vi.useRealTimers(); document.body.innerHTML = '' })

function clock() {
  let t = 0
  return { now: () => t, advance(ms: number) { t += ms; vi.advanceTimersByTime(ms) } }
}

describe('通知倒计时（原型 notification-countdown.js）', () => {
  it('暂停保留剩余时长，全部原因解除后按剩余时长到点且只触发一次', () => {
    const c = clock(), onElapsed = vi.fn()
    const countdown = createCountdown({ durationMs: 8000, tone: 'success', onElapsed, now: c.now })
    c.advance(3000)
    countdown.pause('pointer'); countdown.pause('focus')
    c.advance(20000)
    expect(onElapsed).not.toHaveBeenCalled()
    expect(countdown.state()).toMatchObject({ phase: 'paused', remainingMs: 5000 })
    countdown.resume('pointer')
    expect(countdown.state().phase).toBe('paused')
    countdown.resume('focus')
    c.advance(4999)
    expect(onElapsed).not.toHaveBeenCalled()
    c.advance(1)
    expect(onElapsed).toHaveBeenCalledTimes(1)
    expect(countdown.state()).toMatchObject({ phase: 'elapsed', ratio: 0 })
    countdown.pause('pointer'); countdown.resume('pointer'); c.advance(10000)
    expect(onElapsed).toHaveBeenCalledTimes(1)
  })

  it('取消后不再到点', () => {
    const c = clock(), onElapsed = vi.fn()
    const countdown = createCountdown({ durationMs: 2600, tone: 'success', onElapsed, now: c.now })
    countdown.cancel()
    c.advance(5000)
    expect(onElapsed).not.toHaveBeenCalled()
    expect(countdown.state().phase).toBe('cancelled')
  })
})

describe('收起到铃铛（原型 notification-motion.js）', () => {
  function mountSurface(key: string, size = 100) {
    const bell = document.createElement('button')
    bell.dataset.notificationBell = ''
    const layer = document.createElement('div')
    layer.className = 'notif-layer'
    const card = document.createElement('div')
    card.dataset.notificationKey = key
    for (const el of [bell, card]) el.getBoundingClientRect = () => ({ left: 0, top: 0, width: size, height: size, right: size, bottom: size, x: 0, y: 0, toJSON() { return {} } }) as DOMRect
    layer.appendChild(card)
    document.body.append(bell, layer)
    return { bell, card }
  }

  it('找不到通知面或尺寸为零时同步提交并脉冲铃铛', () => {
    const { bell } = mountSurface('toast:1', 0)
    const commit = vi.fn()
    leave('toast:1', commit)
    expect(commit).toHaveBeenCalledTimes(1)
    expect(isLeaving('toast:1')).toBe(false)
    expect(bell.classList.contains('collapse-pulse')).toBe(true)
    vi.advanceTimersByTime(BELL_PULSE_MS)
    expect(bell.classList.contains('collapse-pulse')).toBe(false)
    const missing = vi.fn()
    leave('task:none', missing)
    expect(missing).toHaveBeenCalledTimes(1)
  })

  it('飞行中重复收起只追加提交，结束后每个提交执行一次', () => {
    const { card } = mountSurface('task:a')
    const first = vi.fn(), second = vi.fn()
    leave('task:a', first)
    leave('task:a', second)
    expect(isLeaving('task:a')).toBe(true)
    expect(card.dataset.notificationMotion).toBe('leaving')
    expect(first).not.toHaveBeenCalled()
    vi.advanceTimersByTime(COLLAPSE_MS)
    expect(first).toHaveBeenCalledTimes(1)
    expect(second).toHaveBeenCalledTimes(1)
    expect(isLeaving('task:a')).toBe(false)
  })

  it('连续收起会重新触发铃铛脉冲', () => {
    const { bell } = mountSurface('x', 0)
    pulseBell()
    vi.advanceTimersByTime(BELL_PULSE_MS - 10)
    pulseBell()
    vi.advanceTimersByTime(20)
    expect(bell.classList.contains('collapse-pulse')).toBe(true)
    vi.advanceTimersByTime(BELL_PULSE_MS)
    expect(bell.classList.contains('collapse-pulse')).toBe(false)
  })
})

