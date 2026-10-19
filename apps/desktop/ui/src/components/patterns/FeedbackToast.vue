<script setup lang="ts">
import UiIconButton from '@/components/ui/UiIconButton.vue'
defineProps<{message:string}>()
defineEmits<{collapse:[]}>()
</script>
<template><div class="toast show" role="status"><span class="toast-txt">{{ message }}</span><UiIconButton size="notification" class="notify-min" label="收起这条操作反馈到铃铛" @click="$emit('collapse')"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M18 9A6 6 0 0 0 6 9c0 6-2.5 7-2.5 7h17S18 15 18 9Z"/><path d="M10.3 20a2 2 0 0 0 3.4 0"/></svg></UiIconButton></div></template>
