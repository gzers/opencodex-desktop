import {afterEach,beforeEach,describe,expect,it,vi} from 'vitest'
import {flushPromises,mount} from '@vue/test-utils'
import {createPinia,setActivePinia} from 'pinia'
import ChannelsView from '@/features/models/ChannelsView.vue'
import ChannelEditor from '@/features/models/ChannelEditor.vue'
import {useModelsStore} from '@/features/models/store'
import {modelApi,emptyRules,type ChannelPolicy,type DraftProjection,type ModelSnapshot} from '@/features/models/api'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{latestPolicy:vi.fn(),project:vi.fn(),stage:vi.fn()}}))
let pinia:ReturnType<typeof createPinia>
beforeEach(()=>{pinia=createPinia();setActivePinia(pinia);vi.clearAllMocks();vi.useFakeTimers();vi.mocked(modelApi.latestPolicy).mockResolvedValue(null)})
afterEach(()=>{pinia._s.forEach(store=>store.$dispose());vi.useRealTimers()})
const make=(id:string)=>({id,label:id,templateRefs:[],modelRefs:{},connection:{providerId:id,protocol:'openai-chat',nodes:[],activeNodeId:'',credentialRef:null,enabled:false,allowPrivateNetwork:false},selection:{mode:'none'},modelOrder:[],discovered:[],defaults:emptyRules(),overrides:{},familyOverrides:{}} as ChannelPolicy)
describe('channel projection owner',()=>{
 it('ignores the previous channel request after switching targets and never mutates the draft',async()=>{
  const store=useModelsStore();store.acceptTransfer({policy:{schemaVersion:1,templateRefs:[],channels:[make('first'),make('second')]},workspace:null,policyDigest:'p',workspaceDigest:'w',libraryDigest:'l',library:{schemaVersion:1,templates:[]}} as ModelSnapshot)
  const frozen=JSON.stringify(store.channelDraft)
  let finishFirst!:(value:DraftProjection)=>void
  vi.mocked(modelApi.project).mockReturnValueOnce(new Promise(resolve=>finishFirst=resolve)).mockResolvedValueOnce({rows:[],knownActiveModels:0,exactCatalogCount:null,notices:[]})
  const wrapper=mount(ChannelsView,{global:{stubs:{ChannelEditor:true,ClientLoadStatus:true,NewChannelDialog:true,ModelPicker:true,UiSelectField:true,UiDialog:true}}});await flushPromises()
  await wrapper.findAll('button').find(b=>b.text()==='编辑')!.trigger('click');await vi.advanceTimersByTimeAsync(210)
  wrapper.findComponent(ChannelEditor).vm.$emit('close');await wrapper.vm.$nextTick();await wrapper.findAll('button').filter(b=>b.text()==='编辑')[1]!.trigger('click');await vi.advanceTimersByTimeAsync(210);await flushPromises()
  finishFirst({rows:[{channelId:'first'}] as never,knownActiveModels:0,exactCatalogCount:null,notices:[]});await flushPromises()
  expect(wrapper.findComponent(ChannelEditor).props('channel').id).toBe('second');expect(wrapper.findComponent(ChannelEditor).props('projection')).toEqual([]);expect(wrapper.findComponent(ChannelEditor).props('projectionBusy')).toBe(false);expect(JSON.stringify(store.channelDraft)).toBe(frozen);wrapper.unmount()
 })
})
