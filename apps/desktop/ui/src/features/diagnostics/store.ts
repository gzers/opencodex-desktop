// 诊断功能切片的状态与动作（IMP-04 §19.4 E：从总 store 拆出，切片四）。
import { defineStore } from 'pinia'
import { runDoctor, type DoctorDto } from './doctor'

export const useDiagnosticsStore = defineStore('diagnostics', {
  state: () => ({
    report: null as DoctorDto | null,
    loading: false,
    error: false,
  }),
  actions: {
    async loadReport() {
      if (this.loading) return
      this.loading = true
      try {
        this.report = await runDoctor()
        this.error = false
      } catch {
        // IPC 失败时保留上一次 Doctor 结果；不臆造官方输出。
        this.error = true
      } finally {
        this.loading = false
      }
    },
  },
})
