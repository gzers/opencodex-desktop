import {createHash} from 'node:crypto';import {readFileSync,writeFileSync} from 'node:fs';import {join} from 'node:path';import assert from 'node:assert/strict';
import {validate,controlledCache} from './cache-candidate.ts';import {validate as legacy} from './producer-r4.ts';
const root=process.env.EVIDENCE!,out=process.env.PROBE_ROOT!,prefix=join(root,'attempt-r4-01/ordinary'),read=(p:string)=>JSON.parse(readFileSync(p,'utf8')),sha=(b:Buffer)=>createHash('sha256').update(b).digest('hex'),clone=(x:any)=>JSON.parse(JSON.stringify(x)),CACHE='codex/models_cache.json',AUDIT='codex/opencodex-catalog-audit.jsonl',CAT='codex/opencodex-catalog.json';
const context=read(join(prefix,'result.json')),names=['before','ready','pre-stop','official-stop'],stages=names.map(s=>read(join(prefix,'capture-'+s+'.json')));
const originalContext=clone(context);originalContext.stageBytesSha256={};originalContext.stageCanonicalSha256={};for(let i=0;i<names.length;i++){const bytes=readFileSync(join(prefix,'capture-'+names[i]+'.json'));const l=context.captures.find((x:any)=>x.stage===names[i]);assert.equal(sha(bytes),l.sha256);originalContext.stageBytesSha256[names[i]]=sha(bytes);originalContext.stageCanonicalSha256[names[i]]=sha(Buffer.from(JSON.stringify(stages[i])))}
const frozenScope={fixture:context.fixture,sid:context.runtime.sid,declaredRoutedSlugs:['fixture/gpt-6.1-sol']};
function fixture(){return {before:clone(stages[0]),ready:clone(stages[1]),pre:clone(stages[2]),after:clone(stages[3]),context:clone(originalContext),scope:clone(frozenScope)}}
function row(x:any,s:string,p=CACHE){return x[s].find((r:any)=>r.Path===p)}
function rebind(x:any){for(const [s,n] of [['before','before'],['ready','ready'],['pre','pre-stop'],['after','official-stop']]){const shaRows=sha(Buffer.from(JSON.stringify(x[s])));x.context.stageCanonicalSha256[n]=shaRows;x.context.stageBytesSha256[n]=shaRows;x.context.captures.find((y:any)=>y.stage===n).sha256=shaRows}}
function content(r:any,value:any){const bytes=Buffer.from(typeof value==='string'?value:JSON.stringify(value));r.Image.Bytes=bytes.toString('base64');r.Image.Sha256=sha(bytes);r.Length=bytes.length}
const checks:any[]=[];
const actual=fixture();assert.equal(legacy(actual.before,actual.ready,actual.pre,actual.after,actual.context.runtime.pid).accepted,false);const result=validate(actual.before,actual.ready,actual.pre,actual.after,actual.context.runtime.pid,{context:actual.context,scope:actual.scope});assert.equal(result.accepted,true);assert.equal(result.cacheCandidate.branch,'controlled-originally-absent-startup-cache');assert.equal(result.rows.filter((r:any)=>r.reason==='controlled-originally-absent-startup-cache').length,1);checks.push({case:'real-r4-captures-proposed-cache-authorization',pass:true,originalServiceAccepted:false,candidatePerPathAuthorization:true});
const negative=(name:string,mutate:(x:any)=>void,resign=true)=>{const x=fixture();mutate(x);if(resign)rebind(x);const r=controlledCache(x);assert.equal(r.accepted,false,name+' unexpectedly accepted');checks.push({case:name,pass:true,rejection:r.reason})};
negative('external-cache-content-after-seal',x=>content(row(x,'after'),{...JSON.parse(Buffer.from(row(x,'after').Image.Bytes,'base64').toString()),client_version:'changed'}));
negative('replacement-file-id',x=>row(x,'after').Image.Id='1:2:3');
negative('wrong-owner',x=>row(x,'after').Image.Owner='S-1-5-21-foreign');
negative('late-acl-change',x=>row(x,'after').Image.Sddl+='(A;;FA;;;WD)');
negative('broad-acl-even-consistent',x=>{for(const s of ['ready','pre','after'])row(x,s).Image.Sddl+='(A;;FA;;;WD)'});
negative('unknown-field-even-in-all-sealed-stages',x=>{for(const s of ['ready','pre','after']){const v=JSON.parse(Buffer.from(row(x,s).Image.Bytes,'base64').toString());v.external=true;content(row(x,s),v)}});
negative('unknown-model-even-if-all-stages-and-catalog-match',x=>{for(const s of ['ready','pre','after']){const v=JSON.parse(Buffer.from(row(x,s).Image.Bytes,'base64').toString());v.models[0].slug='external/model';content(row(x,s),v)}const v=JSON.parse(Buffer.from(row(x,'ready',CAT).Image.Bytes,'base64').toString());v.models[0].slug='external/model';content(row(x,'ready',CAT),v)});
negative('late-audit-append',x=>content(row(x,'after',AUDIT),Buffer.from(row(x,'after',AUDIT).Image.Bytes,'base64').toString()+'{"pid":999,"writer":"external"}\n'));
negative('wrong-audit-writer-pid',x=>{for(const s of ['ready','pre','after']){const v=JSON.parse(Buffer.from(row(x,s,AUDIT).Image.Bytes,'base64').toString());v.pid++;content(row(x,s,AUDIT),JSON.stringify(v)+'\n')}});
negative('no-official-startup-cache-event',x=>{for(const s of ['ready','pre','after']){const v=JSON.parse(Buffer.from(row(x,s,AUDIT).Image.Bytes,'base64').toString());v.writer=v.command='external';content(row(x,s,AUDIT),JSON.stringify(v)+'\n')}});
negative('audit-before-transaction',x=>{for(const s of ['ready','pre','after']){const v=JSON.parse(Buffer.from(row(x,s,AUDIT).Image.Bytes,'base64').toString());v.at='2000-01-01T00:00:00Z';content(row(x,s,AUDIT),JSON.stringify(v)+'\n')}});
negative('original-cache-preexisted',x=>{x.before=x.before.map((r:any)=>r.Path===CACHE?{...clone(row(x,'ready')),Stage:'before'}:r)});
negative('wrong-home',x=>x.context.binding.value.home='C:\\OtherUser\\runtime');
negative('wrong-root-scope',x=>x.scope.fixture='C:\\OtherUser\\fixture');
negative('wrong-sid',x=>x.context.runtime.sid='S-1-5-21-foreign');
negative('wrong-filetime',x=>x.context.runtime.filetime='999');
negative('missing-witness',x=>x.context.binding=null);
negative('job-not-empty',x=>x.context.stop.jobActive=1);
negative('listener-not-gone',x=>x.context.stop.allOriginalListeners[0].gone=false);
negative('unexited-child',x=>x.context.stop.retainedMemberExits[0].exited=false);
negative('nonzero-original-exit',x=>x.context.stop.originalExitCode=91);
negative('unset-stop-budget',x=>x.context.stop.elapsedMs=null);
negative('unverified-package-source',x=>x.context.package.archiveSha256='0'.repeat(64));
negative('late-stop',x=>x.context.stop.elapsedMs=10001);
negative('missing-stage-ledger',x=>x.context.captures=x.context.captures.filter((r:any)=>r.stage!=='ready'),false);
negative('bad-stage-ledger-digest',x=>x.context.captures.find((r:any)=>r.stage==='ready').sha256='bad',false);
negative('unknown-capture-state',x=>row(x,'after').State='unknown');
negative('wrong-stage-name',x=>row(x,'after').Stage='later');
negative('duplicate-cache-path',x=>x.after.push(clone(row(x,'after'))));
negative('corrupt-base64-sha',x=>row(x,'after').Image.Sha256='0'.repeat(64));
negative('inconsistent-captured-length',x=>row(x,'after').Length++);
negative('cache-path-link-state',x=>{row(x,'after').State='unknown';row(x,'after').FailureType='linked-target'});
negative('missing-startup-declarations',x=>x.scope.declaredRoutedSlugs=[]);
// Existing native-cache route remains supported by the unchanged legacy branch.
const native=fixture();const cat=JSON.parse(Buffer.from(row(native,'after',CAT).Image.Bytes,'base64').toString());content(row(native,'after'),{models:cat.models,fetched_at:'2000-01-01T00:00:00Z',client_version:'0.0.0'});const nativeResult=validate(native.before,native.ready,native.pre,native.after,native.context.runtime.pid);assert(nativeResult.accepted);checks.push({case:'existing-native-cache-branch-remains',pass:true});
const noEvidence=fixture();assert.equal(validate(noEvidence.before,noEvidence.ready,noEvidence.pre,noEvidence.after,noEvidence.context.runtime.pid).accepted,false);checks.push({case:'no-bound-context-keeps-legacy-refusal',pass:true});
// New cache branch may not hide independent failures in other paths.
for(const [name,path,text] of [['config-external','codex/config.toml','model="external"\nnew_user_field="keep"'],['catalog-unknown',CAT,'{"models":[{"slug":"unknown-native"}]}'],['backup-external','grok/config.toml.bak-opencodex','external user backup']] as const){const x=fixture();content(row(x,'after',path),text);rebind(x);const r=validate(x.before,x.ready,x.pre,x.after,x.context.runtime.pid,{context:x.context,scope:x.scope});assert.equal(r.cacheCandidate.accepted,true);assert.equal(r.accepted,false);checks.push({case:name+'-not-hidden-by-cache-branch',pass:true})}
writeFileSync(join(out,'out/result.json'),JSON.stringify({result:'pass',checks,count:checks.length,actualR4OldPolicyAccepted:false,newCandidatePerPathAuthorization:true,originalWindowsAcceptedRemains:false,scope:'offline immutable synthetic r4 images; no file compensation, no native Windows rerun or product changes',limits:['Protected capture/witness context is supplied by trusted test host; JSON replay is not independent authority','No malicious same-SID writer proof or arbitrary daily-file provenance','Native handles, root/stage ledger integrity and final revalidation under official locks must be implemented/verified in Windows host','Actual five-path compensation and two-lock reacquisition still not run; service accepted stays false']},null,2));console.log(JSON.stringify({result:'pass',checks:checks.length,candidateAuthorization:true,originalAccepted:false}));
