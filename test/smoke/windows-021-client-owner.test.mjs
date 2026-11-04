import {test} from 'node:test'
import assert from 'node:assert/strict'
import {assertDefaultDiscoveryEnvironment,bindingAdmission,OVERRIDES} from './windows-021-client-owner.mjs'
const root='C:\\Temp\\ocx021-owned'
const env={OCX_WINDOWS_ACTUAL_OWNER:'1',PATH:root+'\\installation\\node_modules\\.bin;C:\\Windows\\System32',...Object.fromEntries(['HOME','USERPROFILE','CODEX_HOME','OPENCODEX_HOME','TEMP','TMP','APPDATA','LOCALAPPDATA'].map(key=>[key,root+'\\'+key]))}
test('default Windows discovery forbids every forced native runtime variable and owns all client directories',()=>{
 assert.doesNotThrow(()=>assertDefaultDiscoveryEnvironment(env,root,'win32'))
 for(const key of OVERRIDES)assert.throws(()=>assertDefaultDiscoveryEnvironment({...env,[key]:'C:\\native.exe'},root,'win32'))
 for(const key of ['HOME','USERPROFILE','CODEX_HOME','OPENCODEX_HOME'])assert.throws(()=>assertDefaultDiscoveryEnvironment({...env,[key]:'C:\\daily'},root,'win32'))
 assert.throws(()=>assertDefaultDiscoveryEnvironment(env,root,'darwin'))
})
test('actual npm cmd binding is preserved as a product block, never substituted with a guessed exe',()=>{
 const runtime={path:root+'\\installation\\node_modules\\.bin\\codex.cmd',version:'0.166.0',source:'path'},before=structuredClone(runtime)
 const receipt=bindingAdmission(runtime,root)
 assert.equal(receipt.admitted,false);assert.equal(receipt.extension,'.cmd');assert.match(receipt.reason,/client_selection_executable/)
 assert.deepEqual(runtime,before)
})
test('only an actual default native binding inside the owned root is admitted',()=>{
 const runtime={path:root+'\\installation\\codex.exe',version:'0.166.0',source:'path'}
 assert.equal(bindingAdmission(runtime,root).admitted,true)
 for(const change of [{source:'environment'},{source:'configured'},{version:'0.165.0'},{path:'C:\\Users\\daily\\codex.exe'}])assert.throws(()=>bindingAdmission({...runtime,...change},root))
})
