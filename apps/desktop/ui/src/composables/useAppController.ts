import { computed, onMounted, onUnmounted, watch } from 'vue'
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'
import { primeAppData } from '@/app/bootstrap'
import { actionLabels, runtimeScenarios, statusScenario } from '@/features/runtime/scenarios'
import {
  formatNotificationTime,
  unreadNotifications as selectUnreadNotifications,
} from '@/features/notifications/model'
import { notificationConfirmLabel } from '@/features/notifications/actions'
import { drainTrayRequests } from '@/features/tray/api'
import { useDiagnosticsController } from '@/features/diagnostics/useDiagnosticsController'
import { useLifecycleController } from '@/app/useLifecycleController'
import { usePanelStore } from '@/features/panel/store'
import { useLifecycleStore } from '@/app/lifecycle/store'
import type { NotificationItem } from '@/types/ui'

// 组合根（IMP-04 §19.4 E）：把诊断与生命周期控制器组合起来，并向调用方暴露与拆分前一致的字段。
export function useAppController(initialize = false) {
  const app = useAppStore()
  const routes = useRouteStore()

  const scenario = computed(() =>
    statusScenario(
      runtimeScenarios[app.statusSnapshot?.matrix.runtime ?? app.runtimeState],
      app.statusSnapshot,
    ),
  )

  const diagnostics = useDiagnosticsController()
  const { logCategory, currentLogKind, setLogCategory, logs, logsState, logsLoading, logsError, loadLogs } =
    diagnostics
  const { doctorShown, doctorReportText, runDoctor } = diagnostics

  const lifecycle = useLifecycleController({ runDoctor, loadLogs })
  const { setRuntimeState, runAction, handleTrayRequests, openRestoreConfirm } = lifecycle

  const unreadList = computed(() =>
    selectUnreadNotifications(app.notifications as NotificationItem[]),
  )
  // 通知中心默认展示全部存活通知；未读只决定角标与圆点语义。
  const liveNotifications = computed(() => app.notifications as NotificationItem[])
  const webdav = computed(() => app.webdavState)

  function openNotification(id: string) {
    const notification = (app.notifications as NotificationItem[]).find(item => item.id === id)
    if (!notification) return
    if (!notification.read) app.markNotificationRead(id)
    const label = notification.kind === 'warning' ? '警告' : notification.kind === 'danger' ? '风险' : '信息'
    app.openModal({
      title: notification.title,
      body: `${notification.detail}<span class="modal-meta">${formatNotificationTime(notification.time)} · ${label}</span>`,
      confirmLabel: notificationConfirmLabel(notification.action),
      onConfirm: () => {
        if (notification.action === 'restore') openRestoreConfirm(notification.target ?? app.runtimeState)
        else if (notification.action === 'sync') routes.go('settings', { section: 'sync' })
        else if (notification.action === 'logs') routes.go('logs', { tab: 'logs' })
        else if (notification.action === 'settings_installation') routes.go('settings', { section: 'installation' })
        else if (notification.action === 'settings_cleanup') routes.go('settings', { section: 'cleanup' })
        else routes.go('settings', { section: 'upgrade' })
      },
      deleteLabel: '删除',
      onDelete: () => deleteNotification(id),
      resolveLabel: notification.resolved ? '' : '标记已解决',
      onResolve: () => resolveNotification(id),
      cancelLabel: '关闭',
    })
  }

  function deleteNotification(id: string) {
    void app.deleteNotification(id).then(() => {
      app.showToast(app.notificationsError ? '通知删除失败；已保留当前通知。' : '通知已删除。')
    })
  }

  function markAllNotificationsRead() {
    app.markAllNotificationsRead()
  }

  function resolveNotification(id: string) {
    void app.markNotificationResolved(id).then(() => {
      app.showToast(app.notificationsError ? '标记已解决失败；已保留当前通知。' : '已标记为已解决。')
    })
  }

  function clearResolvedNotifications() {
    void app.clearResolvedNotifications().then(() => {
      app.showToast(app.notificationsError ? '清理已解决通知失败；已保留当前通知。' : '已清理已解决通知。')
    })
  }

  async function refreshEnvironment() {
    await app.refreshEnvironment()
  }

  /**
   * 顶栏全局「刷新状态」（IMP-06）：全局状态快照 + 当前页主数据。
   * 复用既有链路；不新增采集源或定时器。失败保留旧观测并如实提示，不假装成功。
   */
  async function refreshCurrentView() {
    const route = routes.current
    try {
      await app.loadStatusSnapshot()
      if (route === 'overview') {
        await loadLogs()
      } else if (route === 'panel') {
        // 快照已更新：重新推导面板地址，并请求内嵌官方面板重载。
        await app.loadPanelUrl()
        usePanelStore().requestReload()
      } else if (route === 'extensions') {
        await Promise.all([app.loadExtensions(), app.loadExtensionConfig()])
      } else if (route === 'logs') {
        if (routes.diagnosticsTab === 'notifications') await app.loadNotifications()
        else if (routes.diagnosticsTab === 'doctor') await runDoctor()
        else await loadLogs()
      } else if (route === 'settings') {
        await app.loadRuntimeSource()
        await app.loadPreferences()
        await app.refreshRuntimeVersionFact()
        if (routes.settingsSection === 'sync') await app.loadSyncConfig()
      }
    } catch {
      app.showToast('刷新失败；已保留当前内容。')
      return
    }
    // 状态采集在内部吞掉失败并保留旧快照，这里据其错误位如实提示，不显示成已刷新。
    if (useLifecycleStore().statusError) app.showToast('刷新未成功；已保留上一次运行状态。')
  }

  // 调试夹具只在开发环境挂到 window；正式构建不暴露 setState/setEnvironmentFixture，
  // 正式「重新发现」走 app.refreshEnvironment（IMP-04 §13.6）。
  if (import.meta.env.DEV && typeof window !== 'undefined') {
    ;(window as unknown as Record<string, unknown>).setState = setRuntimeState
    ;(window as unknown as Record<string, unknown>).setEnvironmentFixture = (report: Parameters<typeof app.setEnvironment>[0] | null) => {
      if (report) app.setEnvironment(report)
      else app.setEnvironmentLoading(false)
    }
  }

  let stopStatusEventStream: (() => void) | null = null
  let stopNotificationEventStream: (() => void) | null = null
  let stopRuntimeSourceEventStream: (() => void) | null = null
  let trayRequestTimer: number | null = null
  let trayBridgeFailing = false
  let disposed = false

  onMounted(() => {
    if (!initialize) return
    // 偏好通常已由 `main.ts` 在挂载前取回（落地页要用它决定），这里只兜底加载。
    if (!app.preferences) void app.loadPreferences()
    void app.loadAboutApp()
    // 概览的「数据目录 / OPENCODEX_HOME」是管理器自有路径；先取回配置，避免显示占位。
    void app.startStatusEventStream().then(stop => {
      if (disposed) stop?.()
      else stopStatusEventStream = stop ?? null
    })
    void app.startNotificationEventStream().then(stop => {
      if (disposed) stop?.()
      else stopNotificationEventStream = stop ?? null
    })
    void app.startRuntimeSourceEventStream().then(stop => {
      if (disposed) stop?.()
      else stopRuntimeSourceEventStream = stop ?? null
    })
    void loadLogs()
    // 各功能初始数据由 app 启动装配统一编排（IMP-04 §13.7）。
    primeAppData(app)

    const pollTrayRequests = () => {
      // F-07：托盘命令经前端轮询派发。失败不再静默——按一次可见提示，恢复后登记事件；
      // 菜单能展开与命令真正执行是两件事，不能把 IPC 故障显示成已执行。
      void drainTrayRequests()
        .then(actions => {
          if (trayBridgeFailing) {
            trayBridgeFailing = false
            app.recordEvent('托盘命令通道已恢复。')
          }
          handleTrayRequests(actions)
        })
        .catch(() => {
          if (!trayBridgeFailing) {
            trayBridgeFailing = true
            app.showToast('托盘命令通道暂不可用；可改用应用内按钮。')
          }
        })
    }
    pollTrayRequests()
    trayRequestTimer = window.setInterval(pollTrayRequests, 1000)
  })

  if (initialize) {
    watch(currentLogKind, () => {
      void loadLogs()
      void app.loadNotifications()
    })
  }

  onUnmounted(() => {
    disposed = true
    if (trayRequestTimer !== null) window.clearInterval(trayRequestTimer)
    stopStatusEventStream?.()
    stopStatusEventStream = null
    stopNotificationEventStream?.()
    stopNotificationEventStream = null
    stopRuntimeSourceEventStream?.()
    stopRuntimeSourceEventStream = null
  })

  return {
    refreshEnvironment,
    refreshCurrentView,
    scenario,
    logCategory,
    setLogCategory,
    logs,
    logsState,
    logsLoading,
    logsError,
    loadLogs,
    unreadNotifications: computed(() =>
      selectUnreadNotifications(app.notifications as NotificationItem[]),
    ),
    unreadList,
    liveNotifications,
    webdav,
    actionLabels,
    doctorReportText,
    doctorLoading: diagnostics.doctorLoading,
    doctorError: diagnostics.doctorError,
    doctorShown: doctorShown,
    runDoctor,
    setRuntimeState,
    runAction,
    handleTrayRequests,
    openRestoreConfirm,
    openNotification,
    deleteNotification,
    markAllNotificationsRead,
    resolveNotification,
    clearResolvedNotifications,
  }
}
