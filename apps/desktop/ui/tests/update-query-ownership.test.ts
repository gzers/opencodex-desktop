import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises } from '@vue/test-utils'
const invoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args) }))
beforeEach(() => { vi.resetModules(); invoke.mockReset() })
const status = (channel = 'stable') => ({ channel, currentVersion: '0.1.9', availableVersion: '0.1.10',
  lastCheckedAt: null, signatureVerified: null, error: null })
describe('shared update query ownership', () => {
  it('manual and automatic callers reuse one manager request, then may retry', async () => {
    const api = await import('@/features/updates/update')
    let finish!: (value: unknown) => void
    invoke.mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    const first = api.checkForUpdate()
    const all = Array.from({ length: 100 }, () => api.checkForUpdate())
    expect(all.every(request => request === first)).toBe(true)
    expect(invoke).toHaveBeenCalledOnce()
    finish({ status: 'available', update: status() }); await first
    invoke.mockRejectedValueOnce(new Error('offline'))
    await expect(api.checkForUpdate()).rejects.toThrow('offline')
    invoke.mockResolvedValueOnce({ status: 'up_to_date', update: status() })
    await api.checkForUpdate(); expect(invoke).toHaveBeenCalledTimes(3)
  })
  it('late stable completion cannot replace beta or release the beta request', async () => {
    const api = await import('@/features/updates/update')
    let finishOld!: (value: unknown) => void, finishNew!: (value: unknown) => void
    invoke.mockImplementationOnce(() => new Promise(resolve => { finishOld = resolve }))
    const old = api.checkForUpdate()
    invoke.mockResolvedValueOnce(status('beta')); await api.setUpdateChannel('beta')
    invoke.mockImplementationOnce(() => new Promise(resolve => { finishNew = resolve }))
    const fresh = api.checkForUpdate()
    finishOld({ status: 'available', update: status() }); await old
    expect(api.managerUpdateStatus.value?.channel).toBe('beta')
    expect(api.checkForUpdate()).toBe(fresh)
    finishNew({ status: 'available', update: status('beta') }); await fresh
  })
  it('preference readback also invalidates the old channel; failed switch does not', async () => {
    const api = await import('@/features/updates/update')
    let finish!: (value: unknown) => void
    invoke.mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    const old = api.checkForUpdate()
    invoke.mockRejectedValueOnce(new Error('installing'))
    await expect(api.setUpdateChannel('beta')).rejects.toThrow()
    expect(api.checkForUpdate()).toBe(old)
    invoke.mockResolvedValueOnce(status('beta')); await api.getUpdateStatus()
    finish({ status: 'available', update: status() }); await old
    expect(api.managerUpdateStatus.value?.channel).toBe('beta')
  })
  it('panel callers share a separate request, unaffected by manager channel changes', async () => {
    const panel = await import('@/features/about/api')
    const api = await import('@/features/updates/update')
    let finish!: (value: unknown) => void
    invoke.mockImplementationOnce(() => new Promise(resolve => { finish = resolve }))
    const first = panel.getOfficialRemoteLatest(); await flushPromises()
    expect(panel.getOfficialRemoteLatest()).toBe(first)
    invoke.mockResolvedValueOnce(status('beta')); await api.setUpdateChannel('beta')
    expect(panel.getOfficialRemoteLatest()).toBe(first)
    finish({ tag: 'latest', version: '2.51.0', integrity: null }); await first
    expect(invoke.mock.calls.filter(call => call[0] === 'official_remote_latest')).toHaveLength(1)
  })
})
