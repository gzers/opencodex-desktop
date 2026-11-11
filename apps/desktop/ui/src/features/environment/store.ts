// 环境发现功能切片的状态与动作（IMP-04 §19.4 E：从总 store 拆出，切片十一）。
import { defineStore } from 'pinia'
import { discoverEnvironment, type EnvironmentReport } from './api'

export const useEnvironmentStore = defineStore('environment', {
  state: () => ({
    report: null as EnvironmentReport | null,
    loading: false,
    error: '',
    requestGeneration: 0,
  }),
  actions: {
    set(next: EnvironmentReport) {
      this.requestGeneration++
      this.report = next
      this.loading = false
      this.error = ''
    },
    setLoading(next: boolean) {
      this.loading = next
    },
    // 环境重新发现：正式服务入口（供概览/设置的「重新发现」共用），
    // 不再依赖挂在 window 上的调试全局（IMP-04 §13.6）。Toast 仍由根壳负责。
    async refresh(): Promise<boolean> {
      const generation = ++this.requestGeneration
      this.setLoading(true)
      this.error = ''
      try {
        const report = await discoverEnvironment()
        if (generation !== this.requestGeneration) return true
        this.set(report)
        return true
      } catch {
        if (generation !== this.requestGeneration) return true
        // Failed current discovery invalidates an older MissingNode/ready claim.
        // An error is not evidence of missing system tools or bundle preparation.
        this.report = null
        try {
          const {invoke}=await import('@tauri-apps/api/core')
          const state=await invoke<{phase:string}>('runtime_preparation_status')
          if(generation!==this.requestGeneration)return true
          if(state&&['checking','preparing','verifying','extracting','assembling','activating'].includes(state.phase)){
            this.setLoading(false);return true
          }
        }catch{ /* missing preparation observation never becomes a ready fact */ }
        this.error = '集成运行程序检查未就绪，可稍后重新检查或查看安装状态。'
        this.setLoading(false)
        return false
      }
    },
  },
})
