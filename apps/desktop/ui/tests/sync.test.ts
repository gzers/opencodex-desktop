import {beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useAppStore } from '@/stores/app'
import type { SyncConfigDto, SyncOperationResultDto } from '@/features/sync/api'
import {useSyncStore} from '@/features/sync/store'
import {useConfigSyncV2Store} from '@/features/sync/store-v2'
import {catalog,preview,fact,id} from './helpers/syncWorkspaceV2'
import {mount,flushPromises} from '@vue/test-utils'
import SyncDialogHost from '@/features/sync/SyncDialogHost.vue'
import type {OperationFactV2} from '@/features/sync/api-v2'

const invoke = vi.hoisted(() => vi.fn())
vi.mock('@tauri-apps/api/core', () => ({ invoke }))
const savedConfig = (): SyncConfigDto => ({endpoint:{endpointId:'default',revision:'a'.repeat(64),url:'https://dav.example.test',remotePath:'scope',username:'fixture',credentialRefId:'opaque',tlsPolicy:'verify_required',payloadProtection:'integrity_only',conflictPolicy:'ask'}})

describe('WebDAV sync contract', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    localStorage.clear();useSyncStore().adoptConfig(savedConfig())
  })

  it('loads endpoint config and masks password', async () => {
    const config: SyncConfigDto = {
      endpoint: {
        endpointId: 'default',
        url: 'https://dav.example.test',
        remotePath: 'desktop-sync/current',
        username: 'user',
        credentialRefId: 'cred_12345678-1234-1234-1234-123456789abc',
        tlsPolicy: 'verify_required',
        payloadProtection: 'integrity_only',
        conflictPolicy: 'ask',
      },
    }
    invoke.mockResolvedValue(config)
    const app = useAppStore()
    await app.loadSyncConfig()
    expect(invoke).toHaveBeenCalledWith('get_sync_config')
    expect(app.syncConfig?.endpoint?.url).toBe('https://dav.example.test')
    expect(app.syncConfig?.endpoint).not.toHaveProperty('password')
    expect(app.webdavState).toBe('configured')
  })

  it('tests connection and drives connection state', async () => {
    const result: SyncOperationResultDto = {
      connectionState: 'failed',
      operationState: 'failed',
      message: 'TLS 证书校验失败，已停止连接。请确认服务器证书、系统时间与代理设置。',
      snapshotId: null,
      backupId: null,
      etag: null,
    }
    invoke.mockResolvedValue(result)
    const app = useAppStore()
    await app.testSyncConnection()
    expect(invoke).toHaveBeenCalledWith('test_sync_connection')
    expect(app.webdavState).toBe('failed')
    expect(app.syncStatus?.message).toContain('TLS 证书校验失败')
  })

  it('keeps previous config when save fails', async () => {
    invoke.mockRejectedValue(new Error('denied'))
    const app = useAppStore()
    useSyncStore().config = { endpoint: null }
    await app.saveSyncEndpoint({
      baseUrl: 'https://dav.example.test',
      remotePath: 'desktop-sync/current',
      username: 'user',
      password: 'password',
      conflictPolicy: 'ask',
    })
    expect(app.syncConfig?.endpoint).toBeNull()
  })
})

// 回归：同步冲突提醒由后端写入通知 store；同步结束后前端必须刷新通知，
// 否则冲突通知要等到下次打开通知中心才出现。
describe('sync conflict notification refresh', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    localStorage.clear();useSyncStore().adoptConfig(savedConfig())
  })

  it('reloads notifications after preparing a conflict and never confirms or calls the retired writer',async()=>{invoke.mockImplementation(async command=>command==='config_v2_catalog'?{...catalog(),cloudState:'conflict'}:command==='config_v2_preview'?Promise.reject({code:'sync_remote_conflict',step:'observe',retryability:'after_refresh',recoveryActions:[]}):{items:[]});const app=useAppStore();expect(await app.runSyncNow()).toBe(false);expect(app.webdavState).toBe('failed');expect(useConfigSyncV2Store().catalog?.cloudState).toBe('conflict');await flushPromises();expect(invoke).toHaveBeenCalledWith('config_v2_catalog',{readPassphrase:null});expect(invoke).toHaveBeenCalledWith('list_notifications');expect(invoke.mock.calls.some(c=>c[0]==='config_v2_confirm'||c[0]==='run_sync_now')).toBe(false)})
})

describe('sync terminal result is not command acknowledgement', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    localStorage.clear();useSyncStore().adoptConfig(savedConfig())
  })

  it.each([
    ['stored','observed_equal','observed_equal','settled','synced'],
    ['unknown','unknown','unknown','settled','failed'],
    ['stored','changed','conflict','settled','conflict'],
    ['stored','unknown','unknown','payload_verified','connected'],
  ] as const)('projects independent storage/local/remote facts: %s/%s/%s',async(storage,local,remote,phase,state)=>{
    const result:OperationFactV2={...fact(),storageStatus:storage,phase,relation:{local,remote},...(state==='failed'?{failure:{code:'sync_network',step:'commit_put',retryability:'same_operation_read_only',recoveryActions:[]}}:{})};invoke.mockImplementation(async command=>command==='config_v2_catalog'?catalog():command==='config_v2_preview'?preview():command==='config_v2_confirm'?result:command==='list_notifications'?{items:[]}:null);const w=mount(SyncDialogHost,{global:{stubs:{UiDialog:true,ConfigFileTransferV2:true}}});try{const app=useAppStore(),s=useConfigSyncV2Store();expect(await app.runSyncNow()).toBe(false);expect(invoke.mock.calls.some(c=>c[0]==='config_v2_confirm')).toBe(false);await s.confirm();await flushPromises();expect(useSyncStore().webdavState).toBe(state);expect(s.fact).toEqual(result);expect(useSyncStore().running).toBe(false);expect(invoke).toHaveBeenCalledWith('list_notifications');expect(invoke.mock.calls.some(c=>c[0]==='run_sync_now')).toBe(false)}finally{w.unmount()}
  })
  it('retains an uncertain original execution and refreshes notifications without claiming rollback or no local writes',async()=>{invoke.mockImplementation(async command=>{if(command==='config_v2_catalog')return catalog();if(command==='config_v2_preview')return preview();if(command==='config_v2_confirm')throw {code:'sync_network'};if(command==='config_v2_result')return {...fact(),storageStatus:'unknown',relation:{local:'unknown',remote:'unknown'}};if(command==='list_notifications')return {items:[]};return null});const app=useAppStore(),s=useConfigSyncV2Store();expect(await app.runSyncNow()).toBe(false);await s.confirm();await flushPromises();expect(s.fact?.operationId).toBe(id(101));expect(s.fact?.storageStatus).toBe('unknown');expect(s.resultError).toBe('sync_network');expect(s.resultError).not.toContain('本地内容未修改');expect(invoke).toHaveBeenCalledWith('list_notifications');expect(useSyncStore().running).toBe(false)})
})
