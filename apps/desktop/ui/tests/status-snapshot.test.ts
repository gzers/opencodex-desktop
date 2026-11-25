import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppStore } from '@/stores/app'
import { useLifecycleStore } from '@/app/lifecycle/store'
import type { StatusSnapshot } from '@/contracts/runtimeStatus'

function snapshot(overrides: Partial<StatusSnapshot> = {}): StatusSnapshot {
  return {
    matrix: { runtime: 'stopped', connection: 'unconfigured', operation: 'idle' },
    facts: {
      health: 'unknown',
      runtime_label: null,
      opencodex_home: null,
      data_root: '/fixtures/data-root',
      protection: null,
      reboot_safe: null,
      service_present: null,
      shim_present: null,
      version_drift: null,
    },
    port: 10100,
    pid: null,
    can_start: true,
    can_stop: false,
    can_restart: false,
    source: 'fixture',
    ...overrides,
  }
}

describe('status snapshot integration', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetModules()
  })

  it('loads backend snapshot and preserves frozen enum values', async () => {
    const invoke = vi.fn().mockResolvedValue(snapshot())
    vi.doMock('@tauri-apps/api/core', () => ({ invoke }))
    const { getStatusSnapshot } = await import('@/contracts/runtimeStatus')
    const app = useAppStore()
    await app.loadStatusSnapshot()
    const loaded = await getStatusSnapshot()
    expect(invoke).toHaveBeenCalledWith('get_status_snapshot')
    expect(app.statusSnapshot).toEqual(loaded)
    expect(app.statusSnapshot?.matrix.runtime).toBe('stopped')
    expect(app.statusSnapshot?.source).toBe('fixture')
    expect(app.statusError).toBe(false)
    expect(app.statusLoading).toBe(false)
  })

  it('clears previous action authority when the backend is unavailable', async () => {
    const invoke = vi.fn().mockRejectedValue(new Error('backend unavailable'))
    vi.doMock('@tauri-apps/api/core', () => ({ invoke }))
    await import('@/contracts/runtimeStatus')
    const app = useAppStore()
    const previous = snapshot()
    useLifecycleStore().statusSnapshot = previous
    await app.loadStatusSnapshot()
    expect(app.statusSnapshot).toBeNull()
    expect(app.runtimeState).toBe('unreachable')
    expect(app.processActionBlocked('start')).toBe(true)
    expect(app.statusError).toBe(true)
    expect(app.statusLoading).toBe(false)
  })

  it('clears stale facts at the five-second deadline and ignores a late direct reply', async () => {
    vi.useFakeTimers()
    try {
      let release!: (value: StatusSnapshot) => void
      const invoke = vi.fn(() => new Promise<StatusSnapshot>(resolve => { release = resolve }))
      vi.doMock('@tauri-apps/api/core', () => ({ invoke }))
      const app = useAppStore()
      app.setStatusSnapshot(snapshot({ port: 10100, pid: '4100' }))
      const sample = app.loadStatusSnapshot()
      await vi.dynamicImportSettled()
      await vi.advanceTimersByTimeAsync(4999)
      expect(app.statusLoading).toBe(true)
      await vi.advanceTimersByTimeAsync(1)
      await sample
      expect(app.statusLoading).toBe(false)
      expect(app.statusError).toBe(true)
      expect(app.runtimeState).toBe('unreachable')
      expect(app.statusSnapshot).toBeNull()
      expect(app.processActionBlocked('start')).toBe(true)
      release(snapshot())
      await vi.advanceTimersByTimeAsync(0)
      expect(app.statusSnapshot).toBeNull()
    } finally {
      vi.useRealTimers()
    }
  })

  it('rejects typed failures and lets a newer backend event own the result', async () => {
    const failed = snapshot({ status_failure: 'timeout' })
    const invoke = vi.fn().mockResolvedValueOnce(failed)
    const listen = vi.fn().mockResolvedValue(vi.fn())
    vi.doMock('@tauri-apps/api/core', () => ({ invoke }))
    vi.doMock('@tauri-apps/api/event', () => ({ listen }))
    const app = useAppStore()
    const stop = await app.startStatusEventStream()
    await app.loadStatusSnapshot()
    expect(app.statusSnapshot).toBeNull()
    expect(app.processActionBlocked('start')).toBe(true)

    let release!: (value: StatusSnapshot) => void
    invoke.mockImplementationOnce(() => new Promise<StatusSnapshot>(resolve => { release = resolve }))
    const sample = app.loadStatusSnapshot()
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledTimes(2))
    const current = snapshot()
    listen.mock.calls[0][1]({ payload: current })
    release(failed)
    await sample
    expect(app.statusSnapshot).toEqual(current)
    expect(app.statusError).toBe(false)
    stop?.()
  })

  it('does not start concurrent loads', async () => {
    let resolve: (value: StatusSnapshot) => void = () => {}
    const invoke = vi.fn().mockImplementation(
      () =>
        new Promise<StatusSnapshot>(done => {
          resolve = done
        }),
    )
    vi.doMock('@tauri-apps/api/core', () => ({ invoke }))
    await import('@/contracts/runtimeStatus')
    const app = useAppStore()
    const first = app.loadStatusSnapshot()
    const second = app.loadStatusSnapshot()
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledTimes(1))
    resolve(snapshot())
    await Promise.all([first, second])
    expect(invoke).toHaveBeenCalledTimes(1)
  })

  it('syncs a decisive direct snapshot into runtime state', async () => {
    const invoke = vi.fn().mockResolvedValue(snapshot())
    vi.doMock('@tauri-apps/api/core', () => ({ invoke }))
    await import('@/contracts/runtimeStatus')
    const app = useAppStore()
    expect(app.runtimeState).toBe('loading')
    await app.loadStatusSnapshot()
    expect(app.runtimeState).toBe('stopped')
  })

  it('keeps runtime loading for transient snapshot states', async () => {
    const invoke = vi.fn().mockResolvedValue(snapshot({ matrix: { runtime: 'starting', connection: 'unconfigured', operation: 'idle' } }))
    vi.doMock('@tauri-apps/api/core', () => ({ invoke }))
    await import('@/contracts/runtimeStatus')
    const app = useAppStore()
    await app.loadStatusSnapshot()
    expect(app.statusSnapshot?.matrix.runtime).toBe('starting')
    expect(app.runtimeState).toBe('loading')
  })
})

describe('status polling event integration', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.resetModules()
  })

  it('updates the same snapshot from backend events without extra IPC polling', async () => {
    const unlisten = vi.fn()
    const listen = vi.fn().mockResolvedValue(unlisten)
    const eventPayload = snapshot({ matrix: { runtime: 'running', connection: 'synced', operation: 'idle' } })
    vi.doMock('@tauri-apps/api/event', () => ({ listen }))
    const app = useAppStore()
    const stop = await app.startStatusEventStream()
    expect(stop).toBeTruthy()
    expect(listen).toHaveBeenCalledWith('status-snapshot-changed', expect.any(Function))
    await listen.mock.calls[0][1]({ payload: eventPayload })
    expect(app.statusSnapshot).toEqual(eventPayload)
    expect(app.statusSnapshot?.matrix.runtime).toBe('running')
    expect(app.runtimeState).toBe('running')
    expect(app.statusError).toBe(false)
    stop?.()
    expect(unlisten).toHaveBeenCalled()
  })
})
