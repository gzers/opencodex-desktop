import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { defineComponent, h, ref } from 'vue'
import { createPinia, setActivePinia } from 'pinia'
import UiDialog from '@/components/ui/UiDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import TemplateUsageDialog from '@/features/models/TemplateUsageDialog.vue'
import NewChannelDialog from '@/features/models/NewChannelDialog.vue'
import { modelApi } from '@/features/models/api'
import { useModelsStore } from '@/features/models/store'

vi.mock('@/features/models/api', async original => ({
  ...await original<typeof import('@/features/models/api')>(),
  modelApi: { templateUsage: vi.fn(), previewNewChannel: vi.fn() },
}))
const wrappers: VueWrapper[] = []
beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
  Object.assign(HTMLDialogElement.prototype, {
    showModal(this: HTMLDialogElement) { this.setAttribute('open', '') },
    close(this: HTMLDialogElement) { this.removeAttribute('open') },
  })
})
afterEach(() => { wrappers.splice(0).forEach(w => w.unmount()); document.body.innerHTML = '' })
function key(target: EventTarget, value: string, shiftKey = false) {
  const event = new KeyboardEvent('keydown', { key: value, shiftKey, bubbles: true, cancelable: true })
  target.dispatchEvent(event)
  return event
}
function render(component: Parameters<typeof mount>[0], options = {}) {
  const wrapper = mount(component, { attachTo: document.body, ...options })
  wrappers.push(wrapper)
  return wrapper
}
describe('POST-09 public modal contract and representative consumers', () => {
  it('uses only the trailing header X for exit and removes an empty conditional action slot', async () => {
    render(UiDialog, { props: { open: true, title: '查看' }, slots: { actions: () => false } })
    await flushPromises()
    expect(document.querySelector('dialog header button:last-child')?.getAttribute('aria-label')).toBe('关闭')
    expect(document.querySelector('dialog header button')?.textContent).toBe('')
    expect(document.querySelector('dialog footer')).toBeNull()
  })

  it.each(['x', 'escape', 'cancel', 'backdrop'])('applies the same close guard for %s', async channel => {
    const guard = vi.fn(() => false)
    const w = render(UiDialog, { props: { open: true, title: '受限', beforeClose: guard } })
    await flushPromises()
    const dialog = document.querySelector('dialog')!
    if (channel === 'x') (dialog.querySelector('header button') as HTMLElement).click()
    if (channel === 'escape') key(dialog, 'Escape')
    if (channel === 'cancel') dialog.dispatchEvent(new Event('cancel', { cancelable: true }))
    if (channel === 'backdrop') dialog.dispatchEvent(new MouseEvent('click', { bubbles: true, clientX: -10, clientY: -10 }))
    await flushPromises()
    expect(guard).toHaveBeenCalledTimes(1)
    expect(w.emitted('close')).toBeUndefined()
  })

  it('routes a footer cancellation through the same guard as the header X', async () => {
    const guard = vi.fn(() => false)
    const w = render(UiDialog, { props: { open: true, title: '编辑', beforeClose: guard }, slots: { actions: ({ close }: { close: () => void }) => h('button', { onClick: close }, '取消') } })
    await flushPromises()
    ;(document.querySelector('dialog footer button') as HTMLButtonElement).click()
    ;(document.querySelector('dialog header .modal-close') as HTMLButtonElement).click()
    expect(guard).toHaveBeenCalledTimes(2); expect(w.emitted('close')).toBeUndefined()
  })

  it('keeps a legacy hideClose restriction consistent without hiding the X', async () => {
    const w = render(UiDialog, { props: { open: true, title: '旧限制', hideClose: true } })
    await flushPromises()
    expect(document.querySelector<HTMLButtonElement>('dialog header .modal-close')!.disabled).toBe(true)
    key(document.querySelector('dialog')!, 'Escape')
    document.querySelector('dialog')!.dispatchEvent(new Event('cancel', { cancelable: true }))
    expect(w.emitted('close')).toBeUndefined()
  })

  it('cycles focus in the top modal and restores its actual trigger', async () => {
    const trigger = document.createElement('button'); document.body.append(trigger); trigger.focus()
    const opened = ref(true)
    render(defineComponent({ setup: () => () => h(UiDialog, { open: opened.value, title: '编辑', onClose: () => { opened.value = false } }, { default: () => h('input'), actions: () => h('button', '保存') }) }))
    await flushPromises()
    const first = document.querySelector('dialog header button') as HTMLElement
    const last = document.querySelector('dialog footer button') as HTMLElement
    expect(document.querySelector('dialog')?.contains(document.activeElement)).toBe(true)
    last.focus(); expect(key(last, 'Tab').defaultPrevented).toBe(true); expect(document.activeElement).toBe(first)
    key(first, 'Tab', true); expect(document.activeElement).toBe(last)
    key(last, 'Escape'); await flushPromises(); expect(document.activeElement).toBe(trigger)
  })

  it('does not cancel a dialog when a later mounted menu consumes Escape', async () => {
    const show = ref(false)
    const w = render(defineComponent({ setup: () => () => h(UiDialog, { open: true, title: '编辑' }, () => show.value ? h(UiSelectField, { modelValue: 'one', label: '版本', options: [{ value: 'one', label: '一' }] }) : null) }))
    await flushPromises(); show.value = true; await flushPromises()
    const trigger = document.querySelector('.select-trigger') as HTMLElement
    trigger.click(); await flushPromises(); key(trigger, 'Escape'); await flushPromises()
    expect(w.findComponent(UiDialog).emitted('close')).toBeUndefined()
    expect(document.querySelector('[role=option]')).toBeNull()
  })

  it('returns focus to a pointer trigger that WebKit did not focus on activation', async () => {
    const opened = ref(false)
    render(defineComponent({ setup: () => () => [h('button', { id: 'webkit-pointer-trigger', onClick: () => { opened.value = true } }, '打开'), h(UiDialog, { open: opened.value, title: '指针来源', onClose: () => { opened.value = false } })] }))
    await flushPromises()
    const trigger = document.querySelector<HTMLButtonElement>('#webkit-pointer-trigger')!
    trigger.click(); await flushPromises()
    key(document.querySelector('dialog')!, 'Escape'); await flushPromises()
    expect(document.activeElement).toBe(trigger)
  })

  it('does not let an old asynchronous exit guard resolve a reopened window', async () => {
    let release!: (allowed: boolean) => void
    const guard = vi.fn(() => new Promise<boolean>(resolve => { release = resolve }))
    const w = render(UiDialog, { props: { open: true, title: '异步守卫', beforeClose: guard } })
    await flushPromises(); key(document.querySelector('dialog')!, 'Escape')
    await w.setProps({ open: false } as never); await flushPromises(); await w.setProps({ open: true } as never); await flushPromises()
    release(true); await flushPromises()
    expect(w.emitted('close')).toBeUndefined()
    expect(document.querySelector<HTMLButtonElement>('dialog header .modal-close')!.disabled).toBe(false)
  })

  it('keeps the usage retry action while closing read-only loading without a footer close', async () => {
    useModelsStore().snapshot = { libraryDigest: 'lib' } as never
    vi.mocked(modelApi.templateUsage).mockReturnValue(new Promise(() => {}))
    const w = render(TemplateUsageDialog, { props: { templateId: 't', revisionId: 'r' } })
    await flushPromises()
    expect([...document.querySelectorAll('dialog footer button')].map(b => b.textContent)).toEqual(['重新读取'])
    key(document.querySelector('dialog')!, 'Escape')
    expect(w.emitted('close')).toHaveLength(1)
  })

  it.each(['x', 'escape', 'cancel'])('cancels an edited new-channel buffer through %s without inserting a draft or key', async channel => {
    const store = useModelsStore()
    store.snapshot = { libraryDigest: 'lib', policy: { channels: [] } } as never
    store.channelDraft = { schemaVersion: 1, templateRefs: [], channels: [] }
    const w = render(NewChannelDialog)
    await flushPromises()
    const inputs = document.querySelectorAll<HTMLInputElement>('dialog input')
    inputs[0]!.value = 'unsaved'; inputs[0]!.dispatchEvent(new Event('input', { bubbles: true }))
    const password = document.querySelector<HTMLInputElement>('dialog input[type=password]')!
    password.value = 'fixture-unsaved-key'; password.dispatchEvent(new Event('input', { bubbles: true }))
    if (channel === 'x') (document.querySelector('dialog header button') as HTMLElement).click()
    if (channel === 'escape') key(password, 'Escape')
    if (channel === 'cancel') ([...document.querySelectorAll('dialog footer button')].find(b => b.textContent === '取消') as HTMLElement).click()
    await flushPromises()
    expect(w.emitted('close')).toHaveLength(1)
    expect(store.channelDraft!.channels).toEqual([])
    expect(store.credentialEdits).toEqual({})
    expect(modelApi.previewNewChannel).not.toHaveBeenCalled()
  })
})
