import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { flushPromises, mount, type VueWrapper } from '@vue/test-utils'
import RuntimeActivationRecovery from '@/features/runtime/RuntimeActivationRecovery.vue'
import { activationRecoveryApi, useActivationRecoveryStore, type ActivationPreview } from '@/features/runtime/activation-recovery'

const { invoke, loadSource, loadPaths } = vi.hoisted(() => ({ invoke: vi.fn(), loadSource: vi.fn(), loadPaths: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke }))
vi.mock('@/stores/app', () => ({useAppStore:() => ({loadRuntimeSource:loadSource})}))
vi.mock('@/features/data-root/store', () => ({useDataRootStore:() => ({loadPaths})}))
const preview = ():ActivationPreview => ({attemptId:'original-operation',from:'2.99.0',to:'3.0.0',programRoot:'/fixture/program',configPath:'/fixture/home/config.json',confirmationDigest:'reviewed-content'})
function pending<T>() { let resolve!:(value:T)=>void;const promise=new Promise<T>(done=>{resolve=done});return {promise,resolve} }
let views:VueWrapper[]=[]
function render() {
 const view=mount(RuntimeActivationRecovery,{global:{stubs:{UiDialog:{props:['open'],emits:['close'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'}}}});views.push(view)
 return {view,button:(label:string)=>view.findAll('button').find(b=>b.text()===label)}
}
beforeEach(()=>{setActivePinia(createPinia());invoke.mockReset();loadSource.mockReset().mockResolvedValue(undefined);loadPaths.mockReset().mockResolvedValue(undefined)})
afterEach(()=>{for(const v of views)v.unmount();views=[];vi.restoreAllMocks()})
describe('interrupted version activation recovery',()=>{
 it('only observes on entry and cancellation never commits or removes the operation',async()=>{
  invoke.mockResolvedValue(preview());const {view,button}=render();
  expect(invoke).not.toHaveBeenCalled();await button('检查安装恢复')!.trigger('click');await flushPromises()
  expect(invoke.mock.calls).toEqual([['runtime_activation_preview']]);expect(view.text()).toContain('2.99.0');expect(view.text()).toContain('3.0.0')
  await button('取消')!.trigger('click');expect(useActivationRecoveryStore().open).toBe(false)
  expect(invoke).toHaveBeenCalledTimes(1)
 })
 it.each(['complete','rollback'] as const)('submits only the reviewed %s decision and reloads after a matching receipt',async decision=>{
  invoke.mockResolvedValueOnce(preview()).mockResolvedValueOnce({attemptId:'original-operation',decision,selectedVersion:decision==='complete'?'3.0.0':'2.99.0'})
  const {view,button}=render();await button('检查安装恢复')!.trigger('click');await flushPromises()
  await view.get('input[value="'+decision+'"]').setValue(true)
  await button(decision==='complete'?'确认继续安装':'确认回退')!.trigger('click');await flushPromises()
  expect(invoke.mock.calls[1]).toEqual(['runtime_activation_recover',{request:{attemptId:'original-operation',confirmationDigest:'reviewed-content',decision}}])
  expect(view.text()).toContain(decision==='complete'?'安装恢复已完成':'原安装操作已回退')
  expect(loadSource).toHaveBeenCalledOnce();expect(loadPaths).toHaveBeenCalledOnce()
 })
 it('closing and remounting during apply observes the same promise without duplicate mutation',async()=>{
  const operation=pending<{attemptId:string;decision:'rollback';selectedVersion:string}>()
  invoke.mockResolvedValueOnce(preview()).mockImplementationOnce(()=>operation.promise)
  const first=render();await first.button('检查安装恢复')!.trigger('click');await flushPromises()
  await first.view.get('input[value="rollback"]').setValue(true);await first.button('确认回退')!.trigger('click');await flushPromises()
  await first.button('关闭')!.trigger('click');first.view.unmount()
  expect(useActivationRecoveryStore().executing).toBe(true)
  const second=render();await second.button('查看安装恢复进度')!.trigger('click');await flushPromises()
  expect(invoke).toHaveBeenCalledTimes(2);expect(second.view.text()).toContain('正在恢复原安装操作')
  operation.resolve({attemptId:'original-operation',decision:'rollback',selectedVersion:'2.99.0'});await flushPromises()
  expect(second.view.text()).toContain('原安装操作已回退');expect(invoke).toHaveBeenCalledTimes(2)
 })
 it('a late read after closing cannot expose a confirmation, and a mismatched reply is not success',async()=>{
  const response=pending<ActivationPreview>();vi.spyOn(activationRecoveryApi,'preview').mockReturnValueOnce(response.promise)
  const store=useActivationRecoveryStore();store.start();store.close();response.resolve(preview());await flushPromises();expect(store.preview).toBeNull()
  vi.mocked(activationRecoveryApi.preview).mockResolvedValueOnce(preview());store.start();await flushPromises()
  vi.spyOn(activationRecoveryApi,'recover').mockResolvedValue({attemptId:'different-operation',decision:'complete',selectedVersion:'3.0.0'})
  expect(await store.apply()).toBe(false);expect(store.outcome).toBeNull();expect(store.preview).toBeNull();expect(store.error).toContain('不一致')
 })
 it('failed confirmation preserves an error and requires a new observation before any retry',async()=>{
  invoke.mockResolvedValueOnce(preview()).mockRejectedValueOnce({message:'配置或安装记录已变化，请重新核对'}).mockResolvedValueOnce(null)
  const {view,button}=render();await button('检查安装恢复')!.trigger('click');await flushPromises();await button('确认继续安装')!.trigger('click');await flushPromises()
  expect(view.get('[role="alert"]').text()).toContain('已变化');expect(button('确认继续安装')).toBeUndefined();expect(loadSource).not.toHaveBeenCalled()
  await button('重新核对原操作')!.trigger('click');await flushPromises()
  expect(view.text()).toContain('没有发现待恢复');expect(view.text()).not.toContain('恢复已完成')
  expect(invoke.mock.calls.map(c=>c[0])).toEqual(['runtime_activation_preview','runtime_activation_recover','runtime_activation_preview'])
 })
})
