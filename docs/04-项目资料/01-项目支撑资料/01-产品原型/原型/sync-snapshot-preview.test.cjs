/* 总原型的内存交互回归；不读取真实配置、不连接 WebDAV，不作为实屏或文件写入验收。 */
const test=require('node:test');
const assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');

const plain=value=>JSON.parse(JSON.stringify(value));
const sync=h=>plain(h.w.syncPrototype.state());
const model=h=>plain(h.w.modelPrototype.state());
const scope=['skill:review','mcp:docs'];
const notices=h=>plain(h.w.eval('notifications'));
const deliveries=h=>plain(h.w.prototypeNotificationPublisher.deliveries());
const position=h=>({hash:h.w.location.hash,route:h.document.documentElement.dataset.route,scroll:h.document.querySelector('.main').scrollTop});

async function setup(t){
 const h=await make('#overview');
 h.w.confirm=()=>{throw Error('同步预览不得追加原生确认');};
 t.after(()=>{try{
  assert.deepEqual(h.network,[]);
  assert.deepEqual(h.errors,[]);
  assert.ok(h.writes.every(([key])=>['ocx-theme','ocx-route'].includes(key)),'仅允许壳层既有缓存，不持久化配置或凭据');
 }finally{h.dispose();}});
 assert.deepEqual(sync(h).backups,[]);
 return h;
}

function business(h){
 const s=sync(h),m=model(h);
 return {running:s.running,versions:s.versions,snapshots:s.snapshots,backups:s.backups,
  result:s.result,lastRun:s.lastRun,serial:s.serial,recentActivity:s.recentActivity,
  demoObjects:s.demoObjects,channelAdoption:s.channelAdoption,
  saved:m.saved,draft:m.draft,templates:m.templates,operation:m.operation,
  notifications:notices(h),deliveries:deliveries(h)};
}

async function webdav(h,scene='normal'){
 h.w.syncPrototype.open({tab:'webdav',scope});
 h.w.syncPrototype.setScenario(scene);
 assert.equal(sync(h).scenario,scene,'场景必须已登记');
 h.w.setRoute('overview');
 await h.settle();
 h.w.renderWebdav('ready');
 h.document.querySelector('.main').scrollTop=247;
 return {before:business(h),position:position(h)};
}

function openPreview(h){
 h.click('#syncActions [data-prototype-action=sync]');
 assert.equal(h.document.querySelector('#modalTitle').textContent,'同步预览');
 assert.equal(h.document.querySelector('#modalConfirm').textContent,'确认更新');
 const s=sync(h),p=s.preview;
 assert.ok(p,'点击同步先生成预览');
 assert.ok(['local','remote'].includes(p.source));
 assert.ok(p.snapshot,'预览必须标明来源快照');
 assert.ok(s.snapshots.local&&s.snapshots.remote,'快照元数据与对象内容版本分别可读');
 assert.deepEqual(p.snapshot,s.snapshots[p.source],'预览元数据来自所选来源');
 assert.ok(Date.parse(p.snapshot.updatedAt)>=Date.parse(s.snapshots[p.source==='local'?'remote':'local'].updatedAt),'整份快照的较新来源决定方向');
 assert.ok(Array.isArray(p.rows));
 assert.deepEqual(p.rows.map(row=>row.id).sort(),[...scope].sort());
 assert.equal(s.running,false);
 return p;
}

function dismiss(h,action){
 if(action==='取消')h.click('#modalCancel');
 else if(action==='Escape')h.document.dispatchEvent(new h.w.KeyboardEvent('keydown',{key:'Escape',bubbles:true}));
 else{
  const buttons=[...h.document.querySelectorAll('#modal .modal-head button')];
  const close=buttons.find(button=>/关闭|close/i.test([button.getAttribute('aria-label'),button.title,button.textContent].join(' '))||['×','✕','✖'].includes(button.textContent.trim()));
  assert.ok(close,'预览右上角必须有可识别的关闭按钮');
  assert.equal(close.disabled,false);
  close.click();
 }
 assert.equal(sync(h).dialogKind,'');
}

async function finished(h){
 for(let i=0;i<100&&sync(h).running;i++)await h.wait(50);
 const s=sync(h);
 assert.equal(s.running,false,'确认的操作应完成');
 assert.ok(s.result,'必须有本次操作回执');
 return s.result;
}

async function confirm(h){
 assert.equal(h.document.querySelector('#modalConfirm').disabled,false);
 h.click('#modalConfirm');
 assert.equal(sync(h).running,true,'只有确认后才启动执行');
 return finished(h);
}

