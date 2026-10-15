/* 保存正文不覆盖、内置基线和依赖保护、短下拉与明确清理；原型内存DOM不代表生产持久化。 */
const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
const tpl=(h,id='gpt')=>h.state().templates.find(t=>t.id===id);
const policies=h=>JSON.stringify({saved:h.state().saved,draft:h.state().draft,records:h.state().records});
const listSelect=(h,id='gpt')=>h.page().match(new RegExp('<select id="mpTemplateListVersion-'+id+'"[\\s\\S]*?<\\/select>'))?.[0]||'';
const optionNumbers=html=>[...html.matchAll(/<option value="(\d+)"/g)].map(m=>Number(m[1]));
function menus(h){let menu;h.context.window.prototypeActionMenu={close(){},open(input){menu=input;}};return ()=>menu;}
function copied(h){h.open('gpt');h.click('copy-template');const id=h.state().templateScratch.id;h.click('save-template');h.runConfirm();return id;}
function saveRevision(h,text){h.meta('description',text);h.click('save-template');h.runConfirm();}
test('来源明确区分内置和自建，系统初始基线无论换默认仍始终保护',()=>{
 const h=make(),menu=menus(h);h.click('tab',{tab:'templates'});assert.match(h.page(),/data-template-origin="builtin"/);assert.doesNotMatch(h.page(),/>初始配置</);assert.equal(tpl(h).versions[0].systemBaseline,true);assert.equal(tpl(h).versions[0].origin,'软件内置');const baseline=JSON.stringify(tpl(h).versions[0]);
 h.open('qwen');saveRevision(h,'新的人工Qwen配置');assert.equal(tpl(h,'qwen').originType,'builtin');assert.equal(tpl(h,'qwen').versions[1].systemBaseline,undefined);h.click('set-main',{id:'qwen:2'});h.runConfirm();h.event('change','[data-version]',{},'1');h.click('template-version-history',{id:'qwen'});assert.equal(h.nodes.get('mpVersionDelete-qwen-1').disabled,true);assert.match(h.nodes.get('mpVersionDelete-qwen-1').getAttribute('title'),/系统初始/);h.click('delete-version',{id:'qwen:1'});assert.equal(tpl(h,'qwen').versions.length,2);assert.ok(h.proto.versionProtectionReasons('qwen',1).includes('系统初始内置基线'));assert.equal(JSON.stringify(tpl(h).versions[0]),baseline);
});
test('修改默认内置版本只产生用户新版本，原正文/默认/采用不变；复制不继承基线保护',()=>{
 const h=make();h.open('gpt');const old=JSON.stringify(tpl(h).versions[0]),before=policies(h);saveRevision(h,'人工新版本');assert.equal(JSON.stringify(tpl(h).versions[0]),old);assert.equal(tpl(h).main,1);assert.equal(tpl(h).versions.at(-1).systemBaseline,undefined);assert.equal(policies(h),before);h.click('copy-template');const id=h.state().templateScratch.id;assert.equal(h.state().templateScratch.originType,'user');h.click('save-template');h.runConfirm();assert.equal(tpl(h,id).versions[0].systemBaseline,undefined);assert.equal(tpl(h,id).versions[0].systemCurrent,undefined);assert.equal(tpl(h,id).versions[0].origin,'复制新建');assert.match(h.page(),/data-template-origin="user"/);
});
test('同模型族可以独立共存，保存复制不覆盖内置或采用；重叠采用仍阻止运行保存',()=>{
 const h=make(),id=copied(h);const source=JSON.stringify(tpl(h).versions[0]);assert.deepEqual(tpl(h,id).versions[0].match,tpl(h).versions[0].match);const initial=h.state().saved.globalRefs.gpt;assert.equal(h.state().saved.globalRefs[id],undefined);h.click('adopt-version');h.nodes.get('mpAdoptScope').value='global';h.runConfirm();assert.equal(h.state().draft.globalRefs.gpt,initial);assert.equal(h.state().draft.globalRefs[id],id+':1');assert.ok(h.proto.routePlan().errors.some(e=>e.includes('模版匹配冲突')));h.click('save');assert.equal(h.state().operation,null);assert.equal(h.state().saved.globalRefs[id],undefined);assert.equal(JSON.stringify(tpl(h).versions[0]),source);
});
test('内置与自建模版全部展示，菜单无隐藏和恢复，内置不可整条删除',()=>{
 const h=make(),menu=menus(h);h.click('tab',{tab:'templates'});const before=policies(h);
 assert.match(h.page(),/data-template-origin="builtin"/);assert.doesNotMatch(h.page(),/data-mp="toggle-hidden-templates"|显示隐藏的内置模版/);
 h.click('template-version-history',{id:'gpt'});assert.doesNotMatch(h.dialog(),/隐藏内置模版|恢复显示内置模版|id="mpDeleteUserTemplate"/);h.context.closeModal();
 h.click('toggle-builtin-template',{id:'gpt'});h.click('toggle-hidden-templates');assert.match(h.page(),/data-mp="open-template-editor" data-id="gpt"/);assert.equal(tpl(h).hidden,undefined);
 assert.equal(h.proto.resolveRoute('newapi','newapi::0').error,'');h.click('delete-user-template',{id:'gpt'});assert.ok(tpl(h));assert.equal(policies(h),before);
 h.event('input','[data-template-search]',{},tpl(h).name);assert.match(h.page(),/data-mp="open-template-editor" data-id="gpt"/);h.event('input','[data-template-search]',{},'');
 const id=copied(h);h.click('back-templates');assert.match(h.page(),new RegExp('data-mp="open-template-editor" data-id="'+id+'"'));assert.match(h.page(),/data-mp="open-template-editor" data-id="gpt"/);assert.match(h.page(),/data-template-origin="user"/);
});
test('下拉最近5加当前/默认，管理查看版本打开编辑但不改默认/策略',()=>{
 const h=make();h.proto.scene('template-history');const before=policies(h);assert.deepEqual(optionNumbers(listSelect(h)),[18,17,16,15,14,1]);h.event('change','[data-template-row-version]',{templateRowVersion:'gpt'},'history');assert.match(h.dialog(),/aria-label="全部保存版本"/);assert.match(h.dialog(),/系统初始内置基线/);h.click('history-select-version',{id:'gpt',n:'2'});assert.equal(h.state().templateDetail,true);assert.equal(h.state().buffers.gpt.base,2);assert.equal(tpl(h).main,1);assert.equal(policies(h),before);h.click('back-templates');h.event('change','[data-template-row-version]',{templateRowVersion:'gpt'},'2');assert.deepEqual(optionNumbers(listSelect(h)),[18,17,16,15,14,2,1]);
});

