<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useRouteStore } from '@/stores/routes'
import { useThemeStore } from '@/app/appearance/theme'
import { useAppController } from '@/composables/useAppController'
import { useNotificationsStore } from '@/features/notifications/store'
import type { NotificationItem } from '@/types/ui'

const routes = useRouteStore()
const theme = useThemeStore()
const controller = useAppController()
// `subtitleOverride`：概览按运行/环境准备双形态改写副标题（UI规范 §25.1，原型同款做法）。
defineProps<{ compact?: boolean; subtitleOverride?: string }>()
const notifications = useNotificationsStore()
const panelOpen = computed({ get: () => notifications.panelOpen, set: value => { notifications.panelOpen = value } })
const bellPulsing = computed(() => routes.bellPulse)
// 顶栏全局刷新（IMP-06）：转圈期间禁止重复点击，失败由编排层给 Toast，不改写任何状态。
const refreshing = ref(false)

async function refreshCurrentView() {
  if (refreshing.value) return
  refreshing.value = true
  try {
    await controller.refreshCurrentView()
  } finally {
    refreshing.value = false
  }
}

const themeOptions = [
  { value: 'light', label: '浅色' },
  { value: 'dark', label: '深色' },
  { value: 'system', label: '跟随系统' },
] as const

const pillClass = computed(() => controller.scenario.value.pillClass)
const unread = computed(() => controller.unreadNotifications.value.length)
const hasDanger = computed(() => controller.unreadList.value.some((n: NotificationItem) => !n.read && n.kind === 'danger'))

function closePanel() {
  panelOpen.value = false
  document.removeEventListener('click', onDocumentClick)
}

function onDocumentClick(event: MouseEvent) {
  const target = event.target as HTMLElement
  // 通知中心已移到全局通知层；点它内部不算「外部点击」。
  if (!target.closest('.notification-wrap') && !target.closest('.notif-layer')) closePanel()
}

function togglePanel() {
  panelOpen.value = !panelOpen.value
  if (panelOpen.value) document.addEventListener('click', onDocumentClick)
  else document.removeEventListener('click', onDocumentClick)
}

watch(panelOpen, open => {
  if (!open) document.removeEventListener('click', onDocumentClick)
})

onBeforeUnmount(() => {
  panelOpen.value = false
  document.removeEventListener('click', onDocumentClick)
})
</script>

<template>
  <header class="topbar" :class="{ 'topbar-compact': compact }">
    <div v-if="!compact" class="page-title">
      <h1>{{ routes.meta.title }}</h1>
      <p>{{ subtitleOverride ?? routes.meta.subtitle }}</p>
    </div>
    <div class="topbar-actions">
      <div class="theme-seg" role="group" aria-label="主题">
        <button
          v-for="option in themeOptions"
          :key="option.value"
          :class="{ active: theme.setting === option.value }"
          :title="option.label"
          :aria-label="option.label"
          @click="theme.apply(option.value)"
        >
          <svg v-if="option.value === 'light'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/></svg>
          <svg v-else-if="option.value === 'dark'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/></svg>
          <svg v-else viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="2.5" y="4" width="19" height="13" rx="2"/><path d="M8.5 20.5h7M12 17v3.5"/></svg>
        </button>
      </div>
      <span v-if="!compact && routes.current !== 'panel' && routes.current !== 'overview'" class="state-pill" :class="pillClass">{{ controller.scenario.value.label }}</span>
      <button
        class="icon-btn topbar-refresh"
        :class="{ 'is-busy': refreshing }"
        type="button"
        title="刷新状态"
        aria-label="刷新状态"
        :aria-busy="refreshing ? 'true' : 'false'"
        :disabled="refreshing"
        @click="refreshCurrentView"
      >
        <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M20.5 12a8.5 8.5 0 1 1-2.6-6.1"/><path d="M18.2 3.4v3.3h-3.3"/></svg>
      </button>
      <div class="notification-wrap">
        <button
          class="notification-btn"
          :class="{ 'bell-pulse': bellPulsing }"
          :aria-expanded="panelOpen ? 'true' : 'false'"
          title="通知"
          @click.stop="togglePanel"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M18 9A6 6 0 0 0 6 9c0 6-2.5 7-2.5 7h17S18 15 18 9Z"/><path d="M10.3 20a2 2 0 0 0 3.4 0"/></svg>
          <span class="notification-dot" :class="{ show: unread > 0, danger: hasDanger }">{{ unread }}</span>
        </button>
      </div>
    </div>
  </header>
</template>
