import {test} from 'node:test'
import assert from 'node:assert/strict'
import {win32} from 'node:path'
import {runtimeAcceptanceLayout,assert023RuntimeAdmission,assertVersionedRuntimeAdmission,readInitialRuntimeEnvironment,nativePathClipboardMatches} from './windows023-runtime-layout.mjs'
import {assert023ReadyIpcSafety,prepare021ReadyWebdav} from './windows-021-ready-webdav.mjs'
const root=win32.join('C:/','owned','中文 Spaces'),layout=runtimeAcceptanceLayout('0.2.23',root)
const facts={about:{platform:'Windows',version:'0.2.23'},dataRoot:{aggregateRoot:win32.join(root,'data'),currentDataRoot:layout.dataRoot,activeDataRoot:layout.dataRoot,currentOpencodexHome:layout.opencodexHome,opencodexHome:layout.opencodexHome,opencodexHomeMode:'inside',runtimeActive:true}}
test('023 admits only the bound D/H of its explicit physical sandbox, retaining older version-specific layouts',()=>{
 assert.deepEqual(assert023RuntimeAdmission(facts,root),layout)
 assert.equal(layout.dataRoot,win32.join(root,'data','desktop'))
 assert.equal(layout.installationRoot,win32.join(root,'program'))
 assert.equal(runtimeAcceptanceLayout('0.2.22',root).dataRoot,root)
 for(const key of ['aggregateRoot','currentDataRoot','activeDataRoot','currentOpencodexHome','opencodexHome'])assert.throws(()=>assert023RuntimeAdmission({...facts,dataRoot:{...facts.dataRoot,[key]:win32.join(root,'old')}},root))
 for(const version of ['0.2.230','0.2.25','0.2.23-rc.1',undefined])assert.throws(()=>runtimeAcceptanceLayout(version,root))
 assert.throws(()=>runtimeAcceptanceLayout('0.2.23','relative'))
})
test('024 admits only its own active layout without admitting it to the 023 native driver',()=>{
 const candidate={...facts,about:{...facts.about,version:'0.2.24'}}
 assert.deepEqual(assertVersionedRuntimeAdmission(candidate,root),layout)
 assert.throws(()=>assert023RuntimeAdmission(candidate,root))
 for(const key of ['aggregateRoot','currentDataRoot','activeDataRoot','currentOpencodexHome','opencodexHome']){
  assert.throws(()=>assertVersionedRuntimeAdmission({...candidate,dataRoot:{...candidate.dataRoot,[key]:root}},root))
 }
 assert.throws(()=>assertVersionedRuntimeAdmission({...candidate,dataRoot:{...candidate.dataRoot,runtimeActive:false}},root))
})
test('023 ready observation admits only explicit V2 reads and never expands legacy write authorization',()=>{
 assert.doesNotThrow(()=>assert023ReadyIpcSafety(['app_about','config_v2_catalog','config_v2_read_scope','model_paths_v2_snapshot']))
 for(const command of ['config_v2_confirm','config_v2_resume','config_v2_save_scope','model_v2_save','config_sync_observe','unknown'])assert.throws(()=>assert023ReadyIpcSafety(['app_about',command]))
 assert.throws(()=>assert023ReadyIpcSafety(['config_v2_catalog']))
 assert.throws(()=>assert023ReadyIpcSafety(['app_about'],['config_v2_confirm']))
})
test('wrong 023 HOME fails before any command, reload or endpoint fixture is touched',async()=>{
 let calls=0
 await assert.rejects(prepare021ReadyWebdav({invoke:async()=>{calls++},call:async()=>{calls++}}, {...facts,dataRoot:{...facts.dataRoot,currentOpencodexHome:root}},root,true))
 assert.equal(calls,0)
})

test('fresh 023 refuses only typed unresolved/pending while preserving real success and all other failures',async()=>{
 const ready={gate:'ready',node:{found:true},npm:{found:true}}
 assert.equal(await readInitialRuntimeEnvironment(async()=>ready,'0.2.23'),ready)
 for(const code of ['unresolved','pending']){
  const error={code:14,message:runtimeErrorMessage(code)}
  const value=await readInitialRuntimeEnvironment(async()=>{throw error},'0.2.23')
  assert.equal(value.readyVerified,false);assert.equal(value.gate,'integrated_runtime_not_ready')
  await assert.rejects(readInitialRuntimeEnvironment(async()=>{throw error},'0.2.22'),e=>e===error)
 }
 for(const error of [{code:14,message:runtimeErrorMessage('invalid')},{code:13,message:runtimeErrorMessage('unresolved')},new Error('disk corrupt')])await assert.rejects(readInitialRuntimeEnvironment(async()=>{throw error},'0.2.23'),e=>e===error)
})
const runtimeErrorMessage = reason => 'managed runtime operation failed: discovery_runtime_'+reason+'; 集成运行程序尚未准备完成'
test('024 WebView discovery keeps the original typed refusal and rejects unexpected errors',async()=>{
 const discover=Function('return ('+readInitialRuntimeEnvironment.toString()+')')()
 const ready={gate:'ready'}
 assert.equal(await discover(async()=>ready,'0.2.24'),ready)
 for(const detail of ['unresolved','pending']){
  const error={code:14,message:runtimeErrorMessage(detail)}
  assert.deepEqual(await discover(async()=>{throw error},'0.2.24'),{
   gate:'integrated_runtime_not_ready',code:error.code,message:error.message,readyVerified:false,
  })
 }
 for(const error of [
  {code:14,message:runtimeErrorMessage('invalid')},
  {code:13,message:runtimeErrorMessage('unresolved')},
  {code:14,message:runtimeErrorMessage('unresolved_extra')},
  {code:14,message:runtimeErrorMessage('invalid')+' discovery_runtime_unresolved'},
 ]){
  await assert.rejects(discover(async()=>{throw error},'0.2.24'),e=>e===error)
 }
})

test('native multiline clipboard accepts Windows CRLF transport only, never changed paths, order or dropped lines',()=>{
 const expected=['original path1','original path2'].join(String.fromCharCode(10))
 assert(nativePathClipboardMatches(expected,expected))
 assert(nativePathClipboardMatches(expected.replaceAll(String.fromCharCode(10),String.fromCharCode(13,10)),expected))
 for(const actual of [expected+'extra',expected.split(String.fromCharCode(10)).reverse().join(String.fromCharCode(10)),expected.replace('path2','path3'),expected.replaceAll(String.fromCharCode(10),String.fromCharCode(13)),null])assert.equal(nativePathClipboardMatches(actual,expected),false)
})
