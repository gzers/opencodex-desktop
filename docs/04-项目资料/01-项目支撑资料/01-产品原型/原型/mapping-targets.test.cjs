const test=require('node:test'),assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');
const json=x=>JSON.parse(JSON.stringify(x));
async function setup(t){const h=await make('#sync?tab=webdav');t.after(()=>{assert.deepEqual(h.errors,[]);assert.equal(h.network.length,0);h.dispose()});return h;}
const state=h=>json(h.w.syncPrototype.state()),info=(h,id)=>json(h.w.extensionPrototype.platformSyncInfo(id));
async function finish(h){for(let i=0;i<60&&state(h).running;i++)await h.wait(40);assert.equal(state(h).running,false);return state(h).result;}
function preview(h,ids,scene='download'){h.w.syncPrototype.open({tab:'webdav',scope:ids});h.w.syncPrototype.setScenario(scene);h.click('[data-sx=check]');}
test('平台列表只显示已发现的平台、原始路径和映射结果，无配置编辑',async t=>{
 const h=await setup(t);h.click('[data-sx=expand][data-id=extension-platforms]');const d=h.document;
 assert.equal(d.querySelectorAll('.sx-platform-object').length,24);assert.equal(d.querySelectorAll('[data-sx-target-value],.sx-platform-object select').length,0);assert.equal(state(h).selected.filter(x=>x.startsWith('skill-platform:')).length,24);
 const claude=d.querySelector('[data-sx-object="skill-platform:claude"]').closest('.sx-platform-object'),codex=d.querySelector('[data-sx-object="skill-platform:codex"]').closest('.sx-platform-object');
 assert.match(claude.textContent,/~\/.claude\/skills/);assert.match(claude.textContent,/未映射/);assert.match(codex.textContent,/已映射/);assert.equal(d.querySelector('[data-sx-object="skill-platform:gajae"]'),null);
 h.change('[data-sx-object="skill-platform:claude"]',true);h.change('[data-sx-object="skill-platform:claude"]',false);assert.equal(info(h,'claude').mapped,false);
});
test('未映射平台也能导出，载荷只含平台标识；路径和映射结果不传输',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'file',scope:['skill-platform:claude'],action:'export'});assert.match(h.document.querySelector('.sx-platform-associations').textContent,/未映射也可同步/);h.click('#modalConfirm');
 const values=state(h).lastExport.snapshot.extensions.objects;assert.deepEqual(values,[{id:'skill-platform:claude',payload:{schemaVersion:2,kind:'skill-platform',platformId:'claude'}}]);assert.equal(info(h,'claude').mapped,false);
});
test('未映射平台下载只保存关联；未选平台和已有映射保持',async t=>{
 const h=await setup(t),before=info(h,'codex');preview(h,['skill-platform:claude']);assert.match(h.document.querySelector('.sx-platform-associations').textContent,/使用 Skills 前需完成本机映射/);h.click('#modalConfirm');const result=await finish(h);
 assert.equal(result.failed.length,0);assert.equal(result.downloaded,1);assert.equal(info(h,'claude').mapped,false);assert.deepEqual(info(h,'codex'),before);
 assert.ok(h.w.extensionPrototype.syncReadiness(['skill-platform:claude']).some(x=>x.reason==='待映射'));h.click('[data-sx=result-details]');assert.match(h.document.querySelector('.sx-platform-associations').textContent,/未映射/);h.click('[data-sx=extensions]');await h.settle();assert.ok(h.document.querySelector('.xp3-sync-pending'));
});
test('只指定内容的入口沿用已选Agent平台；映射不是内容同步前置条件',async t=>{
 const h=await setup(t);preview(h,['skill:design']);h.click('#modalConfirm');const result=await finish(h);assert.equal(result.failed.length,0);assert.equal(result.items.find(x=>x.id==='skill:design').status,'completed');assert.equal(h.w.extensionPrototype.syncObject('skill:design').enabled,false);assert.equal(info(h,'claude').mapped,false);
 assert.equal(result.extensionIds.some(x=>x.startsWith('skill-platform:')),true);
});
test('文件导入Agent平台默认选中；只显示本机已发现平台',async t=>{
 const h=await setup(t);h.w.syncPrototype.setScenario('download');h.w.syncPrototype.setImportSample('plaintext');h.w.syncPrototype.open({tab:'file',action:'import'});h.click('[data-sx=choose-import-file]');
 const inputs=[...h.document.querySelectorAll('[data-sx-import-object^="skill-platform:"]')];assert.equal(inputs.length,24);assert.ok(inputs.every(x=>x.checked));assert.equal(h.document.querySelector('[data-sx-import-object="skill-platform:gajae"]'),null);
 for(const el of [...h.document.querySelectorAll('[data-sx-import-object]:checked')])h.change('[data-sx-import-object="'+el.dataset.sxImportObject+'"]',false);
 h.change('[data-sx-import-object="skill-platform:claude"]',true);assert.equal(h.document.querySelectorAll('[data-sx-platform-result]').length,1);h.click('#modalConfirm');const result=await finish(h);assert.equal(result.failed.length,0);assert.equal(info(h,'claude').mapped,false);
});
test('检测消失从选择和列表移除；在途消失跳过关联但保留内容同步',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',scope:['skill-platform:codex','skill:design']});h.w.setRoute('extensions');h.w.extensionPrototype.refresh();h.change('#xp3DetectionScene','codex-missing');h.click('[data-xp3=detect]');await h.wait(850);h.w.closeModal();
 assert.equal(info(h,'codex').installed,false);assert.equal(state(h).selected.includes('skill-platform:codex'),false);h.w.syncPrototype.open({tab:'webdav'});h.click('[data-sx=expand][data-id=extension-platforms]');assert.equal(h.document.querySelector('[data-sx-object="skill-platform:codex"]'),null);
 h.w.setRoute('extensions');h.change('#xp3DetectionScene','normal');h.click('[data-xp3=detect]');await h.wait(850);h.w.closeModal();
 // Detection completes during the frozen content transfer; the unavailable association must be skipped.
 h.change('#xp3DetectionScene','codex-missing');h.click('[data-xp3=detect]');await h.wait(50);h.w.syncPrototype.open({tab:'webdav',scope:['skill:design','skill-platform:codex']});h.w.syncPrototype.setScenario('download');h.click('[data-sx=check]');h.click('#modalConfirm');
 const result=await finish(h);assert.equal(result.items.find(x=>x.id==='skill:design').status,'skipped');assert.equal(info(h,'codex').installed,false);assert.equal(result.items.find(x=>x.id==='skill-platform:codex').status,'skipped');assert.match(result.items.find(x=>x.id==='skill-platform:codex').reason,/本机已无法发现/);
});
test('手动验证发现的平台进入选择，取消选择不改变映射',async t=>{
 const h=await setup(t);h.w.setRoute('extensions');h.w.extensionPrototype.refresh();h.click('#xp3RightRows [data-ic-action=install][data-id=gajae]');h.input('#xp3InstallPath','/opt/gjc/gjc');h.click('[data-platform-action=validate-install]');await h.wait(500);h.click('[data-platform-action=save-install]');h.w.closeModal();
 h.w.syncPrototype.open({tab:'webdav'});h.click('[data-sx=expand][data-id=extension-platforms]');assert.ok(h.document.querySelector('[data-sx-object="skill-platform:gajae"]'));assert.equal(info(h,'gajae').mapped,false);
 h.change('[data-sx-object="skill-platform:gajae"]',true);h.change('[data-sx-object="skill-platform:gajae"]',false);assert.equal(info(h,'gajae').mapped,false);
});
test('非法关联、旧目标配置不能写入映射；Windows只显示本机路径',async t=>{
 const h=await setup(t),before=info(h,'codex');
 for(const payload of [{schemaVersion:2,kind:'skill-platform',platformId:'claude'},{schemaVersion:2,kind:'skill-platform',platformId:'codex',mapped:false},{schemaVersion:2,kind:'skill-mapping-target',platformId:'codex',desired:false}])assert.equal(h.w.extensionPrototype.applySyncObject('skill-platform:codex',payload).ok,false);
 assert.deepEqual(info(h,'codex'),before);h.w.setRoute('extensions');h.change('#xp3DirectoryOS','windows');h.w.syncPrototype.open({tab:'webdav'});h.click('[data-sx=expand][data-id=extension-platforms]');assert.match(h.document.querySelector('[data-sx-object="skill-platform:claude"]').closest('.sx-platform-object').textContent,/%USERPROFILE%\\.claude\\skills/);
});
