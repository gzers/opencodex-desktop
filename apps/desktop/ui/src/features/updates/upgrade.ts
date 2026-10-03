import type { RestoreGuidance } from '@/contracts/runtimeStatus'

export interface UpgradeBackupResult {
  backupId: string
  directory: string
  targetPath: string
}

export async function createUpgradeBackup(): Promise<UpgradeBackupResult> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<UpgradeBackupResult>('create_upgrade_backup')
}

export interface RestoreRiskSummary {
  guidance: RestoreGuidance
  preferencesConfigured: boolean
  extensionConfigured: boolean
  syncEndpointConfigured: boolean
  opencodexHome: string | null
}

export async function getRestoreRiskSummary(): Promise<RestoreRiskSummary> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<RestoreRiskSummary>('restore_risk_summary')
}

export async function createRestoreBackup(): Promise<UpgradeBackupResult> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<UpgradeBackupResult>('create_restore_backup')
}
