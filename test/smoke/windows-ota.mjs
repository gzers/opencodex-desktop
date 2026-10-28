// Released application's actual signed stable updater. No intercepted IPC.
import assert from 'node:assert/strict'
import {writeFile,readFile} from 'node:fs/promises'
import {join} from 'node:path'
import {createHash} from 'node:crypto'
import {discoverMainTarget} from './windows-cdp.mjs'
import {verifyWindowsModelUi,modelConfigPath,assertModelSandbox} from './windows-model-ui.mjs'
const [endpoint,output,sandbox,phase,version]=process.argv.slice(2)
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
 const facts={about:await invoke('app_about'),dataRoot:await invoke('get_data_root_config')};assertModelSandbox(facts,sandbox)
 const config=modelConfigPath(facts,sandbox)
 if(phase==='verify'){
  assert.equal(facts.about.version,version)
  const prior=JSON.parse(await readFile(join(output,'ota-data-before.json'),'utf8'))
  assert.equal(hash(await readFile(config)),prior.configSha256,'actual config and synthetic credentials changed during OTA')
  const status=await invoke('get_update_status');assert.equal(status.currentVersion,version);assert.equal(status.pendingRestart,null)
  await writeFile(join(output,'ota-verified.json'),JSON.stringify({result:'pass',version,actualReopenedApplication:true,configAndCredentialBytesUnchanged:true,immutableSandboxUnchanged:true,pendingRestartCleared:true},null,2)+'\n')
 }else{
  assert.equal(facts.about.version,'0.2.16')
  if(phase==='prepare-exit'){
   await verifyWindowsModelUi(call,output,facts,sandbox)
   await writeFile(join(output,'ota-data-before.json'),JSON.stringify({configSha256:hash(await readFile(config)),fixture:'production file save and synthetic keys through released IPC; no daily configuration'},null,2)+'\n')
  }
  await invoke('set_update_channel',{channel:'stable'})
  const checked=await invoke('check_for_update',{trigger:'user'});assert.equal(checked.update.availableVersion,version)
  await evaluate('window.__otaProgress=[];window.__otaChannel=window.__TAURI_INTERNALS__.transformCallback(m=>window.__otaProgress.push(m),false);true')
  const channel=await evaluate('window.__otaChannel')
  await invoke('install_update',{candidateVersion:version,channel:'stable',backup:true,progress:'__CHANNEL__:'+channel})
  const status=await invoke('get_update_status');assert.equal(status.signatureVerified,true);assert.equal(status.pendingRestart,version)
  const progress=await evaluate('window.__otaProgress');assert(progress.some(p=>p.stage==='pending_restart'))
  await writeFile(join(output,phase+'.json'),JSON.stringify({result:'pass',from:facts.about.version,to:version,signatureVerified:true,pendingRestart:status.pendingRestart,progress,productionStableEndpoint:true},null,2)+'\n')
  const action=phase==='prepare-exit'?"window.__TAURI_INTERNALS__.invoke('plugin:process|exit',{code:0})":"window.__TAURI_INTERNALS__.invoke('restart_after_update')"
  await evaluate('setTimeout(()=>{'+action+'},150);true')
 }
}finally{ws.close();for(const p of pending.values())clearTimeout(p.timer)}
