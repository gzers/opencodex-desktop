import { beforeEach, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { applyNativeAppearance } from '@/app/appearance/nativeWindow'
import { useThemeStore } from '@/app/appearance/theme'
import { useEffectsStore } from '@/app/appearance/effects'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn() }))
beforeEach(() => {
  window.localStorage.clear()
  setActivePinia(createPinia())
  delete document.documentElement.dataset.nativeMaterial
})

it('native material fallback never changes the user quality or explicit theme', () => {
  const effects = useEffectsStore(), theme = useThemeStore()
  effects.setSetting('high'); theme.apply('light')
  applyNativeAppearance({ platform: 'Windows', material: 'solid', reason: 'inactive', theme: 'dark' })
  expect(effects.setting).toBe('high')
  expect(theme.setting).toBe('light')
  expect(theme.resolved).toBe('light')
  expect(document.documentElement.dataset.nativeMaterial).toBe('solid')
})

it('Windows system-theme events update only the resolved theme, without a preference write', () => {
  const theme = useThemeStore()
  theme.apply('system')
  applyNativeAppearance({ platform: 'Windows', material: 'mica', reason: 'mica', theme: 'dark' })
  expect(theme.setting).toBe('system')
  expect(theme.resolved).toBe('dark')
  expect(window.localStorage.getItem('ocx-theme')).toBe('system')
  expect(document.documentElement.dataset.theme).toBe('dark')
  applyNativeAppearance(null)
  expect(document.documentElement.dataset.nativeMaterial).toBe('mica')
})
