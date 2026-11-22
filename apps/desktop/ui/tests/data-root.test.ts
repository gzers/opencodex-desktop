import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppStore } from '@/stores/app'
import { useDataRootStore } from '@/features/data-root/store'
import type { PathsSnapshotV2 } from '@/features/data-root/api'

const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

function pathsSnapshot(): PathsSnapshotV2 {
  return { schemaVersion: 1,
    aggregateRoot: { path: '/fixtures/U', status: 'valid' },
    desktopRoot: { path: '/fixtures/U/desktop', status: 'valid' },
    opencodexHome: { path: '/fixtures/explicit HOME', status: 'valid' },
    configPath: { path: '/fixtures/explicit HOME/config.json', status: 'missing' },
    installationRoot: { path: '/fixtures/I', status: 'valid' },
    runtimePrefix: { path: '/fixtures/program/runtime', status: 'valid' },
    runtimeEntry: { path: '/fixtures/program/bin/ocx', status: 'valid' },
    managerExecutable: { path: '/fixtures/I/desktop', status: 'valid' },
    program: { packageVersion: '2.93.0', manifestVersion: '2.93.0', sourceVersion: '2.93.0', consistency: 'consistent' },
    currentAggregateRoot: '/fixtures/U', pendingRestart: false,
  }
}

