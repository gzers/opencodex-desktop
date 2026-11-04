/* Neutral fixture for COL-LOCAL-20261014-01; not an extension manager consumer. */
(() => {
 'use strict';
 const $=(s,r=document)=>r.querySelector(s),$$=(s,r=document)=>[...r.querySelectorAll(s)];
 const esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 const button=(text,action,attrs='',cls='ghost')=>'<button type="button" class="btn '+cls+'" data-ic-demo="'+action+'" '+attrs+'>'+text+'</button>';
 const resources=[{id:'design',label:'design-studio',kind:'skills',description:'产品设计与界面检查',version:'v1.8.0'},{id:'development',label:'development-knowledge',kind:'skills',description:'开发知识与实现参考',version:'v2.3.1'},{id:'governance',label:'project-governance',kind:'skills',description:'项目事实与协作资料',version:'v1.5.0'},{id:'audit',label:'skill-audit',kind:'skills',description:'Skill 结构与触发检查',version:'v1.1.3'},{id:'writing',label:'technical-writing',kind:'skills',description:'技术文档与评审记录',version:'v0.4.0'},{id:'filesystem',label:'文件系统',kind:'mcp',description:'限定目录读取工具',version:'stdio'},{id:'docs',label:'开发文档',kind:'mcp',description:'框架文档查询',version:'HTTP'},{id:'browser',label:'浏览器检查',kind:'mcp',description:'网页内容与布局检查',version:'stdio'}];
 const clients=[{id:'codex',label:'Codex',description:'用户级 · 本机',status:'可配置'},{id:'claude',label:'Claude Code',description:'用户级 · 本机',status:'可配置'},{id:'cursor',label:'Cursor',description:'用户级 · 本机',status:'可配置'},{id:'kilo',label:'Kilo',description:'用户级 · 本机',status:'可配置'},{id:'gemini',label:'Gemini CLI',description:'用户级 · 本机',status:'配置冲突',disabled:true},{id:'hermes',label:'Hermes Agent',description:'用户级 · 本机',status:'未发现',disabled:true}];
 let host,tool,leftAPI,rightAPI,views={resource:{current:'design',query:'',rightQuery:'',picks:new Set()},client:{current:'codex',query:'',rightQuery:'',picks:new Set()}};
 let mode='resource',kind='skills',density='normal',scene='normal',saved=new Set(['design|codex','design|claude','development|codex','governance|claude','filesystem|codex']),draft=new Map(),detailOpen=false,origin=null,planVersion=0,flash='',long=false;
 const state=()=>({mode,kind,density,scene,saved:[...saved],draft:[...draft],current:views[mode].current,picks:[...views[mode].picks],query:views[mode].query,rightQuery:views[mode].rightQuery});
 const isResource=()=>mode==='resource';
 const pair=(r,c)=>r+'|'+c;
 const enabled=(r,c)=>draft.has(pair(r,c))?draft.get(pair(r,c)):saved.has(pair(r,c));
 const changes=()=>[...draft].filter(([key,value])=>value!==saved.has(key));
 function setPair(r,c,value){const key=pair(r,c);if(value===saved.has(key))draft.delete(key);else draft.set(key,value);planVersion++;}
 function leftRows(){let list=isResource()?resources.filter(r=>r.kind===kind):clients;
  if(long){list=list.map((r,i)=>({...r,label:i===0?r.label+' · 长名称与多工作空间配置说明':r.label}));const original=[...list];for(let i=0;i<30;i++)list.push({...original[i%original.length],id:'sample-'+i,label:'长列表样例 '+String(i+1).padStart(2,'0'),disabled:true,description:'只读容量样例',status:'样例'});}
  return list.filter(r=>(r.label+' '+r.description).toLowerCase().includes(views[mode].query.toLowerCase()));
 }
 const chosen=()=>{const v=views[mode];return isResource()&&v.picks.size?resources.filter(r=>r.kind===kind&&v.picks.has(r.id)):[(isResource()?resources:clients).find(r=>r.id===v.current)].filter(Boolean);};
 const actions=r=>[{id:'more',label:'更多操作',icon:'⋯'}];
 function rightRows(){
  const objects=chosen();if(!objects.length)return [];
  const list=(isResource()?clients:resources.filter(r=>r.kind===kind));
  return list.filter(r=>(r.label+' '+r.description).toLowerCase().includes(views[mode].rightQuery.toLowerCase())).map(r=>{
   const pairs=objects.map(o=>isResource()?[o.id,r.id]:[r.id,o.id]),n=pairs.filter(([a,b])=>enabled(a,b)).length,value=n===0?false:n===pairs.length?true:'mixed';
   const pending=pairs.some(([a,b])=>draft.has(pair(a,b))),shared=isResource()?['cursor','kilo'].includes(r.id)&&kind==='skills':['cursor','kilo'].includes(objects[0].id)&&kind==='skills';
   return {...r,selected:value,pending,status:r.disabled?r.status:pending?'待应用':value==='mixed'?'部分启用':value===true?'已启用':shared?'共享可用':'未启用',tone:r.disabled?'warning':pending?'warning':'neutral',actions:[{id:'detail',label:'查看详情',icon:'ⓘ'},{id:'toggle',kind:'toggle',label:value==='mixed'?'全部启用':value===true?'停用':'启用',disabled:!!r.disabled,reason:r.disabled?r.status:''}]};
  });
 }
 function html(){return '<div class="ic-workbench"><div class="ic-mode-bar"><div class="ic-type-switch" role="group" aria-label="样例资源类型">'+[['skills','Skills'],['mcp','MCP']].map(([id,text])=>'<button type="button" data-ic-demo="kind" data-value="'+id+'" aria-pressed="'+(kind===id)+'">'+text+'</button>').join('')+'</div><div class="ic-mode-switch" role="group" aria-label="双视图模式">'+[['resource','按扩展'],['client','按平台']].map(([id,text])=>'<button type="button" data-ic-demo="mode" data-value="'+id+'" aria-pressed="'+(mode===id)+'">'+text+'</button>').join('')+'</div></div><div class="ic-panes"><section class="ic-pane" aria-label="左侧对象列表"><header class="ic-pane-head"><div class="ic-pane-caption"><h4 id="icLeftTitle"></h4><span id="icLeftCount"></span></div><input type="search" class="input" id="icLeftSearch" aria-label="搜索左侧对象" placeholder="搜索名称"><div class="ic-context" id="icLeftContext"></div></header><div id="icLeftRows" class="ic-collection"></div></section><section class="ic-pane" aria-label="右侧关系表格"><header class="ic-pane-head"><div class="ic-pane-caption"><h4 id="icRightTitle"></h4><span id="icRightCount"></span></div><input type="search" class="input" id="icRightSearch" aria-label="搜索右侧表格" placeholder="搜索名称或说明"><div class="ic-context" id="icRightContext"></div></header><div id="icRightRows" class="ic-collection"></div></section></div><footer class="ic-footer"><div class="ic-footer-status" id="icDraftStatus" role="status"></div><div class="ic-footer-actions">'+button('撤销修改','revert')+button('预览变更','preview','','primary')+'</div></footer></div><p class="ic-demo-note"><span><i class="ic-demo-dot"></i>底色：当前对象</span><span>☑ 左侧批量选择 · 右侧快速启用</span><span>名称 / ⓘ：详情　⋯：更多操作</span></p>';}
 function renderStatus(){$('#icDraftStatus',host).innerHTML='<strong>待变更 '+changes().length+' 项</strong> · '+esc(flash||'切换视图保留草稿');}
 function update(){
  if(!host)return;host.dataset.density=density;const v=views[mode],left=leftRows(),right=rightRows(),objects=chosen();
  $('#icLeftTitle',host).textContent=isResource()?(kind==='skills'?'Skills':'MCP'):'客户端';$('#icRightTitle',host).textContent=isResource()?'客户端':(kind==='skills'?'Skills':'MCP');
  $('#icLeftCount',host).textContent=left.length+' 项';$('#icRightCount',host).textContent=right.length+' 项';
  $('#icLeftContext',host).innerHTML=isResource()&&v.picks.size?'已勾选 '+v.picks.size+' 项 · '+button('清除','clear-picks'):isResource()?'点击行定位，勾选后批量配置':'点击行切换客户端';
  const current=(isResource()?resources:clients).find(r=>r.id===v.current);
  $('#icRightContext',host).innerHTML='<span>'+(isResource()?'配置目标：':'当前平台：')+'</span><strong>'+esc(objects.length>1?'所选 '+objects.length+' 项扩展':objects[0]?.label||'尚未选择')+'</strong>'+(current&&!left.some(r=>r.id===current.id)?button('当前对象在筛选外','reveal'):'');
  renderStatus();
  $('[data-ic-demo="preview"]',host).disabled=!changes().length;$('[data-ic-demo="revert"]',host).disabled=!changes().length;
  $$('[data-ic-demo="mode"]',host).forEach(b=>b.setAttribute('aria-pressed',String(b.dataset.value===mode)));$$('[data-ic-demo="kind"]',host).forEach(b=>b.setAttribute('aria-pressed',String(b.dataset.value===kind)));
  $('#icLeftSearch',host).value=v.query;$('#icRightSearch',host).value=v.rightQuery;
  leftAPI.update({current:v.current,selection:isResource(),rows:left.map(r=>({...r,selected:v.picks.has(r.id),status:r.status||(r.version||''),actions:actions(r)}))});
  rightAPI.update({rows:right,status:scene==='normal'?'ready':scene==='empty'?'ready':scene,emptyLabel:'没有匹配的关系。'});
  if(scene==='empty')rightAPI.update({rows:[]});
 }
 function modal(title,body,options={}){
  const returnFocus=document.activeElement;origin=returnFocus;detailOpen=true;
  openModal(title,body,options.confirm||'关闭',options.onConfirm||null,{hideNote:true,hideConfirm:!options.onConfirm,cancelLabel:options.onConfirm?'取消':'关闭',prototypeAnnotation:{title:'交互集合公共样例',body:'样例只使用内存 fixture。详情、菜单和勾选不自动激活其他行，也不读写客户端。'},...options});
  const box=$('#modal');box.classList.add('modal-wide');box.setAttribute('role','dialog');box.setAttribute('aria-modal','true');box.setAttribute('aria-labelledby','modalTitle');
  window.__modalOnClose=()=>{detailOpen=false;box.removeAttribute('role');box.removeAttribute('aria-modal');box.removeAttribute('aria-labelledby');if(origin?.isConnected)origin.focus({preventScroll:true});};
 }
 function detail(r){
  if(!r.kind){modal(r.label+' · 详情','<dl class="ic-detail-facts"><dt>名称</dt><dd>'+esc(r.label)+'</dd><dt>说明</dt><dd>'+esc(r.description)+'</dd><dt>状态</dt><dd>'+esc(r.status||'样例')+'</dd></dl>');return;}
  const entry=resources.find(x=>x.id===r.id)||r,returnFocus=document.activeElement;
  const returnHost=returnFocus.closest('#icLeftRows,#icRightRows'),returnAttribute=returnFocus.hasAttribute('data-ic-detail')?'data-ic-detail':returnFocus.hasAttribute('data-ic-action')?'data-ic-action':null,returnValue=returnAttribute&&returnFocus.getAttribute(returnAttribute),returnId=returnFocus.dataset.id;
  const mcp={filesystem:{command:'node',args:'/example/mcp-filesystem.js /example/workspace',env:'WORKSPACE_ROOT',landing:'~/.codex/config.toml → mcp_servers',md:'## 完整配置\n\n本地 stdio 声明，仅展示命令与环境变量键名，不执行。'},docs:{command:'—',args:'—',env:'DOCUMENTATION_TOKEN（仅引用名称）',landing:'~/.codex/config.toml → mcp_servers',md:'## 完整配置\n\n远程端点 https://example.com/mcp；凭据只展示环境变量引用。'},browser:{command:'node',args:'/example/mcp-browser.js',env:'—',landing:'~/.codex/config.toml → mcp_servers'}};
  const definitions={filesystem:{type:'stdio',command:'node',args:['/example/mcp-filesystem.js','/example/workspace'],env:{WORKSPACE_ROOT:'环境变量引用'}},docs:{type:'streamableHttp',url:'https://example.com/mcp',headers:{Authorization:'环境变量引用 · 已掩码'}},browser:{type:'stdio',command:'node',args:['/example/mcp-browser.js']}};
  if(mcp[entry.id])mcp[entry.id].md='## 完整配置\n\n'+String.fromCharCode(96).repeat(3)+'json\n'+JSON.stringify(definitions[entry.id],null,2)+'\n'+String.fromCharCode(96).repeat(3)+'\n\n命令与参数仅为演示数据；环境变量仅展示键名，凭据不展示原值。';
  const getTargets=()=>clients.map(c=>({id:c.id,label:c.label,enabled:enabled(entry.id,c.id),state:c.disabled?c.status:draft.has(pair(entry.id,c.id))?'待应用':enabled(entry.id,c.id)?'已启用':['cursor','kilo'].includes(c.id)&&entry.kind==='skills'?'共享可用':'未启用',disabled:!!c.disabled,reason:c.disabled?c.status:''}));
  detailOpen=true;origin=returnFocus;
  window.PrototypeExtensionDetail.open({kind:entry.kind,name:entry.label,description:entry.description,source:entry.kind==='mcp'?entry.version:'本地源库',version:entry.kind==='skills'?entry.version:'—',updated:'2026-10-30',facts:entry.kind==='mcp'?mcp[entry.id]||{}:window.PrototypeExtensionDetail.sample('skills',entry.label)},
   {getTargets,onToggleTarget(id,value){setPair(entry.id,id,value);flash='';update();},onAction(action){flash=({update:'检查 / 更新',uninstall:'卸载',edit:'编辑',delete:'删除'})[action]+' '+entry.label+' · 公共样例仅演示入口';renderStatus();},onClose(){detailOpen=false;const back=returnFocus?.isConnected?returnFocus:returnAttribute&&returnHost?[...returnHost.querySelectorAll('['+returnAttribute+']')].find(el=>el.getAttribute(returnAttribute)===returnValue&&(!returnId||el.dataset.id===returnId)):null;back?.focus({preventScroll:true});},prototypeAnnotation:{title:'复用既有扩展详情',body:'复用原 Skill / MCP 正文、字段、标题操作及客户端状态。启用只修改本样例草稿，不读文件或执行服务器。'}});
 }

 function rowAction(r,a,anchor){
  if(a==='detail'){detail(r);return;}
  if(a==='toggle'){selectRight([r.id],r.selected!==true);return;}
  if(a==='inspect'){flash='已检查 '+r.label+'，当前对象与勾选未改变。';renderStatus();return;}
  if(a==='more')window.prototypeActionMenu.open({anchor,label:r.label+' 的操作',items:[{label:'查看详情',onSelect:()=>{anchor.focus({preventScroll:true});detail(r);}},{label:'检查此项',onSelect:()=>{flash='已检查 '+r.label+'，当前对象与勾选未改变。';renderStatus();}},{label:'暂不可用的操作',disabled:true,reason:'此样例只读'}]});
 }
 function selectRight(ids,value){flash='';for(const r of rightRows().filter(r=>ids.includes(r.id)&&!r.disabled))for(const o of chosen()){isResource()?setPair(o.id,r.id,value):setPair(r.id,o.id,value);}update();}
 function mount(element){
  if(!element)return;leftAPI?.destroy();rightAPI?.destroy();host=element;host.classList.add('ic-demo');host.innerHTML=html();
  leftAPI=window.PrototypeInteractiveCollection.mount($('#icLeftRows',host),{kind:'list',label:'对象列表',onActivate(r){if(!r.disabled){views[mode].current=r.id;views[mode].picks.clear();flash='';update();}},onDetail:detail,onAction:rowAction,onSelect(r,value){value?views[mode].picks.add(r.id):views[mode].picks.delete(r.id);flash='';update();}});
  rightAPI=window.PrototypeInteractiveCollection.mount($('#icRightRows',host),{kind:'table',label:'分发关系表格',selectionLabel:'启用',selectionAllLabel:'启用当前结果的全部可用项',onDetail:detail,onAction:rowAction,onSelect(r,value){selectRight([r.id],value);},onSelectMany:selectRight,onRetry(){scene='normal';syncTools();update();}});
  host.addEventListener('input',e=>{if(e.target.id==='icLeftSearch'){views[mode].query=e.target.value;update();}if(e.target.id==='icRightSearch'){views[mode].rightQuery=e.target.value;update();}});
  host.addEventListener('click',e=>{const b=e.target.closest('[data-ic-demo]');if(!b)return;const a=b.dataset.icDemo;
   if(a==='mode'){const previous=views[mode];previous.leftScroll=$('#icLeftRows',host).scrollTop;previous.rightScroll=$('#icRightRows',host).scrollTop;mode=b.dataset.value;flash='';update();$('#icLeftRows',host).scrollTop=views[mode].leftScroll||0;$('#icRightRows',host).scrollTop=views[mode].rightScroll||0;}
   if(a==='kind'){kind=b.dataset.value;const r=resources.find(r=>r.kind===kind);if(!resources.some(x=>x.id===views.resource.current&&x.kind===kind))views.resource.current=r.id;views.resource.picks.clear();flash='';update();}
   if(a==='clear-picks'){views[mode].picks.clear();update();}
   if(a==='reveal'){views[mode].query='';update();}
   if(a==='revert'){draft.clear();planVersion++;flash='';update();}
   if(a==='preview')preview();
  });update();
 }
 function preview(){const version=planVersion,items=changes();if(!items.length)return;modal('预览关系变更','<p class="modal-lead">'+items.length+' 项关系变更 · 两种视图共用同一份草稿</p><table class="ic-plan"><thead><tr><th>扩展</th><th>客户端</th><th>操作</th></tr></thead><tbody>'+items.map(([key,value])=>{const [r,c]=key.split('|');return '<tr><td>'+esc(resources.find(x=>x.id===r)?.label)+'</td><td>'+esc(clients.find(x=>x.id===c)?.label)+'</td><td>'+(value?'增加':'移除')+'</td></tr>';}).join('')+'</tbody></table>',{confirm:'应用样例变更',onConfirm(){if(version!==planVersion){flash='草稿已变化，请重新预览。';update();return;}for(const [key,value] of items)value?saved.add(key):saved.delete(key);draft.clear();planVersion++;flash='已应用 '+items.length+' 项样例变更';update();}});}
 function syncTools(){if(!tool)return;$('#icDemoScene',tool).value=scene;$('#icDemoDensity',tool).value=density;$('#icDemoLong',tool).checked=long;window.prototypeSelect?.enhance(tool);}
 function attachTools(parent){tool=document.createElement('div');tool.dataset.ctFor='interactive-collections';tool.innerHTML='<label>数据状态<select id="icDemoScene"><option value="normal">正常</option><option value="empty">空结果</option><option value="loading">读取中</option><option value="failed">读取失败</option></select></label><label>行密度<select id="icDemoDensity"><option value="normal">标准</option><option value="compact">紧凑</option></select></label><label><span><input type="checkbox" id="icDemoLong">长名称与长列表</span></label><p class="pctl-hint">公共样式探索 · 版本未定。固定左列表、右表格；当前对象、批量勾选、关系勾选和详情 / 操作分别处理。尚未接入扩展业务。</p>';parent.append(tool);syncTools();tool.addEventListener('change',e=>{if(e.target.id==='icDemoScene')scene=e.target.value;if(e.target.id==='icDemoDensity')density=e.target.value;if(e.target.id==='icDemoLong')long=e.target.checked;update();});}
 document.addEventListener('keydown',e=>{if(!detailOpen||e.key!=='Tab')return;const items=$$('button,input,a[href],[tabindex="0"]',$('#modal')).filter(el=>!el.disabled&&!el.hidden&&el.getClientRects().length);if(e.shiftKey&&document.activeElement===items[0]){e.preventDefault();items.at(-1)?.focus();}else if(!e.shiftKey&&document.activeElement===items.at(-1)){e.preventDefault();items[0]?.focus();}},true);
 new MutationObserver(()=>{if(document.documentElement.dataset.route!=='components'&&detailOpen)closeModal();}).observe(document.documentElement,{attributes:true,attributeFilter:['data-route']});
 window.interactiveCollectionDemo={mount,attachTools,state};
})();
