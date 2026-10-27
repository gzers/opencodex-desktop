import { defineStore } from 'pinia'

/** 本次保存的瞬时反馈；不投影为任务、持久通知或客户端待处理状态。 */
export const useModelSaveFeedback = defineStore('model-save-feedback', {
  state: () => ({ open: false, revision: 0 }),
  actions: {
    showSaved() {
      this.revision += 1
      this.open = true
    },
    dismiss() {
      this.open = false
    },
  },
})
