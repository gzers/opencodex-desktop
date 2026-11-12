// Desktop-owned adapter to the UNMODIFIED official config lock. The holder
// blocks on its parent's pipe, with no timeout that could unlock live work.
import {readSync,writeSync,realpathSync,lstatSync,readFileSync} from 'node:fs';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
let p;const emit=kind=>writeSync(1,JSON.stringify({schema:'ocxd-config-write-guard/1',kind,operation:p.operation,pid:process.pid})+'\n');
try{
 p=JSON.parse(process.argv.at(-1));
 const keys=['schema','operation','packageRoot','profile','home','codex'];
 if(Object.keys(p).length!==keys.length||Object.keys(p).some(k=>!keys.includes(k))||p.schema!=='ocxd-config-write-guard/1'||!/^[a-f0-9-]{36}$/.test(p.operation)||process.env.HOME!==p.profile||process.env.OPENCODEX_HOME!==p.home||process.env.CODEX_HOME!==p.codex)throw Error('input');
 for(const path of [p.packageRoot,p.profile,p.home,p.codex])if(realpathSync.native(path)!==path||!lstatSync(path).isDirectory())throw Error('context');
 const pkg=JSON.parse(readFileSync(join(p.packageRoot,'package.json'),'utf8'));if(pkg.name!=='@bitkyc08/opencodex'||pkg.version!=='2.99.0')throw Error('source');
 const {withConfigWriteLock}=await import(pathToFileURL(join(p.packageRoot,'src/codex/config-write-lock.ts')).href);
 const {stillHeld}=await import(pathToFileURL(join(p.packageRoot,'src/codex/prompt-lock.ts')).href);
 let released=false;
 const result=withConfigWriteLock(join(p.codex,'config.toml'),held=>{
  if(!stillHeld(held))throw Error('unheld');emit('held');
  const expected=Buffer.from('release:'+p.operation+'\n');const received=Buffer.alloc(expected.length);let used=0;
  while(used<received.length){const n=readSync(0,received,used,received.length-used,null);if(!n)break;used+=n;}
  if(!stillHeld(held))throw Error('superseded');
  // Parent death/pipe EOF releases normally, but cannot create a success reply.
  released=used===expected.length&&received.equals(expected);
 });
 if(!result.ok||!released)throw Error('not-released');
 try{lstatSync(join(p.codex,'config.toml.ocx-write.lock'));throw Error('lock-remains');}catch(e){if(e?.code!=='ENOENT')throw e;}
 emit('released');
}catch{if(p&&typeof p.operation==='string')emit('failed');process.exitCode=1;}
