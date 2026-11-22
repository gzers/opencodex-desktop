import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { DOMWrapper, flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent, h, ref } from 'vue'
import AppModal from '@/components/AppModal.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import { useAppStore } from '@/stores/app'
import { installNativeDialogStub } from './helpers/nativeDialog'

const mounted: VueWrapper[] = []
let pinia: ReturnType<typeof createPinia>
beforeEach(() => { pinia = createPinia(); setActivePinia(pinia); installNativeDialogStub() })
afterEach(() => { mounted.splice(0).forEach(w => w.unmount()); document.body.innerHTML = '' })
function render() { const w = mount(AppModal, { global: { plugins: [pinia] }, attachTo: document.body }); mounted.push(w); return w }
function surface() { return new DOMWrapper(document.body).get('dialog.app-modal-dialog') }
async function exit(channel: string) {
  const dialog = surface()
  if (channel === 'x') await dialog.get('header .modal-close').trigger('click')
  if (channel === 'escape') dialog.element.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }))
  if (channel === 'backdrop') await dialog.trigger('click', { clientX: -10, clientY: -10 })
  if (channel === 'cancel') await dialog.get('.modal-actions .ghost').trigger('click')
  await flushPromises()
}

describe('POST-09 dynamic confirmation and input lifecycle', () => {
  it('renders a view without an exit footer and preserves its real copy action', async () => {
    render(); const app = useAppStore()
    app.openModal({ title: '详情', body: '<p>只读</p>', cancelLabel: '关闭' }); await flushPromises()
    expect(surface().find('.modal-actions').exists()).toBe(false)
    expect(surface().findAll('header .modal-close')).toHaveLength(1)
    app.openModal({ title: '详情', body: '<p>只读</p>', cancelLabel: '关闭', confirmLabel: '复制地址' }); await flushPromises()
    expect(surface().findAll('.modal-actions button').map(b => b.text())).toEqual(['复制地址'])
  })

  it.each(['x', 'escape', 'backdrop', 'cancel'])('returns null through %s with no confirmation or write', async channel => {
    render(); const app = useAppStore(); const confirm = vi.fn()
    const pending = app.openInputModal({ title: '编辑', body: '', fields: [{ key: 'name', label: '名称', value: 'before' }], confirmLabel: '保存', onConfirm: confirm })
    await flushPromises(); await surface().get('input').setValue('unsaved')
    await exit(channel)
    await expect(pending).resolves.toBeNull()
    expect(confirm).not.toHaveBeenCalled(); expect(app.modal).toBeNull()
  })

  it.each(['x', 'escape', 'backdrop', 'cancel'])('keeps input intact when the common guard refuses %s', async channel => {
    render(); const app = useAppStore(); const guard = vi.fn(() => false); const cancel = vi.fn()
    app.openModal({ title: '编辑', body: '', fields: [{ key: 'name', label: '名称' }], confirmLabel: '保存', beforeClose: guard, onCancel: cancel })
    await flushPromises(); await surface().get('input').setValue('keep-me')
    await exit(channel)
    expect(guard).toHaveBeenCalledTimes(1); expect(cancel).not.toHaveBeenCalled()
    expect(surface().get<HTMLInputElement>('input').element.value).toBe('keep-me')
    expect(app.modal?.title).toBe('编辑')
  })

  it('cannot let an old asynchronous guard cancel a replacement modal', async () => {
    render(); const app = useAppStore(); let release!: (allowed: boolean) => void
    const oldCancel = vi.fn(); const newCancel = vi.fn()
    app.openModal({ title: '旧', body: '', confirmLabel: '保存', beforeClose: () => new Promise(resolve => { release = resolve }), onCancel: oldCancel })
    await flushPromises(); await exit('escape'); await exit('backdrop')
    expect(surface().get('header .modal-close').attributes('disabled')).toBeDefined()
    app.openModal({ title: '新', body: '', confirmLabel: '保存', onCancel: newCancel })
    await flushPromises(); release(true); await flushPromises()
    expect(app.modal?.title).toBe('新'); expect(newCancel).not.toHaveBeenCalled(); expect(oldCancel).not.toHaveBeenCalled()
    await exit('x'); expect(newCancel).toHaveBeenCalledTimes(1)
  })

  it('lets a nested AppModal consume Escape before its native owner and returns focus to that trigger', async () => {
    const open = ref(true); const parentClosed = vi.fn()
    const host = defineComponent({ setup: () => () => [h(UiDialog, { open: open.value, title: '父层', onClose: parentClosed }, () => h('button', { id: 'nested-trigger', onClick: () => useAppStore().openModal({ title: '子层', body: '确认', confirmLabel: '确认' }) }, '嵌套确认')), h(AppModal)] })
    const w = mount(host, { global: { plugins: [pinia] }, attachTo: document.body }); mounted.push(w)
    await flushPromises(); const trigger = document.querySelector<HTMLElement>('#nested-trigger')!; trigger.focus(); trigger.click(); await flushPromises()
    await exit('escape')
    expect(useAppStore().modal).toBeNull(); expect(parentClosed).not.toHaveBeenCalled(); expect(document.activeElement).toBe(trigger)
    trigger.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }))
    expect(parentClosed).toHaveBeenCalledTimes(1)
  })
})
