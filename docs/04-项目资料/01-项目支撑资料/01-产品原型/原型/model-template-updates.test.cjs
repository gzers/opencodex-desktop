const {test}=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const vm=require('node:vm');
const path=require('node:path');
const U=require('./model-template-updates.js');
const prototypeRoot=path.resolve(__dirname,'..');
const harnessFile=path.resolve(__dirname,'../../../03-项目协作资料/01-协作包/2026-10-04-模型配置管理与同步/01-需求分析/05-原型/qa/template-workflow.source.mjs');
const make=require('./model-prototype-test-harness.cjs');
function check(h){h.open('gpt');h.configureAgent();h.proto.openTemplateUpdate();h.click('update-check');h.drain();}

test('同秒版本有不同完整 ID 与短标识；北京时间固定，短标识冲突扩展',()=>{
 const time=Date.parse('2026-10-13T06:32:08Z'),a=U.stamp([],time,'aa-a7f2'),b=U.stamp([a],time,'bb-a7f2');
 assert.match(a.versionLabel,/2026-10-13 14:32:08 · A7F2/);assert.notEqual(a.shortId,b.shortId);assert.notEqual(a.revisionId,b.revisionId);
 assert.notEqual(U.stamp([a],time,a.revisionId).revisionId,a.revisionId);
});
test('更新新增型号默认可选，人工默认值不被静默替换；接受后原输入不变',()=>{
 const current={rules:{default:'medium'},match:{include:[]}},rows=U.changes(current,{rules:{default:'high'},variants:[{id:'x',name:'新型号',suffix:'new-model',rules:{}}]});
 assert.equal(rows[0].selected,false);assert.equal(rows[1].selected,true);
 const next=U.compose(current,rows);assert.equal(next.rules.default,'medium');assert.equal(next.variants.length,1);assert.equal(current.variants,undefined);
 rows[0].selected=true;assert.equal(U.compose(current,rows).rules.default,'high');
});
test('恢复按稳定模型 ID 合并：撤回引用，保留后续别名、其他模型和草稿冲突',()=>{
 const before={providers:[{id:'p',refs:{gpt:'gpt:1'},models:[{id:'m',naming:{alias:'old'},overrides:{default:'low'}}]}]},after=structuredClone(before);after.providers[0].refs.gpt='gpt:2';after.providers[0].models[0].overrides.default='high';
 const now=structuredClone(after);now.providers[0].models[0].naming.alias='later';now.providers[0].models[0].overrides.default='max';now.providers.push({id:'q',refs:{}});
 const result=U.restore(now,before,after);assert.equal(result.value.providers[0].refs.gpt,'gpt:1');assert.equal(result.value.providers[0].models[0].naming.alias,'later');assert.equal(result.value.providers[0].models[0].overrides.default,'max');assert.equal(result.value.providers.length,2);assert.equal(result.conflicts.length,1);
});
test('总原型按依赖顺序加载两个模块；17 族初始版本使用时间标签',()=>{
 const html=fs.readFileSync(path.join(__dirname,'index.html'),'utf8');assert.ok(html.indexOf('src="model-template-updates.js"')<html.indexOf('src="model-config.js'));
 const h=make(),state=h.state();assert.equal(state.templates.length,17);
 const versions=state.templates.flatMap(t=>t.versions);assert.equal(new Set(versions.map(v=>v.revisionId)).size,versions.length);
 assert.ok(versions.every(v=>/^2026-10-\d\d \d\d:\d\d:\d\d · /.test(v.versionLabel)));
 h.click('tab',{tab:'templates'});assert.doesNotMatch(h.page(),/>v[123]</);assert.doesNotMatch(h.page(),/检查所选更新/);
 h.event('change','[data-template-select]',{templateSelect:'gpt'},'',true);assert.match(h.page(),/检查所选更新/);
});
test('无需勾选即可在列表点击检查全部更新，自动检查17族并打开差异，旧配置不变',()=>{
 const h=make();h.configureAgent();h.click('tab',{tab:'templates'});assert.match(h.page(),/id="mpUpdateAll".*检查全部更新/);
 const before=h.state();h.click('update-all');assert.equal(h.state().templateDetail,false);assert.equal(h.state().updateScope,'all');assert.equal(h.state().updateRun.status,'running');assert.match(h.nodes.get('mpAssistantHost').innerHTML,/更新模型资料/);
 h.click('update-all');assert.equal(h.state().updateRun.id,1);h.drain();assert.equal(h.state().updateRun.items.length,17);assert.match(h.dialog(),/mpUpdateTree/);assert.equal(h.nodes.get('modalMask').style.display,'flex');
 assert.equal(JSON.stringify(h.state().templates),JSON.stringify(before.templates));assert.equal(JSON.stringify(h.state().saved),JSON.stringify(before.saved));
});
test('首次全部更新先配置执行模型，保存设置后自动继续；取消不会后来偷偷启动',()=>{
 const h=make();h.click('tab',{tab:'templates'});h.click('update-all');assert.equal(h.state().updateRun,null);assert.ok(h.nodes.has('mpAgentModelChoice'));
 h.context.window.modelPickerPrototype={choose(options){options.onSelect({kind:'models',models:[{kind:'channel',channelId:'newapi',modelId:'gpt-5.4'}]});return true;}};h.click('agent-pick-model');h.runConfirm();assert.equal(h.state().updateRun.status,'running');h.drain();assert.equal(h.state().updateRun.items.length,17);
 const cancelled=make();cancelled.click('update-all');cancelled.context.closeModal();cancelled.configureAgent();cancelled.drain();assert.equal(cancelled.state().updateRun,null);
});
test('列表所选入口一次点击只检查勾选项，不跳走或扩大为全部',()=>{
 const h=make();h.configureAgent();h.click('tab',{tab:'templates'});h.event('change','[data-template-select]',{templateSelect:'glm'},'',true);h.event('change','[data-template-select]',{templateSelect:'hy'},'',true);
 h.click('update-selected');h.drain();assert.deepEqual(h.state().updateRun.items.map(i=>i.id),['glm','hy']);assert.equal(h.state().templateDetail,false);assert.match(h.dialog(),/mpUpdateTree/);
});
test('当前更新经差异确认保存新版本；不改主指针与渠道引用；保留旧版内容',()=>{
 const h=make(),initial=h.state();check(h);assert.equal(h.state().assistantTask,'update');assert.equal(h.state().updateRun.status,'ready');
 h.click('update-review');assert.match(h.dialog(),/模型资料|人工值默认保留/);h.runConfirm();const state=h.state();assert.equal(state.updateRun.status,'saved');
 const t=state.templates.find(t=>t.id==='gpt');assert.equal(t.versions.length,4);assert.equal(t.main,1);assert.equal(t.versions[3].basedOn,t.versions[0].revisionId);assert.equal(t.versions[3].rules.default,'medium');assert.ok(t.versions[3].variants.length);
 assert.equal(JSON.stringify(state.draft),JSON.stringify(initial.draft));assert.equal(JSON.stringify(t.versions.slice(0,3)),JSON.stringify(initial.templates[0].versions));
});
test('再次检查当前已更新版本，无变化不会新增；取消差异不保存',()=>{
 const h=make();check(h);h.click('update-review');h.context.closeModal();assert.equal(h.state().templates[0].versions.length,3);
 h.proto.scene('update-none');h.click('update-check');h.drain();h.click('update-review');h.runConfirm();assert.equal(h.state().templates[0].versions.length,3);assert.equal(h.nodes.get('modalConfirm').disabled,true);
});
test('检查期间人工编辑使保存过期，旧结果不覆盖当前缓冲',()=>{
 const h=make();check(h);h.click('update-review');h.click('edit-template');h.meta('description','检查后的人工备注');h.runConfirm();assert.match(h.nodes.get('mpUpdateError').textContent,/已变化/);assert.equal(h.state().templates[0].versions.length,3);assert.equal(h.state().buffers.gpt.description,'检查后的人工备注');
});
test('检索失败、停止和场景切换使晚回无效，库与引用保留',()=>{
 const h=make();h.configureAgent();h.proto.scene('update-fail');h.click('update-check');h.drain();assert.equal(h.state().updateRun.status,'failed');
 h.proto.scene('update-late');h.click('update-check');h.click('update-stop');h.drain();assert.equal(h.state().updateRun.status,'stopped');
 h.click('update-check');h.proto.scene('normal');h.drain();assert.equal(h.state().updateRun,null);assert.equal(h.state().templates[0].versions.length,3);
});
test('批量每模版独立版本并登记同一批次，主指针不切换',()=>{
 const h=make();h.configureAgent();h.proto.openTemplateUpdate();h.event('change','[data-update-scope]',{},'all');h.click('update-check');h.drain();h.click('update-review');
 // 任意所选条目都在最终确认时校验，不只校验第一个。
 const run=vm.runInContext('window.modelPrototype.state()',h.context);assert.ok(run.updateRun.items.length===17);
 h.runConfirm();const state=h.state();assert.equal(state.updateBatches.length,1);assert.ok(state.updateBatches[0].versions.length>5);assert.ok(state.templates.every(t=>t.main===1));
 for(const op of state.updateBatches[0].versions){const v=state.templates.find(t=>t.id===op.id).versions.at(-1);assert.equal(v.batchId,state.updateBatches[0].id);}
});
test('采用新版本再保存，恢复到旧引用；后续人工别名与新版记录保留',()=>{
 const h=make();check(h);h.click('update-review');h.runConfirm();const t=h.state().templates[0],newN=t.versions.at(-1).n,labels=t.versions.map(v=>v.versionLabel);
 h.click('adopt-version');h.nodes.get('mpAdoptScope').value='global';h.runConfirm();h.click('save');h.runConfirm();h.drain();assert.equal(h.state().saved.globalRefs.gpt,'gpt:'+newN);
 const record=h.state().records.at(-1);h.click('restore',{id:String(record.id)});h.runConfirm();assert.equal(h.state().draft.globalRefs.gpt,'gpt:1');assert.equal(h.state().saved.globalRefs.gpt,'gpt:'+newN);
 h.click('save');h.runConfirm();h.drain();assert.equal(h.state().saved.globalRefs.gpt,'gpt:1');assert.equal(h.state().records.at(-1).restores,record.id);assert.deepEqual(h.state().templates[0].versions.map(v=>v.versionLabel),labels);
});
test('确认可用要求实际采用及核对；确认快照可找回，受依赖保护不能删除',()=>{
 const h=make();h.open('gpt');h.click('version-menu');h.click('confirm-template-use');h.runConfirm();assert.equal(h.state().confirmedConfigurations.length,1);
 const confirmation=h.state().confirmedConfigurations[0];assert.match(confirmation.label,/\d\d:\d\d:\d\d/);
 h.click('template-recovery',{id:'gpt'});assert.match(h.dialog(),/确认可用/);h.click('restore-confirmed',{id:confirmation.id});assert.match(h.dialog(),/当时确认的整套/);
 h.click('delete-version',{id:'gpt:1'});assert.equal(h.state().templates[0].versions.length,3);
});
test('批量所选中的第二个模版校验失败时，第一个有效模版也不保存',()=>{
 const h=make();h.open('claude');h.click('edit-template');h.meta('name','');h.configureAgent();h.open('gpt');h.proto.openTemplateUpdate();h.event('change','[data-update-scope]',{},'all');h.click('update-check');h.drain();
 const before=h.state().templates.map(t=>t.versions.length);h.click('update-review');h.runConfirm();assert.match(h.nodes.get('mpUpdateError').textContent,/名称/);assert.deepEqual(h.state().templates.map(t=>t.versions.length),before);assert.equal(h.state().updateBatches.length,0);
});
test('差异组仅选择非冲突项；人工字段需单独明确勾选才替换',()=>{
 const h=make();h.configureAgent();h.proto.scene('update-conflict');h.click('update-check');h.drain();h.click('update-review');const item=h.state().updateRun.items[0],conflict=item.rows.find(r=>r.conflict);assert.ok(conflict);
 h.event('change','[data-update-group]',{updateGroup:'gpt'},'',true);assert.equal(h.state().updateRun.items[0].rows.find(r=>r.id===conflict.id).selected,false);
 h.event('change','[data-update-pick]',{updatePick:'gpt:'+conflict.id},'',true);h.runConfirm();assert.equal(h.state().templates[0].versions.at(-1).rules.default,'high');
});
test('更新详情为只读第二层，关闭后保留选择和父差异页',()=>{
 const h=make();check(h);h.click('update-review');const rows=JSON.stringify(h.state().updateRun.items[0].rows);h.click('update-details',{id:'gpt:change-1'});
 assert.ok(h.nodes.has('mpUpdateDetail'));assert.match(h.nodes.get('mpUpdateDetail').innerHTML,/官方型号资料/);assert.match(h.nodes.get('mpUpdateDetail').innerHTML,/只读/);
 h.click('update-details-close');assert.equal(h.nodes.has('mpUpdateDetail'),false);assert.equal(JSON.stringify(h.state().updateRun.items[0].rows),rows);assert.match(h.dialog(),/mpUpdateTree/);
});
test('人工保存与外部 Agent 保存也生成时间版本，主指针不自动变更',()=>{
 const h=make();h.open('gpt');h.click('edit-template');h.meta('description','人工说明');h.click('save-template');h.runConfirm();const manual=h.state().templates[0].versions.at(-1);assert.match(manual.versionLabel,/\d\d:\d\d:\d\d/);assert.ok(manual.basedOn);
 const task=h.startExternal();h.incoming(task,[h.operation(task.targets[0],{...task.targets[0].content,description:'外部说明'})]);h.runConfirm();const external=h.state().templates[0].versions.at(-1);assert.equal(external.origin,'外部 Agent');assert.match(external.versionLabel,/\d\d:\d\d:\d\d/);assert.notEqual(external.revisionId,manual.revisionId);assert.equal(h.state().templates[0].main,1);
});
test('恢复预览后编辑导致确认过期，不覆盖新草稿',()=>{
 const h=make();check(h);h.click('update-review');h.runConfirm();h.click('adopt-version');h.nodes.get('mpAdoptScope').value='global';h.runConfirm();h.click('save');h.runConfirm();h.drain();const record=h.state().records.at(-1);
 h.click('restore',{id:String(record.id)});h.click('toggle-model',{id:'newapi::0'});const draft=JSON.stringify(h.state().draft);h.runConfirm();assert.match(h.nodes.get('mpRestoreError').textContent,/已变化/);assert.equal(JSON.stringify(h.state().draft),draft);
});
