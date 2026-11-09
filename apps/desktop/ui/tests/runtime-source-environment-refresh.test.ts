import {createPinia,setActivePinia} from 'pinia'
import {flushPromises,mount,type VueWrapper} from '@vue/test-utils'
import {defineComponent,h} from 'vue'
import {afterEach,beforeEach,expect,it,vi} from 'vitest'
import {useAppController} from '@/composables/useAppController'
import {useRuntimeStore} from '@/features/runtime/store'
import {useEnvironmentStore} from '@/features/environment/store'
import {useFeedbackStore} from '@/app/feedback/store'
import type {RuntimeSourceDto} from '@/features/runtime/api'
const ipc=vi.hoisted(()=>vi.fn())
const events=vi.hoisted(()=>new Map<string,(event:unknown)=>void>())
vi.mock('@tauri-apps/api/core',()=>({invoke:ipc}))
vi.mock('@tauri-apps/api/event',()=>({listen:vi.fn(async(name:string,cb:(event:unknown)=>void)=>{events.set(name,cb);return ()=>events.delete(name)})}))
vi.mock('@/app/bootstrap',()=>({primeAppData:(app:{refreshEnvironment:()=>Promise<void>;loadRuntimeSource:()=>Promise<boolean>})=>{void app.refreshEnvironment();void app.loadRuntimeSource()}}))
vi.mock('@/features/updates/useUpdateScheduler',()=>({useUpdateScheduler:()=>{}}))
vi.mock('@/features/tray/api',()=>({drainTrayRequests:async()=>[],onTrayRequestsAvailable:async()=>()=>{}}))
vi.mock('@/features/panel/api',()=>({syncEmbeddedPanel:async()=>({visible:false,panelUrl:null})}))
Object.defineProperty(window,'matchMedia',{writable:true,value:()=>({matches:false,addEventListener(){},removeEventListener(){}})})
const Root=defineComponent({setup(){useAppController(true);return ()=>h('div')}})
const Page=defineComponent({setup(){useAppController(false);return ()=>h('div')}})
let source:RuntimeSourceDto,wrapper:VueWrapper|undefined
function sourceDto(kind:RuntimeSourceDto['kind']='managed'):RuntimeSourceDto{return {kind,path:kind==='managed'?'/I/runtime/bin/ocx':null,version:kind==='managed'?'2.97.0':null,resolvedAt:'first',insideDataRoot:false,managedEntry:kind==='managed'?'/I/runtime/bin/ocx':null,managedPrefix:'/I/runtime',defaultPrefix:'/I/runtime',explicitPath:null,history:[]}}
const report=(ready:boolean)=>({node:{found:ready,path:ready?'/I/runtime/bin/node':null,version:null},npm:{found:ready,path:ready?'/I/runtime/bin/npm':null,version:null},ocx:{found:ready,path:ready?'/I/runtime/bin/ocx':null,version:null},gate:ready?'ready':'missing_node',shortCircuited:!ready,brewFound:false})
const discoveries=()=>ipc.mock.calls.filter(c=>c[0]==='discover_environment')
beforeEach(()=>{setActivePinia(createPinia());events.clear();ipc.mockReset();vi.useFakeTimers();source=sourceDto('unresolved');ipc.mockImplementation(async command=>command==='runtime_source'?structuredClone(source):command==='discover_environment'?report(source.kind==='managed'):undefined)})
afterEach(()=>{wrapper?.unmount();wrapper=undefined;vi.useRealTimers()})
it('actual source event -> loadSource -> root orchestration refreshes an initial MissingNode report',async()=>{
 wrapper=mount(Root);await flushPromises();expect(useEnvironmentStore().report?.gate).toBe('missing_node');const count=discoveries().length
 source=sourceDto();events.get('runtime-source-changed')!({payload:null});await flushPromises()
 expect(ipc).toHaveBeenCalledWith('runtime_source',undefined);expect(discoveries()).toHaveLength(count+1);expect(discoveries().at(-1)).toEqual(['discover_environment',{request:null}]);expect(useEnvironmentStore().report).toEqual(report(true))
 expect(useRuntimeStore().source?.path).toBe('/I/runtime/bin/ocx')
})
it('stable source rereads, timestamps and history do not loop; identity changes do refresh',async()=>{
 wrapper=mount(Root);await flushPromises();source=sourceDto();events.get('runtime-source-changed')!({});await flushPromises();const count=discoveries().length
 source={...source,resolvedAt:'later',history:[{action:'read',result:'ok',package:'ocx',version:'2.97.0',target:'/I/runtime',at:'later'}]};events.get('runtime-source-changed')!({});await flushPromises();await vi.advanceTimersByTimeAsync(120000);expect(discoveries()).toHaveLength(count)
 source={...source,path:'/I/runtime/next/ocx',version:'2.98.0'};events.get('runtime-source-changed')!({});await flushPromises();expect(discoveries()).toHaveLength(count+1)
})
it('only initialized root watches and it stops on unmount',async()=>{
 wrapper=mount(Page);useRuntimeStore().source=sourceDto();await flushPromises();expect(discoveries()).toHaveLength(0);wrapper.unmount()
 wrapper=mount(Root);await flushPromises();const count=discoveries().length;wrapper.unmount();useRuntimeStore().source={...sourceDto(),version:'3.0.0'};await flushPromises();expect(discoveries()).toHaveLength(count);expect(events.has('runtime-source-changed')).toBe(false)
})
it('managed discovery error invalidates old report and shows a factual root hint, never a guessed ready result',async()=>{
 wrapper=mount(Root);await flushPromises();const original=ipc.getMockImplementation()!;ipc.mockImplementation(async(command,args)=>{if(command==='discover_environment')throw {code:'discovery_node_probe_failed'};return original(command,args)})
 source=sourceDto();events.get('runtime-source-changed')!({});await flushPromises();expect(useEnvironmentStore().report).toBeNull();expect(useEnvironmentStore().loading).toBe(false);expect(useEnvironmentStore().error).toBe('集成运行程序检查未就绪，可稍后重新检查或查看安装状态。');expect(useFeedbackStore().toast).toBe(useEnvironmentStore().error)
})
it('initial slow MissingNode completion cannot replace event-driven integrated-ready paths',async()=>{
 let finish!:(value:unknown)=>void
 const original=ipc.getMockImplementation()!;let count=0
 ipc.mockImplementation(async(command,args)=>{if(command==='discover_environment'&&count++===0)return new Promise(resolve=>{finish=resolve});return original(command,args)})
 wrapper=mount(Root);await flushPromises();source=sourceDto();events.get('runtime-source-changed')!({});await flushPromises();expect(useEnvironmentStore().report?.gate).toBe('ready')
 finish(report(false));await flushPromises();expect(useEnvironmentStore().report).toEqual(report(true));expect(useEnvironmentStore().error).toBe('')
})
