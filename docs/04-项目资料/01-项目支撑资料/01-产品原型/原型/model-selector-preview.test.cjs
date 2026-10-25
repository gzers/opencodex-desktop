/* 保存与预览合并：保存先打开当前草稿的保存预览，确认才写入；不代表真实客户端目录。 */
const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
const policy=h=>JSON.stringify({draft:h.state().draft,saved:h.state().saved,pending:h.state().pending,records:h.state().records});
test('保存是唯一入口：先打开保存预览，列表与详情都不再有独立预览按钮',()=>{
 const h=make();assert.doesNotMatch(h.page(),/data-mp="selector-preview"|data-mp="mp-budget"|id="mpBudget"/);assert.doesNotMatch(h.page(),/data-mp="order-preview"|data-mp="budget"/);
 const button=h.page().match(/<button id="mpPolicySave"[^>]*>/)[0];assert.match(button,/aria-label="保存"/);assert.doesNotMatch(button,/先预览当前草稿/);
 h.click('save');assert.match(h.dialog(),/完整选择器顺序/);assert.match(h.dialog(),/模型激活数 89\/100/);assert.doesNotMatch(h.dialog(),/本次变更与写入|协议写入计划/);h.context.closeModal();h.click('edit-provider',{id:'newapi'});h.click('save');assert.match(h.dialog(),/模型激活数 89\/100/);
});
test('预览固定当前草稿，展示全部可选型号与不随启停变化的 Fast 文字标记',()=>{
 const h=make(),before=policy(h);h.click('save');const rows=h.proto.projection(),base=rows.filter(r=>r.type==='base').length,fast=rows.filter(r=>r.type==='fast').length,hidden=rows.filter(r=>r.type==='hidden').length;
 assert.doesNotMatch(h.dialog(),/预览来源|data-preview|mpPreviewSource/);assert.ok(h.dialog().includes('基础模型 '+base+' · Fast '+fast+' · 系统占用 '+hidden));const selected=h.state().draft.providers.filter(p=>p.enabled&&p.id!=='native').reduce((n,p)=>n+(p.mode==='all'?p.models.length:p.models.filter(m=>m.selected).length),0);assert.ok((h.dialog().match(/data-mp="preview-model-toggle"/g)||[]).length>selected);assert.equal((h.dialog().match(/class="mp-preview-fast"/g)||[]).length,(h.dialog().match(/class="mp-preview-item is-on has-fast"/g)||[]).length+(h.dialog().match(/class="mp-preview-item is-off has-fast"/g)||[]).length);assert.equal((h.dialog().match(/class="mp-preview-item hidden-item"/g)||[]).length,hidden);assert.match(h.dialog(),/已停用 · 点击恢复/);assert.equal(policy(h),before);
});
test('点击渠道停用隐藏整组模型并保留恢复入口，再次点击恢复原选择和顺序',()=>{
 const h=make();h.click('save');const source=h.state().draft.providers.find(p=>p.id==='aimami'),selected=source.models.filter(m=>m.selected).map(m=>m.key),order=source.models.map(m=>m.key);assert.match(h.dialog(),/data-mp="preview-channel-toggle" data-id="aimami"/);
 h.click('preview-channel-toggle',{id:'aimami'});const off=h.state().draft.providers.find(p=>p.id==='aimami');assert.equal(off.enabled,false);assert.deepEqual(off.models.filter(m=>m.selected).map(m=>m.key),selected);assert.deepEqual(off.models.map(m=>m.key),order);assert.match(h.dialog(),/mp-preview-group is-off/);assert.match(h.dialog(),/点击恢复该渠道/);assert.doesNotMatch(h.dialog(),new RegExp('preview-model-toggle" data-id="'+selected[0].replace(/[:]/g,'\\\\:')));
 h.click('preview-channel-toggle',{id:'aimami'});assert.equal(h.state().draft.providers.find(p=>p.id==='aimami').enabled,true);assert.match(h.dialog(),/data-mp="preview-model-toggle"/);assert.equal(JSON.stringify(h.state().saved),JSON.stringify(h.state().saved));
});
test('点击型号启停进入当前草稿，停用标签保留恢复；关闭弹窗不丢失，保存失败仍保留草稿',()=>{
 const h=make();h.click('save');const key='aimami::0',model=h.state().draft.providers.find(p=>p.id==='aimami').models[0],before=model.selected;h.click('preview-model-toggle',{id:key});
 assert.equal(h.state().draft.providers.find(p=>p.id==='aimami').models[0].selected,!before);assert.match(h.dialog(),/class="mp-preview-item is-off[^>]*data-mp="preview-model-toggle" data-id="aimami::0"/);h.click('preview-model-toggle',{id:key});assert.equal(h.state().draft.providers.find(p=>p.id==='aimami').models[0].selected,before);
 h.click('preview-model-toggle',{id:key});h.context.closeModal();assert.equal(h.state().draft.providers.find(p=>p.id==='aimami').models[0].selected,!before);h.click('save');h.runConfirm();h.drain();assert.equal(h.state().saved.providers.find(p=>p.id==='aimami').models[0].selected,!before);
});
test('原生渠道与系统占用只读，快速启停不绕过保存门禁',()=>{
 const h=make();h.proto.scene('over-budget');h.click('save');assert.match(h.dialog(),/mp-preview-group is-off/);assert.doesNotMatch(h.dialog(),/data-mp="preview-channel-toggle" data-id="native"/);assert.equal((h.dialog().match(/class="mp-preview-item hidden-item"/g)||[]).length,2);h.context.closeModal();h.click('save');assert.equal(h.state().operation,null);
});
test('待重启提示保留且没有来源切换所需的保存/丢弃副作用',()=>{
 const h=make();h.proto.scene('pending-draft');const before=policy(h);h.click('save');assert.match(h.dialog(),/仍待重启 \/ 客户端核对/);assert.doesNotMatch(h.dialog(),/data-preview/);assert.equal(policy(h),before);
});
