/* 源码 VM + 最小 DOM 替身：验证交互接线，不代表浏览器布局、焦点或实屏验收。 */
const {test}=require('node:test');
const assert=require('node:assert/strict');
const vm=require('node:vm');
const fs=require('node:fs');
const path=require('node:path');
const M=require('./maintenance-state.js');
function harness(){
  const nodes=new Map(), handlers={}, queue=[], intervals=[], observers=[];
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
  const win={MaintenanceModel:M,prototypeSelect:{enhance(){}},prototypePanel:{refresh(){}},addEventListener(){},toast(){}};
  let currentModal=null;
  const context={window:win,document:doc,location:{search:''},URLSearchParams,console,
    setTimeout(fn,delay){const timer={fn,delay,active:true};queue.push(timer);return timer;},
    clearTimeout(timer){if(timer)timer.active=false;},setInterval(fn){intervals.push(fn);},
    MutationObserver:class{constructor(fn){observers.push(fn);}observe(){}},
    openModal(title,body,label,handler,opts){currentModal={title,body,label,handler,opts};node('#modalMask').style.display='flex';node('#maintenancePreBackup').checked=true;},
    closeModal(){node('#modalMask').style.display='none';},toast(){},openSettingsSection(){},setRoute(){}};
  vm.runInNewContext(fs.readFileSync(path.join(__dirname,'maintenance-ui.js'),'utf8'),context);
  function click(action,target=''){
    const button={dataset:{maintAction:action,target},disabled:false,closest:()=>node('#testNotesRegion')};
    handlers.click.forEach(fn=>fn({target:{closest:()=>button}}));
  }
  function change(id,value){handlers.change.forEach(fn=>fn({target:{id,value}}));}
  function flush(delay){for(const timer of queue.slice())if(timer.active&&timer.delay===delay){timer.active=false;timer.fn();}}
  const finishApply=()=>[600,450,500,500,450,650,800].forEach(flush);
  return {win,nodes,node,doc,click,change,flush,finishApply,close:context.closeModal,modal:()=>currentModal};
}
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
