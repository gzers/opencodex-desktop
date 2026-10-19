<script setup lang="ts">
import UiButton from '@/components/ui/UiButton.vue'
import type {ModalContentSpec} from './modalContent'
defineProps<{modal:ModalContentSpec;titleId:string;values:Record<string,string>}>()
const emit=defineEmits<{confirm:[];cancel:[];resolve:[];delete:[];field:[key:string,value:string]}>()
</script>
<template>
 <h3 :id="titleId">{{ modal.title }}</h3><div class="modal-body" v-html="modal.body"></div>
 <div v-if="modal.fields?.length" class="modal-fields"><label v-for="field in modal.fields" :key="field.key" class="modal-field"><span class="modal-field-label">{{ field.label }}</span><input class="input" :type="field.type??'text'" :placeholder="field.placeholder" :aria-label="field.label" :value="values[field.key]??''" autocapitalize="off" autocorrect="off" spellcheck="false" @input="emit('field',field.key,($event.target as HTMLInputElement).value)" @keydown.enter.prevent="emit('confirm')" /></label></div>
 <div class="modal-actions"><UiButton v-if="modal.resolveLabel" variant="ghost" @click="emit('resolve')">{{ modal.resolveLabel }}</UiButton><UiButton v-if="modal.deleteLabel" variant="ghost" @click="emit('delete')">{{ modal.deleteLabel }}</UiButton><UiButton variant="ghost" @click="emit('cancel')">{{ modal.cancelLabel }}</UiButton><UiButton v-if="modal.confirmLabel" :variant="modal.primaryKind==='danger'?'danger':'primary'" @click="emit('confirm')">{{ modal.confirmLabel }}</UiButton></div>
</template>
