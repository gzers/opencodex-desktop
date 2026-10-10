/* 源码 VM + 最小 DOM 替身：验证交互接线，不代表浏览器布局、焦点或实屏验收。 */
const {test}=require('node:test');
const assert=require('node:assert/strict');
const vm=require('node:vm');
const fs=require('node:fs');
const path=require('node:path');
const M=require('./maintenance-state.js');
function harness(){
  const nodes=new Map(), handlers={}, queue=[], intervals=[], observers=[], messages=[];
  function node(id){
    if(!nodes.has(id))nodes.set(id,{id,innerHTML:'',textContent:'',dataset:{},style:{},isConnected:true,classList:{add(){},remove(){},contains(){return false;}},checked:true,disabled:false,
      setAttribute(){},querySelector(selector){return node(id+' '+selector);},
      insertAdjacentHTML(_,html){this.innerHTML+=html;},appendChild(){},remove(){},focus(){}});
    return nodes.get(id);
  }
  const doc={activeElement:null,hidden:false,documentElement:{dataset:{route:'overview'}},
    querySelector:node,getElementById:id=>node('#'+id),createElement:()=>node('created'),
    addEventListener(type,fn){(handlers[type]??=[]).push(fn);}};
  node('#modalMask').style.display='none';
  const windowHandlers={};
  const win={MaintenanceModel:M,PrototypeTreeTable:require('./tree-table.js'),prototypeSelect:{enhance(){}},prototypePanel:{refresh(){}},
    addEventListener(type,fn){(windowHandlers[type]??=[]).push(fn);},
    dispatchEvent(event){(windowHandlers[event.type]||[]).forEach(fn=>fn(event));},toast(message){messages.push(message);}};
  let currentModal=null;
  const context={window:win,document:doc,location:{search:''},URLSearchParams,console,
    setTimeout(fn,delay){const timer={fn,delay,active:true};queue.push(timer);return timer;},
    clearTimeout(timer){if(timer)timer.active=false;},setInterval(fn){intervals.push(fn);},
    MutationObserver:class{constructor(fn){observers.push(fn);}observe(){}},
    openModal(title,body,label,handler,opts){currentModal={title,body,label,handler,opts};node('#modalMask').style.display='flex';node('#maintenancePreBackup').checked=true;},
    closeModal(){node('#modalMask').style.display='none';},toast(message){messages.push(message);},openSettingsSection(){},setRoute(){}};
  vm.runInNewContext(fs.readFileSync(path.join(__dirname,'maintenance-ui.js'),'utf8'),context);
  function click(action,target=''){
    const button={dataset:{maintAction:action,target},disabled:false,closest:()=>node('#testNotesRegion')};
    handlers.click.forEach(fn=>fn({target:{closest:()=>button}}));
  }
  function change(id,value){const target=node('#'+id);target.id=id;target.value=value;handlers.change.forEach(fn=>fn({target}));}
  function flush(delay){for(const timer of queue.slice())if(timer.active&&timer.delay===delay){timer.active=false;timer.fn();}}
  const finishApply=()=>[600,450,500,500,450,650,800].forEach(flush);
  return {win,nodes,node,doc,click,change,flush,finishApply,messages,close:context.closeModal,modal:()=>currentModal};
}
test('目录模型切换后备份页与生成确认同步保存位置，原有备份记录保持',()=>{
  const h=harness(), before=h.win.maintenancePrototype.model.state.backups.length;
  const backups=require('./data-paths.js').paths('windows').backups;
  h.win.dataPathsPrototype={current:()=>({backups})};h.win.dispatchEvent({type:'prototype:data-paths'});
  assert.ok(h.node('#card-backup-data').innerHTML.includes(backups));
  h.click('backup-dialog');assert.ok(h.modal().body.includes(backups));
  assert.equal(h.win.maintenancePrototype.model.state.backups.length,before);
});
test('备份树表显示两文件用途与操作；区域始终显示且路径安全转义',()=>{
  const h=harness();
  const initial=h.node('#card-backup-data').innerHTML;
  assert.ok(initial.includes('id="maintenanceFileRow-root"'));
  assert.ok(initial.includes('id="maintenanceFileRow-year"'));
  assert.ok(initial.includes('id="maintenanceFileRow-month"'));
  assert.ok(initial.includes('data-target="month" aria-expanded="false"'));
  assert.ok(!initial.includes('id="maintenanceFileRow-action"'));
  for(const target of ['month','action','record'])h.click('backup-tree-toggle',target);
  const body=h.node('#card-backup-data').innerHTML;
  assert.ok(body.includes('preferences.json'));
  assert.ok(body.includes('backup-manifest.json'));
  assert.ok(body.includes('<table class="mnt-backup-tree">'));
  for(const heading of ['名称','用途','操作'])assert.ok(body.includes('>'+heading+'</th>'));
  assert.ok(!body.includes('<pre'));
  assert.equal((body.match(/打开文件<\/button>/g)||[]).length,2);
  assert.ok(body.includes('没有单独的校验摘要文件'));
  assert.ok(body.includes('tabindex="0" role="region"'));
  h.win.dataPathsPrototype={current:()=>({backups:'X:/<data>/backups'})};
  h.win.dispatchEvent({type:'prototype:data-paths'});
  const next=h.node('#card-backup-data').innerHTML;
  assert.ok(next.includes('<section id="maintenanceBackupFiles"'));
  assert.ok(!next.includes('<details id="maintenanceBackupFiles"'));
  assert.ok(next.includes('<h3 id="maintenanceBackupFilesTitle">备份文件与目录</h3>'));
  assert.ok(next.includes('X:/&lt;data&gt;/backups'));
  assert.ok(!next.includes('X:/<data>'));
});
test('树表逐级折叠；父级展开保留子级折叠，叶节点与未知对象无操作',()=>{
  const h=harness(),html=()=>h.node('#card-backup-data').innerHTML;
  for(const target of ['month','action','record'])h.click('backup-tree-toggle',target);
  h.click('backup-tree-toggle','record');
  assert.ok(!html().includes('id="maintenanceFileRow-preferences"'));
  assert.ok(html().includes('data-target="record" aria-expanded="false"'));
  h.click('backup-tree-toggle','month');assert.ok(!html().includes('id="maintenanceFileRow-record"'));
  h.click('backup-tree-toggle','month');assert.ok(html().includes('id="maintenanceFileRow-record"'));
  assert.ok(!html().includes('id="maintenanceFileRow-preferences"'));
  h.win.dispatchEvent({type:'prototype:data-paths'});
  assert.ok(!html().includes('id="maintenanceFileRow-preferences"'));
  h.click('backup-tree-toggle','record');assert.ok(html().includes('id="maintenanceFileRow-preferences"'));
  const before=html();h.click('backup-tree-toggle','preferences');h.click('backup-tree-toggle','unknown');
  assert.equal(html(),before);h.click('backup-node-open','unknown');assert.equal(h.modal(),null);
});
test('两平台文件和目录打开均先警告，取消或确认不改变备份状态',()=>{
  for(const platform of ['windows','macos']){
    const h=harness(), paths=require('./data-paths.js').paths(platform);
    h.win.dataPathsPrototype={current:()=>paths};h.win.dispatchEvent({type:'prototype:data-paths'});
    const before=JSON.stringify(h.win.maintenancePrototype.model.state);
    h.click('backup-node-open','root');assert.equal(h.modal().title,'打开备份目录');
    assert.ok(h.modal().body.includes(paths.backups));assert.ok(h.modal().body.includes('请勿直接移动或删除'));
    assert.equal(h.modal().label,'继续打开');h.close();assert.equal(h.messages.length,0);
    h.click('backup-node-open','preferences');assert.equal(h.modal().title,'打开备份文件');
    const sep=platform==='windows'?'\\':'/';
    const file=[paths.backups,'2026','10','upgrade','bk_20261010093000_1a2b3c4d','preferences.json'].join(sep);
    assert.ok(h.modal().body.includes(file));assert.ok(h.modal().body.includes('请勿直接修改'));
    h.modal().handler();assert.equal(h.messages.at(-1),'原型：已确认打开文件，未访问本机文件。');
    h.click('backup-node-open','record');h.modal().handler();
    assert.equal(h.messages.at(-1),'原型：已确认打开目录，未访问本机文件。');
    assert.equal(JSON.stringify(h.win.maintenancePrototype.model.state),before);
  }
});
test('确认前目录变化阻止旧目标打开，路径 HTML 转义',()=>{
  const h=harness();let root='X:/<data>/backups';
  h.win.dataPathsPrototype={current:()=>({platform:'macos',backups:root})};
  h.click('backup-node-open','manifest');
  assert.ok(h.modal().body.includes('X:/&lt;data&gt;/backups'));
  assert.ok(!h.modal().body.includes('X:/<data>'));
  root='Y:/other/backups';h.win.dispatchEvent({type:'prototype:data-paths'});h.modal().handler();
  assert.equal(h.messages.at(-1),'目录已变化，请重新选择打开目标。');
});
test('概览只有检查按钮，管理器 / 面板状态行可按对象查看；手动检查进入结果步骤',()=>{
  const h=harness();
  assert.equal((h.node('#card-overview-upgrade .mod-actions').innerHTML.match(/<button/g)||[]).length,1);
  assert.ok(!h.node('#card-overview-upgrade .mod-actions').innerHTML.includes('查看更新'));
  assert.ok(h.node('#card-overview-upgrade .mod-status').innerHTML.includes('面板'));
  h.click('check-all');h.flush(850);
  assert.equal(h.modal().title,'检查更新结果');
  assert.ok(h.modal().body.includes('data-target="runtime"'));
  h.click('update-detail','runtime');
  assert.ok(h.modal().title.includes('OpenCodex 面板'));
  assert.ok(h.modal().body.includes('id="maintenancePreBackup" checked'));
});
test('确认弹窗读取当前备份选项；失败不关闭，取消勾选后只更新选定对象',()=>{
  const h=harness();h.click('check-all');h.flush(850);
  h.click('update-detail','manager');h.change('maintenanceBackupResult','failed');
  const before=h.win.maintenancePrototype.model.state.backups.length;
  h.modal().handler();
  assert.equal(h.node('#modalMask').style.display,'flex');
  assert.equal(h.win.maintenancePrototype.model.state.updates.manager.phase,'available');
  assert.equal(h.win.maintenancePrototype.model.state.backups.length,before);
  h.node('#maintenancePreBackup').checked=false;h.modal().handler();h.finishApply();
  assert.equal(h.win.maintenancePrototype.model.state.updates.manager.phase,'restart-required');
  assert.equal(h.win.maintenancePrototype.model.state.updates.manager.current,'0.1.9');
  h.click('restart','manager');
  assert.equal(h.win.maintenancePrototype.model.state.updates.manager.current,'0.1.10');
  assert.equal(h.win.maintenancePrototype.model.state.updates.runtime.current,'2.50.0');
  assert.equal(h.win.maintenancePrototype.model.state.backups.length,before);
});
test('结果上下排列、两对象都有内部说明与安全参考链接；设置共用详情和进度',()=>{
  const h=harness();h.click('check-all');h.flush(850);
  assert.ok(h.modal().body.includes('mnt-release-notes compact'));
  assert.ok(h.modal().body.includes('rel="noopener noreferrer"'));
  h.doc.documentElement.dataset.route='settings';h.click('apply-dialog','runtime');
  assert.ok(h.modal().body.includes('role="region"'));
  assert.ok(h.modal().body.includes('兼容性说明'));
  h.modal().handler();h.flush(600);
  assert.ok(h.node('#card-upgrade-official').innerHTML.includes('aria-valuenow="18"'));
  assert.ok(h.node('#maintenanceProgress').innerHTML.includes('aria-valuenow="18"'));
  h.close();h.finishApply();
  assert.equal(h.node('#modalMask').style.display,'none');
  h.click('progress','runtime');
  assert.ok(h.modal().body.includes('更新完成'));
  assert.ok(!h.modal().body.includes('正在应用更新'));
});
test('说明失败与缺失独立于检查结果；重试不检查或安装，说明文本转义',()=>{
  const h=harness();h.click('check-all');h.flush(850);
  h.change('maintenanceNotesScenario','failed');h.click('update-detail','manager');
  assert.ok(h.modal().body.includes('重试说明'));
  const u=h.win.maintenancePrototype.model.state.updates.manager, generation=u.generation;
  h.click('notes-retry','manager');h.flush(650);h.click('update-detail','manager');
  assert.ok(h.modal().body.includes('概览更新入口'));
  assert.equal(u.generation,generation);assert.equal(u.phase,'available');
  h.change('maintenanceNotesScenario','empty');h.click('update-detail','runtime');
  assert.ok(h.modal().body.includes('尚未提供'));
  u.source='<img src=x onerror=alert(1)>';h.click('update-detail','manager');
  assert.ok(!h.modal().body.includes('<img'));
  assert.ok(h.modal().body.includes('&lt;img'));
});
test('取消阻止后续进度；校验失败不进入应用阶段，当前版本保留',()=>{
  const h=harness();h.click('check-all');h.flush(850);h.click('update-detail','runtime');h.modal().handler();
  h.click('cancel','runtime');h.finishApply();
  const u=h.win.maintenancePrototype.model.state.updates.runtime;
  assert.equal(u.phase,'available');assert.equal(u.current,'2.50.0');
  assert.ok(h.node('#maintenanceProgress').innerHTML.includes('已取消'));
  h.change('maintenanceScenario','failed');h.click('update-detail','runtime');h.modal().handler();
  h.finishApply();assert.equal(u.phase,'failed');assert.equal(u.current,'2.50.0');
  assert.ok(!h.node('#card-upgrade-official').innerHTML.includes('正在应用更新'));
});
test('离开概览或已有弹窗时，检查结果不抢弹窗；注册表展示多来源及接入状态',()=>{
  const h=harness();h.click('check-all');h.doc.documentElement.dataset.route='sync';h.flush(850);
  assert.equal(h.modal(),null);
  h.click('registry');
  assert.ok(h.modal().body.includes('route.overview.enter'));
  assert.ok(h.modal().body.includes('planned'));
  const old=h.modal();h.click('check-all');h.doc.documentElement.dataset.route='overview';h.flush(850);
  assert.equal(h.modal(),old);
});

