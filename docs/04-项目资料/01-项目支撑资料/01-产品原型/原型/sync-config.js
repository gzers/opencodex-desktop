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
 const groups=[['channels','渠道与模型'],['templates','模型模版'],['accounts','账户'],['extension-platforms','Agent 平台'],['skills','Skills'],['mcp','MCP'],['preferences','桌面偏好'],['endpoints','WebDAV 配置'],['dialog-policies','对话配置'],['dialog-history','对话历史']];
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
   {id:'preferences:desktop',group:'preferences',name:'通用偏好',summary:'主题、语言与可移植显示偏好',source:'Desktop',fields:'排除机器绝对路径、窗口坐标、日志、缓存、执行绑定、未提交草稿及本机文件/WebDAV范围、凭据勾选、加密选项和口令',files:'preferences.json（字段白名单）'},
   {id:'endpoint:personal',group:'endpoints',name:'个人 WebDAV',summary:'文件同步默认包含 · 密码随配置同步',source:'Desktop WebDAV 配置',fields:'服务地址、目录、用户名与连接密码；导入为未启用连接，不改当前连接或本机同步范围；不包含同步加密口令',password:true,files:'sync_endpoints / 受保护凭据部分'}
  ); return list.map(accountDefaults).concat(window.extensionPrototype?.syncObjects()||[],window.conversationPrototype?.syncObjects()||[]);
 }
 const importSamples={
  'extension-legacy':{file:'opencodex-legacy-extensions.ocx',encrypted:false},
  encrypted:{file:'opencodex-device-config.ocx',encrypted:true,credentials:{keys:['channel:newapi'],extensions:['mcp:documentation']}},
  plaintext:{file:'opencodex-plaintext-config.ocx',encrypted:false,credentials:{keys:['channel:newapi'],extensions:['mcp:documentation']}},
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
  const context=importPlan.context,explicitPlatforms=context?.some(id=>id.startsWith('skill-platform:')),contentOnly=context?.some(id=>extensionContent(find(id)))&&!explicitPlatforms,wanted=context?new Set([...context,...(contentOnly?catalog.filter(x=>x.platformAssociation).map(x=>x.id):[])]):null;
  importPlan.rows=directImportRows(importSample).filter(x=>!wanted||wanted.has(x.id));
  importPlan.selected=new Set(importPlan.rows.filter(x=>!contentOnly||!find(x.id)?.platformAssociation||chosenPlatforms(state().selected).includes(x.id.slice(15))).map(x=>x.id));
  renderImport();
  if(importFile.encrypted)$('#sxUnlock').focus();
 }
 let detailOpen=false,lastComparison=null;
 let catalog=collect(),tab='file',query='',accountPlatform='all',originFocus=null,dialogKind='',scenario='normal',serial=0,renderedTab=null;
 const expanded={file:new Set(),webdav:new Set()},searchCollapsed={file:new Set(),webdav:new Set()},scrolls={file:{list:0,secrets:0},webdav:{list:0,secrets:0}};
 const fresh=mode=>{const items=catalog.filter(x=>eligible(x)&&!x.defaultExcluded&&(mode==='file'||x.group!=='endpoints'));return {selected:new Set(items.map(x=>x.id)),keys:new Set(mode==='webdav'?items.filter(x=>x.key).map(x=>x.id):[]),accountInfoOnly:new Set(),endpointConfigOnly:new Set(),extensions:new Set(mode==='webdav'?items.filter(x=>x.extensionSecret).map(x=>x.id):[])};};
 const scopes={file:fresh('file'),webdav:fresh('webdav')};
 const savedScopes={file:'',webdav:''};
 let connection={name:'个人 WebDAV',url:'https://dav.example.com',path:'/opencodex/config-sync/v1',user:'demo-user',configured:true,credentialConfigured:true,credentialRevision:1,revision:'mock-endpoint:1'},savedScope='',result=null,lastExport=null,importPlan=null,channelAdoption=null;
 let connectionObservation=null,connectionRequest=0,dialogSession=0,candidateRequest=0,connectionRevisionSerial=1,connectionFormRevision=0,connectionFormProof=null,candidateProof=null;
 let importedConnections={status:'ready',items:[],error:''};
 const connectionIdentity=c=>c.revision;
 const connectionFields=c=>JSON.stringify([c.url,c.path,c.user,c.credentialRevision]);
 const observationTime=()=>new Date().toISOString();
 function connectionStatus(){
  const valid=connectionObservation?.endpointRevision===connectionIdentity(connection),o=valid?connectionObservation:null;
  const state=!connection.configured?'unconfigured':!o?'configured':o.status==='checking'?'checking':o.status==='success'?'recent-success':'recent-failure';
  return {state,label:{unconfigured:'未配置',configured:'已配置 · 未检测',checking:'正在检测','recent-success':'最近检测成功','recent-failure':'最近检测失败'}[state],observation:clone(o),connection:{...connection}};
 }
 function projectConnection(){const s=connectionStatus();window.renderWebdav?.(s.state,s.observation);return s;}
 function observeConnection(status,source='连接检测',remoteTime=null,observedAt=observationTime()){connectionObservation={endpointRevision:connectionIdentity(connection),status,source,observedAt,remoteTime};projectConnection();}
 function setConnectionScenario(scene){
  if(scene==='reload'){projectConnection();render();return;}
  connectionRequest++;connection.configured=scene!=='unconfigured';connectionObservation=null;
  if(['recent-success','recent-failure','checking'].includes(scene))observeConnection(scene==='checking'?'checking':scene==='recent-success'?'success':'failure','原型检测示例',scene==='recent-success'?'2026-11-04T02:24:00Z':null);
  projectConnection();render();
 }
 function importedEntry(){
  if(importedConnections.status==='ready'&&!importedConnections.items.length)return '';
  const status=importedConnections.status,label=status==='loading'?'导入的连接 · 正在读取…':status==='error'?'导入的连接 · 读取失败，重试':'导入的连接（'+importedConnections.items.length+'）';
  return btn(label,'imported-connections',status==='loading'?'disabled aria-busy="true"':'')+'<p class="sx-muted">'+(status==='error'?'读取失败不代表没有候选。':'候选尚未启用；确认启用后才替换当前连接。')+'</p>';
 }
 function updateImportedEntry(){const el=$('#sxImportedConnectionEntry');if(el){el.innerHTML=importedEntry();el.hidden=!el.innerHTML;}}
 function setCandidateScenario(scene){
  candidateRequest++;const items=['objects','missing-password'].includes(scene)?[{id:'imported-personal',name:'另一台设备的 WebDAV',owner:'desktop',url:'https://imported.example.test',path:'/opencodex/config-sync/v1',user:'imported-user',credentialConfigured:scene==='objects',active:false}]:[];
  importedConnections={status:scene==='loading'?'loading':scene==='error'?'error':'ready',items,error:scene==='error'?'无法读取导入的连接，请重试。':''};updateImportedEntry();
 }
 function importedConnectionsDialog(){
  const status=importedConnections.status;
  showDialog('imported-connections','导入的连接',status==='error'?'<p class="sx-error" role="alert">'+esc(importedConnections.error)+'</p>'+btn('重试读取','retry-imported'):status==='loading'?'<p role="status">正在读取导入的连接…</p>':importedConnections.items.length?'<p>以下候选尚未启用，不参与当前同步。</p><ul class="sx-outcomes">'+importedConnections.items.map(c=>'<li><div><strong>'+esc(c.name)+'</strong><small>'+esc(c.url+c.path)+' · 非活动候选'+(c.credentialConfigured?'':' · 缺少连接密码')+'</small></div>'+btn('查看并启用','activate-imported','data-id="'+esc(c.id)+'"')+'</li>').join('')+'</ul>':'<p>没有导入的连接。</p>','',null,{mode:'view'});
 }
 function retryImported(){
  importedConnections={status:'loading',items:[],error:''};importedConnectionsDialog();
  const request=++candidateRequest,session=dialogSession;
  setTimeout(()=>{if(request!==candidateRequest||session!==dialogSession||dialogKind!=='imported-connections')return;importedConnections={status:'ready',items:[],error:''};closeModal();connectionDialog();},260);
 }
 function activateImported(id){
  const candidate=importedConnections.items.find(c=>c.id===id);if(!candidate)return;
  const baseline=connectionIdentity(connection),candidateDigest=JSON.stringify(candidate);candidateProof=null;
  showDialog('activate-imported','启用导入的连接？','<p>将用 '+esc(candidate.name)+' 替换当前连接 '+esc(connection.name)+'；本机同步范围保持。</p><dl class="sx-detail-list"><dt>服务地址</dt><dd>'+esc(candidate.url)+'</dd><dt>同步目录</dt><dd>'+esc(candidate.path)+'</dd><dt>用户名</dt><dd>'+esc(candidate.user)+'</dd></dl>'+(!candidate.credentialConfigured?'<label class="sx-field">连接密码<input class="input" type="password" id="sxCandidatePassword" autocomplete="off"></label><p class="sx-muted">候选缺少密码，填写后才能启用。</p>':'')+btn('测试连接','test-imported','data-id="'+esc(id)+'"')+'<p class="sx-muted" id="sxCandidateTest" role="status"></p>','确认启用',()=>{
   if(baseline!==connectionIdentity(connection)||candidateDigest!==JSON.stringify(importedConnections.items.find(c=>c.id===id))){setText('#sxError','连接或候选已变化，请重新核对。');return;}
   if(candidate.owner!=='desktop'){setText('#sxError','候选归属不匹配，不能启用。');return;}
   if(!candidate.credentialConfigured&&!$('#sxCandidatePassword')?.value){setText('#sxError','请填写连接密码。');return;}
   const proof=candidateProof;connectionRequest++;connection={...connection,...candidate,configured:true,credentialConfigured:true,credentialRevision:connection.credentialRevision+1,revision:'mock-endpoint:'+ ++connectionRevisionSerial};delete connection.owner;delete connection.active;
   connectionObservation=null;closeModal();if(proof?.candidateDigest===candidateDigest)observeConnection(proof.status,'候选表单检测',null,proof.observedAt);projectConnection();render();
  },{mode:'confirm'});
  $('#modalConfirm').disabled=!candidate.credentialConfigured;
 }
 function testImported(id){
  const candidate=importedConnections.items.find(c=>c.id===id);if(!candidate||dialogKind!=='activate-imported')return;
  if(!candidate.credentialConfigured&&!$('#sxCandidatePassword')?.value){setText('#sxCandidateTest','缺少连接密码，未执行检测。');return;}
  const session=dialogSession,digest=JSON.stringify(candidate),request=++candidateRequest,failed=scenario==='network-error';
  setText('#sxCandidateTest','正在检测候选连接…');
  setTimeout(()=>{if(session!==dialogSession||request!==candidateRequest||dialogKind!=='activate-imported'||digest!==JSON.stringify(importedConnections.items.find(c=>c.id===id)))return;candidateProof={candidateDigest:digest,status:failed?'failure':'success',observedAt:observationTime()};setText('#sxCandidateTest','候选最近检测'+(failed?'失败':'成功')+' · '+candidateProof.observedAt+'；当前连接观察保持。');},260);
 }
 let run=null,runSerial=0,lastRun=null,failedRun=null,connectionTestId=null,preview=null;
 const backups=[],snapshots={local:null,remote:null};
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
 let platformDefaultsInitialized=false;
 const localFingerprints={},extensionRemote={},knownObjects=new Set(catalog.map(x=>x.id)),versions={local:{},remote:{}},protection={enabled:false,passphrase:''},importCredentialPrefs={keys:true,extensions:true},demoObjects={};
 // 同名 MCP 在各客户端定义不同时，只记客户端名与差异字段名，不记值；选定来源后记为受管定义，之后不再询问。
 let mcpVariants={},managedMcp={};
 const conflict=id=>managedMcp[id]?null:clone(mcpVariants[id]);
 const stamp=n=>'2026-10-30T'+String(9+Math.floor(n/60)).padStart(2,'0')+':'+String(n%60).padStart(2,'0')+':00+08:00';
 function seedVersions(scene='normal'){
  snapshots.local={revision:1,updatedAt:stamp(20),digest:'snapshot:local-v1'};
  snapshots.remote={revision:2,updatedAt:stamp(30),digest:'snapshot:remote-v2'};
  if(scene==='upload'){snapshots.local={revision:3,updatedAt:stamp(40),digest:'snapshot:local-v3'};}
  if(scene==='same')snapshots.remote=clone(snapshots.local);
  if(scene==='mcp-source')snapshots.local={revision:3,updatedAt:stamp(40),digest:'snapshot:local-v3'};
  for(const x of catalog.filter(eligible)){
   const local={updatedAt:stamp(0),revision:1,digest:x.id+':v1'};let remote=clone(local);
   if(scene==='download'||scene==='normal'&&['channel:newapi','template:gpt','skill:design','preferences:desktop','mcp:documentation'].includes(x.id))remote={updatedAt:scene==='normal'&&x.id!=='mcp:documentation'?stamp(30):stamp(5),revision:2,digest:x.id+':v2'};
   if(scene==='upload')Object.assign(local,{updatedAt:stamp(6),revision:2,digest:x.id+':local-v2'});
   if(scene==='mcp-source'&&['skill:design','mcp:documentation'].includes(x.id))Object.assign(local,{updatedAt:stamp(21),revision:2,digest:x.id+':local-v2'});
   // 此对象的本机内容时间虽更新，本次仍只服从整份快照来源，不逐对象换方向。
   if(scene==='normal'&&x.id==='mcp:documentation')Object.assign(local,{updatedAt:stamp(21),revision:3,digest:x.id+':local-v3'});
   versions.local[x.id]=local;versions.remote[x.id]=remote;
   demoObjects[x.id]={revision:1,files:x.group==='skills'?(x.payload?.files||[]).map(f=>f.path):null};if(x.extensionObject){extensionRemote[x.id]=scene==='same'?clone(window.extensionPrototype.syncObject(x.id)):window.extensionPrototype.syncFixture(x.id);}
  }
 }
 seedVersions();
 function fingerprint(id){if(catalog.find(x=>x.id===id)?.extensionObject)return JSON.stringify(window.extensionPrototype?.syncObject(id));const model=window.modelPrototype?.state();if(id.startsWith('channel:'))return JSON.stringify(model?.saved.providers.find(p=>p.id===id.slice(8)));if(id.startsWith('template:'))return JSON.stringify(model?.templates.find(t=>t.id===id.slice(9)));return null;}
 function refreshLocalVersions(){let changed=false;for(const x of catalog){const value=fingerprint(x.id);if(value===null)continue;if(localFingerprints[x.id]!==undefined&&localFingerprints[x.id]!==value){const revision=(versions.local[x.id]?.revision||1)+1;versions.local[x.id]={updatedAt:stamp(40+revision),revision,digest:x.id+':local-edit-'+revision};changed=true;}localFingerprints[x.id]=value;}if(changed){const revision=snapshots.local.revision+1;snapshots.local={revision,updatedAt:stamp(40+revision),digest:'snapshot:local-edit-'+revision};}}
 refreshLocalVersions();


 function pruneUnavailablePlatforms(){
  for(const scope of Object.values(scopes))for(const id of [...scope.selected])if(id.startsWith('skill-platform:')&&!window.extensionPrototype?.platformSyncInfo(id.slice(15))?.installed)scope.selected.delete(id);
 }
 function integrateExtensionCatalog(){
  catalog=collect();
  for(const x of catalog){
   if(!versions.local[x.id]){
    versions.local[x.id]={updatedAt:stamp(x.id==='mcp:documentation'?21:0),revision:x.id==='mcp:documentation'?3:1,digest:x.id+':v1'};versions.remote[x.id]=['skill:design','mcp:documentation'].includes(x.id)?{updatedAt:stamp(x.id==='mcp:documentation'?5:30),revision:2,digest:x.id+':v2'}:clone(versions.local[x.id]);
    demoObjects[x.id]={revision:1,files:x.payload?.files?.map(f=>f.path)||null};
    if(x.extensionObject){extensionRemote[x.id]=window.extensionPrototype.syncFixture(x.id);}
   }
   if(!knownObjects.has(x.id)){knownObjects.add(x.id);for(const key of ['file','webdav'])if(eligible(x)&&!x.defaultExcluded&&(!x.platformAssociation||!platformDefaultsInitialized)&&(key==='file'||x.group!=='endpoints')){scopes[key].selected.add(x.id);if(key==='webdav'&&x.extensionSecret)scopes[key].extensions.add(x.id);}}
  }
  if(catalog.some(x=>x.platformAssociation))platformDefaultsInitialized=true;pruneUnavailablePlatforms();refreshLocalVersions();rememberScope();if(!run)render();
 }
 const extensionIds=rows=>rows.filter(x=>find(x.id)?.extensionObject).map(x=>x.id);

 function platformInfo(id){return window.extensionPrototype?.platformSyncInfo(id.slice(15));}

 function platformResultHTML(x){
  const info=platformInfo(x.id);
  return '<div class="sx-platform-result"><span class="sx-platform-path" title="'+esc(info?.path||'目录待核实')+'">'+esc(info?.path||'目录待核实')+'</span><span class="sx-platform-state">'+esc(!info?.installed?'平台未发现':info.mapped?'已映射':'未映射')+'</span></div>';
 }
 function platformAssociationsHTML(rows,mode='preview'){
  const platforms=rows.filter(x=>x.id.startsWith('skill-platform:'));if(!platforms.length)return '';
  return '<section class="sx-platform-associations"><h3>所选 Agent 平台</h3><p class="sx-muted">未映射也可同步。所选平台决定本次 Skills / MCP 范围，未选平台的 MCP 启用关系保持；不创建或解除目录映射。</p><ul>'+platforms.map(row=>{
   const info=platformInfo(row.id),status=info?.condition||'平台未发现',outcome=!info?.installed?'内容可保存；先检测或配置本机平台':!info.supported?'内容可保存；此平台的 Skills 支持待核实':info.mapped?'本机已有映射保持':'内容可保存；使用 Skills 前需完成本机映射';
   return '<li data-sx-platform-result="'+esc(row.id)+'"><div><strong>'+esc(row.name)+'</strong><span>'+esc(status)+'</span></div><p title="'+esc(info?.path||'目录待核实')+'">'+esc(info?.path||'目录待核实')+'</p><small>'+esc(mode==='export'?'只导出平台关联；接收设备重新检查路径与映射':outcome)+'</small></li>';
  }).join('')+'</ul></section>';
 }
 function extensionTransferHTML(rows,after=false){
  const ids=extensionIds(rows);if(!ids.length)return '';
  const pending=window.extensionPrototype?.syncReadiness(ids,after?{}:Object.fromEntries(rows.filter(x=>x.extensionPayload).map(x=>[x.id,x.extensionPayload])))||[];
  return '<section class="sx-extension-transfer"><strong>'+(after?'拓展内容已保存':'拓展同步范围')+'</strong><p>按所选 Agent 平台同步所选 Skills / MCP。Skills 共享源只保存一份，源更新会影响本机所有已映射平台。</p><p class="sx-muted">安装路径、平台版本、检测结果和映射状态保留本机；不会自动安装平台或创建目录映射。</p>'+(pending.length?'<div class="sx-extension-pending"><span>'+pending.length+' 项本机接入待处理</span><ul>'+pending.map(x=>'<li>'+esc(x.name)+' · '+esc(x.reason)+'</li>').join('')+'</ul>'+(after?btn('前往拓展处理','extensions'):'<p class="sx-muted">缺少本机条件时仍保存拓展内容，完成后可前往拓展处理。</p>')+'</div>':'<p class="sx-muted">当前所选平台的本机条件已满足。</p>')+'</section>';
 }
 function localExtensionResult(){
  if(!result?.extensionIds?.length)return '';
  const pending=window.extensionPrototype?.syncReadiness(result.extensionIds)||[];
  return pending.length?'<div class="sx-extension-result" role="status"><div><strong>拓展内容已同步</strong><span>'+pending.length+' 项平台接入待处理 · 不影响内容保存</span></div>'+btn('前往拓展处理','extensions')+'</div>':'';
 }
 document.addEventListener('prototype:extensions-ready',integrateExtensionCatalog);
 document.addEventListener('prototype:platform-availability',integrateExtensionCatalog);
 function savedContentDecision(){
  const ids=[...closure(scopes.webdav).ids].sort(),scopeKey=signature(scopes.webdav);
  const contentVersions=Object.fromEntries(['local','remote'].map(side=>{
   const snapshot=snapshots[side],items=ids.map(id=>versions[side][id]),present=items.filter(Boolean);
   const times=present.map(v=>Date.parse(v.updatedAt)),valid=times.every(Number.isFinite);
   return [side,{exists:present.length>0,scopeKey:snapshot.scopeKey||scopeKey,contentDigest:JSON.stringify(ids.map(id=>[id,versions[side][id]?.digest||null,find(id)?.group==='mcp'?(side==='local'?window.extensionPrototype.syncObject(id):extensionRemote[id])?.platformIds?.filter(x=>chosenPlatforms(scopes.webdav.selected).includes(x)).sort():null])),savedAt:snapshot.savedAt??(valid&&times.length?new Date(Math.max(...times)).toISOString():''),timeKind:snapshot.timeKind||'saved',bound:snapshot.bound!==false&&present.every(v=>!!v.digest),clockSkew:!!snapshot.clockSkew}];
  }));
  return {...window.G0Contracts.decideSavedContent(contentVersions.local,contentVersions.remote),contentVersions};
 }
 function snapshotSource(){return savedContentDecision().source;}
 function scopedMcpDifferent(id){if(find(id)?.group!=='mcp')return false;const platforms=chosenPlatforms(scopes.webdav.selected),a=window.extensionPrototype.syncObject(id),b=extensionRemote[id];return !!a&&!!b&&JSON.stringify(a.platformIds.filter(x=>platforms.includes(x)).sort())!==JSON.stringify(b.platformIds.filter(x=>platforms.includes(x)).sort());}
 function direction(id,source=snapshotSource()){const a=versions.local[id],b=versions.remote[id];return a?.digest===b?.digest&&!scopedMcpDifferent(id)?'skip':source==='local'?'upload':'download';}
 const state=()=>scopes[tab], find=id=>catalog.find(x=>x.id===id);
 const setText=(selector,value)=>{const el=$(selector);if(el)el.textContent=value;};
 const btn=(text,action,attrs='',primary=false)=>'<button type="button" class="btn '+(primary?'primary':'ghost')+'" data-sx="'+action+'" '+attrs+'>'+text+'</button>';
 const check=(attr,label,checked=false,disabled=false)=>'<label class="sx-check"><input type="checkbox" '+attr+(checked?' checked':'')+(disabled?' disabled':'')+'><span>'+label+'</span></label>';

 const extensionContent=x=>x&&['skills','mcp'].includes(x.group);
 function chosenPlatforms(selected){return [...selected].filter(id=>id.startsWith('skill-platform:')&&window.extensionPrototype?.platformSyncInfo(id.slice(15))?.installed).map(id=>id.slice(15));}
 function scopedExtensionPayload(id,payload,platformIds){
  if(!payload)return payload;const value=clone(payload);
  if(value.kind==='mcp')value.platformIds=value.platformIds.filter(id=>platformIds.includes(id));
  return value;
 }
 function mergeRemoteExtension(id,payload,platformIds){
  const previous=extensionRemote[id];
  if(payload?.kind==='mcp'&&previous){const scope=new Set(platformIds);return {...clone(payload),platformIds:[...new Set([...(previous.platformIds||[]).filter(id=>!scope.has(id)),...payload.platformIds.filter(id=>scope.has(id))])].sort()};}
  return clone(payload);
 }
 function syncScopeNote(selected=state().selected){
  const n=chosenPlatforms(selected).length;
  return n?'已选择 '+n+' 个 Agent 平台；同步所选 Skills / MCP。未选平台的 MCP 启用关系保持。':'未选择 Agent 平台，本次不同步 Skills / MCP；其他配置可继续同步。';
 }
 function closure(s=state()){
  const deps=new Map();
  for(const id of s.selected){for(const ref of find(id)?.deps||[]){const split=ref.lastIndexOf(':'),tid=ref.slice(0,split),revision=Number(ref.slice(split+1));if(!tid)continue;const key='template:'+tid;if(!deps.has(key))deps.set(key,new Set());deps.get(key).add(revision);}}
  const hasPlatforms=chosenPlatforms(s.selected).length>0;return {deps,ids:new Set([...s.selected,...deps.keys()].filter(id=>find(id)&&eligible(find(id))&&(hasPlatforms||!extensionContent(find(id))))) };
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
    const gated=extensionContent(x)&&!chosenPlatforms(s.selected).length,dependent=deps.has(x.id)&&!s.selected.has(x.id),included=Object.values(secretIds(s)).some(items=>items.includes(x.id));
    let mark=x.platformAssociation?'平台关联':x.history?'本地恢复包中选择':dependent?'依赖补齐':x.group==='accounts'?(x.selectable===false?'暂不支持':x.login&&!s.accountInfoOnly.has(x.id)?'含登录状态':'仅账户信息'):x.password?(included?'含密码':'仅配置'):included?'含凭据':x.key?'不带 Key':'配置';
    return {className:'sx-object'+(x.platformAssociation?' sx-platform-object':'')+(gated?' sx-extension-gated':''),depth:1,label:x.name,
      selection:'<input type="checkbox" data-sx-object="'+esc(x.id)+'" aria-label="选择 '+esc(x.name)+'" '+(ids.has(x.id)?'checked':'')+(gated?' disabled title="请先选择 Agent 平台"':x.selectable===false?' disabled title="'+esc(x.capability?.reason||'未支持')+'"':x.history?' disabled title="使用本地重装恢复流程；当前未接入远端历史交换"':dependent?' disabled title="所选渠道需要此精确版本"':'')+'>',
      nameHtml:'<button class="sx-object-name" data-sx="detail" data-id="'+esc(x.id)+'">'+esc(x.name)+'</button>'+(x.platformAssociation?'':'<small>'+esc(dependent?'仅精确版本 v'+[...deps.get(x.id)].join(' / v'):x.password?(s.endpointConfigOnly.has(x.id)?'仅配置 · 不含连接密码':'密码随配置同步 · 导入后待启用'):x.summary)+'</small>')+(x.platformAssociation?platformResultHTML(x):''),
      cells:x.platformAssociation?[]:['<span class="sx-mark '+(included?'secret':x.group==='accounts'&&!x.login?'warn':'')+'">'+mark+'</span>']};
   });
   return '<div class="sx-group">'+window.PrototypeTreeTable.row({className:'sx-group-head',label,
     selection:'<input type="checkbox" data-sx-group="'+id+'" aria-label="选择'+label+'" '+(all?'checked':'')+(selectable.length&&(!['skills','mcp'].includes(id)||chosenPlatforms(s.selected).length)?'':' disabled')+'>',
     toggleAttrs:'data-sx="expand" data-id="'+id+'"',expanded:open,
     cells:['<span class="sx-group-count">'+(selectable.length?count+' / '+selectable.length:id==='accounts'?'暂无可选能力':'本地恢复')+'</span>']})+(id==='extension-platforms'&&open?'<p class="sx-platform-help">已发现的 Agent 平台默认选中。选择同步平台；路径和映射结果仅供查看，未映射也可同步。</p>':'')+(id==='accounts'&&open?'<label class="sx-account-filter">账户平台<select data-sx-account-platform aria-label="账户列表平台筛选"><option value="all">全部平台</option>'+Object.entries(accountCapabilities.platforms).map(([key,label])=>'<option value="'+key+'" '+(accountPlatform===key?'selected':'')+'>'+label+'</option>').join('')+'</select></label>':'')+children+'</div>';
  }).join('');
  return '<div class="sx-toolbar"><input class="input" id="sxSearch" type="search" aria-label="搜索配置对象" placeholder="搜索配置对象" value="'+esc(query)+'"></div><div class="sx-table proto-tree" role="region" aria-label="配置对象列表" tabindex="0">'+window.PrototypeTreeTable.row({className:'sx-table-head',label:tab==='file'?'导出范围':'配置对象',selection:'<input type="checkbox" data-sx-all aria-label="选择当前搜索结果" '+(visible.filter(eligible).length&&visible.filter(eligible).every(x=>ids.has(x.id))?'checked':'')+'>',cells:['<span>范围与凭据</span>']})+''+(rows||'<div class="sx-empty">没有匹配的配置对象<br>更换关键词或清除搜索。</div>')+'</div><p class="sx-extension-scope-note" role="status">'+esc(syncScopeNote())+' 同步无需先映射。</p>'+sxAnn('01','同步范围与选择规则','范围就是最后一次的勾选结果，文件导出与 WebDAV 各记一份，下次打开直接沿用；表头复选框可一键全选或取消当前搜索结果。同平台支持多个独立账户；账户平台与OpenCodex管理方、调用方分开，Google Antigravity、Kiro不归类为Codex。账户能力按账户平台、管理方、授权来源和认证类型登记；当前正常流程仅账户信息。默认导出当前已保存文件快照；仅显式指定 draft 来源才导出当前模型渠道内存草稿。模版只取已保存版本，不带模版编辑缓冲。搜索仅改变显示，隐藏勾选保留。凭据随所选范围/包内能力选择，默认不加密。普通同步不传输这些操作偏好，也不切换另一台设备的当前连接。每次导入在包内提供范围默认全选，显式入口范围只列对应对象。点击同步先看预览，确认才执行。原型只记在当前文档内存，刷新后复位；不读写真实文件。');
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
  catalog=collect();pruneUnavailablePlatforms();const selected=closure().ids.size,retry=result?.tab===tab&&result.failed?.length?btn('重试失败项','retry',run?'disabled':''):'';
  host.innerHTML='<div class="sx-tabs" data-page-tabs data-tabs-scroll="inner" role="tablist" aria-label="同步方式">'+[['file','文件同步'],['webdav','WebDAV 同步']].map(([id,name])=>'<button type="button" role="tab" id="sxTab-'+id+'" aria-controls="sxPanel" aria-selected="'+(tab===id)+'" tabindex="'+(tab===id?'0':'-1')+'" data-sx="tab" data-id="'+id+'">'+name+'</button>').join('')+'</div>'+
   '<div id="sxPanel" role="tabpanel" aria-labelledby="sxTab-'+tab+'"><header class="sx-top"><div><h2>'+(tab==='file'?'导入与导出':'设备间同步')+'</h2><p class="sx-muted">'+(tab==='file'?'按下方范围导出配置；导入时在所选文件内选择范围。':'较新版本覆盖所选范围，确认前可查看变化。')+'</p></div><div class="sx-actions">'+(tab==='file'?(result?.tab===tab&&result.items?.length?btn('查看结果','result-details'):'')+retry+btn('导入配置','import')+btn('导出配置','export',selected?'':'disabled',true):(result?.tab===tab&&result.items?.length?btn('查看结果','result-details'):'')+retry+btn(run?'同步中…':'同步','check',run||!selected?'disabled':'',true))+'</div></header>'+
   (tab==='webdav'?'<div class="sx-connection"><div class="sx-connection-info"><div><strong>'+esc(connection.name)+'</strong></div><small class="sx-muted" title="'+esc(connection.url+connection.path)+'">'+esc(connection.url+connection.path)+'</small><small class="sx-muted">'+esc(connectionStatus().label)+'</small></div>'+btn('连接设置','connection')+'</div>'+sxAnn('03','新版覆盖与预览确认','S18-01/03/04/05：范围和凭据选择在当前文档自动记住，同步先按所选范围的可靠保存时间确定一个来源，展示变化；确认后写mock文件，取消不写。概览入口全程留在概览。Skill整体更新并移除本Skill旧文件；官方openai忽略。下载与导入默认勾选更新前备份，备份内容对应实际更新范围。右侧可演示本机新版、云端新版、无变化、备份失败及失败重试。同名MCP在各客户端定义不同时，预览只列客户端名与差异字段名并要求选择来源；不选则本项本次不同步，其余对象照常更新；选定后记为受管定义，之后不再询问（右侧选“同名MCP定义不同”）。客户端应用维护、含本机路径的MCP（如node_repl）默认不同步，可手动勾选。无共同基线也按可靠保存时间判新；S22-POST-01最新原型：来源行下拉可切换上传/下载；无法判新时云端有有效配置默认下载，云端空目录默认上传，仍须确认；读取失败不作空目录。整对象删除传播与未保存编辑仍待确认；拓展安装路径、检测、版本与映射状态保留本机；本轮不演示删除用户对象，也不把建议写为生产结论。'):'') +
   '<div class="sx-layout"><section class="sx-scope" aria-label="'+(tab==='file'?'导出范围':'同步范围')+'">'+scopeHTML()+'</section>'+credentialsHTML()+'</div>' +
   localExtensionResult()+(run&&run.tab===tab?'<div class="sx-result" role="status" aria-live="polite"><strong>'+(run.tab==='file'?'正在导入':'正在同步')+'</strong><span>正在更新所选 '+run.ids.length+' 项内容…</span></div>':'') +
   '</div>';
  if(window.conversationPrototype?.renderRecovery($('#sxPanel',host),tab)){renderedTab=tab;return;}
  if(tab==='file'&&window.conversationPrototype?.scope()==='conversations')$('.sx-layout',host).insertAdjacentHTML('beforebegin',window.conversationPrototype.recoveryEntry());
  $('.sx-table',host).scrollTop=scrolls[tab].list;$('.sx-secrets',host).scrollTop=scrolls[tab].secrets;renderedTab=tab;
  for(const el of $$('[data-sx-group]',host)){const items=catalog.filter(x=>eligible(x)&&x.group===el.dataset.sxGroup&&(x.name+' '+x.summary).toLowerCase().includes(query.toLowerCase())&&(x.group!=='accounts'||accountPlatform==='all'||x.accountPlatform===accountPlatform)),n=items.filter(x=>closure().ids.has(x.id)).length;el.indeterminate=n>0&&n<items.length;}
  for(const el of $$('[data-sx-secret]',host)){const key=el.dataset.sxSecret,prop={keys:'key',extensions:'extensionSecret'}[key],items=catalog.filter(x=>closure().ids.has(x.id)&&x[prop]),n=items.filter(x=>state()[key].has(x.id)).length;el.indeterminate=n>0&&n<items.length;}
  const all=$('[data-sx-all]',host),shown=catalog.filter(x=>eligible(x)&&(tab==='file'||x.group!=='endpoints')&&(x.name+' '+x.summary).toLowerCase().includes(query.toLowerCase())&&(x.group!=='accounts'||accountPlatform==='all'||x.accountPlatform===accountPlatform)),n=shown.filter(x=>closure().ids.has(x.id)).length;if(all)all.indeterminate=n>0&&n<shown.length;
 }
 function showDialog(kind,title,body,confirm,handler,opts={}){
  dialogSession++;
  if(!dialogKind)originFocus=document.activeElement;
  dialogKind=kind;
  openModal(title,'<div class="sx-dialog">'+body+'<div class="sx-error" id="sxError" role="alert"></div></div>',confirm,handler,{confirmKeepsOpen:true,hideNote:true,...opts});
  const annotation=$('#ann-modal-demo .annotation-source');
  if(opts.annotationText&&annotation)for(const text of [opts.annotationText].flat()){
   const paragraph=document.createElement('p');paragraph.textContent=text;annotation.querySelector('.annotation-footer').before(paragraph);
  }
  $('#modal').classList.remove('sx-comparison');$('#modal').classList.add('modal-wide');$('#modalConfirm').disabled=false;
  window.__modalOnClose=()=>{dialogSession++;candidateRequest++;for(const el of [$('.modal-head'),$('.modal-actions')])if(el)el.inert=false;detailOpen=false;$('#modal').classList.remove('sx-comparison');dialogKind='';preview=null;$('#modalBody').innerHTML='';$('#modalConfirm').disabled=false;if(originFocus?.isConnected)originFocus.focus();originFocus=null;};
 }
 function detail(id){
  const x=find(id);if(!x)return;

  if(x.platformAssociation){const info=platformInfo(id);showDialog('detail',x.name+' · Agent 平台','<dl class="sx-detail-list"><dt>原始路径</dt><dd>'+esc(info?.path||'目录待核实')+'</dd><dt>本机映射</dt><dd>'+esc(info?.condition||'平台未发现')+'</dd><dt>是否可同步</dt><dd>未映射也可选择同步。取消所有 Agent 平台后，Skills / MCP 不参与本次同步；选择不改变本机映射。</dd><dt>接收后使用</dt><dd>接收设备重新检查安装、路径和映射；条件不足时保留内容并提示处理。</dd></dl>','',null,{hideConfirm:true});return;}
  if(x.history){window.conversationPrototype?.openRecovery('export','thread:'+id.slice('dialog-history:'.length));return;}
  const accountOptions=x.group==='accounts'&&!x.login?'<details class="sx-account-options" open><summary>同步选项</summary>'+check('aria-label="仅账户信息"','仅账户信息',true,true)+check('aria-label="同步登录状态"','同步登录状态',false,true)+'<p class="sx-muted">'+esc(x.capability.reason)+'</p></details>':x.login?'<details class="sx-account-options"><summary>同步选项</summary>'+check('data-sx-account-info="'+esc(id)+'"','仅同步账户信息，不含登录状态',state().accountInfoOnly.has(id),!state().selected.has(id))+'<p class="sx-muted">'+(state().selected.has(id)?'默认随账户同步登录状态；修改后立即计入当前范围。':'先勾选此账户，再调整同步选项。')+'</p></details>':'';
  if(x.group==='accounts'){
   showDialog('detail',x.name,'<dl class="sx-detail-list"><dt>邮箱 / 标签</dt><dd>'+esc(x.email||x.name)+'</dd><dt>来源账户</dt><dd>'+esc(x.sourceAccountId||x.id)+'</dd><dt>账户平台</dt><dd>'+esc(accountCapabilities.platforms[x.accountPlatform]||x.accountPlatform)+'</dd><dt>管理方</dt><dd>'+esc(x.source)+'</dd><dt>授权来源 / 认证</dt><dd>'+esc(x.provider+' / '+x.authKind)+'</dd><dt>可选范围</dt><dd>'+esc(x.selectable===false?'暂不支持选择':'仅账户身份与显示信息')+'</dd></dl>'+accountOptions,'关闭',closeModal,{cancelLabel:'关闭',hideConfirm:true,annotationText:'账户平台与能力门已获本轮原型确认。对象、状态均为固定原型资料；不读取真实账户或登录材料。生产按已确认的适配器与真实证据登记，不能由原型成功状态晋升支持。'});return;
  }
  const extensionNote=x.extensionObject?extensionTransferHTML([{id:x.id}]):'';
  const endpointOptions=x.password?'<details class="sx-account-options"><summary>同步选项</summary>'+check('data-sx-endpoint-config="'+esc(id)+'"','仅同步配置，不含密码',state().endpointConfigOnly.has(id),!state().selected.has(id))+'<p class="sx-muted">默认随配置同步连接密码；导入后需另行启用连接。同步加密口令始终不进入配置包。</p></details>':'';
  showDialog('detail',x.name,'<p class="sx-muted">配置对象详情 · 敏感值不展示</p><dl class="sx-detail-list"><dt>归属</dt><dd>'+esc(x.source)+'</dd>'+(x.group==='accounts'?'<dt>账户平台</dt><dd>'+esc(accountCapabilities.platforms[x.accountPlatform]||x.accountPlatform)+'</dd><dt>授权来源 / 认证</dt><dd>'+esc(x.provider+' / '+x.authKind)+'</dd><dt>登录迁移</dt><dd>'+esc(x.capability.reason)+'</dd>':'')+'<dt>包含字段</dt><dd>'+esc(x.fields)+'</dd>'+(x.group==='mcp'?'<dt>定义来源</dt><dd>'+esc(managedMcp[id]?'已采用 '+managedMcp[id].source+'（受管定义）':mcpVariants[id]?'各客户端定义不同：'+mcpVariants[id].clients.join('、')+'（差异字段：'+mcpVariants[id].fields.join('、')+'）；同步或导出时选择来源':x.defaultExcluded?'由客户端应用维护，默认不同步':'各客户端定义一致')+'</dd>':'')+'<dt>配置来源</dt><dd>'+esc(x.files)+'</dd>'+(x.endpoint?'<dt>当前连接地址</dt><dd>'+esc(x.endpoint)+'</dd>':'')+(x.nodes?.length?'<dt>服务节点</dt><dd>'+x.nodes.map(n=>esc(n.name)+(n.current?' · 当前':'')+'：'+esc(n.url||'待配置')).join('<br>')+'</dd>':'')+(versions.local[id]?'<dt>内容版本</dt><dd>v'+versions.local[id].revision+' · '+esc(versions.local[id].updatedAt.replace('T',' ').replace('+08:00',''))+'</dd>':'')+(demoObjects[id]?.files?'<dt>当前文件</dt><dd>'+demoObjects[id].files.map(esc).join('<br>')+'</dd>':'')+'<dt>依赖</dt><dd>'+esc(x.deps?.length?x.deps.join('、'):'无额外模版依赖')+'</dd>'+(x.models?.length?'<dt>已选模型</dt><dd>'+x.models.map(esc).join('、')+'</dd>':'')+'<dt>凭据</dt><dd>'+esc(x.key?'API Key 单独选择；不显示或复制掩码值':x.login?'登录状态默认随账户同步，接收端自动恢复并检查':x.group==='accounts'?'当前只同步已支持的账户信息；本机已有登录保持，新设备需登录':x.extensionSecret?'授权头 / 环境变量单独选择':x.password?'连接密码默认随配置同步，可选仅配置；不包含同步加密口令':'不携带凭据')+'</dd></dl>'+extensionNote+accountOptions+endpointOptions+(['accounts','endpoints','dialog-policies','skills','mcp','extension-platforms'].includes(x.group)?'':'<p class="sx-note">同一渠道原位更新，官方 openai 保持本机自带能力。</p>'),'关闭',closeModal,{cancelLabel:'关闭',hideConfirm:true});
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
  const platforms=chosenPlatforms(state().selected),extensionSnapshot=window.extensionPrototype?.portableSnapshot([...cl.ids]);if(extensionSnapshot)extensionSnapshot.objects=extensionSnapshot.objects.map(x=>({...x,payload:scopedExtensionPayload(x.id,x.payload,platforms)}));return {extensions:extensionSnapshot||null,modelChannels:providers.length?{providers,globalRefs:clone(policy.globalRefs),fastMode:policy.fastMode,fastRows:policy.fastRows}:null,templates:(m?.templates||[]).filter(t=>cl.ids.has('template:'+t.id)).map(t=>({...clone(t),versions:t.versions.filter(v=>state().selected.has('template:'+t.id)||cl.deps.get('template:'+t.id)?.has(v.n)).map(clone)}))};
 }
 function exportPreview(source='saved'){
  // 仅显式 source:'draft' 使用当前内存渠道；普通入口始终重新取已保存文件快照。
  source=source==='draft'?'draft':'saved';catalog=collect(source);pruneUnavailablePlatforms();
  if(!closure().ids.size)return;
  const cl=closure(),secrets=secretIds(),count=secretCount();
  const manifest={demo:true,kind:'configuration-sync',objects:[...cl.ids],dependencies:[...cl.deps].map(([id,versions])=>({id,revisions:[...versions]})),credentials:secrets,accounts:[...cl.ids].map(find).filter(x=>x?.group==='accounts').map(x=>({id:x.id,accountPlatform:x.accountPlatform,owner:x.owner,provider:x.provider,sourceAccountId:x.sourceAccountId,subject:x.subject||null,tenant:x.tenant||null,label:x.name,email:x.email||null})),source:source==='draft'?'draft':'committed',snapshotSource:source,encryptionRequired:false,snapshot:exportSnapshot(source,cl,secrets),agentPlatforms:chosenPlatforms(state().selected),modules:[...cl.ids].filter(id=>find(id)?.group==='dialog-policies').map(id=>({id,schemaVersion:1,content:clone(find(id).payload)}))};
  const destination='<div class="sx-field"><span>保存位置</span><div class="sx-destination" role="group" aria-label="保存位置"><label class="sx-check"><input type="radio" name="sxDestination" value="default" checked><span>默认导出目录</span></label><label class="sx-check"><input type="radio" name="sxDestination" value="custom"><span>自定义目录</span></label></div><div id="sxCustomDir" hidden><div class="sx-dir-row"><input id="sxDirPath" class="input" type="text" readonly placeholder="尚未选择目录"><button type="button" class="btn ghost" data-sx="choose-dir">选择目录…</button></div><p class="sx-muted">未选择目录时按默认导出目录保存。</p></div></div>';
  const conflicts=[...cl.ids].filter(id=>find(id)?.group==='mcp'&&conflict(id));
  const sourceBlock=conflicts.length?'<div class="sx-info sx-mcp-conflicts"><strong>同名 MCP 定义不同</strong><ul class="sx-outcomes">'+conflicts.map(id=>'<li><div><strong>'+esc(find(id).name)+'</strong><small>'+esc('在 '+mcpVariants[id].clients.join('、')+' 中不同（差异字段：'+mcpVariants[id].fields.join('、')+'）')+'</small>'+sourcePicker(id,'暂不导出')+'</div></li>').join('')+'</ul></div>':'';
  const scopesLabel=groups.filter(([id])=>[...cl.ids].some(key=>find(key)?.group===id)).map(([,label])=>label).join(' / ');
  const packagePath='<div class="sx-export-layout"><div class="sx-export-tree" aria-label="导出目录结构"><p class="sx-export-kicker">生成位置</p><code>导出目录/<br>└── 2026-10-23/<br>&nbsp;&nbsp;&nbsp;&nbsp;└── 备份包-14-30-12-8F3A/<br>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;├── 本次所选配置…<br>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;├── 模版与所选版本<br>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;└── manifest / 校验信息</code><p class="sx-muted">按日期归组，一次导出一个可独立拷贝的备份包。同日多次导出各自建包，不混写、不覆盖。</p></div><div class="sx-export-summary"><strong>'+cl.ids.size+' 项配置 · 同一备份包</strong><p class="sx-muted">'+(count?'含 '+count+' 项凭据；可选择加密，不加密则凭据以明文写入，文件可被直接读取。口令始终不写入包。':'不含凭据，可直接使用完整性校验。')+'复制整个备份包即可带到另一台设备。</p><p class="sx-muted">包内按 '+scopesLabel+' 分组，导入时仍按所选范围识别。</p></div></div>';
  const body=destination+packagePath+platformAssociationsHTML([...cl.ids].map(id=>({id,name:find(id)?.name,extensionPayload:clone(window.extensionPrototype.syncObject(id))})),'export')+extensionTransferHTML([...cl.ids].map(id=>({id})))+sourceBlock+'<div class="sx-info"><strong>'+cl.ids.size+' 项配置 · '+(count?'含 '+count+' 项凭据':'不含任何凭据')+'</strong><br>渠道 Key '+secrets.keys.length+' 项 / 账户登录态 '+secrets.accounts.length+' 项 / 扩展凭据 '+secrets.extensions.length+' 项 / WebDAV 密码 '+secrets.endpointPasswords.length+' 项</div>'+check('id="sxEncrypt"','加密配置包',false,false)+'<div id="sxPassFields" hidden><div class="sx-grid"><label class="sx-field"><span>加密口令（至少 8 位）</span><input id="sxPass" class="input" type="password" autocomplete="new-password"></label><label class="sx-field"><span>再次输入</span><input id="sxPassAgain" class="input" type="password" autocomplete="new-password"></label></div></div><details><summary>查看内容清单</summary><pre>'+esc(JSON.stringify(manifest,null,2))+'</pre></details>';
  showDialog('export','确认导出',body,'生成配置包',()=>{
   const encrypted=$('#sxEncrypt').checked,p=$('#sxPass').value;if(encrypted&&(p.length<8||p!==$('#sxPassAgain').value)){setText('#sxError','请输入至少 8 位的加密口令，并保持两次一致。');return;}// 文件保护可选；明文说明已在当前确认中展示，不增加确认步骤。
   const custom=$('[name=sxDestination]:checked')?.value==='custom';const dir=custom?($('#sxDirPath')?.value.trim()||''):'';if(custom&&!dir){setText('#sxError','请选择自定义目录，或改回默认导出目录。');return;}const picked=pickedSources(),deferredIds=conflicts.filter(id=>!picked[id]),ids=[...cl.ids].filter(id=>!deferredIds.includes(id));if(!ids.length){setText('#sxError','请为同名 MCP 选择来源，或调整导出范围。');return;}
   const mcpSources=Object.fromEntries(conflicts.filter(id=>picked[id]).map(id=>[id,picked[id]])),kept=new Set(ids),exported={...manifest,objects:ids,credentials:Object.fromEntries(Object.entries(manifest.credentials).map(([k,v])=>[k,v.filter(id=>kept.has(id))])),mcpSources,snapshot:{...manifest.snapshot,extensions:manifest.snapshot.extensions?{...manifest.snapshot.extensions,objects:manifest.snapshot.extensions.objects.filter(x=>kept.has(x.id))}:null}};
   const exportRoot=dir||'默认导出目录',packagePathShown=exportRoot+'/2026-10-23/备份包-14-30-12-8F3A',operationId=++runSerial;
   const failed=scenario==='backup-fail',receipt={tab:'file',kind:'export',operationId,total:ids.length,skipped:deferredIds.length,packagePath:failed?null:packagePathShown,protection:{encrypted},failed:failed?[{id:'export',reason:'目标目录无法写入'}]:[],errorCode:failed?'write':'none'};
   if(!failed){for(const [id,source] of Object.entries(mcpSources))managedMcp[id]={source};}
   if(!failed)lastExport={...exported,encrypted,destination:dir||'default',packagePath:packagePathShown,simulated:true};
   closeModal();render();recordActivity('export',ids.length,!failed);publishReceipt(failed?'sync.export.failed':'sync.export.complete',receipt);
  },{annotationText:['选择范围 → 确认内容 → 生成配置包','未选择的凭据不进入包，也不会由模版依赖重新带入。']});
 }
 // 一次预览只使用一份来源快照，不把各对象分别拼成双向结果。
 function updateRows(source=snapshotSource()){return catalog.filter(eligible).map(x=>({id:x.id,name:x.name,group:x.group,extensionPayload:x.extensionObject?scopedExtensionPayload(x.id,source==='remote'?extensionRemote[x.id]:window.extensionPrototype.syncObject(x.id),chosenPlatforms(state().selected)):undefined,direction:direction(x.id,source),local:clone(versions.local[x.id]),remote:clone(versions.remote[x.id])})).map(x=>x.group==='mcp'&&x.direction==='upload'&&conflict(x.id)?{...x,conflict:conflict(x.id)}:x);}
 function rememberScope(){savedScopes.file=signature(scopes.file);savedScope=savedScopes.webdav=signature(scopes.webdav);}
 function changeSummary(row){
  if(row.direction==='skip')return '内容一致，保持当前配置';
  if(row.conflict)return '同名定义在 '+row.conflict.clients.join('、')+' 中不同（差异字段：'+row.conflict.fields.join('、')+'），请选择来源';
  if(!row.local||!row.remote)return '新增配置';
  if(row.group==='channels'){
   const m=window.modelPrototype?.state(),incoming=window.modelPrototype?.syncFixture().channels.find(x=>x.id===row.id)?.provider,current=m?.saved.providers.find(x=>'channel:'+x.id===row.id);
   if(incoming&&current){const before=(row.direction==='upload'?incoming:current).models.filter(x=>x.selected).length,after=(row.direction==='upload'?current:incoming).models.filter(x=>x.selected).length;return '更新已选模型 '+before+' → '+after+'；应用来源中的模型选择、顺序与服务节点';}
  }
  if(find(row.id)?.extensionObject){const v=row.extensionPayload;if(v?.schemaVersion===1)return '旧版逐平台启用关系需转换；本次保留本机内容';if(row.group==='skills')return '更新 Skill 内容与资源文件；共享源'+(v?.enabled?'启用':'停用')+'对已映射平台同步生效';if(row.group==='extension-platforms')return '保存所选平台关联；未映射也可同步，本机目录映射保持';return '更新 MCP 定义与目标平台；本机路径需在接收设备配置';}
  if(row.group==='templates'){const incoming=window.modelPrototype?.syncFixture().templates.find(x=>'template:'+x.id===row.id);if(incoming)return '更新模版内容至 v'+incoming.version.n+'，已有历史版本保留';}
  const summaries={channels:'更新服务节点、模型选择与顺序',templates:'更新已保存模版内容',accounts:'更新账户信息，保留本机登录',skills:'替换Skill内容；移除旧文件 scripts/legacy.js，新增 references/checklist.md',mcp:'替换MCP定义与所选凭据',preferences:'更新所选桌面偏好',endpoints:'导入连接配置，不切换当前连接'};
  return summaries[row.group]||'更新所选配置';
 }
 function sourcePicker(id,empty){const c=conflict(id);if(!c)return '';return '<label class="sx-mcp-source"><span>采用来源</span><select data-sx-mcp-source="'+esc(id)+'" aria-label="'+esc(find(id)?.name||id)+' 采用来源"><option value="">'+empty+'</option>'+c.clients.map(n=>'<option value="'+esc(n)+'">'+esc(n)+'</option>').join('')+'</select></label>';}
 function pickedSources(){return Object.fromEntries($$('[data-sx-mcp-source]').map(el=>[el.dataset.sxMcpSource,el.value]));}
 function changesHTML(rows){
  const changed=rows.filter(x=>x.direction!=='skip'),same=rows.length-changed.length;
  return '<div class="sx-preview-summary"><strong>'+changed.length+' 项变化</strong><span class="sx-muted">'+same+' 项保持不变</span></div>'+(changed.length?'<ul class="sx-outcomes sx-change-list">'+changed.map(x=>'<li><div><strong>'+esc(x.name)+'</strong><small>'+esc(changeSummary(x))+'</small>'+(x.conflict?sourcePicker(x.id,'暂不同步'):'')+'</div><span class="sx-mark">'+(x.conflict?'选择来源':!x.local||!x.remote?'新增':'更新')+'</span></li>').join('')+'</ul>':'<p class="sx-muted">所选内容已一致，无需更新。</p>');
 }
 const directionStyle=document.createElement('style');directionStyle.id='sxDirectionStyle';directionStyle.textContent='.sx-source-facts{align-items:center}.sx-source-facts dd{min-width:0}.sx-source-field{display:block;width:min(100%,320px)}.sx-source-field select{width:100%}.sx-source-note{margin:8px 0 16px}';document.head.append(directionStyle);
 function syncDirectionOptions(mode,blocked=false){
  return '<dl class="sx-detail-list sx-snapshot-facts sx-source-facts"><dt><label for="sxDirection">本次来源</label></dt><dd><div class="sx-source-field"><select id="sxDirection" name="sxDirection" aria-label="本次来源"'+(blocked?' disabled':'')+'>'+[['upload','本机配置 → 云端（上传）'],['download','云端配置 → 本机（下载）']].map(([value,label])=>'<option value="'+value+'"'+(mode===value?' selected':'')+'>'+label+'</option>').join('')+'</select></div></dd></dl>';
 }
 function syncPreview(mode='auto'){
  mode=['upload','download'].includes(mode)?mode:'auto';
  catalog=collect();pruneUnavailablePlatforms();rememberScope();refreshLocalVersions();const decision=savedContentDecision(),ids=closure(scopes.webdav).ids;
  const blocked=scenario==='network-error',manual=mode!=='auto',fallback=!manual&&decision.status==='unknown';
  const selectedMode=manual?mode:decision.source==='local'?'upload':decision.contentVersions.remote.exists?'download':'upload';
  const source=!manual&&decision.source==='unchanged'?'unchanged':selectedMode==='upload'?'local':'remote';
  const options=syncDirectionOptions(selectedMode,blocked);
  const bindDirection=()=>$('#sxDirection')?.addEventListener('change',e=>{syncPreview(e.target.value);$('#sxDirection')?.focus();});
  if(blocked){
   showDialog('sync-preview','同步预览',options+'<div class="sx-version-unknown" role="alert"><h3>云端读取失败</h3><p>请先恢复连接并重新读取；当前不能生成上传或下载预览。</p></div>','确认更新',()=>{},{mode:'confirm',annotationText:'手动方向仅替代本次判新，不绕过云端读取、身份、凭据或内容保护；读取失败禁用方向与确认。'});
   preview={source:'undetermined',directionMode:mode,rows:[],blocked:true};$('#modalConfirm').disabled=true;return;
  }
  const chosen=['local','remote'].includes(source)?source:'local';
  if(!decision.contentVersions[chosen].exists&&source!=='unchanged'){
   showDialog('sync-preview','同步预览',options+'<p class="sx-error" role="alert">'+(chosen==='local'?'本机':'云端')+'没有所选内容，不能作为本次来源。请调整范围或方向。</p>'+versionFacts(decision),'确认更新',()=>{},{mode:'confirm'});
   preview={source,directionMode:mode,rows:[],blocked:true};$('#modalConfirm').disabled=true;bindDirection();return;
  }
  const rows=updateRows(chosen).filter(x=>ids.has(x.id)),secrets=clone(secretIds(scopes.webdav));
  const frozen={source,directionMode:mode,selectedDirection:selectedMode,defaultReason:manual?'manual':fallback?'remote-presence':decision.status==='same'?'same-content':!decision.contentVersions.remote.exists||!decision.contentVersions.local.exists?'source-presence':'saved-content',snapshot:source==='unchanged'?null:clone(snapshots[chosen]),decision:clone(decision),connectionIdentity:connectionIdentity(connection),scopeKey:signature(scopes.webdav),rows:clone(rows),agentPlatforms:chosenPlatforms(scopes.webdav.selected),secrets,backup:source==='remote',encrypted:protection.enabled,scenario};
  const changed=rows.some(x=>x.direction!=='skip'),action=selectedMode==='upload'?'确认上传':'确认下载';
  const warning=manual?'<p class="sx-muted sx-source-note" role="status">已手动选择'+(selectedMode==='upload'?'上传，以本机配置更新云端':'下载，以云端配置更新本机')+'。确认后更新所选范围。</p>':fallback?'<p class="sx-muted sx-source-note" role="status">保存时间不足以判定新版；'+(decision.contentVersions.remote.exists?'云端已有配置，默认下载。':'云端暂无配置，默认上传。')+'可切换来源，确认后才更新。</p>':'<p class="sx-muted sx-source-note">'+(source==='unchanged'?'所选内容已一致，无需更新。':!decision.contentVersions.remote.exists?'云端暂无配置，默认上传。可切换来源，确认后才更新。':!decision.contentVersions.local.exists?'云端已有配置，本机暂无所选内容，默认下载。':'已按内容保存时间预选，可切换来源。')+'</p>';
  const facts=versionFacts(decision);
  showDialog('sync-preview','同步预览',options+warning+facts+(source==='remote'&&changed?check('id="sxPreviewBackup"','更新前备份',true):'')+changesHTML(rows)+platformAssociationsHTML(rows)+extensionTransferHTML(rows)+(frozen.encrypted?'<label class="sx-field"><span>同步保护口令</span><input id="sxPreviewPass" type="password" class="input" autocomplete="off" placeholder="请输入保护口令" value="'+esc(protection.passphrase)+'"></label>':''),action,()=>{
   if(!changed)return;
   refreshLocalVersions();
   if(frozen.scenario!==scenario||frozen.connectionIdentity!==connectionIdentity(connection)||frozen.scopeKey!==signature(scopes.webdav)||JSON.stringify(frozen.decision)!==JSON.stringify(savedContentDecision())){setText('#sxError','连接、内容、范围或保存版本已变化，请关闭后重新预览。');$('#modalConfirm').disabled=true;return;}
   if(frozen.encrypted&&!$('#sxPreviewPass')?.value){setText('#sxError','请填写保护口令。');return;}
   if(frozen.encrypted)protection.passphrase=$('#sxPreviewPass').value;
   const backup=!!$('#sxPreviewBackup')?.checked,picked=pickedSources();
   const rows=frozen.rows.map(x=>!x.conflict?x:picked[x.id]?{...x,mcpSource:picked[x.id]}:{...x,direction:'skip',deferred:true,reason:'未选择来源，本次未同步'});
   closeModal();startUpdate('webdav',rows,frozen.secrets,{encrypted:frozen.encrypted},undefined,{backup,source:frozen.source,snapshot:frozen.snapshot,scenario:frozen.scenario,agentPlatforms:frozen.agentPlatforms});
  },{mode:changed?'confirm':'view',hideConfirm:!changed,cancelLabel:'取消',annotationText:'S22-POST-01最新采用：原来源行改为本次来源下拉，上传/下载两项。可靠保存时间可判新则预选新版；无法判新时云端有有效配置默认下载，否则默认上传；读取失败不能当作空云端。切换重算预览，不直接执行；确认才更新所选范围。下载默认备份，取消/X/Escape不执行，重开按当前事实重新预选。时间未知仍显示未知，不绕过强ETag/CAS、范围/连接/内容变化及凭据保护；仅内存原型，软件未实现。'});
  preview=frozen;$('#modalConfirm').disabled=!changed;bindDirection();
 }
 function versionFacts(decision){return '<dl class="sx-detail-list sx-version-facts">'+['local','remote'].map(side=>{const v=decision.contentVersions[side],known=v.timeKind==='saved'&&v.bound!==false&&!v.clockSkew&&Number.isFinite(Date.parse(v.savedAt));return '<dt>'+(side==='local'?'本机':'云端')+'保存时间</dt><dd>'+esc(known?v.savedAt.replace('T',' ').replace(/\.\d{3}Z$/,' UTC'):'未知')+'</dd>';}).join('')+'</dl>';}
 function directImportRows(kind){
  const present=new Set([...catalog.filter(x=>x.extensionObject&&(!x.defaultExcluded||x.platformAssociation)).map(x=>x.id),'template:gpt','channel:newapi','channel:legacy','skill:design','mcp:documentation','preferences:desktop','endpoint:personal','account:chatgpt','account:chatgpt-work','account:antigravity','account:device']);
  const items=updateRows('remote').filter(x=>present.has(x.id)).map(x=>({...x,extensionPayload:find(x.id)?.extensionObject?clone(extensionRemote[x.id]):x.extensionPayload,direction:Date.parse(snapshots.local.updatedAt)>Date.parse(snapshots.remote.updatedAt)?'skip':x.direction}));
  if(kind==='extension-legacy')return items.filter(x=>x.group==='skills'||find(x.id)?.platformAssociation).map(x=>x.group==='skills'?{...x,direction:'download',extensionPayload:{schemaVersion:1,enablement:{claude:true,codex:false}}}:x);
  if(kind==='keyless')return items.filter(x=>['channels','templates'].includes(x.group));
  if(kind==='config-only')return items.filter(x=>x.group==='endpoints');
  if(kind==='legacy')return items.filter(x=>x.group==='preferences');
  return items;
 }

 function importEffectiveRows(){
  const platforms=chosenPlatforms(importPlan.selected);
  return (importPlan.rows||[]).filter(x=>importPlan.selected.has(x.id)&&(!extensionContent(find(x.id))||platforms.length)).map(x=>({...x,extensionPayload:scopedExtensionPayload(x.id,x.extensionPayload,platforms)}));
 }
 function updateImportScopeView(){
  const platforms=chosenPlatforms(importPlan.selected);
  for(const el of $$('[data-sx-import-object]')){
   const x=find(el.dataset.sxImportObject),gated=extensionContent(x)&&!platforms.length;
   el.disabled=gated;el.checked=gated?false:importPlan.selected.has(x?.id);
  }
  const scopeNote=$('#sxImportPlatformScope');if(!scopeNote)return;scopeNote.textContent=syncScopeNote(importPlan.selected);
  const previous=$('#modalBody .sx-platform-associations');if(previous)previous.remove();
  const rows=importEffectiveRows(),markup=platformAssociationsHTML(rows);if(markup)$('#modalBody .sx-dialog').insertAdjacentHTML('afterbegin',markup);
  const old=$('#modalBody .sx-extension-transfer');if(old)old.remove();
  const note=extensionTransferHTML(rows);if(note)scopeNote.insertAdjacentHTML('afterend',note);
 }
 function importForm(){
  const rows=importPlan.rows||[],effectiveRows=importEffectiveRows();
  return '<div class="sx-field"><span>配置文件</span><div class="sx-file-picker">'+btn('选择文件','choose-import-file')+'<span id="sxImportFileName" aria-live="polite">'+esc(importFile?.file||'尚未选择文件')+'</span></div></div>'+
   '<label class="sx-field" id="sxUnlockField" '+(!importFile?.encrypted?'hidden':'')+'><span>解密口令</span><input class="input" id="sxUnlock" type="password" autocomplete="off" placeholder="请输入配置包的加密口令"></label>'+
   (rows.length?platformAssociationsHTML(effectiveRows)+'<p class="sx-platform-help" id="sxImportPlatformScope" role="status">'+esc(syncScopeNote(importPlan.selected))+'</p>'+extensionTransferHTML(effectiveRows)+'<p class="sx-muted">文件版本：'+esc(snapshots.remote.updatedAt.replace('T',' ').replace('+08:00',''))+'</p>'+check('id="sxImportBackup"','更新前备份',importPlan.backup)+'<fieldset class="sx-import-scope"><legend>导入范围</legend>'+groups.map(([group,label])=>{const items=rows.filter(x=>x.group===group);return items.length?'<div class="sx-import-group"><strong>'+label+'</strong>'+items.map(x=>check('data-sx-import-object="'+esc(x.id)+'"',esc(x.name)+'<small class="sx-import-change">'+esc(changeSummary(x))+'</small>',importPlan.selected.has(x.id))).join('')+'</div>':'';}).join('')+'</fieldset>'+check('data-sx-import-keys','包含包内渠道 API Key',importPlan.keys,!importSecretIds('keys').length)+check('data-sx-import-secrets','包含包内扩展凭据',importPlan.extensions,!importSecretIds('extensions').length):'');
 }
 function importSecretIds(kind){
  return (importFile?.credentials?.[kind]||[]).filter(id=>importEffectiveRows().some(x=>x.id===id));
 }
 function syncImportControls(){
  for(const [kind,attr] of [['keys','keys'],['extensions','secrets']]){
   const input=$('[data-sx-import-'+attr+']');if(!input)continue;
   input.disabled=!importSecretIds(kind).length;input.checked=!input.disabled&&!!importPlan[kind];
  }
  $('#modalConfirm').disabled=!importFile||!importEffectiveRows().length;
 }
 function renderImport(){
  showDialog('import','导入配置',importForm(),'确认更新',()=>{
   if(!importFile)return;
   if(importFile.kind==='corrupt'){setText('#sxError','文件无法读取，请重新选择配置包。');return;}
   if(importFile.encrypted&&$('#sxUnlock').value!=='demo-sync'){setText('#sxError','解密口令不正确，请重新输入。');return;}
   if(!importPlan.selected.size)return;
   const rows=importEffectiveRows();
   const secrets={keys:importPlan.keys?importSecretIds('keys'):[],extensions:importPlan.extensions?importSecretIds('extensions'):[]};
   const backup=!!$('#sxImportBackup')?.checked;closeModal();startUpdate('file',rows,secrets,undefined,undefined,{backup,source:'file',snapshot:clone(snapshots.remote),agentPlatforms:chosenPlatforms(importPlan.selected)});
  },{prototypeAnnotation:{title:'导入示例与测试口令',body:'加密示例包的解密口令：demo-sync。选择示例文件后，填入“解密口令”框即可；不加密的示例包无需口令。此口令仅供原型模拟，不是WebDAV连接密码，也不用于真实文件。'},annotationText:['S18方案演示：选择文件与范围 → 预览变化 → 确认后备份并更新；不会读取真实文件。','官方openai不进入导入范围。同一渠道按固定身份原位更新，不创建导入副本。凭据选项由包内实际内容与已选对象决定，与是否加密无关；未勾选凭据表示保留本机已有值。','未保存编辑的处理仍待定，本轮示例更新mock已保存文件并保留原编辑缓冲；不把此建议当已确认生产规则。']});
  updateImportScopeView();syncImportControls();
 }
 function importStart(context={}){importFile=null;importPlan={rows:[],selected:new Set(),keys:importCredentialPrefs.keys,extensions:importCredentialPrefs.extensions,context:context.scope||null,backup:true};renderImport();}
 function startUpdate(mode,rows,secrets=secretIds(scopes.webdav),transferProtection,retryOperationId,options={}){
  if(run)return;
  const encrypted=transferProtection?.encrypted??(mode==='webdav'?protection.enabled:!!importFile?.encrypted);
  if(mode==='webdav'&&encrypted&&!protection.passphrase){result={tab:mode,title:'请填写保护口令',detail:'填写后再同步。',inputNeeded:true};render();$('#sxProtectionPass')?.focus();return;}
  result=null;channelAdoption=null;
  const operation={id:retryOperationId||++runSerial,tab:mode,ids:rows.map(x=>x.id),rows:clone(rows),secrets:clone(secrets),protection:{encrypted},scenario:options.scenario||scenario,backup:!!options.backup,agentPlatforms:clone(options.agentPlatforms??chosenPlatforms(state().selected)),source:options.source||snapshotSource(),snapshot:clone(options.snapshot||snapshots[snapshotSource()])};run=operation;render();document.dispatchEvent(new CustomEvent('prototype:sync-started',{detail:{mode}}));
  window.prototypeNotificationPublisher.publish(mode==='file'?'sync.import.started':'sync.started',{operationId:String(operation.id),total:rows.length});
  setTimeout(()=>completeUpdate(operation),650);
 }
 function completeUpdate(operation){
  if(run!==operation)return;
  const failed=[],done=[],skips=[],issue=operation.scenario;
  const availablePlatforms=operation.agentPlatforms.filter(id=>window.extensionPrototype.platformSyncInfo(id)?.installed);const rows=operation.rows.map(row=>row.id.startsWith('skill-platform:')&&!window.extensionPrototype.platformSyncInfo(row.id.slice(15))?.installed?{...row,direction:'skip',reason:'本机已无法发现平台，本次跳过'}:extensionContent(find(row.id))&&!availablePlatforms.length?{...row,direction:'skip',reason:'所选 Agent 平台均未发现，本次拓展内容未同步'}:row);
  const changed=rows.filter(x=>x.direction!=='skip'),existingBackup=backups.find(x=>x.operationId===operation.id),backupFailed=operation.backup&&!existingBackup&&issue==='backup-fail';
  if(existingBackup)operation.backupId=existingBackup.id;
  if(operation.backup&&changed.length&&!existingBackup&&!backupFailed&&issue!=='network-error'){
   const model=window.modelPrototype?.state(),ids=new Set(changed.map(x=>x.id));
   const backup={id:'sync-backup-'+operation.id,operationId:operation.id,ids:[...ids],items:changed.map(x=>({id:x.id,version:clone(versions.local[x.id]),content:clone(demoObjects[x.id]),extensionPayload:clone(window.extensionPrototype?.syncObject(x.id))})),channels:clone(model?.saved.providers.filter(x=>ids.has('channel:'+x.id))||[]),templates:clone(model?.templates.filter(x=>ids.has('template:'+x.id))||[])};
   backups.push(backup);operation.backupId=backup.id;
  }
  const partialId=rows.find(x=>x.group==='mcp')?.id||rows.find(x=>x.direction!=='skip')?.id;
  for(const row of rows){
   if(row.direction==='skip'){skips.push({...row,status:'skipped'});continue;}
   if(issue==='network-error'||backupFailed||issue==='write-fail'||issue==='partial'&&row.id===partialId){failed.push({...row,status:'failed',reason:issue==='network-error'?'连接中断，内容未更新':backupFailed?'更新前备份未完成，原内容保留':issue==='write-fail'?'本地文件无法写入，原内容保留':'目标文件被占用，旧内容保留'});continue;}
   if(row.direction==='download'&&['channels','templates'].includes(row.group)){
    const model=window.modelPrototype;
    const applied=model.applySyncUpdates({contract:model.syncFixture().contract,source:operation.tab==='file'?'file':'webdav',channels:row.group==='channels'?[{id:row.id,credential:operation.secrets.keys.includes(row.id)}]:[],templates:row.group==='templates'?[row.id.slice(9)]:[]});
    if(!applied.ok){failed.push({...row,status:'failed',reason:applied.message});continue;}
    if(row.group==='channels')channelAdoption={status:'written',target:'mock-current-channel-file',source:operation.tab,channels:applied.channels};
   }

   if(find(row.id)?.extensionObject){
    if(row.direction==='download'){
     const applied=window.extensionPrototype.applySyncObject(row.id,row.extensionPayload,{platformIds:operation.agentPlatforms.filter(id=>window.extensionPrototype.platformSyncInfo(id)?.installed)});
     if(!applied.ok){failed.push({...row,status:'failed',reason:applied.message});continue;}
    }else extensionRemote[row.id]=mergeRemoteExtension(row.id,row.extensionPayload,availablePlatforms);
   }
   const version=clone(row.direction==='download'?row.remote:row.local)||{updatedAt:stamp(5),revision:2,digest:row.id+':v2'};
   versions.local[row.id]=clone(version);versions.remote[row.id]=clone(version);
   if(row.direction==='download')localFingerprints[row.id]=fingerprint(row.id);
   if(row.direction==='download')demoObjects[row.id]={revision:version.revision,files:row.group==='skills'?(row.extensionPayload?.files||[]).map(f=>f.path):null};
   if(row.group==='mcp'){if(row.mcpSource)managedMcp[row.id]={source:row.mcpSource};else if(row.direction==='download'){managedMcp[row.id]={source:'云端定义'};delete mcpVariants[row.id];}}
   done.push({...row,status:'completed',credential:operation.secrets.keys.includes(row.id)||operation.secrets.extensions.includes(row.id)});
  }
  run=null;serial++;
  const downloaded=done.filter(x=>x.direction==='download').length,uploaded=done.filter(x=>x.direction==='upload').length;
  const title=failed.length?(done.length?'部分同步完成':'同步未完成'):operation.tab==='file'?'导入完成':'同步完成';
  const deferred=skips.filter(x=>x.deferred).length,detail='已更新 '+done.length+' 项，未变化 '+(skips.length-deferred)+' 项'+(deferred?'，待选来源 '+deferred+' 项':'')+(failed.length?'，'+failed.length+' 项未完成':'');
  result={extensionIds:done.filter(x=>x.direction==='download'&&find(x.id)?.extensionObject).map(x=>x.id),tab:operation.tab,operationId:operation.id,title,detail:detail+'。',downloaded,uploaded,skipped:skips.length,failed,items:[...done,...skips,...failed],protection:clone(operation.protection),source:operation.source,snapshot:clone(operation.snapshot),backupId:operation.backupId||null};lastRun=clone(result);failedRun=failed.length?{...operation,rows:failed.map(({status,reason,...x})=>x)}:null;
  lastComparison=null;render();
  recordActivity(operation.tab==='file'?'import':'sync',done.length+skips.length,failed.length===0,{uploaded,downloaded,remoteTime:operation.snapshot?.updatedAt});
  const previous=receipts.get(String(operation.id)),items=new Map((previous?.items||[]).map(x=>[x.id,x]));
  for(const item of result.items)items.set(item.id,item);
  const all=[...items.values()],failedItems=all.filter(x=>x.status==='failed');
  publishReceipt(operation.tab==='file'?(failedItems.length?'sync.import.failed':'sync.import.complete'):(failedItems.length?'sync.failed':'sync.complete'),{...result,items:all,total:all.length,failed:failedItems,downloaded:all.filter(x=>x.status==='completed'&&x.direction==='download').length,uploaded:all.filter(x=>x.status==='completed'&&x.direction==='upload').length,skipped:all.filter(x=>x.status==='skipped').length,errorCode:failedItems.length?(backupFailed?'backup':issue==='network-error'?'connection':all.some(x=>x.status==='completed')?'partial':'write'):'none'});
 }
 function retryFailed(){if(!failedRun||run)return;const p=failedRun;startUpdate(p.tab,p.rows,p.secrets,p.protection,p.id,{backup:p.backup,source:p.source,snapshot:p.snapshot,scenario:'normal',agentPlatforms:p.agentPlatforms});if(run)run.scenario='normal';}
 function resultDetails(record=result){
  if(!record?.items?.length)return;
  showDialog('result','同步结果','<p class="sx-muted">传输保护：'+(record.protection?.encrypted?'加密':'不加密')+'</p>'+platformAssociationsHTML(record.items.filter(x=>x.status==='completed'))+extensionTransferHTML(record.items.filter(x=>record.extensionIds?.includes(x.id)),true)+'<ul class="sx-outcomes">'+record.items.map(x=>'<li><div><strong>'+esc(x.name)+'</strong><small>'+esc(x.reason||(x.direction==='skip'?'内容一致，无需更新':'已按本次来源版本更新'))+'</small></div><span class="sx-mark">'+(x.status==='failed'?'失败':x.status==='skipped'?'跳过':'完成')+'</span></li>').join('')+'</ul>'+(record.failed?.length&&failedRun?.id===record.operationId?btn('重试失败项','receipt-retry','data-id="'+record.operationId+'"'):'') ,'关闭',closeModal,{hideConfirm:true,cancelLabel:'关闭',annotationText:'同一操作结果的只读明细，由注册通知或现有结果入口查看；不是新通知表面。'});
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
  if(run)return;if(!connection.configured){connectionDialog();return;}
  refreshLocalVersions();const ids=closure(scopes.webdav).ids;
  observeConnection(scenario==='network-error'?'failure':'success','同步读取',scenario==='network-error'?null:snapshots.remote.updatedAt);
  syncPreview();
 }
 function testConnection(){
  if(!validateConnection())return;
  const request=++connectionRequest,session=dialogSession,identity=connectionIdentity(connection),formRevision=connectionFormRevision;
  const unchanged=$('#sxDavURL').value.trim().replace(/\/$/,'')===connection.url&&$('#sxDavPath').value.trim()===connection.path&&$('#sxDavUser').value.trim()===connection.user&&!$('#sxDavPassword').value;
  const failed=scenario==='network-error',operationId=connectionTestId||++runSerial;setText('#sxConnectionTest','正在检测…');if(unchanged)observeConnection('checking');
  setTimeout(()=>{
   if(request!==connectionRequest||identity!==connectionIdentity(connection))return;
   if(unchanged)observeConnection(failed?'failure':'success');
   if(session!==dialogSession||dialogKind!=='connection')return;
   connectionFormProof={baseline:identity,session,formRevision,status:failed?'failure':'success',observedAt:observationTime()};
   setText('#sxConnectionTest',(unchanged?'当前已保存连接':'未保存连接')+(failed?'最近检测失败':'最近检测成功')+' · '+observationTime());
   connectionTestId=failed?operationId:null;
   publishReceipt(failed?'sync.connection.failed':'sync.connected',{operationId,kind:'connection',total:1,failed:failed?[{id:'connection'}]:[],errorCode:failed?'connection':'none'});
  },260);
 }
 function connectionDialog(){
  connectionFormProof=null;connectionFormRevision++;
  showDialog('connection','WebDAV 连接','<label class="sx-field"><span>服务地址</span><input class="input" id="sxDavURL" value="'+esc(connection.url)+'"></label><label class="sx-field"><span>独立配置同步目录</span><input class="input" id="sxDavPath" value="'+esc(connection.path)+'"></label><div class="sx-grid"><label class="sx-field"><span>用户名</span><input class="input" id="sxDavUser" value="'+esc(connection.user)+'"></label><label class="sx-field"><span>连接密码</span><input class="input" id="sxDavPassword" type="password" placeholder="本机已保存 · 留空保留"></label></div><div class="sx-actions">'+btn('测试连接','test-connection')+'</div><p class="sx-muted" id="sxConnectionTest" role="status">'+esc(connectionStatus().label)+'</p><div id="sxImportedConnectionEntry" '+(!importedEntry()?'hidden':'')+'>'+importedEntry()+'</div>','保存连接',()=>{
   if(!validateConnection())return;
   const proof=connectionFormProof&&connectionFormProof.session===dialogSession&&connectionFormProof.formRevision===connectionFormRevision&&connectionFormProof.baseline===connection.revision?connectionFormProof:null;const next={...connection,configured:true,url:$('#sxDavURL').value.trim().replace(/\/$/,''),path:$('#sxDavPath').value.trim(),user:$('#sxDavUser').value.trim(),credentialConfigured:connection.credentialConfigured||!!$('#sxDavPassword').value,credentialRevision:connection.credentialRevision+($('#sxDavPassword').value?1:0)};if(connectionFields(next)!==connectionFields(connection)){next.revision='mock-endpoint:'+ ++connectionRevisionSerial;connectionRequest++;connectionObservation=null;}connection=next;closeModal();if(proof)observeConnection(proof.status,'保存前表单检测',null,proof.observedAt);projectConnection();render();publishReceipt('sync.connection.saved',{operationId:++runSerial,kind:'connection',total:1});
  },{annotationText:'WebDAV 配置可通过文件同步迁移，导入后需另行启用。当前示例不发起任何网络请求，请勿填写真实凭据。同步加密口令与连接密码分开；同步包不会覆盖正在使用的连接、定时规则或本机范围。'});
 }
 function validateConnection(){
  try{const u=new URL($('#sxDavURL').value.trim());if(u.protocol!=='https:'||u.username||u.password||u.search||u.hash)throw Error();if(!$('#sxDavPath').value.startsWith('/')||!$('#sxDavUser').value.trim())throw Error();return true;}catch{setText('#sxError','请填写不含内嵌凭据的 HTTPS 地址、以 / 开头的目录和用户名。');return false;}
 }
 function open(options={}){
  catalog=collect(options.action==='export'&&options.source==='draft'?'draft':'saved');tab=options.tab==='webdav'?'webdav':'file';query='';channelAdoption=null;
  if(options.scope){const s=state(),platforms=chosenPlatforms(s.selected),explicitPlatforms=options.scope.some(id=>id.startsWith('skill-platform:')),contentOnly=!explicitPlatforms&&options.scope.some(id=>extensionContent(find(id))),wanted=contentOnly?[...options.scope,...platforms.map(id=>'skill-platform:'+id)]:options.scope;s.selected=new Set(wanted.filter(id=>find(id)&&eligible(find(id))&&(tab==='file'||find(id).group!=='endpoints')));s.keys=new Set(tab==='webdav'?[...s.selected].filter(id=>find(id)?.key):[]);s.accountInfoOnly.clear();s.endpointConfigOnly.clear();s.extensions=new Set(tab==='webdav'?[...s.selected].filter(id=>find(id)?.extensionSecret):[]);}
  rememberScope();
  const stayOnPage=!!options.stayOnPage&&['import','export','check','connection','result'].includes(options.action);
  if(!stayOnPage)setRoute('sync?tab='+tab);
  render(true);if(!stayOnPage)host.closest('.main').scrollTop=0;
  if(options.action==='import')importStart(options);else if(options.action==='export')exportPreview(options.source);else if(options.action==='connection')connectionDialog();else if(options.action==='check')syncCheck();else if(options.action==='result')resultDetails();
 }
 document.addEventListener('click',e=>{
  const el=e.target.closest('[data-sx]');if(!el||el.disabled)return;const a=el.dataset.sx,id=el.dataset.id;
  if(a==='extensions'){closeModal();setRoute('extensions');return;}
  if(a==='result-details')resultDetails();
  else if(a==='preview-close')closeModal();
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
  else if(a==='test-connection')testConnection();
  else if(a==='imported-connections')importedConnectionsDialog();
  else if(a==='retry-imported')retryImported();
  else if(a==='activate-imported')activateImported(id);
  else if(a==='test-imported')testImported(id);
  else if(a==='check')syncCheck();
 });
 document.addEventListener('change',e=>{
  const el=e.target,s=state(),focusAttrs=[...e.target.attributes].filter(a=>a.name.startsWith('data-sx-'));const restoreFocus=()=>{const match=$$('input',dialogKind?$('#modal'):host).find(x=>focusAttrs.length&&focusAttrs.every(a=>x.getAttribute(a.name)===a.value));match?.focus({preventScroll:true});};let redraw=false;
  if(el.matches('[data-sx-object]')){if(extensionContent(find(el.dataset.sxObject))&&!chosenPlatforms(s.selected).length)return;selectObject(s,el.dataset.sxObject,el.checked);redraw=true;}
  else if(el.matches('[data-sx-group],[data-sx-all]')){for(const x of catalog.filter(x=>(tab==='file'||x.group!=='endpoints')&&(!el.dataset.sxGroup||x.group===el.dataset.sxGroup)&&(x.name+' '+x.summary).toLowerCase().includes(query.toLowerCase())&&(x.group!=='accounts'||accountPlatform==='all'||x.accountPlatform===accountPlatform))){selectObject(s,x.id,el.checked);}redraw=true;}
  else if(el.matches('[data-sx-account-platform]')){accountPlatform=el.value;redraw=true;}
  else if(el.matches('[data-sx-account-info]')){el.checked?s.accountInfoOnly.add(el.dataset.sxAccountInfo):s.accountInfoOnly.delete(el.dataset.sxAccountInfo);redraw=true;}
  else if(el.matches('[data-sx-endpoint-config]')){el.checked?s.endpointConfigOnly.add(el.dataset.sxEndpointConfig):s.endpointConfigOnly.delete(el.dataset.sxEndpointConfig);redraw=true;}
  else if(el.matches('[data-sx-secret]')){const key=el.dataset.sxSecret,prop={keys:'key',extensions:'extensionSecret'}[key];s[key].clear();if(el.checked)for(const x of catalog.filter(x=>closure().ids.has(x.id)&&x[prop]))s[key].add(x.id);redraw=true;}
  else if(el.matches('[data-sx-secret-object]')){el.checked?s[el.dataset.kind].add(el.dataset.sxSecretObject):s[el.dataset.kind].delete(el.dataset.sxSecretObject);redraw=true;}
  else if(el.id==='sxEncrypt')$('#sxPassFields').hidden=!el.checked;
  else if(el.name==='sxDestination'){const box=$('#sxCustomDir');if(box)box.hidden=el.value!=='custom';if(el.value!=='custom'){const input=$('#sxDirPath');if(input)input.value='';}setText('#sxError','');}
  else if(el.matches('[data-sx-protect]')){protection.enabled=el.checked;redraw=true;}
  else if(el.matches('[data-sx-import-object]')){el.checked?importPlan.selected.add(el.dataset.sxImportObject):importPlan.selected.delete(el.dataset.sxImportObject);updateImportScopeView();syncImportControls();}
  else if(el.id==='sxImportBackup')importPlan.backup=el.checked;
  else if(el.matches('[data-sx-import-keys]'))importCredentialPrefs.keys=importPlan.keys=el.checked;
  else if(el.matches('[data-sx-import-secrets]'))importCredentialPrefs.extensions=importPlan.extensions=el.checked;
  if(redraw){rememberScope();for(const key of ['keys','accountInfoOnly','endpointConfigOnly','extensions'])for(const id of s[key])if(!s.selected.has(id))s[key].delete(id);render();restoreFocus();}
 });
 document.addEventListener('input',e=>{if(dialogKind&&e.target.closest('.sx-dialog')&&e.target.matches('input')){setText('#sxError','');if(dialogKind==='connection'){connectionRequest++;connectionFormRevision++;connectionFormProof=null;connectionTestId=null;setText('#sxConnectionTest','未保存的编辑 · 当前连接观察保持');}if(dialogKind==='activate-imported'){candidateRequest++;candidateProof=null;$('#modalConfirm').disabled=!$('#sxCandidatePassword')?.value;}}});
 host.addEventListener('input',e=>{if(e.target.id==='sxProtectionPass'){protection.passphrase=e.target.value;return;}if(e.target.id!=='sxSearch')return;query=e.target.value;searchCollapsed[tab].clear();const pos=e.target.selectionStart;render(true);$('#sxSearch').focus({preventScroll:true});try{$('#sxSearch').setSelectionRange(pos,pos);}catch{}});
 host.addEventListener('keydown',e=>{if(e.target.getAttribute('role')==='tab'&&['ArrowLeft','ArrowRight','Home','End'].includes(e.key)){e.preventDefault();tab=e.key==='Home'?'file':e.key==='End'?'webdav':tab==='file'?'webdav':'file';history.replaceState(null,'','#sync?tab='+tab);render();$('#sxTab-'+tab).focus();}});
 document.addEventListener('keydown',e=>{if(detailOpen||!dialogKind||e.key!=='Tab')return;const items=$$('button:not([disabled]),input:not([disabled]),select:not([disabled]),[tabindex="0"]',$('#modal')).filter(x=>x.getClientRects().length);if(!items.length)return;const first=items[0],last=items.at(-1);if(e.shiftKey&&document.activeElement===first){e.preventDefault();last.focus();}else if(!e.shiftKey&&document.activeElement===last){e.preventDefault();first.focus();}});
 function routeChanged(){if(document.documentElement.dataset.route!=='sync')return;const p=new URLSearchParams(location.hash.split('?')[1]||'');tab=p.get('tab')==='webdav'?'webdav':'file';catalog=collect();render();}
 new MutationObserver(routeChanged).observe(document.documentElement,{attributes:true,attributeFilter:['data-route']});
 window.addEventListener('hashchange',routeChanged);
 const scenes=[['normal','云端新版 · 预览后更新本机'],['download','云端新版 · 全范围有变化'],['upload','本机新版 · 预览后更新云端'],['same','内容相同 · 无需更新'],['partial','一项失败 · 其余完成'],['network-error','连接中断 · 旧内容保留'],['backup-fail','更新前备份失败'],['write-fail','本地写入失败 · 可重试'],['mcp-source','同名MCP定义不同 · 选择来源'],['unknown-time','时间未知 · 云端有配置默认下载'],['remote-empty','云端空目录 · 默认上传'],['same-time','无法判新 · 同刻异内容'],['legacy-mtime','无法判新 · 仅历史mtime'],['invalid-time','无法判新 · 无效时间'],['unbound-version','无法判新 · 摘要绑定不明'],['clock-skew','无法判新 · 时钟异常'],['local-no-baseline','无共同基线 · 本机11月4日较新'],['remote-no-baseline','无共同基线 · 云端11月4日较新']];
 const importSelect=$('#sxImportSample');if(importSelect&&!importSelect.querySelector('[value="extension-legacy"]'))importSelect.insertAdjacentHTML('beforeend','<option value="extension-legacy">旧版拓展包 · 启用关系待转换</option>');
 const sceneSelect=$('#sxMockScenario');if(sceneSelect)sceneSelect.innerHTML=scenes.map(([id,label])=>'<option value="'+id+'">'+label+'</option>').join('');
 function setScenario(value){if(!scenes.some(([id])=>id===value))return;scenario=value;mcpVariants=value==='mcp-source'?{'mcp:documentation':{clients:['Codex','Claude Code'],fields:['env','type']}}:{};managedMcp={};seedVersions(['download','upload','same','mcp-source'].includes(value)?value:'normal');if(['unknown-time','legacy-mtime','invalid-time','unbound-version','clock-skew','same-time'].includes(value)){
   if(value==='unknown-time')snapshots.local.timeKind='unknown';
   if(value==='legacy-mtime')snapshots.local.timeKind='mtime';
   if(value==='invalid-time')snapshots.local.savedAt='invalid';
   if(value==='unbound-version')snapshots.local.bound=false;
   if(value==='clock-skew')snapshots.local.clockSkew=true;
   if(value==='same-time')for(const side of ['local','remote'])snapshots[side].savedAt='2026-11-04T12:00:00Z';
  }
  if(value==='remote-empty'){for(const id of Object.keys(versions.remote))delete versions.remote[id];snapshots.local.timeKind='unknown';snapshots.remote={revision:0,digest:'',updatedAt:'',timeKind:'unknown'};}
  if(['local-no-baseline','remote-no-baseline'].includes(value)){const newer=value==='local-no-baseline'?'local':'remote';for(const side of ['local','remote']){snapshots[side].savedAt=side===newer?'2026-11-04T02:00:00Z':'2026-10-30T02:00:00Z';snapshots[side].updatedAt=snapshots[side].savedAt;}}
  if(sceneSelect)sceneSelect.value=value;result=null;render();}
 function setProtectionDemo(mode){
  if(!['plaintext','encrypted'].includes(mode))return;
  protection.enabled=mode==='encrypted';if(protection.enabled&&!protection.passphrase)protection.passphrase='demo-sync';
  render();window.prototypePanel?.refresh();
 }
 window.syncPrototype={open,connectionStatus,setConnectionScenario,setCandidateScenario,refreshCatalog:()=>{if(dialogKind)closeModal();catalog=collect();render();},state:()=>({connectionStatus:clone(connectionStatus()),importedConnections:clone(importedConnections),tab,accountPlatform,accountCapabilities:accountCapabilities.entries.map(x=>({...x})),selected:[...state().selected],effective:[...closure().ids],accountInfoOnly:[...state().accountInfoOnly],endpointConfigOnly:[...state().endpointConfigOnly],secrets:secretIds(),savedScope,savedScopes:clone(savedScopes),dirty:false,lastExport:clone(lastExport),recentActivity:clone(recentActivity),result:clone(result),channelAdoption:clone(channelAdoption),dialogKind,detailOpen,lastComparison:null,comparison:null,preview:clone(preview),snapshots:clone(snapshots),backups:clone(backups),serial,scenario,importSample,importFile:clone(importFile),running:!!run,lastRun:clone(lastRun),versions:clone(versions),demoObjects:clone(demoObjects),extensionRemote:clone(extensionRemote),managedMcp:clone(managedMcp),mcpVariants:clone(mcpVariants),protection:{enabled:protection.enabled,configured:!!protection.passphrase}}),setImportSample,setScenario,setProtectionDemo,openReceipt,render};
 projectConnection();routeChanged();render();
})();
