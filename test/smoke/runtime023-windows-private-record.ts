/** Candidate source for the maintained runtime; not installed or activated. */
import {dlopen,ptr,FFIType as T} from 'bun:ffi';
import {openSync,closeSync,fstatSync,lstatSync} from 'node:fs';
import {basename,dirname,win32} from 'node:path';

type Native = ReturnType<typeof native>;
let loaded: Native | undefined;
const WINDOWS = process.platform === 'win32';
const ARCH = process.arch;
function refusal(): never { throw new Error('Private runtime record creation or identity could not be proved'); }
function wide(value:string):Uint16Array {
  const result=new Uint16Array(value.length+1);
  for(let n=0;n<value.length;n++)result[n]=value.charCodeAt(n);
  return result;
}
function native() {
  // Production placement: src/lib/windows-runtime-private-record.ts.
  const {resolveTrustedWindowsSystemDirectory}=require('./windows-elevation') as typeof import('./windows-elevation');
  const {resolveWindowsCurrentSid}=require('../codex/windows-current-sid') as typeof import('../codex/windows-current-sid');
  const sid=resolveWindowsCurrentSid(()=>refusal());
  if(!/^S-1-[0-9-]+$/.test(sid)||sid==='S-1-5-18'||sid==='S-1-5-32-544')refusal();
  const directory=resolveTrustedWindowsSystemDirectory();
  const kernel=dlopen(win32.join(directory,'kernel32.dll'),{
    CreateFileW:{args:[T.ptr,T.u32,T.u32,T.ptr,T.u32,T.u32,T.u64],returns:T.u64},
    GetFileInformationByHandle:{args:[T.u64,T.ptr],returns:T.i32},
    GetFileType:{args:[T.u64],returns:T.u32},
    SetFileInformationByHandle:{args:[T.u64,T.i32,T.ptr,T.u32],returns:T.i32},
    CloseHandle:{args:[T.u64],returns:T.i32},
    LocalFree:{args:[T.u64],returns:T.u64},
  });
  const security=dlopen(win32.join(directory,'advapi32.dll'),{
    ConvertStringSecurityDescriptorToSecurityDescriptorW:{args:[T.ptr,T.u32,T.ptr,T.ptr],returns:T.i32},
  });
  return {kernel:kernel.symbols,security:security.symbols,sid};
}
function applicable(target:string):boolean {
  if(!WINDOWS||basename(target).toLowerCase()!=='runtime-port.json')return false;
  if(ARCH!=='x64')refusal();
  return true;
}
function validate(path:string):void {
  if(!win32.isAbsolute(path)||path.startsWith(String.fromCharCode(92,92,63,92))||path.startsWith(String.fromCharCode(92,92,46,92)))refusal();
  const root=win32.parse(path).root;
  for(const part of path.slice(root.length).replaceAll(String.fromCharCode(92),'/').split('/')) {
    if(!part||part==='.'||part==='..'||Array.from(part).some(c=>c.charCodeAt(0)<32||[58,60,62,34,124,63,42].includes(c.charCodeAt(0)))||/[ .]$/.test(part)||/^(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])$/i.test(part.split('.')[0]!))refusal();
  }
}
function info(api:Native,handle:bigint,directory:boolean) {
  const storage=new ArrayBuffer(52),bytes=new Uint8Array(storage),v=new DataView(storage);
  if(!api.kernel.GetFileInformationByHandle(handle,ptr(bytes))||api.kernel.GetFileType(handle)!==1)refusal();
  const flags=v.getUint32(0,true),links=v.getUint32(40,true);
  if(flags&0x400||!!(flags&0x10)!==directory||!directory&&links!==1)refusal();
  return {volume:v.getUint32(28,true),index:(BigInt(v.getUint32(44,true))<<32n)|BigInt(v.getUint32(48,true)),size:(BigInt(v.getUint32(32,true))<<32n)|BigInt(v.getUint32(36,true))};
}
function same(a:ReturnType<typeof info>,b:ReturnType<typeof info>):boolean{return a.volume===b.volume&&a.index===b.index;}

/** Retained directory handles cover CREATE_NEW, writing and every rename attempt. */
export function pinWindowsRuntimeRecordPublication(target:string):(()=>void)|undefined {
  if(!applicable(target))return undefined;
  validate(target);const api=loaded??=native(),parents:string[]=[],pins:bigint[]=[];
  let parent=dirname(target);
  for(;;){parents.push(parent);const next=dirname(parent);if(next===parent)break;parent=next;}
  try {
    for(const path of parents.reverse()) {
      const name=wide(path),h=BigInt(api.kernel.CreateFileW(ptr(name),0x80000000,3,null,3,0x02000000|0x00200000,0n));
      if(h===0n||h===0xffffffffffffffffn)refusal();
      pins.push(h);info(api,h,true);
    }
    return ()=>{for(const h of pins.reverse())if(!api.kernel.CloseHandle(h))refusal();};
  } catch(error) {for(const h of pins.reverse())api.kernel.CloseHandle(h);throw error;}
}

/** Empty CREATE_NEW only. No SetSecurityInfo or permission change on old objects. */
export function createWindowsRuntimeRecordTemp(path:string,target:string):number|undefined {
  if(!applicable(target))return undefined;
  validate(path);validate(target);
  if(dirname(path)!==dirname(target)||!basename(path).startsWith(basename(target)+'.ocx.')||!path.endsWith('.tmp'))refusal();
  const api=loaded??=native(),descriptor=new BigUint64Array(1);
  const sddl=wide('O:'+api.sid+'D:P(A;;FA;;;'+api.sid+')');
  if(!api.security.ConvertStringSecurityDescriptorToSecurityDescriptorW(ptr(sddl),1,ptr(descriptor),null))refusal();
  const storage=new ArrayBuffer(24),attributes=new Uint8Array(storage),v=new DataView(storage);
  v.setUint32(0,24,true);v.setBigUint64(8,descriptor[0]!,true);v.setUint32(16,0,true);
  const name=wide(path);let h=0n,fd:number|undefined,created:ReturnType<typeof info>|undefined,completed=false;
  try {
    // DELETE access is only for exact-handle cleanup of this new EMPTY object
    // if the descriptor handoff fails. Sharing deliberately excludes DELETE.
    h=BigInt(api.kernel.CreateFileW(ptr(name),0xc0030000,3,ptr(attributes),1,0x80|0x00200000,0n));
    if(h===0n||h===0xffffffffffffffffn)refusal();
    created=info(api,h,false);if(created.size!==0n)refusal();
    fd=openSync(path,'r+');
    const opened=fstatSync(fd,{bigint:true}),named=lstatSync(path,{bigint:true}),again=info(api,h,false);
    if(!same(created,again)||again.size!==0n||!opened.isFile()||opened.nlink!==1n||opened.size!==0n||opened.ino===0n||opened.ino!==named.ino||opened.dev!==named.dev||opened.ino!==created.index)refusal();
    completed=true;return fd;
  } finally {
    try {
      if(h!==0n&&h!==0xffffffffffffffffn) {
        try {
          if(!completed) {
            if(fd!==undefined)closeSync(fd);
            // Only the retained CREATE_NEW object, unchanged and still empty, is removed.
            if(created){const now=info(api,h,false);if(same(created,now)&&now.size===0n){const disposition=new Uint8Array([1]);if(!api.kernel.SetFileInformationByHandle(h,4,ptr(disposition),1))refusal();}}
          }
        } finally { api.kernel.CloseHandle(h); }
      }
    } finally { api.kernel.LocalFree(descriptor[0]!); }
  }
}
