/* 唯一总原型：直接同步/导入的内存交互，不代表真实文件与原生Windows验收。 */
const test=require('node:test'),assert=require('node:assert/strict');
const fs=require('node:fs'),path=require('node:path');
const {make}=require('./component-topic-harness.cjs');
const json=x=>JSON.stringify(x),model=h=>h.w.modelPrototype.state(),sync=h=>h.w.syncPrototype.state();
const first=policy=>policy.providers.find(p=>p.id==='newapi').models[0];
async function setup(t){const h=await make('#sync?tab=file');h.w.confirm=()=>{throw Error('禁止额外确认')};t.after(()=>{try{assert.equal(h.network.length,0);assert.deepEqual(h.errors,[])}finally{h.dispose()}});return h;}
function patch(h,alias){assert.equal(h.w.modelPrototype.commitModelPatches('newapi',[{key:'newapi::0',naming:{alias}}]),'');}
function exportPreview(h,options={}){h.w.syncPrototype.open({tab:'file',scope:['channel:newapi'],action:'export',...options});}
function finishExport(h){h.change('#sxEncrypt',false);h.click('#modalConfirm');return sync(h).lastExport;}
async function finished(h){for(let i=0;i<80&&sync(h).running;i++)await h.wait(50);assert.equal(sync(h).running,false);return sync(h).result;}
function chooseImport(h,sample='keyless',scope=['channel:newapi']){h.w.syncPrototype.setImportSample(sample);h.w.syncPrototype.open({tab:'file',scope,action:'import'});assert.equal(h.document.querySelector('#modalConfirm').disabled,true);h.click('[data-sx=choose-import-file]');if(sample==='encrypted')h.input('#sxUnlock','demo-sync');}
test('普通导出冻结当前已保存渠道文件内容，未保存修改不混入；只有保存确认更新基准',async t=>{
 const h=await setup(t),saved=json(model(h).saved);
 patch(h,'memory-only');exportPreview(h);const file=finishExport(h);
 assert.equal(file.source,'committed');assert.equal(file.snapshotSource,'saved');
 assert.equal(first(file.snapshot.modelChannels).naming,undefined);
 assert.equal(first(model(h).draft).naming.alias,'memory-only');assert.equal(json(model(h).saved),saved);
 assert.equal(file.snapshot.modelChannels.providers.length,1);assert.equal(file.snapshot.modelChannels.providers[0].id,'newapi');
 assert.equal(file.snapshot.modelChannels.providers[0].connection.credentialRef,undefined);
 assert.equal(first(file.snapshot.modelChannels).official,undefined);
 h.w.setRoute('models');h.click('#mpPolicySave');
 assert.equal(json(model(h).saved),saved);assert.equal(model(h).operation,null);
 h.click('#modalConfirm');assert.equal(model(h).operation.status,'running');assert.equal(json(model(h).saved),saved);
 for(let i=0;i<100&&model(h).operation?.status==='running';i++)await h.wait(50);
 assert.equal(model(h).operation.status,'written');assert.equal(first(model(h).saved).naming.alias,'memory-only');
 exportPreview(h);assert.equal(first(finishExport(h).snapshot.modelChannels).naming.alias,'memory-only');
});

test('只有显式 draft 来源导出内存渠道；预览后继续编辑不改冻结内容，下一次普通导出仍用 saved',async t=>{
 const h=await setup(t),saved=json(model(h).saved);
 patch(h,'chosen-draft');exportPreview(h,{source:'draft'});patch(h,'later-draft');
 const file=finishExport(h);assert.equal(file.source,'draft');assert.equal(file.snapshotSource,'draft');
 assert.equal(first(file.snapshot.modelChannels).naming.alias,'chosen-draft');
 assert.equal(first(model(h).draft).naming.alias,'later-draft');assert.equal(json(model(h).saved),saved);
 file.snapshot.modelChannels.providers[0].name='外部修改';
 assert.equal(sync(h).lastExport.snapshot.modelChannels.providers[0].name,'NewApi');
 exportPreview(h);assert.equal(first(finishExport(h).snapshot.modelChannels).naming,undefined);
});

test('仅存在草稿的新渠道可显式导出，普通已保存来源不把它列为可导出对象',async t=>{
 const h=await setup(t),saved=json(model(h).saved);
 h.w.setRoute('models');h.click('[data-mp=copy-provider][data-id=newapi]');h.click('#modalConfirm');
 assert.equal(model(h).draft.providers.some(p=>p.id==='newapi-copy'),true);
 exportPreview(h,{source:'draft',scope:['channel:newapi-copy']});const file=finishExport(h);
 assert.equal(file.snapshot.modelChannels.providers[0].id,'newapi-copy');assert.equal(json(model(h).saved),saved);
 exportPreview(h,{scope:['channel:newapi-copy']});assert.equal(sync(h).selected.length,0);assert.equal(sync(h).dialogKind,'');
 assert.equal(json(sync(h).lastExport),json(file));assert.equal(model(h).operation,null);
});

