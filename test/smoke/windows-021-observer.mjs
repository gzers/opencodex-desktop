// Test tooling only. A non-pausing V8 function breakpoint observes command
// names at the real, non-writable Tauri invoke. No replacement IPC or results.
import assert from 'node:assert/strict'

const TRACE='__ocx021IpcTrace'
const reads=new Set([
 'app_about','get_data_root_config','get_preferences','get_sync_config','get_sync_status',
 'model_agent_preferences', // AgentExecutionField mount reads bounded local sandbox preferences.
 'drain_tray_requests','get_status_snapshot','app_status','list_notifications','runtime_source',
 'app_foreground','window_appearance',
 'official_project_facts','official_remote_cache','get_update_status','update_schedule_plan',
 'discover_environment','get_managed_path_targets','list_extensions','extension_config','tray_state','read_logs',
 'list_imported_sync_connections','model_file_snapshot','model_file_credential',
 'model_upstream_snapshot','model_policy_latest','model_policy_history',
 'model_policy_preview','model_template_agent_settings','model_execution_settings','model_agent_directory',
 'plugin:event|listen','plugin:event|unlisten',
 'config_sync_scope','config_sync_latest_upload','config_sync_catalog',
 'transfer_choices','transfer_protection','transfer_result','import_credential_choices',
 'portable_channel_catalog','portable_account_catalog','portable_extension_catalog','portable_latest_incoming',
])
export function assert021IpcSafety(commands,writes=[],extraReads=[]){
 assert(Array.isArray(commands)&&commands.length>0,'021 IPC observation must be live, not an empty assertion')
 assert(commands.includes('app_about'),'021 IPC observer sentinel missing')
 assert(extraReads.every(command=>command==='config_sync_observe'),'Only the adopted readonly version check can extend legacy observation')
 const allowed=new Set([...reads,...writes,...extraReads])
 for(const command of commands)assert(allowed.has(command),'Unexpected native command in bounded 021 probe: '+command)
}
export async function observe021Ipc({call,evaluate,invoke},safety=assert021IpcSafety){
 await call('Debugger.enable')
 await evaluate('(()=>{if(globalThis.'+TRACE+')throw Error("021 IPC observer already installed");globalThis.'+TRACE+'={commands:[],invoke:window.__TAURI_INTERNALS__.invoke};return true})()')
 let breakpointId
 try{
  const {result,exceptionDetails}=await call('Runtime.evaluate',{expression:'window.__TAURI_INTERNALS__.invoke',objectGroup:'ocx021-ipc',returnByValue:false})
  assert(!exceptionDetails&&result?.objectId,'Actual Tauri invoke must be inspectable')
  ;({breakpointId}=await call('Debugger.setBreakpointOnFunctionCall',{
   objectId:result.objectId,condition:'(globalThis.'+TRACE+'.commands.push(String(arguments[0])),false)',
  }))
  await invoke('app_about')
  const commands=await evaluate('globalThis.'+TRACE+'.commands')
  assert.equal(commands.filter(command=>command==='app_about').length,1,'021 IPC observer failed its real-command sentinel')
 }catch(error){
  if(breakpointId)await call('Debugger.removeBreakpoint',{breakpointId})
  await evaluate('delete globalThis.'+TRACE)
  await call('Runtime.releaseObjectGroup',{objectGroup:'ocx021-ipc'})
  throw error
 }
 let closed=false
 return {
  async finish(writes=[],extraReads=[]){
   assert(!closed,'021 IPC observer already finished');closed=true
   try{
    const observation=await evaluate('({commands:globalThis.'+TRACE+'.commands,unchanged:globalThis.'+TRACE+'.invoke===window.__TAURI_INTERNALS__.invoke})')
    assert(observation.unchanged,'021 probe replaced real IPC')
    safety(observation.commands,writes,extraReads)
    return {method:'non-pausing CDP function breakpoint; real-command sentinel',commands:observation.commands,invokeUnchanged:true}
   }finally{
    await call('Debugger.removeBreakpoint',{breakpointId})
    await evaluate('delete globalThis.'+TRACE)
    await call('Runtime.releaseObjectGroup',{objectGroup:'ocx021-ipc'})
   }
  },
 }
}

