import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useRouteStore } from '@/stores/routes'

const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: () => Promise.resolve(null) }))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: () => ({ matches: false, addEventListener: () => {}, removeEventListener: () => {} }),
})

// IMP-04 §13.3 A03：引导安装事件必须在设置页挂载时注册、卸载时成对移除，不能累积监听。
describe('settings guide-installation listener', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    invoke.mockResolvedValue(null)
  })

  it('挂载时事件可导航，卸载后事件不再生效', () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const routes = useRouteStore()
    routes.go('overview')

    const wrapper = mount(SettingsRoute, { global: { plugins: [pinia] } })
    window.dispatchEvent(new Event('opencodex:guide-installation'))
    expect(routes.current).toBe('settings')
    expect(routes.settingsSection).toBe('installation')

    routes.go('overview')
    wrapper.unmount()
    window.dispatchEvent(new Event('opencodex:guide-installation'))
    expect(routes.current).toBe('overview')
  })
})
