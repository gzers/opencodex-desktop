// Test-only lifetime proof. Own synthetic processes, no proxy/client/service.
// No TerminateProcess, terminate-job or kill-on-close flag is used.
import assert from 'node:assert/strict';
import {dlopen,ptr,FFIType as T} from 'bun:ffi';
import {writeFileSync,existsSync,renameSync} from 'node:fs';
import {join,resolve,relative,isAbsolute} from 'node:path';
import {pathToFileURL} from 'node:url';
assert(process.platform==='win32'&&process.arch==='x64'&&process.env.GITHUB_ACTIONS==='true');
const root=resolve(process.env.OCXD_LEASE_PROBE_ROOT),boundary=resolve(process.env.RUNNER_TEMP);
const rel=relative(boundary,root);assert(rel&&!rel.startsWith('..')&&!isAbsolute(rel));
const runtime=resolve(process.env.OCXD_RECORD_PROBE_PACKAGE);
const {resolveTrustedWindowsSystemDirectory}=await import(pathToFileURL(join(runtime,'src/lib/windows-elevation.ts')).href);
const system=resolveTrustedWindowsSystemDirectory();
const lib=dlopen(join(system,'kernel32.dll'),{
 CreateFileW:{args:[T.ptr,T.u32,T.u32,T.ptr,T.u32,T.u32,T.u64],returns:T.u64},
 CloseHandle:{args:[T.u64],returns:T.i32},
 LockFileEx:{args:[T.u64,T.u32,T.u32,T.u32,T.u32,T.ptr],returns:T.i32},
 UnlockFileEx:{args:[T.u64,T.u32,T.u32,T.u32,T.ptr],returns:T.i32},
 CreateJobObjectW:{args:[T.ptr,T.ptr],returns:T.u64},
 AssignProcessToJobObject:{args:[T.u64,T.u64],returns:T.i32},
 IsProcessInJob:{args:[T.u64,T.u64,T.ptr],returns:T.i32},
 QueryInformationJobObject:{args:[T.u64,T.i32,T.ptr,T.u32,T.ptr],returns:T.i32},
 CreateProcessW:{args:[T.ptr,T.ptr,T.ptr,T.ptr,T.i32,T.u32,T.ptr,T.ptr,T.ptr,T.ptr],returns:T.i32},
 ResumeThread:{args:[T.u64],returns:T.u32},
 GetProcessId:{args:[T.u64],returns:T.u32},
 GetProcessTimes:{args:[T.u64,T.ptr,T.ptr,T.ptr,T.ptr],returns:T.i32},
 GetExitCodeProcess:{args:[T.u64,T.ptr],returns:T.i32},
 WaitForSingleObject:{args:[T.u64,T.u32],returns:T.u32},
 GetLastError:{args:[],returns:T.u32},
});
const k=lib.symbols;
const wide=s=>{const v=new Uint16Array(s.length+1);for(let i=0;i<s.length;i++)v[i]=s.charCodeAt(i);return v;};
const wait=async(predicate,label,ms=7000)=>{const until=performance.now()+ms;while(!predicate()){if(performance.now()>=until)throw Error(label+' diagnostic deadline');await Bun.sleep(20);}};
function lock(){
 const name=wide(join(root,'lifetime.lease'));
 const handle=BigInt(k.CreateFileW(ptr(name),0xc0020000,3,null,3,0x80|0x200000,0n));assert(handle!==0n&&handle!==0xffffffffffffffffn);
 const storage=new ArrayBuffer(32),overlapped=new Uint8Array(storage);
 if(!k.LockFileEx(handle,3,0,1,0,ptr(overlapped))){const error=k.GetLastError();k.CloseHandle(handle);assert.equal(error,33,'Only lock contention is busy');return null;}
 return {release(){assert(k.UnlockFileEx(handle,0,1,0,ptr(overlapped)));assert(k.CloseHandle(handle));}};
}
function birth(h){const t=new BigUint64Array(4);assert(k.GetProcessTimes(h,ptr(t.subarray(0,1)),ptr(t.subarray(1,2)),ptr(t.subarray(2,3)),ptr(t.subarray(3,4))));return t[0];}
function launch(role,job=0n){
 const quote=s=>{assert(!s.includes(String.fromCharCode(34)));return String.fromCharCode(34)+s+String.fromCharCode(34);};
 const app=wide(process.execPath),line=wide(quote(process.execPath)+' --no-env-file '+quote(import.meta.path)+' '+role),cwd=wide(root);
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
const role=process.argv[2]??'controller';
if(role==='grandchild'){
 writeFileSync(join(root,'child-ready'),'1');
 await wait(()=>existsSync(join(root,'release-child')),'child release',12000);
 lib.close();
}else if(role==='service'){
 const lease=lock();assert(lease,'Service must acquire own lease');const child=launch('grandchild');
 writeFileSync(join(root,'service-ready'),'1');
 try{await wait(()=>existsSync(join(root,'release-service')),'service release',10000);}
 finally{lease.release();child.close();lib.close();}
}else{
 assert.equal(role,'controller');const began=performance.now(),job=BigInt(k.CreateJobObjectW(null,null));assert(job!==0n);
 let service;
 try{
  service=launch('service',job);const inJob=new Int32Array(1);assert(k.IsProcessInJob(service.handle,job,ptr(inJob)));assert.equal(inJob[0],1);
  await wait(()=>existsSync(join(root,'service-ready'))&&existsSync(join(root,'child-ready')),'both ready');
  assert(active(job)>=2,'Grandchild must inherit the actual job');assert.equal(lock(),null,'Absent runtime record cannot mean stopped while lease held');
  assert.throws(()=>renameSync(join(root,'lifetime.lease'),join(root,'moved.lease')),'Pinned lease cannot be replaced');
  assert.equal(birth(service.handle),service.created);assert.equal(k.GetProcessId(service.handle),service.pid);
  writeFileSync(join(root,'release-service'),'1');await wait(()=>k.WaitForSingleObject(service.handle,0)===0,'original service exit');
  const exit=new Uint32Array(1);assert(k.GetExitCodeProcess(service.handle,ptr(exit)));assert.equal(exit[0],0);
  assert(active(job)>=1,'Original PID exit alone cannot prove all controlled children exited');
  const afterExit=lock();assert(afterExit);afterExit.release();
  writeFileSync(join(root,'release-child'),'1');await wait(()=>active(job)===0,'controlled set empty');
  const finalLease=lock();assert(finalLease);finalLease.release();
  console.log(JSON.stringify({accepted:false,diagnosticOnly:true,runtimeChanged:false,productStarted:false,serviceMutation:false,terminationApiUsed:false,killOnCloseUsed:false,suspendedJobAdmission:true,actualInheritedChildVerified:true,recordAbsentWhileLeaseBusyRefused:true,liveLeaseReplacementRefused:true,retainedServicePidAndCreationVerified:true,serviceExitWithChildAliveRefused:true,finalControlledSetEmpty:true,finalLeaseUnheld:true,milliseconds:performance.now()-began,limits:'Synthetic same-SID processes only. Not full HMAC/official restoration/lifecycle/foreign-SID or unmodified Bun detachment integration proof.'}));
 }finally{
  writeFileSync(join(root,'release-service'),'1');writeFileSync(join(root,'release-child'),'1');
  await wait(()=>active(job)===0,'all owned processes cleanup',15000);
  if(service)service.close();assert(k.CloseHandle(job));lib.close();
 }
}
