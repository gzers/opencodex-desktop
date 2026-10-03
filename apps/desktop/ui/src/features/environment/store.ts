// 环境发现功能切片的状态与动作（IMP-04 §19.4 E：从总 store 拆出，切片十一）。
import { defineStore } from 'pinia'
import { discoverEnvironment, type EnvironmentReport } from './api'

export const useEnvironmentStore = defineStore('environment', {
  state: () => ({
    report: null as EnvironmentReport | null,
    loading: false,
  }),
  actions: {
    set(next: EnvironmentReport) {
      this.report = next
      this.loading = false
    },
    setLoading(next: boolean) {
      this.loading = next
    },
    // 环境重新发现：正式服务入口（供概览/设置的「重新发现」共用），
    // 不再依赖挂在 window 上的调试全局（IMP-04 §13.6）。Toast 仍由根壳负责。
    async refresh(): Promise<boolean> {
      this.setLoading(true)
      try {
        this.set(await discoverEnvironment())
        return true
      } catch {
        this.setLoading(false)
        return false
      }
    },
  },
})
