import {beforeEach,describe,expect,it,vi} from 'vitest'
import {mount,flushPromises} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import ChannelsView from '@/features/models/ChannelsView.vue'
import {useModelsStore} from '@/features/models/store'
import {useModelSaveFeedback} from '@/app/feedback/modelSave'
import {useFeedbackStore} from '@/app/feedback/store'
import {useModalStore} from '@/app/modal/store'
import {emptyRules,modelApi,type ModelSnapshot} from '@/features/models/api'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{latestPolicy:vi.fn(),policyResult:vi.fn(),applyPolicy:vi.fn(),preparePolicy:vi.fn(),writeFile:vi.fn(),load:vi.fn(),project:vi.fn(),policySaveBlocked:vi.fn().mockResolvedValue(undefined)}}))
const dialog={props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'}
function initial():ModelSnapshot{return {library:{schemaVersion:1,templates:[]},libraryDigest:'library',policyDigest:'current-file',workspaceDigest:'unused',workspace:null,policy:{schemaVersion:1,templateRefs:[],channels:[{id:'channel',label:'demo',templateRefs:[],modelRefs:{},connection:{providerId:'demo',protocol:'openai-chat',nodes:[{id:'node',name:'默认节点',url:'https://example.test'}],activeNodeId:'node',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'whitelist',models:['custom-id']},modelOrder:['custom-id'],discovered:[{id:'custom-id',label:'custom-id',rules:emptyRules()}],defaults:emptyRules(),overrides:{},familyOverrides:{}}]}}}
function render(){return mount(ChannelsView,{global:{stubs:{UiDialog:dialog,ChannelEditor:true,ChannelNodeSelect:true,UiSelectField:true}}})}
async function confirm(w:ReturnType<typeof render>){await w.get('[data-testid="policy-save"]').trigger('click');await flushPromises();await w.findAll('button').find(b=>b.text()==='确认保存')!.trigger('click');await flushPromises()}
beforeEach(()=>{setActivePinia(createPinia());vi.clearAllMocks();useModelsStore().acceptTransfer(initial());vi.mocked(modelApi.project).mockResolvedValue({rows:[],knownActiveModels:0,exactCatalogCount:null,notices:[]})})
describe('current-file channel saves',()=>{
 it('does not reload or gate on unfinished historical plans',async()=>{
  vi.mocked(modelApi.latestPolicy).mockRejectedValue({code:'plan_incomplete'});const w=render();await flushPromises()
  expect(modelApi.latestPolicy).not.toHaveBeenCalled();expect(w.text()).not.toContain('待重启');expect(w.text()).not.toContain('调整本次保存范围');w.unmount()
 })
 it('confirms the preview and writes the current file exactly once',async()=>{
  const s=useModelsStore();s.channelDraft!.channels[0]!.connection.enabled=false
  vi.mocked(modelApi.writeFile).mockImplementation(async(_expected,policy)=>({changed:true,snapshot:{...initial(),policy:structuredClone(policy),policyDigest:'written-file'}}))
  const show=vi.spyOn(useModelSaveFeedback(),'showSaved');const w=render();await flushPromises();await confirm(w)
  expect(modelApi.writeFile).toHaveBeenCalledTimes(1);expect(vi.mocked(modelApi.writeFile).mock.calls[0]![0]).toBe('current-file');expect(modelApi.preparePolicy).not.toHaveBeenCalled();expect(modelApi.applyPolicy).not.toHaveBeenCalled();expect(s.channelDirty).toBe(false);expect(show).toHaveBeenCalledOnce();expect(w.find('.save-result').exists()).toBe(false);w.unmount()
 })
 it('saves a Key-only edit independently of model discovery',async()=>{
  const s=useModelsStore();const policy=JSON.stringify(s.channelDraft);s.editCredential('channel','replacement-value','original-value')
  vi.mocked(modelApi.writeFile).mockResolvedValue({changed:true,snapshot:{...initial(),policyDigest:'new-file'}})
  const w=render();await flushPromises();await confirm(w)
  expect(vi.mocked(modelApi.writeFile).mock.calls[0]![2]).toEqual([{channelId:'channel',action:'replace',value:'replacement-value'}]);expect(JSON.stringify(s.channelDraft)).toBe(policy);expect(s.credentialEdits).toEqual({});w.unmount()
 })
 it.each(['modeled fields','raw material only'])('adopts %s in memory, then uses the existing Save with the actual file digest and receipt',async kind=>{
  const s=useModelsStore(),incoming=initial().policy
  incoming.channels[0]!.connection.importedConfig={id:'imported-source',digest:'raw-provider-material'}
  if(kind==='modeled fields'){
   s.channelDraft!.channels[0]!.label='unsaved local label';s.editCredential('channel','unsaved replacement','saved key')
   incoming.channels[0]!.defaults.contextWindow={mode:'custom',value:128000}
  }
  await s.adoptTransfer({expectedPolicy:'current-file',mergedPolicy:incoming},async()=>({...initial(),libraryDigest:'received-library',workspaceDigest:'received-workspace',workspace:{basePolicy:'current-file',policy:incoming}}))
  expect(modelApi.writeFile).not.toHaveBeenCalled();expect(s.channelDirty).toBe(true);expect(s.channelBaseline).toBe('current-file')
  let written:ModelSnapshot|undefined
  vi.mocked(modelApi.writeFile).mockImplementation(async(_expected,policy)=>{
   const consumed=structuredClone(policy);delete consumed.channels[0]!.connection.importedConfig
   written={...initial(),libraryDigest:'received-library',policyDigest:'written-file',policy:consumed}
   return {changed:true,snapshot:written}
  })
  const w=render();await flushPromises();await confirm(w)
  expect(modelApi.writeFile).toHaveBeenCalledOnce()
  const [expected,policy,keys]=vi.mocked(modelApi.writeFile).mock.calls[0]!
  expect(expected).toBe('current-file');expect(policy.channels[0]!.connection.importedConfig).toEqual({id:'imported-source',digest:'raw-provider-material'})
  if(kind==='modeled fields'){
   expect(policy.channels[0]!.label).toBe('unsaved local label');expect(policy.channels[0]!.defaults.contextWindow).toEqual({mode:'custom',value:128000})
   expect(keys).toEqual([{channelId:'channel',action:'replace',value:'unsaved replacement'}])
  }else expect(keys).toEqual([])
  expect(s.channelDirty).toBe(false);expect(s.credentialEdits).toEqual({});expect(s.channelBaseline).toBe('written-file')
  vi.mocked(modelApi.load).mockResolvedValue(written!);await s.load()
  expect(s.channelDraft).toEqual(written!.policy);expect(s.channelDraft!.channels[0]!.connection.importedConfig).toBeUndefined();w.unmount()
 })
 it('retains edits on a real file conflict and reports an unsuccessful write',async()=>{
  const s=useModelsStore();s.channelDraft!.channels[0]!.label='edited';vi.mocked(modelApi.writeFile).mockRejectedValue({code:'file_changed',message:'配置文件已在其它地方修改'})
  const w=render();await flushPromises();await confirm(w)
  expect(s.channelDirty).toBe(true);expect(s.channelDraft!.channels[0]!.label).toBe('edited');expect(useFeedbackStore().toast).toContain('未写入');expect(w.text()).not.toContain('调整本次保存范围');w.unmount()
 })
 it('refreshes clean data from file and retains only unsaved user edits',async()=>{
  const s=useModelsStore();const fresh=initial();fresh.policy.channels[0]!.connection.nodes[0]!.url='https://new.example.test';fresh.policyDigest='external-file';vi.mocked(modelApi.load).mockResolvedValue(fresh)
  await s.load();expect(s.channelDraft!.channels[0]!.connection.nodes[0]!.url).toBe('https://new.example.test');s.channelDraft!.channels[0]!.label='unsaved'
  await s.load();expect(s.channelDraft!.channels[0]!.label).toBe('unsaved');expect(s.channelBaseline).toBe('external-file')
 })
 it('deleting a different channel preserves the official default reference',async()=>{
  const s=useModelsStore(),snap=initial(),other=structuredClone(snap.policy.channels[0]!);other.id='default';other.connection.providerId='other';other.label='other';snap.policy.channels.push(other);snap.policy.defaultChannelId='default';s.acceptTransfer(snap)
  const w=render();await flushPromises();await w.findAll('button').find(b=>b.text()==='删除')!.trigger('click');useModalStore().resolve('confirm');await flushPromises()
  expect(s.channelDraft!.channels.map(c=>c.id)).toEqual(['default']);expect(s.channelDraft!.defaultChannelId).toBe('default');w.unmount()
 })
})
