/* 唯一总原型的内存 DOM / 状态检查；不替代浏览器像素或真实文件验收。 */
const test=require('node:test'),assert=require('node:assert/strict');
const fs=require('node:fs'),path=require('node:path');
const {make}=require('./component-topic-harness.cjs');
const json=x=>JSON.stringify(x);
const model=h=>h.w.modelPrototype.state();
const sync=h=>h.w.syncPrototype.state();
const first=policy=>policy.providers.find(p=>p.id==='newapi').models[0];
async function setup(t){
 const h=await make('#sync?tab=file');
 h.w.confirm=()=>{throw Error('不允许新增确认步骤');};
 t.after(()=>{try{assert.equal(h.network.length,0);assert.deepEqual(h.errors,[]);}finally{h.dispose();}});
 return h;
}
function patch(h,alias){assert.equal(h.w.modelPrototype.commitModelPatches('newapi',[{key:'newapi::0',naming:{alias}}]),'');}
function exportPreview(h,options={}){h.w.syncPrototype.open({tab:'file',scope:['channel:newapi'],action:'export',...options});}
function finishExport(h){h.change('#sxEncrypt',false);h.click('#modalConfirm');return sync(h).lastExport;}
function importPreview(h,sample='keyless',scope=['channel:newapi']){
 h.w.syncPrototype.setImportSample(sample);
 h.w.syncPrototype.open({tab:'file',scope,action:'import'});
 assert.equal(h.document.querySelector('#modalConfirm').disabled,true);
 h.click('[data-sx=choose-import-file]');
 if(sample==='encrypted')h.input('#sxUnlock','demo-sync');
 h.click('#modalConfirm');
 assert.equal(sync(h).comparison.mode,'import');
}
function webdavPreview(h){
 h.w.syncPrototype.open({tab:'webdav',scope:['channel:newapi']});
 h.click('[data-sx=save-scope]');h.click('[data-sx=check]');
 h.click('[data-sx=compare-expand][data-id=channels]');
}

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
 assert.match(h.document.querySelector('.sx-note').textContent,/可加密/);
 h.click('[data-sx=export]');assert.equal(h.document.querySelector('#sxEncrypt').checked,true);
 h.click('#modalConfirm');assert.match(h.document.querySelector('#sxError').textContent,/至少 8 位/);assert.equal(sync(h).lastExport,null);
 const file=finishExport(h);assert.equal(file.encryptionRequired,false);assert.equal(file.encrypted,false);
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

async function saveDraft(h){
 h.w.setRoute('models');h.click('#mpPolicySave');h.click('#modalConfirm');
 assert.equal(model(h).operation.status,'running');
 for(let i=0;i<120&&model(h).operation?.status==='running';i++)await h.wait(50);
 assert.equal(model(h).operation.status,'written');
}
for(const sample of ['keyless','encrypted'])test(sample+' 实际导入 → 当前内存草稿 → 原保存预览确认 → mock 已保存文件',async t=>{
 const h=await setup(t),saved=json(model(h).saved),records=json(model(h).records);
 assert.equal(h.w.modelPrototype.commitModelPatches('newapi',[{key:'newapi::10',naming:{alias:'keep-other-local'}}]),'');
 importPreview(h,sample);
 h.click('[data-sx=compare-expand][data-id=channels]');h.click('[data-sx=compare-detail][data-id="channel:newapi"]');assert.equal(h.document.querySelectorAll('#sxCompareDetail tbody tr').length,15);h.click('[data-sx=compare-detail-close]');
 h.click('[data-sx=compare-expand][data-id=templates]');h.click('[data-sx=compare-detail][data-id="template:gpt"]');assert.equal(h.document.querySelectorAll('#sxCompareDetail tbody tr').length,10);h.click('[data-sx=compare-detail-close]');
 if(sample==='encrypted')h.change('[data-sx-compare-credential="channel:newapi"]',true);
 h.click('#modalConfirm');
 assert.equal(sync(h).dialogKind,'');assert.equal(sync(h).channelAdoption.status,'draft');assert.equal(sync(h).channelAdoption.target,'model-channel-memory-draft');
 assert.equal(sync(h).channelAdoption.source,'file');assert.equal(first(model(h).draft).naming.alias,'primary-model');
 const incoming=model(h).draft.providers.find(p=>p.id==='newapi'),original=model(h).saved.providers.find(p=>p.id==='newapi');
 assert.equal(incoming.refs.gpt,'gpt:4');assert.equal(incoming.nodes[0].url,'https://newapi.example.com/sync/v1');assert.equal(incoming.models.filter(m=>m.selected).length,19);
 assert.equal(incoming.models.some(m=>m.id==='gpt-5-mini'),false);assert.equal(incoming.models.filter(m=>m.id.endsWith('--sync-demo')).length,3);
 assert.equal(incoming.models.find(m=>m.key==='newapi::10').naming.alias,'keep-other-local');
 assert.equal(incoming.connection.credentialRef===original.connection.credentialRef,sample==='keyless');
 assert.equal(json(model(h).saved),saved);assert.equal(json(model(h).records),records);assert.equal(model(h).operation,null);
 assert.equal(model(h).templates.find(t=>t.id==='gpt').main,1);assert.ok(model(h).templates.find(t=>t.id==='gpt').versions.some(v=>v.n===4));
 assert.match(sync(h).result.detail,/渠道已加入草稿/);assert.doesNotMatch(sync(h).result.detail,/已写入|写入与读回/);
 h.w.setRoute('models');h.click('#mpPolicySave');assert.equal(h.document.querySelector('#modalTitle').textContent,'保存预览');assert.equal(json(model(h).saved),saved);
 h.click('#modalCancel');assert.equal(json(model(h).saved),saved);assert.equal(first(model(h).draft).naming.alias,'primary-model');
 await saveDraft(h);assert.equal(first(model(h).saved).naming.alias,'primary-model');assert.notEqual(json(model(h).saved),saved);
 exportPreview(h);assert.equal(first(finishExport(h).snapshot.modelChannels).naming.alias,'primary-model');assert.doesNotMatch(json(sync(h)),/demo-sync-newapi-key/);
});

