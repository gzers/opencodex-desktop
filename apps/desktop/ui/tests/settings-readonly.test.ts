import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useRouteStore } from '@/stores/routes'

const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    addEventListener: () => {},
    removeEventListener: () => {},
  }),
})

function mountSection(section: 'general' | 'backup' | 'extensions') {
  const pinia = createPinia()
  setActivePinia(pinia)
  const routes = useRouteStore()
  routes.go('settings', { section })
  const wrapper = mount(SettingsRoute, { global: { plugins: [pinia] } })
  return { wrapper, routes }
}

function rowText(wrapper: ReturnType<typeof mount>, title: string) {
  const row = wrapper.findAll('.setting-row').find(node => node.text().includes(title))
  expect(row, `找不到设置行：${title}`).toBeTruthy()
  return row!
}

// Q2-33 收口：这些契约/边界上无法兑现为行为的设置，界面只陈述事实，不再提供假开关。
// 「随 Codex 启动 OpenCodex」已于 2026-09-27 改为经官方 shim 真实读写，见
// settings-codex-shim.test.ts。
describe('settings that cannot be honored are read-only', () => {
  beforeEach(() => {
    invoke.mockReset()
    invoke.mockResolvedValue(null)
  })

  it('fixes backup integrity to SHA-256 instead of offering unimplemented algorithms', async () => {
    const { wrapper } = mountSection('backup')
    await nextTick()
    const row = rowText(wrapper, '备份完整性校验')
    expect(row.find('.setting-readonly').text()).toBe('SHA-256')
    expect(row.find('.select').exists()).toBe(false)
    await wrapper.unmount()
  })

  it('states that backups do not carry Skills / MCP snapshots', async () => {
    const { wrapper } = mountSection('extensions')
    await nextTick()
    for (const title of ['备份包含 Skills 目录', '备份包含 MCP 配置']) {
      const row = rowText(wrapper, title)
      expect(row.find('.setting-readonly').text()).toBe('不含')
      expect(row.find('button.toggle').exists()).toBe(false)
    }
    await wrapper.unmount()
  })

  it('fixes the MCP same-name policy to always-ask', async () => {
    const { wrapper } = mountSection('extensions')
    await nextTick()
    const row = rowText(wrapper, '同名冲突策略')
    expect(row.find('.setting-readonly').text()).toBe('每次询问')
   expect(row.find('.select').exists()).toBe(false)
   await wrapper.unmount()
 })

  // 2026-09-27 用户批注：落地页由面板状态决定这件事不必占一行展示。
  it('不再展示「首次落地页」这一行，也不提供任何落地页开关', async () => {
    const { wrapper } = mountSection('general')
    await nextTick()
    expect(wrapper.text()).not.toContain('首次落地页')
    expect(wrapper.text()).not.toContain('随面板状态')
    expect(wrapper.text()).not.toContain('启动后自动打开面板')
    await wrapper.unmount()
  })
})
