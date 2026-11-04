// 应用级模态框（IMP-04 §19.4 E：从总 store 拆出，切片八）。
// 承载可编辑输入型对话框，替代 WKWebView 中不可用的 window.prompt。
import { defineStore } from 'pinia'
import type { ModalState } from '@/stores/app'

export const useModalStore = defineStore('modal', {
  state: () => ({
    current: null as ModalState | null,
    generation:0,
    closing: false,
  }),
  actions: {
    open(modal: ModalState) {
      this.generation++
      this.closing = false
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
      if (!modal || modal.busy || this.closing) return
      const generation = this.generation
      const finish = (allowed: boolean | void) => {
        // A late guard from an old window must never resolve a replacement.
        if (allowed === false || this.generation !== generation || this.current !== modal || modal.busy) return
        this.generation++
        this.current = null
        this.closing = false
        if (result === 'confirm') modal.onConfirm?.(values)
        if (result === 'cancel') modal.onCancel?.()
        if (result === 'delete') modal.onDelete?.()
        if (result === 'resolve') modal.onResolve?.()
      }
      try {
        const allowed = result === 'cancel' ? modal.beforeClose?.() : undefined
        if (allowed instanceof Promise) {
          this.closing = true
          void allowed.then(finish).catch(() => {}).finally(() => {
            if (this.generation === generation) this.closing = false
          })
        } else finish(allowed)
      } catch { /* Guard errors keep the current modal and its input intact. */ }
    },
  },
})
