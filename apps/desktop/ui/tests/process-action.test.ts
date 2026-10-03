import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppStore } from '@/stores/app'
import { useLifecycleStore } from '@/app/lifecycle/store'
import { statusScenario, runtimeScenarios } from '@/features/runtime/scenarios'
import type { StatusSnapshot } from '@/contracts/runtimeStatus'

const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

function backendResult(state: string, result: 'started' | 'stopped' = 'started') {
  return {
    lifecycleState: state,
    result,
    canStart: state === 'stopped',
    canStop: state === 'running',
    canRestart: state === 'running',
  }
}

function snapshot(runtime: StatusSnapshot['matrix']['runtime'], pid = '39421'): StatusSnapshot {
  return {
    matrix: { runtime, connection: 'synced', operation: 'idle' },
    facts: {
      health: runtime === 'running' ? 'healthy' : 'unknown',
      runtime_label: null,
      opencodex_home: '/fixtures/opencodex-home',
      data_root: '/fixtures/data-root',
      protection: null,
      reboot_safe: null,
      service_present: null,
      shim_present: null,
      version_drift: null,
    },
    port: runtime === 'running' ? 10100 : null,
    pid: runtime === 'running' ? pid : null,
    can_start: runtime !== 'running',
    can_stop: runtime === 'running',
    can_restart: runtime === 'running',
    source: 'fixture',
  }
}

