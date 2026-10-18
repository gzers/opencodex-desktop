import { mount } from '@vue/test-utils'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { nextTick } from 'vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import UiDialog from '@/components/ui/UiDialog.vue'
import StickyTabs from '@/components/layout/StickyTabs.vue'
afterEach(() => { document.body.innerHTML = ''; vi.unstubAllGlobals(); vi.restoreAllMocks() })
describe('model page public primitives', () => {
  it('selection emits the value, closes the menu and restores trigger focus', async () => {
    const wrapper = mount(UiSelectField, { attachTo: document.body, props: { modelValue: 'a', label: '保存版本', options: [{ value: 'a', label: '版本A' }, { value: 'b', label: '版本B' }] } })
    await wrapper.get('button').trigger('click'); await nextTick()
    const option = [...document.querySelectorAll<HTMLButtonElement>('[role=option]')].find(node => node.textContent === '版本B')!
    option.click(); await nextTick()
    expect(wrapper.emitted('update:modelValue')).toEqual([['b']]); expect(document.querySelector('[role=listbox]')).toBeNull()
    expect(document.activeElement).toBe(wrapper.get('.select-trigger').element); wrapper.unmount()
  })
  it('a busy dialog rejects Escape dismissal but a normal dialog reports close', async () => {
    HTMLDialogElement.prototype.showModal = function () { this.setAttribute('open', '') }
    HTMLDialogElement.prototype.close = function () { this.removeAttribute('open') }
    const wrapper = mount(UiDialog, { props: { open: true, title: '版本管理', busy: true }, slots: { default: '<p>同一份保存内容</p>' } })
    await nextTick(); const dialog = document.querySelector('dialog')!
    dialog.dispatchEvent(new Event('cancel', { cancelable: true })); expect(wrapper.emitted('close')).toBeUndefined()
    await wrapper.setProps({ busy: false }); dialog.dispatchEvent(new Event('cancel', { cancelable: true }))
    expect(wrapper.emitted('close')).toHaveLength(1); wrapper.unmount()
  })
  it('sticky offsets follow wrapped height changes and are removed on unmount', async () => {
    let resize!: ResizeObserverCallback
    const disconnect = vi.fn()
    vi.stubGlobal('ResizeObserver', class { constructor(callback: ResizeObserverCallback) { resize = callback } observe() {} disconnect = disconnect })
    const wrapper = mount(StickyTabs, { attachTo: document.body, slots: { default: '<div role=tablist>Tab</div>' } })
    const bar = wrapper.get('.sticky-page-tabs').element as HTMLElement
    Object.defineProperty(bar, 'offsetHeight', { configurable: true, value: 72 })
    resize([], {} as ResizeObserver); expect(bar.parentElement!.style.getPropertyValue('--page-tabs-height')).toBe('72px')
    const parent = bar.parentElement!; wrapper.unmount()
    expect(parent.style.getPropertyValue('--page-tabs-height')).toBe(''); expect(disconnect).toHaveBeenCalled()
  })
})
