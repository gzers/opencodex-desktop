// 数据目录功能切片的状态与动作（IMP-04 §19.4 E：从总 store 拆出，切片三）。
// 根 store 通过 getter/委托保留 `app.dataRoot*` 读取与动作调用。
import { defineStore } from 'pinia'
import {
  getDataRootConfig,
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
        if (this.lastValidation !== 'valid') {
          this.error = '目标目录结构不是当前版本，已停止初始化。'
          return false
        }
        this.lastResult = await initializeDataRoot(path)
        this.path = path
        return true
      } catch {
        this.error = '数据目录校验或初始化失败；未修改目标目录。'
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
    async switchTo(targetPath: string, migrateData: boolean) {
      if (!this.allowBindingChange()) return false
      const path = targetPath.trim()
      if (!path) {
        this.error = '请输入显式数据目录路径。'
        return false
      }
      this.switching = true
      this.error = ''
      try {
        const result = await switchDataRootCommand(path, migrateData ? 'migrate_data' : 'reference_only')
        if (result.status === 'blocked') {
          this.error = result.blocked === 'running'
            ? '面板或安装任务仍在运行，暂时不能切换路径。'
            : '切换已被安全规则阻止。'
          return false
        }
        if (result.config) this.config = result.config
        this.reconciliationRequired = result.reconciliationRequired === true
        this.pendingRestart = result.status === 'restart_required' || this.reconciliationRequired
        return true
      } catch {
        this.error = migrateData
          ? '迁移未提交，旧绑定保留；目标可能留下隔离的未完成副本，请勿直接使用。'
          : '数据目录引用切换未完成；请重新读取路径配置。'
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
            : '路径不能与数据目录互相嵌套。'
          return false
        }
        if (result.config) this.config = result.config
        this.reconciliationRequired = result.reconciliationRequired === true
        this.pendingRestart = result.status === 'restart_required' || this.reconciliationRequired
        return true
      } catch {
        this.error = 'OPENCODEX_HOME 保存失败。'
        return false
      } finally {
        this.homeSaving = false
      }
    },
    allowBindingChange() {
      if (this.switching || this.homeSaving || this.saving) return false
      if (this.pendingRestart || this.reconciliationRequired || this.config?.runtimeActive === false) {
        this.error = '路径变更正在等待重启核对；请先重启管理器。'
        return false
      }
      return true
    },
  },
})
