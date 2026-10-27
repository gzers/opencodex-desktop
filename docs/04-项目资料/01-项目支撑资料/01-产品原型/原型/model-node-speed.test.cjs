/* 节点测速只产生会话观察，不变更策略；确定性mock不发网络请求。 */
const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
const p=h=>h.state().draft.providers.find(p=>p.id==='newapi');
const policy=h=>JSON.stringify({draft:h.state().draft,saved:h.state().saved,pending:h.state().pending,records:h.state().records});
const result=(h,id)=>h.state().nodeSpeedResults['newapi::'+id];
function add(h,name,url){h.click('add-node');h.nodes.get('mpServiceNodeName').value=name;h.nodes.get('mpServiceNodeURL').value=url;h.runConfirm();return p(h).nodes.at(-1).id;}
test('列表节点选择与测速可直接操作，不点击行进入详情；选择仍先入草稿',()=>{
 const h=make(),saved=JSON.stringify(h.state().saved);assert.match(h.page(),/data-mp="node-picker-open" data-channel="newapi"/);assert.doesNotMatch(h.page(),/data-mp="node-speed-current"|data-mp="node-speed-all"|data-mp="node-picker-open" data-channel="native"/);
 h.click('node-picker-open',{channel:'newapi'});assert.match(h.nodes.get('mpNodePicker').innerHTML,/data-mp="node-speed-one"/);assert.match(h.nodes.get('mpNodePicker').innerHTML,/data-mp="node-speed-all"/);h.click('node-picker-choose',{channel:'newapi',id:'backup'});assert.equal(p(h).activeNodeId,'backup');assert.equal(h.state().channelDetail,false);assert.equal(JSON.stringify(h.state().saved),saved);assert.equal(h.state().operation,null);
 h.click('node-speed-current',{channel:'newapi'});assert.equal(result(h,'backup').status,'running');h.drain();assert.equal(result(h,'backup').status,'success');assert.equal(result(h,'primary'),undefined);
});
test('单节点测速不会改变选择、清单、连接状态或保存差异；只显示接口耗时',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});const baseline=policy(h);h.click('node-speed-one',{channel:'newapi',id:'backup'});assert.equal(result(h,'backup').status,'running');h.drain();assert.equal(result(h,'backup').status,'success');assert.ok(result(h,'backup').ms>0);assert.ok(result(h,'backup').finishedAt);assert.equal(policy(h),baseline);assert.equal(p(h).activeNodeId,'primary');
 h.proto.render();assert.match(h.page(),new RegExp(result(h,'backup').ms+' ms'));assert.match(h.page(),/data-speed-status="success"/);assert.match(h.page(),/接口响应耗时/);
});
test('一键覆盖当前渠道全部节点，最多两项并行；完成不自动选择或保存',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});const third=add(h,'第三节点','https://third.example.com/v1'),baseline=policy(h);h.click('node-speed-all',{channel:'newapi'});
 const run=h.state().nodeSpeedRuns.newapi;assert.equal(run.items.length,3);assert.equal(run.items.filter(i=>i.status==='running').length,2);assert.equal(result(h,third).status,'waiting');assert.equal(h.state().nodeSpeedRuns.a6api,undefined);h.drain();assert.equal(h.state().nodeSpeedRuns.newapi.status,'complete');assert.ok(h.state().nodeSpeedRuns.newapi.items.every(i=>i.status==='success'));assert.equal(policy(h),baseline);assert.equal(p(h).activeNodeId,'primary');
});
test('部分失败与超时分别展示，失败不携带伪造耗时或改变渠道认证 / 额度',()=>{
 const h=make(),baseline=policy(h);h.proto.setNodeSpeedScene('mixed');h.click('node-speed-all',{channel:'newapi'});h.drain();assert.equal(result(h,'primary').status,'success');assert.equal(result(h,'backup').status,'failed');assert.equal(result(h,'backup').ms,undefined);
 h.proto.setNodeSpeedScene('timeout');h.click('node-speed-all',{channel:'newapi'});h.drain();assert.equal(result(h,'primary').status,'timeout');assert.equal(result(h,'primary').ms,undefined);assert.equal(policy(h),baseline);
 h.click('node-speed-all',{channel:'a6api'});h.drain();assert.equal(h.state().draft.providers.find(p=>p.id==='a6api').availability.kind,'no-credit');
});
test('停止与重新测速使旧回调无效，测速场景切换保留草稿',()=>{
 const h=make();h.click('node-picker-open',{channel:'newapi'});h.click('node-picker-choose',{channel:'newapi',id:'backup'});const baseline=policy(h);h.click('node-speed-all',{channel:'newapi'});h.click('node-speed-stop',{channel:'newapi'});assert.equal(h.state().nodeSpeedRuns.newapi.status,'cancelled');h.drain();assert.equal(result(h,'primary').status,'cancelled');
 h.click('node-speed-one',{channel:'newapi',id:'primary'});h.proto.setNodeSpeedScene('mixed');h.click('node-speed-one',{channel:'newapi',id:'primary'});h.drain();assert.equal(result(h,'primary').status,'failed');assert.equal(result(h,'backup'),undefined);assert.equal(policy(h),baseline);
});
test('测速晚回绑定节点地址，编辑URL后结果失效；重测只采用新地址',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});h.click('node-speed-one',{channel:'newapi',id:'primary'});h.click('edit-node',{id:'primary'});h.nodes.get('mpServiceNodeName').value='默认节点';h.nodes.get('mpServiceNodeURL').value='https://changed.example.com/v1';h.runConfirm();h.drain();assert.equal(h.state().nodeSpeedRuns.newapi.items[0].status,'stale');
 h.proto.render();assert.match(h.nodes.get('mpNodeSpeed-newapi-primary').innerHTML||h.page(),/data-speed-status="untested"/);h.click('node-speed-one',{channel:'newapi',id:'primary'});h.drain();assert.equal(result(h,'primary').status,'success');assert.match(result(h,'primary').fingerprint,/changed.example.com/);
});
test('未确认认证阻止测速；测速更新不重建未保存的连接输入或中断弹窗',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});h.event('input','[data-channel-config]',{channelConfig:'name'},'未确认名字');h.click('node-speed-all',{channel:'newapi'});assert.equal(h.state().nodeSpeedRuns.newapi,undefined);h.click('reset-channel-connection');h.click('node-speed-all',{channel:'newapi'});
 h.event('input','[data-channel-config]',{channelConfig:'name'},'测试期间名字');const field=h.nodes.get('mpChannelInline-name');field.value='测试期间名字';h.click('save');const dialog=h.dialog();h.drain();assert.equal(h.nodes.get('mpChannelInline-name'),field);assert.equal(field.value,'测试期间名字');assert.equal(h.state().channelConnections.newapi.name,'测试期间名字');assert.equal(h.dialog(),dialog);assert.equal(h.nodes.get('modalMask').style.display,'flex');
});
test('共用Key更换后旧测速失效；一键结果不写入渠道JSON或保存记录',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});h.click('node-speed-all',{channel:'newapi'});h.drain();assert.equal(result(h,'primary').status,'success');h.event('input','[data-channel-config]',{channelConfig:'key'},'demo-replacement');h.click('apply-channel-connection');
 h.proto.render();assert.match(h.page(),/data-speed-status="untested"/);assert.doesNotMatch(JSON.stringify(h.state().draft),/nodeSpeed|finishedAt|demo-replacement/);
});
test('部分完成后停止保留已完成结果，未完成项不计为完成，也不会被晚回覆盖',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});add(h,'第三节点','https://third.example.com/v1');const queue=[];h.context.setTimeout=fn=>queue.push(fn);h.click('node-speed-all',{channel:'newapi'});queue.shift()();assert.equal(result(h,'primary').status,'success');h.click('node-speed-stop',{channel:'newapi'});
 while(queue.length)queue.shift()();assert.equal(result(h,'primary').status,'success');assert.equal(result(h,'backup').status,'cancelled');const items=h.state().nodeSpeedRuns.newapi.items;assert.equal(items.filter(i=>i.status==='success').length,1);h.proto.render();assert.match(h.page(),/测速已停止 1 \/ 3/);
});
test('删除后重建同ID同URL渠道也不复用旧认证的测速结果或晚回',()=>{
 const h=make();h.click('node-speed-current',{channel:'newapi'});h.click('delete-provider',{id:'newapi'});h.runConfirm();h.click('new-channel');for(const [id,value] of [['mpChannelName','NewApi 重建'],['mpChannelId','newapi'],['mpChannelURL','https://newapi.example.com/v1'],['mpChannelKey','new-demo-key']])h.nodes.get(id).value=value;
 h.runConfirm();h.nodes.get('mpChannelManual').value='test-model';h.click('channel-manual');h.event('change','[data-channel-model]',{channelModel:'test-model'},'',true);h.runConfirm();h.drain();assert.equal(h.state().nodeSpeedRuns.newapi.items[0].status,'stale');h.proto.render();assert.match(h.page(),/data-speed-status="untested"/);
});

