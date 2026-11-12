// Desktop-owned bounded lock holder. Imports the admitted official 2.99 files
// unchanged. No client mutations, credential reads, service commands or network.
import { createHash } from 'node:crypto';
import { lstatSync, readFileSync, realpathSync, writeSync,openSync,closeSync,fstatSync,readSync,constants } from 'node:fs';
import { join, isAbsolute } from 'node:path';
import { pathToFileURL } from 'node:url';

const schema='ocxd-client-coordination/2';
let stage='input', acquired=false, released=false;
let params;
const safePath=p=>typeof p==='string'&&isAbsolute(p)&&!/[\x00-\x1f\x7f]/.test(p);
const emit=value=>writeSync(1,JSON.stringify(value)+'\n');
const identity=path=>{
  const s=lstatSync(path,{bigint:true});
  if(!s.isFile()||s.isSymbolicLink()||s.nlink!==1n||s.uid!==BigInt(process.getuid())||(s.mode&0o777n)!==0o600n)throw Error('unsafe-lock');
  return {device:String(s.dev),inode:String(s.ino)};
};
function optionalLock(path){
  try{return identity(path);}catch(e){if(e?.code==='ENOENT')return null;throw e;}
}
const same=(a,b)=>a.device===b.device&&a.inode===b.inode;
function ownerReceipt(home) {
  const path=join(home,'opencodex-journal.json');let before;
  try{before=lstatSync(path,{bigint:true});}catch(e){if(e?.code==='ENOENT')return 'absent';throw e;}
  if(!before.isFile()||before.isSymbolicLink()||before.nlink!==1n||before.uid!==BigInt(process.getuid())||before.size>1048576n)throw Error('owner-journal-unavailable');
  const fd=openSync(path,constants.O_RDONLY|constants.O_NOFOLLOW|constants.O_NONBLOCK);
  try {
    const opened=fstatSync(fd,{bigint:true});
    if(opened.dev!==before.dev||opened.ino!==before.ino||opened.size!==before.size)throw Error('owner-journal-changed');
    const buffer=Buffer.alloc(1048577);let used=0;
    while(used<buffer.length){const n=readSync(fd,buffer,used,buffer.length-used,null);if(!n)break;used+=n;}
    if(used>1048576)throw Error('owner-journal-over-limit');
    const bytes=buffer.subarray(0,used);const after=lstatSync(path,{bigint:true});
    if(after.dev!==opened.dev||after.ino!==opened.ino||after.size!==BigInt(bytes.length)||after.mtimeNs!==before.mtimeNs||after.ctimeNs!==before.ctimeNs)throw Error('owner-journal-changed');
    return createHash('sha256').update(JSON.stringify([String(after.dev),String(after.ino),String(after.uid),String(after.gid),String(after.mode&0o7777n),createHash('sha256').update(bytes).digest('hex')])).digest('hex');
  }finally{closeSync(fd);}
}
try {
  if(process.platform!=='darwin')throw Error('unsupported-platform');
  params=JSON.parse(process.argv.at(-1));
  const keys=['schema','operation','packageRoot','profile','codexHome','opencodexHome','releasePath','holdMs'];
  if(!params||Object.keys(params).length!==keys.length||Object.keys(params).some(k=>!keys.includes(k))
    ||params.schema!==schema||!/^[a-f0-9-]{36}$/.test(params.operation)
    ||!Number.isSafeInteger(params.holdMs)||params.holdMs<1||params.holdMs>2000
    ||!['packageRoot','profile','codexHome','opencodexHome','releasePath'].every(k=>safePath(params[k]))
    ||process.env.HOME!==params.profile||process.env.CODEX_HOME!==params.codexHome||process.env.OPENCODEX_HOME!==params.opencodexHome)throw Error('invalid-input');
  const p=params;
  for(const path of [p.packageRoot,p.profile,p.codexHome,p.opencodexHome]) {
    if(realpathSync.native(path)!==path||!lstatSync(path).isDirectory())throw Error('noncanonical-context');
  }
  try{lstatSync(p.releasePath);throw Error('release-already-present');}catch(e){if(e?.code!=='ENOENT')throw e;}
  const declaration=JSON.parse(readFileSync(join(p.packageRoot,'package.json'),'utf8'));
  if(declaration.name!=='@bitkyc08/opencodex'||declaration.version!=='2.99.0')throw Error('wrong-official-version');
  stage='imports';
  const imp=relative=>import(pathToFileURL(join(p.packageRoot,'src',relative)).href);
  const [{resolveNativeProfileContext},{withNativeMainExclusiveClaim},{openCodexCoordinatorTransaction},user,{withCatalogWriteSerialization},{withClientLifecycleSync}]=await Promise.all([
    imp('codex/native-profile-store.ts'),imp('codex/native-main-claim.ts'),imp('codex/transition-state.ts'),imp('codex/user-identity.ts'),imp('codex/catalog-write-serialization.ts'),imp('client/lifecycle-lock.ts')]);
  stage='context';
  const context=resolveNativeProfileContext();
  if(context.codexHome!==p.codexHome||context.configDir!==p.opencodexHome)throw Error('context-mismatch');
  const uid=user.resolveEffectiveUserIdentity();
  const npath=user.resolveCodexCoordinatorDatabasePath(uid,context.codexHome);
  const kpath=user.resolveCodexCatalogSerializationDatabasePath(uid,context.codexHome);
  const lpath=join(user.resolveEffectiveUserRuntimeRoot(uid),'client-desktop-lifecycle.sqlite');
  const claim=join(p.codexHome,'.opencodex-native-main.claim.sqlite');
  // Refuse an unsafe existing file before an official primitive might narrow
  // its mode. Never repair an old object's permissions on Desktop's behalf.
  for(const path of [claim,npath,lpath,kpath])for(const suffix of ['','-journal','-wal','-shm'])optionalLock(path+suffix);
  const expectedHash=createHash('sha256').update(JSON.stringify([p.packageRoot,p.profile,p.codexHome,p.opencodexHome])).digest('hex');
  let locks,owner;
  const paths=[claim,npath,lpath,kpath];
  stage='claim';
  await withNativeMainExclusiveClaim(context,async()=>{
    // The official constructor initializes a clean first-use database inside
    // its transaction. Rolling that initialization back leaves an empty N
    // which the next legitimate startup can no longer admit after routing was
    // published. Persist ONLY that official clean initial row after successful
    // coordination; existing versions/failed callbacks still roll back.
    let freshNative=false;
    try{freshNative=lstatSync(npath).size===0;}catch(e){if(e?.code==='ENOENT')freshNative=true;else throw e;}
    stage='native';const n=openCodexCoordinatorTransaction(npath);
    const nativeBefore=n.version();let nativeCommitted=false;
    try {
      stage='lifecycle';
      const result=withClientLifecycleSync(()=>{
        stage='catalog';
        return withCatalogWriteSerialization(context.codexHome,()=>{
        acquired=true;stage='held';
        locks=paths.map(identity);
        owner=ownerReceipt(context.codexHome);
        const expires=performance.now()+p.holdMs;
        emit({schema,kind:'held',operation:p.operation,pid:process.pid,contextSha256:expectedHash,locks,paths,owner});
        const parent=process.ppid;
        while(performance.now()<expires){
          if(process.ppid!==parent)throw Error('parent-changed');
          try {
            const s=lstatSync(p.releasePath);
            if(!s.isFile()||s.isSymbolicLink()||s.nlink!==1||s.uid!==process.getuid()||(s.mode&0o777)!==0o600||s.size!==p.operation.length+1)throw Error('invalid-release');
            const fdBytes=readFileSync(p.releasePath,'utf8');
            if(fdBytes!==p.operation+'\n')throw Error('invalid-release');
            released=true;break;
          }catch(e){if(e?.code!=='ENOENT')throw e;}
          Atomics.wait(new Int32Array(new SharedArrayBuffer(4)),0,0,5);
        }
        if(!released)throw Error('hold-deadline');
        n.assertStablePath();
        if(paths.map(identity).some((value,i)=>!same(value,locks[i])))throw Error('lock-replaced');
        if(ownerReceipt(context.codexHome)!==owner)throw Error('owner-journal-changed');
        },{intent:'restore',writer:'codex-restore'});
      });
      if(result.kind!=='completed')throw Error('catalog-refused');
      if(freshNative){
        const current=n.version();
        if(nativeBefore.nativeGeneration!==0||nativeBefore.currentTxId!==null||current.nativeGeneration!==0||current.currentTxId!==null)throw Error('initial-native-state-changed');
        n.assertStablePath();n.commit();nativeCommitted=true;
      }
    }finally{if(!nativeCommitted)n.rollback();n.close();}
  },{waitMs:0});
  // Emitted only after every official callback's finally has released its lock.
  stage='released';
  const final=paths.map(identity);
  if(final.some((value,i)=>!same(value,locks[i])))throw Error('lock-replaced');
  emit({schema,kind:'released',operation:p.operation,pid:process.pid,contextSha256:expectedHash,locks:final,paths,owner});
}catch {
  // No upstream exception text, HOME, credential material or arbitrary argv.
  emit({schema,kind:'failed',stage,acquired,released});
  process.exitCode=1;
}
