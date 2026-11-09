// 数据目录功能切片的状态与动作（IMP-04 §19.4 E：从总 store 拆出，切片三）。
// 根 store 通过 getter/委托保留 `app.dataRoot*` 读取与动作调用。
import { defineStore } from 'pinia'
import {
  getDataRootConfig,
  getPathsSnapshotV2,
  type PathsSnapshotV2,
  initializeDataRoot,
  setOpencodexHome,
  switchDataRoot as switchDataRootCommand,
  validateDataRootStructure,
  type DataRootConfig,
  type DataRootInitialization,
  type DataRootValidation,
} from './api'

export const useDataRootStore = defineStore('data-root', {
  state: () => ({
    path: '',
    paths: null as PathsSnapshotV2 | null,
    pathsLoading: false,
    pathsStale: false,
    pathsError: '',
    pathsRead: 0,
    config: null as DataRootConfig | null,
    configLoading: false,
    configError: false,
    switching: false,
    homeSaving: false,
    pendingRestart: false,
    reconciliationRequired: false,
    saving: false,
    lastResult: null as DataRootInitialization | null,
    lastValidation: null as DataRootValidation | null,
    error: '',
  }),
  actions: {
    async loadPaths() {
      const read = ++this.pathsRead
      this.pathsLoading = true
      this.pathsStale = this.paths !== null
      this.pathsError = ''
      try {
        const paths = await getPathsSnapshotV2()
        if (read !== this.pathsRead) return
        this.paths = paths
        this.pathsStale = false
        if (paths.pendingRestart) this.pendingRestart = true
      } catch (error) {
        if (read !== this.pathsRead) return
        this.pathsStale = this.paths !== null
        this.pathsError = error instanceof Error ? error.message : '实际位置读取失败。'
      } finally {
        if (read === this.pathsRead) this.pathsLoading = false
      }
    },
    async save(rootPath: string) {
      if (!this.allowBindingChange()) return false
      const path = rootPath.trim()
      if (!path) {
        this.error = '请输入显式数据目录路径。'
        return false
      }
      this.saving = true
      this.error = ''
      try {
        this.lastValidation = await validateDataRootStructure(path)
        if (this.lastValidation !== 'valid' && this.lastValidation !== 'uninitialized') {
          this.error = '目标目录结构不是当前版本，已停止初始化。'
          return false
        }
        this.lastResult = await initializeDataRoot(path)
        this.lastValidation = 'valid'
        this.path = path
        return true
      } catch {
        this.error = '用户目录校验或初始化未完成；请重新读取实际位置核对。'
        return false
      } finally {
        this.saving = false
      }
    },
    async loadConfig() {
      if (this.configLoading) return
      this.configLoading = true
      try {
        this.config = await getDataRootConfig()
        if (!this.config.runtimeActive) this.pendingRestart = true
        this.configError = false
      } catch {
        this.configError = true
      } finally {
        this.configLoading = false
      }
    },
    async switchTo(targetPath: string, legacyMigrationRequested = false) {
      if (!this.allowBindingChange()) return false
      if (legacyMigrationRequested) {
        this.error = '不支持数据迁移；请明确选择新版目录，仅切换引用。'
        return false
      }
      const path = targetPath.trim()
      if (!path) {
        this.error = '请输入显式数据目录路径。'
        return false
      }
      this.switching = true
      this.error = ''
      try {
        const result = await switchDataRootCommand(path, 'reference_only')
        if (result.status === 'blocked') {
          this.error = result.blocked === 'running'
            ? '面板或安装任务仍在运行，暂时不能切换路径。'
            : result.blocked === 'runtime_unverified'
              ? '未能确认面板运行状态，暂时不能切换路径；请接入面板并重新检查后重试。'
            : result.blocked === 'future_version' ? '目标目录结构版本过新，未切换引用。'
              : result.blocked === 'corrupted' ? '目标目录结构损坏，未切换引用。'
                : '切换已被安全规则阻止。'
          return false
        }
        if (result.config) this.config = result.config
        this.reconciliationRequired = result.reconciliationRequired === true
        this.pendingRestart = result.status === 'restart_required' || this.reconciliationRequired
        return true
      } catch {
        this.error = '用户目录引用切换结果未确认；请重新读取路径配置核对，原目录内容未由本操作复制或删除。'
        // An IPC rejection can arrive after a pointer save; read back before another write.
        await this.loadConfig()
        await this.loadPaths()
        return false
      } finally {
        this.switching = false
      }
    },
    async saveHome(mode: 'inside' | 'external', externalPath?: string) {
      if (!this.allowBindingChange()) return false
      if (mode === 'external' && !externalPath?.trim()) {
        this.error = '请输入显式 OPENCODEX_HOME 路径。'
        return false
      }
      this.homeSaving = true
      this.error = ''
      try {
        const result = await setOpencodexHome(mode, externalPath?.trim())
        if (result.status === 'blocked') {
          this.error = result.blocked === 'running'
            ? '面板或安装任务仍在运行，暂时不能切换路径。'
            : result.blocked === 'runtime_unverified'
              ? '未能确认面板运行状态，暂时不能切换路径；请接入面板并重新检查后重试。'
            : '路径不能与数据目录互相嵌套。'
          return false
        }
        if (result.config) this.config = result.config
        this.reconciliationRequired = result.reconciliationRequired === true
        this.pendingRestart = result.status === 'restart_required' || this.reconciliationRequired
        return true
      } catch {
        this.error = 'OpenCodex HOME 保存结果未确认；请重新读取路径配置核对。'
        await this.loadConfig()
        await this.loadPaths()
        return false
      } finally {
        this.homeSaving = false
      }
    },
    allowBindingChange() {
      if (this.switching || this.homeSaving || this.saving) return false
      if (this.pendingRestart || this.reconciliationRequired || this.paths?.pendingRestart || this.configError || this.config?.runtimeActive === false) {
        this.error = '路径变更正在等待重启核对；请先重启管理器。'
        return false
      }
      return true
    },
  },
})
