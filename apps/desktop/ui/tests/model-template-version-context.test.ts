import {beforeEach,describe,it,expect,vi} from 'vitest'
import {mount,flushPromises} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import {defineComponent} from 'vue'
import ModelsRoute from '@/routes/ModelsRoute.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import {useModelsStore} from '@/features/models/store'
import {useModalStore} from '@/app/modal/store'
import {modelApi,emptyContent,type ModelSnapshot} from '@/features/models/api'
vi.mock('@tauri-apps/api/event',()=>({listen:vi.fn(async()=>()=>{})}))
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{load:vi.fn(),setDefault:vi.fn()}}))
function snapshot():ModelSnapshot{return{library:{schemaVersion:1,templates:[{id:'family',label:'<img src=x>模型族',source:'user',defaultRevision:'old',revisions:['old','new'].map(id=>({id,createdAt:'2026-10-20T00:00:00Z',savedLabel:id,digest:id,baseRevision:null,changes:[],content:emptyContent()}))}]},policy:{schemaVersion:1,templateRefs:[{templateId:'family',revisionId:'old'}],channels:[]},workspace:null,libraryDigest:'library',policyDigest:'policy',workspaceDigest:'workspace'}}
const dialog=defineComponent({props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'})
function render(){return mount(ModelsRoute,{global:{stubs:{AppTopbar:true,ChannelsView:true,AgentPanel:true,BatchAgentDialog:true,ExternalTaskDialog:true,UiDialog:dialog,UiSelectField:true,ModelRuleFields:true}}})}
beforeEach(()=>{setActivePinia(createPinia());vi.clearAllMocks();vi.mocked(modelApi.load).mockResolvedValue(snapshot());vi.mocked(modelApi.setDefault).mockImplementation(async()=>{const s=snapshot();s.library.templates[0]!.defaultRevision='new';return s});useModelsStore().view.tab='templates'})
describe('template browser/default version separation',()=>{
 it('starts from the saved default and prevents list version changes while an editor buffer is dirty',async()=>{
  const wrapper=render();await flushPromises();const select=wrapper.findAllComponents(UiSelectField).find(c=>String(c.props('label')).endsWith('保存版本'))!
  expect(select.props('modelValue')).toBe('old');expect((select.props('options') as {value:string;label:string}[]).find(o=>o.value==='old')!.label).toContain('默认')
  const store=useModelsStore();store.edit(store.snapshot!.library.templates[0]!,'old');store.draft!.label='dirty edit';await flushPromises();expect(select.props('disabled')).toBe(true)
  select.vm.$emit('update:modelValue','new');await flushPromises();expect(store.selected.family).toBeUndefined();expect(store.draft!.baseRevision).toBe('old');wrapper.unmount()
 })
 it('confirms default changes with an escaped label and rejects a stale confirmation without upgrading adoption',async()=>{
  const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='版本管理')!.trigger('click');await flushPromises()
  const button=wrapper.findAll('button').find(b=>b.text()==='设为默认')!;await button.trigger('click');const modal=useModalStore();expect(modelApi.setDefault).not.toHaveBeenCalled();expect(modal.current!.body).toContain('&lt;img src=x&gt;');expect(modal.current!.body).not.toContain('<img')
  const store=useModelsStore();store.snapshot!.libraryDigest='later';modal.resolve('confirm');await flushPromises();expect(modelApi.setDefault).not.toHaveBeenCalled();expect(wrapper.text()).toContain('确认已过期')
  store.snapshot!.libraryDigest='library';await button.trigger('click');modal.resolve('confirm');await flushPromises();expect(modelApi.setDefault).toHaveBeenCalledWith('library','family','new');expect(store.snapshot!.policy.templateRefs).toEqual([{templateId:'family',revisionId:'old'}]);wrapper.unmount()
 })
})
