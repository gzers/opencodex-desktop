/* 执行：node maintenance-state.test.cjs；不启动浏览器或读写应用数据。 */
const {test}=require('node:test');
const assert=require('node:assert/strict');
const M=require('./maintenance-state.js');
const now=Date.UTC(2026,9,9), day=86400000;
test('保留数量或天数满足其一即保留，固定项不占数量',()=>{
  const items=[
    {id:'pin',created:now-day*90,pinned:true},
    {id:'recent',created:now-day*40,pinned:false},
    {id:'old',created:now-day*50,pinned:false}
  ];
  assert.deepEqual(M.cleanupCandidates(items,{count:1,days:30},now).map(i=>i.id),['old']);
  assert.deepEqual(M.cleanupCandidates(items,{count:1,days:60},now),[]);
});
test('时间边界恰好等于保留天数不删除',()=>{
  assert.deepEqual(M.cleanupCandidates([{id:'a',created:now-day*30,pinned:false}],{count:0,days:30},now),[]);
});
test('清理确认重新计算固定与策略保护',()=>{
  const m=M.create(now), ids=M.cleanupCandidates(m.state.backups,m.state.policy,now).map(i=>i.id);
  assert.equal(ids.length,3);
  m.pin(ids[0]); assert.equal(m.clean(ids,now),2);
  assert.ok(m.state.backups.find(i=>i.id===ids[0]));
  m.setPolicy(100,365,'manual');assert.equal(m.clean(ids,now),0);
});
test('自动轮换仅在创建成功后，固定项始终保留',()=>{
  const m=M.create(now);m.setPolicy(1,1,'after-create');
  assert.equal(m.state.backups.length,14);
  m.backup('手动',now);
  assert.equal(m.state.backups.length,3); // 新备份、仍在天数内的一份、固定项。
  assert.ok(m.state.backups.find(i=>i.id==='sample-13'));
});
test('校验失败阻止恢复；恢复创建固定的当前偏好保护备份',()=>{
  const m=M.create(now);assert.equal(m.restore('sample-11',now),false);
  assert.equal(m.state.backups.length,14);
  m.setPolicy(1,1,'after-create');assert.equal(m.restore('sample-12',now),true);
  assert.ok(m.state.backups.find(i=>i.id==='sample-12'));
  assert.ok(m.state.backups.some(i=>i.reason==='恢复前'&&i.pinned));
  assert.equal(m.state.notifications.find(i=>i.id==='backup.restore.failed'&&i.object==='sample-11').resolved,false);
});
test('策略非法值不修改状态',()=>{
  const m=M.create(now), old=m.state.policy;
  for(const count of [0,1.5,101,NaN]) assert.equal(m.setPolicy(count,30,'manual'),false);
  assert.equal(m.setPolicy(10,366,'manual'),false);
  assert.equal(m.setPolicy(10,30,'unknown'),false);
  assert.equal(m.state.policy,old);
});
test('更新并发互斥，取消后迟到响应不能改写版本',()=>{
  const m=M.create(now), token=m.begin('manager');
  assert.equal(m.begin('manager'),null);m.cancel('manager');
  assert.equal(m.finish('manager',token,'available',now),false);
  assert.equal(m.state.updates.manager.phase,'idle');
  const next=m.begin('manager');assert.equal(m.finish('manager',next,'available',now),true);
  assert.equal(m.state.updates.runtime.phase,'idle');
});
test('查询不改变版本；只有确认应用成功才改变版本',()=>{
  const m=M.create(now), check=m.begin('manager');
  assert.equal(m.finish('manager',check,'complete',now),false);
  m.finish('manager',check,'available',now);
  assert.equal(m.state.updates.manager.current,'0.1.9');
  const apply=m.begin('manager','applying');m.finish('manager',apply,'complete',now);
  assert.equal(m.state.updates.manager.current,'0.1.10');
  assert.equal(m.begin('manager','applying'),null);
});
test('应用失败保留当前版本并允许重新检查',()=>{
  const m=M.create(now);m.setScenario('runtime','available');
  m.finish('runtime',m.begin('runtime','applying'),'failed',now);
  assert.equal(m.state.updates.runtime.current,'2.50.0');
  assert.notEqual(m.begin('runtime'),null);
});
test('重复失败去重、按冷却再次投递，恢复与已读独立',()=>{
  const m=M.create(now);m.emit('update.failed','manager',now,{stage:'checking',channel:'stable'});
  assert.equal(m.emit('update.failed','manager',now+1,{stage:'checking',channel:'stable'}),null);
  assert.equal(m.state.deliveries.length,1);
  assert.equal(m.state.notifications[0].occurrences,2);
  assert.ok(m.emit('update.failed','manager',now+day,{stage:'checking',channel:'stable'}));
  assert.equal(m.state.notifications.length,1);
  m.emit('update.latest','manager',now+day+1,{stage:'checking',channel:'stable'});
  const failed=m.state.notifications.find(i=>i.id==='update.failed');
  assert.equal(failed.resolved,true);assert.equal(failed.read,false);
  m.emit('update.failed','manager',now+day+2,{stage:'checking',channel:'stable'});
  assert.equal(m.state.notifications.filter(i=>i.id==='update.failed').length,2);
  assert.notEqual(m.state.notifications[0].uid,failed.uid);
});
test('全量候选事件字段完整，未注册事件拒绝投递',()=>{
  assert.equal(new Set(M.registry.map(i=>i.id)).size,M.registry.length);
  for(const spec of M.registry) for(const key of ['module','trigger','source','category','severity','nature','checkPeriod','reminderPeriod','cooldown','dedupe','recovery','delivery','retention','redaction']) assert.notEqual(spec[key],undefined);
  assert.throws(()=>M.create(now).emit('unknown','test',now),/未注册/);
});
test('每次创建独立内存，刷新重新创建不沿用策略或版本',()=>{
  const a=M.create(now),b=M.create(now);a.setPolicy(1,1,'after-create');a.setScenario('manager','available');a.backup();
  assert.equal(b.state.policy.count,10);assert.equal(b.state.updates.manager.phase,'idle');assert.equal(b.state.backups.length,14);
});
test('注册配置的触发源、任务与解除引用有效，未注册来源被拒绝',()=>{
  const ids=new Set(M.registry.map(e=>e.id));
  for(const spec of M.registry){
    assert.ok(spec.triggers.length);
    for(const origin of spec.triggers)assert.ok(Object.hasOwn(M.config.triggers,origin));
    for(const recovery of spec.recovery)assert.ok(ids.has(recovery.split(':')[0]));
    assert.ok(['prototype','planned'].includes(spec.implemented));
  }
  for(const job of M.config.jobs)for(const origin of job.triggers)assert.ok(Object.hasOwn(M.config.triggers,origin));
  const m=M.create(now);
  assert.throws(()=>m.requestChecks('unknown',now),/未注册/);
  assert.throws(()=>m.emit('update.available','manager',now,{origin:'data.changed'}),/未注册/);
  assert.equal(m.begin('manager','checking','unknown'),null);
  assert.equal(m.state.notifications.length,0);
});
test('启动延迟、多来源合并与成功缓存；不发生自动安装',()=>{
  const m=M.create(now);
  assert.equal(m.requestChecks('app.ready',now+14999).length,0);
  const batch=m.requestChecks('app.ready',now+15000);assert.equal(batch.length,2);
  assert.equal(m.requestChecks('route.overview.enter',now+15000).length,0);
  assert.equal(m.requestChecks('network.online',now+15000).length,0);
  for(const r of batch)m.finish(r.target,r.generation,'available',now+15000);
  assert.equal(m.requestChecks('route.overview.enter',now+day-1).length,0);
  assert.equal(m.requestChecks('schedule.due',now+day+15000).length,2);
  assert.equal(m.state.updates.manager.current,'0.1.9');
});
test('手动检查跳过缓存；beta 的面板周期不随管理器缩短',()=>{
  const m=M.create(now);
  for(const r of m.requestChecks('user.action',now))m.finish(r.target,r.generation,'latest',now);
  m.setUpdatePolicy('beta',now);
  const batch=m.requestChecks('policy.changed',now);assert.deepEqual(batch.map(r=>r.target),['manager']);
  for(const r of batch)m.finish(r.target,r.generation,'latest',now);
  const auto=m.requestChecks('schedule.due',now+6*3600000);assert.deepEqual(auto.map(r=>r.target),['manager']);
  m.finish('manager',auto[0].generation,'latest',now+6*3600000);
  assert.equal(m.requestChecks('user.action',now+6*3600000+1).length,2);
});
test('手动策略禁止全部自动源；只取消管理器旧结果，面板在途检查继续',()=>{
  const m=M.create(now),old=m.requestChecks('user.action',now);
  m.setUpdatePolicy('manual',now+1);
  for(const r of old)assert.equal(m.finish(r.target,r.generation,'available',now+2),r.target==='runtime');
  for(const origin of ['app.ready','route.overview.enter','schedule.due','window.resume','network.online','retry.due','policy.changed'])assert.equal(m.requestChecks(origin,now+day*100).length,0);
  assert.equal(m.requestChecks('user.action',now+day).length,2);
});
test('失败按5m、30m、2h退避，耗尽后回到正常周期；网络恢复不绕过退避',()=>{
  const m=M.create(now);let at=now;
  for(const delay of [300000,1800000,7200000,day]){
    const token=m.begin('manager');m.finish('manager',token,'failed',at);
    assert.equal(m.state.updates.manager.lastChecked,null);
    assert.equal(m.state.updates.manager.nextDue,at+delay);
    assert.equal(m.requestChecks('network.online',at+delay-1).filter(r=>r.target==='manager').length,0);
    at+=delay;
  }
  const batch=m.requestChecks('retry.due',at);
  assert.equal(batch.filter(r=>r.target==='manager').length,1);
});
test('恢复后只执行一次到期检查，不补跑错过周期',()=>{
  const m=M.create(now),batch=m.requestChecks('window.resume',now+day*100);
  assert.equal(batch.length,2);
  assert.equal(m.requestChecks('schedule.due',now+day*100).length,0);
  for(const r of batch)m.finish(r.target,r.generation,'latest',now+day*100);
  assert.equal(m.requestChecks('route.overview.enter',now+day*100+1).length,0);
});
test('后台已最新静默，手动检查有toast；可用更新按版本与通道去重',()=>{
  const m=M.create(now);let batch=m.requestChecks('app.ready',now+15000);
  for(const r of batch)m.finish(r.target,r.generation,'latest',now+15000);
  assert.deepEqual(m.state.deliveries[0].delivery,['log']);
  const token=m.begin('manager');m.finish('manager',token,'latest',now+15001);
  assert.ok(m.state.deliveries[0].delivery.includes('toast'));
  const context={revision:'0.1.10',channel:'stable',origin:'app.ready'};
  assert.ok(m.emit('update.available','manager',now+15002,context));
  assert.equal(m.emit('update.available','manager',now+15003,{...context,origin:'route.overview.enter'}),null);
  assert.ok(m.emit('update.available','manager',now+15004,{...context,revision:'0.1.11'}));
  assert.ok(m.emit('update.available','manager',now+15005,{...context,channel:'beta'}));
});
test('检查成功只解除检查失败，不误解应用失败；完成更新解除可用版本通知',()=>{
  const m=M.create(now);
  m.emit('update.failed','manager',now,{stage:'applying',channel:'stable',revision:'0.1.10'});
  m.emit('update.failed','manager',now,{stage:'checking',channel:'stable'});
  m.emit('update.available','manager',now+1,{stage:'checking',channel:'stable',revision:'0.1.10'});
  assert.equal(m.state.notifications.find(n=>n.id==='update.failed'&&n.stage==='applying').resolved,false);
  assert.equal(m.state.notifications.find(n=>n.id==='update.failed'&&n.stage==='checking').resolved,true);
  m.emit('update.complete','manager',now+2,{stage:'applying',channel:'stable',revision:'0.1.10'});
  assert.equal(m.state.notifications.find(n=>n.id==='update.available').resolved,true);
});
test('备份是更新确认选项；取消勾选不创建，失败阻断，过期确认无副作用',()=>{
  const m=M.create(now);m.setScenario('manager','available');let u=m.state.updates.manager;
  const count=m.state.backups.length;
  assert.equal(m.confirmApply('manager',u.generation,u.candidate,true,now,false),null);
  assert.equal(m.state.backups.length,count);assert.equal(u.phase,'available');
  assert.notEqual(m.confirmApply('manager',u.generation,u.candidate,false,now,false),null);
  assert.equal(m.state.backups.length,count);m.cancel('manager',now);
  const stale=u.generation;m.setScenario('manager','available');
  assert.equal(m.confirmApply('manager',stale,u.candidate,true,now),null);
  assert.equal(m.state.backups.length,count);
  assert.notEqual(m.confirmApply('manager',u.generation,u.candidate,true,now),null);
  assert.equal(m.state.backups.length,count+1);
  assert.equal(m.state.backups[0].reason,'升级前');
});
test('一次性不同事务分别投递；冻结配置不可写，未知任务没有副作用',()=>{
  const m=M.create(now);
  m.emit('sync.complete','object',now,{operationId:'a'});
  m.emit('sync.complete','object',now+1,{operationId:'b'});
  assert.equal(m.state.deliveries.length,2);
  assert.ok(Object.isFrozen(M.config.events[0].triggers));
  assert.equal(m.requestChecks('data.changed',now+day).length,0);
});
test('进度按事务版本保护，拒绝倒退与非法百分比；写入阶段不能取消',()=>{
  const model=M.create(now);model.setScenario('runtime','available');
  const u=model.state.updates.runtime, token=model.confirmApply('runtime',u.generation,u.candidate,false,now);
  assert.equal(model.progress('runtime',token-1,'downloading',20),false);
  assert.equal(model.progress('runtime',token,'downloading',20),true);
  assert.equal(model.progress('runtime',token,'downloading',19),false);
  assert.equal(model.progress('runtime',token,'downloading',101),false);
  assert.equal(model.progress('runtime',token,'verifying'),true);
  assert.equal(model.progress('runtime',token,'downloading',100),false);
  assert.equal(model.progress('runtime',token,'installing'),true);
  assert.equal(model.cancel('runtime',now),false);
  assert.equal(model.setUpdatePolicy('beta',now),true);
  assert.equal(u.phase,'applying');
  assert.equal(model.finish('runtime',token,'complete',now),true);
  assert.equal(u.progress.stage,'complete');
  assert.equal(model.progress('runtime',token,'installing'),false);
});
test('管理器安装就绪不改运行版本；待重启不重复查装，重启回读才完成并解除提醒',()=>{
  const model=M.create(now);model.setScenario('manager','available');
  const u=model.state.updates.manager,token=model.confirmApply('manager',u.generation,u.candidate,false,now);
  assert.equal(model.finish('manager',token,'restart-required',now),true);
  assert.equal(u.current,'0.1.9');
  assert.equal(model.begin('manager'),null);
  assert.equal(model.confirmApply('manager',token,u.candidate,false,now),null);
  assert.ok(model.state.notifications.some(n=>n.id==='update.restart.required'&&!n.resolved));
  assert.equal(model.restart('runtime',now),false);
  assert.equal(model.restart('manager',now),true);
  assert.equal(u.current,'0.1.10');
  assert.ok(model.state.notifications.filter(n=>n.id==='update.restart.required').every(n=>n.resolved));
  assert.equal(model.restart('manager',now),false);
});

