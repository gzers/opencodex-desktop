<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import ModalContent from '@/components/patterns/ModalContent.vue'
import { useAppStore } from '@/stores/app'
import { useModalStore } from '@/app/modal/store'

const app = useAppStore()
const modalStore = useModalStore()
const modal = computed(() => app.modal)
const inputValues = ref<Record<string, string>>({})
const content = ref<InstanceType<typeof UiDialog> | null>(null)
function cancel() { app.resolveModal('cancel') }
function confirm() { app.resolveModal('confirm', { ...inputValues.value }) }
watch(modal, async value => {
  inputValues.value = Object.fromEntries((value?.fields ?? []).map(field => [field.key, field.value ?? '']))
  if (!value) return
  await nextTick()
  if (modal.value !== value) return
  const input = content.value?.element?.querySelector<HTMLInputElement>('.modal-field input')
  input?.focus()
  input?.select()
}, { immediate: true })
</script>
<template>
  <UiDialog ref="content" :open="!!modal" :title="modal?.title ?? ''" :wide="modal?.wide" :busy="modal?.busy || modalStore.closing" panel-class="app-modal-dialog" @close="cancel">
    <ModalContent v-if="modal" :modal="modal" :values="inputValues" :busy="modalStore.closing" @field="(key, value) => inputValues[key] = value" @confirm="confirm" @cancel="cancel" @resolve="app.resolveModal('resolve')" @delete="app.resolveModal('delete')" />
  </UiDialog>
</template>
