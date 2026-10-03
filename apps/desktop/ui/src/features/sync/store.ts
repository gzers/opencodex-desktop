// 同步功能切片的状态与动作（IMP-04 §19.4 E：从总 store 拆出，切片五）。
// 本 store 负责端点配置与连接/同步的 IPC、自身状态，以及界面对外的连接状态投影
// `webdavState`（E 切片十五归位）；Toast、通知刷新等跨域编排仍由根壳动作负责，
// 避免与根 store 形成循环依赖。
import { defineStore } from 'pinia'
import type { ConnectionState } from '@/types/ui'
import {
  deleteSyncEndpoint as deleteSyncEndpointCommand,
  getSyncConfig,
  saveSyncEndpoint as saveSyncEndpointCommand,
  testSyncConnection as testSyncConnectionCommand,
  type SaveSyncEndpointRequest,
  type SyncConfigDto,
  type SyncOperationResultDto,
} from './api'
import { runSyncNow as runSyncNowCommand } from '@/platform/workspace'

function failedStatus(message: string): SyncOperationResultDto {
  return {
    connectionState: 'failed',
    operationState: 'failed',
    message,
    snapshotId: null,
    backupId: null,
    etag: null,
  }
}

export const useSyncStore = defineStore('sync', {
  state: () => ({
    // 连接状态投影：随端点配置与一次同步/测试结果更新，供概览状态卡展示。
    webdavState: 'unconfigured' as ConnectionState,
    config: null as SyncConfigDto | null,
    status: null as SyncOperationResultDto | null,
    saving: false,
    testing: false,
    running: false,
  }),
  actions: {
    setWebdavState(next: ConnectionState) {
      this.webdavState = next
    },
    async loadConfig(): Promise<SyncConfigDto | null> {
      try {
        const config = await getSyncConfig()
        this.config = config
        return config
      } catch {
        // IPC 失败时保留未配置态；不臆造端点或凭据状态。
        return null
      }
    },
    async saveEndpoint(request: SaveSyncEndpointRequest): Promise<boolean> {
      if (this.saving) return false
      this.saving = true
      try {
        this.config = await saveSyncEndpointCommand(request)
        return true
      } catch {
        return false
      } finally {
        this.saving = false
      }
    },
    async deleteEndpoint(deleteCredentials: boolean): Promise<boolean> {
      if (this.saving) return false
      this.saving = true
      try {
        this.config = await deleteSyncEndpointCommand(deleteCredentials)
        return true
      } catch {
        return false
      } finally {
        this.saving = false
      }
    },
    async testConnection(): Promise<SyncOperationResultDto> {
      if (this.testing) return this.status ?? failedStatus('WebDAV 连接测试失败；本地内容未修改。')
      this.testing = true
      try {
        this.status = await testSyncConnectionCommand()
      } catch {
        this.status = failedStatus('WebDAV 连接测试失败；本地内容未修改。')
      } finally {
        this.testing = false
      }
      return this.status
    },
    async runNow(): Promise<boolean> {
      if (this.running) return false
      this.running = true
      try {
        this.status = await runSyncNowCommand()
        return true
      } catch {
        this.status = failedStatus('同步执行失败；本地内容未修改。')
        return false
      } finally {
        this.running = false
      }
    },
  },
})
