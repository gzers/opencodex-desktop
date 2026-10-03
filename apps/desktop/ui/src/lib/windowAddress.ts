import type { StatusSnapshot } from '@/contracts/runtimeStatus'

export function windowAddress(snapshot: StatusSnapshot | null): string {
  if (snapshot?.matrix.runtime !== 'running' || snapshot.port === null) return ''
  return `本地 · 127.0.0.1:${snapshot.port}`
}
