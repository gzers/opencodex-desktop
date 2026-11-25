// 环境发现功能切片的状态与动作（IMP-04 §19.4 E：从总 store 拆出，切片十一）。
import { defineStore } from 'pinia'
import { discoverEnvironment, type EnvironmentReport } from './api'
import { usePreparationStore } from './preparation'
import { buildEnvironmentPresentation, type EnvironmentPresentation } from './presentation'

export const useEnvironmentStore = defineStore('environment', {
  state: () => ({
    report: null as EnvironmentReport | null,
    loading: false,
    error: '',
    requestGeneration: 0,
    checkedPreparationAttempt: null as string | null,
  }),
  getters: {
    checkingPreparation(): boolean {
      const preparation = usePreparationStore()
      return preparation.active || preparation.retrying ||
        (preparation.suppliedAttempt !== null && preparation.suppliedAttempt !== this.checkedPreparationAttempt)
    },
    ready(): boolean {
      const preparation = usePreparationStore()
      return !this.checkingPreparation &&
        (!preparation.needsAttention || preparation.snapshot?.attemptId === this.checkedPreparationAttempt) &&
        !this.loading && !this.error && this.report?.gate === 'ready'
    },
    presentation(): EnvironmentPresentation {
      return buildEnvironmentPresentation(this.report, this.loading || this.checkingPreparation, this.error)
    },
  },
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
      const preparation = usePreparationStore()
      this.checkedPreparationAttempt = preparation.active ? null : preparation.snapshot?.attemptId ?? null
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
        await preparation.initialObservation()
        if (generation !== this.requestGeneration) return true
        this.setLoading(false)
        if (preparation.active && !preparation.error) return true
        this.error = '集成运行程序检查未就绪，可稍后重新检查或查看安装状态。'
        return false
      }
    },
  },
})