/* 仅检查 mock 路径行为；没有真实测速证据。 */
function configure(h,url,protocol='Anthropic'){
 h.click('edit-provider',{id:'newapi'});h.event('change','[data-channel-config]',{channelConfig:'protocol'},protocol);h.click('apply-channel-connection');
 h.click('edit-node',{id:'primary'});h.nodes.get('mpServiceNodeName').value='默认节点';h.nodes.get('mpServiceNodeURL').value=url;h.runConfirm();
}
test('根 / 版本 / 完整清单 / 自定义前缀共用自动补齐，测速与获取清单保持同一结果且不改基址',()=>{
 for(const [protocol,input,path] of [
  ['Anthropic','https://service.example','/v1/models'],
  ['Responses','https://service.example/v1/','/v1/models'],
  ['Chat','https://service.example/v1/models','/v1/models'],
  ['Anthropic','https://service.example/proxy/v1','/proxy/v1/models'],
  ['Responses','https://service.example/proxy','/proxy/v1/models'],
  ['Google','https://service.example','/v1beta/models'],
  ['Ollama','https://service.example','/api/tags']
 ]){
  const h=make();configure(h,input,protocol);const savedURL=h.state().draft.providers[0].nodes[0].url;
  h.click('node-speed-one',{channel:'newapi',id:'primary'});h.drain();assert.equal(result(h,'primary').status,'success');assert.equal(result(h,'primary').path,'https://service.example'+path);assert.equal(result(h,'primary').count,8);
  h.click('refresh');h.drain();assert.equal(h.state().discoveryResults.newapi.path,result(h,'primary').path);assert.equal(h.state().draft.providers[0].nodes[0].url,savedURL);
  assert.doesNotMatch(result(h,'primary').path,/v1\/v1|models\/models/);
 }
});
test('HTML 与路径不存在有限同源回退，保留代理前缀；只得到有效清单才有耗时',()=>{
 for(const [scene,mode] of [['discovery-html','html-fallback'],['discovery-path','path-fallback']]){
  const h=make();h.proto.scene(scene);configure(h,'https://service.example:8443/proxy/v1');h.proto.setNodeSpeedScene(mode);
  h.click('node-speed-one',{channel:'newapi',id:'primary'});h.drain();const r=result(h,'primary');assert.equal(r.status,'success');assert.equal(r.attemptedPaths.length,2);
  assert.ok(r.attemptedPaths.every(p=>p.startsWith('https://service.example:8443/proxy/')));assert.equal(r.path,'https://service.example:8443/proxy/models');assert.ok(r.ms>0);
  h.click('refresh');h.drain();assert.deepEqual(h.state().discoveryResults.newapi.attemptedPaths,r.attemptedPaths);
 }
});
test('认证 / DNS / TLS / 超时 / 无效清单不重试路径且不伪造耗时',()=>{
 for(const mode of ['auth','dns','tls','timeout','invalid-list']){
  const h=make();h.proto.setNodeSpeedScene(mode);h.click('node-speed-one',{channel:'newapi',id:'primary'});h.drain();const r=result(h,'primary');
  assert.notEqual(r.status,'success');assert.equal(r.ms,undefined);assert.equal(r.attemptedPaths.length,1);assert.ok(r.reason);
 }
});
