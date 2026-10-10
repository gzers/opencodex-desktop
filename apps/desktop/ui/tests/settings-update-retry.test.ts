import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'
import { useModalStore } from '@/app/modal/store'

const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: async () => null }))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: () => ({ matches: false, addEventListener() {}, removeEventListener() {} }),
})

const available = {
  channel: 'stable', currentVersion: '0.1.3', availableVersion: '0.1.6',
  signatureVerified: false, error: null, lastCheckedAt: '2026-10-05T00:00:00Z',
}
let wrappers: VueWrapper[] = []
let pinia: ReturnType<typeof createPinia>
const install = vi.fn()

async function mountUpgrade() {
  useRouteStore().go('settings', { section: 'upgrade' })
  const wrapper = mount(SettingsRoute, { global: { plugins: [pinia] } })
  wrappers.push(wrapper)
  await flushPromises()
  return wrapper
}

function button(wrapper: VueWrapper, label: string) {
  const found = wrapper.findAll('button').find(node => node.text() === label)
  expect(found, label).toBeDefined()
  return found!
}

beforeEach(() => {
  window.location.hash = ''
  pinia = createPinia()
  setActivePinia(pinia)
  install.mockReset().mockResolvedValue(undefined)
  invoke.mockReset().mockImplementation((command: string) => {
    if (command === 'get_update_status') return Promise.resolve({ ...available })
    if (command === 'check_for_update') return Promise.resolve({ status: 'available', update: { ...available } })
    if (command === 'install_update') return install()
    return Promise.resolve(null)
  })
})

afterEach(() => {
  wrappers.forEach(wrapper => wrapper.unmount())
  wrappers = []
  vi.restoreAllMocks()
})

describe('settings app-update installation', () => {
  it('repeated page mounts read shared metadata without network checks and allow manual checking', async () => {
    for (let i = 0; i < 20; i++) {
      const wrapper = await mountUpgrade()
      expect(button(wrapper, '检查应用更新').attributes('disabled')).toBeUndefined()
      wrapper.unmount(); wrappers = []
    }
    expect(invoke.mock.calls.filter(call => call[0] === 'check_for_update')).toHaveLength(0)
    const wrapper = await mountUpgrade()
    await button(wrapper, '检查应用更新').trigger('click'); await flushPromises()
    expect(invoke.mock.calls.filter(call => call[0] === 'check_for_update')).toHaveLength(1)
  })
  it('allows retry on the same page after the real store handles an installation failure', async () => {
    install.mockRejectedValueOnce(new Error('download interrupted'))
    const wrapper = await mountUpgrade()
    await button(wrapper, '安装更新').trigger('click')
    useModalStore().resolve('confirm')
    await flushPromises()
    expect(install).toHaveBeenCalledTimes(1)
    expect(wrapper.text()).toContain('更新安装失败；已保留当前版本。')
    expect(button(wrapper, '重新安装').attributes('disabled')).toBeUndefined()

    await button(wrapper, '重新安装').trigger('click')
    expect(useModalStore().current?.title).toBe('安装应用更新')
    useModalStore().resolve('confirm')
    await flushPromises()
    expect(install).toHaveBeenCalledTimes(2)
    expect(useAppStore().appUpdateError).toBe('')
  })

  it('prevents duplicate confirmation and allows reopening after cancel', async () => {
    const wrapper = await mountUpgrade()
    const open = vi.spyOn(useAppStore(), 'openModal')
    await button(wrapper, '安装更新').trigger('click')
    await button(wrapper, '重新安装').trigger('click')
    expect(open).toHaveBeenCalledTimes(1)
    expect(install).not.toHaveBeenCalled()
    useModalStore().resolve('cancel')
    await flushPromises()
    await button(wrapper, '安装更新').trigger('click')
    expect(open).toHaveBeenCalledTimes(2)
  })

  it('blocks both buttons during installation, including after the page remounts', async () => {
    let finish!: () => void
    install.mockImplementationOnce(() => new Promise<void>(resolve => { finish = resolve }))
    const wrapper = await mountUpgrade()
    await button(wrapper, '安装更新').trigger('click')
    useModalStore().resolve('confirm')
    await flushPromises()
    expect(button(wrapper, '安装中').attributes('disabled')).toBeDefined()
    expect(button(wrapper, '重新安装').attributes('disabled')).toBeDefined()
    wrapper.unmount()
    wrappers = []
    const remounted = await mountUpgrade()
    await button(remounted, '重新安装').trigger('click')
    expect(useModalStore().current).toBeNull()
    expect(install).toHaveBeenCalledTimes(1)
    finish()
    await flushPromises()
  })

  it('releases confirmation state even if the store unexpectedly rejects', async () => {
    const wrapper = await mountUpgrade()
    vi.spyOn(useAppStore(), 'installAppUpdate').mockRejectedValueOnce(new Error('unexpected rejection'))
    await button(wrapper, '安装更新').trigger('click')
    useModalStore().resolve('confirm')
    await flushPromises()
    await button(wrapper, '重新安装').trigger('click')
    expect(useModalStore().current?.title).toBe('安装应用更新')
  })
})
