export type DataRootValidation = 'valid' | 'future_version' | 'corrupted'

export interface DataRootInitialization {
  created: boolean
  structureVersion: string
}

export async function initializeDataRoot(rootPath: string): Promise<DataRootInitialization> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<DataRootInitialization>('initialize_data_root', { request: { rootPath } })
}

export async function validateDataRootStructure(rootPath: string): Promise<DataRootValidation> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<DataRootValidation>('validate_data_root_structure', { rootPath })
}

export type OpenCodexHomeMode = 'inside' | 'external'
export type DataRootSwitchMode = 'reference_only' | 'migrate_data'
export type DataRootSwitchStatus = 'applied' | 'restart_required' | 'blocked'
export type DataRootSwitchBlocked =
  | 'running'
  | 'runtime_unverified'
  | 'nested'
  | 'future_version'
  | 'corrupted'

export interface DataRootConfig {
  activeDataRoot: string
  opencodexHomeMode: OpenCodexHomeMode
  opencodexHome: string
  currentDataRoot: string
  currentOpencodexHome: string
  runtimeActive: boolean
}

export interface DataRootSwitchResult {
  status: DataRootSwitchStatus
  blocked: DataRootSwitchBlocked | null
  config: DataRootConfig | null
  reconciliationRequired: boolean
}

export async function getDataRootConfig(): Promise<DataRootConfig> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<DataRootConfig>('get_data_root_config')
}

export async function switchDataRoot(
  targetPath: string,
  mode: DataRootSwitchMode,
): Promise<DataRootSwitchResult> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<DataRootSwitchResult>('switch_data_root', {
    request: { targetPath, mode },
  })
}

export async function setOpencodexHome(
  mode: OpenCodexHomeMode,
  externalPath?: string,
): Promise<DataRootSwitchResult> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<DataRootSwitchResult>('set_opencodex_home_config', {
    request: { mode, externalPath },
  })
}
