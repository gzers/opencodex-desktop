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
        || !['ready', 'unavailable', 'failed'].includes(value.notesState)) throw new Error('Release detail identity mismatch')
      return value
    }).finally(() => { if (pending.get(key) === request) pending.delete(key) })
  pending.set(key, request)
  return request
}
