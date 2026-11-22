import {mount,flushPromises,type VueWrapper} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import {beforeEach,afterEach,expect,it,vi} from 'vitest'
import IntegratedPreparation from '@/features/environment/components/IntegratedPreparation.vue'
import {useEnvironmentStore} from '@/features/environment/store'
const invoke=vi.hoisted(()=>vi.fn())
vi.mock('@tauri-apps/api/core',()=>({invoke}))
const snapshot=(phase:string,rest={})=>({attemptId:'original-attempt',revision:1,phase,failedPhase:null,failure:null,retryable:false,...rest})
let views:VueWrapper[]=[]
function render(){const w=mount(IntegratedPreparation);views.push(w);return w}
beforeEach(()=>{setActivePinia(createPinia());invoke.mockReset()})
afterEach(()=>{views.forEach(v=>v.unmount());views=[];vi.useRealTimers()})
it('shows real extract progress without inventing passed dependencies or running service',async()=>{
 invoke.mockResolvedValue(snapshot('extracting'));const w=render();await flushPromises();expect(w.text()).toContain('展开运行包与依赖');const rows=w.findAll('li');expect(rows[0]!.text()).toContain('已完成');expect(rows[2]!.text()).toContain('进行中');expect(rows[3]!.text()).toContain('待执行');expect(w.text()).not.toContain('缺少 Node');expect(w.text()).not.toContain('服务已运行');expect(w.findAll('button').some(b=>b.text()==='重新准备内置环境')).toBe(false)
})
it('explicit retry binds the last failure and refuses duplicate while command is pending',async()=>{
 let finish!:(v:unknown)=>void;invoke.mockResolvedValueOnce(snapshot('failed',{failedPhase:'verifying',failure:'integrity',retryable:true})).mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve})).mockResolvedValue(snapshot('extracting',{revision:3,attemptId:'new-attempt'}));const w=render();await flushPromises();expect(w.text()).toContain('校验未通过');const retry=w.findAll('button').find(b=>b.text()==='重新准备内置环境')!;await retry.trigger('click');await flushPromises();expect(invoke.mock.calls[1]).toEqual(['runtime_preparation_retry',{request:{attemptId:'original-attempt'}}]);expect(retry.attributes('disabled')).toBeDefined();finish(snapshot('extracting',{revision:3,attemptId:'new-attempt'}));await flushPromises();expect(invoke.mock.calls.filter(c=>c[0]==='runtime_preparation_retry')).toHaveLength(1)
})
it('completed supply triggers one real dependency check and does not start a service',async()=>{
 const state=useEnvironmentStore();const refresh=vi.spyOn(state,'refresh').mockResolvedValue(true);invoke.mockResolvedValue(snapshot('supplied'));const w=render();await flushPromises();expect(refresh).toHaveBeenCalledOnce();expect(w.text()).toContain('程序已供应');expect(w.text()).not.toContain('服务已运行');expect(invoke.mock.calls).toEqual([['runtime_preparation_status']])
})
it('removal or recovery-required state cannot acquire a supply retry',async()=>{
 invoke.mockResolvedValue(snapshot('review',{failedPhase:'checking',failure:'recovery_required'}));const w=render();await flushPromises();expect(w.text()).toContain('继续原操作');expect(w.findAll('button').some(b=>b.text()==='重新准备内置环境')).toBe(false)
})
it('unreadable observation remains unknown and does not show missing system Node',async()=>{
 invoke.mockRejectedValue(new Error('unavailable'));const w=render();await flushPromises();expect(w.text()).toContain('准备状态暂不可读取');expect(w.text()).not.toContain('缺少 Node');expect(w.emitted('observing')).toEqual([[true]])
})
it('unmount drops late result without dependency execution or a continuing timer',async()=>{
 let finish!:(v:unknown)=>void;invoke.mockImplementation(()=>new Promise(resolve=>{finish=resolve}));const refresh=vi.spyOn(useEnvironmentStore(),'refresh').mockResolvedValue(true);const w=render();await flushPromises();w.unmount();views=[];finish(snapshot('supplied'));await flushPromises();expect(refresh).not.toHaveBeenCalled();expect(invoke).toHaveBeenCalledOnce()
})
