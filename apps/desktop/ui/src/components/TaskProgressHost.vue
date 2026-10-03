<script setup lang="ts">
import { computed } from 'vue'
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'
import { runtimeLabel } from '@/lib/labels'
import type { LifecycleAction } from '@/types/ui'

const app = useAppStore()
const routes = useRouteStore()

const processAction = computed(() => app.processProgressAction)
const processRuntime = computed(() => app.statusSnapshot?.matrix.runtime ?? app.runtimeState)
const processActionLabel = computed(() => {
  const labels: Record<LifecycleAction, string> = { start: '启动', stop: '停止', restart: '重启' }
  return processAction.value ? labels[processAction.value] : '处理'
})
const processRuntimeLabel = computed(() => runtimeLabel(processRuntime.value, '检查中'))
const processProgressClass = computed(() => ({
  done: app.processProgressCompleted,
  error: Boolean(app.processProgressError),
}))
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
    {
      label: `${action === 'stop' ? '停止' : action === 'restart' ? '重启' : '启动'}指令已发出`,
      state: 'done',
    },
    {
      label: action === 'stop' ? '等待官方进程退出' : '等待官方进程启动',
      state: failed ? 'pending' : secondDone ? 'done' : 'active',
    },
    {
      label: action === 'stop' ? '确认代理已停止' : '等待就绪健康检查',
      state: failed ? 'pending' : thirdDone ? 'done' : 'active',
    },
  ]
})
// 收起只隐藏任务卡，不取消后台任务；重新打开按钮回到铃铛旁的入口。
function collapse() {
  app.dismissProcessProgress()
  routes.requestNotificationAttention()
}
</script>

<template>
  <section
    v-if="processAction && app.processProgressOpen"
    class="process-progress"
    :class="processProgressClass"
    role="status"
    aria-live="polite"
    aria-atomic="true"
  >
      <div class="process-progress-head">
        <div>
          <h3>{{ processProgressTitle }}</h3>
          <p>{{ processProgressMessage }}</p>
        </div>
        <button
          class="notify-min"
          type="button"
          title="收起到铃铛"
          aria-label="收起任务卡到铃铛"
          @click="collapse"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M18 9A6 6 0 0 0 6 9c0 6-2.5 7-2.5 7h17S18 15 18 9Z"/><path d="M10.3 20a2 2 0 0 0 3.4 0"/></svg>
        </button>
      </div>
      <div class="process-progress-track" aria-hidden="true"></div>
      <ol class="process-progress-steps">
        <li
          v-for="step in processProgressSteps"
          :key="step.label"
          class="process-progress-step"
          :class="step.state"
        >{{ step.label }}</li>
      </ol>
      <div class="process-progress-actions">
        <button class="btn ghost notif-ctl" type="button" @click="app.dismissProcessProgress()">
          {{ app.processActionBusy ? '后台运行' : '关闭' }}
        </button>
      </div>
  </section>
</template>
