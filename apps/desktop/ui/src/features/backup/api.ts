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
