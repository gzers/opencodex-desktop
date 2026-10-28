/* 对话原型 · 版本未定 · 所有对象、保存、压缩和恢复均在内存模拟。 */
(() => {
 'use strict';
 const $=(s,r=document)=>r.querySelector(s), $$=(s,r=document)=>[...r.querySelectorAll(s)];
 const clone=x=>JSON.parse(JSON.stringify(x));
 const esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 const host=$('#conversationContent'),globalHost=$('#conversationGlobalPolicy');if(!host)return;let editingKey=null;
 const own=(o,k)=>Object.prototype.hasOwnProperty.call(o,k);
 const btn=(label,action,attrs='',primary=false)=>'<button type="button" class="btn '+(primary?'primary':'ghost')+'" data-cv="'+action+'" '+attrs+'>'+label+'</button>';
 const opts=(entries,value)=>entries.map(([id,label])=>'<option value="'+esc(id)+'" '+(String(id)===String(value)?'selected':'')+'>'+esc(label)+'</option>').join('');
 const units=n=>n>=1073741824?(n/1073741824).toFixed(2)+' GiB':(n/1048576).toFixed(2)+' MiB';
 const projects=[{id:'tools',name:'桌面管理器',path:'~/Projects/desktop-manager'},{id:'site',name:'文档网站',path:'~/Projects/docs-site'}];
 const seedThreads=[
  {id:'config',project:'tools',name:'模型配置与对话边界',bytes:1887437},
  {id:'review',project:'tools',parent:'config',name:'审阅配置继承',bytes:838861},
  {id:'window',project:'tools',name:'窗口布局适配',bytes:3355443},
  {id:'long',project:'tools',name:'长期任务与工具记录',bytes:10834055004},
  {id:'old',project:'tools',name:'旧方案整理',bytes:12582912,archived:true},
  {id:'web',project:'site',name:'导航与文档检索',bytes:6710886},
  {id:'notes',project:null,name:'临时需求记录',bytes:838861}
 ];
 let threads=clone(seedThreads);
 const initial={global:{},'project:tools':{'subagent.model':'newapi|gpt-5.4','subagent.effort':'high'},'thread:config':{'subagent.effort':'medium'}};
 let saved=clone(initial),drafts={},orphanDrafts={},revisions={},savedStates={},tab='objects',selected='project:tools',query='',archived=false,onlyOwn=false,detailOpen=false;
 let objectNavigation=null,objectPanel=null,objectNodesSignature='',objectFilterSignature='',scenario='normal',source='local',notice='',noticeScope='project:tools',pagePositions={};
 const defaults={'naming.model':'newapi|gpt-5.4-mini','search.path':'sidecar','search.model':'newapi|gpt-5.4-mini','search.effort':'medium','search.fast':false,'subagent.model':'newapi|gpt-5.4','subagent.effort':'medium','subagent.fast':false,'subagent.roster':[{kind:'channel',channelId:'newapi',modelId:'gpt-5.4-mini'}],'subagent.concurrency':4,'subagent.instructions':''};
 const requestedDesignScope=new URLSearchParams(location.search).get('design');
 let designScope=requestedDesignScope==='capabilities'?'capabilities':requestedDesignScope==='extensions'?'extensions':requestedDesignScope==='conversations'||location.hash.startsWith('#conversations')?'conversations':'models';
 let recovery=null,operationToken=0,exported=null,imported=[],restoreCounter=0,recoveryScene='normal';
 let titleToken=0,titleDialog=null;
 const advancedStates={};
 const currentKey=()=>editingKey||selected;
 const editorHost=()=>editingKey==='global'?globalHost:host;
 const policy=(key,editing=true)=>editing&&own(drafts,key)?drafts[key]:(saved[key]||{});
 const scopedPolicy=(key,content)=>Object.fromEntries(Object.entries(content).filter(([field])=>own(defaults,field)&&(key==='global'||!field.startsWith('naming.'))));
 const buffer=key=>drafts[key]||(drafts[key]=clone(saved[key]||{}));
 const dirty=key=>JSON.stringify(policy(key))!==JSON.stringify(saved[key]||{});
 const dirtyKeys=()=>Object.keys(drafts).filter(dirty);
 function object(key){if(key==='global')return {name:'全局对话策略',kind:'全局',key};const [kind,id]=key.split(':');const item=kind==='project'?projects.find(p=>p.id===id):threads.find(t=>t.id===id);return {...(item||{name:'对象暂不可用'}),key,kind:kind==='project'?'项目':'对话'};}
 function resolve(key,field,editing=true,skipOwn=false){
  let value=defaults[field],from='Codex / OpenCodex 默认';
  const chain=['global'];const o=object(key);if(!field.startsWith('naming.')&&o.kind==='对话'&&o.project)chain.push('project:'+o.project);if(!field.startsWith('naming.')&&key!=='global')chain.push(key);
  for(const k of chain)if(!(skipOwn&&k===key)&&own(policy(k,editing&&k===key),field)){value=policy(k,editing&&k===key)[field];from=k==='global'?'全局对话策略':k===key?'本'+o.kind+'设置':object(k).kind+' · '+object(k).name;}
  return {value,from};
 }
 const effortLabels={none:'无',minimal:'最低',low:'低',medium:'中',high:'高',xhigh:'极高',max:'最高',ultra:'超高'};
 const modelRef=id=>{const [channelId,...parts]=String(id).split('|');const modelId=parts.join('|');return channelId==='native'?{kind:'native',nativeModelId:modelId}:{kind:'channel',channelId,modelId};};
 const modelId=ref=>(ref.kind==='native'?'native':ref.channelId)+'|'+(ref.nativeModelId||ref.modelId);
 let modelCatalog=null;
 function modelList(refresh=false){
  if(modelCatalog&&!refresh)return modelCatalog;
  const state=window.modelPrototype?.state(),resolved=window.modelPrototype?.agentChoices?.()||[],entries=[];
  for(const p of state?.saved.providers||[])for(const m of p.models){
   const row=resolved.find(row=>modelId(row.ref)===p.id+'|'+m.id);
   const values={context:row?.context??m.overrides?.context??p.defaults?.context??m.official?.context,default:row?.effort??m.overrides?.default??p.defaults?.default??m.official?.default,efforts:row?.efforts?.length?row.efforts:m.overrides?.efforts??p.defaults?.efforts??m.official?.efforts,fast:row?.fastCapability??m.overrides?.fast??p.defaults?.fast??m.official?.fast,modalities:row?.modalities||m.official?.modalities};
   const reason=!p.enabled?'渠道已停用':({'no-credit':'渠道无额度','auth-failed':'认证未通过','connection-failed':'连接不可用',unavailable:'上游不可用'})[p.availability?.kind]||(!m.selected?'模型未启用':'');
   const ref=p.id==='native'?{kind:'native',nativeModelId:m.id}:{kind:'channel',channelId:p.id,modelId:m.id};
   entries.push({id:p.id+'|'+m.id,ref,channelName:p.name,name:m.naming?.displayName||m.id,alias:m.naming?.alias||'',label:p.name+' / '+m.id,reason,disabled:!!reason,context:values.context??null,efforts:values.efforts||[],effort:values.default??null,fastCapability:values.fast||'unknown',fastDefault:false,modalities:values.modalities||[]});
  }
  modelCatalog=entries;return entries;
 }
 const modelName=id=>modelList().find(m=>m.id===id)?.label||id;
 function rosterRefs(value){return Array.isArray(value)?clone(value):String(value||'').split('\n').filter(Boolean).map(modelRef);}
 function actualEffort(key,prefix){const value=resolve(key,prefix+'.effort').value;return value==='model-default'?modelList().find(m=>m.id===resolve(key,prefix+'.model').value)?.effort:value;}
 function combinationText(ref,withOptions=true){const m=modelList().find(m=>m.id===modelId(ref));return (m?.label||modelId(ref))+(withOptions?' · '+(effortLabels[ref.reasoningEffort||m?.effort]||'推理未知')+(ref.fast?' · Fast':''):'');}
 function selectedModelRefs(key,field){
  if(field==='subagent.roster')return rosterRefs(resolve(key,field).value);
  const ref=modelRef(resolve(key,field).value);if(field==='naming.model')return [ref];
  const prefix=field.split('.')[0],effort=resolve(key,prefix+'.effort').value,fast=resolve(key,prefix+'.fast').value;
  if(effort&&effort!=='model-default')ref.reasoningEffort=effort;
  if(typeof fast==='boolean')ref.fast=fast;
  return [ref];
 }
 function display(field,value){if(field.endsWith('.model'))return modelName(value);if(field==='subagent.roster')return rosterRefs(value).map(ref=>combinationText(ref)).join('、')||'未选择推荐模型';if(field.endsWith('.fast'))return value?'开启':'关闭';return ({sidecar:'OpenCodex 搜索 sidecar',hosted:'原生 hosted search',service:'搜索服务','model-default':'模型默认',...effortLabels})[value]??(value===''?'未设置':value);}
 function modelField(field,label,note=''){
  const key=currentKey(),p=policy(key),multiple=field==='subagent.roster',prefix=field.split('.')[0],fields=multiple||prefix==='naming'?[field]:[field,prefix+'.effort',prefix+'.fast'];
  const custom=fields.some(f=>own(p,f)),disabled=!!object(key).archived,refs=selectedModelRefs(key,field),withOptions=prefix!=='naming';
  const modelSource=resolve(key,field).from,optionSources=!multiple&&withOptions?[resolve(key,prefix+'.effort').from,resolve(key,prefix+'.fast').from]:[];
  const source=optionSources.some(from=>from!==modelSource)?'型号：'+modelSource+' · 推理：'+resolve(key,prefix+'.effort').from+' · 快速：'+resolve(key,prefix+'.fast').from:modelSource;
  const inherited=!custom?'继承上层':'当前选择';
  const choices=refs.map(ref=>'<span class="cv-model-choice"><span class="cv-model-name">'+esc(modelName(modelId(ref)))+'</span>'+(withOptions?'<span class="cv-model-options">'+esc(effortLabels[ref.reasoningEffort||modelList().find(m=>m.id===modelId(ref))?.effort]||'推理未知')+(ref.fast?' · Fast':'')+'</span>':'')+'</span>').join('');
  return '<div class="cv-field cv-model-field" data-cv-model-field="'+field+'"><div class="cv-field-head"><label>'+label+'</label><span class="cv-field-origin">'+(custom?'本层自定义':'继承上层')+'</span></div><button type="button" class="cv-model-trigger" data-cv="pick-model" data-cv-picker="'+field+'" aria-haspopup="dialog" aria-label="选择'+label+'" '+(disabled?'disabled':'')+'><span class="cv-model-choices">'+(choices||'<span class="cv-model-empty">未选择推荐模型</span>')+'</span><span class="cv-model-action">'+(multiple?'选择模型':'更换')+'<svg viewBox="0 0 20 20" aria-hidden="true"><path d="m7 4 6 6-6 6"/></svg></span></button><div class="cv-effective"><strong>'+esc(inherited+(multiple?' · '+refs.length+' 个模型':''))+'</strong><span>来源：'+esc(source)+'</span></div>'+(custom?btn('恢复继承','model-inherit','data-field="'+field+'" '+(disabled?'disabled':'')):'')+(note?'<small class="cv-muted">'+note+'</small>':'')+'</div>';
 }
 function openModelSelection(field){
  if(!['naming.model','search.model','subagent.model','subagent.roster'].includes(field))return;
  const key=currentKey(),targetSource=source;if(object(key).archived||key!=='global'&&field==='naming.model')return;
  const original=JSON.stringify(policy(key)),refs=selectedModelRefs(key,field),multiple=field==='subagent.roster',options=field!=='naming.model';
  const labels={'naming.model':'命名模型','search.model':'搜索模型','subagent.model':'子 Agent 默认模型','subagent.roster':'推荐模型列表'};
  const focus=()=>{const scope=key==='global'?globalHost:host;$('[data-cv-picker="'+field+'"]',scope)?.focus({preventScroll:true});};
  const picker=window.modelPickerPrototype;
  if(!picker?.choose){noticeScope=key;notice='公共模型选择器尚未加载，请刷新原型。';render();return;}
  picker.choose({title:'选择'+labels[field],mode:multiple?'multiple':'single',reasoning:options,fast:options,ordered:multiple,models:refs,getCatalog:()=>modelList(true),
   onSelect:result=>{
    if(source!==targetSource||object(key).archived||JSON.stringify(policy(key))!==original)return;
    if(JSON.stringify(result.models)===JSON.stringify(refs)){focus();return;}
    const draft=buffer(key);
    if(multiple)draft[field]=clone(result.models);
    else{
     const ref=result.models[0];if(!ref)return;
     const changedModel=modelId(ref)!==modelId(refs[0]);
     if(changedModel||own(draft,field))draft[field]=modelId(ref);
     if(options){const prefix=field.split('.')[0];if(changedModel||ref.reasoningEffort!==refs[0].reasoningEffort)draft[prefix+'.effort']=ref.reasoningEffort||'model-default';if(changedModel||ref.fast!==refs[0].fast)draft[prefix+'.fast']=typeof ref.fast==='boolean'?ref.fast:false;}
    }
    noticeScope=key;notice='选择已加入当前配置，保存后生效。';render();focus();
   },onCancel:()=>focus()
  });
 }
 function ann(){return '<div class="proto-annotation" id="ann-cv-01" data-annotation-number="01" data-annotation-title="对话配置 · 版本未定" data-annotation-category="设计探索 · 对话"><button type="button" class="annotation-marker" aria-label="原型注释 01：对话配置 · 版本未定" aria-expanded="false" aria-controls="annotationPopover"><span>原型</span><span>01</span><span>＋</span></button><div class="annotation-source" hidden><p>本页只使用虚构对象和内存设置，不连接 Codex、模型或文件。模型目录复用模型页已保存样例；项目覆盖不改模型模版。保存与实际采用分别演示。</p></div></div>';}
 function field(field,label,type='select',entries=[],note=''){
  if(type==='model'||type==='models')return modelField(field,label,note);
  const key=currentKey(),o=object(key),custom=own(policy(key),field),effective=resolve(key,field),inherited=resolve(key,field,true,true),disabled=!!o.archived;
  const attrs=' id="'+(key==='global'?'cvGlobal-':'cv-')+field.replaceAll('.','-')+'" data-cv-field="'+field+'" aria-label="'+label+'" '+(disabled?'disabled':'');
  const inheritLabel='继承：'+display(field,inherited.value)+'（'+inherited.from+'）';
  let control='';
  if(type==='select'){
   const values=opts(entries,custom?effective.value:undefined);
   const unknown=custom&&!entries.some(([id])=>String(id)===String(effective.value))?'<option selected disabled value="'+esc(effective.value)+'">'+esc(display(field,effective.value))+' · 当前值不兼容</option>':'';
   control='<select'+attrs+'><option value="__inherit" '+(!custom?'selected':'')+'>'+esc(inheritLabel)+'</option>'+unknown+values+'</select>';
  }else if(type==='textarea')control='<textarea class="input"'+attrs+'>'+esc(effective.value)+'</textarea>';
  else control='<input class="input" type="number" min="1" max="32"'+attrs+' value="'+esc(effective.value)+'">';
  return '<div class="cv-field"><div class="cv-field-head"><label>'+label+'</label><span class="cv-field-origin">'+(custom?'本层自定义':'继承上层')+'</span></div><div class="cv-field-control">'+control+'</div><div class="cv-effective"><strong>当前有效：'+esc(display(field,effective.value))+'</strong><span>来源：'+(effective.from==='全局对话策略'?'<button type="button" class="cv-source-link" data-cv="global-policy">全局对话策略</button>':esc(effective.from))+'</span></div>'+(custom||type==='models'?btn('恢复继承','field-inherit','data-field="'+field+'" '+(disabled?'disabled':'')):'')+(note?'<small class="cv-muted">'+note+'</small>':'')+'</div>';
 }

 function fieldError(key){
  const problems=[];
  for(const prefix of ['naming','search','subagent']){
   if(prefix==='naming'&&key!=='global'||prefix==='search'&&resolve(key,'search.path').value!=='sidecar')continue;
   const id=resolve(key,prefix+'.model').value,m=modelList().find(m=>m.id===id);
   if(!m||m.disabled||scenario==='no-model')problems.push('所选'+({naming:'命名',search:'搜索',subagent:'子 Agent'})[prefix]+'模型暂不可用；保留引用，实际采用待处理。');
   if(prefix!=='naming'&&m&&actualEffort(key,prefix)&&!m.efforts.includes(actualEffort(key,prefix)))problems.push('推理档位与所选模型不匹配，请调整本层或上层设置。');
  }
  for(const prefix of ['search','subagent']){if(prefix==='search'&&resolve(key,'search.path').value!=='sidecar')continue;const m=modelList().find(m=>m.id===resolve(key,prefix+'.model').value);if(resolve(key,prefix+'.fast').value===true&&m?.fastCapability!=='supported')problems.push('已选快速模式当前不可用，请重新选择模型组合。');}
  for(const ref of rosterRefs(resolve(key,'subagent.roster').value)){const m=modelList().find(m=>m.id===modelId(ref));if(!m||m.disabled||ref.reasoningEffort&&!m.efforts.includes(ref.reasoningEffort)||ref.fast===true&&m.fastCapability!=='supported')problems.push('推荐列表中有模型组合暂不可用；保留引用，请重新选择。');}
  const count=resolve(key,'subagent.concurrency').value;if(!Number.isInteger(count)||count<1||count>32)problems.push('最大并发请输入 1–32 的整数。');
  return problems;
 }
 function groupStatus(group){
  if(scenario==='offline')return '执行端未连接';
  if(scenario==='unsupported'&&(currentKey()!=='global'||group==='subagent'))return '此作用域暂未接通';
  if(savedStates[currentKey()]==='observed')return '已观察采用 · 示例';
  if(['received','saved'].includes(savedStates[currentKey()]))return '已保存 · 采用待观察';
  return '尚未观察采用';
 }
 function editor(){
  const key=currentKey(),o=object(key),isGlobal=key==='global',prefix=isGlobal?'cvGlobal':'cv',path=resolve(key,'search.path').value,issues=fieldError(key);
  const dirtyCount=dirtyKeys().length;
  const tools='<details class="cv-tools"><summary class="btn ghost">配置工具</summary><div class="cv-menu">'+btn('查看有效配置','effective')+btn('恢复本层继承','inherit',o.archived?'disabled':'')+btn('丢弃本对象修改','discard',dirty(key)?'':'disabled')+btn('导出当前配置','config-export')+btn('导入对话配置','config-import')+btn('导出重装恢复包','recovery-export')+btn('导入恢复包','recovery-import')+btn('查看模型规则','models')+'</div></details>';
  return '<div class="cv-editor"><div class="cv-editor-top">'+(isGlobal?'':btn('← 返回项目与对话','back','',''))+'<div class="cv-heading"><div><div class="cv-muted">'+o.kind+(o.kind==='对话'?' · '+esc(projects.find(p=>p.id===o.project)?.name||'未分配项目'):'')+'</div><h2>'+esc(isGlobal?'全局默认':o.name)+'</h2><div class="cv-muted">'+(o.kind==='全局'?'影响沿用全局设置的项目与对话':o.kind==='项目'?'影响该项目中继承设置的对话':'只保存当前对话的专属覆盖')+'</div></div>'+tools+'</div>'+(o.kind==='对话'?'<div class="cv-actions">'+btn('修改标题','rename',o.archived?'disabled':'')+btn('按当前内容重新命名','regenerate-title',o.archived?'disabled':'')+'<small class="cv-muted">修改标题是一次操作；生成时使用已保存的全局命名模型。</small></div>':'')+'<div class="cv-actions"><button class="btn primary cv-save" data-cv="save" '+(!dirty(key)||o.archived?'disabled':'')+'>保存'+(o.kind==='全局'?'全局':o.kind==='项目'?'项目':'对话')+'配置<span id="'+prefix+'DirtyState">'+(dirty(key)?'有未保存修改':'已保存')+'</span></button><span class="cv-muted" id="'+prefix+'DraftCount">'+(dirtyCount?'共 '+dirtyCount+' 个对象有草稿':'未单独设置的字段自动继承')+'</span></div></div>'+
   (notice&&noticeScope===key?'<div class="cv-banner" role="status">'+esc(notice)+'</div>':'')+
   (o.archived?'<div class="cv-banner">此对话已归档，专属配置已清理。这里只预览继承值；取消归档后可重新设置。</div>'+(orphanDrafts[key]?'<details><summary>查看归档前未保存输入</summary><p class="cv-muted">临时保留供手动复制，不会重新激活此对话配置。</p><textarea class="input" readonly aria-label="归档前未保存输入">'+esc(JSON.stringify(orphanDrafts[key],null,2))+'</textarea></details>':''):'')+
   (scenario==='offline'?'<div class="cv-banner cv-warning">数据源暂不可达，显示上次对象信息。已有绑定可以继续编辑和保存，执行采用等待重连。</div>':'')+
   (issues.length?'<div class="cv-banner cv-warning" role="status">'+issues.map(esc).join('<br>')+'</div>':'')+
   (key==='global'?'<section class="cv-section" aria-labelledby="'+prefix+'NamingTitle"><div class="cv-heading"><h3 id="'+prefix+'NamingTitle">命名模型</h3><span class="cv-status">'+groupStatus('naming')+'</span></div><p class="cv-muted">沿用 Codex 自动命名，只设置全局使用的命名模型。</p>'+field('naming.model','命名模型','model')+'</section>':'')+
   '<section class="cv-section" aria-labelledby="'+prefix+'SearchTitle"><div class="cv-heading"><h3 id="'+prefix+'SearchTitle">联网搜索</h3><span class="cv-status">'+groupStatus('search')+'</span></div><p class="cv-muted">先确认搜索路径，再选择该路径实际使用的模型。</p>'+field('search.path','搜索路径','select',[['sidecar','OpenCodex 搜索 sidecar'],['hosted','原生 hosted search'],['service','搜索服务']])+(path==='sidecar'?field('search.model','搜索模型','model'):'<div class="cv-banner">'+(path==='hosted'?'此路径由原生上游执行，sidecar 模型设置不影响它。':'此路径使用搜索服务，不提供 LLM 模型与推理档位设置。')+'</div>')+'</section>'+
   '<section class="cv-section" aria-labelledby="'+prefix+'AgentTitle"><div class="cv-heading"><h3 id="'+prefix+'AgentTitle">子 Agent</h3><span class="cv-status">'+groupStatus('subagent')+'</span></div><p class="cv-muted">用于 Codex 对话中的子任务。模型模版页的自动填充助手另行配置。</p>'+field('subagent.model','子 Agent 默认模型','model',[],'执行端支持硬性默认时采用；推荐列表不会替代默认值。')+'<details class="cv-advanced"><summary>推荐模型、并发与公共指令</summary>'+field('subagent.roster','推荐模型列表','models',[],'可多选模型，并为每项设置推理与快速模式；推荐不保证执行器强制选择。')+field('subagent.concurrency','最大并发','number',[],'限制范围取决于执行器；降低数值不会取消运行中的任务。')+field('subagent.instructions','公共子 Agent 指令','textarea')+'<p class="cv-muted">已有角色的模型 / 推理映射待对应执行器接通；新建角色和权限编辑未纳入本轮通用表单。</p></details></section>'+
   executionDetails(key,isGlobal)+'</div>';
 }
 function executionDetails(key,isGlobal){
  const text=isGlobal?'命名仍由 Codex 原有流程触发；搜索在下一次请求构建时读取，子 Agent 在新任务创建时读取。保存全局默认后，实际采用仍由具体对话记录确认。':'保存成功只代表配置已保存。搜索在下一次请求构建时读取，子 Agent 在新任务创建时读取；尚未接通的路径继续标记为待观察。';
  return '<details class="cv-advanced"><summary>'+(isGlobal?'执行采用详情':'执行采用与连接详情')+'</summary><p class="cv-muted">'+text+'</p>'+(isGlobal?'':'<div data-transport-context="conversation" data-transport-object="'+esc(key)+'" data-transport-source="'+esc(source)+'"></div><div data-context-policy-observation data-context-object="'+esc(key)+'" data-context-source="'+esc(source)+'"></div>')+'</details>';
 }
 function updateDirty(){const scope=editorHost(),key=currentKey(),prefix=key==='global'?'cvGlobal':'cv',b=$('[data-cv="save"]',scope);if(b)b.disabled=!dirty(key)||!!object(key).archived;const s=$('#'+prefix+'DirtyState',scope);if(s)s.textContent=dirty(key)?'有未保存修改':'已保存';const c=$('#'+prefix+'DraftCount',scope);if(c)c.textContent=dirtyKeys().length?'共 '+dirtyKeys().length+' 个对象有草稿':'未单独设置的字段自动继承';syncObjectNavigation();}
 function objectNodes(){
  if(scenario==='empty')return [];
  const state=(key,archived=false)=>archived?{status:'归档'}:dirty(key)?{status:'未保存',tone:'warning'}:Object.keys(saved[key]||{}).length?{status:'专属'}:{};
  const items=projects.map(p=>({id:'project:'+p.id,label:p.name,kind:'folder',keywords:p.path,own:dirty('project:'+p.id)||Object.keys(saved['project:'+p.id]||{}).length>0,...state('project:'+p.id)}));
  items.push({id:'group:unassigned',label:'未分配项目',kind:'group',selectable:false});
  for(const t of threads){const key='thread:'+t.id;items.push({id:key,parent:t.parent&&threads.some(p=>p.id===t.parent)?'thread:'+t.parent:t.project&&projects.some(p=>p.id===t.project)?'project:'+t.project:'group:unassigned',label:t.name,kind:t.parent?'agent':'chat',description:'',archived:!!t.archived,own:dirty(key)||Object.keys(saved[key]||{}).length>0,...state(key,t.archived)});}
  return items;
 }
 function objectPredicate(node){return archived?!!node.archived:!node.archived&&(!onlyOwn||node.own);}
 function resetObjectFilters(){query='';archived=false;onlyOwn=false;objectFilterSignature='';syncObjectNavigation();}
 function syncObjectTools(state){
  if(!objectPanel)return;const input=$('#cvObjectSearch',objectPanel);if(input.value!==state.query)input.value=state.query;query=state.query;
  $('[data-cv="tree-clear"]',objectPanel).hidden=!query;
  $('#cvObjectCount',objectPanel).textContent=(state.filtered?state.matches:state.rows.length)+' 项';
  for(const b of $$('[data-cv="tree-filter"]',objectPanel))b.setAttribute('aria-pressed',String(b.dataset.filter===(archived?'archived':onlyOwn?'own':'all')));
 }
 function syncObjectNavigation(){
  if(!objectNavigation)return;
  const nodes=objectNodes(),signature=JSON.stringify(nodes),wantedQuery=query,filter=(archived?'archived':onlyOwn?'own':'all');
  if(signature!==objectNodesSignature){objectNodesSignature=signature;objectNavigation.setNodes(nodes);}
  if(objectNavigation.state().selected!==selected)objectNavigation.setSelected(selected);
  if(filter!==objectFilterSignature){objectFilterSignature=filter;objectNavigation.setFilter(objectPredicate,{reveal:filter!=='all'});}
  if(objectNavigation.state().query!==wantedQuery)objectNavigation.setQuery(wantedQuery);
  syncObjectTools(objectNavigation.state());
 }
 function mountObjectNavigation(){
  const mount=$('#cvTreeMount',host);if(!mount)return;
  if(!objectPanel){
   objectPanel=document.createElement('aside');objectPanel.className='cv-tree-panel tn-panel';objectPanel.setAttribute('aria-label','项目与对话导航');
   objectPanel.innerHTML='<div class="tn-panel-head"><h3>项目与对话</h3><div class="tn-panel-tools"><button type="button" class="btn ghost btn-sm" data-cv="refresh" aria-label="刷新项目与对话" title="刷新">↻</button><details class="tn-panel-more"><summary aria-label="项目与对话导航操作" title="更多操作">⋯</summary><div class="tn-panel-menu">'+btn('全部展开','tree-expand')+btn('全部收起','tree-collapse')+'</div></details></div></div><div class="tn-panel-search"><input class="input" id="cvObjectSearch" type="text" role="searchbox" aria-label="搜索项目或对话" placeholder="搜索项目或对话" autocomplete="off" value="'+esc(query)+'"><button type="button" class="tn-panel-clear" data-cv="tree-clear" aria-label="清空项目与对话搜索" hidden>×</button></div><div class="tn-panel-filter" role="group" aria-label="项目与对话筛选">'+[['all','全部'],['own','专属'],['archived','归档']].map(([id,label])=>'<button type="button" data-cv="tree-filter" data-filter="'+id+'" aria-pressed="'+(id==='all')+'">'+label+'</button>').join('')+'<span id="cvObjectCount" class="tn-panel-count" role="status"></span></div><div id="cvObjectTree"></div>';
   objectNavigation=window.PrototypeTreeNavigation.mount($('#cvObjectTree',objectPanel),{nodes:objectNodes(),selected,expanded:['project:tools','thread:config','group:unassigned'],label:'项目与对话',density:'normal',onActivate(node){choose(node.id);},onUpdate:syncObjectTools,onClear:resetObjectFilters,onReveal(){const node=objectNavigation.store.node(selected);archived=!!node?.archived;onlyOwn=false;query='';objectFilterSignature='';syncObjectNavigation();}});
   objectNodesSignature=JSON.stringify(objectNodes());
  }
  mount.replaceWith(objectPanel);syncObjectNavigation();
 }
 function render(){
  modelCatalog=null;
  const previousScope=editingKey;editingKey=null;
  const scroller=host.closest('.main'),position=scroller?.scrollTop||0,navFocus=objectPanel?.contains(document.activeElement)?document.activeElement:null,navCaret=navFocus?.selectionStart,navScroll=objectPanel?.querySelector('.tn-tree')?.scrollTop||0;
  objectPanel?.remove();
  if(host.dataset.objectKey)advancedStates[host.dataset.objectKey]=$$('.cv-advanced[open]',host).map(el=>$('summary',el)?.textContent);
  host.innerHTML='<div class="cv-note-anchor cv-object-heading"><h2>项目与对话</h2><div class="cv-actions">'+btn('全局对话策略','global-policy')+'</div>'+ann()+'</div>'+(scenario==='sources'?'<div class="cv-source"><label class="cv-muted">数据源 <select id="cvSource" aria-label="Codex 数据源">'+opts([['local','本机 Codex'],['dev','开发机 Codex · 仅预览']],source)+'</select></label></div>':'')+(source==='dev'?'<div class="cv-empty">开发机来源尚未连接。此处只读预览，不会把本机覆盖套给另一来源。<br>'+btn('返回本机','local')+'</div>':'<div class="cv-workspace '+(detailOpen?'cv-detail-open':'')+'">'+'<div id="cvTreeMount"></div>'+editor()+'</div>');
  mountObjectNavigation();if(objectPanel){const tree=$('.tn-tree',objectPanel);if(tree)tree.scrollTop=navScroll;}
  if(navFocus?.isConnected&&navFocus.getClientRects().length){navFocus.focus({preventScroll:true});if(navCaret!=null&&navFocus.setSelectionRange)navFocus.setSelectionRange(navCaret,navCaret);}
  $$('[data-cv="back"]',host).forEach(b=>b.classList.add('cv-back'));
  host.dataset.objectKey=currentKey();for(const el of $$('.cv-advanced',host))el.open=(advancedStates[currentKey()]||[]).includes($('summary',el)?.textContent);
  window.prototypeSelect?.enhance();window.prototypePanel?.refresh();window.transportPrototype?.refreshContexts();window.contextPolicyPrototype?.refreshObservations();if(scroller)scroller.scrollTop=position;renderGlobal();editingKey=previousScope;
 }
 function renderGlobal(){if(!globalHost)return;modelCatalog=null;const previous=editingKey;editingKey='global';const opened=$$('.cv-advanced[open]',globalHost).map(el=>$('summary',el)?.textContent);globalHost.innerHTML=editor();for(const el of $$('.cv-advanced',globalHost))el.open=opened.includes($('summary',el)?.textContent);window.prototypeSelect?.enhance(globalHost);editingKey=previous;}
 function openGlobal(){if(window.openDialogPolicy)window.openDialogPolicy('models');else window.openSettingsSection?.('dialog-policy');}

 function choose(key){const main=host.closest('.main');pagePositions[currentKey()]=main.scrollTop;selected=key;detailOpen=true;notice='';render();main.scrollTop=pagePositions[key]||0;requestAnimationFrame(()=>{if(objectPanel&&!objectPanel.getClientRects().length){const heading=$('.cv-editor h2',host);heading?.setAttribute('tabindex','-1');heading?.focus({preventScroll:true});}});}
 function editTitle(generate=false){
  const o=object(currentKey());if(o.kind!=='对话'||o.archived)return;
  const target=threads.find(t=>t.id===o.id),snapshot=target.name,model=resolve('global','naming.model',false).value;
  titleDialog={id:o.id,title:snapshot,model};const token=++titleToken;
  openModal(generate?'按当前内容重新命名':'修改对话标题','<p class="cv-muted">'+(generate?'读取本对话当前可确认的内容生成候选；先预览，可编辑后再采用。命名模型来自已保存的全局设置。':'只修改显示标题，不改项目归属、模型或当前执行任务。')+'</p><div class="cv-banner">当前标题：'+esc(snapshot)+(generate?'<br>命名模型：'+esc(modelName(model)):'')+'</div><label class="cv-path-field">新标题<input class="input" id="cvTitleInput" aria-label="新标题" maxlength="100" value="'+(generate?'':esc(snapshot))+'"></label><p class="cv-muted" id="cvTitleStatus" role="status">'+(generate?'正在生成候选 · 内存演示':'修改立即更新显示标题，和配置保存分别处理。')+'</p><p class="cv-error" id="cvTitleError" role="alert"></p>','采用标题',()=>{
   const latest=threads.find(t=>t.id===o.id),value=$('#cvTitleInput')?.value.trim();
   if(!latest||latest.archived){$('#cvTitleError').textContent='对象已归档或不可用，未修改标题。';return;}
   if(!value){$('#cvTitleError').textContent='请输入标题。';return;}
   if(latest.name!==snapshot){$('#cvTitleError').textContent='当前标题已经改变，请关闭后重新生成或编辑。';return;}
   if(scenario==='offline'){ $('#cvTitleError').textContent='数据源未连接，尚未回写标题；输入保留。';return;}
   latest.name=value;titleToken++;titleDialog=null;closeModal();notice='标题已更新（演示），当前任务和配置保持。';render();
  },{confirmKeepsOpen:true,hideNote:true});
  window.__modalOnClose=()=>{titleToken++;titleDialog=null;};
  if(generate){$('#modalConfirm').disabled=true;setTimeout(()=>{
   if(token!==titleToken||titleDialog?.id!==o.id)return;
   const m=modelList().find(m=>m.id===model);
   if(scenario==='offline'||!m||m.disabled||scenario==='no-model'){$('#cvTitleStatus').textContent='命名模型或数据源暂不可用，原有标题保持。';$('#cvTitleInput').value=snapshot;$('#modalConfirm').disabled=false;return;}
   if(!$('#cvTitleInput').value.trim())$('#cvTitleInput').value=o.id==='config'?'对话模型选择与继承交互调整':snapshot+' · 当前任务';
   $('#cvTitleStatus').textContent='候选已生成（示例），未采用。可编辑后采用；真实内容读取与模型调用尚未接入。';$('#modalConfirm').disabled=false;
  },450);}else requestAnimationFrame(() => $('#cvTitleInput')?.focus());
 }
 function commit(keys,quiet=false){
  noticeScope=keys.length===1?keys[0]:currentKey();
  if(keys.some(k=>object(k).archived)){notice='归档对象不能保存新的专属覆盖。';render();return false;}
  if(keys.some(k=>fieldError(k).some(x=>x.includes('整数')||x.includes('不匹配')||x.includes('快速模式')||x.includes('推荐列表')))){notice='请先修正档位组合或并发数；草稿已保留。';render();return false;}
  if(scenario==='save-fail'){notice='保存失败，草稿已保留。请稍后重试。';render();return false;}
  for(const k of keys){const p=scopedPolicy(k,clone(policy(k)));if(Object.keys(p).length)saved[k]=p;else delete saved[k];delete drafts[k];revisions[k]=(revisions[k]||0)+1;savedStates[k]=scenario==='offline'||scenario==='unsupported'?'pending':'saved';}
  notice=quiet?'':'配置已保存。实际采用状态按功能分别观察。';render();return true;
 }
 function openRecovery(mode='export',context=currentKey()){
  if(!recovery||recovery.mode!==mode){recovery={mode,step:0,context,selected:new Set(),folds:new Set(['tools','site','unassigned']),withArchived:false,withChildren:true,withParents:true,withAssets:false,reviewed:false,destination:'~/Documents/Codex-重装恢复.zip',target:'~/Projects',package:null,error:'',progress:0,status:'idle',includeConfig:true,restoreGlobal:recoveryScene!=='conflict'};
   if(mode==='export'){
    const o=object(context);for(const t of threads)if(!t.archived&&!t.parent&&(context==='global'||o.kind==='项目'&&t.project===o.id||o.kind==='对话'&&t.id===o.id))recovery.selected.add(t.id);
   }
  }
  window.syncPrototype?.open({tab:'file'});window.syncPrototype?.render();
 }
 function policyImport(){
  window.syncPrototype?.open({tab:'file'});
  const previous=window.syncPrototype?.state().lastExport?.modules;
  const modules=previous?.length?previous:[{id:'dialog-policy:project:tools',content:{'subagent.effort':'high'}}];
  const targets=['global',...projects.map(p=>'project:'+p.id),...threads.filter(t=>!t.archived).map(t=>'thread:'+t.id)];
  openModal('导入对话配置','<p class="cv-muted">只导入配置，来源与目标分别选择；此演示不读取磁盘、不导入历史。将采用所选覆盖，未包含字段保持目标设置。</p><label class="cv-path-field">包内配置<select id="cvPolicySource" aria-label="包内配置">'+opts(modules.map((m,i)=>[i,m.id]),0)+'</select></label><label class="cv-path-field">应用到<select id="cvPolicyTarget" aria-label="配置目标">'+opts(targets.map(k=>[k,object(k).name+' · '+object(k).kind]),targets.includes(currentKey())?currentKey():'global')+'</select></label><pre id="cvPolicyPreview">'+esc(JSON.stringify(modules[0].content,null,2))+'</pre><div id="cvPolicyError" class="cv-error" role="alert"></div>','采用所选配置',()=>{
   const target=$('#cvPolicyTarget').value,entry=modules[Number($('#cvPolicySource').value)];if(dirty(target)){$('#cvPolicyError').textContent='目标对象有未保存修改，请先返回对话页处理；草稿保持。';return;}
   if(object(target).archived){$('#cvPolicyError').textContent='归档对象不能创建专属覆盖。';return;}
   if(target!=='global'&&Object.keys(entry.content).some(f=>f.startsWith('naming.'))){$('#cvPolicyError').textContent='命名模型仅支持全局，请选择全局目标，或返回仅选择项目 / 对话的搜索与子 Agent 配置。';return;}
   if(Object.keys(entry.content).some(f=>!own(defaults,f))){$('#cvPolicyError').textContent='包中包含当前原型不支持的字段，尚未应用。';return;}
   saved[target]=scopedPolicy(target,{...(saved[target]||{}),...clone(entry.content)});delete drafts[target];revisions[target]=(revisions[target]||0)+1;savedStates[target]='pending';closeModal();window.syncPrototype?.refreshCatalog();noticeScope=target;notice='已导入到 '+object(target).name+'；未改变 WebDAV 范围，实际采用待观察。';tab='objects';if(target!=='global')selected=target;detailOpen=true;render();
   const node=$('#sxPanel');node.insertAdjacentHTML('beforeend','<p class="sx-result" role="status">'+esc(notice)+' '+btn('查看目标配置','config-target','data-key="'+target+'"')+'</p>');
  },{hideNote:true,confirmKeepsOpen:true});window.prototypeSelect?.enhance();
  $('#cvPolicySource').addEventListener('change',e=>$('#cvPolicyPreview').textContent=JSON.stringify(modules[Number(e.target.value)].content,null,2));
 }
 function recThreads(){const r=recovery;const ids=new Set([...r.selected].filter(id=>threads.some(t=>t.id===id&&(!t.archived||r.withArchived))));if(r.withChildren)for(const t of threads)if(ids.has(t.parent)&&(!t.archived||r.withArchived))ids.add(t.id);return threads.filter(t=>ids.has(t.id));}
 function recKeys(){const keys=new Set();if(!recovery.includeConfig)return [];for(const t of recThreads()){keys.add('thread:'+t.id);if(recovery.withParents){keys.add('global');if(t.project)keys.add('project:'+t.project);}}return [...keys];}
 function recSize(){return recThreads().reduce((n,t)=>n+t.bytes,0)+(recovery.withAssets?12582912:0)+recKeys().length*2048;}
 function recSelection(){
  const r=recovery;let rows='';
  for(const p of [...projects,{id:'unassigned',name:'未分配项目'}]){
   const list=threads.filter(t=>(t.project||'unassigned')===p.id&&(!t.archived||r.withArchived)&&!t.parent);
   if(!list.length)continue;const picked=list.filter(t=>r.selected.has(t.id)).length;
   rows+=window.PrototypeTreeTable.row({className:'cv-recovery-row',label:p.name,selection:'<input type="checkbox" data-cv-project-pick="'+p.id+'" aria-label="选择项目 '+p.name+' 的当前对话" '+(picked===list.length?'checked':'')+' '+(picked&&picked<list.length?'data-partial="true"':'')+'>',toggleAttrs:'data-cv="rec-fold" data-id="'+p.id+'"',expanded:r.folds.has(p.id),nameHtml:'<strong>'+p.name+'</strong><small>'+picked+' / '+list.length+' 个主对话</small>',cells:['<span class="cv-size">'+units(list.reduce((n,t)=>n+t.bytes,0))+'</span>']});
   if(r.folds.has(p.id))for(const t of list)rows+=window.PrototypeTreeTable.row({className:'cv-recovery-row',depth:1,label:t.name,selection:'<input type="checkbox" data-cv-thread-pick="'+t.id+'" aria-label="导出 '+esc(t.name)+'" '+(r.selected.has(t.id)?'checked':'')+'>',nameHtml:esc(t.name)+(t.archived?'<small>已归档</small>':'')+(threads.some(c=>c.parent===t.id)?'<small>包含可选子 Agent 对话</small>':''),cells:['<span class="cv-size">'+units(t.bytes)+'</span>']});
  }
  return '<div class="cv-rec-options"><label><input type="checkbox" data-cv-rec="withArchived" '+(r.withArchived?'checked':'')+'>包括归档对话</label><label><input type="checkbox" data-cv-rec="withChildren" '+(r.withChildren?'checked':'')+'>携带关联子 Agent 对话</label></div><div class="proto-tree">'+rows+'</div><div class="cv-rec-options"><label><input type="checkbox" data-cv-rec="includeConfig" '+(r.includeConfig?'checked':'')+'>关联配置</label><label><input type="checkbox" data-cv-rec="withParents" '+(r.withParents?'checked':'')+'>携带上层配置</label><label><input type="checkbox" data-cv-rec="withAssets" '+(r.withAssets?'checked':'')+'>附件与产物 · 示例 12 MiB</label></div><p class="cv-muted">这是一次性本地恢复包。未选择的对话不进入包，也不会改变 WebDAV 范围。</p>';
 }
 function recoveryPanel(){
  const r=recovery,importing=r.mode==='import',steps=importing?['选择恢复包','核对与映射','恢复结果']:['选择范围','体积与位置','导出结果'];
  let content='',actions='';
  if(!importing&&r.step===0){content=recSelection();actions=btn('下一步：体积与位置','rec-next',recThreads().length?'':'disabled',true);}
  if(!importing&&r.step===1){const draftsInScope=recKeys().filter(dirty);content='<dl class="cv-summary"><div><dt>所选历史</dt><dd>'+recThreads().length+' 份 · '+units(recThreads().reduce((n,t)=>n+t.bytes,0))+'</dd></div><div><dt>关联配置</dt><dd>'+recKeys().length+' 个作用域</dd></div><div><dt>原始量合计</dt><dd>'+units(recSize())+'</dd></div><div><dt>压缩后大小</dt><dd>生成时计算</dd></div></dl>'+ (recThreads().some(t=>t.bytes>1073741824)?'<div class="cv-banner cv-warning">包含 GiB 级原生记录。完整恢复保留这些记录，预览不会假定固定压缩率；可返回调整范围。</div>':'')+'<label class="cv-path-field">保存到重装后仍可访问的位置<input class="input" id="cvExportPath" value="'+esc(r.destination)+'" placeholder="例如外置磁盘或不会被清理的目录"></label><p class="cv-muted">此路径为内存示例，不会生成真实文件。包不包含整个项目源码或账户凭据。</p>'+(draftsInScope.length?'<div class="cv-banner">范围内有 '+draftsInScope.length+' 个未保存对象。'+btn('保存相关草稿后导出','rec-save-drafts')+'<label><input type="checkbox" data-cv-rec="reviewed" '+(r.reviewed?'checked':'')+'>仅导出已保存配置</label></div>':'');actions=btn('返回范围','rec-back')+btn('生成本地恢复包','rec-run',!draftsInScope.length||r.reviewed?'':'disabled',true);}
  if(importing&&r.step===0){content='<div class="cv-banner">选择重装前保存的本地包，先读取清单再恢复。对话配置和历史均不自动开启同步。</div>'+btn('选择本地恢复包','rec-file')+(r.package?'<dl class="cv-summary"><div><dt>恢复包</dt><dd>'+esc(r.package.name)+'</dd></div><div><dt>内容</dt><dd>'+r.package.threads.length+' 份对话 · '+Object.keys(r.package.policies).length+' 份配置</dd></div></dl>':'<p class="cv-muted">尚未选择文件。原型选择只载入示例，不读取磁盘。</p>');actions=btn('读取并核对','rec-next',r.package?'':'disabled',true);}
  if(importing&&r.step===1){content='<div class="cv-banner">'+(recoveryScene==='conflict'?'目标已存在同名项目。恢复为新对话，保留原对象；旧 ID 仅用于包内映射。':'目标为重装后的空环境，先恢复记录并重新关联配置。')+'</div><div class="cv-map-row"><span>目标 Codex 来源</span><strong>本机 Codex · 重装后</strong></div><label class="cv-map-row"><span>项目工作目录</span><input class="input" id="cvImportRoot" value="'+esc(r.target)+'"></label><div class="cv-map-row"><span>恢复项目的子 Agent 默认模型</span><button type="button" class="cv-model-trigger" data-cv="rec-model" aria-haspopup="dialog" aria-label="选择恢复后的模型"><span class="cv-model-name">'+esc(combinationText(r.modelChoice||modelRef(r.model||'newapi|gpt-5.4')))+'</span><span class="cv-model-action">更换</span></button></div><label class="cv-rec-options"><input type="checkbox" data-cv-rec="restoreGlobal" '+(r.restoreGlobal?'checked':'')+'>采用包内全局对话配置（影响目标环境的继承值）</label><dl class="cv-summary"><div><dt>项目 / 对话</dt><dd>'+r.package.threads.length+' 份记录，恢复后建立身份映射</dd></div><div><dt>配置</dt><dd>按新对象关联，继承关系保留</dd></div><div><dt>下一步</dt><dd>恢复后由用户主动继续对话</dd></div></dl><p class="cv-muted">不恢复运行中的请求、终端或命名任务，不自动开始模型调用。缺依赖 / 版本不支持由结果单独说明。</p>';actions=btn('返回文件','rec-back')+btn('恢复项目与对话','rec-run','',true);}
  if(r.step===2){content='<p role="status">'+esc(r.status==='running'?(importing?'正在恢复记录与配置':'正在生成恢复包'):r.status==='complete'?(importing?'恢复完成 · 内存演示':'恢复包已准备 · 内存演示'):r.status==='partial'?'部分完成 · 可重试未完成步骤':r.status==='cancelled'?'已取消，源数据保持':'操作未完成')+'</p><progress class="cv-progress" max="100" value="'+r.progress+'"></progress>'+(r.status==='running'?'<p class="cv-muted">'+(r.progress<50?'核对范围与读取历史':'处理关联配置与完成校验')+'。大文件分段处理，不需要一次加载全部记录。</p>':r.status==='complete'?'<ol class="cv-result-list">'+(importing?'<li>'+r.package.threads.length+' 份对话历史已恢复。</li><li>配置已关联到恢复后的对象，未沿用旧执行状态。</li><li>未自动启动任务，WebDAV 范围保持原选择。</li>':'<li>保存位置：'+esc(r.destination)+'</li><li>'+recThreads().length+' 份历史及 '+recKeys().length+' 个作用域配置。</li><li>原对象归档不会删除这个独立恢复包。</li>')+'</ol>':'<div class="cv-banner">'+esc(r.error||'记录已恢复，配置绑定尚未完成。重试仅完成剩余步骤，不重复创建对话。')+'</div>');actions=r.status==='running'?btn('取消本次操作','rec-cancel'):r.status==='complete'?(importing?btn('查看恢复后的对话','rec-view','',true):btn('演示重装后导入此包','rec-import','',true))+btn('返回文件同步','rec-close'):btn('返回核对','rec-back')+btn('重试未完成步骤','rec-retry','',true);}
  return '<div class="cv-recovery"><header class="cv-heading"><div><h2>'+(importing?'导入重装恢复包':'导出重装恢复包')+'</h2><p class="cv-muted">本地文件 · 历史与配置关联恢复</p></div>'+btn('返回文件同步','rec-close',r.status==='running'?'disabled':'')+'</header><ol class="cv-steps">'+steps.map((s,i)=>'<li '+(r.step===i?'aria-current="step"':'')+'>'+(i+1)+' '+s+'</li>').join('')+'</ol><div class="cv-recovery-content">'+content+(r.error&&r.step!==2?'<div class="cv-banner cv-error" role="alert">'+esc(r.error)+'</div>':'')+'</div><footer class="cv-recovery-footer"><span class="cv-muted">'+(importing?'只恢复本次选择的对象':recThreads().length+' 份记录 · 原始量 '+units(recSize()))+'</span><div class="cv-actions">'+actions+'</div></footer></div>';
 }
 function selectRecoveryModel(){
  const r=recovery;if(!r||r.mode!=='import'||r.step!==1)return;
  r.target=$('#cvImportRoot')?.value.trim()||r.target;
  window.modelPickerPrototype?.choose({title:'选择恢复后的子 Agent 模型',mode:'single',reasoning:true,fast:true,models:[r.modelChoice||modelRef(r.model||'newapi|gpt-5.4')],getCatalog:()=>modelList(true),onSelect:result=>{if(recovery!==r||!result.models[0])return;r.modelChoice=clone(result.models[0]);r.model=modelId(r.modelChoice);redrawRecovery();$('[data-cv="rec-model"]')?.focus({preventScroll:true});}});
 }
 function redrawRecovery(){window.syncPrototype?.render();}
 function runRecovery(retry=false){
  const r=recovery;if(!r)return;
  if(!retry&&r.mode==='export'){r.destination=$('#cvExportPath')?.value.trim()||r.destination;if(!r.destination){r.error='请选择保存位置。';redrawRecovery();return;}}
  if(!retry&&r.mode==='import'){r.target=$('#cvImportRoot')?.value.trim()||'';if(!r.target){r.error='请指定恢复后的项目工作目录。';redrawRecovery();return;}}
  const token=++operationToken;r.step=2;r.status='running';r.error='';r.progress=15;redrawRecovery();
  const advance=()=>{
   if(token!==operationToken||recovery!==r)return;r.progress=Math.min(100,r.progress+25);
   if(r.progress<100){redrawRecovery();setTimeout(advance,250);return;}
   if(recoveryScene==='space'){r.status='failed';r.error='目标位置空间不足。未生成完整包，源历史和配置保持。';}
   else if(recoveryScene==='unsupported'){r.status='failed';r.error='目标版本不支持包内原生记录格式。原包保留，没有写入会话。';}
   else if(r.mode==='export'){r.status='complete';exported={name:'Codex-重装恢复.zip',threads:clone(recThreads()),policies:Object.fromEntries(recKeys().map(k=>[k,clone(saved[k]||{})])),originalBytes:recSize()};}
   else {
    if(!r.mapping){
     const prefix='restored-'+(++restoreCounter)+'-';r.mapping=Object.fromEntries(r.package.threads.map((t,i)=>[t.id,prefix+i]));
     r.projectMapping=Object.fromEntries([...new Set(r.package.threads.map(t=>t.project).filter(Boolean))].map(id=>[id,prefix+id]));
     for(const [oldId,newId] of Object.entries(r.projectMapping))projects.push({id:newId,name:(projects.find(p=>p.id===oldId)?.name||'恢复项目')+' · 恢复',path:r.target});
     imported=r.package.threads.map(t=>({...t,id:r.mapping[t.id],project:r.projectMapping[t.project]||null,parent:r.mapping[t.parent]||undefined,archived:false}));threads.push(...imported);
    }
    if(!retry&&recoveryScene==='partial')r.status='partial';
    else {r.status='complete';if(r.restoreGlobal)saved.global=scopedPolicy('global',{...(saved.global||{}),...(r.package.policies.global||{})});
     for(const [oldId,newId] of Object.entries(r.projectMapping))saved['project:'+newId]=scopedPolicy('project:'+newId,{...(r.package.policies['project:'+oldId]||{}),...(recovery.includeConfig&&r.model?{'subagent.model':r.model,...(r.modelChoice?{'subagent.effort':r.modelChoice.reasoningEffort||'model-default','subagent.fast':r.modelChoice.fast===true}:{})}:{})});
     for(const t of r.package.threads){const p=scopedPolicy('thread:'+r.mapping[t.id],clone(r.package.policies['thread:'+t.id]||{}));if(Object.keys(p).length)saved['thread:'+r.mapping[t.id]]=p;}
    }
   }
   redrawRecovery();
  };setTimeout(advance,250);
 }
 function setScope(value){designScope=['models','conversations','extensions','capabilities'].includes(value)?value:'models';document.documentElement.dataset.designScope=designScope;$$('[data-conversation-feature]').forEach(el=>el.hidden=!['conversations','capabilities'].includes(designScope));const control=$('#prototypeDesignScope');if(control)control.value=designScope;
  if(!['conversations','capabilities'].includes(designScope)){operationToken++;recovery=null;if(document.documentElement.dataset.route==='conversations')setRoute(designScope==='extensions'?'extensions':'models');}
  window.syncPrototype?.refreshCatalog();window.prototypePanel?.refresh();
 }
 function scene(value){scenario=value;notice='';noticeScope=document.querySelector('#settings-dialog-policy.active [data-dp-panel="models"].active')?'global':selected;if(value==='archive'){tab='objects';selected='thread:config';detailOpen=true;const t=threads.find(t=>t.id==='config');t.archived=true;if(dirty(selected))orphanDrafts[selected]=clone(policy(selected));delete saved[selected];delete drafts[selected];archived=true;notice='已收到归档事件：清理专属覆盖，保留项目设置。';}
  else if(value==='unarchive'){tab='objects';selected='thread:config';detailOpen=true;threads.find(t=>t.id==='config').archived=false;archived=false;notice='取消归档后继承当前项目 / 全局；旧专属覆盖不会自动恢复。';}

  else if(value==='observed'){savedStates[noticeScope]='observed';notice='本场景模拟已观察到采用，不代表真实执行接入完成。';}
  render();
 }
 function restoreFocus(attribute,value){const el=$('['+attribute+'="'+value+'"]',editorHost());(el?.closest('.select')?.querySelector('.select-trigger')||el)?.focus({preventScroll:true});}
 document.addEventListener('click',e=>{
  const b=e.target.closest('[data-cv]');if(!b||b.disabled)return;const a=b.dataset.cv,id=b.dataset.id,previousScope=editingKey;editingKey=globalHost?.contains(b)?'global':null;noticeScope=currentKey();try{
  if(a==='global-policy'||a==='tab'&&id==='global'){openGlobal();return;}
  if(a==='tab'){tab='objects';setRoute('conversations');render();return;}
  else if(a==='select')choose(b.dataset.key);
  else if(a==='back'){detailOpen=false;render();objectPanel?.querySelector('.tn-tree')?.focus({preventScroll:true});}
  else if(a==='tree-clear'){resetObjectFilters();$('#cvObjectSearch',objectPanel)?.focus({preventScroll:true});}
  else if(a==='tree-filter'){archived=b.dataset.filter==='archived';onlyOwn=b.dataset.filter==='own';syncObjectNavigation();}
  else if(a==='tree-expand'||a==='tree-collapse'){a==='tree-expand'?objectNavigation?.expandAll():objectNavigation?.collapseAll();const menu=b.closest('details');if(menu)menu.open=false;}

  else if(a==='pick-model')openModelSelection(b.dataset.cvPicker);
  else if(a==='model-inherit'){const field=b.dataset.field,prefix=field.split('.')[0],fields=field==='subagent.roster'||prefix==='naming'?[field]:[field,prefix+'.effort',prefix+'.fast'];const draft=buffer(currentKey());for(const f of fields)delete draft[f];notice='该模型组合已恢复继承，保存后生效。';render();const scope=editorHost();$('[data-cv-picker="'+field+'"]',scope)?.focus({preventScroll:true});}
  else if(a==='save')commit([currentKey()]);
  else if(a==='rename')editTitle(false);
  else if(a==='regenerate-title')editTitle(true);
  else if(a==='field-inherit'){delete buffer(currentKey())[b.dataset.field];notice='该字段已恢复继承到草稿，保存后生效。';render();}
  else if(a==='inherit'){buffer(currentKey());drafts[currentKey()]={};notice='本层覆盖已清除到草稿；保存后继承上层。';render();}
  else if(a==='discard'){delete drafts[currentKey()];notice='已丢弃当前对象的草稿。';render();}
  else if(a==='refresh'){notice=scenario==='offline'?'连接仍不可达，保留上次列表和编辑。':'列表已刷新，当前编辑和分页保持。';render();}
  else if(a==='local'){source='local';render();}
  else if(a==='effective'){const fields=Object.keys(defaults).filter(f=>currentKey()==='global'||!f.startsWith('naming.'));openModal(object(currentKey()).name+' · 有效配置','<dl class="cv-summary">'+fields.map(f=>{const v=resolve(currentKey(),f);return '<div><dt>'+esc(f)+'</dt><dd>'+esc(display(f,v.value))+'<br><small>'+esc(v.from)+'</small></dd></div>';}).join('')+'</dl>','关闭',closeModal,{hideConfirm:true,cancelLabel:'关闭',hideNote:true});}
  else if(a==='models')setRoute('models');
  else if(a==='config-import')policyImport();
  else if(a==='config-target'){if(b.dataset.key==='global'){openGlobal();return;}tab='objects';selected=b.dataset.key;detailOpen=true;setRoute('conversations');render();}
  else if(a==='config-export'){const key=currentKey(),go=()=>window.syncPrototype?.open({tab:'file',scope:['dialog-policy:'+key],action:'export'});if(dirty(key))openModal('导出使用已保存配置','<p>当前对象还有草稿。可以先保存后导出，或只导出已保存内容。</p>','保存后导出',()=>{if(commit([key],true)){closeModal();go();}},{confirmKeepsOpen:true,onExtra:()=>{closeModal();go();},extraLabel:'仅导出已保存'});else go();}
  else if(a==='recovery-export')openRecovery('export',b.dataset.context||currentKey());else if(a==='recovery-import')openRecovery('import');
  else if(a==='scene')scene(id);
  else if(a==='rec-close'){operationToken++;recovery=null;redrawRecovery();}
  else if(a==='rec-fold'){recovery.folds.has(id)?recovery.folds.delete(id):recovery.folds.add(id);redrawRecovery();}
  else if(a==='rec-next'){if(recovery.mode==='import'&&!recovery.package)return;recovery.step=1;recovery.error='';redrawRecovery();}
  else if(a==='rec-back'){operationToken++;recovery.step=Math.max(0,recovery.step-1);recovery.error='';redrawRecovery();}
  else if(a==='rec-model')selectRecoveryModel();
  else if(a==='rec-file'){recovery.package=exported||{name:'Codex-重装前示例.zip',threads:clone(seedThreads.filter(t=>['config','review'].includes(t.id))),policies:clone(initial)};redrawRecovery();}
  else if(a==='rec-save-drafts'){if(commit(recKeys().filter(dirty),true)){recovery.reviewed=true;redrawRecovery();}else recovery.error='相关草稿未保存，已保留输入。请返回对话页处理或仅导出已保存配置。';redrawRecovery();}
  else if(a==='rec-run')runRecovery();else if(a==='rec-retry')runRecovery(true);
  else if(a==='rec-cancel'){operationToken++;recovery.status='cancelled';redrawRecovery();}
  else if(a==='rec-import'){recovery=null;openRecovery('import','global');recovery.package=exported;redrawRecovery();}
  else if(a==='rec-view'){recovery=null;tab='objects';archived=false;onlyOwn=false;query='';objectFilterSignature='';selected='thread:'+(imported[0]?.id||'config');detailOpen=true;notice='恢复后的对话已关联配置。此处未开始执行。';setRoute('conversations');render();objectNavigation?.reveal();}
 }finally{editingKey=previousScope;}
 });
 document.addEventListener('change',e=>{
  const el=e.target,previousScope=editingKey;editingKey=globalHost?.contains(el)?'global':null;noticeScope=currentKey();try{
  if(el.id==='prototypeDesignScope'){setScope(el.value);return;}
  if(el.dataset.cvMode){const key=currentKey(),field=el.dataset.cvMode;const p=buffer(key);if(el.value==='inherit')delete p[field];else p[field]=resolve(key,field).value;render();restoreFocus('data-cv-mode',field);}
  else if(el.dataset.cvField){const field=el.dataset.cvField;if(el.value==='__inherit')delete buffer(currentKey())[field];else buffer(currentKey())[field]=el.multiple?[...el.selectedOptions].map(o=>o.value).join('\n'):el.type==='number'?Number(el.value):el.value;if(el.tagName==='TEXTAREA'){updateDirty();return;}render();restoreFocus('data-cv-field',field);}
  else if(el.id==='cvSource'){source=el.value;render();}
  else if(el.dataset.cvThreadPick){el.checked?recovery.selected.add(el.dataset.cvThreadPick):recovery.selected.delete(el.dataset.cvThreadPick);redrawRecovery();}
  else if(el.dataset.cvProjectPick){for(const t of threads.filter(t=>(t.project||'unassigned')===el.dataset.cvProjectPick&&!t.parent&&(!t.archived||recovery.withArchived)))el.checked?recovery.selected.add(t.id):recovery.selected.delete(t.id);redrawRecovery();}
  else if(el.dataset.cvRec){recovery[el.dataset.cvRec]=el.checked;redrawRecovery();}
  else if(el.id==='cvRecoveryScene')recoveryScene=el.value;
 }finally{editingKey=previousScope;}
 });
 function onEditorInput(e){const el=e.target,previousScope=editingKey;editingKey=globalHost?.contains(el)?'global':null;noticeScope=currentKey();try{if(el.id==='cvObjectSearch'){query=el.value;objectNavigation?.setQuery(query);}else if(el.dataset.cvField){if(el.tagName==='SELECT')return;buffer(currentKey())[el.dataset.cvField]=el.type==='number'?Number(el.value):el.value;updateDirty();}}finally{editingKey=previousScope;}}
 host.addEventListener('input',onEditorInput);globalHost?.addEventListener('input',onEditorInput);
 document.addEventListener('input',e=>{if(!recovery)return;if(e.target.id==='cvExportPath')recovery.destination=e.target.value;if(e.target.id==='cvImportRoot')recovery.target=e.target.value;});

 // 标题来自对应协作包；版本未定不借用0.2.0，选项ID及design查询保持兼容。
 const prototypeVersions=[{id:'models',version:'0.2.0',name:'模型配置管理与同步'},{id:'conversations',version:null,name:'Codex 对话配置管理'},{id:'extensions',version:null,name:'多 Agent 平台扩展管理'},{id:'capabilities',version:null,name:'渠道特殊能力与代理兼容'}];
 const scopeControl=document.createElement('div');scopeControl.className='cv-scope-control';scopeControl.id='prototypeVersionControl';scopeControl.innerHTML='<label>原型版本<select id="prototypeDesignScope" aria-label="原型版本">'+opts(prototypeVersions.map(item=>[item.id,(item.version||'待定版本')+' · '+item.name]),designScope)+'</select></label>';
 const rail=$('.prototype-side .proto-rail');rail.querySelector('.pctl-bar').before(scopeControl);
 const demo=document.createElement('div');demo.className='proto-card';demo.id='conversationDemoCard';demo.dataset.protoContexts='conversations sync settings:dialog-policy';demo.dataset.policyParts='models';demo.innerHTML='<div class="proto-note-title">对话探索 · 版本未定</div><div class="pctl-grid">'+[['normal','正常'],['offline','离线'],['empty','空列表'],['no-model','模型不可用'],['unsupported','局部未接通'],['save-fail','保存失败'],['archive','对话归档'],['unarchive','取消归档'],['observed','观察到采用'],['sources','多个来源']].map(([id,label])=>btn(label,'scene','data-id="'+id+'"')).join('')+'</div><label class="proto-mock-field"><span>恢复结果场景</span><select id="cvRecoveryScene">'+opts([['normal','正常完成'],['space','空间不足'],['partial','恢复部分完成'],['unsupported','原生格式不支持'],['conflict','目标已有同名项目']],recoveryScene)+'</select></label><p class="pctl-hint">虚构项目、对话及大小。只改变内存；不读写真实文件、不调用模型或 WebDAV。子 Agent 采用和完整原生恢复仍待生产接入。</p>';($('#protoStdCards .proto-group-body')||$('#protoStdCards')).appendChild(demo);
 const updateDemo=()=>{demo.hidden=!['conversations','capabilities'].includes(designScope)||!['conversations','sync','settings'].includes(document.documentElement.dataset.route);};
 new MutationObserver(()=>{if(document.documentElement.dataset.route==='conversations'){if(!['conversations','capabilities'].includes(designScope))setScope('conversations');render();}updateDemo();}).observe(document.documentElement,{attributes:true,attributeFilter:['data-route']});
 window.conversationPrototype={state:()=>clone({designScope,tab,selected,saved,drafts,revisions,scenario,source,imported,recovery:recovery?{mode:recovery.mode,step:recovery.step,status:recovery.status,selected:[...recovery.selected],progress:recovery.progress}:null}),scope:()=>designScope,setScope,scene,render,renderGlobal,openGlobal,resolve,openRecovery,navigationState:()=>objectNavigation?.state(),
  syncObjects:()=>designScope!=='conversations'?[]:[...['global',...projects.map(p=>'project:'+p.id),...threads.filter(t=>!t.archived&&!t.parent).map(t=>'thread:'+t.id)].map(key=>({id:'dialog-policy:'+key,group:'dialog-policies',name:object(key).name+' · '+object(key).kind+'配置',summary:Object.keys(saved[key]||{}).length+' 个本层覆盖 · 默认不同步',source:'Desktop 对话策略',fields:key==='global'?'全局命名模型 / 搜索 / 子 Agent；不含历史与执行状态':'搜索 / 子 Agent 作用域覆盖；不含命名配置、历史与执行状态',files:'分层对话策略文件',defaultExcluded:true,payload:clone(saved[key]||{}),deps:[]})),...threads.filter(t=>!t.archived&&!t.parent).map(t=>({id:'dialog-history:'+t.id,group:'dialog-history',name:t.name,summary:units(t.bytes)+' · 本地重装恢复',source:'Codex 原生历史',fields:'原生历史与关联关系；不含整个项目源码',files:'原生恢复载荷',defaultExcluded:true,webdavUnsupported:true,history:true,deps:[]}))],
  recoveryHTML:()=>recoveryPanel(),renderRecovery:(panel,activeTab)=>{updateDemo();if(designScope!=='conversations'||activeTab!=='file'||!recovery)return false;panel.innerHTML=recoveryPanel();$$('[data-partial]',panel).forEach(el=>el.indeterminate=true);window.prototypeSelect?.enhance();return true;},
  recoveryEntry:()=>designScope==='conversations'?'<div class="cv-sync-entry"><div><strong>重装恢复</strong><p class="cv-muted">所选项目、对话与配置，一次性保存到本地。</p></div><div class="cv-actions">'+btn('导入对话配置','config-import')+btn('导入恢复包','recovery-import')+btn('导出重装恢复包','recovery-export','data-context="global"')+'</div></div>':''};
 setScope(designScope);render();updateDemo();
})();
