import assert from 'node:assert/strict'
import {readFile,writeFile} from 'node:fs/promises'
import {join,win32} from 'node:path'
import {verify} from 'node:crypto'
const root='D:/Users/Administrator/WorkSpace/Test/post021-candidate2-targeted-20261105'
assert(process.argv[2], 'An explicit isolated review output path is required')
const folder=join(root,'evidence-attempt2'), read=async name=>JSON.parse(await readFile(join(folder,name+'.json'),'utf8'))
const plan=JSON.parse(await readFile(join(root,'plan-attempt2.json'),'utf8'))
const result=await read('result');assert.equal(result.acceptance,true);assert.equal(result.result,'passed-candidate2-native-managed-fresh-rename-cancel-retry')
assert.equal(result.account.owner.toLowerCase(),'ezio-mini\\administrator');assert.equal(result.account.sessionId,1)
assert.equal(result.manager.appsTree,'ca89905344d0d29e761bcb8ffa24d22769c9cb4c')
assert.equal(result.manager.sha256,'f62c474c06cb3fe2f77fe23d38e99c0783d86e727dbd538cfe7b4d36dd8c347d')
const installed=await read('managed-install-readback'),preserved=await read('failed-install-preservation')
assert.equal(installed.install.source,'offline');assert.equal(installed.source.kind,'managed');assert.equal(installed.source.version,'2.97.0-post021.ownerquery.2');assert.equal(installed.entryVersion,'opencodex 2.97.0-post021.ownerquery.2');assert.equal(installed.installedFilesVerified,2522)
assert.equal(installed.install.tarballSha256,'bba03ba0cf02c7bcd1895f63c670cb4a2f4b5272657145e45701b20d5e0f1957')
assert.equal(preserved.source.path,installed.source.path);assert.equal(preserved.source.version,installed.source.version);assert.equal(preserved.source.kind,'managed');assert.equal(preserved.allPackageFilesUnchanged,true);assert.equal(preserved.entryUnchanged,true);assert(preserved.error.includes('not_gzip'))
const input=await read('fresh-rename-trusted-input');assert.equal(input.value,plan.toProvider);assert(input.events.length&&input.events.every(e=>e.trusted))
const saved=await read('fresh-rename-saved'),fresh=await read('fresh-first-confirm-readback'),stable=await read('config-stability-through-first-confirm')
assert.equal(saved.firstConfirmationCount,0);assert.equal(fresh.firstConfirmationCount,1)
assert.equal(fresh.configSha256Before,saved.configSha256);assert.equal(fresh.configSha256After,saved.configSha256)
assert.deepEqual(saved.knownModelsBefore,saved.knownModelsSaved);assert.deepEqual(saved.knownModelsSaved,fresh.knownModels);assert.equal(fresh.knownModels.updatedAt,'2026-11-05T00:00:00.000Z')
assert.deepEqual(saved.recentArrivalsBefore,saved.recentArrivalsSaved);assert.deepEqual(saved.recentArrivalsSaved,fresh.recentArrivals)
assert(stable.observations.length>=2&&stable.observations.every(row=>row.matchesSaved&&row.sha256===saved.configSha256))
assert(fresh.newReference&&fresh.oldReferenceAbsent&&fresh.pendingCleared)
const cancel=await read('cancel-readback');assert(cancel.configUnchanged&&cancel.pendingUnchanged&&cancel.noClientStopped);assert.equal(cancel.firstConfirmationCount,0)
const failure=await read('restart-failure');assert(/服务|运行|连接/.test(failure.error));assert(!failure.error.includes('manage backups'))
const final=await read('final-readback');assert.equal(final.contextWindow,65591);assert(final.newReference&&final.oldReferenceAbsent&&final.pendingCleared);assert.equal(final.source.kind,'managed');assert.equal(final.source.version,installed.source.version)
for(const name of ['05-fresh-first-confirm','07-service-retry'])assert.equal((await read(name+'-public')).state.state,'ui-confirmed')
const shell=snapshot=>snapshot.own.filter(row=>win32.basename(row.path??'').toLowerCase()==='chatgpt.exe')
const same=(a,b)=>a.pid===b.pid&&a.createdAtUtcTicks===b.createdAtUtcTicks
const before=shell(await read('process-after-cancel-before-any-confirm')),after=shell(await read('05-fresh-first-confirm-process-after')),afterFailure=shell(await read('process-after-service-failure')),afterRetry=shell(await read('07-service-retry-process-after'))
assert(before.length&&after.length&&afterRetry.length);assert(before.every(p=>!after.some(q=>same(p,q))));assert(after.every(p=>afterFailure.some(q=>same(p,q))));assert(afterFailure.every(p=>!afterRetry.some(q=>same(p,q))))
const beforeConfigs=await read('protected-configs-before'),proxy=await read('protected-daily-proxy-before');let challenges=[]
for(const evidence of [beforeConfigs,...await Promise.all(['cancel-boundary','05-fresh-first-confirm-boundary','failure-boundary','07-service-retry-boundary'].map(read))]){
  const envelope=evidence.envelope??evidence.protectedConfigs.envelope
  assert(verify(null,Buffer.from(envelope.payload),plan.dailyGuard.publicKey,Buffer.from(envelope.signature,'base64')))
  const payload=JSON.parse(envelope.payload);assert.equal(payload.runId,plan.dailyGuard.runId);assert.equal(payload.sid,'S-1-5-21-587408064-856152790-2583328096-1001');assert.equal(payload.sessionId,2);assert.deepEqual(payload.configs,plan.protectedConfigBaseline);assert(!challenges.includes(payload.nonce));challenges.push(payload.nonce)
  if(evidence.otherOwnerChanges){assert.deepEqual(evidence.otherOwnerChanges,[]);assert.deepEqual(evidence.dailyProxy,proxy)}
}
const report={result:'passed-independent-evidence-review',atUtc:new Date().toISOString(),runtimeVersion:installed.source.version,tarballSha256:installed.install.tarballSha256,managerSha256:result.manager.sha256,managedEntry:installed.source.path,checks:['native-managed-full-version-2522files','failed-invalidarchive-retains-source-entry-allfiles','fresh-trusted-rename-firstactualconfirm','exactconfigbytes-and-knownModels-recentArrivals-baselines','pendingcleared-newref-oldabsent','cancel-noexit-config-pendingretained','serviceunavailable-noclientexit-publicretry-realnewgeneration','five-independent-signed-daily-checkpoints-otherowner-protected'],generations:{before,afterFresh:after,afterFailure,afterRetry},acceptanceScope:'Candidate2 targeted Windows contribution only; no production package/source adoption or release acceptance'}
await writeFile(process.argv[2],JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report))
