import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {mount,flushPromises} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import ModelsRoute from '@/routes/ModelsRoute.vue'
import {useModelsStore} from '@/features/models/store'
import {emptyContent,modelApi} from '@/features/models/api'
vi.mock('@tauri-apps/api/event',()=>({listen:vi.fn(async()=>()=>{})}))
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{load:vi.fn(),stage:vi.fn()}}))
let frames:Map<number,FrameRequestCallback>;let serial:number
beforeEach(()=>{setActivePinia(createPinia());frames=new Map();serial=0;vi.stubGlobal('requestAnimationFrame',(cb:FrameRequestCallback)=>{frames.set(++serial,cb);return serial});vi.stubGlobal('cancelAnimationFrame',(id:number)=>frames.delete(id))})
afterEach(()=>vi.unstubAllGlobals())
describe('final sidebar assistant entry',()=>{
 it('opens from the sidebar only, restores its focus and hides with the editor',async()=>{
  const pinia=createPinia();setActivePinia(pinia);const store=useModelsStore();store.view.tab='templates'
  vi.mocked(modelApi.load).mockResolvedValue({library:{schemaVersion:1,templates:[]},policy:{schemaVersion:1,templateRefs:[],channels:[]},libraryDigest:'library',policyDigest:'policy',workspace:null,workspaceDigest:'workspace'})
  const wrapper=mount(ModelsRoute,{attachTo:document.body,global:{plugins:[pinia],stubs:{AppTopbar:true,ChannelsView:true,BatchAgentDialog:true,ExternalTaskDialog:true,ModelRuleFields:true,UiDialog:true,UiSelectField:true,AgentPanel:{props:['active'],emits:['close'],template:'<aside id="model-agent-panel"><button @click="$emit(\'close\')">收起助手</button></aside>'}}}})
  await flushPromises();store.draft={templateId:null,baseRevision:null,label:'新建',content:emptyContent()};store.view.templateEditor=true;await flushPromises()
  expect(wrapper.find('.model-agent-launcher').exists()).toBe(false);const trigger=wrapper.get<HTMLButtonElement>('[aria-label="Agent自动填充"]');trigger.element.focus();await trigger.trigger('click');expect(wrapper.findAll('#model-agent-panel')).toHaveLength(1);expect(wrapper.find('dialog[open]').exists()).toBe(false);expect(wrapper.find('.model-agent-launcher').exists()).toBe(false)
  await wrapper.findAll('button').find(b=>b.text()==='收起助手')!.trigger('click');await flushPromises();expect(document.activeElement).toBe(wrapper.get('[aria-label="Agent自动填充"]').element)
  await wrapper.get('[aria-label="Agent自动填充"]').trigger('click');expect(wrapper.get('[aria-label="Agent自动填充"]').attributes('aria-expanded')).toBe('true')
  store.view.tab='channels';await flushPromises();expect(wrapper.get('#model-agent-panel').isVisible()).toBe(false)
  store.view.tab='templates';await flushPromises();expect(wrapper.get('#model-agent-panel').isVisible()).toBe(true);wrapper.unmount();expect(frames.size).toBe(0)
 })
})
