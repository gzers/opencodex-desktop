const test=require('node:test'),assert=require('node:assert/strict');
const {decideSavedContent}=require('./g0-contracts.js');
const {make}=require('./component-topic-harness.cjs');
const saved=(digest,time)=>({exists:true,scopeKey:'selected',contentDigest:digest,savedAt:time,timeKind:'saved',bound:true});
test('POST04 内容优先、可靠UTC保存时间判新，无基线不固定云端',()=>{
 const a=saved('a','2026-11-04T10:00:00+08:00'),b=saved('b','2026-10-30T02:00:00Z');
 assert.equal(decideSavedContent(a,b).source,'local');assert.equal(decideSavedContent(b,a).source,'remote');
 assert.equal(decideSavedContent(a,{...b,contentDigest:'a',savedAt:''}).status,'same');
 for(const bad of [{savedAt:''},{savedAt:'invalid'},{timeKind:'mtime'},{timeKind:'unknown'},{bound:false},{clockSkew:true},{scopeKey:'other'},{contentDigest:''}])assert.equal(decideSavedContent({...a,...bad},b).status,'unknown');
 assert.equal(decideSavedContent(a,{...b,savedAt:'2026-11-04T02:00:00Z'}).status,'unknown');
 assert.equal(decideSavedContent(a,{exists:false}).source,'local');assert.equal(decideSavedContent({exists:false},a).source,'remote');
 assert.equal(decideSavedContent({exists:false},{exists:false}).status,'same');assert.equal(decideSavedContent(a,{readError:true}).status,'unknown');
});
test('POST04 唯一总原型：未知/同刻/mtime/异常均禁用确认且取消无写入，已知时间对称预览',async t=>{
 const h=await make('#sync?tab=webdav');t.after(()=>h.dispose());const s=()=>h.w.syncPrototype.state();
 for(const scene of ['unknown-time','same-time','legacy-mtime','invalid-time','unbound-version','clock-skew']){
  h.w.syncPrototype.setScenario(scene);h.w.syncPrototype.open({tab:'webdav',scope:['skill:design'],action:'check'});
  assert.equal(h.document.querySelector('#modalTitle').textContent,'同步预览');assert.match(h.document.querySelector('#modalBody').textContent,/无法判定新版/);
  assert.equal(h.document.querySelector('#modalConfirm').disabled,true);assert.equal(h.document.querySelector('#modalConfirm').hidden,false);assert.equal(s().preview.source,'undetermined');
  const before=JSON.stringify({versions:s().versions,backups:s().backups,serial:s().serial});h.click('#modalConfirm');h.click('#modalClose');
  assert.equal(s().running,false);assert.equal(JSON.stringify({versions:s().versions,backups:s().backups,serial:s().serial}),before);
 }
 for(const [scene,source] of [['local-no-baseline','local'],['remote-no-baseline','remote']]){
  h.w.syncPrototype.setScenario(scene);h.w.syncPrototype.open({tab:'webdav',scope:['skill:design'],action:'check'});assert.equal(s().preview.source,source);assert.equal(h.document.querySelector('#modalConfirm').disabled,false);h.click('#modalCancel');
 }
 h.w.syncPrototype.setScenario('normal');h.w.syncPrototype.open({tab:'webdav',scope:['mcp:documentation'],action:'check'});assert.equal(s().preview.source,'local');h.click('#modalCancel');
 h.w.syncPrototype.open({tab:'webdav',scope:['mcp:documentation','skill:design'],action:'check'});assert.equal(s().preview.source,'remote');h.click('#modalCancel');
 h.w.syncPrototype.setScenario('upload');h.w.syncPrototype.open({tab:'webdav',scope:['skill:design'],action:'check'});
 h.w.syncPrototype.setScenario('download');h.click('#modalConfirm');assert.equal(s().running,false);assert.equal(h.document.querySelector('#modalConfirm').disabled,true);assert.match(h.document.querySelector('#sxError').textContent,/重新预览/);
 assert.deepEqual(h.network,[]);assert.deepEqual(h.errors,[]);assert.ok(h.writes.every(([key])=>['ocx-route','ocx-theme'].includes(key)));
});
