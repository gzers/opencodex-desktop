import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {mount,flushPromises} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import RuntimeMotionMark from '@/features/runtime/components/RuntimeMotionMark.vue'
import {useEffectsStore} from '@/app/appearance/effects'
import ModelsRoute from '@/routes/ModelsRoute.vue'
import {useModelsStore} from '@/features/models/store'
import {emptyContent,modelApi} from '@/features/models/api'
vi.mock('@tauri-apps/api/event',()=>({listen:vi.fn(async()=>()=>{})}))
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{load:vi.fn(),stage:vi.fn()}}))
let frames:Map<number,FrameRequestCallback>;let serial:number
beforeEach(()=>{setActivePinia(createPinia());frames=new Map();serial=0;vi.stubGlobal('requestAnimationFrame',(cb:FrameRequestCallback)=>{frames.set(++serial,cb);return serial});vi.stubGlobal('cancelAnimationFrame',(id:number)=>frames.delete(id))})
afterEach(()=>vi.unstubAllGlobals())
describe('shared compact assistant mark',()=>{
 it('uses independent SVG identities and the shared quality/lifecycle strategy without a backdrop or GPU surface',async()=>{
  const first=mount(RuntimeMotionMark,{props:{state:'running',compact:true}}),second=mount(RuntimeMotionMark,{props:{state:'running',compact:true}});await flushPromises()
  expect(first.find('svg').exists()).toBe(true);expect(first.find('.motion-backdrop').exists()).toBe(false);expect(first.find('canvas').exists()).toBe(false)
  expect(first.get('linearGradient').attributes('id')).not.toBe(second.get('linearGradient').attributes('id'));expect(frames.size).toBe(2)
  const effects=useEffectsStore();effects.setSetting('mid');await flushPromises();expect(frames.size).toBe(0)
  effects.setSetting('low');await flushPromises();expect(frames.size).toBe(0);expect(first.find('svg').exists()).toBe(true)
  effects.setSetting('high');await flushPromises();expect(frames.size).toBe(2)
  effects.setVisible(false);await flushPromises();expect(frames.size).toBe(0)
  effects.setVisible(true);await flushPromises();expect(frames.size).toBe(2)
  await first.setProps({active:false});expect(frames.size).toBe(1);first.unmount();second.unmount();expect(frames.size).toBe(0)
 })
 it('opens one nonmodal assistant from either trigger, restores launcher focus, and hides with the editor',async()=>{
  const pinia=createPinia();setActivePinia(pinia);useEffectsStore().setSetting('low');const store=useModelsStore();store.view.tab='templates'
  vi.mocked(modelApi.load).mockResolvedValue({library:{schemaVersion:1,templates:[]},policy:{schemaVersion:1,templateRefs:[],channels:[]},libraryDigest:'library',policyDigest:'policy',workspace:null,workspaceDigest:'workspace'})
  const wrapper=mount(ModelsRoute,{attachTo:document.body,global:{plugins:[pinia],stubs:{AppTopbar:true,ChannelsView:true,BatchAgentDialog:true,ExternalTaskDialog:true,ModelRuleFields:true,UiDialog:true,UiSelectField:true,AgentPanel:{props:['active'],emits:['close'],template:'<aside id="model-agent-panel"><button @click="$emit(\'close\')">收起助手</button></aside>'}}}})
  await flushPromises();store.draft={templateId:null,baseRevision:null,label:'新建',content:emptyContent()};store.view.templateEditor=true;await flushPromises()
  const trigger=wrapper.get<HTMLButtonElement>('.model-agent-launcher');trigger.element.focus();await trigger.trigger('click');expect(wrapper.findAll('#model-agent-panel')).toHaveLength(1);expect(wrapper.find('dialog[open]').exists()).toBe(false);expect(wrapper.find('.model-agent-launcher').exists()).toBe(false)
  await wrapper.findAll('button').find(b=>b.text()==='收起助手')!.trigger('click');await flushPromises();expect(document.activeElement).toBe(wrapper.get('.model-agent-launcher').element)
  await wrapper.get('[aria-label="Agent自动填充"]').trigger('click');expect(wrapper.get('[aria-label="Agent自动填充"]').attributes('aria-expanded')).toBe('true')
  store.view.tab='channels';await flushPromises();expect(wrapper.get('#model-agent-panel').isVisible()).toBe(false)
  store.view.tab='templates';await flushPromises();expect(wrapper.get('#model-agent-panel').isVisible()).toBe(true);wrapper.unmount();expect(frames.size).toBe(0)
 })
})