test('文件保护可选：带渠道凭据的明文导出保留选择，既有确认后完成，不弹第二次确认',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'file',scope:['channel:newapi']});
 h.change('[data-sx-secret=keys]',true);
 assert.match(h.document.querySelector('.sx-note').textContent,/随范围同步/);
 h.click('[data-sx=export]');assert.equal(h.document.querySelector('#sxEncrypt').checked,false);assert.equal(h.document.querySelector('#sxPassFields').hidden,true);
 h.click('#modalConfirm');const file=sync(h).lastExport;assert.equal(file.encryptionRequired,false);assert.equal(file.encrypted,false);
 assert.equal(file.credentials.keys[0],'channel:newapi');assert.match(file.snapshot.modelChannels.providers[0].connection.credentialRef,/^mock-key:/);
 assert.doesNotMatch(json(file),/demo-newapi-key|demo-sync/);assert.equal(sync(h).dialogKind,'');
});

test('选用文件加密时仍校验口令，两次一致才完成；口令不进入导出状态',async t=>{
 const h=await setup(t);exportPreview(h);h.change('#sxEncrypt',true);
 h.input('#sxPass','mock-only-pass');h.input('#sxPassAgain','different');h.click('#modalConfirm');
 assert.equal(sync(h).lastExport,null);assert.match(h.document.querySelector('#sxError').textContent,/两次一致/);
 h.input('#sxPassAgain','mock-only-pass');h.click('#modalConfirm');
 assert.equal(sync(h).lastExport.encrypted,true);assert.equal(sync(h).lastExport.encryptionRequired,false);
 assert.doesNotMatch(json(sync(h).lastExport),/mock-only-pass/);
});

test('取消导出不保存渠道，也不替换上一份导出快照',async t=>{
 const h=await setup(t);exportPreview(h);finishExport(h);const last=json(sync(h).lastExport),saved=json(model(h).saved);
 patch(h,'cancelled-export');const draft=json(model(h).draft);exportPreview(h,{source:'draft'});h.click('#modalCancel');
 assert.equal(json(sync(h).lastExport),last);assert.equal(json(model(h).saved),saved);assert.equal(json(model(h).draft),draft);assert.equal(model(h).operation,null);
});

test('模版导出只取已保存版本；模版缓冲、保存新版本与渠道草稿/文件保存相互独立',async t=>{
 const h=await setup(t),saved=json(model(h).saved),draft=json(model(h).draft);
 h.w.setRoute('models');h.click('[data-mp=tab][data-tab=templates]');h.click('[data-mp=open-template-editor][data-id=gpt]');
 h.input('#mpTemplateDescription','buffer-not-exported');const library=json(model(h).templates),buffers=json(model(h).buffers);
 exportPreview(h,{source:'draft',scope:['template:gpt']});const file=finishExport(h);
 assert.equal(file.snapshot.modelChannels,null);assert.equal(file.snapshot.templates.length,1);
 assert.doesNotMatch(json(file),/buffer-not-exported/);assert.equal(json(model(h).templates),library);assert.equal(json(model(h).buffers),buffers);
 h.w.setRoute('models');h.click('[data-mp=save-template]');h.click('#modalConfirm');
 assert.equal(model(h).templates.find(t=>t.id==='gpt').versions.at(-1).description,'buffer-not-exported');
 assert.equal(model(h).templates.find(t=>t.id==='gpt').main,1);assert.equal(json(model(h).saved),saved);assert.equal(json(model(h).draft),draft);assert.equal(model(h).operation,null);
});


