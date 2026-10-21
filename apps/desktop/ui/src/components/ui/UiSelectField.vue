<script setup lang="ts">
import { computed, ref } from 'vue'
import UiSelectMenu from './UiSelectMenu.vue'
const props = defineProps<{ modelValue: string; options: readonly { value: string; label: string; disabled?: boolean }[]; label: string; disabled?: boolean; size?: 'sm' | 'md' }>()
const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
const open = ref(false)
const selected = computed(() => props.options.find(item => item.value === props.modelValue)?.label ?? props.modelValue)
function choose(value: string) { emit('update:modelValue', value); open.value = false }
</script>
<template>
  <div class="select" :class="{ 'select-sm': size === 'sm' }">
    <button type="button" class="select-trigger" :disabled="disabled" :aria-label="label" aria-haspopup="listbox" :aria-expanded="open" @click="open = !open">
      <span>{{ selected }}</span>
    </button>
    <UiSelectMenu :open="open" role="listbox" :aria-label="label" @close="open = false">
      <button v-for="option in options" :key="option.value" type="button" role="option" class="select-option" :class="{ selected: option.value === modelValue }" :aria-selected="option.value === modelValue" :disabled="option.disabled" @click="choose(option.value)">{{ option.label }}</button>
    </UiSelectMenu>
  </div>
</template>
<style scoped>
.select { min-width: 0; }
.select-trigger { width: 100%; justify-content: space-between; gap: var(--space-2); }
.select-trigger > span:first-child { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.select-sm .select-trigger { min-height: var(--control-sm); padding: 5px 10px; font-size: var(--text-label); line-height: 16px; }
</style>
