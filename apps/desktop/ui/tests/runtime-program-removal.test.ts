import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {createPinia,setActivePinia} from 'pinia'
import {flushPromises,mount,type VueWrapper} from '@vue/test-utils'
import RuntimeProgramRemoval from '@/features/runtime/RuntimeProgramRemoval.vue'
import {useProgramRemovalStore,type RemovalReview,type RemovalOutcome} from '@/features/runtime/program-removal'
const {invoke,source,paths}=vi.hoisted(()=>({invoke:vi.fn(),source:vi.fn(),paths:vi.fn()}))
vi.mock('@tauri-apps/api/core',()=>({invoke}))
vi.mock('@/stores/app',()=>({useAppStore:()=>({loadRuntimeSource:source})}))
vi.mock('@/features/data-root/store',()=>({useDataRootStore:()=>({loadPaths:paths})}))
const review=(resume=false):RemovalReview=>({id:'review-id',resume,commandPath:'/fixture/.local/bin/ocx',shellPaths:['/fixture/.zshrc'],confirmationDigest:'review-hash',removal:{operationId:'original-operation',versions:['2.99.0','3.0.0'],programRoot:'/fixture/program',preservedHome:'/fixture/opencodex-home',files:24,bytes:2048,confirmationDigest:'core-plan-hash'}})
const done=():RemovalOutcome=>({operationId:'original-operation',removedVersions:['2.99.0','3.0.0'],configurationPreserved:true})
function pending<T>(){let resolve!:(value:T)=>void;const promise=new Promise<T>(r=>{resolve=r});return {promise,resolve}}
let views:VueWrapper[]=[]
function render(){const view=mount(RuntimeProgramRemoval,{global:{stubs:{UiDialog:{props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'}}}});views.push(view);return {view,button:(label:string)=>view.findAll('button').find(b=>b.text()===label)}}
beforeEach(()=>{setActivePinia(createPinia());invoke.mockReset();source.mockReset().mockResolvedValue(undefined);paths.mockReset().mockResolvedValue(undefined)})
afterEach(()=>{for(const view of views)view.unmount();views=[]})
describe('preserving versioned program uninstall',()=>{
 it('only reads on entry and cancel creates no removal or backup',async()=>{
  invoke.mockResolvedValue(review());const {view,button}=render();expect(invoke).not.toHaveBeenCalled()
  await button('卸载程序')!.trigger('click');await flushPromises();expect(view.text()).toContain('配置与备份');expect(view.text()).toContain('/fixture/.zshrc')
  expect(view.text()).not.toContain('清空 OPENCODEX_HOME');expect(view.find('input[type="checkbox"]').exists()).toBe(false)
  await button('取消')!.trigger('click');expect(invoke.mock.calls).toEqual([['runtime_removal_preview']])
 })
 it.each([false,true])('confirms the exact reviewed %s operation and checks the retained configuration receipt',async resume=>{
  invoke.mockResolvedValueOnce(review(resume)).mockResolvedValueOnce(done());const {view,button}=render()
  await button('卸载程序')!.trigger('click');await flushPromises();await button(resume?'确认继续原卸载':'确认卸载程序')!.trigger('click');await flushPromises()
  expect(invoke.mock.calls[1]).toEqual(['runtime_removal_apply',{request:{id:'review-id',confirmationDigest:'review-hash'}}])
  expect(view.text()).toContain('程序卸载已完成');expect(view.text()).toContain('配置和备份已保留');expect(source).toHaveBeenCalledOnce();expect(paths).toHaveBeenCalledOnce()
 })
 it('closing and reopening an in-flight removal never dispatches a duplicate request',async()=>{
  const response=pending<RemovalOutcome>();invoke.mockResolvedValueOnce(review()).mockImplementationOnce(()=>response.promise)
  const first=render();await first.button('卸载程序')!.trigger('click');await flushPromises();await first.button('确认卸载程序')!.trigger('click');await flushPromises();await first.button('关闭')!.trigger('click');first.view.unmount()
  expect(useProgramRemovalStore().executing).toBe(true);const second=render();await second.button('查看卸载进度')!.trigger('click');await flushPromises();expect(invoke).toHaveBeenCalledTimes(2)
  response.resolve(done());await flushPromises();expect(second.view.text()).toContain('程序卸载已完成');expect(invoke).toHaveBeenCalledTimes(2)
 })
 it('a failed or mismatched reply cannot claim completion and requires original-state reinspection',async()=>{
  invoke.mockResolvedValueOnce(review()).mockResolvedValueOnce({...done(),configurationPreserved:false}).mockResolvedValueOnce(review(true))
  const {view,button}=render();await button('卸载程序')!.trigger('click');await flushPromises();await button('确认卸载程序')!.trigger('click');await flushPromises()
  expect(view.get('[role="alert"]').text()).toContain('不一致');expect(button('确认卸载程序')).toBeUndefined();expect(source).not.toHaveBeenCalled()
  await button('重新核对原操作')!.trigger('click');await flushPromises();expect(button('确认继续原卸载')).toBeDefined();expect(invoke).toHaveBeenCalledTimes(3)
 })
 it('null observation says already removed but never fabricates this request succeeding',async()=>{
  invoke.mockResolvedValue(null);const {view,button}=render();await button('卸载程序')!.trigger('click');await flushPromises()
  expect(view.text()).toContain('已卸载，没有待继续');expect(view.text()).not.toContain('程序卸载已完成');expect(useProgramRemovalStore().outcome).toBeNull()
 })
})
