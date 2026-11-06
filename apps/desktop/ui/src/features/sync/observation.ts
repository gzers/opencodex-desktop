import type { ConnectionState } from '@/types/ui'
import type { SaveSyncEndpointRequest, SyncEndpointDto } from './api'

export type WebdavState = ConnectionState | 'configured' | 'connected' | 'unknown'
export type ObservedState = 'connected' | 'synced' | 'conflict' | 'failed'
export interface ObservationToken { revision: string; generation: number }
export interface ConnectionObservation {
  revision: string
  state: ObservedState
  observedAt: string
  source: 'test' | 'sync-preview' | 'sync' | 'version-check'
  remoteTime?: string | null
  remoteExists?: boolean
  remoteRead?: 'present' | 'absent' | 'failed'
  remoteRevision?: string | null
  remoteObservedAt?: string
  comparison?: 'same' | null
}
export type ObservationDetails = Partial<Omit<ConnectionObservation, 'revision' | 'state'>>
export function remoteVersionHint(value: ConnectionObservation | null, reading = false): string {
  if (reading) return '读取中'
  if (value?.remoteRead === 'failed') return '读取失败'
  if (value?.remoteExists === false || value?.remoteRead === 'absent') return '未发现快照'
  if (value?.remoteTime) return '上次观察'
  if (value?.remoteExists || value?.remoteRead === 'present') return '已读取 · 保存时间未知'
  return '尚未读取'
}
export function connectionAddress(url: string, remotePath: string): string {
  return url.replace(/\/+$/, '') + '/' + remotePath.replace(/^\/+|\/+$/g, '')
}
const STORAGE_KEY = 'opencodex:webdav-observation:v1'
const opaqueRevision = (value: string) => /^[a-f0-9]{64}$/.test(value)
export function endpointBinding(endpoint: SyncEndpointDto | null | undefined): string {
  if (!endpoint) return ''
  if (endpoint.revision) return endpoint.revision
  // Older DEV fixtures have no revision. They can be observed in memory, but
  // cannot restore a credential observation across a reload.
  return JSON.stringify([endpoint.endpointId, endpoint.url.replace(/\/$/, ''), endpoint.remotePath.replace(/^\/+|\/+$/g, ''), endpoint.username, endpoint.credentialRefId])
}
export function sameEndpointForm(value: SaveSyncEndpointRequest, endpoint: SyncEndpointDto | null | undefined): boolean {
  return !!endpoint && value.baseUrl.trim().replace(/\/+$/, '') === endpoint.url.replace(/\/+$/, '')
    && value.remotePath.trim().replace(/^\/+|\/+$/g, '') === endpoint.remotePath.replace(/^\/+|\/+$/g, '')
    && value.username.trim() === endpoint.username
}
export function formSignature(value: SaveSyncEndpointRequest): string {
  // Kept only in the open form. Never persist or log this secret-bearing value.
  return JSON.stringify([value.baseUrl.trim().replace(/\/+$/, ''), value.remotePath.trim().replace(/^\/+|\/+$/g, ''), value.username.trim(), value.password, value.conflictPolicy])
}
export function readObservation(revision: string): ConnectionObservation | null {
  if (!opaqueRevision(revision)) return null
  try {
    const value = JSON.parse(localStorage.getItem(STORAGE_KEY) ?? 'null')
    if (!value || value.revision !== revision || !['connected', 'synced', 'conflict', 'failed'].includes(value.state)
      || !['test', 'sync-preview', 'sync', 'version-check'].includes(value.source) || typeof value.observedAt !== 'string'
      || !Number.isFinite(Date.parse(value.observedAt))) return null
    return {
      revision, state: value.state, source: value.source, observedAt: value.observedAt,
      ...(value.remoteTime === null || typeof value.remoteTime === 'string' && Number.isFinite(Date.parse(value.remoteTime)) ? { remoteTime: value.remoteTime } : {}),
      ...(typeof value.remoteExists === 'boolean' ? { remoteExists: value.remoteExists } : {}),
      ...(['present', 'absent', 'failed'].includes(value.remoteRead) ? { remoteRead: value.remoteRead } : {}),
      ...(value.remoteRevision === null || typeof value.remoteRevision === 'string' && value.remoteRevision.length <= 128 ? { remoteRevision: value.remoteRevision } : {}),
      ...(typeof value.remoteObservedAt === 'string' && Number.isFinite(Date.parse(value.remoteObservedAt)) ? { remoteObservedAt: value.remoteObservedAt } : {}),
      ...(value.comparison === 'same' ? { comparison: 'same' } : {}),
    }
  } catch { return null }
}
export function persistObservation(value: ConnectionObservation): void {
  if (!opaqueRevision(value.revision)) return
  try { localStorage.setItem(STORAGE_KEY, JSON.stringify(value)) } catch { /* A display cache is not a successful save requirement. */ }
}
