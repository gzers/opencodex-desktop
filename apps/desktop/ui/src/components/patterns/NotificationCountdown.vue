<script setup lang="ts">
/**
 * 通知自动收起倒计时条（S18-09，原型 notification-countdown.js）。
 *
 * 以父元素为宿主：挂上 has-notification-countdown、aria-describedby 与暂停监听。
 * 计时由传入的 Countdown 负责，这里只画进度、登记暂停原因；卸载时撤销自己加的一切。
 */
import { computed, onBeforeUnmount, onMounted, ref, useId, watch } from 'vue'
import type { Countdown } from '@/app/feedback/countdown'

const props = defineProps<{ countdown: Countdown }>()
const helpId = 'notificationCountdownHelp-' + useId()
const track = ref<HTMLElement | null>(null)
const fill = ref<HTMLElement | null>(null)
const seconds = computed(() => Math.round(props.countdown.durationMs / 1000))

let host: HTMLElement | null = null
let frame: number | null = null
let lastRatio: number | null = null
let unsubscribe: (() => void) | undefined
const added = new Set<string>()
const offs: Array<() => void> = []

function reduced(): boolean {
  return typeof window.matchMedia === 'function' && window.matchMedia('(prefers-reduced-motion: reduce)').matches
}
function stopFrame() {
  if (frame !== null && typeof cancelAnimationFrame === 'function') cancelAnimationFrame(frame)
  frame = null
}
function paint() {
  frame = null
  const state = props.countdown.state()
  const ratio = reduced() ? Math.min(1, Math.ceil(state.remainingMs / 1000) * 1000 / state.durationMs) : state.ratio
  if (fill.value && lastRatio !== ratio) { fill.value.style.transform = 'scaleX(' + ratio + ')'; lastRatio = ratio }
  if (track.value) track.value.dataset.phase = state.phase
  if (state.phase === 'running' && typeof requestAnimationFrame === 'function') frame = requestAnimationFrame(paint)
}
function repaint() { stopFrame(); paint() }
function pause(reason: string) { added.add(reason); props.countdown.pause(reason) }
function resume(reason: string) { added.delete(reason); props.countdown.resume(reason) }
// 宿主不变而倒计时换新（同卡复用、新一轮成功）时，把当前暂停原因转交给新倒计时。
watch(() => props.countdown, (next, previous) => {
  unsubscribe?.()
  for (const reason of added) { previous.resume(reason); next.pause(reason) }
  lastRatio = null
  unsubscribe = next.subscribe(repaint)
  repaint()
})
function listen(target: EventTarget, type: string, fn: (event: Event) => void) {
  target.addEventListener(type, fn)
  offs.push(() => target.removeEventListener(type, fn))
}

onMounted(() => {
  host = track.value?.parentElement ?? null
  if (host) {
    host.classList.add('has-notification-countdown')
    const describedBy = (host.getAttribute('aria-describedby') ?? '').split(/\s+/).filter(Boolean)
    host.setAttribute('aria-describedby', [...describedBy, helpId].join(' '))
    listen(host, 'pointerenter', () => pause('pointer'))
    listen(host, 'pointerleave', () => resume('pointer'))
    listen(host, 'focusin', () => pause('focus'))
    listen(host, 'focusout', event => {
      const next = (event as FocusEvent).relatedTarget
      if (!(next instanceof Node) || !host?.contains(next)) resume('focus')
    })
    if (host.contains(document.activeElement)) pause('focus')
    try { if (host.matches(':hover')) pause('pointer') } catch { /* 旧环境不支持 :hover 查询时按未悬停处理。 */ }
  }
  listen(document, 'visibilitychange', () => (document.hidden ? pause('hidden') : resume('hidden')))
  if (document.hidden) pause('hidden')
  unsubscribe = props.countdown.subscribe(repaint)
  repaint()
})

onBeforeUnmount(() => {
  stopFrame()
  unsubscribe?.()
  for (const off of offs.splice(0)) off()
  for (const reason of [...added]) resume(reason)
  if (host) {
    host.classList.remove('has-notification-countdown')
    const rest = (host.getAttribute('aria-describedby') ?? '').split(/\s+/).filter(id => id && id !== helpId)
    if (rest.length) host.setAttribute('aria-describedby', rest.join(' '))
    else host.removeAttribute('aria-describedby')
  }
  host = null
})
</script>

<template>
  <div ref="track" class="notification-countdown" :data-tone="countdown.tone" aria-hidden="true"><span ref="fill" class="notification-countdown-fill"></span></div>
  <span :id="helpId" class="notification-countdown-help">本通知将在{{ seconds }}秒后收起；鼠标悬停或键盘焦点停留时暂停。</span>
</template>
