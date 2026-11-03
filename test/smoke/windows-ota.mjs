// Released application's actual signed updater. No intercepted IPC/endpoints.
import assert from 'node:assert/strict'
import {writeFile,readFile} from 'node:fs/promises'
import {join} from 'node:path'
import {createHash} from 'node:crypto'
import {discoverMainTarget} from './windows-cdp.mjs'
import {otaChannel} from './windows-ota-channel.mjs'
import {modelConfigPath,assertModelSandbox} from './windows-model-ui.mjs'
import {seedOtaBaseline} from './windows-ota-fixture.mjs'
const [endpoint,output,sandbox,phase,version,fromVersion='0.2.18',requestedChannel]=process.argv.slice(2)
const channel=otaChannel(requestedChannel)
assert.equal(new URL(endpoint).hostname,'127.0.0.1')
assert(['prepare-exit','prepare-restart','verify'].includes(phase))
const target=await discoverMainTarget(endpoint)
const ws=new WebSocket(target.webSocketDebuggerUrl),pending=new Map();let id=0
await new Promise((resolve,reject)=>{const timer=setTimeout(()=>reject(Error('OTA CDP open deadline')),10000);ws.addEventListener('open',()=>{clearTimeout(timer);resolve()},{once:true});ws.addEventListener('error',reject,{once:true})})
ws.addEventListener('message',e=>{const m=JSON.parse(e.data),p=pending.get(m.id);if(p){pending.delete(m.id);clearTimeout(p.timer);m.error?p.reject(Error(m.error.message)):p.resolve(m.result)}})
function call(method,params={}){return new Promise((resolve,reject)=>{const current=++id,timer=setTimeout(()=>{pending.delete(current);reject(Error('OTA CDP deadline: '+method))},180000);pending.set(current,{resolve,reject,timer});ws.send(JSON.stringify({id:current,method,params}))})}
async function evaluate(expression){const r=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!r.exceptionDetails,JSON.stringify(r.exceptionDetails));return r.result.value}
const invoke=(name,args={})=>evaluate('window.__TAURI_INTERNALS__.invoke('+JSON.stringify(name)+','+JSON.stringify(args)+')')
const hash=bytes=>createHash('sha256').update(bytes).digest('hex')
try{
 let ready=false
 for(let n=0;n<120;n++){ready=await evaluate("!!window.__TAURI_INTERNALS__?.invoke&&!!document.querySelector('#app')?.children.length&&document.body.innerText.length>30");if(ready)break;await new Promise(resolve=>setTimeout(resolve,250))}
 assert(ready,'released application did not mount its actual UI and IPC bridge')
 const facts={about:await invoke('app_about'),dataRoot:await invoke('get_data_root_config')};assertModelSandbox(facts,sandbox)
 const config=modelConfigPath(facts,sandbox)
 if(phase==='verify'){
  assert.equal(facts.about.version,version)
  const prior=JSON.parse(await readFile(join(output,'ota-data-before.json'),'utf8'))
  assert.equal(hash(await readFile(config)),prior.configSha256,'actual config and synthetic credentials changed during OTA')
  const snapshot=await invoke('model_file_snapshot');assert(snapshot.policy.channels.some(c=>c.connection.providerId===prior.providerId),'updated application cannot reopen the baseline saved provider')
  const status=await invoke('get_update_status');assert.equal(status.currentVersion,version);assert.equal(status.channel,channel);assert.equal(status.pendingRestart,null)
  // Inspect the installed WebView2 payload with the production CSS scale token.
  // The token is temporary: no persisted preferences or update requests change.
  const scaleSamples=[]
  for(const scale of [100,150,200]){
   await evaluate("document.documentElement.style.setProperty('--ui-zoom',"+JSON.stringify(String(scale/100))+");[...document.querySelectorAll('.nav button')].find(b=>b.textContent.trim()==='概览').click();true")
   let opened=false
   for(let n=0;n<100;n++){
    opened=await evaluate("(()=>{const b=document.querySelector('button[aria-label^=\"查看桌面管理器更新详情\"]');if(!b||b.disabled)return false;b.click();return true})()")
    if(opened)break;await new Promise(resolve=>setTimeout(resolve,50))
   }
   assert(opened,'actual updated Overview manager detail opener unavailable')
   let sample
   for(let n=0;n<100;n++){
    sample=await evaluate("(()=>{const m=document.querySelector('[data-testid=update-center] .update-center'),b=m?.querySelector('.update-center-foot button');if(!m||!b)return null;const r=m.getBoundingClientRect(),a=b.getBoundingClientRect();return {buttonHeight:a.height,buttonX:a.x,buttonRight:a.right,buttonBottom:a.bottom,x:r.x,y:r.y,right:r.right,bottom:r.bottom,viewportWidth:innerWidth,viewportHeight:innerHeight,zoom:getComputedStyle(m).zoom}})()")
    if(sample?.buttonHeight>0)break;await new Promise(resolve=>setTimeout(resolve,50))
   }
   assert(sample&&sample.x>=-1&&sample.y>=-1&&sample.right<=sample.viewportWidth+1&&sample.bottom<=sample.viewportHeight+1,'updated native dialog outside viewport: '+JSON.stringify(sample))
   assert(sample.buttonX>=-1&&sample.buttonRight<=sample.viewportWidth+1&&sample.buttonBottom<=sample.viewportHeight+1,'updated native close action outside viewport: '+JSON.stringify(sample))
   scaleSamples.push({scale,...sample})
   await evaluate("document.querySelector('[data-testid=update-center] button[aria-label=\"关闭更新中心\"]').click();true")
  }
  for(const sample of scaleSamples)assert(Math.abs(sample.buttonHeight/scaleSamples[0].buttonHeight-sample.scale/100)<.03,'updated native dialog does not follow shared scale: '+JSON.stringify(scaleSamples))
  await evaluate("document.documentElement.style.setProperty('--ui-zoom','1');true")
  await writeFile(join(output,'update-scale-native.json'),JSON.stringify({result:'pass',version,host:'actual updated Overview UpdateCenter',engine:'Windows installed WebView2',samples:scaleSamples,configurationUnchanged:true,scope:'actual installed binary CSS scale token; no new update/check request or persisted preference write'},null,2)+'\n')
  await writeFile(join(output,'ota-verified.json'),JSON.stringify({result:'pass',version,channel,updatedApplicationReadback:true,automaticRestartEvidence:'ota-result.json OS process observation',configAndCredentialBytesUnchanged:true,immutableSandboxUnchanged:true,pendingRestartCleared:true},null,2)+'\n')
 }else{
  assert.equal(facts.about.version,fromVersion)
  if(phase==='prepare-exit'){
   await seedOtaBaseline(invoke,output,facts,sandbox)
  }
  await invoke('set_update_channel',{channel})
  const checked=await invoke('check_for_update',{trigger:'user'});assert.equal(checked.update.channel,channel);assert.equal(checked.update.availableVersion,version)
  await evaluate('window.__otaProgress=[];window.__otaChannel=window.__TAURI_INTERNALS__.transformCallback(raw=>{if(raw.message)window.__otaProgress.push(raw.message)},false);true')
  const progressChannel=await evaluate('window.__otaChannel')
  await invoke('install_update',{candidateVersion:version,channel,backup:true,progress:'__CHANNEL__:'+progressChannel})
  const status=await invoke('get_update_status');assert.equal(status.signatureVerified,true);assert.equal(status.pendingRestart,version)
  let progress=[];for(let n=0;n<40;n++){progress=await evaluate('window.__otaProgress');if(progress.some(p=>p.stage==='pending_restart'))break;await new Promise(resolve=>setTimeout(resolve,50))}assert(progress.some(p=>p.stage==='pending_restart'),'real channel never delivered pending_restart: '+JSON.stringify(progress))
  await writeFile(join(output,phase+'.json'),JSON.stringify({result:'pass',from:facts.about.version,to:version,channel,signatureVerified:true,pendingRestart:status.pendingRestart,progress,productionChannelEndpoint:true},null,2)+'\n')
  const action=phase==='prepare-exit'?"window.__TAURI_INTERNALS__.invoke('plugin:process|exit',{code:0})":"window.__TAURI_INTERNALS__.invoke('restart_after_update')"
  await evaluate('setTimeout(()=>{'+action+'},150);true')
 }
}finally{ws.close();for(const p of pending.values())clearTimeout(p.timer)}
