import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount } from '@vue/test-utils'
import { defineComponent } from 'vue'
import ConnectionDialog from '@/features/sync/ConnectionDialog.vue'
import { useSyncStore } from '@/features/sync/store'
import { getSyncConfig, saveSyncEndpoint, testSyncCandidate, type SyncConfigDto, type SyncOperationResultDto } from '@/features/sync/api'
vi.mock('@/features/sync/api', async original => ({ ...await original<typeof import('@/features/sync/api')>(), getSyncConfig: vi.fn(), saveSyncEndpoint: vi.fn(), testSyncCandidate: vi.fn() }))
const dialog = defineComponent({ props: ['open', 'busy'], emits: ['close'], template: '<section v-if="open"><button aria-label="关闭" :disabled="busy" @click="$emit(\'close\')">×</button><slot/><slot name="actions" :close="()=> $emit(\'close\')"/></section>' })
const render = () => mount(ConnectionDialog, { global: { stubs: { UiDialog: dialog, ImportedConnections: true } } })
const config = (revision = 'a'.repeat(64)): SyncConfigDto => ({ endpoint: { endpointId: 'default', revision, url: 'https://dav.example.test', remotePath: 'scope', username: 'fixture', credentialRefId: 'opaque', tlsPolicy: 'verify_required', payloadProtection: 'integrity_only', conflictPolicy: 'ask' } })
const success: SyncOperationResultDto = { connectionState: 'synced', operationState: 'succeeded', message: 'test succeeded', snapshotId: null, backupId: null, etag: null }
const click = async (w: ReturnType<typeof render>, text: string) => { await w.findAll('button').find(b => b.text() === text)!.trigger('click'); await flushPromises() }
beforeEach(() => { setActivePinia(createPinia()); localStorage.clear(); vi.clearAllMocks(); vi.mocked(getSyncConfig).mockResolvedValue(config()); vi.mocked(testSyncCandidate).mockResolvedValue(success); vi.mocked(saveSyncEndpoint).mockResolvedValue(config('b'.repeat(64))) })
describe('connection observation integration', () => {
  it('tests the saved endpoint and updates the shared state without saving', async () => {
    const w = render(); await flushPromises(); const store = useSyncStore()
    expect(store.webdavState).toBe('configured'); expect(w.text()).toContain('已配置 · 未检测'); await click(w, '测试连接')
    expect(store.webdavState).toBe('connected'); expect(saveSyncEndpoint).not.toHaveBeenCalled()
    await click(w, '取消'); expect(w.emitted('close')).toHaveLength(1); w.unmount()
    const reopened = render(); await flushPromises(); expect(reopened.text()).toContain('最近检测成功'); expect(reopened.text()).toMatch(/\d{4}\/\d{2}\/\d{2}/); reopened.unmount()
  })
  it('does not leak an unsaved candidate result, then carries exact tested proof through saving', async () => {
    const w = render(); await flushPromises(); const store = useSyncStore()
    await w.findAll('input')[1]!.setValue('new-scope'); await click(w, '测试连接')
    expect(w.text()).toContain('test succeeded'); expect(store.webdavState).toBe('configured')
    vi.mocked(saveSyncEndpoint).mockResolvedValue({ endpoint: { ...config('b'.repeat(64)).endpoint!, remotePath: 'new-scope' } })
    await click(w, '保存连接'); expect(store.webdavState).toBe('connected')
    expect(store.observation?.revision).toBe('b'.repeat(64)); expect(w.emitted('close')).toHaveLength(1); w.unmount()
  })
  it('discards proof when the form changes after testing', async () => {
    const w = render(); await flushPromises(); await w.findAll('input')[1]!.setValue('tested-scope'); await click(w, '测试连接')
    await w.findAll('input')[1]!.setValue('another-scope'); await click(w, '保存连接')
    expect(useSyncStore().webdavState).toBe('configured'); expect(useSyncStore().observation).toBeNull(); w.unmount()
  })
  it('does not apply a late response after its saved endpoint revision changes', async () => {
    let resolve!: (value: SyncOperationResultDto) => void
    vi.mocked(testSyncCandidate).mockReturnValue(new Promise(r => { resolve = r }))
    const w = render(); await flushPromises(); const store = useSyncStore()
    await w.findAll('button').find(b => b.text() === '测试连接')!.trigger('click')
    store.adoptConfig(config('c'.repeat(64))); resolve(success); await flushPromises()
    expect(w.text()).not.toContain('test succeeded'); expect(store.webdavState).toBe('configured'); w.unmount()
  })
  it('supports a tested new connection without claiming it is active before save', async () => {
    vi.mocked(getSyncConfig).mockResolvedValue({ endpoint: null })
    const w = render(); await flushPromises(); const fields = w.findAll('input')
    for (const [index, value] of ['https://dav.example.test', 'scope', 'fixture', 'synthetic-secret'].entries()) await fields[index]!.setValue(value)
    await click(w, '测试连接'); expect(w.text()).toContain('test succeeded'); expect(useSyncStore().webdavState).toBe('unconfigured')
    await click(w, '保存连接'); expect(useSyncStore().webdavState).toBe('connected')
    expect(JSON.stringify(localStorage)).not.toContain('synthetic-secret'); w.unmount()
  })
})
