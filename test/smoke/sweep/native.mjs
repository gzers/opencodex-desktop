// One independent isolated native probe. Every failed checkpoint remains a
// failure; later safe checks still run. No stores, fake IPC or ready injection.
import assert from 'node:assert/strict'
import {readFile,writeFile,mkdir} from 'node:fs/promises'
import {join} from 'node:path'
import {discoverMainTarget} from '../windows-cdp.mjs'
import {runtimeAcceptanceLayout,assert023RuntimeAdmission} from '../windows023-runtime-layout.mjs'
import {verifyWindowsPathUi} from '../windows-path-ui.mjs'
import {verifyWindowsQueryUi} from '../windows-query-ui.mjs'
import {runtimeInstallDomExpression,runtimeInstallExpression,closeRuntimeInstallTerminal} from '../windows-021-runtime-host.mjs'
const [endpoint,output,sandbox,_dependency,probe]=process.argv.slice(2)
assert(['paths','install','query'].includes(probe),'Explicit diagnostic probe required')
const target=await discoverMainTarget(endpoint),ws=new WebSocket(target.webSocketDebuggerUrl),pending=new Map();let id=0
await new Promise((resolve,reject)=>{ws.addEventListener('open',resolve,{once:true});ws.addEventListener('error',reject,{once:true})})
ws.addEventListener('message',e=>{const m=JSON.parse(e.data),p=pending.get(m.id);if(p){pending.delete(m.id);clearTimeout(p.timer);m.error?p.reject(Error(m.error.message)):p.resolve(m.result)}})
function call(method,params={}){return new Promise((resolve,reject)=>{const current=++id,timer=setTimeout(()=>{pending.delete(current);reject(Error('CDP timeout: '+method))},30000);pending.set(current,{resolve,reject,timer});ws.send(JSON.stringify({id:current,method,params}))})}
async function evaluate(expression){const r=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!r.exceptionDetails,JSON.stringify(r.exceptionDetails));return r.result.value}
async function wait(expression,timeout=20000){const end=Date.now()+timeout;while(Date.now()<end){const v=await evaluate(expression);if(v)return v;await new Promise(r=>setTimeout(r,200))}throw Error('UI wait: '+expression)}
async function click(text,selector='.route-section.active'){assert(await evaluate('(()=>{const root=document.querySelector('+JSON.stringify(selector)+');const b=[...root?.querySelectorAll("button")??[]].find(b=>(b.textContent.trim()==='+JSON.stringify(text)+'||b.getAttribute("aria-label")==='+JSON.stringify(text)+')&&!b.disabled&&b.getClientRects().length);if(!b)return false;b.click();return true})()'),'Missing real enabled action: '+text)}
const facts={};const checks=[]
async function check(name,action){try{const evidence=await action();const r={name,status:'pass',accepted:false,evidence};checks.push(r);await writeFile(join(output,'sweep-'+name+'.json'),JSON.stringify(r,null,2),{flag:'wx'});return evidence}catch(error){const r={name,status:'fail',accepted:false,firstError:error.message.split(/\r?\n/)[0],stack:error.stack};checks.push(r);await writeFile(join(output,'sweep-'+name+'.json'),JSON.stringify(r,null,2),{flag:'wx'});return null}}
async function source(){return evaluate("window.__TAURI_INTERNALS__.invoke('runtime_source')")}
async function snapshotBytes(s){assert(s?.managedEntry,'Actual installed managed source is a prerequisite');return {source:s,entry:await readFile(s.managedEntry),manifest:await readFile(join(s.managedPrefix,'.runtime-manifest.json'))}}
async function assertPreserved(before){assert(before,'No actual preceding managed install to preserve');const now=await source();for(const k of ['kind','path','version'])assert.deepEqual(now[k],before.source[k]);assert.deepEqual(await readFile(before.source.managedEntry),before.entry);assert.deepEqual(await readFile(join(before.source.managedPrefix,'.runtime-manifest.json')),before.manifest)}
async function install(version){
 // Close only an actual settled installation via the same enabled header X.
 const existing=await evaluate(runtimeInstallExpression('0.2.23'))
 if(existing.present){assert.equal(existing.installing,false,'Previous installation still in flight; no competing mutation');await closeRuntimeInstallTerminal({wait,click},'0.2.23',existing.done?'done':'failed')}
 await click('设置','.side');await wait("!!document.querySelector('.settings-tabs')");await click('安装配置');await click('安装程序')
 const expression=(fn,...args)=>runtimeInstallDomExpression('0.2.23',fn,...args)
 await wait(expression(root=>!!document.querySelector(root+' #wiz-prefix')))
 await evaluate(expression((root,value)=>{const input=document.querySelector(root+' input[aria-label="版本"]');input.value=value;input.dispatchEvent(new Event('input',{bubbles:true}));document.querySelector(root+' [data-testid=wiz-ack]').click()},version))
 await click('开始安装','dialog.runtime-install-dialog[open]')
 const terminal=await wait('(()=>{const s='+runtimeInstallExpression('0.2.23')+';return !s.installing&&(s.error||s.done)?s:false})()',300000)
 const image=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,'install-'+version+'.png'),Buffer.from(image.data,'base64'))
 return terminal
}
try{
 await wait("!!window.__TAURI_INTERNALS__?.invoke&&!!document.querySelector('.side')")
 facts.about=await evaluate("window.__TAURI_INTERNALS__.invoke('app_about')");facts.dataRoot=await evaluate("window.__TAURI_INTERNALS__.invoke('get_data_root_config')")
 const layout=runtimeAcceptanceLayout(facts.about.version,sandbox);assert023RuntimeAdmission(facts,sandbox)
 if(probe==='paths'){
  await check('path-snapshot-copy',async()=>{await click('设置','.side');await wait("!!document.querySelector('.settings-tabs')");return verifyWindowsPathUi(call,output,layout.dataRoot)})
 }else{
  const initial=await check('initial-install',async()=>{const r=await install('2.91.0');assert(r.done&&!r.error,JSON.stringify(r));return snapshotBytes(await source())})
  if(probe==='install'){
   await check('missing-version-retains-original',async()=>{assert(initial,'Prerequisite initial-install failed; no prior package was fabricated');const r=await install('0.0.0-ocx-missing-native-recovery');assert(r.error.includes('npm pack')&&r.error.includes('日志：'));assert(r.console.includes('npm error')&&!r.offlineSteps&&!r.scriptsEnabled);await assertPreserved(initial);return {error:r.error,bytesPreserved:true}})
   await check('recover-upgrade',async()=>{const r=await install('2.93.0');assert(r.done&&!r.error,JSON.stringify(r));const s=await source();assert.equal(s.kind,'managed');assert.equal(s.version,'2.93.0');return s})
   await check('post-install-readiness',async()=>{const r=await evaluate("window.__TAURI_INTERNALS__.invoke('discover_environment',{request:null})");assert.equal(r.gate,'ready');return r})
  }else{
   await check('query-update-recovery',async()=>{assert(initial,'Prerequisite initial-install failed; real query needs actual managed source');await closeRuntimeInstallTerminal({wait,click},'0.2.23','done');await click('设置','.side');await wait("!!document.querySelector('.settings-tabs')");return verifyWindowsQueryUi(call,output,sandbox,{dataRoot:layout.dataRoot,diagnosticSweep:true})})
  }
 }
} catch(error){await check('probe-precondition',async()=>{throw error})}
finally{await writeFile(join(output,'native-sweep.json'),JSON.stringify({accepted:false,diagnosticOnly:true,probe,facts,checks,status:checks.length&&checks.every(c=>c.status==='pass')?'pass':'fail'},null,2),{flag:'wx'});ws.close();for(const p of pending.values())clearTimeout(p.timer)}
if(!checks.length||checks.some(c=>c.status==='fail'))process.exitCode=1
