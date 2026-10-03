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
    saving: false,
    lastResult: null as DataRootInitialization | null,
    lastValidation: null as DataRootValidation | null,
    error: '',
  }),
  actions: {
    async save(rootPath: string) {
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
        this.configError = false
      } catch {
        this.configError = true
      } finally {
        this.configLoading = false
      }
    },
    async switchTo(targetPath: string, migrateData: boolean) {
      if (this.switching) return false
      const path = targetPath.trim()
      if (!path) {
        this.error = '请输入显式数据目录路径。'
        return false
      }
      this.switching = true
      this.error = ''
      try {
        const result = await switchDataRootCommand(path, migrateData ? 'migrate_data' : 'reference_only')
        if (result.config) this.config = result.config
        if (result.status === 'blocked') {
          this.error = '切换已被安全规则阻止。'
          return false
        }
        return true
      } catch {
        this.error = '数据目录切换失败；未修改目标目录。'
        return false
      } finally {
        this.switching = false
      }
    },
    async saveHome(mode: 'inside' | 'external', externalPath?: string) {
      if (this.homeSaving) return false
      if (mode === 'external' && !externalPath?.trim()) {
        this.error = '请输入显式 OPENCODEX_HOME 路径。'
        return false
      }
      this.homeSaving = true
      this.error = ''
      try {
        const result = await setOpencodexHome(mode, externalPath?.trim())
        if (result.config) this.config = result.config
        if (result.status === 'blocked') {
          this.error = '路径不能与数据目录互相嵌套。'
          return false
        }
        return true
      } catch {
        this.error = 'OPENCODEX_HOME 保存失败。'
        return false
      } finally {
        this.homeSaving = false
      }
    },
  },
})