test('切换管理器通道保留面板下载事务，设置进度继续直到完成',()=>{
  const h=harness();h.click('check-all');h.flush(850);h.click('update-detail','runtime');h.modal().handler();h.flush(600);
  const u=h.win.maintenancePrototype.model.state.updates.runtime, token=u.generation;
  assert.equal(h.node('#maintenanceUpdatePolicy').disabled,false);
  h.change('maintenanceUpdatePolicy','beta');assert.equal(u.generation,token);assert.equal(u.phase,'applying');
  h.finishApply();assert.equal(u.phase,'complete');assert.equal(u.current,'2.51.0');
  assert.ok(h.node('#card-upgrade-official').innerHTML.includes('更新完成'));
});
test('管理器检查中切通道只替换管理器请求；面板原检查按时完成',()=>{
  const h=harness();h.click('check-all');const panel=h.win.maintenancePrototype.model.state.updates.runtime;
  const token=panel.generation;h.change('maintenanceUpdatePolicy','beta');assert.equal(panel.generation,token);
  h.flush(850);assert.equal(panel.phase,'available');assert.equal(panel.source,'官方 npm 包（示例）');
  assert.equal(h.win.maintenancePrototype.model.state.updates.manager.phase,'available');
});
test('同通道不重查；被阻止的策略切换恢复选择值',()=>{
  const h=harness();h.click('check-all');h.flush(850);const model=h.win.maintenancePrototype.model;
  const generation=model.state.updates.manager.generation;h.change('maintenanceUpdatePolicy','stable');
  assert.equal(model.state.updates.manager.generation,generation);
  h.click('update-detail','manager');h.modal().handler();h.finishApply();
  assert.equal(h.node('#maintenanceUpdatePolicy').disabled,true);
  h.change('maintenanceUpdatePolicy','beta');assert.equal(model.state.updatePolicy.channel,'stable');
  assert.equal(h.node('#maintenanceUpdatePolicy').value,'stable');
});
test('备份失败确认重试成功解除对应事务；略过备份不会标记已恢复',()=>{
  const h=harness();h.click('check-all');h.flush(850);h.click('update-detail','manager');
  h.change('maintenanceBackupResult','failed');h.modal().handler();
  const model=h.win.maintenancePrototype.model,failure=model.state.notifications.find(n=>n.id==='backup.failed');
  model.backup('手动');assert.equal(failure.resolved,false);
  h.change('maintenanceBackupResult','valid');h.modal().handler();assert.equal(failure.resolved,true);assert.equal(failure.read,false);
  const other=harness();other.click('check-all');other.flush(850);other.click('update-detail','runtime');
  other.change('maintenanceBackupResult','failed');other.modal().handler();
  other.node('#maintenancePreBackup').checked=false;other.modal().handler();other.finishApply();
  assert.equal(other.win.maintenancePrototype.model.state.notifications.find(n=>n.id==='backup.failed').resolved,false);
});
