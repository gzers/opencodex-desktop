const test=require('node:test'),assert=require('node:assert/strict');
const {create}=require('./tree-navigation.js'),{make}=require('./component-topic-harness.cjs');
const nodes=[{id:'p',label:'项目',kind:'folder'},{id:'c',parent:'p',label:'配置对话',kind:'chat',own:true},{id:'a',parent:'c',label:'审阅配置继承',kind:'agent'},{id:'other',parent:'p',label:'窗口布局',kind:'chat'},{id:'g',label:'未分配',selectable:false},{id:'off',parent:'g',label:'离线对象',disabled:true}];
const store=()=>create({nodes,selected:'c',expanded:['p','c','g']});
test('展开与选择独立，折叠父级不改所选ID，定位恢复路径',()=>{
 const s=store();s.toggle('p',false);assert.equal(s.state().selected,'c');assert.equal(s.state().selectionOutside,true);s.reveal();assert.equal(s.state().selectionOutside,false);assert.deepEqual(s.state().expanded,['c','g','p']);assert.equal(s.state().active,'c');
});
test('搜索子Agent保留祖先顺序与层级，搜索内可折叠，清空恢复日常展开',()=>{
 const s=store();s.toggle('p',false);const before=s.state().expanded;s.setQuery('审阅');assert.deepEqual(s.state().rows.map(n=>[n.id,n.depth]),[['p',0],['c',1],['a',2]]);assert.equal(s.state().selected,'c');s.toggle('c',false);assert.deepEqual(s.state().rows.map(n=>n.id),['p','c']);s.expandAll();assert.deepEqual(s.state().expanded,before);s.collapseAll();assert.deepEqual(s.state().expanded,before);s.setQuery('');assert.deepEqual(s.state().expanded,before);assert.equal(s.state().rows.some(n=>n.id==='c'),false);
});
test('搜索项目展示允许的后代，筛选保留上下文与未命中的已选引用',()=>{
 const s=store();s.setQuery('项目');assert.deepEqual(s.state().rows.map(n=>n.id),['p','c','a','other']);s.setQuery('');s.setFilter(n=>n.id==='other');assert.deepEqual(s.state().rows.map(n=>n.id),['p','other']);assert.equal(s.state().selected,'c');assert.equal(s.state().selectionOutside,true);s.reveal();assert.equal(s.state().filtered,false);assert.equal(s.state().selectionOutside,false);
});
test('基础可见范围不强制展开，显式筛选自动揭示父级但不污染展开偏好',()=>{
 const s=create({nodes,expanded:[]});s.setFilter(n=>n.id!=='off',{reveal:false});assert.deepEqual(s.state().rows.map(n=>n.id),['p','g']);s.setFilter(n=>n.id==='a');assert.deepEqual(s.state().rows.map(n=>n.id),['p','c','a']);s.setFilter(null);assert.deepEqual(s.state().expanded,[]);
});
test('键盘焦点独立移动，左右折叠父级，明确激活才选择；分组与禁用不选择',()=>{
 const s=store();s.focus('c');s.key('ArrowDown');assert.equal(s.state().active,'a');assert.equal(s.state().selected,'c');assert.deepEqual(s.key('Enter'),{type:'select',id:'a'});s.key('ArrowLeft');assert.equal(s.state().active,'c');s.key('ArrowLeft');assert.equal(s.state().rows.some(n=>n.id==='a'),false);assert.equal(s.state().selected,'a');s.focus('g');s.key(' ');assert.equal(s.state().selected,'a');assert.equal(s.select('off'),false);
});
test('目录更新与短暂清空保留所选引用，重新出现不自动发起选择',()=>{
 const s=store();s.setNodes([]);assert.equal(s.state().selected,'c');assert.equal(s.state().selectedLabel,'配置对话');assert.equal(s.state().selectionMissing,true);s.setNodes(nodes);assert.equal(s.state().selectionMissing,false);assert.equal(s.select('c'),false);
});
test('目录延用既有循环、缺父与重复校验，不放宽树表结构合同',()=>{
 for(const input of [[{id:'a',parent:'missing'}],[{id:'a',parent:'b'},{id:'b',parent:'a'}],[{id:'a'},{id:'a'}]])assert.throws(()=>create({nodes:input}));
});
test('公共组件接线：选择/展开/搜索/键盘、状态、目录登记与业务隔离',async t=>{
 const h=await make('#components?section=tree-navigation'),{w,document:d}=h,$=q=>d.querySelector(q);t.after(()=>h.dispose());
 const state=()=>w.treeNavigationDemo.state(),business=JSON.stringify(w.conversationPrototype.state());
 assert.equal(w.componentTopicPrototype.state().section,'tree-navigation');assert.equal(w.PrototypeComponentCatalog.components.find(x=>x.id==='navigation.tree').status.production,false);assert.equal(w.PrototypeComponentCatalog.components.find(x=>x.id==='navigation.tree').productionRevision,null);
 const tree=$('#tnDemoTree .tn-tree');assert.equal(tree.getAttribute('role'),'tree');assert.equal(tree.tabIndex,0);assert.equal(d.querySelectorAll('#ctTreeNavigationDemo [role=tree]').length,1);
 h.click('[data-tn-id=tools] [data-tn-toggle]');assert.equal(state().tree.selected,'config');assert.equal($('#tnDemoTree .tn-outside').hidden,false);h.click('[data-tn-reveal]');assert.equal(state().tree.selectionOutside,false);
 h.input('#tnDemoSearch','审阅');assert.deepEqual([...d.querySelectorAll('#tnDemoTree [role=treeitem]')].map(el=>[el.dataset.tnId,el.getAttribute('aria-level')]),[['tools','1'],['config','2'],['review','3']]);assert.equal(state().activations,0);
 h.click('[data-tn-id=review]');assert.equal(state().tree.selected,'review');assert.equal(state().activations,1);assert.equal(d.activeElement,tree);assert.ok(d.getElementById(tree.getAttribute('aria-activedescendant')));
 h.input('#tnDemoSearch','不存在');assert.match(tree.textContent,/没有匹配/);assert.equal(state().tree.selected,'review');h.click('[data-tn-clear]');assert.equal($('#tnDemoSearch').value,'');assert.equal(state().tree.selected,'review');
 h.click('[data-tn-id=config]');const before=state().activations;tree.dispatchEvent(new w.KeyboardEvent('keydown',{key:'ArrowDown',bubbles:true}));assert.equal(state().tree.active,'review');assert.equal(state().activations,before);tree.dispatchEvent(new w.KeyboardEvent('keydown',{key:'Enter',bubbles:true}));assert.equal(state().tree.selected,'review');assert.equal(state().activations,before+1);
 h.change('#tnDemoDensity','compact');assert.equal($('#tnDemoTree').dataset.density,'compact');h.change('#tnDemoScene','loading');assert.equal(tree.getAttribute('aria-busy'),'true');assert.equal(state().tree.selected,'review');h.change('#tnDemoScene','failed');h.click('[data-tn-retry]');assert.equal(state().scene,'normal');assert.equal(state().tree.selected,'review');
 h.change('#tnDemoScene','long');h.change('#tnDemoWidth','224');assert.equal($('#ctTreeNavigationDemo .tn-demo-layout').style.getPropertyValue('--tn-demo-width'),'224px');assert.equal(state().tree.selected,'review');
 const another=d.createElement('div');d.body.appendChild(another);let changes=0,activations=0;const isolated=w.PrototypeTreeNavigation.mount(another,{nodes:[{id:'unsafe',label:'<img src=x onerror=alert(1)>',kind:'chat'}],onSelect(){changes++;},onActivate(){activations++;}});assert.equal(another.querySelector('img'),null);assert.match(another.textContent,/<img src=x/);isolated.setSelected('unsafe');assert.equal(changes,0);assert.equal(activations,0);another.querySelector('[data-tn-id]').click();assert.equal(changes,0);assert.equal(activations,1);another.querySelector('.tn-tree').dispatchEvent(new w.KeyboardEvent('keydown',{key:'Enter',bubbles:true}));assert.equal(changes,0);assert.equal(activations,2);assert.equal(state().tree.selected,'review');isolated.destroy();another.remove();
 assert.equal(h.network.length,0);assert.equal(JSON.stringify(w.conversationPrototype.state()),business);assert.deepEqual(h.errors,[]);
});
