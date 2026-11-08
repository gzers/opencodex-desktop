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
const plan = { sourceKind: 'managed', sourcePath: source.path, managed: true, backupId: null, removable: true, reason: null, removeObjects: ['fixture package'], runtimeObjects: [], dataObjects: [], residueCandidates: [], officialCommand: 'ocx uninstall', externalCommand: null }
const result = { scope: 'full', sourceKind: 'unresolved', sourcePath: null, backupId: 'fixture', backupDirectory: '/fixture/backup', steps: [{ name: '移除包体与入口', status: 'ok', detail: 'fixture' }], residue: [{ path: '/fixture/runtime', status: 'cleared' }], officialOutput: [], needsRestart: false, message: 'fixture complete' }
const wrappers: VueWrapper[] = []
let pinia: ReturnType<typeof createPinia>
beforeEach(() => {
  pinia = createPinia(); setActivePinia(pinia); installNativeDialogStub(); invoke.mockReset()
  invoke.mockImplementation(async command => command === 'runtime_source' ? source : command === 'plan_runtime_uninstall' ? plan : null)
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
  await flushPromises(); await w.get('[data-testid="runtime-uninstall"]').trigger('click'); await flushPromises(); return w
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

  it('cancels uninstall preparation without issuing any uninstall request', async () => {
    await uninstall()
    expect(dialog().findAll('footer button').map(b => b.text())).toEqual(['取消', '卸载'])
    await dialog().findAll('footer button').find(b => b.text() === '取消')!.trigger('click'); await flushPromises()
    expect(document.querySelector('dialog[open]')).toBeNull()
    expect(invoke.mock.calls.filter(([command]) => command === 'uninstall_runtime')).toHaveLength(0)
  })

  it.each(['x', 'escape', 'native-cancel', 'backdrop'])('guards uninstall %s while executing, then exposes a result without an exit footer', async channel => {
    await uninstall(); let finish!: (value: unknown) => void
    const original = invoke.getMockImplementation()!
    invoke.mockImplementation((command, args) => command === 'uninstall_runtime' ? new Promise(resolve => { finish = resolve }) : original(command, args))
    await dialog().get('[data-testid="uninstall-ack"]').setValue(true)
    await dialog().get('[data-testid="uninstall-confirm"]').trigger('click'); await flushPromises()
    await exit(channel)
    expect(document.querySelector('dialog[open]')).not.toBeNull()
    expect(invoke.mock.calls.filter(([command]) => command === 'uninstall_runtime')).toHaveLength(1)
    finish(result); await flushPromises()
    expect(dialog().find('footer').exists()).toBe(false)
    expect(dialog().get('[data-testid="uninstall-progress-title"]').text()).toContain('卸载完成')
    await exit('x'); expect(document.querySelector('dialog[open]')).toBeNull()
  })
})
