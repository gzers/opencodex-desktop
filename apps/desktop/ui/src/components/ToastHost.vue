<script setup lang="ts">
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

<template>
  <div v-if="app.toast" class="notif-host" aria-live="polite">
    <div class="toast show" role="status">
      <span class="toast-txt">{{ app.toast }}</span>
      <button
        class="notify-min"
        type="button"
        title="收起到铃铛"
        aria-label="收起这条操作反馈到铃铛"
        @click="collapse"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M18 9A6 6 0 0 0 6 9c0 6-2.5 7-2.5 7h17S18 15 18 9Z"/><path d="M10.3 20a2 2 0 0 0 3.4 0"/></svg>
      </button>
    </div>
  </div>
</template>
