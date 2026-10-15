/* 两类列表显式入口与统一编辑表单的边界；不替代浏览器布局。 */
const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
const bodyEditor=h=>{assert.match(h.page(),/id="mpTemplateWorkspace"/);assert.match(h.page(),/id="mpTemplateName"/);assert.doesNotMatch(h.page(),/mp-rule-summary|mp-editor-title-button|编辑为新候选|来源与使用情况/);};
test('渠道列表名称与行空白不可进入，行内编辑按钮唯一进入原渠道编辑页',()=>{
 const h=make(),before=JSON.stringify(h.state().saved),rows=h.state().draft.providers;
 assert.equal((h.page().match(/<div class="mp-provider-name">/g)||[]).length,rows.length);assert.doesNotMatch(h.page(),/<button[^>]*class="mp-provider-name"|data-mp="view-provider"/);
 assert.equal((h.page().match(/data-mp="edit-provider"/g)||[]).length,rows.length);
 h.clickTarget({dataset:{provider:'newapi'},closest:()=>null});assert.equal(h.state().channelDetail,false);
 h.event('change','[data-scope]',{scope:'newapi'},'',true);assert.equal(h.state().channelDetail,false);h.click('toggle-provider',{id:'newapi'});assert.equal(h.state().channelDetail,false);
 h.click('edit-provider',{id:'newapi'});assert.equal(h.state().channelDetail,true);assert.match(h.page(),/mpChannelWorkspace/);assert.equal(JSON.stringify(h.state().saved),before);
});
test('模版列表名称为文本，勾选不导航；唯一行内编辑直接加载完整表单，不入库',()=>{
 const h=make();h.click('tab',{tab:'templates'});const before=JSON.stringify(h.state().templates);
 assert.equal((h.page().match(/<div class="mp-template-name">/g)||[]).length,17);assert.doesNotMatch(h.page(),/<button[^>]*class="mp-template-name"|data-mp="view-template"/);
 assert.equal((h.page().match(/data-mp="open-template-editor"/g)||[]).length,17);h.clickTarget({dataset:{id:'gpt'},closest:()=>null});assert.equal(h.state().templateDetail,false);
 h.event('change','[data-template-select]',{templateSelect:'gpt'},'',true);assert.equal(h.state().templateDetail,false);
 h.click('open-template-editor',{id:'gpt'});bodyEditor(h);assert.equal(h.state().buffers.gpt.base,1);assert.equal(h.proto.templateHeadingState().id,'clean');assert.equal(h.proto.templateHeadingState().usage.id,'pending');assert.equal(JSON.stringify(h.state().templates),before);
});
test('保存后仍留同一编辑页并展示来源与采用状态，再次编辑保存不改变旧版',()=>{
 const h=make();h.open('gpt');const old=JSON.stringify(h.state().templates[0].versions);h.meta('description','保存后继续编辑');h.click('save-template');h.runConfirm();bodyEditor(h);
 assert.equal(h.state().buffers.gpt.base,4);assert.equal(h.proto.templateHeadingState().id,'clean');assert.equal(h.proto.templateHeadingState().usage.id,'unadopted');assert.match(h.page(),/data-template-usage="unadopted"/);assert.match(h.page(),/id="mpTemplateSection-sources"/);
 h.meta('description','第二次修改');h.click('save-template');h.runConfirm();bodyEditor(h);assert.equal(h.state().buffers.gpt.base,5);assert.equal(h.state().templates[0].versions[3].description,'保存后继续编辑');assert.equal(JSON.stringify(h.state().templates[0].versions.slice(0,3)),old);assert.equal(h.state().saved.globalRefs.gpt,'gpt:1');
});
test('未修改时可在同一编辑器浏览旧版本，改旧版只另存新版本',()=>{
 const h=make();h.open('gpt');assert.equal(h.nodes.get('mpTemplateVersion').disabled,false);const old=JSON.stringify(h.state().templates[0].versions[1]);h.event('change','[data-version]',{},'2');bodyEditor(h);assert.equal(h.state().buffers.gpt.base,2);assert.equal(h.state().buffers.gpt.rules.default,'high');assert.equal(h.nodes.get('mpTemplateVersion').disabled,false);
 h.meta('description','基于历史版的修订');h.click('save-template');h.runConfirm();const next=h.state().templates[0].versions.at(-1);assert.equal(next.basedOn,h.state().templates[0].versions[1].revisionId);assert.equal(JSON.stringify(h.state().templates[0].versions[1]),old);assert.equal(h.state().buffers.gpt.base,next.n);bodyEditor(h);
});
test('有修改时禁用版本切换和采用，旧事件也无法绕过；保存后恢复可用',()=>{
 const h=make();h.open('gpt');h.meta('description','仍未保存');assert.equal(h.nodes.get('mpTemplateVersion').disabled,true);assert.equal(h.nodes.get('mpTemplateAdopt').disabled,true);
 const draft=JSON.stringify(h.state().draft);h.event('change','[data-version]',{},'2');assert.equal(h.state().buffers.gpt.base,1);assert.equal(h.state().buffers.gpt.description,'仍未保存');h.click('adopt-version');assert.notEqual(h.nodes.get('modalMask').style.display,'flex');assert.equal(JSON.stringify(h.state().draft),draft);
 h.click('save-template');h.runConfirm();assert.equal(h.nodes.get('mpTemplateVersion').disabled,false);assert.equal(h.nodes.get('mpTemplateAdopt').disabled,false);h.click('adopt-version');h.nodes.get('mpAdoptScope').value='global';h.runConfirm();assert.equal(h.state().draft.globalRefs.gpt,'gpt:4');assert.equal(h.state().saved.globalRefs.gpt,'gpt:1');
});
test('清洁编辑仍允许版本管理和用户确认，未使用版本可删除后加载另一个编辑基准',()=>{
 const h=make();let menu;h.context.window.prototypeActionMenu={close(){},open(input){menu=input;}};h.open('gpt');h.click('version-menu');assert.equal(menu.items.find(i=>i.label==='确认这套配置可用').disabled,false);
 h.click('confirm-template-use');h.runConfirm();bodyEditor(h);assert.equal(h.proto.templateHeadingState().usage.id,'confirmed');
 h.event('change','[data-version]',{},'3');h.click('version-menu');menu.items.find(i=>i.label==='版本管理').onSelect();assert.equal(h.nodes.get('mpVersionDefault-gpt-3').disabled,false);assert.equal(h.nodes.get('mpVersionDelete-gpt-3').disabled,false);h.click('history-delete-version',{id:'gpt',n:'3'});h.runConfirm();bodyEditor(h);assert.equal(h.state().templates[0].versions.length,2);assert.equal(h.state().buffers.gpt.base,1);
});
test('Agent 更新与外部候选保存都回到统一编辑器，无变化返回不创建额外版本',()=>{
 const h=make();h.configureAgent();h.open('gpt');h.proto.openTemplateUpdate();h.click('update-check');h.drain();h.click('update-review');h.runConfirm();bodyEditor(h);assert.equal(h.state().buffers.gpt.base,4);
 const task=h.startExternal('current');h.incoming(task,[h.operation(task.targets[0],{...task.targets[0].content,description:'外部候选内容'})]);h.runConfirm();bodyEditor(h);
 const versions=JSON.stringify(h.state().templates);h.click('back-templates');assert.equal(h.state().templateDetail,false);assert.equal(h.state().buffers.gpt,undefined);assert.equal(JSON.stringify(h.state().templates),versions);assert.doesNotMatch(h.page(),/data-mp="view-template"|>查看<\/button>/);
});
test('官方原生来源使用同一渠道页面，仅按实际能力禁用编辑，不增加查看页面',()=>{
 const h=make();h.click('edit-provider',{id:'native'});assert.match(h.page(),/mpChannelWorkspace/);assert.match(h.page(),/官方管理/);assert.doesNotMatch(h.page(),/data-mp="order-preview"/);const before=JSON.stringify(h.state().draft);
 assert.match(h.proto.commitModelPatches('native',[{key:'native::0',rules:{context:64000}}]),/不可编辑/);assert.equal(JSON.stringify(h.state().draft),before);h.click('back-channels');assert.match(h.page(),/data-mp="edit-provider"[^>]*aria-label="编辑 官方 \/ 原生/);
});
