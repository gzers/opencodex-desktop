const test=require('node:test'),assert=require('node:assert/strict');
const contract=require('./modal-contract.js'),{make}=require('./component-topic-harness.cjs');
test('关闭合同区分查看、编辑和真实业务关闭动作',()=>{
 assert.deepEqual(contract.resolve('关闭',false),{mode:'view',primary:false,cancel:false,cancelLabel:'取消'});
 assert.equal(contract.resolve('关闭',true).primary,true); // 关闭CLI控制面是业务动作。
 assert.equal(contract.resolve('复制完整任务',true,{cancelLabel:'关闭'}).cancel,false);
 assert.equal(contract.resolve('复制完整任务',true,{cancelLabel:'关闭'}).primary,true);
 assert.equal(contract.resolve('保存',true,{mode:'form'}).cancel,true);
 assert.equal(contract.resolve('确认更新',true,{mode:'view',hideConfirm:true}).primary,false);
});
test('组件专题的查看/确认/表单复用公共X与分类页脚，取消不提交',async t=>{
 const h=await make('#components?section=overlays'),d=h.document;t.after(()=>h.dispose());assert.deepEqual(h.errors,[]);
 const opener=h.click('[data-ct=dialog-detail]');await h.settle();
 assert.equal(d.querySelectorAll('#modal > .modal-head > #modalClose').length,1);
 assert.equal(d.querySelector('#modalActions').hidden,true);assert.equal(d.querySelector('#modalCancel').hidden,true);assert.equal(d.querySelector('#modalConfirm').hidden,true);
 h.click('#modalClose');assert.equal(d.querySelector('#modalMask').style.display,'none');assert.equal(d.activeElement,opener);
 const before=d.querySelector('#ctDialogResult').textContent;h.click('[data-ct=dialog-confirm]');await h.settle();assert.equal(d.querySelector('#modalActions').hidden,false);assert.equal(d.querySelector('#modalCancel').textContent,'取消');h.click('#modalClose');assert.equal(d.querySelector('#ctDialogResult').textContent,before);
 h.click('[data-ct=dialog-form]');h.input('#ctDialogName','仅本次输入');h.click('#modalCancel');assert.equal(d.querySelector('#ctDialogResult').textContent,before);assert.deepEqual(h.errors,[]);
});
test('查看型保留实际业务动作，不留下底部关闭或空栏',async t=>{
 const h=await make(),d=h.document;t.after(()=>h.dispose());let copies=0;
 h.w.openModal('外部任务','<p>示例任务</p>','复制完整任务',()=>copies++,{mode:'view',confirmKeepsOpen:true});
 assert.equal(d.querySelector('#modalCancel').hidden,true);assert.equal(d.querySelector('#modalConfirm').hidden,false);assert.equal(d.querySelector('#modalActions').hidden,false);h.click('#modalConfirm');assert.equal(copies,1);h.click('#modalClose');
 h.w.openModal('只读结果','<p>未变化</p>','关闭',h.w.closeModal,{hideConfirm:true,cancelLabel:'关闭'});assert.equal(d.querySelector('#modalActions').hidden,true);h.click('#modalClose');assert.deepEqual(h.errors,[]);
});
test('统一关闭守卫覆盖X/Escape/遮罩，任务取消与关闭分离',async t=>{
 const h=await make(),d=h.document;t.after(()=>h.dispose());h.w.openRuntimeInstall('net');h.change('#installAck',true);h.click('#modalConfirm');
 assert.equal(d.querySelector('#modalClose').disabled,true);h.click('#modalMask');assert.equal(d.querySelector('#modalMask').style.display,'flex');
 d.dispatchEvent(new h.w.KeyboardEvent('keydown',{key:'Escape',bubbles:true}));assert.equal(d.querySelector('#modalMask').style.display,'flex');assert.equal(d.querySelector('#modalCancel').textContent,'取消安装');h.click('#modalCancel');assert.equal(d.querySelector('#modalMask').style.display,'none');
 h.w.openRuntimeInstall('net');h.change('#installAck',true);h.click('#modalConfirm');h.w.finishInstall();assert.equal(d.querySelector('#modalClose').disabled,false);assert.equal(d.querySelector('#modalActions').hidden,true);h.click('#modalClose');assert.deepEqual(h.errors,[]);
});
