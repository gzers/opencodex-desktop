const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
const p=h=>h.state().draft.providers.find(p=>p.id==='newapi');
test('额度默认继承渠道Key和节点，原测试窗口从操作列打开',()=>{
 const h=make();assert.ok(h.page().includes('<span>额度</span>'));assert.doesNotMatch(h.page(),/mp-provider-node-cell/);
 assert.match(h.page(),/Key 未设上限/);assert.match(h.page(),/账户余额未提供/);
 h.click('node-picker-open',{channel:'newapi'});assert.equal(h.state().nodePickerChannel,'newapi');
 h.click('node-picker-close');h.click('edit-provider',{id:'newapi'});
 assert.match(h.page(),/额度获取/);assert.match(h.page(),/留空沿用渠道 API Key/);assert.equal(p(h).quotaConfig,undefined);
});
test('额度查询只更新观察，不修改渠道配置或触发模型检测',()=>{
 const h=make(),before=JSON.stringify({draft:h.state().draft,saved:h.state().saved});
 h.click('quota-refresh',{channel:'newapi'});assert.equal(h.state().quotaObservations.newapi.state,'loading');h.drain();
 assert.equal(h.state().quotaObservations.newapi.state,'unlimited');assert.equal(JSON.stringify({draft:h.state().draft,saved:h.state().saved}),before);
 assert.equal(Object.keys(h.state().nodeSpeedResults).length,0);
 h.click('quota-refresh',{channel:'aimami'});h.drain();assert.equal(h.state().quotaObservations.aimami.state,'unsupported');
});
test('留空默认项和重复change不制造修改，不重建查询按钮或独立凭据',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});
 h.event('input','[data-quota-config]',{quotaConfig:'url'},'');assert.equal(p(h).quotaConfig,undefined);
 h.event('input','[data-quota-config]',{quotaConfig:'key'},'mock-private-key');const ref=p(h).quotaConfig.credentialRef;
 h.event('change','[data-quota-config]',{quotaConfig:'key'},'mock-private-key');assert.equal(p(h).quotaConfig.credentialRef,ref);
 h.click('quota-reset');assert.equal(p(h).quotaConfig,undefined);
});
test('独立额度配置不修改模型连接，秘密仅为不透明引用，晚回不覆盖新配置',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});const connection=JSON.stringify(p(h).connection);
 h.event('input','[data-quota-config]',{quotaConfig:'url'},'https://balance.example.test/quota');
 h.event('input','[data-quota-config]',{quotaConfig:'key'},'mock-only-independent-secret');
 assert.equal(p(h).quotaConfig.url,'https://balance.example.test/quota');assert.match(p(h).quotaConfig.credentialRef,/^mock-key:/);
 assert.doesNotMatch(JSON.stringify(h.state()),/mock-only-independent-secret/);assert.equal(JSON.stringify(p(h).connection),connection);
 h.click('quota-refresh',{channel:'newapi'});h.event('input','[data-quota-config]',{quotaConfig:'url'},'https://other.example.test/quota');h.drain();
 assert.equal(h.state().quotaObservations.newapi.state,'idle');h.click('quota-reset');assert.equal(p(h).quotaConfig,undefined);assert.equal(JSON.stringify(p(h).connection),connection);
});
test('无效返回路径阻止额度测试，字段修正后查询及当前渠道保存保持隔离',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});
 h.event('input','[data-quota-config]',{quotaConfig:'url'},'https://balance.example.test/quota');
 h.event('input','[data-quota-config]',{quotaConfig:'remainingPath'},'data.balance()');
 h.click('quota-refresh',{channel:'newapi'});assert.notEqual(h.state().quotaObservations.newapi.state,'loading');
 h.event('input','[data-quota-config]',{quotaConfig:'remainingPath'},'data.balance');h.click('quota-refresh',{channel:'newapi'});h.drain();
 assert.equal(h.state().quotaObservations.newapi.state,'success');assert.equal(h.state().saved.providers[0].quotaConfig,undefined);
 h.click('save');h.drain();assert.equal(h.state().saved.providers[0].quotaConfig.remainingPath,'data.balance');
});
test('复制渠道保留查询格式但不隐式复制独立查询凭据',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});
 h.event('input','[data-quota-config]',{quotaConfig:'url'},'https://balance.example.test/quota');
 h.event('input','[data-quota-config]',{quotaConfig:'key'},'mock-only-query-secret');
 h.click('back-channels');h.click('copy-provider',{id:'newapi'});
 h.nodes.get('mpCopyProviderName').value='额度副本';h.nodes.get('mpCopyProviderId').value='quota-copy';h.runConfirm();
 const copy=h.state().draft.providers.find(p=>p.id==='quota-copy');
 assert.equal(copy.quotaConfig.url,'https://balance.example.test/quota');assert.equal(copy.quotaConfig.credentialRef,undefined);
 assert.doesNotMatch(JSON.stringify(h.state()),/mock-only-query-secret/);
});
