const test=require('node:test'),assert=require('node:assert/strict'),make=require('./model-prototype-test-harness.cjs');
const clone=x=>JSON.parse(JSON.stringify(x));
function started(range='current'){const h=make();h.open('gpt');h.startExternal(range);return h;}
function sample(h){return clone(h.proto.externalSample());}
function submit(h,data,commit=true){return h.proto.receiveExternalSubmission(data,commit);}
test('一步外部任务入口，完整内容一次复制，无队列/两份复制入口或全库资料',()=>{
 const h=started(),task=h.state().externalTask,pack=h.proto.externalTaskPack(),text=h.nodes.get('mpExternalPrompt').value;
 assert.equal(text,h.proto.externalCombinedText());assert.match(text,/以下是本次目标、字段契约/);assert.ok(text.length<20000);assert.ok(JSON.stringify(pack).length<15000);assert.equal(pack.targets.length,1);assert.equal(pack.context.channels.length,0);
 assert.equal(pack.targets[0].baseRevisionId,task.targets[0].baseRevisionId);assert.equal(Object.hasOwn(pack.targets[0],'nextRevision'),false);assert.match(pack.boundary.versionOwner,/软件分配/);
 assert.doesNotMatch(JSON.stringify(pack),/https:\/\/|keyConfigured|channelAuthConfigured|model-policy-bindings|Fast 总开关|nodes|modelTemplateAgent/);
 h.context.closeModal();h.click('external-agent');assert.ok(h.nodes.has('mpExternalScene'));assert.doesNotMatch(h.dialog(),/本次会话任务|data-mp="external-open"|复制任务资料|新建任务/);assert.equal(h.state().externalTask.taskId,task.taskId);
});
test('随附结果示例使用真实基准且可校验，回传资料的修改不影响任务契约',()=>{
 for(const range of ['current','create']){const h=started(range),pack=h.proto.externalTaskPack(),before=JSON.stringify(h.state().templates),example=clone(pack.submission);const result=submit(h,example,false);assert.equal(result.ok,true);assert.equal(JSON.stringify(h.state().templates),before);pack.contract.fields.content.rules.protocol.enum.push('fake');assert.equal(h.proto.externalTaskPack().contract.fields.content.rules.protocol.enum.includes('fake'),false);}
});
test('现代提交无需提供新版本号，保存回执与真实新增记录相符，默认/采用不变',()=>{
 const h=started(),before=h.state(),data=sample(h),result=submit(h,data),after=h.state(),t=after.templates.find(t=>t.id==='gpt'),v=t.versions.at(-1);
 assert.equal(result.ok,true);assert.equal(result.state,'saved');assert.equal(t.versions.length,before.templates[0].versions.length+1);assert.equal(v.n,4);assert.equal(t.main,1);assert.equal(JSON.stringify(after.saved),JSON.stringify(before.saved));
 assert.equal(result.receipt.versions[0].revisionId,v.revisionId);assert.equal(result.receipt.versions[0].versionLabel,v.versionLabel);assert.equal(v.basedOn,before.templates[0].versions[0].revisionId);assert.equal(v.origin,'外部 Agent');
 assert.deepEqual(JSON.parse(JSON.stringify(v.rules)),data.operations[0].content.rules);assert.equal(v.summary,'修改模版说明');
 h.proto.openExternalTask(before.externalTask.taskId);assert.match(h.nodes.get('mpExternalReceiptHost').innerHTML,/提交回执|已保存新版本/);
});
test('同一提交重试返回原回执，不重复版本；改内容复用提交ID和完成后新提交拒绝',()=>{
 const h=started(),data=sample(h),first=submit(h,data),count=h.state().templates[0].versions.length,retry=submit(h,data);
 assert.equal(retry.replayed,true);assert.equal(JSON.stringify(retry.receipt),JSON.stringify(first.receipt));assert.equal(h.state().templates[0].versions.length,count);
 const changed=clone(data);changed.operations[0].content.description='另一结果';assert.equal(submit(h,changed).ok,false);changed.submissionId+='-new';assert.equal(submit(h,changed).ok,false);assert.equal(h.state().templates[0].versions.length,count);
});
test('预检不写库，允许原内容同ID提交；预检后修改提交内容拒绝，取消后晚回无副作用',()=>{
 const h=started(),data=sample(h),before=JSON.stringify(h.state().templates);assert.equal(submit(h,data,false).state,'ready');assert.equal(JSON.stringify(h.state().templates),before);
 const different=clone(data);different.operations[0].content.description='同ID不同内容';assert.match(submit(h,different).message,/提交ID/);assert.equal(JSON.stringify(h.state().templates),before);
 h.click('external-cancel');assert.equal(h.state().externalTask.cancelled,true);assert.equal(submit(h,data).state,'cancelled');assert.equal(JSON.stringify(h.state().templates),before);
});
test('基准变化/越界/错误契约/伪造版本号和混合坏项均整批拒绝',()=>{
 for(const mutate of [d=>d.contractVersion='wrong',d=>d.operations[0].revision=99,d=>d.operations.push({...clone(d.operations[0]),id:'claude'}),d=>d.operations[0].content.rules.fake=true]){
  const h=started(),d=sample(h),before=JSON.stringify(h.state().templates);mutate(d);const result=submit(h,d);assert.equal(result.ok,false);assert.equal(JSON.stringify(h.state().templates),before);
 }
 const h=started(),d=sample(h);h.meta('description','期间人工修改');assert.equal(submit(h,d).state,'conflict');assert.equal(h.state().buffers.gpt.description,'期间人工修改');assert.equal(h.state().templates[0].versions.length,3);
});
test('新建限定范围并保护内置标记；修订任务不默认允许新增模版',()=>{
 const h=started(),d=sample(h);d.operations=[{action:'create',id:'private-new',baseDigest:'',content:clone(d.operations[0].content)}];assert.match(submit(h,d).message,/范围/);
 const fresh=started('create'),created=sample(fresh);assert.equal(created.operations[0].action,'create');assert.equal(Object.hasOwn(created.operations[0],'revision'),false);created.operations[0].content.systemBaseline=true;assert.equal(submit(fresh,created).ok,false);delete created.operations[0].content.systemBaseline;created.submissionId+='-fixed';const result=submit(fresh,created);assert.equal(result.ok,true);const t=fresh.state().templates.find(t=>t.id===created.operations[0].id);assert.equal(t.main,0);assert.equal(t.originType,'user');assert.equal(t.versions[0].systemBaseline,undefined);
});
test('多个任务结果不串目标，已完成任务可重开；无变化有核对回执但不增版本',()=>{
 const h=started(),first=sample(h);h.open('claude');h.startExternal('current');const current=h.state().externalTask.taskId;const result=submit(h,first);assert.equal(result.ok,true);assert.equal(h.state().externalTask.taskId,current);assert.equal(h.state().templateDetail,true);
 assert.equal(h.proto.openExternalTask(first.taskId),true);assert.equal(h.state().externalTask.state,'saved');
 h.startExternal('current');const d=sample(h);d.operations[0].content=clone(h.state().externalTask.targets[0].content);const before=JSON.stringify(h.state().templates),unchanged=submit(h,d);assert.equal(unchanged.receipt.status,'unchanged');assert.equal(unchanged.receipt.versions.length,0);assert.equal(JSON.stringify(h.state().templates),before);
});
test('人工与外部路径使用相同字段校验/摘要/时间版本，保存内容一致且均不自动采用',()=>{
 const manual=make();manual.open('gpt');manual.meta('description','相同说明');manual.click('save-template');manual.runConfirm();const a=manual.state().templates[0].versions.at(-1);
 const outside=started(),d=sample(outside);d.operations[0].content.description='相同说明';assert.equal(submit(outside,d).ok,true);const b=outside.state().templates[0].versions.at(-1);
 for(const k of ['description','match','rules','summary'])assert.equal(JSON.stringify(a[k]),JSON.stringify(b[k]));assert.equal(manual.state().templates[0].main,outside.state().templates[0].main);assert.equal(manual.state().saved.globalRefs.gpt,outside.state().saved.globalRefs.gpt);
});

