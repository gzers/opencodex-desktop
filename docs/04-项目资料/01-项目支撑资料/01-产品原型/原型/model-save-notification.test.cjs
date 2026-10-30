/* 模型保存必须注册到应用壳层通知中心 / 任务卡，不得回到模型页内自建进度卡。 */
const {test}=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const events=require('./maintenance-events.js');

const dir=__dirname;
const source=()=>fs.readFileSync(path.join(dir,'index.html'),'utf8');
const model=()=>fs.readFileSync(path.join(dir,'model-config.js'),'utf8');

test('模型保存事件注册到共享任务与通知中心',()=>{
 const spec=events.events.find(item=>item.id==='model.policy.save');
 assert.ok(spec);
 assert.deepEqual(spec.delivery,['task','center']);
 assert.equal(spec.implemented,'prototype');
 assert.equal(spec.consumer,'modelSaveNotificationBridge');
 assert.deepEqual(spec.dedupeFields,['operationId']);
});

test('保存确认被拦下登记独立通知，且不接收业务内容',()=>{
 const spec=events.events.find(item=>item.id==='model.policy.save.blocked');
 assert.ok(spec);
 assert.equal(spec.module,'模型配置');
 assert.equal(spec.severity,'warning');
 assert.deepEqual(spec.delivery,['toast','center']);
 assert.equal(spec.implemented,'prototype');
 assert.equal(spec.consumer,'modelSaveBlockedBridge');
 assert.match(spec.redaction,/不接收计划、渠道名、URL、Key/);
 const html=source(),js=model();
 assert.match(html,/window\.modelSaveBlockedBridge=function/);
 assert.match(js,/modelSaveBlockedBridge/);
 assert.doesNotMatch(js,/mpSaveError/);
});

test('模型页移除独立进度卡，保存状态只经通知宿主桥接',()=>{
 const html=source(),js=model();
 assert.match(html,/id="notifHost"/);
 assert.match(html,/id="notificationPanel"/);
 assert.match(html,/modelSaveNotificationBridge/);
 assert.match(js,/modelSaveNotificationBridge\?\./);
 assert.doesNotMatch(js,/function progress\(\)/);
 assert.doesNotMatch(js,/mp-progress/);
});

