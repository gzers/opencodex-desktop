import assert from 'node:assert/strict'
import {readFile,writeFile,mkdir,open} from 'node:fs/promises'
import {spawn,execFileSync} from 'node:child_process'
import {join,resolve,win32} from 'node:path'
import {fileURLToPath} from 'node:url'
import {createHash} from 'node:crypto'
import {connect} from './cdp-session.mjs'
import {observePublicRestart} from './formal-restart-observation.mjs'
import {waitForNativeReady} from './formal-native-ready.mjs'
import {assertModelSandbox} from '../windows-model-ui.mjs'

const [planFile,probeFile]=process.argv.slice(2),plan=JSON.parse(await readFile(planFile,'utf8'))
assert.equal(plan.candidateKind,'local-runtime-source-candidate-not-published2.84.0')
const root=resolve(plan.runRoot);assert.equal(root,resolve('D:/Users/Administrator/WorkSpace/Test/post021-formal-20261105-1518'))
const ps=join(process.env.SystemRoot??'C:/Windows','System32/WindowsPowerShell/v1.0/powershell.exe')
const owner=execFileSync(ps,['-NoProfile','-Command','[Security.Principal.WindowsIdentity]::GetCurrent().Name'],{encoding:'utf8',windowsHide:true,timeout:15000}).trim()
assert.equal(owner.toLowerCase(),plan.expectedOwner.toLowerCase(),'Administrator login required')
const probe=JSON.parse(await readFile(probeFile,'utf8'))
assert.equal(probe.result,'default-process-probe-observed');assert.equal(probe.candidateKind,plan.candidateKind)
assert.equal(probe.adapterSha256,plan.candidateAdapterSha256);assert(probe.defaultList?.length)
assert.equal(probe.calls.find(c=>c.phase==='default-list').timeout,10000)
assert.equal(createHash('sha256').update(await readFile(join(plan.officialPackage,'src/codex/desktop-app/windows.ts'))).digest('hex'),plan.candidateAdapterSha256)
const output=join(root,'owner-probe-candidate-restart');await mkdir(output,{recursive:false})
const save=(name,value)=>writeFile(join(output,name),JSON.stringify(value,null,2)+'\n')
const snapshot=async name=>{
 const value=JSON.parse(execFileSync(ps,['-NoProfile','-NonInteractive','-File',fileURLToPath(new URL('./formal-msix-snapshot.ps1',import.meta.url))],{encoding:'utf8',windowsHide:true,timeout:90000}))
 await save(name+'.json',value);assert.equal(value.owner.toLowerCase(),owner.toLowerCase());assert.equal(value.sessionId,1);assert.equal(value.unknownOwnerCount,0);return value
}
const report={result:'running',acceptance:false,owner,candidateKind:plan.candidateKind,startedAtUtc:new Date().toISOString(),candidateAdapterSha256:plan.candidateAdapterSha256}
await save('candidate-started.json',report)
const sleep=ms=>new Promise(r=>setTimeout(r,ms))
let service,cdp
try{
 const before=await snapshot('process-before-service-switch')
 const old=JSON.parse(await readFile(join(root,'bootstrap-recovery','formal-result.json'),'utf8'))
 const original=JSON.parse(await readFile(join(root,'formal-result.json'),'utf8'))
 assert(original.formalBefore.every(s=>before.own.some(p=>p.pid===s.pid&&p.createdAtUtcTicks===s.createdAtUtcTicks)),'Original authorized client generation changed')
 const oldHealth=await(await fetch('http://127.0.0.1:'+plan.servicePort+'/healthz',{signal:AbortSignal.timeout(10000)})).json()
 assert.equal(oldHealth.pid,old.servicePid)
 const oldServicePid=old.servicePid;assert(Number.isSafeInteger(oldServicePid)&&oldServicePid>0)
 assert.equal(plan.serviceSwitchIdentity?.pid,oldServicePid)
 assert.equal(plan.serviceSwitchIdentity?.createdAtUtcTicks,'639294604342922550','Exact previously observed test-service generation required')
 // Only retire this exact owned test service after owner/session/image/create
 // and command-line admission. Never use a package-wide or fixed-PID kill.
 const stopScript=[
  "$ErrorActionPreference='Stop'",
  "$p=Get-CimInstance Win32_Process -Filter 'ProcessId="+oldServicePid+"'",
  "if(!$p -or $p.SessionId -ne 1){throw 'Owned service absent or session changed'}",
  "if([string]$p.CreationDate.ToUniversalTime().Ticks -cne '"+plan.serviceSwitchIdentity.createdAtUtcTicks+"'){throw 'Recorded service generation changed'}",
  "$o=Invoke-CimMethod -InputObject $p -MethodName GetOwner",
  "if($o.ReturnValue -ne 0 -or ($o.Domain+'\\'+$o.User) -ine 'EZIO-MINI\\Administrator'){throw 'Service owner changed'}",
  "$expectedBun='"+plan.bunExe.replaceAll("'","''")+"'",
  "if($p.ExecutablePath -ine $expectedBun -or $p.CommandLine -notlike '*formal-msix-service.ts*' -or $p.CommandLine -notlike '*post021-formal-20261105-1518*'){throw 'Test service path/command changed'}",
  "$handle=Get-Process -Id $p.ProcessId;[void]$handle.Handle",
  "$delta=$handle.StartTime.ToUniversalTime().Ticks-$p.CreationDate.ToUniversalTime().Ticks",
  "if($handle.Path -ine $expectedBun -or $delta -lt 0 -or $delta -gt 9){throw 'Service native generation changed'}",
  "$admitted=[ordered]@{pid=$p.ProcessId;path=$p.ExecutablePath;createdAtUtcTicks=[string]$p.CreationDate.ToUniversalTime().Ticks;session=$p.SessionId;owner=$o.Domain+'\\'+$o.User;nativeTickDelta=$delta}",
  "Stop-Process -InputObject $handle;if(!$handle.WaitForExit(10000)){throw 'Owned test service did not stop'}",
  "$admitted|ConvertTo-Json -Compress"
 ].join('\n')
 const stopped=JSON.parse(execFileSync(ps,['-NoProfile','-NonInteractive','-Command',stopScript],{encoding:'utf8',windowsHide:true,timeout:20000}))
 await save('owned-test-service-stop.json',stopped)
 const env={...process.env,HOME:'C:/Users/Administrator',USERPROFILE:'C:/Users/Administrator',CODEX_HOME:'C:/Users/Administrator/.codex',OPENCODEX_HOME:join(root,'sandbox','opencodex-home'),CODEX_CLI_PATH:plan.codexExe,DO_NOT_TRACK:'1',PATH:join(root,'tools')+';'+win32.dirname(plan.codexExe)+';C:/Windows/System32;C:/Windows'}
 for(const key of ['OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY','OPENCODEX_WINDOWS_SMOKE_CDP_PORT'])delete env[key]
 const so=await open(join(output,'service.stdout.log'),'wx'),se=await open(join(output,'service.stderr.log'),'wx')
 service=spawn(plan.bunExe,[fileURLToPath(new URL('./formal-msix-service.ts',import.meta.url)),resolve(planFile),root,output],{cwd:root,env,detached:true,windowsHide:true,stdio:['ignore',so.fd,se.fd]})
 await so.close();await se.close();assert(service.pid);report.servicePid=service.pid
 let ready;for(let n=0;n<180;n++){
  try{ready=JSON.parse(await readFile(join(output,'service-ready.json'),'utf8'));break}catch(error){if(error.code!=='ENOENT')throw error}
  assert(service.exitCode===null,'Candidate test service exited');await sleep(500)
 }
 assert.equal(ready?.pid,service.pid);assert.equal(ready.candidateKind,plan.candidateKind);assert.equal(ready.restartSourceSha256,plan.candidateAdapterSha256)
 cdp=await connect('http://127.0.0.1:'+plan.cdpPort)
 const evaluate=(fn,...args)=>cdp.evaluate('('+fn.toString()+')('+args.map(v=>JSON.stringify(v)).join(',')+')')
 await waitForNativeReady(evaluate)
 assertModelSandbox({about:await cdp.invoke('app_about'),dataRoot:await cdp.invoke('get_data_root_config')},join(root,'sandbox'))
 const saved=await cdp.invoke('model_file_snapshot')
 assert.equal(saved.policy.channels.find(c=>c.connection.providerId==='Post021AdminFixture').defaults.contextWindow.value,65547)
 const state=await evaluate(observePublicRestart)
 assert.equal(state.state,'failed');assert(state.error.includes('官方无法读取 Codex 桌面进程'),'Unexpected error; do not retry a different operation')
 const shot=async name=>{const value=await cdp.call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,name+'.png'),Buffer.from(value.data,'base64'))}
 const finalBefore=await snapshot('process-before-public-retry')
 const oldShells=finalBefore.own.filter(p=>win32.basename(p.path??'').toLowerCase()==='chatgpt.exe')
 assert(oldShells.length&&oldShells.every(p=>p.sessionId===1))
 await shot('before-candidate-public-retry')
 report.publicRetryAtUtc=new Date().toISOString();await save('public-retry-started.json',report)
 await evaluate(()=>{const button=[...document.querySelectorAll('dialog[open] button')].find(b=>b.textContent.trim()==='重试重启'&&!b.disabled);if(!button)throw Error('Public retry action absent');button.click()})
 let outcome;const deadline=Date.now()+150000
 while(Date.now()<deadline){outcome=await evaluate(observePublicRestart);if(outcome.state!=='pending')break;await sleep(250)}
 await save('public-restart-result.json',outcome??{state:'deadline'})
 await shot('after-candidate-public-retry')
 const after=await snapshot('process-after-public-retry')
 const survived=oldShells.filter(s=>after.own.some(p=>p.pid===s.pid&&p.createdAtUtcTicks===s.createdAtUtcTicks))
 const replaced=after.own.filter(p=>win32.basename(p.path??'').toLowerCase()==='chatgpt.exe'&&!oldShells.some(s=>s.pid===p.pid&&s.createdAtUtcTicks===p.createdAtUtcTicks))
 const otherBefore=finalBefore.other.filter(p=>['chatgpt.exe','opencodex-desktop.exe'].includes(win32.basename(p.path??'').toLowerCase()))
 const otherChanged=otherBefore.filter(s=>!after.other.some(p=>p.pid===s.pid&&p.createdAtUtcTicks===s.createdAtUtcTicks))
 const policyAfter=await cdp.invoke('model_file_snapshot'),config=await readFile('C:/Users/Administrator/.codex/config.toml','utf8')
 await save('candidate-readback.json',{ui:outcome,oldSurvivors:survived,newShells:replaced,otherOwnerDesktopChanges:otherChanged,policyBefore:saved.policyDigest,policyAfter:policyAfter.policyDigest,contextWindow:policyAfter.policy.channels.find(c=>c.connection.providerId==='Post021AdminFixture').defaults.contextWindow.value,codexConfigSha256:createHash('sha256').update(config).digest('hex'),codexConfigBytes:Buffer.byteLength(config),containsExpectedLoopbackRoute:config.includes('127.0.0.1:'+plan.servicePort)})
 assert.equal(outcome?.state,'ui-confirmed','Candidate default restart not confirmed by real UI')
 assert.equal(survived.length,0);assert(replaced.length&&replaced.every(p=>p.sessionId===1&&win32.resolve(p.path).toLowerCase()===win32.resolve(original.formalBefore[0].path).toLowerCase()))
 assert.equal(otherChanged.length,0);assert.equal(policyAfter.policy.channels.find(c=>c.connection.providerId==='Post021AdminFixture').defaults.contextWindow.value,65547)
 assert(config.includes('127.0.0.1:'+plan.servicePort))
 report.result='passed-local-runtime-candidate-default-restart';report.acceptance=true;report.limit='Local source candidate verified; original published2.84.0 remains failed. No claim of integrated/released runtime or model-generation behavior.'
}catch(error){report.result='failed';report.error={message:error.message,stack:error.stack};process.exitCode=1}
finally{report.finishedAtUtc=new Date().toISOString();report.cleanup='Exact testservice switch only; authorized testclient and candidate service retained for review.';await save('result.json',report);cdp?.close();service?.unref();console.log(JSON.stringify({result:report.result,error:report.error?.message,output}))}
