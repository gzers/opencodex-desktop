import assert from 'node:assert/strict'
import {readFile,writeFile,mkdir,readdir,stat} from 'node:fs/promises'
import {join,resolve,win32} from 'node:path'
import {fileURLToPath,pathToFileURL} from 'node:url'
import {execFileSync} from 'node:child_process'
import {createHash} from 'node:crypto'
import {connect} from './cdp-session.mjs'
import {observePublicRestart} from './formal-restart-observation.mjs'
import {assertModelSandbox} from '../windows-model-ui.mjs'
const [planFile]=process.argv.slice(2),plan=JSON.parse(await readFile(planFile,'utf8')),root=resolve(plan.runRoot)
assert.equal(root,resolve('D:/Users/Administrator/WorkSpace/Test/post021-formal-20261105-1518'))
const ps=join(process.env.SystemRoot??'C:/Windows','System32/WindowsPowerShell/v1.0/powershell.exe')
const owner=execFileSync(ps,['-NoProfile','-Command','[Security.Principal.WindowsIdentity]::GetCurrent().Name'],{encoding:'utf8',windowsHide:true,timeout:15000}).trim()
assert.equal(owner.toLowerCase(),'ezio-mini\\administrator')
const previous=join(root,'installed-runtime-validation-20261105'),result=JSON.parse(await readFile(join(previous,'result.json'),'utf8'))
assert.equal(result.result,'failed')
const failure=JSON.parse(await readFile(join(previous,'05-rename-default-restart-public-result.json'),'utf8'))
assert(failure.error.includes('同步期间配置文件已变化'))
const output=join(root,'installed-runtime-rename-followup-20261105');await mkdir(output,{recursive:false})
const sha=b=>createHash('sha256').update(b).digest('hex'),save=(name,v)=>writeFile(join(output,name+'.json'),JSON.stringify(v,null,2)+'\n')
const home=join(root,'sandbox','opencodex-home'),path=join(home,'config.json'),report={result:'running',acceptance:false,owner,startedAtUtc:new Date().toISOString(),runtimeVersion:plan.expectedRuntimeVersion,actions:'Read-only structural diagnosis, then one guarded existing public retry; no reseed, resave, service switch or second client launch'}
await save('started',report)
let cdp
const snapshot=async name=>{const s=JSON.parse(execFileSync(ps,['-NoProfile','-NonInteractive','-File',fileURLToPath(new URL('./formal-msix-snapshot.ps1',import.meta.url))],{encoding:'utf8',windowsHide:true,timeout:90000}));await save(name,s);assert.equal(s.owner.toLowerCase(),owner.toLowerCase());assert.equal(s.sessionId,1);assert.equal(s.unknownOwnerCount,0);return s}
const shells=s=>s.own.filter(p=>win32.basename(p.path??'').toLowerCase()==='chatgpt.exe')
function differences(a,b,prefix=''){
 if(JSON.stringify(a)===JSON.stringify(b))return[]
 if(a&&b&&typeof a==='object'&&typeof b==='object'&&!Array.isArray(a)&&!Array.isArray(b))return [...new Set([...Object.keys(a),...Object.keys(b)])].sort().flatMap(k=>differences(a[k],b[k],prefix+'/'+k))
 return[{path:prefix,beforeSha256:sha(JSON.stringify(a)??'undefined'),afterSha256:sha(JSON.stringify(b)??'undefined')}]
}
try{
 const beforeBytes=await readFile(path),disk=JSON.parse(beforeBytes),images=[]
 assert.deepEqual(Object.keys(disk.providers),['Post021AdminRenamed']);assert.equal(disk.defaultProvider,'Post021AdminRenamed')
 async function scan(folder){for(const e of await readdir(folder,{withFileTypes:true})){
  const p=join(folder,e.name)
  if(e.isDirectory())await scan(p)
  else if(/^[0-9]+[.](before|after)$/.test(e.name)&&(await stat(p)).size<16*1024*1024){
   const bytes=await readFile(p);let value;try{value=JSON.parse(bytes)}catch{continue}
   if(value?.providers&&Object.hasOwn(value.providers,'Post021AdminRenamed'))images.push({image:win32.relative(root,p),sha256:sha(bytes),changedPaths:differences(value,disk)})
  }
 }}
 await scan(join(root,'sandbox','backups'))
 const packageJson=JSON.parse(await readFile(join(plan.officialPackage,'package.json'),'utf8'));assert.equal(packageJson.version,plan.expectedRuntimeVersion)
 const pendingPath=join(root,'sandbox','manager-state','provider-client-renames.json'),pendingBefore=await readFile(pendingPath)
 await save('read-only-drift-diagnosis',{configSha256:sha(beforeBytes),providerNames:Object.keys(disk.providers),defaultProvider:disk.defaultProvider,renameJournalComparisons:images,pendingSha256:sha(pendingBefore),runtimeVersion:packageJson.version,secretPolicy:'Only changed JSON paths and value digests; no raw config, credentials or history exported'})
 const dailyPath='C:/Users/15119/.codex/config.toml',dailyDigest=sha(await readFile(dailyPath))
 cdp=await connect('http://127.0.0.1:'+plan.cdpPort)
 assertModelSandbox({about:await cdp.invoke('app_about'),dataRoot:await cdp.invoke('get_data_root_config')},join(root,'sandbox'))
 const evaluate=(fn,...args)=>cdp.evaluate('('+fn.toString()+')('+args.map(v=>JSON.stringify(v)).join(',')+')')
 const state=await evaluate(observePublicRestart);assert.equal(state.state,'failed');assert(state.error.includes('同步期间配置文件已变化'))
 const before=await snapshot('process-before-explicit-retry')
 assert(sha(await readFile(path))===sha(beforeBytes),'Saved config changed during read-only diagnosis')
 assert((await readFile(pendingPath)).equals(pendingBefore),'Pending rename generation changed')
 const saved=await cdp.invoke('model_file_snapshot'),channel=saved.policy.channels.find(c=>c.connection.providerId==='Post021AdminRenamed')
 assert(channel&&channel.defaults.contextWindow.value===65558)
 const shot=async name=>{const v=await cdp.call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,name+'.png'),Buffer.from(v.data,'base64'))}
 await shot('before-explicit-rename-retry')
 await evaluate(()=>{const b=[...document.querySelectorAll('dialog[open] button')].find(b=>b.textContent.trim()==='重试重启'&&!b.disabled);if(!b)throw Error('Existing public retry absent');b.click()})
 let outcome;const deadline=Date.now()+150000
 while(Date.now()<deadline){outcome=await evaluate(observePublicRestart);if(outcome.state!=='pending')break;await new Promise(r=>setTimeout(r,200))}
 await save('public-retry-result',outcome??{});await shot('after-explicit-rename-retry')
 const after=await snapshot('process-after-explicit-retry'),old=shells(before),next=shells(after)
 const oldSurvivors=old.filter(p=>next.some(v=>v.pid===p.pid&&v.createdAtUtcTicks===p.createdAtUtcTicks))
 const newGenerations=next.filter(p=>!old.some(v=>v.pid===p.pid&&v.createdAtUtcTicks===p.createdAtUtcTicks))
 const criticalOther=before.other.filter(p=>['chatgpt.exe','codex.exe','opencodex-desktop.exe'].includes(win32.basename(p.path??'').toLowerCase()))
 const otherChanges=criticalOther.filter(p=>!after.other.some(v=>v.pid===p.pid&&v.createdAtUtcTicks===p.createdAtUtcTicks))
 const config=await readFile('C:/Users/Administrator/.codex/config.toml','utf8'),policy=JSON.parse(await readFile(path,'utf8')),snapshotAfter=await cdp.invoke('model_file_snapshot')
 const pendingAfter=await readFile(pendingPath).catch(e=>{if(e.code==='ENOENT')return null;throw e})
 await save('readback',{ui:outcome,oldSurvivors,newDesktopGenerations:newGenerations,currentDesktopGenerations:next,otherOwnerProcessChanges:otherChanges,dailyConfigBefore:dailyDigest,dailyConfigAfter:sha(await readFile(dailyPath)),codexConfigSha256:sha(config),newModelReferencePresent:config.includes('Post021AdminRenamed/raw-model'),oldModelReferenceAbsent:!config.includes('Post021AdminFixture/raw-model'),localRoute:config.includes('127.0.0.1:'+plan.servicePort),providerNames:Object.keys(policy.providers),defaultProvider:policy.defaultProvider,contextWindow:policy.providers.Post021AdminRenamed.contextWindow,policyBefore:saved.policyDigest,policyAfter:snapshotAfter.policyDigest,pendingRenameBefore:sha(pendingBefore),pendingRenameAfter:pendingAfter?sha(pendingAfter):null})
 assert.equal(outcome?.state,'ui-confirmed');assert.equal(oldSurvivors.length,0);assert(next.length)
 assert(next.every(p=>p.sessionId===1&&win32.resolve(p.path).toLowerCase()===win32.resolve(old[0].path).toLowerCase()))
 assert.equal(otherChanges.length,0);assert.equal(sha(await readFile(dailyPath)),dailyDigest)
 assert(config.includes('Post021AdminRenamed/raw-model')&&!config.includes('Post021AdminFixture/raw-model'))
 assert(config.includes('127.0.0.1:'+plan.servicePort));assert.equal(policy.providers.Post021AdminRenamed.contextWindow,65558)
 assert.equal(saved.policyDigest,snapshotAfter.policyDigest)
 report.result='passed-installed-candidate-explicit-rename-retry';report.acceptance=true
}catch(error){report.result='failed';report.error={message:error.message,stack:error.stack};process.exitCode=1}
finally{report.finishedAtUtc=new Date().toISOString();await save('result',report);cdp?.close();console.log(JSON.stringify({result:report.result,error:report.error?.message,output}))}
