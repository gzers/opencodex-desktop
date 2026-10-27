/* 集成节点选择器交互；布局和顶层浮层位置仍需浏览器验证。 */
const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
const p=h=>h.state().draft.providers.find(p=>p.id==='newapi');
const policy=h=>JSON.stringify({draft:h.state().draft,saved:h.state().saved,records:h.state().records});
test('列表具有独立节点列，仅单行触发器，无行下检测按钮与结果区',()=>{
 const h=make();assert.match(h.page(),/<span>渠道<\/span><span>服务节点<\/span><span>模型<\/span>/);assert.match(h.page(),/<div class="mp-provider-node-cell"><button id="mpChannelNode-newapi"/);assert.doesNotMatch(h.page(),/mp-provider-node-tools|mp-node-test-actions|data-node-current|data-speed-control/);
 const name=h.page().match(/<div class="mp-provider-name">([\s\S]*?)<\/div>/)[1];assert.doesNotMatch(name,/mpChannelNode|node-picker|button|select/);
 h.click('node-picker-open',{channel:'newapi'});assert.equal(h.state().nodePickerChannel,'newapi');assert.equal(h.nodes.get('mpNodePicker').getAttribute('role'),'dialog');assert.equal(h.nodes.get('mpChannelNode-newapi').getAttribute('aria-expanded'),'true');assert.equal(h.context.document.activeElement.id,'mpNodePickerChoice-primary');
});
test('选择与检测分离，单测更新同一浮层和焦点，随后选节点只改草稿',()=>{
 const h=make(),before=policy(h);h.click('node-picker-open',{channel:'newapi'});const panel=h.nodes.get('mpNodePicker'),test=h.nodes.get('mpNodePickerTest-backup');test.focus();h.click('node-speed-one',{channel:'newapi',id:'backup'});assert.equal(test.textContent,'检测中…');h.drain();assert.equal(h.nodes.get('mpNodePicker'),panel);assert.equal(h.context.document.activeElement,test);assert.match(h.nodes.get('mpNodePickerSpeed-backup').innerHTML,/data-speed-status="success"/);assert.equal(policy(h),before);assert.equal(p(h).activeNodeId,'primary');
 h.click('node-picker-choose',{channel:'newapi',id:'backup'});assert.equal(p(h).activeNodeId,'backup');assert.equal(h.state().saved.providers[0].activeNodeId,'primary');assert.equal(h.state().nodePickerChannel,null);assert.equal(h.context.document.activeElement.id,'mpChannelNode-newapi');assert.equal(h.state().channelDetail,false);
});
test('集成一键检测和停止，保持浮层并同步列表摘要，关闭不取消在途检测',()=>{
 const h=make(),before=policy(h);h.click('node-picker-open',{channel:'newapi'});const panel=h.nodes.get('mpNodePicker'),all=h.nodes.get('mpNodePickerAll');h.click('node-speed-all',{channel:'newapi'});assert.equal(all.dataset.mp,'node-speed-stop');assert.equal(all.textContent,'停止检测');assert.equal(h.nodes.get('mpNodePickerTest-backup').disabled,true);h.click('node-speed-stop',{channel:'newapi'});assert.equal(all.textContent,'一键检测');assert.equal(h.nodes.get('mpNodePicker'),panel);h.drain();assert.match(h.nodes.get('mpNodePickerSpeed-primary').innerHTML,/cancelled/);
 h.click('node-speed-all',{channel:'newapi'});h.click('node-picker-close');h.drain();assert.equal(h.state().nodePickerChannel,null);assert.match(h.nodes.get('mpNodeSpeedCurrent-newapi').innerHTML,/success/);h.click('node-picker-open',{channel:'newapi'});assert.match(h.nodes.get('mpNodePicker').innerHTML,/data-speed-status="success"/);assert.equal(policy(h),before);
});
test('Esc回到触发器，方向键只移焦点，外部点击和焦点离开关闭不选择',()=>{
 const h=make();h.click('node-picker-open',{channel:'newapi'});h.key('ArrowDown',h.nodes.get('mpNodePickerChoice-primary'));assert.equal(h.context.document.activeElement.id,'mpNodePickerChoice-backup');assert.equal(p(h).activeNodeId,'primary');h.key('Escape',h.context.document.activeElement);assert.equal(h.state().nodePickerChannel,null);assert.equal(h.context.document.activeElement.id,'mpChannelNode-newapi');
 h.click('node-picker-open',{channel:'newapi'});h.pointer(h.nodes.get('mpNodePickerTest-primary'));assert.equal(h.state().nodePickerChannel,'newapi');h.pointer(h.nodes.get('mpPolicySave'));assert.equal(h.state().nodePickerChannel,null);h.click('node-picker-open',{channel:'newapi'});h.focusTarget(h.nodes.get('mpPolicySave'));assert.equal(h.state().nodePickerChannel,null);assert.equal(p(h).activeNodeId,'primary');
});
test('单实例、直接连接编辑与保存锁继续有效，返回详情和重绘关闭悬空浮层',()=>{
 const h=make();h.click('node-picker-open',{channel:'newapi'});h.click('node-picker-open',{channel:'aimami'});assert.equal(h.state().nodePickerChannel,'aimami');assert.equal(h.nodes.get('mpChannelNode-newapi').getAttribute('aria-expanded'),'false');
 h.click('edit-provider',{id:'newapi'});assert.equal(h.state().nodePickerChannel,null);h.event('input','[data-channel-config]',{channelConfig:'name'},'连接内存编辑');h.click('back-channels');
 h.click('node-picker-open',{channel:'newapi'});assert.equal(h.state().nodePickerChannel,'newapi');assert.equal(p(h).name,'连接内存编辑');assert.equal(h.state().saved.providers[0].name,'NewApi');h.click('node-picker-close');
 h.click('save');h.runConfirm();h.click('node-picker-open',{channel:'newapi'});assert.equal(h.state().nodePickerChannel,null);h.drain();assert.equal(h.state().saved.providers[0].name,'连接内存编辑');
});