function outsideScope(before,after,ids){
 const selected=new Set(ids);
 for(const side of ['local','remote']){
  const keys=new Set([...Object.keys(before.versions[side]),...Object.keys(after.versions[side])]);
  for(const id of keys)if(!selected.has(id))assert.deepEqual(after.versions[side][id],before.versions[side][id],side+' 范围外对象 '+id);
 }
 for(const id of new Set([...Object.keys(before.demoObjects),...Object.keys(after.demoObjects)])){
  if(!selected.has(id))assert.deepEqual(after.demoObjects[id],before.demoObjects[id],'范围外文件内容 '+id);
 }
}

function singleSource(before,after,preview,result){
 const source=preview.source,destination=source==='remote'?'local':'remote';
 const direction=source==='remote'?'download':'upload';
 const changed=scope.filter(id=>before.versions.local[id].digest!==before.versions.remote[id].digest);
 assert.ok(changed.length,'场景必须存在真实差异');
 assert.equal(result.failed.length,0);
 assert.deepEqual(result.items.map(row=>row.id).sort(),[...scope].sort());
 assert.equal(result.downloaded,direction==='download'?changed.length:0);
 assert.equal(result.uploaded,direction==='upload'?changed.length:0);
 assert.deepEqual([...new Set(result.items.filter(row=>row.status==='completed').map(row=>row.direction))],[direction]);
 assert.ok(preview.rows.every(row=>row.direction==='skip'||row.direction===direction),'预览也只能有一个来源方向');
 for(const id of scope)assert.deepEqual(after.versions[destination][id],before.versions[source][id],id+' 使用所选快照，即使对象自身时间方向不同');
 assert.deepEqual(after.versions[source],before.versions[source],'来源对象内容保持不变');
 outsideScope(before,after,scope);
}

function resultNotice(h,result,eventId){
 const items=notices(h).filter(item=>item.eventId===eventId);
 assert.equal(items.length,1,'确认操作在通知中心保留一个结果');
 const mixed=/下载更新\s*\d+\s*项.*上传\s*\d+\s*项/;
 assert.doesNotMatch(items[0].title+' '+items[0].detail,mixed);
 assert.doesNotMatch(result.title+' '+result.detail,mixed);
 assert.doesNotMatch(h.document.querySelector('#toastEl').textContent,mixed);
 const records=deliveries(h);
 assert.equal(records.filter(item=>item.eventId===eventId).length,1);
 return records.find(item=>item.eventId===eventId);
}

function backupBeforeWrite(before,after,result,ids){
 assert.equal(after.backups.length,before.backups.length+1,'一次确认只生成一份备份');
 assert.deepEqual(after.backups.slice(0,-1),before.backups,'既有备份保留');
 const backup=after.backups.at(-1);
 assert.equal(backup.operationId,result.operationId);
 assert.deepEqual(backup.items.map(item=>item.id).sort(),[...ids].sort());
 for(const item of backup.items){
  assert.deepEqual(item.version,before.versions.local[item.id],'备份保存写入前的对象版本');
  assert.deepEqual(item.content,before.demoObjects[item.id],'备份保存写入前的文件内容');
 }
 const selected=new Set(ids);
 assert.deepEqual(backup.channels,before.saved.providers.filter(item=>selected.has('channel:'+item.id)),'渠道备份取写入前的 saved');
}

for(const action of ['取消','右上关闭','Escape'])test('概览同步预览：'+action+' 不执行、不备份、不通知，保留路由与滚动',async t=>{
 const h=await setup(t),context=await webdav(h,'download');
 const p=openPreview(h);assert.equal(p.source,'remote');
 assert.equal(h.document.querySelector('#sxPreviewBackup').checked,true);
 assert.deepEqual(position(h),context.position);
 assert.deepEqual(business(h),context.before,'打开预览没有业务副作用');
 dismiss(h,action);
 // 超过正常执行的计时，捕获取消后残留任务或延迟通知。
 await h.wait(750);
 assert.deepEqual(business(h),context.before);
 assert.deepEqual(position(h),context.position);
});

