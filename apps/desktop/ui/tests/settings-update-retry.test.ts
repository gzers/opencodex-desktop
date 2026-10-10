import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'
import { managerInstallProgress, managerUpdateStatus } from '@/features/updates/update'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
  Channel: class { onmessage: unknown },
}))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: async () => null }))
Object.defineProperty(window, 'matchMedia', { writable: true, value: () => ({ matches: false, addEventListener() {}, removeEventListener() {} }) })
const available = { channel: 'stable' as const, currentVersion: '0.1.9', availableVersion: '0.1.10', signatureVerified: false, error: null, lastCheckedAt: '2026-10-10T00:00:00Z' }
let wrappers: VueWrapper[] = []
let pinia: ReturnType<typeof createPinia>
const install = vi.fn()
async function mountUpgrade() {
  useRouteStore().go('settings', { section: 'upgrade' })
  const wrapper = mount(SettingsRoute, { global: { plugins: [pinia], stubs: { teleport: true } } })
  wrappers.push(wrapper); await flushPromises(); return wrapper
}
function button(wrapper: VueWrapper, label: string) {
  const found = wrapper.findAll('button').find(node => node.text() === label)
  expect(found, label).toBeDefined(); return found!
}
beforeEach(() => {
  window.location.hash = ''
  pinia = createPinia(); setActivePinia(pinia)
  managerInstallProgress.value = null; managerUpdateStatus.value = { ...available }
  install.mockReset().mockResolvedValue(undefined)
  invoke.mockReset().mockImplementation((command: string) => {
    if (command === 'get_update_status') return Promise.resolve({ ...available })
    if (command === 'check_for_update') return Promise.resolve({ status: 'available', update: { ...available } })
    if (command === 'install_update') return install()
    return Promise.resolve(null)
  })
})
afterEach(() => { wrappers.forEach(wrapper => wrapper.unmount()); wrappers = []; vi.restoreAllMocks() })

describe('settings shared update installation', () => {
  it('repeated page mounts do not check remotely; explicit check opens the shared results', async () => {
    for (let i = 0; i < 5; i++) { const wrapper = await mountUpgrade(); wrapper.unmount(); wrappers = [] }
    expect(invoke.mock.calls.filter(call => call[0] === 'check_for_update')).toHaveLength(0)
    const wrapper = await mountUpgrade()
    await button(wrapper, '检查更新').trigger('click'); await flushPromises()
    expect(invoke.mock.calls.filter(call => call[0] === 'check_for_update')).toHaveLength(1)
    expect(wrapper.findAll('[role="dialog"]')).toHaveLength(1)
  })
  it('allows retry on the same page through the real update API without another modal', async () => {
    install.mockRejectedValueOnce(new Error('download interrupted'))
    const wrapper = await mountUpgrade()
    await button(wrapper, '查看管理器更新').trigger('click'); await flushPromises()
    await button(wrapper, '安装管理器更新').trigger('click'); await flushPromises()
    expect(install).toHaveBeenCalledTimes(1)
    expect(wrapper.text()).toContain('download interrupted')
    expect(useAppStore().modal).toBeNull()
    await button(wrapper, '重试安装').trigger('click'); await flushPromises()
    expect(install).toHaveBeenCalledTimes(2)
    const args = invoke.mock.calls.find(call => call[0] === 'install_update')?.[1]
    expect(args).toMatchObject({ candidateVersion: '0.1.10', channel: 'stable', backup: true })
  })
  it('keeps the active operation observable after hiding and remounting the route', async () => {
    let finish!: () => void
    install.mockImplementationOnce(() => new Promise<void>(resolve => { finish = resolve }))
    const wrapper = await mountUpgrade()
    await button(wrapper, '查看管理器更新').trigger('click'); await flushPromises()
    await button(wrapper, '安装管理器更新').trigger('click'); await flushPromises()
    await button(wrapper, '隐藏').trigger('click')
    wrapper.unmount(); wrappers = []
    const remounted = await mountUpgrade()
    await button(remounted, '查看管理器更新').trigger('click'); await flushPromises()
    expect(button(remounted, '更新中').attributes('disabled')).toBeDefined()
    expect(install).toHaveBeenCalledTimes(1)
    finish(); await flushPromises()
  })
})
