import assert from 'node:assert/strict'
import {readFile,writeFile,mkdir,copyFile,open,access} from 'node:fs/promises'
import {spawn,execFileSync} from 'node:child_process'
import {join,resolve,relative,isAbsolute,win32} from 'node:path'
import {fileURLToPath} from 'node:url'
import {createHash} from 'node:crypto'
import {connect} from './cdp-session.mjs'
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
 service=spawn(plan.bunExe,[fileURLToPath(new URL('./formal-msix-service.ts',import.meta.url)),resolve(planFile),root],{cwd:root,env,windowsHide:true,stdio:['ignore',serviceOut.fd,serviceErr.fd]})
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
 manager=spawn(plan.managerExe,[],{cwd:root,env:managerEnv,windowsHide:false,stdio:['ignore',managerOut.fd,managerErr.fd]})
 await managerOut.close();await managerErr.close();assert(manager.pid);report.managerPid=manager.pid
 for(let n=0;n<100;n++){try{cdp=await connect('http://127.0.0.1:'+plan.cdpPort);break}catch{assert(manager.exitCode===null,'Manager exited');await sleep(500)}}
 assert(cdp,'Native manager CDP unavailable')
 const evaluate=(fn,...args)=>cdp.evaluate('('+fn.toString()+')('+args.map(v=>JSON.stringify(v)).join(',')+')')
 const facts={about:await cdp.invoke('app_about'),dataRoot:await cdp.invoke('get_data_root_config')}
 assertModelSandbox(facts,join(root,'sandbox'));await save('manager-facts.json',facts)
 const managerIdentity=(await snapshot('process-manager')).own.find(p=>p.pid===manager.pid)
 assert(managerIdentity && managerIdentity.sessionId===1 && win32.resolve(managerIdentity.path).toLowerCase()===win32.resolve(plan.managerExe).toLowerCase())
 const shot=async name=>{const value=await cdp.call('Page.captureScreenshot',{format:'png'});await writeFile(join(root,name+'.png'),Buffer.from(value.data,'base64'))}
 const wait=async(fn,label,timeout=30000)=>{const end=Date.now()+timeout;while(Date.now()<end){const v=await evaluate(fn);if(v)return v;await sleep(250)}throw Error('Native UI deadline: '+label)}
 await evaluate(async()=>{const p=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia;p._s.get('routes').go('models');await p._s.get('model-config').load();p._s.get('model-config').view.tab='channels'})
 await wait(()=>!![...document.querySelectorAll('tr[data-channel]')].find(r=>r.textContent.includes('Post021AdminFixture')),'fixture channel')
 const beforePolicy=await cdp.invoke('model_file_snapshot');report.policyBefore=beforePolicy.policyDigest
 await evaluate(()=>{const row=[...document.querySelectorAll('tr[data-channel]')].find(r=>r.textContent.includes('Post021AdminFixture'));const button=[...row.querySelectorAll('button')].find(b=>b.textContent.trim()==='编辑');if(!button)throw Error('Public channel edit missing');button.click()})
 await wait(()=>!!document.querySelector('[aria-label="保存渠道草稿"]'),'channel editor')
 await evaluate(()=>{const m=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia._s.get('model-config');const c=m.channelDraft.channels.find(c=>c.connection.providerId==='Post021AdminFixture');if(!c)throw Error('Fixture draft missing');c.defaults.contextWindow={mode:'custom',value:65547}})
 await wait(()=>{const b=document.querySelector('[aria-label="保存渠道草稿"]');return b&&!b.disabled},'public Save enabled')
 await shot('01-before-real-save')
 // Observe only the final IPC result. Forward the original command unchanged.
 await evaluate(()=>{const api=window.__TAURI_INTERNALS__,original=api.invoke.bind(api);window.__formalRefresh=[];api.invoke=async(cmd,args,options)=>{try{const value=await original(cmd,args,options);if(cmd==='model_policy_refresh_cache')window.__formalRefresh.push({result:value,at:new Date().toISOString()});return value}catch(e){if(cmd==='model_policy_refresh_cache')window.__formalRefresh.push({error:String(e),at:new Date().toISOString()});throw e}}})
 await evaluate(()=>document.querySelector('[aria-label="保存渠道草稿"]').click())
 await wait(()=>[...document.querySelectorAll('button')].some(b=>b.textContent.trim()==='立即重启 Codex'),'actual Save result notice')
 const saved=await cdp.invoke('model_file_snapshot')
 assert.notEqual(saved.policyDigest,beforePolicy.policyDigest,'Real Save must change disk policy')
 assert.equal(saved.policy.channels.find(c=>c.connection.providerId==='Post021AdminFixture').defaults.contextWindow.value,65547)
 await save('save-readback.json',{beforeDigest:beforePolicy.policyDigest,afterDigest:saved.policyDigest,contextWindow:65547,mode:'Actual public Save handler generated notice; no showSaved call'})
 await evaluate(()=>[...document.querySelectorAll('button')].find(b=>b.textContent.trim()==='立即重启 Codex').click())
 await wait(()=>[...document.querySelectorAll('dialog[open] button')].some(b=>b.textContent.trim()==='确认重启'),'production restart confirmation')
 await shot('02-native-restart-confirmation')
 const finalBefore=await snapshot('process-before-restart')
 assert(finalBefore.own.some(p=>shells.some(s=>s.pid===p.pid&&s.createdAtUtcTicks===p.createdAtUtcTicks)),'Formal target changed before restart')
 await evaluate(()=>[...document.querySelectorAll('dialog[open] button')].find(b=>b.textContent.trim()==='确认重启').click())
 const outcome=await wait(()=>window.__formalRefresh?.[0],'official default restart result',150000)
 await save('production-restart-result.json',outcome)
 assert(outcome.result?.success && outcome.result?.restarted,'Production manager did not confirm complete default restart')
 const after=await snapshot('process-after-restart')
 assert(shells.every(s=>!after.own.some(p=>p.pid===s.pid&&p.createdAtUtcTicks===s.createdAtUtcTicks)),'Old formal shell generation survived')
 const replacements=after.own.filter(p=>win32.basename(p.path??'').toLowerCase()==='chatgpt.exe' && !shells.some(s=>s.pid===p.pid&&s.createdAtUtcTicks===p.createdAtUtcTicks))
 assert(replacements.length && replacements.every(p=>p.sessionId===1&&win32.resolve(p.path).toLowerCase().startsWith(packageRoot)),'New formal shell not proved')
 for(const originalOther of finalBefore.other){
  const current=after.other.find(p=>p.pid===originalOther.pid)
  assert(current && current.createdAtUtcTicks===originalOther.createdAtUtcTicks,'Other-owner baseline process changed during restart')
 }
 const activeConfig=await readFile(preflight.config.path,'utf8'),actualPolicy=JSON.parse(await readFile(join(env.OPENCODEX_HOME,'config.json'),'utf8'))
 assert.equal(actualPolicy.providers.Post021AdminFixture.contextWindow,65547)
 assert(activeConfig.includes('127.0.0.1:'+plan.servicePort),'Relaunched client routing config escaped local service')
 const integrationAfter=await api('/api/native-integrations')
 await save('config-after-readback.json',{managerPolicyDigest:(await cdp.invoke('model_file_snapshot')).policyDigest,providerContextWindow:65547,codexConfigSha256:hash(activeConfig),codexConfigBytes:Buffer.byteLength(activeConfig),routingContainsExpectedLocalPort:true,integration:integrationAfter.clients.find(v=>v.clientId==='codex'),formalNewProcesses:replacements,limitation:'Disk and official integration readback; no assertion of model-generation result or UI model-picker state'})
 await shot('03-after-default-restart')
 report.result='passed-default-restart-and-disk-readback';report.acceptance=true
 report.checks=['public channel Save wrote realconfig and generated realnotice','production confirmation called default officialrestart with noadapteroverrides','oldAdministratorformalpackageprocessesexited andnewgenerationstarted','other-owner baselineprocessidentities unchanged','savedpolicy65547 and localCodexconfigrouting readback']
}catch(error){report.result='failed';report.error={message:error.message,stack:error.stack};process.exitCode=1}
finally{
 report.finishedAtUtc=new Date().toISOString();report.cleanup='Processes and authorized testprofile retained for inspection; no broadkill orrestore. Privatebackup stayslocal.'
 await save('formal-result.json',report);cdp?.close()
 service?.unref();manager?.unref()
 console.log(JSON.stringify({result:report.result,error:report.error?.message,output:root}))
}
