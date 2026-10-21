<script setup lang="ts">
import UiButton from '@/components/ui/UiButton.vue'
withDefaults(defineProps<{ label: string; value: string; interactive?: boolean; disabled?: boolean }>(), { interactive: false, disabled: false })
defineEmits<{ activate: [] }>()
</script>
<template>
 <UiButton v-if="interactive" class="info-row" surface="plain" variant="ghost" size="sm" :disabled="disabled" @click="$emit('activate')"><span class="info-row-label">{{ label }}</span><span class="info-row-value">{{ value }}</span><span class="info-row-chevron" aria-hidden="true">›</span></UiButton>
 <div v-else class="info-row"><span class="info-row-label">{{ label }}</span><span class="info-row-value">{{ value }}</span></div>
</template>
