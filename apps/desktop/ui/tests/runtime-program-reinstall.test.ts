import {afterEach,beforeEach,expect,it,vi} from 'vitest'
import {createPinia,setActivePinia} from 'pinia'
import {mount,flushPromises,type VueWrapper} from '@vue/test-utils'
import RuntimeProgramReinstall from '@/features/runtime/RuntimeProgramReinstall.vue'
import {useProgramReinstallStore,type BuiltinReview} from '@/features/runtime/program-reinstall'
import {useFeedbackStore} from '@/app/feedback/store'
const {invoke,source,paths}=vi.hoisted(()=>({invoke:vi.fn(),source:vi.fn(),paths:vi.fn()}))
vi.mock('@tauri-apps/api/core',()=>({invoke}))
vi.mock('@/stores/app',()=>({useAppStore:()=>({loadRuntimeSource:source})}))
vi.mock('@/features/data-root/store',()=>({useDataRootStore:()=>({loadPaths:paths})}))
const review=(resume=false):BuiltinReview=>({version:'2.99.0',resume,programRoot:'/fixture/program',configPath:'/fixture/user/home/config.json',confirmationDigest:'original-bundle-review'})
let views:VueWrapper[]=[]
function render(){const view=mount(RuntimeProgramReinstall,{global:{stubs:{UiDialog:{props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'}}}});views.push(view);return{view,button:(label:string)=>view.findAll('button').find(b=>b.text()===label)}}
beforeEach(()=>{setActivePinia(createPinia());invoke.mockReset();source.mockReset().mockResolvedValue(null);paths.mockReset().mockResolvedValue(null)})
afterEach(()=>{views.forEach(v=>v.unmount());views=[];useFeedbackStore().clearToast()})
it('preview and cancel do not install or re-enable a terminal command',async()=>{
 invoke.mockResolvedValue(review());const {view,button}=render();expect(invoke).not.toHaveBeenCalled();await button('重装内置程序')!.trigger('click');await flushPromises();expect(view.text()).toContain('/fixture/user/home/config.json');expect(view.text()).toContain('不会自动启用终端命令');await button('取消')!.trigger('click');expect(invoke.mock.calls).toEqual([['runtime_builtin_preview']])
})
it.each([false,true])('confirms the exact current resource and original continuation=%s',async resume=>{
 invoke.mockResolvedValueOnce(review(resume)).mockResolvedValueOnce({version:'2.99.0',configurationPreserved:true});const {view,button}=render();await button('重装内置程序')!.trigger('click');await flushPromises();await button(resume?'确认继续原重装':'确认重新安装')!.trigger('click');await flushPromises();expect(invoke.mock.calls[1]).toEqual(['runtime_builtin_reinstall',{request:{confirmationDigest:'original-bundle-review'}}]);expect(view.text()).toContain('2.99.0 已重新安装');expect(source).toHaveBeenCalledOnce();expect(paths).toHaveBeenCalledOnce()
})
it('late preview cannot reopen a dismissed dialog',async()=>{
 let finish!:(r:BuiltinReview)=>void;invoke.mockImplementation(()=>new Promise(resolve=>{finish=resolve}));const {view,button}=render();await button('重装内置程序')!.trigger('click');await button('取消')!.trigger('click');finish(review());await flushPromises();expect(view.text()).not.toContain('确认重新安装');expect(useProgramReinstallStore().review).toBeNull()
})
it('close and remount observe one retained in-flight install',async()=>{
 let finish!:(r:unknown)=>void;invoke.mockResolvedValueOnce(review()).mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve}));const first=render();await first.button('重装内置程序')!.trigger('click');await flushPromises();await first.button('确认重新安装')!.trigger('click');await first.button('关闭')!.trigger('click');first.view.unmount();const second=render();await second.button('查看重装进度')!.trigger('click');expect(invoke).toHaveBeenCalledTimes(2);finish({version:'2.99.0',configurationPreserved:true});await flushPromises();expect(second.view.text()).toContain('已重新安装');expect(invoke).toHaveBeenCalledTimes(2)
})
it('wrong receipt or failed preview never produces installation success',async()=>{
 invoke.mockResolvedValueOnce(review()).mockResolvedValueOnce({version:'3.0.0',configurationPreserved:true}).mockRejectedValueOnce(new Error('program still in use'));const {view,button}=render();await button('重装内置程序')!.trigger('click');await flushPromises();await button('确认重新安装')!.trigger('click');await flushPromises();expect(view.get('[role="alert"]').text()).toContain('不一致');expect(source).not.toHaveBeenCalled();expect(button('确认重新安装')).toBeUndefined();await button('重新核对')!.trigger('click');await flushPromises();expect(useProgramReinstallStore().outcome).toBeNull();expect(button('确认重新安装')).toBeUndefined()
})
it('repair confirms only its own admitted-image API and never dispatches reinstall',async()=>{
 invoke.mockResolvedValueOnce(review()).mockResolvedValueOnce({version:'2.99.0',configurationPreserved:true})
 const view=mount(RuntimeProgramReinstall,{props:{repair:true},global:{stubs:{UiDialog:{props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'}}}});views.push(view)
 await view.get('[data-testid="runtime-builtin-repair"]').trigger('click');await flushPromises();expect(view.text()).toContain('原目录将保留')
 await view.findAll('button').find(b=>b.text()==='确认修复程序')!.trigger('click');await flushPromises()
 expect(invoke.mock.calls).toEqual([['runtime_builtin_repair_preview'],['runtime_builtin_repair',{request:{confirmationDigest:'original-bundle-review'}}]])
 expect(view.text()).toContain('2.99.0 已修复');expect(source).toHaveBeenCalledOnce()
})
