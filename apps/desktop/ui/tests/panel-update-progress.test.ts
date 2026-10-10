import { describe, expect, it, vi } from 'vitest'
import { installOfficialUpdate } from '../src/features/runtime/api'
const core = vi.hoisted(() => ({ invoke: vi.fn(), channel: null as any }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: core.invoke, Channel: class { onmessage: any; constructor() { core.channel = this } } }))
describe('panel update channel ownership', () => {
  it('rejects foreign, duplicate and late progress without fabricated percentages', async () => {
    let complete!: (v: any) => void
    core.invoke.mockImplementation(() => new Promise(resolve => { complete = resolve }))
    const observe = vi.fn()
    const install = installOfficialUpdate('2.51.0', observe)
    await vi.waitFor(() => expect(core.invoke).toHaveBeenCalled())
    const send = (extra = {}) => core.channel.onmessage({ operationId: 'owned', candidateVersion: '2.51.0', sequence: 1, phase: 'downloading', line: 'actual stage', ...extra })
    send({ candidateVersion: 'other' }); send({ sequence: 0 }); send({ phase: 'unknown' })
    expect(observe).not.toHaveBeenCalled()
    send(); send(); send({ operationId: 'foreign', sequence: 2 })
    expect(observe).toHaveBeenCalledTimes(1)
    expect(observe).toHaveBeenCalledWith({ phase: 'downloading', percent: 0, line: 'actual stage' })
    send({ phase: 'verifying', sequence: 2 })
    expect(observe).toHaveBeenCalledTimes(2)
    complete({ version: '2.51.0' }); await install
    send({ phase: 'done', sequence: 3 })
    expect(observe).toHaveBeenCalledTimes(2)
  })
})
