import { beforeEach,describe,expect,it,vi } from 'vitest'
import { effectScope,nextTick,reactive } from 'vue'
import { useNodeMeasurements } from '@/features/models/useNodeMeasurements'
import { emptyRules,modelApi,type ChannelPolicy,type NodeMeasurementSet } from '@/features/models/api'
vi.mock('@/features/models/api',async original=>({...await original<typeof import('@/features/models/api')>(),modelApi:{measureNodes:vi.fn(),stopNodeMeasurement:vi.fn().mockResolvedValue(undefined)}}))
function channel():ChannelPolicy{return {id:'channel',label:'测试',connection:{providerId:'provider',protocol:'openai-chat',nodes:[{id:'one',name:'主节点',url:'https://one.example.test'},{id:'two',name:'备用节点',url:'https://two.example.test'}],activeNodeId:'one',credentialRef:null,enabled:false,allowPrivateNetwork:false},selection:{mode:'none'},templateRefs:[],modelRefs:{},modelOrder:[],discovered:[],defaults:emptyRules(),overrides:{},familyOverrides:{}}}
beforeEach(()=>vi.clearAllMocks())
describe('node measurements remain tied to their connection inputs',()=>{
 it('returns measurements without switching nodes, discovering models or enabling a channel',async()=>{
  const value=reactive(channel());const before=JSON.stringify(value);const scope=effectScope();const state=scope.run(()=>useNodeMeasurements(()=>value,()=>''))!
  vi.mocked(modelApi.measureNodes).mockImplementation(async request=>({runId:request.runId,stopped:false,results:request.nodes.map(n=>({nodeId:n.id,elapsedMs:34,modelCount:7,error:null}))}))
  await state.measure(['two']);expect(modelApi.measureNodes).toHaveBeenCalledWith(expect.objectContaining({nodes:[{id:'two',name:'备用节点',url:'https://two.example.test'}],apiKey:null}))
  expect(state.results.value.two?.modelCount).toBe(7);expect(JSON.stringify(value)).toBe(before);scope.stop()
 })
 it('cancels the pending run and ignores late results after a URL or credential change',async()=>{
  const value=reactive(channel());const scope=effectScope();const state=scope.run(()=>useNodeMeasurements(()=>value,()=>''))!
  let finish!:(value:NodeMeasurementSet)=>void
  vi.mocked(modelApi.measureNodes).mockReturnValue(new Promise(resolve=>{finish=resolve}))
  const waiting=state.measure(['one']);const originalRun=state.run.value
  value.connection.nodes[0]!.url='https://changed.example.test';await nextTick()
  expect(modelApi.stopNodeMeasurement).toHaveBeenCalledWith(originalRun)
  finish({runId:originalRun,stopped:false,results:[{nodeId:'one',elapsedMs:1,modelCount:7,error:null}]});await waiting
  expect(state.results.value).toEqual({});expect(state.run.value).toBe('');scope.stop()
 })
 it('invalidates a request when the running provider changes, preserving node and model identity',async()=>{
  const value=reactive(channel()),scope=effectScope(),state=scope.run(()=>useNodeMeasurements(()=>value,()=>''))!
  let finish!:(value:NodeMeasurementSet)=>void
  vi.mocked(modelApi.measureNodes).mockReturnValueOnce(new Promise(resolve=>{finish=resolve}))
  const waiting=state.measure(['one']),id=state.run.value
  value.connection.providerId='renamed-provider';await nextTick()
  expect(modelApi.stopNodeMeasurement).toHaveBeenCalledWith(id)
  finish({runId:id,stopped:false,results:[{nodeId:'one',elapsedMs:4,modelCount:1,error:null}]});await waiting
  expect(state.results.value).toEqual({});expect(value.connection.activeNodeId).toBe('one');scope.stop()
 })
 it('ignores a late stop error after invalidation and a new request',async()=>{
  const value=reactive(channel()),scope=effectScope(),state=scope.run(()=>useNodeMeasurements(()=>value,()=>''))!
  let rejectStop!:(cause:Error)=>void
  vi.mocked(modelApi.measureNodes).mockImplementation(()=>new Promise(()=>{}))
  vi.mocked(modelApi.stopNodeMeasurement).mockImplementationOnce(()=>new Promise((_resolve,reject)=>{rejectStop=reject}))
  void state.measure(['one']);const stopping=state.stop()
  value.connection.nodes[0]!.url='https://changed.example.test';await nextTick();void state.measure(['two'])
  const current=state.run.value;rejectStop(new Error('旧停止请求失败'));await stopping
  expect(state.run.value).toBe(current);expect(state.error.value).toBe('');scope.stop()
 })
})