for(const scene of ['download','upload','normal'])test(scene+'：确认后整次只采用一个快照来源，范围外不变',async t=>{
 const h=await setup(t),context=await webdav(h,scene),p=openPreview(h);
 if(scene==='download')assert.equal(p.source,'remote');
 if(scene==='upload')assert.equal(p.source,'local');
 if(scene==='normal'){
  assert.ok(Date.parse(context.before.versions.remote['skill:review'].updatedAt)>Date.parse(context.before.versions.local['skill:review'].updatedAt));
  assert.ok(Date.parse(context.before.versions.local['mcp:docs'].updatedAt)>Date.parse(context.before.versions.remote['mcp:docs'].updatedAt));
 }
 const backup=h.document.querySelector('#sxPreviewBackup');
 if(p.source==='remote')assert.equal(backup.checked,true);
 else assert.equal(backup,null,'上传不提供本地更新前备份');
 assert.deepEqual(business(h),context.before);
 assert.deepEqual(position(h),context.position);
 const result=await confirm(h),after=sync(h);
 singleSource(context.before,after,p,result);
 if(p.source==='remote')backupBeforeWrite(context.before,after,result,scope);
 else assert.deepEqual(after.backups,context.before.backups);
 assert.deepEqual(position(h),context.position);
 assert.deepEqual(model(h).saved,context.before.saved,'扩展范围不写渠道');
 assert.equal(deliveries(h).filter(item=>item.eventId==='sync.started').length,1);
 resultNotice(h,result,'sync.complete');
 if(scene==='download'){
  const completed=business(h);h.w.renderWebdav('ready');openPreview(h);dismiss(h,'取消');
  assert.deepEqual(business(h),completed,'后续取消保留已有备份和结果通知');
 }
});

test('same：仍展示同步预览，禁用无需更新的确认且不产生执行或备份',async t=>{
 const h=await setup(t),context=await webdav(h,'same');openPreview(h);
 assert.equal(h.document.querySelector('#modalConfirm').disabled,true);
 assert.match(h.document.querySelector('#modalBody').textContent,/无需更新|已一致|内容一致/);
 h.click('#modalConfirm');await h.wait(750);
 assert.deepEqual(business(h),context.before);
 assert.deepEqual(position(h),context.position);
 dismiss(h,'取消');
 assert.deepEqual(business(h),context.before);
});

test('下载可取消备份选择，确认仍更新；再次预览没有重复变更',async t=>{
 const h=await setup(t),context=await webdav(h,'download'),p=openPreview(h);
 h.change('#sxPreviewBackup',false);
 assert.deepEqual(business(h),context.before);
 const result=await confirm(h),after=sync(h);
 singleSource(context.before,after,p,result);
 assert.deepEqual(after.backups,[]);
 resultNotice(h,result,'sync.complete');
 const beforeAgain=business(h);h.w.renderWebdav('ready');openPreview(h);
 assert.equal(h.document.querySelector('#modalConfirm').disabled,true);
 h.click('#modalConfirm');await h.wait(750);
 assert.deepEqual(business(h),beforeAgain);
 dismiss(h,'取消');
});

async function importPreview(h,ids=['channel:newapi']){
 h.w.syncPrototype.setImportSample('keyless');
 h.document.querySelector('.main').scrollTop=319;
 const context={before:business(h),position:position(h)};
 h.w.syncPrototype.open({tab:'file',scope:ids,action:'import',stayOnPage:true});
 const modal=h.document.querySelector('#modal');
 assert.equal(h.document.querySelector('#modalTitle').textContent,'导入配置');
 assert.equal(h.document.querySelector('#modalConfirm').disabled,true);
 assert.deepEqual(business(h),context.before);
 h.click('[data-sx=choose-import-file]');
 assert.equal(h.document.querySelector('#modal'),modal,'读文件后在同一窗口展示变更');
 assert.equal(h.document.querySelector('#modalTitle').textContent,'导入配置');
 assert.equal(h.document.querySelector('#sxImportBackup').checked,true);
 assert.equal(h.document.querySelector('#modalConfirm').disabled,false);
 const body=h.document.querySelector('#modalBody').cloneNode(true);
 body.querySelector('#sxImportBackup').closest('label').remove();
 body.querySelectorAll('[hidden],.annotation-source').forEach(node=>node.remove());
 assert.match(body.textContent,/NewApi/);
 assert.match(body.textContent,/变更|差异|更新|替换|新增/,'展示变更条目，而不只是文件名和范围');
 assert.deepEqual(business(h),context.before,'选择文件只预览，不写入或备份');
 assert.deepEqual(position(h),context.position);
 return context;
}

for(const action of ['取消','右上关闭','Escape'])test('导入读文件后'+action+' 不备份、不写入、不通知',async t=>{
 const h=await setup(t),context=await importPreview(h);
 dismiss(h,action);await h.wait(750);
 assert.deepEqual(business(h),context.before);
 assert.deepEqual(position(h),context.position);
});

