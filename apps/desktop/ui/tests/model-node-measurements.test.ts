import { afterEach,beforeEach,describe,expect,it,vi } from 'vitest'
import { effectScope,nextTick,reactive } from 'vue'
import { useNodeMeasurements } from '@/features/models/useNodeMeasurements'
import {modelV2Api as modelApi,type NodeMeasurementSetV2 as NodeMeasurementSet,type ReadBaseV2} from '@/features/models/api-v2'
import {workspaceChannel,workspaceId,workspaceHash} from './helpers/modelWorkspaceV2'
vi.mock('@tauri-apps/api/core',()=>({invoke:vi.fn()}))
const expected:ReadBaseV2={dtoVersion:2,draftId:workspaceId('draft'),draftGeneration:1,sourceDigest:workspaceHash(1),libraryDigest:workspaceHash(2),policyDigest:workspaceHash(3),rootRevision:workspaceHash(4)}
function channel(){const c=workspaceChannel('channel');c.connection.enabled=false;return c}
beforeEach(()=>{vi.clearAllMocks();vi.spyOn(modelApi,'measureNodes');vi.spyOn(modelApi,'stopMeasurement').mockResolvedValue()})
afterEach(()=>vi.restoreAllMocks())
describe('node measurements remain tied to their connection inputs',()=>{
 it('returns measurements without switching nodes, discovering models or enabling a channel',async()=>{
  const value=reactive(channel());const before=JSON.stringify(value);const scope=effectScope();const state=scope.run(()=>useNodeMeasurements(()=>value,()=>'',{expected:()=>expected,discoveryKind:()=> 'openai_list'}))!
  vi.mocked(modelApi.measureNodes).mockImplementation(async request=>({runId:request.runId,stopped:false,results:request.nodeIds.map(id=>({nodeId:id,elapsedMs:34,modelCount:7,error:null}))}))
  await state.measure([workspaceId('channelbackup')]);expect(modelApi.measureNodes).toHaveBeenCalledWith({expected,channelId:workspaceId('channel'),nodeIds:[workspaceId('channelbackup')],discoveryKind:'openai_list',runId:expect.any(String)})
  expect(state.results.value[workspaceId('channelbackup')]?.modelCount).toBe(7);expect(JSON.stringify(value)).toBe(before);scope.stop()
 })
 it('cancels the pending run and ignores late results after a URL or credential change',async()=>{
  const value=reactive(channel());const scope=effectScope();const state=scope.run(()=>useNodeMeasurements(()=>value,()=>'',{expected:()=>expected,discoveryKind:()=> 'openai_list'}))!
  let finish!:(value:NodeMeasurementSet)=>void
  vi.mocked(modelApi.measureNodes).mockReturnValue(new Promise(resolve=>{finish=resolve}))
  const waiting=state.measure([workspaceId('channelnode')]);const originalRun=state.run.value
  value.connection.nodes[0]!.url='https://changed.example.test';await nextTick()
  expect(modelApi.stopMeasurement).toHaveBeenCalledWith(originalRun)
  finish({runId:originalRun,stopped:false,results:[{nodeId:workspaceId('channelnode'),elapsedMs:1,modelCount:7,error:null}]});await waiting
  expect(state.results.value).toEqual({});expect(state.run.value).toBe('');scope.stop()
 })
 it('invalidates a request when the running provider changes, preserving node and model identity',async()=>{
  const value=reactive(channel()),scope=effectScope(),state=scope.run(()=>useNodeMeasurements(()=>value,()=>'',{expected:()=>expected,discoveryKind:()=> 'openai_list'}))!
  let finish!:(value:NodeMeasurementSet)=>void
  vi.mocked(modelApi.measureNodes).mockReturnValueOnce(new Promise(resolve=>{finish=resolve}))
  const waiting=state.measure([workspaceId('channelnode')]),id=state.run.value
  value.connection.providerId='renamed-provider';await nextTick()
  expect(modelApi.stopMeasurement).toHaveBeenCalledWith(id)
  finish({runId:id,stopped:false,results:[{nodeId:workspaceId('channelnode'),elapsedMs:4,modelCount:1,error:null}]});await waiting
  expect(state.results.value).toEqual({});expect(value.connection.activeNodeId).toBe(workspaceId('channelnode'));scope.stop()
 })
 it('ignores a late stop error after invalidation and a new request',async()=>{
  const value=reactive(channel()),scope=effectScope(),state=scope.run(()=>useNodeMeasurements(()=>value,()=>'',{expected:()=>expected,discoveryKind:()=> 'openai_list'}))!
  let rejectStop!:(cause:Error)=>void
  vi.mocked(modelApi.measureNodes).mockImplementation(()=>new Promise(()=>{}))
  vi.mocked(modelApi.stopMeasurement).mockImplementationOnce(()=>new Promise((_resolve,reject)=>{rejectStop=reject}))
  void state.measure([workspaceId('channelnode')]);const stopping=state.stop()
  value.connection.nodes[0]!.url='https://changed.example.test';await nextTick();void state.measure([workspaceId('channelbackup')])
  const current=state.run.value;rejectStop(new Error('旧停止请求失败'));await stopping
  expect(state.run.value).toBe(current);expect(state.error.value).toBe('');scope.stop()
 })
})
