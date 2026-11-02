/**
 * 通知收起到铃铛的动效（S18-09，原型 notification-motion.js 与 index.html pulseBell）。
 *
 * 只操作 DOM，不引用任何 store：调用方传入「真正移除」的提交函数，这里负责飞向铃铛、
 * 余下卡片 FLIP 归位、铃铛脉冲和焦点转移。没有 DOM、找不到元素、尺寸为零或用户
 * 要求减少动效时直接同步提交，保证状态与界面不因动效失败而卡住。
 */
import { nextTick } from 'vue'

export const COLLAPSE_MS = 420
export const REFLOW_MS = 240
export const BELL_PULSE_MS = 560
const PULSE_CLASS = 'collapse-pulse'

interface Flight { el: HTMLElement; animation?: Animation; fallback?: ReturnType<typeof setTimeout>; commits: Array<() => void>; hadFocus: boolean; done: boolean }
const flights = new Map<string, Flight>()
let pulseTimer: ReturnType<typeof setTimeout> | undefined

function hasDom(): boolean { return typeof document !== 'undefined' && typeof window !== 'undefined' }
function reducedMotion(): boolean {
  return typeof window.matchMedia === 'function' && window.matchMedia('(prefers-reduced-motion: reduce)').matches
}
function bell(): HTMLElement | null { return document.querySelector<HTMLElement>('[data-notification-bell]') }
function findSurface(key: string): HTMLElement | null {
  for (const el of document.querySelectorAll<HTMLElement>('[data-notification-key]')) if (el.dataset.notificationKey === key) return el
  return null
}
function clamp(value: number, min: number, max: number): number { return Math.min(max, Math.max(min, value)) }

/** 每次收起都重新触发铃铛脉冲：先摘类、强制回流再挂上，连续收起也会再播一次。 */
export function pulseBell(): void {
  if (!hasDom()) return
  const target = bell()
  if (!target) return
  target.classList.remove(PULSE_CLASS)
  void target.offsetWidth
  target.classList.add(PULSE_CLASS)
  if (pulseTimer !== undefined) clearTimeout(pulseTimer)
  pulseTimer = setTimeout(() => { pulseTimer = undefined; target.classList.remove(PULSE_CLASS) }, BELL_PULSE_MS)
}

export function isLeaving(key: string): boolean { return flights.has(key) }

function restore(el: HTMLElement) {
  el.style.pointerEvents = ''
  delete el.dataset.notificationMotion
}

function surfaceRects(): Map<HTMLElement, DOMRect> {
  const rects = new Map<HTMLElement, DOMRect>()
  for (const el of document.querySelectorAll<HTMLElement>('.notif-layer [data-notification-key]')) if (!el.dataset.notificationMotion) rects.set(el, el.getBoundingClientRect())
  return rects
}

function reflow(before: Map<HTMLElement, DOMRect>) {
  if (reducedMotion()) return
  for (const [el, rect] of before) {
    if (!el.isConnected || typeof el.animate !== 'function') continue
    const dy = rect.top - el.getBoundingClientRect().top
    if (Math.abs(dy) < 1) continue
    el.animate([{ transform: 'translateY(' + dy + 'px)' }, { transform: 'translateY(0)' }], { duration: REFLOW_MS, easing: 'cubic-bezier(.2,.8,.2,1)' })
  }
}

function finish(key: string, flight: Flight) {
  if (flight.done) return
  flight.done = true
  if (flight.fallback !== undefined) clearTimeout(flight.fallback)
  flights.delete(key)
  const before = surfaceRects()
  for (const commit of flight.commits) commit()
  void nextTick(() => {
    reflow(before)
    pulseBell()
    if (flight.el.isConnected) { flight.animation?.cancel(); restore(flight.el) }
    else if (flight.hadFocus) {
      const active = document.activeElement
      if (!active || active === document.body) bell()?.focus({ preventScroll: true })
    }
  })
}

/**
 * 把通知面收起到铃铛，结束后执行 commit。同一 key 正在飞行时再次调用只追加提交，
 * 提交函数各自只执行一次。
 */
export function leave(key: string, commit: () => void): void {
  const running = flights.get(key)
  if (running) { running.commits.push(commit); return }
  if (!hasDom()) { commit(); pulseBell(); return }
  const el = findSurface(key)
  const target = bell()
  if (!el || !target || reducedMotion()) { commit(); pulseBell(); return }
  const rect = el.getBoundingClientRect()
  const bellRect = target.getBoundingClientRect()
  if (rect.width <= 0 || rect.height <= 0) { commit(); pulseBell(); return }

  const dx = bellRect.left + bellRect.width / 2 - (rect.left + rect.width / 2)
  const dy = bellRect.top + bellRect.height / 2 - (rect.top + rect.height / 2)
  const sx = clamp((bellRect.width / rect.width) * 0.42, 0.025, 0.12)
  const sy = clamp((bellRect.height / rect.height) * 0.42, 0.025, 0.18)
  const flight: Flight = { el, commits: [commit], hadFocus: el.contains(document.activeElement), done: false }
  flights.set(key, flight)
  el.style.pointerEvents = 'none'
  el.dataset.notificationMotion = 'leaving'

  const animate = typeof el.animate === 'function'
  if (animate) {
    flight.animation = el.animate([
      { offset: 0, transform: 'translate(0,0) scale(1)', clipPath: 'polygon(0 0,100% 0,100% 100%,0 100%)', opacity: 1 },
      { offset: 0.3, transform: 'translate(' + dx * 0.12 + 'px,' + dy * 0.12 + 'px) scale(.9,.96)', clipPath: 'polygon(12% 0,100% 0,92% 100%,0 100%)', opacity: 0.95 },
      { offset: 0.72, transform: 'translate(' + dx * 0.7 + 'px,' + dy * 0.65 + 'px) scale(.32,.55)', clipPath: 'polygon(34% 0,100% 0,76% 100%,0 100%)', opacity: 0.65 },
      { offset: 1, transform: 'translate(' + dx + 'px,' + dy + 'px) scale(' + sx + ',' + sy + ')', clipPath: 'polygon(50% 0,100% 0,75% 100%,25% 100%)', opacity: 0 },
    ], { duration: COLLAPSE_MS, easing: 'cubic-bezier(.4,0,.2,1)', fill: 'forwards' })
    flight.animation.addEventListener('finish', () => finish(key, flight))
  }
  flight.fallback = setTimeout(() => finish(key, flight), animate ? 500 : COLLAPSE_MS)
}

/** 通知被新内容替换或被手动清除时，停止飞行并丢弃尚未执行的提交。 */
export function cancelLeave(key: string): void {
  const flight = flights.get(key)
  if (!flight) return
  flight.done = true
  flights.delete(key)
  if (flight.fallback !== undefined) clearTimeout(flight.fallback)
  flight.animation?.cancel()
  restore(flight.el)
}