test('WebDAV 采用远端渠道真实进入当前草稿，保存确认前不改文件；保留本机/跳过不采用',async t=>{
 const h=await setup(t),saved=json(model(h).saved);
 webdavPreview(h);h.change('[data-sx-diff-choice="channel:newapi"]','remote');h.click('#modalConfirm');
 assert.equal(sync(h).dialogKind,'');assert.equal(sync(h).channelAdoption.source,'webdav');assert.equal(sync(h).channelAdoption.status,'draft');
 assert.equal(first(model(h).draft).naming.alias,'primary-model');assert.equal(json(model(h).saved),saved);assert.equal(model(h).operation,null);
 await saveDraft(h);assert.equal(first(model(h).saved).naming.alias,'primary-model');
 const draft=json(model(h).draft),savedAfter=json(model(h).saved);
 for(const direction of ['local','skip']){
  h.w.syncPrototype.open({tab:'webdav',action:'check'});h.click('[data-sx=compare-expand][data-id=channels]');
  h.change('[data-sx-diff-choice="channel:newapi"]',direction);h.click('#modalConfirm');
  assert.equal(sync(h).dialogKind,'');assert.equal(sync(h).channelAdoption,null);assert.equal(sync(h).lastComparison.items.some(x=>x.group==='channels'),false);
  assert.equal(json(model(h).saved),savedAfter);assert.equal(json(model(h).draft),draft);
 }
});

test('连接字段直接进入编辑后，导入可合并独立名称，凭据重叠仍保留输入',async t=>{
 const h=await setup(t);h.w.setRoute('models');h.click('[data-mp=edit-provider][data-id=newapi]');h.input('#mpChannelInline-name','independent-local-name');const saved=json(model(h).saved);
 importPreview(h);h.click('#modalConfirm');assert.equal(sync(h).dialogKind,'');assert.equal(model(h).draft.providers[0].name,'independent-local-name');assert.equal(first(model(h).draft).naming.alias,'primary-model');assert.equal(json(model(h).saved),saved);assert.equal(model(h).operation,null);
 h.w.setRoute('models');h.click('[data-mp=back-channels]');h.click('[data-mp=discard]');h.click('#modalConfirm');h.click('[data-mp=edit-provider][data-id=newapi]');h.input('#mpChannelInline-key','demo-local-edit-protected');const draft=json(model(h).draft);
 importPreview(h,'encrypted');h.change('[data-sx-compare-credential="channel:newapi"]',true);h.click('#modalConfirm');assert.match(h.document.querySelector('#sxError').textContent,/重叠草稿/);assert.equal(json(model(h).draft),draft);assert.equal(json(model(h).saved),saved);assert.equal(model(h).operation,null);
});

