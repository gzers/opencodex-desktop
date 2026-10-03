<script setup lang="ts">
import { computed, onMounted } from 'vue'
import AppTopbar from '@/components/AppTopbar.vue'
import NotificationItem from '@/features/notifications/components/NotificationItem.vue'
import { useAppController } from '@/composables/useAppController'
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'
import { triggerUiSelfCheckFault } from '@/features/diagnostics/selfCheck'

const routes = useRouteStore()
const app = useAppStore()
const controller = useAppController()

const logs = computed(() => controller.logs.value)

// 界面异常自检：故意抛出受控异常，交给全局错误边界置位兜底条（见 features/diagnostics/selfCheck.ts）。
// 在模板里写成 `@click="runUiSelfCheck()"`，Vue 会把处理器内的异常路由到 app.config.errorHandler。
function runUiSelfCheck() {
  triggerUiSelfCheckFault()
}
const logsState = computed(() => controller.logsState.value)
const logCategoryLabel = computed(() => (controller.logCategory.value === 'audit' ? '调用日志' : '应用日志'))
const logsMeta = computed(() => {
  const prefix = `${logCategoryLabel.value} · `
  if (controller.logsError.value) return `${prefix}读取失败 · 本地滚动截断 · 只读`
  if (controller.logsLoading.value) return `${prefix}读取中 · 本地滚动截断 · 只读`
  if (logsState.value?.fileMissing) return `${prefix}暂无日志文件 · 本地滚动截断 · 只读`
  const fallback = logsState.value?.fallbackFile
  if (fallback) return `${prefix}当前 ${logs.value.length} 行 · 显示 ${fallback}（请求的日志暂不存在） · 本地滚动截断 · 只读`
  return `${prefix}当前 ${logs.value.length} 行 · 本地滚动截断 · 只读`
})
// 日志内容按纯文本渲染：不做 HTML 注入（内容来自本地文件，且已由后端脱敏）。
const logText = computed(() => logs.value.join('\n'))

// 进入诊断中心时重读一次：日志会在应用运行期间增长，沿用启动时那一次读取会显示过期内容。
onMounted(() => { void controller.loadLogs() })
const liveNotifications = computed(() => app.notifications)
const unreadCount = computed(() => app.notifications.filter(item => !item.read && !item.resolved).length)
const unresolvedCount = computed(() => app.notifications.filter(item => !item.resolved).length)
const historyMeta = computed(
  () => `共 ${app.notifications.length} 条 · 未读 ${unreadCount.value} · 未解决 ${unresolvedCount.value} · 保留最近 30 天`,
)
function openLogsDirectory() {
  void app.loadManagedPathTargets().then(loaded => {
    const target = app.managedPathTargets?.find(item => item.key === 'logs')
    if (!loaded || !target) {
      app.showToast('日志目录不可用；已保留当前窗口。')
      return
    }
    void app.openManagedPath(target.path).then(opened => {
      if (!opened) app.showToast('日志目录不可用；已保留当前窗口。')
    })
  })
}
function openClearLogsModal() {
  app.openModal({
    title: '清理日志',
    body: '<p class="modal-lead">将清空数据目录日志分区中的本地日志文件内容；日志目录与文件本身保留。</p>',
    confirmLabel: '清理',
    onConfirm: () => {
      void app.cleanupLocalLogs().then(success => {
        app.showToast(success ? '本地日志文件已清空。' : '日志清理失败；原始文件已保留。')
        if (success) void controller.loadLogs()
      })
    },
  })
}

// 「清理通知」默认只清已解决；存在未处理通知时改文案并二次确认（FZ-43.3）。
function openClearResolvedNotificationsModal() {
  const pending = unresolvedCount.value
  app.openModal({
    title: pending > 0 ? `还有 ${pending} 条未处理通知` : '清理已解决通知',
    body: pending > 0
      ? '<p class="modal-lead">默认只清理已解决通知；继续将连同未处理通知一起清理。</p>'
      : '<p class="modal-lead">将清理全部已解决通知；当前没有未处理通知。</p>',
    confirmLabel: '清理',
    onConfirm: () => {
      if (pending > 0) {
        void app.clearNotifications()
        app.showToast('全部通知已清理。')
      } else {
        void app.clearResolvedNotifications()
        app.showToast('已清理已解决通知。')
      }
    },
  })
}