// These functions are serialized into the real WebView. They only observe DOM
// and the browser clock; they never access Vue internals or change timers.
export function install021NoticeObservation(){
 if(globalThis.__ocx021Notice)throw Error('021 notice observer already installed')
 const anchor=[...document.querySelectorAll('.side button')].find(e=>e.textContent.trim()==='模型')
 if(!anchor)throw Error('Persistent native focus anchor missing')
 anchor.focus()
 const trace={anchor,shownAt:null,leavingAt:null,removedAt:null,phases:[],focus:[]}
 function inspect(){
  const card=document.querySelector('.notif-layer [data-notification-key^="model-save:"]'),now=performance.now()
  if(card){
   if(trace.shownAt===null)trace.shownAt=now
   const phase=card.querySelector('.notification-countdown')?.dataset.phase
   if(phase&&trace.phases.at(-1)?.phase!==phase)trace.phases.push({phase,at:now})
   if(card.dataset.notificationMotion==='leaving'&&trace.leavingAt===null)trace.leavingAt=now
  }else if(trace.shownAt!==null&&trace.removedAt===null)trace.removedAt=now
 }
 const focus=event=>trace.focus.push({at:performance.now(),inNotice:!!event.target.closest?.('.notif-layer'),onAnchor:event.target===anchor})
 const observer=new MutationObserver(inspect)
 observer.observe(document.body,{childList:true,subtree:true,attributes:true,attributeFilter:['data-phase','data-notification-motion']})
 document.addEventListener('focusin',focus,true)
 trace.dispose=()=>{observer.disconnect();document.removeEventListener('focusin',focus,true)}
 globalThis.__ocx021Notice=trace;inspect()
 return document.activeElement===anchor
}
export function sample021Notice(){
 const trace=globalThis.__ocx021Notice,card=document.querySelector('.notif-layer [data-notification-key^="model-save:"]')
 if(!trace)throw Error('021 notice observer missing')
 const fill=card?.querySelector('.notification-countdown-fill')?.style.transform??''
 return {now:performance.now(),present:!!card,key:card?.dataset.notificationKey,phase:card?.querySelector('.notification-countdown')?.dataset.phase,
  ratio:fill.startsWith('scaleX(')?Number(fill.slice(7,-1)):null,help:card?.querySelector('.notification-countdown-help')?.textContent,
  shownAt:trace.shownAt,leavingAt:trace.leavingAt,removedAt:trace.removedAt,phases:trace.phases,focus:trace.focus,
  focusOnAnchor:document.activeElement===trace.anchor,focusInNotice:!!card?.contains(document.activeElement),
  hidden:document.hidden,hovered:!!card?.matches(':hover')}
}
export function assert021NoticeTiming(trace){
 assert(Number.isFinite(trace.shownAt)&&Number.isFinite(trace.removedAt),'Actual notice appearance/removal required')
 const end=trace.leavingAt??trace.removedAt
 let runningMs=0
 assert.equal(trace.phases[0]?.phase,'running','021 notice must start running')
 for(let i=0;i<trace.phases.length;i++){
  const phase=trace.phases[i],until=trace.phases[i+1]?.at??end
  if(phase.phase==='running')runningMs+=Math.max(0,until-phase.at)
 }
 // Permit observer/frame scheduling, without accepting a shorter TTL. The
 // original native wait budget remains ten seconds after the final resume.
 assert(runningMs>=7950&&runningMs<=8500,'021 notice did not consume its real eight-second running budget: '+runningMs)
 assert.equal(trace.focusOnAnchor,true,'Auto-collapse stole the current native focus')
 return {runningMs,wallMs:trace.removedAt-trace.shownAt}
}

// Remember the real async consumer's loading DOM, including detached mutation
// nodes. An absent lazy component before its import resolves is not "empty".
export function install021CandidateObservation(){
 if(globalThis.__ocx021Candidates)throw Error('021 candidate observer already installed')
 const trace={loadingSeen:false,settledAt:null}
 const isLoading=node=>node instanceof Element&&(node.matches('.imported-connections')||!!node.querySelector('.imported-connections'))&&node.textContent.includes('正在读取导入的连接')
 const inspect=records=>{
  for(const record of records)for(const node of [...record.addedNodes,...record.removedNodes])if(isLoading(node))trace.loadingSeen=true
  if(document.querySelector('.imported-connections [role=status]'))trace.loadingSeen=true
  if(trace.loadingSeen&&document.querySelector('dialog.sync-connection-dialog[open]')&&!document.querySelector('.imported-connections')&&trace.settledAt===null)trace.settledAt=performance.now()
 }
 const observer=new MutationObserver(inspect)
 observer.observe(document.body,{childList:true,subtree:true})
 trace.dispose=()=>observer.disconnect();globalThis.__ocx021Candidates=trace
 return true
}
