import {beforeEach,describe,expect,it,vi} from 'vitest'
import {mount,flushPromises} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import {defineComponent} from 'vue'
import ChannelBatchDialog from '@/features/models/ChannelBatchDialog.vue'
import UiSelectField from '@/components/ui/UiSelectField.vue'
import ModelRuleFields from '@/features/models/ModelRuleFields.vue'
import {useModelsStore} from '@/features/models/store'
import {emptyRules,emptyContent,modelApi,type ChannelPolicy,type ModelSnapshot,type ModelPolicy,type DraftProjection} from '@/features/models/api'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{project:vi.fn(),preview:vi.fn()}}))
function channel(id:string):ChannelPolicy{return{id,label:'渠道 '+id,templateRefs:[{templateId:'family',revisionId:'version-a'}],modelRefs:{},connection:{providerId:id,protocol:'openai-chat',nodes:[],activeNodeId:'',credentialRef:'keep-local',enabled:false,allowPrivateNetwork:false},selection:{mode:'whitelist',models:['a']},discovered:[{id:'a',label:'a',rules:emptyRules()},{id:'b',label:'b',rules:emptyRules()}],modelOrder:['b','a'],defaults:emptyRules(),familyOverrides:{family:{...emptyRules(),fast:{mode:'custom',value:true}}},overrides:{a:{...emptyRules(),contextWindow:{mode:'custom',value:32000}},b:emptyRules()}}}
function projection(policy:ModelPolicy):DraftProjection{return{rows:policy.channels.flatMap(c=>c.discovered.map(m=>({channelId:c.id,providerId:c.connection.providerId,modelId:m.id,label:m.label,enabled:false,selected:false,discovered:true,templateRef:{templateId:'family',revisionId:'version-a'},rules:emptyRules(),source:{}}))),knownActiveModels:0,exactCatalogCount:0,notices:[]}}
const dialog=defineComponent({template:'<section><slot/><slot name="actions"/></section>'})
function render(mode:'adopt'|'family'='adopt'){return mount(ChannelBatchDialog,{props:{mode,targets:['one','two']},global:{stubs:{UiDialog:dialog,UiSelectField:true,ModelRuleFields:true}}})}
function choose(wrapper:ReturnType<typeof render>,label:string,value:string){wrapper.findAllComponents(UiSelectField).find(c=>c.props('label')===label)!.vm.$emit('update:modelValue',value);return flushPromises()}
const click=(wrapper:ReturnType<typeof render>,name:string)=>wrapper.findAll('button').find(b=>b.text()===name)!.trigger('click')
beforeEach(()=>{setActivePinia(createPinia());vi.clearAllMocks();const store=useModelsStore();const state:ModelSnapshot={library:{schemaVersion:1,templates:[{id:'family',label:'测试族',source:'user',defaultRevision:'version-b',revisions:['version-a','version-b'].map(id=>({id,createdAt:'2026-10-20T00:00:00Z',savedLabel:'测试族',digest:id,baseRevision:null,changes:[],content:emptyContent()}))}]},policy:{schemaVersion:1,templateRefs:[],channels:[channel('one'),channel('two'),channel('outside')]},workspace:null,libraryDigest:'lib',policyDigest:'saved',workspaceDigest:'workspace'};store.snapshot=state;store.restoreChannels();vi.mocked(modelApi.project).mockImplementation(async p=>projection(p));vi.mocked(modelApi.preview).mockImplementation(async(_content,_channel,id)=>({matched:id==='a',reason:'fixture',variantId:null,rules:emptyRules()}))})
describe('atomic channel-family batch review',()=>{
 it('adopts an exact saved revision only after confirmation and preserves other channels, exceptions, connection, selection and order',async()=>{
  const store=useModelsStore(),before=JSON.stringify(store.channelDraft),wrapper=render()
  await choose(wrapper,'模型族 / 模版','family');await choose(wrapper,'精确保存版本','version-b');await click(wrapper,'比较本次修改');await flushPromises()
  expect(JSON.stringify(store.channelDraft)).toBe(before);await click(wrapper,'加入草稿');await flushPromises()
  expect(store.channelDraft!.channels.map(c=>c.templateRefs[0]?.revisionId)).toEqual(['version-b','version-b','version-a']);expect(wrapper.emitted('close')).toHaveLength(1)
  const expected=JSON.parse(before);expected.channels[0].templateRefs[0].revisionId='version-b';expected.channels[1].templateRefs[0].revisionId='version-b';expect(store.channelDraft).toEqual(expected);expect(store.snapshot!.library.templates[0]!.defaultRevision).toBe('version-b');wrapper.unmount()
 })
 it('can explicitly change the global family default without rewriting local adoption or model rules',async()=>{
  const store=useModelsStore(),before=JSON.stringify(store.channelDraft!.channels),wrapper=render();await choose(wrapper,'模型族 / 模版','family');await choose(wrapper,'采用范围','global');await click(wrapper,'比较本次修改');await flushPromises();await click(wrapper,'加入草稿')
  expect(store.channelDraft!.templateRefs).toEqual([{templateId:'family',revisionId:'version-b'}]);expect(JSON.stringify(store.channelDraft!.channels)).toBe(before);wrapper.unmount()
 })
 it('clears only explicitly selected matching model rule overrides, preserving model refs and naming',async()=>{
  const store=useModelsStore();store.channelDraft!.channels[0]!.modelRefs={a:{templateId:'family',revisionId:'version-a'}};store.channelDraft!.channels[0]!.naming={a:{alias:'route-a'}}
  const wrapper=render();await choose(wrapper,'模型族 / 模版','family');await wrapper.get('input[type=checkbox]').setValue(true);await click(wrapper,'比较本次修改');await flushPromises();expect(store.channelDraft!.channels[0]!.overrides.a).toBeDefined();await click(wrapper,'加入草稿')
  expect(store.channelDraft!.channels[0]!.overrides.a).toBeUndefined();expect(store.channelDraft!.channels[0]!.overrides.b).toBeDefined();expect(store.channelDraft!.channels[2]!.overrides.a).toBeDefined();expect(store.channelDraft!.channels[0]!.modelRefs.a!.revisionId).toBe('version-a');expect(store.channelDraft!.channels[0]!.naming!.a!.alias).toBe('route-a');wrapper.unmount()
 })
 it('changes only chosen family fields and refuses a stale confirmation without changing later input',async()=>{
  const store=useModelsStore(),wrapper=render('family');await choose(wrapper,'模型族 / 模版','family');await choose(wrapper,'上下文窗口批量操作','custom');wrapper.getComponent(ModelRuleFields).vm.$emit('update:modelValue',{...emptyRules(),contextWindow:{mode:'custom',value:64000}});await flushPromises();await click(wrapper,'比较本次修改');await flushPromises()
  store.channelDraft!.channels[2]!.label='later outside input';await click(wrapper,'加入草稿');expect(wrapper.text()).toContain('确认已过期');expect(store.channelDraft!.channels[0]!.familyOverrides.family!.contextWindow.mode).toBe('inherit')
  await click(wrapper,'比较本次修改');await flushPromises();await click(wrapper,'加入草稿');expect(store.channelDraft!.channels[0]!.familyOverrides.family!.contextWindow).toEqual({mode:'custom',value:64000});expect(store.channelDraft!.channels[0]!.familyOverrides.family!.fast).toEqual({mode:'custom',value:true});expect(store.channelDraft!.channels[2]!.label).toBe('later outside input');wrapper.unmount()
 })
 it('rejects a late comparison after any workspace drift and refuses unresolved row buffers',async()=>{
  const store=useModelsStore(),wrapper=render();await choose(wrapper,'模型族 / 模版','family')
  let finish!:(value:DraftProjection)=>void;vi.mocked(modelApi.project).mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve}))
  await click(wrapper,'比较本次修改');store.channelDraft!.channels[1]!.label='later input';finish(projection(store.channelDraft!));await flushPromises()
  expect(wrapper.text()).toContain('比较期间工作区');expect(wrapper.findAll('button').find(b=>b.text()==='加入草稿')!.attributes('disabled')).toBeDefined()
  const base={rules:emptyRules(),naming:{},reference:null};store.modelRows.one={selected:[],expanded:[],buffers:{a:{...base,rules:{...emptyRules(),fast:{mode:'custom',value:true}},base,error:''}}}
  await click(wrapper,'比较本次修改');await flushPromises();expect(wrapper.text()).toContain('未确认型号编辑');expect(store.channelDraft!.channels[1]!.label).toBe('later input');wrapper.unmount()
 })
 it('stops comparison without accepting a late result or changing the draft',async()=>{
  const store=useModelsStore(),before=JSON.stringify(store.channelDraft),wrapper=render();await choose(wrapper,'模型族 / 模版','family')
  let finish!:(value:DraftProjection)=>void;vi.mocked(modelApi.project).mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve}))
  await click(wrapper,'比较本次修改');await click(wrapper,'停止比较');finish(projection(store.channelDraft!));await flushPromises()
  expect(wrapper.text()).toContain('比较已停止');expect(JSON.stringify(store.channelDraft)).toBe(before);expect(wrapper.findAll('button').find(b=>b.text()==='加入草稿')!.attributes('disabled')).toBeDefined();wrapper.unmount()
 })
 it('keeps the complete batch unchanged after any target validation fails',async()=>{
  const store=useModelsStore(),before=JSON.stringify(store.channelDraft),wrapper=render();await choose(wrapper,'模型族 / 模版','family');vi.mocked(modelApi.project).mockRejectedValueOnce({message:'second channel invalid'})
  await click(wrapper,'比较本次修改');await flushPromises();expect(wrapper.text()).toContain('second channel invalid');expect(JSON.stringify(store.channelDraft)).toBe(before);expect(wrapper.findAll('button').find(b=>b.text()==='加入草稿')!.attributes('disabled')).toBeDefined();wrapper.unmount()
 })
})