test('范围和凭据自动记住；没有单独保存范围或逐次复核',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',scope:['channel:newapi']});
 assert.equal(h.document.querySelector('[data-sx=save-scope]'),null);assert.equal(h.document.querySelector('[data-sx-reviewed]'),null);
 h.change('[data-sx-secret=keys]',true);assert.equal(sync(h).dirty,false);const saved=sync(h).savedScope;
 h.w.setRoute('models');h.w.syncPrototype.open({tab:'webdav'});assert.equal(sync(h).savedScope,saved);assert.equal(sync(h).secrets.keys[0],'channel:newapi');
 h.click('[data-sx=check]');assert.equal(sync(h).dialogKind,'');await finished(h);assert.equal(sync(h).channelAdoption.status,'written');assert.equal(first(model(h).saved).naming.alias,'primary-model');
});
for(const scene of ['upload','download','same'])test(scene+' 按固定内容版本执行，第二次同步无重复更新',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',scope:['skill:review','mcp:docs']});h.w.syncPrototype.setScenario(scene);const before=sync(h).versions;
 h.click('[data-sx=check]');const result=await finished(h);assert.equal(result.failed.length,0);
 assert.equal(result.uploaded,scene==='upload'?2:0);assert.equal(result.downloaded,scene==='download'?2:0);assert.equal(result.skipped,scene==='same'?2:0);
 if(scene==='download'){assert.deepEqual([...sync(h).demoObjects['skill:review'].files],['SKILL.md','scripts/review.js','references/checklist.md']);assert.equal(sync(h).versions.local['skill:review'].updatedAt,before.remote['skill:review'].updatedAt);}
 h.click('[data-sx=check]');const again=await finished(h);assert.equal(again.uploaded+again.downloaded,0);assert.equal(again.skipped,2);
});
test('一次同步可同时上传和下载，当前范围以外对象不改',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',scope:['skill:review','mcp:docs']});const before=json(sync(h).versions.local['channel:newapi']);
 h.click('[data-sx=check]');const result=await finished(h);assert.equal(result.downloaded,1);assert.equal(result.uploaded,1);assert.equal(json(sync(h).versions.local['channel:newapi']),before);
});
test('运行中更改范围只作用下一次，不要求重开旧计划',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',scope:['skill:review','mcp:docs']});h.click('[data-sx=check]');h.click('[data-sx=clear]');const result=await finished(h);assert.equal(result.items.length,2);assert.equal(sync(h).selected.length,0);assert.equal(h.document.querySelector('[data-sx=check]').disabled,true);
});
test('独立失败不阻断其余对象；只重试失败项并清除旧错误',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',scope:['skill:review','mcp:docs']});h.w.syncPrototype.setScenario('partial');h.click('[data-sx=check]');const result=await finished(h);assert.equal(result.downloaded,1);assert.equal(result.failed[0].id,'mcp:docs');const version=json(sync(h).versions.local['skill:review']);
 h.click('[data-sx=retry]');const retry=await finished(h);assert.equal(retry.failed.length,0);assert.equal(retry.items.length,1);assert.equal(retry.uploaded,1);assert.equal(json(sync(h).versions.local['skill:review']),version);
});
test('网络失败不改内容，可从原页面重试',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',scope:['skill:review']});h.w.syncPrototype.setScenario('network-error');const before=json(sync(h).versions);h.click('[data-sx=check]');const result=await finished(h);assert.equal(result.failed.length,1);assert.equal(json(sync(h).versions),before);h.click('[data-sx=retry]');assert.equal((await finished(h)).downloaded,1);
});
for(const sample of ['keyless','encrypted'])test(sample+' 导入在同一窗口选范围后直接更新saved，无需再保存',async t=>{
 const h=await setup(t),saved=json(model(h).saved),native=json(model(h).saved.providers.find(p=>p.id==='native'));chooseImport(h,sample);assert.equal(h.document.querySelector('#modalTitle').textContent,'导入配置');assert.equal(h.document.querySelector('[data-sx=compare-expand]'),null);
 if(sample==='encrypted')h.change('[data-sx-import-keys]',true);
 h.click('#modalConfirm');assert.equal(sync(h).dialogKind,'');await finished(h);assert.notEqual(json(model(h).saved),saved);assert.equal(first(model(h).saved).naming.alias,'primary-model');assert.equal(sync(h).channelAdoption.target,'mock-current-channel-file');assert.equal(model(h).operation,null);
 assert.equal(model(h).saved.providers.filter(p=>p.id==='newapi').length,1);assert.equal(json(model(h).saved.providers.find(p=>p.id==='native')),native);assert.ok(model(h).templates.find(t=>t.id==='gpt').versions.some(v=>v.n===4));
 exportPreview(h);assert.equal(first(finishExport(h).snapshot.modelChannels).naming.alias,'primary-model');
});
test('导入未包含的对象和本机未保存编辑保留，待定策略只演示不伪称已批准',async t=>{
 const h=await setup(t);patch(h,'local-buffer');const before=json(model(h).saved.providers.find(p=>p.id==='a6api'));chooseImport(h);h.click('#modalConfirm');await finished(h);assert.equal(first(model(h).saved).naming.alias,'primary-model');assert.equal(first(model(h).draft).naming.alias,'local-buffer');assert.equal(json(model(h).saved.providers.find(p=>p.id==='a6api')),before);
});
test('官方openai/native不进入同步范围，旧官方条目不新增，自定义渠道仍原位更新',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',scope:['channel:native','channel:openai','channel:newapi']});assert.deepEqual([...sync(h).selected],['channel:newapi']);const native=json(model(h).saved.providers.find(p=>p.id==='native'));
 h.click('[data-sx=check]');await finished(h);assert.equal(json(model(h).saved.providers.find(p=>p.id==='native')),native);assert.equal(model(h).saved.providers.filter(p=>p.id==='newapi').length,1);
});
test('加密为可选且记住选择，不含口令不执行；口令不暴露在状态中',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',scope:['skill:review']});h.change('[data-sx-protect]',true);h.click('[data-sx=check]');assert.equal(sync(h).running,false);assert.match(sync(h).result.title,/口令/);h.input('#sxProtectionPass','fixture-only-pass');h.click('[data-sx=check]');await finished(h);assert.equal(sync(h).protection.configured,true);assert.doesNotMatch(json(sync(h)),/fixture-only-pass/);
});
test('没有文件、错误口令和损坏包均不执行；取消不改内容',async t=>{
 const h=await setup(t),saved=json(model(h).saved);chooseImport(h,'encrypted');h.input('#sxUnlock','wrong');h.click('#modalConfirm');assert.match(h.document.querySelector('#sxError').textContent,/口令/);assert.equal(json(model(h).saved),saved);h.click('#modalCancel');chooseImport(h,'corrupt');h.click('#modalConfirm');assert.match(h.document.querySelector('#sxError').textContent,/无法读取/);assert.equal(json(model(h).saved),saved);h.click('#modalCancel');
});
test('结果详情只读，不产生第二次采用；同一模版导入不重复版本',async t=>{
 const h=await setup(t);for(let i=0;i<2;i++){chooseImport(h,'keyless',['template:gpt']);h.click('#modalConfirm');await finished(h);}assert.equal(model(h).templates.find(t=>t.id==='gpt').versions.filter(v=>v.n===4).length,1);
 const saved=json(model(h).saved);h.click('[data-sx=result-details]');assert.equal(h.document.querySelector('#modalTitle').textContent,'同步结果');assert.equal(h.document.querySelector('[data-sx=compare-expand]'),null);h.click('#modalCancel');assert.equal(json(model(h).saved),saved);
});
test('原型保持内存边界，普通同步没有旧比较或保存范围门',()=>{
 const source=fs.readFileSync(path.join(__dirname,'sync-config.js'),'utf8');for(const token of ['fetch(', 'XMLHttpRequest', 'WebSocket(', 'showOpenFilePicker(', 'showSaveFilePicker(', 'require(', 'localStorage.setItem(', 'sessionStorage.setItem(', '__TAURI__', 'invoke(', 'confirm('])assert.equal(source.includes(token),false,token);
 assert.doesNotMatch(source,/btn\('保存本机范围'|btn\('同步配置'|data-sx-compare-choice|function syncPreview|stageSyncChannels/);
});

test('Windows面板反馈示例区分页面失败和代理未运行，重试不重复启动',async t=>{
 const h=await setup(t);h.w.windowPanelMock('load-failed');assert.equal(h.document.querySelector('#panelEmpty strong').textContent,'官方面板加载失败');assert.match(h.document.querySelector('#panelEmpty span').textContent,/正在运行/);assert.equal(h.document.querySelector('#panelErrorBrowser').hidden,false);h.click('#panelRetryBtn');assert.equal(h.document.querySelector('#panelReal').classList.contains('show'),true);
 h.w.windowPanelMock('empty');assert.match(h.document.querySelector('#panelEmpty strong').textContent,/未运行/);assert.equal(h.document.querySelector('#panelErrorBrowser').hidden,true);
});

test('同步结果沿已有注册事件/Toast/铃铛，收起后仍可查看',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',scope:['skill:review']});h.click('[data-sx=check]');await finished(h);const notices=h.w.eval('notifications.filter(n=>n.eventId==="sync.complete")');assert.ok(notices.length);assert.equal(notices[0].title,'同步完成');assert.equal(h.document.querySelectorAll('#taskHost [data-task-notification^="sync-demo:"]').length,0);h.click('#toastEl [data-collapse=toast]');await h.wait(300);h.click('#notificationBtn');assert.match(h.document.querySelector('#notificationPanel').textContent,/同步完成/);
});

test('旧包不会反向覆盖已保存的新内容，比较在后台完成',async t=>{
 const h=await setup(t);chooseImport(h);h.click('#modalConfirm');await finished(h);patch(h,'newer-local-save');h.w.setRoute('models');h.click('#mpPolicySave');h.click('#modalConfirm');for(let i=0;i<120&&model(h).operation?.status==='running';i++)await h.wait(50);assert.equal(first(model(h).saved).naming.alias,'newer-local-save');
 chooseImport(h);h.click('#modalConfirm');const result=await finished(h);assert.equal(result.downloaded,0);assert.equal(result.skipped,1);assert.equal(first(model(h).saved).naming.alias,'newer-local-save');assert.equal(sync(h).dialogKind,'');
});

test('明文包含凭据也可导入，资格由所选包内对象决定，无解密或额外确认',async t=>{
 const h=await setup(t);chooseImport(h,'plaintext',['channel:newapi','mcp:docs']);
 assert.equal(sync(h).importFile.kind,'plaintext');assert.equal(h.document.querySelector('#sxUnlockField').hidden,true);
 for(const selector of ['[data-sx-import-keys]','[data-sx-import-secrets]'])assert.equal(h.document.querySelector(selector).disabled,false);
 h.change('[data-sx-import-keys]',true);h.change('[data-sx-import-secrets]',true);h.click('#modalConfirm');await finished(h);
 assert.match(model(h).saved.providers.find(p=>p.id==='newapi').connection.credentialRef,/^mock-key:/);
 assert.equal(sync(h).result.items.find(x=>x.id==='channel:newapi').credential,true);assert.equal(sync(h).dialogKind,'');
});

test('取消勾选包内对象即禁用其凭据选项，不误把其它渠道凭据带入；重选保留偏好',async t=>{
 const h=await setup(t);chooseImport(h,'plaintext',['channel:newapi','channel:legacy','mcp:docs']);
 h.change('[data-sx-import-keys]',true);h.change('[data-sx-import-secrets]',true);
 h.change('[data-sx-import-object="channel:newapi"]',false);h.change('[data-sx-import-object="mcp:docs"]',false);
 assert.equal(h.document.querySelector('[data-sx-import-keys]').disabled,true);assert.equal(h.document.querySelector('[data-sx-import-keys]').checked,false);
 assert.equal(h.document.querySelector('[data-sx-import-secrets]').disabled,true);
 h.change('[data-sx-import-object="channel:newapi"]',true);assert.equal(h.document.querySelector('[data-sx-import-keys]').checked,true);
 h.click('#modalCancel');chooseImport(h,'keyless');assert.equal(h.document.querySelector('[data-sx-import-keys]').checked,false);assert.equal(h.document.querySelector('[data-sx-import-keys]').disabled,true);h.click('#modalCancel');
});

test('文件页明确导出范围，导入范围来自所选文件而非继承页面导出勾选',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'file',scope:['skill:review']});
 assert.ok(h.document.querySelector('section[aria-label="导出范围"]'));assert.match(h.document.querySelector('.sx-table-head').textContent,/导出范围/);
 assert.match(h.document.querySelector('.sx-top p').textContent,/导入.*文件.*选择范围/);
 h.w.syncPrototype.setImportSample('keyless');h.click('[data-sx=import]');h.click('[data-sx=choose-import-file]');
 assert.equal(h.document.querySelector('fieldset legend').textContent,'导入范围');assert.ok(h.document.querySelector('[data-sx-import-object="channel:newapi"]'));assert.equal(h.document.querySelector('[data-sx-import-object="skill:review"]'),null);h.click('#modalCancel');
 assert.deepEqual([...sync(h).selected],['skill:review']);h.w.syncPrototype.open({tab:'webdav'});assert.ok(h.document.querySelector('section[aria-label="同步范围"]'));
});

