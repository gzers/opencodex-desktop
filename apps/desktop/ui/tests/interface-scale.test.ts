import { createPinia, setActivePinia } from 'pinia'
import { mount, type VueWrapper } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import App from '@/App.vue'
import type { PreferencesDto } from '@/features/preferences/api'
import { useRouteStore } from '@/stores/routes'

const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: () => Promise.resolve(null),
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: () => ({ matches: false, addEventListener: () => {}, removeEventListener: () => {} }),
})

function dto(overrides: Partial<PreferencesDto> = {}): PreferencesDto {
  return {
    schemaVersion: 1,
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

function mountSettingsGeneral() {
  const wrapper = mount(App, { global: { plugins: [createPinia()] } })
  useRouteStore().go('settings', { section: 'general' })
  return wrapper
}

async function flush(wrapper: VueWrapper) {
  await wrapper.vm.$nextTick()
  await wrapper.vm.$nextTick()
  await new Promise((resolve) => setTimeout(resolve, 0))
  await wrapper.vm.$nextTick()
}

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms))
const uiZoom = () => document.documentElement.style.getPropertyValue('--ui-zoom')
const savedCalls = () => invoke.mock.calls.filter((call) => call[0] === 'save_preferences')

describe('settings · 界面缩放真正生效', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    document.documentElement.style.removeProperty('--ui-zoom')
    document.body.innerHTML = ''
    invoke.mockImplementation((command: string, args?: { preferences?: PreferencesDto }) => {
      if (command === 'get_preferences') return Promise.resolve(dto())
      if (command === 'save_preferences') return Promise.resolve(args?.preferences ?? dto())
      return Promise.resolve(undefined)
    })
  })

  // 回归：此前缩放只写进无人消费的 `--panel-zoom`，且在保存完成前多次触发会被丢弃。
  it('previews instantly and persists only the final value of a rapid drag', async () => {
    const wrapper = mountSettingsGeneral()
    await flush(wrapper)
    expect(uiZoom()).toBe('1')

    const slider = wrapper.find('input[aria-label="界面缩放"]')
    await slider.setValue('120')
    await slider.setValue('135')
    await slider.setValue('150')
    // 拖动过程中界面立即跟随，不必等落盘。
    expect(uiZoom()).toBe('1.5')

    await wait(360)
    expect(savedCalls().length).toBe(1)
    expect((savedCalls()[0][1] as { preferences: PreferencesDto }).preferences.interfaceScale).toBe(150)
    expect(uiZoom()).toBe('1.5')
  })

  it('normalizes an out-of-range readout and writes the effective value back', async () => {
    const wrapper = mountSettingsGeneral()
    await flush(wrapper)

    const input = wrapper.find('input[aria-label="界面缩放数值"]')
    ;(input.element as HTMLInputElement).value = '999'
    await input.trigger('change')
    await wait(20)

    expect(uiZoom()).toBe('2')
    expect((input.element as HTMLInputElement).value).toBe('200')
    expect((savedCalls().at(-1)![1] as { preferences: PreferencesDto }).preferences.interfaceScale).toBe(200)
  })

  it('keeps the displayed value, real ratio and persisted value consistent on failure', async () => {
    invoke.mockImplementation((command: string) => {
      if (command === 'get_preferences') return Promise.resolve(dto({ interfaceScale: 110 }))
      if (command === 'save_preferences') return Promise.reject(new Error('write failed'))
      return Promise.resolve(undefined)
    })
    const wrapper = mountSettingsGeneral()
    await flush(wrapper)
    expect(uiZoom()).toBe('1.1')

    const slider = wrapper.find('input[aria-label="界面缩放"]')
    await slider.setValue('180')
    expect(uiZoom()).toBe('1.8')
    await wait(360)

    // 保存失败：显示值、实际比例与磁盘值一起回退到上次保存值。
    expect(uiZoom()).toBe('1.1')
    expect((slider.element as HTMLInputElement).value).toBe('110')
  })

  // 回归：设置页在偏好已加载之后才挂载（应用启动后再进入设置）时，
  // 局部偏好副本停留在默认值 100，数字读数与刻度高亮都与实际比例/持久化值不符。
  it('shows the persisted scale when the settings page mounts after preferences are already loaded', async () => {
    invoke.mockImplementation((command: string) => {
      if (command === 'get_preferences') return Promise.resolve(dto({ interfaceScale: 137 }))
      return Promise.resolve(undefined)
    })
    // 保证初始路由是概览：设置页要等偏好加载完成之后才挂载。
    window.location.hash = '#overview'
    const wrapper = mount(App, { global: { plugins: [createPinia()] } })
    // 先让启动阶段的偏好加载完成，再进入设置页（此时 SettingsRoute 才挂载）。
    await flush(wrapper)
    expect(uiZoom()).toBe('1.37')
    expect(wrapper.find('input[aria-label="界面缩放数值"]').exists()).toBe(false)

    useRouteStore().go('settings', { section: 'general' })
    await flush(wrapper)

    const readout = wrapper.find('input[aria-label="界面缩放数值"]')
    expect((readout.element as HTMLInputElement).value).toBe('137')
    const slider = wrapper.find('input[aria-label="界面缩放"]')
    expect((slider.element as HTMLInputElement).value).toBe('137')
    // 没有任何刻度被标为选中（137 不是离散刻度），但绝不能错误地高亮 100。
    const active = wrapper.findAll('button.range-tick').filter(btn => btn.classes().includes('active'))
    expect(active.length).toBe(0)
    expect(uiZoom()).toBe('1.37')
  })
})
