const test=require('node:test'),assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');
test('交互集合：当前行、详情、勾选、操作和共用草稿分别保持',async t=>{
 const h=await make('#components?section=interactive-collections');t.after(()=>h.dispose());const d=h.document,w=h.w,s=()=>w.interactiveCollectionDemo.state();assert.deepEqual(h.errors,[]);
 const initialBusiness=JSON.stringify(w.extensionPrototype.state());
 await t.test('组件同一渲染器提供列表/表格，标题可看详情且不激活',()=>{
  assert.equal(d.querySelector('#icLeftRows').dataset.kind,'list');assert.equal(d.querySelector('#icRightRows').dataset.kind,'table');
  h.click('#icLeftRows [data-ic-detail=development]');assert.match(d.querySelector('#modalTitle').textContent,/development-knowledge/);assert.equal(s().current,'design');assert.equal(s().picks.length,0);w.closeModal();
 });
 await t.test('行激活和复选分别处理，多个资源关系出现半选',()=>{
  h.click('#icLeftRows [data-ic-row=development]');assert.equal(s().current,'development');
  h.click('#icLeftRows [data-ic-check=design]');h.click('#icLeftRows [data-ic-check=development]');assert.equal(s().current,'development');assert.equal(s().picks.length,2);
  assert.equal(d.querySelector('#icRightRows [data-ic-check=claude]').indeterminate,true);
 });
 await t.test('行内操作不激活或改选择，详情关闭返回入口',()=>{
  const before=JSON.stringify(s());h.click('#icRightRows [data-ic-action=detail][data-id=cursor]');assert.equal(JSON.stringify(s()),before);w.closeModal();
  h.click('#icLeftRows [data-ic-action=more][data-id=governance]');assert.equal(s().current,'development');assert.ok(w.prototypeActionMenu);w.prototypeActionMenu.close();
 });
 await t.test('勾选修改共同草稿，切视图与搜索保留同一关系',()=>{
  h.click('#icRightRows [data-ic-check=cursor]');assert.equal(s().draft.length,2);
  h.click('[data-ic-demo=mode][data-value=client]');assert.equal(s().mode,'client');assert.equal(d.querySelectorAll('#icLeftRows [data-ic-check]').length,0);
  h.click('#icLeftRows [data-ic-row=cursor]');assert.equal(d.querySelector('#icRightRows [data-ic-check=design]').checked,true);assert.equal(s().draft.length,2);
  h.input('#icLeftSearch','zz-not-found');assert.match(d.querySelector('#icLeftRows').textContent,/没有匹配/);assert.equal(s().current,'cursor');assert.equal(s().draft.length,2);
  h.click('[data-ic-demo=reveal]');assert.equal(s().query,'');
  h.click('[data-ic-demo=mode][data-value=resource]');assert.equal(s().current,'development');assert.equal(s().picks.length,2);
 });
 await t.test('预览取消保留草稿，应用后两视图看到同一结果，业务状态不变',()=>{
  h.click('[data-ic-demo=preview]');assert.match(d.querySelector('#modalBody').textContent,/Cursor/);w.closeModal();assert.equal(s().draft.length,2);
  h.click('[data-ic-demo=preview]');h.click('#modalConfirm');assert.equal(s().draft.length,0);assert.ok(s().saved.includes('design|cursor'));assert.ok(s().saved.includes('development|cursor'));
  assert.equal(JSON.stringify(w.extensionPrototype.state()),initialBusiness);assert.equal(h.network.length,0);assert.deepEqual(h.errors,[]);
 });
});

test('启用复选与行内操作同源，原扩展详情在旧行和公共样例共用',async t=>{
 const h=await make('#components?section=interactive-collections');t.after(()=>h.dispose());const d=h.document,w=h.w,s=()=>w.interactiveCollectionDemo.state();
 await t.test('快速启用、行内停用、半选全部启用且不激活其他行',()=>{
  h.click('#icRightRows [data-ic-check=cursor]');assert.equal(s().draft.length,1);assert.equal(d.querySelector('#icRightRows [data-ic-action=toggle][data-id=cursor]').textContent,'停用');
  h.click('#icRightRows [data-ic-action=toggle][data-id=cursor]');assert.equal(s().draft.length,0);assert.equal(s().current,'design');assert.equal(s().picks.length,0);
  h.click('#icLeftRows [data-ic-check=design]');h.click('#icLeftRows [data-ic-check=development]');const toggle=d.querySelector('#icRightRows [data-ic-action=toggle][data-id=claude]');assert.equal(toggle.textContent,'全部启用');assert.equal(toggle.getAttribute('aria-pressed'),'mixed');h.click('#icRightRows [data-ic-action=toggle][data-id=claude]');assert.equal(d.querySelector('#icRightRows [data-ic-check=claude]').checked,true);assert.equal(s().draft.length,1);
  assert.equal(d.querySelector('#icRightRows [data-ic-action=toggle][data-id=gemini]').disabled,true);
 });
 await t.test('Skill复用正文、字段、标题动作与目标，详情修改返回同一草稿',()=>{
  const name=d.querySelector('#skillsList .skill-title h3').textContent;w.openExtensionDetail(d.querySelector('#skillsList .skill-row'));assert.ok(d.querySelector('.skill-detail-doc'));assert.match(d.querySelector('#modalTitle').textContent,new RegExp(name));w.closeModal();
  h.click('#icLeftRows [data-ic-detail=design]');assert.ok(d.querySelector('.skill-detail-md h4'));assert.match(d.querySelector('.skill-detail-facts').textContent,/入口文件SKILL.md/);assert.ok(d.querySelector('[aria-label="检查 / 更新 design-studio"]'));assert.ok(d.querySelector('[aria-label="卸载 design-studio"]'));
  h.click('[data-extension-detail-target=cursor]');assert.equal(s().draft.length,2);assert.equal(s().current,'design');assert.equal(s().picks.length,2);w.closeModal();assert.equal(d.activeElement.dataset.icDetail,'design');
 });
 await t.test('MCP复用命令、参数、环境变量和配置落点，标题编辑删除仍可见',()=>{
  h.click('[data-ic-demo=kind][data-value=mcp]');h.click('#icLeftRows [data-ic-detail=filesystem]');assert.match(d.querySelector('.skill-detail-facts').textContent,/命令node/);assert.match(d.querySelector('.skill-detail-facts').textContent,/WORKSPACE_ROOT/);assert.match(d.querySelector('.skill-detail-facts').textContent,/配置落点/);assert.ok(d.querySelector('[aria-label="编辑 文件系统"]'));assert.ok(d.querySelector('[aria-label="删除 文件系统"]'));w.closeModal();assert.deepEqual(h.errors,[]);assert.equal(h.network.length,0);
 });
});