for(const kind of ['row','alias'])test(kind+' 未确认输入受到保护，采用失败不改草稿/文件/模版；处理输入后可重新采用',async t=>{
 const h=await setup(t);h.w.setRoute('models');h.click('[data-mp=edit-provider][data-id=newapi]');
 if(kind==='alias')h.input('[data-model-alias="newapi::0"]','unconfirmed-alias');
 if(kind==='row'){h.click('[data-mp=expand-model][data-id="newapi::0"]');h.change('[data-model-quick="newapi::0"] [data-quick-mode=context]','custom');}
 const before=model(h),saved=json(before.saved),draft=json(before.draft),templates=json(before.templates),connections=json(before.channelConnections),edits=json(before.modelEdits);
 importPreview(h);h.click('#modalConfirm');assert.match(h.document.querySelector('#sxError').textContent,/未确认/);assert.equal(sync(h).dialogKind,'import');
 assert.equal(json(model(h).saved),saved);assert.equal(json(model(h).draft),draft);assert.equal(json(model(h).templates),templates);assert.equal(json(model(h).channelConnections),connections);assert.equal(json(model(h).modelEdits),edits);
 assert.equal(model(h).operation,null);assert.equal(sync(h).channelAdoption,null);
 h.click('#modalCancel');h.w.setRoute('models');
 if(kind==='row')h.click('[data-mp=reset-model-row][data-id="newapi::0"]');
 if(kind==='alias'){assert.equal(h.document.querySelector('[data-model-alias="newapi::0"]').value,'unconfirmed-alias');h.input('[data-model-alias="newapi::0"]','');}
 importPreview(h);h.click('#modalConfirm');assert.equal(sync(h).dialogKind,'');assert.equal(first(model(h).draft).naming.alias,'primary-model');
});

test('已加入草稿的重叠字段保留，比较期间的新修改使旧应用过期',async t=>{
 const h=await setup(t);patch(h,'local-overlap');const draft=json(model(h).draft),saved=json(model(h).saved),templates=json(model(h).templates);
 importPreview(h);h.click('#modalConfirm');assert.match(h.document.querySelector('#sxError').textContent,/重叠草稿/);assert.equal(json(model(h).draft),draft);assert.equal(json(model(h).saved),saved);assert.equal(json(model(h).templates),templates);
 h.click('#modalCancel');assert.equal(h.w.modelPrototype.commitModelPatches('newapi',[{key:'newapi::0',naming:{}}]),'');
 importPreview(h);patch(h,'late-input');const late=json(model(h).draft);h.click('#modalConfirm');
 assert.match(h.document.querySelector('#sxError').textContent,/渠道已变化/);assert.equal(json(model(h).draft),late);assert.equal(json(model(h).saved),saved);assert.equal(json(model(h).templates),templates);
});

test('未确认的无关渠道输入及草稿保留；模版单独接纳不自动采用渠道',async t=>{
 const h=await setup(t),saved=json(model(h).saved);h.w.setRoute('models');h.click('[data-mp=edit-provider][data-id=a6api]');h.input('#mpChannelInline-name','keep-unrelated-input');
 const connections=json(model(h).channelConnections);importPreview(h);h.click('#modalConfirm');
 assert.equal(sync(h).dialogKind,'');assert.equal(first(model(h).draft).naming.alias,'primary-model');assert.equal(model(h).channelConnections.a6api.name,'keep-unrelated-input');assert.equal(json(model(h).channelConnections),connections);assert.equal(json(model(h).saved),saved);
 h.w.setRoute('models');h.click('[data-mp=back-channels]');h.click('[data-mp=discard]');h.click('#modalConfirm');const draft=json(model(h).draft);
 importPreview(h,'keyless',['template:gpt']);h.click('#modalConfirm');assert.equal(sync(h).dialogKind,'');assert.equal(json(model(h).draft),draft);assert.equal(json(model(h).saved),saved);assert.equal(model(h).templates.find(t=>t.id==='gpt').main,1);
});

for(const mapping of ['new','rebind'])test('身份映射 '+mapping+' 新增渠道只进入草稿，不复用本机 Key',async t=>{
 const h=await setup(t),saved=json(model(h).saved);importPreview(h,'keyless',['channel:legacy']);h.click('[data-sx=compare-expand][data-id=channels]');h.change('[data-sx-import-choice="channel:legacy"]',mapping);h.click('#modalConfirm');
 const p=model(h).draft.providers.find(p=>p.id===(mapping==='new'?'legacy-imported':'legacy'));assert.ok(p);assert.equal(p.connection.keyConfigured,false);assert.equal(p.connection.credentialRef,undefined);assert.equal(p.enabled,false);assert.equal(p.nodes[0].url,'https://new-private.example.com/v1');assert.equal(json(model(h).saved),saved);assert.equal(model(h).operation,null);
});

for(const scene of ['backup-fail','partial'])test(scene+' 应用失败不先暂存渠道或模版版本，保留原内存与文件',async t=>{
 const h=await setup(t),draft=json(model(h).draft),saved=json(model(h).saved),templates=json(model(h).templates);h.w.syncPrototype.setScenario(scene);importPreview(h);h.click('#modalConfirm');
 assert.equal(sync(h).channelAdoption,null);assert.equal(json(model(h).draft),draft);assert.equal(json(model(h).saved),saved);assert.equal(json(model(h).templates),templates);assert.equal(model(h).operation,null);
});