describe('data root integration', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
  })

  it('validates before initialization and stores created result', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'validate_data_root_structure') return 'valid'
      if (command === 'initialize_data_root') return { created: true, structureVersion: '2' }
      throw new Error(`unexpected command: ${command}`)
    })
    const app = useAppStore()
    await app.saveDataRoot('/fixtures/U')
    expect(invoke).toHaveBeenCalledWith('validate_data_root_structure', { rootPath: '/fixtures/U' })
    expect(app.dataRootLastValidation).toBe('valid')
    expect(app.dataRootPath).toBe('/fixtures/U')
    expect(app.dataRootError).toBe('')
  })

  it('stops before initialization for invalid structure', async () => {
    invoke.mockResolvedValue('corrupted')
    const app = useAppStore()
    const saved = await app.saveDataRoot('/fixtures/bad-root')
    expect(saved).toBe(false)
    expect(invoke).toHaveBeenCalledTimes(1)
    expect(app.dataRootLastValidation).toBe('corrupted')
    expect(app.dataRootError).toContain('已停止初始化')
    expect(app.dataRootPath).toBe('')
  })

  it('initializes an empty directory without treating it as damaged', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'validate_data_root_structure') return 'uninitialized'
      if (command === 'initialize_data_root') return { created: true, structureVersion: '2' }
      throw new Error(`unexpected command: ${command}`)
    })
    const app = useAppStore()
    await expect(app.saveDataRoot('/fixtures/empty-root')).resolves.toBe(true)
    expect(invoke).toHaveBeenCalledWith('initialize_data_root', {
      request: { rootPath: '/fixtures/empty-root' },
    })
    expect(app.dataRootLastValidation).toBe('valid')
    expect(app.dataRootLastResult?.created).toBe(true)
    expect(app.dataRootError).toBe('')
  })

  it('keeps future-version directories outside initialization', async () => {
    invoke.mockResolvedValue('future_version')
    const app = useAppStore()
    await expect(app.saveDataRoot('/fixtures/future-root')).resolves.toBe(false)
    expect(invoke).toHaveBeenCalledTimes(1)
    expect(app.dataRootLastValidation).toBe('future_version')
    expect(app.dataRootPath).toBe('')
  })

  it('reports initialization uncertainty without claiming a successful binding', async () => {
    invoke.mockRejectedValue(new Error('backend unavailable'))
    const app = useAppStore()
    const saved = await app.saveDataRoot('/fixtures/failing-root')
    expect(saved).toBe(false)
    expect(app.dataRootError).toContain('重新读取实际位置核对')
    expect(app.dataRootLastResult).toBeNull()
    expect(invoke).toHaveBeenCalledExactlyOnceWith('validate_data_root_structure', { rootPath: '/fixtures/failing-root' })
    expect(app.dataRootPath).toBe('')
    expect(app.dataRootSaving).toBe(false)
  })

  it('rejects blank explicit path before invoking backend', async () => {
    invoke.mockResolvedValue('valid')
    const app = useAppStore()
    const saved = await app.saveDataRoot('   ')
    expect(saved).toBe(false)
    expect(invoke).not.toHaveBeenCalled()
    expect(app.dataRootError).toContain('显式数据目录路径')
  })

  it('distinguishes unverified runtime from busy work without saving a binding', async () => {
    const app = useAppStore()
    invoke.mockResolvedValue({ status: 'blocked', blocked: 'runtime_unverified', config: null })
    await expect(app.switchDataRoot('/fixtures/target', false)).resolves.toBe(false)
    expect(app.dataRootError).toContain('未能确认面板运行状态')
    expect(app.dataRootError).not.toContain('仍在运行')
    await expect(app.saveOpencodexHome('external', '/fixtures/home')).resolves.toBe(false)
    expect(app.dataRootError).toContain('未能确认面板运行状态')
    expect(app.dataRootPendingRestart).toBe(false)
    expect(app.dataRootConfig).toBeNull()
  })

  it('loads and projects runtime data root config', async () => {
    const config = {
      aggregateRoot: '/fixtures/U', activeDataRoot: '/fixtures/U/desktop',
      opencodexHomeMode: 'inside',
      opencodexHome: '/fixtures/U/opencodex',
      currentDataRoot: '/fixtures/U/desktop',
      currentOpencodexHome: '/fixtures/U/opencodex',
      runtimeActive: true,
    }
    invoke.mockResolvedValueOnce(config)
    const app = useAppStore()
    await app.loadDataRootConfig()
    expect(invoke).toHaveBeenCalledWith('get_data_root_config')
    expect(app.dataRootConfig).toEqual(config)
    expect(app.dataRootConfigError).toBe(false)
    expect(app.dataRootConfigLoading).toBe(false)
  })

  it('switches data root by reference and preserves blocked state', async () => {
    const app = useAppStore()
    invoke.mockResolvedValueOnce({ status: 'blocked', blocked: 'nested', config: null })
    await expect(app.switchDataRoot('/fixtures/nested', false)).resolves.toBe(false)
    expect(app.dataRootError).toContain('安全规则')
    expect(app.dataRootSwitching).toBe(false)
    invoke.mockResolvedValueOnce({
      status: 'restart_required',
      blocked: null,
      config: {
        aggregateRoot: '/fixtures/target', activeDataRoot: '/fixtures/target/desktop',
        opencodexHomeMode: 'inside',
        opencodexHome: '/fixtures/target/opencodex',
        currentDataRoot: '/fixtures/U/desktop',
        currentOpencodexHome: '/fixtures/U/opencodex',
        runtimeActive: false,
      },
    })
    await expect(app.switchDataRoot('/fixtures/target', false)).resolves.toBe(true)
    expect(invoke).toHaveBeenCalledWith('switch_data_root', {
      request: { targetPath: '/fixtures/target', mode: 'reference_only' },
    })
    expect(app.dataRootConfig?.aggregateRoot).toBe('/fixtures/target')
    expect(app.dataRootConfig?.activeDataRoot).toBe('/fixtures/target/desktop')

    invoke.mockReset()
    await expect(app.switchDataRoot('/fixtures/nested', false)).resolves.toBe(false)
    expect(invoke).not.toHaveBeenCalled()
    expect(app.dataRootError).toContain('等待重启')
    expect(app.dataRootSwitching).toBe(false)
  })

  it('latches an uncertain reference switch across rereads and blocks HOME edits', async () => {
    const app = useAppStore()
    const old = { aggregateRoot: '/fixtures/U', activeDataRoot: '/fixtures/U/desktop', opencodexHomeMode: 'inside', opencodexHome: '/fixtures/U/opencodex', currentDataRoot: '/fixtures/U/desktop', currentOpencodexHome: '/fixtures/U/opencodex', runtimeActive: true }
    invoke.mockResolvedValueOnce({ status: 'restart_required', reconciliationRequired: true, blocked: null, config: { ...old, aggregateRoot: '/fixtures/target', activeDataRoot: '/fixtures/target/desktop', runtimeActive: false } })
    await expect(app.switchDataRoot('/fixtures/target', false)).resolves.toBe(true)
    expect(invoke).toHaveBeenCalledWith('switch_data_root', { request: { targetPath: '/fixtures/target', mode: 'reference_only' } })
    expect(app.dataRootReconciliationRequired).toBe(true)
    expect(app.dataRootPendingRestart).toBe(true)
    expect(app.dataRootConfig?.currentDataRoot).toBe('/fixtures/U/desktop')
    invoke.mockResolvedValueOnce(old)
    await app.loadDataRootConfig() // Old binding readback cannot release a freeze.
    invoke.mockReset()
    await expect(app.saveOpencodexHome('inside')).resolves.toBe(false)
    await expect(app.saveDataRoot('/fixtures/new')).resolves.toBe(false)
    expect(invoke).not.toHaveBeenCalled()
    expect(app.dataRootReconciliationRequired).toBe(true)
  })

  it('reads back an uncertain reference save before allowing further binding changes', async () => {
    const app = useAppStore()
    invoke.mockResolvedValueOnce({ aggregateRoot: '/fixtures/U', activeDataRoot: '/fixtures/U/desktop', opencodexHomeMode: 'inside', opencodexHome: '/fixtures/U/opencodex', currentDataRoot: '/fixtures/U/desktop', currentOpencodexHome: '/fixtures/U/opencodex', runtimeActive: true })
    await app.loadDataRootConfig()
    const old = app.dataRootConfig
    invoke.mockImplementation(async command => {
      if (command === 'switch_data_root') throw new Error('response lost')
      if (command === 'get_data_root_config') return old
      if (command === 'model_paths_v2_snapshot') return pathsSnapshot()
      throw new Error('unexpected command: ' + command)
    })
    await expect(app.switchDataRoot('/fixtures/target', false)).resolves.toBe(false)
    expect(app.dataRootError).toContain('引用切换结果未确认')
    expect(invoke.mock.calls.slice(-3).map(([command]) => command)).toEqual(['switch_data_root', 'get_data_root_config', 'model_paths_v2_snapshot'])
    expect(useDataRootStore().paths).toEqual(pathsSnapshot())
    expect(app.dataRootError).not.toContain('未修改目标')
    expect(app.dataRootConfig).toEqual(old)
    expect(app.dataRootPendingRestart).toBe(false)
    expect(app.dataRootSwitching).toBe(false)
  })

  it('does not send concurrent path commands while a reference switch is running', async () => {
    const app = useAppStore()
    let finish!: (value: unknown) => void
    invoke.mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    const switching = app.switchDataRoot('/fixtures/target', false)
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledTimes(1))
    await expect(app.saveOpencodexHome('external', '/fixtures/home')).resolves.toBe(false)
    await expect(app.switchDataRoot('/fixtures/other', false)).resolves.toBe(false)
    expect(invoke).toHaveBeenCalledTimes(1)
    finish({ status: 'blocked', blocked: 'running', config: null, reconciliationRequired: false })
    await expect(switching).resolves.toBe(false)
    expect(app.dataRootSwitching).toBe(false)
  })

  it('saves external OPENCODEX_HOME after validating blank input', async () => {
    const app = useAppStore()
    await expect(app.saveOpencodexHome('external', '   ')).resolves.toBe(false)
    expect(invoke).not.toHaveBeenCalled()

    invoke.mockResolvedValueOnce({
      status: 'restart_required',
      blocked: null,
      config: {
        aggregateRoot: '/fixtures/U', activeDataRoot: '/fixtures/U/desktop',
        opencodexHomeMode: 'external',
        opencodexHome: '/fixtures/external-home',
        currentDataRoot: '/fixtures/U/desktop',
        currentOpencodexHome: '/fixtures/U/opencodex',
        runtimeActive: false,
      },
    })
    await expect(app.saveOpencodexHome('external', '/fixtures/external-home')).resolves.toBe(true)
    expect(invoke).toHaveBeenCalledWith('set_opencodex_home_config', {
      request: { mode: 'external', externalPath: '/fixtures/external-home' },
    })
    expect(app.dataRootConfig?.opencodexHomeMode).toBe('external')
  })

  it('preserves the existing binding when HOME save is blocked by a running task', async () => {
    const app = useAppStore()
    const before = {
      aggregateRoot: '/fixtures/U', activeDataRoot: '/fixtures/U/desktop', opencodexHomeMode: 'inside' as const,
      opencodexHome: '/fixtures/U/opencodex', currentDataRoot: '/fixtures/U/desktop',
      currentOpencodexHome: '/fixtures/U/opencodex', runtimeActive: true,
    }
    invoke.mockResolvedValueOnce(before)
    await app.loadDataRootConfig()
    invoke.mockResolvedValueOnce({ status: 'blocked', blocked: 'running', config: null })
    await expect(app.saveOpencodexHome('external', '/fixtures/next-home')).resolves.toBe(false)
    expect(app.dataRootConfig).toEqual(before)
    expect(app.dataRootError).toContain('面板或安装任务仍在运行')
    expect(app.dataRootHomeSaving).toBe(false)
  })
  it('rejects retired migration before IPC instead of silently treating it as a reference switch', async () => {
    const app = useAppStore()
    await expect(app.switchDataRoot('/fixtures/target', true)).resolves.toBe(false)
    expect(app.dataRootError).toContain('不支持数据迁移')
    expect(invoke).not.toHaveBeenCalled()
    expect(app.dataRootConfig).toBeNull()
  })

  it('loads independent actual path facts without inferring a missing config from HOME', async () => {
    const paths = pathsSnapshot()
    invoke.mockResolvedValue(paths)
    const store = useDataRootStore()
    await store.loadPaths()
    expect(invoke).toHaveBeenCalledExactlyOnceWith('model_paths_v2_snapshot')
    expect(store.paths).toEqual(paths)
    expect(store.paths?.configPath.status).toBe('missing')
    expect(store.paths?.opencodexHome.status).toBe('valid')
    expect(store.pathsLoading).toBe(false)
    expect(store.pathsStale).toBe(false)
  })

  it('rejects malformed initial snapshot facts rather than fabricating normal paths', async () => {
    invoke.mockResolvedValue({ ...pathsSnapshot(), configPath: { path: '/unverified/config.json', status: 'unknown' } })
    const store = useDataRootStore()
    await store.loadPaths()
    expect(store.paths).toBeNull()
    expect(store.pathsError).toContain('未推断路径或文件状态')
    expect(store.pathsLoading).toBe(false)
    expect(invoke).toHaveBeenCalledExactlyOnceWith('model_paths_v2_snapshot')
  })

})