test('文件保存仅备份与原子写入，不调用真实网络/CLI/文件，不读取旧计划准入或提供保存后核对动作',()=>{
 const html=source(),js=model();
 assert.match(html,/MODEL_SAVE_PHASES=\['备份','原子写入'\]/);
 assert.doesNotMatch(js,/function verify\(|checkSavedClient|operation\.plan|saveScope|unfinished|latestPolicy|\b(?:fetch|XMLHttpRequest|WebSocket|invoke|exec|spawn|readFile|writeFile)\s*\(/);
 assert.doesNotMatch(html,/taskReopenHint|reopenHidden|data-task-act="model-(?:verify|check)"/);
});

test('共享任务卡支持收起到铃铛和模型保存动作',()=>{
 const html=source();
 assert.match(html,/data-collapse="task"/);
 assert.match(html,/data-task-act=/);
 assert.match(html,/modelSaveNotificationAction/);
 assert.match(html,/openModelSaveNotification/);
});

/* 完整总原型 DOM 接线；不替代实屏布局或真实文件 / 网络 / 重启验收。 */
const {make}=require('./component-topic-harness.cjs');
const selector='[data-task-notification^="model-policy-save:"]';
async function saved(h){
 h.click('[data-mp=toggle-provider][data-id=aimami]');h.click('#mpPolicySave');
 assert.equal(h.document.querySelector('#modalTitle').textContent,'保存预览');
 h.click('#modalConfirm');assert.equal(h.document.querySelector('#modalMask').style.display,'none');
 for(let i=0;i<120&&h.w.modelPrototype.state().operation.status==='running';i++)await h.wait(30);
 assert.equal(h.w.modelPrototype.state().operation.status,'written');
 return h.w.modelPrototype.state().operation.id;
}
function productText(h){const copy=h.document.querySelector('#mpContent').cloneNode(true);copy.querySelectorAll('.proto-annotation,.annotation-source').forEach(n=>n.remove());return copy.textContent;}
function noEffects(h){assert.deepEqual(h.network,[]);assert.ok(h.writes.every(([key])=>['ocx-theme','ocx-route'].includes(key)),'仅允许评审壳既有主题/路由缓存，不写凭据或配置');assert.deepEqual(h.errors,[]);}

test('确认保存后仅原 notifHost 一次已写入反馈，默认焦点立即重启；稍后清掉字符串通知ID与计时器',async t=>{
 const h=await make('#models');t.after(()=>h.dispose());const d=h.document,id=await saved(h),key='model-policy-save:'+id;
 const card=d.querySelector(selector);assert.ok(card.closest('#notifHost'));assert.equal(d.querySelector('#mpContent .task-card'),null);
 assert.equal(d.querySelectorAll(selector).length,1);assert.equal(card.querySelector('h4').textContent,'已写入');assert.equal(card.querySelector('.task-steps'),null);
 assert.deepEqual([...card.querySelectorAll('[data-task-act]')].map(b=>b.textContent),['立即重启 Codex','稍后']);
 assert.equal(d.activeElement,card.querySelector('[data-task-act=model-restart]'));
 h.w.modelPrototype.render();h.w.modelPrototype.render();assert.equal(d.querySelector(selector),card);
 h.click('[data-task-act=model-later]');assert.equal(d.querySelector(selector),null);
 assert.equal(h.w.eval('taskTimers.has('+JSON.stringify(key)+')'),false);
 assert.equal(h.w.modelPrototype.state().pending,false);assert.equal(h.w.modelPrototype.state().operation.status,'completed');
 const item=h.w.eval('notifications.find(n=>n.id==='+JSON.stringify(key)+')');assert.equal(item.resolved,true);assert.equal(item.read,true);assert.equal(item.deleted,false);
 h.w.setRoute('settings');h.w.setRoute('models');h.w.modelPrototype.render();
 h.click('#notificationBtn');assert.equal(d.querySelectorAll(selector).length,0);h.click('[data-notification="'+key+'"]');assert.equal(d.querySelector(selector),null);
 h.w.modelSaveNotificationBridge({id,status:'written',step:5});assert.equal(d.querySelector(selector),null);
 h.click('[data-mp=toggle-provider][data-id=aimami]');assert.match(productText(h),/未保存修改/);assert.doesNotMatch(productText(h),/待重启|客户端核对|目视确认/);
 noEffects(h);
});

test('收起仅入现有铃铛，打开铃铛不铺开任务，点对应通知才恢复；重启复用共享确认弹窗',async t=>{
 const h=await make('#models');t.after(()=>h.dispose());const d=h.document,id=await saved(h),key='model-policy-save:'+id;
 h.click(selector+' [data-collapse=task]');await h.wait(h.w.NotificationMotion.durationMs+40);assert.equal(d.querySelector(selector),null);assert.equal(d.querySelector('#taskReopenHint,.task-reopen'),null);
 h.w.modelPrototype.render();assert.equal(d.querySelector(selector),null);
 h.click('#notificationBtn');assert.ok(d.querySelector('#notificationPanel.show'));assert.equal(d.querySelector(selector),null);
 h.click('[data-notification="'+key+'"]');assert.equal(d.querySelectorAll(selector).length,1);
 h.click('[data-task-act=model-restart]');await h.settle();assert.equal(d.querySelector('#modalTitle').textContent,'重启 Codex');assert.match(d.querySelector('#modalBody').textContent,/结束当前正在进行的 Codex 对话/);
 assert.equal(d.activeElement,d.querySelector('#modalConfirm'));h.click('#modalCancel');assert.ok(d.querySelector(selector));
 h.click('[data-task-act=model-restart]');h.click('#modalConfirm');assert.equal(d.querySelector(selector),null);assert.equal(h.w.modelPrototype.state().operation.status,'completed');
 assert.match(d.querySelector('#toastEl').textContent,/原型模拟/);assert.doesNotMatch(productText(h),/待重启|客户端核对/);noEffects(h);
});

test('旧 success / confirmed / pending 仅保留完成历史，不产生提示；清理沿用 openModal 并清除任务',async t=>{
 const h=await make('#models');t.after(()=>h.dispose());const d=h.document;
 for(const [i,status] of ['success','confirmed','pending'].entries()){
  h.w.modelSaveNotificationBridge({id:900+i,status,step:5,pending:true});
  assert.equal(d.querySelector(selector),null);assert.equal(h.w.eval('notifications.find(n=>n.id==="model-policy-save:'+ (900+i) +'").modelSave.state'),'completed');
 }
 await saved(h);h.click('#notificationBtn');const panel=d.querySelector('#notificationPanel');h.click('#notificationPurge');
 assert.equal(panel.classList.contains('show'),false);assert.equal(d.querySelector('#modalTitle').textContent,'清理全部通知');assert.equal(d.querySelector('#modalMask').style.display,'flex');
 h.click('#modalCancel');assert.ok(d.querySelector(selector));h.click('#notificationBtn');h.click('#notificationPurge');h.click('#modalConfirm');
 assert.equal(d.querySelector(selector),null);assert.equal(h.w.eval('liveNotifications().length'),0);h.w.modelPrototype.render();assert.equal(d.querySelector(selector),null);noEffects(h);
});

test('终态焦点停留时暂停，离开后8秒收起且仅从铃铛查看，重渲染不重复提示',async t=>{
 const h=await make('#models');t.after(()=>h.dispose());await saved(h);
 const card=h.document.querySelector(selector),timer=h.w.NotificationCountdown.get(card);assert.equal(timer.state().phase,'paused');
 h.document.querySelector('#notificationBtn').focus();assert.equal(timer.state().phase,'running');await h.wait(8150+h.w.NotificationMotion.durationMs);
 assert.equal(h.document.querySelector(selector),null);h.w.modelPrototype.render();assert.equal(h.document.querySelector(selector),null);
 h.click('#notificationBtn');assert.equal(h.document.querySelector(selector),null);assert.equal(h.document.querySelector('#taskReopenHint,.task-reopen'),null);noEffects(h);
});

test('Key 显隐、离线详情保存并返回、空输入保持与明确清空沿同一内存保存链',async t=>{
 const h=await make('#models');t.after(()=>h.dispose());const d=h.document,$=q=>d.querySelector(q),state=()=>h.w.modelPrototype.state();
 const saved=async()=>{for(let i=0;i<120&&state().operation?.status==='running';i++)await h.wait(30);assert.equal(state().operation.status,'written');assert.equal(state().channelDetail,false);assert.notEqual($('#modalMask').style.display,'flex');};
 h.w.modelPrototype.scene('discovery-fail');h.click('[data-mp=edit-provider][data-id=newapi]');
 const original=$('#mpChannelInline-key').value,models=JSON.stringify(state().draft.providers[0].models);assert.equal(original,'demo-newapi-key');assert.equal($('#mpChannelInline-key').type,'password');
 h.click('[data-mp=toggle-channel-key]');assert.equal($('#mpChannelInline-key').type,'text');assert.equal($('[data-mp=toggle-channel-key]').getAttribute('aria-pressed'),'true');h.click('[data-mp=toggle-channel-key]');assert.equal($('#mpChannelInline-key').type,'password');
 const replacement='demo-only-replaced<&"';h.input('#mpChannelInline-key',replacement);h.click('[data-mp=back-channels]');h.click('[data-mp=edit-provider][data-id=newapi]');assert.equal($('#mpChannelInline-key').value,replacement);assert.equal($('#mpChannelInline-key').type,'password');
 assert.equal($('[data-mp=apply-channel-connection]'),null);assert.equal($('[data-mp=reset-channel-connection]'),null);assert.equal(JSON.stringify(state().draft.providers[0].models),models);
 h.click('#mpPolicySave');assert.notEqual($('#modalMask').style.display,'flex');assert.equal(state().operation.channelId,'newapi');await saved();h.click('[data-task-act=model-later]');
 h.click('[data-mp=edit-provider][data-id=newapi]');assert.equal($('#mpChannelInline-key').value,replacement);assert.equal($('#mpChannelInline-key').type,'password');const ref=state().saved.providers[0].connection.credentialRef;
 h.input('#mpChannelInline-key','');assert.equal(state().draft.providers[0].connection.credentialRef,ref);h.click('[data-mp=back-channels]');h.click('[data-mp=edit-provider][data-id=newapi]');assert.equal($('#mpChannelInline-key').value,replacement);
 assert.equal(JSON.stringify(state()).includes(replacement),false);assert.equal(h.w.eval('JSON.stringify(notifications)').includes(replacement),false);
 h.click('[data-mp=clear-channel-key]');assert.match($('#mpChannelKeyStatus').textContent,/清空/);assert.equal(state().saved.providers[0].connection.keyConfigured,true);h.click('#mpPolicySave');await saved();
 assert.equal(state().saved.providers[0].connection.keyConfigured,false);h.click('[data-task-act=model-later]');h.click('[data-mp=edit-provider][data-id=newapi]');assert.equal($('#mpChannelInline-key').value,'');noEffects(h);
});
