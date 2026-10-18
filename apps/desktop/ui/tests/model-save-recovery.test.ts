import { beforeEach, describe, expect, it, vi } from 'vitest'
import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import ChannelsView from '@/features/models/ChannelsView.vue'
import { useModelsStore } from '@/features/models/store'
import { modelApi, type ModelSnapshot } from '@/features/models/api'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{latestPolicy:vi.fn(),policyResult:vi.fn(),applyPolicy:vi.fn(),stage:vi.fn()}}))
const dialog={props:['open'],template:'<section v-if="open"><slot/><slot name="actions"/></section>'}
const plan=(id='original')=>({id,phase:'visibility_written',catalogCount:3,currentCatalogCount:2,changedChannels:['测试渠道'],error:'order_write'})
function render(){return mount(ChannelsView,{global:{stubs:{UiDialog:dialog,ChannelEditor:true,ModelPicker:true,UiSelectField:true}}})}
beforeEach(()=>{setActivePinia(createPinia());vi.clearAllMocks();const store=useModelsStore();store.snapshot={library:{schemaVersion:1,templates:[]},policy:{schemaVersion:1,channels:[],templateRefs:[]},libraryDigest:'library',policyDigest:'policy',workspaceDigest:'workspace',workspace:null} as ModelSnapshot})
describe('recovering a saved operation',()=>{
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
