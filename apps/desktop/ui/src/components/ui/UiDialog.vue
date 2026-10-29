<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, useId, watch } from 'vue'
import UiButton from './UiButton.vue'
const props = defineProps<{ open: boolean; title: string; wide?: boolean; busy?: boolean; panelClass?: string; hideClose?: boolean }>()
const emit = defineEmits<{ close: [] }>()
const dialog = ref<HTMLDialogElement | null>(null)
const titleId = useId()
let origin: HTMLElement | null = null
let disposed=false
watch(() => props.open, async open => {
  await nextTick()
  if(disposed||props.open!==open)return
  const element = dialog.value
  if (!element) return
  if (open && !element.open) { origin = document.activeElement instanceof HTMLElement ? document.activeElement : null; element.showModal() }
  else if (!open && element.open) { element.close(); origin?.focus();origin=null }
}, { immediate: true })
function close() { if (!props.busy) emit('close') }
function backdrop(event: MouseEvent) {
  const box = dialog.value?.getBoundingClientRect()
  if (event.target === dialog.value && box && (event.clientX < box.left || event.clientX > box.right || event.clientY < box.top || event.clientY > box.bottom)) close()
}
onBeforeUnmount(() => {disposed=true;if(dialog.value?.open){dialog.value.close();origin?.focus()}origin=null})
</script>
<template>
  <Teleport to="body">
    <dialog ref="dialog" class="ui-dialog" :class="[panelClass, { wide }]" :aria-labelledby="titleId" @cancel.prevent="close" @click="backdrop">
      <header><h2 :id="titleId">{{ title }}</h2><UiButton v-if="!hideClose" variant="ghost" :disabled="busy" @click="close">关闭</UiButton></header>
      <div class="ui-dialog-content"><slot /></div>
      <footer v-if="$slots.actions"><slot name="actions" /></footer>
    </dialog>
  </Teleport>
</template>
<style scoped>
.ui-dialog { color: var(--text); background: var(--surface-overlay-bg, var(--panel)); border: 1px solid var(--surface-overlay-border, var(--border)); border-radius: var(--radius-lg); padding: 0; width: min(560px, calc(100vw / var(--ui-zoom) - 32px)); max-height: calc(100vh / var(--ui-zoom) - 40px); zoom: var(--ui-zoom); box-shadow: var(--surface-overlay-shadow, var(--shadow)); backdrop-filter: var(--surface-overlay-blur); -webkit-backdrop-filter: var(--surface-overlay-blur); }
.ui-dialog.wide { width: min(850px, calc(100vw / var(--ui-zoom) - 32px)); }
.ui-dialog[open] { display:flex; flex-direction:column; }
.ui-dialog::backdrop { background: var(--overlay-scrim); }
header, footer { flex-shrink:0; display: flex; align-items: center; justify-content: space-between; gap: var(--space-3); padding: 16px 20px; }
header { border-bottom: 1px solid var(--border); }
header h2 { margin: 0; font-size: var(--text-title); }
footer { justify-content: flex-end; border-top: 1px solid var(--border); }
.ui-dialog-content { padding: 20px; overflow: auto; min-height:0; }
</style>
