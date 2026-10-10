import { invoke } from '@tauri-apps/api/core'
import { shallowRef } from 'vue'

export const managerUpdateStatus = shallowRef<UpdateStatusDto | null>(null)

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
  status: 'up_to_date' | 'available' | 'failed' | 'superseded'
  update: UpdateStatusDto
}

export async function getUpdateStatus(): Promise<UpdateStatusDto> {
  const epoch = channelEpoch
  const result = await invoke<UpdateStatusDto>('get_update_status')
  if (epoch === channelEpoch) publishStatus(result)
  return result
}

export async function setUpdateChannel(channel: UpdateChannel): Promise<UpdateStatusDto> {
  const status = await invoke<UpdateStatusDto>('set_update_channel', { channel })
  // Invalidate only after a successful switch. A refused switch retains ownership.
  publishStatus(status)
  return status
}

export async function installUpdate(): Promise<void> {
  await invoke('install_update')
}

let pendingCheck: Promise<CheckUpdateResult> | null = null
let activeChannel: UpdateChannel = 'stable'
let channelEpoch = 0

function publishStatus(status: UpdateStatusDto) {
  if (activeChannel !== status.channel) { pendingCheck = null; channelEpoch++ }
  activeChannel = status.channel
  managerUpdateStatus.value = status
}

export function checkForUpdate(): Promise<CheckUpdateResult> {
  if (pendingCheck) return pendingCheck
  const epoch = channelEpoch
  const request = invoke<CheckUpdateResult>('check_for_update').then(result => {
    if (epoch === channelEpoch && result.status !== 'superseded') managerUpdateStatus.value = result.update
    return result
  }).finally(() => {
    if (pendingCheck === request) pendingCheck = null
  })
  pendingCheck = request
  return request
}
