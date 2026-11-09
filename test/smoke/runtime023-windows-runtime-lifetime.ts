/** Candidate maintained-runtime protocol. Not installed/activated by this file. */
import {dlopen,ptr,read,FFIType as T} from 'bun:ffi';
import {createHash,createHmac,randomUUID} from 'node:crypto';
import {realpathSync,existsSync} from 'node:fs';
import {win32} from 'node:path';

export interface WindowsRuntimeLifetime {
  schema:'ocxd-runtime-lifetime/1';
  home:string;
  sid:string;
  pid:number;
  created:string;
  generation:string;
  job:string;
  leaseVolume:number;
  leaseIndex:string;
}
let active:ReturnType<typeof acquireWindowsRuntimeLifetime>;
export function currentWindowsRuntimeLifetime():WindowsRuntimeLifetime|undefined{return active?.binding;}
export function createWindowsRuntimeLifetimeProof(secret:string,challenge:string,port:number):string|undefined{return active?.proof(secret,challenge,port);}
const IS_WINDOWS=process.platform==='win32';
function refuse(code='authority-unknown'):never{throw new Error('Windows runtime lifetime authority cannot be proved: '+code);}
function wide(s:string){const out=new Uint16Array(s.length+1);for(let i=0;i<s.length;i++)out[i]=s.charCodeAt(i);return out;}
function load(){
  const {resolveTrustedWindowsSystemDirectory}=require('./windows-elevation') as typeof import('./windows-elevation');
  const {resolveWindowsCurrentSid}=require('../codex/windows-current-sid') as typeof import('../codex/windows-current-sid');
  const system=resolveTrustedWindowsSystemDirectory(),sid=resolveWindowsCurrentSid(()=>refuse());
  if(!/^S-1-[0-9-]+$/.test(sid)||sid==='S-1-5-18'||sid==='S-1-5-32-544')refuse();
  const kernel=dlopen(win32.join(system,'kernel32.dll'),{
    CreateFileW:{args:[T.ptr,T.u32,T.u32,T.ptr,T.u32,T.u32,T.u64],returns:T.u64},
    LockFileEx:{args:[T.u64,T.u32,T.u32,T.u32,T.u32,T.ptr],returns:T.i32},
    UnlockFileEx:{args:[T.u64,T.u32,T.u32,T.u32,T.ptr],returns:T.i32},
    GetFileInformationByHandle:{args:[T.u64,T.ptr],returns:T.i32},
    GetFileType:{args:[T.u64],returns:T.u32},
    CreateJobObjectW:{args:[T.ptr,T.ptr],returns:T.u64},
    QueryInformationJobObject:{args:[T.u64,T.i32,T.ptr,T.u32,T.ptr],returns:T.i32},
    AssignProcessToJobObject:{args:[T.u64,T.u64],returns:T.i32},
    IsProcessInJob:{args:[T.u64,T.u64,T.ptr],returns:T.i32},
    GetCurrentProcess:{args:[],returns:T.u64},
    GetProcessId:{args:[T.u64],returns:T.u32},
    GetProcessTimes:{args:[T.u64,T.ptr,T.ptr,T.ptr,T.ptr],returns:T.i32},
    CloseHandle:{args:[T.u64],returns:T.i32},
    GetLastError:{args:[],returns:T.u32},
    LocalFree:{args:[T.u64],returns:T.u64},
    lstrlenW:{args:[T.u64],returns:T.i32},
    lstrcpynW:{args:[T.ptr,T.u64,T.i32],returns:T.ptr},
  });
  const security=dlopen(win32.join(system,'advapi32.dll'),{
    ConvertStringSecurityDescriptorToSecurityDescriptorW:{args:[T.ptr,T.u32,T.ptr,T.ptr],returns:T.i32},
    ConvertSidToStringSidW:{args:[T.u64,T.ptr],returns:T.i32},
    GetSecurityInfo:{args:[T.u64,T.i32,T.u32,T.ptr,T.ptr,T.ptr,T.ptr,T.ptr],returns:T.u32},
    GetSecurityDescriptorControl:{args:[T.u64,T.ptr,T.ptr],returns:T.i32},
    IsValidAcl:{args:[T.u64],returns:T.i32},
    GetAce:{args:[T.u64,T.u32,T.ptr],returns:T.i32},
  });
  return {kernel,security,sid};
}
type Api=ReturnType<typeof load>;
function sidText(api:Api,address:bigint):string{
  const out=new BigUint64Array(1),a=api.security.symbols,k=api.kernel.symbols;
  if(!a.ConvertSidToStringSidW(address,ptr(out)))refuse();
  try{const length=k.lstrlenW(out[0]!);if(length<5||length>256)refuse();const buffer=new Uint16Array(length+1);k.lstrcpynW(ptr(buffer),out[0]!,length+1);return String.fromCharCode(...buffer.subarray(0,length));}
  finally{k.LocalFree(out[0]!);}
}
function privateAuthority(api:Api,handle:bigint,kind:number,expectedMask:number){
  const owner=new BigUint64Array(1),dacl=new BigUint64Array(1),sd=new BigUint64Array(1),a=api.security.symbols,k=api.kernel.symbols;
  if(a.GetSecurityInfo(handle,kind,5,ptr(owner),null,ptr(dacl),null,ptr(sd))!==0)refuse();
  try{
    if(sidText(api,owner[0]!)!==api.sid)refuse();
    const control=new Uint16Array(1),revision=new Uint32Array(1);
    if(!a.GetSecurityDescriptorControl(sd[0]!,ptr(control),ptr(revision))||!(control[0]!&0x1000)||dacl[0]===0n||!a.IsValidAcl(dacl[0]!))refuse();
    const acl=Number(dacl[0]);if(!Number.isSafeInteger(acl)||acl<=0||read.u16(acl,4)!==1)refuse();
    const ace=new BigUint64Array(1);if(!a.GetAce(dacl[0]!,0,ptr(ace)))refuse();const address=Number(ace[0]);
    if(!Number.isSafeInteger(address)||address<=0||read.u8(address,0)!==0||read.u8(address,1)!==0)refuse();
    const count=read.u8(address,9),size=read.u16(address,2),mask=read.u32(address,4);
    if(count>15||size!==16+count*4||![0x10000000,expectedMask].includes(mask)||sidText(api,ace[0]!+8n)!==api.sid)refuse();
  }finally{k.LocalFree(sd[0]!);}
}
function descriptor(api:Api,work:(attributes:Uint8Array)=>bigint):bigint{
  const sd=new BigUint64Array(1),text=wide('O:'+api.sid+'D:P(A;;GA;;;'+api.sid+')');
  if(!api.security.symbols.ConvertStringSecurityDescriptorToSecurityDescriptorW(ptr(text),1,ptr(sd),null))refuse();
  const storage=new ArrayBuffer(24),attributes=new Uint8Array(storage),v=new DataView(storage);v.setUint32(0,24,true);v.setBigUint64(8,sd[0]!,true);
  try{return work(attributes);}finally{api.kernel.symbols.LocalFree(sd[0]!);}
}
function fileInfo(api:Api,handle:bigint,directory=false){
  const storage=new ArrayBuffer(52),buffer=new Uint8Array(storage),v=new DataView(storage),k=api.kernel.symbols;
  if(!k.GetFileInformationByHandle(handle,ptr(buffer))||k.GetFileType(handle)!==1)refuse();
  const flags=v.getUint32(0,true);if(flags&0x400||!!(flags&0x10)!==directory||!directory&&v.getUint32(40,true)!==1)refuse();
  return {volume:v.getUint32(28,true),index:((BigInt(v.getUint32(44,true))<<32n)|BigInt(v.getUint32(48,true))).toString()};
}
function activeCount(api:Api,job:bigint){
  const storage=new ArrayBuffer(48),buffer=new Uint8Array(storage),returned=new Uint32Array(1);
  if(!api.kernel.symbols.QueryInformationJobObject(job,1,ptr(buffer),buffer.length,ptr(returned))||returned[0]!==48)refuse();
  return new DataView(storage).getUint32(40,true);
}

