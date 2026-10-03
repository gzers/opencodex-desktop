<script setup lang="ts">
import { computed, ref } from 'vue'
import { useRouteStore } from '@/stores/routes'
import { useAppController } from '@/composables/useAppController'
import { filterByCategory, type NotificationCategory } from '../model'
import NotificationItem from './NotificationItem.vue'

defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: []; collapse: [] }>()

const routes = useRouteStore()
const controller = useAppController()
const activeCategory = ref<NotificationCategory | 'all'>('all')

const categoryTabs: { value: NotificationCategory | 'all'; label: string }[] = [
  { value: 'all', label: '全部' },
  { value: 'run', label: '运行' },
  { value: 'sync', label: '同步' },
  { value: 'update', label: '更新' },
  { value: 'system', label: '系统' },
]

// 默认展示全部存活通知；分类只做分组，不隐藏未读 / 未解决 / 历史。
const liveList = computed(() => controller.liveNotifications.value)
const visibleList = computed(() => filterByCategory(liveList.value, activeCategory.value))
const countFor = (cat: NotificationCategory | 'all') => filterByCategory(liveList.value, cat).length

function goToHistory() {
  emit('close')
  routes.go('logs', { tab: 'notifications' })
}

function clearResolved() {
  controller.clearResolvedNotifications()
}
</script>

<template>
  <div class="notification-panel" :class="{ show: open }" role="dialog" aria-label="通知中心">
    <div class="notification-head">
      <h3>通知</h3>
      <div class="notification-head-actions">
        <button class="notification-clear" @click="goToHistory">历史</button>
        <button class="notification-clear" @click="controller.markAllNotificationsRead()">全部已读</button>
        <button class="notification-clear" @click="clearResolved">清理已解决</button>
        <button
          class="notify-min"
          type="button"
          title="收起到铃铛"
          aria-label="收起通知中心到铃铛"
          @click="emit('collapse')"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M18 9A6 6 0 0 0 6 9c0 6-2.5 7-2.5 7h17S18 15 18 9Z"/><path d="M10.3 20a2 2 0 0 0 3.4 0"/></svg>
        </button>
      </div>
    </div>
    <div class="notification-cats" role="tablist" aria-label="通知分类">
      <button
        v-for="tab in categoryTabs"
        :key="tab.value"
        role="tab"
        :aria-selected="activeCategory === tab.value ? 'true' : 'false'"
        :class="{ active: activeCategory === tab.value }"
        @click="activeCategory = tab.value"
      >
        {{ tab.label }}
        <span class="n-count">{{ countFor(tab.value) }}</span>
      </button>
    </div>
    <div class="notification-list">
      <NotificationItem
        v-for="notification in visibleList"
        :key="notification.id"
        :item="notification"
        @open="controller.openNotification"
        @delete="controller.deleteNotification"
      />
      <div v-if="visibleList.length === 0" class="notification-empty">当前分类没有通知。</div>
    </div>
  </div>
</template>
