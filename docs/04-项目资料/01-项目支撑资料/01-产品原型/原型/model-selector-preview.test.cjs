/* 预览顺序与预算共用投影，来源切换只读；不代表真实客户端目录。 */
const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
const policy=h=>JSON.stringify({draft:h.state().draft,saved:h.state().saved,pending:h.state().pending,records:h.state().records});
test('顶部合并为预览选择器唯一入口，列表正文没有重复按钮，详情也进入同一弹窗',()=>{
 const h=make();assert.equal((h.page().match(/data-mp="selector-preview"/g)||[]).length,1);
 const button=h.page().slice(h.page().indexOf('<button class="mp-budget'),h.page().indexOf('</button>',h.page().indexOf('<button class="mp-budget'))+9);assert.match(button,/<span>预览选择器<\/span>/);assert.doesNotMatch(button,/<strong>89\/100/);assert.doesNotMatch(h.page(),/data-mp="order-preview"|data-mp="budget"/);
 h.click('selector-preview');assert.match(h.dialog(),/完整选择器顺序/);assert.match(h.dialog(),/模型激活数 89\/100/);h.context.closeModal();h.click('edit-provider',{id:'newapi'});assert.equal((h.page().match(/data-mp="selector-preview"/g)||[]).length,1);h.click('selector-preview');assert.match(h.dialog(),/模型激活数 89\/100/);
});
test('默认预览草稿，数量/基础/Fast/隐藏项与展示条目一致，停用示例没有发布项',()=>{
 const h=make(),before=policy(h);h.click('selector-preview');const rows=h.proto.projection(),base=rows.filter(r=>r.type==='base').length,fast=rows.filter(r=>r.type==='fast').length,hidden=rows.filter(r=>r.type==='hidden').length;
 assert.ok(h.dialog().includes('基础模型 '+base+' · Fast '+fast+' · 系统占用 '+hidden));assert.equal((h.dialog().match(/class="mp-preview-item /g)||[]).length,rows.length);assert.equal((h.dialog().match(/class="mp-preview-item fast"/g)||[]).length,fast);assert.equal((h.dialog().match(/class="mp-preview-item hidden-item"/g)||[]).length,hidden);assert.match(h.dialog(),/停用 · 保留已选/);assert.doesNotMatch(h.dialog(),/<h3>示例渠道 01<\/h3>/);assert.equal(policy(h),before);
});
test('切换已保存/草稿来源同步更新总数与完整顺序，不保存/丢弃草稿或改变状态',()=>{
 const h=make(),savedCount=h.proto.projection().length;h.click('toggle-provider',{id:'aimami'});const before=policy(h),draftCount=h.proto.projection().length;assert.notEqual(draftCount,savedCount);h.click('selector-preview');assert.ok(h.dialog().includes('模型激活数 '+draftCount+'/100'));assert.doesNotMatch(h.dialog(),/<h3>Aimami<\/h3>/);
 h.event('change','[data-preview]',{},'saved');assert.ok(h.dialog().includes('模型激活数 '+savedCount+'/100'));assert.match(h.dialog(),/<h3>Aimami<\/h3>/);h.event('change','[data-preview]',{},'draft');assert.ok(h.dialog().includes('模型激活数 '+draftCount+'/100'));assert.equal(policy(h),before);
});
test('超限与待重启保持真实语义，预览仍可打开且不解除保存门禁',()=>{
 const h=make();h.proto.scene('over-budget');h.click('selector-preview');assert.match(h.dialog(),/mp-preview-budget-summary over-budget/);assert.match(h.dialog(),/超出激活预算/);h.context.closeModal();h.click('save');assert.equal(h.state().operation,null);
 h.proto.scene('pending-draft');const before=policy(h);h.click('selector-preview');h.event('change','[data-preview]',{},'saved');assert.match(h.dialog(),/仍待重启 \/ 客户端核对/);assert.equal(policy(h),before);
});
