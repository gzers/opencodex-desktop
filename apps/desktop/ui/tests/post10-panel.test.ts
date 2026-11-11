import {afterEach,beforeEach,expect,it,vi} from 'vitest'
import {createPinia,setActivePinia} from 'pinia'
import {flushPromises,mount,type VueWrapper} from '@vue/test-utils'
import PanelRoute from '@/routes/PanelRoute.vue'
import {useLifecycleStore} from '@/app/lifecycle/store'
import {usePreferencesStore} from '@/features/preferences/store'
import {usePanelStore} from '@/features/panel/store'
import {useNotificationsStore} from '@/features/notifications/store'
import {PANEL_LOAD_NOTICE_MS} from '@/config/runtimeDefaults'
import type {PreferencesDto} from '@/features/preferences/api'
import type {StatusSnapshot} from '@/contracts/runtimeStatus'
const invoke=vi.fn()
vi.mock('@tauri-apps/api/core',()=>({invoke:(...args:unknown[])=>invoke(...args)}))
let wrapper:VueWrapper|undefined
const snapshot:StatusSnapshot={matrix:{runtime:'running',connection:'unconfigured',operation:'idle'},facts:{client_recovery:'not_required',health:'healthy',runtime_label:null,opencodex_home:'/isolated/official',data_root:'/isolated',protection:null,reboot_safe:null,service_present:null,shim_present:null,version_drift:null},port:10100,pid:'123',can_start:false,can_stop:true,can_restart:true,source:'fixture'}
const prefs={interfaceScale:100,panelMode:'embedded',theme:'dark',visualEffects:'mid',glowRender:'mesh',themeNeedsImport:false} as PreferencesDto
beforeEach(()=>{
 vi.useFakeTimers();setActivePinia(createPinia());invoke.mockReset();useLifecycleStore().statusSnapshot=snapshot
 usePreferencesStore().data={...prefs}
 vi.spyOn(HTMLElement.prototype,'getBoundingClientRect').mockReturnValue({x:64,y:0,left:64,top:0,right:1180,bottom:760,width:1116,height:760,toJSON:()=>({})} as DOMRect)
 invoke.mockImplementation((command:string,args?:{request?:{action?:string}})=>{
  if(command==='get_preferences')return Promise.resolve({...prefs})
  if(command==='sync_embedded_panel')return Promise.resolve({visible:args?.request?.action!=='hide',panelUrl:'http://127.0.0.1:10100/web',ready:true,generation:1})
  return Promise.resolve(undefined)
 })
})
afterEach(()=>{wrapper?.unmount();wrapper=undefined;vi.restoreAllMocks();vi.useRealTimers();document.body.innerHTML=''})
async function render(){wrapper=mount(PanelRoute,{attachTo:document.body});await flushPromises();return wrapper}

it('times out a pending native show, exposes retry and ignores its late visible result',async()=>{
 let resolveShow!:(value:unknown)=>void
 invoke.mockImplementation((command:string,args?:{request?:{action?:string}})=>command==='sync_embedded_panel'&&args?.request?.action==='show'?new Promise(resolve=>{resolveShow=resolve}):Promise.resolve({visible:false,panelUrl:null}))
 await render();expect(resolveShow).toBeTypeOf('function')
 await vi.advanceTimersByTimeAsync(PANEL_LOAD_NOTICE_MS+1);await flushPromises()
 expect(wrapper!.text()).toContain('官方面板加载失败')
 expect(usePanelStore().loadTrace.outcome).toBe('timeout')
 resolveShow({visible:true,panelUrl:'http://127.0.0.1:10100/web',ready:true,generation:1});await flushPromises()
 window.dispatchEvent(new CustomEvent('ocxd-panel',{detail:{kind:'load',value:'ready',generation:1}}));await flushPromises()
 expect(wrapper!.text()).toContain('官方面板加载失败')
 invoke.mockResolvedValue({visible:true,panelUrl:'http://127.0.0.1:10100/web',ready:true,generation:3})
 await wrapper!.findAll('button').find(b=>b.text()==='重试加载')!.trigger('click');await flushPromises()
 expect(wrapper!.text()).toContain('官方面板已嵌入主窗口')
 expect(invoke.mock.calls.some(([cmd])=>cmd==='process_action')).toBe(false)
})

it('bounds the preference wait before any native show is issued',async()=>{
 usePreferencesStore().data=null
 invoke.mockImplementation(command=>command==='get_preferences'?new Promise(()=>{}):Promise.resolve({visible:false,panelUrl:null}))
 await render();await vi.advanceTimersByTimeAsync(PANEL_LOAD_NOTICE_MS+1);await flushPromises()
 expect(wrapper!.text()).toContain('官方面板加载失败')
 expect(invoke.mock.calls.some(([cmd,args])=>cmd==='sync_embedded_panel'&&args.request.action==='show')).toBe(false)
 expect(usePanelStore().loadTrace.outcome).toBe('timeout')
})

