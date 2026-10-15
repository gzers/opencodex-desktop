/* 共用执行设置的业务缓冲 / 权限与生命周期；选择器本身另做接线测试。 */
const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
function picker(h){let options;h.context.window.modelPickerPrototype={choose(input){options=input;h.context.closeModal();return true;}};return {get:()=>options,select(ref){options.onSelect({kind:'models',models:[ref]});},cancel(){options.onCancel({routeChanged:false});}};}
const ref=(modelId='gpt-5.4',effort)=>({kind:'channel',channelId:'newapi',modelId,...(effort?{reasoningEffort:effort}:{})});
test('执行设置替换旧三个下拉，采用公共单选推理弹窗；选择前后都不直接保存',()=>{
 const h=make(),p=picker(h),before=JSON.stringify(h.state().agent);h.click('agent-settings');assert.match(h.dialog(),/本机共用设置/);assert.match(h.dialog(),/单个更新.*全部更新/);assert.doesNotMatch(h.dialog(),/id="mpAgentProvider"|id="mpAgentModel"|id="mpAgentEffort"/);
 h.click('agent-pick-model');assert.equal(p.get().mode,'single');assert.equal(p.get().reasoning,true);p.select(ref('gpt-5.4-mini','high'));assert.equal(JSON.stringify(h.state().agent),before);assert.match(h.dialog(),/gpt-5.4-mini/);
 h.runConfirm();assert.equal(h.state().agent.modelId,'gpt-5.4-mini');assert.equal(h.state().agent.effort,'high');
});
test('取消公共选择器与取消设置均保留原值和编辑；选择原档返回不固化显示默认',()=>{
 const h=make();h.configureAgent();h.open('gpt');h.click('edit-template');h.meta('description','未保存说明');const p=picker(h),original=JSON.stringify(h.state().agent);h.click('agent-settings');h.click('agent-pick-model');p.cancel();assert.equal(JSON.stringify(h.state().agent),original);assert.match(h.dialog(),/gpt-5.4/);
 h.click('agent-pick-model');p.select(ref('gpt-5.4-mini'));h.context.closeModal();assert.equal(JSON.stringify(h.state().agent),original);assert.equal(h.state().buffers.gpt.description,'未保存说明');
 h.click('agent-settings');h.click('agent-pick-model');p.select(ref());h.runConfirm();assert.equal(h.state().agent.effort,'inherit');assert.match(h.dialog(),/模型默认/);
});
test('自动填充、单个与全部更新共享同一设置；已有任务冻结设置，新任务取新值',()=>{
 const h=make();h.configureAgent();h.open('gpt');h.proto.openTemplateUpdate();h.click('update-check');const old=h.state().updateRun.execution;assert.equal(old.modelId,'gpt-5.4');
 const p=picker(h);h.click('agent-settings');h.click('agent-pick-model');p.select(ref('gpt-5.4-mini','high'));h.runConfirm();assert.equal(h.state().updateRun.execution.modelId,'gpt-5.4');h.drain();
 h.open('claude');h.proto.openTemplateUpdate();h.click('update-check');assert.equal(h.state().updateRun.execution.modelId,'gpt-5.4-mini');assert.equal(h.state().updateRun.execution.effort,'high');h.drain();
 h.click('tab',{tab:'templates'});h.click('update-all');assert.equal(h.state().updateRun.execution.modelId,'gpt-5.4-mini');h.drain();
 h.open('gpt');h.click('edit-template');h.click('generate');h.event('change','[data-assistant-task]',{},'fill');h.nodes.get('mpGenerationTarget').value='默认高';h.click('generate-run');h.drain();assert.equal(h.state().proposal.execution.modelId,'gpt-5.4-mini');assert.equal(h.state().proposal.execution.effort,'high');
});
test('选择器使用已保存的有效规则，草稿能力不提前影响执行选择',()=>{
 const h=make();const original=h.proto.agentChoices().find(x=>x.ref.channelId==='newapi'&&x.ref.modelId==='gpt-5.4');
 h.proto.commitModelPatches('newapi',[{key:'newapi::0',rules:{context:64000,efforts:['low','high'],default:'low'}}]);
 assert.equal(h.proto.agentChoices().find(x=>x.ref.channelId==='newapi'&&x.ref.modelId==='gpt-5.4').effort,original.effort);
 h.click('save');h.runConfirm();h.drain();const choice=h.proto.agentChoices().find(x=>x.ref.channelId==='newapi'&&x.ref.modelId==='gpt-5.4');assert.equal(choice.context,64000);assert.deepEqual(choice.efforts,['low','high']);assert.equal(choice.effort,'low');
});
test('无额度和原生无绑定记录不可选；保存时再次验证目录和推理，禁用不支持 Fast',()=>{
 const h=make(),p=picker(h);h.click('agent-settings');h.click('agent-pick-model');const catalog=p.get().getCatalog();assert.ok(catalog.filter(x=>x.ref.channelId==='a6api').every(x=>/无额度/.test(x.reason)));assert.ok(catalog.filter(x=>x.ref.kind==='native').every(x=>/未绑定/.test(x.reason)));
 p.select({...ref(),reasoningEffort:'unsupported'});h.runConfirm();assert.match(h.nodes.get('mpAgentSettingsError').textContent,/档位/);assert.equal(h.state().agent.providerId,'');
 h.click('agent-settings');h.click('agent-pick-model');p.select(ref('claude-opus-4-6'));assert.doesNotMatch(h.dialog(),/id="mpAgentFast"/);h.runConfirm();assert.equal(h.state().agent.fast,false);
});
test('首先请求全部更新，经公共选择器配置后继续原范围；取消不启动',()=>{
 const h=make(),p=picker(h);h.click('tab',{tab:'templates'});h.click('update-all');h.click('agent-pick-model');p.select(ref('gpt-5.4','low'));assert.equal(h.state().updateRun,null);h.runConfirm();assert.equal(h.state().updateRun.status,'running');assert.equal(h.state().updateScope,'all');h.drain();assert.equal(h.state().updateRun.items.length,17);
});

