/* 概览三卡与范围工具栏：只验证「有用、简洁、高效」这一轮的原型口径。 */
const test=require('node:test'),assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');
const sync=h=>h.w.syncPrototype.state();
async function setup(t){const h=await make('#overview');t.after(()=>h.dispose());return h;}
async function finished(h){
 for(let i=0;i<80&&!sync(h).result;i++)await h.wait(50);
 assert.equal(sync(h).running,false);return sync(h).result;
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
 h.click('[data-sx=choose-import-file]');h.click('#modalConfirm');const result=await finished(h);
 assert.equal(h.document.querySelector('#overviewImportActivity').textContent.split(' · ')[1],result.items.length+' 项');
 h.w.syncPrototype.open({tab:'file',scope:['channel:newapi'],action:'export'});h.click('#modalConfirm');
 const exported=sync(h).lastExport.objects.length;
 assert.equal(h.document.querySelector('#overviewExportActivity').textContent.split(' · ')[1],exported+' 项');
 assert.match(h.document.querySelector('#overviewExportActivity').textContent,/^\d{2}:\d{2} · /);
});

test('WebDAV 卡片给连接状态、线上时间与方向色，主按钮统一为同步',async t=>{
 const h=await setup(t);
 for(const [state,text,cls] of [['pending-upload','待上传','is-ok'],['pending-download','待下载','is-info'],['bidirectional','双向更新','is-info'],['synced','已一致','is-idle'],['partial','未完成','is-warn']]){
  h.w.renderWebdav(state);
  assert.equal(h.document.querySelector('#syncDirectionText').textContent,text);
  assert.ok(h.document.querySelector('#syncDirection').classList.contains(cls),state);
  // 方向与线上版本同一行，不再单独占一行。
  assert.equal(h.document.querySelectorAll('#card-overview-sync .mod-status>span').length,2);
  assert.ok(h.document.querySelector('#syncRemoteTime').closest('.ov-state-side').contains(h.document.querySelector('#syncDirectionText')));
  assert.equal(h.document.querySelector('#syncActions').textContent.split('同步')[0],'');
  assert.notEqual(h.document.querySelector('#syncRemoteTime').textContent,'—');
  assert.match(h.document.querySelector('#card-overview-sync').textContent,/连接状态/);
  assert.doesNotMatch(h.document.querySelector('#card-overview-sync').textContent,/较新覆盖/);
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