for(const backup of [true,false])test('导入确认：'+(backup?'默认备份后写入':'取消备份仍可写入')+'，仅采用已选渠道',async t=>{
 const h=await setup(t),context=await importPreview(h,['channel:newapi','channel:legacy']);
 h.change('[data-sx-import-object="channel:legacy"]',false);
 if(!backup)h.change('#sxImportBackup',false);
 assert.deepEqual(business(h),context.before);
 const result=await confirm(h),after=sync(h),m=model(h);
 assert.equal(result.failed.length,0);assert.equal(result.downloaded,1);assert.equal(result.uploaded,0);
 assert.deepEqual(result.items.map(item=>item.id),['channel:newapi']);
 assert.equal(m.saved.providers.find(provider=>provider.id==='newapi').models[0].naming.alias,'primary-model');
 assert.equal(m.saved.providers.filter(provider=>provider.id==='newapi').length,1);
 for(const provider of context.before.saved.providers.filter(provider=>provider.id!=='newapi')){
  assert.deepEqual(m.saved.providers.find(item=>item.id===provider.id),provider,provider.id+' 保留本机内容');
 }
 // 所选渠道自带 GPT 模版依赖，依赖补齐不属于范围泄漏。
 assert.deepEqual(m.templates.filter(item=>item.id!=='gpt'),context.before.templates.filter(item=>item.id!=='gpt'),'无关模版不导入');
 outsideScope(context.before,after,['channel:newapi']);
 if(backup)backupBeforeWrite(context.before,after,result,['channel:newapi']);
 else assert.deepEqual(after.backups,[]);
 assert.equal(deliveries(h).filter(item=>item.eventId==='sync.import.started').length,1);
 resultNotice(h,result,'sync.import.complete');
 assert.deepEqual(position(h),context.position);
});

for(const mode of ['webdav','import'])for(const backup of [true,false])test(mode+' 备份失败场景：'+(backup?'阻止写入并保留原内容':'取消备份后允许执行'),async t=>{
 const h=await setup(t);
 let context,p;
 if(mode==='webdav'){context=await webdav(h,'backup-fail');p=openPreview(h);assert.equal(p.source,'remote');}
 else{h.w.syncPrototype.setScenario('backup-fail');assert.equal(sync(h).scenario,'backup-fail');context=await importPreview(h);}
 if(!backup)h.change(mode==='webdav'?'#sxPreviewBackup':'#sxImportBackup',false);
 assert.deepEqual(business(h),context.before);
 const result=await confirm(h),after=sync(h);
 assert.deepEqual(after.backups,[],'失败的备份或未选择备份都不得登记成功备份');
 if(backup){
  assert.ok(result.failed.length);assert.equal(result.downloaded+result.uploaded,0);
  assert.match(result.failed.map(item=>item.reason).join(' '),/备份/);
  assert.deepEqual(after.versions,context.before.versions);
  assert.deepEqual(after.demoObjects,context.before.demoObjects);
  assert.deepEqual(model(h).saved,context.before.saved);
  assert.deepEqual(model(h).draft,context.before.draft);
  assert.deepEqual(model(h).templates,context.before.templates);
  assert.equal(resultNotice(h,result,mode==='webdav'?'sync.failed':'sync.import.failed').payload.errorCode,'backup');
 }else{
  assert.equal(result.failed.length,0);assert.ok(result.downloaded>0);assert.equal(result.uploaded,0);
  if(mode==='webdav')singleSource(context.before,after,p,result);
  else assert.notDeepEqual(model(h).saved,context.before.saved);
  resultNotice(h,result,mode==='webdav'?'sync.complete':'sync.import.complete');
 }
 assert.deepEqual(position(h),context.position);
});

for(const mode of ['webdav','import'])test(mode+' write-fail 独立于备份：取消备份仍写入失败并保留原内容',async t=>{
 const h=await setup(t);let context;
 if(mode==='webdav'){context=await webdav(h,'write-fail');openPreview(h);}
 else{h.w.syncPrototype.setScenario('write-fail');assert.equal(sync(h).scenario,'write-fail');context=await importPreview(h);}
 h.change(mode==='webdav'?'#sxPreviewBackup':'#sxImportBackup',false);
 const result=await confirm(h),after=sync(h);
 assert.ok(result.failed.length);assert.equal(result.downloaded+result.uploaded,0);
 assert.deepEqual(after.backups,[]);assert.deepEqual(after.versions,context.before.versions);
 assert.deepEqual(after.demoObjects,context.before.demoObjects);
 assert.deepEqual(model(h).saved,context.before.saved);assert.deepEqual(model(h).draft,context.before.draft);
 assert.deepEqual(model(h).templates,context.before.templates);
 assert.equal(resultNotice(h,result,mode==='webdav'?'sync.failed':'sync.import.failed').payload.errorCode,'write');
 assert.deepEqual(position(h),context.position);
});
