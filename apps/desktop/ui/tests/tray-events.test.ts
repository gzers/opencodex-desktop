import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { defineComponent, h } from 'vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppController } from '@/composables/useAppController'
const invoke = vi.fn(), listen = vi.fn(), stop = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }))
vi.mock('@tauri-apps/api/event', () => ({ listen: (...args: unknown[]) => listen(...args) }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: () => Promise.resolve(null) }))
vi.mock('@/features/panel/api', () => ({ syncEmbeddedPanel: () => Promise.resolve({ visible: false, panelUrl: null }) }))
Object.defineProperty(window, 'matchMedia', { writable: true, value: () => ({ matches: false, addEventListener() {}, removeEventListener() {} }) })
const Harness = defineComponent({ setup() { useAppController(true); return () => h('div') } })
let event: () => void
describe('event-driven tray draining', () => {
  beforeEach(() => {
    setActivePinia(createPinia()); vi.useFakeTimers(); invoke.mockReset(); stop.mockReset()
    invoke.mockResolvedValue(undefined)
    listen.mockImplementation((name, cb) => { if (name === 'tray-requests-available') { event = cb; return Promise.resolve(stop) } return Promise.resolve(() => {}) })
  })
  afterEach(() => vi.useRealTimers())
  it('handles clicks immediately, coalesces in-flight events and stops on unmount', async () => {
    let drains = 0, finish: (value: unknown[]) => void = () => {}
    invoke.mockImplementation((name) => {
      if (name !== 'drain_tray_requests') return Promise.resolve(undefined)
      drains++
      if (drains === 2) return new Promise(done => { finish = done })
      return Promise.resolve([])
    })
    const wrapper = mount(Harness); await flushPromises(); expect(drains).toBe(1)
    event(); await flushPromises(); expect(drains).toBe(2)
    event(); event(); event(); expect(drains).toBe(2)
    finish([]); await flushPromises(); expect(drains).toBe(3)
    await vi.advanceTimersByTimeAsync(29_999); expect(drains).toBe(3)
    await vi.advanceTimersByTimeAsync(1); expect(drains).toBe(4)
    wrapper.unmount(); expect(stop).toHaveBeenCalledTimes(1)
    event(); await vi.advanceTimersByTimeAsync(120_000); expect(drains).toBe(4)
  })
})
