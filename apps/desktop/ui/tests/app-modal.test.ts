import { createPinia, setActivePinia } from 'pinia'
import { mount, type VueWrapper } from '@vue/test-utils'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import AppModal from '@/components/AppModal.vue'
import App from '@/App.vue'
import { useAppStore } from '@/stores/app'

const invoke = vi.fn()

vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}))

Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: () => ({ matches: false, addEventListener: () => {}, removeEventListener: () => {} }),
})

function mountModal() {
  return mount(AppModal, { global: { plugins: [createPinia()] }, attachTo: document.body })
}

async function flush(wrapper: VueWrapper) {
  await wrapper.vm.$nextTick()
  await wrapper.vm.$nextTick()
  await new Promise((resolve) => setTimeout(resolve, 0))
}

function open(confirmable = true) {
  const app = useAppStore()
  const onConfirm = vi.fn()
  const onCancel = vi.fn()
  app.openModal({
    title: '确认操作',
    body: '<p>正文</p>',
    confirmLabel: confirmable ? '确认' : undefined,
    onConfirm,
    onCancel,
  })
  return { app, onConfirm, onCancel }
}

describe('app modal', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    invoke.mockResolvedValue(undefined)
    document.body.innerHTML = ''
  })

  it('renders a labelled modal dialog with modal semantics', async () => {
    const wrapper = mountModal()
    const { app } = open()
    await flush(wrapper)

    const dialog = wrapper.find('.modal')
    expect(dialog.attributes('role')).toBe('dialog')
    expect(dialog.attributes('aria-modal')).toBe('true')
    const labelledBy = dialog.attributes('aria-labelledby')
    expect(labelledBy).toBeTruthy()
    expect(wrapper.find('h3').attributes('id')).toBe(labelledBy)
    expect(wrapper.find('h3').text()).toBe('确认操作')
    app.resolveModal('cancel')
  })

  it('cancels on Escape and never confirms', async () => {
    const wrapper = mountModal()
    const { app, onConfirm, onCancel } = open()
    await flush(wrapper)

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', cancelable: true }))
    await flush(wrapper)

    expect(app.modal).toBeNull()
    expect(onCancel).toHaveBeenCalledTimes(1)
    expect(onConfirm).not.toHaveBeenCalled()
    expect(wrapper.find('.modal').exists()).toBe(false)
  })

  it('ignores Escape when no modal is open', async () => {
    mountModal()
    expect(() => window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))).not.toThrow()
  })

  it('cancels when the mask itself is clicked but not the dialog body', async () => {
    const wrapper = mountModal()
    const { onConfirm, onCancel } = open()
    await flush(wrapper)

    await wrapper.find('.modal').trigger('click')
    expect(onCancel).not.toHaveBeenCalled()

    await wrapper.find('.modal-mask').trigger('click')
    await flush(wrapper)
    expect(onCancel).toHaveBeenCalledTimes(1)
    expect(onConfirm).not.toHaveBeenCalled()
  })

  it('moves focus into the dialog on open and restores it on close', async () => {
    const wrapper = mountModal()
    const source = document.createElement('button')
    source.textContent = '来源'
    document.body.appendChild(source)
    source.focus()

    const { app } = open()
    await flush(wrapper)

    const primary = wrapper.find('.modal .btn.primary').element
    expect(document.activeElement).toBe(primary)
    expect(wrapper.find('.modal').element.contains(document.activeElement)).toBe(true)

    app.resolveModal('confirm')
    await flush(wrapper)

    expect(document.activeElement).toBe(source)
  })

  it('keeps Tab focus cycling inside the dialog', async () => {
    const wrapper = mountModal()
    open()
    await flush(wrapper)

    const buttons = wrapper.findAll('.modal .btn')
    expect(buttons.length).toBeGreaterThan(1)
    const first = buttons[0].element as HTMLElement
    const last = buttons[buttons.length - 1].element as HTMLElement

    last.focus()
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', cancelable: true }))
    expect(document.activeElement).toBe(first)

    first.focus()
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Tab', shiftKey: true, cancelable: true }))
    expect(document.activeElement).toBe(last)
  })

  // 回归：WKWebView 不支持 window.prompt；拓展页改用应用内输入型对话框。
  it('renders input fields, prefills values and returns them on confirm', async () => {
    const wrapper = mountModal()
    const app = useAppStore()
    const pending = app.openInputModal({
      title: '编辑 MCP 服务器',
      body: '<p>修改启动命令</p>',
      fields: [
        { key: 'name', label: 'MCP 名称', placeholder: '例如 context7' },
        { key: 'command', label: '启动命令', value: 'npx -y existing' },
      ],
      confirmLabel: '下一步',
    })
    await flush(wrapper)

    const inputs = wrapper.findAll('.modal-field input')
    expect(inputs.length).toBe(2)
    expect((inputs[0].element as HTMLInputElement).value).toBe('')
    expect((inputs[1].element as HTMLInputElement).value).toBe('npx -y existing')
    // 焦点落在首个输入框，便于直接键入。
    expect(document.activeElement).toBe(inputs[0].element)

    await inputs[0].setValue('audit-mcp')
    await wrapper.find('.modal .btn.primary').trigger('click')
    await expect(pending).resolves.toEqual({ name: 'audit-mcp', command: 'npx -y existing' })
    expect(app.modal).toBeNull()
  })

  it('resolves null when an input dialog is cancelled', async () => {
    const wrapper = mountModal()
    const app = useAppStore()
    const pending = app.openInputModal({
      title: '恢复 Skill',
      body: '<p>输入名称</p>',
      fields: [{ key: 'name', label: 'Skill 名称' }],
      confirmLabel: '恢复',
    })
    await flush(wrapper)

    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', cancelable: true }))
    await flush(wrapper)
    await expect(pending).resolves.toBeNull()
    expect(app.modal).toBeNull()
  })
})

describe('notification layer vs modal stacking', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    invoke.mockReset()
    invoke.mockResolvedValue(undefined)
    window.location.hash = '#overview'
  })

  // 回归：任务卡比遮罩更宽，且通知层 z-index(80) 高于模态遮罩(50)，
  // 对话框打开时任务卡会盖住居中对话框的内容。通知层必须在对话框打开时降到遮罩之下。
  it('drops the notification layer below the modal mask while a dialog is open', async () => {
    const pinia = createPinia()
    setActivePinia(pinia)
    const wrapper = mount(App, { global: { plugins: [pinia] } })
    await new Promise(resolve => setTimeout(resolve, 0))
    await wrapper.vm.$nextTick()
    const layer = wrapper.find('.notif-layer')
    expect(layer.exists()).toBe(true)
    expect(layer.classes()).not.toContain('under-modal')

    const app = useAppStore()
    app.openModal({ title: '确认操作', body: '<p>正文</p>', confirmLabel: '确认' })
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.notif-layer').classes()).toContain('under-modal')

    app.resolveModal('cancel')
    await wrapper.vm.$nextTick()
    expect(wrapper.find('.notif-layer').classes()).not.toContain('under-modal')
  })
})
