import assert from 'node:assert/strict'
import {readFile,writeFile,mkdir,open} from 'node:fs/promises'
import {spawn,execFileSync} from 'node:child_process'
import {join,resolve,win32} from 'node:path'
import {fileURLToPath,pathToFileURL} from 'node:url'
import {createHash} from 'node:crypto'
import {connect} from './cdp-session.mjs'
import {observePublicRestart} from './formal-restart-observation.mjs'
import {waitForNativeReady} from './formal-native-ready.mjs'
import {assertModelSandbox} from '../windows-model-ui.mjs'

const [planFile]=process.argv.slice(2),plan=JSON.parse(await readFile(planFile,'utf8'))
const root=resolve(plan.runRoot)
assert.equal(root,resolve('D:/Users/Administrator/WorkSpace/Test/post021-formal-20261105-1518'))
assert.equal(plan.candidateKind,'local-installable-candidate-not-upstream-published')
const ps=join(process.env.SystemRoot??'C:/Windows','System32/WindowsPowerShell/v1.0/powershell.exe')
const psFile=(name,args=[])=>execFileSync(ps,['-NoProfile','-NonInteractive','-File',fileURLToPath(new URL(name,import.meta.url)),...args],{encoding:'utf8',windowsHide:true,timeout:90000})
const owner=execFileSync(ps,['-NoProfile','-Command','[Security.Principal.WindowsIdentity]::GetCurrent().Name'],{encoding:'utf8',windowsHide:true,timeout:15000}).trim()
assert.equal(owner.toLowerCase(),'ezio-mini\\administrator','Use Administrator login session, not an elevated15119 terminal')
const sha=b=>createHash('sha256').update(b).digest('hex')
const artifact=JSON.parse(await readFile(plan.artifactManifest,'utf8'))
assert.equal(artifact.kind,plan.candidateKind);assert.equal(artifact.version,plan.expectedRuntimeVersion)
assert.equal(sha(await readFile(plan.artifactArchive)),artifact.sha256)
const packageJson=JSON.parse(await readFile(join(plan.officialPackage,'package.json'),'utf8'))
assert.equal(packageJson.version,artifact.version);assert.equal(packageJson.post021Candidate.kind,artifact.kind)
for(const row of artifact.files){const data=await readFile(join(plan.officialPackage,row.path));assert.equal(data.length,row.bytes);assert.equal(sha(data),row.sha256,row.path)}
assert.equal(sha(await readFile(plan.managerExe)),plan.managerSha256)
const output=join(root,'installed-runtime-validation-20261105');await mkdir(output,{recursive:false})
const save=(name,v)=>writeFile(join(output,name+'.json'),JSON.stringify(v,null,2)+'\n')
const sleep=ms=>new Promise(r=>setTimeout(r,ms))
const report={result:'running',acceptance:false,owner,startedAtUtc:new Date().toISOString(),runtimeVersion:packageJson.version,tarballSha256:artifact.sha256,upstreamPatchSha256:artifact.reviewedPatchSha256,checks:[]}
await save('started',report)
const env={...process.env,HOME:'C:/Users/Administrator',USERPROFILE:'C:/Users/Administrator',CODEX_HOME:'C:/Users/Administrator/.codex',OPENCODEX_HOME:join(root,'sandbox','opencodex-home'),CODEX_CLI_PATH:plan.codexExe,DO_NOT_TRACK:'1',PATH:join(root,'tools')+';'+win32.dirname(plan.codexExe)+';C:/Windows/System32;C:/Windows'}
for(const key of ['OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY','OPENCODEX_WINDOWS_SMOKE_CDP_PORT'])delete env[key]
const snapshot=async name=>{const v=JSON.parse(psFile('./formal-msix-snapshot.ps1'));await save(name,v);assert.equal(v.owner.toLowerCase(),owner.toLowerCase());assert.equal(v.sessionId,1);assert.equal(v.unknownOwnerCount,0);return v}
const shells=s=>s.own.filter(p=>win32.basename(p.path??'').toLowerCase()==='chatgpt.exe')
let cdp,service
async function stopOwned(identity,name){
 const path=join(output,name+'-identity.json');await writeFile(path,JSON.stringify(identity))
 const value=JSON.parse(psFile('./formal-owned-service-stop.ps1',['-IdentityFile',path]));await save(name+'-stop',value)
}
async function startService(name){
 const attempt=join(output,name);await mkdir(attempt)
 const so=await open(join(attempt,'stdout.log'),'wx'),se=await open(join(attempt,'stderr.log'),'wx')
 service=spawn(plan.bunExe,[fileURLToPath(new URL('./formal-msix-service.ts',import.meta.url)),resolve(planFile),root,attempt],{cwd:root,env,detached:true,windowsHide:true,stdio:['ignore',so.fd,se.fd]})
 await so.close();await se.close();assert(service.pid)
 let ready;for(let n=0;n<180;n++){try{ready=JSON.parse(await readFile(join(attempt,'service-ready.json'),'utf8'));break}catch(e){if(e.code!=='ENOENT')throw e};assert.equal(service.exitCode,null);await sleep(500)}
 assert.equal(ready?.pid,service.pid);assert.equal(ready.package,plan.expectedRuntimeVersion)
 const identity=JSON.parse(execFileSync(ps,['-NoProfile','-Command',"$p=Get-CimInstance Win32_Process -Filter 'ProcessId="+service.pid+"';[ordered]@{pid=$p.ProcessId;path=$p.ExecutablePath;createdAtUtcTicks=[string]$p.CreationDate.ToUniversalTime().Ticks;sessionId=$p.SessionId;owner=[Security.Principal.WindowsIdentity]::GetCurrent().Name}|ConvertTo-Json -Compress"],{encoding:'utf8',windowsHide:true,timeout:15000}))
 assert.equal(identity.sessionId,1);assert.equal(win32.resolve(identity.path).toLowerCase(),win32.resolve(plan.bunExe).toLowerCase())
 await save(name+'-identity',identity);service.unref();return identity
}
try{
 const initial=await snapshot('process-initial')
 assert(initial.own.some(p=>p.pid===plan.currentFormalIdentity.pid&&p.createdAtUtcTicks===plan.currentFormalIdentity.createdAtUtcTicks),'Formal target generation changed')
 assert(initial.own.some(p=>p.pid===30804&&p.createdAtUtcTicks==='639294604444368260'&&win32.resolve(p.path).toLowerCase()===win32.resolve(plan.managerExe).toLowerCase()),'Manager generation changed')
 const dailyPath='C:/Users/15119/.codex/config.toml',dailyBefore=sha(await readFile(dailyPath))
 const criticalOther=initial.other.filter(p=>['chatgpt.exe','codex.exe','opencodex-desktop.exe'].includes(win32.basename(p.path??'').toLowerCase()))
 const boundary=async(s,name)=>{
  const changes=criticalOther.filter(p=>!s.other.some(v=>v.pid===p.pid&&v.createdAtUtcTicks===p.createdAtUtcTicks))
  const value={otherOwnerProcessChanges:changes,dailyConfigBefore:dailyBefore,dailyConfigAfter:sha(await readFile(dailyPath))};await save(name,value)
  assert.equal(changes.length,0);assert.equal(value.dailyConfigAfter,dailyBefore)
 }
 cdp=await connect('http://127.0.0.1:'+plan.cdpPort)
 const evaluate=(fn,...args)=>cdp.evaluate('('+fn.toString()+')('+args.map(v=>JSON.stringify(v)).join(',')+')')
 await waitForNativeReady(evaluate)
 assertModelSandbox({about:await cdp.invoke('app_about'),dataRoot:await cdp.invoke('get_data_root_config')},join(root,'sandbox'))
 const wait=async(fn,label,timeout=30000)=>{const end=Date.now()+timeout;while(Date.now()<end){const v=await evaluate(fn);if(v)return v;await sleep(200)}throw Error('Native UI deadline: '+label)}
 const shot=async name=>{const v=await cdp.call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,name+'.png'),Buffer.from(v.data,'base64'))}
 const health=await(await fetch('http://127.0.0.1:'+plan.servicePort+'/healthz',{signal:AbortSignal.timeout(5000)})).json()
 assert.equal(health.pid,plan.serviceSwitchIdentity.pid)
 await stopOwned(plan.serviceSwitchIdentity,'previous-test-service')
 let serviceIdentity=await startService('installed-candidate-service')
 const desktop=await import(pathToFileURL(join(plan.officialPackage,'src/codex/desktop-app/windows.ts')).href)
 const processes=await import(pathToFileURL(join(plan.officialPackage,'src/codex/app-server-processes.ts')).href)
 const install=desktop.windowsDesktopAppAdapter.discover(desktop.windowsDefaultExec);assert(install)
 const probeStarted=performance.now(),nativeRows=desktop.windowsDesktopAppAdapter.listProcesses(desktop.windowsDefaultExec,install);assert(nativeRows?.length)
 const appStarted=performance.now(),appRows=processes.listWindowsSnapshots()
 await save('installed-native-queries',{desktopElapsedMs:appStarted-probeStarted,appserverElapsedMs:performance.now()-appStarted,desktopCount:nativeRows.length,appServerPids:appRows.filter(p=>processes.isCodexAppServerCommandLine(p.commandLine,p.executable)).map(p=>p.pid),desktopTimeoutMs:10000,appserverTimeoutMs:8000})
 await evaluate(async()=>{const p=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia;p._s.get('routes').go('models');await p._s.get('model-config').load();p._s.get('model-config').view.tab='channels'})
 const oldProvider='Post021AdminFixture',newProvider='Post021AdminRenamed'
 await wait(()=>[...document.querySelectorAll('tr[data-channel]')].some(r=>r.textContent.includes('Post021AdminFixture')),'fixture channel')
 const beforeSave=await cdp.invoke('model_file_snapshot')
 await evaluate(()=>{const row=[...document.querySelectorAll('tr[data-channel]')].find(r=>r.textContent.includes('Post021AdminFixture'));[...row.querySelectorAll('button')].find(b=>b.textContent.trim()==='编辑').click()})
 await wait(()=>!!document.querySelector('[aria-label="保存渠道草稿"]'),'channel editor')
 await evaluate(()=>{const m=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('model-config');m.channelDraft.channels.find(c=>c.connection.providerId==='Post021AdminFixture').defaults.contextWindow={mode:'custom',value:65558}})
 await wait(()=>{const b=document.querySelector('[aria-label="保存渠道草稿"]');return b&&!b.disabled},'Save enabled')
 await evaluate(()=>document.querySelector('[aria-label="保存渠道草稿"]').click())
 await wait(()=>[...document.querySelectorAll('button')].some(b=>b.textContent.trim()==='立即重启 Codex'),'real saved notice')
 const saved=await cdp.invoke('model_file_snapshot');assert.notEqual(saved.policyDigest,beforeSave.policyDigest)
 assert.equal(saved.policy.channels.find(c=>c.connection.providerId===oldProvider).defaults.contextWindow.value,65558)
 await save('real-save-readback',{beforeDigest:beforeSave.policyDigest,afterDigest:saved.policyDigest,contextWindow:65558,source:'public Save; no artificial notice'})
 await shot('01-real-save')
 await evaluate(()=>[...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='立即重启 Codex').click())
 await wait(()=>[...document.querySelectorAll('dialog[open] button')].some(b=>b.textContent.trim()==='确认重启'),'real confirm')
 await stopOwned(serviceIdentity,'controlled-service-unavailable')
 await evaluate(()=>[...document.querySelectorAll('dialog[open] button')].find(b=>b.textContent.trim()==='确认重启').click())
 const failed=await wait(()=>{const d=[...document.querySelectorAll('dialog[open]')].find(d=>d.querySelector('[role="alert"]'));return d?.querySelector('[role="alert"]')?.textContent.trim()},'controlled service failure',60000)
 assert(!failed.includes('manage backups')&&!failed.includes('os error 2'));assert(/服务|运行|连接/.test(failed))
 await save('controlled-failure',{error:failed,policyDigest:(await cdp.invoke('model_file_snapshot')).policyDigest});await shot('02-real-failure-retry')
 const afterFailure=await snapshot('process-after-controlled-failure')
 assert(shells(initial).every(p=>shells(afterFailure).some(v=>v.pid===p.pid&&v.createdAtUtcTicks===p.createdAtUtcTicks)),'Failure stopped a client')
 assert.equal((await cdp.invoke('model_file_snapshot')).policyDigest,saved.policyDigest);await boundary(afterFailure,'boundary-after-failure')
 serviceIdentity=await startService('restored-installed-service')
 async function restartThroughPublic(before,name,buttonLabel){
  await evaluate(label=>{const button=[...document.querySelectorAll('dialog[open] button')].find(b=>b.textContent.trim()===label&&!b.disabled);if(!button)throw Error('Public restart button unavailable');button.click()},buttonLabel)
  const end=Date.now()+150000;let state
  while(Date.now()<end){state=await evaluate(observePublicRestart);if(state.state!=='pending')break;await sleep(200)}
  await save(name+'-public-result',state??{});await shot(name+'-native')
  assert.equal(state?.state,'ui-confirmed','Complete public restart not confirmed')
  const after=await snapshot(name+'-process-after'),old=shells(before),next=shells(after)
  assert(old.every(p=>!next.some(v=>v.pid===p.pid&&v.createdAtUtcTicks===p.createdAtUtcTicks)))
  assert(next.length&&next.every(p=>p.sessionId===1&&win32.resolve(p.path).toLowerCase()===win32.resolve(old[0].path).toLowerCase()))
  await boundary(after,name+'-boundary');return after
 }
 const afterRetry=await restartThroughPublic(afterFailure,'03-failure-retry','重试重启')
 const retryConfig=await readFile('C:/Users/Administrator/.codex/config.toml','utf8')
 assert(retryConfig.includes(oldProvider+'/raw-model')&&retryConfig.includes('127.0.0.1:'+plan.servicePort))
 assert.equal((await cdp.invoke('model_file_snapshot')).policy.channels.find(c=>c.connection.providerId===oldProvider).defaults.contextWindow.value,65558)
 await save('retry-readback',{contextWindow:65558,codexConfigSha256:sha(retryConfig),localRoute:true})
 await evaluate(async()=>{const m=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('model-config');await m.load()})
 await wait(()=>[...document.querySelectorAll('tr[data-channel]')].some(r=>r.textContent.includes('Post021AdminFixture')),'channel after retry')
 await evaluate(()=>{const row=[...document.querySelectorAll('tr[data-channel]')].find(r=>r.textContent.includes('Post021AdminFixture'));[...row.querySelectorAll('button')].find(b=>b.textContent.trim()==='编辑').click()})
 await wait(()=>!!document.querySelector('#channel-config input'),'rename editor')
 await evaluate(()=>{const input=document.querySelector('#channel-config input');Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(input,'Post021AdminRenamed');input.dispatchEvent(new Event('input',{bubbles:true}))})
 await wait(()=>{const b=document.querySelector('[aria-label="保存渠道草稿"]');return b&&!b.disabled},'rename Save enabled')
 const renameBefore=await cdp.invoke('model_file_snapshot')
 await shot('04-before-rename-save')
 await evaluate(()=>document.querySelector('[aria-label="保存渠道草稿"]').click())
 await wait(()=>[...document.querySelectorAll('button')].some(b=>b.textContent.trim()==='立即重启 Codex'),'rename actual saved notice')
 const renameSaved=await cdp.invoke('model_file_snapshot'),renamed=renameSaved.policy.channels.find(c=>c.connection.providerId===newProvider)
 assert(renamed&&renamed.id===renameBefore.policy.channels.find(c=>c.connection.providerId===oldProvider).id)
 assert(!renameSaved.policy.channels.some(c=>c.connection.providerId===oldProvider));assert.equal(renamed.defaults.contextWindow.value,65558)
 const originalChannel=renameBefore.policy.channels.find(c=>c.connection.providerId===oldProvider)
 assert.deepEqual(renamed.selection,originalChannel.selection)
 assert.deepEqual(renamed.modelOrder,originalChannel.modelOrder)
 assert.deepEqual(renamed.discovered,originalChannel.discovered)
 await evaluate(()=>[...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='立即重启 Codex').click())
 await wait(()=>[...document.querySelectorAll('dialog[open] button')].some(b=>b.textContent.trim()==='确认重启'),'rename restart confirm')
 const afterRename=await restartThroughPublic(afterRetry,'05-rename-default-restart','确认重启')
 const config=await readFile('C:/Users/Administrator/.codex/config.toml','utf8'),policy=JSON.parse(await readFile(join(env.OPENCODEX_HOME,'config.json'),'utf8'))
 assert(config.includes(newProvider+'/raw-model')&&!config.includes(oldProvider+'/raw-model'))
 assert.deepEqual(Object.keys(policy.providers),[newProvider]);assert.equal(policy.defaultProvider,newProvider)
 assert.equal(policy.providers[newProvider].contextWindow,65558);assert(config.includes('127.0.0.1:'+plan.servicePort))
 await save('rename-readback',{oldProvider,newProvider,stableChannelId:renamed.id,contextWindow:65558,codexConfigSha256:sha(config),configBytes:Buffer.byteLength(config),oldReferenceAbsent:true,localRoute:true,finalDesktopGenerations:shells(afterRename),activeTestServiceIdentity:serviceIdentity})
 report.result='passed-installed-local-candidate-save-retry-rename';report.acceptance=true
 report.checks=['installed archive2522files exact','default desktop/appserver native queries bounded','actual save65558 retained through controlled offline failure','same real notice public retry restarted formal MSIX','public rename Save rewrote current client model reference and default provider','rename default restart produced new official process generations','other owner desktop/appserver generations and15119config unchanged']
}catch(error){report.result='failed';report.error={message:error.message,stack:error.stack};process.exitCode=1}
finally{report.finishedAtUtc=new Date().toISOString();report.limit='Installable local prerelease candidate only; no upstream release, production source change, model inference or real sync. Authorized test processes retained.';await save('result',report);cdp?.close();service?.unref();console.log(JSON.stringify({result:report.result,error:report.error?.message,output}))}
