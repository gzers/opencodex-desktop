/* Agent 设置页边界与既有渠道策略迁移；不代表浏览器视觉 QA。 */
const {test}=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),vm=require('node:vm');
const make=require('./model-prototype-test-harness.cjs');
function picker(h){let options;h.context.window.modelPickerPrototype={choose(input){options=input;h.context.closeModal();return true;}};return {select(fast,modelId='gpt-5.4'){options.onSelect({kind:'models',models:[{kind:'channel',channelId:'newapi',modelId,fast}]});}};}
test('设置页只承载 Agent 配置，文件与行为说明在隐藏注释，按钮前后一致',()=>{
 const h=make();const page=()=>h.nodes.get('mpSettingsContent').innerHTML;
 assert.match(page(),/<h2>Agent 配置<\/h2>/);assert.match(page(),/模型模版助手 · 本机共用/);assert.match(page(),/data-mp="agent-settings">Agent 配置/);assert.doesNotMatch(page(),/data-fast-mode|data-fast-rows|<details|调整 Agent/);
 const display=page().replace(/<div class="annotation-source" hidden>[\s\S]*?<\/div>/g,'');assert.doesNotMatch(display,/model-policy\.json|preferences\.json|实现尚未|持久化|同步过滤/);
 h.configureAgent();assert.match(page(),/data-mp="agent-settings">Agent 配置/);assert.match(page(),/快速已关闭/);
 const html=fs.readFileSync(path.join(__dirname,'index.html'),'utf8');assert.match(html,/data-settings-section="models" data-anchor="models">Agent 配置/);assert.match(html,/id="settings-models"/);
});
test('弹窗交互说明不再占正文，传给共用原型注释，模型选择和保存仍可达',()=>{
 const h=make();let opts;const original=h.context.openModal;h.context.openModal=(...args)=>{opts=args[4];return original(...args);};
 h.click('agent-settings');assert.doesNotMatch(h.dialog(),/选择后先返回这里|运行中的任务继续|实现尚未/);assert.match(opts.prototypeAnnotation.body,/选择后先返回这里/);assert.match(opts.prototypeAnnotation.body,/运行中的任务继续/);assert.match(h.dialog(),/id="mpAgentModelChoice"/);
});
test('Agent 快速状态随组合与能力显示，支持能力不等于已开启，取消不更新摘要',()=>{
 const h=make(),p=picker(h),summary=()=>h.nodes.get('mpSettingsContent').innerHTML;
 h.click('agent-settings');h.click('agent-pick-model');p.select(true);assert.doesNotMatch(summary(),/快速已开启/);h.runConfirm();assert.match(summary(),/快速已开启/);
 h.click('agent-settings');h.click('agent-pick-model');p.select(false);h.context.closeModal();assert.match(summary(),/快速已开启/);
 h.click('agent-settings');h.click('agent-pick-model');p.select(false);h.runConfirm();assert.match(summary(),/快速已关闭/);
 h.click('agent-settings');h.click('agent-pick-model');p.select(false,'claude-opus-4-6');h.runConfirm();assert.match(summary(),/快速能力待核对/);
 h.proto.commitModelPatches('newapi',[{key:'newapi::6',rules:{fast:'unsupported'}}]);h.click('save');h.runConfirm();h.drain();assert.match(summary(),/不支持快速模式/);
});
test('高级 Fast 入口已移除，旧事件无效，Agent 选择不改既有运行策略与兼容目录',()=>{
 const h=make(),p=picker(h);h.configureAgent();const before=JSON.stringify(h.state().saved),draft=JSON.stringify(h.state().draft),count=h.proto.projection().length;
 assert.doesNotMatch(h.page(),/data-mp="more"|data-mp="model-advanced"/);h.click('more');
 h.click('model-advanced');assert.notEqual(h.nodes.get('modalMask').style.display,'flex');h.event('change','[data-fast-mode]',{},'off');h.event('change','[data-fast-rows]',{},'',false);assert.equal(JSON.stringify(h.state().draft),draft);
 h.click('agent-settings');h.click('agent-pick-model');p.select(true);h.runConfirm();assert.equal(h.state().agent.fast,true);assert.equal(JSON.stringify(h.state().saved),before);assert.equal(JSON.stringify(h.state().draft),draft);assert.equal(h.proto.projection().length,count);
});
test('共用弹窗原型注释自定义标题和内容安全转义，其他弹窗默认说明仍保留',()=>{
 const html=fs.readFileSync(path.join(__dirname,'index.html'),'utf8'),begin=html.indexOf('function openModal(title,body,confirmText,handler,opts){'),end=html.indexOf('\nfunction closeModal()',begin);
 const node=()=>({innerHTML:'',textContent:'',hidden:false,dataset:{},classList:{add(){},remove(){}},focus(){}}),nodes=new Map(),ctx={window:{},document:{activeElement:null,getElementById(id){if(!nodes.has(id))nodes.set(id,node());return nodes.get(id);}},requestAnimationFrame(){},logEsc:value=>String(value).replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c])),modalTitle:node(),modalBody:node(),modalConfirm:node(),modalCancel:node(),modalDelete:node(),modalMask:node(),modal:node(),confirmHandler:null,confirmKeepsOpen:false,deleteHandler:null,extraHandler:null};ctx.modalMask.style={};
 vm.createContext(ctx);vm.runInContext(html.slice(begin,end),ctx);ctx.openModal('Agent','<p>执行模型</p>','保存',null,{hideNote:true,prototypeAnnotation:{title:'<Agent>',body:'说明 <script>invalid</script>'}});
 assert.match(nodes.get('modalAnnotationHost').innerHTML,/annotation-source" hidden><p>说明 &lt;script&gt;/);assert.doesNotMatch(nodes.get('modalAnnotationHost').innerHTML,/<script>invalid/);assert.equal((nodes.get('modalAnnotationHost').innerHTML.match(/id="ann-modal-demo"/g)||[]).length,1);
 ctx.openModal('其他','<p>其他内容</p>','关闭',null,{hideNote:true});assert.equal(ctx.modalBody.innerHTML,'<p>其他内容</p>');assert.match(nodes.get('modalAnnotationHost').innerHTML,/确认动作只演示流程与文案/);assert.doesNotMatch(nodes.get('modalAnnotationHost').innerHTML,/invalid/);
});
