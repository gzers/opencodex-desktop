export type SyncConflictPolicy = 'ask' | 'keep-local' | 'keep-remote' | 'keep-both'

export interface SyncCredentialRefDto {
  refId: string
  backend: string
  serviceName: string
  accountKey: string
  purpose: string
  createdAt: string
  updatedAt: string
}

export interface SyncEndpointDto {
  endpointId: string
  url: string
  remotePath: string
  username: string
  credentialRefId: string
  tlsPolicy: string
  payloadProtection: string
  conflictPolicy: SyncConflictPolicy
}

export interface SyncConfigDto {
  endpoint: SyncEndpointDto | null
}

export interface SaveSyncEndpointRequest {
  baseUrl: string
  remotePath: string
  username: string
  password: string
  conflictPolicy: SyncConflictPolicy
}

export type SyncConnectionState =
  | 'conflict'
  | 'syncing'
  | 'connecting'
  | 'failed'
  | 'disconnected'
  | 'unconfigured'
  | 'synced'

export type SyncOperationState =
  | 'rolling_back'
  | 'applying'
  | 'backing_up'
  | 'validating'
  | 'failed'
  | 'succeeded'
  | 'cancelled'
  | 'idle'

export interface SyncOperationResultDto {
  connectionState: SyncConnectionState
  operationState: SyncOperationState
  message: string
  snapshotId: string | null
  backupId: string | null
  etag: string | null
}

export async function getSyncConfig(): Promise<SyncConfigDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<SyncConfigDto>('get_sync_config')
}

export async function saveSyncEndpoint(request: SaveSyncEndpointRequest): Promise<SyncConfigDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<SyncConfigDto>('save_sync_endpoint', { request })
}

export async function deleteSyncEndpoint(deleteCredentials: boolean): Promise<SyncConfigDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<SyncConfigDto>('delete_sync_endpoint', { deleteCredentials })
}

export async function testSyncConnection(): Promise<SyncOperationResultDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<SyncOperationResultDto>('test_sync_connection')
}

export async function getSyncStatus(): Promise<SyncOperationResultDto> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<SyncOperationResultDto>('get_sync_status')
}
