<script setup lang="ts">
import FeedbackToast from '@/components/patterns/FeedbackToast.vue'
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'

const app = useAppStore()
const routes = useRouteStore()

// 收起只隐藏 Toast，不产生已读 / 已解决副作用。
function collapse() {
  app.clearToast()
  routes.requestNotificationAttention()
}
</script>

<template><div v-if="app.toast" class="notif-host" aria-live="polite"><FeedbackToast :message="app.toast" @collapse="collapse" /></div></template>
