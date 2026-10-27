import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent } from 'vue'
import TaskProgressHost from '@/components/TaskProgressHost.vue'
import AppTopbar from '@/components/AppTopbar.vue'
import NotificationCenter from '@/features/notifications/components/NotificationCenter.vue'
import { useModelSaveFeedback } from '@/app/feedback/modelSave'
import { useFeedbackStore } from '@/app/feedback/store'
import { MAX_VISIBLE_TASKS, useOperationTasks } from '@/app/feedback/tasks'
import { useNotificationsStore } from '@/features/notifications/store'
import { modelApi } from '@/features/models/api'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/event', () => ({ listen: vi.fn().mockResolvedValue(() => {}) }))

const Harness = defineComponent({
  components: { TaskProgressHost, AppTopbar, NotificationCenter },
  setup() { return { notifications: useNotificationsStore() } },
  template: `<div><AppTopbar compact /><div class="notif-layer"><NotificationCenter :open="notifications.panelOpen" @close="notifications.panelOpen=false" @collapse="notifications.panelOpen=false" /><TaskProgressHost /></div></div>`,
})
let wrapper: VueWrapper

// jsdom 没有原生模态层；保留真实 UiDialog，只补浏览器的 open 属性操作。
beforeAll(() => {
  Object.defineProperty(HTMLDialogElement.prototype, 'showModal', { configurable: true, value() { this.setAttribute('open', '') } })
  Object.defineProperty(HTMLDialogElement.prototype, 'close', { configurable: true, value() { this.removeAttribute('open') } })
  Object.defineProperty(window, 'matchMedia', { configurable: true, value: () => ({ matches: false, addEventListener() {}, removeEventListener() {} }) })
})
beforeEach(() => {
  const pinia = createPinia()
  setActivePinia(pinia)
  wrapper = mount(Harness, { attachTo: document.body, global: { plugins: [pinia] } })
})
afterEach(() => {
  wrapper.unmount()
  useOperationTasks().dispose()
  vi.restoreAllMocks()
  document.body.innerHTML = ''
})
const button = (text: string) => wrapper.findAll('button').find(node => node.text() === text)!
async function showSaved() { useModelSaveFeedback().showSaved(); await flushPromises() }
async function openRestart() { await showSaved(); await button('立即重启 Codex').trigger('click'); await flushPromises() }
function dialogButton(text: string) { return Array.from(document.querySelectorAll<HTMLButtonElement>('dialog[open] button')).find(node => node.textContent?.trim() === text)! }

describe('一次保存反馈的真实组件交互（模拟 API，不代表真机重启）', () => {
  it('结果在全局宿主使用原型任务卡，默认聚焦立即；稍后结束且铃铛不复活', async () => {
    await showSaved()
    expect(wrapper.findAll('.notif-layer .process-progress')).toHaveLength(1)
    expect(wrapper.find('.save-result').exists()).toBe(false)
    expect(document.activeElement).toBe(button('立即重启 Codex').element)
    expect(wrapper.get('.process-progress').find('.notify-min').exists()).toBe(false)
    await button('稍后').trigger('click')
    expect(useModelSaveFeedback().open).toBe(false)
    expect(wrapper.find('.process-progress').exists()).toBe(false)
    await wrapper.get('.notification-btn').trigger('click')
    expect(wrapper.find('.process-progress').exists()).toBe(false)
    expect(useOperationTasks().tasks).toHaveLength(0)
    expect(useNotificationsStore().list).toHaveLength(0)
  })

  it('Esc 关闭本次反馈，重新挂载也不留下待重启或持久通知', async () => {
    await showSaved()
    window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))
    await flushPromises()
    expect(useModelSaveFeedback().open).toBe(false)
    wrapper.unmount()
    wrapper = mount(Harness, { attachTo: document.body })
    expect(wrapper.find('.process-progress').exists()).toBe(false)
    expect(wrapper.text()).not.toContain('待重启')
    expect(useNotificationsStore().list).toHaveLength(0)
  })

  it('重启前只打开公共确认弹窗，取消不调用接口；确认弹窗 Esc 彻底关闭反馈', async () => {
    const refresh = vi.spyOn(modelApi, 'refreshCache')
    await openRestart()
    expect(document.querySelector('dialog.ui-dialog[open]')?.textContent).toContain('立即结束当前正在进行的 Codex 对话')
    expect(refresh).not.toHaveBeenCalled()
    dialogButton('取消').click()
    await flushPromises()
    expect(document.querySelector('dialog[open]')).toBeNull()
    expect(useModelSaveFeedback().open).toBe(true)
    await button('立即重启 Codex').trigger('click')
    await flushPromises()
    document.querySelector('dialog[open]')!.dispatchEvent(new Event('cancel', { cancelable: true }))
    await flushPromises()
    expect(useModelSaveFeedback().open).toBe(false)
    expect(refresh).not.toHaveBeenCalled()
  })

  it.each([
    { success: true, restarted: true, summary: '官方已接受重启请求。' },
    { success: false, restarted: false, summary: 'Codex 未重新启动：官方返回拒绝。' },
    { success: true, restarted: false, summary: '缓存已同步；未执行桌面重启。' },
  ])('确认后如实显示返回 summary：$summary', async result => {
    const refresh = vi.spyOn(modelApi, 'refreshCache').mockResolvedValue(result)
    const toast = vi.spyOn(useFeedbackStore(), 'showToast')
    await openRestart()
    dialogButton('确认重启').click()
    dialogButton('确认重启').click()
    await flushPromises()
    expect(refresh).toHaveBeenCalledExactlyOnceWith(true)
    expect(toast).toHaveBeenCalledWith(result.summary, { kind: result.success ? 'info' : 'error' })
    expect(useModelSaveFeedback().open).toBe(false)
    expect(wrapper.find('.process-progress').exists()).toBe(false)
    expect(useNotificationsStore().list).toHaveLength(0)
    expect(useOperationTasks().tasks).toHaveLength(0)
  })

  it('接口异常也结束反馈，仅显示真实错误', async () => {
    vi.spyOn(modelApi, 'refreshCache').mockRejectedValue(new Error('本机重启连接断开'))
    await openRestart()
    dialogButton('确认重启').click()
    await flushPromises()
    expect(useFeedbackStore().toast).toContain('本机重启连接断开')
    expect(useModelSaveFeedback().open).toBe(false)
    expect(useNotificationsStore().list).toHaveLength(0)
  })

  it('新保存替换旧结果后，旧重启请求返回不能关闭新结果', async () => {
    let finish!: (result: Awaited<ReturnType<typeof modelApi.refreshCache>>) => void
    vi.spyOn(modelApi, 'refreshCache').mockReturnValue(new Promise(resolve => { finish = resolve }))
    await openRestart()
    dialogButton('确认重启').click()
    await flushPromises()
    await showSaved()
    finish({ success: true, restarted: true, summary: '旧请求已结束' })
    await flushPromises()
    expect(useModelSaveFeedback().open).toBe(true)
    expect(wrapper.findAll('.process-progress')).toHaveLength(1)
    await button('稍后').trigger('click')
    expect(useModelSaveFeedback().open).toBe(false)
  })
})

