import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { afterEach, expect, it, vi } from 'vitest'
import { installEffectsRuntime, useEffectsStore } from '@/app/appearance/effects'
import RuntimeMotionMark from '@/features/runtime/components/RuntimeMotionMark.vue'
const invoke = vi.fn(), listen = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }))
vi.mock('@tauri-apps/api/event', () => ({ listen: (...args: unknown[]) => listen(...args) }))
afterEach(() => vi.unstubAllGlobals())
it('native events override an older foreground query and stop the overview renderer', async () => {
  setActivePinia(createPinia())
  vi.stubGlobal('__TAURI_INTERNALS__', {})
  vi.stubGlobal('matchMedia', () => ({ matches: false, addEventListener: () => {} }))
  const pending = new Map<number, FrameRequestCallback>(); let id = 0
  vi.stubGlobal('requestAnimationFrame', (cb: FrameRequestCallback) => { pending.set(++id, cb); return id })
  vi.stubGlobal('cancelAnimationFrame', (key: number) => pending.delete(key))
  let event: (value: { payload: boolean }) => void = () => {}
  listen.mockImplementation((name, callback) => { if (name === 'app-foreground-changed') event = callback; return Promise.resolve(() => {}) })
  let resolve: (value: boolean) => void = () => {}
  invoke.mockImplementation(() => new Promise<boolean>(done => { resolve = done }))
  installEffectsRuntime(); await flushPromises()
  expect(useEffectsStore().strategy.animated).toBe(false)
  event({ payload: true }); resolve(false); await flushPromises()
  expect(useEffectsStore().strategy.animated).toBe(true)
  const wrapper = mount(RuntimeMotionMark, { props: { state: 'running', active: true } })
  expect(pending.size).toBe(1)
  event({ payload: false }); await nextTick()
  expect(useEffectsStore().effective).toBe('high')
  expect(pending.size).toBe(0)
  event({ payload: true }); await nextTick()
  expect(pending.size).toBe(1)
  wrapper.unmount(); expect(pending.size).toBe(0)
})
