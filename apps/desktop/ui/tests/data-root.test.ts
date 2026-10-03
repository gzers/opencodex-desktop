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
    invoke.mockResolvedValueOnce({
      status: 'restart_required',
      blocked: null,
      config: {
        activeDataRoot: '/fixtures/target',
        opencodexHomeMode: 'inside',
        opencodexHome: '/fixtures/target/opencodex-home',
        runtimeActive: false,
      },
    })
    await expect(app.switchDataRoot('/fixtures/target', false)).resolves.toBe(true)
    expect(invoke).toHaveBeenCalledWith('switch_data_root', {
      request: { targetPath: '/fixtures/target', mode: 'reference_only' },
    })
    expect(app.dataRootConfig?.activeDataRoot).toBe('/fixtures/target')

    invoke.mockReset()
    invoke.mockResolvedValueOnce({ status: 'blocked', blocked: 'nested', config: null })
    await expect(app.switchDataRoot('/fixtures/nested', false)).resolves.toBe(false)
    expect(app.dataRootError).toContain('安全规则')
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
        runtimeActive: true,
      },
    })
    await expect(app.saveOpencodexHome('external', '/fixtures/external-home')).resolves.toBe(true)
    expect(invoke).toHaveBeenCalledWith('set_opencodex_home_config', {
      request: { mode: 'external', externalPath: '/fixtures/external-home' },
    })
    expect(app.dataRootConfig?.opencodexHomeMode).toBe('external')
  })
})
