import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useAppStore } from '@/stores/app'
import type { SyncConfigDto, SyncOperationResultDto } from '@/features/sync/api'
import { useSyncStore } from '@/features/sync/store'

const invoke = vi.hoisted(() => vi.fn())
vi.mock('@tauri-apps/api/core', () => ({ invoke }))

describe('WebDAV sync contract', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
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
    expect(app.webdavState).toBe('disconnected')
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
  })

  it('reloads notifications after a sync run', async () => {
    invoke.mockImplementation((command: string) => {
      if (command === 'run_sync_now') {
        return Promise.resolve({
          connectionState: 'synced',
          operationState: 'succeeded',
          message: '检测到本地与远端冲突；已暂停覆盖并保留双方历史，请确认后重试。',
          snapshotId: 'snap',
          backupId: null,
          etag: null,
        })
      }
      if (command === 'list_notifications') {
        return Promise.resolve({ items: [], aggregate: { total: 0, unread: 0, unresolved: 0, dangerUnread: 0 } })
      }
      return Promise.resolve(undefined)
    })
    const app = useAppStore()
    await app.runSyncNow()
    await new Promise(resolve => setTimeout(resolve, 0))
    expect(invoke).toHaveBeenCalledWith('run_sync_now')
    expect(invoke).toHaveBeenCalledWith('list_notifications')
  })
})
