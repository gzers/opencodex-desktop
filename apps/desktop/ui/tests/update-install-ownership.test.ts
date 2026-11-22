import { beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises } from '@vue/test-utils'
const invoke = vi.fn()
class MockChannel { onmessage = (_value: unknown) => {} }
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => invoke(...args), Channel: MockChannel }))
beforeEach(() => { vi.resetModules(); invoke.mockReset() })
const options = { candidateVersion: '0.1.10', channel: 'stable' as const, backup: true }
const snapshot = { ...options, currentVersion: '0.1.9', availableVersion: '0.1.10', error: null }
const message = { operationId: 'op1', target: 'manager', candidateVersion: '0.1.10', channel: 'stable',
  generation: 1, sequence: 1, stage: 'downloading', downloadedBytes: 10, totalBytes: 100 }
describe('confirmed update operation ownership', () => {
  it('reuses the same confirmation but refuses a different candidate or protection choice', async () => {
    const api = await import('@/features/updates/update')
    let finish!: () => void
    invoke.mockImplementation((command: string) => command === 'install_update'
      ? new Promise<void>(resolve => { finish = resolve }) : Promise.resolve(snapshot))
    const first = api.installUpdate(options)
    expect(api.installUpdate({ ...options })).toBe(first)
    await expect(api.installUpdate({ ...options, candidateVersion: '0.1.11' })).rejects.toThrow('active')
    await expect(api.installUpdate({ ...options, backup: false })).rejects.toThrow('active')
    expect(invoke.mock.calls.filter(call => call[0] === 'install_update')).toHaveLength(1)
    finish(); await first
    expect(invoke.mock.calls.some(call => call[0] === 'restart_after_update')).toBe(false)
  })
  it('rejects wrong channel/candidate, stale sequence, different operation, and late completion messages', async () => {
    const api = await import('@/features/updates/update')
    let finish!: () => void
    invoke.mockImplementation((command: string) => command === 'install_update'
      ? new Promise<void>(resolve => { finish = resolve }) : Promise.resolve(snapshot))
    const request = api.installUpdate(options); await flushPromises()
    const observer = invoke.mock.calls.find(call => call[0] === 'install_update')![1].progress as MockChannel
    observer.onmessage({ ...message, channel: 'beta' }); expect(api.managerInstallProgress.value).toBeNull()
    observer.onmessage({ ...message, candidateVersion: '0.1.11' }); expect(api.managerInstallProgress.value).toBeNull()
    observer.onmessage(message); expect(api.managerInstallProgress.value?.sequence).toBe(1)
    observer.onmessage({ ...message, sequence: 2, operationId: 'op2' })
    observer.onmessage({ ...message, sequence: 0 })
    observer.onmessage({ ...message, sequence: 3, generation: 2 })
    expect(api.managerInstallProgress.value?.sequence).toBe(1)
    observer.onmessage({ ...message, sequence: 2, downloadedBytes: 20 })
    expect(api.managerInstallProgress.value?.downloadedBytes).toBe(20)
    finish(); await request
    observer.onmessage({ ...message, sequence: 100, stage: 'failed' })
    expect(api.managerInstallProgress.value?.sequence).toBe(2)
  })
  it('rejects invented totals and retries after failure without restarting automatically', async () => {
    const api = await import('@/features/updates/update')
    invoke.mockImplementation((command: string, args: { progress: MockChannel }) => {
      if (command === 'install_update') {
        args.progress.onmessage({ ...message, totalBytes: 0 })
        args.progress.onmessage({ ...message, totalBytes: 5 })
        return Promise.reject(new Error('signature'))
      }
      return Promise.resolve(snapshot)
    })
    await expect(api.installUpdate(options)).rejects.toThrow('signature')
    expect(api.managerInstallProgress.value).toBeNull()
    invoke.mockResolvedValue(undefined)
    await api.installUpdate(options)
    expect(invoke.mock.calls.filter(call => call[0] === 'install_update')).toHaveLength(2)
    expect(invoke.mock.calls.some(call => call[0] === 'restart_after_update')).toBe(false)
  })
})
