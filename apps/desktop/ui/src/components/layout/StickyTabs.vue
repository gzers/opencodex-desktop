<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
const bar = ref<HTMLElement | null>(null)
let observer: ResizeObserver | undefined
let parent: HTMLElement | null = null
let main: HTMLElement | null = null
let stuck = false
let stuckWritten = false
// Tab 在滚动内容里的自然位置（不含滚动量）。用 offsetTop 链累加得到，
// 与当前滚动量和 sticky 偏移无关；先前用视口坐标反推，吸顶后会把自己算成新基准。
function naturalOffset(): number {
  let node: HTMLElement | null = bar.value
  let total = 0
  while (node && node !== main) {
    total += node.offsetTop
    node = node.offsetParent as HTMLElement | null
  }
  return total
}
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
/**
 * 吸顶判定优先用视口坐标：Tab 的 sticky 负偏移会被浏览器夹在滚动口边缘，
 * 因此「贴住」的那一刻它的上沿就等于滚动容器上边框内侧。
 * 早先只比 offsetTop 链与 scrollTop，一旦 offsetParent 跳过 .main（.main 是 static），
 * 累加值就不再是相对滚动位置的偏移，判定永远不成立，玻璃材质也就永远不出现。
 * jsdom 不做布局、所有矩形为 0，此时退回 offsetTop 链，保持既有测试语义。
 */
function syncStuck() {
  if (!bar.value || !main) return
  const barRect = bar.value.getBoundingClientRect?.()
  const mainRect = main.getBoundingClientRect?.()
  const measurable = !!barRect && !!mainRect && (barRect.height > 0 || mainRect.height > 0)
  const next = measurable
    ? barRect!.top <= mainRect!.top + (main.clientTop || 0) + 0.5
    : main.scrollTop + (main.clientTop || 0) >= naturalOffset() - 0.5
  // 初次挂载必须写一次，否则 data-stuck 缺失、样式退回“未吸顶”。
  if (stuckWritten && next === stuck) return
  stuck = next
  stuckWritten = true
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
/* 吸顶材质用 --motion-sticky（90ms）：跟手才有“贴上去就出现”的感觉；常规 180ms 会明显落后于滚动。 */
.sticky-page-tabs { position: sticky; top: calc(-1 * var(--page-tabs-inset-top,0px)); margin-inline-start: calc(-1 * var(--page-tabs-inset-left,0px)); margin-inline-end: calc(-1 * var(--page-tabs-inset-right,0px)); padding-inline-start: var(--page-tabs-inset-left,0px); padding-inline-end: var(--page-tabs-inset-right,0px); z-index: var(--z-sticky); background: transparent; border-bottom: 1px solid transparent; transition: background-color var(--motion-sticky,.09s) ease-out, border-bottom-color var(--motion-sticky,.09s) ease-out; }
/* 未到顶不铺底色，避免静止状态出现一条色带；吸顶后玻璃材质淡入。 */
.sticky-page-tabs[data-stuck="true"] { background: var(--surface-sticky-bg, var(--surface-panel-bg)); backdrop-filter: var(--surface-sticky-blur); -webkit-backdrop-filter: var(--surface-sticky-blur); border-bottom-color: var(--border); }
.sticky-page-tabs :deep([role=tablist]) { margin-top: 0; margin-bottom: 0; border-bottom-color: transparent; }
@supports not ((backdrop-filter:blur(1px)) or (-webkit-backdrop-filter:blur(1px))) { .sticky-page-tabs[data-stuck="true"] { background: var(--panel); } }
</style>

