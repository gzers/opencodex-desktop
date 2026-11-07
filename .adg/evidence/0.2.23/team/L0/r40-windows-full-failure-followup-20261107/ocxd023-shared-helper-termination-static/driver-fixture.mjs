import assert from 'node:assert/strict';
import {writeFile,readFile,open,rename,lstat,readdir,unlink} from 'node:fs/promises';
import {createHmac,randomBytes,timingSafeEqual} from 'node:crypto';
import {request as loopbackRequest} from 'node:http';
import {join,resolve} from 'node:path';
import {discoverMainTarget} from './windows-cdp.mjs';
const [endpoint,output,commandFile]=process.argv.slice(2);
let ws;let seq=0;const pending=new Map();
function call(method,params={},timeout=10000){const id=++seq;return new Promise((resolve,reject)=>{const timer=setTimeout(()=>{pending.delete(id);reject(Error('CDP deadline'));},timeout);pending.set(id,{resolve:v=>{clearTimeout(timer);resolve(v);},reject});ws.send(JSON.stringify({id,method,params}));});}
let runtimeStage='command';let runtimePublic={};
function runtimeFailureCode(error){
 const codes=new Map([['Actual service not healthy within production start budget','runtime_start_not_healthy'],['B claimed running on peer port','peer_port_claimed_running'],['B published an owned service record on A port','peer_port_owned_record'],['No explicit production port-conflict diagnostic','port_conflict_unproven'],['Manager did not confirm stop','manager_stop_not_confirmed'],['Stop did not clear own service record/state','runtime_stop_incomplete'],['Production attestation failed','runtime_attestation_failed'],['Runtime record changed','runtime_record_changed'],['Public manager command failed','public_manager_command_rejected']]);
 return codes.get(error?.message)||'runtime_binding_or_transport_failed';
}
const runtimeSleep=ms=>new Promise(r=>setTimeout(r,ms));
function fixtureConfig(port){assert(Number.isInteger(port)&&port>=1024&&port<=65535);return {port,hostname:'127.0.0.1',providers:{}};}
function publicRecord(record){
 assert(Number.isInteger(record.pid)&&record.pid>0&&Number.isInteger(record.port)&&record.port>=1024&&record.port<=65535,'Runtime public identity invalid');
 return {pid:record.pid,port:record.port};
}
function proofMatches(secret,challenge,pid,port,proof){
 if(typeof secret!=='string'||secret.length!==43||typeof proof!=='string')return false;
 const expected=createHmac('sha256',secret).update('opencodex-local-management-v1\n'+challenge+'\n'+pid+'\n'+port).digest();
 const received=Buffer.from(proof,'base64url');return received.length===expected.length&&timingSafeEqual(received,expected);
}
function runtimeSelfTest(){
 const secret=Buffer.alloc(32,7).toString('base64url'),challenge='fixture';
 const proof=createHmac('sha256',secret).update('opencodex-local-management-v1\nfixture\n42\n54321').digest('base64url');
 assert(proofMatches(secret,challenge,42,54321,proof));assert(!proofMatches(secret,challenge,43,54321,proof));
 assert(!proofMatches(secret,challenge,42,54322,proof));assert(!proofMatches(secret,challenge,42,54321,'bad'));
 assert.deepEqual(publicRecord({pid:42,port:54321,attestationSecret:secret}),{pid:42,port:54321});
 assert(!JSON.stringify(publicRecord({pid:42,port:54321,attestationSecret:secret})).includes(secret));
 assert.deepEqual(fixtureConfig(54321),{port:54321,hostname:'127.0.0.1',providers:{}});assert.throws(()=>fixtureConfig(0));
 assert.notEqual(Buffer.from(JSON.stringify(fixtureConfig(54321)),'utf8')[0],0xef);
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
 assert(same(c.home,join(process.env.USERPROFILE,'.codex-desktop/opencodex/home')),'Own real H required');
 assert(same(c.root,join(process.env.USERPROFILE,'.codex-desktop/desktop')),'Own real D required');
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
 const health=async()=>{
  const r=await record();assert(r&&r.port===c.port&&(!c.expectedPid||r.pid===c.expectedPid),'Runtime record changed');
  const challenge=randomBytes(32).toString('base64url');
  // Literal loopback, no proxies/redirects, no admin key or inference request.
  const response=await new Promise((accept,reject)=>{
   const req=loopbackRequest({hostname:'127.0.0.1',port:r.port,path:'/healthz',method:'GET',headers:{'x-opencodex-attestation-challenge':challenge}},res=>{
    let bytes=0;const chunks=[];res.on('data',chunk=>{bytes+=chunk.length;if(bytes>65536){res.destroy();reject(Error('Bounded health required'));}else chunks.push(chunk);});
    res.on('error',reject);res.on('end',()=>accept({status:res.statusCode,proof:res.headers['x-opencodex-attestation-proof'],body:Buffer.concat(chunks)}));
   });req.on('error',reject);req.setTimeout(5000,()=>req.destroy(Error('Loopback health deadline')));req.end();
  });
  assert(response.status===200&&proofMatches(r.attestationSecret,challenge,r.pid,r.port,response.proof),'Production attestation failed');
  const h=JSON.parse(response.body.toString('utf8'));assert(h.service==='opencodex'&&h.pid===r.pid&&h.port===r.port,'Health identity changed');
  const again=await record();assert(again&&again.pid===r.pid&&again.port===r.port&&again.attestationSecret===r.attestationSecret,'Runtime binding changed');
  return {record:publicRecord(r),attestation:'verified-production-challenge',healthService:h.service,version:typeof h.version==='string'?h.version:null};
 };
 const action=async action=>{
  await bound();const r=await runtimeInvoke('process_action',{request:{action,confirm:true}},action==='stop'?15000:35000);
  const value={lifecycleState:r.lifecycleState,result:r.result};runtimePublic.action=value;return value;
 };
 const statusConflict=s=>/conflict|port[_ -]?(in[_ -]?use|occupied)/i.test(String(s.protection))||s.runtime==='external_takeover';
 const diagnosticConflict=async()=>{
  // Read bounded own logs in memory; serialize only a boolean marker, never lines.
  for(const directory of [join(c.home,'logs'),join(c.root,'logs')]){
   let names;try{names=await readdir(directory);}catch(e){if(e.code==='ENOENT')continue;throw e;}
   for(const name of names.filter(n=>/\.log(?:\.1)?$/.test(n)).slice(0,32)){
    const path=join(directory,name),st=await lstat(path);if(!st.isFile()||st.isSymbolicLink()||st.size>8*1024*1024)continue;
    const text=await readFile(path,'utf8');if(/EADDRINUSE|端口.{0,30}(占用|冲突)|port.{0,60}(in use|conflict|occupied)/i.test(text))return true;
   }
  }return false;
 };
 runtimeStage='binding';await bound();
 let result={result:'pass-runtime-step',mode:c.mode,home:c.home,root:c.root,port:c.port};
 if(c.mode==='start'||c.mode==='conflict'){
  runtimeStage='own-config';assert(!(await record()),'Fresh service record required');
  for(const path of [process.env.USERPROFILE,join(process.env.USERPROFILE,'.codex-desktop'),join(process.env.USERPROFILE,'.codex-desktop/opencodex'),c.home]){
   const st=await lstat(path);assert(st.isDirectory()&&!st.isSymbolicLink(),'Own H has a link');
  }
  const configPath=join(c.home,'config.json'),temp=join(c.home,'config.json.two-user-'+randomBytes(12).toString('hex'));
  let old;try{const st=await lstat(configPath);assert(st.isFile()&&!st.isSymbolicLink()&&st.size<512*1024);old=JSON.parse(await readFile(configPath,'utf8'));}catch(e){if(e.code!=='ENOENT')throw e;}
  assert(!old||!old.providers||Object.keys(old.providers).length===0,'Fixture H contains providers');
  const handle=await open(temp,'wx',0o600);
  try{await handle.writeFile(JSON.stringify(fixtureConfig(c.port))+'\n','utf8');await handle.sync();}finally{await handle.close();}
  try{await rename(temp,configPath);}catch(e){await unlink(temp).catch(()=>{});throw e;}
  result.config={path:configPath,providers:0,utf8Bom:false,atomicRename:true};
  runtimeStage='manager-start';result.action=await action('start');
  const until=Date.now()+30000;let last;
  if(c.mode==='conflict'){
   let marker=false;
   do{assert(!(await record()),'B published an owned service record on A port');last=await snapshot();
    assert(last.runtime!=='running','B claimed running on peer port');marker=marker||statusConflict(last)||await diagnosticConflict();await runtimeSleep(250);
   }while(Date.now()<until);
   assert(marker,'No explicit production port-conflict diagnostic');
   result.conflict={rejected:true,ownedRecordAbsent:true,diagnosticMarker:true};result.status=last;
  }else{
   runtimeStage='wait-owned-health';let observed;
   do{try{observed=await health();last=await snapshot();if(last.runtime==='running'&&last.health==='healthy'&&Number(last.pid)===observed.record.pid&&last.port===c.port)break;}catch{}await runtimeSleep(250);}while(Date.now()<until);
   assert(observed&&last?.runtime==='running'&&last.health==='healthy'&&Number(last.pid)===observed.record.pid&&last.port===c.port,'Actual service not healthy within production start budget');
   result={...result,...observed,status:last};
  }
 }else if(c.mode==='observe'){
  runtimeStage='observe';result={...result,...await health(),status:await snapshot()};
  assert(result.status.runtime==='running'&&result.status.health==='healthy'&&Number(result.status.pid)===result.record.pid&&result.status.port===c.port,'Actual running state lost');
 }else{
  runtimeStage='before-stop';result.before=await health();
  runtimeStage='manager-stop';result.action=await action('stop');assert(result.action.result==='stopped','Manager did not confirm stop');
  const until=Date.now()+10000;let last;
  do{last=await snapshot();if(!(await record())&&last.runtime!=='running')break;await runtimeSleep(250);}while(Date.now()<until);
  assert(!(await record())&&last.runtime!=='running','Stop did not clear own service record/state');result.status=last;result.stopped=true;
 }
 runtimeStage='final-binding';await bound();
 await writeFile(join(output,'runtime-step.json'),JSON.stringify(result,null,2)+'\n');
}
if(endpoint==='--runtime-self-test'){runtimeSelfTest();process.exit(0);}
try{
 const target=await discoverMainTarget(endpoint,{timeoutMs:60000});
 const url=new URL(target.webSocketDebuggerUrl);assert(['127.0.0.1','localhost','[::1]'].includes(url.hostname));assert.equal(url.port,new URL(endpoint).port);
 ws=new WebSocket(target.webSocketDebuggerUrl);await new Promise((r,j)=>{ws.addEventListener('open',r,{once:true});ws.addEventListener('error',()=>j(Error('CDP unavailable')),{once:true});setTimeout(()=>j(Error('CDP open deadline')),10000).unref();});
 ws.addEventListener('message',e=>{const v=JSON.parse(e.data);const p=pending.get(v.id);if(p){pending.delete(v.id);v.error?p.reject(Error('CDP rejected')):p.resolve(v.result);}});
 if(commandFile){await runtimeCommand(JSON.parse(await readFile(commandFile,'utf8')));}
 else {
 let facts;
 for(let i=0;i<120;i++){
  const r=await call('Runtime.evaluate',{expression:'(async()=>{if(!window.__TAURI_INTERNALS__?.invoke||!document.querySelector("#app")?.children.length)return null;const i=window.__TAURI_INTERNALS__.invoke;return {mounted:true,textLength:document.body.innerText.length,about:await i("app_about"),paths:await i("model_paths_v2_snapshot"),config:await i("get_data_root_config"),preferences:await i("get_preferences"),runtime:await i("runtime_source")};})()',awaitPromise:true,returnByValue:true});
  if(!r.exceptionDetails&&r.result?.value?.mounted){facts=r.result.value;if(facts.paths.program.consistency==='consistent')break;}
  await new Promise(r=>setTimeout(r,1000));
 }
 assert(facts?.mounted&&facts.textLength>30,'Actual first screen unavailable');assert.equal(facts.about.platform,'Windows');
 const image=await call('Page.captureScreenshot',{format:'png'});await writeFile(join(output,'first-screen.png'),Buffer.from(image.data,'base64'));
 await writeFile(join(output,'first-screen.json'),JSON.stringify({mounted:true,textLength:facts.textLength,about:facts.about,paths:facts.paths,config:facts.config,cliEnabled:facts.preferences.cliEnabled,runtime:facts.runtime},null,2)+'\n');
 assert.equal(facts.paths.program.consistency,'consistent','NotReady: staged runtime unverified');
 assert.equal(facts.paths.runtimeEntry.status,'valid');assert.equal(facts.paths.runtimePrefix.status,'valid');
 }
}catch(error){
 if(commandFile){await writeFile(join(output,'runtime-step.json'),JSON.stringify({result:'fail',failureCode:runtimeFailureCode(error),stage:runtimeStage,publicFacts:runtimePublic})+'\n');process.exitCode=1;}else{throw error;}
}finally{ws?.close();}
