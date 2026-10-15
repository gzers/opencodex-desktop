/* 列表操作作用于所选保存版本；最小 DOM 不替代浏览器布局。 */
const {test}=require('node:test'),assert=require('node:assert/strict');
const make=require('./model-prototype-test-harness.cjs');
const rowVersion=(h,id,n)=>h.event('change','[data-template-row-version]',{templateRowVersion:id},String(n));
const library=h=>JSON.stringify(h.state().templates),policy=h=>JSON.stringify({draft:h.state().draft,saved:h.state().saved});
const selectTag=(h,id)=>h.page().match(new RegExp('<select id="mpTemplateListVersion-'+id+'"[\\s\\S]*?<\\/select>'))?.[0];
const control=(h,kind,id,n)=>{const tag=h.dialog().match(new RegExp('<button[^>]* id="mpVersion'+kind+'-'+id+'-'+n+'"[^>]*>'))?.[0];assert.ok(tag);return {disabled:/\bdisabled\b/.test(tag),tag};};
const manage=(h,id='gpt')=>h.click('template-version-history',{id});
const act=(h,action,id,n)=>h.click(action,{id,n:String(n)});
test('单一版本列与直接管理/匹配入口；选择不导航、不改变库/默认/已采用策略',()=>{
 const h=make();h.click('tab',{tab:'templates'});const before=library(h),saved=policy(h);
 assert.doesNotMatch(h.page(),/版本与使用情况|data-mp="template-list-menu"/);assert.match(h.page(),/data-mp="template-version-history"/);assert.match(h.page(),/data-mp="template-inspect"/);assert.match(selectTag(h,'gpt'),/value="1" selected[^>]*>[^<]* · 内置基线<\/option>/);
 rowVersion(h,'gpt',2);assert.equal(h.state().templateListVersions.gpt,2);assert.equal(h.state().templateDetail,false);assert.equal(library(h),before);assert.equal(policy(h),saved);h.click('template-list-menu',{id:'gpt'});assert.notEqual(h.nodes.get('modalMask').style.display,'flex');h.click('open-template-editor',{id:'gpt'});assert.equal(h.state().buffers.gpt.base,2);
});
test('列表复制使用所选保存版本，源编辑缓冲不串入副本且原库不变',()=>{
 const h=make();h.click('tab',{tab:'templates'});rowVersion(h,'gpt',2);const before=library(h),source=h.state().templates[0].versions[1];h.click('copy-list-template',{id:'gpt'});const id=h.state().templateScratch.id;assert.equal(h.state().buffers[id].rules.default,source.rules.default);assert.equal(h.state().templateScratch.copiedFrom.revisionId,source.revisionId);assert.equal(library(h),before);
});
test('匹配与使用严格按所选版本，区分已保存引用与未保存草稿，匹配不等于采用',()=>{
 const h=make();h.click('tab',{tab:'templates'});h.click('template-inspect',{id:'gpt'});assert.match(h.dialog(),/实际使用/);assert.match(h.dialog(),/规则匹配/);assert.match(h.dialog(),/全局族默认/);assert.match(h.dialog(),/NewApi/);assert.doesNotMatch(h.dialog(),/当前草稿（尚未保存）/);h.context.closeModal();rowVersion(h,'gpt',2);h.click('template-inspect',{id:'gpt'});assert.match(h.dialog(),/没有配置引用/);assert.doesNotMatch(h.dialog(),/全局族默认/);h.context.closeModal();
 h.click('open-template-editor',{id:'gpt'});h.click('adopt-version');h.nodes.get('mpAdoptScope').value='global';h.runConfirm();h.click('back-templates');rowVersion(h,'gpt',2);h.click('template-inspect',{id:'gpt'});assert.match(h.dialog(),/当前草稿（尚未保存）/);assert.match(h.dialog(),/全局族默认/);
});
test('合并预览只读，不修改库/策略且不跳详情',()=>{
 const h=make();h.click('tab',{tab:'templates'});rowVersion(h,'gpt',2);const before=library(h),saved=policy(h),label=h.state().templates[0].versions[1].versionLabel;h.click('template-inspect',{id:'gpt'});assert.match(h.dialog(),/mp-match-results/);assert.ok(h.dialog().includes(label));assert.match(h.dialog(),/命中/);assert.equal(h.state().templateDetail,false);assert.equal(library(h),before);assert.equal(policy(h),saved);
});
test('版本管理只提供三类动作，查看/编辑与设默认分别改变编辑对象和默认指针',()=>{
 const h=make();h.click('tab',{tab:'templates'});const saved=policy(h);manage(h);assert.doesNotMatch(h.dialog(),/data-mp="retain-version"|data-mp="history-archive-version"|data-mp="cleanup-template-versions"/);assert.equal(control(h,'Default','gpt',2).disabled,false);
 act(h,'history-select-version','gpt',2);assert.equal(h.state().templateDetail,true);assert.equal(h.state().buffers.gpt.base,2);assert.equal(h.state().templates[0].main,1);assert.equal(policy(h),saved);
 manage(h);act(h,'history-set-main','gpt',2);assert.equal(h.state().templates[0].main,1);h.runConfirm();assert.equal(h.state().templates[0].main,2);assert.equal(policy(h),saved);assert.equal(control(h,'Default','gpt',2).disabled,true);assert.equal(control(h,'Delete','gpt',2).disabled,true);
});
test('管理删除只移除未引用单版，返回管理；旧确认遇到新编辑拒绝',()=>{
 const h=make();h.click('tab',{tab:'templates'});rowVersion(h,'gpt',3);manage(h);assert.equal(control(h,'Delete','gpt',3).disabled,false);act(h,'history-delete-version','gpt',3);h.context.closeModal();manage(h);act(h,'history-delete-version','gpt',3);h.runConfirm();assert.deepEqual(h.state().templates[0].versions.map(v=>v.n),[1,2]);assert.equal(h.state().templateDetail,false);assert.equal(h.state().templateListVersions.gpt,1);assert.equal(control(h,'Delete','gpt',1).disabled,true);
 act(h,'history-delete-version','gpt',2);h.click('open-template-editor',{id:'gpt'});h.meta('description','确认期间的新编辑');h.runConfirm();assert.match(h.nodes.get('mpListVersionDeleteError').textContent,/已变化/);assert.equal(h.state().templates[0].versions.length,2);
});
test('已有未保存编辑时锁定版本，管理不丢草稿且变更动作禁用',()=>{
 const h=make();h.open('gpt');h.meta('description','保留草稿');const before=JSON.stringify(h.state().buffers.gpt);manage(h);assert.equal(control(h,'Default','gpt',2).disabled,true);assert.equal(control(h,'Open','gpt',2).disabled,true);assert.equal(control(h,'Delete','gpt',3).disabled,true);assert.equal(JSON.stringify(h.state().buffers.gpt),before);h.context.closeModal();rowVersion(h,'gpt',2);assert.equal(JSON.stringify(h.state().buffers.gpt),before);
});
test('所选版本匹配规则与摘要一致，零命中新版不误用默认规则',()=>{
 const h=make();h.open('gpt');h.match('include','prefix:never-matches-');h.meta('description','独立新版');h.click('save-template');h.runConfirm();h.click('back-templates');rowVersion(h,'gpt',4);h.click('template-inspect',{id:'gpt'});assert.match(h.dialog(),/命中 0 /);assert.match(h.dialog(),/零命中/);h.context.closeModal();rowVersion(h,'gpt',1);h.click('template-inspect',{id:'gpt'});assert.match(h.dialog(),/命中 [1-9]\d* /);assert.equal(h.state().templates[0].main,1);
});
test('自建最后一版可明确删除，删除后模版退出且不影响其他模版',()=>{
 const h=make();h.open('gpt');h.click('copy-template');const id=h.state().templateScratch.id;h.click('save-template');h.runConfirm();h.click('back-templates');manage(h,id);assert.equal(control(h,'Delete',id,1).disabled,false);assert.doesNotMatch(h.dialog(),/id="mpDeleteUserTemplate"/);act(h,'history-delete-version',id,1);assert.match(h.dialog(),/最后一个版本/);h.runConfirm();assert.equal(h.state().templates.some(t=>t.id===id),false);assert.equal(h.state().templates.find(t=>t.id==='gpt').versions.length,3);
});
