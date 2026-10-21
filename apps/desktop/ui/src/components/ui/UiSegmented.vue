<script setup lang="ts">
import {computed} from 'vue'
const props = defineProps<{ modelValue: string; label: string; options: { value: string; label: string; disabled?: boolean }[]; disabled?: boolean }>()
const tabValue=computed(()=>props.options.find(o=>o.value===props.modelValue&&!o.disabled)?.value??props.options.find(o=>!o.disabled)?.value)
const emit = defineEmits<{ 'update:modelValue': [string] }>()
function choose(value: string) { if (!props.disabled) emit('update:modelValue', value) }
function move(event: KeyboardEvent, value: string) {
 if (!['ArrowLeft','ArrowRight','ArrowUp','ArrowDown','Home','End'].includes(event.key) || props.disabled) return
 const choices = props.options.filter(o => !o.disabled); if (!choices.length) return
 const index = choices.findIndex(o => o.value === value)
 const next = event.key === 'Home' ? 0 : event.key === 'End' ? choices.length - 1 : (index + (['ArrowRight','ArrowDown'].includes(event.key) ? 1 : -1) + choices.length) % choices.length
 const option = choices[next]!; event.preventDefault(); choose(option.value)
 const buttons = [...(event.currentTarget as HTMLElement).parentElement!.querySelectorAll<HTMLButtonElement>('button:not(:disabled)')]; buttons[next]?.focus()
}
</script>
<template><div class="ui-segmented" role="radiogroup" :aria-label="label" :aria-disabled="disabled || undefined"><button v-for="option in options" :key="option.value" type="button" role="radio" :aria-checked="modelValue === option.value" :tabindex="!disabled && tabValue===option.value ? 0 : -1" :disabled="disabled || option.disabled" @click="choose(option.value)" @keydown="move($event, option.value)">{{ option.label }}</button></div></template>
