// One application timer; route components only read the shared results.
import { invoke } from '@tauri-apps/api/core'

export type ScheduledTarget = 'manager_stable' | 'manager_beta' | 'panel'
export interface UpdateSchedulePlan { targets: ScheduledTarget[]; nextDelayMs: number | null }
interface Dependencies {
  plan: () => Promise<UpdateSchedulePlan>
  check: (target: ScheduledTarget) => Promise<unknown>
  visible: () => boolean
  busy: () => boolean
  onError: () => void
}
export const getUpdateSchedulePlan = () => invoke<UpdateSchedulePlan>('update_schedule_plan')

export function createUpdateScheduler(deps: Dependencies) {
  let timer: ReturnType<typeof setTimeout> | null = null
  let started = false
  let disposed = false
  let running = false
  let pendingWake = false
  let notBefore = 0
  function clear() { if (timer !== null) clearTimeout(timer); timer = null }
  function arm(delay: number | null) {
    clear()
    if (disposed || !started || delay === null || !deps.visible()) return
    timer = setTimeout(() => { timer = null; void tick() }, Math.max(100, delay, notBefore - Date.now()))
  }
  async function tick() {
    if (disposed || !deps.visible()) return
    if (running) { pendingWake = true; return }
    if (deps.busy()) { arm(60_000); return }
    running = true
    try {
      const plan = await deps.plan()
      for (const target of plan.targets) {
        if (disposed || !deps.visible() || deps.busy()) break
        try { await deps.check(target) } catch { deps.onError() }
      }
      if (!disposed) {
        // Network outcomes changed durable deadlines; calculate the next single wake.
        const next = plan.targets.length ? await deps.plan() : plan
        arm(next.nextDelayMs)
      }
    } catch {
      // Corrupt/unavailable state must not produce a tight query loop.
      deps.onError(); arm(30 * 60_000)
    } finally {
      running = false
      if (pendingWake && !disposed) { pendingWake = false; arm(100) }
    }
  }
  return {
    start() { if (started || disposed) return; started = true; notBefore = Date.now() + 45_000; arm(45_000) },
    wake() { if (!started || disposed) return; if (running) pendingWake = true; else arm(100) },
    stop() { disposed = true; clear() },
  }
}