test('通知专题导入复用共享窗口，普通同步不再提供覆盖确认或手选冲突侧',async t=>{
 const h=await setup(t);assert.equal(h.w.eval('NOTIF_SCEN.some(s=>s[1]==="覆盖同步"||s[1]==="WebDAV 冲突")'),false);
 h.w.eval('runNotifScenario(NOTIF_SCEN.findIndex(s=>s[1]==="配置导入"))');
 assert.equal(sync(h).dialogKind,'import');assert.ok(h.document.querySelector('[data-sx=choose-import-file]'));assert.equal(h.document.querySelector('#modalConfirm').textContent,'导入');h.click('#modalCancel');
 h.w.renderWebdav('synced');assert.doesNotMatch(h.document.querySelector('#card-overview-sync').textContent,/每次询问|冲突策略/);
 assert.match(h.document.querySelector('#card-overview-sync').textContent,/较新覆盖/);h.w.renderWebdav('partial');assert.equal(h.document.querySelector('#syncTag').textContent,'部分完成');
});

test('初次WebDAV提供的可用范围默认全选，凭据跟随范围；分享预设仍不带凭据',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav'});
 const all=[...h.document.querySelectorAll('[data-sx-group]:not([disabled])')];assert.ok(all.length>=5);assert.ok(all.every(el=>el.checked));assert.ok(sync(h).selected.length>all.length);
 assert.equal(h.document.querySelector('[data-sx-secret=keys]').checked,true);assert.equal(h.document.querySelector('[data-sx-secret=extensions]').checked,true);
 h.change('[data-sx-preset]','share');assert.equal(sync(h).secrets.keys.length,0);assert.equal(sync(h).secrets.extensions.length,0);
 h.change('[data-sx-preset]','all');assert.ok(sync(h).secrets.keys.length);assert.ok(sync(h).secrets.extensions.length);
});

