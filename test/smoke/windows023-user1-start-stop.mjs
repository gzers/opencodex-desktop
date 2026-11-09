// One real Desktop Start, then only on unchanged proof an immediate normal Stop.
import assert from 'node:assert/strict';
import {readFile,writeFile,open,rename,lstat,readdir,unlink,mkdir} from 'node:fs/promises';
import {createHmac,randomBytes,timingSafeEqual} from 'node:crypto';
import {request as loopbackRequest} from 'node:http';
import {spawn as diagnosticSpawn} from 'node:child_process';
import {join,resolve} from 'node:path';
import {discoverMainTarget} from './windows-cdp.mjs';
const [endpoint,output,input]=process.argv.slice(2),config=JSON.parse(await readFile(input,'utf8'));
let ws,seq=0;const pending=new Map(),trace=[],actions=[];
const epoch=performance.now(),nl=String.fromCharCode(10);
function note(stage,extra={}){trace.push({sequence:trace.length+1,stage,elapsedMilliseconds:performance.now()-epoch,atUtc:new Date().toISOString(),...extra})}
function call(method,params={},timeout=10000){const id=++seq;return new Promise((resolve,reject)=>{const timer=setTimeout(()=>{pending.delete(id);reject(Error('CDP deadline'))},timeout);pending.set(id,{resolve:v=>{clearTimeout(timer);resolve(v)},reject});ws.send(JSON.stringify({id,method,params}))})}
/*USER1_RUNTIME_CONTROL*/
const originalRuntimeInvoke=runtimeInvoke;
runtimeInvoke=async(command,args={},timeout=10000)=>{
 note('invoke-begin',{command,action:args.request?.action??null});
 try{
  if(command==='process_action'){
   const expression='window.__TAURI_INTERNALS__.invoke('+JSON.stringify(command)+','+JSON.stringify(args)+').then(value=>({raw:JSON.stringify(value),value}),error=>({rejected:JSON.stringify(error)}))';
   const answer=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true},timeout);
   assert(!answer.exceptionDetails,'Public manager command failed');
   const response=answer.result?.value,n=actions.length+1;
   actions.push({sequence:n,action:args.request.action,elapsedMilliseconds:performance.now()-epoch,response});
   await writeFile(join(output,'action-'+String(n).padStart(3,'0')+'.json'),JSON.stringify(actions.at(-1),null,2)+nl,{flag:'wx'});
   if(response?.raw!==undefined)await writeFile(join(output,'action-'+String(n).padStart(3,'0')+'.raw.json'),response.raw,{flag:'wx'});
   note('action-return',{action:args.request.action,sequence:n,rejected:response?.rejected??null});
   if(response?.rejected)throw Error('Public manager command failed');return response.value;
  }
  const value=await originalRuntimeInvoke(command,args,command==='get_status_snapshot'?5000:timeout);
  note('invoke-return',{command});return value;
 }catch(error){note('invoke-failed',{command,error:error.message});throw error}
};
const outcome={accepted:false,startConfirmed:false,normalStopExecuted:false,source:config.commit};
try{
 const target=await discoverMainTarget(endpoint,{timeoutMs:60000});
 assert.equal(new URL(target.webSocketDebuggerUrl).port,new URL(endpoint).port);
 ws=new WebSocket(target.webSocketDebuggerUrl);
 await new Promise((r,j)=>{ws.addEventListener('open',r,{once:true});ws.addEventListener('error',()=>j(Error('CDP unavailable')),{once:true});setTimeout(()=>j(Error('CDP open deadline')),10000).unref()});
 ws.addEventListener('message',e=>{const v=JSON.parse(e.data),p=pending.get(v.id);if(p){pending.delete(v.id);v.error?p.reject(Error('CDP rejected')):p.resolve(v.result)}});
 const paths=await runtimeInvoke('model_paths_v2_snapshot');assert.equal(paths.program.consistency,'consistent');
 const same=p=>resolve(p).replaceAll(String.fromCharCode(92),'/').replace('//?/','').toLowerCase();
 assert.equal(same(paths.desktopRoot.path),same(config.root));assert.equal(same(paths.opencodexHome.path),same(config.home));
 await writeFile(join(output,'actual-paths.json'),JSON.stringify(paths,null,2)+nl,{flag:'wx'});
 note('start-proof-begin');
 await runtimeCommand({mode:'start',port:0,expectedPid:0,root:config.root,home:config.home});
 const started=JSON.parse(await readFile(join(output,'runtime-step.json'),'utf8'));
 await rename(join(output,'runtime-step.json'),join(output,'start-runtime-step.json'));
 outcome.startConfirmed=started.result==='pass-runtime-step';assert(outcome.startConfirmed);
 outcome.start={pid:started.record.pid,port:started.record.port};note('start-confirmed',outcome.start);
 // No peer or retry between successful proof and the original Stop command.
 outcome.normalStopExecuted=true;note('normal-stop-begin');
 await runtimeCommand({mode:'stop',port:started.record.port,expectedPid:started.record.pid,root:config.root,home:config.home});
 await rename(join(output,'runtime-step.json'),join(output,'stop-runtime-step.json'));
 outcome.normalStopConfirmed=true;note('normal-stop-confirmed');
}catch(error){
 outcome.failure={stage:runtimeStage,message:error.message,publicFacts:runtimePublic};
 note('first-failure',{stage:runtimeStage,message:error.message});
 if(!outcome.startConfirmed)outcome.normalStopExecuted=false;process.exitCode=1;
}finally{
 outcome.startReceiptCount=actions.filter(a=>a.action==='start').length;
 outcome.successfulStartReceiptCount=actions.filter(a=>a.action==='start'&&a.response?.value?.result==='started').length;
 outcome.uniqueSuccessfulStartReceipt=outcome.startReceiptCount===1&&outcome.successfulStartReceiptCount===1;
 outcome.receipts=actions;outcome.trace=trace;
 await writeFile(join(output,'user1-result.json'),JSON.stringify(outcome,null,2)+nl,{flag:'wx'});
 ws?.close();for(const p of pending.values())clearTimeout(p.timer);
 // Retain this driver generation for parent-side native identity collection.
 if(config.driverRelease){const until=Date.now()+10000;while(true){try{await lstat(config.driverRelease);break}catch{if(Date.now()>until)break;await new Promise(r=>setTimeout(r,20))}}}
}
