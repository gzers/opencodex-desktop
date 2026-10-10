import { describe, expect, it, vi, afterEach } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { allowsFeedbackSignal } from '@/config/eventRegistry'
import { useFeedbackStore } from '@/app/feedback/store'

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
    vi.advanceTimersByTime(1000)
    store.showToast('second')
    vi.advanceTimersByTime(1000)
    expect(store.toast).toBe('second')
    vi.advanceTimersByTime(900)
    expect(store.toast).toBe('')
    expect(store.recentEvents).toHaveLength(8)
    expect(store.uiFault).toBe('render failed')
    expect(write).not.toHaveBeenCalled()
    write.mockRestore()
  })
})
