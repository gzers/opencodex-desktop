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

  it('系统减少动态把高档降为中档；低档仍为低档', () => {
    expect(resolveEffectiveEffects('high', true)).toBe('mid')
    expect(resolveEffectiveEffects('high', false)).toBe('high')
    expect(resolveEffectiveEffects('mid', true)).toBe('mid')
    expect(resolveEffectiveEffects('low', true)).toBe('low')
    expect(resolveEffectiveEffects('low', false)).toBe('low')
  })

  it('策略分开用户保存值与有效表现，并区分可见性', () => {
    const high = effectsStrategy('high', false, true)
    expect(high.setting).toBe('high')
    expect(high.effective).toBe('high')
    expect(high.ambientAllowed).toBe(true)
    expect(high.solid).toBe(false)

    // 系统减少动态：有效降中档，连续动画关闭，但保留玻璃（非实底）。
    const reduced = effectsStrategy('high', true, true)
    expect(reduced.effective).toBe('mid')
    expect(reduced.ambientAllowed).toBe(false)
    expect(reduced.solid).toBe(false)

    // 低档：实底，无动画。
    const low = effectsStrategy('low', false, true)
    expect(low.effective).toBe('low')
    expect(low.solid).toBe(true)
    expect(low.animated).toBe(false)

    // 高档但页面隐藏：暂停连续动画，但不改变有效档位（返回后恢复）。
    const hidden = effectsStrategy('high', false, false)
    expect(hidden.effective).toBe('high')
    expect(hidden.ambientAllowed).toBe(false)
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

  it('系统减少动态把高档有效表现降为中档，但不改用户保存值', () => {
    const store = useEffectsStore()
    store.setSetting('high')
    store.setReducedMotion(true)
    expect(effectsAttr()).toBe('mid')
    expect(store.setting).toBe('high')
    // 解除系统限制后恢复用户选择。
    store.setReducedMotion(false)
    expect(effectsAttr()).toBe('high')
  })

  it('低档在系统减少动态下仍为低档', () => {
    const store = useEffectsStore()
    store.setSetting('low')
    store.setReducedMotion(true)
    expect(effectsAttr()).toBe('low')
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
    const wrapper = mount(App, { global: { plugins: [createPinia()] } })
    useRouteStore().go('settings', { section: 'general' })
    await wrapper.vm.$nextTick()
    await wrapper.vm.$nextTick()
    const row = wrapper.find('[data-testid="setting-visual-effects"]')
    expect(row.exists()).toBe(true)
    const options = row.findAll('.select-option').map(o => o.text())
    expect(options).toEqual(['高（默认）', '中', '低'])
  })
})
