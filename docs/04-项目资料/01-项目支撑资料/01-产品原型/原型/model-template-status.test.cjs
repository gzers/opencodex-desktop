/* 状态与保存边界接线测试；不替代实际主题、标题高度或焦点验收。 */
const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
function status(h){return h.proto.templateHeadingState();}
test('新建按钮只有保存，不再重复状态；标题列出新建状态并提供8种状态注释',()=>{
 const h=make();h.click('new-template');assert.equal(status(h).id,'new');assert.equal(status(h).tone,'warning');
 const control=h.page().match(/<div id="mpTemplateSaveStatus"[\s\S]*?<\/button><\/div>/)?.[0];
 assert.ok(control);assert.match(control,/aria-label="保存"/);assert.doesNotMatch(control,/保存候选|新模版|未保存|<small/);assert.match(control,/>保存<\/span>/);
 assert.match(h.page(),/模版的 8 种主要状态/);assert.match(h.page(),/annotation-source" hidden><p>编辑状态/);assert.match(h.page(),/mpTemplateHeadingStatus/);
});
test('进入编辑但无变化为未修改；输入、还原和非法字段即时刷新标题状态',()=>{
 const h=make();h.open('gpt');h.click('edit-template');assert.equal(status(h).id,'clean');assert.equal(status(h).tone,'neutral');
 const original=h.state().buffers.gpt.description;h.meta('description','新的说明');assert.equal(status(h).id,'dirty');assert.match(h.nodes.get('mpTemplateHeadingStatus').innerHTML,/data-template-state="dirty"/);
 h.meta('description',original);assert.equal(status(h).id,'clean');h.meta('name','');assert.equal(status(h).id,'invalid');assert.equal(status(h).tone,'danger');h.meta('name',h.state().templates.find(t=>t.id==='gpt').name);assert.equal(status(h).id,'clean');
});
test('新模版先未保存，提交非法内容后红色提示；修正后回到可保存的新建状态',()=>{
 const h=make();h.click('new-template');h.click('save-template');assert.equal(status(h).id,'invalid');assert.equal(h.state().templateScratch.versions.length,0);
 h.match('include','prefix:my-');assert.equal(status(h).id,'new');h.click('save-template');h.runConfirm();assert.equal(status(h).id,'clean');assert.equal(status(h).usage.id,'unadopted');assert.equal(status(h).usage.tone,'info');
});
test('保存生成独立版本但不采用，状态蓝色；候选与旧规则均保留',()=>{
 const h=make();h.open('gpt');h.click('edit-template');h.meta('description','新版本说明');const old=h.state().templates[0].versions[0].rules;h.click('save-template');h.runConfirm();assert.equal(status(h).id,'clean');assert.equal(status(h).usage.id,'unadopted');assert.equal(status(h).usage.tone,'info');assert.deepEqual(h.state().templates[0].versions[0].rules,old);assert.equal(h.state().saved.globalRefs.gpt,'gpt:1');
});
test('已采用待确认橙色，用户实际确认后绿色，换用其他版后历史确认中性保留',()=>{
 const h=make();h.open('gpt');assert.equal(status(h).id,'clean');assert.equal(status(h).usage.id,'pending');assert.equal(status(h).usage.tone,'warning');h.click('confirm-template-use');h.runConfirm();assert.equal(status(h).id,'clean');assert.equal(status(h).usage.id,'confirmed');assert.equal(status(h).usage.tone,'success');
 h.click('edit-template');h.meta('description','待采用版本');h.click('save-template');h.runConfirm();h.click('adopt-version');h.nodes.get('mpAdoptScope').value='global';h.runConfirm();
 h.open('gpt');h.click('adopt-version');h.nodes.get('mpAdoptVersion').value='4';h.nodes.get('mpAdoptScope').value='channels';for(const id of ['newapi','a6api','aimami']){const input=h.nodes.get('mpAdoptChannel-'+id);input.checked=true;input.onchange();}h.nodes.get('mpAdoptVersion').value='4';h.runConfirm();h.click('save');h.runConfirm();h.drain();h.open('gpt');h.event('change','[data-version]',{},'1');
 assert.equal(status(h).id,'clean');assert.equal(status(h).usage.id,'previous');assert.equal(status(h).usage.tone,'neutral');assert.equal(h.state().confirmedConfigurations.length,1);
});
