<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from 'vue'
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'
import AppSidebar from '@/components/AppSidebar.vue'
import { useAppController } from '@/composables/useAppController'
import PanelRoute from '@/routes/PanelRoute.vue'
import OverviewRoute from '@/routes/OverviewRoute.vue'
import ExtensionsRoute from '@/routes/ExtensionsRoute.vue'
import LogsRoute from '@/routes/LogsRoute.vue'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import TrayRoute from '@/routes/TrayRoute.vue'
import AppModal from '@/components/AppModal.vue'
import ToastHost from '@/components/ToastHost.vue'
import TaskProgressHost from '@/components/TaskProgressHost.vue'
import NotificationCenter from '@/features/notifications/components/NotificationCenter.vue'
import { useNotificationsStore } from '@/features/notifications/store'

const app = useAppStore()
const routes = useRouteStore()
const notifications = useNotificationsStore()
useAppController(true)
// 运行期异常兜底的恢复动作：整页重载，回到干净状态（IMP-04 §14.3）。
function reloadUi() {
  window.location.reload()
}
// 2026-09-28 用户要求：界面故障提示带恢复动作，改为**居中弹窗**（原来是窗口顶部一条横条）。
// 关闭路径 = 「关闭提示」/ 点遮罩 / Esc；三者都只清提示，不触碰其它状态。
function onFaultKeydown(event: KeyboardEvent) {
  if (event.key !== 'Escape' || !app.uiFault) return
  event.preventDefault()
  app.clearUiFault()
}
onMounted(() => window.addEventListener('keydown', onFaultKeydown))
onBeforeUnmount(() => window.removeEventListener('keydown', onFaultKeydown))
// Overlay keeps the native macOS window buttons above the web content.
const isMacOS = /^Mac/.test(navigator.platform)
const isWindows = /^Win/.test(navigator.platform)
const component = computed(() => {
  switch (routes.current) {
    case 'panel': return PanelRoute
    case 'overview': return OverviewRoute
    case 'extensions': return ExtensionsRoute
    case 'logs': return LogsRoute
    case 'settings': return SettingsRoute
    default: return undefined
  }
})
</script>

<template>
  <div class="app-window" :class="{ 'tray-mode': routes.current === 'tray', 'panel-route': routes.current === 'panel', 'is-macos': isMacOS, 'is-windows': isWindows }">
    <div v-if="app.uiFault" class="ui-fault-modal" role="alertdialog" aria-modal="true" aria-label="界面出现异常">
      <div class="ui-fault-mask" @click="app.clearUiFault()"></div>
      <div class="ui-fault">
        <h3>界面出现异常</h3>
        <p class="ui-fault-text">{{ app.uiFault }}</p>
        <div class="ui-fault-actions">
          <button class="btn ghost" type="button" @click="app.clearUiFault()">关闭提示</button>
          <button class="btn" type="button" @click="reloadUi()">重新加载界面</button>
        </div>
      </div>
    </div>
    <div class="desktop-stage" :class="{ 'panel-mode': routes.current === 'panel' }">
      <!-- macOS（titleBarStyle: Overlay）没有原生标题栏可拖：这块 28px 条带就是唯一拖拽区。
           data-tauri-drag-region 由 Tauri 注入脚本接管（mousedown → startDragging），
           所以它必须可命中（.titlebar 不能 pointer-events:none），否则窗口拖不动。 -->
      <header v-if="!isWindows" class="titlebar" aria-label="窗口控制区" data-tauri-drag-region></header>
      <div class="app-shell" :class="{ 'panel-mode': routes.current === 'panel' }">
        <AppSidebar />
        <main class="main">
          <component v-if="component" :is="component" />
        </main>
      </div>
    </div>
    <div v-if="routes.current === 'tray'" class="tray-stage"><TrayRoute /></div>
    <!-- 通知层是全局壳层：通知中心、Toast、任务卡在同一列顺次避让，不互相压盖。
         对话框打开时整层降到遮罩之下：任务卡比遮罩宽，会盖住居中对话框的内容。 -->
    <div class="notif-layer" :class="{ 'under-modal': app.modal !== null }">
      <NotificationCenter
        :open="app.notificationPanelOpen"
        @close="notifications.panelOpen = false"
        @collapse="notifications.panelOpen = false"
      />
      <ToastHost />
      <TaskProgressHost />
    </div>
    <AppModal />
  </div>
</template>
