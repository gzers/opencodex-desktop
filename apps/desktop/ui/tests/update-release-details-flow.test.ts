import { createPinia, setActivePinia } from 'pinia'
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import UpdateCenter from '@/features/updates/UpdateCenter.vue'
import { managerUpdateStatus } from '@/features/updates/update'
import { useUpdatesStore } from '@/features/updates/store'
const mocks = vi.hoisted(() => ({ invoke: vi.fn(), install: vi.fn(), check: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: (...args: unknown[]) => mocks.invoke(...args) }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: async () => null }))
vi.mock('@/features/updates/update', async original => ({ ...await original<typeof import('@/features/updates/update')>(),
  installUpdate: (...args: unknown[]) => mocks.install(...args), checkForUpdate: () => mocks.check(),
}))
let wrapper: VueWrapper | null = null
let pinia: ReturnType<typeof createPinia>
beforeEach(() => {
  vi.clearAllMocks(); pinia = createPinia(); setActivePinia(pinia)
  managerUpdateStatus.value = { channel: 'beta', currentVersion: '0.1.9', availableVersion: '0.1.10-beta.1', lastCheckedAt: '2026-10-14', signatureVerified: false, error: null, releaseUrl: 'https://github.com/gzers/opencodex-desktop/releases/tag/v0.1.10-beta.1' }
  useUpdatesStore().officialRemote = { tag: 'latest', version: '2.84.0', integrity: null }
})
afterEach(() => { wrapper?.unmount(); wrapper = null; vi.restoreAllMocks() })
function render(target: 'official' | 'manager') {
  wrapper = mount(UpdateCenter, { props: { open: true, target }, global: { plugins: [pinia], stubs: { teleport: true } } });return wrapper
}
describe('release detail integration through the shared update center', () => {
  it('loads the exact panel release body without running a version check, backup or install', async () => {
    mocks.invoke.mockImplementation(async (command, args) => command === 'get_update_status' ? managerUpdateStatus.value
      : command === 'update_release_details' ? { target: args.target, channel: args.channel, candidateVersion: args.candidateVersion, notesState: 'ready', notes: 'official 2.84.0 release body', notesSource: 'github_release', releaseUrl: 'https://github.com/lidge-jun/opencodex/releases/tag/v2.84.0', fetchedAt: 100, publishedAt: null, cached: false } : null)
    const w = render('official'); await flushPromises()
    expect(mocks.invoke.mock.calls).toContainEqual(['update_release_details', { target: 'official', channel: 'official', candidateVersion: '2.84.0', force: false }])
    expect(w.text()).toContain('official 2.84.0 release body')
    expect(mocks.check).not.toHaveBeenCalled();expect(mocks.install).not.toHaveBeenCalled()
    expect(mocks.invoke.mock.calls.some(call => call[0] === 'create_upgrade_backup' || call[0] === 'official_remote_latest')).toBe(false)
  })
  it('never presents a guessed manager page when the matching Release is absent', async () => {
    mocks.invoke.mockImplementation(async (command, args) => command === 'get_update_status' ? managerUpdateStatus.value
      : command === 'update_release_details' ? { target: args.target, channel: args.channel, candidateVersion: args.candidateVersion, notesState: 'unavailable', notes: null, notesSource: null, releaseUrl: null, fetchedAt: 100, publishedAt: null, cached: false } : null)
    const w=render('manager');await flushPromises()
    expect(w.text()).not.toContain('查看网页详情')
    expect(w.text()).toContain('尚未提供本版本更新说明')
    expect(managerUpdateStatus.value?.availableVersion).toBe('0.1.10-beta.1')
  })
  it('notes failure has an independent retry and retains the confirmed manager candidate', async () => {
    let calls=0
    mocks.invoke.mockImplementation(async (command,args) => command==='get_update_status' ? managerUpdateStatus.value
      : command==='update_release_details' ? { target:args.target,channel:args.channel,candidateVersion:args.candidateVersion,notesState:++calls===1?'failed':'ready',notes:calls===1?null:'recovered notes',notesSource:null,releaseUrl:null,fetchedAt:100,publishedAt:null,cached:false }:null)
    const w=render('manager');await flushPromises()
    const retry=w.findAll('button').find(button=>button.text()==='重试说明');expect(retry).toBeDefined()
    await retry!.trigger('click');await flushPromises()
    expect(w.text()).toContain('recovered notes');expect(managerUpdateStatus.value?.error).toBeNull()
    expect(mocks.check).not.toHaveBeenCalled();expect(mocks.install).not.toHaveBeenCalled()
  })
})