test('导入在所提供范围默认全选含包内可用凭据，显式入口只显示对应范围',async t=>{
 const h=await setup(t);chooseImport(h,'plaintext',['channel:newapi','mcp:docs']);
 const items=[...h.document.querySelectorAll('[data-sx-import-object]')];assert.deepEqual(items.map(el=>el.dataset.sxImportObject).sort(),['channel:newapi','mcp:docs']);assert.ok(items.every(el=>el.checked));
 assert.equal(h.document.querySelector('[data-sx-import-keys]').checked,true);assert.equal(h.document.querySelector('[data-sx-import-secrets]').checked,true);h.click('#modalCancel');
 h.w.syncPrototype.open({tab:'file',action:'import'});h.click('[data-sx=choose-import-file]');assert.ok([...h.document.querySelectorAll('[data-sx-import-object]')].every(el=>el.checked));assert.ok(h.document.querySelectorAll('[data-sx-import-object]').length>items.length);h.click('#modalCancel');
});

test('文件与WebDAV范围分别记住，跨页、导入和导出均不把本机操作偏好装入配置包',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'file',scope:['preferences:desktop']});const file=sync(h).savedScopes.file;
 h.w.syncPrototype.open({tab:'webdav',scope:['skill:review','mcp:docs']});h.change('[data-sx-secret=extensions]',false);const dav=sync(h).savedScopes.webdav;
 h.w.setRoute('models');h.w.syncPrototype.open({tab:'file'});assert.equal(sync(h).savedScopes.file,file);assert.deepEqual([...sync(h).selected],['preferences:desktop']);
 h.click('[data-sx=export]');h.change('#sxEncrypt',false);h.click('#modalConfirm');assert.doesNotMatch(json(sync(h).lastExport),/savedScopes|savedScope|passphrase|sync-demo|demo-sync/);
 chooseImport(h,'config-only',['endpoint:personal']);h.click('#modalConfirm');await finished(h);assert.equal(sync(h).savedScopes.webdav,dav);
 h.w.syncPrototype.open({tab:'webdav'});assert.deepEqual([...sync(h).selected],['skill:review','mcp:docs']);assert.equal(sync(h).secrets.extensions.length,0);
 h.w.syncPrototype.open({tab:'file',scope:['preferences:desktop']});h.click('[data-sx=clear]');assert.deepEqual(JSON.parse(sync(h).savedScopes.file).ids,[]);assert.equal(sync(h).savedScopes.webdav,dav);
});

