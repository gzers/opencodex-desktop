// Desktop adapter to unchanged official public lock APIs. No timer unlocks a
// live parent. One exact destination is registered BEFORE any directory move,
// so pipe EOF can release the original lock even before relocation is acked.
import {readSync,writeSync,realpathSync,lstatSync,readFileSync,openSync,closeSync,fstatSync,constants} from 'node:fs';
import {join,dirname,isAbsolute,normalize} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const schema='ocxd-config-write-guard/2';
let p,anchor,planned,rootFd,lockFd,rootStat,lockStat,lockBytes,held,stillHeld,release;
let phase='input';
const hash=path=>createHash('sha256').update(path).digest('hex');
const emit=(kind,target=anchor)=>writeSync(1,JSON.stringify({schema,kind,operation:p.operation,pid:process.pid,targetSha256:hash(target),phase})+'\n');
const stat=path=>lstatSync(path,{bigint:true});
const sameRoot=(a,b)=>a.dev===b.dev&&a.ino===b.ino&&a.birthtimeNs===b.birthtimeNs&&a.uid===b.uid&&a.gid===b.gid&&a.mode===b.mode;
const same=(a,b)=>sameRoot(a,b)&&a.nlink===b.nlink;
const absent=path=>{try{stat(path);return false;}catch(e){if(e?.code==='ENOENT')return true;throw e;}};
function exact(path){
 if(realpathSync.native(path)!==path)throw Error('anchor');
 const r=stat(path),l=stat(join(path,'config.toml.ocx-write.lock'));
 // APFS directory nlink counts entries and changes when our own lock/claims
 // are created. It is not directory generation or protection. File nlink
 // remains exact and single; root identity/uid/gid/mode are still retained.
 if(!r.isDirectory()||!sameRoot(r,rootStat)||!sameRoot(fstatSync(rootFd,{bigint:true}),rootStat)||!l.isFile()||l.nlink!==1n||!same(l,lockStat)||!same(fstatSync(lockFd,{bigint:true}),lockStat))throw Error('identity');
 const bytes=readFileSync(join(path,'config.toml.ocx-write.lock'));
 if(!bytes.equals(lockBytes)||!same(stat(join(path,'config.toml.ocx-write.lock')),lockStat))throw Error('changed');
 const handle={path:join(path,'config.toml.ocx-write.lock'),token:held.token};
 if(!stillHeld(handle))throw Error('superseded');return handle;
}
function readCommand(){
 const bytes=[];const one=Buffer.alloc(1);
 for(let i=0;i<16384;i++){const n=readSync(0,one,0,1,null);if(!n){if(bytes.length)throw Error('partial');return null;}if(one[0]===10)return JSON.parse(Buffer.from(bytes).toString('utf8'));bytes.push(one[0]);}
 throw Error('length');
}
function cleanup(){
 if(!held)return;
 // Check only the two explicitly registered locations, never scan or guess.
 // A replacement root/lock cannot match the retained native inode.
 for(const candidate of [...new Set([anchor,planned].filter(Boolean))]){
  try{const handle=exact(candidate);if(!release(handle))throw Error('release');held=null;return;}catch{}
 }
}
try{
 p=JSON.parse(process.argv.at(-1));anchor=p.target;
 const keys=['schema','operation','packageRoot','profile','home','codex','target'];
 if(Object.keys(p).length!==keys.length||Object.keys(p).some(k=>!keys.includes(k))||p.schema!==schema||!/^[a-f0-9-]{36}$/.test(p.operation)||process.env.HOME!==p.profile||process.env.OPENCODEX_HOME!==p.home||process.env.CODEX_HOME!==p.codex)throw Error('input');
 if(p.target!==p.codex&&p.target!==join(p.profile,'.grok'))throw Error('target');
 for(const path of [p.packageRoot,p.profile,p.home,p.codex,p.target])if(realpathSync.native(path)!==path||!stat(path).isDirectory())throw Error('context');
 const pkg=JSON.parse(readFileSync(join(p.packageRoot,'package.json'),'utf8'));if(pkg.name!=='@bitkyc08/opencodex'||pkg.version!=='2.99.0')throw Error('source');
 const {acquireConfigWriteLock}=await import(pathToFileURL(join(p.packageRoot,'src/codex/config-write-lock.ts')).href);
 ({stillHeld,release}=await import(pathToFileURL(join(p.packageRoot,'src/codex/prompt-lock.ts')).href));
 phase='root-pin';rootFd=openSync(anchor,constants.O_RDONLY|constants.O_DIRECTORY|constants.O_NOFOLLOW);rootStat=fstatSync(rootFd,{bigint:true});
 phase='acquire';
 const acquired=await acquireConfigWriteLock(join(anchor,'config.toml'),{timeoutMs:0});if(!acquired.ok)throw Error('locked');held=acquired.handle;
 phase='initial-check';lockFd=openSync(held.path,constants.O_RDONLY|constants.O_NOFOLLOW);lockStat=fstatSync(lockFd,{bigint:true});lockBytes=readFileSync(lockFd);exact(anchor);emit('held');
 let success=false;
 for(let count=0;count<4;count++){
  phase='command';const cmd=readCommand();if(cmd===null)break;
  if(cmd.operation!==p.operation||!['prepare','relocate','release'].includes(cmd.kind))throw Error('command');
  if(cmd.kind==='prepare'){
   phase='prepare';
   if(Object.keys(cmd).length!==3||planned||anchor!==p.codex||typeof cmd.target!=='string'||!isAbsolute(cmd.target)||normalize(cmd.target)!==cmd.target||cmd.target===anchor||realpathSync.native(dirname(cmd.target))!==dirname(cmd.target)||!absent(cmd.target))throw Error('plan');
   exact(anchor);planned=cmd.target;emit('prepared',planned);continue;
  }
  if(cmd.kind==='relocate'){
   phase='relocate';
   if(Object.keys(cmd).length!==3||!planned||cmd.target!==planned||anchor!==p.codex||!absent(anchor))throw Error('move');
   exact(planned);anchor=planned;emit('relocated');continue;
  }
  phase='release';if(Object.keys(cmd).length!==2)throw Error('release-command');
  const handle=exact(anchor);if(!release(handle)||!absent(handle.path))throw Error('release');held=null;success=true;emit('released');break;
 }
 if(!success)throw Error('not-released');
}catch{if(p&&typeof p.operation==='string'&&typeof anchor==='string')emit('failed');process.exitCode=1;}
finally{cleanup();if(lockFd!==undefined)closeSync(lockFd);if(rootFd!==undefined)closeSync(rootFd);}
