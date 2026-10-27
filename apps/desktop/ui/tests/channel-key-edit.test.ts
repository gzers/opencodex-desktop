import {beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import ChannelEditor from '@/features/models/ChannelEditor.vue'
import {useModelsStore} from '@/features/models/store'
import {emptyRules,modelApi,type ChannelPolicy} from '@/features/models/api'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{credential:vi.fn(),discover:vi.fn(),project:vi.fn(),measureNodes:vi.fn(),forgetCredential:vi.fn()}}))
function channel():ChannelPolicy{return{id:'channel',label:'demo',templateRefs:[],modelRefs:{},connection:{providerId:'demo',protocol:'openai-chat',nodes:[{id:'node',name:'默认节点',url:'https://example.test'}],activeNodeId:'node',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'all'},modelOrder:['model'],discovered:[{id:'model',label:'model',rules:emptyRules()}],defaults:emptyRules(),overrides:{},familyOverrides:{}}}
function render(){const s=useModelsStore();const c=channel();s.acceptTransfer({library:{schemaVersion:1,templates:[]},libraryDigest:'library',policyDigest:'file',policy:{schemaVersion:1,templateRefs:[],channels:[c]},workspace:null,workspaceDigest:''});return mount(ChannelEditor,{props:{channel:s.channelDraft!.channels[0]!,templates:[],existing:true},global:{stubs:{ChannelModelTable:true,UiSelectField:true}}})}
beforeEach(()=>{setActivePinia(createPinia());vi.clearAllMocks();vi.mocked(modelApi.credential).mockResolvedValue({configured:true,value:'fixture-existing-key',source:'配置文件',viewable:true})})
describe('editing an existing channel Key',()=>{
 it('loads the stored value masked, lets the eye reveal and hide it without changing configuration',async()=>{
  const w=render();await flushPromises();const key=w.get<HTMLInputElement>('input[aria-label="API Key"]')
  expect(key.element.value).toBe('fixture-existing-key');expect(key.attributes('type')).toBe('password');expect(useModelsStore().channelDirty).toBe(false)
  await w.get('button[aria-label="显示 API Key"]').trigger('click');expect(key.attributes('type')).toBe('text')
  await w.get('button[aria-label="隐藏 API Key"]').trigger('click');expect(key.attributes('type')).toBe('password');expect(useModelsStore().credentialEdits).toEqual({});w.unmount()
 })
 it('records replacing or clearing only the Key as a saveable edit without fetching a model list',async()=>{
  const w=render();await flushPromises();const key=w.get('input[aria-label="API Key"]');await key.setValue('fixture-replacement-key')
  expect(useModelsStore().channelDirty).toBe(true);expect(useModelsStore().credentialEdits.channel).toEqual({channelId:'channel',action:'replace',value:'fixture-replacement-key'});expect(modelApi.discover).not.toHaveBeenCalled()
  await key.setValue('');expect(useModelsStore().credentialEdits.channel).toBeUndefined()
  await w.findAll('button').find(b=>b.text()==='清空 Key')!.trigger('click');expect(useModelsStore().credentialEdits.channel).toEqual({channelId:'channel',action:'clear',value:null});w.unmount()
 })
 it('retains the entered value and its save intent when model discovery fails',async()=>{
  vi.mocked(modelApi.discover).mockRejectedValue({message:'离线'})
  const w=render();await flushPromises();const key=w.get<HTMLInputElement>('input[aria-label="API Key"]');await key.setValue('fixture-replacement-key')
  await w.findAll('button').find(b=>b.text()==='获取模型列表')!.trigger('click');await flushPromises()
  expect(key.element.value).toBe('fixture-replacement-key');expect(useModelsStore().credentialEdits.channel!.action).toBe('replace');expect(modelApi.discover).toHaveBeenCalledWith(expect.objectContaining({providerId:'demo'}));w.unmount()
 })
 it('uses the newly saved Key as the baseline for a second edit without leaving the editor',async()=>{
  const w=render();await flushPromises();const s=useModelsStore();await w.get('input[aria-label="API Key"]').setValue('fixture-second-key')
  vi.mocked(modelApi.credential).mockResolvedValue({configured:true,value:'fixture-second-key',source:'配置文件',viewable:true})
  s.acceptTransfer({...s.snapshot!,policyDigest:'after-save'});await flushPromises()
  await w.get('input[aria-label="API Key"]').setValue('fixture-existing-key')
  expect(s.credentialEdits.channel!.value).toBe('fixture-existing-key');w.unmount()
 })
 it('keeps loaded Keys behind backend origin validation after the node URL changes',async()=>{
  vi.mocked(modelApi.discover).mockRejectedValue({code:'credential_unconfirmed',message:'该节点没有可确认的凭据'})
  const w=render();await flushPromises();useModelsStore().channelDraft!.channels[0]!.connection.nodes[0]!.url='https://other.example.test'
  await w.findAll('button').find(b=>b.text()==='获取模型列表')!.trigger('click');await flushPromises()
  expect(modelApi.discover).toHaveBeenCalledWith(expect.objectContaining({url:'https://other.example.test',providerId:'demo',apiKey:null}));w.unmount()
 })
})
