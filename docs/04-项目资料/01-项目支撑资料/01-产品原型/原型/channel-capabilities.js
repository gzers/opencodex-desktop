/* Codex 对话策略与渠道能力：仅内存原型，不执行网络、认证、模型调用或持久化。 */
(function(root,factory){
 const api=factory();if(typeof module==='object'&&module.exports)module.exports=api;
 else root.channelCapabilitiesPrototype=api;
})(typeof window==='object'?window:globalThis,function(){
 'use strict';
 const clone=v=>JSON.parse(JSON.stringify(v)),esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 const capabilities=[['call','模型调用'],['fast','Fast'],['ultra','Ultrafast'],['ws','WebSocket'],['compact','原生远程压缩'],['background','后台压缩'],['notes','历史与笔记']];
 const modelCapabilities=new Set(['call','fast','ultra','compact']);
 const supportWords={supported:'支持',unknown:'待核对',unsupported:'不支持'};
 const sceneWords={normal:'Krill 当前状态',unverified:'尚未认证',auth:'认证失败',network:'网络失败',ready:'支持且已接通'};
 function access(p){return p?.connection?.specialAccess||{site:'generic',mode:'key'};}
 function routes(p,n){
  let u;try{u=new URL(n?.url);}catch{return {origin:'',modelBase:'',models:'',native:'',valid:false};}
  const prefix=u.pathname.replace(/\/+$/,'').replace(/\/models$/,'')||'/v1';
  const preset={modelBase:prefix,models:prefix+'/models',native:access(p).site==='krill'?'/backend-api/codex':''};
  const paths=n?.paths?{...preset,...n.paths}:preset;
  return {origin:u.origin,...paths,valid:true};
 }
 function endpoint(p,n,kind='models'){const r=routes(p,n);return r.valid&&r[kind]?r.origin+r[kind]:'';}
 function validatePaths(paths){
  for(const [k,v] of Object.entries(paths||{})){
   if(!v&&['native','compact'].includes(k))continue;
   if(typeof v!=='string'||!v.startsWith('/')||v.startsWith('//')||/[?#\\\s]/.test(v)||v.includes('://')||v.split('/').some(s=>s==='.'||s==='..'))return '请填写以 / 开头的接口路径，不含域名、空白、查询参数或相对跳转。';
  }return '';
 }
 function capabilityRequests(p,n,id,protocol,modelId=''){
  const r=routes(p,n),base=endpoint(p,n,'modelBase'),native=endpoint(p,n,'native');
  const join=(base,path)=>base?base.replace(/\/+$/,'')+path:'';
  const suffix={Responses:'/responses',Chat:'/chat/completions',Anthropic:'/messages',Ollama:'/api/chat'}[protocol];
  const wireSuffix=protocol==='Ollama'&&base.endsWith('/api')?'/chat':suffix;
  const call=protocol==='Google'&&modelId?join(base,'/models/'+encodeURIComponent(modelId)+':streamGenerateContent'):wireSuffix?join(base,wireSuffix):'';
  if(['call','fast','ultra'].includes(id))return {urls:call?[call]:[],source:n?.paths?.modelBase?'节点自定义模型基址':'节点模型基址',note:id==='fast'?'与模型调用共用地址，通过请求参数选择 Fast。':id==='ultra'?'与模型调用共用地址，通过请求参数选择 Ultrafast。':protocol?'模型协议：'+protocol:'模型协议待配置，先采用模版或设置单模型协议。'};
  if(id==='ws'){const url=protocol==='Responses'?call:join(native,'/responses');return {urls:url?[url.replace(/^http/,'ws')]:[],source:'复用响应接口',note:'WebSocket 握手与完整响应分别验证，不单设一条请求路径。'};}
  if(id==='compact')return {urls:n?.paths?.compact?[r.origin+n.paths.compact]:protocol==='Responses'&&base?[join(base,'/responses/compact')]:[],source:n?.paths?.compact?'此节点的压缩接口覆盖':'从模型请求基址派生',note:'修改地址不改变协议兼容或站点支持情况。'};
  if(id==='background')return {urls:[],source:'引擎调度',note:'复用相应压缩机制，无独立后台接口；仍需核对引擎与协议条件。'};
  return {urls:native?['/alpha/notes/v2/thread_hint','/alpha/history/v2/list_windows','/alpha/notes/v2/list_files_by_prefix'].map(path=>join(native,path)):[],source:n?.paths&&Object.hasOwn(n.paths,'native')?'此节点的原生服务前缀':'站点适配预设',note:'提示、历史列表和笔记列表分别检查；列表成功不代表读写流程已通过。'};
 }
 function servicePaths(n,mode,values){
  const next={...(n?.paths||{})};delete next.native;delete next.compact;
  if(mode==='custom'){next.native=(values.native||'').trim().replace(/\/+$/,'')||((values.native||'').trim()==='/'?'/':'');const compact=(values.compact||'').trim().replace(/\/+$/,'');if(compact)next.compact=compact;}
  return Object.keys(next).length?next:null;
 }
 function fingerprint(p,nodeId){const n=p?.nodes?.find(n=>n.id===(nodeId||p.activeNodeId));return JSON.stringify([n?.id,n?.url,n?.paths||null,p?.connection?.auth,!!p?.connection?.keyConfigured,p?.connection?.credentialRef,access(p)]);}
 function createStore(){
  const records=new Map(),runs=new Map();let serial=0,scene='normal',bound=null,applied=null;
  const key=(p,nid)=>p.id+'::'+(nid||p.activeNodeId);
  function observation(p,nid){return records.get(key(p,nid))||null;}
  function status(p,nid){const r=observation(p,nid),valid=!!r&&r.fingerprint===fingerprint(p,nid);return {record:r,valid,stale:!!r&&!valid,running:runs.get(key(p,nid))?.status==='running'};}
  function auth(p,nid){
   const s=status(p,nid);if(s.running&&s.record?.auth==='checking')return {id:'checking',label:'认证检查中',tone:'info'};
   if(s.stale)return {id:'stale',label:'连接已变化 · 需复验',tone:'warning'};
   const id=s.valid?s.record.auth:'unknown';
   return {id,label:{passed:access(p).mode==='pat'?'原生身份已通过':'凭据检查已通过',failed:'认证失败',network:'本次连接失败',unknown:'尚未认证',checking:'认证检查中'}[id]||'尚未认证',tone:id==='passed'?'success':['failed'].includes(id)?'danger':id==='network'?'warning':'neutral'};
  }
  function row(p,modelId,id,nid){
   const a=access(p),s=status(p,nid),r=s.valid?s.record:null,krill=a.site==='krill';
   let support='unknown',basis='未取得适用的能力依据',detail='选择接入方式后检测';
   const explicitModel=['gpt-6.1-sol','gpt-6-astra','gpt-6.1-terra'].includes(modelId);
   if(krill){
    if(id==='call'){support='supported';basis='该站点接入说明';}
    if(id==='fast'&&explicitModel){support='supported';basis='所选型号目录声明';}
    if(id==='ultra'&&explicitModel){support=modelId==='gpt-6.1-terra'?'unsupported':'supported';basis=modelId==='gpt-6.1-terra'?'所选型号明确不支持':'官网说明与型号目录';}
    if(id==='ws'){support='supported';basis='站点接入规则 · 当前节点需另验';}
    if(id==='notes'){support='unknown';basis='旧版客户端机制存在';}
   }
   const observed=modelCapabilities.has(id)?r?.models?.[modelId]?.[id]:r?.features?.[id],applicable=!!observed;
   if(applicable&&observed.support){support=observed.support;basis=observed.basis||basis;}
   let condition='待检测',tone='neutral';
   if(s.stale){condition='需重新检测';tone='warning';detail='此结果属于旧连接，不能用于当前地址或认证。';}
   else if(s.running&&r?.checking?.includes(id)){condition='验证中…';tone='info';detail='正在模拟所选功能验证';}
   else if(!r||r.auth==='unknown'){condition='待认证';detail='先检查当前节点的认证与目录。';}
   else if(r.auth==='failed'){condition='认证未通过';tone='danger';detail='凭据检查未通过（401），原配置保留。';}
   else if(r.auth==='network'){condition='本次检测失败';tone='warning';detail='连接超时，已有支持依据保留；可重试。';}
   else if(modelCapabilities.has(id)&&!modelId){condition='先获取模型列表';detail='接入认证与链路项目可以先检查；型号相关项目需从当前渠道模型列表选择。';}
   else if(r.auth==='passed'){
    const v=applicable?observed:null;condition=support==='unsupported'?'不可使用':v?.condition||(support==='supported'&&['fast','ultra'].includes(id)?'可选择':'待验证');tone=support==='unsupported'?'neutral':v?.tone||'neutral';detail=v?.detail||'认证通过不代表本项已经验证。';
    if(v?.support){support=v.support;basis=v.basis||basis;}
   }
   if(support==='unsupported'){condition='当前型号不可使用';detail='可更换为支持该档位的型号，其他能力不受影响。';}
   return {id,name:capabilities.find(x=>x[0]===id)?.[1]||id,support,supportLabel:supportWords[support],basis,condition,tone,detail};
  }
  function outcome(p,modelId){
   const pat=access(p).mode==='pat',ready=scene==='ready',krill=access(p).site==='krill';
   return Object.fromEntries(capabilities.map(([id])=>[id,
    id==='call'?{condition:'可使用',tone:'success',detail:'模拟生成响应完成；型号与目标一致。',support:'supported',basis:'本轮有效响应'}:
    id==='fast'?{condition:'可选择',tone:'success',detail:'所选型号支持 Fast；本次接受 priority 参数。',...(krill?{}:{condition:'待核对',tone:'neutral',detail:'未取得此站点的型号档位声明。'})}:
    id==='ultra'?{condition:modelId==='gpt-6.1-terra'?'当前型号不可使用':krill?'可选择':'待核对',tone:krill?'success':'neutral',detail:krill?'Sol／Astra 支持 Ultrafast。default 回显另属效果核实，不否定支持。':'需按此站点与型号核对，不能套用 Krill 结论。'}:
    id==='ws'?{condition:ready?'可使用':krill?'本机入口待开启':'待核对',tone:ready?'success':'warning',detail:ready?'上游与本机入口均已模拟接通，实际采用仍待观察。':'Krill 上游支持 WSS；当前 OpenCodex 入口未启用，第三方出口仍走 HTTP/SSE。'}:
    id==='compact'?{condition:ready?'可使用':'当前为代理摘要',tone:ready?'success':'warning',detail:ready?'模拟原生状态回传与续聊通过。':'压缩及续聊可完成，但代理摘要不能标作原生远程压缩。',...(ready?{support:'supported',basis:'模拟原生协议与续聊'}:{})}:
    id==='background'?{condition:ready?'可使用':'引擎条件待接通',tone:ready?'success':'warning',detail:ready?'模拟远程 v2 与后台调度前提满足。':'需要相应引擎、远程 v2，且历史与笔记模式不同时开启；真实长上下文未验证。',...(ready?{support:'supported',basis:'模拟引擎与调度检查'}:{})}:
    {condition:!pat?'需原生身份接入':ready?'可使用':'当前服务不可用',tone:ready&&pat?'success':'warning',detail:!pat?'普通 API Key 凭据通过不等于原生 PAT 身份成立。':ready?'仅供评审的“服务恢复且完整接通”样例，非当前 Krill 结果。':'原生 thread_hint、历史列表及笔记列表均 404。客户端工具存在，完整服务未接通。',...(ready&&pat?{support:'supported',basis:'模拟完整历史／笔记服务'}:{})}
   ]));
  }
  function begin(p,nid,kind,modelId,ids){
   const k=key(p,nid),old=records.get(k),run={token:++serial,key:k,providerId:p.id,nodeId:nid,fingerprint:fingerprint(p,nid),modelId,kind,ids:ids||capabilities.map(x=>x[0]),status:'running'};
   run.previous=old?.fingerprint===run.fingerprint?clone(old):null;runs.set(k,run);records.set(k,{...(old?.fingerprint===run.fingerprint?old:{}),fingerprint:run.fingerprint,auth:kind==='connection'?'checking':old?.auth||'unknown',checking:kind==='features'?run.ids:[],modelId});return clone(run);
  }
  function finish(p,run){
   const current=runs.get(run.key);if(!current||current.token!==run.token||current.status!=='running')return false;
   current.status='done';if(!p||fingerprint(p,run.nodeId)!==run.fingerprint){current.status='stale';return false;}
   const old=records.get(run.key),authId=scene==='auth'?'failed':scene==='network'?'network':'passed';
   const next={...old,fingerprint:run.fingerprint,checking:[],finishedAt:new Date().toLocaleTimeString('zh-CN',{hour12:false}),modelId:run.modelId};
   if(run.kind==='connection'){next.auth=authId;next.directory=authId==='passed'?'身份与凭据检查通过':'身份与凭据尚未通过';next.features=old?.features||{};}
   else if(old?.auth==='passed'){
    if(scene==='network'){next.lastError='网络超时 · 已有支持与上次结果保留';}
    else {const all=outcome(p,run.modelId);next.features={...(old?.features||{})};next.models=clone(old?.models||{});for(const id of run.ids){if(modelCapabilities.has(id)){if(!run.modelId)continue;next.models[run.modelId]||={};next.models[run.modelId][id]={...all[id],modelId:run.modelId};}else next.features[id]={...all[id]};}delete next.lastError;}
   }
   records.set(run.key,next);return true;
  }
  function stop(p,nid){const r=runs.get(key(p,nid));if(r?.status==='running'){r.status='stopped';if(r.previous)records.set(r.key,clone(r.previous));else{const v=records.get(r.key);if(v){v.checking=[];if(v.auth==='checking')v.auth='unknown';}}return true;}return false;}
  function binding(providers,savedProviders){
   if(!bound)return {label:'当前 Codex 实例尚未绑定接入',tone:'neutral',active:false};
   const p=providers.find(p=>p.id===bound.pid),saved=savedProviders?.find(p=>p.id===bound.pid);
   if(p?.enabled===false)return {label:p.name+' · 渠道已停用',tone:'warning',active:false};
   if(!p||fingerprint(p,bound.nodeId)!==bound.fingerprint)return {label:'已绑定连接发生变化 · 需复验',tone:'warning',active:false};
   if(!saved||fingerprint(saved,bound.nodeId)!==bound.fingerprint)return {label:p.name+' · 接入修改尚未保存',tone:'warning',active:false};
   if(auth(p,bound.nodeId).id!=='passed')return {label:p.name+' · 当前认证需要重新检查',tone:'warning',active:false};
   return {label:p.name+' · '+(p.nodes.find(n=>n.id===bound.nodeId)?.name||'节点')+(applied?.fingerprint===bound.fingerprint?' · 已模拟连接，采用待观察':' · 已绑定，待重新连接 Codex'),tone:applied?.fingerprint===bound.fingerprint?'success':'warning',active:applied?.fingerprint===bound.fingerprint,pid:p.id};
  }
  return {status,auth,row,begin,finish,stop,binding,setScene:v=>{if(sceneWords[v])scene=v;},scene:()=>scene,
   bind(p){if(auth(p).id!=='passed'||p.enabled===false)return false;bound={pid:p.id,nodeId:p.activeNodeId,fingerprint:fingerprint(p)};applied=null;return true;},
   reconnect(ps,saved){const b=binding(ps,saved);if(!b.pid)return false;applied=clone(bound);return true;},
   seed(p){if(access(p).site!=='krill'||scene==='unverified')return;const run=this.begin(p,p.activeNodeId,'connection',p.models[0]?.id);this.finish(p,run);if(scene!=='auth'&&scene!=='network'){const f=this.begin(p,p.activeNodeId,'features',p.models[0]?.id);this.finish(p,f);}},
   reset(){records.clear();runs.clear();bound=applied=null;serial++;},snapshot:()=>clone({records:[...records],runs:[...runs],bound,applied,scene})};
 }
 const store=createStore();let bridge=null,view={nodeId:null,modelId:null,addressDraft:null,ids:new Set(['call','ws'])};
 function enabled(){if(typeof document==='undefined')return false;const scope=document.documentElement.dataset.designScope;return scope?scope==='conversations':['conversations','capabilities'].includes(new URLSearchParams(location.search).get('design'));}
 const allowed=p=>!p?.prototypeCapabilityFixture||enabled();
 function heading(p){if(!enabled()||p.id==='native'||p.availability?.kind&&!['unknown','available'].includes(p.availability.kind))return null;return store.auth(p);}
 const btn=(label,action,attrs='',primary=false)=>'<button type="button" class="btn '+(primary?'primary':'ghost')+' btn-sm" data-cc="'+action+'" '+attrs+'>'+label+'</button>';
 const option=(v,t,s)=>'<option value="'+esc(v)+'" '+(v===s?'selected':'')+'>'+esc(t)+'</option>';
 function seed(policy,makeProvider){
  if(!enabled()||policy.providers.some(p=>p.id==='krill-demo'))return;
  const p=makeProvider('krill-demo','Krill',3,3);p.prototypeCapabilityFixture=true;p.connection.specialAccess={site:'krill',mode:'pat'};
  ['gpt-6.1-sol','gpt-6-astra','gpt-6.1-terra'].forEach((id,i)=>p.models[i].id=id);
  p.nodes=[{id:'primary',name:'海外线路',url:'https://api.kbsfdxc.net/codex/v1'},{id:'backup',name:'备用线路',url:'https://backup.krill.example/codex/v1'}];p.activeNodeId='primary';policy.providers.push(p);
 }
 // 检测从唯一渠道清单取值，不按启用开关另造一份目录。
 function models(p){return p.models||[];}
 function selectedModel(p){return models(p).find(m=>m.id===view.modelId)?.id||models(p)[0]?.id||'';}
 function authHTML(p,nid){const a=store.auth(p,nid),s=store.status(p,nid);return '<div class="cc-auth '+a.tone+'" role="status"><span class="cc-status-icon">'+(a.id==='passed'?'✓':a.id==='failed'?'!':'◦')+'</span><div><strong>'+esc(a.label)+'</strong><small>'+esc(s.valid?s.record.directory||'当前节点身份':s.stale?'旧结果保留，当前连接需要复验':'接入身份与特殊能力分别检查')+(s.record?.finishedAt?' · '+esc(s.record.finishedAt):'')+'</small></div></div>';}
 function summaryHTML(p){if(!enabled()||p.id==='native')return '';return btn('特殊能力','open-features','data-channel="'+esc(p.id)+'" aria-haspopup="dialog"');}
 function credentialTypeHTML(p){if(!enabled()||p.id==='native')return '';const selected=p.connection.auth==='none'?'none':access(p).mode==='pat'?'pat':'key';return '<label>凭据类型<select class="mp-select" data-cc-credential data-channel="'+esc(p.id)+'" aria-label="凭据类型" '+(bridge?.busy()?'disabled':'')+'>'+option('key','API Key',selected)+(access(p).site==='krill'?option('pat','原生身份（PAT）',selected):'<option value="pat" disabled>原生身份（需站点适配）</option>')+option('none','无需认证',selected)+'</select></label>';}
 function connectionHTML(p){
  if(!enabled()||p.id==='native')return '';
  return '<div class="cc-credential-status" data-cc-auth-host="'+esc(p.id)+'">'+authHTML(p)+'</div><div class="cc-instance"><div><span>当前 Codex 实例</span><strong data-cc-binding>'+esc(store.binding(bridge.providers(),bridge.savedProviders()).label)+'</strong></div>'+btn('用于当前 Codex','bind','data-channel="'+esc(p.id)+'" '+(store.auth(p).id!=='passed'||p.enabled===false||bridge?.busy()?'disabled':''))+'</div>';
 }
 function nodeSettingsHTML(p){if(!enabled()||p.id==='native')return '';return '<div class="cc-node-settings"><label>站点适配<select class="mp-select" data-cc-setting="site" data-channel="'+esc(p.id)+'" aria-label="站点适配" '+(bridge?.busy()?'disabled':'')+'>'+option('generic','通用兼容',access(p).site)+option('krill','Krill',access(p).site)+'</select></label><p class="cc-muted">提供默认接口路径；每个节点可展开修改并查看最终地址。</p></div>';}
 function nodeHTML(p,n){if(!enabled()||p.id==='native')return '';const r=routes(p,n);return '<small class="cc-node-note">模型 '+esc(r.modelBase||'待配置')+' · '+(n.paths?'自定义路径':'默认路径')+'</small><small class="cc-node-note" data-cc-node-auth="'+esc(p.id)+'::'+esc(n.id)+'">'+esc(store.auth(p,n.id).label)+'</small>';}
 function nodeActionHTML(p,n){if(!enabled()||p.id==='native')return '';return btn('特殊能力','open-features','data-channel="'+esc(p.id)+'" data-node="'+esc(n.id)+'" aria-haspopup="dialog" aria-label="'+esc(n.name+' · 特殊能力')+'" '+(bridge?.busy()?'disabled':''));}
 function pathPreviewHTML(p,n){const r=routes(p,n);return '<dl class="cc-path-preview"><dt>模型请求基址</dt><dd>'+esc(endpoint(p,n,'modelBase')||'待配置')+'</dd><dt>模型清单</dt><dd>'+esc(access(p).site==='generic'&&!n?.paths?'按本次清单获取方式解析':endpoint(p,n,'models')||'待配置')+'</dd><dt>原生服务前缀</dt><dd>'+esc(endpoint(p,n,'native')||'未配置')+'</dd></dl>';}
 function pathEditorHTML(p,n){
  if(!enabled())return '';const r=routes(p,n),preserved={...(n?.paths||{})};delete preserved.modelBase;delete preserved.models;const custom=!!(n?.paths&&(Object.hasOwn(n.paths,'modelBase')||Object.hasOwn(n.paths,'models')));
  return '<details class="cc-path-editor" data-cc-preserved-paths="'+esc(JSON.stringify(preserved))+'" '+(custom?'open':'')+'><summary>模型接口路径 <span class="cc-muted">'+(custom?'自定义':'跟随站点适配')+'</span></summary><label class="cc-path-mode">路径设置<select class="mp-select" data-cc-path-mode aria-label="路径设置">'+option('default','跟随站点适配',custom?'custom':'default')+option('custom','自定义',custom?'custom':'default')+'</select></label><div class="cc-path-fields" '+(custom?'':'hidden')+'>'+[['modelBase','模型请求基址路径'],['models','模型清单路径']].map(([id,name])=>'<label>'+name+'<input class="input" data-cc-path="'+id+'" aria-label="'+name+'" value="'+esc(r[id]||'')+'" placeholder="/v1"></label>').join('')+'</div><div data-cc-path-preview>'+pathPreviewHTML(p,n)+'</div></details>'+(n?.id?'<div class="cc-node-special-link">'+btn('加入草稿并配置特殊服务地址','configure-service-paths','data-channel="'+esc(p.id)+'" data-node="'+esc(n.id)+'"')+'</div>':'<p class="cc-muted">新增节点后，可在该行“特殊能力”中配置特殊服务地址。</p>');
 }
 function readPaths(root){if(!enabled())return null;const editor=root.querySelector('.cc-path-editor');if(!editor)return null;const preserved=JSON.parse(editor.dataset.ccPreservedPaths||'{}');if(root.querySelector('[data-cc-path-mode]')?.value==='custom')for(const el of root.querySelectorAll('[data-cc-path]'))preserved[el.dataset.ccPath]=el.value.trim().replace(/\/+$/,'')||(el.value.trim()==='/'?'/':'');return Object.keys(preserved).length?preserved:null;}
 function updatePathPreview(){const root=document.querySelector('#modalBody'),editor=root?.querySelector('.cc-path-editor'),p=bridge?.provider(bridge.current());if(!editor||!p)return;const custom=root.querySelector('[data-cc-path-mode]').value==='custom';editor.querySelector('summary span').textContent=custom?'自定义':'跟随站点适配';root.querySelector('.cc-path-fields').hidden=!custom;const n={url:root.querySelector('#mpServiceNodeURL')?.value,paths:readPaths(root)};root.querySelector('[data-cc-path-preview]').innerHTML=pathPreviewHTML(p,n);}
 function modelProtocol(p,mid){return bridge?.modelProtocol?.(p.id,mid)||null;}
 function requestDetailsHTML(p,n,mid,id){const request=capabilityRequests(p,n,id,modelProtocol(p,mid),mid);return '<div class="cc-request-detail"><small>地址来源：'+esc(request.source)+'</small>'+request.urls.map(url=>'<code>'+esc(url)+'</code>').join('')+'<small>'+esc(request.note)+'</small>'+(id==='compact'&&enabled()?btn('配置压缩请求地址','focus-request-settings','data-part="compact" data-channel="'+esc(p.id)+'" data-node="'+esc(n?.id)+'"'):'')+(id==='notes'&&enabled()?btn('配置原生服务前缀','focus-request-settings','data-part="native" data-channel="'+esc(p.id)+'" data-node="'+esc(n?.id)+'"'):'')+'</div>';}
 function modelSelectorHTML(p,mid){return '<label class="cc-cap-model">适用型号<select class="mp-select" data-cc-target="model" aria-label="特殊能力适用型号" '+(!mid?'disabled':'')+'>'+(!mid?option('','先获取模型列表',''):'')+models(p).map(m=>option(m.id,m.id,mid)).join('')+'</select></label>';}
 function tableHTML(p,nid,mid,interactive=false){
  const n=p.nodes.find(n=>n.id===nid);
  return [['model','型号能力',capabilities.filter(([id])=>modelCapabilities.has(id))],['connection','节点与接入',capabilities.filter(([id])=>!modelCapabilities.has(id))]].map(([scope,title,items])=>'<section class="cc-cap-group" aria-label="'+title+'"><div class="cc-cap-group-head"><h4>'+title+(scope==='model'&&!mid?' · 先获取模型列表':'')+'</h4>'+(interactive&&scope==='model'?modelSelectorHTML(p,mid):'')+'</div><div class="cc-table" role="table" aria-label="'+title+'支持与当前条件"><div class="cc-table-head" role="row"><span>功能</span><span>支持情况</span><span>当前使用条件</span></div>'+items.map(([id,name])=>{
   const r=store.row(p,mid,id,nid),missing=modelCapabilities.has(id)&&!mid,missingAddress=id!=='background'&&!capabilityRequests(p,n,id,modelProtocol(p,mid),mid).urls.length;if(missingAddress&&!missing&&r.support!=='unsupported'){r.condition=modelCapabilities.has(id)&&!modelProtocol(p,mid)?'模型协议待配置':'请求地址待配置';r.detail='先配置此节点的请求地址和适用模型协议，再验证对应能力。';r.tone='warning';}return '<div class="cc-cap-row" role="row" data-capability="'+id+'"><div class="cc-cap-name">'+(interactive?'<input type="checkbox" data-cc-check="'+id+'" aria-label="验证'+name+'" '+(view.ids.has(id)?'checked':'')+' '+(store.status(p,nid).running||missing||missingAddress?'disabled':'')+'>':'')+'<span>'+name+'</span></div><span class="cc-support" data-support="'+r.support+'">'+(r.support==='supported'?'✓ ':r.support==='unsupported'?'— ':'◦ ')+r.supportLabel+'</span><details><summary class="cc-condition '+r.tone+'">'+esc(r.condition)+'</summary><p>'+esc(r.detail)+'</p><small>依据：'+esc(r.basis)+' · '+esc(modelCapabilities.has(id)?mid||'未选择型号':'此节点与接入身份')+'</small>'+requestDetailsHTML(p,n,mid,id)+'</details></div>';
  }).join('')+'</div></section>').join('');
 }
 function addressDraft(p){const n=p.nodes.find(n=>n.id===view.nodeId);if(!view.addressDraft)view.addressDraft={baseline:fingerprint(p,view.nodeId),mode:n?.paths&&(Object.hasOwn(n.paths,'native')||Object.hasOwn(n.paths,'compact'))?'custom':'default',native:routes(p,n).native,compact:n?.paths?.compact||'',error:''};return view.addressDraft;}
 function samePaths(a,b){const normalized=v=>Object.fromEntries(Object.keys(v||{}).sort().map(k=>[k,v[k]]));return JSON.stringify(normalized(a))===JSON.stringify(normalized(b));}
 function pendingAddress(p){const n=p.nodes.find(n=>n.id===view.nodeId),d=addressDraft(p);return !samePaths(servicePaths(n,d.mode,d),n?.paths);}
 function requestPreviewHTML(p,n){const compact=capabilityRequests(p,n,'compact',modelProtocol(p,selectedModel(p)),selectedModel(p));return '<dl class="cc-path-preview"><dt>模型请求基址</dt><dd>'+esc(endpoint(p,n,'modelBase')||'待配置')+'</dd><dt>原生服务前缀</dt><dd>'+esc(endpoint(p,n,'native')||'未配置')+'</dd><dt>压缩请求地址</dt><dd>'+esc(compact.urls[0]||'需先配置对应模型协议')+'</dd></dl>';}
 function requestSettingsHTML(p){const d=addressDraft(p),n=p.nodes.find(n=>n.id===view.nodeId);return '<details id="ccRequestSettings" class="cc-request-settings"><summary>请求地址 <span data-cc-request-mode>'+ (d.mode==='custom'?'节点自定义':'跟随站点适配')+'</span></summary><div class="cc-request-settings-body"><p class="cc-muted">与此节点共用一份路径配置；模型基址在节点编辑中维护。</p><label class="cc-path-mode">特殊服务路径<select class="mp-select" data-cc-service-mode aria-label="特殊服务路径设置">'+option('default','跟随站点适配',d.mode)+option('custom','自定义',d.mode)+'</select></label><div data-cc-service-fields '+(d.mode==='custom'?'':'hidden')+'><label>原生服务前缀<input class="input" data-cc-service-path="native" aria-label="原生服务前缀" value="'+esc(d.native)+'" placeholder="/backend-api/codex"></label><details class="cc-service-advanced" '+(d.compact?'open':'')+'><summary>高级覆盖 · 压缩接口</summary><label>压缩接口路径<input class="input" data-cc-service-path="compact" aria-label="压缩接口路径" value="'+esc(d.compact)+'" placeholder="留空，按模型基址派生"></label><p class="cc-muted">仅在站点需要不同压缩路径时设置；不会自动改变能力支持。</p></details></div><div data-cc-service-preview>'+requestPreviewHTML(p,n)+'</div><p id="ccRequestError" class="cc-request-error" role="alert"></p><div class="cc-request-actions">'+btn('取消修改','cancel-request-paths')+btn('加入草稿','apply-request-paths','',true)+'</div></div></details>';}
 function syncAddressEditor(p){const root=document.querySelector('#ccRequestSettings');if(!root)return;const d=addressDraft(p),n=p.nodes.find(n=>n.id===view.nodeId),candidate={...n,paths:servicePaths(n,d.mode,d)},running=store.status(p,view.nodeId).running;root.querySelector('[data-cc-service-fields]').hidden=d.mode!=='custom';root.querySelector('[data-cc-request-mode]').textContent=d.mode==='custom'?'节点自定义':'跟随站点适配';root.querySelector('[data-cc-service-preview]').innerHTML=requestPreviewHTML(p,candidate);root.querySelector('#ccRequestError').textContent=d.error||'';root.querySelectorAll('input,select').forEach(el=>el.disabled=running);root.querySelector('[data-cc="apply-request-paths"]').disabled=running||!pendingAddress(p);root.querySelector('[data-cc="cancel-request-paths"]').disabled=running||!pendingAddress(p);}
 function resetAddressEditor(p){view.addressDraft=null;const root=document.querySelector('#ccRequestSettings');if(root){const expanded=root.open;root.outerHTML=requestSettingsHTML(p);document.querySelector('#ccRequestSettings').open=expanded;}refresh();}
 function applyAddress(p){const d=addressDraft(p),n=p.nodes.find(n=>n.id===view.nodeId);if(!n||bridge.busy()||store.status(p,view.nodeId).running)return;if(fingerprint(p,view.nodeId)!==d.baseline){d.error='节点配置已变化，请取消修改后重新核对。';syncAddressEditor(p);return;}const next=servicePaths(n,d.mode,d),error=validatePaths(next||{});if(error){d.error=error;syncAddressEditor(p);return;}view.addressDraft=null;bridge.edit(p.id,target=>{const node=target.nodes.find(n=>n.id===view.nodeId);if(next)node.paths=next;else delete node.paths;});resetAddressEditor(p);bridge.notice('请求地址已加入节点草稿；保存渠道后生效，相关结果需复验');}
 function modelHTML(p,m){if(!enabled()||p.id==='native')return '';const supported=[['fast','Fast'],['ultra','Ultrafast'],['compact','原生压缩']].map(([id,name])=>({id,name,result:store.row(p,m.id,id)})).filter(item=>item.result.support==='supported');return supported.length?'<span class="cc-model-capabilities" aria-label="已确认支持的能力">'+supported.map(({id,name,result})=>'<span class="cc-model-capability" data-supported-capability="'+id+'" title="'+esc(name+' · '+result.basis+' · '+result.condition)+'">'+name+'</span>').join('')+'</span>':'';}
 function dialogHTML(p,original){return original;}
 function selectedIds(p){const mid=selectedModel(p),n=p.nodes.find(n=>n.id===view.nodeId);return [...view.ids].filter(id=>(!modelCapabilities.has(id)||!!mid)&&(id==='background'||capabilityRequests(p,n,id,modelProtocol(p,mid),mid).urls.length>0));}
 function featureDialogHTML(p){const mid=selectedModel(p),n=p.nodes.find(n=>n.id===view.nodeId);return '<div class="cc-detection '+(document.querySelector('.window')?.offsetHeight<680?'cc-compact':'')+'" id="ccDetection" data-channel="'+esc(p.id)+'" data-node="'+esc(view.nodeId)+'"><div class="cc-node-context" aria-label="当前查看节点"><strong>'+esc(n?.name||'节点已移除')+'</strong><code>'+esc(n?.url||'未配置地址')+'</code><small>'+esc(access(p).mode==='pat'?'原生身份（PAT）':p.connection.auth==='none'?'无需认证':'API Key')+'</small></div><div class="cc-identity-row"><div data-cc-dialog-auth>'+authHTML(p,view.nodeId)+'</div>'+btn('认证接入','check-auth','data-channel="'+esc(p.id)+'"')+'</div>'+requestSettingsHTML(p)+(!mid?'<p class="cc-empty-models">尚无模型，可先认证接入或验证链路。'+btn('前往模型列表','go-models','data-channel="'+esc(p.id)+'"')+'</p>':'')+'<div id="ccFeatureResults">'+tableHTML(p,view.nodeId,mid,true)+'</div><div class="cc-run-status" id="ccRunStatus" role="status">'+runStatus(p)+'</div><div class="cc-detect-footer"><p class="cc-muted">验证可能产生模型用量。历史与笔记不写入内容。</p>'+btn('验证所选能力','check-features','data-channel="'+esc(p.id)+'"',true)+btn('停止','stop','data-channel="'+esc(p.id)+'"')+'</div></div>';}
 function runStatus(p){const s=store.status(p,view.nodeId);return pendingAddress(p)?'请求地址修改尚未加入草稿，请先加入或取消修改':s.running?'验证中，结果仅对应此节点、路径与凭据':s.stale?'连接已变化，旧结果需要复验':s.record?.lastError||(!s.valid?'请先认证此节点的接入身份':s.record.finishedAt?'上次检查 '+s.record.finishedAt+' · 不自动保存或切换节点':'已选择 '+selectedIds(p).length+' 项能力');}
 function refresh(){
  if(!bridge||typeof document==='undefined')return;if(!enabled()){policyBanner();return;}
  const replaceTable=(el,html)=>{const opened=[...el.querySelectorAll('.cc-cap-row details[open]')].map(d=>d.closest('[data-capability]').dataset.capability);el.innerHTML=html;for(const id of opened){const details=el.querySelector('[data-capability="'+id+'"] details');if(details)details.open=true;}};
  for(const p of bridge.providers()){
   document.querySelectorAll('[data-cc-auth-host="'+p.id+'"]').forEach(el=>el.innerHTML=authHTML(p));
   for(const n of p.nodes||[])document.querySelectorAll('[data-cc-node-auth="'+p.id+'::'+n.id+'"]').forEach(el=>el.textContent=store.auth(p,n.id).label);
   const state=heading(p),badge=p.id===bridge.current()&&document.querySelector('#mpChannelHeadingStatus [data-channel-availability]');if(state&&badge){badge.textContent=state.label;badge.className='mp-template-state '+state.tone;badge.dataset.channelAvailability=state.id;}
   document.querySelectorAll('[data-cc="bind"][data-channel="'+p.id+'"]').forEach(b=>b.disabled=bridge.busy()||p.enabled===false||store.auth(p).id!=='passed');
  }
  document.querySelectorAll('[data-cc-binding]').forEach(el=>el.textContent=store.binding(bridge.providers(),bridge.savedProviders()).label);
  const dialog=document.querySelector('#ccDetection');if(dialog&&document.querySelector('#modalMask')?.style.display==='flex'){
   const p=bridge.provider(dialog.dataset.channel);if(p){dialog.querySelector('[data-cc-dialog-auth]').innerHTML=authHTML(p,view.nodeId);replaceTable(dialog.querySelector('#ccFeatureResults'),tableHTML(p,view.nodeId,selectedModel(p),true));dialog.querySelector('#ccRunStatus').textContent=runStatus(p);const running=store.status(p,view.nodeId).running,dirty=pendingAddress(p),authAction=dialog.querySelector('[data-cc="check-auth"]');authAction.disabled=running||bridge.busy()||dirty;authAction.textContent='认证接入';dialog.querySelector('[data-cc="check-features"]').disabled=running||dirty||!selectedIds(p).length||store.auth(p,view.nodeId).id!=='passed';dialog.querySelector('[data-cc="stop"]').disabled=!running;dialog.querySelectorAll('[data-cc-target]').forEach(x=>{x.disabled=running||(x.dataset.ccTarget==='model'&&!selectedModel(p));x.value=x.dataset.ccTarget==='node'?view.nodeId:selectedModel(p);});syncAddressEditor(p);}
  }
  policyBanner();const controls=document.querySelector('#ccDetection');if(controls)window.prototypeSelect?.enhance(controls);
 }
 function run(p,kind){if(pendingAddress(p))return;const nid=view.nodeId||p.activeNodeId,n=p.nodes.find(n=>n.id===nid);if(!n?.url||(p.connection.auth!=='none'&&!p.connection?.keyConfigured)){bridge.notice('请先配置服务节点与凭据');return;}if(kind==='features'&&(store.auth(p,nid).id!=='passed'||!selectedIds(p).length))return;if(store.status(p,nid).running)return;const r=store.begin(p,nid,kind,selectedModel(p),selectedIds(p));refresh();setTimeout(()=>{store.finish(bridge.provider(p.id),r);refresh();window.dispatchEvent(new Event('prototype:channel-capabilities'));},650);}
 function open(pid,part,anchor){if(!enabled()||bridge?.busy())return;const p=bridge.provider(pid);if(!p)return;if(part==='connection'){bridge.open(pid,anchor);return;}view.nodeId=p.nodes.some(n=>n.id===anchor?.dataset.node)?anchor.dataset.node:p.activeNodeId;view.modelId=selectedModel(p);view.addressDraft=null;const node=p.nodes.find(n=>n.id===view.nodeId);bridge.featureModal(p.name+' · '+(node?.name||'节点')+' · 特殊能力',featureDialogHTML(p),()=>{view.addressDraft=null;const source=anchor?.isConnected?anchor:document.querySelector('[data-cc="open-features"][data-channel="'+p.id+'"][data-node="'+view.nodeId+'"]');source?.focus({preventScroll:true});});refresh();}
 function policyBanner(){const page=document.querySelector('#settings-dialog-policy');if(!page)return;let el=page.querySelector('#ccPolicyBinding');if(!enabled()){el?.remove();return;}if(!el){el=document.createElement('div');el.id='ccPolicyBinding';el.className='cc-policy-binding';page.prepend(el);}const b=store.binding(bridge.providers(),bridge.savedProviders()),p=b.pid&&bridge.provider(b.pid),notes=p&&store.row(p,selectedModel(p),'notes');el.innerHTML='<span>实例接入</span><strong>'+esc(b.label)+'</strong><small>'+esc(notes?'历史与笔记：'+notes.condition:'依赖功能需绑定实际模型连接，不能借用其他渠道认证')+'</small>'+btn('查看渠道接入','manage-binding','data-channel="'+esc(b.pid||'krill-demo')+'"');}
 function connect(api){
  bridge=api;if(typeof document==='undefined')return;bridge.ensureFixture();for(const p of bridge.providers())if(p.prototypeCapabilityFixture)store.seed(p);
  const card=document.createElement('div');card.id='channelCapabilityDemoCard';card.className='proto-card';card.dataset.protoContexts='models:channels';card.innerHTML='<div class="proto-note-title">渠道接入与能力 · 协作包</div><p class="pctl-hint">内存模拟；不访问真实服务或使用真实 Key。切换情景只改变检测演示。</p><div class="pctl-grid">'+Object.entries(sceneWords).map(([id,name])=>btn(name,'scene','data-scene="'+id+'"')).join('')+'</div><div class="pctl-grid">'+btn('查看 Krill 接入','demo-channel')+btn('模拟重新连接 Codex','reconnect')+'</div>';(document.querySelector('#protoStdCards .proto-group-body')||document.querySelector('#protoStdCards'))?.appendChild(card);
  document.addEventListener('click',e=>{
   const b=e.target.closest?.('[data-cc]');if(!b||b.disabled||!enabled())return;const a=b.dataset.cc,p=bridge.provider(b.dataset.channel||document.querySelector('#ccDetection')?.dataset.channel||bridge.current());
   if(a==='open-connection'||a==='open-features')open(p.id,a==='open-features'?'features':'connection',b);
   else if(a==='apply-request-paths')applyAddress(p);
   else if(a==='cancel-request-paths')resetAddressEditor(p);
   else if(a==='focus-request-settings'){if(document.querySelector('#modalMask')?.style.display!=='flex'||document.querySelector('#ccDetection')?.dataset.channel!==p.id||document.querySelector('#ccDetection')?.dataset.node!==b.dataset.node)open(p.id,'features',b);const details=document.querySelector('#ccRequestSettings');if(!details)return;details.open=true;const advanced=details.querySelector('.cc-service-advanced');if(b.dataset.part==='compact'&&advanced)advanced.open=true;const target=details.querySelector('[data-cc-service-path="'+b.dataset.part+'"]');if(target&&!target.closest('[hidden]'))target.focus({preventScroll:true});details.scrollIntoView({block:'start',behavior:'smooth'});}
   else if(a==='configure-service-paths'){document.querySelector('#modalConfirm').click();if(document.querySelector('#modalMask')?.style.display==='flex')return;open(p.id,'features',b);document.querySelector('#ccRequestSettings').open=true;}
   else if(a==='check-auth')run(p,'connection');else if(a==='check-features')run(p,'features');
   else if(a==='stop'){store.stop(p,view.nodeId);refresh();}
   else if(a==='bind'){if(store.bind(p)){refresh();bridge.notice('接入已绑定到当前实例 · 待重新连接');}}
   else if(a==='reconnect'){const ok=store.reconnect(bridge.providers(),bridge.savedProviders());refresh();bridge.notice(ok?'已模拟重新连接 · 实际采用待观察':'先保存并认证所绑定的连接');}
   else if(a==='manage-binding'||a==='demo-channel')bridge.show(p?.id||'krill-demo');
   else if(a==='go-models')bridge.goModels(p.id);
   else if(a==='scene'){store.reset();store.setScene(b.dataset.scene);const demo=bridge.provider('krill-demo');if(demo)store.seed(demo);bridge.render();refresh();bridge.notice('原型场景：'+sceneWords[b.dataset.scene]+'；不改变真实服务');}
  });
  document.addEventListener('change',e=>{
   if(!enabled())return;const el=e.target;
   if(el.hasAttribute('data-cc-service-mode')){const p=bridge.provider(document.querySelector('#ccDetection').dataset.channel);const d=addressDraft(p);d.mode=el.value;d.error='';refresh();}
   else if(el.dataset.ccSetting){const p=bridge.provider(el.dataset.channel);if(!p||bridge.busy())return;bridge.edit(p.id,target=>{target.connection.specialAccess={...access(target),[el.dataset.ccSetting]:el.value};if(el.dataset.ccSetting==='site'&&el.value==='generic')target.connection.specialAccess.mode='key';});refresh();}
   else if(el.hasAttribute('data-cc-credential')){const p=bridge.provider(el.dataset.channel);if(!p||bridge.busy())return;bridge.edit(p.id,target=>{target.connection.auth=el.value==='none'?'none':'apikey';target.connection.specialAccess={...access(target),mode:el.value==='pat'?'pat':'key'};target.connection.keyConfigured=target.connection.auth==='apikey'&&!!target.connection.credentialRef;});refresh();}
   else if(el.dataset.ccTarget==='model'){view.modelId=el.value;refresh();}
   else if(el.dataset.ccCheck){el.checked?view.ids.add(el.dataset.ccCheck):view.ids.delete(el.dataset.ccCheck);refresh();}
   else if(el.hasAttribute('data-cc-path-mode'))updatePathPreview();
  });
  document.addEventListener('input',e=>{if(enabled()&&e.target.hasAttribute('data-cc-service-path')){const p=bridge.provider(document.querySelector('#ccDetection').dataset.channel),d=addressDraft(p);d[e.target.dataset.ccServicePath]=e.target.value;d.error='';refresh();return;}if(enabled()&&(e.target.hasAttribute('data-cc-path')||e.target.id==='mpServiceNodeURL'))updatePathPreview();});
  new MutationObserver(()=>{if(!enabled())bridge.closeFeatures();card.hidden=!enabled();bridge.ensureFixture();if(enabled()){const p=bridge.provider('krill-demo');if(p&&!store.status(p).record)store.seed(p);}if(!enabled()&&bridge.provider('krill-demo')?.id===bridge.current())bridge.show('newapi',false);else bridge.render();policyBanner();window.prototypePanel?.refresh();}).observe(document.documentElement,{attributes:true,attributeFilter:['data-design-scope']});
  card.hidden=!enabled();policyBanner();if(enabled()&&new URLSearchParams(location.search).get('channel')==='krill-demo')setTimeout(()=>bridge.show('krill-demo'),0);
 }
 return {enabled,allowed,seed,connect,heading,credentialTypeHTML,connectionHTML,nodeSettingsHTML,nodeActionHTML,pathEditorHTML,readPaths,validatePaths,routes,endpoint,capabilityRequests,servicePaths,summaryHTML,modelHTML,nodeHTML,dialogHTML,refresh,open,createStore,fingerprint,state:()=>store.snapshot()};
});
