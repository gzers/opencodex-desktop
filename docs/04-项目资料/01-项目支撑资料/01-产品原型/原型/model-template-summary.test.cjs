/* 摘要的证据与各保存路径一致性，不用执行来源/采用状态替代内容比较。 */
const {test}=require('node:test'),assert=require('node:assert/strict');
const summary=require('./model-template-summary.js'),make=require('./model-prototype-test-harness.cjs');
const base=()=>({name:'模型规则',description:'原说明',match:{include:[{type:'prefix',value:'gpt-'}],exclude:[],channels:[],channelMode:'all',caseSensitive:false,priority:0},rules:{protocol:'Responses',efforts:['low','medium'],default:'medium'},variants:[]});
const tpl=h=>h.state().templates.find(t=>t.id==='gpt');
test('示例版本摘要来自实际相邻差异，选版本同步摘要且不再假装候选状态',()=>{
 const h=make(),versions=tpl(h).versions;assert.equal(versions[0].summary,'内置基线');assert.equal(versions[1].summary,'调整可选推理等级；调整默认推理等级');assert.equal(versions[2].summary,'调整可选推理等级');assert.equal(versions[2].summaryFacts.basis.revisionId,versions[1].revisionId);assert.equal(versions[2].summaryFacts.changes[0].path.join('.'),'rules.efforts');
 h.click('tab',{tab:'templates'});h.event('change','[data-template-row-version]',{templateRowVersion:'gpt'},'3');assert.match(h.page(),/title="调整可选推理等级">调整可选推理等级/);assert.doesNotMatch(h.page(),/新增候选配置|初始配置/);assert.match(h.page(),/版本摘要的生成依据/);
});
test('字段摘要区分设置/调整/恢复继承，保留字段前后事实和来源身份',()=>{
 const old=base(),next=base();next.rules.context=128000;next.rules.default='low';delete next.rules.protocol;next.description='新说明';const result=summary.summarize(old,next,{kind:'revision',templateId:'sample',revisionId:'original'});
 assert.deepEqual(result.entries,['修改模版说明','恢复模型协议继承','设置上下文窗口','调整默认推理等级']);assert.match(result.text,/另 1 项/);assert.match(result.fullText,/调整默认推理等级/);assert.equal(result.changes.find(c=>c.path.join('.')==='rules.context').after,128000);assert.equal(result.changes.find(c=>c.path.join('.')==='rules.default').before,'medium');assert.equal(result.basis.revisionId,'original');assert.equal(old.rules.context,undefined);
});
test('对象键/集合排列变化不虚构修改，渠道范围all的非参与清单也不产生变化',()=>{
 const old=base(),next=base();old.rules.modalities=['text','image'];next.rules={default:'medium',efforts:['medium','low'],modalities:['image','text'],protocol:'Responses'};next.match.channels=['irrelevant-channel'];const result=summary.summarize(old,next,{kind:'revision'});assert.equal(result.changed,false);assert.deepEqual(result.changes,[]);assert.equal(result.text,'内容无变化');
});
test('型号按稳定ID比较增删/字段/顺序，摘要数量基于实际差异，不是建议条数',()=>{
 const old=base(),next=base();old.variants=[{id:'a',name:'A',suffix:'-a',rules:{context:64000}},{id:'b',name:'B',suffix:'-b',rules:{}},{id:'deleted',name:'旧规则',suffix:'-old',rules:{}}];next.variants=[{id:'b',name:'B',suffix:'-b',rules:{}},{id:'a',name:'A',suffix:'-a',rules:{context:128000}},{id:'new1',name:'新规则1',suffix:'-new1',rules:{}},{id:'new2',name:'新规则2',suffix:'-new2',rules:{}}];
 const result=summary.summarize(old,next,{kind:'revision'});assert.deepEqual(result.entries,['新增 2 个型号规则','删除 1 个型号规则','调整型号「A」上下文窗口','调整型号规则顺序']);assert.equal(result.changes.find(c=>c.path.join('.')==='variants.added').after.length,2);assert.match(result.text,/另 1 项/);assert.match(result.fullText,/调整型号规则顺序/);
});
test('首次/复制/内置基线按事实说明，来源缺失不猜候选或初始，无变化为明确空差异',()=>{
 assert.equal(summary.summarize(null,base(),{kind:'create'}).text,'首次创建模版');assert.equal(summary.summarize(null,base(),{kind:'baseline'}).text,'内置基线');assert.equal(summary.summarize(base(),base(),{kind:'copy'}).text,'复制为独立模版');assert.equal(summary.summarize(null,base(),{kind:'revision'}).text,'变更依据缺失');assert.equal(summary.summarize(base(),base(),{kind:'revision'}).changed,false);
});
test('人工保存冻结具体摘要，多项简短展示和完整记录一致，换默认与查看版本不改摘要',()=>{
 const h=make();h.open('gpt');const original=JSON.stringify(tpl(h).versions[0]);h.meta('name','GPT个人规则');h.meta('description','新的说明');h.match('include','prefix:gpt-custom-');h.match('priority','9');h.click('save-template');h.runConfirm();const v=tpl(h).versions.at(-1);assert.deepEqual(v.summaryFacts.entries,['修改模版名称','修改模版说明','调整包含规则','调整匹配优先级']);assert.match(v.summary,/另 1 项/);assert.equal(v.summaryFacts.basis.revisionId,tpl(h).versions[0].revisionId);const evidence=JSON.stringify(v.summaryFacts);assert.equal(JSON.stringify(tpl(h).versions[0]),original);
 h.click('set-main',{id:'gpt:'+v.n});h.runConfirm();assert.equal(JSON.stringify(tpl(h).versions.at(-1).summaryFacts),evidence);h.click('back-templates');assert.ok(h.page().includes(v.summaryFacts.fullText));
 h.event('change','[data-template-row-version]',{templateRowVersion:'gpt'},'1');h.click('open-template-editor',{id:'gpt'});assert.equal(h.proto.templateHeadingState().id,'clean');assert.equal(h.state().buffers.gpt.name,'GPT个人规则');
});
test('Agent更新摘要比较最终内容与已保存基准，计入原人工编辑和实际接受的型号',()=>{
 const h=make();h.configureAgent();h.open('gpt');h.meta('description','待保存的人工说明');h.proto.openTemplateUpdate();h.click('update-check');h.drain();h.click('update-review');h.runConfirm();const v=tpl(h).versions.at(-1);assert.equal(v.origin,'Agent 更新');assert.ok(v.summaryFacts.entries.includes('修改模版说明'));const additions=v.summaryFacts.changes.find(c=>c.path.join('.')==='variants.added');assert.equal(additions.after.length,v.variants.length);assert.ok(v.summaryFacts.entries.includes('新增 '+v.variants.length+' 个型号规则'));assert.equal(v.summaryFacts.basis.revisionId,tpl(h).versions[0].revisionId);
});
test('外部修订生成实际变化摘要，无变化结果不新增版本，也不相信调用方摘要字段',()=>{
 const h=make();h.open('gpt');const task=h.startExternal('current'),target=task.targets[0];h.incoming(task,[h.operation(target,{...target.content,description:'外部实际说明'})]);h.runConfirm();const v=tpl(h).versions.at(-1);assert.equal(v.summary,'修改模版说明');assert.equal(v.summaryFacts.basis.revisionId,tpl(h).versions[0].revisionId);assert.equal(v.origin,'外部 Agent');
 const nextTask=h.startExternal('current'),count=tpl(h).versions.length;h.incoming(nextTask,[h.operation(nextTask.targets[0])]);assert.match(h.dialog(),/内容无变化/);h.runConfirm();assert.equal(tpl(h).versions.length,count);assert.equal(h.state().externalTask.completed,true);
 h.click('external-result');h.nodes.get('mpExternalJSON').value=JSON.stringify({kind:'template-batch-result',taskId:nextTask.taskId,operations:[{...h.operation(nextTask.targets[0]),summary:'调用方随意声明'}]});h.runConfirm();assert.match(h.nodes.get('mpExternalResultError').textContent,/字段无效/);assert.equal(tpl(h).versions.length,count);
});
test('复制首版有复制来源与实际改动事实，删掉普通来源后新摘要仍可解释',()=>{
 const h=make();h.open('gpt');h.event('change','[data-version]',{},'3');h.click('copy-template');const id=h.state().templateScratch.id;h.meta('description','副本说明');h.click('save-template');h.runConfirm();const copy=h.state().templates.find(t=>t.id===id).versions[0];assert.ok(copy.summary.startsWith('复制为独立模版'));assert.ok(copy.summaryFacts.entries.includes('修改模版说明'));assert.equal(copy.summaryFacts.basis.revisionId,tpl(h).versions[2].revisionId);assert.equal(h.state().templates.find(t=>t.id===id).copySummaryBase,undefined);
 h.click('delete-version',{id:'gpt:3'});const current=h.state().templates.find(t=>t.id===id).versions[0];assert.equal(current.summary,copy.summary);assert.deepEqual(current.summaryFacts,copy.summaryFacts);
});
