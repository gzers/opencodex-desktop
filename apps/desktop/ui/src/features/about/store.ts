// 关于功能切片的状态与动作（IMP-04 §19.4 E：从总 store 拆出，切片四）。
import { defineStore } from 'pinia'
import { getAppAbout, type AboutAppDto } from './api'

export const useAboutStore = defineStore('about', {
  state: () => ({
    app: null as AboutAppDto | null,
  }),
  actions: {
    async load() {
      try {
        this.app = await getAppAbout()
      } catch {
        // 关于页保留构建期兜底；不臆造运行时元数据。
      }
    },
  },
})