test('场景切换生成预置指令；只检查在服务层拒绝写入，报告不改库',()=>{
 const h=started(),before=JSON.stringify(h.state().templates),original=sample(h);
 h.nodes.get('mpExternalScene').value='audit';h.proto.updateExternalPreview();h.runConfirm();const task=h.state().externalTask;assert.equal(task.scope.readOnly,true);assert.match(h.nodes.get('mpExternalPrompt').value,/只检查|仅检查/);assert.equal(task.scope.allowCreate,false);assert.equal(task.scope.allowRevision,false);
 const bad={...original,taskId:task.taskId,submissionId:task.taskId+'-bad'};assert.equal(submit(h,bad).ok,false);assert.equal(JSON.stringify(h.state().templates),before);
 const report=sample(h);report.findings=[{templateId:'gpt',severity:'warning',message:'资料需要进一步核对',sources:['用户资料，未实测']}];const result=submit(h,report);assert.equal(result.state,'reviewed');assert.equal(result.receipt.status,'reviewed');assert.equal(result.receipt.versions.length,0);assert.equal(JSON.stringify(h.state().templates),before);assert.equal(submit(h,report).replayed,true);
});
test('未复制的准备不注册提交，范围/场景冲突禁用复制，不能丢失已复制的旧基准',()=>{
 const h=make();h.open('gpt');h.click('external-new');assert.equal(h.state().externalTasks.length,0);assert.equal(submit(h,sample(h)).ok,false);
 const originalDialog=h.nodes.get('modalBody');h.nodes.get('mpExternalScope').value='create';h.nodes.get('mpExternalScene').value='update';assert.equal(h.proto.updateExternalPreview(),false);assert.equal(h.nodes.get('modalConfirm').disabled,true);assert.match(h.nodes.get('mpExternalError').textContent,/需要已有模版/);
 h.nodes.get('mpExternalScene').value='private';assert.equal(h.proto.updateExternalPreview(),true);assert.equal(h.nodes.get('modalBody'),originalDialog);assert.match(h.nodes.get('mpExternalPrompt').value,/私有/);h.runConfirm();assert.equal(h.state().externalTasks.length,1);const id=h.state().externalTask.taskId;
 h.nodes.get('mpExternalRequest').value='增加用户补充要求';h.proto.updateExternalPreview();assert.notEqual(h.state().externalTask.taskId,id);assert.equal(h.state().externalTasks.length,1);assert.match(h.nodes.get('mpExternalPrompt').value,/增加用户补充要求/);
});

test('仅复制的外部任务不长期锁定历史版本，删除基准后旧提交拒绝',()=>{
 const h=make();h.open('gpt');h.click('copy-template');h.click('save-template');h.runConfirm();const id=h.state().templateScratch?.id||h.state().templates.at(-1).id;h.startExternal('current');const data=sample(h);assert.equal(h.proto.versionProtectionReasons(id,1).some(r=>r.includes('外部')),false);
 h.click('history-delete-version',{id,n:'1'});h.runConfirm();assert.equal(h.state().templates.some(t=>t.id===id),false);assert.equal(submit(h,data).state,'conflict');
});