test('公共选择器连续编辑同时带回推理与Fast，设置保存前不写共用偏好',()=>{
 const h=make(),p=picker(h),before=JSON.stringify(h.state().agent);h.click('agent-settings');h.click('agent-pick-model');assert.equal(p.get().fast,true);assert.equal(p.get().getCatalog().find(x=>x.ref.modelId==='gpt-5.4'&&x.ref.channelId==='newapi').fastCapability,'supported');p.select({...ref('gpt-5.4','high'),fast:true});assert.equal(JSON.stringify(h.state().agent),before);assert.doesNotMatch(h.dialog(),/id="mpAgentFast"|>使用 Fast</);assert.match(h.dialog(),/高.*快速已开启/);h.runConfirm();assert.equal(h.state().agent.fast,true);assert.equal(h.state().agent.effort,'high');
});
test('快速模式从选择器带回保存，重开保留，显式关闭不会被旧值覆盖',()=>{
 const h=make(),p=picker(h);h.click('agent-settings');h.click('agent-pick-model');p.select({...ref('gpt-5.4','high'),fast:true});h.runConfirm();
 h.click('agent-settings');h.click('agent-pick-model');assert.equal(p.get().models[0].fast,true);p.select({...ref('gpt-5.4','high'),fast:false});assert.doesNotMatch(h.dialog(),/快速已开启/);assert.equal(h.state().agent.fast,true);h.runConfirm();assert.equal(h.state().agent.fast,false);
});
test('选择器和设置取消保留已启用快速模式，换型号不继承旧型号的 Fast',()=>{
 const h=make(),p=picker(h);h.click('agent-settings');h.click('agent-pick-model');p.select({...ref(),fast:true});h.runConfirm();
 h.click('agent-settings');h.click('agent-pick-model');p.cancel();h.runConfirm();assert.equal(h.state().agent.fast,true);
 h.click('agent-settings');h.click('agent-pick-model');p.select({...ref(),fast:false});h.context.closeModal();assert.equal(h.state().agent.fast,true);
 h.click('agent-settings');h.click('agent-pick-model');p.select(ref('gpt-5.4-mini'));h.runConfirm();assert.equal(h.state().agent.fast,false);assert.equal(h.state().agent.modelId,'gpt-5.4-mini');
});
test('选择后 Fast 能力变化时保存拒绝，任务快照继续保留启动时快速模式',()=>{
 const h=make(),p=picker(h);h.click('agent-settings');h.click('agent-pick-model');p.select({...ref(),fast:true});h.runConfirm();h.open('gpt');h.proto.openTemplateUpdate();h.click('update-check');assert.equal(h.state().updateRun.execution.fast,true);
 h.click('agent-settings');h.click('agent-pick-model');p.select({...ref(),fast:false});h.runConfirm();assert.equal(h.state().agent.fast,false);assert.equal(h.state().updateRun.execution.fast,true);h.drain();
 h.click('agent-settings');h.click('agent-pick-model');p.select({...ref(),fast:true});h.runConfirm();h.proto.commitModelPatches('newapi',[{key:'newapi::0',rules:{fast:'unsupported'}}]);h.click('save');h.runConfirm();h.drain();h.click('agent-settings');h.runConfirm();assert.match(h.nodes.get('mpAgentSettingsError').textContent,/Fast 支持已变化/);assert.equal(h.state().agent.fast,true);
});
