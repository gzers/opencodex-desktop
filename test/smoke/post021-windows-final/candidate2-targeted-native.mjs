import assert from 'node:assert/strict'
import {readFile,writeFile,mkdir,open,copyFile,access} from 'node:fs/promises'
import {spawn,execFileSync} from 'node:child_process'
import {join,resolve,win32} from 'node:path'
import {fileURLToPath} from 'node:url'
import {createHash} from 'node:crypto'
import {watch} from 'node:fs'
import {connect} from './cdp-session.mjs'
import {waitForNativeReady} from './formal-native-ready.mjs'
import {observePublicRestart} from './formal-restart-observation.mjs'
import {assertModelSandbox} from '../windows-model-ui.mjs'
const [planFile]=process.argv.slice(2),plan=JSON.parse(await readFile(planFile,'utf8')),root=resolve(plan.runRoot)
assert.equal(root,resolve('D:/Users/Administrator/WorkSpace/Test/post021-candidate2-targeted-20261105'))
const ps=join(process.env.SystemRoot??'C:/Windows','System32/WindowsPowerShell/v1.0/powershell.exe')
const psCall=script=>execFileSync(ps,['-NoProfile','-NonInteractive','-Command',script],{encoding:'utf8',windowsHide:true,timeout:20000})
const account=JSON.parse(psCall('[ordered]@{owner=[Security.Principal.WindowsIdentity]::GetCurrent().Name;sid=[Security.Principal.WindowsIdentity]::GetCurrent().User.Value;sessionId=(Get-Process -Id $PID).SessionId}|ConvertTo-Json -Compress'))
assert.equal(account.owner.toLowerCase(),'ezio-mini\\administrator');assert.equal(account.sessionId,1)
const sha=b=>createHash('sha256').update(b).digest('hex'),sleep=ms=>new Promise(r=>setTimeout(r,ms))
const manifest=JSON.parse(await readFile(plan.artifactManifest,'utf8')),candidate=JSON.parse(await readFile(plan.managerMetadata,'utf8'))
assert.equal(manifest.version,'2.97.0-post021.ownerquery.2');assert.equal(manifest.sha256,'bba03ba0cf02c7bcd1895f63c670cb4a2f4b5272657145e45701b20d5e0f1957')
assert.equal(sha(await readFile(plan.artifactArchive)),manifest.sha256);assert.equal(sha(await readFile(plan.managerExe)),candidate.sha256)
assert.equal(candidate.appsTree,plan.managerAppsTree);assert.equal(candidate.requestedHead,'ca46bc1f3c35988d206e1a2949c708d4f23aab50')
const output=join(root,'evidence');await mkdir(output,{recursive:false})
const save=(name,v)=>writeFile(join(output,name+'.json'),JSON.stringify(v,null,2)+'\n')
const report={result:'running',acceptance:false,startedAtUtc:new Date().toISOString(),account,manager:candidate,runtimeVersion:manifest.version,tarballSha256:manifest.sha256,checks:[]}
await save('started',report)
const sandbox=join(root,'sandbox'),home=join(sandbox,'home'),ocxHome=join(sandbox,'opencodex-home'),packageRoot=join(sandbox,'runtime/opencodex/node_modules/@bitkyc08/opencodex')
const snapshot=async name=>{const v=JSON.parse(execFileSync(ps,['-NoProfile','-NonInteractive','-File',fileURLToPath(new URL('./formal-msix-snapshot.ps1',import.meta.url))],{encoding:'utf8',windowsHide:true,timeout:90000}));await save(name,v);assert.equal(v.owner.toLowerCase(),account.owner.toLowerCase());assert.equal(v.sessionId,1);assert.equal(v.unknownOwnerCount,0);return v}
const shells=v=>v.own.filter(p=>win32.basename(p.path??'').toLowerCase()==='chatgpt.exe')
const identity=async(pid,path,marker)=>{
 const v=JSON.parse(psCall('$p=Get-CimInstance Win32_Process -Filter \'ProcessId='+pid+'\';[ordered]@{pid=$p.ProcessId;path=$p.ExecutablePath;createdAtUtcTicks=[string]$p.CreationDate.ToUniversalTime().Ticks;sessionId=$p.SessionId}|ConvertTo-Json -Compress'))
 assert.equal(v.sessionId,1);assert.equal(win32.resolve(v.path).toLowerCase(),win32.resolve(path).toLowerCase());return{...v,owner:account.owner,sid:account.sid,sha256:sha(await readFile(path)),commandMarker:marker}
}
const stop=async(v,name)=>{const file=join(output,name+'-admission.json');await writeFile(file,JSON.stringify(v));const r=JSON.parse(execFileSync(ps,['-NoProfile','-NonInteractive','-File',fileURLToPath(new URL('./candidate2-owned-stop.ps1',import.meta.url)),'-IdentityFile',file],{encoding:'utf8',windowsHide:true,timeout:30000}));await save(name+'-stopped',r)}
const env={...process.env,HOME:'C:/Users/Administrator',USERPROFILE:'C:/Users/Administrator',CODEX_HOME:'C:/Users/Administrator/.codex',OPENCODEX_HOME:ocxHome,CODEX_CLI_PATH:plan.codexExe,DO_NOT_TRACK:'1',PATH:join(root,'tools')+';'+win32.dirname(plan.codexExe)+';C:/Windows/System32;C:/Windows'}
for(const k of ['OPENCODEX_SANDBOX','OPENCODEX_SANDBOX_ROOT','OPENCODEX_SANDBOX_BOUNDARY','OPENCODEX_WINDOWS_SMOKE_CDP_PORT'])delete env[k]
let cdp,manager,service,watcher,watchQueue=Promise.resolve();const observations=[]
async function startService(name){
 const folder=join(output,name);await mkdir(folder);const so=await open(join(folder,'stdout.log'),'wx'),se=await open(join(folder,'stderr.log'),'wx')
 service=spawn(plan.bunExe,[fileURLToPath(new URL('./candidate2-service.ts',import.meta.url)),resolve(planFile),folder],{cwd:root,env,detached:true,windowsHide:true,stdio:['ignore',so.fd,se.fd]});await so.close();await se.close();assert(service.pid)
 const deadline=Date.now()+90000;let ready
 while(Date.now()<deadline){try{ready=JSON.parse(await readFile(join(folder,'service-ready.json'),'utf8'));break}catch(e){if(e.code!=='ENOENT')throw e};assert.equal(service.exitCode,null);await sleep(500)}
 assert.equal(ready?.pid,service.pid);assert.equal(ready.version,manifest.version);service.unref();const v=await identity(service.pid,plan.bunExe,'candidate2-service.ts');await save(name+'-identity',v);return v
}
try{
 const initial=await snapshot('process-initial');assert(initial.own.some(p=>p.pid===plan.currentFormal.pid&&p.createdAtUtcTicks===plan.currentFormal.createdAtUtcTicks),'Existing disposable formal generation changed')
 const other=initial.other.filter(p=>['chatgpt.exe','codex.exe','opencodex-desktop.exe'].includes(win32.basename(p.path??'').toLowerCase()))
 const daily=[];for(const path of plan.protectedConfigPaths){const b=await readFile(path);daily.push({path,bytes:b.length,sha256:sha(b)})}await save('protected-configs-before',daily)
 const boundary=async(v,name)=>{const changes=other.filter(p=>!v.other.some(q=>q.pid===p.pid&&q.createdAtUtcTicks===p.createdAtUtcTicks));const hashes=[];for(const row of daily)hashes.push({...row,afterSha256:sha(await readFile(row.path))});await save(name,{otherOwnerChanges:changes,protectedConfigs:hashes});assert.equal(changes.length,0);assert(hashes.every(x=>x.sha256===x.afterSha256))}
 const backup=join(root,'private-backup');await mkdir(backup);await copyFile('C:/Users/Administrator/.codex/config.toml',join(backup,'config.before.toml'))
 const managerEnv={...env,OPENCODEX_SANDBOX:'1',OPENCODEX_SANDBOX_ROOT:sandbox,OPENCODEX_SANDBOX_BOUNDARY:root,OPENCODEX_WINDOWS_SMOKE_CDP_PORT:String(plan.cdpPort),APPDATA:join(root,'manager-profile/Roaming'),LOCALAPPDATA:join(root,'manager-profile/Local'),TEMP:join(root,'temp'),TMP:join(root,'temp')}
 const so=await open(join(output,'manager.stdout.log'),'wx'),se=await open(join(output,'manager.stderr.log'),'wx');manager=spawn(plan.managerExe,[],{cwd:root,env:managerEnv,detached:true,windowsHide:false,stdio:['ignore',so.fd,se.fd]});await so.close();await se.close();report.managerPid=manager.pid
 cdp=await connect('http://127.0.0.1:'+plan.cdpPort)
 const evaluate=(fn,...args)=>cdp.evaluate('('+fn.toString()+')('+args.map(v=>JSON.stringify(v)).join(',')+')')
 await waitForNativeReady(evaluate);assertModelSandbox({about:await cdp.invoke('app_about'),dataRoot:await cdp.invoke('get_data_root_config')},sandbox)
 await save('manager-identity',await identity(manager.pid,plan.managerExe,''))
 const shot=async name=>{const s=await cdp.call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,name+'.png'),Buffer.from(s.data,'base64'))}
 const sourceBefore=await cdp.invoke('runtime_source');assert.equal(sourceBefore.kind,'unresolved');await save('source-before-install',sourceBefore)
 const preview=await cdp.invoke('preview_offline_package',{path:plan.artifactArchive});assert(preview.ok);assert.equal(preview.version,manifest.version);await save('native-offline-preview',preview)
 const install=await cdp.invoke('install_runtime',{request:{source:'offline',offlinePath:plan.artifactArchive,version:manifest.version,allowScripts:false}})
 assert.equal(install.version,manifest.version);assert.equal(install.tarballSha256,manifest.sha256);assert.equal(install.source,'offline')
 const installed=await cdp.invoke('runtime_source');assert.equal(installed.kind,'managed');assert.equal(installed.version,manifest.version);assert.equal(installed.explicitPath,null)
 for(const row of manifest.files){const b=await readFile(join(packageRoot,row.path));assert.equal(b.length,row.bytes);assert.equal(sha(b),row.sha256,row.path)}
 const entryBefore=await readFile(installed.path),pkgBefore=await readFile(join(packageRoot,'package.json'))
 const entryVersion=execFileSync(plan.nodeExe,[join(packageRoot,'bin/ocx.mjs'),'--version'],{encoding:'utf8',timeout:15000,windowsHide:true,env:{...env,PATH:win32.dirname(plan.nodeExe)+';C:/Windows/System32;C:/Windows'}}).trim();assert(entryVersion.includes(manifest.version))
 await save('managed-install-readback',{install,source:installed,entryVersion,installedFilesVerified:manifest.files.length,entrySha256:sha(entryBefore),packageSha256:sha(pkgBefore)});await shot('01-managed-installed')
 let installFailure;try{await cdp.invoke('install_runtime',{request:{source:'offline',offlinePath:plan.invalidArchive,allowScripts:false}})}catch(e){installFailure=e.message}assert(installFailure)
 const preserved=await cdp.invoke('runtime_source');assert.equal(preserved.kind,'managed');assert.equal(preserved.version,manifest.version);assert.equal(preserved.path,installed.path)
 assert((await readFile(preserved.path)).equals(entryBefore));assert((await readFile(join(packageRoot,'package.json'))).equals(pkgBefore))
 for(const row of manifest.files)assert.equal(sha(await readFile(join(packageRoot,row.path))),row.sha256,row.path)
 await save('failed-install-preservation',{error:installFailure,source:preserved,allPackageFilesUnchanged:true,entryUnchanged:true});await shot('02-install-failure-preserved')
 await stop({...plan.previousService,owner:account.owner,sid:account.sid},'old-candidate1-test-service')
 let serviceIdentity=await startService('candidate2-service-initial')
 const token=(await readFile(join(ocxHome,'admin-api-token'),'utf8')).trim()
 const integrationResponse=await fetch('http://127.0.0.1:'+plan.servicePort+'/api/native-integrations/codex',{method:'PUT',headers:{Authorization:'Bearer '+token,'Content-Type':'application/json'},body:JSON.stringify({enabled:true}),signal:AbortSignal.timeout(30000)});assert(integrationResponse.ok)
 const integrations=await(await fetch('http://127.0.0.1:'+plan.servicePort+'/api/native-integrations',{headers:{Authorization:'Bearer '+token},signal:AbortSignal.timeout(30000)})).json();const binding=integrations.clients.find(c=>c.clientId==='codex');assert(binding.installed&&win32.resolve(binding.configPath).toLowerCase()===win32.resolve('C:/Users/Administrator/.codex/config.toml').toLowerCase());await save('native-client-binding',binding)
 const wait=async(fn,label,timeout=30000)=>{const end=Date.now()+timeout;while(Date.now()<end){const v=await evaluate(fn);if(v)return v;await sleep(150)}throw Error('Native UI deadline: '+label)}
 await evaluate(async()=>{const p=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia;p._s.get('routes').go('models');await p._s.get('model-config').load();p._s.get('model-config').view.tab='channels'})
 await wait(()=>[...document.querySelectorAll('tr[data-channel]')].some(r=>r.textContent.includes('Post021AdminRenamed')),'fresh seeded channel')
 const pendingPath=join(sandbox,'manager-state/provider-client-renames.json');await assert.rejects(access(pendingPath),e=>e.code==='ENOENT','Fresh rename must start without pending intent')
 const configPath=join(ocxHome,'config.json'),seed=JSON.parse(await readFile(configPath,'utf8'));assert(!Object.hasOwn(seed.providers,plan.toProvider))
 assert.deepEqual(Object.keys(seed.providers),[plan.fromProvider]);const baselineBefore=seed.modelDiscovery
 const policyBefore=await cdp.invoke('model_file_snapshot')
 await evaluate(()=>{const r=[...document.querySelectorAll('tr[data-channel]')].find(r=>r.textContent.includes('Post021AdminRenamed'));[...r.querySelectorAll('button')].find(b=>b.textContent.trim()==='编辑').click()})
 await wait(()=>!!document.querySelector('#channel-config input'),'fresh rename input')
 await evaluate(()=>{const e=document.querySelector('#channel-config input');if(!e||e.disabled)throw Error('Rename input unavailable');window.__post021RenameInputs=[];e.addEventListener('input',event=>window.__post021RenameInputs.push({trusted:event.isTrusted,value:event.target.value}),{once:true});e.focus()})
 await cdp.call('Input.dispatchKeyEvent',{type:'keyDown',key:'a',code:'KeyA',windowsVirtualKeyCode:65,modifiers:2})
 await cdp.call('Input.dispatchKeyEvent',{type:'keyUp',key:'a',code:'KeyA',windowsVirtualKeyCode:65,modifiers:2})
 await cdp.call('Input.insertText',{text:plan.toProvider})
 const inputEvidence=await evaluate(()=>({value:document.querySelector('#channel-config input')?.value,events:window.__post021RenameInputs}))
 assert.equal(inputEvidence.value,plan.toProvider);assert(inputEvidence.events.length&&inputEvidence.events.every(e=>e.trusted));await save('fresh-rename-trusted-input',inputEvidence)
 await wait(()=>{const b=document.querySelector('[aria-label="保存渠道草稿"]');return b&&!b.disabled},'rename Save enabled')
 await shot('03-fresh-rename-input');await evaluate(()=>document.querySelector('[aria-label="保存渠道草稿"]').click())
 await wait(()=>[...document.querySelectorAll('button')].some(b=>b.textContent.trim()==='立即重启 Codex'),'actual rename saved notice')
 const savedBytes=await readFile(configPath),saved=JSON.parse(savedBytes),savedPolicy=await cdp.invoke('model_file_snapshot'),renamed=savedPolicy.policy.channels.find(c=>c.connection.providerId===plan.toProvider)
 assert(renamed&&renamed.id===policyBefore.policy.channels.find(c=>c.connection.providerId===plan.fromProvider).id);assert.deepEqual(Object.keys(saved.providers),[plan.toProvider]);assert.equal(saved.defaultProvider,plan.toProvider)
 assert.deepEqual(saved.modelDiscovery.knownModels[plan.toProvider],baselineBefore.knownModels[plan.fromProvider]);assert.deepEqual(saved.modelDiscovery.recentArrivals?.[plan.toProvider],baselineBefore.recentArrivals?.[plan.fromProvider])
 const pendingBefore=await readFile(pendingPath);await save('fresh-rename-saved',{configSha256:sha(savedBytes),policyDigest:savedPolicy.policyDigest,channelId:renamed.id,from:plan.fromProvider,to:plan.toProvider,knownModelsBefore:baselineBefore.knownModels[plan.fromProvider],knownModelsSaved:saved.modelDiscovery.knownModels[plan.toProvider],recentArrivalsBefore:baselineBefore.recentArrivals?.[plan.fromProvider],recentArrivalsSaved:saved.modelDiscovery.recentArrivals?.[plan.toProvider],pendingSha256:sha(pendingBefore),firstConfirmationCount:0})
 await evaluate(()=>[...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='立即重启 Codex').click());await wait(()=>[...document.querySelectorAll('dialog[open] button')].some(b=>b.textContent.trim()==='确认重启'),'first restart confirm')
 const beforeCancel=await snapshot('process-before-cancel');await evaluate(()=>[...document.querySelectorAll('dialog[open] button')].find(b=>b.textContent.trim()==='取消').click());await wait(()=>!document.querySelector('dialog[open]'),'cancelled confirm')
 assert((await readFile(configPath)).equals(savedBytes));assert((await readFile(pendingPath)).equals(pendingBefore));const afterCancel=await snapshot('process-after-cancel');assert(shells(beforeCancel).every(p=>shells(afterCancel).some(q=>q.pid===p.pid&&q.createdAtUtcTicks===p.createdAtUtcTicks)));await boundary(afterCancel,'cancel-boundary');await save('cancel-readback',{configUnchanged:true,pendingUnchanged:true,noClientStopped:true,firstConfirmationCount:0});await shot('04-cancel-preserves-save')
 await evaluate(()=>[...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='立即重启 Codex').click());await wait(()=>[...document.querySelectorAll('dialog[open] button')].some(b=>b.textContent.trim()==='确认重启'),'first actual confirmation')
 const sample=async trigger=>{try{const b=await readFile(configPath);observations.push({at:new Date().toISOString(),trigger,sha256:sha(b),matchesSaved:b.equals(savedBytes)})}catch(e){observations.push({at:new Date().toISOString(),trigger,error:e.code})}}
 watcher=watch(ocxHome,(_event,name)=>{if(String(name)==='config.json')watchQueue=watchQueue.then(()=>sample('filesystem-event'))});await sample('before-first-confirm')
 async function clickRestart(label,name,before){
  const started=performance.now();await evaluate(label=>{const b=[...document.querySelectorAll('dialog[open] button')].find(b=>b.textContent.trim()===label&&!b.disabled);if(!b)throw Error('Restart action missing');b.click()},label)
  const end=Date.now()+150000;let state;while(Date.now()<end){state=await evaluate(observePublicRestart);if(state.state!=='pending')break;await sleep(100)}await save(name+'-public',{elapsedMs:performance.now()-started,state});await shot(name+'-native');assert.equal(state?.state,'ui-confirmed','Actual complete restart required')
  const after=await snapshot(name+'-process-after'),old=shells(before),next=shells(after);assert(old.every(p=>!next.some(q=>q.pid===p.pid&&q.createdAtUtcTicks===p.createdAtUtcTicks)));assert(next.length&&next.every(p=>p.sessionId===1&&win32.resolve(p.path).toLowerCase()===win32.resolve(old[0].path).toLowerCase()));await boundary(after,name+'-boundary');return after
 }
 const afterFresh=await clickRestart('确认重启','05-fresh-first-confirm',afterCancel);await sample('after-first-confirm');watcher.close();watcher=null;await watchQueue;await save('config-stability-through-first-confirm',{savedSha256:sha(savedBytes),observations});assert(observations.every(v=>v.matchesSaved===true),'Any configuration rewrite during first-confirm is a failure')
 const activeConfig=await readFile('C:/Users/Administrator/.codex/config.toml','utf8'),afterBytes=await readFile(configPath),afterDisk=JSON.parse(afterBytes);assert(afterBytes.equals(savedBytes));assert.deepEqual(afterDisk.modelDiscovery,saved.modelDiscovery);assert(activeConfig.includes(plan.toProvider+'/raw-model')&&!activeConfig.includes(plan.fromProvider+'/raw-model'));await assert.rejects(access(pendingPath),e=>e.code==='ENOENT')
 await save('fresh-first-confirm-readback',{configSha256Before:sha(savedBytes),configSha256After:sha(afterBytes),knownModels:saved.modelDiscovery.knownModels[plan.toProvider],recentArrivals:saved.modelDiscovery.recentArrivals?.[plan.toProvider],codexConfigSha256:sha(activeConfig),newReference:true,oldReferenceAbsent:true,pendingCleared:true,firstConfirmationCount:1,newGenerations:shells(afterFresh)})
 // Separate normal save drives the unavailable-service retry; it cannot replace
 // the entirely fresh rename/first-confirm proof above.
 await evaluate(async()=>{const m=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('model-config');await m.load()});await evaluate(name=>{const r=[...document.querySelectorAll('tr[data-channel]')].find(r=>r.textContent.includes(name));[...r.querySelectorAll('button')].find(b=>b.textContent.trim()==='编辑').click()},plan.toProvider);await wait(()=>!!document.querySelector('[aria-label="保存渠道草稿"]'),'retry fixture editor')
 await evaluate(name=>{const m=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('model-config');m.channelDraft.channels.find(c=>c.connection.providerId===name).defaults.contextWindow={mode:'custom',value:65591}},plan.toProvider);await wait(()=>{const b=document.querySelector('[aria-label="保存渠道草稿"]');return b&&!b.disabled},'retry Save');await evaluate(()=>document.querySelector('[aria-label="保存渠道草稿"]').click());await wait(()=>[...document.querySelectorAll('button')].some(b=>b.textContent.trim()==='立即重启 Codex'),'retry actual saved notice')
 const retryBytes=await readFile(configPath);await evaluate(()=>[...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='立即重启 Codex').click());await wait(()=>[...document.querySelectorAll('dialog[open] button')].some(b=>b.textContent.trim()==='确认重启'),'failure confirmation')
 await stop(serviceIdentity,'candidate2-controlled-stop');await evaluate(()=>[...document.querySelectorAll('dialog[open] button')].find(b=>b.textContent.trim()==='确认重启').click());const failure=await wait(()=>[...document.querySelectorAll('dialog[open] [role="alert"]')].map(e=>e.textContent.trim()).find(Boolean),'unavailable-service failure',60000);assert(/服务|运行|连接/.test(failure));assert(!failure.includes('manage backups'));assert((await readFile(configPath)).equals(retryBytes));await save('restart-failure',{error:failure,savedConfigSha256:sha(retryBytes)});await shot('06-service-failure')
 const afterFailure=await snapshot('process-after-service-failure');assert(shells(afterFresh).every(p=>shells(afterFailure).some(q=>q.pid===p.pid&&q.createdAtUtcTicks===p.createdAtUtcTicks)));await boundary(afterFailure,'failure-boundary')
 serviceIdentity=await startService('candidate2-service-restored');const final=await clickRestart('重试重启','07-service-retry',afterFailure);const finalConfig=JSON.parse(await readFile(configPath,'utf8'));assert.equal(finalConfig.providers[plan.toProvider].contextWindow,65591);const finalClient=await readFile('C:/Users/Administrator/.codex/config.toml','utf8');assert(finalClient.includes(plan.toProvider+'/raw-model')&&!finalClient.includes(plan.fromProvider+'/raw-model'));await assert.rejects(access(pendingPath),e=>e.code==='ENOENT');const finalSource=await cdp.invoke('runtime_source');assert.equal(finalSource.kind,'managed');assert.equal(finalSource.version,manifest.version)
 await save('final-readback',{source:finalSource,providerNames:Object.keys(finalConfig.providers),contextWindow:65591,newReference:true,oldReferenceAbsent:true,pendingCleared:true,codexConfigSha256:sha(finalClient),testServiceIdentity:serviceIdentity,managerIdentity:await identity(manager.pid,plan.managerExe,''),formalGenerations:shells(final)})
 report.result='passed-candidate2-native-managed-fresh-rename-cancel-retry';report.acceptance=true;report.checks=['nativeofflinefullversionmanagedsource','failedinstallpreservesentryallpackagefilesandsource','freshrenamefirstconfirmexactconfigbytesanddiscoverytimestampsstable','newclientreferenceoldabsentpendingcleared','cancelnostopandpendingretained','servicefailureandsamepublicretry','otherowneranddailyconfigunchanged']
}catch(error){report.result='failed';report.error={message:error.message,stack:error.stack};process.exitCode=1}
finally{watcher?.close();await watchQueue;report.finishedAtUtc=new Date().toISOString();report.scope='Isolated candidate only; no production runtime/source adoption, inference, real sync, publicPR, merge or release. Failure evidence and test processes retained.';await save('result',report);cdp?.close();manager?.unref();service?.unref();console.log(JSON.stringify({result:report.result,error:report.error?.message,output}))}
