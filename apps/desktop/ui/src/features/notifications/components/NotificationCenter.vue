<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import UiButton from '@/components/ui/UiButton.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import { useOperationTasks, operationTitles, operationPhaseText } from '@/app/feedback/tasks'
import { useRouteStore } from '@/stores/routes'
import { useAppController } from '@/composables/useAppController'
import { filterByCategory, type NotificationCategory } from '../model'
import NotificationItem from './NotificationItem.vue'

const props = defineProps<{ open: boolean }>()
const emit = defineEmits<{ close: []; collapse: [] }>()

const routes = useRouteStore()
const controller = useAppController()
const operations = useOperationTasks()
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
// 任务只是本次运行的内存观察，不伪造持久通知或通知历史。打开铃铛不改变 collapsed。
const collapsedTasks = computed(() => operations.tasks.filter(task => task.collapsed))
const visibleTasks = computed(() => activeCategory.value === 'all' || activeCategory.value === 'system' ? collapsedTasks.value : [])
const countFor = (cat: NotificationCategory | 'all') => filterByCategory(liveList.value, cat).length + (cat === 'all' || cat === 'system' ? collapsedTasks.value.length : 0)

function viewTask() {
  emit('close')
  routes.go('models')
}

function goToHistory() {
  emit('close')
  routes.go('logs', { tab: 'notifications' })
}

const confirming = ref(false)
watch(() => props.open, open => { if (!open) confirming.value = false })
const finishedTasks = computed(() => operations.tasks.filter(task => task.phase !== 'running'))
const clearCount = computed(() => liveList.value.length + finishedTasks.value.length)
const pendingCount = computed(() => liveList.value.filter(item => !item.resolved).length + finishedTasks.value.filter(task=>task.phase==='failed'||task.phase==='review').length)
function askClear() {
  if (!clearCount.value) return
  confirming.value = true
}
async function confirmClear() {
  confirming.value = false
  if (await controller.clearAllNotifications()) operations.clearFinished()
}
</script>

<template>
  <div class="notification-panel" :class="{ show: open }" role="dialog" aria-label="通知中心">
    <div class="notification-head">
      <h3>通知中心</h3>
      <div class="notification-head-actions">
        <button class="notification-clear" @click="goToHistory">历史</button>
        <button class="notification-clear" @click="controller.markAllNotificationsRead()">全部已读</button>
        <button class="notification-clear" :disabled="!clearCount" @click="askClear">清理</button>
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
      <div v-for="task in visibleTasks" :key="task.id" class="notification-item" :class="{ danger: task.phase === 'failed' }" :data-operation-task="task.id">
        <span class="severity" aria-hidden="true"></span>
        <button class="n-main" type="button" :aria-label="`查看任务：${operationTitles[task.kind]}`" @click="viewTask">
          <strong>{{ operationTitles[task.kind] }}</strong>
          <span class="n-detail">{{ operationPhaseText(task) }}</span>
        </button>
        <UiButton class="notification-clear" @click="viewTask">查看模型配置</UiButton>
      </div>
      <NotificationItem
        v-for="notification in visibleList"
        :key="notification.id"
        :item="notification"
        @open="controller.openNotification"
        @delete="controller.deleteNotification"
      />
      <div v-if="visibleList.length === 0 && visibleTasks.length === 0" class="notification-empty">当前分类没有通知。</div>
    </div>
  </div>
  <UiDialog :open="confirming" title="清理全部通知" @close="confirming = false">
    <p>将清理全部 {{ clearCount }} 条通知<span v-if="pendingCount">（其中 {{ pendingCount }} 条未处理）</span>。正在执行的任务继续运行。</p>
    <template #actions>
      <UiButton variant="ghost" @click="confirming = false">取消</UiButton>
      <UiButton variant="danger" @click="confirmClear">确认清理</UiButton>
    </template>
  </UiDialog>
</template>

