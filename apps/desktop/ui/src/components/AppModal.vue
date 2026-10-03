<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, useId, watch } from 'vue'
import { useAppStore } from '@/stores/app'

const app = useAppStore()
const modal = computed(() => app.modal)
const primaryClass = computed(() => modal.value?.primaryKind === 'danger' ? 'btn danger' : 'btn primary')
// 标题 id 用于 aria-labelledby；同一时刻只会渲染一个对话框。
const titleId = `modal-title-${useId()}`

const dialog = ref<HTMLElement | null>(null)
const inputValues = ref<Record<string, string>>({})
let restoreFocus: HTMLElement | null = null

const FOCUSABLE = 'button:not([disabled]):not([hidden]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'

function focusables(): HTMLElement[] {
  const root = dialog.value
  if (!root) return []
  return Array.from(root.querySelectorAll<HTMLElement>(FOCUSABLE))
}

function cancel(): void {
  // Escape 与点击遮罩只允许取消，绝不触发确认。
  app.resolveModal('cancel')
}

// 输入型对话框：确认时把各字段当前取值一并回传（取消不回传）。
function confirm(): void {
  app.resolveModal('confirm', { ...inputValues.value })
}

function onKeydown(event: KeyboardEvent): void {
  if (!modal.value) return
  if (event.key === 'Escape') {
    event.preventDefault()
    cancel()
    return
  }
  if (event.key !== 'Tab') return
  const list = focusables()
  if (list.length === 0) {
    event.preventDefault()
    dialog.value?.focus()
    return
  }
  const first = list[0]
  const last = list[list.length - 1]
  const active = document.activeElement as HTMLElement | null
  const inside = active !== null && dialog.value?.contains(active) === true
  if (event.shiftKey) {
    if (!inside || active === first) {
      event.preventDefault()
      last.focus()
    }
  } else if (!inside || active === last) {
    event.preventDefault()
    first.focus()
  }
}

watch(modal, async (value, previous) => {
  if (value && !previous) {
    inputValues.value = Object.fromEntries(
      (value.fields ?? []).map((field) => [field.key, field.value ?? '']),
    )
    restoreFocus = document.activeElement instanceof HTMLElement ? document.activeElement : null
    await nextTick()
    const firstInput = dialog.value?.querySelector<HTMLInputElement>('.modal-field input')
    if (firstInput) {
      firstInput.focus()
      firstInput.select()
      return
    }
    const list = focusables()
    // 焦点优先给主操作，其次第一个可聚焦控件，最后退回对话框本身。
    const primary = list.find((el) => el.classList.contains('primary') || el.classList.contains('danger'))
    ;(primary ?? list[0] ?? dialog.value)?.focus()
  } else if (!value && previous) {
    inputValues.value = {}
    restoreFocus?.focus()
    restoreFocus = null
  }
})

onMounted(() => window.addEventListener('keydown', onKeydown))
onBeforeUnmount(() => {
  window.removeEventListener('keydown', onKeydown)
  restoreFocus = null
})
</script>

<template>
  <div v-if="modal" class="modal-mask" @click.self="cancel">
    <div
      ref="dialog"
      class="modal"
      :class="{ 'modal-wide': modal.wide }"
      role="dialog"
      aria-modal="true"
      :aria-labelledby="titleId"
      tabindex="-1"
    >
      <h3 :id="titleId">{{ modal.title }}</h3>
      <div class="modal-body" v-html="modal.body"></div>
      <div v-if="modal.fields?.length" class="modal-fields">
        <label v-for="field in modal.fields" :key="field.key" class="modal-field">
          <span class="modal-field-label">{{ field.label }}</span>
          <input
            class="input"
            :type="field.type ?? 'text'"
            :placeholder="field.placeholder"
            :aria-label="field.label"
            v-model="inputValues[field.key]"
            autocapitalize="off"
            autocorrect="off"
            spellcheck="false"
            @keydown.enter.prevent="confirm()"
          />
        </label>
      </div>
      <div class="modal-actions">
        <button
          v-if="modal.resolveLabel"
          class="btn ghost"
          @click="app.resolveModal('resolve')"
        >{{ modal.resolveLabel }}</button>
        <button
          v-if="modal.deleteLabel"
          class="btn ghost"
          @click="app.resolveModal('delete')"
        >{{ modal.deleteLabel }}</button>
        <button class="btn ghost" @click="cancel">{{ modal.cancelLabel }}</button>
        <button v-if="modal.confirmLabel" :class="primaryClass" @click="confirm()">{{ modal.confirmLabel }}</button>
      </div>
    </div>
  </div>
</template>
