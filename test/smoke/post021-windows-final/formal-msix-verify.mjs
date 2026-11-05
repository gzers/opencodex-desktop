import assert from 'node:assert/strict'
import {readFile,writeFile} from 'node:fs/promises'
import {join,win32} from 'node:path'
import {createHash} from 'node:crypto'
import {assertModelSandbox} from '../windows-model-ui.mjs'
import {waitForNativeReady} from './formal-native-ready.mjs'
import {observePublicRestart} from './formal-restart-observation.mjs'
export async function verifyNative({cdp,plan,root,preflight,snapshot,api,report,managerPid,shells,save}){
 const sleep=ms=>new Promise(r=>setTimeout(r,ms)),hash=data=>createHash('sha256').update(data).digest('hex')
 const env={OPENCODEX_HOME:join(root,'sandbox','opencodex-home')}
 const packageRoot=win32.resolve(preflight.packages[0].installLocation).toLowerCase()+String.fromCharCode(92)
 const evaluate=(fn,...args)=>cdp.evaluate('('+fn.toString()+')('+args.map(v=>JSON.stringify(v)).join(',')+')')
 await waitForNativeReady(evaluate)
 const facts={about:await cdp.invoke('app_about'),dataRoot:await cdp.invoke('get_data_root_config')}
 assertModelSandbox(facts,join(root,'sandbox'));await save('manager-facts.json',facts)
 const managerIdentity=(await snapshot('process-manager')).own.find(p=>p.pid===managerPid)
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
 let observed;const restartDeadline=Date.now()+150000
 while(Date.now()<restartDeadline){observed=await evaluate(observePublicRestart);if(observed.state!=='pending')break;await sleep(300)}
 await save('production-restart-result.json',{source:'public UI; not a raw native result',observation:observed})
 assert(observed?.state==='ui-confirmed','Production manager did not confirm complete default restart')
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
}
