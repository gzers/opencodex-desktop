/* 渠道操作分工与全渠道草稿边界，不替代真实浏览器布局和焦点。 */
const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
const provider=(h,id,which='draft')=>h.state()[which].providers.find(p=>p.id===id);
const heading=h=>h.nodes.get('mpChannelHeadingStatus')?.innerHTML||h.page().match(/<header id="mpChannelHeader"[\s\S]*?<\/header>/)?.[0]||'';
// 最小 DOM 的父级 outerHTML 替换不重建后代，用最新状态片段核对按钮。
const saveButton=h=>{const html=h.nodes.get('mpPolicyStatus')?.outerHTML||h.page(),tag=html.match(/<button id="mpPolicySave"[^>]*>/)?.[0];assert.ok(tag);return {disabled:/\bdisabled\b/.test(tag)};};
test('详情状态在标题，保存只显示保存；列表移除历史和更多，直接提供还原',()=>{
 const h=make();assert.match(h.page(),/id="mpPolicyReset"/);assert.doesNotMatch(h.page(),/<button[^>]*data-mp="(?:history|more)"/);assert.doesNotMatch(h.page(),/配置工具|data-mp="settings"|data-mp="import"|data-mp="export"/);
 assert.doesNotMatch(h.page(),/data-mp="undo"|data-mp="redo"/);h.click('edit-provider',{id:'newapi'});assert.doesNotMatch(h.page(),/mp-channel-edit-actions|data-mp="undo"|data-mp="redo"|配置工具|data-mp="settings"|data-mp="more"|data-mp="order-preview"|保存与恢复/);assert.match(h.page(),/data-save-scope="current-channel"/);assert.match(heading(h),/data-channel-state="clean"/);
 const save=h.page().match(/<button id="mpPolicySave"[\s\S]*?<\/button>/)[0];assert.match(save,/aria-label="保存"/);assert.doesNotMatch(save,/先预览当前草稿/);assert.match(save,/<span class="mp-nav-action-label">保存<\/span>/);assert.doesNotMatch(save,/<small|未修改|已保存|全渠道/);assert.equal(saveButton(h).disabled,true);
 h.click('toggle-model',{id:'newapi::0'});assert.equal(saveButton(h).disabled,false);h.click('save');assert.notEqual(h.nodes.get('modalMask').style.display,'flex');assert.equal(h.state().operation.status,'running');assert.equal(h.state().operation.channelId,'newapi');h.drain();assert.equal(h.state().channelDetail,false);assert.equal(provider(h,'newapi','saved').models[0].selected,false);
});
test('顶部还原无修改时禁用，修改后预览确认；旧历史和更多事件不再打开窗口',()=>{
 const h=make();assert.match(h.page().match(/<button id="mpPolicyReset"[^>]*>/)[0],/disabled/);h.click('more');h.click('history');assert.notEqual(h.nodes.get('modalMask').style.display,'flex');
 h.click('edit-provider',{id:'newapi'});h.click('toggle-model',{id:'newapi::0'});h.click('back-channels');assert.doesNotMatch(h.page().match(/<button id="mpPolicyReset"[^>]*>/)[0],/disabled/);h.click('discard');assert.match(h.dialog(),/受影响范围/);assert.match(h.dialog(),/最近成功保存的配置/);h.runConfirm();assert.equal(JSON.stringify(h.state().draft),JSON.stringify(h.state().saved));
});
test('跨渠道草稿保留，返回不写入；旧撤销重做事件没有动作，保存差异仍列全部渠道',()=>{
 const h=make(),saved=JSON.stringify(h.state().saved);h.click('edit-provider',{id:'newapi'});h.click('toggle-model',{id:'newapi::0'});h.click('edit-provider',{id:'aimami'});h.click('toggle-model',{id:'aimami::0'});const draft=JSON.stringify(h.state().draft);h.click('edit-provider',{id:'newapi'});
 assert.match(heading(h),/warning" data-channel-state="dirty"/);assert.match(heading(h),/data-status-scope="current-channel"/);h.click('undo');h.click('redo');assert.equal(JSON.stringify(h.state().draft),draft);
 h.click('back-channels');assert.equal(h.state().channelDetail,false);assert.equal(JSON.stringify(h.state().draft),draft);assert.equal(JSON.stringify(h.state().saved),saved);assert.equal(h.state().operation,null);h.click('save');assert.match(h.dialog(),/NewApi/);assert.match(h.dialog(),/Aimami/);h.context.closeModal();assert.equal(JSON.stringify(h.state().saved),saved);
});
test('连接输入直接改草稿，旧局部动作无效；校验失败和返回都保留输入，修正后直接保存',()=>{
 const h=make(),blocked=[],saved=JSON.stringify(h.state().saved);h.context.window.modelSaveBlockedBridge=p=>blocked.push(p);h.click('edit-provider',{id:'newapi'});
 h.nodes.get('mpChannelInline-name').value='';h.event('input','[data-channel-config]',{channelConfig:'name'},'');assert.equal(provider(h,'newapi').name,'');assert.equal(h.proto.stagedChannelConnections().length,0);assert.match(heading(h),/data-channel-state="dirty"/);assert.doesNotMatch(h.page(),/data-mp="(?:apply-channel-connection|reset-channel-connection)"/);
 const edited=JSON.stringify(h.state().draft);h.click('apply-channel-connection');h.click('reset-channel-connection');assert.equal(JSON.stringify(h.state().draft),edited);h.click('save');assert.equal(h.state().operation,null);assert.equal(blocked.at(-1).key,'channel-validation');assert.equal(h.nodes.get('mpChannelInline-name').value,'');assert.equal(JSON.stringify(h.state().saved),saved);assert.equal(h.state().channelDetail,true);
 h.event('input','[data-channel-config]',{channelConfig:'name'},'NewApi 临时名称');h.click('back-channels');assert.equal(provider(h,'newapi').name,'NewApi 临时名称');assert.equal(JSON.stringify(h.state().saved),saved);h.click('edit-provider',{id:'newapi'});assert.equal(h.nodes.get('mpChannelInline-name').value,'NewApi 临时名称');h.click('save');assert.notEqual(h.nodes.get('modalMask').style.display,'flex');h.drain();assert.equal(provider(h,'newapi','saved').name,'NewApi 临时名称');assert.equal(h.state().channelDetail,false);
});
test('丢弃确认列全渠道及连接字段变化，取消保留；确认回成功基线，Agent和模版保留',()=>{
 const h=make();h.configureAgent();const agent=JSON.stringify(h.state().agent),templates=JSON.stringify(h.state().templates);h.click('edit-provider',{id:'newapi'});h.click('toggle-model',{id:'newapi::0'});h.click('edit-provider',{id:'aimami'});h.click('toggle-model',{id:'aimami::0'});h.event('input','[data-channel-config]',{channelConfig:'name'},'Aimami 临时连接名');h.click('back-channels');
 const dirty=JSON.stringify(h.state().draft),connections=JSON.stringify(h.state().channelConnections);h.click('discard');assert.match(h.dialog(),/受影响范围/);assert.match(h.dialog(),/NewApi/);assert.match(h.dialog(),/Aimami/);assert.match(h.dialog(),/名称/);assert.match(h.dialog(),/Aimami 临时连接名/);assert.doesNotMatch(h.dialog(),/未确认的连接编辑/);h.context.closeModal();assert.equal(JSON.stringify(h.state().draft),dirty);assert.equal(JSON.stringify(h.state().channelConnections),connections);
 h.click('discard');h.runConfirm();assert.equal(JSON.stringify(h.state().draft),JSON.stringify(h.state().saved));assert.deepEqual(h.state().channelConnections,{});assert.deepEqual(h.state().modelEdits,{});assert.equal(JSON.stringify(h.state().agent),agent);assert.equal(JSON.stringify(h.state().templates),templates);assert.match(h.page(),/data-channel-state="clean"/);
});
test('丢弃预览后发生新修改时拒绝旧确认，不清空更新内容',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});h.click('toggle-model',{id:'newapi::0'});h.click('discard');h.click('toggle-model',{id:'newapi::1'});const newer=JSON.stringify(h.state().draft);h.runConfirm();assert.match(h.nodes.get('mpDiscardError').textContent,/已变化/);assert.equal(JSON.stringify(h.state().draft),newer);
});
test('仅改连接字段也能全局丢弃，纯新建编辑缓冲不误伤模版对象',()=>{
 const h=make();h.click('new-template');const id=h.state().templateScratch.id;h.meta('description','未保存模版内容');h.click('tab',{tab:'channels'});h.click('edit-provider',{id:'newapi'});h.event('input','[data-channel-config]',{channelConfig:'name'},'只改连接名称');h.click('back-channels');
 h.click('discard');assert.match(h.dialog(),/名称/);assert.match(h.dialog(),/只改连接名称/);h.runConfirm();assert.equal(h.state().buffers[id].description,'未保存模版内容');assert.equal(h.state().templateScratch.id,id);assert.equal(JSON.stringify(h.state().draft),JSON.stringify(h.state().saved));
});
test('保存中还原禁用，成功保存成为新还原基准；不留待重启且底层记录保留',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});h.click('toggle-model',{id:'newapi::0'});h.click('back-channels');h.click('save');h.runConfirm();const draft=JSON.stringify(h.state().draft);h.click('undo');h.click('redo');h.click('discard');assert.equal(JSON.stringify(h.state().draft),draft);assert.match(h.page().match(/<button id="mpPolicyReset"[^>]*>/)[0],/disabled/);h.drain();
 assert.match(h.page(),/neutral" data-channel-state="clean"/);assert.ok(h.state().records.some(r=>r.status==='成功'));assert.equal(provider(h,'newapi','saved').models[0].selected,false);h.click('edit-provider',{id:'newapi'});h.click('toggle-model',{id:'newapi::0'});h.click('back-channels');h.click('discard');h.runConfirm();assert.equal(provider(h,'newapi').models[0].selected,false);assert.equal(JSON.stringify(h.state().draft),JSON.stringify(h.state().saved));assert.equal(h.state().pending,false);
});
test('传入的既有 Fast 策略草稿仍能核对与丢弃，历史不复活待重启且候选保留',()=>{
 const h=make();h.proto.scene('incoming');assert.equal(h.state().draft.fastMode,'inherit');h.click('save');assert.match(h.dialog(),/Fast 请求策略/);assert.doesNotMatch(h.dialog(),/本次变更与写入/);h.context.closeModal();h.click('discard');h.runConfirm();assert.equal(h.state().draft.fastMode,'on');assert.ok(h.state().incoming);
 h.proto.scene('pending-draft');h.click('discard');h.runConfirm();assert.equal(h.state().pending,false);
 h.proto.scene('incoming');assert.ok(h.state().incoming);h.click('discard');h.runConfirm();assert.ok(h.state().incoming);
});
test('启停与连接状态独立显示，停用不清选择；官方渠道沿用同一标题',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});assert.match(heading(h),/success" data-channel-enabled="true"/);assert.match(heading(h),/neutral" data-channel-availability="unknown"/);assert.match(heading(h),/已选 17 \/ 发现 30/);
 const before=provider(h,'newapi').models.filter(m=>m.selected).length;h.click('toggle-provider',{id:'newapi'});assert.match(heading(h),/neutral" data-channel-enabled="false"/);assert.equal(provider(h,'newapi').models.filter(m=>m.selected).length,before);
 h.click('edit-provider',{id:'a6api'});assert.match(heading(h),/warning" data-channel-availability="no-credit"/);h.click('edit-provider',{id:'native'});assert.match(heading(h),/data-channel-managed/);assert.doesNotMatch(heading(h),/data-channel-availability|data-channel-enabled/);
});
test('备份与原子写入失败保留草稿，可直接重试；预算超限阻止保存',()=>{
 for(const [scene,state] of [['backup-fail','failed'],['partial','failed'],['timeout','failed']]){
  const h=make();h.proto.scene(scene);h.click('edit-provider',{id:'newapi'});h.event('input','[data-channel-config]',{channelConfig:'name'},'失败后保留');const saved=JSON.stringify(h.state().saved),draft=JSON.stringify(h.state().draft),records=h.state().records.length;h.click('save');assert.notEqual(h.nodes.get('modalMask').style.display,'flex');assert.match(heading(h),/info" data-channel-state="saving"/);assert.equal(saveButton(h).disabled,true);h.drain();assert.match(heading(h),new RegExp('data-channel-state="'+state+'"'));assert.equal(JSON.stringify(h.state().saved),saved);assert.equal(JSON.stringify(h.state().draft),draft);assert.equal(h.state().records.length,records);assert.equal(h.nodes.get('mpChannelInline-name').value,'失败后保留');assert.equal(h.state().channelDetail,true);assert.doesNotMatch(h.page(),/data-mp="(?:verify|client-check)"/);h.click('save');assert.equal(h.state().operation.status,'running');h.click('stop');h.drain();assert.equal(JSON.stringify(h.state().saved),saved);
 }
 const h=make(),blocked=[];h.context.window.modelSaveBlockedBridge=p=>blocked.push(p);h.proto.scene('over-budget');h.click('edit-provider',{id:'a6api'});const saved=JSON.stringify(h.state().saved),draft=JSON.stringify(h.state().draft);assert.match(heading(h),/danger" data-channel-state="over-budget"/);assert.equal(saveButton(h).disabled,false);h.click('save');assert.notEqual(h.nodes.get('modalMask').style.display,'flex');assert.match(blocked.at(-1).reason,/超出激活预算/);assert.equal(h.state().operation,null);assert.equal(JSON.stringify(h.state().saved),saved);assert.equal(JSON.stringify(h.state().draft),draft);assert.equal(h.state().channelDetail,true);
});
test('清单获取中与失败使用独立状态，不以获取失败冒充保存失败',()=>{
 const h=make();h.proto.scene('discovery-fail');h.click('edit-provider',{id:'newapi'});h.click('refresh');assert.match(heading(h),/info" data-channel-discovery="loading"/);h.drain();assert.match(heading(h),/danger" data-channel-discovery="failed"/);assert.doesNotMatch(heading(h),/data-channel-state="failed"/);
});
