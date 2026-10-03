import { invoke } from '@tauri-apps/api/core'

export type UpdateChannel = 'stable' | 'beta'

export interface UpdateStatusDto {
  channel: UpdateChannel
  availableVersion: string | null
  currentVersion: string
  lastCheckedAt: string | null
  signatureVerified: boolean | null
  error: string | null
}

export interface CheckUpdateResult {
  status: 'up_to_date' | 'available' | 'failed'
  update: UpdateStatusDto
}

export async function getUpdateStatus(): Promise<UpdateStatusDto> {
  return await invoke<UpdateStatusDto>('get_update_status')
}

export async function setUpdateChannel(channel: UpdateChannel): Promise<UpdateStatusDto> {
  return await invoke<UpdateStatusDto>('set_update_channel', { channel })
}

export async function installUpdate(): Promise<void> {
  await invoke('install_update')
}

export async function checkForUpdate(): Promise<CheckUpdateResult> {
  return await invoke<CheckUpdateResult>('check_for_update')
}