test('本页加密与不加密mock共用范围，运行中的保护选择被冻结并在结果中可见',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',scope:['skill:review','mcp:docs']});const scope=sync(h).savedScopes.webdav;
 h.click('[data-sx-protection-demo=encrypted]');assert.equal(sync(h).protection.enabled,true);assert.equal(sync(h).protection.configured,true);assert.equal(h.document.querySelector('[data-sx-protection-demo=encrypted]').getAttribute('aria-pressed'),'true');
 h.click('[data-sx=check]');h.click('[data-sx-protection-demo=plaintext]');let result=await finished(h);assert.equal(result.protection.encrypted,true);assert.equal(sync(h).savedScopes.webdav,scope);
 h.click('[data-sx=result-details]');assert.match(h.document.querySelector('.sx-dialog .sx-muted').textContent,/传输保护：加密/);h.click('#modalCancel');
 h.click('[data-sx=check]');result=await finished(h);assert.equal(result.protection.encrypted,false);assert.equal(sync(h).savedScopes.webdav,scope);assert.doesNotMatch(json(sync(h)),/demo-sync/);
 h.w.syncPrototype.setScenario('partial');h.click('[data-sx-protection-demo=encrypted]');h.click('[data-sx=check]');h.click('[data-sx-protection-demo=plaintext]');result=await finished(h);assert.equal(result.failed.length,1);h.click('[data-sx=retry]');result=await finished(h);assert.equal(result.failed.length,0);assert.equal(result.protection.encrypted,true);assert.equal(sync(h).savedScopes.webdav,scope);
});

