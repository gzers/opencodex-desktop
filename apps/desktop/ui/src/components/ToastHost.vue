<script setup lang="ts">
import FeedbackToast from '@/components/patterns/FeedbackToast.vue'
import { currentToastCountdown, toastKey, useFeedbackStore } from '@/app/feedback/store'

const feedback = useFeedbackStore()

// 收起只隐藏 Toast，不产生已读 / 已解决副作用；动效结束后由 motion 触发铃铛脉冲。
function collapse() {
  feedback.collapseToast()
}
</script>

<template><div v-if="feedback.toast" class="notif-host" aria-live="polite"><FeedbackToast :key="feedback.toastRevision" :message="feedback.toast" :notification-key="toastKey(feedback.toastRevision)" :countdown="currentToastCountdown()" @collapse="collapse" /></div></template>
