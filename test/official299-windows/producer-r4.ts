// Narrow synthetic-fixture provenance policy. Observing a pathname/hash alone is insufficient.
import {readFileSync,writeFileSync} from 'node:fs';
export function validate(before:any[],ready:any[],pre:any[],after:any[],runtimePid:number){
 const get=(rows:any[],path:string)=>rows.find(r=>r.Path===path),bytes=(r:any)=>r?.State==='file'?Buffer.from(r.Image.Bytes,'base64'):null;
 const text=(r:any)=>bytes(r)?.toString('utf8'),json=(r:any)=>JSON.parse(text(r)??'null');
 const canonical=(x:any):string=>JSON.stringify(x&&typeof x==='object'&&!Array.isArray(x)?Object.fromEntries(Object.keys(x).sort().map(k=>[k,JSON.parse(canonical(x[k]))])):Array.isArray(x)?x.map(v=>JSON.parse(canonical(v))):x);
 const equal=(x:any,y:any)=>canonical(x)===canonical(y),rows:any[]=[];
 const paths=['codex/config.toml','codex/opencodex-catalog.json','codex/models_cache.json','codex/opencodex-catalog-audit.jsonl','grok/config.toml.bak-opencodex'];
 for(const path of paths){let accepted=false,reason='unknown';try{
  const b=get(before,path),d=get(ready,path),p=get(pre,path),a=get(after,path);
  if([b,d,p,a].some(x=>!x||!['file','absent'].includes(x.State)))throw Error('unknown-stage-proof');
  if(d.State!==p.State||(d.State==='file'&&(d.Image.Id!==p.Image.Id||d.Image.Sha256!==p.Image.Sha256)))throw Error('late-prestop-change');
  if(path==='grok/config.toml.bak-opencodex'){
   const expected=b.State==='file'?bytes(b):bytes(get(before,'grok/config.toml'));
   accepted=a.State==='file'&&!!expected&&bytes(a)!.equals(expected)&&d.State==='file'&&bytes(d)!.equals(expected);reason='exact-original-backup-or-original-Grok-copy';
  }else if(path.endsWith('audit.jsonl')){
   const original=text(b)??'',actual=text(a)??'';
   if(!actual.startsWith(original))throw Error('original-audit-prefix-changed');
   const lines=actual.slice(original.length).split('\n').filter(Boolean).map(s=>JSON.parse(s));
   const fields=['at','pid','ppid','command','opencodexHome','target','outcome','reason','intent','writer','configSource','routedBefore','routedAfter'];
   accepted=lines.every(v=>Object.keys(v).every(k=>fields.includes(k))&&v.pid===runtimePid&&['retained-sync','cache-invalidate','codex-restore','catalog-restore','startup-cache','cache-from-catalog','convergence','sync-cache'].includes(v.writer)&&v.command===v.writer&&Number.isFinite(Date.parse(v.at)));
   // Windows existing audit is not appended by the official adapter. Require no late suffix.
   accepted=accepted&&text(a)===text(p);reason='known-official-writer-PID-schema-and-no-late-audit-append';
  }else if(path==='codex/config.toml'){
   const bobj=Bun.TOML.parse(text(b)!),dobj=Bun.TOML.parse(text(d)!),aobj=Bun.TOML.parse(text(a)!);
   const flatten=(v:any,prefix=''):any=>Object.assign({},...Object.entries(v).map(([k,x])=>x&&typeof x==='object'&&!Array.isArray(x)?flatten(x,prefix+k+'.'):{[prefix+k]:x}));
   const bf=flatten(bobj),df=flatten(dobj),af=flatten(aobj);accepted=Object.entries(af).every(([k,v])=>(k in bf&&equal(v,bf[k]))||(k in df&&equal(v,df[k])));
   if(/fixture\/|127\.0\.0\.1:\d+|localhost:\d+/.test(JSON.stringify(aobj)))throw Error('post-stop-routing-residue');reason='TOML-values-from-original-or-controlled-ready-only-no-routing';
  }else if(path==='codex/opencodex-catalog.json'){
   const bobj=json(b),dobj=json(d),aobj=json(a);if(!Array.isArray(aobj?.models)||!Array.isArray(bobj?.models))throw Error('catalog-shape-unknown');
   accepted=aobj.models.every((v:any)=>!String(v.slug).includes('/')&&bobj.models.some((x:any)=>x.slug===v.slug)&&(bobj.models.some((x:any)=>equal(x,v))||dobj.models.some((x:any)=>equal(x,v))));
   accepted=accepted&&Object.keys(aobj).every(k=>k==='models'||equal(aobj[k],bobj[k])||equal(aobj[k],dobj[k]));reason='native-row-subset-of-fixed-original-or-ready-projection';
  }else{
   const aobj=json(a),cat=json(get(after,'codex/opencodex-catalog.json'));
   const models=Array.isArray(aobj)?aobj:aobj?.models;
   accepted=Array.isArray(models)&&models.every((v:any)=>!String(v.slug).includes('/')&&cat.models.some((x:any)=>equal(v,x)));
   if(!Array.isArray(aobj))accepted=accepted&&Object.keys(aobj).every(k=>['models','fetched_at','client_version'].includes(k));reason='native-cache-rows-equal-sealed-restored-catalog';
  }
 }catch(e:any){reason=e.message;accepted=false;}rows.push({path,accepted,reason});}
 return {accepted:rows.every(r=>r.accepted),rows,scope:'explicit synthetic transaction only; no proof for arbitrary daily files or malicious same-SID producer impersonation'};
}
if(import.meta.main){const [before,ready,pre,after,pid,out]=process.argv.slice(2);writeFileSync(out,JSON.stringify(validate(...[before,ready,pre,after].map(p=>JSON.parse(readFileSync(p,'utf8'))) as [any,any,any,any],Number(pid))));}
