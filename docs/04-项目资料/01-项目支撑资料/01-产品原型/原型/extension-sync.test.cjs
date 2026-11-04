const test=require('node:test'),assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');
const plain=v=>JSON.parse(JSON.stringify(v));
async function setup(t){const h=await make('#sync?tab=webdav');t.after(()=>{assert.deepEqual(h.errors,[]);assert.equal(h.network.length,0);h.dispose()});return h;}
const sync=h=>plain(h.w.syncPrototype.state()),ext=h=>plain(h.w.extensionPrototype.state());
async function finish(h){for(let i=0;i<50&&sync(h).running;i++)await h.wait(50);assert.equal(sync(h).running,false);return sync(h).result;}
function transfer(h,scope,scenario='download'){h.w.syncPrototype.open({tab:'webdav',scope});h.w.syncPrototype.setScenario(scenario);h.click('[data-sx=check]');h.click('#modalConfirm');}
test('同步范围直接来自拓展库，默认选择Agent平台，应用自带MCP仍默认不选',async t=>{
 const h=await setup(t),items=plain(h.w.extensionPrototype.syncObjects());assert.equal(items.filter(x=>x.group==='skills').length,7);assert.equal(items.filter(x=>x.group==='mcp').length,5);
 assert.ok(sync(h).selected.includes('skill:design'));assert.ok(sync(h).selected.includes('mcp:documentation'));assert.ok(!sync(h).selected.includes('mcp:node_repl'));assert.ok(sync(h).selected.includes('skill-platform:claude'));
 h.w.setRoute('extensions');h.w.extensionPrototype.refresh();h.click('#xp3LeftRows [data-ic-row=development]');h.click('#xp3RightRows [data-ic-action=toggle][data-id=codex]');assert.equal(h.w.extensionPrototype.syncObject('skill:development').enabled,false);
 h.w.syncPrototype.open({tab:'file',scope:['skill:development'],action:'export'});h.click('#modalConfirm');assert.equal(sync(h).lastExport.snapshot.extensions.objects[0].payload.enabled,false);
});
test('导出冻结内容且共享源只打包一次，机器信息与绝对路径不进入包',async t=>{
 const h=await setup(t),scope=['skill:design','mcp:filesystem','skill-platform:claude'];
 h.w.syncPrototype.open({tab:'file',scope,action:'export'});
 const r=plain(h.w.extensionPrototype.syncObject('skill:design'));r.enabled=false;h.w.extensionPrototype.applySyncObject('skill:design',r);
 h.click('#modalConfirm');const file=sync(h).lastExport,payload=file.snapshot.extensions;
 assert.equal(payload.objects.length,3);assert.equal(payload.objects.filter(x=>x.id==='skill:design').length,1);assert.equal(payload.objects.find(x=>x.id==='skill:design').payload.enabled,true,'导出预览冻结');
 assert.doesNotMatch(JSON.stringify(payload),/installations|mapped|checking|directoryOS|localBindings|\/example\/|\/usr\/|USERPROFILE|Applications|lastOpenedPath/);
 assert.deepEqual(payload.objects.find(x=>x.id==='mcp:filesystem').payload.definition.args,[{binding:'entry'},{binding:'workspace'}]);
});
test('平台本机检测变化不改变可同步内容版本，Skill变化会使旧预览失效',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',scope:['skill:design']});h.w.syncPrototype.setScenario('download');h.click('[data-sx=check]');const before=sync(h).versions.local['skill:design'];h.w.closeModal();
 h.w.setRoute('extensions');h.change('#xp3DirectoryOS','windows');h.w.syncPrototype.open({tab:'webdav',scope:['skill:design']});h.click('[data-sx=check]');assert.deepEqual(sync(h).versions.local['skill:design'],before);
 const payload=plain(h.w.extensionPrototype.syncObject('skill:design'));payload.enabled=!payload.enabled;h.w.extensionPrototype.applySyncObject('skill:design',payload);h.click('#modalConfirm');assert.match(h.document.querySelector('#sxError').textContent,/已变化/);assert.equal(sync(h).running,false);
});
test('下载写回同一拓展库但不复制安装与映射，待接入不是同步失败',async t=>{
 const h=await setup(t);h.w.setRoute('extensions');h.change('#xp3DirectoryOS','windows');
 const before=ext(h);transfer(h,['skill:design','mcp:filesystem','skill-platform:claude']);const result=await finish(h);
 assert.equal(result.failed.length,0);assert.equal(result.downloaded,3);assert.equal(ext(h).resources.find(r=>r.id==='design').shared,false);assert.match(ext(h).resources.find(r=>r.id==='design').description,/同步内容示例/);
 const after=ext(h);for(const [id,value] of Object.entries(before.installations.windows))assert.deepEqual(after.installations.windows[id],value);for(const [id,value] of Object.entries(before.mappings.windows))assert.deepEqual(after.mappings.windows[id],value);
 assert.ok(!after.mappings.windows.claude.mapped);const pending=plain(h.w.extensionPrototype.syncReadiness(result.extensionIds));assert.ok(pending.some(x=>x.reason==='待映射'));assert.ok(!pending.some(x=>x.id==='gajae'),'未选平台不进入本次MCP关系');assert.ok(pending.some(x=>x.reason==='路径待配置'));
 assert.ok(h.document.querySelector('.sx-extension-result'));h.click('[data-sx=result-details]');assert.match(h.document.querySelector('#modalBody').textContent,/本机接入待处理/);h.click('[data-sx=extensions]');await h.settle();assert.equal(h.document.documentElement.dataset.route,'extensions');assert.ok(h.document.querySelector('.xp3-sync-pending'));
 const backup=sync(h).backups[0].items.find(x=>x.id==='skill:design');assert.equal(backup.extensionPayload.enabled,true);
});
test('本机路径处理可完成；内容不重复传输，已有本机路径保留',async t=>{
 const h=await setup(t);h.w.setRoute('extensions');h.change('#xp3DirectoryOS','windows');transfer(h,['mcp:filesystem']);await finish(h);h.w.setRoute('extensions');h.w.extensionPrototype.openLocalBindings('filesystem');
 h.input('#xp3Bind-entry','relative');h.input('#xp3Bind-workspace','C:\\Work');h.click('#modalConfirm');assert.match(h.document.querySelector('#xp3BindingError').textContent,/绝对路径/);
 h.input('#xp3Bind-entry','C:\\Tools\\filesystem.js');h.click('#modalConfirm');assert.equal(h.w.extensionPrototype.syncReadiness(['mcp:filesystem']).some(x=>x.reason==='路径待配置'),false);
 transfer(h,['mcp:filesystem']);await finish(h);assert.equal(h.w.extensionPrototype.syncReadiness(['mcp:filesystem']).some(x=>x.reason==='路径待配置'),false);
});
test('部分失败和失败重试保持拓展对象与一次备份，不覆盖未选对象',async t=>{
 const h=await setup(t),other=plain(h.w.extensionPrototype.syncObject('skill:development'));
 transfer(h,['skill:design','mcp:documentation'],'partial');const first=await finish(h);assert.equal(first.failed.length,1);assert.equal(first.downloaded,1);const saved=plain(h.w.extensionPrototype.syncObject('skill:design')),backup=JSON.stringify(sync(h).backups);
 h.click('[data-sx=retry]');const next=await finish(h);assert.equal(next.failed.length,0);assert.equal(JSON.stringify(sync(h).backups),backup);assert.deepEqual(plain(h.w.extensionPrototype.syncObject('skill:design')),saved);assert.deepEqual(plain(h.w.extensionPrototype.syncObject('skill:development')),other);
});
test('上传更新远端可移植内容，文件导入也写回拓展库；旧包保持原内容',async t=>{
 const h=await setup(t);transfer(h,['skill:design'],'upload');await finish(h);assert.deepEqual(sync(h).extensionRemote['skill:design'],plain(h.w.extensionPrototype.syncObject('skill:design')));
 h.w.syncPrototype.setScenario('download');h.w.syncPrototype.setImportSample('plaintext');h.w.syncPrototype.open({tab:'file',scope:['skill:development'],action:'import'});h.click('[data-sx=choose-import-file]');h.click('#modalConfirm');await finish(h);assert.match(h.w.extensionPrototype.syncObject('skill:development').description,/同步内容示例/);
 const before=plain(h.w.extensionPrototype.syncObject('skill:design'));h.w.syncPrototype.setImportSample('extension-legacy');h.w.syncPrototype.open({tab:'file',scope:['skill:design'],action:'import'});h.click('[data-sx=choose-import-file]');h.click('#modalConfirm');const result=await finish(h);assert.equal(result.failed.length,1);assert.match(result.failed[0].reason,/旧版.*转换/);assert.deepEqual(plain(h.w.extensionPrototype.syncObject('skill:design')),before);
});
