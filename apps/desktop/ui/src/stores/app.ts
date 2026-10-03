import { defineStore } from "pinia"
import {
  installRuntime as installRuntimeCommand,
  uninstallRuntime as uninstallRuntimeCommand,
  type InstallSourceKind,
  type RuntimeInstallRequest,
  type RuntimeUninstallRequest,
} from "@/features/runtime/api"
import { useRuntimeStore } from "@/features/runtime/store"
import { runtimeErrorText } from "@/features/runtime/errors"
import { useNotificationsStore } from "@/features/notifications/store"
import { useSyncStore } from "@/features/sync/store"
import { usePreferencesStore } from "@/features/preferences/store"
import { useMigrationStore } from "@/features/migration/store"
import { useEnvironmentStore } from "@/features/environment/store"
import { useUpdatesStore } from "@/features/updates/store"
import { useModalStore } from "@/app/modal/store"
import { useFeedbackStore } from "@/app/feedback/store"
import { useLifecycleStore } from "@/app/lifecycle/store"
import { usePathsStore } from "@/app/paths/store"
import { usePanelStore } from "@/features/panel/store"
import { useExtensionsStore } from "@/features/extensions/store"
import { useAboutStore } from "@/features/about/store"
import type {
  ExtensionWriteCommand,
  ExtensionClientId,
  ExtensionSyncMethod,
} from "@/features/extensions/api"
import { useDiagnosticsStore } from "@/features/diagnostics/store"
import { cleanupLocalLogs } from "@/platform/workspace"
import type { LifecycleAction } from "@/types/ui"
import type { StatusSnapshot } from "@/contracts/runtimeStatus"
import { useDataRootStore } from "@/features/data-root/store"
import type { PreferencesDto } from "@/features/preferences/api"
import { exit } from "@tauri-apps/plugin-process"
import type { EnvironmentReport } from "@/features/environment/api"
import type { ConnectionState, RuntimeState } from "@/types/ui"
import type { SaveSyncEndpointRequest } from "@/features/sync/api"

export interface ModalState {
  title: string
  body: string
  confirmLabel?: string
  primaryKind?: "primary" | "danger"
  cancelLabel?: string
  deleteLabel?: string
  resolveLabel?: string
  wide?: boolean
  // 需要文本输入时使用；确认时把各字段取值一并回传。
  fields?: ModalField[]
  onConfirm?: (values?: Record<string, string>) => void
  onCancel?: () => void
  onDelete?: () => void
  onResolve?: () => void
}

export interface ModalField {
  key: string
  label: string
  placeholder?: string
  value?: string
  type?: "text" | "password"
}

/** 受管目录分区键 → 中文名称（路径投影缺失时的兜底文案，避免泄漏英文 key）。 */
const MANAGED_PATH_LABELS: Record<string, string> = {
  manager_state: "管理器设置目录",
  backups: "备份目录",
  logs: "日志目录",
  exports: "导出目录",
  cache: "缓存目录",
  sync_state: "同步状态目录",
}

// 兼容再导出：`runtimeErrorText` 已随 E 切片十二迁入 features/runtime/errors.ts。
export { runtimeErrorText } from "@/features/runtime/errors"