test('保存失败保留已导入草稿与原文件，可通过原保存确认重试',async t=>{
 const h=await setup(t);h.w.modelPrototype.scene('backup-fail');const saved=json(model(h).saved);
 importPreview(h);h.click('#modalConfirm');const draft=json(model(h).draft);h.w.setRoute('models');h.click('#mpPolicySave');h.click('#modalConfirm');
 for(let i=0;i<100&&model(h).operation.status==='running';i++)await h.wait(50);
 assert.equal(model(h).operation.status,'failed');assert.equal(json(model(h).saved),saved);assert.equal(json(model(h).draft),draft);
 h.click('#mpPolicySave');h.click('#modalConfirm');assert.equal(model(h).operation.status,'running');assert.equal(json(model(h).saved),saved);assert.equal(json(model(h).draft),draft);
});

test('桥接只接受固定 fixture；未知渠道、重复身份、缺精确依赖及保存忙碌态均不写状态',async t=>{
 const h=await setup(t),baseline=model(h),draft=json(baseline.draft),saved=json(baseline.saved),templates=json(baseline.templates);
 const request={contract:'prototype-sync-fixture/1',source:'file',baseline,channels:[],templates:[]};
 for(const channels of [[null],[{id:'channel:native',credential:false}],[{id:'channel:newapi',credential:false},{id:'channel:newapi',credential:false}],[{id:'channel:newapi',credential:false}]]){
  assert.equal(h.w.modelPrototype.stageSyncChannels({...request,channels}).ok,false);
  assert.equal(json(model(h).draft),draft);assert.equal(json(model(h).saved),saved);assert.equal(json(model(h).templates),templates);
 }
 patch(h,'save-in-progress');h.w.setRoute('models');h.click('#mpPolicySave');h.click('#modalConfirm');
 const during=json(model(h).draft);const response=h.w.modelPrototype.stageSyncChannels({...request,channels:[{id:'channel:newapi',credential:false}],templates:['gpt']});
 assert.equal(response.ok,false);assert.match(response.message,/正在保存/);assert.equal(json(model(h).draft),during);assert.equal(json(model(h).templates),templates);
});

test('重复接纳模版不重复新增版本，人工模版缓冲保留且不触发渠道采用',async t=>{
 const h=await setup(t),saved=json(model(h).saved),draft=json(model(h).draft);h.w.setRoute('models');h.click('[data-mp=tab][data-tab=templates]');h.click('[data-mp=open-template-editor][data-id=gpt]');h.input('#mpTemplateDescription','keep-template-buffer');const buffers=json(model(h).buffers);
 for(let i=0;i<2;i++){importPreview(h,'keyless',['template:gpt']);h.click('#modalConfirm');assert.equal(sync(h).dialogKind,'');assert.equal(sync(h).channelAdoption,null);}
 assert.equal(model(h).templates.find(t=>t.id==='gpt').versions.filter(v=>v.n===4).length,1);assert.equal(model(h).templates.find(t=>t.id==='gpt').main,1);assert.equal(json(model(h).buffers),buffers);assert.equal(json(model(h).draft),draft);assert.equal(json(model(h).saved),saved);
});

test('非渠道范围仍可比较应用；未选择或损坏文件不创建渠道候选，不影响模型状态',async t=>{
 const h=await setup(t),saved=json(model(h).saved),draft=json(model(h).draft);
 importPreview(h,'legacy',['preferences:desktop']);h.click('#modalConfirm');
 assert.equal(sync(h).dialogKind,'');assert.equal(sync(h).lastComparison.items[0].id,'preferences:desktop');assert.equal(sync(h).channelAdoption,null);
 h.w.syncPrototype.setImportSample('corrupt');h.w.syncPrototype.open({tab:'file',action:'import'});
 h.click('[data-sx=choose-import-file]');h.click('#modalConfirm');
 assert.match(h.document.querySelector('#sxError').textContent,/完整性校验失败/);assert.equal(sync(h).channelAdoption,null);
 assert.equal(json(model(h).saved),saved);assert.equal(json(model(h).draft),draft);assert.equal(model(h).operation,null);
});

test('同步原型保持 mock 边界，不增加系统确认、真实网络、文件或宿主命令调用',()=>{
 const code=fs.readFileSync(path.join(__dirname,'sync-config.js'),'utf8');
 for(const token of ['fetch(', 'XMLHttpRequest', 'WebSocket(', 'showOpenFilePicker(', 'showSaveFilePicker(', 'require(', 'localStorage.setItem(', 'sessionStorage.setItem(', '__TAURI__', 'invoke(', 'confirm('])assert.equal(code.includes(token),false,token);
 assert.doesNotMatch(code,/所选凭据需加密|encryptionRequired:count>0/);
});
