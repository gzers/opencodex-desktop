import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useAboutStore } from '@/features/about/store'
import { useRouteStore } from '@/stores/routes'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: async () => null }))

it('Windows explains unsupported IPC and cannot claim the CLI is online', async () => {
  const pinia = createPinia()
  setActivePinia(pinia)
  invoke.mockResolvedValue(null)
  useAboutStore().app = {
    name: 'OpenCodeX Desktop', version: '0.1.8', platform: 'Windows',
    identifier: 'com.gzers.opencodex.desktop', framework: 'Tauri v2', license: 'MIT License',
  }
  useRouteStore().go('settings', { section: 'cli' })
  const wrapper = mount(SettingsRoute, { global: { plugins: [pinia] } })
  try {
    await flushPromises()
    const row = wrapper.findAll('.setting-row').find(row => row.text().includes('启用 CLI 控制面'))!
    expect(row.text()).toContain('Windows 暂不支持 CLI 控制面')
    expect(row.find('button.toggle').attributes('disabled')).toBeDefined()
    await row.find('button.toggle').trigger('click')
    expect(invoke.mock.calls.some(([command]) => command === 'save_preferences')).toBe(false)
    expect(wrapper.find('.cli-badge.ok').exists()).toBe(false)
    expect(wrapper.text()).not.toContain('本机 IPC 已启动')
  } finally { wrapper.unmount() }
})