export const useAppStore = defineStore("app", {
  state: () => ({
    initialized: true,
    appExitError: "",
  }),
  getters: {
    // —— 通知功能切片（features/notifications/store.ts）的兼容只读代理 ——
    // 写入点（面板开合、通知列表）改用 useNotificationsStore()。
    notifications: () => useNotificationsStore().list,
    notificationsLoading: () => useNotificationsStore().loading,
    notificationsError: () => useNotificationsStore().error,
    notificationPanelOpen: () => useNotificationsStore().panelOpen,
    // —— 扩展功能切片（features/extensions/store.ts）的兼容只读代理 ——
    extensions: () => useExtensionsStore().discovered,
    extensionsLoading: () => useExtensionsStore().loading,
    extensionsError: () => useExtensionsStore().error,
    extensionConfig: () => useExtensionsStore().config,
    extensionConfigLoading: () => useExtensionsStore().configLoading,
    extensionConfigError: () => useExtensionsStore().configError,
    extensionWriteError: () => useExtensionsStore().writeError,
    extensionToggleBusy: () => useExtensionsStore().toggleBusy,
    // —— 数据目录功能切片（features/data-root/store.ts）的兼容只读代理 ——
    dataRootPath: () => useDataRootStore().path,
    dataRootConfig: () => useDataRootStore().config,
    dataRootConfigLoading: () => useDataRootStore().configLoading,
    dataRootConfigError: () => useDataRootStore().configError,
    dataRootSwitching: () => useDataRootStore().switching,
    dataRootHomeSaving: () => useDataRootStore().homeSaving,
    dataRootSaving: () => useDataRootStore().saving,
    dataRootLastResult: () => useDataRootStore().lastResult,
    dataRootLastValidation: () => useDataRootStore().lastValidation,
    dataRootError: () => useDataRootStore().error,
    // —— 关于 / 诊断功能切片的兼容只读代理 ——
    aboutApp: () => useAboutStore().app,
    doctorReport: () => useDiagnosticsStore().report,
    doctorLoading: () => useDiagnosticsStore().loading,
    doctorError: () => useDiagnosticsStore().error,
    // —— 同步功能切片的兼容只读代理 ——
    syncConfig: () => useSyncStore().config,
    syncStatus: () => useSyncStore().status,
    syncSaving: () => useSyncStore().saving,
    syncTesting: () => useSyncStore().testing,
    syncRunning: () => useSyncStore().running,
    // —— 偏好功能切片的兼容只读代理 ——
    preferences: () => usePreferencesStore().data,
    preferencesLoading: () => usePreferencesStore().loading,
    preferencesError: () => usePreferencesStore().error,
    // —— 配置迁移功能切片的兼容只读代理 ——
    migrationExporting: () => useMigrationStore().exporting,
    migrationImporting: () => useMigrationStore().importing,
    migrationError: () => useMigrationStore().error,
    migrationLastExport: () => useMigrationStore().lastExport,
    migrationLastImport: () => useMigrationStore().lastImport,
    // —— 应用级模态框的兼容只读代理 ——
    modal: () => useModalStore().current,
    // —— 受管路径投影的兼容只读代理 ——
    managedPathTargets: () => usePathsStore().managed,
    agentPathTargets: () => usePathsStore().agent,
    // —— 更新/升级/恢复功能切片的兼容只读代理 ——
    officialProject: () => useUpdatesStore().officialProject,
    officialProjectError: () => useUpdatesStore().officialProjectError,
    officialProjectLoading: () => useUpdatesStore().officialProjectLoading,
    upgradeBackupBusy: () => useUpdatesStore().upgradeBackupBusy,
    upgradeLastBackup: () => useUpdatesStore().upgradeLastBackup,
    upgradeBackupError: () => useUpdatesStore().upgradeBackupError,
    restoreBusy: () => useUpdatesStore().restoreBusy,
    restoreSummaryLoading: () => useUpdatesStore().restoreSummaryLoading,
    restoreSummaryError: () => useUpdatesStore().restoreSummaryError,
    restoreRiskSummary: () => useUpdatesStore().restoreRiskSummary,
    restoreLastBackup: () => useUpdatesStore().restoreLastBackup,
    restoreError: () => useUpdatesStore().restoreError,
    appUpdateBusy: () => useUpdatesStore().appUpdateBusy,
    appUpdateError: () => useUpdatesStore().appUpdateError,
    // —— 环境发现功能切片的兼容只读代理 ——
    environment: () => useEnvironmentStore().report,
    environmentLoading: () => useEnvironmentStore().loading,
    // —— 运行来源 / 安装 / 卸载功能切片的兼容只读代理 ——
    runtimeSource: () => useRuntimeStore().source,
    runtimeSourceLoading: () => useRuntimeStore().sourceLoading,
    runtimeSourceError: () => useRuntimeStore().sourceError,
    runtimeInstalling: () => useRuntimeStore().installing,
    runtimeInstall: () => useRuntimeStore().install,
    runtimeInstallLines: () => useRuntimeStore().installLines,
    runtimeInstallOutcome: () => useRuntimeStore().installOutcome,
    runtimeInstallError: () => useRuntimeStore().installError,
    offlinePreview: () => useRuntimeStore().offlinePreview,
    offlinePreviewLoading: () => useRuntimeStore().offlinePreviewLoading,
    runtimeUninstallBusy: () => useRuntimeStore().uninstallBusy,
    runtimeUninstallPlan: () => useRuntimeStore().uninstallPlan,
    runtimeUninstallPlanLoading: () => useRuntimeStore().uninstallPlanLoading,
    runtimeUninstallPlanError: () => useRuntimeStore().uninstallPlanError,
    runtimeUninstallResult: () => useRuntimeStore().uninstallResult,
    runtimeUninstallError: () => useRuntimeStore().uninstallError,
    officialUninstallObservation: () => useRuntimeStore().officialUninstallObservation,
    installModalRequest: () => useRuntimeStore().installModalRequest,
    // —— 官方面板嵌入功能切片的兼容只读代理 ——
    panelUrl: () => usePanelStore().url,
    panelLoading: () => usePanelStore().loading,
    panelError: () => usePanelStore().error,
    panelLoaded: () => usePanelStore().loaded,
    // —— 界面反馈域（app/feedback/store.ts）的兼容只读代理 ——
    toast: () => useFeedbackStore().toast,
    uiFault: () => useFeedbackStore().uiFault,
    recentEvents: () => useFeedbackStore().recentEvents,
    // —— 运行生命周期与状态域（app/lifecycle/store.ts）的兼容只读代理 ——
    runtimeState: () => useLifecycleStore().runtimeState,
    runtimeAnnouncePrimed: () => useLifecycleStore().runtimeAnnouncePrimed,
    statusLoading: () => useLifecycleStore().statusLoading,
    statusError: () => useLifecycleStore().statusError,
    statusSnapshot: () => useLifecycleStore().statusSnapshot,
    lifecycleState: () => useLifecycleStore().lifecycleState,
    processActionBusy: () => useLifecycleStore().processActionBusy,
    processAction: () => useLifecycleStore().processAction,
    processProgressAction: () => useLifecycleStore().processProgressAction,
    processProgressOpen: () => useLifecycleStore().processProgressOpen,
    processProgressError: () => useLifecycleStore().processProgressError,
    processProgressTimedOut: () => useLifecycleStore().processProgressTimedOut,
    processProgressSawTransition: () => useLifecycleStore().processProgressSawTransition,
    processProgressCompleted: () => useLifecycleStore().processProgressCompleted,
    // —— 同步连接状态（features/sync/store.ts）的兼容只读代理 ——
    webdavState: () => useSyncStore().webdavState,
  },
  actions: {
    // —— 界面反馈域（app/feedback/store.ts）的动作委托 ——
    showToast(message: string) {
      return useFeedbackStore().showToast(message)
    },
    clearToast() {
      return useFeedbackStore().clearToast()
    },
    recordEvent(message: string) {
      return useFeedbackStore().recordEvent(message)
    },
    reportUiFault(message: string, info?: string) {
      return useFeedbackStore().reportUiFault(message, info)
    },
    clearUiFault() {
      return useFeedbackStore().clearUiFault()
    },
    // —— 运行生命周期与状态域（app/lifecycle/store.ts）的动作委托 ——
    setStatusSnapshot(snapshot: StatusSnapshot | null) {
      return useLifecycleStore().setStatusSnapshot(snapshot)
    },
    syncProcessProgress(snapshot: StatusSnapshot | null) {
      return useLifecycleStore().syncProcessProgress(snapshot)
    },
    dismissProcessProgress() {
      return useLifecycleStore().dismissProcessProgress()
    },
    reopenProcessProgress() {
      return useLifecycleStore().reopenProcessProgress()
    },
    requestProcessAction(action: LifecycleAction) {
      return useLifecycleStore().requestProcessAction(action)
    },
    // —— 运行来源 / 安装 / 卸载功能切片（features/runtime/store.ts）的委托与根壳编排 ——
    /**
     * 运行来源卡片的第一行事实；失败时保留上一次结果并标记错误，
     * 不把「读不到」显示成「未解析」（那是两件不同的事）。
     */
    async loadRuntimeSource() {
      return useRuntimeStore().loadSource()
    },
    async setRuntimeSourcePath(path: string | null) {
      const runtime = useRuntimeStore()
      const result = await runtime.setSourcePath(path)
      if (!result.ok) {
        this.showToast(result.error)
        return false
      }
      this.showToast(path ? "已切换为指定运行来源。" : "已恢复自动发现。")
      // 来源换了，版本事实也要跟着换：读一次版本让后端把 `resolved_version`
      // 回填到当前来源，再重读来源，卡片才不会停在「未知」（`FZ-48`）。
      await this.refreshRuntimeVersionFact()
      await this.loadStatusSnapshot()
      return true
    },
    async restoreDiscoveredRuntime() {
      const result = await useRuntimeStore().restoreDiscovered()
      if (!result.ok) return false
      this.showToast("已恢复自动发现。")
      await this.refreshRuntimeVersionFact()
      await this.loadStatusSnapshot()
      return true
    },
    async previewOfflinePackage(path: string) {
      return useRuntimeStore().previewOffline(path)
    },
    clearOfflinePreview() {
      useRuntimeStore().clearOfflinePreview()
    },
    /**
     * 执行一次托管安装。进度由后端事件推进；这里只负责终态收口与来源刷新。
     * 后端调用与自身状态落位在功能 store，跨域编排（版本事实、状态快照、Toast）留在根壳。
     */
    async installRuntime(request: RuntimeInstallRequest) {
      const runtime = useRuntimeStore()
      if (runtime.installing) return false
      runtime.beginInstall()
      try {
        const outcome = await installRuntimeCommand(request)
        runtime.setInstallOutcome(outcome)
        // 不能只凭命令返回值就说「安装完成」：必须核实运行来源真的切到了托管安装。
        // 真实链路里出现过「命令返回成功、磁盘上没有前缀、来源仍是未解析」的情形，
        // 那时界面若直接报完成，用户会以为装好了。
        await this.loadRuntimeSource()
        if (runtime.source?.kind !== "managed") {
          runtime.failInstall(
            "安装命令已返回，但运行来源没有切换到托管安装；请刷新运行来源并查看诊断中心日志。",
          )
          return false
        }
        runtime.markInstallDone()
        // 装完就刷新「官方版本事实」：版本升级 / 关于两处都读它，否则会停在装之前的
        // 「未发现」或上一个来源的版本（真机：装完 2.66.0，升级卡仍显示未发现）。
        await this.loadOfficialProject()
        // 环境发现（Node/npm/ocx 是否存在）与运行来源是两套事实：装完必须重跑一次，
        // 否则「安装形态 / 环境检测」仍停在装之前的「未发现」。
        await this.refreshEnvironment()
        await this.loadStatusSnapshot()
        this.showToast(
          outcome.needsRestart
            ? `已安装 OpenCodex ${outcome.version}；代理正在运行，重启后生效。`
            : `已安装 OpenCodex ${outcome.version}。`,
        )
        return true
      } catch (error) {
        runtime.failInstall(runtimeErrorText(error))
        await this.loadRuntimeSource()
        return false
      } finally {
        runtime.finishInstall()
      }
    },
    async cancelRuntimeInstall() {
      return useRuntimeStore().cancelInstall()
    },
    async loadUninstallPlan() {
      return useRuntimeStore().loadUninstallPlan()
    },
    async uninstallRuntime(request: RuntimeUninstallRequest) {
      const runtime = useRuntimeStore()
      if (runtime.uninstallBusy) return false
      runtime.beginUninstall()
      try {
        const result = await uninstallRuntimeCommand(request)
        runtime.setUninstallResult(result)
        await this.loadRuntimeSource()
        // 卸载后重新读版本事实：读不到时按 `FZ-48` 保留上次结果并标记错误，
        // 而不是继续把已移除来源的版本当作当前版本展示。
        await this.loadOfficialProject()
        await runtime.loadOfficialUninstallObservation()
        // 卸掉之后再跑一次环境发现：ocx 入口已被移除，环境卡 / 概览「安装形态」/
        // 环境门禁都要立刻不再显示「已发现」，不能停在卸载前的事实。
        await this.refreshEnvironment()
        await this.loadStatusSnapshot()
        this.showToast(result.message)
        return true
      } catch (error) {
        runtime.failUninstall(runtimeErrorText(error))
        return false
      } finally {
        runtime.finishUninstall()
      }
    },
    async loadOfficialUninstallObservation() {
      return useRuntimeStore().loadOfficialUninstallObservation()
    },
    requestInstallModal(source: InstallSourceKind) {
      useRuntimeStore().requestInstallModal(source)
    },
    consumeInstallModalRequest() {
      return useRuntimeStore().consumeInstallModalRequest()
    },
    /**
     * 后端在来源变化后广播；前端据此刷新卡片，避免停留在旧事实。
     */
    async startRuntimeSourceEventStream() {
      return useRuntimeStore().startSourceEventStream()
    },
    async startStatusEventStream() {
      return useLifecycleStore().startStatusEventStream()
    },
    startNotificationEventStream() {
      return useNotificationsStore().startEventStream()
    },
    async loadPanelUrl() {
      // 面板地址由根壳持有的状态快照推导，推导与落位在 features/panel/store.ts。
      return usePanelStore().loadUrl(this.statusSnapshot)
    },
    async openPanelInBrowser() {
      const panel = usePanelStore()
      if (!panel.url) await this.loadPanelUrl()
      if (!panel.url || !panel.url.startsWith("http://127.0.0.1:"))
        return false
      try {
        return await usePathsStore().openExternal(panel.url)
      } catch {
        return false
      }
    },
    async loadStatusSnapshot() {
      return useLifecycleStore().loadStatusSnapshot()
    },
    // —— 数据目录功能切片（features/data-root/store.ts）的动作委托 ——
    async saveDataRoot(rootPath: string) {
      return useDataRootStore().save(rootPath)
    },
    async loadDataRootConfig() {
      return useDataRootStore().loadConfig()
    },
    async switchDataRoot(targetPath: string, migrateData: boolean) {
      return useDataRootStore().switchTo(targetPath, migrateData)
    },
    async saveOpencodexHome(mode: "inside" | "external", externalPath?: string) {
      return useDataRootStore().saveHome(mode, externalPath)
    },
    async loadAboutApp() {
      return useAboutStore().load()
    },
    // —— 更新/升级/恢复功能切片（features/updates/store.ts）的动作委托 ——
    async loadOfficialProject() {
      await useUpdatesStore().loadProject()
      // 读版本成功后，后端会把版本回填到**当前来源**（`FZ-48`）；这里再读一次来源，
      // 卡片「版本」才不会停在「未知」（真机：启动时的版本检查与来源读取是并发的）。
      if (!useUpdatesStore().officialProjectError && useUpdatesStore().officialProject?.version) {
        await this.loadRuntimeSource()
      }
    },
    /**
     * 读一次当前来源的版本，让后端把 `resolved_version` 与当前来源对齐，再重读来源。
     * 顺序很重要：先读版本（后端按当前来源回填）→ 再读来源（拿到带版本的事实）。
     */
    async refreshRuntimeVersionFact() {
      await this.loadOfficialProject()
      await this.loadRuntimeSource()
    },
    async exitApp() {
      this.appExitError = ""
      try {
        await exit(0)
      } catch {
        this.appExitError = "退出失败；桌面壳保持当前状态。"
        this.showToast(this.appExitError)
      }
    },

    async installAppUpdate() {
      const ok = await useUpdatesStore().installAppUpdate()
      if (ok) this.showToast("更新已安装；正在重启。")
      else this.showToast("更新安装失败；已保留当前版本。")
      return ok
    },
    async openOverviewDataRoot() {
      const loaded = await this.loadManagedPathTargets()
      if (!loaded) {
        this.showToast("数据目录不可用；已保留当前窗口。")
        return false
      }
      const target = this.managedPathTargets?.[0]?.path.replace(
        /\/manager-state$/,
        "",
      )
      if (!target) {
        this.showToast("数据目录不可用；已保留当前窗口。")
        return false
      }
      const opened = await this.openManagedPath(target)
      if (!opened) this.showToast("数据目录不可用；已保留当前窗口。")
      return opened
    },
    async openOverviewOpencodexHome() {
      const loaded = await this.loadManagedPathTargets()
      if (!loaded) {
        this.showToast("OPENCODEX_HOME 不可用；已保留当前窗口。")
        return false
      }
      const target =
        this.dataRootConfig?.opencodexHome ??
        this.statusSnapshot?.facts.opencodex_home
      if (!target) {
        this.showToast("OPENCODEX_HOME 不可用；已保留当前窗口。")
        return false
      }
      const opened = await this.openManagedPath(target)
      if (!opened) this.showToast("OPENCODEX_HOME 不可用；已保留当前窗口。")
      return opened
    },
    async createUpgradeBackup() {
      const backup = await useUpdatesStore().createUpgradeBackup()
      if (backup) this.showToast(`升级前备份已生成：${backup.backupId}`)
      else this.showToast("升级前备份失败；当前配置未修改。")
    },
    async loadRestoreRiskSummary() {
      return useUpdatesStore().loadRestoreSummary()
    },
    async createRestoreBackup() {
      const backup = await useUpdatesStore().createRestoreBackup()
      if (backup) this.showToast(`restore 前备份已生成：${backup.backupId}`)
      else this.showToast("restore 前备份失败；已保留当前配置。")
      return backup
    },
    // —— 环境发现功能切片（features/environment/store.ts）的动作委托 ——
    setEnvironment(next: EnvironmentReport) {
      return useEnvironmentStore().set(next)
    },
    // 环境重新发现：正式服务入口（供概览/设置的「重新发现」共用），
    // 不再依赖挂在 window 上的调试全局（IMP-04 §13.6）。
    async refreshEnvironment() {
      const ok = await useEnvironmentStore().refresh()
      if (!ok) this.showToast("环境检查暂不可用；已保留当前状态。")
    },
    setEnvironmentLoading(next: boolean) {
      return useEnvironmentStore().setLoading(next)
    },
    // —— 运行生命周期与状态域（app/lifecycle/store.ts）的动作委托 ——
    setRuntimeState(next: RuntimeState) {
      return useLifecycleStore().setRuntimeState(next)
    },
    // —— 同步连接状态（features/sync/store.ts）的动作委托 ——
    setWebdavState(next: ConnectionState) {
      return useSyncStore().setWebdavState(next)
    },
    async loadNotifications() {
      return useNotificationsStore().load()
    },
    // —— 同步功能切片（features/sync/store.ts）的动作委托（webdavState/Toast 仍由根壳编排） ——
    async loadSyncConfig() {
      const config = await useSyncStore().loadConfig()
      useSyncStore().setWebdavState(config?.endpoint ? "disconnected" : "unconfigured")
    },
    async saveSyncEndpoint(request: SaveSyncEndpointRequest) {
      const ok = await useSyncStore().saveEndpoint(request)
      if (!ok) {
        this.showToast("WebDAV 端点保存失败；已保留原配置。")
        return
      }
      useSyncStore().setWebdavState(useSyncStore().config?.endpoint ? "disconnected" : "unconfigured")
      this.showToast("WebDAV 端点已保存。")
    },
    async deleteSyncEndpoint(deleteCredentials: boolean) {
      const ok = await useSyncStore().deleteEndpoint(deleteCredentials)
      if (!ok) {
        this.showToast("WebDAV 端点删除失败；已保留原配置。")
        return
      }
      useSyncStore().setWebdavState(useSyncStore().config?.endpoint ? "disconnected" : "unconfigured")
      this.showToast("WebDAV 端点已删除。")
    },
    async testSyncConnection() {
      const status = await useSyncStore().testConnection()
      useSyncStore().setWebdavState(status.connectionState)
      this.showToast(status.message || "WebDAV 连接已测试。")
    },
    // —— 受管路径投影（app/paths/store.ts）的动作委托 ——
    async loadManagedPathTargets() {
      return usePathsStore().load()
    },
    async openManagedPath(path?: string) {
      return usePathsStore().open(path)
    },
    async openExternalLink(url: string) {
      return usePathsStore().openExternal(url)
    },
    async openLocalDocument(documentId: "license" | "third-party") {
      return usePathsStore().openDocument(documentId)
    },
    async openManagedPathByKey(key: string) {
      if (!this.managedPathTargets) await this.loadManagedPathTargets()
      const target = this.managedPathTargets?.find(item => item.key === key)
      if (!target) {
        this.showToast(`${MANAGED_PATH_LABELS[key] ?? '目标目录'}暂不可用；请稍后重试。`)
        return false
      }
      const opened = await this.openManagedPath(target.path)
      if (!opened) this.showToast(`${target.label}打开失败；请检查系统权限。`)
      return opened
    },
    async runSyncNow() {
      const ok = await useSyncStore().runNow()
      const status = useSyncStore().status
      useSyncStore().setWebdavState(status?.connectionState ?? "failed")
      if (ok) {
        // 冲突提醒由后端写入同一份通知 store；同步结束后刷新一次，
        // 让「同步冲突提醒」产生的通知立即可见，而不是等到下次打开通知中心。
        void this.loadNotifications()
      }
      return ok
    },
    async cleanupLocalLogs() {
      try {
        await cleanupLocalLogs()
        return true
      } catch {
        return false
      }
    },
    async cleanupNotifications() {
      return useNotificationsStore().cleanupByRetention()
    },

    // —— 偏好功能切片（features/preferences/store.ts）的动作委托 ——
    async loadPreferences() {
      return usePreferencesStore().load()
    },
    async savePreferences(preferences: PreferencesDto) {
      return usePreferencesStore().save(preferences)
    },
    async setInterfaceScale(value: unknown) {
      return usePreferencesStore().setScale(value)
    },
    async setVisualEffects(value: unknown) {
      return usePreferencesStore().setVisualEffects(value)
    },
    async setGlowRender(value: unknown) {
      return usePreferencesStore().setGlowRender(value)
    },
    async restorePreferences() {
      return usePreferencesStore().restore()
    },
    async loadDoctorReport() {
      return useDiagnosticsStore().loadReport()
    },
    // —— 扩展功能切片（features/extensions/store.ts）的动作委托 ——
    async loadExtensionConfig() {
      return useExtensionsStore().loadConfig()
    },
    async executeExtensionWrite(command: ExtensionWriteCommand) {
      return useExtensionsStore().write(command)
    },
    async toggleExtensionClient(client: ExtensionClientId, enabled: boolean) {
      return useExtensionsStore().toggleClient(client, enabled)
    },
    async setExtensionSourceDir(path: string | null) {
      return useExtensionsStore().setSourceDir(path)
    },
    async setExtensionSyncMethod(method: ExtensionSyncMethod) {
      return useExtensionsStore().setSyncMethod(method)
    },
    async resyncExtensionSkills() {
      return useExtensionsStore().resyncSkills()
    },
    async loadExtensions() {
      return useExtensionsStore().load()
    },
    async markNotificationRead(id: string) {
      return useNotificationsStore().markRead(id)
    },
    async markAllNotificationsRead() {
      return useNotificationsStore().markAllRead()
    },
    async deleteNotification(id: string) {
      return useNotificationsStore().remove(id)
    },
    async clearNotifications() {
      return useNotificationsStore().clearAll()
    },
    async clearReadNotifications() {
      return useNotificationsStore().clearRead()
    },
    async markNotificationResolved(id: string) {
      return useNotificationsStore().markResolved(id)
    },
    async clearResolvedNotifications() {
      return useNotificationsStore().clearResolved()
    },
    notificationAggregate() {
      return useNotificationsStore().aggregate
    },
    // 新容器不含额外口令：导出只需选择目标（数据根 exports 分区）。
    // —— 配置迁移功能切片（features/migration/store.ts）的动作委托 ——
    async exportMigration() {
      return useMigrationStore().exportConfig()
    },
    async importMigration(passphrase = "") {
      return useMigrationStore().importConfig(passphrase)
    },
    // —— 应用级模态框（app/modal/store.ts）的动作委托 ——
    openModal(modal: ModalState) {
      return useModalStore().open(modal)
    },
    openInputModal(modal: ModalState): Promise<Record<string, string> | null> {
      return useModalStore().openInput(modal)
    },
    resolveModal(
      result: "confirm" | "cancel" | "delete" | "resolve",
      values?: Record<string, string>,
    ) {
      return useModalStore().resolve(result, values)
    },
  },
})
