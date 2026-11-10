// Test-only bounded cross-process barrier. No namespace reimplementation or lock unlink.
import {join} from 'node:path';import {writeFileSync,existsSync} from 'node:fs';
const [source,ready,release,result,holdArg]=process.argv.slice(2),hold=Math.min(2000,Number(holdArg));
const imp=(p:string)=>import(join(source,'src',p));
const {resolveNativeProfileContext}=await imp('codex/native-profile-store.ts');
const {withNativeMainExclusiveClaim}=await imp('codex/native-main-claim.ts');
const {openCodexCoordinatorTransaction}=await imp('codex/transition-state.ts');
const {resolveEffectiveUserIdentity,resolveCodexCoordinatorDatabasePath,resolveCodexCatalogSerializationDatabasePath}=await imp('codex/user-identity.ts');
const {withCatalogWriteSerialization}=await imp('codex/catalog-write-serialization.ts');
const context=resolveNativeProfileContext(),identity=resolveEffectiveUserIdentity(),npath=resolveCodexCoordinatorDatabasePath(identity,context.codexHome),kpath=resolveCodexCatalogSerializationDatabasePath(identity,context.codexHome);
const r:any={pid:process.pid,home:context.codexHome,claim:false,n:false,k:false,released:false,normalResult:false,namespace:{npath,kpath},barrierMaxMs:hold};
try{await withNativeMainExclusiveClaim(context,async()=>{r.claim=true;const n=openCodexCoordinatorTransaction(npath);r.n=true;try{const k=withCatalogWriteSerialization(context.codexHome,()=>{r.k=true;const deadline=performance.now()+hold;writeFileSync(ready,JSON.stringify(r),{flag:'wx'});while(!existsSync(release)&&performance.now()<deadline){Atomics.wait(new Int32Array(new SharedArrayBuffer(4)),0,0,5);}if(!existsSync(release))throw Error('bounded-native-compensation-barrier-expired');return 'native-host-completed';},{intent:'restore',writer:'codex-restore'});if(k.kind!=='completed')throw Error('official-K-refused:'+k.reason);}finally{n.rollback();n.close();}},{waitMs:0});r.released=true;r.normalResult=true;}
catch(e:any){r.error={name:e.name,code:e.code??null,message:e.message};process.exitCode=1;}
finally{writeFileSync(result,JSON.stringify(r));}
