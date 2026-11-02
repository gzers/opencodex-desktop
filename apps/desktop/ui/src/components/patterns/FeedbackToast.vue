<script setup lang="ts">
import UiButton from '@/components/ui/UiButton.vue'
import UiIconButton from '@/components/ui/UiIconButton.vue'
import NotificationCountdown from '@/components/patterns/NotificationCountdown.vue'
import type { Countdown } from '@/app/feedback/countdown'
defineProps<{message:string;notificationKey?:string;countdown?:Countdown|null;actionLabel?:string}>()
defineEmits<{collapse:[];action:[]}>()
</script>
<template><div class="toast show" role="status" :data-notification-key="notificationKey"><span class="toast-txt">{{ message }}</span><UiButton v-if="actionLabel" size="sm" variant="ghost" @click="$emit('action')">{{ actionLabel }}</UiButton><UiIconButton size="notification" class="notify-min" label="收起这条操作反馈到铃铛" @click="$emit('collapse')"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M18 9A6 6 0 0 0 6 9c0 6-2.5 7-2.5 7h17S18 15 18 9Z"/><path d="M10.3 20a2 2 0 0 0 3.4 0"/></svg></UiIconButton><NotificationCountdown v-if="countdown" :countdown="countdown" /></div></template>
