import {test} from 'node:test'
import assert from 'node:assert/strict'
import {Session} from 'node:inspector/promises'
import {mkdtemp,mkdir,writeFile,readFile,rm} from 'node:fs/promises'
import {tmpdir} from 'node:os'
import {join} from 'node:path'
import {runInNewContext} from 'node:vm'
import {assert021Admission,assert021RenameReadback,assert021RenameBackups,assert021DetectionLayout,assert021WebdavState,verifyWindows021Probes,close021Dialog,native021Click} from './windows-021-probes.mjs'
import {observe021Ipc,assert021IpcSafety,assert021NoticeTiming,install021NoticeObservation,sample021Notice,install021CandidateObservation} from './windows-021-observer.mjs'

const root='C:\\runner\\owned-sandbox',home=root+'\\opencodex-home'
const facts={about:{platform:'Windows',version:'0.2.21'},dataRoot:{currentDataRoot:root,activeDataRoot:root,runtimeActive:true,opencodexHomeMode:'inside',currentOpencodexHome:home,opencodexHome:home}}
test('021 probes fail closed before IPC or filesystem access for wrong version/platform/immutable roots',async()=>{
 assert.equal(assert021Admission(facts,root),home+'\\config.json')
 for(const bad of [
  {...facts,about:{...facts.about,version:'0.2.20'}},{...facts,about:{...facts.about,version:'0.2.21-rc.1'}},
  {...facts,about:{...facts.about,platform:'macOS'}},
  ...[{runtimeActive:false},{currentDataRoot:'C:\\Users\\daily'},{activeDataRoot:'C:\\elsewhere'},{opencodexHomeMode:'external'},{currentOpencodexHome:'C:\\Users\\daily'},{opencodexHome:'C:\\Users\\daily'}].map(change=>({...facts,dataRoot:{...facts.dataRoot,...change}})),
 ]){
  let calls=0
  await assert.rejects(verifyWindows021Probes({invoke:async()=>{calls++}},bad,root));assert.equal(calls,0)
 }
})
test('021 orchestrator rereads actual Tauri facts before seeding, preserving a failing evidence receipt',async t=>{
 const output=await mkdtemp(join(tmpdir(),'ocx021-facts-'));t.after(()=>rm(output,{recursive:true,force:true}))
 const commands=[]
 await assert.rejects(verifyWindows021Probes({output,invoke:async command=>{commands.push(command);return command==='app_about'?facts.about:{...facts.dataRoot,currentDataRoot:'C:\\Users\\daily'}}},facts,root),/immutable startup root/)
 assert.deepEqual(commands,['app_about','get_data_root_config'])
 const receipt=JSON.parse(await readFile(join(output,'native021-behavior.json'),'utf8'))
 assert.equal(receipt.result,'fail');assert(!receipt.rename)
})
test('non-pausing CDP observer really observes a non-writable invoke and never replaces it',async t=>{
 const session=new Session();session.connect();t.after(()=>session.disconnect())
 const originalWindow=Object.getOwnPropertyDescriptor(globalThis,'window'),calls=[]
 Object.defineProperty(globalThis,'window',{value:{__TAURI_INTERNALS__:{}},configurable:true})
 t.after(()=>{if(originalWindow)Object.defineProperty(globalThis,'window',originalWindow);else delete globalThis.window;delete globalThis.__ocx021IpcTrace})
 const invoke=function(command){calls.push(command);return Promise.resolve({version:'0.2.21'})}
 Object.defineProperty(window.__TAURI_INTERNALS__,'invoke',{value:invoke})
 const descriptor=Object.getOwnPropertyDescriptor(window.__TAURI_INTERNALS__,'invoke')
 const driver={call:(method,args)=>session.post(method,args),evaluate:async expression=>{const value=await session.post('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!value.exceptionDetails);return value.result.value},invoke:command=>window.__TAURI_INTERNALS__.invoke(command)}
 const observer=await observe021Ipc(driver)
 await driver.invoke('model_file_credential')
 await driver.invoke('model_agent_preferences')
 const result=await observer.finish()
 assert.deepEqual(result.commands,['app_about','model_file_credential','model_agent_preferences']);assert.deepEqual(calls,result.commands)
 assert.deepEqual(Object.getOwnPropertyDescriptor(window.__TAURI_INTERNALS__,'invoke'),descriptor)
 assert.equal(globalThis.__ocx021IpcTrace,undefined)
 for(const command of ['model_node_measure','model_agent_preferences_save']){
  const failed=await observe021Ipc(driver)
  await driver.invoke(command) // Synthetic Node function, never real backend/network.
  await assert.rejects(failed.finish(),/Unexpected native command/)
  assert.equal(globalThis.__ocx021IpcTrace,undefined,'failed observations must remove their trace and breakpoint')
 }
})
test('IPC safety requires a live sentinel and rejects network, restart, keychain and extra writes',()=>{
 assert.doesNotThrow(()=>assert021IpcSafety(['app_about','get_sync_config','list_imported_sync_connections']))
 assert.doesNotThrow(()=>assert021IpcSafety(['app_about','model_file_save'],['model_file_save']))
 for(const commands of [[],['model_file_snapshot'],...['model_node_measure','model_channel_discover','model_policy_refresh_cache','test_sync_connection','test_sync_candidate','config_sync_check','config_sync_publish','config_sync_download','activate_imported_sync_connection','model_agent_generate','model_agent_preferences_save','model_file_save','anything_unreviewed'].map(c=>['app_about',c])])assert.throws(()=>assert021IpcSafety(commands))
})

function renameFixture(){
 const from='native021-old',to='native021-new',channelId='stable-id',ids=['vendor/a:8b','vendor-a:8b','Mixed_Case/V2']
 const provider={apiKey:'synthetic-only',models:ids,native021Opaque:{literal:from+'/leave-opaque',keep:42}}
 const before={providers:{[from]:provider,[from+'0']:{apiKey:'synthetic-neighbor'}},defaultProvider:from,modelPickerOrder:[from+'/'+ids[0],from+'0/untouched',from+'/'+ids[1],from+'/'+ids[2]],disabledModels:[from+'/'+ids[2]],futureRoot:{literal:from+'/leave-opaque'}}
 const after=structuredClone(before);after.providers[to]=after.providers[from];delete after.providers[from];after.defaultProvider=to;after.modelPickerOrder=[to+'/'+ids[0],from+'0/untouched',to+'/'+ids[1],to+'/'+ids[2]];after.disabledModels=[to+'/'+ids[2]]
 const beforeSnapshot={policyDigest:'old-file',policy:{schemaVersion:1,defaultChannelId:channelId,channels:[{id:channelId,label:from,modelOrder:ids,connection:{providerId:from,nodes:[{id:'stable-node',url:'https://fixture.invalid'}],activeNodeId:'stable-node',credentialRef:null}},{id:'neighbor-id',label:from+'0',connection:{providerId:from+'0'}}]}}
 const afterSnapshot=structuredClone(beforeSnapshot);afterSnapshot.policyDigest='new-file';afterSnapshot.policy.channels[0].label=to;afterSnapshot.policy.channels[0].connection.providerId=to
 return {from,to,channelId,before,after,beforeSnapshot,afterSnapshot}
}
test('rename readback validates entire real file, exact namespaces, stable IDs, defaults, order and Keys',()=>{
 assert.doesNotThrow(()=>assert021RenameReadback(renameFixture()))
 for(const mutate of [
  x=>{x.after.providers[x.to].apiKey='lost-key'},
  x=>{x.after.providers[x.from]={}},
  x=>{delete x.after.providers[x.from+'0']},
  x=>{x.after.defaultProvider=x.from},
  x=>{x.after.modelPickerOrder.reverse()},
  x=>{x.after.modelPickerOrder[1]=x.to+'0/untouched'},
  x=>{x.after.modelPickerOrder[0]=x.to+'/vendor-a:8b'},
  x=>{x.after.providers[x.to].native021Opaque.literal=x.to+'/leave-opaque'},
  x=>{x.afterSnapshot.policy.channels[0].id='new-identity'},
  x=>{x.afterSnapshot.policy.defaultChannelId='neighbor-id'},
  x=>{x.afterSnapshot.policy.channels.reverse()},
  x=>{x.afterSnapshot.policyDigest=x.beforeSnapshot.policyDigest},
 ]){const value=renameFixture();mutate(value);assert.throws(()=>assert021RenameReadback(value))}
})
test('name-only rename requires persisted display name while protecting the full provider document',()=>{
 const value=renameFixture();value.updatesDisplayName=true
 value.before.providers[value.from].name=value.from
 value.after.providers[value.to].name=value.to
 assert.doesNotThrow(()=>assert021RenameReadback(value))
 for(const mutate of [
  x=>{x.after.providers[x.to].name=x.from},
  x=>{delete x.after.providers[x.to].name},
  x=>{x.after.providers[x.to].apiKey='lost-key'},
  x=>{x.after.providers[x.to].native021Opaque.literal=x.to+'/leave-opaque'},
  x=>{x.after.providers[x.from+'0'].name='changed-neighbor'},
 ]){const changed=structuredClone(value);mutate(changed);assert.throws(()=>assert021RenameReadback(changed))}
 assert.throws(()=>assert021RenameReadback({...value,updatesDisplayName:false}),'historical ID contract must not silently adopt display-name writes')
})
test('rename backup validation requires one exact ordinary backup plus exact transaction pre/post images',async t=>{
 const output=await mkdtemp(join(tmpdir(),'ocx021-backups-'));t.after(()=>rm(output,{recursive:true,force:true}))
 const before={bytes:Buffer.from('exact-prewrite\n'),backups:['older']},transaction='rename-12345678-1234-1234-1234-123456789abc',after={bytes:Buffer.from('exact-postwrite\n'),backups:['older','ordinary',transaction].sort()},base=join(output,'backups','model-config')
 await mkdir(join(base,'ordinary'),{recursive:true});await mkdir(join(base,transaction));await mkdir(join(output,'manager-state'))
 await writeFile(join(base,'ordinary','config.json'),before.bytes);await writeFile(join(base,transaction,'0.before'),before.bytes);await writeFile(join(base,transaction,'0.after'),after.bytes)
 await assert021RenameBackups(before,after,output)
 for(const backups of [['ordinary',transaction],['older',transaction],['older','ordinary'],['older','ordinary',transaction,'extra']])await assert.rejects(assert021RenameBackups(before,{...after,backups},output))
 await writeFile(join(base,transaction,'0.before'),'wrong preimage');await assert.rejects(assert021RenameBackups(before,after,output),/preimage/)
 await writeFile(join(base,transaction,'0.before'),before.bytes);await writeFile(join(base,transaction,'0.after'),'wrong postimage');await assert.rejects(assert021RenameBackups(before,after,output),/postimage/)
 await writeFile(join(base,transaction,'0.after'),after.bytes);await writeFile(join(output,'manager-state','provider-rename-pending.json'),'pending');await assert.rejects(assert021RenameBackups(before,after,output),/pending recovery/)
})
test('eight-second notice timing subtracts actual confirmation pauses and rejects shortened TTL/focus theft',()=>{
 const value={shownAt:100,removedAt:13600,leavingAt:13200,focusOnAnchor:true,phases:[{phase:'running',at:100},{phase:'paused',at:1100},{phase:'running',at:6200},{phase:'elapsed',at:13200}]}
 assert.equal(assert021NoticeTiming(value).runningMs,8000)
 for(const change of [{phases:[{phase:'running',at:100},{phase:'elapsed',at:7100}]},{phases:[{phase:'running',at:100},{phase:'elapsed',at:7900}]},{phases:[{phase:'running',at:100},{phase:'elapsed',at:8900}]},{shownAt:null},{removedAt:null},{focusOnAnchor:false},{phases:[]}])assert.throws(()=>assert021NoticeTiming({...value,...change}))
})
test('notice DOM observation measures real mutations/focus without installing any timers or app state',()=>{
 let present=false,time=10,mutation,focused,phase='running',transform='scaleX(0.75)'
 const anchor={textContent:'模型',focus(){focused=anchor}},listeners=new Map()
 const card={dataset:{},contains:e=>e===card,matches:()=>false,querySelector:selector=>selector==='.notification-countdown'?{dataset:{phase}}:selector==='.notification-countdown-fill'?{style:{transform}}:{textContent:'本通知将在8秒后收起'}}
 const document={body:{},hidden:false,get activeElement(){return focused},querySelectorAll:()=>[anchor],querySelector:()=>present?card:null,addEventListener:(type,fn)=>listeners.set(type,fn),removeEventListener:type=>listeners.delete(type)}
 const context={document,performance:{now:()=>time},MutationObserver:class {constructor(callback){mutation=callback}observe(){}disconnect(){mutation=null}}}
 runInNewContext('('+install021NoticeObservation.toString()+')()',context)
 assert.equal(focused,anchor);present=true;mutation()
 const sample=()=>JSON.parse(JSON.stringify(runInNewContext('('+sample021Notice.toString()+')()',context)))
 assert.equal(sample().ratio,0.75);assert(sample().focusOnAnchor)
 time=1010;phase='paused';mutation();time=3010;phase='running';mutation();time=9010;phase='elapsed';card.dataset.notificationMotion='leaving';mutation();time=9430;present=false;mutation()
 assert.equal(sample().phases.length,4);assert.equal(sample().removedAt,9430)
 context.__ocx021Notice.dispose();assert.equal(mutation,null);assert.equal(listeners.size,0)
})
test('native detection layout rejects off-center/legacy/footer/auto-request scenes',()=>{
 const value={native:true,open:true,title:'Native · 节点检测',closeCount:1,closeEnabled:true,footerCount:0,left:220,top:200,right:780,bottom:600,width:560,height:400,centerX:500,centerY:400,viewportWidth:1000,viewportHeight:800,statuses:['未检测']}
 assert.doesNotThrow(()=>assert021DetectionLayout(value))
 for(const change of [{native:false},{open:false},{closeCount:0},{closeCount:2},{closeEnabled:false},{footerCount:1},{centerX:400},{centerY:320},{top:-20},{statuses:[]},{statuses:['检测中…']},{statuses:['123 ms']}])assert.throws(()=>assert021DetectionLayout({...value,...change}))
})
test('WebDAV configuration proof binds the real revision, untouched observation and actual rendered or gated DOM',()=>{
 const value={state:'configured',revision:'actual-revision',hasObservation:false,card:{text:'WebDAV 同步 连接状态 已配置 · 未检测',dots:0},gate:{}}
 assert.equal(assert021WebdavState(value,'actual-revision'),'rendered')
 assert.match(assert021WebdavState({...value,card:null,gate:{present:true,settled:true,checks:3}},'actual-revision'),/^environment-gated/)
 for(const change of [{state:'disconnected'},{state:'connected'},{state:'failed'},{revision:'other'},{hasObservation:true},{card:null},{card:{text:'最近检测成功',dots:0}},{card:{text:'已配置 · 未检测',dots:1}},{card:null,gate:{present:true,settled:false,checks:3}},{card:null,gate:{present:true,settled:true,checks:2}}])assert.throws(()=>assert021WebdavState({...value,...change},'actual-revision'))
})
function nativeDialogFixture(){
 let opened=true;const commands=[],cancelEvents=[]
 class Dialog {constructor(){this.open=true}getBoundingClientRect(){return {left:220,top:200,right:780,bottom:600}}dispatchEvent(event){cancelEvents.push(event.type);opened=false;this.open=false}}
 const dialog=new Dialog(),button={disabled:false,scrollIntoView(){},getBoundingClientRect:()=>({left:730,top:210,width:30,height:30}),contains:e=>e===button}
 const document={querySelector:selector=>!opened?null:selector.includes('.modal-close')?button:dialog,elementFromPoint:(x,y)=>x===745&&y===225?button:dialog}
 const evaluate=async expression=>runInNewContext(expression,{document,innerWidth:1000,innerHeight:800,HTMLDialogElement:Dialog,Event})
 const driver={evaluate,wait:async(expression,label)=>{const value=await evaluate(expression);assert(value,label);return value},call:async(method,args)=>{commands.push({method,...args});if(args.type==='mouseReleased'||args.type==='keyUp'){opened=false;dialog.open=false}},click:async(text,selector)=>{commands.push({text,selector});opened=false;dialog.open=false}}
 return {driver,commands,cancelEvents,document,button}
}
for(const exit of ['x','escape','cancel-button','native-cancel','backdrop'])test('021 dialog driver executes '+exit+' against the native host contract',async()=>{
 const fixture=nativeDialogFixture()
 await close021Dialog(fixture.driver,'dialog.ui-dialog[open]',exit)
 if(exit==='escape')assert.deepEqual(fixture.commands.map(({method,type,key,windowsVirtualKeyCode})=>({method,type,key,windowsVirtualKeyCode})),[{method:'Input.dispatchKeyEvent',type:'keyDown',key:'Escape',windowsVirtualKeyCode:27},{method:'Input.dispatchKeyEvent',type:'keyUp',key:'Escape',windowsVirtualKeyCode:27}])
 else if(exit==='native-cancel')assert.deepEqual(fixture.cancelEvents,['cancel'])
 else if(exit==='cancel-button')assert.deepEqual(fixture.commands,[{text:'取消',selector:'dialog.ui-dialog[open] > footer'}])
 else assert.deepEqual(fixture.commands.map(c=>c.type),['mouseMoved','mousePressed','mouseReleased'])
})
test('native X refuses an occluded target before issuing any real mouse events',async()=>{
 const fixture=nativeDialogFixture();fixture.document.elementFromPoint=()=>null
 await assert.rejects(native021Click(fixture.driver,'dialog.ui-dialog[open] .modal-close'),/hit-testable/)
 assert.deepEqual(fixture.commands,[])
})
test('empty-candidate proof waits through a lazy import and real loading, never accepting loading/error/nonempty DOM',()=>{
 let callback,stage='lazy-import',time=100
 class Element {constructor(text){this.textContent=text}matches(selector){return selector==='.imported-connections'}querySelector(){return null}}
 const section=new Element('正在读取导入的连接…'),document={body:{},querySelector:selector=>selector==='dialog.sync-connection-dialog[open]'?{}:selector==='.imported-connections'?['loading','error','nonempty'].includes(stage)?section:null:selector==='.imported-connections [role=status]'&&stage==='loading'?section:null}
 const context={document,Element,performance:{now:()=>time},MutationObserver:class {constructor(fn){callback=fn}observe(){}disconnect(){callback=null}}}
 runInNewContext('('+install021CandidateObservation.toString()+')()',context)
 callback([]);assert.equal(context.__ocx021Candidates.loadingSeen,false);assert.equal(context.__ocx021Candidates.settledAt,null)
 stage='loading';callback([{addedNodes:[section],removedNodes:[]}]);assert(context.__ocx021Candidates.loadingSeen);assert.equal(context.__ocx021Candidates.settledAt,null)
 for(const value of ['error','nonempty']){stage=value;callback([]);assert.equal(context.__ocx021Candidates.settledAt,null)}
 stage='empty';time=700;callback([{addedNodes:[],removedNodes:[section]}]);assert.equal(context.__ocx021Candidates.settledAt,700)
 context.__ocx021Candidates.dispose();assert.equal(callback,null)
})
