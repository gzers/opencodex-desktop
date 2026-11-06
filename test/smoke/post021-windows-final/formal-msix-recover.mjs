import assert from 'node:assert/strict'
import {readFile,writeFile,mkdir,open,access} from 'node:fs/promises'
import {spawn,execFileSync} from 'node:child_process'
import {join,resolve,win32} from 'node:path'
import {fileURLToPath} from 'node:url'
import {createHash} from 'node:crypto'
import {connect} from './cdp-session.mjs'
import {verifyNative} from './formal-msix-verify.mjs'

const [planFile]=process.argv.slice(2),plan=JSON.parse(await readFile(planFile,'utf8')),root=resolve(plan.runRoot)
assert(root===resolve('D:/Users/Administrator/WorkSpace/Test/post021-formal-20261105-1518'),'Only this initialized acceptance run can be recovered')
const output=join(root,'bootstrap-recovery'),ps=join(process.env.SystemRoot??'C:/Windows','System32/WindowsPowerShell/v1.0/powershell.exe')
const psCall=script=>execFileSync(ps,['-NoProfile','-NonInteractive','-Command',script],{encoding:'utf8',windowsHide:true,timeout:20000})
const owner=psCall('[Security.Principal.WindowsIdentity]::GetCurrent().Name').trim()
assert.equal(owner.toLowerCase(),plan.expectedOwner.toLowerCase(),'Administrator login required')
const previous=JSON.parse(await readFile(join(root,'formal-result.json'),'utf8'))
assert(previous.result==='failed'&&previous.error?.message.includes("reading 'invoke'"),'Only the observed pre-Save bootstrap failure may recover')
for(const name of ['save-readback.json','production-restart-result.json'])await assert.rejects(access(join(root,name)),{code:'ENOENT'},'Do not repeat an uncertain Save or restart')
assert.equal(createHash('sha256').update(await readFile(plan.managerExe)).digest('hex'),plan.managerSha256)
await mkdir(output,{recursive:false})
const save=(name,value)=>writeFile(join(output,name),JSON.stringify(value,null,2)+'\n')
const snapshot=async name=>{
 const value=JSON.parse(execFileSync(ps,['-NoProfile','-NonInteractive','-File',fileURLToPath(new URL('./formal-msix-snapshot.ps1',import.meta.url))],{encoding:'utf8',windowsHide:true,timeout:60000}))
 assert.equal(value.owner.toLowerCase(),owner.toLowerCase());assert.equal(value.sessionId,1);assert.equal(value.unknownOwnerCount,0)
 await save(name+'.json',value);return value
}
const report={result:'running',acceptance:false,owner,startedAtUtc:new Date().toISOString(),checks:[],previousFailure:'Initial formal-result preserved; no config backup/seed or formal-app launch replay'}
await save('recovery-started.json',report)
const sleep=ms=>new Promise(r=>setTimeout(r,ms))
let service,manager,cdp
try{
 const before=await snapshot('process-recovery-before'),shells=previous.formalBefore
 assert(shells.length&&shells.every(s=>before.own.some(p=>p.pid===s.pid&&p.createdAtUtcTicks===s.createdAtUtcTicks)),'Original formal client generation changed; inspect before any action')
 for(const oldPid of [previous.servicePid,previous.managerPid]){
  assert(Number.isSafeInteger(oldPid))
  assert.equal(psCall("if(Get-CimInstance Win32_Process -Filter 'ProcessId="+oldPid+"'){'LIVE'}else{'GONE'}").trim(),'GONE','Old helper PID is live or reused; do not replace')
 }
 const env={...process.env,HOME:'C:\\Users\\Administrator',USERPROFILE:'C:\\Users\\Administrator',CODEX_HOME:'C:\\Users\\Administrator\\.codex',OPENCODEX_HOME:join(root,'sandbox','opencodex-home'),CODEX_CLI_PATH:plan.codexExe,DO_NOT_TRACK:'1',PATH:join(root,'tools')+';'+win32.dirname(plan.codexExe)+';C:/Windows/System32;C:/Windows'}
 for(const k of ['OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY','OPENCODEX_WINDOWS_SMOKE_CDP_PORT'])delete env[k]
 const so=await open(join(output,'service.stdout.log'),'wx'),se=await open(join(output,'service.stderr.log'),'wx')
 service=spawn(plan.bunExe,[fileURLToPath(new URL('./formal-msix-service.ts',import.meta.url)),resolve(planFile),root,output],{cwd:root,env,detached:true,windowsHide:true,stdio:['ignore',so.fd,se.fd]})
 await so.close();await se.close();assert(service.pid);report.servicePid=service.pid
 let ready
 for(let n=0;n<180;n++){
  try{ready=JSON.parse(await readFile(join(output,'service-ready.json'),'utf8'));break}catch(e){if(e.code!=='ENOENT')throw e}
  assert(service.exitCode===null,'Recovered official service exited');await sleep(500)
 }
 assert(ready?.pid===service.pid&&ready?.port===plan.servicePort,'Recovered service did not become ready')
 const token=(await readFile(join(env.OPENCODEX_HOME,'admin-api-token'),'utf8')).trim()
 const api=async(path,method='GET',body)=>{
  const r=await fetch('http://127.0.0.1:'+plan.servicePort+path,{method,headers:{Authorization:'Bearer '+token,'Content-Type':'application/json'},body:body===undefined?undefined:JSON.stringify(body),signal:AbortSignal.timeout(60000)})
  const value=await r.json();assert(r.ok,'Official management request failed '+path);return value
 }
 const managerEnv={...env,OPENCODEX_SANDBOX:'1',OPENCODEX_SANDBOX_ROOT:join(root,'sandbox'),OPENCODEX_SANDBOX_BOUNDARY:root,OPENCODEX_WINDOWS_SMOKE_CDP_PORT:String(plan.cdpPort),APPDATA:join(root,'manager-profile','Roaming'),LOCALAPPDATA:join(root,'manager-profile','Local'),TEMP:join(root,'temp'),TMP:join(root,'temp')}
 const mo=await open(join(output,'manager.stdout.log'),'wx'),me=await open(join(output,'manager.stderr.log'),'wx')
 manager=spawn(plan.managerExe,[],{cwd:root,env:managerEnv,detached:true,windowsHide:false,stdio:['ignore',mo.fd,me.fd]})
 await mo.close();await me.close();assert(manager.pid);report.managerPid=manager.pid
 for(let n=0;n<100;n++){try{cdp=await connect('http://127.0.0.1:'+plan.cdpPort);break}catch{assert(manager.exitCode===null,'Recovered manager exited');await sleep(500)}}
 assert(cdp,'Recovered manager CDP unavailable')
 const preflight=JSON.parse((await readFile(plan.preflightFile,'utf8')).replace(/^\uFEFF/,''))
 await verifyNative({cdp,plan,root,preflight,snapshot,api,report,managerPid:manager.pid,shells,save})
}catch(error){report.result='failed';report.error={message:error.message,stack:error.stack};process.exitCode=1}
finally{
 report.finishedAtUtc=new Date().toISOString();report.cleanup='Detached exact testhelpers and authorizedAdministratorclient retained; no globalkill/configreseed/secondformalclientlaunch.'
 await save('formal-result.json',report);cdp?.close();service?.unref();manager?.unref();console.log(JSON.stringify({result:report.result,error:report.error?.message,output}))
}
