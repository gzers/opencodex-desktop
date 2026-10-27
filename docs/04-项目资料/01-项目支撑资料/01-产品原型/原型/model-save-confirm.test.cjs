/* 确认保存即关闭：通过则直接开始写入；被门禁 / 基准变化拦下也关闭，改为登记已注册通知。 */
const {test} = require('node:test'), assert = require('node:assert/strict')
const make = require('./model-prototype-test-harness.cjs')
const open = h => { h.click('save'); assert.equal(h.nodes.get('modalMask').style.display, 'flex') }

test('确认保存后弹窗立即关闭并开始写入，不再停留在弹窗里看进度', () => {
  const h = make()
  h.click('toggle-provider', { id: 'aimami' })
  open(h)
  assert.equal(h.nodes.get('modalConfirm').disabled, false)
  h.runConfirm()
  assert.notEqual(h.nodes.get('modalMask').style.display, 'flex')
  assert.equal(h.state().operation.status, 'running')
})

test('没有可写入修改时确认仍可点，点了关闭并登记“模型配置未保存”通知', () => {
  const h = make(), blocked = []
  h.context.window.modelSaveBlockedBridge = payload => blocked.push(payload)
  open(h)
  assert.equal(h.nodes.get('modalConfirm').disabled, false)
  assert.match(h.nodes.get('modalConfirm').title, /模型配置未保存/)
  h.runConfirm()
  assert.notEqual(h.nodes.get('modalMask').style.display, 'flex')
  assert.equal(blocked.length, 1)
  assert.match(blocked[0].reason, /没有未保存的修改/)
  assert.equal(blocked[0].key, 'gated')
  assert.equal(h.state().operation, null)
})

test('无效连接字段在全局保存时关闭预览并登记原因，保留当前编辑',()=>{
 const h=make(),blocked=[];h.context.window.modelSaveBlockedBridge=payload=>blocked.push(payload);h.click('edit-provider',{id:'newapi'});h.event('input','[data-channel-config]',{channelConfig:'name'},'');h.click('back-channels');open(h);h.runConfirm();
 assert.notEqual(h.nodes.get('modalMask').style.display,'flex');assert.equal(blocked.length,1);assert.match(blocked[0].reason,/渠道名称/);assert.equal(h.state().draft.providers[0].name,'');assert.equal(h.state().saved.providers[0].name,'NewApi');assert.equal(h.state().operation,null);
});

test('超出激活预算时同样关闭并登记通知，不写入运行配置', () => {
  const h = make(), blocked = []
  h.context.window.modelSaveBlockedBridge = payload => blocked.push(payload)
  h.proto.scene('over-budget')
  open(h)
  h.runConfirm()
  assert.notEqual(h.nodes.get('modalMask').style.display, 'flex')
  assert.match(blocked[0].reason, /超出 100/)
  assert.equal(h.state().operation, null)
})

test('在预览里先启停再确认，第一次就能保存且写入的就是刚才那次改动', () => {
  const h = make()
  h.click('save')
  h.click('preview-channel-toggle', { id: 'aimami' })
  h.click('preview-model-toggle', { id: 'newapi::0' })
  assert.equal(h.state().operation, null)
  h.runConfirm()
  assert.equal(h.state().operation.status, 'running')
  const plan = h.state().operation.snapshot
  assert.equal(plan.providers.find(p => p.id === 'aimami').enabled, false)
  assert.equal(plan.providers.find(p => p.id === 'newapi').models.find(m => m.key === 'newapi::0').selected, false)
  h.drain()
  assert.equal(h.state().saved.providers.find(p => p.id === 'aimami').enabled, false)
})

test('打开后改回原样再确认，按没有未保存修改拦下', () => {
  const h = make(), blocked = []
  h.context.window.modelSaveBlockedBridge = payload => blocked.push(payload.reason)
  h.click('save')
  h.click('preview-channel-toggle', { id: 'aimami' })
  h.click('preview-channel-toggle', { id: 'aimami' })
  h.runConfirm()
  assert.deepEqual(blocked, ['没有未保存的修改，本次确认未写入'])
  assert.equal(h.state().operation, null)
})

test('弹窗正文不再含内联错误块，失败原因只走通知', () => {
  const h = make()
  open(h)
  assert.doesNotMatch(h.dialog(), /mpSaveError/)
  assert.doesNotMatch(h.dialog(), /确认后会先备份/)
})

