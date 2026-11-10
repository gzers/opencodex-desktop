// Offline consistency verification. Does not rerun service or reconstruct omitted secrets.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {join} from 'node:path';
import {createHash} from 'node:crypto';
const [root]=process.argv.slice(2),read=p=>JSON.parse(readFileSync(join(root,p),'utf8'));
const c=read('attempt-04/controller.json'),r=read('attempt-04/ordinary/result.json');
assert.equal(r.accepted,false);assert.equal(c.accepted,false);
assert.equal(r.host.sid,c.account.sid);assert.equal(r.runtime.sid,c.account.sid);assert.equal(r.runtime.elevated,false);
assert.equal(r.runtime.pid,r.binding.value.pid);assert.equal(r.runtime.filetime,r.binding.value.filetime);
assert.equal(r.job.limitFlags&0x1800,0);
assert(r.start.pass&&r.start.elapsedMs<30000);assert(r.status.pass&&r.status.elapsedMs<5000);
const receipts=readFileSync(join(root,'attempt-04/ordinary/startup-receipts.jsonl'),'utf8').trim().split(/\r?\n/);
assert.equal(receipts.length,1);assert.equal(receipts[0],r.start.receipt);
const receipt=JSON.parse(receipts[0]);assert.equal(receipt.pid,r.runtime.pid);assert.equal(receipt.filetime,r.runtime.filetime);
const stdout=readFileSync(join(root,'attempt-04/ordinary/startup.stdout.log'),'utf8');
assert.equal(stdout.split(/\r?\n/).filter(s=>s==='Client startup work complete.').length,1);
for(const response of [receipt.health,receipt.ready,r.status.health,r.status.ready]){assert.equal(response.status,200);const b=JSON.parse(response.body);assert.equal(b.pid,r.runtime.pid);assert.equal(b.port,r.port);assert.equal(b.version,'2.99.0');}
assert(r.stop.normalRequested&&r.stop.elapsedMs<10000);assert.equal(r.stop.originalExitCode,0);assert(r.stop.originalExited);
assert.equal(r.stop.jobActive,0);assert.deepEqual(r.stop.jobMembers,[]);assert.deepEqual(r.stop.listenerOwners,[]);
assert.deepEqual(JSON.parse(r.stop.response.body),{success:true,message:'Proxy stopping, native Codex restored.',sharedTeardown:'performed'});
assert.equal(r.cleanup.filter(x=>x.kind==='separate-forced-owned-job').length,0);
const originals=read('attempt-04/ordinary/client-originals.json'),restored=read('attempt-04/ordinary/client-restored.json');
for(const o of originals){assert.equal(createHash('sha256').update(Buffer.from(o.bytesBase64,'base64')).digest('hex'),o.sha256);const a=restored.find(x=>x.relative===o.relative);assert(a?.restored);assert.equal(a.afterSha256,o.sha256);assert.equal(createHash('sha256').update(Buffer.from(a.bytesBase64,'base64')).digest('hex'),o.sha256);}
const before=read('attempt-04/ordinary/client-tree-before.json'),after=read('attempt-04/ordinary/client-tree-after.json');
const extras=after.filter(a=>!before.some(b=>b.relative===a.relative));assert.equal(extras.length,5);assert.equal(r.stop.clientTreeRestored,false);assert.equal(r.stop.pass,false);
assert(c.profileUnloaded&&c.profileDeleted&&c.accountDeleted);
const output={status:'consistent-recorded-failure',accepted:false,start:{pass:true,elapsedMs:r.start.elapsedMs,receiptCount:1},statusAction:{pass:true,elapsedMs:r.status.elapsedMs},stop:{pass:false,elapsedMs:r.stop.elapsedMs,normalApiReceipt:'performed',originalExitCode:0,jobEmpty:true,mainListenerGone:true,fourOriginalFilesRestored:true,completeClientTreeRestored:false,additionalClientFiles:extras},counterexamples:r.counterexamples,accountAndProfileDeleted:true,limits:['Offline verifier cannot recompute omitted health HMAC secrets; execution-time native verification and source are retained.','Grok injection was refused by the official runtime for this synthetic fixture; no Grok changed-and-restored claim.','No native model catalog was present; no catalog restoration claim.','Stop budget readback covers the primary listener. Additional Claude intercept listener is identified in stdout and separately checked after execution; late readback does not extend the action budget.','Prototype checks before source-digest recording retain raw outputs; they are not assigned invented historical source digests.']};
writeFileSync(join(root,'verified-summary.json'),JSON.stringify(output,null,2),{flag:'wx'});
console.log(JSON.stringify({status:output.status,accepted:false,extraFiles:extras.map(x=>x.relative)}));
