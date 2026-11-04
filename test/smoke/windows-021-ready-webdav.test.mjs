import {test} from 'node:test'
import assert from 'node:assert/strict'
import {Session} from 'node:inspector/promises'
import {mkdtemp,readFile,rm} from 'node:fs/promises'
import {tmpdir} from 'node:os'
import {join} from 'node:path'
import {assert021OwnedEndpoint,assert021ActualReadyEnvironment,assert021ReadyWebdavFacts,prepare021ReadyWebdav,verify021ReadyWebdav} from './windows-021-ready-webdav.mjs'

const root='C:\\runner\\owned-sandbox',home=root+'\\opencodex-home'
const facts={about:{platform:'Windows',version:'0.2.21'},dataRoot:{currentDataRoot:root,activeDataRoot:root,runtimeActive:true,opencodexHomeMode:'inside',currentOpencodexHome:home,opencodexHome:home}}
const endpoint={endpointId:'owned-endpoint',revision:'a'.repeat(64),url:'https://webdav.native021.invalid',username:'synthetic-native021',remotePath:'native021-only'},config={endpoint}
const installed={kind:'managed',version:'2.91.0',path:home+'\\runtime\\ocx.cmd',managedPrefix:home+'\\runtime',history:[{result:'success'}]}
const scene={state:'configured',revision:endpoint.revision,hasObservation:false,card:{text:'WebDAV 同步 已配置 · 未检测',dots:0},gate:{present:false,checks:0,settled:false}}
const environment={gate:'ready',node:{found:true},npm:{found:true},ocx:{found:true}}
test('ready WebDAV endpoint admission rejects non-fixture and unbound endpoints',()=>{
 assert.deepEqual(assert021OwnedEndpoint(config),endpoint)
 for(const change of [{url:'https://daily.example'},{username:'daily'},{remotePath:'daily'},{revision:'invented'}, {url:'http://webdav.native021.invalid'}])assert.throws(()=>assert021OwnedEndpoint({endpoint:{...endpoint,...change}}))
 assert.throws(()=>assert021OwnedEndpoint({endpoint:null}))
})
test('ready gate never accepts setup, fake connection observation, changed source or a hidden overview card',()=>{
 assert.doesNotThrow(()=>assert021ReadyWebdavFacts(environment,installed,installed,scene,endpoint))
 for(const gate of ['checking','missing_node','missing_npm'])assert.throws(()=>assert021ReadyWebdavFacts({...environment,gate},installed,installed,scene,endpoint))
 for(const change of [{kind:'unresolved'},{version:'other'},{path:'C:\\daily'},{managedPrefix:'C:\\other'},{history:[]}])assert.throws(()=>assert021ReadyWebdavFacts(environment,{...installed,...change},installed,scene,endpoint))
 for(const change of [{state:'connected'},{hasObservation:true},{card:null,gate:{present:true,settled:true,checks:3}},{gate:{present:true}},{revision:'other'}])assert.throws(()=>assert021ReadyWebdavFacts(environment,installed,installed,{...scene,...change},endpoint))
})
test('actual managed source can satisfy the production ready_via_source branch without inventing automatic ocx discovery',()=>{
 const discovery={...environment,gate:'missing_ocx',ocx:{found:false}}
 assert.equal(assert021ActualReadyEnvironment(discovery,installed),'ready_via_source')
 assert.equal(assert021ReadyWebdavFacts(discovery,installed,installed,scene,endpoint),'ready_via_source')
 for(const source of [{kind:'unresolved'}, {...installed,path:null}, {...installed,version:null}])assert.throws(()=>assert021ActualReadyEnvironment(discovery,source))
 for(const key of ['node','npm'])assert.throws(()=>assert021ActualReadyEnvironment({...discovery,[key]:{found:false}},installed))
})
test('ready fixture rereads immutable roots before any seed and refuses an absent prior endpoint unless the recovery owner explicitly seeds',async()=>{
 const commands=[]
 const driver={invoke:async command=>{commands.push(command);return command==='app_about'?facts.about:command==='get_data_root_config'?facts.dataRoot:{endpoint:null}}}
 await assert.rejects(prepare021ReadyWebdav(driver,facts,root,false),/earlier 021 startup probe/)
 assert.deepEqual(commands,['app_about','get_data_root_config','get_sync_config'])
 commands.length=0
 const bad={...driver,invoke:async command=>{commands.push(command);return command==='app_about'?facts.about:{...facts.dataRoot,currentDataRoot:'C:\\Users\\daily'}}}
 await assert.rejects(prepare021ReadyWebdav(bad,facts,root,true),/immutable startup root/)
 assert(!commands.includes('save_sync_endpoint'))
})
test('ready preparation uses production save/readback and a normal full reload, never environment injection',async()=>{
 const commands=[],cdp=[],waits=[];let saved=null
 const driver={invoke:async(command,args)=>{commands.push({command,args});if(command==='app_about')return facts.about;if(command==='get_data_root_config')return facts.dataRoot;if(command==='get_sync_config')return saved??{endpoint:null};if(command==='list_imported_sync_connections')return {items:[]};if(command==='save_sync_endpoint'){saved=config;assert.equal(args.request.baseUrl,endpoint.url);return config}assert.fail(command)},call:async(method,params)=>cdp.push({method,params}),wait:async expression=>{waits.push(expression);return true}}
 const prepared=await prepare021ReadyWebdav(driver,facts,root,true)
 assert(prepared.created&&prepared.fixtureReloaded);assert.deepEqual(prepared.config,config)
 assert.deepEqual(cdp,[{method:'Page.reload',params:{ignoreCache:true}}]);assert.equal(waits.length,1)
 assert.equal(commands.filter(c=>c.command==='save_sync_endpoint').length,1)
})

