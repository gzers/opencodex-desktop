import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { DOMWrapper, flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import App from '@/App.vue'
import SettingsRoute from '@/routes/SettingsRoute.vue'
import { useAppStore } from '@/stores/app'
import { useRouteStore } from '@/stores/routes'
import { useFeedbackStore } from '@/app/feedback/store'
import { useRuntimeStore } from '@/features/runtime/store'
import { installNativeDialogStub } from './helpers/nativeDialog'

const invoke = vi.hoisted(() => vi.fn())
vi.mock('@tauri-apps/api/core', () => ({ invoke }))
vi.mock('@tauri-apps/api/event', () => ({ listen: async () => () => {} }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: async () => null }))
vi.mock('@/composables/useAppController', () => ({ useAppController: () => ({}) }))
Object.defineProperty(window, 'matchMedia', { configurable: true, value: () => ({ matches: false, addEventListener() {}, removeEventListener() {} }) })
const source = { kind: 'managed', path: '/fixture/runtime/ocx', version: '2.50.0', resolvedAt: null, insideDataRoot: true, managedEntry: '/fixture/runtime/ocx', managedPrefix: '/fixture/runtime', defaultPrefix: '/fixture/runtime', explicitPath: null, history: [] }
const wrappers: VueWrapper[] = []
let pinia: ReturnType<typeof createPinia>
beforeEach(() => {
  pinia = createPinia(); setActivePinia(pinia); installNativeDialogStub(); invoke.mockReset()
  // 程序卸载走 runtime_removal_* 的复核 → 确认两步。旧计划命令已不在界面里。
  const removal = { id: 'removal-fixture', resume: false, commandPath: null, shellPaths: [], confirmationDigest: 'digest-fixture', removal: { operationId: 'op-fixture', versions: ['2.50.0'], programRoot: '/fixture/runtime', preservedHome: '/fixture/home', files: 12, bytes: 2048, confirmationDigest: 'digest-fixture' } }
  invoke.mockImplementation(async command => {
    if (command === 'runtime_source') return source
    if (command === 'runtime_protection_status') return { state: 'none', version: null, backupId: null }
    if (command === 'runtime_removal_preview') return removal
    if (command === 'runtime_removal_apply') return { operationId: 'op-fixture', removedVersions: ['2.50.0'], configurationPreserved: true }
    return null
  })
})
afterEach(() => { wrappers.splice(0).forEach(w => w.unmount()); document.body.innerHTML = '' })
function dialog(selector = 'dialog[open]') { return new DOMWrapper(document.body).get(selector) }
async function exit(channel: string, selector = 'dialog[open]') {
  const root = dialog(selector)
  if (channel === 'x') await root.get('header .modal-close').trigger('click')
  if (channel === 'escape') root.element.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }))
  if (channel === 'native-cancel') root.element.dispatchEvent(new Event('cancel', { cancelable: true }))
  if (channel === 'backdrop') await root.trigger('click', { clientX: -10, clientY: -10 })
  await flushPromises()
}
async function fault() {
  useRouteStore().go('tray')
  const w = mount(App, { global: { plugins: [pinia], stubs: { AppSidebar: true, TrayRoute: true, ToastHost: true, TaskProgressHost: true, NotificationCenter: true, RuntimeInstallDialog: true, SyncDialogHost: true } }, attachTo: document.body }); wrappers.push(w)
  useFeedbackStore().reportUiFault('Fixture rendering error'); await flushPromises(); return w
}
async function uninstall() {
  const backend=invoke.getMockImplementation()!
  invoke.mockImplementation((command,...args)=>command==='runtime_protection_status'?Promise.resolve({state:'none',version:null,backupId:null}):backend(command,...args))
  useRuntimeStore().source = source as never
  useRuntimeStore().protection={state:'none',version:null,backupId:null}
  useRouteStore().go('settings', { section: 'installation' })
  const w = mount(SettingsRoute, { global: { plugins: [pinia], stubs: { AppTopbar: true, BackupManagement: true, AgentExecutionField: true } }, attachTo: document.body }); wrappers.push(w)
  await flushPromises(); await w.get('[data-testid="runtime-program-removal"]').trigger('click'); await flushPromises(); return w
}

describe('POST-09 remaining representative hosts', () => {
  it.each(['x', 'escape', 'native-cancel', 'backdrop'])('closes only the alert through %s and retains its real reload action', async channel => {
    await fault()
    expect(dialog().attributes('role')).toBe('alertdialog')
    expect(dialog().findAll('footer button').map(b => b.text())).toEqual(['重新加载界面'])
    invoke.mockClear(); await exit(channel)
    expect(useFeedbackStore().uiFault).toBe(''); expect(invoke).not.toHaveBeenCalled()
  })

  it('lets a nested confirmation consume Escape before clearing an underlying fault', async () => {
    await fault(); useAppStore().openModal({ title: '嵌套确认', body: '<p>只取消确认</p>', confirmLabel: '确认' }); await flushPromises()
    await exit('escape', 'dialog.app-modal-dialog[open]')
    expect(useAppStore().modal).toBeNull(); expect(useFeedbackStore().uiFault).toBe('Fixture rendering error')
    await exit('escape'); expect(useFeedbackStore().uiFault).toBe('')
  })

  it('cancels program removal preparation without issuing any removal request', async () => {
    await uninstall()
    expect(dialog().findAll('footer button').map(b => b.text())).toEqual(['取消', '确认卸载程序'])
    await dialog().findAll('footer button').find(b => b.text() === '取消')!.trigger('click'); await flushPromises()
    expect(document.querySelector('dialog[open]')).toBeNull()
    expect(invoke.mock.calls.filter(([command]) => command === 'runtime_removal_apply')).toHaveLength(0)
  })

  // 进行中允许关窗：操作继续，按钮变成「查看卸载进度」可再打开；任何退出方式都不得重复提交。
  it.each(['x', 'escape', 'native-cancel', 'backdrop'])('lets %s close the progress view without resubmitting removal', async channel => {
    const w = await uninstall(); let finish!: (value: unknown) => void
    const original = invoke.getMockImplementation()!
    invoke.mockImplementation((command, args) => command === 'runtime_removal_apply'
      ? new Promise(resolve => { finish = resolve })
      : original(command, args))
    await dialog().findAll('footer button').find(b => b.text() === '确认卸载程序')!.trigger('click'); await flushPromises()
    await exit(channel)
    expect(document.querySelector('dialog[open]')).toBeNull()
    expect(invoke.mock.calls.filter(([command]) => command === 'runtime_removal_apply')).toHaveLength(1)
    const reopen = w.get('[data-testid="runtime-program-removal"]')
    expect(reopen.text()).toBe('查看卸载进度')
    await reopen.trigger('click'); await flushPromises()
    expect(dialog().text()).toContain('正在移除已核对的程序文件')
    expect(invoke.mock.calls.filter(([command]) => command === 'runtime_removal_apply')).toHaveLength(1)
    finish({ operationId: 'op-fixture', removedVersions: ['2.50.0'], configurationPreserved: true }); await flushPromises()
    expect(dialog().text()).toContain('程序卸载已完成')
    await exit('x'); expect(document.querySelector('dialog[open]')).toBeNull()
  })

})
