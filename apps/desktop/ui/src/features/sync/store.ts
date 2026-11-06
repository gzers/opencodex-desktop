import { defineStore } from 'pinia'
import type { ConnectionState } from '@/types/ui'
import {
  deleteSyncEndpoint as deleteSyncEndpointCommand,
  getSyncConfig,
  saveSyncEndpoint as saveSyncEndpointCommand,
  testSyncConnection as testSyncConnectionCommand,
  remoteVersionApi,
  type SaveSyncEndpointRequest, type SyncConfigDto, type SyncOperationResultDto,
} from './api'
import { runSyncNow as runSyncNowCommand } from '@/platform/workspace'
import { endpointBinding, persistObservation, readObservation, type ConnectionObservation, type ObservationDetails, type ObservationToken, type ObservedState, type WebdavState } from './observation'

function failedStatus(message: string): SyncOperationResultDto {
  return { connectionState: 'failed', operationState: 'failed', message, snapshotId: null, backupId: null, etag: null }
}
export const useSyncStore = defineStore('sync', {
  state: () => ({
    webdavState: 'unconfigured' as WebdavState,
    config: null as SyncConfigDto | null,
    configReadError: '',
    status: null as SyncOperationResultDto | null,
    observation: null as ConnectionObservation | null,
    observationGeneration: 0,
    observationSequence: 0,
    versionRequest: 0, versionAttempt: '', checkingVersion: false,
    configRequest: 0,
    saving: false, testing: false, running: false,
    connectionOpen: false, resultOpen: false, resultId: null as string | null,
    lastResultId: null as string | null, fileResultId: null as string | null, webdavResultId: null as string | null,
    pendingAction: null as null | 'sync' | 'export' | 'connection',
  }),
  actions: {
    openConnection() { this.connectionOpen = true },
    rememberResult(id: string, remote: boolean) { this.lastResultId = id; if (remote) this.webdavResultId = id; else this.fileResultId = id },
    openResult(id?: string) { this.resultId = id ?? this.lastResultId; this.resultOpen = true },
    closeResult() { this.resultOpen = false },
    setWebdavState(next: ConnectionState) { this.webdavState = next },
    requestAction(action: 'sync' | 'export' | 'connection') { this.pendingAction = action },
    takeAction(action: 'sync' | 'export' | 'connection'): boolean { if (this.pendingAction !== action) return false; this.pendingAction = null; return true },
    observationToken(): ObservationToken { return { revision: endpointBinding(this.config?.endpoint), generation: this.observationGeneration } },
    matchesConfig(token: ObservationToken): boolean { return token.revision === endpointBinding(this.config?.endpoint) && token.generation === this.observationGeneration },
    ownsObservation(token: ObservationToken): boolean { return !!token.revision && this.matchesConfig(token) },
    adoptConfig(config: SyncConfigDto, invalidate = false) {
      const before = endpointBinding(this.config?.endpoint), after = endpointBinding(config.endpoint)
      this.config = config; this.configReadError = ''
      if (before !== after || invalidate) {
        this.observationGeneration++
        this.observation = invalidate ? null : readObservation(after)
        this.status = null
        this.webdavState = !config.endpoint ? 'unconfigured' : this.observation?.state ?? 'configured'
      } else if (config.endpoint && this.webdavState === 'unconfigured') {
        this.observation = readObservation(after)
        this.webdavState = this.observation?.state ?? 'configured'
      } else if (!config.endpoint) { this.observation = null; this.webdavState = 'unconfigured' }
    },
    beginObservation(token: ObservationToken, state: 'connecting' | 'syncing') { if (this.ownsObservation(token)) { this.observationSequence++; this.webdavState = state } },
    recordObservation(token: ObservationToken, state: ObservedState, details: ObservationDetails = {}): boolean {
      if (!this.ownsObservation(token)) return false
      this.observationSequence++
      const previous = this.observation?.revision === token.revision ? this.observation : null
      const observedAt = details.observedAt && Number.isFinite(Date.parse(details.observedAt)) ? details.observedAt : new Date().toISOString()
      const observation: ConnectionObservation = {
        revision: token.revision, state, observedAt, source: details.source ?? 'test',
        ...(previous?.remoteTime !== undefined ? { remoteTime: previous.remoteTime } : {}),
        ...(previous?.remoteExists !== undefined ? { remoteExists: previous.remoteExists } : {}),
        ...(details.remoteTime !== undefined ? { remoteTime: details.remoteTime && Number.isFinite(Date.parse(details.remoteTime)) ? details.remoteTime : null } : {}),
        ...(details.remoteExists !== undefined ? { remoteExists: details.remoteExists } : {}),
        ...(previous?.remoteRead ? { remoteRead: previous.remoteRead } : {}),
        ...(previous?.remoteRevision !== undefined ? { remoteRevision: previous.remoteRevision } : {}),
        ...(previous?.remoteObservedAt ? { remoteObservedAt: previous.remoteObservedAt } : {}),
        ...(details.remoteRead ? { remoteRead: details.remoteRead } : {}),
        ...(details.remoteRevision !== undefined ? { remoteRevision: details.remoteRevision } : {}),
        ...(details.remoteObservedAt ? { remoteObservedAt: details.remoteObservedAt } : {}),
        ...(details.comparison === 'same' ? { comparison: 'same' } : {}),
      }
      this.observation = observation; this.webdavState = state; persistObservation(observation)
      return true
    },
    async inspectRemoteVersion(once = false): Promise<boolean> {
      const token = this.observationToken()
      if (!this.ownsObservation(token)) return false
      if (this.running || this.testing || this.webdavState === 'syncing') return false
      const attempt = token.generation + ':' + token.revision
      if (once && this.versionAttempt === attempt) return false
      this.versionAttempt = attempt
      const request = ++this.versionRequest
      this.checkingVersion = true; this.beginObservation(token, 'connecting')
      const sequence = this.observationSequence
      const current = () => request === this.versionRequest && this.ownsObservation(token) && sequence === this.observationSequence
      try {
        const value = await remoteVersionApi.read()
        if (!current()) return false
        this.recordObservation(token, 'connected', { source: 'version-check', remoteTime: value.remoteSavedAt, remoteExists: value.remoteExists, remoteRevision: value.remoteRevision, remoteRead: value.remoteExists ? 'present' : 'absent', remoteObservedAt: new Date().toISOString() })
        return true
      } catch {
        if (current()) this.recordObservation(token, 'failed', { source: 'version-check', remoteRead: 'failed', remoteObservedAt: new Date().toISOString() })
        return false
      } finally {
        if (request === this.versionRequest) this.checkingVersion = false
        if (current()) this.endObservation(token)
      }
    },
    endObservation(token: ObservationToken) {
      if (this.ownsObservation(token) && ['connecting', 'syncing'].includes(this.webdavState)) this.webdavState = this.observation?.state ?? 'configured'
    },
    async loadConfig(): Promise<SyncConfigDto | null> {
      if (this.saving) return this.config
      const request = ++this.configRequest
      try {
        const config = await getSyncConfig()
        if (request !== this.configRequest) return this.config
        this.adoptConfig(config)
        return config
      } catch {
        if (request === this.configRequest) {
          this.configReadError = '无法读取已保存连接，请重新读取。'
          if (!this.config) this.webdavState = 'unknown'
        }
        return null
      }
    },
    async commitEndpoint(request: SaveSyncEndpointRequest): Promise<SyncConfigDto> {
      if (this.saving) throw new Error('连接正在保存，请稍后重试。')
      this.saving = true
      const sequence = ++this.configRequest
      try {
        const revision = this.config?.endpoint?.revision ?? (this.config && !this.config.endpoint ? '' : undefined)
        const config = await saveSyncEndpointCommand({ ...request, ...(revision !== undefined ? { expectedRevision: revision } : {}) })
        if (sequence === this.configRequest) this.adoptConfig(config, true)
        return config
      } finally { this.saving = false }
    },
    async saveEndpoint(request: SaveSyncEndpointRequest): Promise<boolean> { try { await this.commitEndpoint(request); return true } catch { return false } },
    async deleteEndpoint(deleteCredentials: boolean): Promise<boolean> {
      if (this.saving) return false
      this.saving = true
      const request = ++this.configRequest
      try { const config = await deleteSyncEndpointCommand(deleteCredentials); if (request === this.configRequest) this.adoptConfig(config, true); return true }
      catch { return false } finally { this.saving = false }
    },
    async testConnection(): Promise<SyncOperationResultDto> {
      if (this.testing) return this.status ?? failedStatus('WebDAV 连接测试正在进行。')
      const token = this.observationToken(); this.testing = true; this.beginObservation(token, 'connecting')
      try {
        const status = await testSyncConnectionCommand()
        if (this.ownsObservation(token)) this.status = status
        this.recordObservation(token, status.operationState === 'succeeded' ? 'connected' : 'failed')
        return status
      } catch {
        const status = failedStatus('WebDAV 连接测试失败；本地内容未修改。')
        if (this.ownsObservation(token)) this.status = status
        this.recordObservation(token, 'failed'); return status
      } finally { this.testing = false; this.endObservation(token) }
    },
    async runNow(): Promise<boolean> {
      if (this.running) return false
      const token = this.observationToken(); this.running = true; this.beginObservation(token, 'syncing')
      try {
        const status = await runSyncNowCommand()
        if (this.ownsObservation(token)) this.status = status
        if (status.connectionState === 'conflict') this.recordObservation(token, 'conflict', { source: 'sync' })
        else if (status.operationState === 'succeeded' && status.connectionState === 'synced') this.recordObservation(token, 'synced', { source: 'sync' })
        else if (status.operationState === 'failed' || status.connectionState === 'failed') this.recordObservation(token, 'failed', { source: 'sync' })
        else if (status.connectionState === 'synced') this.recordObservation(token, 'connected', { source: 'sync' })
        return status.connectionState === 'synced' && status.operationState === 'succeeded'
      } catch {
        if (this.ownsObservation(token)) this.status = failedStatus('同步执行未完成；可能已有内容应用或上传，请检查同步状态与本地内容后重试。')
        this.recordObservation(token, 'failed', { source: 'sync' }); return false
      } finally { this.running = false; this.endObservation(token) }
    },
  },
})
