import { invoke } from '@tauri-apps/api/core'
export type DetailsTarget = 'manager' | 'official'
export type DetailsChannel = 'stable' | 'beta' | 'official'
export interface ReleaseDetails {
  target: DetailsTarget
  channel: DetailsChannel
  candidateVersion: string
  notesState: 'ready' | 'unavailable' | 'failed'
  notes: string | null
  notesSource: 'updater' | 'github_release' | null
  releaseUrl: string | null
  publishedAt: string | null
  fetchedAt: number
  cached: boolean
  checkedAt: number
  failure: {kind:'network'|'timeout'|'rate_limited'|'http_error'|'identity_mismatch'|'too_large'|'cache';httpStatus:number|null}|null
}
const pending = new Map<string, Promise<ReleaseDetails>>()
/** Root is a local ownership key only; backend resolves the actual admitted root. */
export function getReleaseDetails(target: DetailsTarget, channel: DetailsChannel, candidateVersion: string, root: string, force = false) {
  const key = JSON.stringify([root, target, channel, candidateVersion])
  const current = pending.get(key)
  if (current) return current
  const request = invoke<ReleaseDetails>('update_release_details', { target, channel, candidateVersion, force })
    .then(value => {
      if (!value || value.target !== target || value.channel !== channel || value.candidateVersion !== candidateVersion
        || !['ready', 'unavailable', 'failed'].includes(value.notesState)
        || !Number.isSafeInteger(value.fetchedAt)||!Number.isSafeInteger(value.checkedAt)||value.fetchedAt<0||value.checkedAt<value.fetchedAt
        || typeof value.cached!=='boolean'||value.failure!==null&&(!value.failure||!['network','timeout','rate_limited','http_error','identity_mismatch','too_large','cache'].includes(value.failure.kind))) throw new Error('Release detail identity mismatch')
      return value
    }).finally(() => { if (pending.get(key) === request) pending.delete(key) })
  pending.set(key, request)
  return request
}
