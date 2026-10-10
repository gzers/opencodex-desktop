// 运行来源 / 托管安装 / 卸载功能切片的状态与动作（IMP-04 §19.4 E：从总 store 拆出，切片十二）。
//
// 本切片只承载「来源事实 + 安装/卸载的自身状态与后端动作」。涉及其它域的编排
// （版本事实重读、状态快照刷新、Toast 文案）仍由根壳负责，避免功能 store 反向依赖根 store。
import { defineStore } from 'pinia'
import { runtimeErrorText } from './errors'
import {
  cancelRuntimeInstall as cancelRuntimeInstallCommand,
  getOfficialUninstallObservation,
  getRuntimeSource,
  planRuntimeUninstall as planRuntimeUninstallCommand,
  previewOfflinePackage as previewOfflinePackageCommand,
  restoreDiscoveredRuntime as restoreDiscoveredRuntimeCommand,
  setRuntimeSource as setRuntimeSourceCommand,
  RUNTIME_INSTALL_PROGRESS_EVENT,
  RUNTIME_SOURCE_CHANGED_EVENT,
  type InstallSourceKind,
  type OfflinePackagePreviewDto,
  type RuntimeInstallOutcomeDto,
  type RuntimeInstallProgress,
  type RuntimeSourceDto,
  type RuntimeUninstallPlanDto,
  type RuntimeUninstallResultDto,
} from './api'

// 运行来源读取的在途去重：并发调用共享同一份结果，而不是让后来者空手返回。
// 安装完成后后端会广播 `runtime-source-changed`（事件监听会触发一次读取），
// 安装流程自己也会再读一次；若后来者直接跳过就会拿到切换前的旧来源，
// 从而把一次成功安装误报成「来源没有切换」。
let sourceInFlight: Promise<boolean> | null = null

