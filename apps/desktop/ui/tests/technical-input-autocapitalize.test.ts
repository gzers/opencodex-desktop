import { installNativeDialogStub } from './helpers/nativeDialog'
import { createPinia, setActivePinia } from 'pinia'
import { DOMWrapper, flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import WebDavConnection from '@/features/sync/WebDavConnection.vue'
import AppModal from '@/components/AppModal.vue'
import { useRouteStore } from '@/stores/routes'
import { useAppStore } from '@/stores/app'
import type { PathsSnapshotV2 } from '@/features/data-root/api'

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
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

const pages: Array<ReturnType<typeof mount>> = []
afterEach(() => { pages.splice(0).forEach(page => page.unmount()); document.body.innerHTML = '' })

describe('technical text inputs disable OS auto-capitalization', () => {
  beforeEach(() => { installNativeDialogStub(); setActivePinia(createPinia()); invoke.mockReset(); invoke.mockResolvedValue(null) })

  it('sync endpoint fields (url / path / username) are not auto-capitalized', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    useRouteStore().go('settings', { section: 'sync' })
    const wrapper = mount(WebDavConnection, { global: { plugins: [pinia] } }); pages.push(wrapper)
    await nextTick()
    const fields = wrapper.findAll('.field input')
    // 服务地址 / 远端目录 / 账号 / 应用密码
    expect(fields.length).toBeGreaterThanOrEqual(4)
    for (const input of fields) expectNoAutoText(input)
  })

  it('actual path dialogs disable auto-text while the initialized root remains a read-only backend fact', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const paths: PathsSnapshotV2 = { schemaVersion: 1,
      aggregateRoot: { path: '/fixtures/user root', status: 'missing' }, desktopRoot: { path: '/fixtures/user root/desktop', status: 'missing' },
      opencodexHome: { path: '/fixtures/explicit HOME', status: 'valid' }, configPath: { path: '/fixtures/explicit HOME/config.json', status: 'missing' },
      installationRoot: { path: '/fixtures/I', status: 'valid' }, managerExecutable: { path: '/fixtures/I/desktop', status: 'valid' },
      runtimePrefix: { path: '/fixtures/runtime', status: 'valid' }, runtimeEntry: { path: '/fixtures/runtime/bin/ocx', status: 'valid' },
      program: { packageVersion: '2.93.0', manifestVersion: '2.93.0', sourceVersion: '2.93.0', consistency: 'consistent' },
      currentAggregateRoot: '/fixtures/user root', pendingRestart: false,
    }
    invoke.mockImplementation(async command => {
      if (command === 'model_paths_v2_snapshot') return paths
      if (command === 'get_data_root_config') return { aggregateRoot: paths.aggregateRoot.path, activeDataRoot: paths.desktopRoot.path,
        currentDataRoot: paths.desktopRoot.path, opencodexHome: paths.opencodexHome.path, currentOpencodexHome: paths.opencodexHome.path,
        opencodexHomeMode: 'external', runtimeActive: true }
      if (command === 'runtime_protection_status') return { state: 'none', version: null, backupId: null }
      return null
    })
    useRouteStore().go('settings', { section: 'installation' })
    const wrapper = mount(SettingsRoute, { global: { plugins: [pinia] }, attachTo: document.body }); pages.push(wrapper)
    await flushPromises()
    const dialog = () => new DOMWrapper(document.body).get('[data-testid="paths-dialog"][open]')
    expect(wrapper.find('[data-testid="paths-select"]').exists()).toBe(false)
    expect(wrapper.find('#switch-root-input').exists()).toBe(false)
    expect(wrapper.find('#external-home-input').exists()).toBe(false)
    await wrapper.get('[data-testid="paths-initialize"]').trigger('click'); await flushPromises()
    expect(dialog().get('code').text()).toBe(paths.aggregateRoot.path)
    expect(dialog().find('input').exists()).toBe(false)
    expect(wrapper.find('#data-root-input').exists()).toBe(false)
    expect(invoke.mock.calls.filter(([name]) => ['initialize_data_root', 'switch_data_root', 'set_opencodex_home_config'].includes(name))).toEqual([])
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
    await flushPromises()
    const input = new DOMWrapper(document.body).find('.modal-field input')
    expect(input.exists()).toBe(true)
    expectNoAutoText(input)
    wrapper.unmount()
  })
})
