// Real official owner/claim APIs in a new process. No unlink or hardening bypass.
import {join} from 'node:path';
import {existsSync,readFileSync,writeFileSync,lstatSync} from 'node:fs';
const [source,kind,mode,output,release]=process.argv.slice(2),deadline=Date.now()+4000;
const {resolveNativeProfileContext}=await import(join(source,'src/codex/native-profile-store.ts'));
const {retainNativeMainOwner}=await import(join(source,'src/codex/native-main-owner.ts'));
const {withNativeMainExclusiveClaim}=await import(join(source,'src/codex/native-main-claim.ts'));
const context=resolveNativeProfileContext(),result:any={pid:process.pid,platform:process.platform,kind,mode,acquired:false,released:false,code:null};
const identity=()=>Object.fromEntries(['claim','owner'].map(k=>{const p=join(context.codexHome,'.opencodex-native-main.'+k+'.sqlite');try{const s=lstatSync(p,{bigint:true});return [k,{dev:String(s.dev),ino:String(s.ino),links:String(s.nlink),file:s.isFile(),symlink:s.isSymbolicLink()}]}catch(e:any){return[k,{unknown:true,code:e.code}]}}));
const save=()=>writeFileSync(output,JSON.stringify({...result,identity:identity()}));
async function held(){result.acquired=true;save();if(mode==='hold'){while(!existsSync(release)&&Date.now()<deadline)await Bun.sleep(15);if(!existsSync(release))throw Error('hold envelope elapsed');}}
try {
 if(kind==='owner'){
  const ref=retainNativeMainOwner(context);
  try{let s=ref.snapshot();while(s.status==='acquiring'&&Date.now()<deadline){await Bun.sleep(10);s=ref.snapshot();}
   result.ownerSnapshot=s;if(s.status==='held')await held();else if(s.status==='contended'){result.code='NATIVE_MAIN_OWNER_BUSY';}else throw Error('owner lock unavailable: '+s.status);
  }finally{await ref.release();result.released=true;}
 }else if(kind==='claim'){
  try{await withNativeMainExclusiveClaim(context,held,{waitMs:0});result.released=true;}
  catch(e:any){result.code=e.code??'unknown';if(e.code!=='NATIVE_MAIN_CLAIM_BUSY')throw e;}
 }else throw Error('explicit lock kind required');
 save();
}catch(e:any){result.error={type:e.name,code:e.code??null,message:e.message};save();process.exitCode=1;}
