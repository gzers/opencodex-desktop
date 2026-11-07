import { installNativeDialogStub } from './helpers/nativeDialog'
import { createPinia, setActivePinia } from 'pinia'
import { DOMWrapper, flushPromises, mount } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useDataRootStore } from '@/features/data-root/store'
import { useRuntimeStore } from '@/features/runtime/store'
import { useRouteStore } from '@/stores/routes'
import { pathKeysV2, type DataRootConfig, type PathsSnapshotV2 } from '@/features/data-root/api'

const invoke = vi.fn()
const clipboard = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: async () => null }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
Object.defineProperty(window, 'matchMedia', { writable: true, value: (media: string) => ({ matches: false, media, addEventListener() {}, removeEventListener() {} }) })
Object.defineProperty(navigator, 'clipboard', { configurable: true, value: { writeText: clipboard } })
const config: DataRootConfig = { aggregateRoot: '/actual/U', activeDataRoot: '/actual/D', currentDataRoot: '/actual/D',
  opencodexHomeMode: 'external', opencodexHome: '/explicit/external HOME', currentOpencodexHome: '/explicit/external HOME', runtimeActive: true }
const snapshot = (): PathsSnapshotV2 => ({ schemaVersion: 1,
  aggregateRoot: { path: '/actual/U', status: 'valid' }, desktopRoot: { path: '/actual/D', status: 'valid' },
  opencodexHome: { path: '/explicit/external HOME', status: 'valid' }, configPath: { path: '/explicit/channel.json', status: 'missing' },
  installationRoot: { path: '/custom actual/I', status: 'valid' }, runtimePrefix: { path: '/independent runtime/prefix', status: 'valid' },
  runtimeEntry: { path: '/actual stable/ocx', status: 'valid' }, managerExecutable: { path: '/actual manager.app', status: 'valid' },
  program: { packageVersion: '23.1', manifestVersion: '23.1', sourceVersion: '23.1', consistency: 'consistent' },
  currentAggregateRoot: '/actual/U', pendingRestart: false,
})
const pages: Array<ReturnType<typeof mount>> = []
let paths: PathsSnapshotV2
function read(command: string) {
  if (command === 'model_paths_v2_snapshot') return structuredClone(paths)
  if (command === 'get_data_root_config') return { ...config }
  if (command === 'validate_data_root_structure') return 'valid'
  if (command === 'runtime_protection_status') return { state: 'none', version: null, backupId: null }
  return null
}
async function setup() {
  const pinia = createPinia(); setActivePinia(pinia)
  useRuntimeStore().protection = { state: 'none', version: null, backupId: null }
  useRouteStore().go('settings', { section: 'installation' })
  const page = mount(SettingsRoute, { global: { plugins: [pinia] }, attachTo: document.body }); pages.push(page)
  await vi.dynamicImportSettled()
  await flushPromises()
  return { page, store: useDataRootStore() }
}
function dialog() { return new DOMWrapper(document.body).get('[data-testid="paths-dialog"][open]') }
const writes = () => invoke.mock.calls.filter(([name]) => ['switch_data_root', 'initialize_data_root', 'set_opencodex_home_config', 'install_runtime'].includes(name))
async function choose(page: ReturnType<typeof mount>, target = '/chosen/new U') {
  await page.get('[data-testid="paths-select"]').trigger('click'); await flushPromises()
  await dialog().get('#switch-root-input').setValue(target)
  await dialog().get('[data-testid="paths-preview"]').trigger('click'); await flushPromises()
}
beforeEach(() => { installNativeDialogStub(); invoke.mockReset(); clipboard.mockReset(); clipboard.mockResolvedValue(undefined); paths = snapshot(); invoke.mockImplementation(async command => read(command)) })
afterEach(() => { pages.splice(0).forEach(page => page.unmount()); document.body.innerHTML = '' })

