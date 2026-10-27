/* Scoped detail-save simulation; no real file, API, credentials or release. */
const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
const json=value=>JSON.stringify(value),p=(h,id,source='draft')=>h.state()[source].providers.find(item=>item.id===id);
function edit(h,id,name,key){h.click('edit-provider',{id});if(name!==undefined)h.event('input','[data-channel-config]',{channelConfig:'name'},name);if(key!==undefined)h.event('input','[data-channel-config]',{channelConfig:'key'},key);}
const noModal=h=>assert.notEqual(h.nodes.get('modalMask').style.display,'flex');

test('连接字段直接进入编辑，局部确认与还原消失，获取清单和节点检测保留',()=>{
 const h=make(),before=json(h.state().saved);edit(h,'newapi','直接编辑名称','demo-edited-secret');
 assert.equal(p(h,'newapi').name,'直接编辑名称');assert.equal(h.proto.stagedChannelConnections().length,0);assert.equal(json(h.state().saved),before);
 assert.doesNotMatch(h.page(),/data-mp="(?:apply-channel-connection|reset-channel-connection|review-channel-connection)"/);
 assert.match(h.page(),/data-mp="refresh"/);assert.match(h.page(),/data-mp="node-speed-one"/);
 assert.doesNotMatch(json(h.state()),/demo-edited-secret/);
 h.click('back-channels');edit(h,'newapi');assert.equal(h.nodes.get('mpChannelInline-name').value,'直接编辑名称');assert.equal(h.nodes.get('mpChannelInline-key').value,'demo-edited-secret');
});

test('详情保存不打开完整预览，仅写当前渠道，成功回列表，其它渠道与Key编辑保留',()=>{
 const h=make(),before=h.state().saved,oldOrder=before.providers.map(item=>item.id);edit(h,'newapi','NewApi 待处理','demo-other-key');
 const other=json(p(h,'newapi')),otherBuffer=json(h.state().channelConnections.newapi);edit(h,'aimami','Aimami 已保存','demo-current-key');
 const current=json(p(h,'aimami'));h.click('save');noModal(h);assert.equal(h.state().operation.status,'running');assert.equal(h.state().operation.channelId,'aimami');
 assert.equal(json(h.state().saved),json(before));assert.equal(h.state().channelDetail,true);h.drain();
 assert.equal(h.state().channelDetail,false);assert.equal(json(p(h,'aimami','saved')),current);
 assert.equal(json(p(h,'newapi','saved')),json(before.providers.find(item=>item.id==='newapi')));
 assert.equal(json(p(h,'newapi')),other);assert.equal(json(h.state().channelConnections.newapi),otherBuffer);
 assert.deepEqual(h.state().saved.providers.map(item=>item.id),oldOrder);assert.match(h.page(),/data-channel-state="dirty"/);
 const record=h.state().records.at(-1);assert.equal(json(record.policy),json(h.state().saved));assert.equal(json(record.before),json(before));
 edit(h,'aimami');assert.match(h.page(),/data-status-scope="current-channel"/);assert.match(h.page(),/data-save-gated="true"/);assert.equal(h.nodes.get('mpChannelInline-key').value,'demo-current-key');
 edit(h,'newapi');assert.equal(h.nodes.get('mpChannelInline-key').value,'demo-other-key');
});

test('详情保存后的列表预览只标记剩余修改，取消不撤销已保存渠道',()=>{
 const h=make();edit(h,'newapi','剩余修改');edit(h,'aimami','已单独保存');h.click('save');h.drain();const saved=json(h.state().saved);
 h.click('save');assert.equal(h.nodes.get('modalMask').style.display,'flex');assert.match(h.dialog(),/剩余修改/);assert.match(h.dialog(),/已单独保存/);
 h.context.closeModal();assert.equal(json(h.state().saved),saved);assert.equal(p(h,'newapi').name,'剩余修改');
 h.click('save');h.runConfirm();h.drain();assert.equal(json(h.state().draft),json(h.state().saved));assert.equal(h.state().records.length,3);
});

test('其它渠道的未保存预算超限不阻止当前渠道的有效保存，列表仍保留超限编辑',()=>{
 const h=make();edit(h,'a6api');h.event('change','[data-mode]',{},'all');assert.ok(h.proto.projection().length>100);
 edit(h,'newapi','只存名称');h.click('save');assert.equal(h.state().operation.status,'running');h.drain();assert.equal(p(h,'newapi','saved').name,'只存名称');
 assert.equal(p(h,'a6api','saved').mode,'whitelist');assert.equal(p(h,'a6api').mode,'all');assert.match(h.page(),/data-channel-state="over-budget"/);
});

test('不能借用其它渠道尚未保存的停用来绕过本次实际提交的激活预算',()=>{
 const h=make(),blocked=[];h.context.window.modelSaveBlockedBridge=value=>blocked.push(value);
 h.click('toggle-provider',{id:'a6api'});edit(h,'newapi');h.event('change','[data-mode]',{},'all');const saved=json(h.state().saved);assert.ok(h.proto.projection().length<100);
 h.click('save');noModal(h);assert.equal(h.state().operation,null);assert.equal(h.state().channelDetail,true);assert.equal(json(h.state().saved),saved);assert.equal(blocked.at(-1).key,'channel-budget');
});

