import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppStore } from '@/stores/app'
import { useLifecycleStore } from '@/app/lifecycle/store'
import { healthLabel, runtimeLabel, runtimeSourceLabel, trayRuntimeLabel } from '@/lib/labels'
import type { StatusSnapshot } from '@/contracts/runtimeStatus'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: () => Promise.resolve(null),
}))

function snapshot(runtime: StatusSnapshot['matrix']['runtime']): StatusSnapshot {
  return {
    matrix: { runtime, connection: 'synced', operation: 'idle' },
    facts: {
      health: 'healthy',
      runtime_label: 'bundled',
      opencodex_home: '/fixtures/opencodex-home',
      data_root: '/fixtures/data-root',
      protection: null,
      reboot_safe: null,
      service_present: null,
      shim_present: null,
      version_drift: null,
    },
    port: runtime === 'running' ? 10100 : null,
    pid: runtime === 'running' ? '39421' : null,
    can_start: runtime !== 'running',
    can_stop: runtime === 'running',
    can_restart: runtime === 'running',
    source: 'live',
  } as StatusSnapshot
}

describe('界面文案统一为中文', () => {
  it('运行状态枚举不再泄漏英文', () => {
    expect(runtimeLabel('at_risk')).toBe('存在风险')
    expect(runtimeLabel('external_takeover')).toBe('外部接管')
    expect(runtimeLabel('starting_failed')).toBe('启动失败')
    expect(runtimeLabel('unreachable')).toBe('不可达')
    // 未知/新增枚举回退中文占位，而不是原始英文值。
    expect(runtimeLabel('brand_new_state')).toBe('未知')
    expect(runtimeLabel(null)).toBe('未知')
  })

  it('托盘标签只讲进程事实，不写风险结论', () => {
    // 2026-09-24 用户决策：托盘菜单项不再承载「存在风险」与其说明句。
    expect(trayRuntimeLabel('at_risk')).toBe('未运行')
    expect(trayRuntimeLabel('running')).toBe('运行中')
    expect(trayRuntimeLabel('stopped')).toBe('未运行')
    expect(trayRuntimeLabel('external_takeover')).toBe('外部接管')
    expect(trayRuntimeLabel('brand_new_state')).toBe('未知')
  })

  it('健康与运行时来源转中文', () => {
    expect(healthLabel('healthy')).toBe('正常')
    expect(healthLabel('degraded')).toBe('降级')
    expect(healthLabel('unhealthy')).toBe('异常')
    expect(runtimeSourceLabel('bundled')).toBe('内置')
    expect(runtimeSourceLabel('system')).toBe('系统')
    expect(runtimeSourceLabel('weird')).toBe('未知')
  })
})

describe('任务卡在真实状态迁移后收口并自动退出', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.useFakeTimers()
  })
  afterEach(() => {
    vi.useRealTimers()
  })

  it('start：观测到「运行中」即完成，随后自动关闭通知', () => {
    const app = useAppStore()
    useLifecycleStore().processActionBusy = true
    useLifecycleStore().processAction = 'start'
    useLifecycleStore().processProgressAction = 'start'
    useLifecycleStore().processProgressOpen = true

    // 首次「启动」的典型时序：先 at_risk（编辑器映射到「存在风险」），未完成。
    app.setStatusSnapshot(snapshot('at_risk'))
    expect(app.processProgressCompleted).toBe(false)
    expect(app.processProgressOpen).toBe(true)

    // 代理就绪后后端/兜底轮询给出 running → 完成。
    app.setStatusSnapshot(snapshot('running'))
    expect(app.processProgressCompleted).toBe(true)
    expect(app.processProgressError).toBe('')

    // 成功后自动退出通知（无需用户再点「关闭」）。
    vi.advanceTimersByTime(2600)
    expect(app.processProgressOpen).toBe(false)
  })

  it('失败不自动关闭：保留给用户处理', () => {
    const app = useAppStore()
    useLifecycleStore().processActionBusy = true
    useLifecycleStore().processAction = 'start'
    useLifecycleStore().processProgressAction = 'start'
    useLifecycleStore().processProgressOpen = true

    app.setStatusSnapshot(snapshot('starting_failed'))
    expect(app.processProgressCompleted).toBe(false)
    expect(app.processProgressError).not.toBe('')
    vi.advanceTimersByTime(5000)
    expect(app.processProgressOpen).toBe(true)
  })
})
