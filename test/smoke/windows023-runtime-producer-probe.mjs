// Actual candidate CLI/proxy on synthetic temporary HOME, loopback-only model
// destination, no inference or business sync request. Not Desktop acceptance.
import assert from 'node:assert/strict';
import {dlopen,ptr,FFIType as T} from 'bun:ffi';
import {readFileSync,writeFileSync,mkdirSync,existsSync} from 'node:fs';
import {join,resolve,relative,isAbsolute} from 'node:path';
import {pathToFileURL} from 'node:url';
import {randomBytes,createHmac,timingSafeEqual} from 'node:crypto';
import {connect} from 'node:net';
import {once} from 'node:events';
assert(process.platform==='win32'&&process.arch==='x64'&&process.env.GITHUB_ACTIONS==='true');
const root=resolve(process.env.OCXD_PRODUCER_PROBE_ROOT),runtime=resolve(process.env.OCXD_PRODUCER_PROBE_PACKAGE),boundary=resolve(process.env.RUNNER_TEMP);
for(const p of [root,runtime]){const rel=relative(boundary,p);assert(rel&&!rel.startsWith('..')&&!isAbsolute(rel));}
const profile=join(root,'profile'),home=join(profile,'.codex-desktop/opencodex/home'),client=join(profile,'.codex');
assert(!existsSync(profile));mkdirSync(home,{recursive:true});mkdirSync(client);
writeFileSync(join(client,'config.toml'),'model_provider = "openai"\n');
writeFileSync(join(home,'config.json'),JSON.stringify({hostname:'127.0.0.1',port:0,openaiProviderTierVersion:2,defaultProvider:'producer-fixture',defaultModelAliases:false,providers:{'producer-fixture':{adapter:'openai-chat',baseUrl:'http://127.0.0.1:1/v1',allowPrivateNetwork:true,apiKey:'synthetic-unused',models:['producer-fixture-model'],selectedModels:['producer-fixture-model'],liveModels:false,defaultAliases:false}}}));
const expectedVersion=JSON.parse(readFileSync(join(runtime,'package.json'),'utf8')).version;
const osEnvironment=Object.fromEntries(Object.entries(process.env).filter(([key])=>['systemroot','windir','comspec','path','pathext','temp','tmp','github_actions'].includes(key.toLowerCase())));
const env={...osEnvironment,HOME:profile,USERPROFILE:profile,APPDATA:join(profile,'AppData/Roaming'),LOCALAPPDATA:join(profile,'AppData/Local'),OPENCODEX_HOME:home,CODEX_HOME:client,CLAUDE_CONFIG_DIR:join(profile,'.claude'),OCX_OWNER_REGISTRY_DIR:join(home,'owner-registry-fixture'),CI:'1',OCX_NO_DESKTOP_HANDOFF:'1'};
delete env.OCX_SERVICE;delete env.OCX_SERVICE_MANAGED;delete env.OPENCODEX_API_AUTH_TOKEN;
const child=Bun.spawn([process.execPath,'--no-env-file',join(runtime,'src/cli/index.ts'),'start'],{env,cwd:root,stdin:'ignore',stdout:'ignore',stderr:'ignore'});
child.unref();
const report={accepted:false,diagnosticOnly:true,source:process.env.GITHUB_SHA,producerVersion:expectedVersion,syntheticOnly:true,inferenceRequests:0,businessSyncRequests:0,actualCandidateCliStarted:true,startWithin30000:false,recordAndHealthAttestation:false,lifetimeResponseProof:false,officialStopWithin10000:false,officialClientRestoreConfirmed:false,originalProcessExit:false,controlledSetEmpty:false,leaseUnheld:false,recordAbsent:false,fixtureSafeToRemove:false};
const readRecord=()=>{const b=readFileSync(join(home,'runtime-port.json'));assert(b.length<=8192);return {bytes:b,value:JSON.parse(b)};};
const sleep=ms=>Bun.sleep(ms);
const start=performance.now();let captured,socket,job=0n,library;
async function request(method,path,token,budget=5000){
 assert(socket&&!socket.destroyed);const headers=['Host: 127.0.0.1:'+captured.value.port,'Connection: keep-alive','Accept: application/json'];
 const challenge=randomBytes(32).toString('base64url');headers.push('x-opencodex-attestation-challenge: '+challenge);if(token)headers.push('Authorization: Bearer '+token);if(method==='POST')headers.push('Content-Length: 0');
 const wire=method+' '+path+' HTTP/1.1\r\n'+headers.join('\r\n')+'\r\n\r\n';socket.write(wire);
 let buffer=Buffer.alloc(0),length=null,status=null,proof=null,lifetimeProof=null,bodyStart=0;
 const until=performance.now()+budget;
 while(performance.now()<until){
  let chunk;while((chunk=socket.read())!==null){buffer=Buffer.concat([buffer,chunk]);assert(buffer.length<=73728);}
  const split=buffer.indexOf('\r\n\r\n');
  if(split>=0&&length===null){const lines=buffer.subarray(0,split).toString('utf8').split('\r\n');assert(/^HTTP\/1.1 [0-9]{3}/.test(lines[0]));status=Number(lines[0].split(' ')[1]);const all=new Map();for(const line of lines.slice(1)){const n=line.indexOf(':');assert(n>0);const key=line.slice(0,n).toLowerCase();assert(!all.has(key));all.set(key,line.slice(n+1).trim());}assert(!all.has('transfer-encoding')&&!all.has('content-encoding'));length=Number(all.get('content-length'));assert(Number.isInteger(length)&&length>0&&length<=65536);proof=all.get('x-opencodex-attestation-proof');lifetimeProof=all.get('x-opencodex-runtime-lifetime-proof');bodyStart=split+4;}
  if(length!==null&&buffer.length>=bodyStart+length){assert.equal(buffer.length,bodyStart+length);return {status,proof,lifetimeProof,challenge,value:JSON.parse(buffer.subarray(bodyStart))};}
  assert(!socket.destroyed);await sleep(10);
 }
 throw Error('bounded-original-response-timeout');
}
function verify(response){
 const r=captured.value;assert.equal(response.status,200);assert.equal(response.value.service,'opencodex');assert.equal(response.value.pid,r.pid);assert.equal(response.value.port,r.port);assert.equal(response.value.version,expectedVersion);
 const expected=createHmac('sha256',r.attestationSecret).update('opencodex-local-management-v1\n'+response.challenge+'\n'+r.pid+'\n'+r.port).digest();const actual=Buffer.from(response.proof??'','base64url');assert(actual.length===expected.length&&timingSafeEqual(actual,expected));
 const b=r.desktopLifetime;assert(b&&b.schema==='ocxd-runtime-lifetime/1'&&b.pid===r.pid);
 const tuple=[b.schema,b.home,b.sid,b.pid,b.created,b.generation,b.job,b.leaseVolume,b.leaseIndex];
 const expectedLifetime=createHmac('sha256',r.attestationSecret).update('opencodex-runtime-lifetime-v1\n'+response.challenge+'\n'+r.port+'\n'+JSON.stringify(tuple)).digest();const actualLifetime=Buffer.from(response.lifetimeProof??'','base64url');assert(actualLifetime.length===expectedLifetime.length&&timingSafeEqual(actualLifetime,expectedLifetime));
 assert(readRecord().bytes.equals(captured.bytes));
}
try{
 let ready=false;
 while(performance.now()-start<30000){
  try{
   captured=readRecord();assert(captured.value.pid>0&&captured.value.port>0);
   socket=connect({host:'127.0.0.1',port:captured.value.port});socket.on('error',()=>{});await Promise.race([once(socket,'connect'),sleep(1000).then(()=>{throw Error('connect-timeout');})]);
   const health=await request('GET','/healthz');verify(health);report.recordAndHealthAttestation=true;report.lifetimeResponseProof=true;
   const readiness=await request('GET','/readyz');if(readiness.status===200&&readiness.value.status==='ready'&&readiness.value.pid===captured.value.pid){ready=true;break;}
  }catch{}
  if(socket)socket.destroy();await sleep(100);
 }
 report.startMilliseconds=performance.now()-start;assert(ready&&report.startMilliseconds<30000);report.startWithin30000=true;
 const {resolveTrustedWindowsSystemDirectory}=await import(pathToFileURL(join(runtime,'src/lib/windows-elevation.ts')).href);
 library=dlopen(join(resolveTrustedWindowsSystemDirectory(),'kernel32.dll'),{
  OpenJobObjectW:{args:[T.u32,T.i32,T.ptr],returns:T.u64},QueryInformationJobObject:{args:[T.u64,T.i32,T.ptr,T.u32,T.ptr],returns:T.i32},CreateFileW:{args:[T.ptr,T.u32,T.u32,T.ptr,T.u32,T.u32,T.u64],returns:T.u64},LockFileEx:{args:[T.u64,T.u32,T.u32,T.u32,T.u32,T.ptr],returns:T.i32},UnlockFileEx:{args:[T.u64,T.u32,T.u32,T.u32,T.ptr],returns:T.i32},CloseHandle:{args:[T.u64],returns:T.i32},
 });
 const wide=s=>{const value=new Uint16Array(s.length+1);for(let n=0;n<s.length;n++)value[n]=s.charCodeAt(n);return value;};
 const name=wide(captured.value.desktopLifetime.job);job=BigInt(library.symbols.OpenJobObjectW(4|0x20000,0,ptr(name)));assert(job!==0n);
 // Reattest on the same live TCP connection before a reusable admin token is sent.
 const stopStart=performance.now(),remaining=()=>{const left=10000-(performance.now()-stopStart);assert(left>0);return left;};
 verify(await request('GET','/healthz',undefined,Math.min(5000,remaining())));const token=readFileSync(join(home,'admin-api-token'),'utf8').trim();assert(/^ocx_admin_[A-Za-z0-9_-]{43}$/.test(token));
 const capabilities=await request('GET','/api/stop',token,Math.min(5000,remaining()));assert.equal(capabilities.status,200);assert.equal(capabilities.value.schema,'ocx-stop-capabilities/1');assert.equal(capabilities.value.connectedClientOwnership,true);assert.equal(capabilities.value.pid,captured.value.pid);
 verify(await request('GET','/healthz',undefined,Math.min(5000,remaining())));
 const result=await request('POST','/api/stop',token,remaining());assert.equal(result.status,200);assert.equal(result.value.success,true);assert.equal(result.value.sharedTeardown,'performed','Fresh fixture owns its restore obligation');report.officialClientRestoreConfirmed=true;
 const active=()=>{const storage=new ArrayBuffer(48),bytes=new Uint8Array(storage),returned=new Uint32Array(1);assert(library.symbols.QueryInformationJobObject(job,1,ptr(bytes),bytes.length,ptr(returned)));assert.equal(returned[0],48);return new DataView(storage).getUint32(40,true);};
 while(performance.now()-stopStart<10000){if(active()===0&&child.exitCode!==null&&!existsSync(join(home,'runtime-port.json'))&&!existsSync(join(home,'ocx.pid')))break;await sleep(25);}
 report.stopMilliseconds=performance.now()-stopStart;report.originalProcessExit=child.exitCode===0;report.controlledSetEmpty=active()===0;report.recordAbsent=!existsSync(join(home,'runtime-port.json'))&&!existsSync(join(home,'ocx.pid'));
 const leasePath=wide(join(home,'runtime-lifetime.lease'));const lease=BigInt(library.symbols.CreateFileW(ptr(leasePath),0xc0020000,3,null,3,0x80|0x200000,0n));assert(lease!==0n&&lease!==0xffffffffffffffffn);
 try{const overlap=new Uint8Array(new ArrayBuffer(32));assert(library.symbols.LockFileEx(lease,3,0,1,0,ptr(overlap)));report.leaseUnheld=true;assert(library.symbols.UnlockFileEx(lease,0,1,0,ptr(overlap)));}finally{library.symbols.CloseHandle(lease);}
 report.officialStopWithin10000=report.stopMilliseconds<10000&&report.originalProcessExit&&report.controlledSetEmpty&&report.recordAbsent&&report.leaseUnheld;
 report.fixtureSafeToRemove=report.originalProcessExit&&report.controlledSetEmpty&&report.recordAbsent&&report.leaseUnheld;
 assert(report.officialStopWithin10000);
}catch(error){report.errorType=error?.constructor?.name??'unknown';report.errorCode=error?.code??null;process.exitCode=1;}
finally{
 if(socket)socket.destroy();if(library){if(job!==0n)library.symbols.CloseHandle(job);library.close();}
 writeFileSync(join(root,'producer-report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify(report));
 // Unknown live instances are retained for exact owned cleanup, never killed
 // to obtain a successful Stop assertion or erased with their HOME.
}