test('坏恢复源不会因手动备份或其他源恢复成功解除；缺失源单独登记',()=>{
  const m=M.create(now);m.restore('sample-11',now);m.restore('missing',now+1);
  const invalid=m.state.notifications.find(n=>n.object==='sample-11'), missing=m.state.notifications.find(n=>n.object==='missing');
  m.backup('手动',now+2);m.restore('sample-12',now+3);
  assert.equal(invalid.resolved,false);assert.equal(missing.resolved,false);
  assert.equal(invalid.errorCode,'integrity.invalid');assert.equal(missing.errorCode,'source.missing');
  m.state.backups.find(b=>b.id==='sample-11').integrity='valid';
  assert.equal(m.restore('sample-11',now+4),true);
  assert.equal(invalid.resolved,true);assert.equal(invalid.read,false);assert.equal(missing.resolved,false);
  m.state.backups.find(b=>b.id==='sample-11').integrity='invalid';m.restore('sample-11',now+5);
  assert.equal(m.state.notifications.filter(n=>n.id==='backup.restore.failed'&&n.object==='sample-11'&&!n.resolved).length,1);
});
test('备份创建失败按确认事务隔离；只有同一确认重试成功解除',()=>{
  const m=M.create(now);for(const target of ['manager','runtime']){
    m.setScenario(target,'available');const u=m.state.updates[target];
    m.confirmApply(target,u.generation,u.candidate,true,now,false);
  }
  const manager=m.state.notifications.find(n=>n.operationId.startsWith('upgrade:manager:'));
  const panel=m.state.notifications.find(n=>n.operationId.startsWith('upgrade:runtime:'));
  assert.notEqual(manager.key,panel.key);m.backup('手动',now+1);
  assert.equal(manager.resolved,false);assert.equal(panel.resolved,false);
  const u=m.state.updates.manager;
  assert.notEqual(m.confirmApply('manager',u.generation,u.candidate,true,now+2),null);
  assert.equal(manager.resolved,true);assert.equal(panel.resolved,false);
});
test('缺少解除上下文不能消除异常；错误动作和阶段不视为恢复',()=>{
  const m=M.create(now),context={action:'create',stage:'creating',operationId:'attempt-a'};
  m.emit('backup.failed','preferences',now,context);const failure=m.state.notifications[0];
  for(const ctx of [{},{...context,action:'restore'},{...context,stage:'restoring'},{...context,operationId:'attempt-b'},{...context,operationId:''},{...context,operationId:null}]){
    m.emit('backup.created','preferences',now+1,ctx);assert.equal(failure.resolved,false);
  }
  const invalid=M.create(now),empty={...context,operationId:null};
  invalid.emit('backup.failed','preferences',now,empty);const invalidFailure=invalid.state.notifications[0];
  invalid.emit('backup.created','preferences',now+1,empty);assert.equal(invalidFailure.resolved,false);
  m.emit('backup.created','preferences',now+2,context);assert.equal(failure.resolved,true);
});
test('检查与完成只解除对应通道；完成还须匹配候选版本',()=>{
  const m=M.create(now),failed={stage:'applying',channel:'stable',revision:'0.1.10'};
  m.emit('update.failed','manager',now,failed);const failure=m.state.notifications[0];
  m.emit('update.complete','manager',now+1,{...failed,channel:'beta'});assert.equal(failure.resolved,false);
  m.emit('update.complete','manager',now+2,{...failed,revision:'0.1.11'});assert.equal(failure.resolved,false);
  m.emit('update.complete','manager',now+3,failed);assert.equal(failure.resolved,true);
  m.emit('update.failed','manager',now+4,{stage:'checking',channel:'stable'});const check=m.state.notifications[0];
  m.emit('update.latest','manager',now+5,{stage:'checking',channel:'beta'});assert.equal(check.resolved,false);
  m.emit('update.latest','manager',now+6,{channel:'stable'});assert.equal(check.resolved,false);
  m.emit('update.latest','manager',now+7,{stage:'checking',channel:'stable'});assert.equal(check.resolved,true);
});
test('切通道保留面板成功缓存、已有确认和失败退避；策略触发仅查管理器',()=>{
  for(const outcome of ['available','failed']){
    const m=M.create(now),token=m.begin('runtime');m.finish('runtime',token,outcome,now);
    const snapshot=JSON.stringify(m.state.updates.runtime);
    assert.equal(m.setUpdatePolicy('beta',now+1),true);
    assert.equal(JSON.stringify(m.state.updates.runtime),snapshot);
    assert.deepEqual(m.requestChecks('policy.changed',now+day).map(r=>r.target),['manager']);
    if(outcome==='available')assert.notEqual(m.confirmApply('runtime',token,'2.51.0',false,now+2),null);
    else assert.equal(m.requestChecks('network.online',now+299999).length,0);
  }
});
test('选择当前通道无副作用；管理器写入与待重启阻止切换',()=>{
  const m=M.create(now);m.begin('manager');const snapshot=JSON.stringify(m.state);
  assert.equal(m.setUpdatePolicy('stable',now+1),false);assert.equal(JSON.stringify(m.state),snapshot);
  m.cancel('manager',now);m.setScenario('manager','available');const token=m.begin('manager','applying');
  m.progress('manager',token,'installing');assert.equal(m.setUpdatePolicy('beta',now+2),false);
  m.finish('manager',token,'restart-required',now+3);assert.equal(m.setUpdatePolicy('beta',now+4),false);
});
test('退出手动模式保留面板原到期时间，不绕过退避或补跑',()=>{
  const m=M.create(now);m.finish('runtime',m.begin('runtime'),'failed',now);
  const due=m.state.updates.runtime.nextDue;
  m.setUpdatePolicy('manual',now+1);m.setUpdatePolicy('stable',now+2);
  assert.equal(m.state.updates.runtime.nextDue,due);
  assert.deepEqual(m.requestChecks('policy.changed',now+2).map(r=>r.target),['manager']);
  assert.equal(m.requestChecks('network.online',due-1).length,0);
  assert.deepEqual(m.requestChecks('retry.due',due).map(r=>r.target),['runtime']);
});
