import { createPinia, setActivePinia } from 'pinia'
import { mount, type VueWrapper } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import App from '@/App.vue'
import { useAppController } from '@/composables/useAppController'
import { applyStartupRoute, waitForSettledSnapshot } from '@/startup'
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'
import type { PreferencesDto } from '@/features/preferences/api'
import type { RuntimeState, StatusSnapshot } from '@/contracts/runtimeStatus'
import { usePreferencesStore } from '@/features/preferences/store'
import { useFeedbackStore } from '@/app/feedback/store'

const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: () => Promise.resolve(null),
}))

// 落地到面板路由时组件会同步内嵌面板；这里直接桩掉，避免用例结束后才落地的原生调用。
vi.mock('@/features/panel/api', () => ({
  syncEmbeddedPanel: () =>
    Promise.resolve({ visible: true, panelUrl: 'http://127.0.0.1:7317/web' }),
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: () => ({ matches: false, addEventListener: () => {}, removeEventListener: () => {} }),
})

function dto(overrides: Partial<PreferencesDto> = {}): PreferencesDto {
  return {
    schemaVersion: 1,
    themeNeedsImport: false,
    interfaceScale: 100,
    launchMain: true,
    autoPanel: true,
    panelMode: 'embedded',
    keepProxyOnClose: true,
    lifecycleNotifications: true,
    syncConflictAlerts: true,
    launchWithCodex: true,
    autoBackupUpgrade: true,
    autoBackupImport: true,
    autoBackupSync: true,
    backupRetention: '10',
    backupIntegrity: 'sha-256',
    backupIncludeSkills: true,
    exportIncludeSkills: true,
    mcpConflictPolicy: 'ask',
    mcpMask: true,
    backupIncludeMcp: true,
    exportIncludeMcp: true,
    logRetention: '30d-10000',
    notificationRetention: '30',
    startupCleanup: true,
    cleanupBackupSummary: true,
    cliEnabled: false,
    syncConflictPolicy: 'ask',
    coldSync: true,
    backupBeforeOverwrite: true,
    appUpdateChannel: 'stable', appUpdateAutoCheck: true, appUpdateCheckIntervalSeconds: 86400, theme: 'system',
    visualEffects: 'high',
    glowRender: 'mesh',
    ...overrides,
  }
}

async function flush(wrapper?: VueWrapper) {
  await wrapper?.vm.$nextTick()
  await new Promise(resolve => setTimeout(resolve, 0))
  await wrapper?.vm.$nextTick()
}

function snapshot(runtime: RuntimeState): StatusSnapshot {
  return {
    matrix: { runtime, connection: 'unconfigured', operation: 'idle' },
    facts: {
      health: runtime === 'running' ? 'healthy' : 'unknown',
      runtime_label: null,
      opencodex_home: null,
      data_root: '/tmp/data-root',
      protection: null,
      reboot_safe: null,
      service_present: null,
      shim_present: null,
      version_drift: null,
    },
    port: runtime === 'running' ? 7317 : null,
    pid: null,
    can_start: true,
    can_stop: runtime === 'running',
    can_restart: runtime === 'running',
    source: 'live',
  }
}

/** 复现 `main.ts` 的启动步骤：取偏好 → 等第一份定档快照 → 写落地 hash。 */
async function bootstrapRoute(
  invokeImpl: (command: string) => Promise<unknown>,
  location?: Pick<Location, 'hash'>,
) {
  invoke.mockImplementation((command: string) => invokeImpl(command as string))
  const pinia = createPinia()
  setActivePinia(pinia)
  const app = useAppStore()
  await app.loadPreferences()
  const settled = await waitForSettledSnapshot({
    read: () => app.statusSnapshot,
    refresh: () => app.loadStatusSnapshot(),
    delay: async () => {},
    timeoutMs: 200,
  })
  if (location) applyStartupRoute(settled, location)
  else applyStartupRoute(settled)
  return { pinia, app }
}

describe('startup preferences', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    invoke.mockResolvedValue(undefined)
    window.location.hash = ''
  })

  // 2026-09-24：落地页改由「面板是否已启动」决定（不再是偏好开关）。
  // 这里按 `main.ts` 的同一步骤复现「偏好 → 定档快照 → 落地 hash → 实际渲染的路由」。
  it('lands on the official panel at startup when the panel is already running', async () => {
    const { pinia } = await bootstrapRoute(command =>
      command === 'get_preferences'
        ? Promise.resolve(dto())
        : command === 'get_status_snapshot'
          ? Promise.resolve(snapshot('running'))
          : Promise.resolve(undefined),
    )
    expect(window.location.hash).toBe('#panel')
    const wrapper = mount(App, { global: { plugins: [pinia] } })
    await flush(wrapper)
    expect(useRouteStore().current).toBe('panel')
  })

  it('stays on the overview when the panel is not running', async () => {
    const { pinia } = await bootstrapRoute(command =>
      command === 'get_preferences'
        ? Promise.resolve(dto())
        : command === 'get_status_snapshot'
          ? Promise.resolve(snapshot('stopped'))
          : Promise.resolve(undefined),
    )
    expect(window.location.hash).toBe('#overview')
    const wrapper = mount(App, { global: { plugins: [pinia] } })
    await flush(wrapper)
    expect(useRouteStore().current).toBe('overview')
  })

  it('landing page follows the panel state, not the retired autoPanel preference', async () => {
    const { pinia } = await bootstrapRoute(command =>
      command === 'get_preferences'
        ? Promise.resolve(dto({ autoPanel: false }))
        : command === 'get_status_snapshot'
          ? Promise.resolve(snapshot('running'))
          : Promise.resolve(undefined),
    )
    expect(window.location.hash).toBe('#panel')
    const wrapper = mount(App, { global: { plugins: [pinia] } })
    await flush(wrapper)
    expect(useRouteStore().current).toBe('panel')
  })

  it('does not steal an explicit hash route at startup', async () => {
    window.location.hash = '#logs'
    const { pinia } = await bootstrapRoute(command =>
      command === 'get_preferences'
        ? Promise.resolve(dto())
        : command === 'get_status_snapshot'
          ? Promise.resolve(snapshot('running'))
          : Promise.resolve(undefined),
    )
    const wrapper = mount(App, { global: { plugins: [pinia] } })
    await flush(wrapper)
    expect(useRouteStore().current).toBe('logs')
  })
})

