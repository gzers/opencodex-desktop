import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useSyncStore } from '@/features/sync/store'
import { getSyncConfig, saveSyncEndpoint, testSyncConnection, type SyncConfigDto, type SyncOperationResultDto } from '@/features/sync/api'
import { endpointBinding } from '@/features/sync/observation'
vi.mock('@/features/sync/api', async original => ({ ...await original<typeof import('@/features/sync/api')>(), getSyncConfig: vi.fn(), saveSyncEndpoint: vi.fn(), testSyncConnection: vi.fn() }))
vi.mock('@/platform/workspace', () => ({ runSyncNow: vi.fn() }))
const config = (revision = 'a'.repeat(64)): SyncConfigDto => ({ endpoint: { endpointId: 'default', revision, url: 'https://dav.example.test', remotePath: 'scope', username: 'fixture', credentialRefId: 'opaque', tlsPolicy: 'verify_required', payloadProtection: 'integrity_only', conflictPolicy: 'ask' } })
const success: SyncOperationResultDto = { connectionState: 'synced', operationState: 'succeeded', message: 'ok', snapshotId: null, backupId: null, etag: null }
beforeEach(() => { setActivePinia(createPinia()); localStorage.clear(); vi.clearAllMocks() })
describe('saved endpoint observations', () => {
  it('shows configured/unchecked without networking and keeps timed observations across reload', async () => {
    vi.mocked(getSyncConfig).mockResolvedValue(config())
    const store = useSyncStore(); await store.loadConfig()
    expect(store.webdavState).toBe('configured'); expect(testSyncConnection).not.toHaveBeenCalled()
    store.recordObservation(store.observationToken(), 'connected', { observedAt: '2026-11-04T04:00:00Z' })
    await store.loadConfig(); expect(store.webdavState).toBe('connected')
    expect(store.observation?.observedAt).toBe('2026-11-04T04:00:00Z')
    setActivePinia(createPinia()); const reloaded = useSyncStore(); await reloaded.loadConfig()
    expect(reloaded.webdavState).toBe('connected'); expect(reloaded.observation?.revision).toBe('a'.repeat(64))
  })
  it('invalidates old result on a credential revision even with identical endpoint fields', () => {
    const store = useSyncStore(); store.adoptConfig(config()); const old = store.observationToken()
    store.recordObservation(old, 'connected')
    store.adoptConfig(config('b'.repeat(64)))
    expect(store.webdavState).toBe('configured'); expect(store.observation).toBeNull()
    expect(store.recordObservation(old, 'failed')).toBe(false)
  })
  it('never applies a late test response to a replacement endpoint', async () => {
    let resolve!: (value: SyncOperationResultDto) => void
    vi.mocked(testSyncConnection).mockReturnValue(new Promise(r => { resolve = r }))
    const store = useSyncStore(); store.adoptConfig(config()); const pending = store.testConnection()
    expect(store.webdavState).toBe('connecting')
    store.adoptConfig(config('c'.repeat(64))); resolve(success); await pending
    expect(store.webdavState).toBe('configured'); expect(store.status).toBeNull(); expect(store.observation).toBeNull()
  })
  it('distinguishes config read failure from no saved endpoint and does not erase known observations', async () => {
    const store = useSyncStore(); vi.mocked(getSyncConfig).mockRejectedValue(new Error('unreadable'))
    await store.loadConfig(); expect(store.webdavState).toBe('unknown')
    store.adoptConfig(config()); store.recordObservation(store.observationToken(), 'connected')
    await store.loadConfig(); expect(store.webdavState).toBe('connected'); expect(store.configReadError).toBeTruthy()
  })
  it('save invalidates an older in-flight config read and does not claim success connectivity', async () => {
    let resolve!: (value: SyncConfigDto) => void
    vi.mocked(getSyncConfig).mockReturnValue(new Promise(r => { resolve = r }))
    vi.mocked(saveSyncEndpoint).mockResolvedValue(config('d'.repeat(64)))
    const store = useSyncStore(); const pending = store.loadConfig()
    await store.commitEndpoint({ baseUrl: 'https://dav.example.test', remotePath: 'scope', username: 'fixture', password: 'synthetic-password', conflictPolicy: 'ask' })
    resolve(config()); await pending
    expect(endpointBinding(store.config?.endpoint)).toBe('d'.repeat(64)); expect(store.webdavState).toBe('configured')
    expect(JSON.stringify(localStorage)).not.toContain('synthetic-password')
  })
  it('restores only an exact opaque revision and rejects corrupt cache', async () => {
    localStorage.setItem('opencodex:webdav-observation:v1', JSON.stringify({ revision: 'a'.repeat(64), state: 'connected', source: 'test', observedAt: 'invalid' }))
    const store = useSyncStore(); store.adoptConfig(config()); expect(store.webdavState).toBe('configured')
    store.recordObservation(store.observationToken(), 'connected', { remoteTime: '2026-11-01T04:00:00Z', remoteExists: true })
    const record = localStorage.getItem('opencodex:webdav-observation:v1')!
    expect(record).not.toContain('dav.example'); expect(record).not.toContain('fixture')
    store.adoptConfig(config('e'.repeat(64))); expect(store.observation).toBeNull()
  })
})
