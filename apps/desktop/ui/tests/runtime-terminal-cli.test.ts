import { beforeEach, afterEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { mount, flushPromises, type VueWrapper } from '@vue/test-utils'
import RuntimeTerminalCli from '@/features/runtime/RuntimeTerminalCli.vue'
import { useTerminalCliStore, type TerminalPreview } from '@/features/runtime/terminal-cli'
const {invoke}=vi.hoisted(()=>({invoke:vi.fn()}))
vi.mock('@tauri-apps/api/core',()=>({invoke}))
const initial=(enabled=false):TerminalPreview=>({action:enabled?'disable':'enable',enabled,pending:false,version:enabled?null:'2.99.0',commandPath:'/fixture/.local/bin/ocx',runtimeEntry:'/fixture/program/runtime/bin/ocx',opencodexHome:'/fixture/data/opencodex/home',profilePaths:['/fixture/.zshrc'],conflicts:[],confirmationDigest:'fixture-confirmation'})
const done=(enabled=true)=>({action:enabled?'enable':'disable',enabled,commandPath:initial().commandPath,opencodexHome:initial().opencodexHome})
function deferred<T>() {let resolve!:(v:T)=>void;const promise=new Promise<T>(done=>{resolve=done});return {promise,resolve}}
let views:VueWrapper[]=[]
function render(){const view=mount(RuntimeTerminalCli,{global:{stubs:{UiDialog:{props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'}}}});views.push(view);return {view,button:(label:string)=>view.findAll('button').find(b=>b.text()===label)}}
beforeEach(()=>{setActivePinia(createPinia());invoke.mockReset()})
afterEach(()=>{for(const v of views)v.unmount();views=[]})
describe('official terminal registration',()=>{
 it('inspection and closing do not enable; explicit confirmation passes only action and digest',async()=>{
  invoke.mockResolvedValueOnce(initial()).mockResolvedValueOnce(done())
  const {view,button}=render();expect(invoke).not.toHaveBeenCalled()
  await button('检查 ocx 命令')!.trigger('click');await flushPromises()
  expect(view.text()).toContain('2.99.0');expect(view.text()).toContain('/fixture/.zshrc')
  expect(invoke.mock.calls).toEqual([['runtime_terminal_preview']])
  await button('确认启用')!.trigger('click');await flushPromises()
  expect(invoke.mock.calls[1]).toEqual(['runtime_terminal_apply',{request:{action:'enable',confirmationDigest:'fixture-confirmation'}}])
  expect(view.text()).toContain('请打开新终端使用');expect(button('确认启用')).toBeUndefined()
  await button('关闭')!.trigger('click');expect(useTerminalCliStore().open).toBe(false)
 })
 it('cancels after preview without modifying the entry',async()=>{
  invoke.mockResolvedValue(initial());const {button}=render()
  await button('检查 ocx 命令')!.trigger('click');await flushPromises();await button('关闭')!.trigger('click')
  expect(invoke.mock.calls).toEqual([['runtime_terminal_preview']])
 })
 it('existing command conflicts disable confirmation and show exact affected paths',async()=>{
  invoke.mockResolvedValue({...initial(),conflicts:['/fixture/other/ocx']});const {view,button}=render()
  await button('检查 ocx 命令')!.trigger('click');await flushPromises()
  expect(view.get('[role="alert"]').text()).toContain('未覆盖');expect(view.text()).toContain('/fixture/other/ocx');expect(button('确认启用')).toBeUndefined()
 })
 it('disabled entry confirmation and stale failures use the original dialog without automatic retry',async()=>{
  invoke.mockResolvedValueOnce(initial(true)).mockRejectedValueOnce({message:'Shell文件已变化，请重新核对'})
  const {view,button}=render();await button('检查 ocx 命令')!.trigger('click');await flushPromises();await button('确认停用')!.trigger('click');await flushPromises()
  expect(invoke.mock.calls[1]).toEqual(['runtime_terminal_apply',{request:{action:'disable',confirmationDigest:'fixture-confirmation'}}])
  expect(view.get('[role="alert"]').text()).toContain('已变化');expect(button('确认停用')).toBeUndefined();expect(invoke).toHaveBeenCalledTimes(2)
 })
 it('reopening during apply observes the same operation and rejects a mismatched receipt',async()=>{
  const pending=deferred<ReturnType<typeof done>>()
  invoke.mockResolvedValueOnce(initial()).mockImplementationOnce(()=>pending.promise)
  const first=render();await first.button('检查 ocx 命令')!.trigger('click');await flushPromises();await first.button('确认启用')!.trigger('click');await flushPromises()
  await first.button('关闭')!.trigger('click');first.view.unmount()
  const second=render();await second.button('查看终端配置进度')!.trigger('click');await flushPromises()
  expect(invoke).toHaveBeenCalledTimes(2)
  pending.resolve({...done(),opencodexHome:'/other/home'});await flushPromises()
  expect(second.view.get('[role="alert"]').text()).toContain('不一致');expect(useTerminalCliStore().result).toBe('')
 })
 it('late preview after close cannot expose a stale enable confirmation',async()=>{
  const pending=deferred<TerminalPreview>();invoke.mockReturnValueOnce(pending.promise)
  const {view,button}=render();await button('检查 ocx 命令')!.trigger('click');await button('关闭')!.trigger('click');pending.resolve(initial());await flushPromises()
  expect(useTerminalCliStore().preview).toBeNull();expect(view.find('section').exists()).toBe(false)
 })
})
