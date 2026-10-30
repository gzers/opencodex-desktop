/* 配置同步原型 · 仅内存 fixture；不读取文件/密钥，不连接 WebDAV，不实现加密。 */
(() => {
 'use strict';
 const $=(s,r=document)=>r.querySelector(s), $$=(s,r=document)=>[...r.querySelectorAll(s)];
 const esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 const clone=v=>v===undefined?undefined:JSON.parse(JSON.stringify(v));
 const host=$('#syncContent'); if(!host)return;
 const accountCapabilities=window.SyncAccountCapabilities;
 const eligible=x=>!x.history&&x.selectable!==false;
 const accountDefaults=x=>{if(x.group!=='accounts')return x;const capability=accountCapabilities.describe(x);return {...x,selectable:accountCapabilities.canSelect(x),login:accountCapabilities.canTransferLogin(x),capability,identity:accountCapabilities.identity(x),summary:(accountCapabilities.platforms[x.accountPlatform]||x.accountPlatform)+(x.email?' · '+x.email:'')+' · '+(capability.info==='supported'?'仅账户信息 · 新设备需登录':'暂不支持选择'),fields:'账户身份与显示信息；不包含登录或刷新材料',defaultExcluded:capability.info!=='supported'};};
 const groups=[['channels','渠道与模型'],['templates','模型模版'],['accounts','账户'],['skills','Skills'],['mcp','MCP'],['preferences','桌面偏好'],['endpoints','WebDAV 配置'],['dialog-policies','对话配置'],['dialog-history','对话历史']];
 function collect(source='saved'){
  const m=window.modelPrototype?.state(),list=[];
  const policy=m?.[source];
  for(const p of policy?.providers||[]){
   if(p.id==='native')continue;
   list.push({id:'channel:'+p.id,group:'channels',name:p.name,summary:p.models.filter(x=>x.selected).length+' 个已选模型 · 顺序 / 别名 / 覆盖',source:'Desktop 策略 + OpenCodex 已配置渠道',fields:'服务节点、当前节点、渠道认证、模型协议与模型清单、启停、选择顺序、别名、模版引用与人工覆盖',endpoint:p.nodes?.find(n=>n.id===p.activeNodeId)?.url||'',nodes:(p.nodes||[]).map(n=>({...n,current:n.id===p.activeNodeId})),key:true,deps:[...new Set([...Object.values(p.refs||{}),...Object.values(policy.globalRefs||{}),...p.models.map(x=>x.templateRef).filter(Boolean)])],models:p.models.filter(x=>x.selected).map(x=>x.id),files:'model-policy.json / OpenCodex 渠道配置',modelId:p.id});
  }
  list.push({id:'channel:legacy',group:'channels',name:'私有网关',summary:'2 个已配置模型 · 来自 OpenCodex',source:'OpenCodex 原有配置（非 Desktop 创建）',fields:'已有渠道、协议、端点与显式模型配置；不复制发现缓存',endpoint:'https://private.example.com/v1',key:true,deps:[],files:'OpenCodex 渠道配置'});
  for(const t of m?.templates||[]){list.push({id:'template:'+t.id,group:'templates',name:t.name,summary:t.versions.length+' 个已保存版本 · 主版本 v'+t.main,source:'Desktop 模版库',fields:'已保存版本、匹配规则、协议、能力、版本或后缀与来源；未保存缓冲不进入',files:'model-family-templates.json',versions:t.versions.map(v=>v.n),templateId:t.id});}
  list.push(
   {id:'account:chatgpt',group:'accounts',name:'ChatGPT · 个人账户',email:'personal@example.test',subject:'fixture-chatgpt-personal',sourceAccountId:'pool-chatgpt-personal',accountPlatform:'chatgpt',owner:'opencodex',provider:'chatgpt',authKind:'oauth',source:'OpenCodex账户池',files:'账户信息适配器'},
   {id:'account:chatgpt-work',group:'accounts',name:'ChatGPT · 工作账户',email:'work@example.test',subject:'fixture-chatgpt-work',sourceAccountId:'pool-chatgpt-work',accountPlatform:'chatgpt',owner:'opencodex',provider:'chatgpt',authKind:'oauth',source:'OpenCodex账户池',files:'账户信息适配器'},
   {id:'account:antigravity',group:'accounts',name:'Google Antigravity · 工作账户',email:'gravity-work@example.test',subject:'fixture-antigravity-work',sourceAccountId:'pool-antigravity-work',accountPlatform:'antigravity',owner:'opencodex',provider:'google-antigravity',authKind:'oauth',source:'OpenCodex账户池',files:'账户信息适配器'},
   {id:'account:device',group:'accounts',name:'Kiro · 设备账户',email:'builder@example.test',subject:'fixture-kiro-builder',sourceAccountId:'pool-kiro-builder',accountPlatform:'kiro',owner:'opencodex',provider:'kiro',authKind:'device',source:'OpenCodex账户池',files:'账户信息适配器'},
   {id:'account:claude',group:'accounts',name:'Claude Code · 原生账户',accountPlatform:'claude-code',owner:'platform-native',provider:'anthropic',authKind:'oauth',source:'Claude Code原生凭据',files:'未接入平台适配器'},
   {id:'account:gemini',group:'accounts',name:'Gemini CLI · 原生账户',accountPlatform:'gemini-cli',owner:'platform-native',provider:'google',authKind:'oauth',source:'Gemini CLI原生凭据',files:'未接入平台适配器'},
   {id:'account:opencode',group:'accounts',name:'OpenCode · 原生账户',accountPlatform:'opencode',owner:'platform-native',provider:'provider-specific',authKind:'oauth',source:'OpenCode原生凭据',files:'未接入平台适配器'},
   {id:'skill:review',group:'skills',name:'Code Review Skill',summary:'SKILL.md + 2 个资源文件',source:'本机扩展库',fields:'Skill 实际内容、资源、启用状态；此 fixture 不含凭据',files:'skills/code-review/**'},
   {id:'mcp:docs',group:'mcp',name:'Docs MCP',summary:'HTTP 服务器 · 完整定义',source:'本机扩展库',fields:'传输类型、地址、参数、启用状态；Authorization 需另选扩展凭据',extensionSecret:true,files:'MCP 完整定义 / extension-config.json'},
   {id:'preferences:desktop',group:'preferences',name:'通用偏好',summary:'主题、语言与可移植显示偏好',source:'Desktop',fields:'排除机器绝对路径、窗口坐标、日志、缓存、执行绑定、未提交草稿及本机文件/WebDAV范围、凭据勾选、加密选项和口令',files:'preferences.json（字段白名单）'},
   {id:'endpoint:personal',group:'endpoints',name:'个人 WebDAV',summary:'文件同步默认包含 · 密码随配置同步',source:'Desktop WebDAV 配置',fields:'服务地址、目录、用户名与连接密码；导入为未启用连接，不改当前连接或本机同步范围；不包含同步加密口令',password:true,files:'sync_endpoints / 受保护凭据部分'}
  ); return list.map(accountDefaults).concat(window.conversationPrototype?.syncObjects()||[]);
 }
 const importSamples={
  encrypted:{file:'opencodex-device-config.ocx',encrypted:true,credentials:{keys:['channel:newapi'],extensions:['mcp:docs']}},
  plaintext:{file:'opencodex-plaintext-config.ocx',encrypted:false,credentials:{keys:['channel:newapi'],extensions:['mcp:docs']}},
  keyless:{file:'opencodex-channels.ocx',encrypted:false},
  'config-only':{file:'opencodex-webdav-config.ocx',encrypted:false},
  legacy:{file:'opencodex-legacy-config.ocx',encrypted:false},
  corrupt:{file:'opencodex-damaged-config.ocx',encrypted:false}
 };
 let importSample='encrypted',importFile=null;
 function setImportSample(value){if(Object.hasOwn(importSamples,value)){importSample=value;const el=$('#sxImportSample');if(el)el.value=value;}}
 function chooseImportFile(){
  if(dialogKind!=='import')return;
  importFile={kind:importSample,...clone(importSamples[importSample])};
  refreshLocalVersions();
  const wanted=importPlan.context?new Set(importPlan.context):null;
  importPlan.rows=directImportRows(importSample).filter(x=>!wanted||wanted.has(x.id));
  importPlan.selected=new Set(importPlan.rows.map(x=>x.id));
  renderImport();
  if(importFile.encrypted)$('#sxUnlock').focus();
 }
 let detailOpen=false,lastComparison=null;
 let catalog=collect(),tab='file',query='',accountPlatform='all',originFocus=null,dialogKind='',scenario='normal',serial=0,renderedTab=null;
 const expanded={file:new Set(),webdav:new Set()},searchCollapsed={file:new Set(),webdav:new Set()},scrolls={file:{list:0,secrets:0},webdav:{list:0,secrets:0}};
 const fresh=mode=>{const items=catalog.filter(x=>eligible(x)&&!x.defaultExcluded&&(mode==='file'||x.group!=='endpoints'));return {selected:new Set(items.map(x=>x.id)),keys:new Set(mode==='webdav'?items.filter(x=>x.key).map(x=>x.id):[]),accountInfoOnly:new Set(),endpointConfigOnly:new Set(),extensions:new Set(mode==='webdav'?items.filter(x=>x.extensionSecret).map(x=>x.id):[])};};
 const scopes={file:fresh('file'),webdav:fresh('webdav')};
 const savedScopes={file:'',webdav:''};
 let connection={name:'个人 WebDAV',url:'https://dav.example.com',path:'/opencodex/config-sync/v1',user:'demo-user',connected:true},savedScope='',result=null,lastExport=null,importPlan=null,channelAdoption=null;
 let run=null,runSerial=0,lastRun=null,failedRun=null,connectionTestId=null;
 const receipts=new Map();
 // 概览只记最近一次导入 / 导出的事实：时间与项数，不写固定规则说明。
 const recentActivity={import:null,export:null};
 function activityTime(){const n=(14*60+30+runSerial*3)%(24*60);return String(Math.floor(n/60)).padStart(2,'0')+':'+String(n%60).padStart(2,'0');}
 function recordActivity(kind,count,complete=true,extra={}){
  recentActivity[kind]={time:activityTime(),count,complete};
  document.dispatchEvent(new CustomEvent('prototype:sync-activity',{detail:{kind,...recentActivity[kind],...extra}}));
 }
 function publishReceipt(eventId,receipt){
  receipts.set(String(receipt.operationId),clone(receipt));
  window.prototypeNotificationPublisher.publish(eventId,{operationId:String(receipt.operationId),total:receipt.total??receipt.items?.length??0,downloaded:receipt.downloaded||0,uploaded:receipt.uploaded||0,skipped:receipt.skipped||0,failed:receipt.failed?.length||0,errorCode:receipt.errorCode||'none'});
 }
 const localFingerprints={},versions={local:{},remote:{}},protection={enabled:false,passphrase:''},importCredentialPrefs={keys:true,extensions:true},demoObjects={};
 const stamp=n=>'2026-10-30T'+String(9+Math.floor(n/60)).padStart(2,'0')+':'+String(n%60).padStart(2,'0')+':00+08:00';
 function seedVersions(scene='normal'){
  for(const x of catalog.filter(eligible)){
   const local={updatedAt:stamp(0),revision:1,digest:x.id+':v1'};let remote=clone(local);
   if(scene==='download'||scene==='normal'&&['channel:newapi','template:gpt','skill:review','preferences:desktop'].includes(x.id))remote={updatedAt:stamp(5),revision:2,digest:x.id+':v2'};
   if(scene==='upload'||scene==='normal'&&x.id==='mcp:docs')Object.assign(local,{updatedAt:stamp(6),revision:2,digest:x.id+':local-v2'});
   versions.local[x.id]=local;versions.remote[x.id]=remote;
   demoObjects[x.id]={revision:1,files:x.group==='skills'?['SKILL.md','scripts/review.js','scripts/legacy.js']:null};
  }
 }
 seedVersions();
 function fingerprint(id){const model=window.modelPrototype?.state();if(id.startsWith('channel:'))return JSON.stringify(model?.saved.providers.find(p=>p.id===id.slice(8)));if(id.startsWith('template:'))return JSON.stringify(model?.templates.find(t=>t.id===id.slice(9)));return null;}
 function refreshLocalVersions(){for(const x of catalog){const value=fingerprint(x.id);if(value===null)continue;if(localFingerprints[x.id]!==undefined&&localFingerprints[x.id]!==value){const revision=(versions.local[x.id]?.revision||1)+1;versions.local[x.id]={updatedAt:stamp(10+revision),revision,digest:x.id+':local-edit-'+revision};}localFingerprints[x.id]=value;}}
 refreshLocalVersions();
 function direction(id){const a=versions.local[id],b=versions.remote[id];if(!a)return 'download';if(!b)return 'upload';if(a.digest===b.digest)return 'skip';return Date.parse(a.updatedAt)>=Date.parse(b.updatedAt)?'upload':'download';}
 function versionMark(id){const value=direction(id);return value==='download'?'远端较新':value==='upload'?'本机较新':'已一致';}
 const state=()=>scopes[tab], find=id=>catalog.find(x=>x.id===id);
 const setText=(selector,value)=>{const el=$(selector);if(el)el.textContent=value;};
 const btn=(text,action,attrs='',primary=false)=>'<button type="button" class="btn '+(primary?'primary':'ghost')+'" data-sx="'+action+'" '+attrs+'>'+text+'</button>';
 const check=(attr,label,checked=false,disabled=false)=>'<label class="sx-check"><input type="checkbox" '+attr+(checked?' checked':'')+(disabled?' disabled':'')+'><span>'+label+'</span></label>';
 function closure(s=state()){
  const deps=new Map();
  for(const id of s.selected){for(const ref of find(id)?.deps||[]){const split=ref.lastIndexOf(':'),tid=ref.slice(0,split),revision=Number(ref.slice(split+1));if(!tid)continue;const key='template:'+tid;if(!deps.has(key))deps.set(key,new Set());deps.get(key).add(revision);}}
  return {deps,ids:new Set([...s.selected,...deps.keys()].filter(id=>find(id)&&eligible(find(id))))};
 }
 function secretIds(s=state()){
  const selected=closure(s).ids;
  return {keys:[...s.keys].filter(id=>selected.has(id)&&find(id)?.key),accounts:[...selected].filter(id=>find(id)?.login&&!s.accountInfoOnly.has(id)),extensions:[...s.extensions].filter(id=>selected.has(id)&&find(id)?.extensionSecret),endpointPasswords:[...selected].filter(id=>find(id)?.password&&!s.endpointConfigOnly.has(id))};
 }
 const secretCount=s=>Object.values(secretIds(s)).flat().length;
 const signature=s=>JSON.stringify({ids:[...s.selected].sort(),accountInfoOnly:[...s.accountInfoOnly].filter(id=>s.selected.has(id)).sort(),endpointConfigOnly:[...s.endpointConfigOnly].filter(id=>s.selected.has(id)).sort(),...Object.fromEntries(Object.entries(secretIds(s)).map(([k,v])=>[k,v.sort()]))});
 rememberScope();
 function selectObject(s,id,checked){
  if(find(id)&&!eligible(find(id)))return;
  if(checked){if(!s.selected.has(id)){s.accountInfoOnly.delete(id);s.endpointConfigOnly.delete(id);}s.selected.add(id);}
  else{s.selected.delete(id);s.accountInfoOnly.delete(id);s.endpointConfigOnly.delete(id);}
 }
 function sxAnn(num,title,body){
  return '<div class="proto-annotation" id="ann-sx-'+num+'" data-annotation-number="'+num+'" data-annotation-title="'+title+'" data-annotation-category="实现边界 · 同步"><button type="button" class="annotation-marker" aria-expanded="false" aria-controls="annotationPopover" aria-label="原型注释 '+num+'：'+title+'"><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 2.5h7l3 3v8H3zM10 2.5v3h3M5.5 8h5M5.5 10.5h3"/></svg><span>原型</span><span class="annotation-number">'+num+'</span><span class="annotation-marker-plus" aria-hidden="true">＋</span></button><div class="annotation-source" hidden><p>'+body+'</p><div class="annotation-footer">实现边界 · 同步</div></div></div>';
 }
 function scopeHTML(){
  const scopeLabel=tab==='file'?'导出范围':'同步范围';
  const s=state(),{ids,deps}=closure(),visible=catalog.filter(x=>(tab==='file'||x.group!=='endpoints')&&(x.name+' '+x.summary).toLowerCase().includes(query.toLowerCase())&&(x.group!=='accounts'||accountPlatform==='all'||x.accountPlatform===accountPlatform));
  const rows=groups.map(([id,label])=>{
   const items=visible.filter(x=>x.group===id);if(!items.length)return '';
   const selectable=items.filter(eligible),all=selectable.length>0&&selectable.every(x=>ids.has(x.id)),count=items.filter(x=>ids.has(x.id)).length,open=query?!searchCollapsed[tab].has(id):expanded[tab].has(id);
   const children=window.PrototypeTreeTable.rows(open?items:[],x=>{
    const dependent=deps.has(x.id)&&!s.selected.has(x.id),included=Object.values(secretIds(s)).some(items=>items.includes(x.id));
    let mark=tab==='webdav'&&eligible(x)?versionMark(x.id):x.history?'本地恢复包中选择':dependent?'依赖补齐':x.group==='accounts'?(x.selectable===false?'暂不支持':x.login&&!s.accountInfoOnly.has(x.id)?'含登录状态':'仅账户信息'):x.password?(included?'含密码':'仅配置'):included?'含凭据':x.key?'不带 Key':'配置';
    return {className:'sx-object',depth:1,label:x.name,
      selection:'<input type="checkbox" data-sx-object="'+esc(x.id)+'" aria-label="选择 '+esc(x.name)+'" '+(ids.has(x.id)?'checked':'')+(x.selectable===false?' disabled title="'+esc(x.capability?.reason||'未支持')+'"':x.history?' disabled title="使用本地重装恢复流程；当前未接入远端历史交换"':dependent?' disabled title="所选渠道需要此精确版本"':'')+'>',
      nameHtml:'<button class="sx-object-name" data-sx="detail" data-id="'+esc(x.id)+'">'+esc(x.name)+'</button><small>'+esc(dependent?'仅精确版本 v'+[...deps.get(x.id)].join(' / v'):x.password?(s.endpointConfigOnly.has(x.id)?'仅配置 · 不含连接密码':'密码随配置同步 · 导入后待启用'):x.summary)+'</small>',
      cells:['<span class="sx-mark '+(included?'secret':x.group==='accounts'&&!x.login?'warn':'')+'">'+mark+'</span>']};
   });
   return '<div class="sx-group">'+window.PrototypeTreeTable.row({className:'sx-group-head',label,
     selection:'<input type="checkbox" data-sx-group="'+id+'" aria-label="选择'+label+'" '+(all?'checked':'')+(selectable.length?'':' disabled')+'>',
     toggleAttrs:'data-sx="expand" data-id="'+id+'"',expanded:open,
     cells:['<span class="sx-group-count">'+(selectable.length?count+' / '+selectable.length:id==='accounts'?'暂无可选能力':'本地恢复')+'</span>']})+(id==='accounts'&&open?'<label class="sx-account-filter">账户平台<select data-sx-account-platform aria-label="账户列表平台筛选"><option value="all">全部平台</option>'+Object.entries(accountCapabilities.platforms).map(([key,label])=>'<option value="'+key+'" '+(accountPlatform===key?'selected':'')+'>'+label+'</option>').join('')+'</select></label>':'')+children+'</div>';
  }).join('');
  return '<div class="sx-toolbar"><input class="input" id="sxSearch" type="search" aria-label="搜索配置对象" placeholder="搜索配置对象" value="'+esc(query)+'"></div><div class="sx-table proto-tree" role="region" aria-label="配置对象列表" tabindex="0">'+window.PrototypeTreeTable.row({className:'sx-table-head',label:tab==='file'?'导出范围':'配置对象',selection:'<input type="checkbox" data-sx-all aria-label="选择当前搜索结果" '+(visible.filter(eligible).length&&visible.filter(eligible).every(x=>ids.has(x.id))?'checked':'')+'>',cells:['<span>范围与凭据</span>']})+''+(rows||'<div class="sx-empty">没有匹配的配置对象<br>更换关键词或清除搜索。</div>')+'</div>'+sxAnn('01','同步范围与选择规则','范围就是最后一次的勾选结果，文件导出与 WebDAV 各记一份，下次打开直接沿用；表头复选框可一键全选或取消当前搜索结果。同平台支持多个独立账户；账户平台与OpenCodex管理方、调用方分开，Google Antigravity、Kiro不归类为Codex。账户能力按账户平台、管理方、授权来源和认证类型登记；当前正常流程仅账户信息。默认导出当前已保存文件快照；仅显式指定 draft 来源才导出当前模型渠道内存草稿。模版只取已保存版本，不带模版编辑缓冲。搜索仅改变显示，隐藏勾选保留。凭据随所选范围/包内能力选择，默认不加密。普通同步不传输这些操作偏好，也不切换另一台设备的当前连接。每次导入在包内提供范围默认全选，显式入口范围只列对应对象。点击同步才执行。原型只记在当前文档内存，刷新后复位；不读写真实文件。');
 }
 function credentialsHTML(){
  const s=state(),ids=closure().ids;
  const reviewed='';
  const hasAssets=[...ids].some(id=>['skills','mcp'].includes(find(id)?.group));
  const blocks=[['keys','渠道 API Key','包含所选渠道的 Key','渠道可单独分享，接收方自行填 Key。',x=>x.key],['extensions','扩展凭据','包含所选扩展的凭据','例如 MCP 的 Authorization / 环境变量。',x=>x.extensionSecret]].map(([key,title,label,note,match])=>{
   const items=catalog.filter(x=>ids.has(x.id)&&match(x)),active=items.some(x=>s[key].has(x.id));
   return '<section class="sx-secret-block">'+check('data-sx-secret="'+key+'"',title,active,!items.length)+(key==='extensions'&&hasAssets?reviewed:'')+'<p class="sx-muted">'+note+'</p>'+(active?'<div class="sx-subpicks">'+items.map(x=>check('data-sx-secret-object="'+x.id+'" data-kind="'+key+'"',esc(x.name),s[key].has(x.id))).join('')+'</div>':'')+'</section>';
  }).join('');
  return '<aside class="sx-secrets"><header><h3>其他凭据</h3><p class="sx-muted">账户按平台能力提供可选范围；当前仅账户信息'+(tab==='file'?'，WebDAV 密码随配置选择':'')+'；以下凭据可另选。</p></header>'+ (tab==='webdav'?'<section class="sx-secret-block">'+check('data-sx-protect','加密同步内容',protection.enabled)+'<div '+(!protection.enabled?'hidden':'')+' id="sxProtectionFields"><label class="sx-field"><span>保护口令</span><input class="input" type="password" id="sxProtectionPass" autocomplete="off" placeholder="填写后记住本次选择" value="'+esc(protection.passphrase)+'"></label>'+(result?.inputNeeded?'<p class="sx-error" role="alert">请填写保护口令</p>':'')+'</div><p class="sx-muted">可选，与连接密码独立。</p></section>':'')+blocks+'<p class="sx-note">'+(secretCount()?'所选凭据随范围同步':'当前不携带凭据')+'</p>'+sxAnn('02','凭据选择与加密规则','加密口令独立于 WebDAV 密码，不随包或远端数据保存。导入同一身份时保留本机已有凭据，不用空值覆盖。文件导出与WebDAV默认不加密，含凭据也不自动勾选；仅手动开启加密后填写口令，不增加确认步骤。WebDAV连接密码继续复用；普通同步不要求逐次复核。原型不读取真实密钥、不做真实加密。')+'</aside>';
 }
 function render(resetList=false){
  if(document.documentElement.dataset.route==='sync')host.closest('.main').scrollTop=0;
  if(renderedTab){scrolls[renderedTab].list=$('.sx-table',host)?.scrollTop||0;scrolls[renderedTab].secrets=$('.sx-secrets',host)?.scrollTop||0;}
  if(resetList)scrolls[tab].list=0;
  const selected=closure().ids.size,retry=result?.tab===tab&&result.failed?.length?btn('重试失败项','retry',run?'disabled':''):'';
  host.innerHTML='<div class="sx-tabs" data-page-tabs data-tabs-scroll="inner" role="tablist" aria-label="同步方式">'+[['file','文件同步'],['webdav','WebDAV 同步']].map(([id,name])=>'<button type="button" role="tab" id="sxTab-'+id+'" aria-controls="sxPanel" aria-selected="'+(tab===id)+'" tabindex="'+(tab===id?'0':'-1')+'" data-sx="tab" data-id="'+id+'">'+name+'</button>').join('')+'</div>'+
   '<div id="sxPanel" role="tabpanel" aria-labelledby="sxTab-'+tab+'"><header class="sx-top"><div><h2>'+(tab==='file'?'导入与导出':'设备间同步')+'</h2><p class="sx-muted">'+(tab==='file'?'按下方范围导出配置；导入时在所选文件内选择范围。':'按所选范围双向更新，较新的内容覆盖较旧的内容。')+'</p></div><div class="sx-actions">'+(tab==='file'?(result?.tab===tab&&result.items?.length?btn('查看结果','result-details'):'')+retry+btn('导入配置','import')+btn('导出配置','export',selected?'':'disabled',true):(result?.tab===tab&&result.items?.length?btn('查看结果','result-details'):'')+retry+btn(run?'同步中…':'同步','check',run||!selected?'disabled':'',true))+'</div></header>'+
   (tab==='webdav'?'<div class="sx-connection"><div class="sx-connection-info"><div><strong>'+esc(connection.name)+'</strong></div><small class="sx-muted" title="'+esc(connection.url+connection.path)+'">'+esc(connection.url+connection.path)+'</small></div>'+btn('连接设置','connection')+'</div>'+sxAnn('03','按版本直接更新 · 方案演示','S18-01/03/04/05：范围和凭据选择在当前文档自动记住，普通同步直接写mock文件，不再比较、采用或二次保存。Skill整体更新并移除本Skill旧文件；官方openai忽略。右侧结果场景可演示上传、下载、无变化和失败重试。首次无版本远端优先、整对象删除传播、设备专属字段与未保存编辑仍待确认；本轮不演示删除用户对象，也不把建议写为生产结论。'):'') +
   '<div class="sx-layout"><section class="sx-scope" aria-label="'+(tab==='file'?'导出范围':'同步范围')+'">'+scopeHTML()+'</section>'+credentialsHTML()+'</div>' +
   (run&&run.tab===tab?'<div class="sx-result" role="status" aria-live="polite"><strong>'+(run.tab==='file'?'正在导入':'正在同步')+'</strong><span>正在更新所选 '+run.ids.length+' 项内容…</span></div>':'') +
   '</div>';
  if(window.conversationPrototype?.renderRecovery($('#sxPanel',host),tab)){renderedTab=tab;return;}
  if(tab==='file'&&window.conversationPrototype?.scope()==='conversations')$('.sx-layout',host).insertAdjacentHTML('beforebegin',window.conversationPrototype.recoveryEntry());
  $('.sx-table',host).scrollTop=scrolls[tab].list;$('.sx-secrets',host).scrollTop=scrolls[tab].secrets;renderedTab=tab;
  for(const el of $$('[data-sx-group]',host)){const items=catalog.filter(x=>eligible(x)&&x.group===el.dataset.sxGroup&&(x.name+' '+x.summary).toLowerCase().includes(query.toLowerCase())&&(x.group!=='accounts'||accountPlatform==='all'||x.accountPlatform===accountPlatform)),n=items.filter(x=>closure().ids.has(x.id)).length;el.indeterminate=n>0&&n<items.length;}
  for(const el of $$('[data-sx-secret]',host)){const key=el.dataset.sxSecret,prop={keys:'key',extensions:'extensionSecret'}[key],items=catalog.filter(x=>closure().ids.has(x.id)&&x[prop]),n=items.filter(x=>state()[key].has(x.id)).length;el.indeterminate=n>0&&n<items.length;}
  const all=$('[data-sx-all]',host),shown=catalog.filter(x=>eligible(x)&&(tab==='file'||x.group!=='endpoints')&&(x.name+' '+x.summary).toLowerCase().includes(query.toLowerCase())&&(x.group!=='accounts'||accountPlatform==='all'||x.accountPlatform===accountPlatform)),n=shown.filter(x=>closure().ids.has(x.id)).length;if(all)all.indeterminate=n>0&&n<shown.length;
 }
 function showDialog(kind,title,body,confirm,handler,opts={}){
  if(!dialogKind)originFocus=document.activeElement;
  dialogKind=kind;
  openModal(title,'<div class="sx-dialog">'+body+'<div class="sx-error" id="sxError" role="alert"></div></div>',confirm,handler,{confirmKeepsOpen:true,hideNote:true,...opts});
  const annotation=$('#ann-modal-demo .annotation-source');
  if(opts.annotationText&&annotation)for(const text of [opts.annotationText].flat()){
   const paragraph=document.createElement('p');paragraph.textContent=text;annotation.querySelector('.annotation-footer').before(paragraph);
  }
  $('#modal').classList.remove('sx-comparison');$('#modal').classList.add('modal-wide');$('#modalConfirm').disabled=false;
  window.__modalOnClose=()=>{for(const el of [$('.modal-head'),$('.modal-actions')])if(el)el.inert=false;detailOpen=false;$('#modal').classList.remove('sx-comparison');dialogKind='';$('#modalBody').innerHTML='';$('#modalConfirm').disabled=false;if(originFocus?.isConnected)originFocus.focus();originFocus=null;};
 }
 function detail(id){
  const x=find(id);if(!x)return;
  if(x.history){window.conversationPrototype?.openRecovery('export','thread:'+id.slice('dialog-history:'.length));return;}
  const accountOptions=x.group==='accounts'&&!x.login?'<details class="sx-account-options" open><summary>同步选项</summary>'+check('aria-label="仅账户信息"','仅账户信息',true,true)+check('aria-label="同步登录状态"','同步登录状态',false,true)+'<p class="sx-muted">'+esc(x.capability.reason)+'</p></details>':x.login?'<details class="sx-account-options"><summary>同步选项</summary>'+check('data-sx-account-info="'+esc(id)+'"','仅同步账户信息，不含登录状态',state().accountInfoOnly.has(id),!state().selected.has(id))+'<p class="sx-muted">'+(state().selected.has(id)?'默认随账户同步登录状态；修改后立即计入当前范围。':'先勾选此账户，再调整同步选项。')+'</p></details>':'';
  if(x.group==='accounts'){
   showDialog('detail',x.name,'<dl class="sx-detail-list"><dt>邮箱 / 标签</dt><dd>'+esc(x.email||x.name)+'</dd><dt>来源账户</dt><dd>'+esc(x.sourceAccountId||x.id)+'</dd><dt>账户平台</dt><dd>'+esc(accountCapabilities.platforms[x.accountPlatform]||x.accountPlatform)+'</dd><dt>管理方</dt><dd>'+esc(x.source)+'</dd><dt>授权来源 / 认证</dt><dd>'+esc(x.provider+' / '+x.authKind)+'</dd><dt>可选范围</dt><dd>'+esc(x.selectable===false?'暂不支持选择':'仅账户身份与显示信息')+'</dd></dl>'+accountOptions,'关闭',closeModal,{cancelLabel:'关闭',hideConfirm:true,annotationText:'账户平台与能力门已获本轮原型确认。对象、状态均为固定原型资料；不读取真实账户或登录材料。生产按已确认的适配器与真实证据登记，不能由原型成功状态晋升支持。'});return;
  }
  const endpointOptions=x.password?'<details class="sx-account-options"><summary>同步选项</summary>'+check('data-sx-endpoint-config="'+esc(id)+'"','仅同步配置，不含密码',state().endpointConfigOnly.has(id),!state().selected.has(id))+'<p class="sx-muted">默认随配置同步连接密码；导入后需另行启用连接。同步加密口令始终不进入配置包。</p></details>':'';
  showDialog('detail',x.name,'<p class="sx-muted">配置对象详情 · 敏感值不展示</p><dl class="sx-detail-list"><dt>归属</dt><dd>'+esc(x.source)+'</dd>'+(x.group==='accounts'?'<dt>账户平台</dt><dd>'+esc(accountCapabilities.platforms[x.accountPlatform]||x.accountPlatform)+'</dd><dt>授权来源 / 认证</dt><dd>'+esc(x.provider+' / '+x.authKind)+'</dd><dt>登录迁移</dt><dd>'+esc(x.capability.reason)+'</dd>':'')+'<dt>包含字段</dt><dd>'+esc(x.fields)+'</dd><dt>配置来源</dt><dd>'+esc(x.files)+'</dd>'+(x.endpoint?'<dt>当前连接地址</dt><dd>'+esc(x.endpoint)+'</dd>':'')+(x.nodes?.length?'<dt>服务节点</dt><dd>'+x.nodes.map(n=>esc(n.name)+(n.current?' · 当前':'')+'：'+esc(n.url||'待配置')).join('<br>')+'</dd>':'')+(versions.local[id]?'<dt>内容版本</dt><dd>v'+versions.local[id].revision+' · '+esc(versions.local[id].updatedAt.replace('T',' ').replace('+08:00',''))+'</dd>':'')+(demoObjects[id]?.files?'<dt>当前文件</dt><dd>'+demoObjects[id].files.map(esc).join('<br>')+'</dd>':'')+'<dt>依赖</dt><dd>'+esc(x.deps?.length?x.deps.join('、'):'无额外模版依赖')+'</dd>'+(x.models?.length?'<dt>已选模型</dt><dd>'+x.models.map(esc).join('、')+'</dd>':'')+'<dt>凭据</dt><dd>'+esc(x.key?'API Key 单独选择；不显示或复制掩码值':x.login?'登录状态默认随账户同步，接收端自动恢复并检查':x.group==='accounts'?'当前只同步已支持的账户信息；本机已有登录保持，新设备需登录':x.extensionSecret?'授权头 / 环境变量单独选择':x.password?'连接密码默认随配置同步，可选仅配置；不包含同步加密口令':'不携带凭据')+'</dd></dl>'+accountOptions+endpointOptions+(['accounts','endpoints','dialog-policies'].includes(x.group)?'':'<p class="sx-note">同一渠道原位更新，官方 openai 保持本机自带能力。</p>'),'关闭',closeModal,{cancelLabel:'关闭',hideConfirm:true});
 }
 function exportSnapshot(source,cl,secrets){
  const m=window.modelPrototype?.state(),policy=m?.[source];
  const providers=(policy?.providers||[]).filter(p=>p.id!=='native'&&cl.ids.has('channel:'+p.id)).map(p=>{
   const item=clone(p);
   for(const key of ['refresh','refreshToken','updated','availability','addedInPrototype'])delete item[key];
   item.models=item.models.map(model=>{const next=clone(model);delete next.official;return next;});
   if(item.connection&&!secrets.keys.includes('channel:'+p.id)){delete item.connection.credentialRef;item.connection.keyConfigured=false;}
   return item;
  });
  return {modelChannels:providers.length?{providers,globalRefs:clone(policy.globalRefs),fastMode:policy.fastMode,fastRows:policy.fastRows}:null,templates:(m?.templates||[]).filter(t=>cl.ids.has('template:'+t.id)).map(t=>({...clone(t),versions:t.versions.filter(v=>state().selected.has('template:'+t.id)||cl.deps.get('template:'+t.id)?.has(v.n)).map(clone)}))};
 }
 function exportPreview(source='saved'){
  // 仅显式 source:'draft' 使用当前内存渠道；普通入口始终重新取已保存文件快照。
  source=source==='draft'?'draft':'saved';catalog=collect(source);
  if(!closure().ids.size)return;
  const cl=closure(),secrets=secretIds(),count=secretCount();
  const manifest={demo:true,kind:'configuration-sync',objects:[...cl.ids],dependencies:[...cl.deps].map(([id,versions])=>({id,revisions:[...versions]})),credentials:secrets,accounts:[...cl.ids].map(find).filter(x=>x?.group==='accounts').map(x=>({id:x.id,accountPlatform:x.accountPlatform,owner:x.owner,provider:x.provider,sourceAccountId:x.sourceAccountId,subject:x.subject||null,tenant:x.tenant||null,label:x.name,email:x.email||null})),source:source==='draft'?'draft':'committed',snapshotSource:source,encryptionRequired:false,snapshot:exportSnapshot(source,cl,secrets),modules:[...cl.ids].filter(id=>find(id)?.group==='dialog-policies').map(id=>({id,schemaVersion:1,content:clone(find(id).payload)}))};
  const destination='<div class="sx-field"><span>保存位置</span><div class="sx-destination" role="group" aria-label="保存位置"><label class="sx-check"><input type="radio" name="sxDestination" value="default" checked><span>默认导出目录</span></label><label class="sx-check"><input type="radio" name="sxDestination" value="custom"><span>自定义目录</span></label></div><div id="sxCustomDir" hidden><div class="sx-dir-row"><input id="sxDirPath" class="input" type="text" readonly placeholder="尚未选择目录"><button type="button" class="btn ghost" data-sx="choose-dir">选择目录…</button></div><p class="sx-muted">未选择目录时按默认导出目录保存。</p></div></div>';
  const scopesLabel=groups.filter(([id])=>[...cl.ids].some(key=>find(key)?.group===id)).map(([,label])=>label).join(' / ');
  const packagePath='<div class="sx-export-layout"><div class="sx-export-tree" aria-label="导出目录结构"><p class="sx-export-kicker">生成位置</p><code>导出目录/<br>└── 2026-10-23/<br>&nbsp;&nbsp;&nbsp;&nbsp;└── 备份包-14-30-12-8F3A/<br>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;├── 本次所选配置…<br>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;├── 模版与所选版本<br>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;└── manifest / 校验信息</code><p class="sx-muted">按日期归组，一次导出一个可独立拷贝的备份包。同日多次导出各自建包，不混写、不覆盖。</p></div><div class="sx-export-summary"><strong>'+cl.ids.size+' 项配置 · 同一备份包</strong><p class="sx-muted">'+(count?'含 '+count+' 项凭据；可选择加密，不加密则凭据以明文写入，文件可被直接读取。口令始终不写入包。':'不含凭据，可直接使用完整性校验。')+'复制整个备份包即可带到另一台设备。</p><p class="sx-muted">包内按 '+scopesLabel+' 分组，导入时仍按所选范围识别。</p></div></div>';
  const body=destination+packagePath+'<div class="sx-info"><strong>'+cl.ids.size+' 项配置 · '+(count?'含 '+count+' 项凭据':'不含任何凭据')+'</strong><br>渠道 Key '+secrets.keys.length+' 项 / 账户登录态 '+secrets.accounts.length+' 项 / 扩展凭据 '+secrets.extensions.length+' 项 / WebDAV 密码 '+secrets.endpointPasswords.length+' 项</div>'+check('id="sxEncrypt"','加密配置包',false,false)+'<div id="sxPassFields" hidden><div class="sx-grid"><label class="sx-field"><span>加密口令（至少 8 位）</span><input id="sxPass" class="input" type="password" autocomplete="new-password"></label><label class="sx-field"><span>再次输入</span><input id="sxPassAgain" class="input" type="password" autocomplete="new-password"></label></div></div><details><summary>查看内容清单</summary><pre>'+esc(JSON.stringify(manifest,null,2))+'</pre></details>';
  showDialog('export','确认导出',body,'生成配置包',()=>{
   const encrypted=$('#sxEncrypt').checked,p=$('#sxPass').value;if(encrypted&&(p.length<8||p!==$('#sxPassAgain').value)){setText('#sxError','请输入至少 8 位的加密口令，并保持两次一致。');return;}// 文件保护可选；明文说明已在当前确认中展示，不增加确认步骤。
   const custom=$('[name=sxDestination]:checked')?.value==='custom';const dir=custom?($('#sxDirPath')?.value.trim()||''):'';if(custom&&!dir){setText('#sxError','请选择自定义目录，或改回默认导出目录。');return;}const exportRoot=dir||'默认导出目录',packagePathShown=exportRoot+'/2026-10-23/备份包-14-30-12-8F3A',operationId=++runSerial;
   const failed=scenario==='backup-fail',receipt={tab:'file',kind:'export',operationId,total:cl.ids.size,packagePath:failed?null:packagePathShown,protection:{encrypted},failed:failed?[{id:'export',reason:'目标目录无法写入'}]:[],errorCode:failed?'write':'none'};
   if(!failed)lastExport={...manifest,encrypted,destination:dir||'default',packagePath:packagePathShown,simulated:true};
   closeModal();render();recordActivity('export',cl.ids.size,!failed);publishReceipt(failed?'sync.export.failed':'sync.export.complete',receipt);
  },{annotationText:['选择范围 → 确认内容 → 生成配置包','未选择的凭据不进入包，也不会由模版依赖重新带入。']});
 }
 // 普通导入与同步直接更新已选对象；比较仅在可选结果详情中查看。
 function updateRows(){return catalog.filter(eligible).map(x=>({id:x.id,name:x.name,group:x.group,direction:direction(x.id),local:clone(versions.local[x.id]),remote:clone(versions.remote[x.id])}));}
 function rememberScope(){savedScopes.file=signature(scopes.file);savedScope=savedScopes.webdav=signature(scopes.webdav);}
 function directImportRows(kind){
  const present=new Set(['template:gpt','channel:newapi','channel:legacy','skill:review','mcp:docs','preferences:desktop','endpoint:personal','account:chatgpt','account:chatgpt-work','account:antigravity','account:device']);
  const items=updateRows().filter(x=>present.has(x.id)).map(x=>({...x,direction:x.direction==='download'?'download':'skip'}));
  if(kind==='keyless')return items.filter(x=>['channels','templates'].includes(x.group));
  if(kind==='config-only')return items.filter(x=>x.group==='endpoints');
  if(kind==='legacy')return items.filter(x=>x.group==='preferences');
  return items;
 }
 function importForm(){
  const rows=importPlan.rows||[];
  return '<div class="sx-field"><span>配置文件</span><div class="sx-file-picker">'+btn('选择文件','choose-import-file')+'<span id="sxImportFileName" aria-live="polite">'+esc(importFile?.file||'尚未选择文件')+'</span></div></div>'+
   '<label class="sx-field" id="sxUnlockField" '+(!importFile?.encrypted?'hidden':'')+'><span>解密口令</span><input class="input" id="sxUnlock" type="password" autocomplete="off" placeholder="请输入配置包的加密口令"></label>'+
   (rows.length?'<fieldset class="sx-import-scope"><legend>导入范围</legend>'+groups.map(([group,label])=>{const items=rows.filter(x=>x.group===group);return items.length?'<div class="sx-import-group"><strong>'+label+'</strong>'+items.map(x=>check('data-sx-import-object="'+esc(x.id)+'"',esc(x.name),importPlan.selected.has(x.id))).join('')+'</div>':'';}).join('')+'</fieldset>'+check('data-sx-import-keys','包含包内渠道 API Key',importPlan.keys,!importSecretIds('keys').length)+check('data-sx-import-secrets','包含包内扩展凭据',importPlan.extensions,!importSecretIds('extensions').length):'');
 }
 function importSecretIds(kind){
  return (importFile?.credentials?.[kind]||[]).filter(id=>importPlan.selected.has(id)&&importPlan.rows.some(x=>x.id===id));
 }
 function syncImportControls(){
  for(const [kind,attr] of [['keys','keys'],['extensions','secrets']]){
   const input=$('[data-sx-import-'+attr+']');if(!input)continue;
   input.disabled=!importSecretIds(kind).length;input.checked=!input.disabled&&!!importPlan[kind];
  }
  $('#modalConfirm').disabled=!importFile||!importPlan.selected.size;
 }
 function renderImport(){
  showDialog('import','导入配置',importForm(),'导入',()=>{
   if(!importFile)return;
   if(importFile.kind==='corrupt'){setText('#sxError','文件无法读取，请重新选择配置包。');return;}
   if(importFile.encrypted&&$('#sxUnlock').value!=='demo-sync'){setText('#sxError','解密口令不正确，请重新输入。');return;}
   if(!importPlan.selected.size)return;
   const rows=importPlan.rows.filter(x=>importPlan.selected.has(x.id));
   const secrets={keys:importPlan.keys?importSecretIds('keys'):[],extensions:importPlan.extensions?importSecretIds('extensions'):[]};
   closeModal();startUpdate('file',rows,secrets);
  },{prototypeAnnotation:{title:'导入示例与测试口令',body:'加密示例包的解密口令：demo-sync。选择示例文件后，填入“解密口令”框即可；不加密的示例包无需口令。此口令仅供原型模拟，不是WebDAV连接密码，也不用于真实文件。'},annotationText:['S18方案演示：选择文件与范围 → 导入并直接更新；不会读取真实文件。','官方openai不进入导入范围。同一渠道按固定身份原位更新，不创建导入副本。凭据选项由包内实际内容与已选对象决定，与是否加密无关；未勾选凭据表示保留本机已有值。','未保存编辑的处理仍待定，本轮示例更新mock已保存文件并保留原编辑缓冲；不把此建议当已确认生产规则。']});
  syncImportControls();
 }
 function importStart(context={}){importFile=null;importPlan={rows:[],selected:new Set(),keys:importCredentialPrefs.keys,extensions:importCredentialPrefs.extensions,context:context.scope||null};renderImport();}
 function startUpdate(mode,rows,secrets=secretIds(scopes.webdav),transferProtection,retryOperationId){
  if(run)return;
  const encrypted=transferProtection?.encrypted??(mode==='webdav'?protection.enabled:!!importFile?.encrypted);
  if(mode==='webdav'&&encrypted&&!protection.passphrase){result={tab:mode,title:'请填写保护口令',detail:'填写后再同步。',inputNeeded:true};render();$('#sxProtectionPass')?.focus();return;}
  result=null;channelAdoption=null;
  const operation={id:retryOperationId||++runSerial,tab:mode,ids:rows.map(x=>x.id),rows:clone(rows),secrets:clone(secrets),protection:{encrypted},scenario};run=operation;render();
  window.prototypeNotificationPublisher.publish(mode==='file'?'sync.import.started':'sync.started',{operationId:String(operation.id),total:rows.length});
  setTimeout(()=>completeUpdate(operation),650);
 }
 function completeUpdate(operation){
  if(run!==operation)return;
  const rows=operation.rows,failed=[],done=[],skips=[],issue=operation.scenario;
  const partialId=rows.find(x=>x.group==='mcp')?.id||rows.find(x=>x.direction!=='skip')?.id;
  for(const row of rows){
   if(row.direction==='skip'){skips.push({...row,status:'skipped'});continue;}
   if(issue==='network-error'||issue==='backup-fail'||issue==='partial'&&row.id===partialId){failed.push({...row,status:'failed',reason:issue==='network-error'?'连接中断，内容未更新':issue==='backup-fail'?'本地文件暂时无法写入，旧内容保留':'目标文件被占用，旧内容保留'});continue;}
   if(row.direction==='download'&&['channels','templates'].includes(row.group)){
    const model=window.modelPrototype;
    const applied=model.applySyncUpdates({contract:model.syncFixture().contract,source:operation.tab==='file'?'file':'webdav',channels:row.group==='channels'?[{id:row.id,credential:operation.secrets.keys.includes(row.id)}]:[],templates:row.group==='templates'?[row.id.slice(9)]:[]});
    if(!applied.ok){failed.push({...row,status:'failed',reason:applied.message});continue;}
    if(row.group==='channels')channelAdoption={status:'written',target:'mock-current-channel-file',source:operation.tab,channels:applied.channels};
   }
   const version=clone(row.direction==='download'?row.remote:row.local)||{updatedAt:stamp(5),revision:2,digest:row.id+':v2'};
   versions.local[row.id]=clone(version);versions.remote[row.id]=clone(version);
   if(row.direction==='download')localFingerprints[row.id]=fingerprint(row.id);
   if(row.direction==='download')demoObjects[row.id]={revision:version.revision,files:row.group==='skills'?['SKILL.md','scripts/review.js','references/checklist.md']:null};
   done.push({...row,status:'completed',credential:operation.secrets.keys.includes(row.id)||operation.secrets.extensions.includes(row.id)});
  }
  run=null;serial++;
  const downloaded=done.filter(x=>x.direction==='download').length,uploaded=done.filter(x=>x.direction==='upload').length;
  const title=failed.length?(done.length?'部分同步完成':'同步未完成'):operation.tab==='file'?'导入完成':'同步完成';
  const detail=operation.tab==='file'?('更新 '+downloaded+' 项，跳过 '+skips.length+' 项'+(failed.length?'，'+failed.length+' 项失败':'')):'下载更新 '+downloaded+' 项，上传 '+uploaded+' 项，跳过 '+skips.length+' 项'+(failed.length?'，'+failed.length+' 项失败':'');
  result={tab:operation.tab,operationId:operation.id,title,detail:detail+'。',downloaded,uploaded,skipped:skips.length,failed,items:[...done,...skips,...failed],protection:clone(operation.protection)};lastRun=clone(result);failedRun=failed.length?{...operation,rows:failed.map(({status,reason,...x})=>x)}:null;
  lastComparison=null;render();
  recordActivity(operation.tab==='file'?'import':'sync',done.length+skips.length,failed.length===0,{uploaded,downloaded});
  const previous=receipts.get(String(operation.id)),items=new Map((previous?.items||[]).map(x=>[x.id,x]));
  for(const item of result.items)items.set(item.id,item);
  const all=[...items.values()],failedItems=all.filter(x=>x.status==='failed');
  publishReceipt(operation.tab==='file'?(failedItems.length?'sync.import.failed':'sync.import.complete'):(failedItems.length?'sync.failed':'sync.complete'),{...result,items:all,total:all.length,failed:failedItems,downloaded:all.filter(x=>x.status==='completed'&&x.direction==='download').length,uploaded:all.filter(x=>x.status==='completed'&&x.direction==='upload').length,skipped:all.filter(x=>x.status==='skipped').length,errorCode:failedItems.length?(issue==='network-error'?'connection':failedItems.length<all.length?'partial':'write'):'none'});
 }
 function retryFailed(){if(!failedRun||run)return;const p=failedRun;startUpdate(p.tab,p.rows,p.secrets,p.protection,p.id);if(run)run.scenario='normal';}
 function resultDetails(record=result){
  if(!record?.items?.length)return;
  showDialog('result','同步结果','<p class="sx-muted">传输保护：'+(record.protection?.encrypted?'加密':'不加密')+'</p><ul class="sx-outcomes">'+record.items.map(x=>'<li><div><strong>'+esc(x.name)+'</strong><small>'+esc(x.reason||(x.direction==='download'?'已下载并更新':x.direction==='upload'?'已上传':'内容一致，无需更新'))+'</small></div><span class="sx-mark">'+(x.status==='failed'?'失败':x.status==='skipped'?'跳过':'完成')+'</span></li>').join('')+'</ul>'+(record.failed?.length&&failedRun?.id===record.operationId?btn('重试失败项','receipt-retry','data-id="'+record.operationId+'"'):'') ,'关闭',closeModal,{hideConfirm:true,cancelLabel:'关闭',annotationText:'同一操作结果的只读明细，由注册通知或现有结果入口查看；不是新通知表面。'});
 }
 function openReceipt(operationId){
  const record=receipts.get(String(operationId));if(!record)return false;
  if(record.kind==='export'){
   showDialog('result',record.failed.length?'导出未完成':'导出结果','<p>'+record.total+' 项配置 · '+(record.protection.encrypted?'加密':'不加密')+'</p>'+(record.packagePath?'<p><code>'+esc(record.packagePath)+'</code></p>'+btn('打开导出目录','open-export','data-id="'+record.operationId+'"'):'<p>目标目录无法写入；可重新选择目录后导出。</p>'),'关闭',closeModal,{hideConfirm:true,cancelLabel:'关闭',annotationText:'保存位置留在业务回执；通知只接收操作ID和计数。打开目录也只演示固定反馈，不访问真实文件。'});
  }else if(record.kind==='connection')connectionDialog();
  else resultDetails(record);
  return true;
 }
 function syncCheck(){
  if(run)return;if(!connection.connected){connectionDialog();return;}
  refreshLocalVersions();const ids=closure(scopes.webdav).ids;
  startUpdate('webdav',updateRows().filter(x=>ids.has(x.id)));
 }
 function connectionDialog(){
  showDialog('connection','WebDAV 连接','<label class="sx-field"><span>服务地址</span><input class="input" id="sxDavURL" value="'+esc(connection.url)+'"></label><label class="sx-field"><span>独立配置同步目录</span><input class="input" id="sxDavPath" value="'+esc(connection.path)+'"></label><div class="sx-grid"><label class="sx-field"><span>用户名</span><input class="input" id="sxDavUser" value="'+esc(connection.user)+'"></label><label class="sx-field"><span>连接密码</span><input class="input" id="sxDavPassword" type="password" placeholder="本机已保存 · 留空保留"></label></div><div class="sx-actions">'+btn('测试连接','test-connection')+'</div><p class="sx-muted" id="sxConnectionTest" role="status"></p>','保存连接',()=>{
   if(!validateConnection())return;
   connection={...connection,url:$('#sxDavURL').value.trim().replace(/\/$/,''),path:$('#sxDavPath').value.trim(),user:$('#sxDavUser').value.trim()};closeModal();render();publishReceipt('sync.connection.saved',{operationId:++runSerial,kind:'connection',total:1});
  },{annotationText:'WebDAV 配置可通过文件同步迁移，导入后需另行启用。当前示例不发起任何网络请求，请勿填写真实凭据。同步加密口令与连接密码分开；同步包不会覆盖正在使用的连接、定时规则或本机范围。'});
 }
 function validateConnection(){
  try{const u=new URL($('#sxDavURL').value.trim());if(u.protocol!=='https:'||u.username||u.password||u.search||u.hash)throw Error();if(!$('#sxDavPath').value.startsWith('/')||!$('#sxDavUser').value.trim())throw Error();return true;}catch{setText('#sxError','请填写不含内嵌凭据的 HTTPS 地址、以 / 开头的目录和用户名。');return false;}
 }
 function open(options={}){
  catalog=collect(options.action==='export'&&options.source==='draft'?'draft':'saved');tab=options.tab==='webdav'?'webdav':'file';query='';channelAdoption=null;
  if(options.scope){const s=state();s.selected=new Set(options.scope.filter(id=>find(id)&&eligible(find(id))&&(tab==='file'||find(id).group!=='endpoints')));s.keys=new Set(tab==='webdav'?[...s.selected].filter(id=>find(id)?.key):[]);s.accountInfoOnly.clear();s.endpointConfigOnly.clear();s.extensions=new Set(tab==='webdav'?[...s.selected].filter(id=>find(id)?.extensionSecret):[]);}
  rememberScope();
  const stayOnPage=options.action==='import'&&options.stayOnPage;
  if(!stayOnPage)setRoute('sync?tab='+tab);
  render(true);if(!stayOnPage)host.closest('.main').scrollTop=0;
  if(options.action==='import')importStart(options);else if(options.action==='export')exportPreview(options.source);else if(options.action==='connection')connectionDialog();else if(options.action==='check')syncCheck();
 }
 document.addEventListener('click',e=>{
  const el=e.target.closest('[data-sx]');if(!el||el.disabled)return;const a=el.dataset.sx,id=el.dataset.id;
  if(a==='result-details')resultDetails();
  else if(a==='retry')retryFailed();
  else if(a==='receipt-retry'){if(failedRun?.id===Number(id)){closeModal();retryFailed();}}
  else if(a==='tab'){tab=id;query='';history.replaceState(null,'','#sync?tab='+tab);render();$('#sxTab-'+tab).focus();}
  else if(a==='expand'){const folds=query?searchCollapsed[tab]:expanded[tab];folds.has(id)?folds.delete(id):folds.add(id);render();$('[data-sx="expand"][data-id="'+id+'"]',host)?.focus({preventScroll:true});}
  else if(a==='detail')detail(id);
  else if(a==='export')exportPreview();
  else if(a==='import')importStart();
  else if(a==='choose-import-file')chooseImportFile();
  else if(a==='choose-dir'){const input=$('#sxDirPath');if(!input)return;input.value='/Users/用户名/Documents/opencodex-shared';setText('#sxError','');}
  else if(a==='open-export'){if(receipts.get(String(id))?.packagePath)publishReceipt('sync.export.directory.opened',{operationId:++runSerial,kind:'export',total:1,packagePath:receipts.get(String(id)).packagePath,protection:{encrypted:false},failed:[]});}
  else if(a==='connection')connectionDialog();
  else if(a==='test-connection'&&validateConnection()){const failed=scenario==='network-error',operationId=connectionTestId||++runSerial;connectionTestId=failed?operationId:null;connection.connected=!failed;publishReceipt(failed?'sync.connection.failed':'sync.connected',{operationId,kind:'connection',total:1,failed:failed?[{id:'connection'}]:[],errorCode:failed?'connection':'none'});}
  else if(a==='check')syncCheck();
 });
 document.addEventListener('change',e=>{
  const el=e.target,s=state(),focusAttrs=[...e.target.attributes].filter(a=>a.name.startsWith('data-sx-'));const restoreFocus=()=>{const match=$$('input',dialogKind?$('#modal'):host).find(x=>focusAttrs.length&&focusAttrs.every(a=>x.getAttribute(a.name)===a.value));match?.focus({preventScroll:true});};let redraw=false;
  if(el.matches('[data-sx-object]')){selectObject(s,el.dataset.sxObject,el.checked);redraw=true;}
  else if(el.matches('[data-sx-group],[data-sx-all]')){for(const x of catalog.filter(x=>(tab==='file'||x.group!=='endpoints')&&(!el.dataset.sxGroup||x.group===el.dataset.sxGroup)&&(x.name+' '+x.summary).toLowerCase().includes(query.toLowerCase())&&(x.group!=='accounts'||accountPlatform==='all'||x.accountPlatform===accountPlatform))){selectObject(s,x.id,el.checked);}redraw=true;}
  else if(el.matches('[data-sx-account-platform]')){accountPlatform=el.value;redraw=true;}
  else if(el.matches('[data-sx-account-info]')){el.checked?s.accountInfoOnly.add(el.dataset.sxAccountInfo):s.accountInfoOnly.delete(el.dataset.sxAccountInfo);redraw=true;}
  else if(el.matches('[data-sx-endpoint-config]')){el.checked?s.endpointConfigOnly.add(el.dataset.sxEndpointConfig):s.endpointConfigOnly.delete(el.dataset.sxEndpointConfig);redraw=true;}
  else if(el.matches('[data-sx-secret]')){const key=el.dataset.sxSecret,prop={keys:'key',extensions:'extensionSecret'}[key];s[key].clear();if(el.checked)for(const x of catalog.filter(x=>closure().ids.has(x.id)&&x[prop]))s[key].add(x.id);redraw=true;}
  else if(el.matches('[data-sx-secret-object]')){el.checked?s[el.dataset.kind].add(el.dataset.sxSecretObject):s[el.dataset.kind].delete(el.dataset.sxSecretObject);redraw=true;}
  else if(el.id==='sxEncrypt')$('#sxPassFields').hidden=!el.checked;
  else if(el.name==='sxDestination'){const box=$('#sxCustomDir');if(box)box.hidden=el.value!=='custom';if(el.value!=='custom'){const input=$('#sxDirPath');if(input)input.value='';}setText('#sxError','');}
  else if(el.matches('[data-sx-protect]')){protection.enabled=el.checked;redraw=true;}
  else if(el.matches('[data-sx-import-object]')){el.checked?importPlan.selected.add(el.dataset.sxImportObject):importPlan.selected.delete(el.dataset.sxImportObject);syncImportControls();}
  else if(el.matches('[data-sx-import-keys]'))importCredentialPrefs.keys=importPlan.keys=el.checked;
  else if(el.matches('[data-sx-import-secrets]'))importCredentialPrefs.extensions=importPlan.extensions=el.checked;
  if(redraw){rememberScope();for(const key of ['keys','accountInfoOnly','endpointConfigOnly','extensions'])for(const id of s[key])if(!closure().ids.has(id))s[key].delete(id);render();restoreFocus();}
 });
 document.addEventListener('input',e=>{if(dialogKind&&e.target.closest('.sx-dialog')&&e.target.matches('input')){setText('#sxError','');if(dialogKind==='connection'){connectionTestId=null;connection.connected=false;}}});
 host.addEventListener('input',e=>{if(e.target.id==='sxProtectionPass'){protection.passphrase=e.target.value;return;}if(e.target.id!=='sxSearch')return;query=e.target.value;searchCollapsed[tab].clear();const pos=e.target.selectionStart;render(true);$('#sxSearch').focus({preventScroll:true});try{$('#sxSearch').setSelectionRange(pos,pos);}catch{}});
 host.addEventListener('keydown',e=>{if(e.target.getAttribute('role')==='tab'&&['ArrowLeft','ArrowRight','Home','End'].includes(e.key)){e.preventDefault();tab=e.key==='Home'?'file':e.key==='End'?'webdav':tab==='file'?'webdav':'file';history.replaceState(null,'','#sync?tab='+tab);render();$('#sxTab-'+tab).focus();}});
 document.addEventListener('keydown',e=>{if(detailOpen||!dialogKind||e.key!=='Tab')return;const items=$$('button:not([disabled]),input:not([disabled]),select:not([disabled]),[tabindex="0"]',$('#modal')).filter(x=>x.getClientRects().length);if(!items.length)return;const first=items[0],last=items.at(-1);if(e.shiftKey&&document.activeElement===first){e.preventDefault();last.focus();}else if(!e.shiftKey&&document.activeElement===last){e.preventDefault();first.focus();}});
 function routeChanged(){if(document.documentElement.dataset.route!=='sync')return;const p=new URLSearchParams(location.hash.split('?')[1]||'');tab=p.get('tab')==='webdav'?'webdav':'file';catalog=collect();render();}
 new MutationObserver(routeChanged).observe(document.documentElement,{attributes:true,attributeFilter:['data-route']});
 window.addEventListener('hashchange',routeChanged);
 const scenes=[['normal','双向更新 · 渠道/Skill下载，MCP上传'],['download','远端较新 · 下载并更新'],['upload','本机较新 · 上传'],['same','内容相同 · 无需更新'],['partial','一项失败 · 其余完成'],['network-error','连接中断 · 旧内容保留'],['backup-fail','本地写入失败 · 可重试']];
 const sceneSelect=$('#sxMockScenario');if(sceneSelect)sceneSelect.innerHTML=scenes.map(([id,label])=>'<option value="'+id+'">'+label+'</option>').join('');
 function setScenario(value){if(!scenes.some(([id])=>id===value))return;scenario=value;seedVersions(['download','upload','same'].includes(value)?value:'normal');if(sceneSelect)sceneSelect.value=value;result=null;render();}
 function setProtectionDemo(mode){
  if(!['plaintext','encrypted'].includes(mode))return;
  protection.enabled=mode==='encrypted';if(protection.enabled&&!protection.passphrase)protection.passphrase='demo-sync';
  render();window.prototypePanel?.refresh();
 }
 window.syncPrototype={open,refreshCatalog:()=>{if(dialogKind)closeModal();catalog=collect();render();},state:()=>({tab,accountPlatform,accountCapabilities:accountCapabilities.entries.map(x=>({...x})),selected:[...state().selected],effective:[...closure().ids],accountInfoOnly:[...state().accountInfoOnly],endpointConfigOnly:[...state().endpointConfigOnly],secrets:secretIds(),savedScope,savedScopes:clone(savedScopes),dirty:false,lastExport:clone(lastExport),recentActivity:clone(recentActivity),result:clone(result),channelAdoption:clone(channelAdoption),dialogKind,detailOpen,lastComparison:null,comparison:null,serial,scenario,importSample,importFile:clone(importFile),running:!!run,lastRun:clone(lastRun),versions:clone(versions),demoObjects:clone(demoObjects),protection:{enabled:protection.enabled,configured:!!protection.passphrase}}),setImportSample,setScenario,setProtectionDemo,openReceipt,render};
 routeChanged();render();
})();