/** Must execute before bind/publication and before any service child is spawned. */
export function acquireWindowsRuntimeLifetime(home:string):{binding:WindowsRuntimeLifetime;proof:(secret:string,challenge:string,port:number)=>string}|undefined{
  if(!IS_WINDOWS)return undefined;if(process.arch!=='x64'||!win32.isAbsolute(home)||!existsSync(home))refuse();
  if(active)refuse('process-already-bound');
  const api=load(),k=api.kernel.symbols,pins:bigint[]=[];let lease=0n,job=0n,locked=false,committed=false;
  const supplied=home.replaceAll(String.fromCharCode(92),'/');
  if(supplied.split('/').some(part=>part==='.'||part==='..')||home.includes(String.fromCharCode(0)))refuse();
  const canonical=realpathSync.native(home),normalized=canonical.replaceAll(String.fromCharCode(92),'/').toLowerCase();
  if(normalized.includes('/../')||normalized.includes('/./'))refuse();
  const leasePath=win32.join(canonical,'runtime-lifetime.lease'),leaseName=wide(leasePath),overlapStorage=new ArrayBuffer(32),overlap=new Uint8Array(overlapStorage);
  try{
    // Check/pin the supplied chain, not just its canonical target. An input
    // junction cannot silently become a different trusted HOME anchor.
    let p=home;const parents=[];for(;;){parents.push(p);const next=win32.dirname(p);if(next===p)break;p=next;}
    for(const path of parents.reverse()){
      const name=wide(path),h=BigInt(k.CreateFileW(ptr(name),0x80000000,3,null,3,0x02000000|0x00200000,0n));
      if(h===0n||h===0xffffffffffffffffn)refuse();pins.push(h);fileInfo(api,h,true);
    }
    if(realpathSync.native(home)!==canonical)refuse();
    let createError=0;lease=descriptor(api,attributes=>{const h=BigInt(k.CreateFileW(ptr(leaseName),0xc0020000,3,ptr(attributes),1,0x80|0x00200000,0n));createError=k.GetLastError();return h;});
    if(lease===0xffffffffffffffffn||lease===0n){if(createError!==80&&createError!==183)refuse();lease=BigInt(k.CreateFileW(ptr(leaseName),0xc0020000,3,null,3,0x80|0x00200000,0n));}
    if(lease===0n||lease===0xffffffffffffffffn)refuse();
    const leaseIdentity=fileInfo(api,lease);privateAuthority(api,lease,1,0x1f01ff);
    if(!k.LockFileEx(lease,3,0,1,0,ptr(overlap))){const error=k.GetLastError();refuse(error===33?'lease-busy':'lease-query-failed');}locked=true;
    const jobName='Global'+String.fromCharCode(92)+'opencodex-runtime-'+createHash('sha256').update(api.sid).update(String.fromCharCode(0)).update(normalized).digest('hex'),jobWide=wide(jobName);
    job=descriptor(api,attributes=>BigInt(k.CreateJobObjectW(ptr(attributes),ptr(jobWide))));if(job===0n)refuse();
    privateAuthority(api,job,6,0x1f003f);if(activeCount(api,job)!==0)refuse('controlled-set-not-empty');
    // No breakaway and no kill-on-close are configured; verify the actual flags.
    const limits=new Uint8Array(144),returned=new Uint32Array(1);
    if(!k.QueryInformationJobObject(job,9,ptr(limits),limits.length,ptr(returned))||returned[0]!==144||new DataView(limits.buffer).getUint32(16,true)!==0)refuse();
    const self=BigInt(k.GetCurrentProcess());if(!k.AssignProcessToJobObject(job,self))refuse();
    const member=new Int32Array(1);if(!k.IsProcessInJob(self,job,ptr(member))||member[0]!==1||k.GetProcessId(self)!==process.pid)refuse();
    const times=new BigUint64Array(4);if(!k.GetProcessTimes(self,ptr(times.subarray(0,1)),ptr(times.subarray(1,2)),ptr(times.subarray(2,3)),ptr(times.subarray(3,4))))refuse();
    const binding:WindowsRuntimeLifetime=Object.freeze({schema:'ocxd-runtime-lifetime/1',home:canonical,sid:api.sid,pid:process.pid,created:times[0]!.toString(),generation:randomUUID(),job:jobName,leaseVolume:leaseIdentity.volume,leaseIndex:leaseIdentity.index});
    committed=true;
    // Actual process lifetime owns these handles. No user-space release function
    // can declare absence before the service exits and official restore finishes.
    process.once('exit',()=>{if(locked)k.UnlockFileEx(lease,0,1,0,ptr(overlap));k.CloseHandle(lease);k.CloseHandle(job);for(const h of pins.reverse())k.CloseHandle(h);api.security.close();api.kernel.close();});
    active={binding,proof(secret,challenge,port){
      if(!/^[A-Za-z0-9_-]{43}$/.test(secret)||!/^[A-Za-z0-9_-]{43}$/.test(challenge)||!Number.isInteger(port)||port<1||port>65535)refuse();
      return createHmac('sha256',secret).update('opencodex-runtime-lifetime-v1\n'+challenge+'\n'+port+'\n'+JSON.stringify([binding.schema,binding.home,binding.sid,binding.pid,binding.created,binding.generation,binding.job,binding.leaseVolume,binding.leaseIndex])).digest('base64url');
    }};
    return active;
  }finally{
    if(!committed){if(locked)k.UnlockFileEx(lease,0,1,0,ptr(overlap));if(lease!==0n&&lease!==0xffffffffffffffffn)k.CloseHandle(lease);if(job!==0n)k.CloseHandle(job);for(const h of pins.reverse())k.CloseHandle(h);api.security.close();api.kernel.close();}
  }
}
