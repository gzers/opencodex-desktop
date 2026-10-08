<script setup lang="ts">
// 菜单移出父卡片的 backdrop root，真实模糊其背后的兄弟内容。
// 仍挂在缩放后的主壳里，定位采用 CSS zoom 的逻辑坐标。
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

defineOptions({ inheritAttrs: false })
const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: [] }>()
const anchor = ref<HTMLElement | null>(null)
const menu = ref<HTMLElement | null>(null)
const target = ref<HTMLElement | string>('body')
const position = ref({ left: '0px', top: '0px', minWidth: '0px', maxWidth: '320px', maxHeight: '260px' })
const trigger = () => anchor.value?.closest('.select')?.querySelector<HTMLButtonElement>('.select-trigger')
const options = () => [...(menu.value?.querySelectorAll<HTMLButtonElement>('[role="option"]:not(:disabled)') ?? [])]
const visible = computed(() => props.open)
let observer: ResizeObserver | undefined

function place() {
  const button = trigger()
  if (!button || !menu.value) return
  const zoom = Number(getComputedStyle(button.closest('.app-window') ?? document.documentElement).zoom) || 1
  const bounds = button.getBoundingClientRect()
  const viewportWidth = window.innerWidth / zoom
  const viewportHeight = window.innerHeight / zoom
  const width = Math.min(Math.max(bounds.width / zoom, menu.value.scrollWidth), viewportWidth - 24)
  const height = Math.min(menu.value.scrollHeight, 260)
  const below = viewportHeight - bounds.bottom / zoom - 18
  const above = bounds.top / zoom - 18
  const upwards = below < height && above > below
  const available = Math.max(30, upwards ? above : below)
  const shownHeight = Math.min(height, available)
  position.value = {
    left: `${Math.max(12, Math.min(bounds.right / zoom - width, viewportWidth - width - 12))}px`,
    top: `${upwards ? Math.max(12, bounds.top / zoom - shownHeight - 6) : bounds.bottom / zoom + 6}px`,
    minWidth: `${Math.min(bounds.width / zoom, viewportWidth - 24)}px`,
    maxWidth: `${viewportWidth - 24}px`,
    maxHeight: `${Math.min(260, available)}px`,
  }
}
function close(restore = false) {
  emit('close')
  if (restore) trigger()?.focus()
}
function outside(event: PointerEvent) {
  if (!visible.value) return
  const node = event.target as Node
  if (!menu.value?.contains(node) && !anchor.value?.closest('.select')?.contains(node)) close()
}
function keydown(event: KeyboardEvent) {
  const button = trigger()
  const inside = menu.value?.contains(event.target as Node)
  if (!inside && event.target !== button) return
  if (!visible.value) {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault()
      button?.click()
      void nextTick(() => (event.key === 'ArrowUp' ? options().at(-1) : options()[0])?.focus())
    }
    return
  }
  const items = options()
  const index = items.indexOf(document.activeElement as HTMLButtonElement)
  if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
    event.preventDefault()
    const next = event.key === 'Home' ? 0 : event.key === 'End' ? items.length - 1
      : (index + (event.key === 'ArrowDown' ? 1 : -1) + items.length) % items.length
    items[next]?.focus()
  } else if (event.key === 'Escape') {
    event.preventDefault()
    event.stopPropagation()
    close(true)
  } else if (event.key === 'Tab') {
    if (inside) {
      event.preventDefault()
      const candidates = [...document.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled), a[href], [tabindex="0"]')]
        .filter(node => !menu.value?.contains(node) && node.getClientRects().length > 0)
      const index = candidates.indexOf(button!)
      const next = candidates[index + (event.shiftKey ? -1 : 1)]
      close()
      ;(next ?? button)?.focus()
    } else close()
  }
}
watch(visible, async open => {
  await nextTick()
  if (open) {
    place()
    const button = trigger()
    button?.setAttribute('aria-controls', menu.value!.id)
    observer?.disconnect()
    if (typeof ResizeObserver !== 'undefined') {
      observer = new ResizeObserver(place)
      if (button) observer.observe(button)
    }
  } else {
    observer?.disconnect()
    trigger()?.removeAttribute('aria-controls')
  }
})
onMounted(() => {
  target.value = anchor.value?.closest('.app-window') as HTMLElement ?? document.body
  window.addEventListener('pointerdown', outside)
  window.addEventListener('keydown', keydown)
  window.addEventListener('resize', place)
  window.addEventListener('scroll', place, true)
})
onBeforeUnmount(() => {
  observer?.disconnect()
  window.removeEventListener('pointerdown', outside)
  window.removeEventListener('keydown', keydown)
  window.removeEventListener('resize', place)
  window.removeEventListener('scroll', place, true)
})
const id = `select-menu-${Math.random().toString(36).slice(2)}`
</script>

<template>
  <span ref="anchor" hidden></span>
  <Teleport :to="target">
    <div v-if="visible" :id="id" ref="menu" v-bind="$attrs" class="select-menu select-menu-portal"
      :style="position" @click="close(true)">
      <slot />
    </div>
  </Teleport>
</template>
