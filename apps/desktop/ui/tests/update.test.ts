import { describe, expect, it, vi, beforeEach } from 'vitest'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

import { invoke } from '@tauri-apps/api/core'
const mockInvoke = vi.mocked(invoke)

import { checkForUpdate, getUpdateStatus, setUpdateChannel } from '@/features/updates/update'

describe('update commands', () => {
  beforeEach(() => {
    mockInvoke.mockReset()
  })

  it('status uses the frozen read-only command', async () => {
    mockInvoke.mockResolvedValue({ channel: 'stable', currentVersion: '0.1.0' })
    await expect(getUpdateStatus()).resolves.toEqual({ channel: 'stable', currentVersion: '0.1.0' })
    expect(mockInvoke).toHaveBeenCalledWith('get_update_status')
  })

  it('manual check invokes the real updater command and projects status', async () => {
    const update = { channel: 'stable', currentVersion: '0.1.0', lastCheckedAt: '2026-09-16T00:00:00Z' }
    mockInvoke.mockResolvedValue({ status: 'failed', update })
    await expect(checkForUpdate()).resolves.toEqual({ status: 'failed', update })
    expect(mockInvoke).toHaveBeenCalledWith('check_for_update')
  })

  it('channel switching maps the selected stable or beta channel', async () => {
    mockInvoke.mockResolvedValue({ channel: 'beta', currentVersion: '0.1.0' })
    await expect(setUpdateChannel('beta')).resolves.toEqual({ channel: 'beta', currentVersion: '0.1.0' })
    expect(mockInvoke).toHaveBeenCalledWith('set_update_channel', { channel: 'beta' })
  })
})

describe('app update store contract', () => {
  it('requires available metadata before installation and preserves current version on failure', async () => {
    const { useAppStore } = await import('@/stores/app')
    const installMock = vi.fn()
    vi.doMock('@/features/updates/update', async importOriginal => ({
      ...(await importOriginal<typeof import('@/features/updates/update')>()),
      installUpdate: (...args: unknown[]) => installMock(...args),
    }))
    const { createPinia, setActivePinia } = await import('pinia')
    setActivePinia(createPinia())
    const app = useAppStore()
    installMock.mockRejectedValueOnce(new Error('download failed'))
    await expect(app.installAppUpdate()).resolves.toBe(false)
    expect(installMock).toHaveBeenCalledTimes(1)
    expect(app.appUpdateError).toBe('更新安装失败；已保留当前版本。')
    expect(app.appUpdateBusy).toBe(false)
    installMock.mockResolvedValueOnce(undefined)
    await expect(app.installAppUpdate()).resolves.toBe(true)
    expect(app.appUpdateBusy).toBe(false)
  })
})
