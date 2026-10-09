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
  assert.equal(m.state.notifications.find(i=>i.id==='backup.failed').resolved,true);
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
  const m=M.create(now);m.emit('update.failed','manager',now);
  assert.equal(m.emit('update.failed','manager',now+1),null);
  assert.equal(m.state.deliveries.length,1);
  assert.equal(m.state.notifications[0].occurrences,2);
  assert.ok(m.emit('update.failed','manager',now+day));
  assert.equal(m.state.notifications.length,1);
  m.emit('update.latest','manager',now+day+1);
  const failed=m.state.notifications.find(i=>i.id==='update.failed');
  assert.equal(failed.resolved,true);assert.equal(failed.read,false);
  m.emit('update.failed','manager',now+day+2);
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
  const m=M.create(now);m.setUpdatePolicy('beta',now);
  const batch=m.requestChecks('policy.changed',now);
  for(const r of batch)m.finish(r.target,r.generation,'latest',now);
  const auto=m.requestChecks('schedule.due',now+6*3600000);assert.deepEqual(auto.map(r=>r.target),['manager']);
  m.finish('manager',auto[0].generation,'latest',now+6*3600000);
  assert.equal(m.requestChecks('user.action',now+6*3600000+1).length,2);
});
test('手动策略禁止全部自动源；切通道取消旧结果',()=>{
  const m=M.create(now),old=m.requestChecks('user.action',now);
  m.setUpdatePolicy('manual',now+1);
  for(const r of old)assert.equal(m.finish(r.target,r.generation,'available',now+2),false);
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
  m.emit('update.failed','manager',now,{stage:'applying'});
  m.emit('update.failed','manager',now,{stage:'checking'});
  m.emit('update.available','manager',now+1);
  assert.equal(m.state.notifications.find(n=>n.id==='update.failed'&&n.stage==='applying').resolved,false);
  assert.equal(m.state.notifications.find(n=>n.id==='update.failed'&&n.stage==='checking').resolved,true);
  m.emit('update.complete','manager',now+2);
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
