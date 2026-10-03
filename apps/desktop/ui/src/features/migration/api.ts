// 配置迁移命令契约。新容器（v2）不含额外口令；只有旧版加密容器导入才需要口令。
export interface MigrationImportRequest {
  passphrase?: string
}

export interface MigrationExportResult {
  path: string
  backupId: string | null
  documentSha256: string
  formatVersion: number
  sections: string[]
  excluded: string[]
}

export interface MigrationImportSectionSkip {
  section: string
  reason: string
}

export interface MigrationImportResult {
  backupId: string
  documentSha256: string
  formatVersion: number
  appliedSections: string[]
  skippedSections: MigrationImportSectionSkip[]
  excluded: string[]
}

export async function exportMigration(): Promise<MigrationExportResult> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<MigrationExportResult>('export_migration')
}

export async function importMigration(request: MigrationImportRequest = {}): Promise<MigrationImportResult> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<MigrationImportResult>('import_migration', { request })
}
