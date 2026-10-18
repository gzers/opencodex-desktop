<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
const bar = ref<HTMLElement | null>(null)
let observer: ResizeObserver | undefined
let parent: HTMLElement | null = null
function measure() {
  if (!bar.value || !parent) return
  // offsetHeight is in layout units, independent of the app's CSS zoom.
  parent.style.setProperty('--page-tabs-height', `${bar.value.offsetHeight}px`)
}
onMounted(() => {
  parent = bar.value?.parentElement ?? null
  if (typeof ResizeObserver !== 'undefined' && bar.value) { observer = new ResizeObserver(measure); observer.observe(bar.value) }
  measure()
})
onBeforeUnmount(() => { observer?.disconnect(); parent?.style.removeProperty('--page-tabs-height') })
</script>
<template><div ref="bar" class="sticky-page-tabs"><slot /></div></template>
<style scoped>
.sticky-page-tabs { position: sticky; top: 0; z-index: var(--z-sticky); background: var(--surface-page-bg, var(--panel)); backdrop-filter: var(--surface-panel-blur); -webkit-backdrop-filter: var(--surface-panel-blur); }
.sticky-page-tabs :deep([role=tablist]) { margin-top: 0; margin-bottom: 0; }
</style>
