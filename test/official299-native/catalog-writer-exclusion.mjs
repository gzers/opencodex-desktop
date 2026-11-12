// Test-only caller of the pinned, unmodified distribution's actual primitives.
// No service, network, credentials or live user client files.
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import {pathToFileURL} from 'node:url';
const [pkg, home, codex, expected] = process.argv.slice(-4);
if(!['blocked','released'].includes(expected) || process.env.CODEX_HOME!==codex || process.env.OPENCODEX_HOME!==home) throw Error('test context');
const imp=p=>import(pathToFileURL(join(pkg,'src',p)).href);
const [serial, writer, lifecycle, config] = await Promise.all([
  imp('codex/catalog-write-serialization.ts'), imp('codex/internal/catalog-writer.ts'),
  imp('client/lifecycle-lock.ts'), imp('config.ts'),
]);
const names=['opencodex-catalog.json','models_cache.json','opencodex-catalog-audit.jsonl'];
const snapshot=()=>names.map(name=>{try{return createHash('sha256').update(readFileSync(join(codex,name))).digest('hex');}catch(e){if(e.code==='ENOENT')return 'absent';throw e;}});
const before=snapshot();
const catalog=JSON.stringify({models:[{slug:'native-writer-fixture',display_name:'synthetic K writer'}]});
const prepared={path:join(codex,names[0]),content:catalog};
let forgedRefused=false;
try{writer.replaceActiveCodexCatalog({},codex,prepared);}catch(e){forgedRefused=e?.code==='CODEX_CATALOG_WRITE_PERMIT_REFUSED';}
if(!forgedRefused || JSON.stringify(snapshot())!==JSON.stringify(before)) throw Error('forged permit');
let kRan=false, leaked;
const k=serial.withCatalogWriteSerialization(codex,permit=>{
  kRan=true; leaked=permit;
  writer.replaceActiveCodexCatalog(permit,codex,prepared);
  writer.replaceCodexModelsCache(permit,codex,{path:join(codex,names[1]),content:catalog});
  serial.auditCatalogWriteWithPermit(permit,codex,{target:'catalog',outcome:'written',configSource:'file',routedBefore:0,routedAfter:0});
},{intent:'restore',writer:'codex-restore'});
let lRan=false, lKind;
try{lifecycle.withClientLifecycleSync(()=>{
  lRan=true;
  // Same L-protected direct catalog IO as client/connect.ts, without a network
  // connection or writing any client credential/configuration.
  config.atomicWriteFile(join(codex,names[0]),JSON.stringify({models:[{slug:'native-writer-fixture',display_name:'synthetic L writer'}]}));
});lKind='completed';}catch(e){if(e?.code!=='client_lifecycle_busy')throw e;lKind='busy';}
let revokedRefused=null;
if(leaked){try{writer.replaceActiveCodexCatalog(leaked,codex,prepared);revokedRefused=false;}catch(e){revokedRefused=e?.code==='CODEX_CATALOG_WRITE_PERMIT_REFUSED';}}
const after=snapshot();
if(expected==='blocked'){
  if(k.kind!=='unavailable'||k.reason!=='busy'||kRan||lKind!=='busy'||lRan||JSON.stringify(before)!==JSON.stringify(after))throw Error('writer not excluded');
}else{
  if(k.kind!=='completed'||!kRan||lKind!=='completed'||!lRan||revokedRefused!==true||!after.every((sha,i)=>sha!==before[i]))throw Error('positive writer control failed');
}
console.log(JSON.stringify({expected,k:{kind:k.kind,reason:k.reason,callbackRan:kRan},l:{kind:lKind,callbackRan:lRan},forgedRefused,revokedRefused,before,after}));
