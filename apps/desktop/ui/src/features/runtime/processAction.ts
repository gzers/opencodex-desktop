import type { LifecycleAction, LifecycleResult, ProcessLifecycleState } from '@/types/ui'

export interface ProcessActionResult {
  lifecycleState: ProcessLifecycleState
  result: LifecycleResult | null
  canStart: boolean
  canStop: boolean
  canRestart: boolean
}

export async function runProcessAction(action: LifecycleAction): Promise<ProcessActionResult> {
  const invoke = (await import('@tauri-apps/api/core')).invoke
  return invoke<ProcessActionResult>('process_action', { request: { action, confirm: true } })
}
