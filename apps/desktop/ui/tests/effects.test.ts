import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import App from '@/App.vue'
import {
  DEFAULT_VISUAL_EFFECTS,
  clampVisualEffects,
  effectsStrategy,
  resolveEffectiveEffects,
  useEffectsStore,
} from '@/app/appearance/effects'
import { useRouteStore } from '@/stores/routes'
import type { PreferencesDto } from '@/features/preferences/api'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: () => Promise.resolve(null) }))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: () => ({ matches: false, addEventListener: () => {}, removeEventListener: () => {} }),
})

function dto(overrides: Partial<PreferencesDto> = {}): PreferencesDto {
  return {
    schemaVersion: 1,
    themeNeedsImport: false,
    networkProxyMode: 'none',
    networkProxyScheme: 'http',
    networkProxyHost: '',
    networkNoProxy: '',
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

const effectsAttr = () => document.documentElement.getAttribute('data-effects')

describe('三档特效 · 纯函数策略', () => {
  it('非法值回退默认高档，不臆造档位', () => {
    expect(clampVisualEffects('low')).toBe('low')
    expect(clampVisualEffects('mid')).toBe('mid')
    expect(clampVisualEffects('ultra')).toBe(DEFAULT_VISUAL_EFFECTS)
    expect(clampVisualEffects(undefined)).toBe('high')
    expect(clampVisualEffects(null)).toBe('high')
  })

  it('用户档位是有效档位；中档保留静态光场，后台只暂停调度', () => {
    for (const tier of ['high', 'mid', 'low'] as const) expect(resolveEffectiveEffects(tier)).toBe(tier)
    const high = effectsStrategy('high', true)
    expect(high.animated).toBe(true)
    expect(high.lightAllowed).toBe(true)
    const mid = effectsStrategy('mid', true)
    expect(mid.animated).toBe(false)
    expect(mid.lightAllowed).toBe(true)
    const low = effectsStrategy('low', true)
    expect(low.solid).toBe(true)
    expect(low.lightAllowed).toBe(false)
    const hidden = effectsStrategy('high', false)
    expect(hidden.effective).toBe('high')
    expect(hidden.animated).toBe(false)
    expect(hidden.lightAllowed).toBe(false)
  })

})

describe('三档特效 · 外观 store 唯一写入 data-effects', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    document.documentElement.removeAttribute('data-effects')
  })

  it('设置档位随即写属性；低档为 low', () => {
    const store = useEffectsStore()
    store.setSetting('low')
    expect(effectsAttr()).toBe('low')
    store.setSetting('high')
    expect(effectsAttr()).toBe('high')
  })

  it('页面与原生前后台不能改写用户档位', () => {
    const store = useEffectsStore()
    store.setSetting('high')
    store.setVisible(false)
    store.setForeground(false)
    expect(effectsAttr()).toBe('high')
    expect(store.strategy.animated).toBe(false)
    store.setVisible(true)
    store.setForeground(true)
    expect(store.strategy.animated).toBe(true)
    store.setSetting('mid')
    expect(store.strategy.lightAllowed).toBe(true)
    expect(store.strategy.animated).toBe(false)
  })

})

describe('三档特效 · 偏好接线', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    document.documentElement.removeAttribute('data-effects')
    document.body.innerHTML = ''
  })

  it('加载偏好即把保存档位落到 data-effects', async () => {
    invoke.mockImplementation((command: string) => {
      if (command === 'get_preferences') return Promise.resolve(dto({ visualEffects: 'low' }))
      return Promise.resolve(undefined)
    })
    const { usePreferencesStore } = await import('@/features/preferences/store')
    await usePreferencesStore().load()
    expect(effectsAttr()).toBe('low')
  })

  it('保存失败时回退到持久化档位与属性，不显示未持久化的选择', async () => {
    const { usePreferencesStore } = await import('@/features/preferences/store')
    const store = usePreferencesStore()
    store.data = dto({ visualEffects: 'high' })
    useEffectsStore().setSetting('high')
    invoke.mockImplementation((command: string) => {
      if (command === 'save_preferences') return Promise.reject(new Error('write failed'))
      return Promise.resolve(undefined)
    })
    const ok = await store.setVisualEffects('low')
    expect(ok).toBe(false)
    expect(store.data?.visualEffects).toBe('high')
    expect(effectsAttr()).toBe('high')
  })

  it('保存成功后以回读值为准应用档位', async () => {
    const { usePreferencesStore } = await import('@/features/preferences/store')
    const store = usePreferencesStore()
    store.data = dto({ visualEffects: 'high' })
    invoke.mockImplementation((command: string, args?: { preferences?: PreferencesDto }) => {
      if (command === 'save_preferences') return Promise.resolve(args?.preferences ?? dto())
      return Promise.resolve(undefined)
    })
    const ok = await store.setVisualEffects('mid')
    expect(ok).toBe(true)
    expect(store.data?.visualEffects).toBe('mid')
    expect(effectsAttr()).toBe('mid')
  })

  it('通用设置暴露三档选择控件', async () => {
    invoke.mockImplementation((command: string) => {
      if (command === 'get_preferences') return Promise.resolve(dto())
      return Promise.resolve(undefined)
    })
    const wrapper = mount(App, { attachTo: document.body, global: { plugins: [createPinia()] } })
    useRouteStore().go('settings', { section: 'general' })
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()
    const row = wrapper.find('[data-testid="setting-visual-effects"]')
    expect(row.exists()).toBe(true)
    await row.find('.select-trigger').trigger('click')
    await wrapper.vm.$nextTick()
    const options = [...document.querySelectorAll('.select-menu [role=option]')].map(o => o.textContent)
    expect(options).toEqual(['高（默认）', '中', '低'])
    wrapper.unmount()
  })
})
