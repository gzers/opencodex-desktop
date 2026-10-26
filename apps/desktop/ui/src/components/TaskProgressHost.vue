<script setup lang="ts">
import UiButton from '@/components/ui/UiButton.vue'
import TaskProgressCard from '@/components/patterns/TaskProgressCard.vue'
import { computed, onBeforeUnmount, onMounted } from 'vue'
import { useOperationTasks, operationTitles, operationPhaseText } from '@/app/feedback/tasks'
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'
import { runtimeLabel } from '@/lib/labels'
import type { LifecycleAction } from '@/types/ui'

const app = useAppStore()
const routes = useRouteStore()
const operations = useOperationTasks()
let stopOperations: (() => void) | undefined
let disposed = false
onMounted(async () => {
  try { const { listen } = await import('@tauri-apps/api/event');const stop = await listen('model-operation-progress',event => operations.apply(event.payload));if(disposed)stop();else stopOperations=stop } catch { /* IPC availability is reported by the owning command. */ }
})
onBeforeUnmount(() => { disposed=true;stopOperations?.() })

const processAction = computed(() => app.processProgressAction)
const processRuntime = computed(() => app.statusSnapshot?.matrix.runtime ?? app.runtimeState)
const processActionLabel = computed(() => {
  const labels: Record<LifecycleAction, string> = { start: '启动', stop: '停止', restart: '重启' }
  return processAction.value ? labels[processAction.value] : '处理'
})
const processRuntimeLabel = computed(() => runtimeLabel(processRuntime.value, '检查中'))
const processProgressTitle = computed(() => {
  if (app.processProgressError) return `${processActionLabel.value}未完成`
  if (app.processProgressCompleted) return `${processActionLabel.value}已完成`
  return `正在${processActionLabel.value} OpenCodex`
})
const processProgressMessage = computed(() => {
  if (app.processProgressError) return app.processProgressError
  if (app.processProgressCompleted) {
    return processAction.value === 'stop'
      ? '代理已停止，当前窗口仍可继续操作。'
      : '服务已进入可用状态，现在可以打开面板。'
  }
  return `当前状态：${processRuntimeLabel.value}。指令已返回，正在等待官方状态确认。`
})
const processProgressSteps = computed(() => {
  const action = processAction.value
  if (!action) return []
  const ready = processRuntime.value === 'running'
  const stopped = processRuntime.value === 'stopped'
  const failed = Boolean(app.processProgressError)
  const secondDone = action === 'stop' ? stopped : ready
  const thirdDone = action === 'stop' ? stopped : ready
  return [
    { label: `${action === 'stop' ? '停止' : action === 'restart' ? '重启' : '启动'}指令已发出`, state: 'done' },
    { label: action === 'stop' ? '等待官方进程退出' : '等待官方进程启动', state: failed ? 'pending' : secondDone ? 'done' : 'active' },
    { label: action === 'stop' ? '确认代理已停止' : '等待就绪健康检查', state: failed ? 'pending' : thirdDone ? 'done' : 'active' },
  ]
})
// 收起只隐藏任务卡，不取消后台任务；重新打开按钮回到铃铛旁的入口。
function collapse() {
  app.dismissProcessProgress()
  routes.requestNotificationAttention()
}

/** 有卡片被收起（手动或 8 秒到期）时，给一个显式展开入口，而不是偷偷全部展开。 */
const collapsedTasks = computed(() => operations.tasks.filter(t => t.collapsed))
function reopenCollapsed() { for (const task of collapsedTasks.value) task.collapsed = false }
</script>


<template>
  <TaskProgressCard v-if="processAction&&app.processProgressOpen" :title="processProgressTitle" :message="processProgressMessage" :phase="app.processProgressError?'error':app.processProgressCompleted?'done':'running'" :steps="processProgressSteps" @collapse="collapse">
   <template #actions><UiButton class="ghost notif-ctl" @click="app.dismissProcessProgress()">{{ app.processActionBusy?'后台运行':'关闭' }}</UiButton></template>
  </TaskProgressCard>
  <TaskProgressCard v-for="task in operations.tasks.filter(t=>!t.collapsed)" :key="task.id" :title="operationTitles[task.kind]" :message="operationPhaseText(task)" :phase="task.phase==='failed'?'error':task.phase==='running'?'running':'done'" :show-track="task.phase==='running'" collapse-label="收起任务到通知入口" @collapse="operations.collapse(task.id)">
   <template #actions><UiButton class="ghost notif-ctl" @click="operations.collapse(task.id)">收起</UiButton><UiButton class="notif-ctl" @click="routes.go('models')">查看模型配置</UiButton></template>
  </TaskProgressCard>
  <button
    v-if="collapsedTasks.length"
    type="button"
    class="task-reopen"
    :aria-label="`展开 ${collapsedTasks.length} 个已收起任务`"
    @click="reopenCollapsed"
  >
    {{ collapsedTasks.length }} 个任务已收起 · 展开
  </button>
</template>

<style scoped>
/* 不属于任务卡视觉系统，只做「还有几张被收起」的轻量提示，避免用户以为任务丢了。 */
.task-reopen{align-self:flex-end;border:1px solid var(--border);border-radius:var(--radius);background:var(--panel);color:var(--muted);font-size:var(--text-caption);padding:var(--space-1) var(--space-3);cursor:pointer}
.task-reopen:hover{color:var(--text)}
.task-reopen:focus-visible{outline:2px solid var(--brand);outline-offset:2px}
</style>

