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
let runtimeStage='command';let runtimePublic={};
function runtimeFailureCode(error){
 const codes=new Map([['Actual service not healthy within production start budget','runtime_start_not_healthy'],['Start confirmation exceeded original 30-second budget','runtime_start_confirmation_timeout'],['B claimed running on peer port','peer_port_claimed_running'],['B published an owned service record on A port','peer_port_owned_record'],['No explicit production port-conflict diagnostic','port_conflict_unproven'],['Manager did not confirm stop','manager_stop_not_confirmed'],['Stop did not clear own service record/state','runtime_stop_incomplete'],['Production attestation failed','runtime_attestation_failed'],['Runtime record changed','runtime_record_changed'],['Public manager command failed','public_manager_command_rejected']]);
 return codes.get(error?.message)||'runtime_binding_or_transport_failed';
}
const runtimeSleep=ms=>new Promise(r=>setTimeout(r,ms));
function freshNativeClientTextRestored(text){
 return typeof text==='string'
  && !/^[ \t]*(?:openai_base_url|experimental_realtime_ws_base_url|model_catalog_json)\s*=/m.test(text)
  && !/^[ \t]*model_provider\s*=\s*["']opencodex["']/m.test(text);
}

function fixtureConfig(port){
 assert(Number.isInteger(port)&&(port===0||(port>=1024&&port<=65535)));
 const config={hostname:'127.0.0.1',openaiProviderTierVersion:2,defaultProvider:'sweep-fixture',defaultModelAliases:false,providers:{'sweep-fixture':{adapter:'openai-chat',baseUrl:'http://127.0.0.1:1/v1',allowPrivateNetwork:true,apiKey:'synthetic-unused',models:['sweep-fixture-model'],selectedModels:['sweep-fixture-model'],liveModels:false,defaultAliases:false}}};
 if(port!==0)config.port=port;return config;
}
function publicRecord(record){
 assert(Number.isInteger(record.pid)&&record.pid>0&&Number.isInteger(record.port)&&record.port>=1024&&record.port<=65535,'Runtime public identity invalid');
 return {pid:record.pid,port:record.port};
}
function proofMatches(secret,challenge,pid,port,proof){
 if(typeof secret!=='string'||secret.length!==43||typeof proof!=='string')return false;
 const expected=createHmac('sha256',secret).update('opencodex-local-management-v1\n'+challenge+'\n'+pid+'\n'+port).digest();
 const received=Buffer.from(proof,'base64url');return received.length===expected.length&&timingSafeEqual(received,expected);
}
function assertConflictAction(action){
 assert(action?.failureCode==='port_conflict'&&action.result===null&&action.lifecycleState==='starting_failed'&&action.canStop===false&&action.canRestart===false,'Actual manager conflict rejection not proved');
}
function runtimeSelfTest(){
 const secret=Buffer.alloc(32,7).toString('base64url'),challenge='fixture';
 const proof=createHmac('sha256',secret).update('opencodex-local-management-v1\nfixture\n42\n54321').digest('base64url');
 assert(proofMatches(secret,challenge,42,54321,proof));assert(!proofMatches(secret,challenge,43,54321,proof));
 assert(!proofMatches(secret,challenge,42,54322,proof));assert(!proofMatches(secret,challenge,42,54321,'bad'));
 assert.deepEqual(publicRecord({pid:42,port:54321,attestationSecret:secret}),{pid:42,port:54321});
 assert(!JSON.stringify(publicRecord({pid:42,port:54321,attestationSecret:secret})).includes(secret));
 assert.equal(fixtureConfig(54321).port,54321);assert(!Object.hasOwn(fixtureConfig(0),'port'));
 assert.equal(fixtureConfig(0).defaultProvider,'sweep-fixture');assert.equal(Object.keys(fixtureConfig(0).providers).length,1);assert.equal(fixtureConfig(0).providers['sweep-fixture'].liveModels,false);assert.throws(()=>fixtureConfig(-1));
 assert.notEqual(Buffer.from(JSON.stringify(fixtureConfig(54321)),'utf8')[0],0xef);
 const conflict={failureCode:'port_conflict',result:null,lifecycleState:'starting_failed',canStop:false,canRestart:false};
 assertConflictAction(conflict);
 for(const changed of [{failureCode:null},{result:'started'},{lifecycleState:'running'},{canStop:true},{canRestart:true}])assert.throws(()=>assertConflictAction({...conflict,...changed}));
 assert(freshNativeClientTextRestored(''));assert(freshNativeClientTextRestored('model_provider = "openai"\n'));
 for(const key of ['openai_base_url','experimental_realtime_ws_base_url','model_catalog_json'])assert(!freshNativeClientTextRestored(key+' = "stale"\n'));
 assert(!freshNativeClientTextRestored('model_provider = "opencodex"\n'));
 console.log('pass-runtime-pure-fixtures-only');
}
async function runtimeInvoke(command,args={},timeout=10000){
 const expression='window.__TAURI_INTERNALS__.invoke('+JSON.stringify(command)+','+JSON.stringify(args)+')';
 const r=await call('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true},timeout);
 assert(!r.exceptionDetails,'Public manager command failed');return r.result?.value;
}
async function runtimeCommand(c){
 assert(['start','conflict','observe','stop'].includes(c.mode));
 runtimePublic={mode:c.mode,port:c.port};
 const normalized=p=>{let v=resolve(p);if(v.charCodeAt(0)===92&&v.charCodeAt(1)===92&&v.charCodeAt(2)===63&&v.charCodeAt(3)===92)v=v.slice(4);return v.toLowerCase();};
 const same=(a,b)=>typeof a==='string'&&typeof b==='string'&&normalized(a)===normalized(b);
 assert(same(c.home,join(config.fixtureRoot,'data/opencodex/home')),'Own real H required');
 assert(same(c.root,join(config.fixtureRoot,'data/desktop')),'Own real D required');
 const bound=async()=>{
  const config=await runtimeInvoke('get_data_root_config');const paths=await runtimeInvoke('model_paths_v2_snapshot');
  assert(same(config.currentOpencodexHome,c.home)&&same(config.currentDataRoot,c.root),'Manager H/D binding changed');
  assert(same(paths.opencodexHome.path,c.home)&&same(paths.desktopRoot.path,c.root),'Snapshot H/D binding changed');
  assert(paths.program.consistency==='consistent'&&paths.runtimeEntry.status==='valid'&&paths.runtimePrefix.status==='valid','Actual managed runtime not valid');
  return paths;
 };
 const snapshot=async()=>{
  const s=await runtimeInvoke('get_status_snapshot');
  // Official status may expose paths.config (a FILE) as opencodex_home and
  // omit dataRoot. Verify supplied facts exactly; missing facts gain no authority.
  await bound();assert(s.facts,'Status facts absent');
  assert(!s.facts.opencodex_home||same(s.facts.opencodex_home,c.home)||same(s.facts.opencodex_home,join(c.home,'config.json')),'Status H binding differs');
  assert(!s.facts.data_root||same(s.facts.data_root,c.root)||same(s.facts.data_root,c.home),'Status root binding differs');
  const value={runtime:s.matrix.runtime,health:s.facts.health,source:s.source,pid:s.pid,port:s.port,protection:s.facts.protection,startupStatus:s.facts.startup_status,servicePresent:s.facts.service_present,rebootSafe:s.facts.reboot_safe};
  runtimePublic.status=value;return value;
 };
 const recordPath=join(c.home,'runtime-port.json');
 const record=async()=>{
  let st;try{st=await lstat(recordPath);}catch(e){if(e.code==='ENOENT')return null;throw e;}
  assert(st.isFile()&&!st.isSymbolicLink()&&st.size<=4096,'Unsafe runtime record');
  const r=JSON.parse(await readFile(recordPath,'utf8'));runtimePublic.record=publicRecord(r);return r;
 };
 const requestedPort=c.port;
 const health=async()=>{
  const r=await record();
  if(c.mode==='start'&&requestedPort===0&&c.port===0&&r){c.port=r.port;runtimePublic.automaticPort={requested:'omitted',selected:r.port,source:'actual-opencodex-runtime-record'};runtimePublic.port=r.port;}
  assert(r&&r.port===c.port&&(!c.expectedPid||r.pid===c.expectedPid),'Runtime record changed');
  const challenge=randomBytes(32).toString('base64url');
  // Literal loopback, no proxies/redirects, no admin key or inference request.
  const response=await new Promise((accept,reject)=>{
   const req=loopbackRequest({hostname:'127.0.0.1',port:r.port,path:'/healthz',method:'GET',headers:{'x-opencodex-attestation-challenge':challenge}},res=>{
    let bytes=0;const chunks=[];res.on('data',chunk=>{bytes+=chunk.length;if(bytes>65536){res.destroy();reject(Error('Bounded health required'));}else chunks.push(chunk);});
    res.on('error',reject);res.on('end',()=>accept({status:res.statusCode,proof:res.headers['x-opencodex-attestation-proof'],body:Buffer.concat(chunks)}));
   });req.on('error',reject);req.setTimeout(5000,()=>req.destroy(Error('Loopback health deadline')));req.end();
  });
  runtimePublic.health={status:response.status,proofPresent:typeof response.proof==='string',attestationVerified:proofMatches(r.attestationSecret,challenge,r.pid,r.port,response.proof)};
  assert(response.status===200&&runtimePublic.health.attestationVerified,'Production attestation failed');
  const h=JSON.parse(response.body.toString('utf8'));assert(h.service==='opencodex'&&h.pid===r.pid&&h.port===r.port,'Health identity changed');
  const again=await record();assert(again&&again.pid===r.pid&&again.port===r.port&&again.attestationSecret===r.attestationSecret,'Runtime binding changed');
  return {record:publicRecord(r),attestation:'verified-production-challenge',healthService:h.service,version:typeof h.version==='string'?h.version:null};
 };
 const diagnosticStatus=async()=>{
  const paths=await bound(),install=resolve(paths.runtimePrefix.path,'../..'),node=join(install,'runtime/node/node.exe');
  const packageRoot=join(paths.runtimePrefix.path,'node_modules/@bitkyc08/opencodex');
  const declaration=JSON.parse(await readFile(join(packageRoot,'package.json'),'utf8'));
  const relative=typeof declaration.bin==='string'?declaration.bin:declaration.bin?.ocx;
  assert(typeof relative==='string'&&!relative.includes('..')&&!relative.startsWith('/'),'Diagnostic actual package bin required');
  const script=join(packageRoot,relative);
  const readStatus=async(program,args,budget)=>new Promise(resolveResult=>{
   const env={HOME:process.env.USERPROFILE,USERPROFILE:process.env.USERPROFILE,OPENCODEX_HOME:c.home,CODEX_HOME:join(c.home,'diagnostic-unused-codex'),PATH:join(install,'runtime/node'),CI:'1',OCX_NO_DESKTOP_HANDOFF:'1'};
   for(const key of ['SystemRoot','WINDIR','COMSPEC','TEMP','TMP','APPDATA','LOCALAPPDATA'])if(process.env[key])env[key]=process.env[key];
   const child=diagnosticSpawn(program,args,{env,stdio:['ignore','pipe','ignore'],cwd:c.home,windowsHide:true});
   let output='',overflow=false;const started=Date.now();
   const timer=setTimeout(()=>child.kill(),budget);
   child.stdout.on('data',chunk=>{if(Buffer.byteLength(output)+chunk.length>1024*1024){overflow=true;child.kill();}else output+=chunk.toString('utf8');});
   child.on('error',()=>{clearTimeout(timer);resolveResult({result:'spawn-failed',accepted:false});});
   child.on('close',(code,signal)=>{clearTimeout(timer);let status;try{status=JSON.parse(output);}catch{}
    resolveResult({exitCode:code,signal,elapsedMilliseconds:Date.now()-started,outputOverflow:overflow,parsed:!!status,proxy:status?.proxy?{running:status.proxy.running,pid:status.proxy.pid,port:status.proxy.port}:null,startup:status?.startup?{status:status.startup.status}:null,accepted:false});});
  });
  const result=await readStatus(node,[script,'status','--json'],5000);
  await writeFile(join(output,'actual-readonly-status-diagnostic.json'),JSON.stringify(result)+'\n');
  // Readonly comparisons only AFTER the unchanged production gate has failed.
  // Direct bundled Bun helps distinguish launcher/engine startup from status
  // collection itself; neither this nor the late result can admit a service.
  const bun=join(install,'runtime/bun/bun.exe'),entry=join(packageRoot,'src/cli/index.ts');
  const direct=await readStatus(bun,['--no-env-file',entry,'status','--json'],5000);
  await writeFile(join(output,'direct-bun-status-diagnostic.json'),JSON.stringify(direct)+'\n');
  const late=await readStatus(node,[script,'status','--json'],15000);
  await writeFile(join(output,'late-node-status-diagnostic.json'),JSON.stringify({...late,originalBudgetPassed:false})+'\n');
 };
 const diagnosticReadiness=async()=>{
  // The real thirty-second attempt already failed. These bounded readonly
  // observations cannot accept it or authorize the next user service.
  const samples=[];const until=Date.now()+15000;
  do{
   try{
    const verified=await health();
    const response=await new Promise((accept,reject)=>{
     const req=loopbackRequest({hostname:'127.0.0.1',port:c.port,path:'/readyz',method:'GET'},res=>{let bytes=0;const chunks=[];res.on('data',chunk=>{bytes+=chunk.length;if(bytes>65536){res.destroy();reject(Error('bounded readiness'))}else chunks.push(chunk)});res.on('error',reject);res.on('end',()=>accept({code:res.statusCode,body:Buffer.concat(chunks)}))});
     req.on('error',reject);req.setTimeout(1000,()=>req.destroy(Error('readonly probe timeout')));req.end();
    });
    const body=JSON.parse(response.body.toString('utf8'));
    samples.push({at:Date.now(),code:response.code,identity:body.service==='opencodex'&&body.pid===verified.record.pid&&body.port===c.port,status:['ready','pending','failed'].includes(body.status)?body.status:'invalid'});
   }catch{samples.push({at:Date.now(),status:'observation-failed'})}
   await runtimeSleep(2000);
  }while(Date.now()<until);
  const files={};
  for(const [name,path] of [['codex',join(process.env.USERPROFILE,'.codex')],['claude',join(process.env.USERPROFILE,'.claude')],['runtimeHome',c.home]]){
   try{files[name]=(await readdir(path)).filter(n=>/^[a-zA-Z0-9_.-]{1,80}$/.test(n)).slice(0,64)}catch{files[name]=[]}
  }
  await writeFile(join(output,'late-readiness-diagnostic.json'),JSON.stringify({samples,files,accepted:false,originalBudgetPassed:false})+'\n');
 };
 const action=async action=>{
  await bound();const r=await runtimeInvoke('process_action',{request:{action,confirm:true}},action==='stop'?15000:35000);
  const value={lifecycleState:r.lifecycleState,result:r.result,failureCode:r.failure?.code??null,canStop:r.canStop,canRestart:r.canRestart};runtimePublic.action=value;return value;
 };
 runtimeStage='binding';await bound();
 let result={result:'pass-runtime-step',mode:c.mode,home:c.home,root:c.root,port:c.port};
 if(c.mode==='start'||c.mode==='conflict'){
  runtimeStage='own-config';assert(!(await record()),'Fresh service record required');
  // Default native integration requires an initialized Codex home directory.
  // Each new SID owns this EMPTY fixture; the real runtime must create its own
  // config/catalog and become ready. Never seed artifacts or disable integration.
  const clientHome=join(process.env.USERPROFILE,'.codex');
  try{const st=await lstat(clientHome);assert(st.isDirectory()&&!st.isSymbolicLink(),'Unsafe own Codex fixture')}
  catch(error){if(error.code!=='ENOENT')throw error;await mkdir(clientHome,{mode:0o700})}
  result.clientHomeFixture={emptyDirectoryInitialized:true,configSeeded:false,catalogSeeded:false,integrationDisabled:false};
  for(const path of [config.fixtureRoot,join(config.fixtureRoot,'data'),join(config.fixtureRoot,'data/opencodex'),c.home]){
   const st=await lstat(path);assert(st.isDirectory()&&!st.isSymbolicLink(),'Own H has a link');
  }
  const configPath=join(c.home,'config.json'),temp=join(c.home,'config.json.two-user-'+randomBytes(12).toString('hex'));
  let old;try{const st=await lstat(configPath);assert(st.isFile()&&!st.isSymbolicLink()&&st.size<512*1024);old=JSON.parse(await readFile(configPath,'utf8'));}catch(e){if(e.code!=='ENOENT')throw e;}
  assert(!old||!old.providers||Object.keys(old.providers).length===0||(Object.keys(old.providers).length===1&&JSON.stringify(old.providers)===JSON.stringify(fixtureConfig(0).providers)),'Fixture H contains unexpected providers');
  const handle=await open(temp,'wx',0o600);
  try{await handle.writeFile(JSON.stringify(fixtureConfig(c.port))+'\n','utf8');await handle.sync();}finally{await handle.close();}
  try{await rename(temp,configPath);}catch(e){await unlink(temp).catch(()=>{});throw e;}
  result.config={path:configPath,providers:1,syntheticOnly:true,liveModels:false,utf8Bom:false,atomicRename:true};
  runtimeStage='manager-start';const startBegan=Date.now();result.action=await action('start');
  const until=startBegan+30000;let last;
  if(c.mode==='conflict'){
   // The actual public action has already rejected a fixed peer-port bind.
   // With no owned endpoint, full CLI status cold-start is a separate diagnostic;
   // it cannot replace the action/record/peer OS identity proof or grant success.
   assertConflictAction(result.action);
   do{assert(!(await record()),'B published an owned service record on A port');await runtimeSleep(250);}while(Date.now()<until);
   assert(!(await record()),'B published an owned service record on A port');
   result.conflict={rejected:true,ownedRecordAbsent:true,diagnosticMarker:true,source:'actual-process-action-port_conflict',canStop:false,canRestart:false};
   result.status={source:'actual-conflict-action-and-absent-owned-record',lifecycleState:result.action.lifecycleState,claimedRunning:false};
  }else{
   runtimeStage='wait-owned-health';let observed;
   const samples=[];
   do{try{observed=await health();last=await snapshot();samples.push({at:Date.now(),elapsedMilliseconds:Date.now()-startBegan,health:runtimePublic.health,status:last});if(last.runtime==='running'&&last.health==='healthy'&&Number(last.pid)===observed.record.pid&&last.port===c.port)break;}
    catch(error){samples.push({at:Date.now(),health:runtimePublic.health??null,error:runtimeFailureCode(error)});}
    await runtimeSleep(250);
   }while(Date.now()<until);
   await writeFile(join(output,'owned-health-observations.json'),JSON.stringify({samples:samples.slice(0,64),originalBudgetMilliseconds:30000,startAction:result.action,totalMilliseconds:Date.now()-startBegan,accepted:false})+'\n');
   const startElapsedMilliseconds=Date.now()-startBegan;
   runtimePublic.startBudget={budgetMilliseconds:30000,elapsedMilliseconds:startElapsedMilliseconds};
   const startBudgetPassed=result.action.result==='started'&&startElapsedMilliseconds<=30000;
   if(!(startBudgetPassed&&observed&&last?.runtime==='running'&&last.health==='healthy'&&Number(last.pid)===observed.record.pid&&last.port===c.port)){
    // Additional readonly diagnosis follows the already failed original budget;
    // its result cannot make this attempt pass or authorize the next service.
    await writeFile(join(output,'production-attempt-failed.json'),JSON.stringify({accepted:false,originalBudgetPassed:false,budgetMilliseconds:30000})+'\n');
    const confirmedLate=startElapsedMilliseconds>30000&&result.action.result==='started'&&observed&&last?.runtime==='running'&&last.health==='healthy'&&Number(last.pid)===observed.record.pid&&last.port===c.port;
    throw Error(confirmedLate?'Start confirmation exceeded original 30-second budget':'Actual service not healthy within production start budget');
   }
   result={...result,...observed,status:last,port:c.port,startBudget:{budgetMilliseconds:30000,elapsedMilliseconds:startElapsedMilliseconds},automaticPort:requestedPort===0?runtimePublic.automaticPort:null};
   const clientConfig=join(process.env.USERPROFILE,'.codex/config.toml');
   const clientStat=await lstat(clientConfig);assert(clientStat.isFile()&&!clientStat.isSymbolicLink()&&clientStat.size>0&&clientStat.size<=512*1024,'Actual Codex config injection missing');
   const injected=await readFile(clientConfig,'utf8');assert(injected.includes('openai_base_url')&&injected.includes(':'+c.port),'Codex routing did not use actual assigned port');
   result.clientHomeFixture={...result.clientHomeFixture,actualConfigWritten:true,actualAssignedPortBound:true};
  }
 }else if(c.mode==='observe'){
  runtimeStage='observe';result={...result,...await health(),status:await snapshot()};
  assert(result.status.runtime==='running'&&result.status.health==='healthy'&&Number(result.status.pid)===result.record.pid&&result.status.port===c.port,'Actual running state lost');
 }else{
  runtimeStage='before-stop';result.before=await health();
  const stopBegan=Date.now();runtimeStage='manager-stop';result.action=await action('stop');assert(result.action.result==='stopped','Manager did not confirm stop');
  const until=stopBegan+10000;let last;
  do{last=await snapshot();if(!(await record())&&last.runtime!=='running')break;await runtimeSleep(250);}while(Date.now()<until);
  assert(!(await record())&&last.runtime!=='running','Stop did not clear own service record/state');result.status=last;result.stopped=true;
  const nativeConfig=join(process.env.USERPROFILE,'.codex/config.toml');let nativeText='';
  try{const st=await lstat(nativeConfig);assert(st.isFile()&&!st.isSymbolicLink()&&st.size<=512*1024,'Unsafe fresh native client readback');nativeText=await readFile(nativeConfig,'utf8');}catch(error){if(error.code!=='ENOENT')throw error;}
  assert(freshNativeClientTextRestored(nativeText),'Native client routing/catalog fields survived official Stop');
  result.nativeClientRestoredReadback=true;result.stopBudget={budgetMilliseconds:10000,elapsedMilliseconds:Date.now()-stopBegan};
  assert(result.stopBudget.elapsedMilliseconds<=10000,'Stop confirmation exceeded original shared budget');

 }
 runtimeStage='final-binding';await bound();
 await writeFile(join(output,'runtime-step.json'),JSON.stringify(result,null,2)+'\n');
}
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
 if(config.coordination){
  const coordinated=async(name,value={})=>writeFile(join(output,name+'.json'),JSON.stringify({source:config.commit,...outcome.start,...value})+nl,{flag:'wx'});
  const waitPhase=async(name)=>{
   const deadline=Date.now()+180000;
   while(true){
    try{await lstat(join(output,'abort'));throw Error('Controller aborted peer protocol')}catch(error){if(error.code!=='ENOENT')throw error}
    try{await lstat(join(output,name));return}catch(error){if(error.code!=='ENOENT')throw error}
    assert(Date.now()<deadline,'Peer phase envelope elapsed; no lifecycle retry');
    await new Promise(resolve=>setTimeout(resolve,100));
   }
  };
  await coordinated('started');
  await waitPhase('observe-overlap');
  await runtimeCommand({mode:'observe',...outcome.start,expectedPid:outcome.start.pid,root:config.root,home:config.home});
  await rename(join(output,'runtime-step.json'),join(output,'overlap-runtime-step.json'));
  await coordinated('overlap-confirmed');
  if(config.coordination.role===1){
   await waitPhase('observe-after-peer-stop');
   await runtimeCommand({mode:'observe',...outcome.start,expectedPid:outcome.start.pid,root:config.root,home:config.home});
   await rename(join(output,'runtime-step.json'),join(output,'survivor-runtime-step.json'));
   await coordinated('survivor-confirmed');
  }
  await waitPhase('stop-now');
 }
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
