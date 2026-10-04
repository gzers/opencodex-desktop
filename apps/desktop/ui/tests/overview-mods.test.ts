import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import App from '@/App.vue'
import { useRouteStore } from '@/stores/routes'

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

const mounted: ReturnType<typeof mount>[] = []
afterEach(() => { for (const wrapper of mounted.splice(0)) wrapper.unmount() })

function mountOverview() {
  const pinia = createPinia()
  const wrapper = mount(App, { global: { plugins: [pinia] } })
  mounted.push(wrapper)
  useRouteStore(pinia).go('overview')
  return wrapper
}

// 概览就绪形态需要环境就绪；发现调用返回一份真实形状的就绪报告（不连真实端点）。
function readyEnvironment() {
  const found = { found: true, path: '/fixtures/bin', version: null }
  return { node: { ...found }, npm: { ...found }, ocx: { ...found }, gate: 'ready', shortCircuited: false, brewFound: true }
}

describe('overview module cards', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    invoke.mockImplementation((command: string) =>
      command === 'discover_environment' ? Promise.resolve(readyEnvironment()) : Promise.resolve(undefined),
    )
  })

  it('uses the header ⋯ as the settings entry instead of a third action button', async () => {
    const wrapper = mountOverview()
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))
    await wrapper.vm.$nextTick()

    const mods = wrapper.findAll('.mods .mod')
    expect(mods.length).toBe(3)

    for (const mod of mods) {
      // 右上角 ⋯ 是真实按钮，带可读名称。
      const go = mod.find('button.mod-go')
      expect(go.exists()).toBe(true)
      expect(go.attributes('aria-label')).toBeTruthy()
      expect(go.attributes('aria-hidden')).toBeUndefined()
      // 动作区不再重复放「…设置」按钮（原型用 [data-settings-section] 隐藏）。
      const labels = mod.findAll('.mod-actions .btn').map(btn => btn.text())
      expect(labels.some(label => label.endsWith('设置'))).toBe(false)
    }
  })

  it('navigates to the matching settings section from the header ⋯', async () => {
    const wrapper = mountOverview()
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))
    await wrapper.vm.$nextTick()

    const routes = useRouteStore(wrapper.vm.$pinia)
    const upgrade = wrapper.findAll('.mods .mod')[0]
    expect(upgrade.find('.mod-head h3').text()).toBe('版本升级')

    await upgrade.find('button.mod-go').trigger('click')

    expect(routes.current).toBe('settings')
    expect(routes.settingsSection).toBe('upgrade')
  })
})
