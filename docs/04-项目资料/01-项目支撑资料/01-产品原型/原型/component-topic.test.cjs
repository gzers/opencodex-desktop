const test=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path');
const {make}=require('./component-topic-harness.cjs'),catalog=require('./component-catalog.js');
test('使用建议覆盖全部登记组件，规范/方案/建议可区分，来源文件和章节存在',()=>{
 const statuses=new Set(),slug=heading=>heading.toLowerCase().replace(/[^\p{L}\p{N}_ -]/gu,'').replace(/ /g,'-');
 for(const component of catalog.components){const section=catalog.section(component.section);assert.equal(section.usage.length,4);assert.ok(Object.isFrozen(section.usage));}
 for(const section of catalog.sections)for(const item of section.usage){
  assert.ok(['existing','proposal','suggestion'].includes(item.status));statuses.add(item.status);assert.ok(Object.isFrozen(item));
  const [file,anchor]=item.source.href.split('#'),target=path.resolve(__dirname,file);assert.ok(fs.existsSync(target),item.source.href);
  if(anchor){const headings=fs.readFileSync(target,'utf8').split('\n').filter(line=>/^#{1,6} /.test(line)).map(line=>slug(line.replace(/^#{1,6} /,'').trim()));assert.ok(headings.includes(decodeURIComponent(anchor)),item.source.href);}
 }
 assert.deepEqual([...statuses].sort(),['existing','proposal','suggestion']);assert.equal(catalog.validate(),true);
});
test('组件专题：从真实总原型接线检查交互与隔离（内存DOM，非实屏）',async t=>{
 const h=await make(),{w,document:d}=h,$=q=>d.querySelector(q),state=()=>JSON.parse(JSON.stringify(w.componentTopicPrototype.state()));t.after(()=>h.dispose());
 const baseline=JSON.stringify(w.modelPrototype.state()),trayBaseline=w.eval('JSON.stringify({trayState,trayQuickSettings})');
 const errors=()=>assert.deepEqual(h.errors,[]);
 await t.test('目录、资源、非功能菜单、连续正文、深链接和搜索同源',async()=>{
  errors();assert.deepEqual([...d.querySelectorAll('[aria-labelledby=launchNonFunctionalTitle] button')].map(b=>b.textContent.trim()),['组件与样式']);assert.equal(catalog.validate(),true);assert.equal(catalog.sections.length,19);
  h.click('[data-launch=components]');await h.settle();assert.equal(d.documentElement.dataset.route,'components');assert.ok(d.body.classList.contains('component-topic-mode'));assert.equal(state().section,'brand');
  assert.equal(d.querySelectorAll('[data-ct-section-id]').length,catalog.sections.length);
  for(const s of catalog.sections){
   assert.ok(d.getElementById(s.anchor));assert.ok($('[data-ct-section="'+s.id+'"]'));
   const guide=$('[data-ct-usage="'+s.id+'"]');assert.ok(guide);assert.equal(guide.querySelectorAll('.ct-usage-row').length,4);assert.equal(guide.parentElement.id,s.anchor);
   assert.deepEqual([...guide.querySelectorAll('dt')].map(el=>el.textContent),['默认规格','适用场景','切换条件','避免误用']);
   assert.ok(guide.previousElementSibling.classList.contains('ct-sample'));assert.ok(guide.nextElementSibling.classList.contains('ct-meta'));
   assert.equal(guide.querySelectorAll('.ct-usage-source').length,4);assert.ok(!guide.closest('details'));
   for(const item of s.usage)assert.ok(guide.textContent.includes(item.text));
  }
  h.input('#componentNavSearch','Fast');assert.ok($('#componentNavLinks').textContent.length);h.input('#componentNavSearch','不存在的控件');assert.match($('#componentNavLinks').textContent,/没有匹配/);assert.equal(d.querySelectorAll('[data-ct-section-id]').length,19);h.input('#componentNavSearch','');
  h.click('[data-ct-section=buttons]');assert.equal(state().section,'buttons');assert.match(w.location.hash,/components\?section=buttons/);assert.equal($('[data-ct-section=buttons]').getAttribute('aria-current'),'location');
  h.click('[data-ct=toggle-nav]');assert.ok(d.body.classList.contains('component-nav-collapsed'));h.click('[data-ct=toggle-nav]');
  assert.deepEqual(catalog.sections.slice(0,4).map(s=>s.id),['brand','app-icons','tray-icons','icons-spacing']);assert.equal(state().size,'fluid');assert.ok($('#ctNotificationMount #notificationBtn'));assert.equal($('.topbar #notificationBtn'),null);
  for(const img of d.querySelectorAll('.ct-brand-image')){const ref=img.getAttribute('src');assert.ok(fs.existsSync(path.resolve(__dirname,ref)),ref);assert.ok(Number(img.getAttribute('width'))>=16);assert.ok(img.alt);}
  for(const c of catalog.components.filter(c=>c.layer==='brand')){assert.ok(c.assets.length);for(const ref of c.assets)assert.ok(fs.existsSync(path.resolve(__dirname,ref)));}
  const ids=[...d.querySelectorAll('[id]')].map(el=>el.id),dups=ids.filter((x,i)=>ids.indexOf(x)!==i);assert.deepEqual(dups,[]);
  errors();
 });
 await t.test('亮暗×高中低、四缩放×双窗口不丢输入/选择/滚动，不写偏好',async()=>{
  const name=h.input('#ctName','仍然保留的名称');h.change('#ctCheckbox',false);const selectedBefore=w.modelPickerPrototype.state().pending;
  const writeCount=h.writes.length,scroll=$('.main').scrollTop;
  for(const theme of ['light','dark'])for(const effects of ['high','mid','low'])for(const zoom of [50,100,150,200])for(const size of ['1180','900']){
   for(const [k,v] of Object.entries({theme,effects,zoom,size}))w.componentTopicPrototype.setEnvironment(k,v);
   assert.equal(d.documentElement.dataset.theme,theme);assert.equal(d.documentElement.dataset.effects,effects);assert.equal(name.value,'仍然保留的名称');assert.equal($('#ctCheckbox').checked,false);assert.equal($('.main').scrollTop,scroll);
  }
  assert.deepEqual(w.modelPickerPrototype.state().pending,selectedBefore);assert.equal(h.writes.length,writeCount);assert.equal(h.network.length,0);
  h.click('#themeSeg [data-theme-option=light]');assert.equal(h.writes.length,writeCount);
  for(const [k,v] of Object.entries({theme:'light',effects:'high',zoom:100,size:'1180'}))w.componentTopicPrototype.setEnvironment(k,v);
  errors();
 });
 await t.test('字段校验/Tab键/树表/长表格与动作菜单',async()=>{
  h.input('#ctName','');h.click('[data-ct=validate]');assert.equal($('#ctName').getAttribute('aria-invalid'),'true');assert.equal(d.activeElement.id,'ctName');
  h.input('#ctName','已修正');h.click('[data-ct=validate]');assert.equal($('#ctNameError').hidden,true);
  h.click('#ctTab-advanced');assert.equal($('#ctTab-advanced').getAttribute('aria-selected'),'true');$('#ctTab-advanced').dispatchEvent(new w.KeyboardEvent('keydown',{key:'Home',bubbles:true}));assert.equal($('#ctTab-general').getAttribute('aria-selected'),'true');
  assert.equal($('#ctTree').children.length,5);h.click('[data-ct-tree=models]');assert.equal($('#ctTree').children.length,3);h.click('[data-ct-tree=models]');assert.equal($('#ctTree').children.length,5);
  h.change('#ctTableAll',true);assert.equal(state().selectedRows.length,24);h.change('[data-ct-row="3"]',false);assert.equal($('#ctTableAll').indeterminate,true);h.click('[data-ct=sort-table]');assert.equal(state().selectedRows.length,23);
  assert.equal($('[data-ct=example-save]').disabled,true);assert.equal($('[data-ct=example-reset]').disabled,true);h.input('#ctExampleName','已保存工作空间');assert.equal($('[data-ct=example-save]').classList.contains('primary'),true);h.click('[data-ct=example-save]');assert.equal($('#ctSaveState').textContent,'已保存');assert.equal($('[data-ct=example-save]').disabled,true);h.input('#ctExampleName','未保存的第二次修改');h.click('[data-ct=example-reset]');assert.equal($('#ctExampleName').value,'已保存工作空间');
  h.click('[data-ct=menu]');assert.equal(w.prototypeActionMenu.state().items[1].disabled,true);h.click('[data-action-menu-index="0"]');assert.match($('#ctMenuResult').textContent,/复制/);assert.equal(w.prototypeActionMenu.state(),null);errors();
 });
 await t.test('共享表单弹窗：错误就地、Select同层、Esc与返回焦点',async()=>{
  const opener=h.click('[data-ct=dialog-form]');await h.settle();h.input('#ctDialogName','');h.click('#modalConfirm');assert.equal($('#modalMask').style.display,'flex');assert.equal($('#ctDialogName').getAttribute('aria-invalid'),'true');
  assert.ok($('#ctDialogSelect'));h.input('#ctDialogName','弹窗新名称');h.click('#modalConfirm');assert.equal($('#modalMask').style.display,'none');assert.equal(d.activeElement,opener);assert.match($('#ctDialogResult').textContent,/弹窗新名称/);
  h.click('[data-ct=dialog-detail]');d.activeElement.dispatchEvent(new w.KeyboardEvent('keydown',{key:'Escape',bubbles:true}));assert.equal($('#modalMask').style.display,'none');errors();
 });
 await t.test('同一模型选择器页内/弹窗：确认、取消、推理与Fast，未改业务目录',async()=>{
  assert.equal($('#ctPickerInline .mpk-row')!==null,true);h.click('#ctPickerInline [data-mpk=pick]');await h.settle();
  const range=$('#mpkEffortRange');if(range){range.value='2';range.dispatchEvent(new w.Event('input',{bubbles:true}));}
  const fast=$('[data-effort-fast]');if(fast&&!fast.disabled)fast.click();
  h.click('#ctPickerInline [data-mpk=accept]');assert.equal(state().pickerResult.models.length,1);
  const confirmed=JSON.stringify(state().pickerResult);h.click('[data-ct=picker]');await h.settle();assert.equal(d.documentElement.dataset.route,'components');assert.equal($('#modal').classList.contains('mpk-dialog'),true);
  h.click('#modalCancel');await h.settle();assert.equal(JSON.stringify(state().pickerResult),confirmed);assert.ok($('#ctPickerInline .mpk-row'));
  assert.equal(JSON.stringify(w.modelPrototype.state()),baseline);errors();
 });
 await t.test('模型fixture单多选/长表/空/失败重试/加载完成/失效引用',async()=>{
  h.change('#ctPickerModeTool','multiple');assert.ok($('#ctPickerInline #mpkSelectResults'));h.change('#ctPickerInline #mpkSelectResults',true);h.click('#ctPickerInline [data-mpk=accept]');assert.ok(state().pickerResult.models.length>1);
  h.change('#ctPickerScene','long');assert.equal(w.modelPickerPrototype.state().catalogSize,108);h.change('#ctPickerScene','empty');assert.match($('#ctPickerInline #mpkModels').textContent,/暂无模型/);
  h.change('#ctPickerScene','failed');assert.match($('#ctPickerInline #mpkModels').textContent,/读取失败/);h.click('#ctPickerInline [data-mpk=load-finished]');assert.equal(state().pickerScenario,'normal');assert.ok($('#ctPickerInline .mpk-row'));
  h.change('#ctPickerScene','loading');assert.match($('#ctPickerInline #mpkModels').textContent,/正在读取/);h.click('#ctPickerInline [data-mpk=load-finished]');assert.ok($('#ctPickerInline .mpk-row'));
  h.change('#ctPickerScene','missing');assert.match($('#ctPickerInline #mpkPicked').textContent,/目录缺失/);assert.equal($('#ctPickerInline [data-mpk=accept]').disabled,true);h.change('#ctPickerScene','normal');h.change('#ctPickerModeTool','single');errors();
 });
 await t.test('节点选择与检测分离、停止/晚回失效、弹层键盘与逐项/一键',async()=>{
  w.componentTopicPrototype.navigate('node-picker');h.click('[data-ct=node-open]');assert.ok($('.ct-node-popup'));assert.equal(state().nodeSelected,'primary');
  d.activeElement.dispatchEvent(new w.KeyboardEvent('keydown',{key:'ArrowDown',bubbles:true}));assert.equal(state().nodeSelected,'primary');
  h.click('.ct-node-popup [data-ct-node=hk]');assert.equal(state().nodeSelected,'hk');assert.equal($('.ct-node-popup'),null);assert.deepEqual(state().nodeResults,{});
  w.componentTopicPrototype.detectNodes();await h.wait(80);w.componentTopicPrototype.stopNodes();const stopped=JSON.stringify(state().nodeResults);await h.wait(500);assert.equal(JSON.stringify(state().nodeResults),stopped);
  h.change('#ctNodeScene','mixed');w.componentTopicPrototype.detectNodes();await h.wait(600);assert.equal(state().nodeResults.hk,'连接失败');assert.equal(state().nodeSelected,'hk');
  h.change('#ctNodeScene','stale');assert.equal(state().nodeResults.primary,'结果已过期');h.change('#ctNodeScene','timeout');w.componentTopicPrototype.detectNodes('primary');await h.wait(230);assert.equal(state().nodeResults.primary,'超时');errors();
 });
 await t.test('平台样例：关闭仅影响子窗口；托盘共享模板及禁用原因，不改业务状态',async()=>{
  h.click('#ctWindowSample [data-ct-window=close]');assert.equal(state().window.visibility,'tray');assert.equal($('.window').hidden,false);h.click('[data-ct=window-reopen]');
  h.change('#ctTrayState','stopped');assert.equal($('#ctTraySample [data-ct-tray=stop]').disabled,true);h.click('#ctTraySample [data-ct-tray=start]');assert.equal(state().tray.state,'running');
  h.change('#ctTrayOs','windows');assert.ok($('#ctTraySample .tray-menu.windows'));h.change('#ctTrayState','at_risk');assert.match($('#ctTraySample .tray-title').textContent,/未运行/);
  assert.equal(w.eval('JSON.stringify({trayState,trayQuickSettings})'),trayBaseline);assert.equal(JSON.stringify(w.modelPrototype.state()),baseline);errors();
 });
 await t.test('通知只经登记事件与原宿主；去重、多任务、收起恢复、读/解决/删除/重置隔离',async()=>{
  const api=w.prototypeNotificationPublisher;assert.throws(()=>api.publish('made-up.event'),/未注册/);
  h.click('[data-ct=notify-task-a]');h.click('[data-ct=notify-task-b]');const items=api.state();assert.equal(items.length,2);const a=items.find(n=>n.demo.taskId==='a').id;
  h.click('[data-ct=notify-task-a]');assert.equal(api.state().length,2);assert.equal(d.querySelectorAll('#taskHost [data-task-notification^="component-demo:"]').length,2);
  h.click('[data-task-notification=\"'+a+'\"] [data-collapse=task]');await h.wait(650);assert.equal([...d.querySelectorAll('[data-task-notification]')].filter(n=>n.dataset.taskNotification===a).length,0);assert.equal(api.open(a),true);assert.equal(api.state().find(n=>n.id===a).read,true);
  api.action('next',a);api.action('next',a);api.action('next',a);assert.equal(api.state().find(n=>n.id===a).demo.state,'success');assert.equal(api.state().length,2);
  w.deleteNotification(a);api.publish('component.demo.task',{state:'failed'});assert.equal(api.state().find(n=>n.id===a).deleted,true);assert.equal([...d.querySelectorAll('[data-task-notification]')].filter(n=>n.dataset.taskNotification===a).length,0);
  const before=w.eval('notifications.filter(n=>!n.eventId?.startsWith("component.demo.")).map(n=>n.id).join(",")');api.reset();assert.equal(api.state().length,0);assert.equal(w.eval('notifications.filter(n=>!n.eventId?.startsWith("component.demo.")).map(n=>n.id).join(",")'),before);errors();
 });
 await t.test('离开还原评审环境，功能草稿未变，旧入口继续可用',async()=>{
  w.componentTopicPrototype.setEnvironment('theme','dark');w.setRoute('settings');await h.settle();assert.equal(d.body.classList.contains('component-topic-mode'),false);assert.equal(d.documentElement.dataset.theme,'light');
  assert.equal(JSON.stringify(w.modelPrototype.state()),baseline);assert.equal(h.network.length,0);assert.ok($('.topbar #notificationBtn'));assert.equal($('#ctNotificationMount #notificationBtn'),null);
  w.setRoute('model-picker');await h.settle();assert.equal(d.documentElement.dataset.route,'model-picker');assert.ok($('#modal').classList.contains('mpk-dialog'));w.closeModal();w.setRoute('tray');await h.settle();assert.equal(d.documentElement.dataset.route,'tray');w.syncLaunchActive();assert.equal($('[data-launch=components]').classList.contains('active'),true);
  w.setRoute('overview');await h.settle();assert.equal($('#maintenanceDemoCard').hidden,false);h.click('#maintenanceDemoCard [data-maint-action=registry]');assert.match($('#modalTitle').textContent,/通知事件注册表/);w.closeModal();
  w.windowsPrototype.enter();assert.equal(d.body.classList.contains('windows-mode'),true);assert.equal($('[data-launch=components]').classList.contains('active'),true);w.windowsPrototype.exit();assert.equal(d.body.classList.contains('windows-mode'),false);errors();
 });
});
test('组件专题深链接刷新：章节/环境恢复，实际浏览布局仍须实屏检查',async t=>{
 const h=await make('#components?section=forms&theme=dark&effects=low&zoom=150&size=900&long=1');t.after(()=>h.dispose());
 const s=h.w.componentTopicPrototype.state();assert.deepEqual(h.errors,[]);assert.equal(s.section,'forms');assert.equal(s.theme,'dark');assert.equal(s.effects,'low');assert.equal(s.zoom,150);assert.equal(s.size,'900');assert.equal(s.long,true);
 const main=h.document.querySelector('.main');main.scrollTop=160+catalog.sections.findIndex(s=>s.id==='forms')*800-100-12;main.dispatchEvent(new h.w.Event('scroll'));await h.settle();assert.equal(h.w.componentTopicPrototype.state().section,'forms');
 h.w.componentTopicPrototype.navigate('buttons');h.w.history.back();await h.wait(100);assert.equal(h.w.componentTopicPrototype.state().section,'forms');assert.equal(h.network.length,0);assert.deepEqual(h.errors,[]);
});
