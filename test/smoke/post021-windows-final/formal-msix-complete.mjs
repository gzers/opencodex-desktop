import assert from 'node:assert/strict'
import {readFile,writeFile,mkdir,access} from 'node:fs/promises'
import {execFileSync} from 'node:child_process'
import {join,resolve,win32} from 'node:path'
import {fileURLToPath} from 'node:url'
import {createHash} from 'node:crypto'
import {connect} from './cdp-session.mjs'
import {assertModelSandbox} from '../windows-model-ui.mjs'
import {waitForNativeReady} from './formal-native-ready.mjs'
import {observePublicRestart} from './formal-restart-observation.mjs'
const [planFile]=process.argv.slice(2),plan=JSON.parse(await readFile(planFile,'utf8')),root=resolve(plan.runRoot)
assert.equal(root,resolve('D:/Users/Administrator/WorkSpace/Test/post021-formal-20261105-1518'))
const ps=join(process.env.SystemRoot??'C:/Windows','System32/WindowsPowerShell/v1.0/powershell.exe')
const owner=execFileSync(ps,['-NoProfile','-Command','[Security.Principal.WindowsIdentity]::GetCurrent().Name'],{encoding:'utf8',windowsHide:true,timeout:15000}).trim()
assert.equal(owner.toLowerCase(),plan.expectedOwner.toLowerCase(),'Administrator login required')
const priorRoot=join(root,'bootstrap-recovery'),prior=JSON.parse(await readFile(join(priorRoot,'formal-result.json'),'utf8'))
assert(prior.result==='failed'&&prior.error?.message.includes('formal-msix-snapshot.ps1'),'Only observed snapshot-before-restart failure is admitted')
const saved=JSON.parse(await readFile(join(priorRoot,'save-readback.json'),'utf8'))
assert.equal(saved.contextWindow,65547)
for(const dir of [root,priorRoot])await assert.rejects(access(join(dir,'production-restart-result.json')),{code:'ENOENT'},'Do not repeat any previous or uncertain restart')
const output=join(root,'default-restart-completion');await mkdir(output,{recursive:false})
const save=(name,value)=>writeFile(join(output,name),JSON.stringify(value,null,2)+'\n')
const snapshot=async name=>{
 const v=JSON.parse(execFileSync(ps,['-NoProfile','-NonInteractive','-File',fileURLToPath(new URL('./formal-msix-snapshot.ps1',import.meta.url))],{encoding:'utf8',windowsHide:true,timeout:90000}))
 await save(name+'.json',v);assert.equal(v.owner.toLowerCase(),owner.toLowerCase());assert.equal(v.sessionId,1);assert.equal(v.unknownOwnerCount,0,'Still-live unknown owner must refuse restart');return v
}
const report={result:'running',acceptance:false,owner,startedAtUtc:new Date().toISOString(),mode:'Continue actualsavedpolicy and existingnativeconfirmation; no Save/configinitialization/clientlaunch replay'}
await save('completion-started.json',report)
let cdp
try{
 const before=await snapshot('process-before-restart')
 const known=JSON.parse(await readFile(join(priorRoot,'process-manager.json'),'utf8')),admittedManager=known.own.find(p=>p.pid===prior.managerPid)
 assert(admittedManager&&before.own.some(p=>p.pid===admittedManager.pid&&p.createdAtUtcTicks===admittedManager.createdAtUtcTicks&&p.path===admittedManager.path),'Exact native manager changed')
 const initial=JSON.parse(await readFile(join(root,'formal-result.json'),'utf8')),original=initial.formalBefore
 assert(original.every(s=>before.own.some(p=>p.pid===s.pid&&p.createdAtUtcTicks===s.createdAtUtcTicks)),'Original formal shell generation changed')
 const health=await(await fetch('http://127.0.0.1:'+plan.servicePort+'/healthz',{signal:AbortSignal.timeout(10000)})).json()
 assert.equal(health.pid,prior.servicePid,'Original recovered service changed')
 cdp=await connect('http://127.0.0.1:'+plan.cdpPort)
 const evaluate=(fn,...args)=>cdp.evaluate('('+fn.toString()+')('+args.map(v=>JSON.stringify(v)).join(',')+')')
 await waitForNativeReady(evaluate)
 assertModelSandbox({about:await cdp.invoke('app_about'),dataRoot:await cdp.invoke('get_data_root_config')},join(root,'sandbox'))
 const currentPolicy=await cdp.invoke('model_file_snapshot')
 assert.equal(currentPolicy.policyDigest,saved.afterDigest,'Saved policy drifted after real Save')
 assert.equal((await evaluate(observePublicRestart)).state,'pending','A previous public restart result exists; do not repeat')
 const visible=await evaluate(()=>[...document.querySelectorAll('dialog[open] button')].some(b=>b.textContent.trim()==='确认重启'&&!b.disabled))
 assert(visible,'Actual pending public confirmation no longer available; no artificial notice created')
 const shot=async name=>{const s=await cdp.call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,name+'.png'),Buffer.from(s.data,'base64'))}
 await shot('before-default-restart')
 report.requestedAtUtc=new Date().toISOString();await save('request-started.json',report)
 await evaluate(()=>[...document.querySelectorAll('dialog[open] button')].find(b=>b.textContent.trim()==='确认重启'&&!b.disabled).click())
 let outcome;const deadline=Date.now()+150000
 while(Date.now()<deadline){outcome=await evaluate(observePublicRestart);if(outcome.state!=='pending')break;await new Promise(r=>setTimeout(r,300))}
 await save('production-restart-result.json',{source:'public UI; not a raw native result',observation:outcome??{state:'deadline'}})
 const after=await snapshot('process-after-restart')
 const beforeShells=before.own.filter(p=>win32.basename(p.path??'').toLowerCase()==='chatgpt.exe')
 const newShells=after.own.filter(p=>win32.basename(p.path??'').toLowerCase()==='chatgpt.exe'&&!beforeShells.some(s=>s.pid===p.pid&&s.createdAtUtcTicks===p.createdAtUtcTicks))
 const oldSurvivors=beforeShells.filter(s=>after.own.some(p=>p.pid===s.pid&&p.createdAtUtcTicks===s.createdAtUtcTicks))
 const stableOther=before.other.filter(p=>['chatgpt.exe','opencodex-desktop.exe'].includes(win32.basename(p.path??'').toLowerCase()))
 const otherChanges=stableOther.filter(s=>!after.other.some(p=>p.pid===s.pid&&p.createdAtUtcTicks===s.createdAtUtcTicks))
 await save('process-generation-review.json',{oldSurvivors,newShells,otherOwnerDesktopBaselineCount:stableOther.length,otherOwnerDesktopChanges:otherChanges})
 const config=await readFile('C:/Users/Administrator/.codex/config.toml','utf8'),disk=JSON.parse(await readFile(join(root,'sandbox','opencodex-home','config.json'),'utf8'))
 await save('config-readback.json',{contextWindow:disk.providers.Post021AdminFixture.contextWindow,policyDigest:(await cdp.invoke('model_file_snapshot')).policyDigest,codexConfigSha256:createHash('sha256').update(config).digest('hex'),codexConfigBytes:Buffer.byteLength(config),routingContainsLocalPort:config.includes('127.0.0.1:'+plan.servicePort),scope:'Disk/native-manager readback after actual default-adapter action; model generation/picker refresh not asserted'})
 await shot('after-default-restart')
 assert(outcome?.state==='ui-confirmed','Production manager did not confirm complete default restart')
 assert.equal(oldSurvivors.length,0,'Old formal shell generation survived')
 const expectedImageRoot=win32.dirname(win32.resolve(initial.formalBefore[0].path)).toLowerCase()+win32.sep
 assert(newShells.length&&newShells.every(p=>p.sessionId===1&&win32.resolve(p.path).toLowerCase().startsWith(expectedImageRoot)),'New formal package generation not proved')
 assert.equal(otherChanges.length,0,'Other-owner desktop generation changed')
 assert.equal(disk.providers.Post021AdminFixture.contextWindow,65547)
 assert.equal((await cdp.invoke('model_file_snapshot')).policyDigest,saved.afterDigest)
 assert(config.includes('127.0.0.1:'+plan.servicePort))
 report.result='passed-default-restart-and-disk-readback';report.acceptance=true;report.checks=['actual publicSave65547 retained','actualpublicrestartconfirmation anddefaultpublishedadapter','alloldformalclientgenerationsgone/newAdministratorpackagegenerations','otherownerdesktopbaselineunchanged','configreadbackandtestloopbackroutingretained']
}catch(error){report.result='failed';report.error={message:error.message,stack:error.stack};process.exitCode=1}
finally{report.finishedAtUtc=new Date().toISOString();report.cleanup='Authorizedtestprofile andtestprocesses retained; no manualkill/restore/repeatedrestart.';await save('formal-result.json',report);cdp?.close();console.log(JSON.stringify({result:report.result,error:report.error?.message,output}))}