test('post-install orchestration observes real invoke, verifies visible ready DOM twice and fails a later regression',async t=>{
 const output=await mkdtemp(join(tmpdir(),'ocx021-ready-'));t.after(()=>rm(output,{recursive:true,force:true}))
 const session=new Session();session.connect();t.after(()=>session.disconnect())
 const original=new Map(['window','document','HTMLDialogElement'].map(key=>[key,Object.getOwnPropertyDescriptor(globalThis,key)]))
 t.after(()=>{for(const [key,descriptor] of original){if(descriptor)Object.defineProperty(globalThis,key,descriptor);else delete globalThis[key]}delete globalThis.__ocx021IpcTrace})
 let gate='ready',route='settings',fault=false
 const commands=[],actions=[],cdp=[]
 const card={textContent:'WebDAV 同步 连接状态 已配置 · 未检测'},row={textContent:'线上版本 — · 未检测',closest:()=>card,querySelectorAll:()=>[]}
 const app={__vue_app__:{config:{globalProperties:{$pinia:{state:{value:{sync:{webdavState:'configured',config,observation:null}}}}}}}}
 const document={querySelector:selector=>{
  if(selector==='#app')return app
  if(selector==='dialog.ui-fault-dialog[open]')return fault?{querySelector:()=>({textContent:'界面出现异常'})}:null
  if(selector==='[data-testid=overview-remote-version]')return gate==='ready'&&route==='overview'?row:null
  if(selector==='[data-testid=environment-gate]')return gate==='ready'?null:{querySelector:()=>({textContent:'3 / 3'}),querySelectorAll:()=>[1,2,3]}
  if(selector==='[data-testid=environment-gate] .overview-gate-count')return gate==='ready'?null:{textContent:'3 / 3'}
  if(selector==='.settings-tabs'||selector==='.route-section.active [data-testid=runtime-install]')return route==='settings'?{}:null
  return null
 }}
 for(const [key,value] of Object.entries({window:{__TAURI_INTERNALS__:{}},document,HTMLDialogElement:class{}}))Object.defineProperty(globalThis,key,{value,configurable:true})
 Object.defineProperty(window.__TAURI_INTERNALS__,'invoke',{value:function(command){commands.push(command);return Promise.resolve(command==='app_about'?facts.about:command==='get_data_root_config'?facts.dataRoot:command==='discover_environment'?{...environment,gate}:command==='runtime_source'?installed:command==='get_sync_config'?config:assert.fail(command))}})
 const driver={
  call:async(method,params)=>{cdp.push(method);return method==='Page.captureScreenshot'?{data:''}:session.post(method,params)},
  evaluate:async expression=>{const value=await session.post('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!value.exceptionDetails,JSON.stringify(value.exceptionDetails));return value.result.value},
  click:async(text,selector)=>{actions.push({text,selector});if(text==='概览')route='overview';if(text==='设置')route='settings'},
  invoke:command=>window.__TAURI_INTERNALS__.invoke(command),
 }
 driver.wait=async expression=>{const value=await driver.evaluate(expression);assert(value,'ready native wait');return value}
 const prepared={config,created:false}
 const before=await verify021ReadyWebdav(driver,output,facts,root,prepared,installed,'managed-before-offline-drop')
 const after=await verify021ReadyWebdav(driver,output,facts,root,prepared,installed,'managed-after-offline-drop')
 assert.equal(before.result,'pass');assert.equal(after.result,'pass');assert(before.ipc.commands.includes('app_about'));assert.equal(route,'settings')
 assert.equal(actions.filter(a=>a.text==='概览').length,2);assert.equal(cdp.filter(c=>c==='Page.captureScreenshot').length,2)
 assert(!commands.some(command=>/test_sync|config_sync_check|install_runtime/.test(command)))
 gate='missing_node'
 await assert.rejects(verify021ReadyWebdav(driver,output,facts,root,prepared,installed,'managed-after-offline-drop'),/not ready/)
 const receipt=JSON.parse(await readFile(join(output,'native021-ready-webdav-managed-after-offline-drop.json'),'utf8'))
 assert.equal(receipt.result,'fail');assert.equal(receipt.environment.gate,'missing_node')
 assert.equal(globalThis.__ocx021IpcTrace,undefined)
 gate='ready';fault=true
 await assert.rejects(verify021ReadyWebdav(driver,output,facts,root,prepared,installed,'recovery-installed'),/real UI fault/)
 assert.equal(globalThis.__ocx021IpcTrace,undefined)
})
test('real managed and recovery callers require the ready checkpoint at the appropriate owned install stages',async()=>{
 const install=await readFile(new URL('./windows-install-ui.mjs',import.meta.url),'utf8'),recovery=await readFile(new URL('./windows-install-recovery.mjs',import.meta.url),'utf8'),webview=await readFile(new URL('./windows-webview.mjs',import.meta.url),'utf8')
 const before=install.indexOf("'managed-before-offline-drop'"),drop=install.indexOf('await verifyWindowsOfflineDrop('),after=install.indexOf("'managed-after-offline-drop'")
 assert(before>0&&before<drop&&drop<after)
 assert(recovery.indexOf("report.about.version,'done')",recovery.indexOf('const recovered='))<recovery.indexOf("'recovery-installed'"))
 assert(webview.includes("verifyWindowsInstallUi(call,output,installMode==='managed-install',modelSandbox)"))
 for(const source of [install,recovery])assert(!source.includes('setEnvironmentFixture'))
})
