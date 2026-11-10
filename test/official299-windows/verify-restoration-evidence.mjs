// Offline source/receipt checks only. This verifier never launches a service.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
const [root,code]=process.argv.slice(2),read=p=>JSON.parse(readFileSync(join(root,p),'utf8')),hash=b=>createHash('sha256').update(b).digest('hex');
const checks=[],sources=[];
for(const attempt of ['attempt-r2-01','attempt-r2-02']){
 const loc=read(attempt+'-location.json'),c=read(attempt+'/controller.json');
 for(const [file,key] of [['Native.cs','native'],['host-r2.ps1','host'],['Recovery.cs','recovery'],['lock-worker.ts','lockWorker'],['run.ps1','provisioner']]){const bytes=execFileSync('git',['-C',code,'show',loc.sourceCommit+':test/official299-windows/'+file]);assert.equal(hash(bytes),c.hostHashes[key],attempt+'/'+file);sources.push({attempt,commit:loc.sourceCommit,path:file,sha256:hash(bytes)});}
 assert(c.profileUnloaded&&c.profileDeleted&&c.accountDeleted);
 const exit=read(attempt+'-parent-exit.json');assert.equal(exit.exitCode,1);assert(exit.originalHandleSignalled);
}
const c=read('attempt-r2-02/controller.json'),r=read('attempt-r2-02/ordinary/result.json'),prefix='attempt-r2-02/ordinary/';
assert.equal(r.accepted,false);assert.equal(r.restorationChainPassed,false);assert.equal(r.stableLockExceptionAdopted,false);
assert.equal(r.runtime.sid,c.account.sid);assert.equal(r.host.sid,c.account.sid);assert.equal(r.runtime.elevated,false);
assert.equal(r.runtime.pid,r.binding.value.pid);assert.equal(r.runtime.filetime,r.binding.value.filetime);assert.equal(r.job.limitFlags&0x1800,0);
assert(r.start.pass&&r.start.elapsedMs<30000);assert(r.status.pass&&r.status.elapsedMs<5000);assert(r.stop.elapsedMs<10000);
const receipts=readFileSync(join(root,prefix+'startup-receipts.jsonl'),'utf8').trim().split(/\r?\n/);assert.equal(receipts.length,1);assert.equal(receipts[0],r.start.receipt);
const receipt=JSON.parse(receipts[0]);for(const response of [receipt.health,receipt.ready,r.status.health,r.status.ready]){assert.equal(response.status,200);const body=JSON.parse(response.body);assert.equal(body.pid,r.runtime.pid);assert.equal(body.port,r.port);assert.equal(body.version,'2.99.0');}
assert.equal(readFileSync(join(root,prefix+'startup.stdout.log'),'utf8').split(/\r?\n/).filter(s=>s==='Client startup work complete.').length,1);
assert(r.stop.normalRequested);assert.equal(JSON.parse(r.stop.response.body).sharedTeardown,'performed');assert.equal(r.stop.originalExitCode,0);assert(r.stop.originalExited);assert.equal(r.stop.jobActive,0);assert.deepEqual(r.stop.jobMembers,[]);
assert(r.stop.retainedMemberExits.length>0);assert(r.stop.retainedMemberExits.every(x=>x.exited));assert(r.stop.allListenersGone);assert.equal(r.stop.allOriginalListeners.length,3);assert(r.stop.allOriginalListeners.every(x=>x.gone&&x.remaining.length===0&&x.original.sid===r.runtime.sid));
assert(r.cleanup.every(x=>x.kind!=='separate-forced-owned-job'&&x.kind!=='separate-owned-lock-worker-force'));
const before=r.agentBackup.before,after=read(prefix+'agent-after-compensation.json'),fingerprint=x=>x.Files.map(f=>[f.Name,f.Sha256,f.Sddl,f.Owner]).sort((a,b)=>a[0].localeCompare(b[0]));assert.deepEqual(fingerprint(before),fingerprint(after));assert(r.stop.agentRestored);
const plan=read(prefix+'agent-operation-plan.json');assert.equal(plan.Changes.length,3);assert.equal(plan.TransactionId,r.stop.agentCompensation.transactionId);assert(plan.Changes.some(x=>x.Before===null));assert(plan.Changes.some(x=>x.After===null));assert(plan.Changes.some(x=>x.Before&&x.After));
for(const file of after.Files){assert.equal(hash(Buffer.from(file.Bytes,'base64')),file.Sha256);}
assert(r.injection.grokChanged&&r.injection.catalogChanged&&r.injection.catalogRoutedModelPresent&&r.injection.codexConfigChanged);
assert.equal(r.stop.restored.find(x=>x.relative==='grok/config.toml').restored,true);assert.equal(r.stop.restored.find(x=>x.relative==='codex/config.toml').restored,false);assert.equal(r.stop.catalogRestored,false);
assert.equal(hash(Buffer.from(r.catalogSource.originalBytes,'base64')),r.catalogSource.originalSha256);
assert.equal(r.stop.stableLockReacquisition.length,2);for(const item of r.stop.stableLockReacquisition){assert(item.result.acquired&&item.result.released);assert.equal(item.workerIdentity.sid,r.runtime.sid);assert.equal(item.originalExitCode,0);assert.equal(item.jobActive,0);assert.equal(item.fileIdentityBefore,item.fileIdentityAfter);assert.equal(item.sddlBefore,item.sddlAfter);assert(item.elapsedWithinStopMs<10000);}
const locks=read('prototype-lock-01/lock-prototype.json');assert(locks.pass);assert.equal(locks.checks.length,2);assert(locks.checks.every(x=>!x.blocked.result.acquired&&x.reacquired.result.acquired&&x.fileIdentityBefore===x.fileIdentityAfter&&x.sddlBefore===x.sddlAfter));
const agents=read('prototype-agent-02/recovery-prototype.json');assert(agents.pass);assert.equal(agents.checks.length,9);assert(agents.checks.every(x=>x.pass));assert(agents.checks.find(x=>x.case==='concurrent-write').separateProcessRace.result.refused);
const delta=read('actual-client-tree-delta.json');assert.equal(delta.changedFiles.length,2);assert.equal(delta.extraFiles.length,5);assert.equal(r.stop.clientTreeRestored,false);
const result={status:'consistent-recorded-failure',accepted:false,restorationChainPassed:false,docsBaseline:read('hypothesis.json').docsBaseline,actualSourceCommit:read('attempt-r2-02-location.json').sourceCommit,budgets:{startMs:r.start.elapsedMs,statusMs:r.status.elapsedMs,stopMs:r.stop.elapsedMs},passed:{startupReceiptAndIdentity:true,readyAndHealth:true,agentOriginalBytesAndAcl:true,grokChangedThenOriginalBytes:true,officialLocksReleasedAndReacquired:true,allThreeActualListenersGone:true,originalExitCode:0,jobEmpty:true},failed:{codexConfigOriginalBytes:true,catalogOriginalBytes:true,completeOriginalTree:true},clientDelta:delta,scope:{smallAgentCases:9,actualOfficialWindowsLockKinds:2,fullSuite:false,productionCodeChanged:false,release:false},sources,limits:['Execution-time HMAC verification cannot be recomputed offline without omitted secret.','Synchronous native compensation rollback and separate-process race cases passed; abrupt-crash/durable-restart recovery was not validated.','Changed config/catalog after raw bytes were omitted in actual attempt and cannot be reconstructed after Profile cleanup. Future capture-only code was syntax checked, not rerun as a service.','Stable-lock exception was not adopted. Even adopting only that exception would not resolve the other changed/new files.']};
writeFileSync(join(root,'verified-summary.json'),JSON.stringify(result,null,2),{flag:'wx'});console.log(JSON.stringify({status:result.status,budgets:result.budgets,sourceChecks:sources.length,agentCases:9,listeners:3,changedFiles:2,extraFiles:5}));