export const useRuntimeStore = defineStore('runtime', {
  state: () => ({
    // 运行来源（IMP FZ-48）：卡片第一行事实与安装 / 卸载共用同一份后端解析结果。
    source: null as RuntimeSourceDto | null,
    sourceLoading: false,
    sourceError: '',
    installing: false,
    // 安装进度只存在内存里（后端也不落盘）；联网看命令行明细，离线看分包步骤。
    install: null as RuntimeInstallProgress | null,
    installLines: [] as string[],
    installOutcome: null as RuntimeInstallOutcomeDto | null,
    installError: '',
    offlinePreview: null as OfflinePackagePreviewDto | null,
    offlinePreviewLoading: false,
    uninstallBusy: false,
    uninstallPlan: null as RuntimeUninstallPlanDto | null,
    uninstallPlanLoading: false,
    uninstallPlanError: '',
    uninstallResult: null as RuntimeUninstallResultDto | null,
    uninstallError: '',
    officialUninstallObservation: [] as string[],
    stopSourceEventStream: null as (() => void) | null,
    // 概览门禁的「安装 / 导入离线包」出口：置位后由设置页消费并打开对应弹窗。
    installModalRequest: null as InstallSourceKind | null,
  }),
  actions: {
    /**
     * 运行来源卡片的第一行事实；失败时保留上一次结果并标记错误，
     * 不把「读不到」显示成「未解析」（那是两件不同的事）。
     */
    async loadSource() {
      if (sourceInFlight) return sourceInFlight
      sourceInFlight = (async () => {
        this.sourceLoading = true
        this.sourceError = ''
        try {
          this.source = await getRuntimeSource()
          return true
        } catch (error) {
          this.sourceError = runtimeErrorText(error)
          return false
        } finally {
          this.sourceLoading = false
          sourceInFlight = null
        }
      })()
      return sourceInFlight
    },
    beginSourceChange() {
      this.sourceError = ''
    },
    // 运行来源切换 / 恢复自动发现：只做后端调用与结果落位；
    // Toast、版本事实重读与状态快照刷新由根壳编排。
    async setSourcePath(path: string | null): Promise<{ ok: boolean; error: string }> {
      this.beginSourceChange()
      try {
        this.source = await setRuntimeSourceCommand(path)
        return { ok: true, error: '' }
      } catch (error) {
        const message = runtimeErrorText(error)
        this.sourceError = message
        return { ok: false, error: message }
      }
    },
    async restoreDiscovered(): Promise<{ ok: boolean; error: string }> {
      this.beginSourceChange()
      try {
        this.source = await restoreDiscoveredRuntimeCommand()
        return { ok: true, error: '' }
      } catch (error) {
        const message = runtimeErrorText(error)
        this.sourceError = message
        return { ok: false, error: message }
      }
    },
    async previewOffline(path: string) {
      this.offlinePreviewLoading = true
      try {
        this.offlinePreview = await previewOfflinePackageCommand(path)
        return this.offlinePreview
      } catch (error) {
        this.offlinePreview = {
          ok: false,
          fileName: path.split('/').pop() ?? path,
          fileSize: 0,
          version: null,
          sha256Prefix: null,
          rejectionCode: 'preview_failed',
          message: runtimeErrorText(error),
        }
        return this.offlinePreview
      } finally {
        this.offlinePreviewLoading = false
      }
    },
    clearOfflinePreview() {
      this.offlinePreview = null
    },
    // —— 安装：后端动作由根壳包裹（涉及版本事实与状态快照），这里只提供状态落位 ——
    beginInstall() {
      this.installing = true
      this.install = { phase: 'preparing', percent: 0, line: null }
      this.installLines = []
      this.installOutcome = null
      this.installError = ''
    },
    observeInstallProgress(progress: RuntimeInstallProgress) {
      if (!this.installing) return
      this.install = progress
      if (progress.line) this.installLines = [...this.installLines, progress.line].slice(-400)
    },
    setInstallOutcome(outcome: RuntimeInstallOutcomeDto) {
      this.installOutcome = outcome
    },
    failInstall(message: string) {
      this.installError = message
      this.install = { phase: 'failed', percent: 100, line: message }
    },
    markInstallDone() {
      this.install = { phase: 'done', percent: 100, line: null }
    },
    finishInstall() {
      this.installing = false
    },
    async cancelInstall() {
      try {
        return await cancelRuntimeInstallCommand()
      } catch {
        return false
      }
    },
    // —— 卸载 ——
    /** 只读方案：界面先拿它渲染「将移除的对象」与备份检测，再决定是否执行。 */
    async loadUninstallPlan() {
      this.uninstallPlanLoading = true
      this.uninstallPlanError = ''
      try {
        this.uninstallPlan = await planRuntimeUninstallCommand()
        return true
      } catch (error) {
        this.uninstallPlanError = runtimeErrorText(error)
        return false
      } finally {
        this.uninstallPlanLoading = false
      }
    },
    beginUninstall() {
      this.uninstallBusy = true
      this.uninstallError = ''
      this.uninstallResult = null
    },
    setUninstallResult(result: RuntimeUninstallResultDto) {
      this.uninstallResult = result
    },
    failUninstall(message: string) {
      this.uninstallError = message
    },
    finishUninstall() {
      this.uninstallBusy = false
    },
    async loadOfficialUninstallObservation() {
      try {
        this.officialUninstallObservation = await getOfficialUninstallObservation()
      } catch {
        this.officialUninstallObservation = []
      }
    },
    requestInstallModal(source: InstallSourceKind) {
      this.installModalRequest = source
    },
    consumeInstallModalRequest() {
      const value = this.installModalRequest
      this.installModalRequest = null
      return value
    },
    /**
     * 后端在来源变化后广播；前端据此刷新卡片，避免停留在旧事实。
     */
    async startSourceEventStream() {
      if (this.stopSourceEventStream) return this.stopSourceEventStream
      try {
        const { listen } = await import('@tauri-apps/api/event')
        const unlistenSource = await listen(RUNTIME_SOURCE_CHANGED_EVENT, () => {
          void this.loadSource()
        })
        const unlistenProgress = await listen<RuntimeInstallProgress>(
          RUNTIME_INSTALL_PROGRESS_EVENT,
          (event) => {
            this.observeInstallProgress(event.payload)
          },
        )
        this.stopSourceEventStream = () => {
          void unlistenSource()
          void unlistenProgress()
          this.stopSourceEventStream = null
        }
        return this.stopSourceEventStream
      } catch {
        return null
      }
    },
  },
})
