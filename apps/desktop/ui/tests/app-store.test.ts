import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it } from 'vitest'
import { useAppStore } from '@/stores/app'
import type { StatusSnapshot } from '@/contracts/runtimeStatus'

function snapshot(runtime: StatusSnapshot['matrix']['runtime']): StatusSnapshot {
  return {
    matrix: { runtime, connection: 'synced', operation: 'idle' },
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
    port: null,
    pid: null,
    can_start: true,
    can_stop: false,
    can_restart: false,
    source: 'live',
  }
}

describe('useAppStore', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('starts in the initialized shell state', () => {
    expect(useAppStore().initialized).toBe(true)
  })

  it('records recent events only from real observed runtime transitions', () => {
    const app = useAppStore()
    // 起始没有原型固定样例事件。
    expect(app.recentEvents).toEqual([])
    app.setStatusSnapshot(snapshot('stopped'))
    expect(app.recentEvents).toHaveLength(1)
    expect(app.recentEvents[0]?.message).toContain('未运行')
    // 同一状态重复到达不重复登记。
    app.setStatusSnapshot(snapshot('stopped'))
    expect(app.recentEvents).toHaveLength(1)
    app.setStatusSnapshot(snapshot('running'))
    expect(app.recentEvents).toHaveLength(2)
    expect(app.recentEvents[0]?.message).toContain('运行中')
  })
})