describe('process action integration', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
  })

  it('maps start through the backend lifecycle contract', async () => {
    invoke.mockImplementation((command: string) =>
      command === 'process_action'
        ? Promise.resolve(backendResult('starting'))
        : Promise.resolve(snapshot('starting')),
    )
    const app = useAppStore()
    await app.requestProcessAction('start')
    expect(invoke).toHaveBeenCalledWith('process_action', { request: { action: 'start', confirm: true } })
    expect(app.lifecycleState).toBe('starting')
    expect(app.processActionBusy).toBe(true)
    expect(app.processProgressOpen).toBe(true)

    app.syncProcessProgress(snapshot('running'))
    expect(app.processActionBusy).toBe(false)
    expect(app.processProgressCompleted).toBe(true)
  })

  it('finishes stop immediately and keeps restart busy until running', async () => {
    const app = useAppStore()
    invoke.mockImplementation((command: string) =>
      command === 'process_action'
        ? Promise.resolve(backendResult('stopping', 'stopped'))
        : Promise.resolve(snapshot('stopped')),
    )
    await app.requestProcessAction('stop')
    expect(app.lifecycleState).toBe('stopping')
    expect(app.processActionBusy).toBe(false)

    invoke.mockResolvedValue(backendResult('starting'))
    await app.requestProcessAction('restart')
    expect(app.lifecycleState).toBe('starting')
    expect(app.processActionBusy).toBe(true)
    app.syncProcessProgress(snapshot('stopping'))
    app.syncProcessProgress(snapshot('running'))
    expect(app.processActionBusy).toBe(false)
  })

  it('clears busy and preserves current state on failure', async () => {
    const app = useAppStore()
    useLifecycleStore().lifecycleState = 'running'
    invoke.mockRejectedValue(new Error('backend unavailable'))
    await expect(app.requestProcessAction('stop')).rejects.toThrow('backend unavailable')
    expect(app.lifecycleState).toBe('running')
    expect(app.processActionBusy).toBe(false)
  })

  it('closes progress immediately when backend preserves observation without a result', async () => {
    const app = useAppStore()
    invoke.mockResolvedValue({
      ...backendResult('stopped'),
      result: null,
    })
    await app.requestProcessAction('start')
    expect(app.processActionBusy).toBe(false)
    expect(app.processProgressError).toContain('启动指令未完成')
    expect(app.processProgressCompleted).toBe(false)
  })

  it('projects backend lifecycle and status fields into the overview scenario without inventing data', () => {
    const result = backendResult('running')
    const base = runtimeScenarios.stopped
    const projected = statusScenario(base, {
      matrix: { runtime: 'running', connection: 'synced', operation: 'idle' },
      facts: {
        health: 'healthy',
        runtime_label: 'Running',
        opencodex_home: '/tmp/fixture/opencodex-home',
        data_root: '/tmp/fixture/data-root',
        protection: null,
        reboot_safe: null,
        service_present: null,
        shim_present: null,
        version_drift: null,
      },
      port: 10100,
      pid: '39421',
      can_start: result.canStart,
      can_stop: result.canStop,
      can_restart: result.canRestart,
      source: 'fixture',
    })
    expect(projected.label).toBe('未运行')
    expect(projected.health).toBe('正常')
    expect(projected.port).toBe('10100')
    expect(projected.pid).toBe('39421')
    expect(projected.actions).toEqual(base.actions)
  })

  it('does not start concurrent backend actions', async () => {
    let resolveAction: (value: ReturnType<typeof backendResult>) => void = () => {}
    invoke.mockImplementation((command: string) => {
      if (command === 'get_status_snapshot') return Promise.resolve(snapshot('starting'))
      return new Promise(done => { resolveAction = done })
    })
    const app = useAppStore()
    const first = app.requestProcessAction('start')
    const second = app.requestProcessAction('start')
    await new Promise(resolveTimer => setTimeout(resolveTimer, 50))
    resolveAction(backendResult('starting'))
    await Promise.allSettled([first, second])
    expect(invoke).toHaveBeenCalledTimes(1)
    await expect(second).resolves.toBeUndefined()
    app.syncProcessProgress(snapshot('running'))
  })

  // 回归：真实重启耗时超过 35s 时，任务卡先进入「未完成」；
  // 若此后真实状态才收敛，必须改判为「已完成」，不能一直停在「未完成」。
  it('closes the restart card as completed when the real state converges after the timeout', async () => {
    vi.useFakeTimers()
    invoke.mockImplementation((command: string) =>
      command === 'process_action'
        ? Promise.resolve(backendResult('starting'))
        : Promise.resolve(snapshot('running')),
    )
    const app = useAppStore()
    await app.requestProcessAction('restart')
    expect(app.processProgressTimedOut).toBe(false)

    // 35s 内没观测到状态迁移：进入超时提示，但仍在观察。
    vi.advanceTimersByTime(36_000)
    expect(app.processProgressTimedOut).toBe(true)
    expect(app.processProgressError).not.toBe('')

    // 真实状态随后收敛（stopping → running）：任务卡改判为已完成。
    app.setStatusSnapshot(snapshot('stopping'))
    app.setStatusSnapshot(snapshot('running'))
    expect(app.processProgressCompleted).toBe(true)
    expect(app.processProgressError).toBe('')
    expect(app.processProgressTimedOut).toBe(false)
    vi.useRealTimers()
  })

  // 真实重启可能快到采不到 stopping/starting；PID 变化同样是「真的换了进程」的证据。
  it('closes the restart card when the process identity changed even without an observed transition', async () => {
    vi.useFakeTimers()
    invoke.mockImplementation((command: string) =>
      command === 'process_action'
        ? Promise.resolve(backendResult('starting'))
        : Promise.resolve(snapshot('running', '1111')),
    )
    const app = useAppStore()
    app.setStatusSnapshot(snapshot('running', '1111'))
    await app.requestProcessAction('restart')
    vi.advanceTimersByTime(36_000)
    expect(app.processProgressTimedOut).toBe(true)

    // 新进程已就绪：PID 变了，任务卡应改判为已完成。
    app.setStatusSnapshot(snapshot('running', '2222'))
    expect(app.processProgressCompleted).toBe(true)
    expect(app.processProgressError).toBe('')
    vi.useRealTimers()
  })

  // 2026-10-03 用户确认：at_risk 的进程事实就是「未运行」，概览呈现应与「未运行」逐字一致——
  // 只「启动 OpenCodex」，不再多出「启动保护未启用」说明句或「查看建议」按钮。
  it('presents at-risk exactly like stopped in the overview (start only)', () => {
    const projected = statusScenario(runtimeScenarios.at_risk, {
      matrix: { runtime: 'at_risk', connection: 'disconnected', operation: 'idle' },
      facts: {
        health: 'unknown',
        runtime_label: null,
        opencodex_home: null,
        data_root: '',
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
      source: 'live',
    })
    expect(projected.actions).toEqual(['start'])
  })

  it('matches the stopped overview actions exactly for at-risk', () => {
    const projected = statusScenario(runtimeScenarios.at_risk, {
      matrix: { runtime: 'at_risk', connection: 'disconnected', operation: 'idle' },
      facts: {
        health: 'healthy',
        runtime_label: null,
        opencodex_home: null,
        data_root: '',
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
      source: 'live',
    })
    const stopped = statusScenario(runtimeScenarios.stopped, {
      matrix: { runtime: 'stopped', connection: 'disconnected', operation: 'idle' },
      facts: {
        health: 'unknown',
        runtime_label: null,
        opencodex_home: null,
        data_root: '',
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
      source: 'live',
    })
    expect(projected.actions).toEqual(stopped.actions)
    expect(projected.health).toBe('正常')
  })
})
