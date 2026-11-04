import {test} from 'node:test'
import assert from 'node:assert/strict'
import {assertDefaultDiscoveryEnvironment,bindingAdmission,preserveBindingObservation,OVERRIDES} from './windows-021-client-owner.mjs'
import {mkdtemp,readFile,rm} from 'node:fs/promises'
import {tmpdir} from 'node:os'
import {join} from 'node:path'
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
test('failed ownership preserves actual default settings path/source before refusing it',async t=>{
 const directory=await mkdtemp(join(tmpdir(),'ocx021-binding-capture-'));t.after(()=>rm(directory,{recursive:true,force:true}))
 const out=join(directory,'settings.json'),result={codexRuntime:{path:'C:\\outside\\codex.cmd',source:'path',version:'0.166.0'},admission:{admitted:false,status:'not-yet-validated'}}
 await assert.rejects(preserveBindingObservation(out,result,root),/escaped this runner-owned root/)
 assert.deepEqual(JSON.parse(await readFile(out,'utf8')),result,'Real failing path must survive without admission or substitution')
})
