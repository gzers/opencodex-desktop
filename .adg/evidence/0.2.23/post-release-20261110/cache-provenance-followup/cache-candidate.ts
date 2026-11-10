/* Offline/test-only policy. Native witness/coordinator/exclusion must precede real compensation. */
import {createHash} from 'node:crypto';
import {validate as legacyValidate} from './producer-r4.ts';
const CACHE='codex/models_cache.json',CATALOG='codex/opencodex-catalog.json',AUDIT='codex/opencodex-catalog-audit.jsonl';
const sha=(b:Buffer)=>createHash('sha256').update(b).digest('hex');
const norm=(s:string)=>s.replace(/[\\/]+/g,'/').replace(/\/$/,'').toLowerCase();
const canonical=(x:any):string=>JSON.stringify(x&&typeof x==='object'&&!Array.isArray(x)?Object.fromEntries(Object.keys(x).sort().map(k=>[k,JSON.parse(canonical(x[k]))])):Array.isArray(x)?x.map(v=>JSON.parse(canonical(v))):x);
function requireFact(v:any,reason:string):asserts v {if(!v)throw Error(reason)}
function unique(rows:any[],path:string){const hits=rows.filter(r=>r.Path===path);requireFact(hits.length===1,'duplicate-or-missing-path');return hits[0]}
function raw(row:any){requireFact(row.State==='file'&&row.Error===0&&!row.FailureType,'unreadable-or-unknown');const i=row.Image;requireFact(i&&typeof i.Bytes==='string'&&/^[a-f0-9]{64}$/.test(i.Sha256),'malformed-image');const b=Buffer.from(i.Bytes,'base64');requireFact(b.toString('base64')===i.Bytes&&b.length===row.Length&&b.length<=2*1024*1024&&sha(b)===i.Sha256,'raw-byte-integrity');return b}
function privateAcl(sddl:string,sid:string){
 requireFact(typeof sddl==='string'&&sddl.startsWith('O:'+sid)&&sddl.includes('D:'),'unsafe-owner-or-acl');
 const acl=sddl.slice(sddl.indexOf('D:')+2);requireFact(!acl.includes('S:'),'unknown-sacl');const entries=[...acl.matchAll(/\(([^()]*)\)/g)].map(m=>m[1].split(';'));requireFact(entries.length>0,'missing-dacl');requireFact(entries.every(a=>a.length===6&&a[0]==='A'&&['SY',sid].includes(a[5]))&&entries.some(a=>a[5]===sid),'unexpected-dacl-principal');
}
export function controlledCache(input:{before:any[];ready:any[];pre:any[];after:any[];context:any;scope:{fixture:string;sid:string;declaredRoutedSlugs:readonly string[]}}){
 try{
  const {before,ready,pre,after,context:c,scope}=input;requireFact(c&&scope&&typeof scope.fixture==='string'&&typeof scope.sid==='string','context-missing');requireFact(c.privateRoot?.protected===true&&norm(c.fixture)===norm(scope.fixture)&&norm(c.privateRoot.path)===norm(scope.fixture),'unbound-fixture-root');privateAcl(c.privateRoot.sddl,scope.sid);
  requireFact(c.package?.version==='2.99.0'&&c.package.archiveSha256==='b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c','unverified-official-input');
  const b=c.binding?.value,r=c.runtime;requireFact(r&&b&&r.sid===scope.sid&&b.sid===scope.sid&&r.pid===b.pid&&r.filetime===b.filetime&&norm(b.home)===norm(scope.fixture+'/runtime')&&b.homeIdentity,'unbound-native-generation');requireFact(r.elevated===false&&c.start?.pass&&c.start.elapsedMs<30000&&c.start.receiptCount===1&&c.status?.pass&&c.status.elapsedMs<5000,'startup-or-status-unproven');
  const stop=c.stop;requireFact(stop?.normalRequested&&stop.originalExited&&stop.originalExitCode===0&&stop.jobActive===0&&Array.isArray(stop.jobMembers)&&stop.jobMembers.length===0&&Array.isArray(stop.retainedMemberExits)&&stop.retainedMemberExits.length>0&&stop.retainedMemberExits.every((x:any)=>x.exited)&&Array.isArray(stop.allOriginalListeners)&&stop.allOriginalListeners.length>0&&stop.allOriginalListeners.every((x:any)=>x.gone&&Array.isArray(x.remaining)&&x.remaining.length===0),'normal-exit-or-job-listener-proof-missing');requireFact(JSON.parse(stop.response.body).sharedTeardown==='performed'&&stop.response.status===200&&Number.isSafeInteger(stop.elapsedMs)&&stop.elapsedMs>=0&&stop.elapsedMs<10000,'stop-receipt-or-budget-missing');
  const receipt=JSON.parse(c.start.receipt);requireFact(receipt.pid===r.pid&&receipt.filetime===r.filetime&&receipt.sid===r.sid&&norm(receipt.home)===norm(b.home)&&receipt.homeIdentity===b.homeIdentity,'startup-receipt-generation-mismatch');
  for(const response of [receipt.health,receipt.ready,c.status.health,c.status.ready]){requireFact(response?.status===200,'endpoint-not-ready');const v=JSON.parse(response.body);requireFact(v.pid===r.pid&&v.port===c.port&&v.version==='2.99.0'&&v.service==='opencodex','endpoint-generation-mismatch')}
  // These are caller-observed protected stage bytes, not a self-authenticating JSON authorization.
  const stages=[['before',before],['ready',ready],['pre-stop',pre],['official-stop',after]] as const;
  for(const [stage,rows] of stages){const ledgers=c.captures.filter((x:any)=>x.stage===stage);requireFact(ledgers.length===1,'stage-ledger-missing');const l=ledgers[0];requireFact(l.sha256===input.context.stageBytesSha256[stage]&&sha(Buffer.from(JSON.stringify(rows)))===input.context.stageCanonicalSha256[stage],'protected-stage-integrity');privateAcl(l.sddl,scope.sid)}
  const orig=unique(before,CACHE);requireFact(orig.State==='absent'&&orig.Image===null,'original-cache-not-absent');
  const d=unique(ready,CACHE),p=unique(pre,CACHE),a=unique(after,CACHE);requireFact(d.Stage==='ready'&&p.Stage==='pre-stop'&&a.Stage==='official-stop','wrong-capture-stage');
  const db=raw(d),pb=raw(p),ab=raw(a);for(const x of [d,p,a]){requireFact(x.Image.Name==='models_cache.json'&&x.Image.Owner===scope.sid&&/^[0-9]+:[0-9]+:[0-9]+$/.test(x.Image.Id),'cache-owner-or-identity');privateAcl(x.Image.Sddl,scope.sid)}
  requireFact(d.Image.Id===p.Image.Id&&p.Image.Id===a.Image.Id&&d.Image.Owner===p.Image.Owner&&p.Image.Owner===a.Image.Owner&&d.Image.Sddl===p.Image.Sddl&&p.Image.Sddl===a.Image.Sddl&&db.equals(pb)&&pb.equals(ab),'cache-changed-since-controlled-ready');
  const cache=JSON.parse(ab.toString('utf8'));requireFact(cache&&typeof cache==='object'&&!Array.isArray(cache)&&Object.keys(cache).sort().join(',')==='client_version,fetched_at,models'&&cache.fetched_at==='2000-01-01T00:00:00Z'&&cache.client_version==='0.0.0'&&Array.isArray(cache.models)&&cache.models.length>0,'unknown-cache-wrapper');
  const cat=JSON.parse(raw(unique(ready,CATALOG)).toString('utf8'));requireFact(Array.isArray(cat.models)&&canonical(cache.models)===canonical(cat.models),'cache-not-controlled-ready-catalog');
  requireFact(ab.equals(Buffer.from(JSON.stringify({fetched_at:'2000-01-01T00:00:00Z',client_version:'0.0.0',models:cat.models},null,2)+'\n')),'not-official-cache-wrapper-bytes');
  const slugs=cache.models.map((m:any)=>m?.slug);requireFact(slugs.every((s:any)=>typeof s==='string'&&s.length>0)&&new Set(slugs).size===slugs.length,'ambiguous-cache-model');
  const declarations=scope.declaredRoutedSlugs;requireFact(Array.isArray(declarations)&&declarations.length>0&&new Set(declarations).size===declarations.length,'missing-startup-declarations');
  requireFact(slugs.some((s:string)=>s.includes('/'))&&slugs.every((s:string)=>declarations.includes(s)),'undeclared-cache-route');
  const oldAudit=unique(before,AUDIT),oldBytes=oldAudit.State==='absent'?Buffer.alloc(0):raw(oldAudit),audits=[unique(ready,AUDIT),unique(pre,AUDIT),unique(after,AUDIT)].map(raw);requireFact(audits[0].equals(audits[1])&&audits[1].equals(audits[2])&&audits[2].subarray(0,oldBytes.length).equals(oldBytes),'audit-late-change-or-lost-prefix');
  const text=audits[0].subarray(oldBytes.length).toString('utf8');requireFact(text.endsWith('\n'),'incomplete-audit-record');const lines=text.trim().split('\n').map(s=>JSON.parse(s));
  const t0=Date.parse(c.captures.find((l:any)=>l.stage==='before').utc),t1=Date.parse(c.captures.find((l:any)=>l.stage==='ready').utc);requireFact(Number.isFinite(t0)&&Number.isFinite(t1)&&t0<=t1,'invalid-stage-time');
  requireFact(lines.some(v=>v.pid===r.pid&&v.target==='cache'&&v.writer==='startup-cache'&&v.command==='startup-cache'&&v.intent==='cache'&&v.configSource==='file'&&v.outcome==='written'&&Date.parse(v.at)>=t0&&Date.parse(v.at)<=t1),'no-matching-official-startup-cache-event');
  return {accepted:true,branch:'controlled-originally-absent-startup-cache',action:'compensate-to-original-absence',scope:'bound synthetic transaction only; native exclusion and final full restoration still required'};
 }catch(e:any){return {accepted:false,reason:e.message,scope:'no-cache-compensation-authority'}}
}
export function validate(before:any[],ready:any[],pre:any[],after:any[],runtimePid:number,evidence?:any){
 const original=legacyValidate(before,ready,pre,after,runtimePid);const old=original.rows.find((x:any)=>x.path===CACHE);if(old?.accepted||!evidence)return original;
 const candidate=controlledCache({before,ready,pre,after,...evidence});const rows=original.rows.map((x:any)=>x.path===CACHE&&candidate.accepted?{...x,accepted:true,reason:candidate.branch,action:candidate.action}:x);
 return {...original,accepted:rows.every((x:any)=>x.accepted),rows,cacheCandidate:candidate,scope:'test-only per-path authorization; no full-service acceptance or new artifact exception'};
}
