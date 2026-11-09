// Explicit local Sandbox only; no daily client directory or service mutation here.
import assert from 'node:assert/strict';
import {readFile,writeFile} from 'node:fs/promises';
import {join,win32} from 'node:path';
import {discoverMainTarget} from './windows-cdp.mjs';
const [endpoint,output,input]=process.argv.slice(2),c=JSON.parse(await readFile(input,'utf8'));
const target=await discoverMainTarget(endpoint,{timeoutMs:60000});
const ws=new WebSocket(target.webSocketDebuggerUrl),pending=new Map();let sequence=0;
await new Promise((resolve,reject)=>{ws.addEventListener('open',resolve,{once:true});ws.addEventListener('error',reject,{once:true})});
ws.addEventListener('message',e=>{const m=JSON.parse(e.data),p=pending.get(m.id);if(p){pending.delete(m.id);clearTimeout(p.timer);m.error?p.reject(Error('CDP rejected')):p.resolve(m.result)}});
function call(method,params={},budget=10000){return new Promise((resolve,reject)=>{const id=++sequence,timer=setTimeout(()=>{pending.delete(id);reject(Error('CDP deadline'))},budget);pending.set(id,{resolve,reject,timer});ws.send(JSON.stringify({id,method,params}))})}
async function evaluate(expression,budget=10000){const r=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true},budget);assert(!r.exceptionDetails,'Actual app command failed');return r.result?.value}
const invoke=async(name,args={},budget)=>{
 const value=await evaluate('window.__TAURI_INTERNALS__.invoke('+JSON.stringify(name)+','+JSON.stringify(args)+').catch(error=>({__commandError:{code:error?.code,message:error?.message}}))',budget);
 if(value?.__commandError)throw Error(JSON.stringify(value.__commandError));return value;
};
const normal=p=>{let v=win32.normalize(p);const slash=String.fromCharCode(92);if(v.startsWith(slash+slash+'?'+slash))v=v.slice(4);return v.toLowerCase()};
const same=(a,b)=>typeof a==='string'&&normal(a)===normal(b);
let result={accepted:false,diagnosticOnly:true,mode:c.mode};
try{
 const until=Date.now()+60000;
 while(!await evaluate('!!window.__TAURI_INTERNALS__?.invoke&&!!document.querySelector("#app")?.children.length')){assert(Date.now()<until,'Actual frontend not mounted');await new Promise(r=>setTimeout(r,200))}
 const config=await invoke('get_data_root_config');
 assert(same(config.currentDataRoot,join(c.fixtureRoot,'data/desktop')),'Sandbox D binding differs');
 assert(same(config.currentOpencodexHome,join(c.fixtureRoot,'data/opencodex/home')),'Sandbox H binding differs');
 result.config=config;
 if(c.mode==='prepare'){
  result.source=await invoke('runtime_source');
  assert(result.source.kind==='managed'&&result.source.version==='2.97.0-desktop023.lifecycle.3','Actual product-managed source is required');
  const paths=await invoke('model_paths_v2_snapshot');
  assert(paths.program.consistency==='consistent'&&paths.runtimeEntry.status==='valid'&&paths.runtimePrefix.status==='valid','Product source validation failed');
  result.paths=paths;
  const environment=await invoke('discover_environment',{request:null});
  result.environment={gate:environment.gate};assert(environment.gate==='ready','Environment is not ready');
 }else if(c.mode==='inspect'){
  const started=Date.now(),s=await invoke('get_status_snapshot',{},5500);
  result.status={elapsedMilliseconds:Date.now()-started,runtime:s.matrix.runtime,health:s.facts.health,pid:s.pid,port:s.port};
  assert(result.status.elapsedMilliseconds<=5000,'Original five second status budget failed');
 }else throw Error('Unadmitted fixture command');
 const image=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,'screen.png'),Buffer.from(image.data,'base64'));
 result.result='pass';
}catch(error){result.result='fail';result.error=error.message;process.exitCode=1}
finally{await writeFile(join(output,'rpc-result.json'),JSON.stringify(result,null,2)+String.fromCharCode(10),{flag:'wx'});ws.close();for(const p of pending.values())clearTimeout(p.timer)}
