import {beforeEach,afterEach,describe,it,expect,vi} from 'vitest'
import {mount,flushPromises} from '@vue/test-utils'
import {reactive} from 'vue'
import ChannelNodeSelect from '@/features/models/ChannelNodeSelect.vue'
import {emptyRules,modelApi,type ChannelPolicy,type NodeMeasurementSet} from '@/features/models/api'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{measureNodes:vi.fn(),stopNodeMeasurement:vi.fn(async()=>{})}}))
function channel():ChannelPolicy{return{id:'channel',label:'节点测试渠道',templateRefs:[],modelRefs:{},connection:{providerId:'provider',protocol:'openai-responses',nodes:[{id:'a',name:'当前',url:'https://a.example.test'},{id:'b',name:'备用',url:'https://b.example.test'}],activeNodeId:'a',credentialRef:'same-local-ref',enabled:true,allowPrivateNetwork:false},selection:{mode:'all'},modelOrder:['m'],discovered:[],defaults:emptyRules(),overrides:{},familyOverrides:{}}}
beforeEach(()=>vi.clearAllMocks());afterEach(()=>{document.body.innerHTML=''})
describe('integrated channel node selector',()=>{
 it('measures without changing selection or closing the portal, then manually chooses a node and returns focus',async()=>{
  const value=reactive(channel()),wrapper=mount(ChannelNodeSelect,{props:{channel:value},attachTo:document.body});let finish!:(r:NodeMeasurementSet)=>void
  vi.mocked(modelApi.measureNodes).mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve}))
  await wrapper.get('button[aria-label="节点测试渠道当前节点"]').trigger('click');await flushPromises()
  const measure=document.querySelector('button[aria-label="检测节点 备用"]') as HTMLButtonElement
  const trigger=wrapper.get('button.select-trigger').element as HTMLButtonElement
  trigger.dispatchEvent(new FocusEvent('focusout',{bubbles:true,relatedTarget:measure}));await flushPromises()
  expect(document.querySelector('[role=menu]')).not.toBeNull();measure.click();await flushPromises()
  expect(value.connection.activeNodeId).toBe('a');expect(document.querySelector('[role=menu]')).not.toBeNull()
  const request=vi.mocked(modelApi.measureNodes).mock.calls[0]![0];expect(request.credentialRef).toBe('same-local-ref');expect(request.nodes.map(n=>n.id)).toEqual(['b'])
  finish({runId:request.runId,stopped:false,results:[{nodeId:'b',elapsedMs:42,modelCount:3,error:null}]});await flushPromises()
  expect(document.querySelector('[role=menu]')!.textContent).toContain('42 ms');expect(value.connection.activeNodeId).toBe('a')
  ;(document.querySelector('button[aria-label="选择节点 备用"]') as HTMLButtonElement).click();await flushPromises()
  expect(value.connection.activeNodeId).toBe('b');expect(value.connection.credentialRef).toBe('same-local-ref');expect(value.modelOrder).toEqual(['m']);expect(document.querySelector('[role=menu]')).toBeNull();expect(document.activeElement).toBe(wrapper.get('button.select-trigger').element);wrapper.unmount()
 })
 it('keeps a late measurement invalid after URL changes and honors readonly controls',async()=>{
  const value=reactive(channel()),wrapper=mount(ChannelNodeSelect,{props:{channel:value,readonly:true},attachTo:document.body});let finish!:(r:NodeMeasurementSet)=>void
  vi.mocked(modelApi.measureNodes).mockImplementationOnce(()=>new Promise(resolve=>{finish=resolve}))
  await wrapper.get('button.select-trigger').trigger('click');await flushPromises();expect((document.querySelector('button[aria-label="选择节点 备用"]') as HTMLButtonElement).disabled).toBe(true);expect((document.querySelector('button[aria-label="检测节点 备用"]') as HTMLButtonElement).disabled).toBe(true);await wrapper.setProps({readonly:false})
  ;(document.querySelector('button[aria-label="检测节点 备用"]') as HTMLButtonElement).click();await flushPromises();const request=vi.mocked(modelApi.measureNodes).mock.calls[0]![0]
  value.connection.nodes[1]!.url='https://changed.example.test';await flushPromises();expect(modelApi.stopNodeMeasurement).toHaveBeenCalledWith(request.runId)
  finish({runId:request.runId,stopped:false,results:[{nodeId:'b',elapsedMs:42,modelCount:3,error:null}]});await flushPromises();expect(document.querySelector('[role=menu]')!.textContent).not.toContain('42 ms');expect(value.connection.activeNodeId).toBe('a');wrapper.unmount()
 })
 it('keeps independent detection buttons out of selection and closes with Escape while returning focus',async()=>{
  const wrapper=mount(ChannelNodeSelect,{props:{channel:channel()},attachTo:document.body});await wrapper.get('button.select-trigger').trigger('click');await flushPromises()
  const target=document.querySelector('button[aria-label="选择节点 备用"]') as HTMLButtonElement;target.focus();target.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true,cancelable:true}));await flushPromises()
  expect(document.querySelector('[role=menu]')).toBeNull();expect(document.activeElement).toBe(wrapper.get('button.select-trigger').element);wrapper.unmount()
 })
})
