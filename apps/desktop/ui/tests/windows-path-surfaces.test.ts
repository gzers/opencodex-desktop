import { createPinia, setActivePinia } from 'pinia'
import { mount, flushPromises } from '@vue/test-utils'
import { afterEach, describe, expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useRouteStore } from '@/stores/routes'
import { useAboutStore } from '@/features/about/store'
import { useDataRootStore } from '@/features/data-root/store'
import { useRuntimeStore } from '@/features/runtime/store'
import { usePathsStore } from '@/app/paths/store'
import { useEnvironmentStore } from '@/features/environment/store'

const { invoke, copy } = vi.hoisted(() => ({ invoke: vi.fn(), copy: vi.fn(async () => {}) }))
vi.mock('@tauri-apps/api/core', () => ({ invoke }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: async () => null }))
const pages: Array<ReturnType<typeof mount>> = []
afterEach(() => pages.splice(0).forEach(page => page.unmount()))

describe('Windows path presentation keeps action/storage identities', () => {
  it('renders and copies readable data paths while opening the original backend value', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const root = '\\\\?\\D:\\OpenCodeX Desktop\\data'
    useAboutStore().app = { name: 'OpenCodeX Desktop', version: '0.2.18', platform: 'Windows',
      identifier: 'com.gzers.opencodex.desktop', framework: 'Tauri v2', license: 'MIT License' }
    const config = { currentDataRoot: root, activeDataRoot: root, opencodexHome: root + '\\opencodex-home',
      currentOpencodexHome: root + '\\opencodex-home', opencodexHomeMode: 'inside' as const, runtimeActive: true }
    useDataRootStore().config = config
    const paths = { managed: [{ key: 'backups' as const, label: '备份目录', path: root + '/backups' }], agent: [] }
    usePathsStore().managed = paths.managed
    useRuntimeStore().source = { kind: 'managed', path: root + '/runtime/bin/ocx.cmd', version: '2.93.0', resolvedAt: null,
      managedEntry: root + '/runtime/bin/ocx.cmd', managedPrefix: root + '/runtime/opencodex', defaultPrefix: root + '/runtime/opencodex',
      explicitPath: null, insideDataRoot: true, history: [] }
    const found = (path: string) => ({ found: true, path, version: null })
    useEnvironmentStore().report = { node: found('D:\\Tools Space/node.exe'), npm: found('D:\\Tools Space/npm.cmd'),
      ocx: found(root + '/runtime/bin/ocx.cmd'), gate: 'ready', shortCircuited: false, brewFound: false }
    invoke.mockImplementation(async command => {
      if (command === 'get_data_root_config') return config
      if (command === 'get_managed_path_targets') return paths
      if (command === 'runtime_source') return useRuntimeStore().source
      if (command === 'runtime_protection_status') return { state: 'none', version: null, backupId: null }
      if (command === 'open_managed_path') return { opened: true }
      return null
    })
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: copy } })
    useRouteStore().go('settings', { section: 'installation' })
    const page = mount(SettingsRoute, { global: { plugins: [pinia] } })
    pages.push(page)
    await flushPromises()
    expect(invoke).toHaveBeenCalledWith('get_managed_path_targets')
    expect(page.get('[data-testid="runtime-source-card"] .runtime-path').text()).toBe('D:\\OpenCodeX Desktop\\data\\runtime\\bin\\ocx.cmd')
    const row = page.findAll('.data-root-table tr').find(row => row.text().includes('数据目录 · 当前使用'))!
    await row.get('.cli-copy-btn').trigger('click')
    expect(copy).toHaveBeenLastCalledWith('D:\\OpenCodeX Desktop\\data')
    await row.findAll('button').at(-1)!.trigger('click')
    await flushPromises()
    expect(invoke).toHaveBeenCalledWith('open_managed_path', { path: root })
    expect(useDataRootStore().config?.currentDataRoot).toBe(root)
    expect(page.get('#data-root-input').attributes('placeholder')).toBe('D:\\OpenCodexData')
    expect(page.text()).toContain('D:\\Tools Space\\node.exe')
    expect(page.text()).not.toContain('\\\\?\\D:\\')
  })
})
