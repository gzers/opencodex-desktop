<script lang="ts">
// Keep opening order rather than DOM order: reopening a sibling makes it the top layer.
const modalLayers: HTMLDialogElement[] = []
let interactionOrigin: HTMLElement | null = null
// WebKit mouse activation does not necessarily focus a button. Track the actual
// trigger as well as keyboard focus, once for the lifetime of the modal host.
if (typeof document !== 'undefined') {
  const remember = (event: Event) => {
    if (!(event.target instanceof Element)) return
    interactionOrigin = event.target.closest<HTMLElement>('button, a[href], input, select, textarea, summary, [tabindex]:not([tabindex="-1"])')
  }
  document.addEventListener('click', remember, true)
  document.addEventListener('focusin', remember, true)
}
</script>
<script setup lang="ts">
import { Comment, Fragment, Text, defineComponent, h, isVNode, nextTick, onBeforeUnmount, onMounted, onUpdated, ref, useId, watch, type VNode } from 'vue'
import UiIconButton from './UiIconButton.vue'

defineOptions({ inheritAttrs: false })
const props = defineProps<{
  open: boolean; title: string; wide?: boolean; busy?: boolean; panelClass?: string
  /** Legacy callers may restrict exit, but the shared X remains visible. */
  hideClose?: boolean; iconClose?: boolean
  beforeClose?: () => boolean | void | Promise<boolean | void>
  initialFocus?: 'content' | 'dialog'
  role?: 'dialog' | 'alertdialog'
}>()
const emit = defineEmits<{ close: [] }>()
const dialog = ref<HTMLDialogElement | null>(null)
const titleId = useId()
const guarding = ref(false)
let origin: HTMLElement | null = null
let disposed = false
let opening = 0
const focusableSelector = 'button:not(:disabled), a[href], input:not(:disabled):not([type="hidden"]), select:not(:disabled), textarea:not(:disabled), summary, [tabindex]:not([tabindex="-1"])'
function focusables() {
  return [...(dialog.value?.querySelectorAll<HTMLElement>(focusableSelector) ?? [])].filter(element =>
    element.tabIndex >= 0 && !element.closest('[hidden], [inert], [aria-hidden="true"]') &&
    !element.closest('details:not([open]) > :not(summary)') &&
    getComputedStyle(element).display !== 'none' && getComputedStyle(element).visibility !== 'hidden')
    .sort((a, b) => (a.compareDocumentPosition(b) & Node.DOCUMENT_POSITION_FOLLOWING) ? -1 : 1)
}
function isTop() { return modalLayers.at(-1) === dialog.value }
function focusInside() {
  const element = dialog.value
  if (!element) return
  const list = focusables()
  const input = list.find(node => node.matches('input:not([type="checkbox"]):not([type="radio"]),textarea,select'))
  const preferred = list.find(node => node.hasAttribute('autofocus')) ?? input ?? list.find(node => node.matches('.primary,.danger')) ?? list[0]
  ;(props.initialFocus === 'dialog' ? element : preferred ?? element).focus()
}
function removeLayer(element: HTMLDialogElement) {
  const index = modalLayers.indexOf(element)
  if (index >= 0) modalLayers.splice(index, 1)
}
function release() {
  const element = dialog.value
  if (!element) return
  const wasTop = isTop()
  removeLayer(element)
  if (element.open) element.close()
  if (wasTop) {
    if (origin?.isConnected) origin.focus()
    else modalLayers.at(-1)?.focus()
  }
  origin = null
}
watch(() => props.open, async open => {
  const current = ++opening
  // Capture before native showModal moves focus, including a pre-open mount.
  if (open) {
    guarding.value = false
    origin = interactionOrigin?.isConnected ? interactionOrigin : document.activeElement instanceof HTMLElement ? document.activeElement : null
  }
  await nextTick()
  if (disposed || current !== opening) return
  const element = dialog.value
  if (!element) return
  if (open && !element.open) {
    element.showModal()
    removeLayer(element)
    modalLayers.push(element)
    focusInside()
  } else if (!open) release()
}, { immediate: true })
function requestClose() {
  if (!props.open || props.busy || props.hideClose || guarding.value) return
  const current = opening
  const finish = (allowed: boolean | void) => {
    if (!disposed && props.open && opening === current && !props.busy && !props.hideClose && allowed !== false) emit('close')
  }
  try {
    const result = props.beforeClose?.()
    if (result instanceof Promise) {
      guarding.value = true
      void result.then(finish).catch(() => {}).finally(() => { if (opening === current) guarding.value = false })
    } else finish(result)
  } catch { /* A failed guard must keep every exit channel closed. */ }
}
function keydown(event: KeyboardEvent) {
  const element = dialog.value
  if (event.defaultPrevented || !props.open || !element?.open || !isTop()) return
  if (event.key === 'Escape') {
    if (element.querySelector('[data-ui-escape-owner]')) return
    if (event.target instanceof Node && !element.contains(event.target)) return
    event.preventDefault()
    event.stopPropagation()
    requestClose()
  } else if (event.key === 'Tab') {
    const list = focusables(), first = list[0], last = list.at(-1)
    const inside = element.contains(document.activeElement)
    if (!first || !inside || document.activeElement === element || (event.shiftKey ? document.activeElement === first : document.activeElement === last)) {
      event.preventDefault()
      ;(event.shiftKey ? last ?? element : first ?? element).focus()
    }
  }
}
function cancel(event: Event) {
  event.preventDefault()
  if (isTop() && !dialog.value?.querySelector('[data-ui-escape-owner]')) requestClose()
}
function backdrop(event: MouseEvent) {
  const box = dialog.value?.getBoundingClientRect()
  if (isTop() && event.target === dialog.value && box && (event.clientX < box.left || event.clientX > box.right || event.clientY < box.top || event.clientY > box.bottom)) requestClose()
}
function hasContent(nodes: VNode[]): boolean {
  return nodes.some(node => {
    if (!isVNode(node) || node.type === Comment) return false
    if (node.type === Text) return String(node.children ?? '').trim().length > 0
    if (node.type === Fragment) return hasContent(node.children as VNode[])
    return true
  })
}
const DialogFooter = defineComponent({
  setup(_, { slots }) {
    return () => {
      const children = slots.default?.() ?? []
      return hasContent(children) ? h('footer', children) : null
    }
  },
})
// Content switches can remove the focused control. Keep focus within the same layer.
onUpdated(() => { if (props.open && isTop() && dialog.value?.open && !dialog.value.contains(document.activeElement)) focusInside() })
onMounted(() => window.addEventListener('keydown', keydown))
onBeforeUnmount(() => { disposed = true; opening++; window.removeEventListener('keydown', keydown); release() })
defineExpose({ element: dialog, requestClose })
</script>
<template>
  <Teleport to="body">
    <dialog ref="dialog" v-bind="$attrs" class="ui-dialog" :class="[panelClass, { wide }]" :aria-labelledby="titleId" :role="role ?? 'dialog'" :aria-modal="open ? 'true' : undefined" tabindex="-1" @cancel="cancel" @click="backdrop">
      <header class="ui-dialog-header"><h2 :id="titleId">{{ title }}</h2><div class="ui-dialog-header-actions"><slot name="header-actions" /><UiIconButton class="modal-close" label="关闭" title="关闭" :disabled="busy || hideClose || guarding" @click="requestClose"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="m6 6 12 12M18 6 6 18" /></svg></UiIconButton></div></header>
      <div class="ui-dialog-content"><slot :close="requestClose" /></div>
      <DialogFooter><slot name="actions" :close="requestClose" /></DialogFooter>
    </dialog>
  </Teleport>