describe('lifecycle notification preference', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    invoke.mockResolvedValue(undefined)
    window.location.hash = ''
  })

  const Harness = defineComponent({
    setup() {
      const controller = useAppController(false)
      return { controller }
    },
    render: () => h('div'),
  })

  // 回归：「启停结果通知」此前没有任何消费方，关闭后仍然会弹启停 Toast。
  it('suppresses lifecycle toasts when the preference is off', async () => {
    invoke.mockImplementation((command: string) =>
      command === 'process_action'
        ? Promise.resolve({ lifecycleState: 'starting', result: 'started', canStart: false, canStop: true, canRestart: true })
        : Promise.resolve(undefined),
    )
    const wrapper = mount(Harness)
    const app = useAppStore()
    usePreferencesStore().data = dto({ lifecycleNotifications: false })
    useFeedbackStore().toast = ''
    wrapper.vm.controller.handleTrayRequests(['start'])
    await flush(wrapper)
    await flush(wrapper)
    expect(app.toast).toBe('')
    expect(invoke).toHaveBeenCalledWith('process_action', { request: { action: 'start', confirm: true } })
  })

  it('shows lifecycle toasts when the preference is on', async () => {
    invoke.mockImplementation((command: string) =>
      command === 'process_action'
        ? Promise.resolve({ lifecycleState: 'starting', result: 'started', canStart: false, canStop: true, canRestart: true })
        : Promise.resolve(undefined),
    )
    const wrapper = mount(Harness)
    const app = useAppStore()
    usePreferencesStore().data = dto({ lifecycleNotifications: true })
    useFeedbackStore().toast = ''
    wrapper.vm.controller.handleTrayRequests(['start'])
    await flush(wrapper)
    await flush(wrapper)
    expect(app.toast).not.toBe('')
  })
})
