import { invoke } from '@tauri-apps/api/core'

export type DataRootValidation = 'valid' | 'uninitialized' | 'future_version' | 'corrupted'

export interface DataRootInitialization {
  created: boolean
  structureVersion: string
}

export async function initializeDataRoot(rootPath: string): Promise<DataRootInitialization> {
  return invoke<DataRootInitialization>('initialize_data_root', { request: { rootPath } })
}

export async function validateDataRootStructure(rootPath: string): Promise<DataRootValidation> {
  return invoke<DataRootValidation>('validate_data_root_structure', { rootPath })
}

export type OpenCodexHomeMode = 'inside' | 'external'
export type DataRootSwitchMode = 'reference_only'
export type DataRootSwitchStatus = 'applied' | 'restart_required' | 'blocked'
export type DataRootSwitchBlocked =
  | 'running'
  | 'runtime_unverified'
  | 'nested'
  | 'future_version'
  | 'corrupted'
  | 'unsupported'

export interface DataRootConfig {
  activeDataRoot: string
  /** Aggregate U from L0. Optional only for existing non-settings consumers. */
  aggregateRoot?: string
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
  return invoke<DataRootConfig>('get_data_root_config')
}

export async function switchDataRoot(
  targetPath: string,
  mode: DataRootSwitchMode,
): Promise<DataRootSwitchResult> {
  return invoke<DataRootSwitchResult>('switch_data_root', {
    request: { targetPath, mode },
  })
}

export async function setOpencodexHome(
  mode: OpenCodexHomeMode,
  externalPath?: string,
): Promise<DataRootSwitchResult> {
  return invoke<DataRootSwitchResult>('set_opencodex_home_config', {
    request: { mode, externalPath },
  })
}

export type PathStatusV2 = 'valid' | 'missing' | 'unreadable' | 'corrupted'
export interface PathFactV2 { path: string; status: PathStatusV2 }
export const pathKeysV2 = ['aggregateRoot', 'desktopRoot', 'opencodexHome', 'configPath',
  'installationRoot', 'runtimePrefix', 'runtimeEntry', 'managerExecutable'] as const
export type PathKeyV2 = typeof pathKeysV2[number]
export type PathsSnapshotV2 = Record<PathKeyV2, PathFactV2> & {
  schemaVersion: 1
  program: {
    packageVersion: string | null
    manifestVersion: string | null
    sourceVersion: string | null
    consistency: 'consistent' | 'missing' | 'mismatch' | 'unreadable'
  }
  currentAggregateRoot: string
  pendingRestart: boolean
}

/** Unknown contracts never become invented filesystem facts or legacy paths. */
export async function getPathsSnapshotV2(): Promise<PathsSnapshotV2> {
  const value = await invoke<PathsSnapshotV2>('model_paths_v2_snapshot')
  if (!value || value.schemaVersion !== 1) throw new Error('目录快照结构版本暂不支持。')
  const statuses: string[] = ['valid', 'missing', 'unreadable', 'corrupted']
  if (pathKeysV2.some(key => !value[key] || typeof value[key].path !== 'string'
      || !statuses.includes(value[key].status))
    || typeof value.currentAggregateRoot !== 'string' || typeof value.pendingRestart !== 'boolean'
    || !value.program || !['consistent', 'missing', 'mismatch', 'unreadable'].includes(value.program.consistency)
    || ['packageVersion', 'manifestVersion', 'sourceVersion'].some(key => {
      const version = value.program[key as keyof PathsSnapshotV2['program']]
      return version !== null && typeof version !== 'string'
    })) throw new Error('目录快照内容无效；未推断路径或文件状态。')
  return value
}