</template>
<style scoped>
/* Keep the standard declaration last: production minification coalesces aliases.
   A WebKit-only result leaves current WebView2 transparent without any blur. */
.ui-dialog { color: var(--text); background: var(--surface-overlay-bg, var(--panel)); border: 1px solid var(--surface-overlay-border, var(--border)); border-radius: var(--radius-lg); padding: 0; width: min(560px, calc(100vw / var(--ui-zoom) - 32px)); max-height: calc(100vh / var(--ui-zoom) - 40px); zoom: var(--ui-zoom); box-shadow: var(--surface-overlay-shadow, var(--shadow)); -webkit-backdrop-filter: var(--surface-overlay-blur); backdrop-filter: var(--surface-overlay-blur); }
.ui-dialog.wide { width: min(850px, calc(100vw / var(--ui-zoom) - 32px)); }
.ui-dialog[open] { display:flex; flex-direction:column; overflow:visible; }
.ui-dialog:focus { outline:none; }
.ui-dialog::backdrop { background: var(--overlay-scrim); }
header, footer { flex-shrink:0; display: flex; align-items: center; justify-content: space-between; gap: var(--space-3); padding: 16px 20px; }
header { border-bottom: 1px solid var(--border); }
header h2 { margin: 0; min-width:0; overflow-wrap:anywhere; font-size: var(--text-title); }
.ui-dialog-header-actions { display:flex; align-items:center; gap:var(--space-2); flex:none; margin-left:auto; }
.modal-close { flex:none; }
footer { justify-content: flex-end; border-top: 1px solid var(--border); }
.ui-dialog-content { padding: 20px; overflow: auto; min-height:0; }
</style>
