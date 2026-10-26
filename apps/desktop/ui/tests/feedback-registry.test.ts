import { describe, expect, it, vi, afterEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { allowsFeedbackSignal } from '@/config/eventRegistry'
import { TOAST_ERROR_MS, TOAST_INFO_MS, useFeedbackStore } from '@/app/feedback/store'

afterEach(() => vi.useRealTimers())
describe('memory-only registered UI feedback', () => {
  it('allows only registered active UI signal owners', () => {
    for (const event of ['ui-toast', 'ui-recent-event', 'ui-fault']) expect(allowsFeedbackSignal(event)).toBe(true)
    for (const event of ['unknown', 'manager-install-progress', 'run-start-failed', 'migration-completed']) expect(allowsFeedbackSignal(event)).toBe(false)
  })
  it('keeps feedback bounded and never writes web storage', () => {
    vi.useFakeTimers()
    setActivePinia(createPinia())
    const write = vi.spyOn(Storage.prototype, 'setItem')
    const store = useFeedbackStore()
    for (let i = 0; i < 100; i++) store.recordEvent('event ' + i)
    store.reportUiFault('render failed')
    store.showToast('first')
    vi.advanceTimersByTime(TOAST_INFO_MS - 100)
    store.showToast('second')
    // 后一条覆盖前一条，且重新计时。
    expect(store.toast).toBe('second')
    vi.advanceTimersByTime(TOAST_INFO_MS - 1)
    expect(store.toast).toBe('second')
    vi.advanceTimersByTime(1)
    expect(store.toast).toBe('')
    expect(store.recentEvents).toHaveLength(8)
    expect(store.uiFault).toBe('render failed')
    expect(write).not.toHaveBeenCalled()
    write.mockRestore()
  })
  it('keeps errors on screen longer than ordinary tips, matching the prototype', () => {
    vi.useFakeTimers()
    setActivePinia(createPinia())
    const store = useFeedbackStore()
    expect(TOAST_ERROR_MS).toBeGreaterThan(TOAST_INFO_MS)
    store.showToast('failed', { kind: 'error' })
    vi.advanceTimersByTime(TOAST_ERROR_MS - 1)
    expect(store.toast).toBe('failed')
    vi.advanceTimersByTime(1)
    expect(store.toast).toBe('')
  })
  it('lets a call site override the duration', () => {
    vi.useFakeTimers()
    setActivePinia(createPinia())
    const store = useFeedbackStore()
    store.showToast('long note', { durationMs: 9000 })
    vi.advanceTimersByTime(TOAST_ERROR_MS)
    expect(store.toast).toBe('long note')
    vi.advanceTimersByTime(9000 - TOAST_ERROR_MS)
    expect(store.toast).toBe('')
  })
})
