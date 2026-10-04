import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import PanelRoute from '@/routes/PanelRoute.vue'
import { useAppStore } from '@/stores/app'
import { useLifecycleStore } from '@/app/lifecycle/store'
import { useFeedbackStore } from '@/app/feedback/store'
import { useThemeStore } from '@/app/appearance/theme'
import type { StatusSnapshot } from '@/contracts/runtimeStatus'
import type { PreferencesDto } from '@/features/preferences/api'

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

const snapshot: StatusSnapshot = {
  matrix: { runtime: 'running', connection: 'synced', operation: 'idle' },
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
  port: 10100,
  pid: '39421',
  can_start: false,
  can_stop: true,
  can_restart: true,
  source: 'live',
}

function preferences(panelMode: PreferencesDto['panelMode']): PreferencesDto {
  return {
    interfaceScale: 100,
    launchMain: true,
    autoPanel: true,
    panelMode,
    keepProxyOnClose: true,
    lifecycleNotifications: true,
    syncConflictAlerts: true,
    launchWithCodex: true,
    autoBackupUpgrade: true,
    autoBackupImport: true,
    autoBackupSync: true,
    backupRetention: '5',
    backupIntegrity: 'sha-256',
    backupIncludeSkills: true,
    exportIncludeSkills: true,
    mcpConflictPolicy: 'ask',
    mcpMask: true,
    backupIncludeMcp: true,
    exportIncludeMcp: true,
    logRetention: '7d-5000',
    notificationRetention: '7',
    startupCleanup: true,
    cleanupBackupSummary: true,
    cliEnabled: false,
    syncConflictPolicy: 'ask',
    coldSync: true,
    backupBeforeOverwrite: true,
    appUpdateChannel: 'stable', appUpdateAutoCheck: true, appUpdateCheckIntervalSeconds: 86400, theme: 'system',
    visualEffects: 'high',
    glowRender: 'mesh',
  }
}

