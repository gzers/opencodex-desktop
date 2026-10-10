import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createUpdateScheduler } from '@/features/updates/scheduler'
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
beforeEach(() => { vi.useFakeTimers(); vi.setSystemTime(0) })
afterEach(() => vi.useRealTimers())

function fixture() {
  const state = { visible: true, busy: false }
  const plan = vi.fn().mockResolvedValue({ targets: [], nextDelayMs: 86_400_000 })
  const check = vi.fn().mockResolvedValue(null)
  const onError = vi.fn()
  const scheduler = createUpdateScheduler({ plan, check, onError,
    visible: () => state.visible, busy: () => state.busy })
  return { scheduler, state, plan, check, onError }
}
describe('application update scheduler', () => {
  it('keeps the active query trigger and coalesces a later online wake', async () => {
    const f = fixture(); let done!: () => void
    f.plan.mockResolvedValueOnce({ targets: ['manager_stable'], nextDelayMs: 0 })
    f.check.mockImplementationOnce(() => new Promise<void>(resolve => { done = resolve }))
    f.scheduler.start(); f.scheduler.wake('foreground')
    await vi.advanceTimersByTimeAsync(45_000)
    expect(f.check.mock.calls).toEqual([['manager_stable', 'foreground']])
    for (let i = 0; i < 100; i++) f.scheduler.wake('online')
    expect(vi.getTimerCount()).toBe(0)
    // First plan commits the old request; only the coalesced wake checks panel.
    f.plan.mockResolvedValueOnce({ targets: [], nextDelayMs: 86_400_000 })
      .mockResolvedValueOnce({ targets: ['panel'], nextDelayMs: 0 })
    done(); await vi.advanceTimersByTimeAsync(100)
    expect(f.check.mock.calls).toEqual([['manager_stable', 'foreground'], ['panel', 'online']])
    expect(vi.getTimerCount()).toBe(1); f.scheduler.stop()
  })
  it('coalesces startup triggers into one 45s wake and does not query valid cache', async () => {
    const f = fixture(); f.scheduler.start(); f.scheduler.start()
    for (let i = 0; i < 100; i++) f.scheduler.wake()
    expect(vi.getTimerCount()).toBe(1)
    await vi.advanceTimersByTimeAsync(44_999)
    expect(f.plan).not.toHaveBeenCalled()
    await vi.advanceTimersByTimeAsync(1)
    expect(f.plan).toHaveBeenCalledTimes(1)
    expect(f.check).not.toHaveBeenCalled()
    expect(vi.getTimerCount()).toBe(1)
    f.scheduler.stop(); expect(vi.getTimerCount()).toBe(0)
  })
  it('disabled checks have no timer; re-enabling wakes from a lifecycle event', async () => {
    const f = fixture(); f.plan.mockResolvedValue({ targets: [], nextDelayMs: null })
    f.scheduler.start(); await vi.advanceTimersByTimeAsync(45_000)
    expect(vi.getTimerCount()).toBe(0)
    f.plan.mockResolvedValueOnce({ targets: ['panel'], nextDelayMs: 0 })
    f.scheduler.wake(); await vi.advanceTimersByTimeAsync(100)
    expect(f.check).toHaveBeenCalledWith('panel', 'deadline')
    expect(vi.getTimerCount()).toBe(0)
  })
  it('sleep resumes once without catchup; installations defer queries', async () => {
    const f = fixture(); f.scheduler.start(); f.state.visible = false; f.scheduler.wake()
    expect(vi.getTimerCount()).toBe(0)
    await vi.advanceTimersByTimeAsync(7 * 86_400_000)
    expect(f.plan).not.toHaveBeenCalled()
    f.state.visible = true; f.state.busy = true; f.scheduler.wake()
    await vi.advanceTimersByTimeAsync(100); expect(f.plan).not.toHaveBeenCalled()
    f.state.busy = false; f.scheduler.wake()
    f.plan.mockResolvedValueOnce({ targets: ['manager_stable', 'panel'], nextDelayMs: 0 })
    await vi.advanceTimersByTimeAsync(100)
    expect(f.check.mock.calls).toEqual([['manager_stable', 'deadline'], ['panel', 'deadline']])
    expect(vi.getTimerCount()).toBe(1); f.scheduler.stop()
  })
  it('serializes objects and coalesces triggers arriving during a request', async () => {
    const f = fixture(); let done!: () => void
    f.check.mockImplementationOnce(() => new Promise<void>(resolve => { done = resolve }))
    f.plan.mockResolvedValueOnce({ targets: ['manager_stable', 'panel'], nextDelayMs: 0 })
    f.scheduler.start(); await vi.advanceTimersByTimeAsync(45_000)
    for (let i = 0; i < 100; i++) f.scheduler.wake()
    expect(f.check.mock.calls).toEqual([['manager_stable', 'deadline']])
    expect(vi.getTimerCount()).toBe(0)
    done(); await vi.advanceTimersByTimeAsync(100)
    expect(f.check.mock.calls).toEqual([['manager_stable', 'deadline'], ['panel', 'deadline']])
    expect(vi.getTimerCount()).toBe(1); f.scheduler.stop()
  })
  it('stopping during a pending plan prevents queries and rearming', async () => {
    const f = fixture(); let done!: (value: unknown) => void
    f.plan.mockImplementationOnce(() => new Promise(resolve => { done = resolve }))
    f.scheduler.start(); await vi.advanceTimersByTimeAsync(45_000)
    f.scheduler.stop(); done({ targets: ['panel'], nextDelayMs: 0 })
    await vi.advanceTimersByTimeAsync(1)
    expect(f.check).not.toHaveBeenCalled(); expect(vi.getTimerCount()).toBe(0)
  })
  it('state errors back off without querying', async () => {
    const f = fixture(); f.plan.mockRejectedValue(new Error('invalid state'))
    f.scheduler.start(); await vi.advanceTimersByTimeAsync(45_000)
    expect(f.onError).toHaveBeenCalledOnce(); expect(f.check).not.toHaveBeenCalled()
    await vi.advanceTimersByTimeAsync(30 * 60_000 - 1)
    expect(f.plan).toHaveBeenCalledOnce(); f.scheduler.stop()
  })
})
