// Test-only lifetime proof. Own synthetic processes, no proxy/client/service.
// No TerminateProcess, terminate-job or kill-on-close flag is used.
import assert from 'node:assert/strict';
import {dlopen,ptr,read,FFIType as T} from 'bun:ffi';
import {readFileSync,writeFileSync,existsSync,renameSync} from 'node:fs';
import {join,resolve,relative,isAbsolute} from 'node:path';
import {pathToFileURL} from 'node:url';
import {randomUUID,createHmac,timingSafeEqual,randomBytes} from 'node:crypto';
assert(process.platform==='win32'&&process.arch==='x64'&&process.env.GITHUB_ACTIONS==='true');
const root=resolve(process.env.OCXD_LEASE_PROBE_ROOT),boundary=resolve(process.env.RUNNER_TEMP);
const rel=relative(boundary,root);assert(rel&&!rel.startsWith('..')&&!isAbsolute(rel));
const runtime=resolve(process.env.OCXD_RECORD_PROBE_PACKAGE);
const {resolveTrustedWindowsSystemDirectory}=await import(pathToFileURL(join(runtime,'src/lib/windows-elevation.ts')).href);
const {resolveWindowsCurrentSid}=await import(pathToFileURL(join(runtime,'src/codex/windows-current-sid.ts')).href);
const currentSid=resolveWindowsCurrentSid(()=>{throw Error('Native SID required');});assert(/^S-1-[0-9-]+$/.test(currentSid));
const system=resolveTrustedWindowsSystemDirectory();
const lib=dlopen(join(system,'kernel32.dll'),{
 CreateFileW:{args:[T.ptr,T.u32,T.u32,T.ptr,T.u32,T.u32,T.u64],returns:T.u64},
 CloseHandle:{args:[T.u64],returns:T.i32},
 LockFileEx:{args:[T.u64,T.u32,T.u32,T.u32,T.u32,T.ptr],returns:T.i32},
 UnlockFileEx:{args:[T.u64,T.u32,T.u32,T.u32,T.ptr],returns:T.i32},
 CreateJobObjectW:{args:[T.ptr,T.ptr],returns:T.u64},
 OpenJobObjectW:{args:[T.u32,T.i32,T.ptr],returns:T.u64},
 AssignProcessToJobObject:{args:[T.u64,T.u64],returns:T.i32},
 IsProcessInJob:{args:[T.u64,T.u64,T.ptr],returns:T.i32},
 QueryInformationJobObject:{args:[T.u64,T.i32,T.ptr,T.u32,T.ptr],returns:T.i32},
 CreateProcessW:{args:[T.ptr,T.ptr,T.ptr,T.ptr,T.i32,T.u32,T.ptr,T.ptr,T.ptr,T.ptr],returns:T.i32},
 ResumeThread:{args:[T.u64],returns:T.u32},
 GetProcessId:{args:[T.u64],returns:T.u32},
 GetCurrentProcess:{args:[],returns:T.u64},
 OpenProcess:{args:[T.u32,T.i32,T.u32],returns:T.u64},
 GetProcessTimes:{args:[T.u64,T.ptr,T.ptr,T.ptr,T.ptr],returns:T.i32},
 GetExitCodeProcess:{args:[T.u64,T.ptr],returns:T.i32},
 WaitForSingleObject:{args:[T.u64,T.u32],returns:T.u32},
 GetLastError:{args:[],returns:T.u32},
 LocalFree:{args:[T.u64],returns:T.u64},
 lstrlenW:{args:[T.u64],returns:T.i32},
 lstrcpynW:{args:[T.ptr,T.u64,T.i32],returns:T.ptr},
});
const k=lib.symbols;
const security=dlopen(join(system,'advapi32.dll'),{
 ConvertStringSecurityDescriptorToSecurityDescriptorW:{args:[T.ptr,T.u32,T.ptr,T.ptr],returns:T.i32},
 ConvertSidToStringSidW:{args:[T.u64,T.ptr],returns:T.i32},
 GetAce:{args:[T.u64,T.u32,T.ptr],returns:T.i32},
 IsValidAcl:{args:[T.u64],returns:T.i32},
 GetSecurityInfo:{args:[T.u64,T.i32,T.u32,T.ptr,T.ptr,T.ptr,T.ptr,T.ptr],returns:T.u32},
 GetSecurityDescriptorControl:{args:[T.u64,T.ptr,T.ptr],returns:T.i32},
});
const wide=s=>{const v=new Uint16Array(s.length+1);for(let i=0;i<s.length;i++)v[i]=s.charCodeAt(i);return v;};
const wait=async(predicate,label,ms=7000)=>{const until=performance.now()+ms;while(!predicate()){if(performance.now()>=until)throw Error(label+' diagnostic deadline');await Bun.sleep(20);}};
function lock(){
 const name=wide(join(root,process.env.OCXD_LEASE_PROBE_CANDIDATE==='true'?'runtime-lifetime.lease':'lifetime.lease'));
 const handle=BigInt(k.CreateFileW(ptr(name),0xc0020000,3,null,3,0x80|0x200000,0n));assert(handle!==0n&&handle!==0xffffffffffffffffn);
 const storage=new ArrayBuffer(32),overlapped=new Uint8Array(storage);
 if(!k.LockFileEx(handle,3,0,1,0,ptr(overlapped))){const error=k.GetLastError();k.CloseHandle(handle);assert.equal(error,33,'Only lock contention is busy');return null;}
 return {release(){assert(k.UnlockFileEx(handle,0,1,0,ptr(overlapped)));assert(k.CloseHandle(handle));}};
}
function birth(h){const t=new BigUint64Array(4);assert(k.GetProcessTimes(h,ptr(t.subarray(0,1)),ptr(t.subarray(1,2)),ptr(t.subarray(2,3)),ptr(t.subarray(3,4))));return t[0];}
function launch(role,job=0n){
 const quote=s=>{assert(!s.includes(String.fromCharCode(34)));return String.fromCharCode(34)+s+String.fromCharCode(34);};
 const isLauncher=role==='node-launcher',executable=isLauncher?process.env.OCXD_RECORD_PROBE_NODE:process.execPath;
 assert(executable&&isAbsolute(executable));
 const script=isLauncher?join(import.meta.dir,'windows023-native-detached-launcher.cjs'):import.meta.path;
 const app=wide(executable),line=wide(quote(executable)+(isLauncher?' ':' --no-env-file ')+quote(script)+(isLauncher?'':' '+role)),cwd=wide(root);
 const siStorage=new ArrayBuffer(104),si=new Uint8Array(siStorage);new DataView(siStorage).setUint32(0,104,true);
 const piStorage=new ArrayBuffer(24),pi=new Uint8Array(piStorage),v=new DataView(piStorage);
 assert(k.CreateProcessW(ptr(app),ptr(line),null,null,0,4|0x08000000,null,ptr(cwd),ptr(si),ptr(pi)),'Suspended child creation');
 const processHandle=v.getBigUint64(0,true),thread=v.getBigUint64(8,true),pid=v.getUint32(16,true),created=birth(processHandle);
 assert.equal(k.GetProcessId(processHandle),pid);
 // Assignment happens before the first user instruction, not after a PID scan.
 let assigned=job===0n;
 if(job!==0n){assigned=!!k.AssignProcessToJobObject(job,processHandle);}
 assert.notEqual(k.ResumeThread(thread),0xffffffff);assert(k.CloseHandle(thread));
 if(!assigned){writeFileSync(join(root,'release-service'),'1');writeFileSync(join(root,'release-child'),'1');throw Error('Job admission failed');}
 return {pid,handle:processHandle,created,close(){assert(k.CloseHandle(processHandle));}};
}
function active(job){
 const storage=new ArrayBuffer(48),data=new Uint8Array(storage),returned=new Uint32Array(1);
 assert(k.QueryInformationJobObject(job,1,ptr(data),data.length,ptr(returned)));assert.equal(returned[0],48);
 return new DataView(storage).getUint32(40,true);
}
function privateJob(name){
 const descriptor=new BigUint64Array(1),sddl=wide('O:'+currentSid+'D:P(A;;GA;;;'+currentSid+')');
 assert(security.symbols.ConvertStringSecurityDescriptorToSecurityDescriptorW(ptr(sddl),1,ptr(descriptor),null));
 const storage=new ArrayBuffer(24),sa=new Uint8Array(storage),v=new DataView(storage);v.setUint32(0,24,true);v.setBigUint64(8,descriptor[0],true);
 const text=wide(name);try{const job=BigInt(k.CreateJobObjectW(ptr(sa),ptr(text)));assert(job!==0n);assert.notEqual(k.GetLastError(),183,'Random job name must not already exist');return job;}finally{k.LocalFree(descriptor[0]);}
}
function jobSid(address){
 const text=new BigUint64Array(1);assert(security.symbols.ConvertSidToStringSidW(address,ptr(text)));
 try{const n=k.lstrlenW(text[0]);assert(n>0&&n<256);const buffer=new Uint16Array(n+1);k.lstrcpynW(ptr(buffer),text[0],n+1);return String.fromCharCode(...buffer.subarray(0,n));}
 finally{k.LocalFree(text[0]);}
}
function verifyPrivateJob(job){
 const sd=new BigUint64Array(1),owner=new BigUint64Array(1),dacl=new BigUint64Array(1);
 assert.equal(security.symbols.GetSecurityInfo(job,6,5,ptr(owner),null,ptr(dacl),null,ptr(sd)),0);
 try{
  assert.equal(jobSid(owner[0]),currentSid);const control=new Uint16Array(1),revision=new Uint32Array(1);
  assert(security.symbols.GetSecurityDescriptorControl(sd[0],ptr(control),ptr(revision)));assert(control[0]&0x1000,'Job DACL must remain protected');
  assert(dacl[0]!==0n&&security.symbols.IsValidAcl(dacl[0]));const aclAddress=Number(dacl[0]);assert(Number.isSafeInteger(aclAddress)&&aclAddress>0);
  assert.equal(read.u16(aclAddress,4),1,'Exact one user ACE required');const ace=new BigUint64Array(1);assert(security.symbols.GetAce(dacl[0],0,ptr(ace)));
  const address=Number(ace[0]);assert(Number.isSafeInteger(address)&&address>0);assert.equal(read.u8(address,0),0);assert.equal(read.u8(address,1),0);
  const size=read.u16(address,2);assert(size>=16&&size<=76);const count=read.u8(address,9);assert(count<=15);assert.equal(size,16+count*4);
  assert([0x10000000,0x1f003f].includes(read.u32(address,4)));assert.equal(jobSid(ace[0]+8n),currentSid);
 }finally{k.LocalFree(sd[0]);}
}
function boundProof(record,key){return createHmac('sha256',key).update(JSON.stringify(record)).digest();}
async function candidateContender(expected){
 const child=Bun.spawn([process.execPath,'--no-env-file',import.meta.path,'candidate-contender'],{env:{...process.env,OCXD_EXPECTED_LIFETIME_REFUSAL:expected},stdin:'ignore',stdout:'pipe',stderr:'pipe'});
 const stdout=new Response(child.stdout).text(),stderr=new Response(child.stderr).text();
 const exit=await child.exited;const [out,err]=await Promise.all([stdout,stderr]);
 assert.equal(exit,0,'Contender must observe exact typed refusal, never acquire authority');assert.equal(err,'');assert.equal(JSON.parse(out).refusal,expected);
}
const role=process.argv[2]??'controller';
function holdPublishedJob(){
 const name=process.env.OCXD_LEASE_PROBE_JOB;assert(name&&(name.startsWith('Local'+String.fromCharCode(92)+'ocxd-private-prototype-')||name.startsWith('Global'+String.fromCharCode(92)+'opencodex-runtime-')));
 const w=wide(name),h=BigInt(k.OpenJobObjectW(4|0x20000,0,ptr(w)));assert(h!==0n);verifyPrivateJob(h);
 const member=new Int32Array(1);assert(k.IsProcessInJob(BigInt(k.GetCurrentProcess()),h,ptr(member)));assert.equal(member[0],1);
 return h;
}
if(role==='candidate-contender'){
 const candidate=await import(pathToFileURL(join(runtime,'src/lib/windows-runtime-lifetime.ts')).href);
 let reason;try{candidate.acquireWindowsRuntimeLifetime(root);}catch(error){reason=error.message;}
 assert.equal(reason,'Windows runtime lifetime authority cannot be proved: '+process.env.OCXD_EXPECTED_LIFETIME_REFUSAL);
 console.log(JSON.stringify({accepted:false,diagnosticOnly:true,refusal:process.env.OCXD_EXPECTED_LIFETIME_REFUSAL}));security.close();lib.close();
}else if(role==='grandchild'){
 const publishedJob=holdPublishedJob();
 writeFileSync(join(root,'child-ready'),'1');
 await wait(()=>existsSync(join(root,'release-child')),'child release',12000);
 assert(k.CloseHandle(publishedJob));security.close();lib.close();
}else if(role==='service'){
 let publishedJob,lease,lifetime;
 if(process.env.OCXD_LEASE_PROBE_CANDIDATE==='true'){
  const candidate=await import(pathToFileURL(join(runtime,'src/lib/windows-runtime-lifetime.ts')).href);
  lifetime=candidate.acquireWindowsRuntimeLifetime(root);assert(lifetime);
  process.env.OCXD_LEASE_PROBE_JOB=lifetime.binding.job;publishedJob=holdPublishedJob();
 }else if(process.env.OCXD_LEASE_PROBE_SELF_JOB==='true'){
  publishedJob=privateJob(process.env.OCXD_LEASE_PROBE_JOB);verifyPrivateJob(publishedJob);
  assert(k.AssignProcessToJobObject(publishedJob,BigInt(k.GetCurrentProcess())),'Service must enter private Job before any child spawn');
 }else{publishedJob=holdPublishedJob();}
 if(!lifetime){lease=lock();assert(lease,'Service must acquire own lease');}
 const child=launch('grandchild');
 const record={pid:process.pid,created:birth(BigInt(k.GetCurrentProcess())).toString(),job:process.env.OCXD_LEASE_PROBE_JOB};
 if(lifetime){record.candidateBinding=lifetime.binding;record.candidateProof=lifetime.proof('A'.repeat(43),Buffer.alloc(32,1).toString('base64url'),1);}
 writeFileSync(join(root,'service-ready.tmp'),JSON.stringify(record));
 renameSync(join(root,'service-ready.tmp'),join(root,'service-ready'));
 try{await wait(()=>existsSync(join(root,'release-service')),'service release',10000);}
 finally{if(lease)lease.release();child.close();assert(k.CloseHandle(publishedJob));security.close();lib.close();}
}else{
 assert.equal(role,'controller');const began=performance.now();
 const jobName='Local'+String.fromCharCode(92)+'ocxd-private-prototype-'+randomUUID();process.env.OCXD_LEASE_PROBE_JOB=jobName;
 const candidateLifetime=process.env.OCXD_LEASE_PROBE_CANDIDATE==='true',selfJob=candidateLifetime||process.env.OCXD_LEASE_PROBE_SELF_JOB==='true';
 let jobWide=wide(jobName),job=selfJob?0n:privateJob(jobName);if(job!==0n)verifyPrivateJob(job);
 let service,actualService=0n;
 try{
  const detached=process.env.OCXD_LEASE_PROBE_DETACHED==='true';
  service=launch(detached?'node-launcher':'service',job);
  if(!selfJob){const inJob=new Int32Array(1);assert(k.IsProcessInJob(service.handle,job,ptr(inJob)));assert.equal(inJob[0],1);}
  await wait(()=>existsSync(join(root,'service-ready'))&&existsSync(join(root,'child-ready')),'both ready');
  const binding=JSON.parse(readFileSync(join(root,'service-ready'),'utf8'));assert(Number.isInteger(binding.pid)&&binding.pid>0&&/^[0-9]+$/.test(binding.created));
  if(selfJob){jobWide=wide(binding.job);job=BigInt(k.OpenJobObjectW(4|0x20000,0,ptr(jobWide)));assert(job!==0n);verifyPrivateJob(job);}
  actualService=BigInt(k.OpenProcess(0x101000,0,binding.pid));assert(actualService!==0n);assert.equal(k.GetProcessId(actualService),binding.pid);assert.equal(birth(actualService).toString(),binding.created);
  const serverInJob=new Int32Array(1);assert(k.IsProcessInJob(actualService,job,ptr(serverInJob)));assert.equal(serverInJob[0],1);
  // The server and child retain their own query handles. Reopen a fresh
  // observer handle, then prove membership again; no cached PID authority.
  assert(k.CloseHandle(job));job=BigInt(k.OpenJobObjectW(4|0x20000,0,ptr(jobWide)));assert(job!==0n);verifyPrivateJob(job);
  serverInJob[0]=0;assert(k.IsProcessInJob(actualService,job,ptr(serverInJob)));assert.equal(serverInJob[0],1);
  const wrong=BigInt(k.CreateJobObjectW(null,null));assert(wrong!==0n);
  try{const wrongMember=new Int32Array(1);assert(k.IsProcessInJob(actualService,wrong,ptr(wrongMember)));assert.equal(wrongMember[0],0);}finally{assert(k.CloseHandle(wrong));}
  const generation=randomUUID(),key=randomBytes(32),bound={home:root,sid:currentSid,pid:binding.pid,created:binding.created,job:jobName,generation};
  const proof=boundProof(bound,key);assert(timingSafeEqual(proof,boundProof(bound,key)));
  for(const changed of [{home:root+'-other'},{sid:currentSid+'-other'},{pid:binding.pid+1},{created:String(BigInt(binding.created)+1n)},{job:jobName+'-other'},{generation:randomUUID()}])assert(!timingSafeEqual(proof,boundProof({...bound,...changed},key)));
  assert(!timingSafeEqual(proof,boundProof(bound,randomBytes(32))));
  if(candidateLifetime){
   assert.equal(binding.candidateBinding.pid,binding.pid);assert.equal(binding.candidateBinding.created,binding.created);assert.equal(binding.candidateBinding.sid,currentSid);assert.equal(binding.candidateBinding.job,binding.job);
   const signed=record=>createHmac('sha256','A'.repeat(43)).update('opencodex-runtime-lifetime-v1\n'+Buffer.alloc(32,1).toString('base64url')+'\n1\n'+JSON.stringify([record.schema,record.home,record.sid,record.pid,record.created,record.generation,record.job,record.leaseVolume,record.leaseIndex])).digest('base64url');
   assert.equal(signed(binding.candidateBinding),binding.candidateProof);
   for(const change of [{home:root+'-different'},{sid:currentSid+'-different'},{pid:binding.pid+1},{created:String(BigInt(binding.created)+1n)},{job:binding.job+'-different'},{generation:randomUUID()},{leaseVolume:binding.candidateBinding.leaseVolume+1},{leaseIndex:String(BigInt(binding.candidateBinding.leaseIndex)+1n)}])assert.notEqual(signed({...binding.candidateBinding,...change}),binding.candidateProof);
   if(process.env.OCXD_LEASE_PROBE_RUST_HANDSHAKE==='true'){
    writeFileSync(join(root,'awaiting-rust-live-witness'),'1');
    await wait(()=>existsSync(join(root,'rust-live-witness-observed')),'independent Rust live witness',7000);
   }
  }
  if(detached){await wait(()=>k.WaitForSingleObject(service.handle,0)===0,'Node launcher exit');assert.equal(k.WaitForSingleObject(actualService,0),0x102);}
  assert(active(job)>=2,'Grandchild must inherit the actual job');assert.equal(lock(),null,'Absent runtime record cannot mean stopped while lease held');
  if(candidateLifetime)await candidateContender('lease-busy');
  assert.throws(()=>renameSync(join(root,candidateLifetime?'runtime-lifetime.lease':'lifetime.lease'),join(root,'moved.lease')),'Pinned lease cannot be replaced');
  assert.equal(birth(service.handle),service.created);assert.equal(k.GetProcessId(service.handle),service.pid);
  writeFileSync(join(root,'release-service'),'1');await wait(()=>k.WaitForSingleObject(actualService,0)===0,'original service exit');
  const exit=new Uint32Array(1);assert(k.GetExitCodeProcess(actualService,ptr(exit)));assert.equal(exit[0],0);assert.equal(birth(actualService).toString(),binding.created);
  assert(active(job)>=1,'Original PID exit alone cannot prove all controlled children exited');
  const afterExit=lock();assert(afterExit);afterExit.release();
  if(candidateLifetime)await candidateContender('controlled-set-not-empty');
  writeFileSync(join(root,'release-child'),'1');await wait(()=>active(job)===0,'controlled set empty');
  const finalLease=lock();assert(finalLease);finalLease.release();
  console.log(JSON.stringify({accepted:false,diagnosticOnly:true,runtimeChanged:false,productStarted:false,serviceMutation:false,terminationApiUsed:false,killOnCloseUsed:false,suspendedJobAdmission:!selfJob,serviceEnteredPrivateJobBeforeOwnChildren:selfJob,candidateLifetimeModuleUsed:candidateLifetime,candidateBindingEightMutationsRejected:candidateLifetime,candidateDuplicateWhileLeaseBusyRefused:candidateLifetime,candidateReplacementWhileChildAliveRefused:candidateLifetime,nodeDetachedLauncherUsed:detached,detachedServiceStayedInJob:detached,observerReopenedNamedJob:true,privateJobOwnerAndProtectedDaclVerified:true,bindingSevenTamperCounterexamplesRejected:true,wrongJobMembershipRejected:true,actualInheritedChildVerified:true,recordAbsentWhileLeaseBusyRefused:true,liveLeaseReplacementRefused:true,retainedServicePidAndCreationVerified:true,serviceExitWithChildAliveRefused:true,finalControlledSetEmpty:true,finalLeaseUnheld:true,milliseconds:performance.now()-began,limits:'Synthetic same-SID candidate or LocalJob/test HMAC. Not protected production record publication, server attestation/official restoration/lifecycle/foreign-SID or original official CLI proof.'}));
 }finally{
  writeFileSync(join(root,'release-service'),'1');writeFileSync(join(root,'release-child'),'1');
  if(job===0n)job=BigInt(k.OpenJobObjectW(4|0x20000,0,ptr(jobWide)));
  assert(job!==0n,'Missing job cannot prove cleanup');await wait(()=>active(job)===0,'all owned processes cleanup',15000);
  if(actualService!==0n)assert(k.CloseHandle(actualService));if(service)service.close();assert(k.CloseHandle(job));security.close();lib.close();
 }
}