describe('official panel embedding', () => {
  beforeEach(() => {
    const pinia = createPinia()
    setActivePinia(pinia)
    invoke.mockReset()
    window.localStorage.clear()
    vi.spyOn(HTMLElement.prototype, 'getBoundingClientRect').mockReturnValue({
      x: 64,
      y: 44,
      top: 44,
      left: 64,
      right: 1180,
      bottom: 760,
      width: 1116,
      height: 716,
      toJSON: () => ({}),
    } as DOMRect)
    ;(globalThis as unknown as { __panelTestPinia?: unknown }).__panelTestPinia = pinia
  })

  it('derives only the runtime local panel URL from status', async () => {
    const app = useAppStore()
    useLifecycleStore().statusSnapshot = snapshot
    await expect(app.loadPanelUrl()).resolves.toBe(true)
    expect(app.panelUrl).toBe('http://127.0.0.1:10100/web')
    expect(app.panelError).toBe('')
  })

  it('blocks embedding when runtime is not running', async () => {
    const app = useAppStore()
    useLifecycleStore().statusSnapshot = { ...snapshot, matrix: { ...snapshot.matrix, runtime: 'stopped' }, port: null }
    await expect(app.loadPanelUrl()).resolves.toBe(false)
    expect(app.panelUrl).toBe('')
    expect(app.panelError).toContain('运行且健康可用')
  })

  it('requests a same-window embedded webview scoped to the local web panel', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'get_preferences') return preferences('embedded')
      if (command === 'sync_embedded_panel') return { visible: true, panelUrl: 'http://127.0.0.1:10100/web' }
      throw new Error(`unexpected command: ${command}`)
    })
    const wrapper = mount(PanelRoute, {
      global: {
        plugins: [(globalThis as unknown as { __panelTestPinia: ReturnType<typeof createPinia> }).__panelTestPinia],
      },
    })
    useLifecycleStore().statusSnapshot = snapshot
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith(
      'sync_embedded_panel',
      // 主题随请求下发，只驱动管理器注入浮层；官方页面样式不变。
      expect.objectContaining({ request: expect.objectContaining({ action: 'show', scale: 100, theme: 'dark' }) }),
    ))
    expect(wrapper.find('iframe').exists()).toBe(false)
    await wrapper.unmount()
  })

  // 官方面板只在首屏前读自己的 `ocx-theme`，所以管理器改明暗设置时必须让面板重新进入首屏，
  // 并把新的设置（themeSetting）与浮层配色（theme）一并下发。
  it('reloads the panel with the new theme when the manager theme setting changes', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'get_preferences') return preferences('embedded')
      if (command === 'sync_embedded_panel') return { visible: true, panelUrl: 'http://127.0.0.1:10100/web' }
      throw new Error(`unexpected command: ${command}`)
    })
    const wrapper = mount(PanelRoute, {
      global: {
        plugins: [(globalThis as unknown as { __panelTestPinia: ReturnType<typeof createPinia> }).__panelTestPinia],
      },
    })
    useLifecycleStore().statusSnapshot = snapshot
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith(
      'sync_embedded_panel',
      expect.objectContaining({ request: expect.objectContaining({ action: 'show' }) }),
    ))
    window.dispatchEvent(new CustomEvent('ocxd-panel', { detail: { kind: 'load', value: 'ready' } }))
    await wrapper.vm.$nextTick()
    invoke.mockClear()
    useThemeStore().apply('light')
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith(
      'sync_embedded_panel',
      expect.objectContaining({
        request: expect.objectContaining({
          action: 'show',
          reload: true,
          theme: 'light',
          themeSetting: 'light',
        }),
      }),
    ))
    await wrapper.unmount()
  })

  it('shows offline state and only opens browser when runtime URL exists', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'get_preferences') return preferences('embedded')
      if (command === 'open_external_link') return { opened: true }
      throw new Error(`unexpected command: ${command}`)
    })
    const app = useAppStore()
    useLifecycleStore().statusSnapshot = {
      ...snapshot,
      matrix: { ...snapshot.matrix, runtime: 'stopped' },
      port: null,
      facts: { ...snapshot.facts, health: 'unknown' },
    }
    await app.loadPreferences()
    const wrapper = mount(PanelRoute, {
      global: {
        plugins: [(globalThis as unknown as { __panelTestPinia: ReturnType<typeof createPinia> }).__panelTestPinia],
      },
    })
    await vi.waitFor(() => expect(wrapper.find('.panel-state').text()).toContain('官方面板需要 OpenCodex 处于运行且健康可用状态'))
    expect(wrapper.find('.panel-state').text()).toContain('官方面板需要 OpenCodex 处于运行且健康可用状态')

    const retry = wrapper.findAll('.panel-state button').find(button => button.text() === '重试')
    await retry?.trigger('click')
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.panel-state').text()).toContain('官方面板需要 OpenCodex 处于运行且健康可用状态')

    const browser = wrapper.findAll('.panel-state button').find(button => button.text() === '在浏览器打开')
    await browser?.trigger('click')
    await wrapper.vm.$nextTick()
    expect(invoke).not.toHaveBeenCalledWith('open_external_link', { url: 'http://127.0.0.1:10100/web' })
  })

  // 冷启动可能先落到面板路由、首个状态快照还没到；此时不能给出「请先启动服务」，
  // 也不能要求用户手点「重试」——状态就绪后应自动嵌入。
  it('waits for the first status snapshot and embeds as soon as it is running', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'get_preferences') return preferences('embedded')
      if (command === 'sync_embedded_panel') return { visible: true, panelUrl: 'http://127.0.0.1:10100/web' }
      throw new Error(`unexpected command: ${command}`)
    })
    const pinia = (globalThis as unknown as { __panelTestPinia: ReturnType<typeof createPinia> }).__panelTestPinia
    const app = useAppStore()

    const wrapper = mount(PanelRoute, { global: { plugins: [pinia] } })
    await vi.waitFor(() => expect(wrapper.text()).toContain('正在等待 OpenCodex 状态'))
    expect(wrapper.text()).not.toContain('请先启动服务')
    expect(invoke).not.toHaveBeenCalledWith('sync_embedded_panel', expect.anything())

    app.setStatusSnapshot(snapshot)
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith(
      'sync_embedded_panel',
      expect.objectContaining({ request: expect.objectContaining({ action: 'show' }) }),
    ))
    await wrapper.unmount()
  })

  it('browser preference keeps official GUI out of the embedded frame', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'get_preferences') return preferences('browser')
      throw new Error(`unexpected command: ${command}`)
    })
    const app = useAppStore()
    useLifecycleStore().statusSnapshot = snapshot
    await app.loadPreferences()
    const wrapper = mount(PanelRoute, {
      global: {
        plugins: [(globalThis as unknown as { __panelTestPinia: ReturnType<typeof createPinia> }).__panelTestPinia],
      },
    })
    await vi.waitFor(() => expect(wrapper.find('.panel-state').text()).toContain('浏览器兜底模式已启用'))
    expect(wrapper.find('.panel-state').text()).toContain('浏览器兜底模式已启用')
  })

  it('reuses an already loaded panel instead of reporting a false load timeout', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'get_preferences') return preferences('embedded')
      if (command === 'sync_embedded_panel') return { visible: true, panelUrl: 'http://127.0.0.1:10100/web' }
      throw new Error(`unexpected command: ${command}`)
    })
    const pinia = (globalThis as unknown as { __panelTestPinia: ReturnType<typeof createPinia> }).__panelTestPinia
    const app = useAppStore()
    useLifecycleStore().statusSnapshot = snapshot

    // 首次进入：原生视图新建，等待原生页面加载完成事件。
    const first = mount(PanelRoute, { global: { plugins: [pinia] } })
    await vi.waitFor(() => expect(first.text()).toContain('正在将官方面板嵌入主窗口'))
    window.dispatchEvent(new CustomEvent('ocxd-panel', { detail: { kind: 'load', value: 'ready' } }))
    await nextTick()
    expect(app.panelLoaded).toBe(true)
    expect(first.text()).toContain('官方面板已嵌入主窗口')
    await first.unmount()

    // 回到面板路由：原生子 WebView 被复用，不会再发页面加载事件。
    // 若把它当成「第一次嵌入」，8 秒后会把可用的面板误报成加载超时。
    const second = mount(PanelRoute, { global: { plugins: [pinia] } })
    await vi.waitFor(() => expect(second.text()).toContain('官方面板已嵌入主窗口'))
    expect(second.text()).not.toContain('加载超时')
    expect(app.panelError).toBe('')
    await second.unmount()
  })

  it('reports an explicit failure when the native panel was not embedded', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'get_preferences') return preferences('embedded')
      if (command === 'sync_embedded_panel') return { visible: false, panelUrl: null }
      throw new Error(`unexpected command: ${command}`)
    })
    const pinia = (globalThis as unknown as { __panelTestPinia: ReturnType<typeof createPinia> }).__panelTestPinia
    useLifecycleStore().statusSnapshot = snapshot
    const wrapper = mount(PanelRoute, { global: { plugins: [pinia] } })
    await vi.waitFor(() => expect(wrapper.text()).toContain('官方面板未能嵌入主窗口'))
    // 不能同时停在「正在嵌入」的加载态。
    expect(wrapper.text()).not.toContain('正在将官方面板嵌入主窗口')
    await wrapper.unmount()
  })

  it('hides the panel subview while a notification surface is visible, then restores it', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'get_preferences') return preferences('embedded')
      if (command === 'sync_embedded_panel') return { visible: true, panelUrl: 'http://127.0.0.1:10100/web' }
      throw new Error(`unexpected command: ${command}`)
    })
    const pinia = (globalThis as unknown as { __panelTestPinia: ReturnType<typeof createPinia> }).__panelTestPinia
    useLifecycleStore().statusSnapshot = snapshot

    const wrapper = mount(PanelRoute, { global: { plugins: [pinia] } })
    await vi.waitFor(() => expect(wrapper.text()).toContain('正在将官方面板嵌入主窗口'))
    window.dispatchEvent(new CustomEvent('ocxd-panel', { detail: { kind: 'load', value: 'ready' } }))
    await nextTick()
    invoke.mockClear()

    // 原生子 WebView 与主 WebView 不同层：表面出现时必须先隐藏面板，否则会被官方面板盖住。
    useFeedbackStore().toast = '偏好已保存'
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith(
      'sync_embedded_panel',
      expect.objectContaining({ request: expect.objectContaining({ action: 'hide' }) }),
    ))

    invoke.mockClear()
    useFeedbackStore().toast = ''
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith(
      'sync_embedded_panel',
      expect.objectContaining({ request: expect.objectContaining({ action: 'show' }) }),
    ))
    await wrapper.unmount()
  })
})
