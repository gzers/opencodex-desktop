import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import {reactive} from 'vue'
import ChannelModelTable from '@/features/models/ChannelModelTable.vue'
import ModelRuleFields from '@/features/models/ModelRuleFields.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import {createModelRowsState,pendingModelRows,rowFields,writeRow} from '@/features/models/modelRows'
import {emptyRules,modelApi,type ChannelPolicy,type DraftProjection} from '@/features/models/api'
import {useModelsStore} from '@/features/models/store'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{project:vi.fn()}}))
const dialog={props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'}
const fixture=():ChannelPolicy=>reactive({id:'channel',label:'测试渠道',connection:{providerId:'provider',protocol:'openai-chat',nodes:[{id:'node',name:'主站',url:'https://example.test'}],activeNodeId:'node',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'whitelist',models:['raw/a']},modelOrder:['raw/a','missing','raw/b'],discovered:['raw/a','raw/b'].map(id=>({id,label:id,rules:emptyRules()})),templateRefs:[],modelRefs:{},defaults:emptyRules(),overrides:{},familyOverrides:{}})
const result=():DraftProjection=>({rows:[],knownActiveModels:0,exactCatalogCount:null,notices:[]})
const button=(wrapper:ReturnType<typeof mount>,name:string)=>wrapper.findAll('button').find(b=>b.text()===name)!
let pinia:ReturnType<typeof createPinia>
beforeEach(()=>{pinia=createPinia();setActivePinia(pinia);vi.clearAllMocks();vi.mocked(modelApi.project).mockResolvedValue(result())})
afterEach(()=>pinia._s.forEach(store=>store.$dispose()))
function render(channel=fixture(),state=reactive(createModelRowsState())){return{channel,state,wrapper:mount(ChannelModelTable,{props:{channel,state,models:channel.discovered,templates:[]},global:{stubs:{UiDialog:dialog,UiSelectField:true,ModelRuleFields:true}}})}}
describe('channel model row workspace',()=>{
 it('separates action selection from visibility and retains row buffers across collapse, filter and remount',async()=>{
  const {wrapper,channel,state}=render();await wrapper.get('input[aria-label="操作选择 raw/b"]').setValue(true);expect(channel.selection).toEqual({mode:'whitelist',models:['raw/a']})
  const row=wrapper.get('tr[data-model="raw/a"]');await button(row as never,'展开编辑').trigger('click')
  const fields=wrapper.findComponent(ModelRuleFields);fields.vm.$emit('update:modelValue',{...emptyRules(),contextWindow:{mode:'custom',value:32768}});await flushPromises();expect(channel.overrides['raw/a']).toBeUndefined();expect(pendingModelRows(state)).toEqual(['raw/a'])
  await button(wrapper,'全部收起').trigger('click');expect(pendingModelRows(state)).toEqual(['raw/a']);await wrapper.setProps({models:[channel.discovered[1]!]});expect(wrapper.text()).toContain('未确认编辑');expect(pendingModelRows(state)).toEqual(['raw/a'])
  wrapper.unmount();const again=render(channel,state).wrapper;await button(again.get('tr[data-model="raw/a"]') as never,'展开编辑').trigger('click');expect(again.findComponent(ModelRuleFields).props('modelValue').contextWindow).toEqual({mode:'custom',value:32768})
  await button(again,'加入草稿').trigger('click');await flushPromises();expect(channel.overrides['raw/a']?.contextWindow).toEqual({mode:'custom',value:32768});expect(pendingModelRows(state)).toEqual([]);expect(modelApi.project).toHaveBeenCalledWith(expect.objectContaining({channels:[expect.objectContaining({id:channel.id})]}));again.unmount()
 })
 it('reverts only a rejected alias while preserving pending rule input and original IDs',async()=>{
  const {wrapper,channel,state}=render();await button(wrapper.get('tr[data-model="raw/a"]') as never,'展开编辑').trigger('click');wrapper.findComponent(ModelRuleFields).vm.$emit('update:modelValue',{...emptyRules(),fast:{mode:'custom',value:false}});await flushPromises()
  vi.mocked(modelApi.project).mockRejectedValue({message:'别名与原型号冲突'});const input=wrapper.get('input[aria-label="raw/a路由别名"]');await input.setValue('reserved');await input.trigger('blur');await flushPromises();expect((input.element as HTMLInputElement).value).toBe('');expect(channel.naming?.['raw/a']).toBeUndefined();expect(state.buffers['raw/a']!.rules.fast).toEqual({mode:'custom',value:false});expect(pendingModelRows(state)).toEqual(['raw/a']);expect(channel.discovered[0]!.id).toBe('raw/a');expect(wrapper.text()).toContain('别名与原型号冲突');wrapper.unmount()
 })
 it('rejects all batch changes on validation failure or a stale confirmation and keeps exact hidden selections',async()=>{
  const {wrapper,channel}=render();await wrapper.get('input[aria-label="选择当前型号结果"]').setValue(true);await wrapper.setProps({models:[channel.discovered[0]!]});expect(wrapper.text()).toContain('筛选外 1 项')
  await button(wrapper,'批量停用').trigger('click');vi.mocked(modelApi.project).mockRejectedValueOnce({message:'目标冲突'});await button(wrapper,'整批加入草稿').trigger('click');await flushPromises();expect(channel.selection).toEqual({mode:'whitelist',models:['raw/a']});expect(wrapper.text()).toContain('目标冲突')
  channel.naming={'raw/a':{alias:'later'}};await button(wrapper,'整批加入草稿').trigger('click');await flushPromises();expect(wrapper.text()).toContain('批量确认已过期');expect(modelApi.project).toHaveBeenCalledTimes(1);expect(channel.naming['raw/a']?.alias).toBe('later');wrapper.unmount()
 })
 it('applies only explicit batch fields atomically and preserves naming, references, order and untouched rules',async()=>{
  const {wrapper,channel}=render();channel.overrides['raw/a']={...emptyRules(),contextWindow:{mode:'custom',value:8192},fast:{mode:'custom',value:true}};channel.naming={'raw/a':{alias:'kept'}};channel.modelRefs['raw/a']={templateId:'family',revisionId:'exact'}
  await wrapper.get('input[aria-label="选择当前型号结果"]').setValue(true);await button(wrapper,'批量编辑').trigger('click');const selects=wrapper.findAllComponents(UiSelectField)
  selects.find(c=>c.props('label')==='批量上下文窗口处理')!.vm.$emit('update:modelValue','inherit');await flushPromises();await button(wrapper,'整批加入草稿').trigger('click');await flushPromises()
  expect(channel.overrides['raw/a']!.contextWindow.mode).toBe('inherit');expect(channel.overrides['raw/a']!.fast).toEqual({mode:'custom',value:true});expect(channel.naming['raw/a']!.alias).toBe('kept');expect(channel.modelRefs['raw/a']!.revisionId).toBe('exact');expect(channel.modelOrder).toEqual(['raw/a','missing','raw/b']);expect(channel.selection).toEqual({mode:'whitelist',models:['raw/a']});wrapper.unmount()
 })
 it('cancels model detail without discarding an existing row buffer and rejects a late validator result',async()=>{
  const {wrapper,channel,state}=render();await button(wrapper.get('tr[data-model="raw/a"]') as never,'展开编辑').trigger('click');wrapper.findComponent(ModelRuleFields).vm.$emit('update:modelValue',{...emptyRules(),fast:{mode:'custom',value:false}});await flushPromises()
  await button(wrapper.get('tr[data-model="raw/a"]') as never,'编辑明细').trigger('click');await button(wrapper,'取消').trigger('click');expect(state.buffers['raw/a']!.rules.fast).toEqual({mode:'custom',value:false})
  let finish!:(value:DraftProjection)=>void;vi.mocked(modelApi.project).mockReturnValue(new Promise(resolve=>{finish=resolve}));await button(wrapper,'加入草稿').trigger('click');channel.defaults.fast={mode:'custom',value:true};finish(result());await flushPromises();expect(channel.overrides['raw/a']).toBeUndefined();expect(wrapper.text()).toContain('配置基准已变化');expect(pendingModelRows(state)).toEqual(['raw/a']);wrapper.unmount()
 })
 it('treats prototype-shaped raw IDs as literal dictionary keys without changing object prototypes',()=>{
  const channel=fixture(),state=createModelRowsState();for(const id of ['__proto__','constructor']){channel.discovered.push({id,label:id,rules:emptyRules()});const fields=rowFields(channel,id);fields.rules.fast={mode:'custom',value:false};fields.naming.displayName='Literal ID';writeRow(channel,id,fields);expect(Object.hasOwn(channel.overrides,id)).toBe(true);expect(rowFields(channel,id).rules.fast).toEqual({mode:'custom',value:false});expect(Object.getPrototypeOf(channel.overrides)).toBe(Object.prototype);expect(state.buffers[id]).toBeUndefined()}
 })
 it('exposes pending row ownership to global save without changing submitted policy or rule references',()=>{
  const store=useModelsStore();const state=reactive(createModelRowsState());state.buffers.a={rules:{...emptyRules(),fast:{mode:'custom',value:false}},naming:{},reference:null,base:{rules:emptyRules(),naming:{},reference:null},error:''};store.modelRows.channel=state;expect(store.pendingRows).toEqual([{channel:'channel',model:'a'}]);state.buffers.a.rules=emptyRules();expect(store.pendingRows).toEqual([])
 })
})
