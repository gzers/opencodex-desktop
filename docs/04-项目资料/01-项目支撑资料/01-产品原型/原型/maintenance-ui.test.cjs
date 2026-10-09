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
    if(!nodes.has(id))nodes.set(id,{id,innerHTML:'',textContent:'',dataset:{},style:{},classList:{add(){},remove(){}},checked:true,disabled:false,
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
    const button={dataset:{maintAction:action,target},disabled:false};
    handlers.click.forEach(fn=>fn({target:{closest:()=>button}}));
  }
  function change(id,value){handlers.change.forEach(fn=>fn({target:{id,value}}));}
  function flush(delay){for(const timer of queue.slice())if(timer.active&&timer.delay===delay){timer.active=false;timer.fn();}}
  return {win,nodes,node,doc,click,change,flush,modal:()=>currentModal};
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
  h.node('#maintenancePreBackup').checked=false;h.modal().handler();h.flush(1300);
  assert.equal(h.win.maintenancePrototype.model.state.updates.manager.current,'0.1.10');
  assert.equal(h.win.maintenancePrototype.model.state.updates.runtime.current,'2.50.0');
  assert.equal(h.win.maintenancePrototype.model.state.backups.length,before);
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
