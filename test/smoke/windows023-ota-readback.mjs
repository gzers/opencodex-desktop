// Final installed023/explicit original022 rollback readback; no state injection.
import assert from 'node:assert/strict'
import {readFile,writeFile} from 'node:fs/promises'
import {join} from 'node:path'
import {createHash} from 'node:crypto'
import {discoverMainTarget} from './windows-cdp.mjs'
import {assert023Paths} from './windows023-native-model.mjs'
import {modelConfigPath} from './windows-model-ui.mjs'
const [endpoint,output,sandbox,phase,channel]=process.argv.slice(2)
assert(['verify023','rollback022'].includes(phase))
assert.equal(new URL(endpoint).hostname,'127.0.0.1')
const target=await discoverMainTarget(endpoint),ws=new WebSocket(target.webSocketDebuggerUrl),pending=new Map();let id=0
await new Promise((resolve,reject)=>{const timer=setTimeout(()=>reject(Error('OTA readback connect deadline')),10000);ws.addEventListener('open',()=>{clearTimeout(timer);resolve()},{once:true});ws.addEventListener('error',reject,{once:true})})
ws.addEventListener('message',event=>{const m=JSON.parse(event.data),p=pending.get(m.id);if(p){pending.delete(m.id);clearTimeout(p.timer);m.error?p.reject(Error(m.error.message)):p.resolve(m.result)}})
function call(method,params={}){return new Promise((resolve,reject)=>{const current=++id,timer=setTimeout(()=>{pending.delete(current);reject(Error('OTA readback deadline: '+method))},30000);pending.set(current,{resolve,reject,timer});ws.send(JSON.stringify({id:current,method,params}))})}
async function evaluate(expression){const r=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});assert(!r.exceptionDetails,JSON.stringify(r.exceptionDetails));return r.result.value}
const invoke=(name,args={})=>evaluate('window.__TAURI_INTERNALS__.invoke('+JSON.stringify(name)+','+JSON.stringify(args)+')')
const hash=b=>createHash('sha256').update(b).digest('hex')
const report={result:'fail',phase,channel}
try{
 let mounted=false
 for(let n=0;n<120;n++){mounted=await evaluate("!!window.__TAURI_INTERNALS__?.invoke&&!!document.querySelector('.side')");if(mounted)break;await new Promise(r=>setTimeout(r,250))}
 assert(mounted,'Actual installed application did not mount')
 const facts={about:await invoke('app_about'),dataRoot:await invoke('get_data_root_config')},prior=JSON.parse(await readFile(join(output,'ota-data-before.json'),'utf8'))
 report.facts=facts
 assert.equal(hash(await readFile(join(sandbox,'opencodex-home','config.json'))),prior.configSha256,'old022 config bytes/credentials changed')
 if(phase==='verify023'){
  const paths=assert023Paths(facts,sandbox)
  const snapshot=await invoke('model_v2_file_snapshot')
  assert.equal(snapshot.dtoVersion,2);assert.equal(snapshot.savedPolicy.channels.length,0,'023 must not import old022 channels')
  const status=await invoke('get_update_status');assert.equal(status.currentVersion,'0.2.23');assert.equal(status.pendingRestart,null)
  assert.equal(await evaluate("!!document.querySelector('[data-testid=environment-gate]')||!!document.querySelector('.ov .mods')"),true)
  report.newPaths=paths;report.oldConfigUnchanged=true;report.noMigration=true;report.pendingRestartCleared=true
 }else{
  assert.equal(facts.about.version,'0.2.22')
  assert.equal(hash(await readFile(modelConfigPath(facts,sandbox))),prior.configSha256)
  const snapshot=await invoke('model_file_snapshot');assert(snapshot.policy.channels.some(c=>c.connection.providerId===prior.providerId),'original022 cannot read its original channel on explicit rollback')
  report.oldOriginalChannelReadback=true
 }
 const screenshot=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,phase+'.png'),Buffer.from(screenshot.data,'base64'))
 report.result='pass'
}catch(error){report.error=error.stack;throw error}
finally{await writeFile(join(output,phase+'.json'),JSON.stringify(report,null,2)+'\n');ws.close();for(const p of pending.values())clearTimeout(p.timer)}
