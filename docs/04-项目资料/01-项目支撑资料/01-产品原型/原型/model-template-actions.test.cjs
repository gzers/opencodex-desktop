/* 动作分工与退出 / 外部任务边界；不代表菜单实际布局。 */
const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
function menus(h){let menu;h.context.window.prototypeActionMenu={open(options){menu=options;},close(){}};return ()=>menu;}
function modalOptions(h){let opts;const original=h.context.openModal;h.context.openModal=(...args)=>{opts=args[4];return original(...args);};return ()=>opts;}
test('复制模版直接显示，点击以当前内容建立独立新模版缓冲，不保留单项更多菜单',()=>{
 const h=make();h.open('gpt');h.click('edit-template');const page=h.page();assert.match(page,/外部任务/);assert.doesNotMatch(page,/生成外部任务|导入外部结果|data-mp="external-result"/);assert.match(page,/版本与历史/);assert.match(page,/data-mp="copy-template"/);assert.doesNotMatch(page,/模版工具|同步配置|取消候选编辑|data-mp="compare"|data-mp="template-more"/);
 const library=JSON.stringify(h.state().templates);h.click('copy-template');assert.notEqual(h.nodes.get('modalMask').style.display,'flex');assert.ok(h.state().templateScratch);assert.equal(JSON.stringify(h.state().templates),library);
});
test('版本与历史可比较编辑内容，变更版本动作禁用；比较基准可切且不改草稿',()=>{
 const h=make(),menu=menus(h);h.open('gpt');h.click('edit-template');h.meta('description','新的说明');const before=JSON.stringify(h.state().buffers);h.click('version-menu');assert.equal(menu().label,'版本与历史');assert.ok(menu().items.find(i=>i.label==='与版本比较'));assert.equal(menu().items.some(i=>i.label==='设为以后默认'),false);menu().items.find(i=>i.label==='版本管理').onSelect();assert.equal(h.nodes.get('mpVersionDefault-gpt-2').disabled,true);h.click('version-menu');
 menu().items.find(i=>i.label==='与版本比较').onSelect();assert.ok(h.nodes.has('mpCompareVersion'));h.nodes.get('mpCompareVersion').value='2';h.nodes.get('mpCompareVersion').onchange();assert.match(h.nodes.get('mpVersionComparison').innerHTML,/新的说明/);assert.equal(JSON.stringify(h.state().buffers),before);
});
test('无变化返回直接清除编辑缓冲，已保存版本与采用配置保持',()=>{
 const h=make();h.open('gpt');h.click('edit-template');const versions=JSON.stringify(h.state().templates),policy=JSON.stringify(h.state().saved);h.click('back-templates');assert.equal(h.state().templateDetail,false);assert.equal(h.state().buffers.gpt,undefined);assert.notEqual(h.nodes.get('modalMask').style.display,'flex');assert.equal(JSON.stringify(h.state().templates),versions);assert.equal(JSON.stringify(h.state().saved),policy);
});
test('有修改返回可继续编辑或一轮差异确认保存并返回，保存不自动采用',()=>{
 const h=make(),options=modalOptions(h);h.open('gpt');h.click('edit-template');h.meta('description','退出前保存');h.click('back-templates');assert.equal(options().cancelLabel,'继续编辑');assert.equal(options().extraLabel,'放弃修改');h.context.closeModal();assert.equal(h.state().buffers.gpt.description,'退出前保存');h.click('back-templates');h.runConfirm();assert.equal(h.state().templateDetail,false);assert.equal(h.state().buffers.gpt,undefined);assert.equal(h.state().templates[0].versions.at(-1).description,'退出前保存');assert.equal(h.state().templates[0].main,1);assert.equal(h.state().saved.globalRefs.gpt,'gpt:1');
});
test('放弃只撤销当前未保存编辑，新建放弃不入库，已保存的新版本仍保留',()=>{
 const h=make(),options=modalOptions(h);h.open('gpt');h.click('edit-template');h.meta('description','保存版本');h.click('save-template');h.runConfirm();const versions=JSON.stringify(h.state().templates);h.click('edit-template');h.meta('description','不要的修改');h.click('back-templates');options().onExtra();assert.equal(h.state().buffers.gpt,undefined);assert.equal(JSON.stringify(h.state().templates),versions);
 h.click('new-template');h.click('back-templates');assert.equal(h.nodes.get('modalConfirm').disabled,true);options().onExtra();assert.equal(h.state().templateScratch,null);assert.equal(JSON.stringify(h.state().templates),versions);
});
test('返回确认打开后又编辑使旧保存与放弃过期，不丢新输入',()=>{
 const h=make(),options=modalOptions(h);h.open('gpt');h.click('edit-template');h.meta('description','旧输入');h.click('back-templates');const old=options().onExtra;h.meta('description','确认期间的新输入');old();assert.equal(h.state().buffers.gpt.description,'确认期间的新输入');assert.match(h.nodes.get('mpTemplateSaveError').textContent,/已变化/);h.runConfirm();assert.equal(h.state().templates[0].versions.length,3);
});
test('放弃正在生成的编辑后，晚返回不复活缓冲或创建版本',()=>{
 const h=make(),options=modalOptions(h);h.configureAgent();h.open('gpt');h.click('edit-template');h.meta('description','正在编辑');h.click('generate');h.nodes.get('mpGenerationTarget').value='默认高';h.click('generate-run');h.click('back-templates');options().onExtra();h.drain();assert.equal(h.state().buffers.gpt,undefined);assert.equal(h.state().proposal,null);assert.equal(h.state().templates[0].versions.length,3);
});
test('兼容回传保留缺关联校验，不能跳任务创建或写入',()=>{
 const h=make(),before=JSON.stringify(h.state().templates);h.click('external-result');assert.ok(h.nodes.has('mpExternalTaskJSON'));assert.ok(h.nodes.has('mpExternalJSON'));assert.equal(h.nodes.has('mpExternalScope'),false);h.runConfirm();assert.match(h.nodes.get('mpExternalResultError').textContent,/关联原任务/);assert.equal(h.state().externalTask,null);assert.equal(JSON.stringify(h.state().templates),before);
});
test('首次导入可关联任务包，校验与比较后仅新增候选；无效或过期关联不替换原任务',()=>{
 const h=make(),task={kind:'template-task-batch',taskId:'import-create-task',targets:[]},content={name:'外部私有模型族',description:'待采用',match:{include:[{type:'prefix',value:'private-'}],exclude:[],caseSensitive:false,channelMode:'all',channels:[],priority:0},rules:{}};
 h.click('external-result');h.nodes.get('mpExternalTaskJSON').value=JSON.stringify(task);h.nodes.get('mpExternalJSON').value=JSON.stringify({kind:'template-batch-result',taskId:task.taskId,operations:[{action:'create',id:'private-import',revision:1,baseDigest:'',content}]});h.runConfirm();assert.equal(h.state().templates.some(t=>t.id==='private-import'),false);h.runConfirm();assert.equal(h.state().templates.find(t=>t.id==='private-import').main,0);
 const current=JSON.stringify(h.state().externalTask);h.click('external-result');h.nodes.get('mpExternalTaskJSON').value=JSON.stringify({...task,targets:[{id:'gpt',version:1,libraryDigest:'bad'}]});h.runConfirm();assert.match(h.nodes.get('mpExternalResultError').textContent,/已变化/);assert.equal(JSON.stringify(h.state().externalTask),current);
});
test('同一弹窗内完整任务一次复制，不变更候选/采用，不显示会话任务队列',()=>{
 const h=make();h.open('gpt');const before=JSON.stringify(h.state().templates);h.click('external-new');const modal=h.nodes.get('modalBody'),task=h.state().externalTask,content=h.nodes.get('mpExternalPrompt').value;
 assert.ok(h.nodes.has('mpExternalScope'));assert.ok(h.nodes.has('mpExternalScene'));assert.equal(content,h.proto.externalCombinedText());assert.doesNotMatch(h.dialog(),/复制提示词|复制任务资料|本次会话任务|新建任务|兼容回传/);h.runConfirm();assert.equal(h.nodes.get('modalBody'),modal);assert.equal(h.state().externalTask.taskId,task.taskId);assert.equal(h.nodes.get('mpExternalPrompt').value,content);assert.equal(JSON.stringify(h.state().templates),before);assert.equal(h.state().saved.globalRefs.gpt,'gpt:1');
});
