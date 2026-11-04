/** Presentation and exit policy shared by dynamic modal callers. */
export interface ModalField { key: string; label: string; placeholder?: string; value?: string; type?: 'text' | 'password' }
export interface ModalContentSpec {
  title: string; body: string; confirmLabel?: string; primaryKind?: 'primary' | 'danger'
  cancelLabel?: string; deleteLabel?: string; resolveLabel?: string; wide?: boolean; fields?: ModalField[]
  mode?: 'view' | 'edit' | 'confirm'
  busy?: boolean
  beforeClose?: () => boolean | void | Promise<boolean | void>
}
const exitLabels = new Set(['关闭', '关闭窗口', '知道了', '返回编辑', '完成'])
export function modalMode(modal: ModalContentSpec): 'view' | 'edit' | 'confirm' {
  if (modal.mode) return modal.mode
  if (modal.fields?.length) return 'edit'
  // Historical detail callers use a close label alongside real business actions.
  if (exitLabels.has(modal.cancelLabel ?? '')) return 'view'
  return modal.confirmLabel || modal.resolveLabel || modal.deleteLabel ? 'confirm' : 'view'
}
export function modalCancelLabel(modal: ModalContentSpec) {
  return !modal.cancelLabel || exitLabels.has(modal.cancelLabel) ? '取消' : modal.cancelLabel
}
export function modalHasActions(modal: ModalContentSpec) {
  return modalMode(modal) !== 'view' || !!(modal.confirmLabel || modal.resolveLabel || modal.deleteLabel)
}
