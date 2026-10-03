// 应用级模态框（IMP-04 §19.4 E：从总 store 拆出，切片八）。
// 承载可编辑输入型对话框，替代 WKWebView 中不可用的 window.prompt。
import { defineStore } from 'pinia'
import type { ModalState } from '@/stores/app'

export const useModalStore = defineStore('modal', {
  state: () => ({
    current: null as ModalState | null,
  }),
  actions: {
    open(modal: ModalState) {
      this.current = { cancelLabel: '取消', ...modal }
    },
    // 打开需要文本输入的对话框，返回字段取值；取消（按钮、Esc、遮罩）返回 null。
    openInput(modal: ModalState): Promise<Record<string, string> | null> {
      return new Promise(resolve => {
        this.open({
          ...modal,
          onConfirm: values => resolve(values ?? {}),
          onCancel: () => resolve(null),
        })
      })
    },
    resolve(result: 'confirm' | 'cancel' | 'delete' | 'resolve', values?: Record<string, string>) {
      const modal = this.current
      this.current = null
      if (!modal) return
      if (result === 'confirm') modal.onConfirm?.(values)
      if (result === 'cancel') modal.onCancel?.()
      if (result === 'delete') modal.onDelete?.()
      if (result === 'resolve') modal.onResolve?.()
    },
  },
})
