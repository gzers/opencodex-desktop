const test=require('node:test'),assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');
const {pickModel,selectRow,editOptions}=require('./conversation-picker-test-helper.cjs');
const clean=h=>{assert.deepEqual(h.errors,[]);assert.equal(h.network.length,0);};
test('所有作用域模型均使用公共选择器，普通枚举保留Select，命名只在全局',async t=>{
 const h=await make('#settings?section=dialog-policy&policy=models'),{w,document:d}=h;t.after(()=>h.dispose());
 assert.equal(d.querySelectorAll('#conversationGlobalPolicy [data-cv-picker]').length,4);assert.equal(d.querySelectorAll('#conversationContent [data-cv-picker]').length,3);assert.equal(d.querySelector('#conversationContent [data-cv-picker="naming.model"]'),null);
 assert.equal(d.querySelectorAll('#conversationGlobalPolicy select[data-cv-field$=".model"],#conversationContent select[data-cv-field$=".effort"],select[data-cv-field="subagent.roster"]').length,0);assert.ok(d.querySelector('select[data-cv-field="search.path"]'));
 h.click('#conversationGlobalPolicy [data-cv-picker="naming.model"]');assert.equal(w.modelPickerPrototype.state().mode,'single');assert.equal(w.modelPickerPrototype.state().reasoningEnabled,false);assert.equal(w.modelPickerPrototype.state().fastEnabled,false);assert.ok(d.querySelector('[aria-label="模型渠道"]'));assert.ok(d.querySelector('.mpk-table'));h.click('#modalCancel');clean(h);
});
test('单选确认只改当前草稿，取消不变，同模型仅改档位保留型号继承',async t=>{
 const h=await make('#conversations'),{w,document:d}=h,state=()=>w.conversationPrototype.state();t.after(()=>h.dispose());
 h.click('[data-tn-id="thread:review"]');const before=JSON.stringify(state().saved);
 h.click('#conversationContent [data-cv-picker="subagent.model"]');selectRow(h,'newapi|gpt-5.4');editOptions(h,'xhigh',true);assert.equal(state().drafts['thread:review'],undefined);h.click('#modalCancel');assert.equal(state().drafts['thread:review'],undefined);
 pickModel(h,'#conversationContent','subagent.model','newapi|gpt-5.4','xhigh',true);const draft=state().drafts['thread:review'];assert.equal(draft['subagent.model'],undefined);assert.equal(draft['subagent.effort'],'xhigh');assert.equal(draft['subagent.fast'],true);assert.equal(JSON.stringify(state().saved),before);assert.match(d.querySelector('#conversationContent [data-cv-picker="subagent.model"]').textContent,/极高.*Fast/);
 h.click('#conversationContent [data-cv=save]');assert.equal(state().saved['thread:review']['subagent.fast'],true);clean(h);
});
test('更换模型不套用原型号推理或Fast，恢复继承清除本层组合',async t=>{
 const h=await make('#conversations'),{w,document:d}=h,state=()=>w.conversationPrototype.state();t.after(()=>h.dispose());
 pickModel(h,'#conversationContent','subagent.model','newapi|gpt-5.4','xhigh',true);pickModel(h,'#conversationContent','subagent.model','newapi|gpt-5.4-mini');const p=state().drafts['project:tools'];assert.equal(p['subagent.model'],'newapi|gpt-5.4-mini');assert.equal(p['subagent.effort'],'model-default');assert.equal(p['subagent.fast'],false);
 h.click('#conversationContent [data-cv=model-inherit][data-field="subagent.model"]');const reset=state().drafts['project:tools'];assert.equal(Object.hasOwn(reset,'subagent.model'),false);assert.equal(Object.hasOwn(reset,'subagent.effort'),false);assert.equal(Object.hasOwn(reset,'subagent.fast'),false);assert.equal(w.conversationPrototype.resolve('project:tools','subagent.model').from,'Codex / OpenCodex 默认');clean(h);
});
test('无改动确认不创建覆盖，模型缺失和停用不能绕过公共组件确认',async t=>{
 const h=await make('#settings?section=dialog-policy&policy=models'),{w,document:d}=h;t.after(()=>h.dispose());
 h.click('#conversationGlobalPolicy [data-cv-picker="search.model"]');h.click('#modalConfirm');assert.equal(w.conversationPrototype.state().drafts.global,undefined);
 h.click('#conversationGlobalPolicy [data-cv-picker="naming.model"]');const blocked=[...d.querySelectorAll('.mpk-row.is-disabled')];assert.ok(blocked.length);assert.ok(blocked.every(row=>row.getAttribute('aria-disabled')==='true'));h.click('#modalCancel');clean(h);
});
test('推荐列表多选返回各自推理Fast与优先级，可为空并取消后保留',async t=>{
 const h=await make('#settings?section=dialog-policy&policy=models'),{w,document:d}=h,state=()=>w.conversationPrototype.state();t.after(()=>h.dispose());
 h.click('#conversationGlobalPolicy [data-cv-picker="subagent.roster"]');assert.equal(w.modelPickerPrototype.state().mode,'multiple');selectRow(h,'newapi|gpt-5.4');editOptions(h,'high',true);assert.equal(state().drafts.global,undefined);h.click('#modalConfirm');const refs=state().drafts.global['subagent.roster'];assert.equal(refs.length,2);assert.equal(refs[1].modelId,'gpt-5.4');assert.equal(refs[1].reasoningEffort,'high');assert.equal(refs[1].fast,true);
 h.click('#conversationGlobalPolicy [data-cv-picker="subagent.roster"]');selectRow(h,'newapi|gpt-5.4');h.click('#modalCancel');assert.deepEqual(state().drafts.global['subagent.roster'],refs);
 h.click('#conversationGlobalPolicy [data-cv-picker="subagent.roster"]');h.click('[data-mpk="clear-picked"]');h.click('#modalConfirm');assert.equal(state().drafts.global['subagent.roster'].length,0);h.click('#conversationGlobalPolicy [data-cv=save]');assert.equal(state().saved.global['subagent.roster'].length,0);clean(h);
});
test('恢复映射复用公共弹窗，保留路径输入与模型推理Fast组合',async t=>{
 const h=await make('#conversations'),{w,document:d}=h,state=()=>w.conversationPrototype.state();t.after(()=>h.dispose());
 w.conversationPrototype.openRecovery('import');h.click('[data-cv=rec-file]');h.click('[data-cv=rec-next]');h.input('#cvImportRoot','~/Recovery Target');h.click('[data-cv=rec-model]');assert.ok(d.querySelector('.mpk-table'));selectRow(h,'newapi|gpt-5.4');editOptions(h,'xhigh',true);h.click('#modalConfirm');assert.equal(d.querySelector('#cvImportRoot').value,'~/Recovery Target');assert.match(d.querySelector('[data-cv=rec-model]').textContent,/极高.*Fast/);
 h.click('[data-cv=rec-run]');const until=Date.now()+10000;while(state().recovery?.status==='running'&&Date.now()<until)await h.wait(40);assert.equal(state().recovery.status,'complete');const id=state().imported[0].project;assert.equal(state().saved['project:'+id]['subagent.effort'],'xhigh');assert.equal(state().saved['project:'+id]['subagent.fast'],true);clean(h);
});
