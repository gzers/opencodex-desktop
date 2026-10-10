import type { TreeTableNode } from '@/components/ui/treeTable'

export interface BackupFileNode extends TreeTableNode {
  directory: boolean
  purpose: string
  canOpen: boolean
  children: BackupFileNode[]
}
export interface BackupFilesDto { rootPath: string; nodes: BackupFileNode[] }

export async function getBackupFiles(): Promise<BackupFilesDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<BackupFilesDto>('backup_files')
}
export async function openBackupFile(id: string): Promise<void> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<void>('open_backup_file', { id })
}


export interface PreferencesBackup {
  id: string; action: string; createdAt: string; bytes: number; pinned: boolean
  protected: boolean; protectionReason: string | null; preferencesCandidate: boolean; integrityVerified: boolean
}
export interface CleanupPreview {
  token: string; createdAt: string; candidateIds: string[]; candidateBytes: number
  retainedCount: number; keepRecent: number; keepDays: number
}
export interface BackupPolicy {
  schema_version: 1; mode: 'manual' | 'automatic'; keep_recent: 10; keep_days: 30
}
async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const { invoke } = await import('@tauri-apps/api/core')
  return invoke<T>(command, args)
}
export const listBackups = () => call<PreferencesBackup[]>('list_preferences_backups')
export const createBackup = () => call<{ backup: { backupId: string }; removedIds: string[]; cleanupError: string | null }>('create_preferences_backup')
export const pinBackup = (id: string, pinned: boolean) => call<void>('set_preferences_backup_pinned', { id, pinned })
export const previewCleanup = () => call<CleanupPreview>('preview_preferences_backup_cleanup')
export const executeCleanup = (preview: CleanupPreview) => call<string[]>('cleanup_preferences_backups', { preview })
export const restoreBackup = (id: string) => call<{ backupId: string; protectionBackupId: string; targetPath: string; refreshRequired: boolean; protectionReconciliationPending: boolean }>('restore_preferences_backup', { id })
export const getBackupPolicy = () => call<BackupPolicy>('preferences_backup_policy')
export const saveBackupPolicy = (policy: BackupPolicy) => call<void>('save_preferences_backup_policy', { policy })