test('很多版本仍不自动删除，保存和Agent更新保留历史且不展示独立清理',()=>{
 const h=make();h.proto.scene('template-history');assert.equal(tpl(h).versions.length,18);assert.doesNotMatch(h.page(),/有旧版本可清理/);h.click('template-version-history',{id:'gpt'});assert.doesNotMatch(h.dialog(),/data-mp="cleanup-template-versions"|data-mp="retain-version"|data-mp="history-archive-version"/);h.context.closeModal();h.click('open-template-editor',{id:'gpt'});saveRevision(h,'很多版本仍可保存');assert.equal(tpl(h).versions.length,19);h.configureAgent();h.proto.openTemplateUpdate();h.click('update-check');h.drain();h.click('update-review');h.runConfirm();assert.equal(tpl(h).versions.length,20);assert.equal(tpl(h).versions[0].systemBaseline,true);
});

test('批量删除只处理明确勾选版本，取消保持，确认整个名单后返回管理',()=>{
 const h=make();h.proto.scene('template-history');const before=policies(h),original=JSON.stringify(tpl(h));h.click('template-version-history',{id:'gpt'});
 for(const n of [2,3,4])h.event('change','[data-version-pick]',{versionPick:String(n)},'',true);
 assert.deepEqual(h.state().templateHistoryView.picked,[2,3,4]);h.click('history-delete-selected',{id:'gpt'});assert.match(h.dialog(),/删除 3 个版本/);h.context.closeModal();assert.equal(JSON.stringify(tpl(h)),original);h.click('template-version-history',{id:'gpt'});h.click('history-delete-selected',{id:'gpt'});h.runConfirm();assert.equal(tpl(h).versions.length,15);assert.ok([2,3,4].every(n=>!tpl(h).versions.some(v=>v.n===n)));assert.ok(tpl(h).versions.some(v=>v.systemBaseline));assert.equal(policies(h),before);assert.deepEqual(h.state().templateHistoryView.picked,[]);
});

test('旧保留/归档/清理事件失效，自动保护不靠手动标记',()=>{
 const h=make();h.proto.scene('template-history');const before=JSON.stringify(tpl(h));h.click('retain-version',{id:'gpt',n:'2'});h.click('history-archive-version',{id:'gpt',n:'2'});h.click('cleanup-template-versions',{id:'gpt'});assert.equal(JSON.stringify(tpl(h)),before);assert.notEqual(h.nodes.get('modalMask').style.display,'flex');assert.equal(h.proto.versionProtectionReasons('gpt',2).length,0);assert.ok(h.proto.versionProtectionReasons('gpt',1).includes('系统初始内置基线'));h.click('template-version-history',{id:'gpt'});h.event('change','[data-version-pick]',{versionPick:'1'},'',true);assert.deepEqual(h.state().templateHistoryView.picked,[]);
});

