import type { NotificationsDto } from '@/features/notifications/api'
import type { SyncOperationResultDto } from '@/features/sync/api'

export type ManagedPathKey = 'manager_state' | 'backups' | 'logs' | 'exports' | 'cache' | 'sync_state'

export type AgentPathKind = 'skills-source' | 'skills' | 'mcp'

export interface ManagedPathDto {
  key: ManagedPathKey
  path: string
  label: string
}

export interface AgentPathDto {
  kind: AgentPathKind
  client: string
  label: string
  path: string
}

export interface ManagedPathTargetsDto {
  managed: ManagedPathDto[]
  agent: AgentPathDto[]
}

export interface OpenResultDto {
  opened: boolean
}

export interface LocalCleanupResultDto {
  cleanedLogs: number
  summary: string
}

export async function getManagedPathTargets(): Promise<ManagedPathTargetsDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<ManagedPathTargetsDto>('get_managed_path_targets')
}

export async function openManagedPath(path: string): Promise<OpenResultDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<OpenResultDto>('open_managed_path', { path })
}

export async function openExternalLink(url: string): Promise<OpenResultDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<OpenResultDto>('open_external_link', { url })
}

export async function openLocalDocument(documentId: string): Promise<OpenResultDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<OpenResultDto>('open_local_document', { documentId })
}

export async function runSyncNow(): Promise<SyncOperationResultDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<SyncOperationResultDto>('run_sync_now')
}

export async function cleanupLocalLogs(): Promise<LocalCleanupResultDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<LocalCleanupResultDto>('cleanup_local_logs')
}

export async function cleanupNotifications(): Promise<NotificationsDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<NotificationsDto>('cleanup_local_notifications')
}
