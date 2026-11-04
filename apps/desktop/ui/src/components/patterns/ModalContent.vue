<script setup lang="ts">
import UiButton from '@/components/ui/UiButton.vue'
import { modalCancelLabel, modalHasActions, modalMode, type ModalContentSpec } from './modalContent'
// The dialog host owns the single title bar and X; this view owns body and business actions.
defineProps<{ modal: ModalContentSpec; values: Record<string, string>; titleId?: string; busy?: boolean }>()
const emit = defineEmits<{ confirm: []; cancel: []; resolve: []; delete: []; field: [key: string, value: string] }>()
</script>
<template>
  <div class="modal-body" v-html="modal.body"></div>
  <div v-if="modal.fields?.length" class="modal-fields">
    <label v-for="field in modal.fields" :key="field.key" class="modal-field">
      <span class="modal-field-label">{{ field.label }}</span>
      <input class="input" :type="field.type ?? 'text'" :placeholder="field.placeholder" :aria-label="field.label" :value="values[field.key] ?? ''" :disabled="busy || modal.busy" autocapitalize="off" autocorrect="off" spellcheck="false" @input="emit('field', field.key, ($event.target as HTMLInputElement).value)" @keydown.enter.prevent="!busy && !modal.busy && modal.confirmLabel && emit('confirm')" />
    </label>
  </div>
  <div v-if="modalHasActions(modal)" class="modal-actions">
    <UiButton v-if="modal.resolveLabel" variant="ghost" :disabled="busy || modal.busy" @click="emit('resolve')">{{ modal.resolveLabel }}</UiButton>
    <UiButton v-if="modal.deleteLabel" variant="ghost" :disabled="busy || modal.busy" @click="emit('delete')">{{ modal.deleteLabel }}</UiButton>
    <UiButton v-if="modalMode(modal) !== 'view'" variant="ghost" :disabled="busy || modal.busy" @click="emit('cancel')">{{ modalCancelLabel(modal) }}</UiButton>
    <UiButton v-if="modal.confirmLabel" :variant="modal.primaryKind === 'danger' ? 'danger' : 'primary'" :disabled="busy || modal.busy" @click="emit('confirm')">{{ modal.confirmLabel }}</UiButton>
  </div>
</template>
<style scoped>
.modal-body :deep(p) { color:var(--muted); font-size:var(--text-body); }
</style>
