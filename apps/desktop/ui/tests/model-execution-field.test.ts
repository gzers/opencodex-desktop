import {beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount} from '@vue/test-utils'
import {defineComponent} from 'vue'
import {createPinia,setActivePinia} from 'pinia'
import AgentExecutionField from '@/features/models/AgentExecutionField.vue'
import {modelApi,emptyRules,type AgentSettings,type ExecutionDirectory,type ModelSnapshot} from '@/features/models/api'
import {useModelsStore} from '@/features/models/store'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{agentSettings:vi.fn(),executionDirectory:vi.fn(),saveAgentSettings:vi.fn(),load:vi.fn()}}))
const dialog=defineComponent({props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'})
const picker=defineComponent({props:['open','current','rows','channels','loading','error'],emits:['confirm','close','retry'],template:"<div v-if=\"open\"><p v-if=\"error\">{{error}}</p><span v-if=\"rows.length\">{{rows[0].modelId}}</span><button v-if=\"rows.length\" class=\"pick-temp\" @click=\"$emit('confirm',[{channelId:rows[0].channelId,modelId:rows[0].modelId,reasoningEffort:'none',fast:false}])\">确认临时选择</button><button class=\"pick-retry\" @click=\"$emit('retry')\">重新读取目录</button><button class=\"pick-cancel\" @click=\"$emit('close')\">取消临时选择</button></div>"})
const saved:AgentSettings={providerId:'original',modelId:'old',reasoningEffort:null,fast:null,channelId:'old-channel',connectionDigest:'old-digest'}
/** 组件用模型配置 store 做首屏即时呈现，测试需提供 Pinia 实例。 */
function render(snapshot?:ModelSnapshot,onUpdate?:(value:AgentSettings|null)=>void){const pinia=createPinia();setActivePinia(pinia);if(snapshot)useModelsStore().acceptTransfer(snapshot);return mount(AgentExecutionField,{props:onUpdate?{'onUpdate:modelValue':onUpdate}:{},global:{plugins:[pinia],stubs:{UiDialog:dialog,ModelPicker:picker}}})}
function button(wrapper:ReturnType<typeof render>,name:string){return wrapper.findAll('button').find(b=>b.text()===name)!}
function execution(modelId='exact/native-id',label='渠道',channelId='stable'):ExecutionDirectory{return {channels:[{id:channelId,providerId:'runtime-provider',label}],rows:[{channelId,providerId:'runtime-provider',modelId,label:label+'/'+modelId,enabled:true,selected:false,discovered:true,templateRef:null,rules:emptyRules(),source:{},connectionDigest:'selected-binding'}]}}
function fileSnapshot(digest='file-1',modelId='exact/native-id',label='渠道',channelId='stable'):ModelSnapshot{return {library:{schemaVersion:1,templates:[]},libraryDigest:'library',policyDigest:digest,workspaceDigest:'unused',workspace:null,policy:{schemaVersion:1,templateRefs:[],channels:[{id:channelId,label,templateRefs:[],modelRefs:{},connection:{providerId:'runtime-provider',protocol:'openai-chat',nodes:[{id:'node',name:'默认节点',url:'https://fixture.example.test/v1'}],activeNodeId:'node',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'all'},modelOrder:[modelId],discovered:[{id:modelId,label:modelId,rules:emptyRules()}],defaults:emptyRules(),overrides:{},familyOverrides:{}}]}}}
function deferred<T>(){let resolve!:(value:T)=>void;let reject!:(cause:unknown)=>void;const promise=new Promise<T>((yes,no)=>{resolve=yes;reject=no});return {promise,resolve,reject}}
async function browse(wrapper:ReturnType<typeof render>){await button(wrapper,'更换执行模型').trigger('click');await button(wrapper,'original / old').trigger('click')}
beforeEach(()=>{vi.clearAllMocks();vi.mocked(modelApi.agentSettings).mockResolvedValue({...saved});vi.mocked(modelApi.executionDirectory).mockResolvedValue(execution());vi.mocked(modelApi.saveAgentSettings).mockImplementation(async value=>value)})
describe('page state follows the prototype',()=>{
 it('已配置：标题行摘要 + 四项事实 + 适用范围 + 运行配置边界，按钮为「更换执行模型」',async()=>{
  const wrapper=render();await flushPromises()
  expect(wrapper.text()).toContain('模型模版助手 · 本机共用')
  expect(wrapper.text()).toContain('original / old')
  expect(wrapper.text()).toContain('（模型默认）')
  for(const label of ['执行渠道','型号','推理档位','快速模式'])expect(wrapper.text()).toContain(label)
  expect(wrapper.text()).toContain('适用范围')
  expect(wrapper.text()).toContain('与运行配置的边界')
  expect(button(wrapper,'更换执行模型').exists()).toBe(true)
  expect(wrapper.text()).not.toContain('尚未选择执行模型。自动填充')
  wrapper.unmount()
 })
 it('未配置：摘要提示待配置、空态说明可见、按钮为主色「Agent 配置」',async()=>{
  vi.mocked(modelApi.agentSettings).mockResolvedValue(null)
  const wrapper=render();await flushPromises()
  expect(wrapper.text()).toContain('尚未选择执行模型 · 请配置执行模型')
  expect(wrapper.text()).toContain('尚未选择执行模型。自动填充与模版检查需要先配置一个可用的执行模型。')
  expect(wrapper.text()).not.toContain('适用范围')
  const action=button(wrapper,'Agent 配置')
  expect(action.classes()).toContain('primary')
  wrapper.unmount()
 })
})
describe('saved common execution preference',()=>{
 it('keeps model picker selection temporary until the shared settings save and returns a stable connection binding',async()=>{
  const wrapper=render();await flushPromises();expect(modelApi.executionDirectory).not.toHaveBeenCalled();await button(wrapper,'更换执行模型').trigger('click');await button(wrapper,'original / old').trigger('click');await flushPromises();await button(wrapper,'确认临时选择').trigger('click');await flushPromises()
  expect(modelApi.saveAgentSettings).not.toHaveBeenCalled();expect(wrapper.emitted('update:modelValue')?.at(-1)?.[0]).toEqual(saved)
  await button(wrapper,'保存共用设置').trigger('click');await flushPromises()
  expect(modelApi.saveAgentSettings).toHaveBeenCalledExactlyOnceWith({channelId:'stable',providerId:'runtime-provider',modelId:'exact/native-id',reasoningEffort:'none',fast:false,connectionDigest:'selected-binding'});wrapper.unmount()
 })
 it('cancelling the settings after a temporary choice retains the original committed execution preference',async()=>{
  const wrapper=render();await flushPromises();await button(wrapper,'更换执行模型').trigger('click');await button(wrapper,'original / old').trigger('click');await flushPromises();await button(wrapper,'确认临时选择').trigger('click');await button(wrapper,'取消').trigger('click')
  expect(modelApi.saveAgentSettings).not.toHaveBeenCalled();await button(wrapper,'更换执行模型').trigger('click');expect(button(wrapper,'original / old').exists()).toBe(true);wrapper.unmount()
 })
})

describe('file-backed execution directory refresh',()=>{
 it('reads a new directory after a saved-file snapshot is installed and on every subsequent browse',async()=>{
  const wrapper=render(fileSnapshot());await flushPromises();await browse(wrapper);await flushPromises()
  const selector=wrapper.findComponent(picker)
  expect(selector.props('channels')[0]).toEqual({id:'stable',providerId:'runtime-provider',label:'渠道'})
  await button(wrapper,'取消临时选择').trigger('click')
  // Channel saves install this snapshot through the existing store action.
  useModelsStore().acceptTransfer(fileSnapshot('saved-file','exact/new-id','保存后改名','new-uuid'))
  vi.mocked(modelApi.executionDirectory).mockResolvedValue(execution('exact/new-id','保存后改名','new-uuid'))
  await button(wrapper,'original / old').trigger('click');await flushPromises()
  expect(modelApi.executionDirectory).toHaveBeenCalledTimes(2)
  expect(selector.props('channels')[0]).toEqual({id:'new-uuid',providerId:'runtime-provider',label:'保存后改名'})
  expect(selector.props('rows')[0].modelId).toBe('exact/new-id')
  expect(selector.props('current')).toEqual([{channelId:'old-channel',modelId:'old'}])
  await button(wrapper,'取消临时选择').trigger('click')
  await button(wrapper,'original / old').trigger('click');await flushPromises()
  expect(modelApi.executionDirectory).toHaveBeenCalledTimes(3)
  expect(modelApi.saveAgentSettings).not.toHaveBeenCalled()
  wrapper.unmount()
 })

 it('refreshes an open picker on a file load with the same digest and ignores an older response without touching edits',async()=>{
  const old=deferred<ExecutionDirectory>(),latest=deferred<ExecutionDirectory>()
  vi.mocked(modelApi.executionDirectory).mockReturnValueOnce(old.promise).mockReturnValueOnce(latest.promise)
  const wrapper=render(fileSnapshot());await flushPromises();await browse(wrapper)
  const selector=wrapper.findComponent(picker),store=useModelsStore()
  // The interim file rows cannot become a bound execution choice.
  await button(wrapper,'确认临时选择').trigger('click')
  expect(selector.props('open')).toBe(true)
  store.channelDraft!.channels[0]!.label='未保存输入';store.editCredential('stable','pending-key','')
  const draft=JSON.stringify(store.channelDraft),keys=JSON.stringify(store.credentialEdits)
  vi.mocked(modelApi.load).mockResolvedValue(fileSnapshot('file-1','external/model','外部布局改名','external-uuid'))
  await store.load();await flushPromises()
  expect(modelApi.executionDirectory).toHaveBeenCalledTimes(2)
  old.resolve(execution('old/late','旧目录'));await flushPromises()
  expect(selector.props('loading')).toBe(true)
  expect(selector.props('rows')[0].modelId).toBe('external/model')
  latest.resolve(execution('external/model','外部布局改名','external-uuid'));await flushPromises()
  expect(selector.props('loading')).toBe(false)
  expect(selector.props('channels')[0].id).toBe('external-uuid')
  expect(selector.props('rows')[0].modelId).toBe('external/model')
  expect(JSON.stringify(store.channelDraft)).toBe(draft)
  expect(JSON.stringify(store.credentialEdits)).toBe(keys)
  expect(wrapper.emitted('update:modelValue')?.at(-1)?.[0]).toEqual(saved)
  expect(modelApi.saveAgentSettings).not.toHaveBeenCalled()
  wrapper.unmount()
 })

 it('shows a directory failure even with local rows, retries, and ignores late success and failure from superseded requests',async()=>{
  vi.mocked(modelApi.executionDirectory).mockRejectedValueOnce({message:'读取目录失败'})
  const wrapper=render(fileSnapshot());await flushPromises();await browse(wrapper);await flushPromises()
  const selector=wrapper.findComponent(picker)
  expect(selector.props('error')).toBe('读取目录失败')
  await button(wrapper,'确认临时选择').trigger('click')
  expect(selector.props('open')).toBe(true)
  const lateSuccess=deferred<ExecutionDirectory>(),lateFailure=deferred<ExecutionDirectory>(),latest=deferred<ExecutionDirectory>()
  vi.mocked(modelApi.executionDirectory).mockReturnValueOnce(lateSuccess.promise).mockReturnValueOnce(lateFailure.promise).mockReturnValueOnce(latest.promise)
  for(let i=0;i<3;i++)await button(wrapper,'重新读取目录').trigger('click')
  latest.resolve(execution('latest/model','最新渠道'));await flushPromises()
  expect(selector.props('rows')[0].modelId).toBe('latest/model')
  expect(selector.props('loading')).toBe(false)
  lateSuccess.resolve(execution('stale/model','过期渠道'));lateFailure.reject({message:'过期错误'});await flushPromises()
  expect(selector.props('rows')[0].modelId).toBe('latest/model')
  expect(selector.props('error')).toBe('')
  expect(selector.props('loading')).toBe(false)
  await button(wrapper,'确认临时选择').trigger('click');await button(wrapper,'保存共用设置').trigger('click');await flushPromises()
  expect(vi.mocked(modelApi.saveAgentSettings).mock.calls[0]![0].modelId).toBe('latest/model')
  wrapper.unmount()
 })

 it('keeps a missing saved choice and all channel/Key edits when the actual directory has no match',async()=>{
  vi.mocked(modelApi.executionDirectory).mockResolvedValue({channels:[],rows:[]})
  const wrapper=render(fileSnapshot());await flushPromises()
  const store=useModelsStore();store.channelDraft!.channels[0]!.modelOrder.push('file-only/exact-id');store.editCredential('stable','pending-key','')
  const draft=JSON.stringify(store.channelDraft),keys=JSON.stringify(store.credentialEdits)
  await browse(wrapper);await flushPromises()
  const selector=wrapper.findComponent(picker)
  expect(selector.props('rows')).toEqual([])
  expect(selector.props('current')).toEqual([{channelId:'old-channel',modelId:'old'}])
  expect(wrapper.emitted('update:modelValue')?.at(-1)?.[0]).toEqual(saved)
  expect(modelApi.saveAgentSettings).not.toHaveBeenCalled()
  expect(JSON.stringify(store.channelDraft)).toBe(draft);expect(JSON.stringify(store.credentialEdits)).toBe(keys)
  wrapper.unmount()
 })

 it('disposes pending directory reads and preference loads without later model updates',async()=>{
  const initial=deferred<AgentSettings|null>();vi.mocked(modelApi.agentSettings).mockReturnValueOnce(initial.promise)
  const initialUpdates=vi.fn(),unmounted=render(undefined,initialUpdates);unmounted.unmount();initial.resolve(saved);await flushPromises()
  expect(initialUpdates).not.toHaveBeenCalled()
  const pending=deferred<ExecutionDirectory>();vi.mocked(modelApi.executionDirectory).mockReturnValueOnce(pending.promise)
  const updates=vi.fn(),wrapper=render(undefined,updates);await flushPromises();await browse(wrapper)
  expect(updates).toHaveBeenCalledExactlyOnceWith(saved)
  wrapper.unmount();pending.reject({message:'组件卸载后返回'});await flushPromises()
  expect(updates).toHaveBeenCalledTimes(1)
  expect(modelApi.saveAgentSettings).not.toHaveBeenCalled()
 })
})

