import {beforeEach,describe,expect,it,vi} from 'vitest'
import {mount,flushPromises} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import {defineComponent,nextTick,ref} from 'vue'
import ModelsRoute from '@/routes/ModelsRoute.vue'
import {useModelsStore} from '@/features/models/store'
import {useModelViewPosition} from '@/features/models/useViewPosition'
import {emptyContent,modelApi,type ModelSnapshot} from '@/features/models/api'
vi.mock('@tauri-apps/api/event',()=>({listen:vi.fn(async()=>()=>{})}))
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{load:vi.fn(),save:vi.fn(),stage:vi.fn()}}))
const snapshot=():ModelSnapshot=>({library:{schemaVersion:1,templates:[{id:'family',source:'user',label:'测试族',defaultRevision:'revision',revisions:[{id:'revision',createdAt:'2026-10-20T00:00:00Z',savedLabel:'测试族',content:emptyContent(),digest:'x',baseRevision:null,changes:[]}]}]},policy:{schemaVersion:1,templateRefs:[],channels:[]},libraryDigest:'base',policyDigest:'policy',workspace:null,workspaceDigest:'workspace'})
beforeEach(()=>{setActivePinia(createPinia());vi.clearAllMocks();vi.mocked(modelApi.load).mockResolvedValue(snapshot())})
const button=(wrapper:ReturnType<typeof mount>,label:string)=>wrapper.findAll('button').find(b=>b.text()===label)!
describe('model view context',()=>{
 it('returns to the template list without discarding dirty input and restores it after route remount',async()=>{
  const pinia=createPinia();setActivePinia(pinia);const store=useModelsStore();store.view.tab='templates'
  const options={global:{plugins:[pinia],stubs:{AppTopbar:true,ChannelsView:true,AgentPanel:true,RuntimeMotionMark:true,BatchAgentDialog:true,ExternalTaskDialog:true,ModelRuleFields:true,UiDialog:true,UiSelectField:true}}}
  let wrapper=mount(ModelsRoute,options);await flushPromises();await button(wrapper,'编辑').trigger('click');await wrapper.get('input[aria-label="模版名称"]').setValue('保留输入')
  await button(wrapper,'返回').trigger('click');expect(button(wrapper,'继续编辑').exists()).toBe(true);expect(store.dirty).toBe(true);expect(store.draft!.label).toBe('保留输入')
  wrapper.unmount();wrapper=mount(ModelsRoute,options);await flushPromises();await button(wrapper,'编辑').trigger('click');expect((wrapper.get('input[aria-label="模版名称"]').element as HTMLInputElement).value).toBe('保留输入');expect(modelApi.save).not.toHaveBeenCalled();wrapper.unmount()
 })
 it('restores per-view scroll and list trigger focus without changing business state',async()=>{
  const key=ref('list');const Sample=defineComponent({setup(){const root=ref<HTMLElement|null>(null);useModelViewPosition(root,()=>key.value);return{root,key}},template:`<section ref="root"><button v-if="key==='list'" data-model-focus="edit:one">编辑</button><input v-else aria-label="详情" /></section>`})
  const main=document.createElement('main');main.className='main';document.body.append(main)
  const wrapper=mount(Sample,{attachTo:main});await flushPromises();const trigger=wrapper.get('button').element as HTMLElement;trigger.focus();main.scrollTop=240
  key.value='detail';await flushPromises();expect(main.scrollTop).toBe(0);main.scrollTop=90
  key.value='list';await flushPromises();expect(main.scrollTop).toBe(240);expect(document.activeElement).toBe(wrapper.get('button').element)
  wrapper.unmount();main.remove();await nextTick()
 })
})
