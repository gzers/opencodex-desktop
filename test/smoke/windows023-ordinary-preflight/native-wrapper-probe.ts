// Test-only, read-only Windows APIs. No service/task changes or termination API.
import {dlopen,ptr,FFIType as T} from 'bun:ffi';
import {win32,join} from 'node:path';
import {pathToFileURL} from 'node:url';
const root=process.env.OCXD_OWNERSHIP_DIAG_PACKAGE!;
if(process.platform!=='win32'||process.arch!=='x64')throw Error('Windows x64 test fixture only');
const elevation=await import(pathToFileURL(join(root,'src/lib/windows-elevation.ts')).href);
const system=elevation.resolveTrustedWindowsSystemDirectory();
const kernel=dlopen(join(system,'kernel32.dll'),{
 CreateToolhelp32Snapshot:{args:[T.u32,T.u32],returns:T.u64},
 Process32FirstW:{args:[T.u64,T.ptr],returns:T.i32},
 Process32NextW:{args:[T.u64,T.ptr],returns:T.i32},
 OpenProcess:{args:[T.u32,T.i32,T.u32],returns:T.u64},
 GetProcessId:{args:[T.u64],returns:T.u32},
 GetCurrentProcess:{args:[],returns:T.u64},
 GetProcessTimes:{args:[T.u64,T.ptr,T.ptr,T.ptr,T.ptr],returns:T.i32},
 QueryFullProcessImageNameW:{args:[T.u64,T.u32,T.ptr,T.ptr],returns:T.i32},
 WaitForSingleObject:{args:[T.u64,T.u32],returns:T.u32},
 CloseHandle:{args:[T.u64],returns:T.i32},
 GetLastError:{args:[],returns:T.u32},
 LocalFree:{args:[T.u64],returns:T.u64},
 lstrlenW:{args:[T.u64],returns:T.i32},
 lstrcpynW:{args:[T.ptr,T.u64,T.i32],returns:T.ptr},
});
const advapi=dlopen(join(system,'advapi32.dll'),{
 OpenProcessToken:{args:[T.u64,T.u32,T.ptr],returns:T.i32},
 GetTokenInformation:{args:[T.u64,T.u32,T.ptr,T.u32,T.ptr],returns:T.i32},
 ConvertSidToStringSidW:{args:[T.u64,T.ptr],returns:T.i32},
});
const ntdll=dlopen(join(system,'ntdll.dll'),{
 NtQueryInformationProcess:{args:[T.u64,T.u32,T.ptr,T.u32,T.ptr],returns:T.i32},
});
const k=kernel.symbols,a=advapi.symbols,n=ntdll.symbols;
const wide=(bytes:Uint8Array)=>new TextDecoder('utf-16le',{fatal:true}).decode(bytes);
const fail=(name:string)=>{throw Error(name);};
function sid(handle:bigint):string{
 const token=new BigUint64Array(1);
 if(!a.OpenProcessToken(handle,8,ptr(token)))return fail('token-open');
 try{
  const size=new Uint32Array(1);
  a.GetTokenInformation(token[0]!,1,null,0,ptr(size));
  if(size[0]!<8||size[0]!>65536)return fail('token-size');
  // Bun may materialize a small typed array's .buffer into different storage.
  // Allocate the backing ArrayBuffer before the native API writes its SID pointer.
  const storage=new ArrayBuffer(size[0]!),buffer=new Uint8Array(storage),view=new DataView(storage);
  const start=BigInt(ptr(buffer));
  if(!a.GetTokenInformation(token[0]!,1,ptr(buffer),buffer.length,ptr(size)))return fail('token-read');
  if(BigInt(ptr(buffer))!==start||size[0]!>buffer.length)return fail('token-storage-changed');
  const address=view.getBigUint64(0,true);
  if(address<start||address+8n>start+BigInt(buffer.length))return fail('token-pointer');
  const text=new BigUint64Array(1);
  if(!a.ConvertSidToStringSidW(address,ptr(text)))return fail('token-string');
  try{
   const length=k.lstrlenW(text[0]!);if(length<5||length>256)return fail('sid-length');
   const value=new Uint16Array(length+1);k.lstrcpynW(ptr(value),text[0]!,length+1);
   const result=wide(new Uint8Array(value.buffer,0,length*2));if(!/^S-1-[0-9-]+$/.test(result))return fail('sid-framing');
   return result;
  }finally{k.LocalFree(text[0]!);}
 }finally{k.CloseHandle(token[0]!);}
}
function birth(handle:bigint):bigint{
 const t=new BigUint64Array(4);
 if(!k.GetProcessTimes(handle,ptr(t.subarray(0,1)),ptr(t.subarray(1,2)),ptr(t.subarray(2,3)),ptr(t.subarray(3,4))))return fail('birth');
 return t[0]!;
}
function image(handle:bigint):string{
 const size=new Uint32Array([32768]),buffer=new Uint16Array(size[0]!);
 if(!k.QueryFullProcessImageNameW(handle,0,ptr(buffer),ptr(size)))return fail('image');
 const value=wide(new Uint8Array(buffer.buffer,0,size[0]!*2));if(!win32.isAbsolute(value))return fail('image-path');return value;
}
function command(handle:bigint):string{
 const needed=new Uint32Array(1);n.NtQueryInformationProcess(handle,60,null,0,ptr(needed));
 if(needed[0]!<16||needed[0]!>65552)return fail('command-size');
 const storage=new ArrayBuffer(needed[0]!),buffer=new Uint8Array(storage),v=new DataView(storage),base=BigInt(ptr(buffer));
 if(n.NtQueryInformationProcess(handle,60,ptr(buffer),buffer.length,ptr(needed))<0)return fail('command-read');
 if(BigInt(ptr(buffer))!==base||needed[0]!>buffer.length)return fail('command-storage-changed');
 const length=v.getUint16(0,true),maximum=v.getUint16(2,true),address=v.getBigUint64(8,true);
 if(length===0||length%2||length>maximum||address<base||address+BigInt(length)>base+BigInt(buffer.length))return fail('command-pointer');
 const offset=Number(address-base);return wide(buffer.subarray(offset,offset+length));
}
function matches(command:string,path:string):boolean{
 const c=command.toLowerCase(),p=path.toLowerCase();let index=0;
 while(index<c.length){index=c.indexOf(p,index);if(index<0)return false;const before=index>0?c[index-1]:' ',after=c[index+p.length]??' ';if(/[\s"']/.test(before!)&&/[\s"']/.test(after))return true;index++;}return false;
}
const owner=sid(BigInt(k.GetCurrentProcess())),candidateNames=new Set(['cmd.exe','wscript.exe','cscript.exe']);
function enumerate(paths:string[],expectedPids:number[]){
 const started=performance.now(),snapshot=BigInt(k.CreateToolhelp32Snapshot(2,0));
 if(snapshot===0n||snapshot===0xffffffffffffffffn)return fail('snapshot');
 const rows:any[]=[],unknown:any[]=[],foreign:any[]=[],exited:number[]=[];
 try{
  // PROCESSENTRY32W x64: size568, PID8, executable44, WCHAR[260].
  const entry=new Uint8Array(568),view=new DataView(entry.buffer);view.setUint32(0,568,true);
  let ok=k.Process32FirstW(snapshot,ptr(entry));
  while(ok){
   const pid=view.getUint32(8,true),bytes=entry.subarray(44,564);let end=0;while(end<bytes.length&&(bytes[end]||bytes[end+1]))end+=2;
   const name=wide(bytes.subarray(0,end)).toLowerCase();
   if(candidateNames.has(name)&&pid!==process.pid){
    const h=BigInt(k.OpenProcess(0x1000|0x100000,0,pid));
    if(!h){unknown.push({pid,reason:'open-denied-or-exited'});}else try{
     if(k.GetProcessId(h)!==pid)fail('pid');const born=birth(h),path=image(h),identity=sid(h);
     if(born===0n||win32.basename(path).toLowerCase()!==name)fail('birth-or-image-mismatch');
     if(identity!==owner){foreign.push({pid});}else{
      const cmd=command(h),hit=paths.some(p=>matches(cmd,p));
      if(k.GetProcessId(h)!==pid||birth(h)!==born||image(h)!==path||sid(h)!==identity||k.WaitForSingleObject(h,0)!==0x102)fail('identity-changed-or-exited');
      rows.push({pid,matched:hit,expectedFixture:expectedPids.includes(pid),fullSameHandleIdentity:true});
     }
    }catch(error){if(k.WaitForSingleObject(h,0)===0)exited.push(pid);else unknown.push({pid,reason:error instanceof Error?error.message:'unknown'});}finally{k.CloseHandle(h);}
   }
   ok=k.Process32NextW(snapshot,ptr(entry));
  }
  if(k.GetLastError()!==18)fail('enumeration-incomplete');
 }finally{k.CloseHandle(snapshot);}
 return {milliseconds:performance.now()-started,rows,unknown,foreign,exited,complete:unknown.length===0};
}
const base=join(process.env.OPENCODEX_HOME!,"native probe's");
const target=join(base,'opencodex-service.cmd'),other=join(base,'other-home','opencodex-service.cmd');
const fixturePaths=[target,target+'.backup',other],children:any[]=[];
try{
 for(const value of fixturePaths){children.push(Bun.spawn([join(system,'cmd.exe'),'/d','/q','/k','rem',value],{stdin:'pipe',stdout:'ignore',stderr:'ignore',env:process.env}));}
 await Bun.sleep(100);
 const expected=children.map(p=>p.pid),observations=[];
 for(let sample=0;sample<3;sample++)observations.push(enumerate([target],expected));
 for(const result of observations){for(let i=0;i<expected.length;i++){const row=result.rows.find(r=>r.pid===expected[i]);if(!row||row.matched!==(i===0))fail('fixture-match-or-identity');}}
 console.log(JSON.stringify({probe:'readonly-native-wrapper-enumeration',accepted:false,diagnosticOnly:true,productStarted:false,serviceOrTaskMutation:false,terminationApiUsed:false,fixtureCount:3,exactMatchAndSuffixForeignHomeCounterexamples:true,observations,limitations:'Test-only API prototype. Unknown candidates remain unknown; no service absence or actual Stop verdict. No maintained runtime changed. Command lines and token data omitted.'}));
}finally{
 for(const child of children){child.stdin.write('exit\r\n');child.stdin.end();}
 await Promise.all(children.map(child=>child.exited));
 kernel.close();advapi.close();ntdll.close();
}
