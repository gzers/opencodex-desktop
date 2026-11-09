// Post-install acceptance, reached only after the existing real managed
// installer succeeded. The adopted build may perform its normal readonly
// startup version observation; no connection test, sync or state injection.
import assert from 'node:assert/strict'
import {writeFile} from 'node:fs/promises'
import {join} from 'node:path'
import {assert021Admission,inspect021Webdav,assert021WebdavState} from './windows-021-probes.mjs'
import {observe021Ipc,assert021IpcSafety} from './windows-021-observer.mjs'
import {assert023RuntimeAdmission} from './windows023-runtime-layout.mjs'
import {assertNoNativeUiFault} from './windows-021-public-hosts.mjs'

export function assert021OwnedEndpoint(config){
 const e=config?.endpoint
 assert(e&&new URL(e.url).origin==='https://webdav.native021.invalid'&&new URL(e.url).pathname==='/'&&e.username==='synthetic-native021'&&e.remotePath==='native021-only','Ready WebDAV acceptance requires its owned synthetic endpoint')
 assert(/^[a-f0-9]{64}$/.test(e.revision),'Ready WebDAV needs a real backend endpoint revision')
 return e
}
export function assert023ReadyIpcSafety(commands,writes=[],extraReads=[]){
 assert.deepEqual(writes,[],'023 ready observation cannot authorize writes')
 assert.deepEqual(extraReads,[])
 const v2Reads=new Set(['config_v2_catalog','config_v2_read_scope','model_paths_v2_snapshot'])
 assert021IpcSafety(commands.filter(command=>!v2Reads.has(command)))
}
function admission(facts,expected){
 if(facts.about.version==='0.2.23')return assert023RuntimeAdmission(facts,expected)
 return assert021Admission(facts,expected)
}
async function currentFacts({invoke},expected){
 const facts={about:await invoke('app_about'),dataRoot:await invoke('get_data_root_config')}
 admission(facts,expected)
 return facts
}
export async function prepare021ReadyWebdav(driver,facts,expected,seedIfAbsent=false){
 admission(facts,expected)
 await currentFacts(driver,expected)
 let config=await driver.invoke('get_sync_config'),created=false
 if(!config.endpoint){
  assert.equal(seedIfAbsent,true,'The earlier 021 startup probe must have saved its real endpoint')
  assert.deepEqual((await driver.invoke('list_imported_sync_connections')).items,[],'Ready fixture requires the actual fresh sandbox candidate list')
  config=await driver.invoke('save_sync_endpoint',{request:{baseUrl:'https://webdav.native021.invalid',remotePath:'native021-only',username:'synthetic-native021',password:'synthetic-native021-webdav-password',conflictPolicy:'ask'}})
  created=true
 }
 assert021OwnedEndpoint(config)
 assert.deepEqual(await driver.invoke('get_sync_config'),config,'Ready fixture was not actually saved')
 // A normal full reload lets the production bootstrap load the saved endpoint
 // and real environment. Never assign environment or Pinia state from tooling.
 await driver.call('Page.reload',{ignoreCache:true})
 await driver.wait("document.readyState==='complete'&&!!document.querySelector('.side')")
 await currentFacts(driver,expected)
 return {config,created,fixtureReloaded:true}
}
export function assert021ActualReadyEnvironment(environment,source){
 // Match the production environment presentation: managed installations are
 // deliberately outside automatic/global ocx discovery (ready_via_source).
 assert(environment.node?.found&&environment.npm?.found,'Real installed environment is not ready: Node/npm discovery failed')
 if(environment.gate==='ready')return 'ready'
 assert(environment.gate==='missing_ocx'&&source.kind==='managed'&&source.path&&source.version,'Real installed environment is not ready through an installed runtime source')
 return 'ready_via_source'
}
export function assert021ReadyWebdavFacts(environment,source,installed,scene,endpoint){
 const readiness=assert021ActualReadyEnvironment(environment,source)
 for(const key of ['kind','path','managedPrefix','version','history'])assert.deepEqual(source[key],installed[key],'Ready WebDAV observation changed the installed runtime source: '+key)
 assert.equal(source.kind,'managed','Ready check requires the actual successful managed installation')
 assert.equal(assert021WebdavState(scene,endpoint.revision,{startupVersionCheck:scene.versionCheckAvailable===true}),'rendered','Ready WebDAV must actually render its overview card')
 assert.equal(scene.gate.present,false,'Ready WebDAV must not retain an environment gate')
 return readiness
}
export async function verify021ReadyWebdav(driver,output,facts,expected,prepared,installed,stage){
 admission(facts,expected)
 assert(['managed-before-offline-drop','managed-after-offline-drop','recovery-installed'].includes(stage),'Unknown ready WebDAV checkpoint')
 const endpoint=assert021OwnedEndpoint(prepared.config),report={schemaVersion:1,result:'fail',stage,admittedRoot:expected,endpointRevision:endpoint.revision,fixtureCreated:prepared.created}
 let observer
 try{
  report.facts=await currentFacts(driver,expected)
  report.environment=await driver.invoke('discover_environment',{request:null})
  report.source=await driver.invoke('runtime_source')
  report.readiness=assert021ActualReadyEnvironment(report.environment,report.source)
  assert.deepEqual(await driver.invoke('get_sync_config'),prepared.config,'Ready WebDAV endpoint binding changed')
  observer=await observe021Ipc(driver,facts.about.version==='0.2.23'?assert023ReadyIpcSafety:assert021IpcSafety)
  await assertNoNativeUiFault(driver,facts.about.version)
  await driver.click('概览','.side')
  await driver.wait("!!document.querySelector('[data-testid=overview-remote-version]')||!!document.querySelector('[data-testid=environment-gate] .overview-gate-count')?.textContent.includes('/ 3')")
  // Do not wait past a settled setup gate and turn it into a silent skip.
  await driver.wait("(()=>{const s=document.querySelector('#app')?.__vue_app__?.config.globalProperties.$pinia?.state.value.sync;return s?.config?.endpoint?.revision==="+JSON.stringify(endpoint.revision)+"})()")
  await driver.wait('(()=>{const scene=('+inspect021Webdav.toString()+')();return scene.versionCheckAvailable?scene.state==="failed"&&!scene.checkingVersion:scene.state==="configured"})()')
  report.scene=await driver.evaluate('('+inspect021Webdav.toString()+')()')
  report.readiness=assert021ReadyWebdavFacts(report.environment,report.source,installed,report.scene,endpoint)
  await assertNoNativeUiFault(driver,facts.about.version)
  const shot=await driver.call('Page.captureScreenshot',{format:'png'})
  await writeFile(join(output,'native021-ready-webdav-'+stage+'.png'),Buffer.from(shot.data,'base64'))
  report.finalFacts=await currentFacts(driver,expected)
  assert.deepEqual(await driver.invoke('get_sync_config'),prepared.config,'Viewing ready overview mutated the endpoint')
  report.ipc=await observer.finish([],facts.about.version!=='0.2.23'&&report.scene.versionCheckAvailable?['config_sync_observe']:[]);observer=null
  // Return through production navigation to the install surface owned by the
  // caller, so the unchanged offline-drop/query checks retain their precondition.
  await driver.click('设置','.side')
  await driver.wait("!!document.querySelector('.settings-tabs')")
  await driver.click('安装配置','.route-section.active')
  await driver.wait("!!document.querySelector('.route-section.active [data-testid=runtime-install]')")
  report.result='pass'
  return report
 }catch(error){report.error=error.message;throw error}
 finally{
  try{if(observer)report.ipc=await observer.finish([],facts.about.version!=='0.2.23'&&report.scene?.versionCheckAvailable?['config_sync_observe']:[])}
  catch(error){report.ipcError=error.message;report.result='fail';throw error}
  finally{await writeFile(join(output,'native021-ready-webdav-'+stage+'.json'),JSON.stringify(report,null,2)+String.fromCharCode(10))}
 }
}