const edit=(h,value)=>{h.click('edit-provider',{id:'newapi'});h.event('input','[data-channel-config]',{channelConfig:'key'},value);};
test('只改Key直接点保存：离线不重新选模型，冻结凭据引用且明文不进入预览或记录',()=>{
 const h=make();h.proto.scene('discovery-fail');const old=h.state().saved.providers[0],models=JSON.stringify(old.models);
 edit(h,'demo-direct-key');h.click('save');assert.notEqual(h.nodes.get('modalMask').style.display,'flex');assert.equal(h.state().operation.status,'running');assert.notEqual(h.state().operation.snapshot.providers[0].connection.credentialRef,old.connection.credentialRef);h.drain();
 assert.equal(h.state().operation.status,'written');assert.equal(JSON.stringify(h.state().saved.providers[0].models),models);assert.doesNotMatch(JSON.stringify(h.state()),/demo-direct-key/);
 h.click('back-channels');h.click('edit-provider',{id:'newapi'});assert.equal(h.nodes.get('mpChannelInline-key').value,'demo-direct-key');assert.equal(h.state().channelConnections.newapi.keyVisible,false);
});
test('空输入保持原值；明确清空可直接保存，重开为空，不需要清单接口成功',()=>{
 const h=make(),old=h.state().saved.providers[0].connection.credentialRef;edit(h,'');assert.equal(h.state().draft.providers[0].connection.credentialRef,old);assert.equal(h.state().operation,null);assert.match(h.page().match(/<button id="mpPolicySave"[^>]*>/)[0],/disabled/);
 h.click('clear-channel-key');h.click('save');h.drain();assert.equal(h.state().operation.status,'written');assert.equal(h.state().saved.providers[0].connection.keyConfigured,false);
 h.click('back-channels');h.click('edit-provider',{id:'newapi'});assert.equal(h.nodes.get('mpChannelInline-key').value,'');
});
test('Key编辑在失败时不改已保存值，全局还原回原Key且无局部还原入口',()=>{
 const h=make();h.proto.scene('backup-fail');const old=h.state().saved.providers[0].connection.credentialRef;edit(h,'demo-not-committed');h.click('save');h.drain();
 assert.equal(h.state().operation.status,'failed');assert.equal(h.state().saved.providers[0].connection.credentialRef,old);
 h.click('back-channels');h.click('discard');h.runConfirm();h.click('edit-provider',{id:'newapi'});assert.equal(h.nodes.get('mpChannelInline-key').value,'demo-newapi-key');
 h.event('input','[data-channel-config]',{channelConfig:'key'},'demo-reset');assert.doesNotMatch(h.page(),/data-mp="reset-channel-connection"/);h.click('back-channels');h.click('discard');h.runConfirm();h.click('edit-provider',{id:'newapi'});assert.equal(h.nodes.get('mpChannelInline-key').value,'demo-newapi-key');
});

test('新渠道加入时Key仅进入内存草稿，页级保存前不改已保存快照，丢弃可全部还原',()=>{
 const h=make(),before=JSON.stringify(h.state().saved);h.click('new-channel');
 for(const [id,value] of [['mpChannelName','离线新渠道'],['mpChannelId','offline-new'],['mpChannelURL','https://offline.example'],['mpChannelKey','demo-new-draft-only']])h.nodes.get(id).value=value;
 h.runConfirm();h.nodes.get('mpChannelManual').value='demo-model';h.click('channel-manual');h.event('change','[data-channel-model]',{channelModel:'demo-model'},'',true);h.runConfirm();
 assert.equal(JSON.stringify(h.state().saved),before);assert.equal(h.state().operation,null);assert.ok(h.state().draft.providers.find(p=>p.id==='offline-new').connection.credentialRef);
 assert.equal(h.nodes.get('mpChannelInline-key').value,'demo-new-draft-only');assert.doesNotMatch(JSON.stringify(h.state()),/demo-new-draft-only/);
 h.click('back-channels');h.click('discard');h.runConfirm();assert.equal(h.state().draft.providers.some(p=>p.id==='offline-new'),false);assert.equal(JSON.stringify(h.state().saved),before);
});
