/* COL-LOCAL-20261014-01 · Multi-platform extension mock, memory only. */
(() => {
 'use strict';
 const $=(s,r=document)=>r.querySelector(s), $$=(s,r=document)=>[...r.querySelectorAll(s)];
 const clone=v=>JSON.parse(JSON.stringify(v));
 const esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 const route=$('#route-extensions');if(!route)return;
 const legacy=document.createElement('div');legacy.id='extensionLegacyContent';
 while(route.firstChild)legacy.append(route.firstChild);route.append(legacy);
 const host=document.createElement('div');host.id='extensionManagerContent';host.className='xp-page';host.hidden=true;route.append(host);
 const btn=(label,action,attrs='',primary=false)=>'<button type="button" class="btn '+(primary?'':'ghost')+'" data-xp="'+action+'" '+attrs+'>'+label+'</button>';
 const option=(value,label,selected)=>'<option value="'+esc(value)+'" '+(value===selected?'selected':'')+'>'+esc(label)+'</option>';
 const names=[['codex','Codex'],['claude','Claude Code'],['gemini','Gemini CLI'],['grok','Grok Build'],['opencode','OpenCode'],['hermes','Hermes Agent'],['cursor','Cursor'],['kilo','Kilo'],['cline','Cline CLI'],['claudeDesktop','Claude Desktop'],['pi','Pi'],['omp','OMP / Oh My Pi'],['openclaw','OpenClaw'],['kimi','Kimi Code'],['gajae','Gajae Code / gjc'],['dsh','DeepSeek Harness'],['mcode','MiniMax Code'],['zcode','ZCode'],['commandcode','Command Code'],['prime','Prime Agent'],['aside','Aside'],['raycast','Raycast'],['omo','omo / LazyCodex'],['droid','Factory Droid']];
 const dirs={codex:'~/.codex/skills',claude:'~/.claude/skills',gemini:'~/.gemini/skills',grok:'~/ExampleProject/.agents/skills',opencode:'~/.config/opencode/skills',hermes:'~/.hermes/skills',cursor:'~/.cursor/skills',kilo:'~/.kilo/skills',cline:'~/.cline/skills'};
 const paths={codex:'~/.codex/config.toml',claude:'~/.claude.json',gemini:'~/.gemini/settings.json',grok:'~/.grok/user-settings.json',opencode:'~/.config/opencode/opencode.json',hermes:'~/.hermes/config.yaml',cursor:'~/.cursor/mcp.json',kilo:'~/.config/kilo/kilo.jsonc',cline:'~/.cline/data/settings/cline_mcp_settings.json'};
 const catalog=names.map(([id,name],i)=>({id,name,installed:i<9||['pi','openclaw','aside'].includes(id),evidence:i<6?'既有六平台契约':i<9?'官方扩展文档已核实':'扩展能力待核实',cap:i<9?'ready':'unknown',scope:id==='grok'?'项目级 · 待确认写入范围':'用户级',skills:dirs[id]||null,mcp:paths[id]||null}));
 const seed=[
  {id:'design',kind:'skills',name:'design-studio',description:'产品设计、界面原型与设计检查；按任务加载必要参考。',source:'本地源库',version:'v1.8.0',selected:['codex','claude','gemini','opencode'],projections:{codex:'synced',claude:'synced',gemini:'failed',opencode:'stale'}},
  {id:'development',kind:'skills',name:'development-knowledge',description:'编程语言、框架与实现模式的开发参考。',source:'本地源库',version:'v2.3.1',selected:['codex','claude','gemini'],projections:{codex:'synced',claude:'synced',gemini:'synced'}},
  {id:'governance',kind:'skills',name:'project-governance',description:'已绑定项目的需求、事实、协作资料与回写管理。',source:'本地源库',version:'v1.5.0',selected:['codex','claude'],projections:{codex:'synced',claude:'synced'}},
  {id:'management',kind:'skills',name:'project-management',description:'推进项目、协调阶段、验证并整理交付状态。',source:'本地源库',version:'v1.1.0',selected:['codex','hermes'],projections:{codex:'synced',hermes:'synced'}},
  {id:'audit',kind:'skills',name:'skill-audit',description:'检查 Skill 的触发范围、重复措辞和维护成本。',source:'Git 仓库',version:'v1.1.3',selected:['codex'],projections:{codex:'synced'}},
  {id:'docs',kind:'skills',name:'document-review',description:'整理文档结构与评审意见，保留来源和决定。',source:'Git 仓库',version:'v0.3.0',selected:[],projections:{}},
  {id:'filesystem',kind:'mcp',name:'文件系统',description:'为 Agent 提供限定目录的文件读取工具。',source:'受管定义',version:'stdio',selected:['codex','claude','gemini'],projections:{codex:'synced',claude:'synced',gemini:'failed'},definition:{command:'node',args:['/example/mcp-filesystem.js','/example/workspace'],env:['WORKSPACE_ROOT']}},
  {id:'documentation',kind:'mcp',name:'开发文档',description:'查询框架与库的参考文档。',source:'受管定义',version:'HTTP',selected:['codex','opencode'],projections:{codex:'synced',opencode:'stale'},definition:{url:'https://example.com/mcp',headers:['Authorization（环境变量引用）']}},
  {id:'browser',kind:'mcp',name:'浏览器工具',description:'检查网页内容与界面状态。',source:'受管定义',version:'stdio',selected:['codex'],projections:{codex:'synced'},definition:{command:'node',args:['/example/mcp-browser.js'],env:[]}},
  {id:'git',kind:'mcp',name:'Git 只读查询',description:'查看仓库状态与提交记录。',source:'受管定义',version:'stdio',selected:[],projections:{},definition:{command:'node',args:['/example/mcp-git.js'],env:[]}}
 ];
 let resources=clone(seed),tab=new URLSearchParams(location.hash.split('?')[1]||'').get('tab')||'skills',query='',clientFilter='',platformOpen=false,platformQuery='',status='all',picks=new Set(),clientId=null,clientQuery='',clientStatus='all',capacity='24',scene='normal',resultScene='partial',dialog=null,notice='';
 if(!['skills','mcp','clients'].includes(tab))tab='skills';
 const active=()=>document.documentElement.dataset.designScope==='extensions';
 const label={synced:'已同步',failed:'同步失败',stale:'待更新',none:'未投影',shared:'经共享目录可用',unknown:'尚未检查'};
 function targets(){const base=capacity==='6'?catalog.slice(0,6):catalog;return capacity!=='50'?base:[...base,...Array.from({length:26},(_,i)=>({...catalog[i%6],id:'instance-'+i,name:catalog[i%6].name+' · 容量样例 '+(i+1),installed:true,cap:'unknown',evidence:'容量样例，未核实真实实例',scope:'模拟实例 · 待核实',skills:null,mcp:null}))];}
 const target=id=>targets().find(t=>t.id===id)||catalog.find(t=>t.id===id);
 const resource=id=>resources.find(r=>r.id===id);
 const shared=(r,t)=>r.kind==='skills'&&['cursor','kilo'].includes(t.id);
 const available=t=>t.cap==='ready'&&t.installed&&t.scope==='用户级'&&!(scene==='unsupported'&&t.id==='hermes');
 function reason(t){if(scene==='unsupported'&&t.id==='hermes')return '当前示例不支持此扩展类型';if(t.cap!=='ready')return '待核实扩展适配';if(!t.installed)return '未发现客户端';if(t.scope!=='用户级')return '项目级写入范围未确认';return '';}
 const path=(r,t)=>r.kind==='skills'?t.skills?(t.skills+'/'+r.name):'落点待核实':t.mcp||'落点待核实';
 function observed(r,t){return shared(r,t)?'shared':r.projections[t.id]||'none';}
 const issue=r=>Object.values(r.projections).some(v=>['failed','stale'].includes(v));
 function stats(r){return {chosen:r.selected.length,synced:Object.values(r.projections).filter(v=>v==='synced').length,failed:Object.values(r.projections).filter(v=>v==='failed').length,stale:Object.values(r.projections).filter(v=>v==='stale').length};}
 function annotation(){return '<div class="proto-annotation" id="ann-xp-scope" data-annotation-number="01" data-annotation-title="多平台扩展 · 版本未定" data-annotation-category="COL-LOCAL-20261014-01"><button type="button" class="annotation-marker" aria-expanded="false" aria-controls="annotationPopover" aria-label="原型注释 01：多平台扩展 · 版本未定"><span>原型</span><span>01</span><span>＋</span></button><div class="annotation-source" hidden><p>本包只负责 Skills / MCP 资源摘要、目标管理、客户端页与相关状态。24 项目录是平台调查样例，新增适配为内存模拟；不写文件、不启动 MCP。模型、对话、同步和备份增量各归原包。</p></div></div>';}
 function filtered(){return resources.filter(r=>r.kind===tab&&(!query||(r.name+' '+r.description).toLowerCase().includes(query.toLowerCase()))&&(!clientFilter||r.selected.includes(clientFilter)||shared(r,{id:clientFilter}))&&(status==='all'||status==='issues'&&issue(r)||status==='selected'&&r.selected.length));}
 function render(){
  if(!active())return;
  const focus=document.activeElement,focusId=focus?.id,start=focus?.selectionStart;
  host.innerHTML='<nav class="xp-tabs" data-page-tabs role="tablist" aria-label="扩展分区">'+[['skills','Skills'],['mcp','MCP'],['clients','客户端']].map(([id,name])=>'<button type="button" role="tab" data-xp="tab" data-id="'+id+'" aria-selected="'+(tab===id)+'" tabindex="'+(tab===id?'0':'-1')+'">'+name+'</button>').join('')+'</nav>'+annotation()+(notice?'<div class="xp-note" role="status">'+esc(notice)+'</div>':'')+(tab==='clients'?renderClients():renderResources());
  $$('#xpVisiblePick').forEach(el=>{const rows=filtered();el.indeterminate=rows.some(r=>picks.has(r.id))&&!rows.every(r=>picks.has(r.id));});
  window.prototypeSelect?.enhance();window.prototypeStickyTabs?.refresh();window.prototypePanel?.refresh();
  if(focusId){const replacement=$('#'+focusId,host);if(replacement){replacement.focus();if(start!=null&&replacement.type==='search')replacement.setSelectionRange(start,start);}}
 }
 function renderResources(){
  const list=filtered(),count=resources.filter(r=>r.kind===tab).length,chosen=resources.filter(r=>r.kind===tab&&picks.has(r.id)),heading=tab==='skills'?'Skills 管理':'MCP 管理';
  const filterOptions=[{id:'',name:'全部客户端'},...targets()].filter(t=>t.name.toLowerCase().includes(platformQuery.toLowerCase()));
  const controls='<div class="xp-toolbar"><input class="input" id="xpSearch" type="search" placeholder="搜索'+(tab==='skills'?' Skills 名称或说明':' MCP 名称或说明')+'" aria-label="搜索扩展资源" value="'+esc(query)+'"><div class="xp-filter-wrap"><button class="btn ghost" id="xpPlatformTrigger" data-xp="platform-open" aria-label="按客户端筛选资源：'+esc(target(clientFilter)?.name||'全部客户端')+'" aria-haspopup="listbox" aria-expanded="'+platformOpen+'" aria-controls="xpPlatformResults">'+esc(target(clientFilter)?.name||'全部客户端')+' ▾</button><div class="xp-platform-popover" '+(platformOpen?'':'hidden')+'><input class="input" id="xpPlatformSearch" type="search" aria-label="搜索平台筛选项" placeholder="搜索客户端" value="'+esc(platformQuery)+'"><div id="xpPlatformResults" role="listbox" aria-label="平台筛选项">'+(filterOptions.length?filterOptions.map(t=>'<button type="button" role="option" aria-selected="'+(t.id===clientFilter)+'" data-xp="platform" data-id="'+t.id+'">'+esc(t.name)+'</button>').join(''):'<div class="xp-muted">没有匹配的客户端。</div>')+'</div></div></div></div>';
  const chips='<div class="xp-actions">'+[['all','全部 '+count],['issues','有异常 '+resources.filter(r=>r.kind===tab&&issue(r)).length],['selected','已配置目标']].map(([id,text])=>'<button class="xp-chip '+(status===id?'active':'')+'" data-xp="filter" data-id="'+id+'">'+text+'</button>').join('')+'</div>';
  const batch=chosen.length?'<div class="xp-batch"><strong>已选 '+chosen.length+' 项'+(chosen.some(r=>!list.includes(r))?' · 包含筛选外资源':'')+'</strong>'+btn('增加目标','batch','data-mode="add"')+btn('移除目标','batch','data-mode="remove"')+btn('清除选择','clear-picks')+'</div>':'';
  const tablehead='<div class="xp-table-head"><input type="checkbox" id="xpVisiblePick" aria-label="选择当前结果的全部资源" '+(list.length&&list.every(r=>picks.has(r.id))?'checked':'')+(list.length?'':'disabled')+'><span>名称与说明</span><span class="xp-heading-target">同步目标</span><span class="xp-heading-actions">操作</span></div>';
  const empty=scene==='loading'?'正在读取资源列表…':scene==='error'?'无法读取资源列表。请重试。':scene==='empty'?'源库暂时没有资源。':list.length?'':'没有匹配的资源。';
  return '<div class="xp-head"><div><h2>'+heading+'</h2><p class="xp-muted">'+(tab==='skills'?'维护源库，按需要分发到客户端。':'维护服务器定义，分别配置客户端目标。')+'</p></div><div class="xp-actions">'+btn('客户端','tab','data-id="clients"')+btn('设置','settings')+'</div></div>'+controls+chips+batch+tablehead+'<div class="xp-list">'+(empty?'<div class="xp-empty" role="status">'+empty+(scene==='error'?'<div style="margin-top:12px">'+btn('重试','scene','data-id="normal"')+'</div>':'')+'</div>':list.map(row).join(''))+'</div><div class="xp-foot"><span>'+(!empty?list.length:0)+' / '+count+' 项</span><span>目标选择和实际同步分别显示</span></div>';
 }
 function row(r){const s=stats(r);return '<article class="xp-resource" data-resource-id="'+r.id+'"><input class="xp-resource-check" type="checkbox" data-xp-pick="'+r.id+'" aria-label="选择 '+esc(r.name)+'" '+(picks.has(r.id)?'checked':'')+'><div class="xp-resource-info"><button class="xp-resource-name" data-xp="resource" data-id="'+r.id+'">'+esc(r.name)+'</button><p>'+esc(r.description)+'</p><span class="xp-meta">'+esc(r.source)+' · '+esc(r.version)+'</span></div><button class="xp-summary" data-xp="targets" data-id="'+r.id+'" aria-label="管理 '+esc(r.name)+' 的同步目标">已选 '+s.chosen+' 个 · 已同步 '+s.synced+' 个<span class="'+(s.failed?'xp-warn':'')+'">'+(s.failed||s.stale?[s.failed?'失败 '+s.failed+' 个':'',s.stale?'待更新 '+s.stale+' 个':''].filter(Boolean).join(' · '):s.chosen?r.selected.slice(0,2).map(id=>target(id)?.name||id).join('、'):'尚未选择目标')+'</span></button><div class="xp-actions">'+btn('管理','targets','data-id="'+r.id+'" aria-label="管理 '+esc(r.name)+' 的目标"')+'</div></article>';}
 function renderClients(){
  if(clientId)return renderClientDetail(target(clientId));
  const list=targets().filter(t=>(t.name+' '+t.id).toLowerCase().includes(clientQuery.toLowerCase())&&(clientStatus==='all'||clientStatus==='installed'&&t.installed||clientStatus==='unknown'&&t.cap==='unknown'||clientStatus==='issues'&&resources.some(r=>['failed','stale'].includes(r.projections[t.id]))));
  return '<div class="xp-head"><div><h2>客户端</h2><p class="xp-muted">查看平台能力、配置落点和已分发的资源。</p></div>'+btn('发现客户端','discover')+'</div><div class="xp-toolbar"><input class="input" id="xpClientSearch" type="search" aria-label="搜索客户端" placeholder="搜索客户端名称" value="'+esc(clientQuery)+'"><select class="select" id="xpClientStatus" aria-label="客户端状态筛选">'+[['all','全部 '+targets().length+' 项'],['installed','已发现'],['issues','有异常'],['unknown','待核实适配']].map(([id,name])=>option(id,name,clientStatus)).join('')+'</select></div><div class="xp-client-head"><span>客户端</span><span>扩展能力</span><span class="xp-client-count">受管资源</span><span>状态</span></div><div class="xp-list">'+(list.length?list.map(t=>'<article class="xp-client-row"><div class="xp-client-name"><span class="xp-monogram" aria-hidden="true">'+esc(t.name.slice(0,2))+'</span><div><button class="xp-resource-name" data-xp="client" data-id="'+t.id+'">'+esc(t.name)+'</button><div class="xp-meta">'+esc(t.scope)+'</div></div></div><div class="xp-cap">Skills · '+(t.cap==='ready'?'文档有依据':'待核实')+'<br>MCP · '+(t.cap==='ready'?'文档有依据':'待核实')+'</div><span class="xp-client-count">'+resources.filter(r=>r.selected.includes(t.id)).length+' 项</span><span class="xp-cap">'+(reason(t)||'可管理（演示）')+'<br>'+(t.installed?'本机样例已发现':'未发现')+'</span></article>').join(''):'<div class="xp-empty">没有匹配的客户端。</div>')+'</div>';
 }
 function renderClientDetail(t){
  if(!t){clientId=null;return renderClients();}
  const own=resources.filter(r=>r.selected.includes(t.id)),visible=resources.filter(r=>shared(r,t));
  return '<div class="xp-detail">'+btn('← 返回客户端','client-back')+'<div class="xp-head"><div><h2>'+esc(t.name)+'</h2><p class="xp-muted">'+esc(t.evidence)+' · '+(t.installed?'样例已发现':'样例未发现')+'</p></div>'+btn('配置说明','client-info','data-id="'+t.id+'"')+'</div><dl class="xp-facts"><dt>范围</dt><dd>'+esc(t.scope)+'</dd><dt>Skills</dt><dd>'+esc(t.skills||'搜索目录待核实')+'</dd><dt>MCP</dt><dd>'+esc(t.mcp||'配置节点待核实')+'</dd><dt>适配状态</dt><dd>'+esc(reason(t)||'可管理（内存模拟；生产适配需独立验收）')+'</dd></dl><div class="xp-actions">'+btn('查看 Skills','client-resources','data-kind="skills" data-id="'+t.id+'"')+btn('查看 MCP','client-resources','data-kind="mcp" data-id="'+t.id+'"')+'</div><div class="xp-muted">受管资源 '+own.length+' 项'+(visible.length?' · 另有 '+visible.length+' 项经共享目录可用':'')+'</div><div class="xp-list">'+[...new Set([...own,...visible])].map(r=>'<div class="xp-client-row" style="grid-template-columns:minmax(0,1fr) 130px 100px"><button class="xp-resource-name" data-xp="resource" data-id="'+r.id+'">'+esc(r.name)+'</button><span class="xp-cap">'+(r.kind==='skills'?'Skills':'MCP')+' · '+label[observed(r,t)]+'</span>'+btn('管理目标','targets','data-id="'+r.id+'"')+'</div>').join('')+(!own.length&&!visible.length?'<div class="xp-empty">没有受管资源。</div>':'')+'</div></div>';
 }
 function markDialog(){const el=$('#modal');el.setAttribute('role','dialog');el.setAttribute('aria-modal','true');el.setAttribute('aria-labelledby','modalTitle');}
 function clearDialog(){const el=$('#modal');el.removeAttribute('role');el.removeAttribute('aria-modal');el.removeAttribute('aria-labelledby');$('#modalCancel').disabled=false;$('#modalConfirm').disabled=false;}
 function showInfo(title,body){openModal(title,body,'关闭',null,{hideNote:true,cancelLabel:'关闭',hideConfirm:true,prototypeAnnotation:{title:'本协作包的只读详情',body:'COL-LOCAL-20261014-01。这里是虚构数据或文档中的示例路径，不读取真实文件或启动服务器。'}});$('#modal').classList.add('modal-wide');markDialog();window.__modalOnClose=clearDialog;}
 function showResource(r){showInfo(r.name,'<p class="xp-desc">'+esc(r.description)+'</p><dl class="xp-facts"><dt>类型</dt><dd>'+esc(r.kind==='skills'?'Skills':'MCP')+'</dd><dt>来源</dt><dd>'+esc(r.source)+'</dd><dt>源目录</dt><dd>'+esc(r.kind==='skills'?'~/.agents/skills/'+r.name:'管理器统一 MCP 定义')+'</dd></dl>'+(r.definition?'<pre class="xp-code">'+esc(JSON.stringify(r.definition,null,2))+'</pre>':'<p class="xp-muted">源资源与专用投影分别管理；Cursor / Kilo 可能直接发现共享目录。</p>')+'<div class="xp-actions" style="margin-top:16px">'+btn('管理目标','detail-targets','data-id="'+r.id+'"')+'</div>');}
 function openTargets(ids,mode='single'){
  const rows=ids.map(resource).filter(Boolean);if(!rows.length)return;
  const origin=document.activeElement;
  dialog={ids:rows.map(r=>r.id),kind:rows[0].kind,mode,step:'select',draft:new Set(mode==='single'?rows[0].selected:[]),query:'',filter:'available',plan:[],results:[],origin};
  openModal(mode==='single'?rows[0].name+' · 同步目标':(mode==='add'?'增加目标':'移除目标')+' · '+rows.length+' 项资源','<div id="xpDialogContent"></div>','预览变更',advance,{hideNote:true,confirmKeepsOpen:true,prototypeAnnotation:{title:'目标草稿与实际结果',body:'COL-LOCAL-20261014-01。选择只修改草稿，预览后提交才改变内存投影；取消不修改资源。失败 / 共享目录可用不等于完全禁用。'}});
  $('#modal').classList.add('xp-modal');markDialog();window.__modalOnClose=()=>{$('#modal').classList.remove('xp-modal');clearDialog();dialog=null;const back=origin?.isConnected?origin:$('[data-xp="targets"][data-id="'+rows[0].id+'"]',host);back?.focus();};
  renderDialog();
 }
 function dialogTargets(){return targets().filter(t=>(t.name+' '+t.id).toLowerCase().includes(dialog.query.toLowerCase())&&(dialog.filter==='all'||dialog.filter==='selected'&&dialog.draft.has(t.id)||dialog.filter==='available'&&(available(t)||dialog.draft.has(t.id))));}
 function renderDialog(){
  if(!dialog)return;
  const d=dialog,rows=d.ids.map(resource),selectedCount=d.draft.size,focus=document.activeElement,id=focus?.id,pos=focus?.selectionStart,body=$('#xpDialogContent');
  if(d.step==='select'){
   const list=dialogTargets(),allowed=list.filter(t=>available(t)||d.mode==='single'&&d.draft.has(t.id));
   const bulkChecked=allowed.length&&allowed.every(t=>d.draft.has(t.id));
   body.innerHTML='<div class="xp-dialog"><div class="xp-dialog-tools"><input class="input" id="xpTargetSearch" type="search" aria-label="搜索同步目标" placeholder="搜索客户端或实例" value="'+esc(d.query)+'"><select class="select" id="xpTargetFilter" aria-label="同步目标筛选">'+[['available','已发现与已选'],['selected','只看已选'],['all','全部客户端']].map(([key,text])=>option(key,text,d.filter)).join('')+'</select></div><div class="xp-foot"><label><input type="checkbox" id="xpTargetAll" '+(bulkChecked?'checked':'')+(allowed.length?'':'disabled')+'> 选择当前结果的 '+allowed.length+' 个可操作目标</label><span>全部已选 '+selectedCount+' 个</span></div>'+(d.mode!=='single'?'<div class="xp-muted">'+(d.mode==='add'?'增加所选目标，保留资源已有目标。':'仅移除所选目标的受管投影，保留源资源和其他目标。')+'</div>':'')+'<div class="xp-scroll">'+(list.length?list.map(t=>{const chosen=rows.filter(r=>r.selected.includes(t.id)).length,blocked=reason(t),can=available(t)||d.mode==='single'&&d.draft.has(t.id),state=d.mode==='single'?label[observed(rows[0],t)]:chosen===0?'所选资源均未选择':chosen===rows.length?'所选资源均已选择':'部分资源已选择';return '<div class="xp-target"><input type="checkbox" id="xp-target-'+t.id+'" data-xp-target="'+t.id+'" aria-label="选择同步到 '+esc(t.name)+'" '+(d.draft.has(t.id)?'checked':'')+(can?'':'disabled')+'><label for="xp-target-'+t.id+'"><strong>'+esc(t.name)+'</strong><code>'+esc(path(rows[0],t))+'</code><small>'+esc(t.scope)+' · '+esc(t.evidence)+'</small></label><div class="xp-target-state">'+esc(state)+'<small class="'+(blocked?'xp-warn':'')+'">'+esc(blocked||'可管理（演示）')+'</small></div></div>';}).join(''):'<div class="xp-empty">没有匹配的目标；已选 '+selectedCount+' 个仍保留。</div>')+'</div></div>';
   const all=$('#xpTargetAll');if(all)all.indeterminate=allowed.some(t=>d.draft.has(t.id))&&!bulkChecked;
   $('#modalConfirm').textContent='预览变更';$('#modalConfirm').disabled=d.mode!=='single'&&!selectedCount;
  }else if(d.step==='preview'){
   body.innerHTML='<div class="xp-dialog"><div class="xp-foot"><strong>核对本次变更</strong>'+btn('返回选择','dialog-back')+'</div><div class="xp-muted">'+rows.length+' 项资源 · '+new Set(d.plan.map(p=>p.target)).size+' 个目标 · '+new Set(d.plan.map(p=>p.path)).size+' 个实际落点</div><div class="xp-scroll">'+(d.plan.length?d.plan.map(p=>'<div class="xp-plan-row"><span>'+({add:'新增',remove:'移除',update:'更新'})[p.action]+'</span><div><strong>'+esc(resource(p.resource).name)+' → '+esc(target(p.target).name)+'</strong><code>'+esc(p.path)+'</code></div><span class="xp-target-state">'+(p.blocked?'<span class="xp-warn">'+esc(p.blocked)+'</span>':p.shared?(p.action==='remove'?'共享目录仍可用':'建立专用投影；共享目录已可用'):'待提交')+'</span></div>').join(''):'<div class="xp-empty">目标选择没有变化，现有投影无需更新。</div>')+'</div></div>';
   $('#modalConfirm').textContent=d.plan.length?'确认应用':'完成';$('#modalConfirm').disabled=!!d.plan.length&&d.plan.every(p=>p.blocked);
  }else if(d.step==='running'){
   body.innerHTML='<div class="xp-dialog"><div class="xp-empty" role="status">正在应用 '+d.plan.length+' 项变更…<p class="xp-muted">本次资源与目标已固定。</p></div></div>';
   $('#modalConfirm').textContent='正在应用';$('#modalConfirm').disabled=true;
  }else{
   const failed=d.results.filter(p=>p.status==='failed');
   body.innerHTML='<div class="xp-dialog"><div class="xp-foot"><strong>'+ (failed.length?'部分完成':'全部完成')+'</strong><span>成功 '+(d.results.length-failed.length)+' 项 · 失败 '+failed.length+' 项</span></div><div class="xp-scroll">'+d.results.map(p=>'<div class="xp-plan-row"><span class="'+(p.status==='failed'?'xp-warn':'xp-status')+'">'+(p.status==='failed'?'失败':'完成')+'</span><div><strong>'+esc(resource(p.resource).name)+' → '+esc(target(p.target).name)+'</strong><code>'+esc(p.path)+'</code></div><span class="xp-target-state">'+esc(p.message)+'</span></div>').join('')+'</div>'+(failed.length?'<div class="xp-actions">'+btn('重试失败项','retry')+'</div>':'')+'<p class="xp-muted">已更新资源与客户端视图。配置投影完成后，客户端是否需重载或新会话仍按实际适配器判断。</p></div>';
   $('#modalConfirm').textContent='完成';$('#modalConfirm').disabled=false;
  }
  $('#modalCancel').disabled=d.step==='running';$('#modalCancel').textContent=d.step==='result'?'关闭':'取消';
  window.prototypeSelect?.enhance();if(id){const replacement=$('#'+id,body);if(replacement){replacement.focus();if(pos!=null&&replacement.type==='search')replacement.setSelectionRange(pos,pos);}}
 }
 function plan(){
  const d=dialog,list=[];
  for(const r of d.ids.map(resource)){
   const next=d.mode==='single'?[...d.draft]:d.mode==='add'?[...new Set([...r.selected,...d.draft])]:r.selected.filter(id=>!d.draft.has(id));
   for(const id of new Set([...r.selected,...next])){
    const t=target(id);if(!t)continue;
    const before=r.selected.includes(id),after=next.includes(id),action=!before&&after?'add':before&&!after?'remove':after&&['failed','stale'].includes(r.projections[id])?'update':null;
    if(action)list.push({resource:r.id,target:id,action,path:path(r,t),blocked:action==='remove'?'':reason(t)||(scene==='conflict'&&id==='hermes'?'目标存在外部修改，需先处理冲突':''),shared:shared(r,t)});
   }
  }
  return list;
 }
 function applyPlan(p,retry=false){
  const r=resource(p.resource);if(p.blocked)return {...p,status:'failed',message:p.blocked};
  if(!retry&&(dialog.frozenResultScene||resultScene)==='partial'&&p===dialog.plan.find(x=>!x.blocked)){
   if(p.action!=='remove'&&!r.selected.includes(p.target))r.selected.push(p.target);r.projections[p.target]='failed';return {...p,status:'failed',message:'示例写入失败，可单独重试'};
  }
  if(p.action==='remove'){r.selected=r.selected.filter(id=>id!==p.target);delete r.projections[p.target];}
  else{if(!r.selected.includes(p.target))r.selected.push(p.target);r.projections[p.target]='synced';}
   return {...p,status:'done',message:p.action==='remove'?(p.shared?'受管投影已移除；共享目录仍可用':'受管投影已移除'):p.shared?'专用投影已建立；共享目录仍可用':'配置已投影；下次加载生效'};
 }
 function advance(){if(!dialog)return;if(dialog.step==='select'){dialog.plan=plan();dialog.step='preview';renderDialog();}else if(dialog.step==='preview'){if(!dialog.plan.length){closeModal();return;}const current=dialog;current.frozenResultScene=resultScene;current.step='running';renderDialog();setTimeout(()=>{if(dialog!==current)return;current.results=current.plan.map(p=>applyPlan(p));current.step='result';render();renderDialog();},350);}else if(dialog.step!=='running')closeModal();}
 function setTab(value){tab=value;platformOpen=false;picks.clear();notice='';if(value!=='clients')clientId=null;const hash='#extensions?tab='+tab;history.replaceState(null,'',location.pathname+location.search+hash);render();requestAnimationFrame(()=>$('[data-xp="tab"][data-id="'+tab+'"]',host)?.focus());}
 function closePlatform(){platformOpen=false;const pop=$('.xp-platform-popover',host);if(pop)pop.hidden=true;$('#xpPlatformTrigger',host)?.setAttribute('aria-expanded','false');}
 function refreshScope(){const yes=active();legacy.hidden=yes;host.hidden=!yes;if(document.documentElement.dataset.route==='extensions')$('#routeSubtitle').textContent=yes?'维护 Skills 与 MCP，按客户端管理分发目标与同步结果。':'集中发现与同步本机 Skills 和 MCP 服务器；目标客户端按图标开关。';if(yes)render();window.prototypePanel?.refresh();}
 document.addEventListener('click',e=>{
  if(platformOpen&&!e.target.closest?.('.xp-filter-wrap'))closePlatform();
  const el=e.target.closest?.('[data-xp]');if(!el)return;
  const a=el.dataset.xp,id=el.dataset.id;
  if(a==='tab')setTab(id);
  else if(a==='filter'){status=id;render();}
  else if(a==='platform-open'){platformOpen=!platformOpen;platformQuery='';render();if(platformOpen)$('#xpPlatformSearch')?.focus();}
  else if(a==='platform'){clientFilter=id;platformOpen=false;render();$('#xpPlatformTrigger')?.focus();}
  else if(a==='targets')openTargets([id]);
  else if(a==='batch')openTargets(resources.filter(r=>r.kind===tab&&picks.has(r.id)).map(r=>r.id),el.dataset.mode);
  else if(a==='clear-picks'){picks.clear();render();}
  else if(a==='resource')showResource(resource(id));
  else if(a==='detail-targets'){closeModal();openTargets([id]);}
  else if(a==='client'){clientId=id;render();}
  else if(a==='client-back'){clientId=null;render();}
  else if(a==='client-resources'){clientFilter=id;query='';status='all';setTab(el.dataset.kind);}
  else if(a==='client-info'){const t=target(id);showInfo(t.name+' · 配置说明','<dl class="xp-facts"><dt>能力依据</dt><dd>'+esc(t.evidence)+'</dd><dt>Skills</dt><dd>'+esc(t.skills||'待核实')+'</dd><dt>MCP</dt><dd>'+esc(t.mcp||'待核实')+'</dd><dt>范围</dt><dd>'+esc(t.scope)+'</dd></dl><p class="xp-muted">本轮只展示示例路径；客户端专属设置后续复用设置页数据，生产适配尚未完成。</p>');}
  else if(a==='discover'){notice='已发现 '+targets().filter(t=>t.installed).length+' 个样例目标；没有自动新增资源分发。';render();}
  else if(a==='settings'){setRoute('settings?section=extensions');showSettingsSection('extensions');}
  else if(a==='dialog-back'){dialog.step='select';renderDialog();}
  else if(a==='retry'){dialog.results=dialog.results.map(p=>p.status==='failed'?applyPlan(p,true):p);render();renderDialog();}
  else if(a==='scene'){scene=id;const control=$('#xpScene');if(control)control.value=scene;window.prototypeSelect?.enhance();render();}
 });
 document.addEventListener('change',e=>{
  const el=e.target;
  if(el.dataset.xpPick){el.checked?picks.add(el.dataset.xpPick):picks.delete(el.dataset.xpPick);render();}
  else if(el.id==='xpVisiblePick'){for(const r of filtered())el.checked?picks.add(r.id):picks.delete(r.id);render();}
  else if(el.id==='xpClientFilter'){clientFilter=el.value;render();}
  else if(el.id==='xpClientStatus'){clientStatus=el.value;render();}
  else if(el.dataset.xpTarget&&dialog){el.checked?dialog.draft.add(el.dataset.xpTarget):dialog.draft.delete(el.dataset.xpTarget);renderDialog();}
  else if(el.id==='xpTargetFilter'&&dialog){dialog.filter=el.value;renderDialog();}
  else if(el.id==='xpTargetAll'&&dialog){for(const t of dialogTargets().filter(t=>available(t)||dialog.mode==='single'&&dialog.draft.has(t.id)))el.checked?dialog.draft.add(t.id):dialog.draft.delete(t.id);renderDialog();}
  else if(el.id==='xpCapacity'){capacity=el.value;clientId=null;render();}
  else if(el.id==='xpScene'){scene=el.value;render();}
  else if(el.id==='xpResultScene'){resultScene=el.value;}
 });
 document.addEventListener('input',e=>{const el=e.target;if(el.id==='xpSearch'){query=el.value;render();}else if(el.id==='xpPlatformSearch'){platformQuery=el.value;render();}else if(el.id==='xpClientSearch'){clientQuery=el.value;render();}else if(el.id==='xpTargetSearch'&&dialog){dialog.query=el.value;renderDialog();}});
 host.addEventListener('keydown',e=>{if(e.key==='Escape'&&platformOpen){e.preventDefault();closePlatform();$('#xpPlatformTrigger')?.focus();}});
 host.addEventListener('keydown',e=>{if(e.target.getAttribute('role')==='tab'&&['ArrowLeft','ArrowRight','Home','End'].includes(e.key)){e.preventDefault();const ids=['skills','mcp','clients'],i=ids.indexOf(tab);setTab(e.key==='Home'?ids[0]:e.key==='End'?ids[2]:ids[(i+(e.key==='ArrowRight'?1:2))%3]);}});
 document.addEventListener('keydown',e=>{if(e.key!=='Tab'||$('#modal').getAttribute('role')!=='dialog')return;const items=$$('button:not([disabled]),input:not([disabled]),[tabindex="0"]',$('#modal')).filter(el=>el.getClientRects().length&&!el.hidden);const first=items[0],last=items[items.length-1];if(e.shiftKey&&document.activeElement===first){e.preventDefault();last?.focus();}else if(!e.shiftKey&&document.activeElement===last){e.preventDefault();first?.focus();}});
 document.addEventListener('keydown',e=>{if(dialog?.step==='running'&&e.key==='Escape'){e.preventDefault();e.stopImmediatePropagation();}},true);
 $('#modalMask').addEventListener('click',e=>{if(dialog?.step==='running'&&e.target===$('#modalMask')){e.preventDefault();e.stopImmediatePropagation();}},true);
 const demo=document.createElement('div');demo.className='proto-card xp-demo';demo.id='extensionManagerDemoCard';demo.dataset.protoContexts='extensions';demo.innerHTML='<div class="proto-note-title">多平台扩展 · 本包范围</div><label class="proto-mock-field">目录容量<select id="xpCapacity">'+[['6','六个平台'],['24','24 项平台目录'],['50','50 个模拟目标']].map(([id,text])=>option(id,text,capacity)).join('')+'</select></label><label class="proto-mock-field">页面状态<select id="xpScene">'+[['normal','正常'],['empty','空源库'],['loading','加载中'],['error','读取失败'],['conflict','外部修改冲突'],['unsupported','示例不支持']].map(([id,text])=>option(id,text,scene)).join('')+'</select></label><label class="proto-mock-field">提交结果<select id="xpResultScene">'+[['success','全部完成'],['partial','部分失败']].map(([id,text])=>option(id,text,resultScene)).join('')+'</select></label><p class="pctl-hint">COL-LOCAL-20261014-01 · 版本未定。仅 Skills / MCP 摘要、目标流程、客户端页。新增适配与容量为模拟，不修改真实配置。</p>';($('#protoStdCards .proto-group-body')||$('#protoStdCards')).append(demo);
 new MutationObserver(refreshScope).observe(document.documentElement,{attributes:true,attributeFilter:['data-design-scope','data-route']});
 window.extensionPrototype={active,refresh:render,tab:()=>tab,state:()=>clone({tab,capacity,scene,resources,dialog:dialog?{...dialog,draft:[...dialog.draft],origin:undefined}:null})};
 refreshScope();
})();