test('详情保存不顺带提交全局设置或渠道排序，后续全局保存才采用它们',()=>{
 const h=make();h.proto.scene('incoming');const before=h.state().saved;h.click('move-provider',{id:'aimami',direction:'-1'});const order=h.state().draft.providers.map(item=>item.id);
 edit(h,'newapi','单个渠道');h.click('save');h.drain();assert.equal(h.state().saved.fastMode,before.fastMode);assert.equal(h.state().draft.fastMode,'inherit');
 assert.deepEqual(h.state().saved.providers.map(item=>item.id),before.providers.map(item=>item.id));assert.deepEqual(h.state().draft.providers.map(item=>item.id),order);
 h.click('save');h.runConfirm();h.drain();assert.equal(h.state().saved.fastMode,'inherit');assert.deepEqual(h.state().saved.providers.map(item=>item.id),order);
});

test('详情保存新渠道只新增当前副本，其它待新增渠道和当前排序留在编辑中',()=>{
 const h=make(),original=h.state().saved.providers.map(item=>item.id);h.click('copy-provider',{id:'newapi'});h.runConfirm();h.click('back-channels');h.click('copy-provider',{id:'aimami'});h.runConfirm();h.click('save');noModal(h);h.drain();
 assert.ok(p(h,'aimami-copy','saved'));assert.equal(p(h,'newapi-copy','saved'),undefined);assert.ok(p(h,'newapi-copy'));assert.deepEqual(h.state().saved.providers.filter(item=>original.includes(item.id)).map(item=>item.id),original);
});

test('备份失败、写入失败和超时留在详情，原saved与其它渠道输入均不变',()=>{
 for(const scene of ['backup-fail','partial','timeout']){
  const h=make();h.proto.scene(scene);edit(h,'newapi','失败后保留','demo-keep');edit(h,'aimami','其它编辑');edit(h,'newapi');const saved=json(h.state().saved),draft=json(h.state().draft);
  h.click('save');noModal(h);h.drain();assert.equal(h.state().operation.status,'failed',scene);assert.equal(h.state().channelDetail,true);assert.equal(json(h.state().saved),saved);assert.equal(json(h.state().draft),draft);assert.equal(h.state().records.length,1);
 }
});

test('停止和场景重置后的晚回都不能写入旧的详情保存',()=>{
 const h=make();edit(h,'newapi','准备保存');const saved=json(h.state().saved);h.click('save');h.click('stop');h.drain();assert.equal(h.state().operation.status,'stopped');assert.equal(json(h.state().saved),saved);assert.equal(h.state().channelDetail,true);
 h.click('save');h.proto.scene('normal');h.drain();assert.equal(h.state().operation,null);assert.equal(h.state().channelDetail,false);assert.equal(json(h.state().saved),saved);
});

test('保存基准漂移时保留详情与全部输入，不把旧确认当作可写入',()=>{
 const h=make(),blocked=[];h.proto.scene('drift');h.context.window.modelSaveBlockedBridge=value=>blocked.push(value);edit(h,'newapi','漂移时保留');const saved=json(h.state().saved);
 h.click('save');noModal(h);assert.equal(h.state().operation,null);assert.equal(h.state().channelDetail,true);assert.equal(p(h,'newapi').name,'漂移时保留');assert.equal(json(h.state().saved),saved);assert.equal(blocked.at(-1).key,'stale');
});

test('型号行确认只限制所属渠道；另一渠道保存不丢失待确认规则，确认后可单独保存',async t=>{
 const {make:makeDom}=require('./component-topic-harness.cjs');const h=await makeDom('#models');t.after(()=>h.dispose());const state=()=>h.w.modelPrototype.state();
 h.click('[data-mp=edit-provider][data-id=newapi]');h.click('[data-mp=expand-model][data-id="newapi::0"]');
 h.change('#mpQuick-newapi--0Mode-context','custom');h.change('#mpQuick-newapi--0ContextPreset','64000');
 assert.equal(state().modelEdits['newapi::0'].rules.context,64000);const saved=json(state().saved),pending=json(state().modelEdits['newapi::0']);
 h.click('#mpPolicySave');assert.equal(state().operation,null);assert.equal(state().channelDetail,true);assert.equal(json(state().saved),saved);
 h.click('[data-mp=back-channels]');h.click('[data-mp=edit-provider][data-id=aimami]');h.input('#mpChannelInline-name','另一渠道独立保存');h.click('#mpPolicySave');
 for(let n=0;n<120&&state().operation?.status==='running';n++)await h.wait(20);
 assert.equal(state().operation.status,'written');assert.equal(state().channelDetail,false);assert.equal(json(state().modelEdits['newapi::0']),pending);assert.equal(state().saved.providers[0].models[0].overrides.context,undefined);
 h.click('[data-task-act=model-later]');h.click('[data-mp=edit-provider][data-id=newapi]');h.click('[data-mp=apply-model-row][data-id="newapi::0"]');assert.equal(h.w.modelPrototype.stagedModelEdits().length,0);h.click('#mpPolicySave');
 for(let n=0;n<120&&state().operation?.status==='running';n++)await h.wait(20);
 assert.equal(state().operation.status,'written');assert.equal(state().saved.providers[0].models[0].overrides.context,64000);assert.equal(h.network.length,0);assert.deepEqual(h.errors,[]);
});
