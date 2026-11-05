import assert from 'node:assert/strict'
import {readFile,writeFile,mkdir,copyFile,open,access} from 'node:fs/promises'
import {spawn,execFileSync} from 'node:child_process'
import {join,resolve,relative,isAbsolute,win32} from 'node:path'
import {fileURLToPath} from 'node:url'
import {createHash} from 'node:crypto'
import {connect} from './cdp-session.mjs'
import {verifyNative} from './formal-msix-verify.mjs'
import {assertModelSandbox} from '../windows-model-ui.mjs'

const [planFile]=process.argv.slice(2)
assert(planFile,'Explicit reviewed plan required')
const plan=JSON.parse((await readFile(planFile,'utf8')).replace(/^\uFEFF/,''))
const root=resolve(plan.runRoot),testRoot=resolve('D:/Users/Administrator/WorkSpace/Test')
const suffix=relative(testRoot,root)
assert(suffix && !suffix.startsWith('..') && !isAbsolute(suffix),'Run root escapes authorized test path')
const ps=join(process.env.SystemRoot??'C:/Windows','System32/WindowsPowerShell/v1.0/powershell.exe')
const psCall=(script,args=[])=>execFileSync(ps,['-NoProfile','-NonInteractive','-Command',script,...args],{encoding:'utf8',windowsHide:true,timeout:60000})
const actualOwner=psCall('[Security.Principal.WindowsIdentity]::GetCurrent().Name').trim()
assert.equal(actualOwner.toLowerCase(),plan.expectedOwner.toLowerCase(),'Run inside Administrator login session')
const preflight=JSON.parse((await readFile(plan.preflightFile,'utf8')).replace(/^\uFEFF/,''))
assert.equal(preflight.owner.toLowerCase(),actualOwner.toLowerCase())
assert.equal(preflight.sessionId,1)
assert.equal(preflight.config.path.toLowerCase(),'c:\\users\\administrator\\.codex\\config.toml')
const hash=data=>createHash('sha256').update(data).digest('hex')
assert.equal(hash(await readFile(plan.managerExe)),plan.managerSha256,'Exact MSVC candidate required')
await assert.rejects(access(join(root,'runner-started.json')),{code:'ENOENT'},'Do not repeat a completed/started run')
await mkdir(root,{recursive:true})
const save=(name,value)=>writeFile(join(root,name),JSON.stringify(value,null,2)+'\n')
const report={result:'running',acceptance:false,owner:actualOwner,startedAtUtc:new Date().toISOString(),checks:[]}
await save('runner-started.json',report)
const snapshot=async name=>{
 const value=JSON.parse(execFileSync(ps,['-NoProfile','-NonInteractive','-File',fileURLToPath(new URL('./formal-msix-snapshot.ps1',import.meta.url))],{encoding:'utf8',windowsHide:true,timeout:60000}))
 assert.equal(value.owner.toLowerCase(),actualOwner.toLowerCase());assert.equal(value.sessionId,1)
 assert.equal(value.unknownOwnerCount,0,'Unknown process owner cannot admit restart')
 await save(name+'.json',value);return value
}
const sleep=ms=>new Promise(r=>setTimeout(r,ms))
let service,manager,cdp
try{
 const before=await snapshot('process-before')
 assert(!before.own.some(p=>win32.basename(p.path??'').toLowerCase()==='chatgpt.exe'),'Unexpected existing test-owner client; inspect first')
 const original=await readFile(preflight.config.path)
 assert.equal(hash(original),preflight.config.sha256,'Administrator config changed since preflight')
 await mkdir(join(root,'private-backup'),{recursive:true})
 await writeFile(join(root,'private-backup','administrator-config.before.toml'),original,{flag:'wx'})
 await save('config-before-digest.json',{bytes:original.length,sha256:hash(original),path:preflight.config.path,policy:'User explicitly authorized existingAdministratorCodexdata for testing; rawconfigbackup stayslocal'})
 // Replace only the authorized test account's routing config. Keep its login
 // and history files untouched; no credentials enter the evidence report.
 const seed='model_provider = "opencodex"\nmodel = "Post021AdminFixture/raw-model"\n[model_providers.opencodex]\nname = "OpenCodeX"\nbase_url = "http://127.0.0.1:'+plan.servicePort+'/v1"\nwire_api = "responses"\nrequires_openai_auth = false\n'
 await writeFile(preflight.config.path,seed)
 const env={...process.env,HOME:'C:\\Users\\Administrator',USERPROFILE:'C:\\Users\\Administrator',CODEX_HOME:'C:\\Users\\Administrator\\.codex',OPENCODEX_HOME:join(root,'sandbox','opencodex-home'),CODEX_CLI_PATH:plan.codexExe,DO_NOT_TRACK:'1',PATH:join(root,'tools')+';'+win32.dirname(plan.codexExe)+';C:/Windows/System32;C:/Windows'}
 for(const key of ['OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY','OPENCODEX_WINDOWS_SMOKE_CDP_PORT'])delete env[key]
 const serviceOut=await open(join(root,'service.stdout.log'),'wx'),serviceErr=await open(join(root,'service.stderr.log'),'wx')
 service=spawn(plan.bunExe,[fileURLToPath(new URL('./formal-msix-service.ts',import.meta.url)),resolve(planFile),root],{cwd:root,env,detached:true,windowsHide:true,stdio:['ignore',serviceOut.fd,serviceErr.fd]})
 await serviceOut.close();await serviceErr.close()
 assert(service.pid);report.servicePid=service.pid
 const serviceDeadline=Date.now()+90000
 let ready
 while(Date.now()<serviceDeadline){
  try{ready=JSON.parse(await readFile(join(root,'service-ready.json'),'utf8'));break}catch(error){if(error.code!=='ENOENT')throw error}
  assert(service.exitCode===null,'Official service exited during startup');await sleep(500)
 }
 assert(ready && ready.pid===service.pid && ready.port===plan.servicePort,'Official server startup deadline')
 const token=(await readFile(join(env.OPENCODEX_HOME,'admin-api-token'),'utf8')).trim()
 const api=async(path,method='GET',body)=>{
  const response=await fetch('http://127.0.0.1:'+plan.servicePort+path,{method,headers:{Authorization:'Bearer '+token,'Content-Type':'application/json'},body:body===undefined?undefined:JSON.stringify(body),signal:AbortSignal.timeout(60000)})
  const value=await response.json();assert(response.ok,'Official management request failed: '+path+' '+response.status);return value
 }
 await api('/api/native-integrations/codex','PUT',{enabled:true})
 const integrations=await api('/api/native-integrations'),client=integrations.clients.find(v=>v.clientId==='codex')
 assert(client?.installed && win32.resolve(client.configPath).toLowerCase()===win32.resolve(preflight.config.path).toLowerCase(),'Official client binding must be the authorized Administrator profile')
 await save('native-binding.json',{client,service:ready})
 // Launch the genuine registered package, not OwnedClient or a custom adapter.
 const aumid=preflight.packages[0].family+'!App'
 assert(/^[A-Za-z0-9._]+!App$/.test(aumid))
 psCall("Start-Process 'shell:AppsFolder\\"+aumid+"'")
 let launched
 for(let n=0;n<12;n++){
  await sleep(2000);launched=await snapshot('process-launched')
  if(launched.own.some(p=>win32.basename(p.path??'').toLowerCase()==='chatgpt.exe'))break
 }
 assert(launched?.own.some(p=>win32.basename(p.path??'').toLowerCase()==='chatgpt.exe'),'Formal package did not expose a real shell')
 const packageRoot=win32.resolve(preflight.packages[0].installLocation).toLowerCase()+'\\'
 const shells=launched.own.filter(p=>win32.basename(p.path??'').toLowerCase()==='chatgpt.exe')
 assert(shells.every(p=>p.sessionId===1 && win32.resolve(p.path).toLowerCase().startsWith(packageRoot)),'Unproven formal package owner/session/path')
 report.formalBefore=shells
 const managerEnv={...env,OPENCODEX_SANDBOX:'1',OPENCODEX_SANDBOX_ROOT:join(root,'sandbox'),OPENCODEX_SANDBOX_BOUNDARY:root,OPENCODEX_WINDOWS_SMOKE_CDP_PORT:String(plan.cdpPort),APPDATA:join(root,'manager-profile','Roaming'),LOCALAPPDATA:join(root,'manager-profile','Local'),TEMP:join(root,'temp'),TMP:join(root,'temp')}
 const managerOut=await open(join(root,'manager.stdout.log'),'wx'),managerErr=await open(join(root,'manager.stderr.log'),'wx')
 manager=spawn(plan.managerExe,[],{cwd:root,env:managerEnv,detached:true,windowsHide:false,stdio:['ignore',managerOut.fd,managerErr.fd]})
 await managerOut.close();await managerErr.close();assert(manager.pid);report.managerPid=manager.pid
 for(let n=0;n<100;n++){try{cdp=await connect('http://127.0.0.1:'+plan.cdpPort);break}catch{assert(manager.exitCode===null,'Manager exited');await sleep(500)}}
 assert(cdp,'Native manager CDP unavailable')
 await verifyNative({cdp,plan,root,preflight,snapshot,api,report,managerPid:manager.pid,shells,save})
}catch(error){report.result='failed';report.error={message:error.message,stack:error.stack};process.exitCode=1}
finally{
 report.finishedAtUtc=new Date().toISOString();report.cleanup='Processes and authorized testprofile retained for inspection; no broadkill orrestore. Privatebackup stayslocal.'
 await save('formal-result.json',report);cdp?.close()
 service?.unref();manager?.unref()
 console.log(JSON.stringify({result:report.result,error:report.error?.message,output:root}))
}