describe('收起任务只在既有铃铛面板按需查看', () => {
  it('点铃铛不全展开，面板分类与逐条查看都不改变收起状态', async () => {
    const operations = useOperationTasks()
    const ids = Array.from({ length: MAX_VISIBLE_TASKS + 3 }, () => crypto.randomUUID())
    for (const id of ids) operations.apply({ id, kind: 'research', phase: 'running' })
    await flushPromises()
    const collapsed = operations.tasks.filter(task => task.collapsed).map(task => task.id)
    expect(collapsed).toHaveLength(3)
    expect(wrapper.find('.task-reopen').exists()).toBe(false)
    await wrapper.get('.notification-btn').trigger('click')
    expect(wrapper.get('.notification-panel').classes()).toContain('show')
    expect(wrapper.findAll('[data-operation-task]')).toHaveLength(collapsed.length)
    expect(wrapper.findAll('.process-progress')).toHaveLength(MAX_VISIBLE_TASKS)
    expect(operations.tasks.filter(task => task.collapsed).map(task => task.id)).toEqual(collapsed)
    const category = wrapper.findAll('[role=tab]').find(node => node.text().startsWith('系统'))!
    await category.trigger('click')
    expect(wrapper.findAll('[data-operation-task]')).toHaveLength(collapsed.length)
    await wrapper.get('[data-operation-task] .n-main').trigger('click')
    expect(useNotificationsStore().panelOpen).toBe(false)
    expect(operations.tasks.filter(task => task.collapsed).map(task => task.id)).toEqual(collapsed)
    expect(wrapper.findAll('.process-progress')).toHaveLength(MAX_VISIBLE_TASKS)
    expect(useNotificationsStore().list).toHaveLength(0)
  })
})

describe('清理沿用公共 UiDialog', () => {
  it('点击清理先确认范围；取消/Esc 不清理，确认才调用既有动作', async () => {
    const notifications = useNotificationsStore()
    notifications.list = [
      { id: 'pending', kind: 'warning', category: 'system', title: '待处理', detail: '原因', read: false, time: '' },
      { id: 'done', kind: 'info', category: 'system', title: '已处理', detail: '结果', resolved: true, read: true, time: '' },
    ]
    const clear = vi.spyOn(notifications, 'clearAll').mockImplementation(async () => { notifications.list = [] })
    await wrapper.get('.notification-btn').trigger('click')
    await button('清理').trigger('click')
    await flushPromises()
    expect(wrapper.find('.notification-confirm').exists()).toBe(false)
    expect(document.querySelector('dialog.ui-dialog[open]')?.textContent).toContain('将清理全部 2 条通知（其中 1 条未处理）')
    expect(clear).not.toHaveBeenCalled()
    document.querySelector<HTMLParagraphElement>('dialog[open] p')!.click()
    await flushPromises()
    expect(notifications.panelOpen).toBe(true)
    expect(document.querySelector('dialog[open]')).not.toBeNull()
    dialogButton('取消').click()
    await flushPromises()
    expect(clear).not.toHaveBeenCalled()
    expect(notifications.panelOpen).toBe(true)
    await button('清理').trigger('click')
    await flushPromises()
    document.querySelector('dialog[open]')!.dispatchEvent(new Event('cancel', { cancelable: true }))
    await flushPromises()
    expect(clear).not.toHaveBeenCalled()
    await button('清理').trigger('click')
    await flushPromises()
    dialogButton('确认清理').click()
    await flushPromises()
    expect(clear).toHaveBeenCalledOnce()
    expect(notifications.list).toHaveLength(0)
    expect(document.querySelector('dialog[open]')).toBeNull()
  })
})
