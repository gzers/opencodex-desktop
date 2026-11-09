// Isolated prototype: CREATE_NEW explicit TokenUser owner, original hardening
// and atomic replace. Never changes an existing object's owner or permissions.
import assert from 'node:assert/strict';
import {dlopen,ptr,FFIType as T} from 'bun:ffi';
import {readFileSync,statSync,existsSync,openSync,closeSync,fstatSync,writeFileSync,fsyncSync} from 'node:fs';
import {join,resolve,relative,isAbsolute} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
assert(process.platform==='win32'&&process.arch==='x64'&&process.env.GITHUB_ACTIONS==='true');
const task=resolve(process.env.OCXD_RECORD_PROBE_TASK),h=resolve(process.env.OPENCODEX_HOME),runtime=resolve(process.env.OCXD_RECORD_PROBE_PACKAGE);
const below=(a,p)=>{const r=relative(a,p);return r&&!r.startsWith('..')&&!isAbsolute(r);};
assert(below(task,h)&&below(resolve(process.env.RUNNER_TEMP),runtime));
const metadata=JSON.parse(readFileSync(join(runtime,'package.json'),'utf8'));assert.equal(metadata.version,'2.97.0-desktop023.names.2');
const module=async p=>import(pathToFileURL(join(runtime,p)).href);
const elevation=await module('src/lib/windows-elevation.ts');
const identity=await module('src/codex/windows-current-sid.ts');
const acl=await module('src/lib/windows-secret-acl.ts');
const atomic=await module('src/lib/windows-atomic-replace.ts');
const sid=identity.resolveWindowsCurrentSid(()=>{throw Error('Native SID lookup required');});
assert(/^S-1-[0-9-]+$/.test(sid));
const system=elevation.resolveTrustedWindowsSystemDirectory();
const k=dlopen(join(system,'kernel32.dll'),{
 CreateFileW:{args:[T.ptr,T.u32,T.u32,T.ptr,T.u32,T.u32,T.u64],returns:T.u64},
 WriteFile:{args:[T.u64,T.ptr,T.u32,T.ptr,T.ptr],returns:T.i32},
 FlushFileBuffers:{args:[T.u64],returns:T.i32},
 GetFileInformationByHandle:{args:[T.u64,T.ptr],returns:T.i32},
 GetFileType:{args:[T.u64],returns:T.u32},
 CloseHandle:{args:[T.u64],returns:T.i32},
 LocalFree:{args:[T.u64],returns:T.u64},
});
const a=dlopen(join(system,'advapi32.dll'),{
 ConvertStringSecurityDescriptorToSecurityDescriptorW:{args:[T.ptr,T.u32,T.ptr,T.ptr],returns:T.i32},
});
const wide=s=>{const value=new Uint16Array(s.length+1);for(let i=0;i<s.length;i++)value[i]=s.charCodeAt(i);return value;};
const digest=b=>createHash('sha256').update(b).digest('hex');
function information(handle){
 const storage=new ArrayBuffer(52),bytes=new Uint8Array(storage),v=new DataView(storage);
 assert(k.symbols.GetFileInformationByHandle(handle,ptr(bytes)));
 assert.equal(k.symbols.GetFileType(handle),1);
 assert.equal(v.getUint32(0,true)&(0x10|0x400),0);assert.equal(v.getUint32(40,true),1);
 return [v.getUint32(28,true),v.getUint32(44,true),v.getUint32(48,true)];
}
function createNew(path,content,viaDescriptor=false){
 assert(below(h,path));
 const security=new BigUint64Array(1),sddl=wide('O:'+sid+'D:P(A;;FA;;;'+sid+')');
 assert(a.symbols.ConvertStringSecurityDescriptorToSecurityDescriptorW(ptr(sddl),1,ptr(security),null));
 const storage=new ArrayBuffer(24),sa=new Uint8Array(storage),view=new DataView(storage);view.setUint32(0,24,true);view.setBigUint64(8,security[0],true);view.setUint32(16,0,true);
 const name=wide(path);let handle=0n;
 try{
  handle=BigInt(k.symbols.CreateFileW(ptr(name),0xc0020000,3,ptr(sa),1,0x80|0x200000,0n));
  if(handle===0xffffffffffffffffn||handle===0n)throw Error('CREATE_NEW refused');
  const before=information(handle),bytes=Buffer.from(content),written=new Uint32Array(1);
  if(viaDescriptor){
   // Native no-share-delete handle pins the empty CREATE_NEW object until
   // the existing fs descriptor chain has acquired and checked that object.
   const fd=openSync(path,'r+');
   try{
    const disk=statSync(path,{bigint:true}),opened=fstatSync(fd,{bigint:true});
    assert(opened.isFile()&&opened.nlink===1n&&opened.ino!==0n);assert.equal(disk.ino,opened.ino);assert.equal(disk.dev,opened.dev);
    acl.hardenSecretPath(path,{required:true});
    writeFileSync(fd,content,{encoding:'utf8'});fsyncSync(fd);
    const after=fstatSync(fd,{bigint:true});assert.equal(after.ino,opened.ino);assert.equal(after.dev,opened.dev);assert.equal(after.nlink,1n);
   }finally{closeSync(fd);}
  }else{
   assert(k.symbols.WriteFile(handle,ptr(bytes),bytes.length,ptr(written),null));assert.equal(written[0],bytes.length);
  }
  assert(k.symbols.FlushFileBuffers(handle));
  assert.deepEqual(information(handle),before);
 }finally{
  if(handle!==0n&&handle!==0xffffffffffffffffn)assert(k.symbols.CloseHandle(handle));
  k.symbols.LocalFree(security[0]);
 }
}
try{
 const target=join(h,'native-runtime-port.json'),temp=join(h,'native-runtime-port.new.tmp');assert(!existsSync(target)&&!existsSync(temp));
 const bytes=JSON.stringify({pid:process.pid,port:1,attestationSecret:'AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA'});
 createNew(temp,bytes);const initial=statSync(temp,{bigint:true});assert(initial.isFile()&&initial.nlink===1n&&initial.ino!==0n);
 assert.throws(()=>createNew(temp,'must-not-replace'));
 assert.equal(digest(readFileSync(temp)),digest(Buffer.from(bytes)));
 // Reuse original ACL hardener and atomic rename; creation identity is checked
 // before/after both. Production integration still needs ancestor-pinning proof.
 acl.hardenSecretPath(temp,{required:true});
 const hardened=statSync(temp,{bigint:true});assert.equal(hardened.ino,initial.ino);assert.equal(hardened.dev,initial.dev);assert.equal(hardened.nlink,1n);
 atomic.renameAtomicFile(temp,target);
 const published=statSync(target,{bigint:true});assert.equal(published.ino,initial.ino);assert.equal(published.dev,initial.dev);assert.equal(published.nlink,1n);
 assert.equal(digest(readFileSync(target)),digest(Buffer.from(bytes)));assert(!existsSync(temp));
 const bridgeTemp=join(h,'native-fd-runtime-port.new.tmp'),bridgeTarget=join(h,'native-fd-runtime-port.json');
 createNew(bridgeTemp,bytes,true);const beforeBridge=statSync(bridgeTemp,{bigint:true});
 atomic.renameAtomicFile(bridgeTemp,bridgeTarget);const afterBridge=statSync(bridgeTarget,{bigint:true});
 assert.equal(beforeBridge.ino,afterBridge.ino);assert.equal(beforeBridge.dev,afterBridge.dev);assert.equal(afterBridge.nlink,1n);assert.equal(digest(readFileSync(bridgeTarget)),digest(Buffer.from(bytes)));
 console.log(JSON.stringify({accepted:false,diagnosticOnly:true,explicitNewOwner:true,existingCreateRefused:true,originalAclHardenerUsed:true,originalAtomicRenameUsed:true,identityAndBytesPreserved:true,existingFsDescriptorBridge:true,bridgeCreatedObjectPinned:true,bridgeHardenedBeforeContent:true,noExistingOwnerRepair:true,runtimeChanged:false}));
}finally{k.close();a.close();}
