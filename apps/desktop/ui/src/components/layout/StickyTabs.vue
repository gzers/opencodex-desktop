<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
const bar = ref<HTMLElement | null>(null)
let observer: ResizeObserver | undefined
let parent: HTMLElement | null = null
let main: HTMLElement | null = null
let stuck = false
function measure() {
  if (!bar.value || !parent) return
  // Layout units stay independent of CSS zoom. Compensate the scrollport padding
  // while preserving the original alignment of the controls inside the full-width surface.
  const style = main ? getComputedStyle(main) : null
  parent.style.setProperty('--page-tabs-inset-top', style?.paddingTop ?? '0px')
  parent.style.setProperty('--page-tabs-inset-left', style?.paddingLeft ?? '0px')
  parent.style.setProperty('--page-tabs-inset-right', style?.paddingRight ?? '0px')
  parent.style.setProperty('--page-tabs-height', (bar.value.offsetHeight || 0) + 'px')
  syncStuck()
}
// 吸附线 = 滚动容器顶边 + sticky top；吸顶后 Tab 正停在该线，判定稳定不抖动。
function syncStuck() {
  if (!bar.value || !main) return
  const offset = Number.parseFloat(getComputedStyle(bar.value).top)
  const pinTop = main.getBoundingClientRect().top + (Number.isNaN(offset) ? 0 : offset)
  const next = bar.value.getBoundingClientRect().top <= pinTop + 1
  if (next === stuck) return
  stuck = next
  bar.value.dataset.stuck = next ? 'true' : 'false'
}
function onScroll() { syncStuck() }
onMounted(() => {
  parent = bar.value?.parentElement ?? null
  main = bar.value?.closest<HTMLElement>('.main') ?? null
  if (typeof ResizeObserver !== 'undefined' && bar.value) { observer = new ResizeObserver(measure); observer.observe(bar.value); if(main)observer.observe(main) }
  main?.addEventListener('scroll', onScroll, { passive: true })
  measure()
})
onBeforeUnmount(() => {
  observer?.disconnect()
  main?.removeEventListener('scroll', onScroll)
  for(const key of ['height','inset-top','inset-left','inset-right'])parent?.style.removeProperty('--page-tabs-'+key)
})
</script>
<template><div ref="bar" class="sticky-page-tabs"><slot /></div></template>
<style scoped>
.sticky-page-tabs { position: sticky; top: calc(-1 * var(--page-tabs-inset-top,0px)); margin-inline-start: calc(-1 * var(--page-tabs-inset-left,0px)); margin-inline-end: calc(-1 * var(--page-tabs-inset-right,0px)); padding-inline-start: var(--page-tabs-inset-left,0px); padding-inline-end: var(--page-tabs-inset-right,0px); z-index: var(--z-sticky); background: transparent; border-bottom: 1px solid transparent; transition: background-color var(--motion-normal,.18s) ease, border-bottom-color var(--motion-normal,.18s) ease; }
/* 未到顶不铺底色，避免静止状态出现一条色带；吸顶后玻璃材质淡入。 */
.sticky-page-tabs[data-stuck="true"] { background: var(--surface-sticky-bg, var(--surface-panel-bg)); backdrop-filter: var(--surface-sticky-blur); -webkit-backdrop-filter: var(--surface-sticky-blur); border-bottom-color: var(--border); }
.sticky-page-tabs :deep([role=tablist]) { margin-top: 0; margin-bottom: 0; border-bottom-color: transparent; }
@supports not ((backdrop-filter:blur(1px)) or (-webkit-backdrop-filter:blur(1px))) { .sticky-page-tabs[data-stuck="true"] { background: var(--panel); } }
</style>