it('uses already loaded persisted preferences and records ready stages on native reuse',async()=>{
 await render()
 expect(invoke.mock.calls.some(([cmd])=>cmd==='get_preferences')).toBe(false)
 expect(wrapper!.text()).toContain('官方面板已嵌入主窗口')
 expect(usePanelStore().loadTrace.phases.map(p=>p.phase)).toEqual(expect.arrayContaining(['preferences-ready','status-ready','address-ready','native-request','visible-ready']))
})

it('records a terminal stopped-service outcome and clears readiness from an earlier visit',async()=>{
 usePanelStore().loaded=true
 useLifecycleStore().statusSnapshot={...snapshot,matrix:{...snapshot.matrix,runtime:'stopped'},port:null}
 await render()
 expect(wrapper!.text()).toContain('OpenCodex 未运行')
 expect(usePanelStore().loaded).toBe(false)
 expect(usePanelStore().loadTrace.outcome).toBe('not-running')
 expect(invoke.mock.calls.some(([cmd,args])=>cmd==='sync_embedded_panel'&&args.request.action==='show')).toBe(false)
})

it('keeps navigation ready when Show returns an older snapshot for the same generation',async()=>{
 let resolveShow!:(value:unknown)=>void
 invoke.mockImplementation((command:string,args?:{request?:{action?:string}})=>command==='sync_embedded_panel'&&args?.request?.action==='show'?new Promise(resolve=>{resolveShow=resolve}):Promise.resolve({visible:false,panelUrl:null}))
 await render()
 window.dispatchEvent(new CustomEvent('ocxd-panel',{detail:{kind:'load',value:'ready',generation:5}}));await flushPromises()
 resolveShow({visible:true,panelUrl:'http://127.0.0.1:10100/web',ready:false,generation:5});await flushPromises()
 await vi.advanceTimersByTimeAsync(PANEL_LOAD_NOTICE_MS+1)
 expect(wrapper!.text()).toContain('官方面板已嵌入主窗口')
 expect(usePanelStore().loadTrace.outcome).toBe('visible-ready')
})

it('hides a late show while an overlay is present and restores after the last surface closes',async()=>{
 let resolveShow!:(value:unknown)=>void
 invoke.mockImplementation((command:string,args?:{request?:{action?:string}})=>command==='sync_embedded_panel'&&args?.request?.action==='show'?new Promise(resolve=>{resolveShow=resolve}):Promise.resolve({visible:false,panelUrl:null}))
 await render();useNotificationsStore().panelOpen=true;await flushPromises()
 resolveShow({visible:true,panelUrl:'http://127.0.0.1:10100/web',ready:true,generation:1});await flushPromises()
 expect(invoke).toHaveBeenCalledWith('sync_embedded_panel',expect.objectContaining({request:expect.objectContaining({action:'hide'})}))
 invoke.mockResolvedValue({visible:true,panelUrl:'http://127.0.0.1:10100/web',ready:true,generation:3})
 useNotificationsStore().panelOpen=false;await flushPromises()
 expect(wrapper!.text()).toContain('官方面板已嵌入主窗口')
})

it('restores the panel when the last overlay closes before native Hide returns',async()=>{
 let resolveHide!:(value:unknown)=>void
 let shows=0
 invoke.mockImplementation((command:string,args?:{request?:{action?:string}})=>{
  if(command!=='sync_embedded_panel')return Promise.resolve(undefined)
  if(args?.request?.action==='hide')return new Promise(resolve=>{resolveHide=resolve})
  shows++;return Promise.resolve({visible:true,panelUrl:'http://127.0.0.1:10100/web',ready:true,generation:shows*2})
 })
 await render();expect(shows).toBe(1)
 useNotificationsStore().panelOpen=true;await flushPromises();expect(resolveHide).toBeTypeOf('function')
 useNotificationsStore().panelOpen=false;await flushPromises()
 resolveHide({visible:false,panelUrl:null,ready:true,generation:3});await flushPromises()
 expect(shows).toBe(2)
 expect(wrapper!.text()).toContain('官方面板已嵌入主窗口')
})

it('does not replay a queued restoration after leaving the panel route',async()=>{
 const hides:((value:unknown)=>void)[]=[]
 let shows=0
 invoke.mockImplementation((command:string,args?:{request?:{action?:string}})=>{
  if(command!=='sync_embedded_panel')return Promise.resolve(undefined)
  if(args?.request?.action==='hide')return new Promise(resolve=>hides.push(resolve))
  shows++;return Promise.resolve({visible:true,panelUrl:'http://127.0.0.1:10100/web',ready:true,generation:2})
 })
 await render()
 useNotificationsStore().panelOpen=true;await flushPromises()
 useNotificationsStore().panelOpen=false;await flushPromises()
 wrapper!.unmount();wrapper=undefined
 for(const resolve of hides)resolve({visible:false,panelUrl:null,generation:3})
 await flushPromises()
 expect(shows).toBe(1)
})
