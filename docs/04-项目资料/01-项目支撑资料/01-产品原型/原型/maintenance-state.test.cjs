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
