import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ChannelsView from '@/features/models/ChannelsView.vue'
import { useModelsStore } from '@/features/models/store'
import { modelApi, type ModelSnapshot } from '@/features/models/api'
import {useModalStore} from '@/app/modal/store'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{latestPolicy:vi.fn(),policyResult:vi.fn(),applyPolicy:vi.fn(),stage:vi.fn(),load:vi.fn()}}))
const dialog={props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'}
const plan=(id='original')=>({id,phase:'visibility_written',catalogCount:3,currentCatalogCount:2,changedChannels:['测试渠道'],error:'order_write'})
function render(){return mount(ChannelsView,{global:{stubs:{UiDialog:dialog,ClientLoadStatus:true,ChannelEditor:true,ModelPicker:true,UiSelectField:true}}})}
beforeEach(()=>{setActivePinia(createPinia());vi.clearAllMocks();const store=useModelsStore();store.snapshot={library:{schemaVersion:1,templates:[]},policy:{schemaVersion:1,channels:[],templateRefs:[]},libraryDigest:'library',policyDigest:'policy',workspaceDigest:'workspace',workspace:null} as ModelSnapshot})
describe('recovering a saved operation',()=>{
 it('selects only editable channels and deletion waits for an exact-current-draft confirmation',async()=>{
   const store=useModelsStore();const make=(id:string,providerId:string)=>({id,label:id,templateRefs:[],modelRefs:{},connection:{providerId,protocol:'openai-chat',nodes:[],activeNodeId:'',credentialRef:null,enabled:true,allowPrivateNetwork:false},selection:{mode:'none' as const},modelOrder:[],discovered:[],defaults:{protocol:{mode:'inherit' as const},contextWindow:{mode:'inherit' as const},inputModalities:{mode:'inherit' as const},reasoningEfforts:{mode:'inherit' as const},defaultEffort:{mode:'inherit' as const},fast:{mode:'inherit' as const}},overrides:{},familyOverrides:{}})
   const custom=make('custom','custom');const native=make('native','openai');store.snapshot!.policy.channels=[custom,native];store.restoreChannels()
   vi.mocked(modelApi.latestPolicy).mockResolvedValue(null)
   const wrapper=render();await flushPromises();await wrapper.get('input[aria-label="选择当前结果中的可编辑渠道"]').setValue(true)
   expect(wrapper.text()).toContain('已选 1 个渠道');expect(wrapper.get('input[aria-label="选择渠道 native"]').attributes('disabled')).toBeDefined()
   await wrapper.findAll('button').find(b=>b.text()==='删除所选渠道')!.trigger('click');expect(store.channelDraft!.channels.length).toBe(2)
   store.channelDraft!.channels[0]!.label='later';useModalStore().resolve('confirm');await flushPromises();expect(store.channelDraft!.channels.length).toBe(2);expect(wrapper.text()).toContain('删除确认已过期')
   await wrapper.findAll('button').find(b=>b.text()==='删除所选渠道')!.trigger('click');useModalStore().resolve('confirm');await flushPromises();expect(store.channelDraft!.channels.map(c=>c.id)).toEqual(['native']);expect(store.snapshot!.policy.channels.length).toBe(2);wrapper.unmount()
 })
 it('querying a committed original preserves a later persisted workspace and its next write baseline',async()=>{
   const store=useModelsStore();const saved={...store.snapshot!,workspaceDigest:'later-draft',workspace:{basePolicy:'policy',policy:{schemaVersion:1,templateRefs:[],channels:[]}}}
   vi.mocked(modelApi.latestPolicy).mockResolvedValue(plan())
   vi.mocked(modelApi.policyResult).mockResolvedValue({...plan(),phase:'committed'})
   vi.mocked(modelApi.load).mockResolvedValue(saved)
   vi.mocked(modelApi.stage).mockImplementation(async(_policy,workspace,policy)=>{expect(workspace).toBe('later-draft');return {...saved,workspace:{basePolicy:'policy',policy}}})
   const wrapper=render();await flushPromises();await wrapper.findAll('button').find(b=>b.text()==='查看原保存结果')!.trigger('click');await wrapper.findAll('button').find(b=>b.text()==='核对原计划')!.trigger('click');await flushPromises()
   expect(store.snapshot!.workspaceDigest).toBe('later-draft');expect(store.channelDraft).toEqual(saved.workspace.policy)
   await store.persistDraft();expect(modelApi.stage).toHaveBeenCalledWith('policy','later-draft',saved.workspace.policy);wrapper.unmount()
 })
 it('reopens the original plan after route remount without resubmitting writes',async()=>{
   vi.mocked(modelApi.latestPolicy).mockResolvedValue(plan())
   vi.mocked(modelApi.policyResult).mockResolvedValue(plan())
   const wrapper=render();await flushPromises()
   expect(wrapper.text()).toContain('上次保存尚未完成核对')
   await wrapper.findAll('button').find(b=>b.text()==='查看原保存结果')!.trigger('click')
   await wrapper.findAll('button').find(b=>b.text()==='核对原计划')!.trigger('click');await flushPromises()
   expect(modelApi.policyResult).toHaveBeenCalledExactlyOnceWith('original');expect(modelApi.applyPolicy).not.toHaveBeenCalled();wrapper.unmount()
 })
 it('notification navigation keeps its exact plan even when latest returns another operation',async()=>{
   const store=useModelsStore();store.policyNotificationTarget={planId:'specific-plan'}
   vi.mocked(modelApi.latestPolicy).mockResolvedValue(plan('newer-plan'))
   vi.mocked(modelApi.policyResult).mockResolvedValue(plan('specific-plan'))
   const wrapper=render();await flushPromises()
   expect(modelApi.policyResult).toHaveBeenCalledExactlyOnceWith('specific-plan')
   expect(wrapper.findAll('button').some(b=>b.text()==='核对原计划')).toBe(true)
   await wrapper.findAll('button').find(b=>b.text()==='核对原计划')!.trigger('click');await flushPromises()
   expect(modelApi.policyResult).toHaveBeenLastCalledWith('specific-plan');expect(modelApi.applyPolicy).not.toHaveBeenCalled();wrapper.unmount()
 })
})
