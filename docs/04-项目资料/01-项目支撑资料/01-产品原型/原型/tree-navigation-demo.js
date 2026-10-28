/* 组件专题独立 fixture：项目、对话、子 Agent 仅作树结构样例。业务页复用同一组件，fixture与业务数据分离。 */
(() => {
 'use strict';
 const esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 const $=(q,r=document)=>r.querySelector(q),$$=(q,r=document)=>[...r.querySelectorAll(q)];
 const data=[
  {id:'tools',label:'桌面管理器',kind:'folder',description:'~/Projects/desktop-manager',status:'专属',own:true},
  {id:'config',parent:'tools',label:'模型配置与对话边界',kind:'chat',status:'专属',own:true},
  {id:'review',parent:'config',label:'审阅配置继承',kind:'agent',description:'子 Agent · 配置审阅'},
  {id:'window',parent:'tools',label:'窗口布局适配',kind:'chat'},
  {id:'task',parent:'tools',label:'长期任务与工具记录',kind:'chat',status:'未保存',tone:'warning',own:true},
  {id:'archived',parent:'tools',label:'旧方案整理',kind:'chat',status:'归档',archived:true},
  {id:'site',label:'文档网站',kind:'folder',description:'~/Projects/docs-site'},
  {id:'web',parent:'site',label:'导航与文档检索',kind:'chat'},
  {id:'unassigned',label:'未分配项目',kind:'group',selectable:false},
  {id:'notes',parent:'unassigned',label:'临时需求记录',kind:'chat'},
  {id:'readonly',parent:'unassigned',label:'离线来源中的对话',kind:'chat',disabled:true,status:'离线'}
 ];
 const state={scene:'normal',density:'normal',width:'280',descriptions:false,filter:'all',selected:'config',activations:0};
 let host=null,api=null,tool=null;
 const button=(label,action,attrs='')=>'<button type="button" class="btn ghost btn-sm" data-tn-demo="'+action+'" '+attrs+'>'+label+'</button>';
 function html(){return '<div class="tn-demo"><div class="tn-demo-layout"><section class="tn-demo-panel" aria-label="树形导航示例"><div class="tn-demo-head"><strong>项目与对话</strong><div class="tn-demo-tools">'+button('↻','refresh','aria-label="刷新树形导航样例" title="刷新"')+'<details class="tn-demo-more"><summary aria-label="树形导航操作" title="更多操作">⋯</summary><div class="tn-demo-menu">'+button('全部展开','expand-all')+button('全部收起','collapse-all')+'</div></details></div></div><div class="tn-demo-search"><input id="tnDemoSearch" class="input" type="text" role="searchbox" aria-label="搜索树形对象" placeholder="搜索项目或对话" autocomplete="off"><button type="button" class="tn-demo-search-clear" data-tn-demo="clear" aria-label="清空树形搜索" hidden>×</button></div><div class="tn-demo-filter" role="group" aria-label="树形导航筛选">'+[['all','全部'],['own','专属'],['archived','归档']].map(([id,label])=>'<button type="button" data-tn-demo="filter" data-filter="'+id+'" aria-pressed="'+(id==='all')+'">'+label+'</button>').join('')+'<span class="tn-demo-summary" id="tnDemoCount" role="status"></span></div><div id="tnDemoTree"></div></section><aside class="tn-demo-detail" aria-label="树形导航返回预览"><div class="tn-demo-kicker">当前对象</div><h4 id="tnDemoTitle"></h4><div class="tn-demo-path" id="tnDemoPath"></div><dl id="tnDemoFacts"></dl><div class="tn-demo-keyboard">↑ ↓ 移动焦点　← → 展开或返回父级<br>Enter / 空格 选择　Tab 离开树形导航</div></aside></div></div>';}
 function nodes(){
  if(state.scene==='empty')return [];
  const list=data.map(n=>({...n}));
  if(state.scene==='long'){list[0].label='桌面管理器与跨平台配置工具的长期维护项目';list[1].label='梳理多个渠道之间的模型配置、对话边界与继承关系';list[2].label='独立审阅一个非常长的项目与对话配置继承方案';for(let i=0;i<45;i++)list.push({id:'long-'+i,parent:i%2?'site':'tools',label:'配置与交互记录 '+String(i+1).padStart(2,'0'),kind:'chat',status:i%9===0?'专属':'',own:i%9===0});}
  if(state.scene==='deep'){list.push({id:'agent-2',parent:'review',label:'核对默认模型',kind:'agent'},{id:'agent-3',parent:'agent-2',label:'检查实际继承值',kind:'agent'});}
  return list.map(n=>({...n,...(!state.descriptions?{description:''}:{})}));
 }
 function predicate(n){return state.filter==='archived'?n.archived:!n.archived&&(state.filter!=='own'||n.own);}
 function update(s){
  if(!host)return;$('#tnDemoCount',host).textContent=s.filtered?s.matches+' 项':s.rows.length+' 项';
  const selected=s.nodes.find(n=>n.id===s.selected),type={folder:'项目',chat:'对话',agent:'子 Agent',group:'分组'};
  $('#tnDemoTitle',host).textContent=s.selectedLabel||'尚未选择对象';
  const path=selected?[...api?.store.parents(selected.id)||[],selected.id].map(id=>s.nodes.find(n=>n.id===id)?.label||id):[];
  $('#tnDemoPath',host).textContent=path.join(' / ');
  $('#tnDemoFacts',host).innerHTML=[['对象类型',type[selected?.kind]||'目录未提供'],['配置状态',selected?.status||(selected?.kind==='folder'?'未单独设置':'沿用上层')],['稳定标识',s.selected||'—'],['当前选择',s.selectionOutside?'保留 · 当前列表外':'列表内可见']].map(([label,value])=>'<div><dt>'+label+'</dt><dd>'+esc(value)+'</dd></div>').join('');
  const clear=$('[data-tn-demo="clear"]',host);if(clear)clear.hidden=!s.query;
 }
 function syncFilters(){for(const b of $$('[data-tn-demo="filter"]',host))b.setAttribute('aria-pressed',String(b.dataset.filter===state.filter));}
 function clear(){state.filter='all';$('#tnDemoSearch',host).value='';api.setQuery('');api.setFilter(predicate,{reveal:state.filter!=='all'});syncFilters();}
 function syncTools(){if(!tool)return;for(const [id,value] of [['tnDemoScene',state.scene],['tnDemoDensity',state.density],['tnDemoWidth',state.width]])$( '#'+id,tool).value=value;$('#tnDemoDescriptions',tool).checked=state.descriptions;window.prototypeSelect?.enhance(tool);}
 function setScene(scene){state.scene=scene;if(!api)return;api.setNodes(nodes());api.setStatus(['loading','failed'].includes(scene)?scene:'ready');if(scene==='deep')api.expandAll();syncTools();}
 function mount(element){
  if(!element)return;api?.destroy();host=element;
  host.innerHTML=html();host.querySelector('.tn-demo-layout').style.setProperty('--tn-demo-width',state.width+'px');
  api=window.PrototypeTreeNavigation.mount($('#tnDemoTree',host),{nodes:nodes(),selected:state.selected,expanded:['tools','config','unassigned'],label:'项目与对话树形导航',density:state.density,onSelect(node){state.selected=node.id;state.activations++;},onUpdate:update,onClear:clear,onReveal(){state.filter=api.store.node(state.selected)?.archived?'archived':'all';$('#tnDemoSearch',host).value='';api.setFilter(predicate,{reveal:state.filter!=='all'});syncFilters();},onRetry(){setScene('normal');}});
  api.setFilter(predicate,{reveal:state.filter!=='all'});update(api.state());
  host.addEventListener('input',e=>{if(e.target.id==='tnDemoSearch')api.setQuery(e.target.value);});
  host.addEventListener('click',e=>{const b=e.target.closest('[data-tn-demo]');if(!b)return;const action=b.dataset.tnDemo;if(action==='clear')clear();if(action==='filter'){state.filter=b.dataset.filter;api.setFilter(predicate,{reveal:state.filter!=='all'});syncFilters();}if(action==='expand-all')api.expandAll();if(action==='collapse-all')api.collapseAll();if(action==='refresh')api.render();if(['expand-all','collapse-all'].includes(action))b.closest('details').open=false;});
  setScene(state.scene);
 }
 function attachTools(parent){
  tool=document.createElement('div');tool.dataset.ctFor='tree-navigation';
  tool.innerHTML='<label>树形目录场景<select id="tnDemoScene"><option value="normal">项目、对话与子 Agent</option><option value="long">长名称与长列表</option><option value="deep">多层子任务</option><option value="empty">目录为空</option><option value="loading">读取中</option><option value="failed">读取失败</option></select></label><label>行密度<select id="tnDemoDensity"><option value="normal">标准</option><option value="compact">紧凑</option></select></label><label>导航宽度<select id="tnDemoWidth"><option value="224">224 px · 窄侧栏</option><option value="280">280 px · 默认</option><option value="340">340 px · 宽侧栏</option></select></label><label><span><input id="tnDemoDescriptions" type="checkbox">显示辅助描述</span></label><p class="pctl-hint">公共导航先打样。选择、搜索、展开和焦点分别保存；不写配置。项目与对话原型已复用；生产接入待实施。</p>';
  parent.appendChild(tool);syncTools();
  tool.addEventListener('change',e=>{if(e.target.id==='tnDemoScene')setScene(e.target.value);if(e.target.id==='tnDemoDensity'){state.density=e.target.value;api?.setDensity(state.density);}if(e.target.id==='tnDemoWidth'){state.width=e.target.value;host?.querySelector('.tn-demo-layout').style.setProperty('--tn-demo-width',state.width+'px');}if(e.target.id==='tnDemoDescriptions'){state.descriptions=e.target.checked;api?.setNodes(nodes());}});
 }
 window.treeNavigationDemo={mount,attachTools,setScene,state:()=>({...state,tree:api?.state()})};
})();