test('全选只勾可删除版本，已采用/草稿/恢复/默认版本自动跳过',()=>{
 const h=make();h.proto.scene('template-history');h.event('change','[data-template-row-version]',{templateRowVersion:'gpt'},'2');h.click('open-template-editor',{id:'gpt'});h.click('adopt-version');h.nodes.get('mpAdoptScope').value='global';h.runConfirm();h.click('save');h.runConfirm();h.drain();assert.ok(h.proto.versionProtectionReasons('gpt',2).includes('已保存配置引用'));h.click('template-version-history',{id:'gpt'});h.event('change','[data-version-pick-all]',{},'',true);assert.equal(h.state().templateHistoryView.picked.includes(1),false);assert.equal(h.state().templateHistoryView.picked.includes(2),false);h.click('history-delete-selected',{id:'gpt'});h.runConfirm();assert.deepEqual(tpl(h).versions.map(v=>v.n),[1,2]);assert.equal(h.state().saved.globalRefs.gpt,'gpt:2');
});

test('自建最后一批无引用版本可删除，默认和外部引用仍阻止',()=>{
 const h=make(),id=copied(h);h.click('back-templates');h.click('template-version-history',{id});h.event('change','[data-version-pick]',{versionPick:'1'},'',true);h.click('history-delete-selected',{id});assert.match(h.dialog(),/自建模版将从列表移除/);h.runConfirm();assert.equal(tpl(h,id),undefined);
 const other=copied(h);h.click('set-main',{id:other+':1'});h.runConfirm();assert.ok(h.proto.versionProtectionReasons(other,1).includes('默认版本'));h.click('history-delete-version',{id:other,n:'1'});assert.ok(tpl(h,other));
});

test('删除后不复用内部版本序号，同秒完整版本身份仍独立',()=>{
 const h=make();h.open('gpt');saveRevision(h,'第四版');const old=tpl(h).versions.at(-1);h.click('delete-version',{id:'gpt:4'});assert.equal(tpl(h).versions.length,3);saveRevision(h,'下一保存');const next=tpl(h).versions.at(-1);assert.equal(next.n,5);assert.notEqual(next.revisionId,old.revisionId);assert.notEqual(next.versionLabel,old.versionLabel);
});
test('搜索不丢搜索外勾选；旧删除确认遇到依赖变化整批拒绝',()=>{
 const h=make();h.proto.scene('template-history');const before=policies(h);h.click('template-version-history',{id:'gpt'});h.event('change','[data-version-pick]',{versionPick:'2'},'',true);const label=tpl(h).versions.find(v=>v.n===17).shortId;h.event('input','[data-history-search]',{},label);assert.doesNotMatch(h.dialog(),/data-n="2"/);assert.deepEqual(h.state().templateHistoryView.picked,[2]);h.event('change','[data-version-pick]',{versionPick:'17'},'',true);assert.equal(policies(h),before);h.click('history-delete-selected',{id:'gpt'});h.click('toggle-provider',{id:'aimami'});h.runConfirm();assert.match(h.nodes.get('mpVersionDeleteError').textContent,/已变化/);assert.equal(tpl(h).versions.length,18);
});

test('外部新建始终是自建来源，不能借导入字段伪造内置基线保护',()=>{
 const h=make(),task={kind:'template-task-batch',taskId:'retention-import',targets:[]},content={name:'用户GPT定制',description:'独立规则',match:{include:[{type:'prefix',value:'gpt-'}],exclude:[],caseSensitive:false,channelMode:'all',channels:[],priority:0},rules:{}};
 h.click('external-result');h.nodes.get('mpExternalTaskJSON').value=JSON.stringify(task);h.nodes.get('mpExternalJSON').value=JSON.stringify({kind:'template-batch-result',taskId:task.taskId,operations:[{action:'create',id:'user-gpt',revision:1,baseDigest:'',content:{...content,systemBaseline:true}}]});h.runConfirm();assert.match(h.nodes.get('mpExternalResultError').textContent,/字段无效/);assert.equal(tpl(h,'user-gpt'),undefined);
 h.nodes.get('mpExternalJSON').value=JSON.stringify({kind:'template-batch-result',taskId:task.taskId,operations:[{action:'create',id:'user-gpt',revision:1,baseDigest:'',content}]});h.runConfirm();h.runConfirm();assert.equal(tpl(h,'user-gpt').originType,'user');assert.equal(tpl(h,'user-gpt').versions[0].systemBaseline,undefined);assert.equal(tpl(h,'user-gpt').nextRevision,2);assert.equal(h.state().externalTask.completed,true);
});
