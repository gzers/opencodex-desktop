/* 对话页消费公共树的接线回归；内存DOM不代替实屏或真实Codex存储验证。 */
const test=require('node:test'),assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');
test('项目与对话复用公共树，保留原配置与生命周期',async t=>{
 const h=await make('#conversations'),{w,document:d}=h,$=q=>d.querySelector(q),state=()=>w.conversationPrototype.state(),nav=()=>w.conversationPrototype.navigationState();t.after(()=>h.dispose());
 const tree=$('#cvObjectTree .tn-tree'),panel=$('.cv-tree-panel');
 await t.test('同一公共渲染器、稳定业务ID、三层子Agent与导航外观',()=>{
  assert.ok(tree);assert.equal(tree.getAttribute('role'),'tree');assert.equal(tree.getAttribute('aria-label'),'项目与对话');assert.equal(d.querySelectorAll('#conversationContent [role=tree]').length,1);assert.equal($('.cv-object-row'),null);
  assert.equal($('[data-tn-id="project:tools"]').getAttribute('aria-level'),'1');assert.equal($('[data-tn-id="thread:config"]').getAttribute('aria-level'),'2');assert.equal($('[data-tn-id="thread:review"]').getAttribute('aria-level'),'3');assert.ok(panel.classList.contains('tn-panel'));assert.equal(state().selected,'project:tools');
 });
 await t.test('子Agent搜索保留祖先、折叠不选中、清空恢复、树根DOM不重建',()=>{
  const before=JSON.stringify(state().saved);h.input('#cvObjectSearch','审阅');assert.deepEqual(Array.from(nav().rows,n=>n.id),['project:tools','thread:config','thread:review']);assert.equal(state().selected,'project:tools');
  h.click('[data-tn-id="thread:config"] [data-tn-toggle]');assert.equal(nav().rows.some(n=>n.id==='thread:review'),false);h.click('[data-cv=tree-clear]');assert.equal($('#cvObjectSearch').value,'');assert.equal(nav().rows.some(n=>n.id==='thread:review'),true);assert.equal($('#cvObjectTree .tn-tree'),tree);assert.equal(JSON.stringify(state().saved),before);
 });
 await t.test('点击选择带回原编辑，往返全局设置保留树实例、展开与搜索',async()=>{
  h.click('[data-tn-id="thread:config"]');assert.equal(state().selected,'thread:config');assert.match($('.cv-editor h2').textContent,/模型配置/);
  h.change('[data-cv-field="subagent.effort"]','high');assert.equal(state().drafts['thread:config']['subagent.effort'],'high');assert.equal(state().saved['thread:config']['subagent.effort'],'medium');assert.match($('[data-tn-id="thread:config"]').textContent,/未保存/);assert.equal($('#cvObjectTree .tn-tree'),tree);assert.equal($('.cv-tree-panel'),panel);
  h.input('#cvObjectSearch','模型');const expanded=nav().expanded;h.click('[data-cv=global-policy]');await h.settle();w.setRoute('conversations');await h.settle();assert.equal($('#cvObjectSearch').value,'模型');assert.deepEqual(nav().expanded,expanded);assert.equal($('#cvObjectTree .tn-tree'),tree);assert.equal(state().drafts['thread:config']['subagent.effort'],'high');h.click('[data-cv=tree-clear]');
 });
 await t.test('切对象保留草稿，隐藏所选可定位，筛选不自动保存或采用',()=>{
  h.click('[data-tn-id="thread:review"]');const saved=JSON.stringify(state().saved);h.click('[data-cv=tree-filter][data-filter=own]');assert.equal(state().selected,'thread:review');assert.equal(nav().selectionOutside,true);assert.match($('#cvObjectTree .tn-outside').textContent,/审阅/);h.click('[data-tn-reveal]');assert.equal(nav().selectionOutside,false);assert.equal($('[data-cv=tree-filter][data-filter=all]').getAttribute('aria-pressed'),'true');assert.equal(JSON.stringify(state().saved),saved);
  h.click('[data-tn-id="thread:config"]');assert.equal($('[data-cv-field="subagent.effort"]').value,'high');h.click('[data-tn-id="thread:window"]');h.change('[data-cv-field="subagent.effort"]','medium');h.click('[data-cv=tree-filter][data-filter=own]');assert.ok($('[data-tn-id="thread:window"]'));assert.match($('[data-tn-id="thread:window"]').textContent,/未保存/);h.click('[data-cv=tree-filter][data-filter=all]');
 });
 await t.test('失败保存保留草稿与树状态，原保存仅提交当前对象',()=>{
  h.input('#cvObjectSearch','窗口');const original=state().saved['thread:window'];w.conversationPrototype.scene('save-fail');h.click('[data-cv=save]');assert.match($('#conversationContent').textContent,/保存失败/);assert.deepEqual(state().saved['thread:window'],original);assert.equal(state().drafts['thread:window']['subagent.effort'],'medium');assert.equal($('#cvObjectSearch').value,'窗口');assert.equal($('#cvObjectTree .tn-tree'),tree);
  w.conversationPrototype.scene('normal');h.click('[data-cv=save]');assert.equal(state().saved['thread:window']['subagent.effort'],'medium');assert.equal(state().drafts['thread:window'],undefined);assert.equal(state().drafts['thread:config']['subagent.effort'],'high');assert.match($('[data-tn-id="thread:window"]').textContent,/专属/);h.click('[data-cv=tree-clear]');
 });
 await t.test('归档清理原覆盖并保留未保存输入，取消归档不复活旧覆盖，树选择同步',()=>{
  h.click('[data-tn-id="thread:config"]');w.conversationPrototype.scene('archive');assert.equal(state().selected,'thread:config');assert.equal(nav().selected,'thread:config');assert.equal(state().saved['thread:config'],undefined);assert.equal(state().drafts['thread:config'],undefined);assert.ok($('[aria-label="归档前未保存输入"]'));assert.equal($('[data-cv=tree-filter][data-filter=archived]').getAttribute('aria-pressed'),'true');assert.equal($('[data-cv=save]').disabled,true);assert.match($('[data-tn-id="thread:config"]').textContent,/归档/);
  w.conversationPrototype.scene('unarchive');assert.equal(state().saved['thread:config'],undefined);assert.equal(state().drafts['thread:config'],undefined);assert.equal(nav().selected,'thread:config');assert.equal($('[data-cv=tree-filter][data-filter=all]').getAttribute('aria-pressed'),'true');assert.equal($('#cvObjectTree .tn-tree'),tree);
 });
 await t.test('当前对象再次激活可重进窄窗详情，刷新与键盘不触发保存',()=>{
  h.click('[data-cv=back]');assert.equal($('.cv-workspace').classList.contains('cv-detail-open'),false);h.click('[data-tn-id="thread:config"]');assert.equal($('.cv-workspace').classList.contains('cv-detail-open'),true);
  const before=JSON.stringify(state().saved);h.click('[data-cv=refresh]');assert.equal($('#cvObjectTree .tn-tree'),tree);tree.focus();tree.dispatchEvent(new w.KeyboardEvent('keydown',{key:'ArrowDown',bubbles:true}));assert.equal(state().selected,'thread:config');tree.dispatchEvent(new w.KeyboardEvent('keydown',{key:'Enter',bubbles:true}));assert.equal(state().selected,'thread:review');assert.equal(JSON.stringify(state().saved),before);assert.equal(d.activeElement,tree);
  h.input('#cvObjectSearch','不存在');assert.match(tree.textContent,/没有匹配/);assert.equal(state().selected,'thread:review');h.click('[data-tn-clear]');assert.equal($('#cvObjectSearch').value,'');assert.equal(nav().selected,'thread:review');
 });
 await t.test('目录空态保留当前对象，恢复目录不改变配置，未分配仅作为分组',()=>{
  const before=JSON.stringify(state().saved);w.conversationPrototype.scene('empty');assert.equal(nav().selectionMissing,true);assert.match(tree.textContent,/暂无对象/);assert.equal(state().selected,'thread:review');w.conversationPrototype.scene('normal');assert.equal(nav().selectionMissing,false);h.click('[data-tn-id="group:unassigned"]');assert.equal(state().selected,'thread:review');assert.equal(JSON.stringify(state().saved),before);assert.equal(h.network.length,0);assert.deepEqual(h.errors,[]);
 });
 await t.test('标题采用保持稳定key与草稿，搜索外的当前对象名称立即更新',()=>{
  h.click('[data-tn-id="thread:review"]');h.input('#cvObjectSearch','审阅');const saved=JSON.stringify(state().saved);h.click('[data-cv=rename]');h.input('#cvTitleInput','复核继承更新');h.click('#modalConfirm');assert.equal(state().selected,'thread:review');assert.equal(nav().selected,'thread:review');assert.equal(nav().selectedLabel,'复核继承更新');assert.equal(nav().selectionOutside,true);assert.match($('#cvObjectTree .tn-outside').textContent,/复核继承更新/);assert.equal(JSON.stringify(state().saved),saved);h.click('[data-cv=tree-clear]');
 });
 await t.test('配置导入更新干净目标的有效值，真正脏草稿仍阻止采用',async()=>{
  h.click('[data-tn-id="thread:window"]');h.change('[data-cv-field="subagent.effort"]','high');h.change('[data-cv-field="subagent.effort"]','medium');assert.ok(state().drafts['thread:window']);
  h.click('[data-cv=config-import]');h.change('#cvPolicyTarget','thread:window');h.click('#modalConfirm');assert.equal(state().saved['thread:window']['subagent.effort'],'high');assert.equal(state().drafts['thread:window'],undefined);assert.equal(w.conversationPrototype.resolve('thread:window','subagent.effort').value,'high');
  w.setRoute('conversations');await h.settle();assert.equal(nav().selected,'thread:window');h.change('[data-cv-field="subagent.effort"]','low');h.click('[data-cv=config-import]');h.change('#cvPolicyTarget','thread:window');h.click('#modalConfirm');assert.match($('#cvPolicyError').textContent,/未保存/);assert.equal(state().saved['thread:window']['subagent.effort'],'high');assert.equal(state().drafts['thread:window']['subagent.effort'],'low');w.closeModal();w.setRoute('conversations');await h.settle();h.click('[data-cv=discard]');
 });
 await t.test('恢复新项目与父子映射可直接进入公共树，先前筛选不隐藏新选择',async()=>{
  h.click('[data-cv=tree-filter][data-filter=archived]');w.conversationPrototype.openRecovery('import');h.click('[data-cv=rec-file]');h.click('[data-cv=rec-next]');h.click('[data-cv=rec-run]');const until=Date.now()+10000;while(state().recovery?.status==='running'&&Date.now()<until)await h.wait(50);assert.equal(state().recovery.status,'complete');assert.equal(state().imported.length,2);
  const restored=JSON.parse(JSON.stringify(state().imported));h.click('[data-cv=rec-view]');await h.settle();assert.equal(state().selected,'thread:'+restored[0].id);assert.equal(nav().selected,state().selected);assert.equal(nav().selectionOutside,false);assert.equal($('[data-cv=tree-filter][data-filter=all]').getAttribute('aria-pressed'),'true');assert.equal(nav().nodes.find(n=>n.id==='thread:'+restored[1].id).parent,'thread:'+restored[0].id);assert.equal(nav().nodes.find(n=>n.id==='thread:'+restored[0].id).parent,'project:'+restored[0].project);assert.equal($('#cvObjectTree .tn-tree'),tree);assert.ok(nav().nodes.some(n=>n.id==='project:tools'));assert.equal(h.network.length,0);assert.deepEqual(h.errors,[]);
 });

});