test('导出默认不加密，成功只走注册Toast和铃铛；回执跨路由保留保存位置',async t=>{
 const h=await setup(t);exportPreview(h);assert.equal(h.document.querySelector('#sxEncrypt').checked,false);assert.equal(h.document.querySelector('#sxPassFields').hidden,true);h.click('#modalConfirm');
 assert.equal(sync(h).lastExport.encrypted,false);assert.equal(h.document.querySelector('#sxPanel>.sx-result'),null);assert.equal(sync(h).dialogKind,'');
 const notices=h.w.eval('notifications.filter(n=>n.eventId==="sync.export.complete")');assert.equal(notices.length,1);assert.equal(notices[0].read,true);assert.equal(notices[0].resolved,true);
 const records=h.w.prototypeNotificationPublisher.deliveries();assert.equal(records[0].eventId,'sync.export.complete');assert.deepEqual([...records[0].delivery],['toast','center','log']);assert.doesNotMatch(json(records),/默认导出目录|Users|credentialRef|demo-sync/);
 h.w.setRoute('models');h.click('#notificationBtn');h.click('[data-notification="'+notices[0].id+'"]');assert.equal(h.document.querySelector('#modalTitle').textContent,'导出结果');assert.match(h.document.querySelector('.sx-dialog').textContent,/默认导出目录/);assert.equal(h.document.querySelector('[data-sx=open-export]')!==null,true);h.click('#modalCancel');
});

test('手动开启加密才显示并校验口令，再次导出仍默认不加密',async t=>{
 const h=await setup(t);exportPreview(h);h.change('#sxEncrypt',true);assert.equal(h.document.querySelector('#sxPassFields').hidden,false);h.click('#modalConfirm');assert.match(h.document.querySelector('#sxError').textContent,/至少 8 位/);assert.equal(h.w.prototypeNotificationPublisher.deliveries().length,0);
 h.input('#sxPass','fixture-export-pass');h.input('#sxPassAgain','fixture-export-pass');h.click('#modalConfirm');assert.equal(sync(h).lastExport.encrypted,true);assert.doesNotMatch(json(h.w.prototypeNotificationPublisher.deliveries()),/fixture-export-pass/);exportPreview(h);assert.equal(h.document.querySelector('#sxEncrypt').checked,false);h.click('#modalCancel');
});

test('导出失败有注册原因，保留上次成功包；其它成功不会误解决这次失败',async t=>{
 const h=await setup(t);exportPreview(h);h.click('#modalConfirm');const before=json(sync(h).lastExport);h.w.syncPrototype.setScenario('backup-fail');exportPreview(h);h.click('#modalConfirm');
 assert.equal(json(sync(h).lastExport),before);assert.equal(h.document.querySelector('#sxPanel>.sx-result'),null);const failed=h.w.eval('notifications.find(n=>n.eventId==="sync.export.failed")');assert.equal(failed.resolved,false);
 h.w.syncPrototype.setScenario('normal');exportPreview(h);h.click('#modalConfirm');assert.equal(failed.resolved,false);assert.equal(h.w.prototypeNotificationPublisher.deliveries().some(e=>e.eventId==='sync.export.failed'),true);
});

test('公共发布器拒绝未知/计划事件和敏感自由文本，同一操作重复终态只投递一次',async t=>{
 const h=await setup(t),p=h.w.prototypeNotificationPublisher;
 assert.throws(()=>p.publish('sync.unknown',{operationId:'900'}));assert.throws(()=>p.publish('sync.conflict',{operationId:'900'}));assert.throws(()=>p.publish('sync.complete',{operationId:'900',detail:'secret'}));assert.throws(()=>p.publish('sync.complete',{operationId:'key-value'}));
 p.publish('sync.complete',{operationId:'900',total:1});const count=p.deliveries().length;p.publish('sync.complete',{operationId:'900',total:1});assert.equal(p.deliveries().length,count);assert.equal(h.w.eval('notifications.filter(n=>n.id==="sync-demo:900").length'),1);
});

