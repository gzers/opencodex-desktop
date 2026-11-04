import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { defineComponent } from 'vue'
import SaveResultNotice from '@/features/models/SaveResultNotice.vue'
import NotificationCountdown from '@/components/patterns/NotificationCountdown.vue'
import { useModelSaveFeedback } from '@/app/feedback/modelSave'
import type { Countdown } from '@/app/feedback/countdown'
import { modelApi } from '@/features/models/api'
import { installNativeDialogStub } from './helpers/nativeDialog'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
const Harness = defineComponent({
  components: { SaveResultNotice },
  setup: () => ({ saved: useModelSaveFeedback() }),
  template: '<div><input aria-label="继续编辑" /><SaveResultNotice v-if="saved.open" :key="saved.revision" :open="saved.open" @close="saved.dismiss()" /></div>',
})
let wrapper: VueWrapper
let hidden = false
beforeAll(installNativeDialogStub)
beforeEach(() => {
  vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout', 'performance', 'Date'] })
  hidden = false
  vi.spyOn(document, 'hidden', 'get').mockImplementation(() => hidden)
  setActivePinia(createPinia())
  vi.spyOn(modelApi, 'refreshCache').mockResolvedValue({ success: true, restarted: true, summary: '模拟重启结果' })
  wrapper = mount(Harness, { attachTo: document.body })
  wrapper.get('input').element.focus()
})
afterEach(() => { wrapper.unmount(); vi.restoreAllMocks(); vi.useRealTimers(); document.body.innerHTML = '' })
const button = (text: string) => wrapper.findAll('button').find(node => node.text() === text)!
const timer = () => wrapper.getComponent(NotificationCountdown).props('countdown') as Countdown
async function saved() { useModelSaveFeedback().showSaved(); await flushPromises() }
async function tick(ms: number) { vi.advanceTimersByTime(ms); await flushPromises() }
async function confirm() { await button('立即重启 Codex').trigger('click'); await flushPromises() }

describe('POST01 保存通知与公共确认框', () => {
  it('保留编辑焦点，8秒条和到期同源；到期不重启', async () => {
    await saved()
    expect(document.activeElement).toBe(wrapper.get('input').element)
    expect(timer().state()).toMatchObject({ phase: 'running', remainingMs: 8000, ratio: 1 })
    await tick(2000)
    expect(timer().state()).toMatchObject({ remainingMs: 6000, ratio: .75 })
    await tick(5999); expect(useModelSaveFeedback().open).toBe(true)
    await tick(1); expect(useModelSaveFeedback().open).toBe(false)
    expect(modelApi.refreshCache).not.toHaveBeenCalled()
  })

  it.each(['X', 'Escape', '遮罩', '取消'])('%s只退出确认，保留剩余时间并续计', async exit => {
    await saved(); await tick(2000); await confirm()
    expect(timer().state()).toMatchObject({ phase: 'paused', remainingMs: 6000 })
    await tick(30000)
    const dialog = document.querySelector<HTMLDialogElement>('dialog[open]')!
    expect(dialog).not.toBeNull()
    if (exit === 'X') dialog.querySelector<HTMLButtonElement>('.modal-close')!.click()
    else if (exit === 'Escape') dialog.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }))
    else if (exit === '遮罩') dialog.dispatchEvent(new MouseEvent('click', { clientX: -1, clientY: -1, bubbles: true }))
    else [...dialog.querySelectorAll('button')].find(node => node.textContent === '取消')!.click()
    await flushPromises()
    expect(document.querySelector('dialog[open]')).toBeNull()
    expect(useModelSaveFeedback().open).toBe(true)
    expect(document.activeElement).toBe(button('立即重启 Codex').element)
    expect(timer().state()).toMatchObject({ phase: 'running', remainingMs: 6000 })
    expect(modelApi.refreshCache).not.toHaveBeenCalled()
    await tick(6000); expect(useModelSaveFeedback().open).toBe(false)
  })

  it('主动悬停、键盘进入与页面隐藏逐一解除后才续计', async () => {
    await saved(); await tick(1000)
    const card = wrapper.get('.process-progress')
    await card.trigger('pointerenter')
    button('立即重启 Codex').element.focus()
    hidden = true; document.dispatchEvent(new Event('visibilitychange'))
    await card.trigger('pointerleave'); await tick(30000)
    expect(timer().state().remainingMs).toBe(7000)
    hidden = false; document.dispatchEvent(new Event('visibilitychange')); await tick(30000)
    expect(timer().state().phase).toBe('paused')
    wrapper.get('input').element.focus(); await tick(6999)
    expect(useModelSaveFeedback().open).toBe(true)
    await tick(1); expect(useModelSaveFeedback().open).toBe(false)
    expect(document.activeElement).toBe(wrapper.get('input').element)
  })

  it('确认的自动焦点归还不暂停；再次主动键盘操作才暂停，离开后续计', async () => {
    await saved(); button('立即重启 Codex').element.focus(); await confirm()
    document.querySelector<HTMLButtonElement>('dialog[open] .modal-close')!.click(); await flushPromises()
    expect(document.activeElement).toBe(button('立即重启 Codex').element)
    expect(timer().state().phase).toBe('running')
    await button('立即重启 Codex').trigger('keydown', { key: 'Tab' })
    expect(timer().state().phase).toBe('paused')
    wrapper.get('input').element.focus(); await tick(8000)
    expect(useModelSaveFeedback().open).toBe(false)
  })

  it('新保存重置8秒且旧倒计时不收新卡；稍后直接收起', async () => {
    await saved(); const old = timer(); await tick(4000); await saved()
    expect(old.state().phase).toBe('cancelled')
    await tick(4001); expect(useModelSaveFeedback().open).toBe(true)
    expect(timer().state().remainingMs).toBe(3999)
    await button('稍后').trigger('click')
    expect(useModelSaveFeedback().open).toBe(false)
    expect(modelApi.refreshCache).not.toHaveBeenCalled()
  })

  it('确认请求期间所有退出都受busy约束，旧请求返回不收新卡', async () => {
    let finish!: (value: Awaited<ReturnType<typeof modelApi.refreshCache>>) => void
    vi.mocked(modelApi.refreshCache).mockReturnValueOnce(new Promise(resolve => { finish = resolve }))
    await saved(); await confirm()
    const dialog = document.querySelector<HTMLDialogElement>('dialog[open]')!
    ;[...dialog.querySelectorAll('button')].find(node => node.textContent === '确认重启')!.click(); await flushPromises()
    expect(dialog.querySelector<HTMLButtonElement>('.modal-close')!.disabled).toBe(true)
    dialog.dispatchEvent(new Event('cancel', { cancelable: true })); await tick(30000)
    expect(dialog.open).toBe(true); expect(useModelSaveFeedback().open).toBe(true)
    await saved()
    finish({ success: true, restarted: true, summary: '旧请求完成' }); await flushPromises()
    expect(useModelSaveFeedback().open).toBe(true)
    expect(timer().state().remainingMs).toBe(8000)
    await tick(8000); expect(useModelSaveFeedback().open).toBe(false)
  })
})
