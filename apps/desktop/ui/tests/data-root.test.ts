import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useAppStore } from '@/stores/app'

const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

describe('data root integration', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
  })

  it('validates before initialization and stores created result', async () => {
    invoke.mockImplementation(async (command: string) => {
      if (command === 'validate_data_root_structure') return 'valid'
      if (command === 'initialize_data_root') return { created: true, structureVersion: '1' }
      throw new Error(`unexpected command: ${command}`)
    })
    const app = useAppStore()
    await app.saveDataRoot('/fixtures/data-root')
    expect(invoke).toHaveBeenCalledWith('validate_data_root_structure', { rootPath: '/fixtures/data-root' })
    expect(app.dataRootLastValidation).toBe('valid')
    expect(app.dataRootPath).toBe('/fixtures/data-root')
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

  it('reports failure without modifying target', async () => {
    invoke.mockRejectedValue(new Error('backend unavailable'))
    const app = useAppStore()
    const saved = await app.saveDataRoot('/fixtures/failing-root')
    expect(saved).toBe(false)
    expect(app.dataRootError).toContain('未修改目标目录')
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

  it('loads and projects runtime data root config', async () => {
    const config = {
      activeDataRoot: '/fixtures/data-root',
      opencodexHomeMode: 'inside',
      opencodexHome: '/fixtures/data-root/opencodex-home',
      currentDataRoot: '/fixtures/data-root',
      currentOpencodexHome: '/fixtures/data-root/opencodex-home',
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
        activeDataRoot: '/fixtures/target',
        opencodexHomeMode: 'inside',
        opencodexHome: '/fixtures/target/opencodex-home',
        currentDataRoot: '/fixtures/data-root',
        currentOpencodexHome: '/fixtures/data-root/opencodex-home',
        runtimeActive: false,
      },
    })
    await expect(app.switchDataRoot('/fixtures/target', false)).resolves.toBe(true)
    expect(invoke).toHaveBeenCalledWith('switch_data_root', {
      request: { targetPath: '/fixtures/target', mode: 'reference_only' },
    })
    expect(app.dataRootConfig?.activeDataRoot).toBe('/fixtures/target')

    invoke.mockReset()
    await expect(app.switchDataRoot('/fixtures/nested', false)).resolves.toBe(false)
    expect(invoke).not.toHaveBeenCalled()
    expect(app.dataRootError).toContain('等待重启')
    expect(app.dataRootSwitching).toBe(false)
  })

  it('latches an uncertain migration result across rereads and blocks HOME edits', async () => {
    const app = useAppStore()
    const old = { activeDataRoot: '/fixtures/source', opencodexHomeMode: 'inside', opencodexHome: '/fixtures/source/opencodex-home', currentDataRoot: '/fixtures/source', currentOpencodexHome: '/fixtures/source/opencodex-home', runtimeActive: true }
    invoke.mockResolvedValueOnce({ status: 'restart_required', reconciliationRequired: true, blocked: null, config: { ...old, activeDataRoot: '/fixtures/target', runtimeActive: false } })
    await expect(app.switchDataRoot('/fixtures/target', true)).resolves.toBe(true)
    expect(invoke).toHaveBeenCalledWith('switch_data_root', { request: { targetPath: '/fixtures/target', mode: 'migrate_data' } })
    expect(app.dataRootReconciliationRequired).toBe(true)
    expect(app.dataRootPendingRestart).toBe(true)
    expect(app.dataRootConfig?.currentDataRoot).toBe('/fixtures/source')
    invoke.mockResolvedValueOnce(old)
    await app.loadDataRootConfig() // Old binding readback cannot release a freeze.
    invoke.mockReset()
    await expect(app.saveOpencodexHome('inside')).resolves.toBe(false)
    await expect(app.saveDataRoot('/fixtures/new')).resolves.toBe(false)
    expect(invoke).not.toHaveBeenCalled()
    expect(app.dataRootReconciliationRequired).toBe(true)
  })

  it('reports a partial-copy failure without changing the active config', async () => {
    const app = useAppStore()
    invoke.mockResolvedValueOnce({ activeDataRoot: '/fixtures/source', opencodexHomeMode: 'inside', opencodexHome: '/fixtures/source/opencodex-home', currentDataRoot: '/fixtures/source', currentOpencodexHome: '/fixtures/source/opencodex-home', runtimeActive: true })
    await app.loadDataRootConfig()
    const old = app.dataRootConfig
    invoke.mockRejectedValueOnce(new Error('copy failed'))
    await expect(app.switchDataRoot('/fixtures/target', true)).resolves.toBe(false)
    expect(app.dataRootError).toContain('未完成副本')
    expect(app.dataRootError).not.toContain('未修改目标')
    expect(app.dataRootConfig).toEqual(old)
    expect(app.dataRootPendingRestart).toBe(false)
    expect(app.dataRootSwitching).toBe(false)
  })

  it('does not send concurrent path commands while migration is running', async () => {
    const app = useAppStore()
    let finish!: (value: unknown) => void
    invoke.mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    const migration = app.switchDataRoot('/fixtures/target', true)
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledTimes(1))
    await expect(app.saveOpencodexHome('external', '/fixtures/home')).resolves.toBe(false)
    await expect(app.switchDataRoot('/fixtures/other', false)).resolves.toBe(false)
    expect(invoke).toHaveBeenCalledTimes(1)
    finish({ status: 'blocked', blocked: 'running', config: null, reconciliationRequired: false })
    await expect(migration).resolves.toBe(false)
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
        activeDataRoot: '/fixtures/data-root',
        opencodexHomeMode: 'external',
        opencodexHome: '/fixtures/external-home',
        currentDataRoot: '/fixtures/data-root',
        currentOpencodexHome: '/fixtures/data-root/opencodex-home',
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
      activeDataRoot: '/fixtures/data-root', opencodexHomeMode: 'inside' as const,
      opencodexHome: '/fixtures/data-root/opencodex-home', currentDataRoot: '/fixtures/data-root',
      currentOpencodexHome: '/fixtures/data-root/opencodex-home', runtimeActive: true,
    }
    invoke.mockResolvedValueOnce(before)
    await app.loadDataRootConfig()
    invoke.mockResolvedValueOnce({ status: 'blocked', blocked: 'running', config: null })
    await expect(app.saveOpencodexHome('external', '/fixtures/next-home')).resolves.toBe(false)
    expect(app.dataRootConfig).toEqual(before)
    expect(app.dataRootError).toContain('面板或安装任务仍在运行')
    expect(app.dataRootHomeSaving).toBe(false)
  })
})
