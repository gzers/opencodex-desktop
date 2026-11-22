import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import App from '@/App.vue'
import { useLifecycleStore } from '@/app/lifecycle/store'
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

describe('app topbar', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
  })

  it('does not render duplicate traffic controls', () => {
    const wrapper = mount(App, { global: { plugins: [createPinia()] } })
    expect(wrapper.find('.traffic').exists()).toBe(false)
  })

  it('renders the product logo and title in the sidebar brand area', () => {
    const wrapper = mount(App, { global: { plugins: [createPinia()] } })
    expect(wrapper.find('.titlebar').text()).toBe('')
    expect(wrapper.find('.side .brand .brand-mark').exists()).toBe(true)
    expect(wrapper.find('.side .brand-copy strong').text()).toBe('OpenCodeX-Desktop')
    expect(wrapper.find('.side .brand-copy span').text()).toBe('Desktop Manager')
  })

  it('keeps the logo in the narrow panel rail without a duplicate titlebar brand', async () => {
    const wrapper = mount(App, { global: { plugins: [createPinia()] } })
    useRouteStore().go('panel')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.app-shell.panel-mode').exists()).toBe(true)
    expect(wrapper.find('.side .brand .brand-mark').exists()).toBe(true)
    expect(wrapper.find('.side .brand-copy').exists()).toBe(true)
    expect(wrapper.find('.titlebar-brand').exists()).toBe(false)
  })

  it('does not render runtime address in the titlebar', async () => {
    const wrapper = mount(App, { global: { plugins: [createPinia()] } })
    useLifecycleStore().statusSnapshot = snapshot('stopped', 10100)
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.window-address').exists()).toBe(false)

    useLifecycleStore().statusSnapshot = snapshot('running', 10100)
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.window-address').exists()).toBe(false)
  })
})
