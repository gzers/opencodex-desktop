// 更新/升级/恢复功能切片的状态与动作（IMP-04 §19.4 E：从总 store 拆出，切片十）。
// 只负责 IPC 与自身状态；Toast、来源重读（loadRuntimeSource）等跨域编排仍由根壳负责。
import { defineStore } from 'pinia'
import {
  getOfficialProjectFacts,
  getOfficialRemoteLatest,
  type OfficialProjectDto,
  type OfficialRemoteLatestDto,
} from '@/features/about/api'
import { hasNewerVersion } from './version'
import { createRestoreBackup, createUpgradeBackup, getRestoreRiskSummary } from './upgrade'
import type { RestoreRiskSummary, UpgradeBackupResult } from './upgrade'

export const useUpdatesStore = defineStore('updates', {
  state: () => ({
    officialProject: null as OfficialProjectDto | null,
    officialProjectError: false,
    officialProjectLoading: false,
    // U-03：只读远端查询结果；失败只标记错误，不改写本地事实。
    officialRemote: null as OfficialRemoteLatestDto | null,
    officialRemoteError: false,
    officialRemoteLoading: false,
    // U-04：代跑官方更新的进行态与错误。
    officialUpdateBusy: false,
    officialUpdateError: '',
    upgradeBackupBusy: false,
    upgradeLastBackup: null as UpgradeBackupResult | null,
    upgradeBackupError: '',
    restoreBusy: false,
    restoreSummaryLoading: false,
    restoreSummaryError: false,
    restoreRiskSummary: null as RestoreRiskSummary | null,
    restoreLastBackup: null as UpgradeBackupResult | null,
    restoreError: '',
    appUpdateBusy: false,
    appUpdateError: '',
  }),
  actions: {
    async loadProject() {
      if (this.officialProjectLoading) return
      this.officialProjectLoading = true
      try {
        this.officialProject = await getOfficialProjectFacts()
        this.officialProjectError = false
      } catch {
        // IPC 失败时保留上一次官方版本事实；不猜测版本。
        this.officialProjectError = true
      } finally {
        this.officialProjectLoading = false
      }
    },
    /** 只读远端最新版本查询（U-03）：不安装、不写盘，失败不覆盖已有的本地/上次结果。 */
    async loadRemoteLatest() {
      if (this.officialRemoteLoading) return
      this.officialRemoteLoading = true
      try {
        this.officialRemote = await getOfficialRemoteLatest()
        this.officialRemoteError = false
      } catch {
        this.officialRemoteError = true
      } finally {
        this.officialRemoteLoading = false
      }
    },
    /** 本地版本 vs 远端最新：true/false 表示是否可更新，null 表示无法比较（而不是「已最新」）。 */
    officialUpdateAvailable(): boolean | null {
      const local = this.officialProject?.version ?? null
      const remote = this.officialRemote?.version ?? null
      return hasNewerVersion(local, remote)
    },
    beginOfficialUpdate() {
      this.officialUpdateBusy = true
      this.officialUpdateError = ''
    },
    failOfficialUpdate() {
      this.officialUpdateError = '官方更新安装失败；已保留当前版本。'
    },
    finishOfficialUpdate() {
      this.officialUpdateBusy = false
    },
    async createUpgradeBackup(): Promise<UpgradeBackupResult | null> {
      if (this.upgradeBackupBusy) return null
      this.upgradeBackupBusy = true
      this.upgradeBackupError = ''
      try {
        this.upgradeLastBackup = await createUpgradeBackup()
        return this.upgradeLastBackup
      } catch {
        this.upgradeBackupError = '升级前备份失败；当前配置未修改。'
        return null
      } finally {
        this.upgradeBackupBusy = false
      }
    },
    async loadRestoreSummary() {
      if (this.restoreSummaryLoading) return false
      this.restoreSummaryLoading = true
      try {
        this.restoreRiskSummary = await getRestoreRiskSummary()
        this.restoreSummaryError = false
        return true
      } catch {
        this.restoreSummaryError = true
        return false
      } finally {
        this.restoreSummaryLoading = false
      }
    },
    async createRestoreBackup(): Promise<UpgradeBackupResult | null> {
      if (this.restoreBusy) return null
      this.restoreBusy = true
      this.restoreError = ''
      try {
        this.restoreLastBackup = await createRestoreBackup()
        return this.restoreLastBackup
      } catch {
        this.restoreError = 'restore 前备份失败；已保留当前配置。'
        return null
      } finally {
        this.restoreBusy = false
      }
    },
    async installAppUpdate() {
      if (this.appUpdateBusy) return false
      this.appUpdateBusy = true
      this.appUpdateError = ''
      try {
        // 动态导入：让更新器按需加载，不进主包（与既有做法一致）。
        const { installUpdate } = await import('@/features/updates/update')
        await installUpdate()
        return true
      } catch {
        this.appUpdateError = '更新安装失败；已保留当前版本。'
        return false
      } finally {
        this.appUpdateBusy = false
      }
    },
  },
})
