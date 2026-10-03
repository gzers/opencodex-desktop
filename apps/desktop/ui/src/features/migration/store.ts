// 配置迁移功能切片的状态与动作（IMP-04 §19.4 E：从总 store 拆出，切片七）。
import { defineStore } from 'pinia'
import { usePreferencesStore } from '@/features/preferences/store'
import {
  exportMigration as exportMigrationCommand,
  importMigration as importMigrationCommand,
  type MigrationExportResult,
  type MigrationImportResult,
} from './api'

export const useMigrationStore = defineStore('migration', {
  state: () => ({
    exporting: false,
    importing: false,
    error: '',
    lastExport: null as MigrationExportResult | null,
    lastImport: null as MigrationImportResult | null,
  }),
  actions: {
    async exportConfig() {
      if (this.exporting) return false
      this.exporting = true
      this.error = ''
      try {
        this.lastExport = await exportMigrationCommand()
        return true
      } catch {
        // 导出失败时保留当前配置和上一次导出结果；不回退 mock 数据。
        this.error = '配置导出失败；当前配置未修改。'
        return false
      } finally {
        this.exporting = false
      }
    },
    // 新容器直接导入；仅旧版加密容器需要口令。
    async importConfig(passphrase = '') {
      if (this.importing) return false
      this.importing = true
      this.error = ''
      try {
        const trimmed = passphrase.trim()
        this.lastImport = await importMigrationCommand(trimmed ? { passphrase: trimmed } : {})
        await usePreferencesStore().load()
        return true
      } catch (error) {
        // 导入失败时保留当前偏好；后端保证解密或校验失败不落盘。
        // 错误码 13 = 旧版加密容器需要原口令，前端据此单独提示，而不是当作普通失败。
        const code = (error as { code?: number } | null)?.code
        this.error =
          code === 13 ? '这是旧版加密容器；需要导出时使用的原口令。' : '配置导入失败；当前配置未修改。'
        return false
      } finally {
        this.importing = false
      }
    },
  },
})