test('导入和WebDAV开始只记inline/log；失败重试更新同一条且不替换其它任务',async t=>{
 const h=await setup(t);h.w.prototypeNotificationPublisher.publish('component.demo.task',{taskId:'a'});const task=h.document.querySelector('#taskHost').textContent;
 h.w.syncPrototype.open({tab:'webdav',scope:['skill:review','mcp:docs']});h.w.syncPrototype.setScenario('partial');h.click('[data-sx=check]');const start=h.w.prototypeNotificationPublisher.deliveries()[0];assert.equal(start.eventId,'sync.started');assert.deepEqual([...start.delivery],['inline','log']);
 await finished(h);const fail=h.w.eval('notifications.find(n=>n.eventId==="sync.failed")');assert.equal(h.document.querySelector('#sxPanel>.sx-result'),null);assert.equal(fail.resolved,false);assert.equal(h.document.querySelector('#taskHost').textContent,task);
 h.click('[data-sx=retry]');await finished(h);assert.equal(fail.resolved,true);assert.equal(fail.eventId,'sync.complete');assert.equal(h.w.eval('notifications.filter(n=>n.id==="'+fail.id+'").length'),1);assert.equal(h.document.querySelector('#taskHost').textContent,task);
 h.w.syncPrototype.setScenario('normal');chooseImport(h);h.click('#modalConfirm');await finished(h);const events=h.w.prototypeNotificationPublisher.deliveries().map(x=>x.eventId);assert.ok(events.includes('sync.import.started'));assert.ok(events.includes('sync.import.complete'));
});

test('删除后的同操作重试不复活通知，新的明确同步仍能产生新回执',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',scope:['skill:review']});h.w.syncPrototype.setScenario('network-error');h.click('[data-sx=check]');await finished(h);const notice=h.w.eval('notifications.find(n=>n.eventId==="sync.failed")');notice.deleted=true;
 h.click('[data-sx=retry]');await finished(h);assert.equal(notice.deleted,true);assert.equal(h.w.eval('notifications.filter(n=>n.id==="'+notice.id+'").length'),1);
 h.w.syncPrototype.setScenario('normal');h.click('[data-sx=check]');await finished(h);assert.ok(h.w.eval('notifications.some(n=>n.eventId==="sync.complete"&&!n.deleted)'));
});

test('连接测试成功/失败及保存均有注册投递，字段错误不造通知或泄露连接凭据',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',action:'connection'});h.input('#sxDavURL','broken');h.click('[data-sx=test-connection]');assert.equal(h.w.prototypeNotificationPublisher.deliveries().length,0);
 h.input('#sxDavURL','https://fixture.example.invalid');h.input('#sxDavPassword','fixture-sensitive-value');h.click('[data-sx=test-connection]');assert.equal(h.w.prototypeNotificationPublisher.deliveries()[0].eventId,'sync.connected');
 h.w.syncPrototype.setScenario('network-error');h.click('[data-sx=test-connection]');assert.equal(h.w.prototypeNotificationPublisher.deliveries()[0].eventId,'sync.connection.failed');h.click('#modalConfirm');assert.equal(h.w.prototypeNotificationPublisher.deliveries()[0].eventId,'sync.connection.saved');
 const failed=h.w.eval('notifications.find(n=>n.eventId==="sync.connection.failed")');assert.equal(failed.resolved,false);
 h.w.syncPrototype.open({tab:'webdav',action:'connection'});h.w.syncPrototype.setScenario('normal');h.click('[data-sx=test-connection]');assert.equal(failed.eventId,'sync.connected');assert.equal(failed.resolved,true);h.click('#modalCancel');
 assert.doesNotMatch(json(h.w.prototypeNotificationPublisher.deliveries()),/fixture.example|fixture-sensitive/);
});

test('本轮同步业务不直接写通知或调用底层Toast，启用事件均有owner与公共发布器',()=>{
 const source=fs.readFileSync(path.join(__dirname,'sync-config.js'),'utf8');assert.doesNotMatch(source,/notifications\.(unshift|push)|renderNotifications\(|(?:window\.)?toast\??\.?(?:\(|call)|showTask\(/);
 const config=require('./maintenance-events.js'),events=config.events.filter(e=>e.consumer==='syncPrototype');assert.equal(events.length,12);
 for(const e of events){assert.equal(e.implemented,'prototype');assert.equal(e.publisher,'prototypeNotificationPublisher');assert.equal(e.owner,'configuration-sync');assert.equal(e.toastDurationMs,8000);assert.ok(e.entry);}
});
