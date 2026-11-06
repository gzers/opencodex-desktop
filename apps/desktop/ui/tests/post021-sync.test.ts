import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount, flushPromises } from '@vue/test-utils'
import { useSyncStore } from '@/features/sync/store'
import {getSyncConfig,type SyncConfigDto} from '@/features/sync/api'
import {configV2Api} from '@/features/sync/api-v2'
import {catalog as directory} from './helpers/syncWorkspaceV2'
import { remoteVersionHint } from '@/features/sync/observation'
import { primeAppData } from '@/app/bootstrap'
import SyncRoute from '@/routes/SyncRoute.vue'
vi.mock('@/features/sync/api', async original => ({ ...await original<typeof import('@/features/sync/api')>(), getSyncConfig: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue(undefined) }))
const config = (revision = 'a'.repeat(64)): SyncConfigDto => ({ endpoint: { endpointId: 'default', revision, url: 'https://dav.example.test/base', remotePath: 'device/nested', username: 'fixture', credentialRefId: 'opaque', tlsPolicy: 'verify_required', payloadProtection: 'integrity_only', conflictPolicy: 'ask' } })
beforeEach(() => { setActivePinia(createPinia()); localStorage.clear(); vi.clearAllMocks(); vi.mocked(getSyncConfig).mockResolvedValue(config());vi.spyOn(configV2Api,'catalog') })
describe('post021 remote observations', () => {
  it('startup reads an already configured snapshot once without running a sync', async () => {
    const store = useSyncStore()
    vi.mocked(configV2Api.catalog).mockResolvedValue({...directory(),cloudState:'ready'})
    const app = Object.fromEntries(['loadDataRootConfig', 'loadOfficialProject', 'refreshEnvironment', 'loadRuntimeSource', 'loadStatusSnapshot', 'loadNotifications', 'loadExtensionConfig', 'loadExtensions'].map(key => [key, vi.fn()]))
    app.loadSyncConfig = vi.fn(() => store.loadConfig())
    primeAppData(app as unknown as Parameters<typeof primeAppData>[0]); await flushPromises()
    primeAppData(app as unknown as Parameters<typeof primeAppData>[0]); await flushPromises()
    expect(configV2Api.catalog).toHaveBeenCalledTimes(1)
    expect(store.webdavState).toBe('connected'); expect(store.running).toBe(false)
    expect(remoteVersionHint(store.observation)).toBe('已读取 · 保存时间未知')
    setActivePinia(createPinia()); const reload = useSyncStore(); await reload.loadConfig()
    expect(reload.observation?.remoteRead).toBe('present')
    expect(remoteVersionHint(reload.observation)).not.toBe('尚未读取')
  })
  it('does not network without configuration; failed read can be explicitly retried', async () => {
    const store = useSyncStore(); store.adoptConfig({ endpoint: null })
    await store.inspectRemoteVersion(true); expect(configV2Api.catalog).not.toHaveBeenCalled()
    store.adoptConfig(config()); vi.mocked(configV2Api.catalog).mockRejectedValueOnce(new Error('fixture failure'))
    await store.inspectRemoteVersion(true); expect(remoteVersionHint(store.observation)).toBe('读取失败')
    await store.inspectRemoteVersion(true); expect(configV2Api.catalog).toHaveBeenCalledTimes(1)
    vi.mocked(configV2Api.catalog).mockResolvedValue(directory())
    await store.inspectRemoteVersion(); expect(remoteVersionHint(store.observation)).toBe('未发现快照')
  })
  it('rejects a late version response after endpoint replacement or a newer comparison', async () => {
    const store = useSyncStore(); store.adoptConfig(config())
    let resolve!:(value:ReturnType<typeof directory>)=>void
    vi.mocked(configV2Api.catalog).mockReturnValue(new Promise(r => { resolve = r }))
    const pending = store.inspectRemoteVersion(); store.adoptConfig(config('b'.repeat(64)))
    resolve({...directory(),cloudState:'ready'}); await pending
    expect(store.observation).toBeNull()
    vi.mocked(configV2Api.catalog).mockReturnValue(new Promise(r => { resolve = r }))
    const next = store.inspectRemoteVersion()
    const token = store.observationToken(); store.beginObservation(token, 'connecting')
    store.recordObservation(token, 'connected', { source: 'sync-preview', comparison: 'same', remoteExists: true, remoteRevision: 'newer' })
    resolve({...directory(),cloudState:'ready'}); await next
    expect(store.observation?.remoteRevision).toBe('newer'); expect(store.observation?.comparison).toBe('same')
  })
  it('renders the real connection summary with one directory boundary slash', async () => {
    useSyncStore().adoptConfig(config())
    const w = mount(SyncRoute, { global: { stubs: { AppTopbar: true, ModelTransfer: { template: '<div><slot name="connection" /></div>' } } } })
    await w.findAll('[role=tab]')[1]!.trigger('click'); await flushPromises()
    expect(w.find('.sync-connection-summary small').text()).toBe('https://dav.example.test/base/device/nested')
    w.unmount()
  })
})
