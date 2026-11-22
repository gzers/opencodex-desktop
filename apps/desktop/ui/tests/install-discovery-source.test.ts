import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { afterEach, describe, expect, it, vi } from 'vitest'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useRuntimeStore } from '@/features/runtime/store'
import { useEnvironmentStore } from '@/features/environment/store'
import { useRouteStore } from '@/stores/routes'
import type { PathsSnapshotV2 } from '@/features/data-root/api'

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: async () => null }))
const pages: Array<ReturnType<typeof mount>> = []
afterEach(() => { pages.splice(0).forEach(page => page.unmount()); invoke.mockReset() })
function actualPaths(): PathsSnapshotV2 {
  return { schemaVersion: 1,
    aggregateRoot: { path: '/actual/U', status: 'valid' }, desktopRoot: { path: '/actual/U/desktop', status: 'valid' },
    opencodexHome: { path: '/external/H', status: 'valid' }, configPath: { path: '/external/H/config.json', status: 'missing' },
    installationRoot: { path: '/custom/program I', status: 'valid' }, managerExecutable: { path: '/custom/program I/desktop', status: 'valid' },
    runtimePrefix: { path: '/independent/runtime/prefix', status: 'valid' }, runtimeEntry: { path: '/independent/runtime/bin/ocx', status: 'valid' },
    program: { packageVersion: '2.93.0', manifestVersion: '2.93.0', sourceVersion: '2.93.0', consistency: 'consistent' },
    currentAggregateRoot: '/actual/U', pendingRestart: false,
  }
}
async function setup(paths: PathsSnapshotV2) {
  const pinia = createPinia(); setActivePinia(pinia)
  const runtime = useRuntimeStore()
  runtime.source = { kind: 'managed', path: paths.runtimeEntry.path, version: '2.94.0', resolvedAt: null,
    managedEntry: paths.runtimeEntry.path, managedPrefix: paths.runtimePrefix.path, defaultPrefix: '/old/default/prefix',
    explicitPath: null, insideDataRoot: false, history: [] }
  useEnvironmentStore().report = {
    node: { found: true, path: '/tools/node', version: '24' }, npm: { found: true, path: '/tools/npm', version: '11' },
    ocx: { found: false, path: null, version: '1.0-stale-discovery' }, gate: 'missing_ocx', shortCircuited: false, brewFound: false,
  }
  invoke.mockImplementation(async command => {
    if (command === 'model_paths_v2_snapshot') return structuredClone(paths)
    if (command === 'get_data_root_config') return { aggregateRoot: '/actual/U', currentDataRoot: '/actual/U/desktop', activeDataRoot: '/actual/U/desktop',
      opencodexHomeMode: 'external', opencodexHome: '/external/H', currentOpencodexHome: '/external/H', runtimeActive: true }
    if (command === 'runtime_protection_status') return { state: 'none', version: null, backupId: null }
    return null
  })
  useRouteStore().go('settings', { section: 'installation' })
  const page = mount(SettingsRoute, { global: { plugins: [pinia] } }); pages.push(page)
  await flushPromises()
  return page
}

describe('安装发现采用真实 v2 程序位置和三方证据', () => {
  it('系统 PATH 未发现 ocx 时仍展示已核验的托管程序及非默认安装位置', async () => {
    const paths = actualPaths()
    const page = await setup(paths)
    expect(invoke).toHaveBeenCalledWith('model_paths_v2_snapshot')
    expect(page.get('[data-path-key="installationRoot"] code').text()).toBe('/custom/program I')
    expect(page.get('[data-path-key="runtimePrefix"] code').text()).toBe(paths.runtimePrefix.path)
    expect(page.get('[data-path-key="runtimeEntry"] code').text()).toBe(paths.runtimeEntry.path)
    expect(page.get('[data-testid="program-validity"]').text()).toBe('安装证据一致；运行状态另行核验')
    expect(page.find('[data-testid="runtime-source-card"]').exists()).toBe(true)
    expect(page.get('[data-testid="settings-program-installation"]').text()).not.toContain('/old/default/prefix')
  })

  it('分别显示包、清单、来源版本，不用旧发现版本或托管标签掩盖不一致', async () => {
    const paths = actualPaths()
    paths.program.sourceVersion = '2.94.0'; paths.program.consistency = 'mismatch'
    const page = await setup(paths)
    expect(page.get('[data-testid="settings-program-installation"]').text()).toContain('2.93.0 / 2.93.0 / 2.94.0')
    expect(page.get('[data-testid="settings-program-installation"]').text()).not.toContain('1.0-stale-discovery')
    expect(page.get('[data-testid="program-validity"]').text()).toContain('不一致，待核对')
  })

  it.each(['missing', 'unreadable'] as const)('缓存托管来源不能把 %s 程序证据显示为正常', async consistency => {
    const paths = actualPaths()
    paths.program = { packageVersion: null, manifestVersion: null, sourceVersion: null, consistency }
    paths.runtimeEntry.status = consistency
    const page = await setup(paths)
    expect(page.get('[data-path-key="runtimeEntry"] [data-path-status]').attributes('data-path-status')).toBe(consistency)
    expect(page.get('[data-testid="program-validity"]').text()).toMatch(consistency === 'missing' ? /尚未安装/ : /无法读取，待核对/)
    expect(page.get('[data-testid="program-validity"]').text()).not.toContain('安装证据一致')
    expect(page.get('[data-testid="settings-program-installation"]').text()).toContain('未记录 / 未记录 / 未记录')
  })
})
