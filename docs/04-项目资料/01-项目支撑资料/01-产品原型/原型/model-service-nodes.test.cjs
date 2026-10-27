/* 服务节点共享认证与显式保存边界；最小 DOM 不代替浏览器布局。 */
const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
const p=(h,kind='draft',id='newapi')=>h.state()[kind].providers.find(p=>p.id===id);
const edit=(h,name,url)=>{h.nodes.get('mpServiceNodeName').value=name;h.nodes.get('mpServiceNodeURL').value=url;h.runConfirm();};
test('服务节点取代协议端点，模型无地址选择，所有协议共用同一节点',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});assert.match(h.page(),/02<\/span><span class="mp-nav-label">服务节点/);assert.doesNotMatch(h.page(),/协议端点|指定端点|mpChannelInline-baseUrl|add-endpoint/);assert.equal(p(h).connection.baseUrl,undefined);
 h.click('expand-model',{id:'newapi::0'});assert.doesNotMatch(h.page(),/data-quick-endpoint/);h.click('model-config',{id:'newapi::0'});assert.doesNotMatch(h.dialog(),/mpChannelModelEndpoint/);h.context.closeModal();
 assert.equal(h.proto.commitModelPatches('newapi',[{key:'newapi::6',rules:{protocol:'Anthropic'}}]),'');assert.equal(h.proto.resolveRoute('newapi','newapi::0').nodeId,'primary');assert.equal(h.proto.resolveRoute('newapi','newapi::6').nodeId,'primary');assert.equal(h.proto.resolveRoute('newapi','newapi::6').protocol,'Anthropic');assert.equal(h.proto.resolveRoute('newapi','newapi::6').error,'');
 const baseline=JSON.stringify(h.state().draft);assert.match(h.proto.commitModelPatches('newapi',[{key:'newapi::0',nodeId:'backup'}]),/不单独指定/);assert.equal(JSON.stringify(h.state().draft),baseline);
});
test('新增节点只有名称与URL，取消不写，加入草稿不复制认证或模型',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});const auth=JSON.stringify(p(h).connection),models=JSON.stringify(p(h).models),saved=JSON.stringify(h.state().saved);
 h.click('add-node');assert.doesNotMatch(h.dialog(),/type="password"|mpChannelEndpointAuth|mpChannelEndpointProtocol/);h.nodes.get('mpServiceNodeName').value='取消节点';h.context.closeModal();assert.equal(p(h).nodes.length,2);
 h.click('add-node');edit(h,'国内节点','https://china.example.com/v1/');const node=p(h).nodes.at(-1);assert.equal(node.url,'https://china.example.com/v1');assert.deepEqual(Object.keys(node).sort(),['id','name','url']);assert.equal(p(h).activeNodeId,'primary');assert.equal(JSON.stringify(p(h).connection),auth);assert.equal(JSON.stringify(p(h).models),models);assert.equal(JSON.stringify(h.state().saved),saved);
});
test('手动切换进入草稿，保存前Agent仍用原配置；差异列出地址且保存后保留节点列表',()=>{
 const h=make();h.configureAgent();h.click('edit-provider',{id:'newapi'});const models=JSON.stringify(p(h).models),auth=JSON.stringify(p(h).connection),nodes=JSON.stringify(p(h).nodes),beforeIds=h.proto.routePlan().groups.map(g=>g.id);
 h.click('use-node',{id:'backup'});assert.equal(p(h).activeNodeId,'backup');assert.equal(p(h,'saved').activeNodeId,'primary');assert.equal(JSON.stringify(p(h).models),models);assert.equal(JSON.stringify(p(h).connection),auth);assert.equal(JSON.stringify(p(h).nodes),nodes);assert.equal(h.proto.resolveRoute('newapi','newapi::0').nodeId,'backup');assert.deepEqual(h.proto.routePlan().groups.map(g=>g.id),beforeIds);assert.equal(h.proto.agentInfo().p.activeNodeId,'primary');
 h.click('save');assert.match(h.dialog(),/当前节点 URL/);assert.match(h.dialog(),/backup.newapi.example.com/);assert.match(h.dialog(),/newapi.example.com/);h.runConfirm();h.drain();assert.equal(p(h,'saved').activeNodeId,'backup');assert.equal(h.state().pending,false);assert.equal(p(h,'saved').nodes.length,2);assert.equal(h.proto.agentInfo().p.activeNodeId,'backup');
});
test('编辑当前节点URL保留身份、认证、选择和规则，过期节点编辑拒绝覆盖',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});const auth=JSON.stringify(p(h).connection),models=JSON.stringify(p(h).models);h.click('edit-node',{id:'primary'});edit(h,'主站','https://renamed.example.com/v1');assert.equal(p(h).nodes[0].id,'primary');assert.equal(p(h).activeNodeId,'primary');assert.equal(JSON.stringify(p(h).connection),auth);assert.equal(JSON.stringify(p(h).models),models);assert.equal(p(h).availability.kind,'unknown');assert.equal(p(h,'saved').nodes[0].url,'https://newapi.example.com/v1');
 h.click('edit-node',{id:'primary'});h.click('toggle-model',{id:'newapi::0'});edit(h,'过期名称','https://stale.example.com/v1');assert.match(h.nodes.get('mpServiceNodeError').textContent,/已变化/);assert.equal(p(h).nodes[0].name,'主站');
});
test('节点URL拒绝重复、带凭据和生成接口；名称与地址错误不改变配置',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});const baseline=JSON.stringify(h.state().draft);
 for(const url of ['https://newapi.example.com/v1/','https://user:secret@example.com/v1','https://x.example/v1?key=secret','https://x.example/v1/responses','https://x.example/v1/messages','file:///tmp/local']){
  h.click('add-node');edit(h,'测试节点',url);assert.ok(h.nodes.get('mpServiceNodeError').textContent);assert.equal(JSON.stringify(h.state().draft),baseline);h.context.closeModal();
 }
 h.click('add-node');edit(h,'默认节点','https://unique.example.com');assert.match(h.nodes.get('mpServiceNodeError').textContent,/已存在/);assert.equal(JSON.stringify(h.state().draft),baseline);
});
test('当前节点与最后一条不可删除，非当前删除可取消；旧删除确认失效',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});h.click('delete-node',{id:'primary'});assert.equal(p(h).nodes.length,2);h.click('delete-node',{id:'backup'});h.context.closeModal();assert.equal(p(h).nodes.length,2);
 h.click('delete-node',{id:'backup'});h.click('toggle-model',{id:'newapi::0'});h.runConfirm();assert.match(h.nodes.get('mpServiceNodeError').textContent,/已变化/);assert.equal(p(h).nodes.length,2);h.context.closeModal();h.click('delete-node',{id:'backup'});h.runConfirm();assert.equal(p(h).nodes.length,1);h.click('delete-node',{id:'primary'});assert.equal(p(h).nodes.length,1);assert.equal(p(h,'saved').nodes.length,2);
});
test('未确认认证与保存中锁定节点修改，晚到的清单结果不会覆盖已切节点',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});h.event('input','[data-channel-config]',{channelConfig:'name'},'待确认名称');h.click('use-node',{id:'backup'});assert.equal(p(h).activeNodeId,'primary');h.click('reset-channel-connection');
 h.click('refresh');h.click('use-node',{id:'backup'});h.drain();assert.equal(p(h).activeNodeId,'backup');assert.equal(p(h).refresh,'idle');h.click('save');h.runConfirm();h.click('use-node',{id:'primary'});h.click('delete-node',{id:'primary'});assert.equal(p(h).activeNodeId,'backup');assert.equal(p(h).nodes.length,2);h.drain();assert.equal(p(h,'saved').activeNodeId,'backup');
});
test('返回不保存，全部草稿丢弃还原节点，保存记录恢复先生成旧节点草稿',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});h.click('use-node',{id:'backup'});h.click('back-channels');assert.equal(p(h).activeNodeId,'backup');assert.equal(p(h,'saved').activeNodeId,'primary');h.click('discard');h.runConfirm();assert.equal(p(h).activeNodeId,'primary');
 h.click('edit-provider',{id:'newapi'});h.click('use-node',{id:'backup'});h.click('save');h.runConfirm();h.drain();h.click('restore',{id:String(h.state().records.at(-1).id)});h.runConfirm();assert.equal(p(h).activeNodeId,'primary');assert.equal(p(h,'saved').activeNodeId,'backup');assert.equal(p(h).nodes.length,2);
});
test('新增渠道的BaseURL只创建一条默认节点，后续认证编辑不重复创建或改节点',()=>{
 const h=make();h.click('new-channel');for(const [id,value] of [['mpChannelName','新渠道'],['mpChannelId','new-provider'],['mpChannelURL','https://new.example.com/v1'],['mpChannelKey','demo-only']])h.nodes.get(id).value=value;
 h.runConfirm();h.nodes.get('mpChannelManual').value='test-model';h.click('channel-manual');h.event('change','[data-channel-model]',{channelModel:'test-model'},'',true);h.runConfirm();const next=p(h,'draft','new-provider');assert.equal(next.activeNodeId,'primary');assert.deepEqual(next.nodes,[{id:'primary',name:'默认节点',url:'https://new.example.com/v1'}]);assert.equal(next.connection.baseUrl,undefined);assert.equal(next.connection.keyConfigured,true);
 h.event('input','[data-channel-config]',{channelConfig:'name'},'新渠道改名');h.click('apply-channel-connection');assert.equal(p(h,'draft','new-provider').name,'新渠道改名');assert.deepEqual(p(h,'draft','new-provider').nodes,next.nodes);
});
test('同Key无额度状态不会因新增或切换URL节点被解除',()=>{
 const h=make();h.click('edit-provider',{id:'a6api'});h.click('add-node');edit(h,'备用线路','https://backup.a6api.example.com/v1');h.click('use-node',{id:p(h,'draft','a6api').nodes.at(-1).id});assert.equal(p(h,'draft','a6api').availability.kind,'no-credit');assert.equal(p(h,'draft','a6api').availability.source,'manual');
});
