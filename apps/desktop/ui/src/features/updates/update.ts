import type { UpdateCheckTrigger } from './scheduler'
import { Channel, invoke } from '@tauri-apps/api/core'
import { shallowRef } from 'vue'

export const managerUpdateStatus = shallowRef<UpdateStatusDto | null>(null)
export const managerInstallProgress = shallowRef<UpdateProgress | null>(null)
export interface UpdateProgress {
  operationId: string
  target: 'manager'
  channel: UpdateChannel
  candidateVersion: string
  generation: number
  sequence: number
  stage: 'backup' | 'checking' | 'downloading' | 'installing' | 'pending_restart' | 'failed'
  downloadedBytes: number
  totalBytes: number | null
}
export interface InstallUpdateOptions {
  candidateVersion: string
  channel: UpdateChannel
  backup: boolean
}


export type UpdateChannel = 'stable' | 'beta'

export interface UpdateStatusDto {
  channel: UpdateChannel
  availableVersion: string | null
  currentVersion: string
  lastCheckedAt: string | null
  signatureVerified: boolean | null
  error: string | null
  notes?: string | null
  releaseUrl?: string | null
  publishedAt?: string | null
  pendingRestart?: string | null
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

let pendingInstall: Promise<void> | null = null
let pendingCandidate: InstallUpdateOptions | undefined
export function installUpdate(options?: InstallUpdateOptions): Promise<void> {
  if (pendingInstall) {
    if (options && pendingCandidate && (options.channel !== pendingCandidate.channel || options.candidateVersion !== pendingCandidate.candidateVersion || options.backup !== pendingCandidate.backup)) {
      return Promise.reject(new Error('Another confirmed update is active'))
    }
    return pendingInstall
  }
  pendingCandidate = options ? { ...options } : undefined
  const operation = (async () => {
    const status = options ? null : await getUpdateStatus()
    const chosen = options ?? {
      candidateVersion: status?.availableVersion ?? '', channel: status?.channel ?? 'stable', backup: true,
    }
    if (!chosen.candidateVersion) throw new Error('No confirmed update candidate')
    pendingCandidate = { ...chosen }
    managerInstallProgress.value = null
    let observing = true
    let operationId: string | null = null
    let generation = -1
    let sequence = -1
    const progress = new Channel<UpdateProgress>()
    progress.onmessage = value => {
      if (!observing || value.target !== 'manager' || value.channel !== chosen.channel || value.candidateVersion !== chosen.candidateVersion) return
      if (operationId !== null && (value.operationId !== operationId || value.generation !== generation)) return
      if (!value.operationId || !Number.isSafeInteger(value.generation) || value.generation < 0) return
      if (!Number.isSafeInteger(value.downloadedBytes) || value.downloadedBytes < 0) return
      if (value.totalBytes !== null && (!Number.isSafeInteger(value.totalBytes) || value.totalBytes <= 0 || value.totalBytes < value.downloadedBytes)) return
      if (!Number.isSafeInteger(value.sequence) || value.sequence <= sequence) return
      operationId = value.operationId
      generation = value.generation
      sequence = value.sequence
      managerInstallProgress.value = value
    }
    try {
      await invoke('install_update', { ...chosen, progress })
    } finally {
      observing = false
      // Own the operation across routes and modal dismissal; refresh never invents success.
      await getUpdateStatus().catch(() => undefined)
    }
  })().finally(() => { if (pendingInstall === operation) { pendingInstall = null; pendingCandidate = undefined } })
  pendingInstall = operation
  return operation
}
export async function restartAfterUpdate(): Promise<void> {
  await invoke('restart_after_update')
}


let pendingCheck: Promise<CheckUpdateResult> | null = null
let activeChannel: UpdateChannel = 'stable'
let channelEpoch = 0

function publishStatus(status: UpdateStatusDto) {
  if (activeChannel !== status.channel) { pendingCheck = null; channelEpoch++ }
  activeChannel = status.channel
  managerUpdateStatus.value = status
}

export function checkForUpdate(trigger: UpdateCheckTrigger = 'user'): Promise<CheckUpdateResult> {
  if (pendingCheck) return pendingCheck
  const epoch = channelEpoch
  const request = invoke<CheckUpdateResult>('check_for_update', { trigger }).then(result => {
    if (epoch === channelEpoch && result.status !== 'superseded') managerUpdateStatus.value = result.update
    return result
  }).finally(() => {
    if (pendingCheck === request) pendingCheck = null
  })
  pendingCheck = request
  return request
}
