const test=require('node:test'),assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');
const plain=x=>JSON.parse(JSON.stringify(x)),s=h=>plain(h.w.syncPrototype.state()),resource=(h,id)=>plain(h.w.extensionPrototype.syncObject(id));
async function setup(t){const h=await make('#sync?tab=webdav');t.after(()=>{assert.deepEqual(h.errors,[]);assert.equal(h.network.length,0);h.dispose()});return h;}
async function finish(h){for(let i=0;i<80&&s(h).running;i++)await h.wait(40);assert.equal(s(h).running,false);return s(h).result;}
function selectImport(h,id,on){h.change('[data-sx-import-object="'+id+'"]',on)}
function start(h,ids,scene='download'){h.w.syncPrototype.open({tab:'webdav',scope:ids});h.w.syncPrototype.setScenario(scene);h.click('[data-sx=check]');h.click('#modalConfirm')}
test('首次默认选全部已发现Agent平台，两类内容随平台有效；不包含未发现平台',async t=>{
 const h=await setup(t);assert.equal(s(h).selected.filter(x=>x.startsWith('skill-platform:')).length,24);assert.ok(s(h).effective.includes('skill:design'));assert.ok(s(h).effective.includes('mcp:documentation'));assert.ok(!s(h).selected.includes('skill-platform:gajae'));assert.ok(!s(h).selected.includes('mcp:node_repl'));
 h.w.syncPrototype.open({tab:'file'});assert.equal(s(h).selected.filter(x=>x.startsWith('skill-platform:')).length,24);
});
test('取消全部平台不传Skills/MCP及凭据，其他配置继续；恢复平台保留内容和凭据勾选',async t=>{
 const h=await setup(t);h.click('[data-sx=expand][data-id=skills]');h.click('[data-sx=expand][data-id=mcp]');
 const ids=s(h).effective.filter(x=>/^(skill:|mcp:)/.test(x)),secrets=s(h).secrets.extensions;
 h.change('[data-sx-group=extension-platforms]',false);assert.equal(s(h).effective.filter(x=>/^(skill:|mcp:)/.test(x)).length,0);assert.equal(s(h).secrets.extensions.length,0);assert.ok(s(h).effective.includes('channel:newapi'));assert.match(h.document.querySelector('.sx-extension-scope-note').textContent,/不同步 Skills \/ MCP/);assert.ok(h.document.querySelector('[data-sx-object="skill:design"]').disabled);
 h.w.syncPrototype.open({tab:'webdav',action:'check'});assert.ok(s(h).preview.rows.every(x=>!['skills','mcp'].includes(x.group)));h.w.closeModal();
 h.click('[data-sx=expand][data-id=extension-platforms]');h.change('[data-sx-object="skill-platform:claude"]',true);assert.deepEqual(s(h).effective.filter(x=>/^(skill:|mcp:)/.test(x)),ids);assert.deepEqual(s(h).secrets.extensions,secrets);assert.equal(h.w.extensionPrototype.platformSyncInfo('claude').mapped,false);
});
test('文件导出未选平台排除内容与凭据，选择未映射平台也能导出一次共享源',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'file',scope:['preferences:desktop','skill:design','mcp:filesystem'],action:'export'});h.w.closeModal();h.change('[data-sx-group=extension-platforms]',false);h.click('[data-sx=export]');h.click('#modalConfirm');let file=s(h).lastExport;assert.deepEqual(file.objects,['preferences:desktop']);assert.equal(file.snapshot.extensions.objects.length,0);
 h.w.syncPrototype.open({tab:'file',scope:['skill:design','mcp:filesystem','skill-platform:claude'],action:'export'});h.click('#modalConfirm');file=s(h).lastExport;assert.deepEqual(file.agentPlatforms,['claude']);assert.equal(file.snapshot.extensions.objects.filter(x=>x.id==='skill:design').length,1);assert.deepEqual(file.snapshot.extensions.objects.find(x=>x.id==='mcp:filesystem').payload.platformIds,['claude']);assert.equal(h.w.extensionPrototype.platformSyncInfo('claude').mapped,false);
});
test('只选Claude下载保留其他平台MCP启用位；未映射不阻止内容更新',async t=>{
 const h=await setup(t),before=resource(h,'mcp:documentation');start(h,['skill:design','mcp:documentation','skill-platform:claude']);const result=await finish(h);assert.equal(result.failed.length,0);assert.equal(resource(h,'skill:design').enabled,false);assert.equal(h.w.extensionPrototype.platformSyncInfo('claude').mapped,false);
 const ids=resource(h,'mcp:documentation').platformIds;assert.ok(ids.includes('claude'));assert.equal(ids.includes('codex'),before.platformIds.includes('codex'));assert.ok(!ids.includes('gajae'));
});
test('上传只覆盖所选平台MCP关系，范围外远端关系保留',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',scope:['mcp:documentation','skill-platform:codex']});h.w.syncPrototype.setScenario('upload');const before=s(h).extensionRemote['mcp:documentation'].platformIds;h.click('[data-sx=check]');assert.deepEqual(s(h).preview.agentPlatforms,['codex']);h.click('#modalConfirm');await finish(h);const after=s(h).extensionRemote['mcp:documentation'].platformIds;assert.ok(after.includes('codex'));for(const id of before.filter(x=>x!=='codex'))assert.ok(after.includes(id));
});
test('预览之后平台选择变化阻止执行；进行中变化只影响下一次范围',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',scope:['mcp:documentation','skill-platform:codex']});h.w.syncPrototype.setScenario('download');h.click('[data-sx=expand][data-id=extension-platforms]');h.click('[data-sx=check]');h.change('[data-sx-object="skill-platform:codex"]',false);h.click('#modalConfirm');assert.match(h.document.querySelector('#sxError').textContent,/已变化/);assert.equal(s(h).running,false);h.w.closeModal();
 start(h,['mcp:documentation','skill-platform:codex']);h.change('[data-sx-object="skill-platform:codex"]',false);const result=await finish(h);assert.ok(result.items.some(x=>x.id==='mcp:documentation'&&x.status==='completed'));assert.equal(s(h).effective.includes('mcp:documentation'),false);
});
test('导入默认选平台；取消全部平台只导入其他配置，重新选择恢复原内容选项',async t=>{
 const h=await setup(t);h.w.syncPrototype.setImportSample('plaintext');h.w.syncPrototype.setScenario('download');h.w.syncPrototype.open({tab:'file',action:'import'});h.click('[data-sx=choose-import-file]');const d=h.document;assert.equal(d.querySelectorAll('[data-sx-import-object^="skill-platform:"]:checked').length,24);
 for(const input of [...d.querySelectorAll('[data-sx-import-object^="skill-platform:"]')])selectImport(h,input.dataset.sxImportObject,false);assert.ok(d.querySelector('[data-sx-import-object="skill:design"]').disabled);assert.equal(d.querySelector('[data-sx-import-secrets]').checked,false);assert.match(d.querySelector('#sxImportPlatformScope').textContent,/不同步 Skills \/ MCP/);
 selectImport(h,'skill-platform:claude',true);assert.ok(d.querySelector('[data-sx-import-object="skill:design"]').checked);assert.ok(!d.querySelector('[data-sx-import-object="mcp:documentation"]').disabled);selectImport(h,'skill-platform:claude',false);
 const before=resource(h,'skill:design');h.click('#modalConfirm');const result=await finish(h);assert.ok(result.items.every(x=>!['skills','mcp'].includes(x.group)));assert.deepEqual(resource(h,'skill:design'),before);
});
test('导入仅选一个平台时过滤MCP关系并保留本机未选平台关系',async t=>{
 const h=await setup(t);h.w.syncPrototype.setImportSample('plaintext');h.w.syncPrototype.setScenario('download');h.w.syncPrototype.open({tab:'file',action:'import'});h.click('[data-sx=choose-import-file]');
 for(const input of [...h.document.querySelectorAll('[data-sx-import-object]')])selectImport(h,input.dataset.sxImportObject,false);selectImport(h,'skill-platform:claude',true);selectImport(h,'mcp:documentation',true);h.click('#modalConfirm');await finish(h);assert.deepEqual(resource(h,'mcp:documentation').platformIds.sort(),['claude','codex']);
});
test('平台全部消失后跳过拓展内容；其他对象可完成；手动新发现不重置已记忆选择',async t=>{
 const h=await setup(t);h.w.setRoute('extensions');h.w.extensionPrototype.refresh();h.change('#xp3DetectionScene','codex-missing');h.click('[data-xp3=detect]');await h.wait(50);
 start(h,['skill:design','skill-platform:codex','preferences:desktop']);const result=await finish(h);assert.equal(result.items.find(x=>x.id==='skill:design').status,'skipped');assert.equal(result.items.find(x=>x.id==='preferences:desktop').status,'completed');assert.equal(s(h).selected.includes('skill-platform:codex'),false);
 h.w.setRoute('extensions');h.w.extensionPrototype.refresh();h.click('#xp3RightRows [data-ic-action=install][data-id=gajae]');h.input('#xp3InstallPath','/opt/gjc/gjc');h.click('[data-platform-action=validate-install]');await h.wait(500);h.click('[data-platform-action=save-install]');h.w.closeModal();h.w.syncPrototype.open({tab:'webdav'});h.click('[data-sx=expand][data-id=extension-platforms]');assert.ok(h.document.querySelector('[data-sx-object="skill-platform:gajae"]'));assert.equal(s(h).selected.includes('skill-platform:gajae'),false);
});
