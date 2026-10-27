import {beforeEach,describe,expect,it,vi} from 'vitest'
import {mount,flushPromises} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import ChannelsView from '@/features/models/ChannelsView.vue'
import {useModelsStore} from '@/features/models/store'
import {useModelSaveFeedback} from '@/app/feedback/modelSave'
import {useFeedbackStore} from '@/app/feedback/store'
import {emptyRules,modelApi,type ModelSnapshot,type ModelPolicy} from '@/features/models/api'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{writeFile:vi.fn(),project:vi.fn(),executionDirectory:vi.fn().mockResolvedValue({rows:[]}),policySaveBlocked:vi.fn()}}))
const editor={props:['channel','dirty','saveDisabled','saveStatus'],emits:['save'],template:'<section class="detail"><span class="status">{{saveStatus}}</span><button class="detail-save" :disabled="saveDisabled" @click="$emit(\'save\')">保存</button></section>'}
function initial():ModelSnapshot{return {library:{schemaVersion:1,templates:[]},libraryDigest:'library',policyDigest:'file-before',workspaceDigest:'unused',workspace:null,policy:{schemaVersion:1,templateRefs:[],channels:['a','b'].map(id=>({id,label:id,templateRefs:[],modelRefs:{},connection:{providerId:id,protocol:'openai-chat',nodes:[{id:'node',name:'默认节点',url:'https://example.test'}],activeNodeId:'node',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'whitelist',models:['custom-id']},modelOrder:['custom-id'],discovered:[{id:'custom-id',label:'custom-id',rules:emptyRules()}],defaults:emptyRules(),overrides:{},familyOverrides:{}}))}}}
function render(){return mount(ChannelsView,{global:{stubs:{ChannelEditor:editor,ChannelNodeSelect:true,UiDialog:{props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'}}}})}
function receipt(policy:ModelPolicy){return {changed:true,snapshot:{...initial(),policy:structuredClone(policy),policyDigest:'file-after'}}}
beforeEach(()=>{setActivePinia(createPinia());vi.clearAllMocks();const s=useModelsStore();s.acceptTransfer(initial());s.view.channelEditor='a';vi.mocked(modelApi.project).mockResolvedValue({rows:[],knownActiveModels:1,exactCatalogCount:null,notices:[]})})
describe('current-channel Save navigation and state',()=>{
 it.each(['detail','inactive route'])('discards a delayed list preview after entering %s',async target=>{
  const s=useModelsStore();s.view.channelEditor='';s.channelDraft!.channels[0]!.label='pending';let finish!:(value:Awaited<ReturnType<typeof modelApi.project>>)=>void
  vi.mocked(modelApi.project).mockReturnValueOnce(new Promise(resolve=>{finish=resolve}));const w=render();await flushPromises();await w.get('[data-testid="policy-save"]').trigger('click');await flushPromises()
  if(target==='detail')s.view.channelEditor='a';else await w.setProps({active:false});await flushPromises()
  finish({rows:[],knownActiveModels:1,exactCatalogCount:null,notices:[]});await flushPromises()
  expect(w.text()).not.toContain('确认保存');expect(modelApi.writeFile).not.toHaveBeenCalled();expect(s.view.channelEditor).toBe(target==='detail'?'a':'');w.unmount()
 })
 it('keeps clean A disabled when B has edits, including stale emitted events',async()=>{
  const s=useModelsStore();s.channelDraft!.channels[1]!.label='B pending';const w=render();await flushPromises()
  expect(w.get('.detail-save').attributes('disabled')).toBeDefined();expect(w.get('.status').text()).toBe('未修改')
  w.getComponent(editor).vm.$emit('save');await flushPromises();expect(modelApi.writeFile).not.toHaveBeenCalled();expect(s.view.channelEditor).toBe('a');w.unmount()
 })
 it('writes A without a preview and returns to list with B still dirty',async()=>{
  const s=useModelsStore();s.channelDraft!.channels[0]!.label='A saved';s.channelDraft!.channels[1]!.label='B pending'
  vi.mocked(modelApi.writeFile).mockImplementation(async(_expected,policy)=>receipt(policy));const shown=vi.spyOn(useModelSaveFeedback(),'showSaved');const w=render();await flushPromises();await w.get('.detail-save').trigger('click');await flushPromises()
  expect(modelApi.writeFile).toHaveBeenCalledOnce();expect(vi.mocked(modelApi.writeFile).mock.calls[0]![3]).toEqual({channelId:'a'});expect(s.view.channelEditor).toBe('');expect(s.channelIsDirty('a')).toBe(false);expect(s.channelIsDirty('b')).toBe(true);expect(shown).toHaveBeenCalledOnce();expect(w.text()).not.toContain('确认保存');expect(w.get('[data-testid="policy-save"]').attributes('disabled')).toBeUndefined();w.unmount()
 })
 it('keeps the detail and input on file failure',async()=>{
  const s=useModelsStore();s.channelDraft!.channels[0]!.label='A retained';vi.mocked(modelApi.writeFile).mockRejectedValue({message:'配置文件已变化'});const w=render();await flushPromises();await w.get('.detail-save').trigger('click');await flushPromises();expect(s.view.channelEditor).toBe('a');expect(s.channelDraft!.channels[0]!.label).toBe('A retained');expect(useFeedbackStore().toast).toContain('未写入');expect(w.get('.detail-save').attributes('disabled')).toBeUndefined();w.unmount()
 })
 it.each(['other detail','inactive route','later input'])('does not navigate on a late receipt after %s',async change=>{
  const s=useModelsStore();s.channelDraft!.channels[0]!.label='A submitting';let resolve!:(value:ReturnType<typeof receipt>)=>void;let policy!:ModelPolicy
  vi.mocked(modelApi.writeFile).mockImplementation(async(_expected,value)=>{policy=structuredClone(value);return new Promise(r=>{resolve=r})});const w=render();await flushPromises();w.getComponent(editor).vm.$emit('save');await flushPromises();w.getComponent(editor).vm.$emit('save');await flushPromises();expect(modelApi.writeFile).toHaveBeenCalledOnce()
  if(change==='other detail')s.view.channelEditor='b';else if(change==='inactive route')await w.setProps({active:false});else s.channelDraft!.channels[0]!.label='later A input'
  resolve(receipt(policy));await flushPromises();expect(s.view.channelEditor).toBe(change==='other detail'?'b':'a');if(change==='later input'){expect(s.channelDraft!.channels[0]!.label).toBe('later A input');expect(s.channelIsDirty('a')).toBe(true)}w.unmount()
 })
 it('keeps the clean list Save disabled after saving the only edited channel',async()=>{
  const s=useModelsStore();s.channelDraft!.channels[0]!.label='A saved';vi.mocked(modelApi.writeFile).mockImplementation(async(_expected,policy)=>receipt(policy));const w=render();await flushPromises();await w.get('.detail-save').trigger('click');await flushPromises();expect(w.get('[data-testid="policy-save"]').attributes('disabled')).toBeDefined();await w.get('[data-testid="policy-save"]').trigger('click');await flushPromises();expect(modelApi.writeFile).toHaveBeenCalledOnce();expect(w.text()).not.toContain('确认保存');w.unmount()
 })
})
