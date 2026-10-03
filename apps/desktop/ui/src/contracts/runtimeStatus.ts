export type RuntimeState =
  | 'loading'
  | 'not_found'
  | 'stopped'
  | 'starting'
  | 'pending'
  | 'running'
  | 'stopping'
  | 'starting_failed'
  | 'at_risk'
  | 'external_takeover'
  | 'unreachable'

export type ConnectionState =
  | 'conflict'
  | 'syncing'
  | 'connecting'
  | 'failed'
  | 'disconnected'
  | 'unconfigured'
  | 'synced'

export type OperationState =
  | 'rolling_back'
  | 'applying'
  | 'backing_up'
  | 'validating'
  | 'failed'
  | 'succeeded'
  | 'cancelled'
  | 'idle'

export type HealthState = 'healthy' | 'degraded' | 'unhealthy' | 'unknown'

export type StatusSource = 'unconfigured' | 'live' | 'fixture'

export interface StatusMatrix {
  runtime: RuntimeState
  connection: ConnectionState
  operation: OperationState
}

export interface RuntimeFacts {
  health: HealthState
  runtime_label: string | null
  opencodex_home: string | null
  data_root: string
  protection: string | null
  reboot_safe: boolean | null
  service_present: boolean | null
  shim_present: boolean | null
  version_drift: string | null
}

export interface StatusSnapshot {
  matrix: StatusMatrix
  facts: RuntimeFacts
  port: number | null
  pid: string | null
  can_start: boolean
  can_stop: boolean
  can_restart: boolean
  source: StatusSource
}

export async function getStatusSnapshot(): Promise<StatusSnapshot> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<StatusSnapshot>('get_status_snapshot')
}

export interface RestoreGuidance {
  runtimeState: RuntimeState
  health: HealthState
  startupStatus: string | null
  protection: string | null
  rebootSafe: boolean | null
  servicePresent: boolean | null
  shimPresent: boolean | null
  versionDrift: string | null
  dataRoot: string
  opencodexHome: string | null
}
