import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { DOMWrapper, flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import OverviewRoute from '@/routes/OverviewRoute.vue'
import { useEnvironmentStore } from '@/features/environment/store'
import type { EnvironmentReport } from '@/features/environment/api'
import { installNativeDialogStub } from './helpers/nativeDialog'

const actions = vi.hoisted(() => ({ refreshEnvironment: vi.fn(), runAction: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn(async () => null) }))
vi.mock('@/composables/useAppController', async () => {
  const { computed } = await import('vue'), { runtimeScenarios } = await import('@/features/runtime/scenarios')
  return { useAppController: () => ({ ...actions, scenario: computed(() => runtimeScenarios.stopped), actionLabels: { start: '启动', restart: '重启', stop: '停止', logs: '日志', panel: '面板', advice: '恢复' } }) }
})
Object.defineProperty(window, 'matchMedia', { configurable: true, value: () => ({ matches: false, addEventListener() {}, removeEventListener() {} }) })
const found = { found: true, path: '/fixture/bin', version: '1.0' }
function report(ready = true): EnvironmentReport { return { node: { ...found }, npm: { ...found }, ocx: { ...found, found: ready }, gate: ready ? 'ready' : 'missing_ocx', shortCircuited: false, brewFound: true } }
const wrappers: VueWrapper[] = []
beforeEach(() => { setActivePinia(createPinia()); installNativeDialogStub(); vi.clearAllMocks(); useEnvironmentStore().set(report()) })
afterEach(() => { wrappers.splice(0).forEach(w => w.unmount()); document.body.innerHTML = '' })
function body() { return new DOMWrapper(document.body) }
function button(label: string) { return body().findAll('button').find(b => b.text() === label)! }
async function render() {
  const w = mount(OverviewRoute, { global: { stubs: { AppTopbar: true, RuntimeMotionMark: true, UpdateCenter: true, ModelTransfer: true, EnvironmentGate: true } }, attachTo: document.body }); wrappers.push(w)
  await flushPromises(); await w.get('.motion-mainline').trigger('click'); await flushPromises(); return w
}
async function exit(channel: string, selector: string) {
  const root = body().get(selector)
  if (channel === 'x') await root.get('header .modal-close').trigger('click')
  if (channel === 'escape') root.element.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true, cancelable: true }))
  if (channel === 'native-cancel') root.element.dispatchEvent(new Event('cancel', { cancelable: true }))
  if (channel === 'backdrop') await root.trigger('click', { clientX: -10, clientY: -10 })
  await flushPromises()
}
describe('POST-09 overview observation consumers', () => {
  it('actual overview leaves ready mode on discovery error and renders the existing factual error gate',async()=>{
    const env=useEnvironmentStore();env.report=null;env.error='集成运行程序检查未就绪，可稍后重新检查或查看安装状态。'
    const w=mount(OverviewRoute,{global:{stubs:{AppTopbar:true,RuntimeMotionMark:true,UpdateCenter:true,ModelTransfer:true}},attachTo:document.body});wrappers.push(w);await flushPromises()
    expect(w.get('[data-testid="environment-gate"] h2').text()).toBe('集成运行程序检查未就绪');expect(w.find('.cli-copy-btn').exists()).toBe(false);expect(w.text()).not.toMatch(/Homebrew|brew install|Node.js 官方下载/)
    env.set(report());await flushPromises();expect(w.find('[data-testid="environment-gate"]').exists()).toBe(false)
  })
  it.each(['x', 'escape', 'native-cancel', 'backdrop'])('retains directory business actions and exits the runtime observer via %s', async channel => {
    const w = await render(), root = body().get('dialog.overview-runtime-dialog[open]')
    expect(root.findAll('footer button').map(b => b.text())).toEqual(['打开数据目录', '打开 OPENCODEX_HOME'])
    await exit(channel, 'dialog.overview-runtime-dialog[open]')
    expect(document.querySelector('dialog[open]')).toBeNull(); expect(document.activeElement).toBe(w.get('.motion-mainline').element)
    expect(actions.runAction).not.toHaveBeenCalled(); expect(actions.refreshEnvironment).not.toHaveBeenCalled()
  })

  it('closes the nested environment first and leaves an explicitly requested recheck running', async () => {
    await render(); useEnvironmentStore().set(report(false)); await flushPromises()
    const trigger = button('完整环境指引'); (trigger.element as HTMLButtonElement).focus(); await trigger.trigger('click'); await flushPromises()
    expect(document.querySelectorAll('dialog[open]')).toHaveLength(2)
    let finish!: () => void; actions.refreshEnvironment.mockReturnValue(new Promise<void>(resolve => { finish = resolve }))
    await button('重新检查').trigger('click'); await flushPromises()
    const environment = body().get('dialog.overview-environment-dialog[open]')
    expect(environment.findAll('footer button').map(b => b.text())).not.toContain('关闭')
    await exit('escape', 'dialog.overview-environment-dialog[open]')
    expect(document.querySelectorAll('dialog[open]')).toHaveLength(1); expect(document.activeElement).toBe(trigger.element)
    expect(actions.refreshEnvironment).toHaveBeenCalledTimes(1); expect(actions.runAction).not.toHaveBeenCalled()
    finish(); await flushPromises(); await exit('escape', 'dialog.overview-runtime-dialog[open]')
    expect(document.querySelector('dialog[open]')).toBeNull()
  })
})
