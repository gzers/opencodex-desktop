import { createPinia, setActivePinia } from 'pinia'
import { mount } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import AppModal from '@/components/AppModal.vue'
import { useRouteStore } from '@/stores/routes'
import { useAppStore } from '@/stores/app'

vi.mock('@tauri-apps/api/core', () => ({ invoke: () => Promise.resolve(undefined) }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: () => Promise.resolve(null) }))

// 回归（真机复现）：技术型文本输入（路径/URL/用户名/版本号/确认词）在 WKWebView 中会
// 被系统按「句子」自动首字母大写——例如把账号 `davuser` 变成 `Davuser`，导致真实 WebDAV
// 认证 401。这些字段必须显式关闭自动大写/自动更正/拼写检查。
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    addEventListener: () => {},
    removeEventListener: () => {},
  }),
})

function expectNoAutoText(input: { attributes: (name: string) => string | undefined }) {
  expect(input.attributes('autocapitalize')).toBe('off')
  expect(input.attributes('autocorrect')).toBe('off')
  expect(input.attributes('spellcheck')).toBe('false')
}

describe('technical text inputs disable OS auto-capitalization', () => {
  beforeEach(() => setActivePinia(createPinia()))

  it('sync endpoint fields (url / path / username) are not auto-capitalized', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    useRouteStore().go('settings', { section: 'sync' })
    const wrapper = mount(SettingsRoute, { global: { plugins: [pinia] } })
    await nextTick()
    const fields = wrapper.findAll('.field input')
    // 服务地址 / 远端目录 / 账号 / 应用密码
    expect(fields.length).toBeGreaterThanOrEqual(4)
    for (const input of fields) expectNoAutoText(input)
  })

  it('data-root path fields are not auto-capitalized', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    useRouteStore().go('settings', { section: 'installation' })
    const wrapper = mount(SettingsRoute, { global: { plugins: [pinia] } })
    await nextTick()
    for (const id of ['external-home-input', 'data-root-input', 'switch-root-input']) {
      const input = wrapper.find(`#${id}`)
      expect(input.exists(), `缺少输入框 #${id}`).toBe(true)
      expectNoAutoText(input)
    }
  })

  it('modal input fields are not auto-capitalized', async () => {
    const app = useAppStore()
    app.openModal({
      title: '测试',
      body: '正文',
      confirmLabel: '确定',
      fields: [{ key: 'value', label: '口令' }],
    })
    const wrapper = mount(AppModal)
    await nextTick()
    const input = wrapper.find('.modal-field input')
    expect(input.exists()).toBe(true)
    expectNoAutoText(input)
  })
})