describe('settings paths v2 real Vue / Tauri contract', () => {
  it('uses all eight actual paths verbatim, separating U/D/H and actual custom I', async () => {
    const { page } = await setup()
    for (const key of pathKeysV2) expect(page.get('[data-path-key="' + key + '"] code').text()).toBe(paths[key].path)
    expect(invoke.mock.calls.filter(([c]) => c === 'model_paths_v2_snapshot')).toEqual([['model_paths_v2_snapshot']])
    expect(page.get('[data-path-key="configPath"] [data-path-status]').text()).toBe('缺失')
    expect(page.get('[data-path-key="opencodexHome"] [data-path-status]').text()).toBe('位置有效')
    expect(writes()).toEqual([])
  })
  it('copies full raw paths including Windows extended and long identities; view has only X exit', async () => {
    paths.runtimePrefix.path = '\\\\?\\D:\\' + 'directory with spaces\\'.repeat(40) + 'runtime'
    const { page } = await setup()
    await page.get('[data-testid="paths-details"]').trigger('click'); await flushPromises()
    expect(dialog().classes()).toContain('ui-dialog')
    expect(dialog().find('footer').exists()).toBe(false)
    expect(dialog().find('header button[aria-label="关闭"]').exists()).toBe(true)
    expect(dialog().text()).toContain(paths.runtimePrefix.path)
    await dialog().get('[data-testid="paths-copy"]').trigger('click'); await flushPromises()
    expect(clipboard).toHaveBeenCalledWith(pathKeysV2.slice(0, 4).concat(['installationRoot', 'managerExecutable', 'runtimePrefix', 'runtimeEntry']).map(key => paths[key].path).join('\n'))
    await dialog().get('header button').trigger('click'); await flushPromises()
    expect(new DOMWrapper(document.body).find('[data-testid="paths-dialog"]').exists()).toBe(false)
  })
  it('reports clipboard denial without changing path facts', async () => {
    clipboard.mockRejectedValue(new Error('denied'))
    const { page, store } = await setup()
    await page.get('[data-testid="paths-details"]').trigger('click'); await flushPromises()
    await dialog().get('[data-testid="paths-copy"]').trigger('click'); await flushPromises()
    expect(store.paths).toEqual(paths); expect(writes()).toEqual([])
  })
  it.each(['missing', 'mismatch', 'unreadable'] as const)('does not equate valid existing files with installed: %s', async consistency => {
    paths.program.consistency = consistency
    const { page } = await setup()
    expect(page.get('[data-testid="program-validity"]').text()).toMatch(consistency === 'missing' ? /尚未安装/ : /待核对/)
    expect(page.get('[data-path-key="runtimeEntry"] [data-path-status]').text()).toBe('位置有效')
  })
  it('shows each independent status without propagating missing to related paths', async () => {
    paths.aggregateRoot.status = 'unreadable'; paths.runtimePrefix.status = 'corrupted'; paths.runtimeEntry.status = 'missing'
    const { page } = await setup()
    for (const key of pathKeysV2) expect(page.get('[data-path-key="' + key + '"] [data-path-status]').attributes('data-path-status')).toBe(paths[key].status)
    expect(page.get('[data-testid="program-validity"]').text()).toContain('安装位置异常')
  })
  it('keeps the previous snapshot visible during refresh and after rejection, marking stale', async () => {
    const { page, store } = await setup()
    let reject!: (error: Error) => void
    invoke.mockImplementation(command => command === 'model_paths_v2_snapshot' ? new Promise((_, fail) => { reject = fail }) : Promise.resolve(read(command)))
    await page.get('[data-testid="paths-refresh"]').trigger('click'); await flushPromises()
    expect(page.text()).toContain('非最新'); expect(store.pathsLoading).toBe(true)
    expect(page.get('[data-path-key="runtimeEntry"] code').text()).toBe(paths.runtimeEntry.path)
    reject(new Error('read denied')); await flushPromises()
    expect(store.paths).toEqual(paths); expect(store.pathsStale).toBe(true)
    expect(page.text()).toContain('read denied'); expect(page.get('[data-testid="paths-select"]').attributes('disabled')).toBeDefined()
  })
  it.each([2, 999])('rejects future schema %s while retaining the prior snapshot', async schemaVersion => {
    const { page, store } = await setup()
    invoke.mockImplementation(async command => command === 'model_paths_v2_snapshot' ? { ...paths, schemaVersion } : read(command))
    await page.get('[data-testid="paths-refresh"]').trigger('click'); await flushPromises()
    expect(store.paths).toEqual(paths); expect(page.text()).toContain('结构版本暂不支持'); expect(page.text()).toContain('非最新')
  })
  it('does not fabricate missing facts on first read failure or malformed data', async () => {
    invoke.mockImplementation(async command => command === 'model_paths_v2_snapshot' ? { ...paths, runtimeEntry: { path: '/entry', status: 'future' } } : read(command))
    const { page, store } = await setup()
    expect(store.paths).toBeNull(); expect(page.find('[data-path-status]').exists()).toBe(false)
    expect(page.text()).toContain('尚无可用快照'); expect(page.get('[data-testid="program-validity"]').text()).toBe('尚未核验')
  })
  it('ignores both late success and late rejection from superseded repeated refreshes', async () => {
    const { store } = await setup()
    const requests: Array<{ resolve: (p: PathsSnapshotV2) => void; reject: (e: Error) => void }> = []
    invoke.mockImplementation(command => command === 'model_paths_v2_snapshot' ? new Promise((resolve, reject) => requests.push({ resolve, reject })) : Promise.resolve(read(command)))
    const first = store.loadPaths(), second = store.loadPaths(), third = store.loadPaths(); await flushPromises()
    const latest = snapshot(); latest.installationRoot.path = '/latest actual I'
    requests[2]!.resolve(latest); await third
    requests[0]!.resolve(snapshot()); await first
    requests[1]!.reject(new Error('late denied')); await second
    expect(store.paths).toEqual(latest); expect(store.pathsError).toBe(''); expect(store.pathsStale).toBe(false); expect(store.pathsLoading).toBe(false)
  })
  it.each(['future_version', 'corrupted', 'uninitialized'])('read-only target validation blocks %s without writes', async validation => {
    invoke.mockImplementation(async command => command === 'validate_data_root_structure' ? validation : read(command))
    const { page } = await setup(); await choose(page)
    expect(dialog().find('[data-testid="paths-confirm"]').exists()).toBe(false)
    expect(dialog().get('[role="alert"]').text()).toMatch(/版本过新|损坏|尚未初始化/)
    expect(invoke.mock.calls.filter(([c]) => c === 'validate_data_root_structure')).toEqual([['validate_data_root_structure', { rootPath: '/chosen/new U' }]])
    expect(writes()).toEqual([])
  })
  it('keeps validation transport failure read-only and allows cancellation', async () => {
    invoke.mockImplementation(async command => { if (command === 'validate_data_root_structure') throw new Error('denied'); return read(command) })
    const { page } = await setup(); await choose(page)
    expect(dialog().get('[role="alert"]').text()).toContain('校验失败')
    expect(writes()).toEqual([])
    await dialog().get('header button').trigger('click'); await flushPromises()
    expect(new DOMWrapper(document.body).find('[data-testid="paths-dialog"]').exists()).toBe(false)
  })
  it('refuses future initialization without writing and reports partial initialization uncertainty', async () => {
    const { store } = await setup()
    invoke.mockImplementation(async command => command === 'validate_data_root_structure' ? 'future_version' : read(command))
    expect(await store.save('/explicit future U')).toBe(false); expect(writes()).toEqual([])
    invoke.mockImplementation(async command => {
      if (command === 'initialize_data_root') throw new Error('partial structure')
      return read(command)
    })
    expect(await store.save('/explicit missing U')).toBe(false)
    expect(writes()).toEqual([['initialize_data_root', { request: { rootPath: '/explicit missing U' } }]])
    expect(store.error).toContain('重新读取实际位置核对')
    expect(store.config).toEqual(config)
  })
  it('late validation after X cancellation cannot reopen confirmation or write', async () => {
    let resolve!: (value: string) => void
    invoke.mockImplementation(command => command === 'validate_data_root_structure' ? new Promise(done => { resolve = done }) : Promise.resolve(read(command)))
    const { page } = await setup()
    await page.get('[data-testid="paths-select"]').trigger('click'); await flushPromises()
    await dialog().get('#switch-root-input').setValue('/late/U'); await dialog().get('[data-testid="paths-preview"]').trigger('click'); await flushPromises()
    await dialog().get('header button').trigger('click'); resolve('valid'); await flushPromises()
    expect(new DOMWrapper(document.body).find('[data-testid="paths-dialog"]').exists()).toBe(false); expect(writes()).toEqual([])
  })
  it('initializes only the explicit aggregate U after confirmation; cancellation creates nothing', async () => {
    paths.aggregateRoot.status = 'missing'; paths.desktopRoot.status = 'missing'
    const { page } = await setup()
    await page.get('[data-testid="paths-initialize"]').trigger('click'); await flushPromises()
    await dialog().get('header button').trigger('click'); await flushPromises(); expect(writes()).toEqual([])
    invoke.mockImplementation(async command => command === 'initialize_data_root' ? { created: true, structureVersion: '2' } : read(command))
    await page.get('[data-testid="paths-initialize"]').trigger('click'); await flushPromises()
    await dialog().findAll('footer button').find(b => b.text() === '确认初始化')!.trigger('click'); await flushPromises()
    expect(writes()).toEqual([['initialize_data_root', { request: { rootPath: '/actual/U' } }]])
  })
  it('delegates internal HOME without deriving a path and uses only an explicit external HOME', async () => {
    invoke.mockImplementation(async command => command === 'set_opencodex_home_config' ? { status: 'applied', config, reconciliationRequired: false } : read(command))
    const { page } = await setup()
    const openHome = () => page.findAll('button').find(b => b.text() === '设置 OpenCodex HOME')!.trigger('click')
    await openHome(); await flushPromises()
    await dialog().findAll('button').find(b => b.text() === '使用内部 HOME')!.trigger('click'); await flushPromises()
    expect(writes()[0]).toEqual(['set_opencodex_home_config', { request: { mode: 'inside', externalPath: undefined } }])
    await openHome(); await flushPromises(); await dialog().get('#external-home-input').setValue('/exact external HOME')
    for (const [name, value] of [['autocapitalize', 'off'], ['autocorrect', 'off'], ['spellcheck', 'false']]) expect(dialog().get('#external-home-input').attributes(name!)).toBe(value)
    await dialog().findAll('button').find(b => b.text() === '使用外部路径')!.trigger('click'); await flushPromises()
    expect(writes()[1]).toEqual(['set_opencodex_home_config', { request: { mode: 'external', externalPath: '/exact external HOME' } }])
  })
  it('reads HOME back after a lost save response and freezes later writes if the backend saved it', async () => {
    const { store } = await setup()
    let changed = false
    invoke.mockImplementation(async command => {
      if (command === 'set_opencodex_home_config') { changed = true; throw new Error('response lost') }
      if (changed && command === 'get_data_root_config') return { ...config, opencodexHome: '/saved explicit HOME', runtimeActive: false }
      return read(command)
    })
    expect(await store.saveHome('external', '/saved explicit HOME')).toBe(false)
    expect(store.pendingRestart).toBe(true)
    expect(store.config?.currentOpencodexHome).toBe(config.currentOpencodexHome)
    expect(await store.switchTo('/another U')).toBe(false)
    expect(writes()).toEqual([['set_opencodex_home_config', { request: { mode: 'external', externalPath: '/saved explicit HOME' } }]])
  })
  it('opens the existing runtime installation flow without inventing a recovery invoke', async () => {
    const { page } = await setup()
    await page.get('[data-testid="runtime-install"]').trigger('click'); await flushPromises()
    expect(useRuntimeStore().installModalRequest).toBe('registry')
    expect(writes()).toEqual([])
  })
})