function openClearReadNotificationsModal() {
  app.openModal({
    title: '清理已读通知',
    body: '<p class="modal-lead">仅清空已读通知，未读通知保留在通知中心。</p>',
    confirmLabel: '清理',
    onConfirm: () => { void app.clearReadNotifications(); app.showToast('已读通知已清理。') },
  })
}
</script>

<template>
  <section class="route-section active">
    <AppTopbar />
    <div class="diag-tabs" role="tablist">
      <button :class="{ active: routes.diagnosticsTab === 'doctor' }" @click="routes.go('logs', { tab: 'doctor' })">环境诊断</button>
      <button :class="{ active: routes.diagnosticsTab === 'logs' }" @click="routes.go('logs', { tab: 'logs' })">日志历史</button>
      <button :class="{ active: routes.diagnosticsTab === 'notifications' }" @click="routes.go('logs', { tab: 'notifications' })">通知历史</button>
    </div>
    <article class="card">
      <div v-if="routes.diagnosticsTab === 'doctor'" class="diag-view active">
        <div class="card-head">
          <div><h2>运行环境诊断</h2><p>调用官方 <code>ocx doctor</code>：默认只读，不修改配置、不设置代理、不改网络。</p></div>
          <div class="toolbar"><button class="btn" :disabled="controller.doctorLoading.value" @click="controller.runDoctor()">{{ controller.doctorLoading.value ? '诊断中…' : '环境诊断' }}</button></div>
        </div>
        <pre class="doctor-out" :class="{ show: controller.doctorShown.value }">{{ controller.doctorReportText.value }}</pre>
        <p class="doctor-hint">写入型修复需要先停止代理并显式确认；本应用只展示只读摘要。</p>
        <!-- 界面异常自检：给出兜底条与恢复动作的可复现入口（原型「界面诊断」同款）。 -->
        <div class="selfcheck">
          <div class="card-head">
            <div><h3>界面异常自检</h3><p>主动触发一次界面异常，验证兜底条与恢复动作。仅本次会话生效，不改配置或数据。</p></div>
            <div class="toolbar"><button class="btn ghost" id="uiSelfCheckBtn" type="button" @click="runUiSelfCheck()">界面诊断</button></div>
          </div>
        </div>
      </div>
      <div v-if="routes.diagnosticsTab === 'logs'" class="diag-view active">
        <div class="diag-tabs" role="tablist" aria-label="日志分类">
          <button
            role="tab"
            :aria-selected="controller.logCategory.value === 'app' ? 'true' : 'false'"
            :class="{ active: controller.logCategory.value === 'app' }"
            @click="controller.setLogCategory('app')"
          >应用日志</button>
          <button
            role="tab"
            :aria-selected="controller.logCategory.value === 'audit' ? 'true' : 'false'"
            :class="{ active: controller.logCategory.value === 'audit' }"
            @click="controller.setLogCategory('audit')"
          >调用日志</button>
        </div>
        <div class="logbar">
          <span class="history-meta">{{ logsMeta }}</span>
          <div class="toolbar">
            <button class="btn ghost" @click="routes.go('panel')">面板请求日志</button>
            <button class="btn ghost" @click="openClearLogsModal()">清理日志</button>
            <button class="btn" @click="openLogsDirectory()">打开日志目录</button>
          </div>
        </div>
        <pre class="log">{{ logText }}</pre>
      </div>
      <div v-if="routes.diagnosticsTab === 'notifications'" class="diag-view active">
        <div class="logbar">
          <span class="history-meta">{{ historyMeta }}</span>
          <div class="toolbar">
            <button class="btn ghost" :disabled="!app.notifications.length" @click="openClearResolvedNotificationsModal()">清理已解决</button>
            <button class="btn ghost" :disabled="!app.notifications.some(item => item.read)" @click="openClearReadNotificationsModal()">清理已读</button>
          </div>
        </div>
        <div class="history-list">
          <template v-if="liveNotifications.length">
            <NotificationItem
              v-for="notification in liveNotifications"
              :key="notification.id"
              :item="notification"
              show-time
              @open="controller.openNotification"
              @delete="controller.deleteNotification"
            />
          </template>
          <div v-else class="notification-empty">暂无通知历史。</div>
        </div>
      </div>
    </article>
  </section>
</template>
