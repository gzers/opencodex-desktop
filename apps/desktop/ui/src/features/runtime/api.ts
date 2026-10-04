// 运行来源、托管安装与卸载的前端契约（IMP Track B · B3 / B4 / B5）。
//
// 只做 invoke 包装与类型声明；掩码、校验与真实副作用都在后端。

export type RuntimeSourceKind = 'explicit' | 'managed' | 'discovered' | 'unresolved'

export type InstallSourceKind = 'registry' | 'offline'

export type ProxyScheme = 'http' | 'socks5h'

export type InstallPhase =
  | 'preparing'
  | 'downloading'
  | 'installing'
  | 'validating'
  | 'extracting'
  | 'verifying'
  | 'activating'
  | 'done'
  | 'failed'

export interface InstallHistoryEntry {
  action: string
  result: string
  package: string
  version: string
  target: string
  at: string
  reason?: string | null
}

export interface RuntimeSourceDto {
  kind: RuntimeSourceKind
  path: string | null
  version: string | null
  resolvedAt: string | null
  insideDataRoot: boolean
  managedEntry: string | null
  /** 安装记录登记的托管前缀（FZ-51）；自定义前缀安装时为该目录。 */
  managedPrefix: string
  /** 托管前缀的默认落点 `<数据根>/runtime/opencodex`；「恢复默认」用它。 */
  defaultPrefix: string
  explicitPath: string | null
  history: InstallHistoryEntry[]
}

export interface RuntimeInstallProgress {
  phase: InstallPhase
  percent: number
  line?: string | null
}

export interface RuntimeInstallRequest {
  prefix: string | null
  source: InstallSourceKind
  version: string | null
  offlinePath: string | null
  proxyScheme: ProxyScheme | null
  proxyHost: string | null
  proxyUsername: string | null
  proxySecret: string | null
  allowScripts: boolean
}

export interface RuntimeInstallOutcomeDto {
  package: string
  version: string
  target: string
  entry: string
  tarballSha256: string
  source: InstallSourceKind
  scriptsEnabled: boolean
  npmPath: string
  installedAt: string
  proxyUsed: boolean
  needsRestart: boolean
}

export interface OfflinePackagePreviewDto {
  ok: boolean
  fileName: string
  fileSize: number
  version: string | null
  sha256Prefix: string | null
  rejectionCode: string | null
  message: string
}

export type UninstallScope = 'body' | 'full'
export type UninstallStepStatus = 'ok' | 'skipped' | 'failed'
export type ResidueStatus = 'cleared' | 'present' | 'unknown'

/** 卸载请求（Revision 11）：勾选即确认；`autoBackup` 勾选即承诺成功。 */
export interface RuntimeUninstallRequest {
  scope: UninstallScope
  autoBackup: boolean
  cleanData: boolean
  confirmation: string | null
}

/** 只读卸载方案：界面据此渲染「将移除的对象」与备份检测。 */
export interface RuntimeUninstallPlanDto {
  sourceKind: RuntimeSourceKind
  sourcePath: string | null
  managed: boolean
  backupId: string | null
  removable: boolean
  reason: string | null
  removeObjects: string[]
  runtimeObjects: string[]
  dataObjects: string[]
  residueCandidates: string[]
  officialCommand: string | null
  externalCommand: string | null
}

export interface RuntimeUninstallStepDto {
  name: string
  status: UninstallStepStatus
  detail: string | null
}

export interface RuntimeUninstallResidueDto {
  path: string
  status: ResidueStatus
}

export interface RuntimeUninstallResultDto {
  scope: UninstallScope
  sourceKind: RuntimeSourceKind
  sourcePath: string | null
  backupId: string | null
  backupDirectory: string | null
  steps: RuntimeUninstallStepDto[]
  residue: RuntimeUninstallResidueDto[]
  officialOutput: string[]
  needsRestart: boolean
  message: string
}

/** 安装进度事件名（载荷为 `RuntimeInstallProgress`）。 */
export const RUNTIME_INSTALL_PROGRESS_EVENT = 'runtime-install-progress'
/** 运行来源变化事件名；前端据此刷新卡片。 */
export const RUNTIME_SOURCE_CHANGED_EVENT = 'runtime-source-changed'

/** 运行来源记录损坏时后端返回的稳定错误码前缀。 */
export const RUNTIME_ERROR_CODE_PREFIX = 'runtime-managed'

async function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  const core = await import('@tauri-apps/api/core')
  return core.invoke<T>(command, args)
}

export async function getRuntimeSource(): Promise<RuntimeSourceDto> {
  return invoke<RuntimeSourceDto>('runtime_source')
}

export async function setRuntimeSource(path: string | null): Promise<RuntimeSourceDto> {
  return invoke<RuntimeSourceDto>('set_runtime_source', { path })
}

export async function restoreDiscoveredRuntime(): Promise<RuntimeSourceDto> {
  return invoke<RuntimeSourceDto>('restore_discovered_runtime')
}

export async function previewOfflinePackage(path: string): Promise<OfflinePackagePreviewDto> {
  return invoke<OfflinePackagePreviewDto>('preview_offline_package', { path })
}

export async function installRuntime(request: RuntimeInstallRequest): Promise<RuntimeInstallOutcomeDto> {
  return invoke<RuntimeInstallOutcomeDto>('install_runtime', { request })
}

/** 代跑官方更新（U-04）：后端先解析远端确定版本，再复用受控安装；前端需先显式确认。 */
export async function installOfficialUpdate(): Promise<RuntimeInstallOutcomeDto> {
  return invoke<RuntimeInstallOutcomeDto>('install_official_update')
}

export async function cancelRuntimeInstall(): Promise<boolean> {
  return invoke<boolean>('cancel_runtime_install')
}

export async function planRuntimeUninstall(): Promise<RuntimeUninstallPlanDto> {
  return invoke<RuntimeUninstallPlanDto>('plan_runtime_uninstall')
}

export async function uninstallRuntime(
  request: RuntimeUninstallRequest,
): Promise<RuntimeUninstallResultDto> {
  return invoke<RuntimeUninstallResultDto>('uninstall_runtime', { request })
}

export async function getOfficialUninstallObservation(): Promise<string[]> {
  return invoke<string[]>('official_uninstall_observation')
}
