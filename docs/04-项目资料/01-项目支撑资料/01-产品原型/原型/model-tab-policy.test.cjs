/* Tab右侧状态与保存的业务接线：保存与选择器预览合并为一个入口；实际控件等高/颜色仍需浏览器验证。 */
const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
const saveTag=h=>h.page().match(/<button id="mpPolicySave"[^>]*>/)?.[0]||'';
test('列表状态进入Tab右侧，核心控件按状态/还原/保存排列，预览不再单独占位',()=>{
 const h=make(),page=h.page(),tabs=page.indexOf('id="mpTabsBar"'),status=page.indexOf('id="mpPolicyStatus"'),list=page.indexOf('class="mp-list-toolbar"');assert.ok(tabs<status&&status<list);assert.equal((page.match(/id="mpPolicyStatus"/g)||[]).length,1);
 const controls=page.slice(page.indexOf('class="mp-top-policy-controls"'));assert.doesNotMatch(controls,/mpBudget|mp-budget|selector-preview/);assert.ok(controls.indexOf('data-channel-state="clean"')<controls.indexOf('id="mpPolicyReset"'));assert.ok(controls.indexOf('id="mpPolicyReset"')<controls.indexOf('id="mpPolicySave"'));assert.match(page.match(/<button id="mpPolicyReset"[^>]*>/)[0],/disabled/);assert.match(controls,/mp-policy-chip neutral/);assert.match(controls,/role="status" aria-live="polite"/);assert.match(saveTag(h),/ghost/);assert.doesNotMatch(saveTag(h),/disabled/);assert.doesNotMatch(saveTag(h),/is-changed/);assert.match(saveTag(h),/aria-label="保存"/);
});
test('有修改时列表保存主色强调，超限保持强调提示但禁止写入',()=>{
 const h=make(),saved=JSON.stringify(h.state().saved);h.click('toggle-provider',{id:'aimami'});assert.match(saveTag(h),/primary[^>]*is-changed/);assert.doesNotMatch(saveTag(h),/disabled/);assert.match(h.page(),/mp-policy-chip warning" data-channel-state="dirty"/);assert.equal(JSON.stringify(h.state().saved),saved);assert.doesNotMatch(h.page().match(/<button id="mpPolicyReset"[^>]*>/)[0],/disabled/);h.click('save');assert.match(h.dialog(),/完整选择器顺序/);h.context.closeModal();assert.equal(JSON.stringify(h.state().saved),saved);
 h.proto.scene('over-budget');assert.match(h.page(),/mp-policy-chip danger" data-channel-state="over-budget"/);assert.match(saveTag(h),/is-changed/);assert.doesNotMatch(saveTag(h),/disabled/);h.click('save');assert.match(h.dialog(),/超出激活预算/);assert.equal(h.nodes.get('modalConfirm').disabled,false);h.runConfirm();assert.notEqual(h.nodes.get('modalMask').style.display,'flex');assert.equal(h.state().operation,null);
});
test('待确认编辑/待重启在顶部显式显示，未确认输入禁用保存，详情仍用侧栏',()=>{
 const h=make();h.click('edit-provider',{id:'newapi'});assert.doesNotMatch(h.page().slice(0,h.page().indexOf('id="mpChannelHeader"')),/id="mpPolicyStatus"/);h.event('input','[data-channel-config]',{channelConfig:'name'},'待确认名字');h.click('back-channels');assert.match(h.page(),/mp-policy-chip warning" data-channel-state="staged"/);assert.match(saveTag(h),/is-changed/);h.click('save');assert.match(h.dialog(),/未加入草稿/);assert.equal(h.nodes.get('modalConfirm').disabled,false);h.runConfirm();assert.notEqual(h.nodes.get('modalMask').style.display,'flex');assert.doesNotMatch(h.page().match(/<button id="mpPolicyReset"[^>]*>/)[0],/disabled/);
 h.proto.scene('pending-draft');assert.match(h.page(),/data-channel-state="dirty"/);assert.match(h.page(),/mp-policy-pending" data-channel-pending/);assert.match(h.page(),/上次保存待重启/);assert.equal(h.state().pending,true);
});
test('九种状态有样式示例标签，保存中显示进行中状态且固定栏更新不改变冻结计划',()=>{
 const h=make();const states=[...h.page().matchAll(/data-policy-example="([^"]+)"/g)].map(m=>m[1]);assert.deepEqual(states,['clean','dirty','staged','over-budget','saving','pending','failed','stopped','uncertain']);assert.match(h.page(),/渠道列表的九种状态与保存样式/);assert.match(h.page(),/状态框和保存按钮等高/);
 h.click('toggle-provider',{id:'aimami'});h.click('save');h.runConfirm();const frozen=JSON.stringify(h.state().operation.plan);assert.match(h.page(),/mp-policy-chip info" data-channel-state="saving"/);assert.match(h.page(),/aria-busy="true"/);assert.match(saveTag(h),/disabled/);h.drain();assert.match(h.page(),/mp-policy-chip warning" data-channel-state="pending"/);assert.equal(JSON.stringify(h.state().saved),frozen);
});
