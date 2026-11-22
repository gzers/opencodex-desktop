import { installNativeDialogStub } from './helpers/nativeDialog'
import { createPinia, setActivePinia } from 'pinia'
import { DOMWrapper, mount, flushPromises } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useRouteStore } from '@/stores/routes'
import { useAboutStore } from '@/features/about/store'
import { useDataRootStore } from '@/features/data-root/store'
import { useRuntimeStore } from '@/features/runtime/store'
import { usePathsStore } from '@/app/paths/store'
import type { PathsSnapshotV2 } from '@/features/data-root/api'

const { invoke, copy } = vi.hoisted(() => ({ invoke: vi.fn(), copy: vi.fn(async () => {}) }))
vi.mock('@tauri-apps/api/core', () => ({ invoke }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: async () => null }))
const pages: Array<ReturnType<typeof mount>> = []
beforeEach(() => { installNativeDialogStub(); invoke.mockReset(); copy.mockClear() })
afterEach(() => { pages.splice(0).forEach(page => page.unmount()); document.body.innerHTML = '' })

describe('Windows path presentation keeps action/storage identities', () => {
  it('keeps v2 path snapshots verbatim and opens a readable path through its original backend identity', async () => {
    const pinia = createPinia(); setActivePinia(pinia)
    const root = '\\\\?\\D:\\OpenCodeX User Data'
    const installation = '\\\\?\\C:\\Program Files\\OpenCodeX Desktop'
    const home = '\\\\?\\E:\\Explicit HOME'
    const paths: PathsSnapshotV2 = { schemaVersion: 1,
      aggregateRoot: { path: root, status: 'valid' }, desktopRoot: { path: root + '\\desktop', status: 'valid' },
      opencodexHome: { path: home, status: 'valid' }, configPath: { path: home + '\\config.json', status: 'missing' },
      installationRoot: { path: installation, status: 'valid' }, managerExecutable: { path: installation + '\\ocxd.exe', status: 'valid' },
      runtimePrefix: { path: '\\\\?\\C:\\ProgramData\\OpenCodeX\\runtime\\opencodex', status: 'valid' },
      runtimeEntry: { path: '\\\\?\\C:\\ProgramData\\OpenCodeX\\runtime\\bin\\ocx.cmd', status: 'valid' },
      program: { packageVersion: '2.93.0', manifestVersion: '2.93.0', sourceVersion: '2.93.0', consistency: 'consistent' },
      currentAggregateRoot: root, pendingRestart: false,
    }
    const config = { aggregateRoot: root, currentDataRoot: paths.desktopRoot.path, activeDataRoot: paths.desktopRoot.path,
      opencodexHome: home, currentOpencodexHome: home, opencodexHomeMode: 'external' as const, runtimeActive: true }
    useAboutStore().app = { name: 'OpenCodeX Desktop', version: '0.2.23', platform: 'Windows',
      identifier: 'com.gzers.opencodex.desktop', framework: 'Tauri v2', license: 'MIT License' }
    useRuntimeStore().source = { kind: 'managed', path: paths.runtimeEntry.path, version: '2.93.0', resolvedAt: null,
      managedEntry: paths.runtimeEntry.path, managedPrefix: paths.runtimePrefix.path, defaultPrefix: paths.runtimePrefix.path,
      explicitPath: null, insideDataRoot: false, history: [] }
    const skills = root + '\\Skills Source'
    const targets = { managed: [], agent: [{ kind: 'skills-source' as const, client: 'shared', label: 'Skills 源目录', path: skills }] }
    invoke.mockImplementation(async command => {
      if (command === 'get_data_root_config') return config
      if (command === 'model_paths_v2_snapshot') return structuredClone(paths)
      if (command === 'get_managed_path_targets') return targets
      if (command === 'runtime_protection_status') return { state: 'none', version: null, backupId: null }
      if (command === 'open_managed_path') return { opened: true }
      return null
    })
    Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: copy } })
    useRouteStore().go('settings', { section: 'installation' })
    const page = mount(SettingsRoute, { global: { plugins: [pinia] }, attachTo: document.body }); pages.push(page)
    await flushPromises()
    expect(invoke).toHaveBeenCalledWith('model_paths_v2_snapshot')
    expect(invoke).not.toHaveBeenCalledWith('get_managed_path_targets')
    expect(page.get('[data-path-key="desktopRoot"] code').text()).toBe(paths.desktopRoot.path)
    expect(page.get('[data-path-key="opencodexHome"] code').text()).toBe(home)
    expect(page.get('[data-path-key="installationRoot"] code').text()).toBe(installation)
    expect(page.get('[data-path-key="configPath"] [data-path-status]').text()).toBe('缺失')
    await page.get('[data-testid="paths-details"]').trigger('click'); await flushPromises()
    const dialog = new DOMWrapper(document.body).get('[data-testid="paths-dialog"][open]')
    await dialog.get('[data-testid="paths-copy"]').trigger('click'); await flushPromises()
    const keys = ['aggregateRoot', 'desktopRoot', 'opencodexHome', 'configPath', 'installationRoot', 'managerExecutable', 'runtimePrefix', 'runtimeEntry'] as const
    expect(copy).toHaveBeenLastCalledWith(keys.map(key => paths[key].path).join('\n'))
    await dialog.get('header button[aria-label="关闭"]').trigger('click'); await flushPromises()
    expect(useDataRootStore().paths).toEqual(paths)
    expect(useDataRootStore().config).toEqual(config)

    // Opening belongs to this actual action consumer, not the v2 read-only snapshot.
    useRouteStore().go('settings', { section: 'extensions' }); await flushPromises()
    expect(invoke).toHaveBeenCalledWith('get_managed_path_targets')
    expect(page.get('.cli-copy-btn.skills-source code').text()).toBe('D:\\OpenCodeX User Data\\Skills Source')
    await page.get('.cli-copy-btn.skills-source').trigger('click'); await flushPromises()
    expect(copy).toHaveBeenLastCalledWith('D:\\OpenCodeX User Data\\Skills Source')
    const row = page.findAll('.setting-row').find(row => row.find('.setting-title').text() === 'Skills 源目录')!
    await row.findAll('button').find(button => button.text() === '打开')!.trigger('click'); await flushPromises()
    expect(invoke).toHaveBeenCalledWith('open_managed_path', { path: skills })
    expect(usePathsStore().agent?.[0]?.path).toBe(skills)
    expect(useDataRootStore().paths?.aggregateRoot.path).toBe(root)
  })
})
