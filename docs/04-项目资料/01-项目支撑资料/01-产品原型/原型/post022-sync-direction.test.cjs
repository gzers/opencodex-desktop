const test=require('node:test'),assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');
const scope=['skill:design','skill-platform:codex'];
const state=h=>JSON.parse(JSON.stringify(h.w.syncPrototype.state()));
const choose=(h,value)=>{const e=h.document.querySelector('[name=sxDirection][value="'+value+'"]');assert(e&&!e.disabled);e.checked=true;e.dispatchEvent(new h.w.Event('change',{bubbles:true}));};
const open=h=>h.w.syncPrototype.open({tab:'webdav',scope,action:'check'});
// Only selected mock config is affected. No API, file, credential or real sync.
test('手动方向在同一弹窗重算来源/变化，未知时间不补造，取消/重开默认自动',async t=>{
 const h=await make('#sync?tab=webdav');t.after(()=>h.dispose());h.w.syncPrototype.setScenario('unknown-time');open(h);
 const before=JSON.stringify(state(h).versions),snapshots=JSON.stringify(state(h).snapshots),backups=JSON.stringify(state(h).backups);
 assert.equal(h.document.querySelector('[name=sxDirection]:checked').value,'auto');assert(h.document.querySelector('#modalConfirm').disabled);
 choose(h,'upload');assert.equal(state(h).preview.source,'local');assert.equal(state(h).preview.directionMode,'upload');assert(!h.document.querySelector('#modalConfirm').disabled);assert.equal(h.document.querySelector('#modalConfirm').textContent,'确认上传');
 choose(h,'download');assert.equal(state(h).preview.source,'remote');assert(h.document.querySelector('#sxPreviewBackup').checked);assert.equal(h.document.querySelector('#modalConfirm').textContent,'确认下载');
 assert.equal(JSON.stringify(state(h).versions),before);assert.equal(JSON.stringify(state(h).snapshots),snapshots);h.click('#modalCancel');assert.equal(JSON.stringify(state(h).backups),backups);assert.equal(state(h).running,false);
 open(h);assert.equal(h.document.querySelector('[name=sxDirection]:checked').value,'auto');assert(h.document.querySelector('#modalConfirm').disabled);h.click('#modalClose');
 assert.deepEqual(h.network,[]);assert.deepEqual(h.errors,[]);
});
for(const [value,source,direction] of [['upload','local','upload'],['download','remote','download']])test(value+'：确认才执行指定方向，未选范围保持',async t=>{
 const h=await make('#sync?tab=webdav');t.after(()=>h.dispose());h.w.syncPrototype.setScenario('unknown-time');open(h);choose(h,value);
 const before=state(h),chosen=before.preview.rows.filter(x=>x.direction!=='skip');assert(chosen.length>0);assert(chosen.every(x=>x.direction===direction));
 h.click('#modalConfirm');await new Promise(resolve=>h.w.setTimeout(resolve,2000));const after=state(h);assert.equal(after.lastRun.source,source);assert.equal(after.running,false);
 for(const id of Object.keys(before.versions.local))if(!scope.includes(id)){assert.deepEqual(after.versions.local[id],before.versions.local[id]);assert.deepEqual(after.versions.remote[id],before.versions.remote[id]);}
 assert.equal(after.backups.length,before.backups.length+(value==='download'?1:0));assert.deepEqual(h.network,[]);assert.deepEqual(h.errors,[]);
});
test('手动方向不绕过连接失败/预览变化；内容相同也不产生写入',async t=>{
 const h=await make('#sync?tab=webdav');t.after(()=>h.dispose());h.w.syncPrototype.setScenario('unknown-time');open(h);choose(h,'upload');h.w.syncPrototype.setScenario('download');h.click('#modalConfirm');assert.equal(state(h).running,false);assert(h.document.querySelector('#modalConfirm').disabled);assert.match(h.document.querySelector('#sxError').textContent,/重新预览/);h.click('#modalClose');
 h.w.syncPrototype.setScenario('network-error');open(h);assert([...h.document.querySelectorAll('[name=sxDirection]')].every(e=>e.disabled));assert(h.document.querySelector('#modalConfirm').disabled);h.click('#modalClose');
 h.w.syncPrototype.setScenario('same');open(h);choose(h,'download');assert(h.document.querySelector('#modalConfirm').hidden);assert.equal(state(h).running,false);h.click('#modalClose');assert.deepEqual(h.network,[]);
});
