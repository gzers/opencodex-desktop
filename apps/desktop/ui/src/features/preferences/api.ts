export interface PreferencesDto {
  schemaVersion: number
  /** 一次性历史主题导入信号；不是持久化字段。 */
  themeNeedsImport: boolean
  interfaceScale: number
  launchMain: boolean
  autoPanel: boolean
  panelMode: 'embedded' | 'browser'
  keepProxyOnClose: boolean
  lifecycleNotifications: boolean
  syncConflictAlerts: boolean
  launchWithCodex: boolean
  autoBackupUpgrade: boolean
  autoBackupImport: boolean
  autoBackupSync: boolean
  backupRetention: '5' | '10' | '20'
  backupIntegrity: 'sha-256' | 'sha-512' | 'blake3'
  backupIncludeSkills: boolean
  exportIncludeSkills: boolean
  mcpConflictPolicy: 'ask' | 'keep-target' | 'keep-both'
  mcpMask: boolean
  backupIncludeMcp: boolean
  exportIncludeMcp: boolean
  logRetention: '7d-5000' | '30d-10000' | '90d-30000'
  notificationRetention: '7' | '30' | '90'
  startupCleanup: boolean
  cleanupBackupSummary: boolean
  cliEnabled: boolean
  syncConflictPolicy: 'ask' | 'keep-local' | 'keep-remote' | 'keep-both'
  coldSync: boolean
  backupBeforeOverwrite: boolean
  appUpdateChannel: 'stable' | 'beta'
  appUpdateAutoCheck: boolean
  appUpdateCheckIntervalSeconds: number
  theme: 'light' | 'dark' | 'system'
  networkProxyMode: 'none' | 'system' | 'manual'
  networkProxyScheme: 'http' | 'socks5h'
  networkProxyHost: string
  networkNoProxy: string
  visualEffects: 'high' | 'mid' | 'low'
  /** 背景光渲染方式：WEBGL 网格渐变＋颗粒着色器（默认）/ 纯 CSS 极光（兜底）。 */
  glowRender: 'mesh' | 'css'
}

export async function getPreferences(): Promise<PreferencesDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<PreferencesDto>('get_preferences')
}

export async function savePreferences(preferences: PreferencesDto): Promise<PreferencesDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<PreferencesDto>('save_preferences', { preferences })
}

export async function restoreDefaultPreferences(): Promise<PreferencesDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<PreferencesDto>('restore_default_preferences')
}

/** 网络连通性检查结果（U-05）；只回报成功/失败，不改写任何状态。 */
export interface NetworkProbeDto {
  ok: boolean
  detail: string
}

export async function checkNetworkProxy(): Promise<NetworkProbeDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<NetworkProbeDto>('check_network_proxy')
}
