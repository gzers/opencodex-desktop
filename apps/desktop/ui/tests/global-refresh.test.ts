import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { defineComponent } from 'vue'
import App from '@/App.vue'
import { useAppController } from '@/composables/useAppController'
import { useAppStore } from '@/stores/app'
import { useLifecycleStore } from '@/app/lifecycle/store'
import { usePanelStore } from '@/features/panel/store'
import { useRouteStore } from '@/stores/routes'
import type { StatusSnapshot } from '@/contracts/runtimeStatus'

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

function snapshot(runtime: StatusSnapshot['matrix']['runtime'], port: number | null): StatusSnapshot {
  return {
    matrix: { runtime, connection: 'synced', operation: 'idle' },
    facts: {
      health: 'healthy',
      runtime_label: null,
      opencodex_home: null,
      data_root: '/fixtures/data-root',
      protection: null,
      reboot_safe: null,
      service_present: null,
      shim_present: null,
      version_drift: null,
    },
    port,
    pid: '39421',
    can_start: false,
    can_stop: true,
    can_restart: true,
    source: 'live',
  }
}

describe('IMP-06 顶栏全局刷新', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
  })

  it('状态快照在途时排队补拉一次，不再直接丢弃', async () => {
    const lifecycle = useLifecycleStore()
    let release: (value: StatusSnapshot) => void = () => {}
    invoke.mockImplementationOnce(() => new Promise<StatusSnapshot>(resolve => { release = resolve }))
    invoke.mockImplementation(() => Promise.resolve(snapshot('running', 10100)))

    const first = lifecycle.loadStatusSnapshot()
    // `getStatusSnapshot` 内部是动态 import，调用不是同步发生的。
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledTimes(1))

    // 在途时再点一次：只记一笔待补拉，不并发发起第二个请求。
    const second = lifecycle.loadStatusSnapshot()
    await Promise.resolve()
    expect(invoke).toHaveBeenCalledTimes(1)

    release(snapshot('running', 10100))
    await first
    await second
    // 第一次收口后立即补拉一次，保证「点了就一定会刷到」。
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledTimes(2))
    expect(lifecycle.statusQueued).toBe(false)
  })

  it('普通页面渲染顶栏「刷新状态」按钮；面板页不保留软件顶栏（IMP-07）', async () => {
    const wrapper = mount(App, { global: { plugins: [createPinia()] } })
    useRouteStore().go('settings', { section: 'general' })
    await wrapper.vm.$nextTick()

    const button = wrapper.find('.topbar-refresh')
    expect(button.exists()).toBe(true)
    expect(button.attributes('title')).toBe('刷新状态')
    expect(button.attributes('aria-label')).toBe('刷新状态')
    expect(button.attributes('aria-busy')).toBe('false')
    expect(button.attributes('disabled')).toBeUndefined()

    // IMP-07（推翻 IMP-06）：面板页不保留软件顶栏，把整块空间让给官方页面；
    // 面板自身的操作走注入的品牌悬浮球（Rust assets/panel-hub.js），不在 dist 层渲染。
    useRouteStore().go('panel')
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.topbar').exists()).toBe(false)
    expect(wrapper.find('.topbar-refresh').exists()).toBe(false)
  })

  it('面板重载经计数器通知，面板切片不反向依赖根 store', () => {
    const panel = usePanelStore()
    expect(panel.reloadToken).toBe(0)
    panel.requestReload()
    expect(panel.reloadToken).toBe(1)
  })

  // 逐路由编排：每次刷新都必须先补全局状态快照，再按当前页补该页主数据。
  const Harness = defineComponent({
    setup() {
      const controller = useAppController()
      return { refresh: controller.refreshCurrentView }
    },
    template: '<div />',
  })

  it('按路由刷新当前页主数据（面板/扩展/设置/托盘）', async () => {
    const wrapper = mount(Harness, { global: { plugins: [createPinia()] } })
    const app = useAppStore()
    const routes = useRouteStore()
    const panel = usePanelStore()

    const snapshot = vi.spyOn(app, 'loadStatusSnapshot').mockImplementation(async () => {})
    const loadPanelUrl = vi.spyOn(app, 'loadPanelUrl').mockImplementation(async () => true)
    const requestReload = vi.spyOn(panel, 'requestReload')
    const loadExtensions = vi.spyOn(app, 'loadExtensions').mockImplementation(async () => {})
    const loadExtensionConfig = vi.spyOn(app, 'loadExtensionConfig').mockImplementation(async () => {})
    const loadRuntimeSource = vi.spyOn(app, 'loadRuntimeSource').mockImplementation(async () => true)
    const loadPreferences = vi.spyOn(app, 'loadPreferences').mockImplementation(async () => {})
    const refreshVersion = vi.spyOn(app, 'refreshRuntimeVersionFact').mockImplementation(async () => {})
    const loadSyncConfig = vi.spyOn(app, 'loadSyncConfig').mockImplementation(async () => {})

    // 面板页：快照 + 重载内嵌官方面板。
    routes.go('panel')
    await (wrapper.vm as unknown as { refresh: () => Promise<void> }).refresh()
    expect(snapshot).toHaveBeenCalled()
    expect(loadPanelUrl).toHaveBeenCalled()
    expect(requestReload).toHaveBeenCalled()

    // 扩展页：快照 + 重新发现 Skills/MCP。
    loadPanelUrl.mockClear(); requestReload.mockClear()
    routes.go('extensions')
    await (wrapper.vm as unknown as { refresh: () => Promise<void> }).refresh()
    expect(loadExtensions).toHaveBeenCalled()
    expect(loadExtensionConfig).toHaveBeenCalled()
    expect(requestReload).not.toHaveBeenCalled()

    // 设置页：快照 + 运行来源 / 偏好 / 版本事实。
    loadExtensions.mockClear()
    routes.go('settings', { section: 'general' })
    await (wrapper.vm as unknown as { refresh: () => Promise<void> }).refresh()
    expect(loadRuntimeSource).toHaveBeenCalled()
    expect(loadPreferences).toHaveBeenCalled()
    expect(refreshVersion).toHaveBeenCalled()
    expect(loadSyncConfig).not.toHaveBeenCalled() // 非同步分区不额外读同步配置

    // 设置·同步分区：额外读同步配置。
    routes.go('settings', { section: 'sync' })
    await (wrapper.vm as unknown as { refresh: () => Promise<void> }).refresh()
    expect(loadSyncConfig).toHaveBeenCalled()

    // 托盘：只补全局快照；托盘状态投影跟随同一份快照，不额外调用页面级动作。
    loadRuntimeSource.mockClear(); loadPreferences.mockClear(); refreshVersion.mockClear()
    routes.go('tray')
    await (wrapper.vm as unknown as { refresh: () => Promise<void> }).refresh()
    expect(snapshot).toHaveBeenCalled()
    expect(loadRuntimeSource).not.toHaveBeenCalled()
    expect(loadPreferences).not.toHaveBeenCalled()
  })
})
