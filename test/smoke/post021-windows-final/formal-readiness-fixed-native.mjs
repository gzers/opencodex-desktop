import assert from 'node:assert/strict'
import {readFile,writeFile,mkdir,open} from 'node:fs/promises'
import {join,resolve,win32} from 'node:path'
import {fileURLToPath} from 'node:url'
import {execFileSync,spawn} from 'node:child_process'
import {createHash} from 'node:crypto'
import {connect} from './cdp-session.mjs'
import {waitForNativeReady} from './formal-native-ready.mjs'
import {observePublicRestart} from './formal-restart-observation.mjs'
import {assertModelSandbox} from '../windows-model-ui.mjs'
const [planFile]=process.argv.slice(2),plan=JSON.parse(await readFile(planFile,'utf8')),root=resolve(plan.runRoot)
assert.equal(root,resolve('D:/Users/Administrator/WorkSpace/Test/post021-formal-20261105-1518'))
const ps=join(process.env.SystemRoot??'C:/Windows','System32/WindowsPowerShell/v1.0/powershell.exe')
const owner=execFileSync(ps,['-NoProfile','-Command','[Security.Principal.WindowsIdentity]::GetCurrent().Name'],{encoding:'utf8',windowsHide:true,timeout:15000}).trim()
assert.equal(owner.toLowerCase(),'ezio-mini\\administrator')
const sha=b=>createHash('sha256').update(b).digest('hex')
assert.equal(sha(await readFile(plan.readinessManagerExe)),plan.readinessManagerSha256)
assert.equal(plan.readinessBuildCommit,'b82b78e1f8727262e5c364d26f698432614a90d3')
const output=join(root,'readiness-fixed-native-20261105');await mkdir(output,{recursive:false})
const save=(name,value)=>writeFile(join(output,name+'.json'),JSON.stringify(value,null,2)+'\n'),sleep=ms=>new Promise(r=>setTimeout(r,ms))
const report={result:'running',acceptance:false,owner,startedAtUtc:new Date().toISOString(),managerSha256:plan.readinessManagerSha256,managerBuildCommit:plan.readinessBuildCommit,runtimeVersion:plan.expectedRuntimeVersion,limit:'Local MSVC-ABI debug candidate with rebuilt unchanged UI; not release/installer or upstream published runtime'}
await save('started',report)
let cdp,manager
const snapshot=async name=>{const s=JSON.parse(execFileSync(ps,['-NoProfile','-NonInteractive','-File',fileURLToPath(new URL('./formal-msix-snapshot.ps1',import.meta.url))],{encoding:'utf8',windowsHide:true,timeout:90000}));await save(name,s);assert.equal(s.owner.toLowerCase(),owner.toLowerCase());assert.equal(s.sessionId,1);assert.equal(s.unknownOwnerCount,0);return s}
const shells=s=>s.own.filter(p=>win32.basename(p.path??'').toLowerCase()==='chatgpt.exe')
try{
 const before=await snapshot('process-before-manager-swap'),other=before.other.filter(p=>['chatgpt.exe','codex.exe','opencodex-desktop.exe'].includes(win32.basename(p.path??'').toLowerCase()))
 assert(before.own.some(p=>p.pid===108108&&p.createdAtUtcTicks==='639294833341819380'),'Current formal test root changed')
 const original=before.own.find(p=>p.pid===30804);assert(original)
 const marker=join(root,'tools/official-cli-path.txt'),oldMarker=await readFile(marker)
 assert.equal(sha(oldMarker),'1d976717fe5e2872de2724a3b4ed1741d05e4748298990433d405298efc9e8b7','Exact isolated bootstrap launcher required')
 const packageJson=JSON.parse(await readFile(join(plan.officialPackage,'package.json'),'utf8'))
 assert.equal(packageJson.version,plan.expectedRuntimeVersion);assert.equal(packageJson.post021Candidate?.kind,'local-installable-candidate-not-upstream-published')
 const newMarker=Buffer.from(join(plan.officialPackage,'src/cli/index.ts'),'utf16le')
 await writeFile(join(output,'isolated-cli-marker.before.local'),oldMarker,{flag:'wx'})
 await writeFile(marker,newMarker)
 const cliVersion=execFileSync(join(root,'tools/ocx.exe'),['--version'],{encoding:'utf8',windowsHide:true,timeout:15000,env:{...process.env,CODEX_HOME:'C:/Users/Administrator/.codex',OPENCODEX_HOME:join(root,'sandbox/opencodex-home')}}).trim()
 assert(cliVersion.includes(plan.expectedRuntimeVersion))
 await save('isolated-cli-source',{beforeMarkerSha256:sha(oldMarker),afterMarkerSha256:sha(newMarker),beforePackageVersion:'2.84.0',actualVersionOutput:cliVersion,afterPackageVersion:packageJson.version,target:join(plan.officialPackage,'src/cli/index.ts'),scope:'Only this authorized test-root launcher; API service remains69172 andinstalledcandidate unchanged; no daily source/config or original package modified'})
 const identityFile=join(output,'old-manager-identity.json');await writeFile(identityFile,JSON.stringify(original))
 const stopped=JSON.parse(execFileSync(ps,['-NoProfile','-NonInteractive','-File',fileURLToPath(new URL('./formal-owned-manager-stop.ps1',import.meta.url)),'-IdentityFile',identityFile],{encoding:'utf8',windowsHide:true,timeout:30000}));await save('old-manager-stopped',stopped)
 const dailyPath='C:/Users/15119/.codex/config.toml',dailyBefore=sha(await readFile(dailyPath))
 const env={...process.env,HOME:'C:/Users/Administrator',USERPROFILE:'C:/Users/Administrator',CODEX_HOME:'C:/Users/Administrator/.codex',OPENCODEX_HOME:join(root,'sandbox/opencodex-home'),OPENCODEX_SANDBOX:'1',OPENCODEX_SANDBOX_ROOT:join(root,'sandbox'),OPENCODEX_SANDBOX_BOUNDARY:root,OPENCODEX_WINDOWS_SMOKE_CDP_PORT:String(plan.cdpPort),APPDATA:join(root,'manager-profile/Roaming'),LOCALAPPDATA:join(root,'manager-profile/Local'),TEMP:join(root,'temp'),TMP:join(root,'temp'),DO_NOT_TRACK:'1'}
 const so=await open(join(output,'manager.stdout.log'),'wx'),se=await open(join(output,'manager.stderr.log'),'wx')
 manager=spawn(plan.readinessManagerExe,[],{cwd:root,env,detached:true,windowsHide:false,stdio:['ignore',so.fd,se.fd]});await so.close();await se.close();assert(manager.pid);report.managerPid=manager.pid
 const bridgeDeadline=Date.now()+60000
 while(Date.now()<bridgeDeadline){try{cdp=await connect('http://127.0.0.1:'+plan.cdpPort);break}catch{assert.equal(manager.exitCode,null);await sleep(500)}}
 assert(cdp);const evaluate=(fn,...args)=>cdp.evaluate('('+fn.toString()+')('+args.map(v=>JSON.stringify(v)).join(',')+')')
 await waitForNativeReady(evaluate);assertModelSandbox({about:await cdp.invoke('app_about'),dataRoot:await cdp.invoke('get_data_root_config')},join(root,'sandbox'))
 const health=await(await fetch('http://127.0.0.1:'+plan.servicePort+'/healthz',{signal:AbortSignal.timeout(5000)})).json();assert.equal(health.pid,69172);assert.equal(health.version,plan.expectedRuntimeVersion)
 const wait=async(fn,label)=>{const deadline=Date.now()+30000;while(Date.now()<deadline){const v=await evaluate(fn);if(v)return v;await sleep(200)}throw Error('Native deadline: '+label)}
 const shot=async name=>{const v=await cdp.call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,name+'.png'),Buffer.from(v.data,'base64'))}
 await evaluate(async()=>{const p=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia;p._s.get('routes').go('models');await p._s.get('model-config').load();p._s.get('model-config').view.tab='channels'})
 await wait(()=>[...document.querySelectorAll('tr[data-channel]')].some(r=>r.textContent.includes('Post021AdminRenamed')),'saved renamed channel')
 await evaluate(()=>{const r=[...document.querySelectorAll('tr[data-channel]')].find(r=>r.textContent.includes('Post021AdminRenamed'));[...r.querySelectorAll('button')].find(b=>b.textContent.trim()==='编辑').click()})
 await wait(()=>!!document.querySelector('[aria-label="保存渠道草稿"]'),'editor')
 await evaluate(()=>{const m=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('model-config');m.channelDraft.channels.find(c=>c.connection.providerId==='Post021AdminRenamed').defaults.contextWindow={mode:'custom',value:65569}})
 await wait(()=>{const b=document.querySelector('[aria-label="保存渠道草稿"]');return b&&!b.disabled},'save enabled')
 await evaluate(()=>document.querySelector('[aria-label="保存渠道草稿"]').click())
 await wait(()=>[...document.querySelectorAll('button')].some(b=>b.textContent.trim()==='立即重启 Codex'),'actual fresh saved notice')
 const saved=await cdp.invoke('model_file_snapshot');assert.equal(saved.policy.channels.find(c=>c.connection.providerId==='Post021AdminRenamed').defaults.contextWindow.value,65569)
 await shot('01-saved-pending-rename')
 await evaluate(()=>[...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='立即重启 Codex').click())
 await wait(()=>[...document.querySelectorAll('dialog[open] button')].some(b=>b.textContent.trim()==='确认重启'),'restart confirm')
 const beforeRestart=await snapshot('process-before-public-restart'),old=shells(beforeRestart)
 const attempts=[]
 for(let attempt=0;attempt<2;attempt++){
  const label=attempt?'重试重启':'确认重启',t=performance.now()
  await evaluate(label=>{const b=[...document.querySelectorAll('dialog[open] button')].find(b=>b.textContent.trim()===label&&!b.disabled);if(!b)throw Error('Restart action absent');b.click()},label)
  let outcome;const deadline=Date.now()+150000
  while(Date.now()<deadline){outcome=await evaluate(observePublicRestart);if(outcome.state!=='pending')break;await sleep(100)}
  attempts.push({attempt,elapsedMs:performance.now()-t,outcome});await save('public-attempts',attempts);await shot('02-restart-attempt-'+attempt)
  if(outcome?.state==='ui-confirmed')break
  if(!outcome?.error?.includes('同步期间配置文件已变化')||attempt)break
 }
 const after=await snapshot('process-after-public-restart'),next=shells(after)
 const oldSurvivors=old.filter(p=>next.some(v=>v.pid===p.pid&&v.createdAtUtcTicks===p.createdAtUtcTicks)),newGenerations=next.filter(p=>!old.some(v=>v.pid===p.pid&&v.createdAtUtcTicks===p.createdAtUtcTicks))
 const otherChanges=other.filter(p=>!after.other.some(v=>v.pid===p.pid&&v.createdAtUtcTicks===p.createdAtUtcTicks))
 const config=await readFile('C:/Users/Administrator/.codex/config.toml','utf8'),policy=JSON.parse(await readFile(join(root,'sandbox/opencodex-home/config.json'),'utf8')),pending=await readFile(join(root,'sandbox/manager-state/provider-client-renames.json')).catch(e=>{if(e.code==='ENOENT')return null;throw e})
 await save('readback',{attempts,oldSurvivors,newGenerations,otherOwnerChanges:otherChanges,dailyConfigBefore:dailyBefore,dailyConfigAfter:sha(await readFile(dailyPath)),providerNames:Object.keys(policy.providers),defaultProvider:policy.defaultProvider,contextWindow:policy.providers.Post021AdminRenamed?.contextWindow,codexConfigSha256:sha(config),newModelReferencePresent:config.includes('Post021AdminRenamed/raw-model'),oldModelReferenceAbsent:!config.includes('Post021AdminFixture/raw-model'),localRoute:config.includes('127.0.0.1:'+plan.servicePort),pendingRenameBytes:pending?.length??0})
 assert.equal(attempts.at(-1)?.outcome?.state,'ui-confirmed');assert.equal(oldSurvivors.length,0);assert(newGenerations.length)
 assert.equal(otherChanges.length,0);assert.equal(sha(await readFile(dailyPath)),dailyBefore)
 assert(config.includes('Post021AdminRenamed/raw-model')&&!config.includes('Post021AdminFixture/raw-model'));assert(config.includes('127.0.0.1:'+plan.servicePort))
 assert.deepEqual(Object.keys(policy.providers),['Post021AdminRenamed']);assert.equal(policy.defaultProvider,'Post021AdminRenamed');assert.equal(policy.providers.Post021AdminRenamed.contextWindow,65569)
 report.result='passed-fixed-native-installed-runtime-rename-restart';report.acceptance=true
}catch(error){report.result='failed';report.error={message:error.message,stack:error.stack};process.exitCode=1}
finally{report.finishedAtUtc=new Date().toISOString();await save('result',report);cdp?.close();manager?.unref();console.log(JSON.stringify({result:report.result,error:report.error?.message,output}))}
