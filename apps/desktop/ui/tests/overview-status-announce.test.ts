import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import App from '@/App.vue'
import { useAppStore } from '@/stores/app'
import { useLifecycleStore } from '@/app/lifecycle/store'
import { useRouteStore } from '@/stores/routes'
import { runtimeAnnouncements, runtimeScenarios } from '@/features/runtime/scenarios'
import type { StatusSnapshot } from '@/contracts/runtimeStatus'
import type { RuntimeState } from '@/types/ui'
import type { EnvironmentReport } from '@/features/environment/api'

const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: query === '(prefers-color-scheme: dark)',
    addEventListener: () => {},
    removeEventListener: () => {},
  }),
})

function snapshot(runtime: RuntimeState): StatusSnapshot {
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
    pid: runtime === 'running' ? '39421' : null,
  } as StatusSnapshot
}

function readyEnvironment(): EnvironmentReport {
  const found = { found: true, path: '/fixtures/bin', version: null }
  return { node: { ...found }, npm: { ...found }, ocx: { ...found }, gate: 'ready', shortCircuited: false, brewFound: true }
}

describe('overview status card carries facts and actions only', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    invoke.mockResolvedValue(undefined)
  })

  it('renders the frameless status area without a persistent facts row', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    invoke.mockImplementation((command: string) =>
      command === 'discover_environment' ? Promise.resolve(readyEnvironment()) : Promise.resolve(undefined),
    )
    const wrapper = mount(App, { global: { plugins: [pinia] } })
    useRouteStore().go('overview')
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))
    await wrapper.vm.$nextTick()

    const area = wrapper.find('[data-testid="overview-motion"]')
    expect(area.exists()).toBe(true)
    expect(wrapper.find('.ovb-note').exists()).toBe(false)
    const areaText = area.text()
    expect(areaText).not.toContain('可以直接启动')
    expect(areaText).not.toContain('代理运行中')
    // 端口 / 版本 / 数据目录不再常驻状态区：只在「运行详情」弹窗里。
    expect(areaText).not.toContain('数据目录')
    // 主线状态是详情入口；§25：环境摘要行不再常驻，环境明细并入运行详情。
    expect(area.find('.motion-mainline').exists()).toBe(true)
    expect(area.find('[data-testid="overview-env-summary"]').exists()).toBe(false)
    expect(area.find('.motion-env-row').exists()).toBe(false)
    expect(wrapper.find('[data-testid="environment-gate"]').exists()).toBe(false)

    await area.find('.motion-mainline').trigger('click')
    const modal = wrapper.find('.motion-modal')
    expect(modal.exists()).toBe(true)
    expect(modal.text()).toContain('端口')
    expect(modal.text()).toContain('数据目录')
    // 运行环境明细并入详情（逐项结论）。
    expect(modal.text()).toContain('运行环境')
    expect(modal.text()).toContain('OpenCodex')
  })

  it('keeps the scenario model free of note fields', () => {
    expect('note' in runtimeScenarios.stopped).toBe(false)
    expect('note' in runtimeScenarios.running).toBe(false)
  })
})

describe('runtime state transition announcements', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('does not announce on the first settled snapshot', () => {
    const app = useAppStore()
    app.setStatusSnapshot(snapshot('stopped'))
    expect(app.runtimeAnnouncePrimed).toBe(true)
    expect(app.toast).toBe('')
  })

  it('announces stopped and running exactly once per transition', () => {
    const app = useAppStore()
    app.setStatusSnapshot(snapshot('running'))
    app.clearToast()

    app.setStatusSnapshot(snapshot('stopped'))
    expect(app.toast).toBe(runtimeAnnouncements.stopped)

    app.clearToast()
    app.setStatusSnapshot(snapshot('running'))
    expect(app.toast).toBe(runtimeAnnouncements.running)
  })

  it('stays silent for transitional progress and attention states', () => {
    const app = useAppStore()
    app.setStatusSnapshot(snapshot('stopped'))
    for (const state of [
      'starting',
      'pending',
      'loading',
      'at_risk',
      'external_takeover',
      'unreachable',
      'not_found',
      'starting_failed',
    ] as RuntimeState[]) {
      app.clearToast()
      app.setStatusSnapshot(snapshot(state))
      expect(app.toast, `${state} 不应播报 Toast`).toBe('')
    }
  })

  it('does not double-announce while a lifecycle action reports its own result', () => {
    const app = useAppStore()
    app.setStatusSnapshot(snapshot('stopped'))
    app.clearToast()
    useLifecycleStore().processAction = 'start'

    app.setStatusSnapshot(snapshot('running'))
    expect(app.toast).toBe('')
  })
})
