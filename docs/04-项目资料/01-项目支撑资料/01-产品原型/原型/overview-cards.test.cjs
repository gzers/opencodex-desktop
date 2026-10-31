/* 概览三卡与范围工具栏：只验证「有用、简洁、高效」这一轮的原型口径。 */
const test=require('node:test'),assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');
const sync=h=>h.w.syncPrototype.state();
async function setup(t){const h=await make('#overview');t.after(()=>{try{assert.deepEqual(h.errors,[]);assert.deepEqual(h.network,[]);}finally{h.dispose();}});return h;}
async function finished(h){
 for(let i=0;i<80&&!sync(h).result;i++)await h.wait(50);
 assert.equal(sync(h).running,false);assert.ok(sync(h).result);return sync(h).result;
}

test('文件同步卡片只显示最近导入 / 导出，首次为尚未执行',async t=>{
 const h=await setup(t),card=h.document.querySelector('#card-overview-migration');
 assert.match(card.textContent,/最近导入/);assert.match(card.textContent,/最近导出/);
 assert.equal(h.document.querySelector('#overviewImportActivity').textContent,'尚未导入');
 assert.equal(h.document.querySelector('#overviewExportActivity').textContent,'尚未导出');
 assert.doesNotMatch(card.textContent,/可选对象|独立选择|选择同步范围/);
 assert.equal(h.document.querySelectorAll('#card-overview-migration .mod-actions button').length,2);
});

test('导入与导出完成后卡片写时间与项数，不写固定规则说明',async t=>{
 const h=await setup(t);
 h.w.syncPrototype.setImportSample('keyless');h.w.syncPrototype.open({tab:'file',scope:['channel:newapi'],action:'import',stayOnPage:true});
 assert.equal(h.document.querySelector('#modalConfirm').textContent,'确认更新');assert.equal(h.document.querySelector('#modalConfirm').disabled,true);
 h.click('[data-sx=choose-import-file]');assert.equal(sync(h).running,false);assert.equal(sync(h).backups.length,0);assert.equal(h.document.querySelector('#overviewImportActivity').textContent,'尚未导入');
 assert.equal(h.document.querySelector('#sxImportBackup').checked,true);h.click('#modalConfirm');const result=await finished(h);
 assert.equal(h.document.querySelector('#overviewImportActivity').textContent.split(' · ')[1],result.items.length+' 项');
 h.w.syncPrototype.open({tab:'file',scope:['channel:newapi'],action:'export'});h.click('#modalConfirm');
 const exported=sync(h).lastExport.objects.length;
 assert.equal(h.document.querySelector('#overviewExportActivity').textContent.split(' · ')[1],exported+' 项');
 assert.match(h.document.querySelector('#overviewExportActivity').textContent,/^\d{2}:\d{2} · /);
});

test('WebDAV 卡片给连接状态、线上时间与更新状态色，旧方向别名统一显示有更新',async t=>{
 const h=await setup(t);
 for(const [state,text,cls] of [['pending-upload','有更新','is-ok'],['pending-download','有更新','is-ok'],['bidirectional','有更新','is-ok'],['local-newer','有更新','is-ok'],['remote-newer','有更新','is-ok'],['ready','有更新','is-ok'],['synced','已一致','is-idle'],['partial','未完成','is-warn']]){
  h.w.renderWebdav(state);
  assert.equal(h.document.querySelector('#syncDirectionText').textContent,text);
  assert.ok(h.document.querySelector('#syncDirection').classList.contains(cls),state);
  // 更新状态与线上版本同一行，不再单独占一行。
  assert.equal(h.document.querySelectorAll('#card-overview-sync .mod-status>span').length,2);
  assert.ok(h.document.querySelector('#syncRemoteTime').closest('.ov-state-side').contains(h.document.querySelector('#syncDirectionText')));
  assert.equal(h.document.querySelector('#syncActions').textContent.split('同步')[0],'');
  assert.notEqual(h.document.querySelector('#syncRemoteTime').textContent,'—');
  assert.match(h.document.querySelector('#card-overview-sync').textContent,/连接状态/);
  assert.doesNotMatch(h.document.querySelector('#card-overview-sync').textContent,/较新覆盖|双向|待上传|待下载/);
 }
});

test('范围工具栏只留搜索与表头全选，最后一次选择自动沿用',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav'});
 assert.equal(h.document.querySelector('[data-sx-preset]'),null);
 assert.equal(h.document.querySelector('[data-sx=clear]'),null);
 assert.ok(h.document.querySelector('#sxSearch'));assert.ok(h.document.querySelector('[data-sx-all]'));
 h.change('[data-sx-group="skills"]',false);const dav=sync(h).savedScopes.webdav;
 h.w.syncPrototype.open({tab:'file'});h.w.syncPrototype.open({tab:'webdav'});
 assert.equal(sync(h).savedScopes.webdav,dav);
 assert.equal(sync(h).selected.some(id=>id.startsWith('skill:')),false);
});
