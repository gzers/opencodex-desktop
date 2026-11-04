import {test} from 'node:test'
import assert from 'node:assert/strict'
import {assertDefaultDiscoveryEnvironment,bindingAdmission,preserveBindingObservation,assertRuntimeCorrespondence,nativeLaunchObserver,OVERRIDES} from './windows-021-client-owner.mjs'
import {mkdtemp,readFile,rm} from 'node:fs/promises'
import {tmpdir} from 'node:os'
import {join} from 'node:path'
import {execFile} from 'node:child_process'
import {promisify} from 'node:util'
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
 const out=join(directory,'settings.json'),runtime={path:'C:\\outside\\codex.cmd',source:'path',version:'0.166.0'},result={codexRuntime:runtime,machineObservation:{runtime:{command:runtime.path,source:runtime.source,version:runtime.version},displayPath:runtime.path},admission:{admitted:false,status:'not-yet-validated'}}
 await assert.rejects(preserveBindingObservation(out,result,root))
 assert.deepEqual(JSON.parse(await readFile(out,'utf8')),result,'Real failing path must survive without admission or substitution')
})
test('official display correspondence proves the same raw resolver command without restoring a redacted string',()=>{
 const settings={path:'C:\\Users\\[USER]\\owned\\codex.cmd',source:'path',version:'0.166.0'},machine={runtime:{command:'C:\\Users\\runneradmin\\owned\\codex.cmd',source:'path',version:'0.166.0'},displayPath:settings.path},before=structuredClone(settings)
 assert.doesNotThrow(()=>assertRuntimeCorrespondence(settings,machine));assert.deepEqual(settings,before)
 for(const change of [{displayPath:'other'},{runtime:{...machine.runtime,source:'environment'}},{runtime:{...machine.runtime,version:'0.165.0'}}])assert.throws(()=>assertRuntimeCorrespondence(settings,{...machine,...change}))
})
test('launcher evidence observes an actual native child spawn without replacing its target or arguments',async t=>{
 const root=await mkdtemp(join(tmpdir(),'ocx021-launch-observer-'));t.after(()=>rm(root,{recursive:true,force:true}))
 const trace=join(root,'spawn.json'),preload='data:text/javascript;base64,'+Buffer.from(nativeLaunchObserver(trace)).toString('base64')
 const output=await promisify(execFile)(process.execPath,['--import',preload,'-e',"const {spawn}=require('node:child_process');const c=spawn(process.execPath,['--version'],{stdio:'inherit'});c.on('exit',code=>process.exit(code));"])
 assert.match(output.stdout,/^v[0-9]/)
 const observed=JSON.parse(await readFile(trace,'utf8'));assert.equal(observed.path,process.execPath);assert.deepEqual(observed.args,[process.execPath,'--version']);assert(observed.pid>0)
})
