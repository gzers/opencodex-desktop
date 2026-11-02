/**
 * 通知自动收起倒计时（S18-09，原型 notification-countdown.js）。
 *
 * 到点时刻由 setTimeout 决定，测试里的假计时器可以直接推进；暂停时清掉计时器，
 * 恢复时按剩余时长重新挂上。暂停原因用集合登记：悬停、焦点、页面隐藏、确认框
 * 等多个来源同时存在时，全部解除才继续走。
 */
export type CountdownTone = 'success' | 'error' | 'warning' | 'info' | 'neutral'
export type CountdownPhase = 'running' | 'paused' | 'elapsed' | 'cancelled'
export interface CountdownState { durationMs: number; remainingMs: number; ratio: number; phase: CountdownPhase }
export interface Countdown {
  readonly durationMs: number
  readonly tone: CountdownTone
  pause(reason: string): void
  resume(reason: string): void
  cancel(): void
  state(): CountdownState
  subscribe(listener: (state: CountdownState) => void): () => void
}
export interface CountdownOptions { durationMs: number; tone: CountdownTone; onElapsed: () => void; now?: () => number }

function defaultNow(): number {
  return typeof performance !== 'undefined' && typeof performance.now === 'function' ? performance.now() : Date.now()
}

export function createCountdown(options: CountdownOptions): Countdown {
  const now = options.now ?? defaultNow
  const durationMs = Math.max(0, options.durationMs)
  const reasons = new Set<string>()
  const listeners = new Set<(state: CountdownState) => void>()
  let remainingMs = durationMs
  let startedAt = 0
  let timer: ReturnType<typeof setTimeout> | undefined
  let phase: CountdownPhase = 'running'

  const remaining = () => (phase === 'running' && timer !== undefined ? Math.max(0, remainingMs - (now() - startedAt)) : remainingMs)
  const state = (): CountdownState => {
    const left = phase === 'elapsed' ? 0 : remaining()
    return { durationMs, remainingMs: left, ratio: durationMs > 0 ? left / durationMs : 0, phase }
  }
  const notify = () => { const snapshot = state(); for (const listener of [...listeners]) listener(snapshot) }
  const disarm = () => { if (timer !== undefined) { clearTimeout(timer); timer = undefined } }
  const elapse = () => {
    timer = undefined
    if (phase !== 'running') return
    remainingMs = 0
    phase = 'elapsed'
    notify()
    options.onElapsed()
  }
  const arm = () => {
    disarm()
    startedAt = now()
    timer = setTimeout(elapse, remainingMs)
  }

  arm()
  return {
    durationMs,
    tone: options.tone,
    pause(reason) {
      if (phase === 'elapsed' || phase === 'cancelled') return
      reasons.add(reason)
      if (phase === 'paused') return
      remainingMs = remaining()
      disarm()
      phase = 'paused'
      notify()
    },
    resume(reason) {
      if (phase !== 'paused') { reasons.delete(reason); return }
      reasons.delete(reason)
      if (reasons.size > 0) return
      phase = 'running'
      arm()
      notify()
    },
    cancel() {
      if (phase === 'elapsed' || phase === 'cancelled') return
      remainingMs = remaining()
      disarm()
      reasons.clear()
      phase = 'cancelled'
      notify()
    },
    state,
    subscribe(listener) { listeners.add(listener); return () => { listeners.delete(listener) } },
  }
}
