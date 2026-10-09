/* 配置同步原型 · 仅内存 fixture；不读取文件/密钥，不连接 WebDAV，不实现加密。 */
(() => {
 'use strict';
 const $=(s,r=document)=>r.querySelector(s), $$=(s,r=document)=>[...r.querySelectorAll(s)];
 const esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 const host=$('#syncContent'); if(!host)return;
 const groups=[['channels','渠道与模型'],['templates','模型模版'],['accounts','账户'],['skills','Skills'],['mcp','MCP'],['preferences','桌面偏好'],['endpoints','WebDAV 配置']];
 function collect(){
  const m=window.modelPrototype?.state(),list=[];
  for(const p of m?.saved.providers||[]){
   if(p.id==='native')continue;
   list.push({id:'channel:'+p.id,group:'channels',name:p.name,summary:p.models.filter(x=>x.selected).length+' 个已选模型 · 顺序 / 别名 / 覆盖',source:'Desktop 策略 + OpenCodex 已配置渠道',fields:'连接地址、协议端点、模型清单、启停、选择顺序、别名、模版引用与人工覆盖',endpoint:p.connection?.baseUrl||'https://gateway.example.com/v1',key:true,deps:[...new Set([...Object.values(p.refs||{}),...Object.values(m.saved.globalRefs||{}),...p.models.map(x=>x.templateRef).filter(Boolean)])],models:p.models.filter(x=>x.selected).map(x=>x.id),files:'model-policy.json / OpenCodex 渠道配置',modelId:p.id});
  }
  list.push({id:'channel:legacy',group:'channels',name:'私有网关',summary:'2 个已配置模型 · 来自 OpenCodex',source:'OpenCodex 原有配置（非 Desktop 创建）',fields:'已有渠道、协议、端点与显式模型配置；不复制发现缓存',endpoint:'https://private.example.com/v1',key:true,deps:[],files:'OpenCodex 渠道配置'});
  for(const t of m?.templates||[]){list.push({id:'template:'+t.id,group:'templates',name:t.name,summary:t.versions.length+' 个已保存版本 · 主版本 v'+t.main,source:'Desktop 模版库',fields:'已保存版本、匹配规则、协议、能力、版本或后缀与来源；未保存缓冲不进入',files:'model-family-templates.json',versions:t.versions.map(v=>v.n),templateId:t.id});}
  list.push(
   {id:'account:chatgpt',group:'accounts',name:'ChatGPT · 个人账户',summary:'demo@example.com · 登录状态随账户同步',source:'OpenCodex 管理的账户',fields:'账户身份、授权范围；默认包含可迁移的登录状态与刷新材料',login:true,files:'账户适配器 / 系统凭据存储'},
   {id:'account:device',group:'accounts',name:'企业账户 · 设备绑定',summary:'corp@example.com · 新设备需重新登录',source:'OpenCodex 管理的账户',fields:'包含账户信息；设备绑定的登录状态无法迁移，接收端需重新登录',login:false,files:'账户适配器 / 设备绑定凭据'},
   {id:'skill:review',group:'skills',name:'Code Review Skill',summary:'SKILL.md + 2 个资源文件',source:'本机扩展库',fields:'Skill 实际内容、资源、启用状态；此 fixture 不含凭据',files:'skills/code-review/**'},
   {id:'mcp:docs',group:'mcp',name:'Docs MCP',summary:'HTTP 服务器 · 完整定义',source:'本机扩展库',fields:'传输类型、地址、参数、启用状态；Authorization 需另选扩展凭据',extensionSecret:true,files:'MCP 完整定义 / extension-config.json'},
   {id:'preferences:desktop',group:'preferences',name:'通用偏好',summary:'主题、语言与可移植显示偏好',source:'Desktop',fields:'排除机器绝对路径、窗口坐标、日志、缓存、执行绑定与未提交草稿',files:'preferences.json（字段白名单）'},
   {id:'endpoint:personal',group:'endpoints',name:'个人 WebDAV',summary:'文件同步默认包含 · 密码随配置同步',source:'Desktop WebDAV 配置',fields:'服务地址、目录、用户名与连接密码；导入为未启用连接，不改当前连接或本机同步范围；不包含同步加密口令',password:true,files:'sync_endpoints / 受保护凭据部分'}
  ); return list;
 }
 const importSamples={
  encrypted:{file:'opencodex-device-config.ocx',encrypted:true},
  keyless:{file:'opencodex-channels.ocx',encrypted:false},
  'config-only':{file:'opencodex-webdav-config.ocx',encrypted:false},
  legacy:{file:'opencodex-legacy-config.ocx',encrypted:false},
  corrupt:{file:'opencodex-damaged-config.ocx',encrypted:false}
 };
 let importSample='encrypted',importFile=null;
 function setImportSample(value){if(Object.hasOwn(importSamples,value)){importSample=value;const el=$('#sxImportSample');if(el)el.value=value;}}
 function chooseImportFile(){
  if(dialogKind!=='import')return;
  importFile={kind:importSample,...importSamples[importSample]};
  setText('#sxImportFileName',importFile.file);
  $('#sxUnlockField').hidden=!importFile.encrypted;
  setText('#sxImportHint',importFile.encrypted?'解密口令是导出配置包时设置的加密口令。当前文件口令：demo-sync。与账户密码、WebDAV 连接密码相互独立。':'读取后先比较配置，再确认应用。');
  $('#sxUnlock').value='';setText('#sxError','');$('#modalConfirm').disabled=false;
  if(importFile.encrypted)$('#sxUnlock').focus();
 }
 let detailReturnFocus=null,detailOpen=false,lastComparison=null;
 let catalog=collect(),tab='file',query='',originFocus=null,dialogKind='',scenario='normal',serial=0,renderedTab=null;
 const expanded={file:new Set(),webdav:new Set()},searchCollapsed={file:new Set(),webdav:new Set()},scrolls={file:{list:0,secrets:0},webdav:{list:0,secrets:0}};
 const fresh=mode=>({selected:new Set(catalog.filter(x=>mode==='file'||x.group!=='endpoints').map(x=>x.id)),keys:new Set(),accountInfoOnly:new Set(),endpointConfigOnly:new Set(),extensions:new Set(),preset:'all'});
 const scopes={file:fresh('file'),webdav:fresh('webdav')};
 let connection={name:'个人 WebDAV',url:'https://dav.example.com',path:'/opencodex/config-sync/v1',user:'demo-user',connected:true},savedScope='',result=null,lastExport=null,importPlan=null,diffPlan=null;
 const state=()=>scopes[tab], find=id=>catalog.find(x=>x.id===id);
 const setText=(selector,value)=>{const el=$(selector);if(el)el.textContent=value;};
 const btn=(text,action,attrs='',primary=false)=>'<button type="button" class="btn '+(primary?'primary':'ghost')+'" data-sx="'+action+'" '+attrs+'>'+text+'</button>';
 const check=(attr,label,checked=false,disabled=false)=>'<label class="sx-check"><input type="checkbox" '+attr+(checked?' checked':'')+(disabled?' disabled':'')+'><span>'+label+'</span></label>';
 function closure(s=state()){
  const deps=new Map();
  for(const id of s.selected){for(const ref of find(id)?.deps||[]){const split=ref.lastIndexOf(':'),tid=ref.slice(0,split),revision=Number(ref.slice(split+1));if(!tid)continue;const key='template:'+tid;if(!deps.has(key))deps.set(key,new Set());deps.get(key).add(revision);}}
  return {deps,ids:new Set([...s.selected,...deps.keys()])};
 }
 function secretIds(s=state()){
  const selected=closure(s).ids;
  return {keys:[...s.keys].filter(id=>selected.has(id)&&find(id)?.key),accounts:[...selected].filter(id=>find(id)?.login&&!s.accountInfoOnly.has(id)),extensions:[...s.extensions].filter(id=>selected.has(id)&&find(id)?.extensionSecret),endpointPasswords:[...selected].filter(id=>find(id)?.password&&!s.endpointConfigOnly.has(id))};
 }
 const secretCount=s=>Object.values(secretIds(s)).flat().length;
 const signature=s=>JSON.stringify({ids:[...s.selected].sort(),accountInfoOnly:[...s.accountInfoOnly].filter(id=>s.selected.has(id)).sort(),endpointConfigOnly:[...s.endpointConfigOnly].filter(id=>s.selected.has(id)).sort(),...Object.fromEntries(Object.entries(secretIds(s)).map(([k,v])=>[k,v.sort()]))});
 savedScope=signature(scopes.webdav);
 function preset(value){
  const s=state();s.preset=value;
  if(value==='custom')return;
  s.keys.clear();s.accountInfoOnly.clear();s.endpointConfigOnly.clear();s.extensions.clear();
  s.selected=new Set(catalog.filter(x=>tab==='file'||x.group!=='endpoints').map(x=>x.id));
  if(value==='share')for(const x of catalog){if(x.group==='accounts')s.accountInfoOnly.add(x.id);if(x.password)s.endpointConfigOnly.add(x.id);}
  if(value==='device'){for(const x of catalog){if(x.key)s.keys.add(x.id);if(x.extensionSecret)s.extensions.add(x.id);}}
 }
 function selectObject(s,id,checked){
  if(checked){if(!s.selected.has(id)){s.accountInfoOnly.delete(id);s.endpointConfigOnly.delete(id);}s.selected.add(id);}
  else{s.selected.delete(id);s.accountInfoOnly.delete(id);s.endpointConfigOnly.delete(id);}
 }
 function sxAnn(num,title,body){
  return '<div class="proto-annotation" id="ann-sx-'+num+'" data-annotation-number="'+num+'" data-annotation-title="'+title+'" data-annotation-category="实现边界 · 同步"><button type="button" class="annotation-marker" aria-expanded="false" aria-controls="annotationPopover" aria-label="原型注释 '+num+'：'+title+'"><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 2.5h7l3 3v8H3zM10 2.5v3h3M5.5 8h5M5.5 10.5h3"/></svg><span>原型</span><span class="annotation-number">'+num+'</span><span class="annotation-marker-plus" aria-hidden="true">＋</span></button><div class="annotation-source" hidden><p>'+body+'</p><div class="annotation-footer">实现边界 · 同步</div></div></div>';
 }
 function scopeHTML(){
  const s=state(),{ids,deps}=closure(),visible=catalog.filter(x=>(tab==='file'||x.group!=='endpoints')&&(x.name+' '+x.summary).toLowerCase().includes(query.toLowerCase()));
  const rows=groups.map(([id,label])=>{
   const items=visible.filter(x=>x.group===id);if(!items.length)return '';
   const all=items.every(x=>ids.has(x.id)),count=items.filter(x=>ids.has(x.id)).length,open=query?!searchCollapsed[tab].has(id):expanded[tab].has(id);
   return '<div class="sx-group"><div class="sx-group-head"><input type="checkbox" data-sx-group="'+id+'" aria-label="选择'+label+'" '+(all?'checked':'')+'><button class="sx-group-title" data-sx="expand" data-id="'+id+'" aria-expanded="'+open+'"><span class="sx-chevron" aria-hidden="true">▾</span>'+label+'</button><span class="sx-group-count">'+count+' / '+items.length+'</span></div>'+(open?items.map(x=>{
    const dependent=deps.has(x.id)&&!s.selected.has(x.id),included=Object.values(secretIds(s)).some(items=>items.includes(x.id));
    let mark=dependent?'依赖补齐':x.group==='accounts'?(s.accountInfoOnly.has(x.id)?'仅账户信息':x.login?(included?'含登录状态':'随账户同步'):'需重新登录'):x.password?(included?'含密码':'仅配置'):included?'含凭据':x.key?'不带 Key':'配置';
    return '<div class="sx-object"><input type="checkbox" data-sx-object="'+esc(x.id)+'" aria-label="选择 '+esc(x.name)+'" '+(ids.has(x.id)?'checked':'')+(dependent?' disabled title="所选渠道需要此精确版本"':'')+'><div><button class="sx-object-name" data-sx="detail" data-id="'+esc(x.id)+'">'+esc(x.name)+'</button><small>'+esc(dependent?'仅精确版本 v'+[...deps.get(x.id)].join(' / v'):x.password?(s.endpointConfigOnly.has(x.id)?'仅配置 · 不含连接密码':'密码随配置同步 · 导入后待启用'):x.summary)+'</small></div><span class="sx-mark '+(included?'secret':x.group==='accounts'&&!x.login?'warn':'')+'">'+mark+'</span></div>';
   }).join(''):'')+'</div>';
  }).join('');
  return '<div class="sx-toolbar"><select data-sx-preset aria-label="同步范围预设"><option value="all" '+(s.preset==='all'?'selected':'')+'>全部配置</option><option value="share" '+(s.preset==='share'?'selected':'')+'>分享配置 · 不含凭据</option><option value="device" '+(s.preset==='device'?'selected':'')+'>换设备 · 含可转移凭据</option><option value="custom" '+(s.preset==='custom'?'selected':'')+'>自定义范围</option></select><input class="input" id="sxSearch" type="search" aria-label="搜索配置对象" placeholder="搜索配置对象" value="'+esc(query)+'">'+btn('清空选择','clear',ids.size?'':'disabled')+'</div><div class="sx-table" role="region" aria-label="配置对象列表" aria-describedby="sxScopeHelp" tabindex="0"><div class="sx-table-head"><input type="checkbox" data-sx-all aria-label="选择当前搜索结果" '+(visible.length&&visible.every(x=>ids.has(x.id))?'checked':'')+'><span>配置对象</span><span>范围与凭据</span></div>'+(rows||'<div class="sx-empty">没有匹配的配置对象<br>更换关键词或清除搜索。</div>')+'</div><p class="sx-note sx-scope-help" id="sxScopeHelp">勾选账户默认含可迁移登录状态'+(tab==='file'?'，WebDAV 配置默认含连接密码':'')+'；点击名称可调整同步选项。未保存草稿不参与。'+(query?' 搜索仅改变显示，隐藏勾选保留。':'')+'</p>'+sxAnn('01','同步范围为内存演示','范围预设、勾选与搜索均为内存演示；导入 / 导出走确认流程，不读写真实文件。');
 }
 function credentialsHTML(){
  const s=state(),ids=closure().ids;
  const blocks=[['keys','渠道 API Key','包含所选渠道的 Key','渠道可单独分享，接收方自行填 Key。',x=>x.key],['extensions','扩展凭据','包含所选扩展的凭据','例如 MCP 的 Authorization / 环境变量。',x=>x.extensionSecret]].map(([key,title,label,note,match])=>{
   const items=catalog.filter(x=>ids.has(x.id)&&match(x)),active=items.some(x=>s[key].has(x.id));
   return '<section class="sx-secret-block">'+check('data-sx-secret="'+key+'"',title,active,!items.length)+'<p class="sx-muted">'+note+'</p>'+(active?'<div class="sx-subpicks">'+items.map(x=>check('data-sx-secret-object="'+x.id+'" data-kind="'+key+'"',esc(x.name),s[key].has(x.id))).join('')+'</div>':'')+'</section>';
  }).join('');
  return '<aside class="sx-secrets"><header><h3>其他凭据</h3><p class="sx-muted">账户登录状态随账户选择'+(tab==='file'?'，WebDAV 密码随配置选择':'')+'；以下凭据可另选。</p></header>'+blocks+'<p class="sx-note">'+(secretCount()?'所选凭据必须加密传输。加密口令独立于 WebDAV 密码，不随包或远端数据保存。':'当前不携带凭据。导入同一身份时保留本机已有凭据，不用空值覆盖。')+'</p>'+sxAnn('02','凭据勾选为演示','凭据选择只演示勾选与加密要求，不读取真实密钥；加密口令只描述规则，不做真实加密。')+'</aside>';
 }
 function render(resetList=false){
  if(renderedTab){scrolls[renderedTab].list=$('.sx-table',host)?.scrollTop||0;scrolls[renderedTab].secrets=$('.sx-secrets',host)?.scrollTop||0;}
  if(resetList)scrolls[tab].list=0;
  const selected=closure().ids.size,dirty=signature(scopes.webdav)!==savedScope;
  host.innerHTML='<div class="sx-tabs" role="tablist" aria-label="同步方式">' +
   '<div id="sxPanel" role="tabpanel" aria-labelledby="sxTab-'+tab+'">' +
   (tab==='webdav'?'<div class="sx-connection"><div class="sx-connection-info"><div><strong>'+esc(connection.name)+'</strong> <span class="sx-mark">'+(connection.connected?'演示连接':'未连接')+'</span> <span class="sx-muted">本机：MacBook</span></div><small class="sx-muted" title="'+esc(connection.url+connection.path)+'">'+esc(connection.url+connection.path)+'</small></div>'+btn('连接设置','connection')+'</div>'+sxAnn('03','连接信息为演示数据','连接名称、地址与状态均为演示数据；测试连接、保存与同步仅切换演示状态，不发起真实网络请求。'):'') +
   '<div class="sx-layout"><section class="sx-scope" aria-label="同步范围">'+scopeHTML()+'</section>'+credentialsHTML()+'</div>' +
   (result&&result.tab===tab?'<div class="sx-result" role="status"><strong>'+esc(result.title)+'</strong><span>'+esc(result.detail)+'</span></div>':'') +
   '</div>';
  $('.sx-table',host).scrollTop=scrolls[tab].list;$('.sx-secrets',host).scrollTop=scrolls[tab].secrets;renderedTab=tab;
  for(const el of $$('[data-sx-group]',host)){const items=catalog.filter(x=>x.group===el.dataset.sxGroup&&(x.name+' '+x.summary).toLowerCase().includes(query.toLowerCase())),n=items.filter(x=>closure().ids.has(x.id)).length;el.indeterminate=n>0&&n<items.length;}
  for(const el of $$('[data-sx-secret]',host)){const key=el.dataset.sxSecret,prop={keys:'key',extensions:'extensionSecret'}[key],items=catalog.filter(x=>closure().ids.has(x.id)&&x[prop]),n=items.filter(x=>state()[key].has(x.id)).length;el.indeterminate=n>0&&n<items.length;}
  const all=$('[data-sx-all]',host),shown=catalog.filter(x=>(tab==='file'||x.group!=='endpoints')&&(x.name+' '+x.summary).toLowerCase().includes(query.toLowerCase())),n=shown.filter(x=>closure().ids.has(x.id)).length;if(all)all.indeterminate=n>0&&n<shown.length;
 }
 function showDialog(kind,title,body,confirm,handler,opts={}){
  if(!dialogKind)originFocus=document.activeElement;
  dialogKind=kind;
  openModal(title,'<div class="sx-dialog">'+body+'<div class="sx-error" id="sxError" role="alert"></div></div>',confirm,handler,{confirmKeepsOpen:true,hideNote:true,...opts});
  $('#modal').classList.remove('sx-comparison');$('#modal').classList.add('modal-wide');$('#modalConfirm').disabled=false;
  window.__modalOnClose=()=>{for(const el of [$('.modal-head'),$('.modal-actions')])if(el)el.inert=false;detailOpen=false;detailReturnFocus=null;$('#modal').classList.remove('sx-comparison');dialogKind='';$('#modalBody').innerHTML='';$('#modalConfirm').disabled=false;if(originFocus?.isConnected)originFocus.focus();originFocus=null;};
 }
 function detail(id){
  const x=find(id);if(!x)return;
  const accountOptions=x.login?'<details class="sx-account-options"><summary>同步选项</summary>'+check('data-sx-account-info="'+esc(id)+'"','仅同步账户信息，不含登录状态',state().accountInfoOnly.has(id),!state().selected.has(id))+'<p class="sx-muted">'+(state().selected.has(id)?'默认随账户同步登录状态；修改后立即计入当前范围。':'先勾选此账户，再调整同步选项。')+'</p></details>':'';
  const endpointOptions=x.password?'<details class="sx-account-options"><summary>同步选项</summary>'+check('data-sx-endpoint-config="'+esc(id)+'"','仅同步配置，不含密码',state().endpointConfigOnly.has(id),!state().selected.has(id))+'<p class="sx-muted">默认随配置同步连接密码；导入后需另行启用连接。同步加密口令始终不进入配置包。</p></details>':'';
  showDialog('detail',x.name,'<p class="sx-muted">配置对象详情 · 敏感值不展示</p><dl class="sx-detail-list"><dt>归属</dt><dd>'+esc(x.source)+'</dd><dt>包含字段</dt><dd>'+esc(x.fields)+'</dd><dt>配置来源</dt><dd>'+esc(x.files)+'</dd>'+(x.endpoint?'<dt>连接地址</dt><dd>'+esc(x.endpoint)+'</dd>':'')+'<dt>依赖</dt><dd>'+esc(x.deps?.length?x.deps.join('、'):'无额外模版依赖')+'</dd>'+(x.models?.length?'<dt>已选模型</dt><dd>'+x.models.map(esc).join('、')+'</dd>':'')+'<dt>凭据</dt><dd>'+esc(x.key?'API Key 单独选择；不显示或复制掩码值':x.login?'登录状态默认随账户同步，接收端自动恢复并检查':x.id==='account:device'?'设备绑定的登录状态无法迁移；新设备需重新登录':x.extensionSecret?'授权头 / 环境变量单独选择':x.password?'连接密码默认随配置同步，可选仅配置；不包含同步加密口令':'不携带凭据')+'</dd></dl>'+accountOptions+endpointOptions+(['accounts','endpoints'].includes(x.group)?'':'<p class="sx-note">Desktop 策略和 OpenCodex 字段属于同一渠道时合为一个应用计划，不重复创建；外部漂移先比较。</p>'),'关闭',closeModal,{cancelLabel:'关闭',hideConfirm:true});
 }
 function exportPreview(){
  if(!closure().ids.size)return;
  const cl=closure(),secrets=secretIds(),count=secretCount();
  const manifest={demo:true,kind:'configuration-sync',objects:[...cl.ids],dependencies:[...cl.deps].map(([id,versions])=>({id,revisions:[...versions]})),credentials:secrets,source:'committed',encryptionRequired:count>0};
  const body='<p class="sx-steps">选择范围 → <strong>确认内容</strong> → 生成配置包</p><div class="sx-info"><strong>'+cl.ids.size+' 项配置 · '+(count?'含 '+count+' 项凭据':'不含任何凭据')+'</strong><br>渠道 Key '+secrets.keys.length+' 项 / 账户登录态 '+secrets.accounts.length+' 项 / 扩展凭据 '+secrets.extensions.length+' 项 / WebDAV 密码 '+secrets.endpointPasswords.length+' 项</div>'+check('id="sxEncrypt"',count?'含凭据，必须加密':'为配置包设置加密口令',count,!!count)+'<div id="sxPassFields" '+(!count?'hidden':'')+'><div class="sx-grid"><label class="sx-field"><span>加密口令（至少 8 位）</span><input id="sxPass" class="input" type="password" autocomplete="new-password"></label><label class="sx-field"><span>再次输入</span><input id="sxPassAgain" class="input" type="password" autocomplete="new-password"></label></div></div><details><summary>查看内容清单</summary><pre>'+esc(JSON.stringify(manifest,null,2))+'</pre></details><p class="sx-muted">未选择的凭据不进入包，也不会由模版依赖重新带入。</p>';
  showDialog('export','确认导出',body,'生成配置包',()=>{
   const encrypted=$('#sxEncrypt').checked,p=$('#sxPass').value;if(encrypted&&(p.length<8||p!==$('#sxPassAgain').value)){setText('#sxError','请输入至少 8 位的加密口令，并保持两次一致。');return;}
   lastExport={...manifest,encrypted,simulated:true};result={tab:'file',title:'导出流程演示完成',detail:cl.ids.size+' 项配置；'+(encrypted?'加密容器步骤已演示，未生成真实加密文件。':'不携带凭据，未生成真实文件。')};closeModal();render();
  });
 }
 // 导入与 WebDAV 共用对象树、选择和凭据规则；详情层只读且不重建父弹窗。
 const comparisonGroup=x=>find(x.id)?.group||({channel:'channels',remap:'channels',template:'templates',account:'accounts',token:'accounts',skill:'skills',mcp:'mcp',extension:'mcp',preferences:'preferences',endpoint:'endpoints'}[x.kind])||'preferences';
 const changeLabels={add:'新增',update:'更新',delete:'删除'};
 function comparisonInit(p,mode){
  p.mode=mode;p.expanded=new Set();p.scroll=0;p.pass='';
  p.rows=p.rows.map(x=>({...x,group:comparisonGroup(x),changes:x.changes||{update:1}}));
  p.selected ||=new Set(p.rows.filter(x=>!x.removal).map(x=>x.id));
  p.accountInfoOnly ||=new Set();p.endpointConfigOnly ||=new Set();p.credentialOverrides={};
  for(const x of p.rows)p.credentialOverrides[x.id]=credentialDefault(p,x);
  return p;
 }
 function credentialType(p,x){
  if(x.removal)return '';
  if(p.mode==='import'&&p.source!=='encrypted')return '';
  if(x.group==='channels'&&x.kind!=='remap'&&(p.mode==='import'||scopes.webdav.keys.has(x.id)))return 'Key';
  if(x.group==='accounts'&&x.login)return '登录状态';
  if(x.group==='mcp'&&x.kind!=='extension'&&(p.mode==='import'||scopes.webdav.extensions.has(x.id)))return '扩展凭据';
  if(x.group==='endpoints'&&x.password)return '连接密码';
  return '';
 }
 function credentialDefault(p,x){return ['登录状态','连接密码'].includes(credentialType(p,x));}
 function credentialValue(p,x){
  const type=credentialType(p,x);
  return type==='登录状态'?!p.accountInfoOnly.has(x.id):type==='连接密码'?!p.endpointConfigOnly.has(x.id):!!p.credentialOverrides[x.id];
 }
 function setCredential(p,x,checked){
  p.credentialOverrides[x.id]=checked;
  if(x.group==='accounts')checked?p.accountInfoOnly.delete(x.id):p.accountInfoOnly.add(x.id);
  if(x.group==='endpoints')checked?p.endpointConfigOnly.delete(x.id):p.endpointConfigOnly.add(x.id);
 }
 function comparisonPick(p,x,checked){
  const was=p.selected.has(x.id);selectObject(p,x.id,checked);
  if(!checked||!was&&checked){p.credentialOverrides[x.id]=credentialDefault(p,x);if(checked)setCredential(p,x,credentialDefault(p,x));}
 }
 function changeBadges(items){
  return Object.entries(changeLabels).map(([key,label])=>{const n=items.reduce((sum,x)=>sum+(x.changes[key]||0),0);return n?'<span class="sx-change '+key+'">'+label+' '+n+'</span>':'';}).join('');
 }
 function ruleCheckbox(p,items,group){
  const eligible=items.filter(x=>p.selected.has(x.id)&&credentialType(p,x));
  if(!items.some(x=>credentialType(p,x)))return '';
  const attr=group?'data-sx-compare-rule="'+group+'"':'data-sx-compare-rule="all"';
  const label=p.mode==='diff'?'同步凭据':'采用来源凭据';
  return check(attr+' aria-label="'+(group?groups.find(g=>g[0]===group)?.[1]:'全部对象')+label+'"',group?label:'统一'+label,eligible.length>0&&eligible.every(x=>credentialValue(p,x)),!eligible.length);
 }
 function comparisonOperations(p,x){
  const selected=p.selected.has(x.id),type=credentialType(p,x),mode=p.mode;
  let credential='';
  if(type==='登录状态')credential=check('data-sx-'+(mode==='import'?'import-account-info':'diff-account-info')+'="'+esc(x.id)+'"','仅账户信息',p.accountInfoOnly.has(x.id),!selected);
  else if(type==='连接密码')credential=check('data-sx-import-endpoint-config="'+esc(x.id)+'"','仅配置，不含密码',p.endpointConfigOnly.has(x.id),!selected);
  else if(type)credential=check('data-sx-compare-credential="'+esc(x.id)+'"',(x.kind==='upload'?'同步':'覆盖')+type,credentialValue(p,x),!selected);
  let mapping='';
  if(x.kind==='remap')mapping='<select data-sx-import-choice="'+esc(x.id)+'" aria-label="'+esc(x.name)+'身份映射" '+(!selected?'disabled':'')+'><option value="">选择身份映射</option><option value="new" '+(p.choices[x.id]==='new'?'selected':'')+'>新建渠道</option><option value="rebind" '+(p.choices[x.id]==='rebind'?'selected':'')+'>重新绑定，待补 Key</option></select>';
  if(mode==='diff'&&['conflict','token'].includes(x.kind))mapping='<select data-sx-diff-choice="'+esc(x.id)+'" aria-label="'+esc(x.name)+'同步方向" '+(!selected?'disabled':'')+'><option value="">选择同步方向</option><option value="local" '+(p.choices[x.id]==='local'?'selected':'')+'>保留本机</option><option value="remote" '+(p.choices[x.id]==='remote'?'selected':'')+'>采用远端</option><option value="skip" '+(p.choices[x.id]==='skip'?'selected':'')+'>暂不处理</option></select>';
  return '<div class="sx-compare-ops">'+btn('查看详情','compare-detail','data-id="'+esc(x.id)+'"')+credential+mapping+'</div>';
 }
 function comparisonTree(p){
  if(p.mode==='import'&&p.selected.has('channel:newapi'))p.selected.add('template:gpt');
  const dependent=x=>p.mode==='import'&&x.id==='template:gpt'&&p.selected.has('channel:newapi');
  const rows=groups.map(([id,label])=>{
   const items=p.rows.filter(x=>x.group===id);if(!items.length)return '';
   const open=p.expanded.has(id),count=items.filter(x=>p.selected.has(x.id)).length;
   return '<section class="sx-compare-group"><header class="sx-compare-group-head"><input type="checkbox" data-sx-compare-group="'+id+'" aria-label="选择'+label+'变化" '+(count===items.length?'checked':'')+'><button type="button" class="sx-group-title" data-sx="compare-expand" data-id="'+id+'" aria-expanded="'+open+'"><span class="sx-chevron" aria-hidden="true">▾</span>'+label+'</button><div class="sx-change-list">'+changeBadges(items)+'</div><div class="sx-compare-ops">'+ruleCheckbox(p,items,id)+'</div></header>'+
    '<div '+(!open?'hidden':'')+'>'+items.map(x=>{
     const note=x.group==='accounts'?(p.accountInfoOnly.has(x.id)?'仅账户信息 · 保留本机已有登录状态':x.login?'含登录状态 · 应用时自动恢复并检查':'登录状态无法迁移 · 新设备需重新登录'):x.group==='endpoints'?(p.endpointConfigOnly.has(x.id)?'仅配置，不含密码 · 同身份保留本机密码':x.password?'含连接密码 · 导入后待启用':x.detail):x.detail;
     return '<div class="sx-compare-item" data-sx-compare-item="'+esc(x.id)+'"><input type="checkbox" data-sx-'+(p.mode==='import'?'import':'diff')+'-pick="'+esc(x.id)+'" aria-label="'+(p.mode==='import'?'导入':'同步')+' '+esc(x.name)+'" '+(p.selected.has(x.id)?'checked':'')+(dependent(x)?' disabled title="NewApi 依赖此精确版本"':'')+'><div class="sx-compare-label"><strong>'+esc(x.name)+'</strong><small>'+esc(note)+'</small>'+(x.removal?'<small>明确删除记录 · 默认不选</small>':'')+'</div><div class="sx-change-list">'+changeBadges([x])+'</div>'+comparisonOperations(p,x)+'</div>';
    }).join('')+'</div></section>';
  }).join('');
  return '<div class="sx-compare-tree" role="region" aria-label="配置变化树" tabindex="0"><div class="sx-compare-head"><input type="checkbox" data-sx-compare-all aria-label="选择全部变化" '+(p.rows.length&&p.rows.every(x=>p.selected.has(x.id))?'checked':'')+'><span>配置对象</span><span>变化</span><div class="sx-compare-ops">'+ruleCheckbox(p,p.rows)+'<span class="sx-muted">操作与凭据</span></div></div>'+(rows||'<div class="sx-empty">所选范围没有变化。</div>')+'</div>';
 }
 function rememberComparison(p){if($('#sxCompareMain')){p.scroll=$('.sx-compare-tree')?.scrollTop||0;if($('#sxSyncPass'))p.pass=$('#sxSyncPass').value;}}
 function mountComparison(p){
  $('#modal').classList.add('sx-comparison');$('.sx-compare-tree').scrollTop=p.scroll;
  if($('#sxSyncPass'))$('#sxSyncPass').value=p.pass;
  for(const el of $$('[data-sx-compare-all],[data-sx-compare-group],[data-sx-compare-rule]')){
   const items=p.rows.filter(x=>!el.dataset.sxCompareGroup||x.group===el.dataset.sxCompareGroup);
   const eligible=el.dataset.sxCompareRule?items.filter(x=>p.selected.has(x.id)&&credentialType(p,x)&&(el.dataset.sxCompareRule==='all'||x.group===el.dataset.sxCompareRule)):items;
   const n=eligible.filter(x=>el.dataset.sxCompareRule?credentialValue(p,x):p.selected.has(x.id)).length;
   el.indeterminate=n>0&&n<eligible.length;
  }
 }
 function comparisonBody(p,intro,extra=''){
  return '<div class="sx-compare-main" id="sxCompareMain">'+intro+comparisonTree(p)+extra+'<p class="sx-note">只处理勾选项；未包含对象保持原样。删除只来自明确删除记录。确认后先备份，再应用与核对。</p></div><div class="sx-detail-layer" id="sxCompareDetail" hidden></div>';
 }
 function comparisonFields(x,p){
  const source=p.mode==='import'?'包内':'远端',endpoint=find(x.id)?.endpoint||'https://gateway.example.com/v1';
  const f=(section,...rows)=>({section,rows});
  if(x.group==='channels')return [f('连接与身份',['渠道 ID',x.id,x.kind==='remap'?'待新建或重新绑定':x.id],['服务地址',endpoint,x.kind==='remap'?'https://new-private.example.com/v1':endpoint],['协议','openai-responses','openai-responses'],['API Key','本机已保存',credentialType(p,x)?source+'已携带 · 值隐藏':'未携带 · 保留同身份本机 Key']),f('模型与覆盖',['模型清单','17 个','19 个 · 新增 3 / 删除 1'],['新增模型','—','gpt-5.4-mini、gpt-5.3-codex、gpt-5.2-codex'],['删除模型','gpt-4.1','—'],['gpt-5.4 别名','—','主力模型'],['上下文大小','128,000','256,000'],['启用状态','启用','启用'],['推理等级','低 / 中 / 高','低 / 中 / 高 / 极高'],['快速模式','支持','支持'],['选择顺序','gpt-5.4 → gpt-5.2','gpt-5.4 → gpt-5.4-mini'],['模版引用','gpt:v1','gpt:v4（精确依赖）'])];
  if(x.group==='templates')return [f('版本与匹配',['模版 ID',x.id,x.id],['已保存版本','v1 / v2 / v3','新增 v4'],['主版本','v3','保持 v3'],['匹配规则','gpt-*','gpt-*（排除旧版）'],['协议端点','/responses','/responses']),f('能力与来源',['上下文大小','128,000','256,000'],['推理等级','低 / 中 / 高','低 / 中 / 高 / 极高'],['快速模式','支持','支持'],['版本 / 后缀覆盖','gpt-5.2','gpt-5.2 / gpt-5.4-mini'],['来源','人工维护','已保存版本 · 另一台设备'])];
  if(x.group==='accounts')return [f('账户身份',['账户 ID',x.id,x.id],['邮箱',x.login?'demo@example.com':'corp@example.com',x.login?'demo@example.com':'corp@example.com'],['提供方','OpenAI',x.login?'OpenAI':'企业身份服务'],['授权范围','模型访问','模型访问']),f('登录状态',['登录材料','本机已有 · 值隐藏',x.login?'可迁移 · 值隐藏':'设备绑定 · 无可迁移材料'],['有效性','本机状态待核对',p.mode==='diff'&&!p.verified?'已失效 · 需重新登录':x.login?'应用时自动检查':'新设备需重新登录'],['采用规则','保留当前登录',p.accountInfoOnly.has(x.id)?'仅账户信息，保留本机登录状态':'采用可迁移登录状态并检查'])];
  if(x.group==='skills')return [f('Skill 与资源',['Skill ID',x.id,x.removal?'删除记录':x.id],['启用状态','启用',x.removal?'—':'启用'],['SKILL.md','旧版审查步骤',x.removal?'—':'更新审查步骤与输出要求'],['资源文件','references/old-guide.md',x.removal?'—':'新增 references/checklist.md、scripts/report.js；删除 old-guide.md'],['内容校验','旧版摘要',x.removal?'明确删除':'更新版本 · 摘要已校验'])];
  if(x.kind==='extension')return [f('旧版扩展开关',['对象范围','本机开关','仅启用状态 / 同步方式'],['实际内容','保留本机 Skill / MCP','未包含，不覆盖本机内容'])];
  if(x.group==='mcp')return [f('服务器定义',['服务器 ID',x.id,x.id],['传输类型','HTTP','HTTP'],['地址','https://docs.example.com/mcp','https://docs.example.com/v2/mcp'],['命令 / 参数','HTTP 不适用','HTTP 不适用'],['启用状态','启用','启用']),f('环境与凭据',['环境变量','DOCS_LOCALE=zh-CN','DOCS_LOCALE=en-US'],['请求头','Authorization 已保存 · 值隐藏',credentialType(p,x)?'Authorization 已携带 · 值隐藏':'未携带，保留同身份本机值'],['本机执行绑定','保留当前绑定','不参与迁移'])];
  if(x.group==='endpoints')return [f('WebDAV 配置',['配置 ID',x.id,x.id],['服务地址',connection.url,'https://dav.example.com'],['同步目录',connection.path,'/opencodex/config-sync/v1'],['用户名',connection.user,'demo-user'],['连接密码','本机已保存 · 值隐藏',x.password?'包内携带 · 值隐藏':'未携带，同身份保留本机值']),f('应用后状态',['连接启用','当前连接保持','导入连接待启用'],['本机同步范围 / 定时规则','保留本机','不参与迁移'],['同步加密口令','仅本机输入','不在配置包中'])];
  return [f('桌面偏好',['主题','跟随系统','浅色'],['语言','简体中文','简体中文'],['显示密度','标准','紧凑'],['选择器显示','模型名称','别名优先']),f('本机专属字段',['绝对路径 / 窗口坐标','保留本机','不参与迁移'],['执行绑定 / 缓存 / 草稿','保留本机','不参与迁移'])];
 }
 function comparisonDetail(id){
  const p=dialogKind==='diff'?diffPlan:importPlan,x=p?.rows.find(r=>r.id===id);if(!x)return;
  detailReturnFocus=document.activeElement;detailOpen=true;
  const layer=$('#sxCompareDetail');layer.innerHTML='<section class="sx-detail-card" role="dialog" aria-modal="true" aria-labelledby="sxCompareDetailTitle"><header><div><h3 id="sxCompareDetailTitle">'+esc(x.name)+' · 详情</h3><p class="sx-muted">只读比较 · 敏感值隐藏</p></div>'+btn('关闭','compare-detail-close','id="sxCompareDetailClose"')+'</header><div class="sx-detail-body">'+comparisonFields(x,p).map(section=>({...section,rows:x.kind==='upload'?section.rows.map(([label,local,incoming])=>[label,incoming,local]):section.rows})).map(section=>'<h4>'+esc(section.section)+'</h4><table class="sx-field-compare"><thead><tr><th scope="col">字段</th><th scope="col">本机</th><th scope="col">'+(p.mode==='import'?'配置包':'远端')+'</th></tr></thead><tbody>'+section.rows.map(([label,local,incoming])=>'<tr class="'+(local!==incoming?'changed':'')+'"><th scope="row">'+esc(label)+'</th><td>'+esc(local)+'</td><td>'+esc(incoming)+'</td></tr>').join('')+'</tbody></table>').join('')+'</div><footer class="sx-muted">关闭后返回比较列表，勾选与覆盖规则保持。</footer></section>';
  layer.hidden=false;for(const el of [$('#sxCompareMain'),$('.modal-head'),$('.modal-actions')])if(el)el.inert=true;
  $('#sxCompareDetailClose').focus();
 }
 function closeComparisonDetail(){
  if(!detailOpen)return;detailOpen=false;$('#sxCompareDetail').hidden=true;
  for(const el of [$('#sxCompareMain'),$('.modal-head'),$('.modal-actions')])if(el)el.inert=false;
  detailReturnFocus?.focus({preventScroll:true});detailReturnFocus=null;
 }
 function comparisonResult(p,rows){lastComparison={mode:p.mode,items:rows.map(x=>({id:x.id,group:x.group,changes:x.changes,direction:p.choices[x.id]||(x.kind==='upload'?'upload':'incoming'),credential:!!credentialType(p,x)&&credentialValue(p,x)}))};}
 const importRows=()=>[
  {id:'template:gpt',name:'GPT · Responses / v4',detail:'新增已保存版本 v4 · 原主版本与引用保持',kind:'template',changes:{add:1}},
  {id:'channel:newapi',name:'NewApi',detail:'同一渠道 · 模型新增 3 / 更新 2 / 删除 1',kind:'channel',changes:{add:3,update:2,delete:1}},
  {id:'channel:legacy',name:'私有网关',detail:'同名但端点不同 · 必须确认身份，不复用原 Key',kind:'remap',conflict:true,changes:{add:1}},
  {id:'account:chatgpt',name:'ChatGPT · 个人账户',detail:'账户信息与登录状态更新',kind:'account',login:true,changes:{update:1}},
  {id:'account:device',name:'企业账户 · 设备绑定',detail:'账户信息更新 · 新设备需重新登录',kind:'account',login:false,changes:{update:1}},
  {id:'skill:review',name:'Code Review Skill',detail:'Skill 更新 · 新增 2 个资源 / 删除 1 个旧资源',kind:'skill',changes:{add:2,update:1,delete:1}},
  {id:'skill:legacy-review',name:'旧版 Review Skill',detail:'来源显式标记删除',kind:'skill',removal:true,changes:{delete:1}},
  {id:'mcp:docs',name:'Docs MCP',detail:'服务器地址与环境变量更新',kind:'mcp',changes:{update:1}},
  {id:'preferences:desktop',name:'通用偏好',detail:'主题与显示偏好更新 · 保留本机路径',kind:'preferences',changes:{update:2}},
  {id:'endpoint:personal',name:'个人 WebDAV',detail:'连接配置更新 · 导入后待启用',kind:'endpoint',password:true,changes:{update:1}}
 ];
 function importStart(context={}){
  importFile=null;
  importPlan={rows:importRows(),selected:new Set(importRows().map(x=>x.id)),choices:{},secrets:false,accountInfoOnly:new Set(),endpointConfigOnly:new Set(),source:'encrypted',modelScope:context.scope||null};
  showDialog('import','导入配置','<p class="sx-steps"><strong>读取配置包</strong> → 选择与比较 → 应用结果</p><div class="sx-field"><span>配置文件</span><div class="sx-file-picker">'+btn('选择文件','choose-import-file')+'<span id="sxImportFileName" aria-live="polite">尚未选择文件</span></div></div><label class="sx-field" id="sxUnlockField" hidden><span>解密口令</span><input class="input" id="sxUnlock" type="password" autocomplete="off" aria-describedby="sxImportHint" placeholder="请输入配置包的加密口令"></label><p class="sx-muted" id="sxImportHint">读取后先比较配置，再确认应用。</p>','读取并比较',()=>{
   if(!importFile){setText('#sxError','请先选择配置文件。');return;}
   const kind=importFile.kind;if(kind==='corrupt'){setText('#sxError','容器完整性校验失败。未创建候选、未修改本机配置。');return;}
   if(importFile.encrypted&&$('#sxUnlock').value!=='demo-sync'){setText('#sxError','解密口令不正确，尚未读取内容。');return;}
   importPlan.source=kind;
   if(kind==='keyless')importPlan.rows=importRows().filter(x=>['template','channel','remap'].includes(x.kind));
   if(kind==='config-only')importPlan.rows=importRows().filter(x=>x.kind==='endpoint').map(x=>({...x,password:false,detail:'不含密码 · 同身份保留本机密码，新连接待补密码'}));
   if(kind==='legacy')importPlan.rows=[{id:'preferences:desktop',name:'通用偏好',detail:'旧容器仅含该范围；未包含项不删除',kind:'preferences'},{id:'extension:legacy',name:'扩展开关',detail:'仅启用状态 / 同步方式，不冒充完整 Skill 或 MCP',kind:'extension'}];
   importPlan.selected=new Set(importPlan.rows.filter(x=>!x.removal).map(x=>x.id));
   if(context.scope?.length){const wanted=new Set(context.scope);importPlan.selected=new Set(importPlan.rows.filter(x=>wanted.has(x.id)||x.kind==='template'&&context.scope.some(id=>id.startsWith('channel:'))).map(x=>x.id));}
   comparisonInit(importPlan,'import');importPreview();
  });
  $('#modalConfirm').disabled=true;
 }
 function importPreview(){
  const p=importPlan,encrypted=p.source==='encrypted';rememberComparison(p);
  const intro='<p class="sx-steps">已读取'+(encrypted?'并解密':'')+'配置包 → <strong>选择与比较</strong> → 应用结果</p><p class="sx-muted">勾选要应用的变化，展开分类查看对象。凭据采用规则可统一设置，也可逐项调整。</p>';
  showDialog('import','选择导入内容',comparisonBody(p,intro),'备份并应用',()=>{
   if(!p.selected.size){setText('#sxError','至少选择一项配置。');return;}
   if(p.rows.some(x=>p.selected.has(x.id)&&x.kind==='remap'&&!p.choices[x.id])){setText('#sxError','请先确认同名异端点渠道的身份映射。');return;}
   const applied=p.rows.filter(x=>p.selected.has(x.id));
   const accounts=applied.filter(x=>x.kind==='account').map(x=>({id:x.id,login:encrypted&&x.login&&!p.accountInfoOnly.has(x.id),infoOnly:p.accountInfoOnly.has(x.id)}));
   const endpoints=applied.filter(x=>x.kind==='endpoint').map(x=>({id:x.id,password:encrypted&&x.password&&!p.endpointConfigOnly.has(x.id)}));
   comparisonResult(p,applied);finishApply('file',applied.length,accounts,0,endpoints);
  });mountComparison(p);
 }
 function connectionDialog(){
  showDialog('connection','WebDAV 连接','<p class="sx-muted">WebDAV 配置可通过文件同步迁移，导入后需另行启用。当前示例不发起任何网络请求，请勿填写真实凭据。</p><label class="sx-field"><span>服务地址</span><input class="input" id="sxDavURL" value="'+esc(connection.url)+'"></label><label class="sx-field"><span>独立配置同步目录</span><input class="input" id="sxDavPath" value="'+esc(connection.path)+'"></label><div class="sx-grid"><label class="sx-field"><span>用户名</span><input class="input" id="sxDavUser" value="'+esc(connection.user)+'"></label><label class="sx-field"><span>连接密码（演示占位）</span><input class="input" id="sxDavPassword" type="password" placeholder="本机已保存 · 留空保留"></label></div><div class="sx-info">同步加密口令与连接密码分开。同步包不会覆盖正在使用的连接、定时规则或本机范围。</div><div class="sx-actions">'+btn('测试连接','test-connection')+'</div><p class="sx-muted" id="sxConnectionTest" role="status"></p>','保存连接',()=>{
   if(!validateConnection())return;
   connection={...connection,url:$('#sxDavURL').value.trim().replace(/\/$/,''),path:$('#sxDavPath').value.trim(),user:$('#sxDavUser').value.trim(),connected:true};closeModal();render();
  });
 }
 function validateConnection(){
  try{const u=new URL($('#sxDavURL').value.trim());if(u.protocol!=='https:'||u.username||u.password||u.search||u.hash)throw Error();if(!$('#sxDavPath').value.startsWith('/')||!$('#sxDavUser').value.trim())throw Error();return true;}catch{setText('#sxError','请填写不含内嵌凭据的 HTTPS 地址、以 / 开头的目录和用户名。');return false;}
 }
 function syncCheck(){
  if(signature(scopes.webdav)!==savedScope)return;
  if(!connection.connected){connectionDialog();return;}
  const ids=closure(scopes.webdav).ids;if(!ids.size){result={tab:'webdav',title:'尚未选择同步对象',detail:'选择范围并保存后再同步配置。'};render();return;}
  const rows=[];
  if(ids.has('template:gpt'))rows.push({id:'template:gpt',name:'GPT · Responses / v4',detail:'远端新增已保存版本 → 本机；原主版本与引用保持',kind:'download',changes:{add:1}});
  if(ids.has('channel:newapi'))rows.push({id:'channel:newapi',name:'NewApi',detail:'模型新增 3 / 更新 2 / 删除 1 · 两端修改，需选择方向',kind:'conflict',changes:{add:3,update:2,delete:1}});
  if(ids.has('skill:review'))rows.push({id:'skill:review',name:'Code Review Skill',detail:'远端更新 → 本机 · 新增 2 个资源 / 删除 1 个旧资源',kind:'download',changes:{add:2,update:1,delete:1}});
  if(ids.has('mcp:docs'))rows.push({id:'mcp:docs',name:'Docs MCP',detail:'本机已提交更新 → 远端',kind:'upload',changes:{update:1}});
  if(ids.has('preferences:desktop'))rows.push({id:'preferences:desktop',name:'通用偏好',detail:'远端显示偏好更新 → 本机',kind:'download',changes:{update:2}});
  for(const x of catalog.filter(x=>ids.has(x.id)&&x.group==='accounts'))rows.push(secretIds(scopes.webdav).accounts.includes(x.id)?{id:x.id,name:x.name,detail:scenario==='login-invalid'?'远端登录状态已失效':'两端登录状态不同 · 已自动检查，需选择方向',kind:'token',login:true}:{id:x.id,name:x.name,detail:'仅同步账户信息',kind:'account',login:false});
  diffPlan=comparisonInit({rows,choices:{},verified:scenario!=='login-invalid',scope:signature(scopes.webdav),secretCount:secretCount(scopes.webdav),accountInfoOnly:new Set(scopes.webdav.accountInfoOnly)},'diff');
  syncPreview();
 }
 function syncPreview(){
  const p=diffPlan;rememberComparison(p);
  const intro='<p class="sx-steps"><strong>比较变化</strong> → 确认同步 → 应用与核对</p><p class="sx-muted">只处理本机所选范围。展开分类查看变化，按对象选择采用方向与凭据。</p>';
  const extra=(p.rows.some(x=>x.kind==='token')?'<p id="sxTokenCheck" class="sx-muted" role="status">'+(p.verified?'已自动检查登录身份与状态；采用后自动恢复并检查可用性。':'自动检查未通过：远端登录状态已失效，可保留本机或仅同步账户信息。')+'</p>':'')+(p.secretCount?'<label class="sx-field"><span>同步加密口令</span><input type="password" id="sxSyncPass" class="input" autocomplete="off"></label>':'');
  showDialog('diff','WebDAV · 同步配置',comparisonBody(p,intro,extra),p.rows.length?'确认同步':'关闭',()=>{
   if(!p.rows.length){closeModal();return;}
   if(p.scope!==signature(scopes.webdav)){setText('#sxError','范围已变化，请保存后重新比较。');return;}
   const selected=p.rows.filter(x=>p.selected.has(x.id));
   if(!selected.length){setText('#sxError','至少选择一项变化。');return;}
   if(selected.some(x=>['conflict','token'].includes(x.kind)&&!p.choices[x.id])){setText('#sxError','请处理冲突，或取消勾选该对象。');return;}
   if(selected.some(x=>x.kind==='token'&&p.choices[x.id]==='remote'&&!p.accountInfoOnly.has(x.id))&&!p.verified){setText('#sxError','远端登录状态已失效，请保留本机或仅同步账户信息。');return;}
   if(p.secretCount&&$('#sxSyncPass').value!=='demo-sync'){setText('#sxError','同步口令不正确。连接密码不能代替同步口令。');return;}
   const adopted=selected.filter(x=>!['local','skip'].includes(p.choices[x.id]));
   const accounts=adopted.filter(x=>x.group==='accounts').map(x=>({id:x.id,login:x.kind==='token'&&!p.accountInfoOnly.has(x.id),infoOnly:p.accountInfoOnly.has(x.id)}));
   comparisonResult(p,adopted);finishApply('webdav',adopted.length,accounts,p.rows.length-adopted.length);
  });mountComparison(p);
 }
 function finishApply(mode,count,accounts=[],skipped=0,endpoints=[]){
  let title='应用流程演示完成',detail=count+' 项已按计划处理'+(skipped?'，'+skipped+' 项本次保留 / 跳过':'')+'。配置写入与读回已演示；真实运行尚未执行。';
  if(scenario==='backup-fail'){title='备份失败 · 未应用';detail='未开始写入；原配置和凭据保持，待审内容可重试。';accounts=[];endpoints=[];}
  else if(scenario==='partial'){title='部分完成 · 需要核对';detail='演示部分步骤失败：已完成与待恢复项分开记录，不能报告整体同步成功。';accounts=[];endpoints=[];}
  const accountResults=accounts.map(x=>({id:x.id,status:x.infoOnly?'info-only':!x.login||scenario==='login-invalid'?'needs-login':'restored',name:find(x.id)?.name||x.id}));
  for(const x of accountResults)detail+=' '+x.name+'：'+(x.status==='restored'?'登录状态已恢复，自动检查通过（模拟）。':x.status==='info-only'?'仅账户信息；保留本机已有登录状态，新设备需登录。':'需重新登录（'+(scenario==='login-invalid'?'登录状态已失效':'设备绑定的登录状态无法迁移')+'）。');
  const endpointResults=endpoints.map(x=>({id:x.id,status:x.password?'password-imported':'config-only',enabled:false}));
  for(const x of endpointResults)detail+=' '+(find(x.id)?.name||x.id)+'：'+(x.status==='password-imported'?'配置与连接密码已导入（模拟），连接待启用。':'仅配置；同身份保留本机密码，新连接待补密码，连接待启用。');
  result={tab:mode,title,detail,accounts:accountResults,endpoints:endpointResults,canVerify:false};serial++;closeModal();render();
 }
 function open(options={}){
  catalog=collect();tab=options.tab==='webdav'?'webdav':'file';query='';
  if(options.scope){const s=state();s.selected=new Set(options.scope.filter(id=>find(id)&&(tab==='file'||find(id).group!=='endpoints')));s.preset='custom';s.keys.clear();s.accountInfoOnly.clear();s.endpointConfigOnly.clear();s.extensions.clear();}
  setRoute('sync?tab='+tab);render(true);host.closest('.main').scrollTop=0;
  if(options.action==='import')importStart(options);else if(options.action==='export')exportPreview();else if(options.action==='connection')connectionDialog();else if(options.action==='check')syncCheck();
 }
 document.addEventListener('click',e=>{
  const el=e.target.closest('[data-sx]');if(!el||el.disabled)return;const a=el.dataset.sx,id=el.dataset.id;
  if(a==='compare-detail-close')closeComparisonDetail();
  else if(detailOpen)return;
  else if(a==='compare-detail')comparisonDetail(id);
  else if(a==='compare-expand'){const p=dialogKind==='diff'?diffPlan:importPlan;p.expanded.has(id)?p.expanded.delete(id):p.expanded.add(id);p.mode==='diff'?syncPreview():importPreview();const restore=()=>$('[data-sx="compare-expand"][data-id="'+id+'"]')?.focus({preventScroll:true});restore();requestAnimationFrame(restore);}
  else if(a==='tab'){tab=id;query='';history.replaceState(null,'','#sync?tab='+tab);render();$('#sxTab-'+tab).focus();}
  else if(a==='expand'){const folds=query?searchCollapsed[tab]:expanded[tab];folds.has(id)?folds.delete(id):folds.add(id);render();$('[data-sx="expand"][data-id="'+id+'"]',host)?.focus({preventScroll:true});}
  else if(a==='detail')detail(id);
  else if(a==='export')exportPreview();
  else if(a==='import')importStart();
  else if(a==='choose-import-file')chooseImportFile();
  else if(a==='connection')connectionDialog();
  else if(a==='test-connection'&&validateConnection())setText('#sxConnectionTest','演示：连接和目录权限检查通过。未发起网络请求。');
  else if(a==='save-scope'){savedScope=signature(scopes.webdav);result={tab:'webdav',title:'本机范围已保存（演示）',detail:'上传与接收使用这份范围。已选账户按当前选项同步登录状态；后续新增敏感类别不会自动加入，未选远端对象保留；关闭凭据不会清除历史快照。'};render();}
  else if(a==='check')syncCheck();
  else if(a==='clear'){const s=state();s.selected.clear();s.keys.clear();s.accountInfoOnly.clear();s.endpointConfigOnly.clear();s.extensions.clear();s.preset='custom';render();}
 });
 document.addEventListener('change',e=>{
  const el=e.target,s=state(),focusAttrs=[...e.target.attributes].filter(a=>a.name.startsWith('data-sx-'));const restoreFocus=()=>{const match=$$('input',dialogKind?$('#modal'):host).find(x=>focusAttrs.length&&focusAttrs.every(a=>x.getAttribute(a.name)===a.value));match?.focus({preventScroll:true});};let redraw=false;
  if(el.matches('[data-sx-preset]')){preset(el.value);redraw=true;}
  else if(el.matches('[data-sx-object]')){selectObject(s,el.dataset.sxObject,el.checked);s.preset='custom';redraw=true;}
  else if(el.matches('[data-sx-group],[data-sx-all]')){for(const x of catalog.filter(x=>(tab==='file'||x.group!=='endpoints')&&(!el.dataset.sxGroup||x.group===el.dataset.sxGroup)&&(x.name+' '+x.summary).toLowerCase().includes(query.toLowerCase()))){selectObject(s,x.id,el.checked);}s.preset='custom';redraw=true;}
  else if(el.matches('[data-sx-account-info]')){el.checked?s.accountInfoOnly.add(el.dataset.sxAccountInfo):s.accountInfoOnly.delete(el.dataset.sxAccountInfo);s.preset='custom';redraw=true;}
  else if(el.matches('[data-sx-endpoint-config]')){el.checked?s.endpointConfigOnly.add(el.dataset.sxEndpointConfig):s.endpointConfigOnly.delete(el.dataset.sxEndpointConfig);s.preset='custom';redraw=true;}
  else if(el.matches('[data-sx-secret]')){const key=el.dataset.sxSecret,prop={keys:'key',extensions:'extensionSecret'}[key];s[key].clear();if(el.checked)for(const x of catalog.filter(x=>closure().ids.has(x.id)&&x[prop]))s[key].add(x.id);s.preset='custom';redraw=true;}
  else if(el.matches('[data-sx-secret-object]')){el.checked?s[el.dataset.kind].add(el.dataset.sxSecretObject):s[el.dataset.kind].delete(el.dataset.sxSecretObject);redraw=true;}
  else if(el.id==='sxEncrypt')$('#sxPassFields').hidden=!el.checked;
  else if(el.matches('[data-sx-import-pick],[data-sx-diff-pick],[data-sx-compare-all],[data-sx-compare-group],[data-sx-compare-rule],[data-sx-compare-credential],[data-sx-import-account-info],[data-sx-diff-account-info],[data-sx-import-endpoint-config]')){
   const p=dialogKind==='diff'?diffPlan:importPlan;
   if(el.matches('[data-sx-compare-rule]'))for(const x of p.rows.filter(x=>p.selected.has(x.id)&&credentialType(p,x)&&(el.dataset.sxCompareRule==='all'||x.group===el.dataset.sxCompareRule)))setCredential(p,x,el.checked);
   else if(el.matches('[data-sx-compare-credential]'))setCredential(p,p.rows.find(x=>x.id===el.dataset.sxCompareCredential),el.checked);
   else if(el.matches('[data-sx-import-account-info],[data-sx-diff-account-info],[data-sx-import-endpoint-config]')){const id=el.dataset.sxImportAccountInfo||el.dataset.sxDiffAccountInfo||el.dataset.sxImportEndpointConfig;setCredential(p,p.rows.find(x=>x.id===id),!el.checked);}
   else if(el.matches('[data-sx-compare-all],[data-sx-compare-group]'))for(const x of p.rows.filter(x=>!el.dataset.sxCompareGroup||x.group===el.dataset.sxCompareGroup))comparisonPick(p,x,el.checked);
   else comparisonPick(p,p.rows.find(x=>x.id===(el.dataset.sxImportPick||el.dataset.sxDiffPick)),el.checked);
   p.mode==='diff'?syncPreview():importPreview();restoreFocus();requestAnimationFrame(restoreFocus);
  }
  else if(el.matches('[data-sx-import-choice]')){importPlan.choices[el.dataset.sxImportChoice]=el.value;setText('#sxError','');}
  else if(el.matches('[data-sx-diff-choice]')){diffPlan.choices[el.dataset.sxDiffChoice]=el.value;setText('#sxError','');}
  if(redraw){for(const key of ['keys','accountInfoOnly','endpointConfigOnly','extensions'])for(const id of s[key])if(!closure().ids.has(id))s[key].delete(id);render();restoreFocus();}
 });
 document.addEventListener('input',e=>{if(dialogKind&&e.target.closest('.sx-dialog')&&e.target.matches('input'))setText('#sxError','');});
 host.addEventListener('input',e=>{if(e.target.id!=='sxSearch')return;query=e.target.value;searchCollapsed[tab].clear();const pos=e.target.selectionStart;render(true);$('#sxSearch').focus({preventScroll:true});try{$('#sxSearch').setSelectionRange(pos,pos);}catch{}});
 host.addEventListener('keydown',e=>{if(e.target.getAttribute('role')==='tab'&&['ArrowLeft','ArrowRight','Home','End'].includes(e.key)){e.preventDefault();tab=e.key==='Home'?'file':e.key==='End'?'webdav':tab==='file'?'webdav':'file';history.replaceState(null,'','#sync?tab='+tab);render();$('#sxTab-'+tab).focus();}});
 document.addEventListener('keydown',e=>{if(detailOpen||!dialogKind||e.key!=='Tab')return;const items=$$('button:not([disabled]),input:not([disabled]),select:not([disabled]),[tabindex="0"]',$('#modal')).filter(x=>x.getClientRects().length);if(!items.length)return;const first=items[0],last=items.at(-1);if(e.shiftKey&&document.activeElement===first){e.preventDefault();last.focus();}else if(!e.shiftKey&&document.activeElement===last){e.preventDefault();first.focus();}});
 document.addEventListener('keydown',e=>{
  if(!detailOpen)return;
  if(e.key==='Escape'){e.preventDefault();e.stopImmediatePropagation();closeComparisonDetail();}
  else if(e.key==='Tab'){e.preventDefault();e.stopImmediatePropagation();$('#sxCompareDetailClose').focus();}
 },true);
 document.addEventListener('click',e=>{if(detailOpen&&e.target.id==='sxCompareDetail')closeComparisonDetail();});
 function routeChanged(){if(document.documentElement.dataset.route!=='sync')return;const p=new URLSearchParams(location.hash.split('?')[1]||'');tab=p.get('tab')==='webdav'?'webdav':'file';catalog=collect();render();}
 new MutationObserver(routeChanged).observe(document.documentElement,{attributes:true,attributeFilter:['data-route']});
 window.addEventListener('hashchange',routeChanged);
 window.syncPrototype={open,state:()=>({tab,selected:[...state().selected],effective:[...closure().ids],accountInfoOnly:[...state().accountInfoOnly],endpointConfigOnly:[...state().endpointConfigOnly],secrets:secretIds(),savedScope,dirty:signature(scopes.webdav)!==savedScope,lastExport,result,dialogKind,detailOpen,lastComparison,comparison:dialogKind==='diff'&&diffPlan||dialogKind==='import'&&importPlan?.mode?(()=>{const p=dialogKind==='diff'?diffPlan:importPlan;return {mode:p.mode,selected:[...p.selected],expanded:[...p.expanded],rules:Object.fromEntries(p.rows.filter(x=>credentialType(p,x)).map(x=>[x.id,credentialValue(p,x)]))};})():null,serial,scenario,importSample,importFile}),setImportSample,setScenario:v=>{if(['normal','login-invalid','backup-fail','partial'].includes(v)){scenario=v;const el=$('#sxMockScenario');if(el)el.value=v;}},render};
 routeChanged();render();
})();
