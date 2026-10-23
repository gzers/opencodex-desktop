/* COL-LOCAL-20261004-01: 内存 mock；不连接 API、CLI、WebDAV 或运行文件。 */
(() => {
 'use strict';
 const $=(q,r=document)=>r.querySelector(q), $$=(q,r=document)=>[...r.querySelectorAll(q)];
 const esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 const clone=v=>v===undefined?undefined:JSON.parse(JSON.stringify(v));
 const updates=window.TemplateUpdates,summaryEngine=window.TemplateSummary,seedLibrary=window.ModelTemplateSeeds||[];
 const levels=['none','minimal','low','medium','high','xhigh','max'];
 const labels={none:'无',minimal:'最低',low:'低',medium:'中',high:'高',xhigh:'极高',max:'最高'};
 const protocols=['Responses','Chat','Anthropic','Google','Ollama'];
 const adapterCatalog=['codebuddy','command-code','openai-chat','ollama-native','anthropic','openai-responses','google','kiro','azure','azure-openai','cursor','devin','mimo-free','qoder','claude-cli'];
 const adapters={Responses:'openai-responses',Chat:'openai-chat',Anthropic:'anthropic',Google:'google',Ollama:'ollama-native'};
 const legacyFamilies={GPT:'gpt',Claude:'claude',Gemini:'gemini',Qwen:'qwen',DeepSeek:'deepseek',Grok:'grok'};
 const families=()=>templates.filter(t=>t.versions.length).map(t=>[t.id,t.name]);
 const protocolOptions=v=>opts(protocols.map(x=>[x,adapters[x]]),v)+'<optgroup label="专用适配器 · 需专用连接配置">'+adapterCatalog.filter(x=>!Object.values(adapters).includes(x)).map(x=>'<option disabled>'+x+' · 专用连接待接入</option>').join('')+'</optgroup>';
 const button=(name,action,attrs='',primary=false,danger=false)=>'<button class="btn '+(danger?'danger':primary?'primary':'ghost')+'" data-mp="'+action+'" '+attrs+'>'+name+'</button>';
 const icons={copy:'<rect x="8" y="8" width="12" height="13" rx="2"/><path d="M16 8V3H3v13h5"/>',preview:'<path d="M2 12s3.5-7 10-7 10 7 10 7-3.5 7-10 7-10-7-10-7Z"/><circle cx="12" cy="12" r="3"/>',compare:'<path d="M12 3v18M8 6H3v12h5M16 6h5v12h-5"/>',more:'<circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/>',collapse:'<path d="m6 9 6 6 6-6"/>',maximize:'<path d="M8 3H3v5m13-5h5v5M3 16v5h5m13-5v5h-5"/>',restore:'<path d="M9 3v6H3m18 0h-6V3M3 15h6v6m6 0v-6h6"/>',send:'<path d="M12 20V4m-6 6 6-6 6 6"/>',export:'<path d="M12 16V3m-5 5 5-5 5 5M4 14v6h16v-6"/>'};
 Object.assign(icons,{menu:'<path d="M4 6h16M4 12h16M4 18h16"/>',back:'<path d="m14 6-6 6 6 6"/>',agent:'<path d="m12 3 2.5 6.5L21 12l-6.5 2.5L12 21l-2.5-6.5L3 12l6.5-2.5L12 3Z"/>',settings:'<path d="M4 7h16M4 17h16"/><circle cx="9" cy="7" r="3"/><circle cx="15" cy="17" r="3"/>',edit:'<path d="m15 4 5 5M4 20l4-1L20 7l-5-5L3 14l-1 5Z"/>'});
 Object.assign(icons,{refresh:'<path d="M20 7v5h-5M4 17v-5h5M6 6a8 8 0 0 1 14 6M18 18a8 8 0 0 1-14-6"/>',save:'<path d="M5 3h12l4 4v14H3V3h2ZM7 3v6h10V3M7 21v-7h10v7"/>',draft:'<path d="M7 3h10l4 4v14H3V3h4ZM7 12h10M7 16h7"/>',reset:'<path d="M3 4v6h6M3 10a9 9 0 1 1 2 8"/>',budget:'<path d="M5 21V11h4v10M11 21V3h4v18M17 21V7h4v14"/>'});
 const icon=name=>'<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" aria-hidden="true">'+icons[name]+'</svg>';
 const iconButton=(name,action,mark,attrs='')=>'<button class="icon-btn" data-mp="'+action+'" title="'+esc(name)+'" aria-label="'+esc(name)+'" '+attrs+'>'+icon(mark)+'</button>';
 function templateActions(inNav=false){
  const actions=[['Agent 自动填充','generate','agent'],['外部任务','external-agent','export'],['复制模版','copy-template','copy']];
  return actions.map(([name,action,mark])=>{
   const disabled=action==='copy-template'&&!!templateScratch,title=disabled?'先保存或放弃已有的新模版，再复制':action==='copy-template'?'以当前内容新建独立模版':name;
   const attrs=action==='generate'?'id="mpTemplateAgentAction" aria-controls="mpAssistantPanel" aria-expanded="'+assistantOpen+'"':'';
   return inNav?'<button class="btn ghost mp-nav-action" data-mp="'+action+'" title="'+title+'" aria-label="'+name+'" '+attrs+' '+(disabled?'disabled':'')+'>'+icon(mark)+'<span class="mp-nav-action-label">'+name+'</span></button>':disabled?'':button(name,action,attrs);
  }).join('');
 }
 const opts=(items,value)=>items.map(x=>{const [v,n]=Array.isArray(x)?x:[x,x];return '<option value="'+esc(v)+'" '+(String(v)===String(value)?'selected':'')+'>'+esc(n)+'</option>';}).join('');
 const names=['gpt-5.4','gpt-5.4-mini','gpt-5.3-codex','gpt-5.2','gpt-5.2-codex','gpt-5-mini','claude-opus-4-6','claude-sonnet-4-6','claude-haiku-4-5','gemini-3.1-pro-preview','gemini-3-flash-preview','qwen3-coder','qwen3-max','deepseek-v3.2','deepseek-r1','grok-4','grok-4-fast'];
 function makeProvider(id,name,total,n,protocol='Responses'){
  const models=Array.from({length:total},(_,i)=>{
   const k=i%names.length,modelId=names[k]+(i>=names.length?'--demo-'+Math.floor(i/names.length):'');
   const family=k<6?'GPT':k<9?'Claude':k<11?'Gemini':k<13?'Qwen':k<15?'DeepSeek':'Grok';
   return {key:id+'::'+i,id:modelId,family,selected:i<n,official:{context:family==='GPT'?128000:null,modalities:['text','image'],efforts:['low','medium','high','xhigh'],default:'medium',fast:k<4?'supported':'unknown'},overrides:{}};
  });
  return {id,name,protocol,enabled:true,mode:id==='native'?'native':'whitelist',models,defaults:{},familyOverrides:{},refs:{gpt:'gpt:1'},fastEnabled:'inherit',refresh:'idle',refreshToken:0,updated:'10:24',...(id==='native'?{}:{connection:{auth:'apikey',keyConfigured:true},nodes:[{id:'primary',name:'默认节点',url:'https://'+id+'.example.com/v1'}],activeNodeId:'primary'})};
 }
 const initial={providers:[makeProvider('newapi','NewApi',30,17),makeProvider('a6api','A6api',104,32),makeProvider('aimami','Aimami',12,8),makeProvider('native','官方 / 原生',6,6,'原生')],fastMode:'on',fastRows:true,globalRefs:{gpt:'gpt:1'},hidden:2};
 initial.providers[0].nodes.push({id:'backup',name:'备用节点',url:'https://backup.newapi.example.com/v1'});
 initial.providers[3].models.forEach(m=>m.official.fast='unsupported');
 initial.providers[1].availability={kind:'no-credit',source:'manual'};
 // 增加停用示例渠道供列表滚动评审，沿用原行组件且不增加激活预算。
 for(let i=1;i<=16;i++){
  const id='scroll-demo-'+String(i).padStart(2,'0'),p=makeProvider(id,'示例渠道 '+String(i).padStart(2,'0'),8+i%5,2+i%3);
  p.enabled=false;p.refs={};p.nodes[0].name='主节点';
  p.nodes.push({id:'backup',name:'备用节点',url:'https://backup-'+id+'.example.com/v1'});
  if(i%3===0)p.nodes.push({id:'regional',name:'国内节点',url:'https://regional-'+id+'.example.com/v1'});
  initial.providers.push(p);
 }
 const templates=[{id:'gpt',name:'GPT',family:'GPT',protocol:'Responses',main:1,versions:[
  {n:1,status:'人工',date:'10-03 16:20',rules:{protocol:'Responses',efforts:['low','medium','high','xhigh'],default:'medium',fast:'supported'}},
  {n:2,status:'历史',date:'10-03 18:42',rules:{protocol:'Responses',efforts:['low','medium','high'],default:'high',fast:'supported'}},
  {n:3,status:'候选',date:'10-04 10:30',rules:{protocol:'Responses',efforts:['low','medium','high','xhigh'],default:'high',fast:'supported'}}
 ]},{id:'claude',name:'Claude',family:'Claude',protocol:'Anthropic',main:1,versions:[{n:1,status:'人工',date:'10-03 16:30',rules:{protocol:'Anthropic'}}]},
 {id:'gemini',name:'Gemini',family:'Gemini',protocol:'Google',main:1,versions:[{n:1,status:'人工',date:'10-03 16:40',rules:{protocol:'Google'}}]}];
 templates.forEach(t=>{t.versions.forEach(v=>{v.match={...defaultMatch(),include:[{type:'prefix',value:t.id+'-'}]};v.description='示例预设，可自行修改';v.source='演示 fixture / 未核对';});delete t.family;delete t.protocol;});
 for(const [id,name,prefix] of [['qwen','Qwen','qwen'],['deepseek','DeepSeek','deepseek'],['grok','Grok','grok']])templates.push({id,name,main:1,versions:[{n:1,status:'预设',date:'10-04',match:{...defaultMatch(),include:[{type:'prefix',value:prefix}]},description:'示例预设，能力继承',rules:{}}]});
 for(const seed of seedLibrary)if(!templates.some(t=>t.id===seed.id))templates.push({id:seed.id,name:seed.name,main:1,versions:[{n:1,date:'10-13 09:00',status:'内置初始',description:'初始规则保持继承，可检查型号资料后另存新版本',match:{...defaultMatch(),include:[{type:'prefix',value:seed.prefix}],exclude:[{type:'glob',value:'*embedding*'},{type:'glob',value:'*image*'}]},rules:{},source:'公开资料登记 · 2026-10-13；具体字段按型号核对'}]});
 templates.forEach(t=>t.versions.forEach((v,index)=>{Object.assign(v,updates.stamp(templates.flatMap(x=>x.versions),Date.parse('2026-10-'+(v.date?.slice(3,5)||'04')+'T'+(v.date?.includes(' ')?v.date.split(' ')[1]+':00':'09:00:00')+'+08:00')),{nameAtSave:t.name,origin:index===0?'软件内置':'人工',basedOn:index?t.versions[index-1].revisionId:null});const base=index?t.versions[index-1]:null;v.summaryFacts=summaryEngine.summarize(base?templateVersionSnapshot(t,base):null,templateVersionSnapshot(t,v),{kind:base?'revision':'baseline',templateId:t.id,revisionId:base?.revisionId||null});v.summary=v.summaryFacts.text;}));
 // 模版来源与版本来源分开；系统基线不因换默认或删除其他版本而解除保护。
 templates.forEach(t=>{t.originType='builtin';t.versions[0].systemBaseline=true;t.versions[0].systemCurrent=true;t.nextRevision=Math.max(...t.versions.map(v=>v.n))+1;for(const v of t.versions)if(v.systemBaseline){v.origin='软件内置';}});
 const versionDropdownRecent=5;
 let draft=clone(initial),saved=clone(initial),pending=false,client='目录已核对 · 客户端待验证';
 let channelDetail=false,templateDetail=false,templateSearch='',correctionDraft='',generationTarget='',generationSources='provided';
 let agent={providerId:'',modelId:'',effort:'inherit',fast:false};
 let tab='channels',current='newapi',search='',family='',onlySelected=false,channelSearch='',scope=new Set();
 let templateScratch=null,templateSelection=new Set(),templateListVersions={},templateHistoryView=null,externalTask=null,externalBatch=null,externalPrompt='';
 const externalTasks=new Map();
 const externalSubmissionContract='prototype-template-submission/1';
 let templateId='gpt',versionN=3,buffers={},conversation=[],proposal=null,generationToken=0,editorDigest=0,generating=false;
 let editorSection='basic',editorNavCollapsed=false,editorViewId=null,assistantOpen=false,assistantMaximized=false,assistantInputs={},assistantLock=true,generationOwner='';
 let assistantTask='fill',updateScope='current',updateRun=null,updateSerial=0,updateBatches=[],confirmedConfigurations=[],restoration=null;
 let operation=null,operationId=0,scenario='normal',incoming=null;
 let channelWizard=null,wizardToken=0;
 let channelSection='connection',channelNavCollapsed=false,channelViewId=null,channelConnections={};
 let nodeSpeedResults={},nodeSpeedRuns={},nodeSpeedSerial=0,nodeSpeedScene='normal',nodeAuthRevision={};
 let nodePicker=null;
 const records=[{id:0,time:'10-03 16:20',status:'成功',policy:clone(initial),before:clone(initial),client:'用户已确认（示例）'}];
 const host=$('#mpContent'), provider=()=>draft.providers.find(p=>p.id===current), template=()=>templateScratch?.id===templateId?templateScratch:templates.find(t=>t.id===templateId);
 const version=(id,n)=>templates.find(t=>t.id===id)?.versions.find(v=>v.n===Number(n));
 const versionText=(id,n)=>version(id,n)?.versionLabel||'尚未保存';
 const refText=ref=>{const [id,n]=String(ref||'').split(':');return ref?versionText(id,n):'未引用';};
 function templateUses(policy,id,n){const ref=id+':'+n,rows=[];if(policy.globalRefs?.[id]===ref)rows.push('全局默认');for(const p of policy.providers.filter(p=>p.id!=='native')){if(p.refs?.[id]===ref)rows.push(p.name);if(p.models.some(m=>m.templateRef===ref))rows.push(p.name+' · 单模型');}return rows;}
 function versionState(id,n){const used=templateUses(saved,id,n).length,confirmed=confirmedConfigurations.filter(c=>c.templateId===id&&c.n===n);return used?(confirmed.some(c=>JSON.stringify(c.policy)===JSON.stringify(saved))?'已采用 · 已确认可用':'已采用 · 待确认'):confirmed.length?'曾确认可用':'尚未采用';}
 const templateStates={
  new:{label:'新模版 · 未保存',tone:'warning'},
  clean:{label:'未修改',tone:'neutral'},
  dirty:{label:'未保存修改',tone:'warning'},
  invalid:{label:'配置待修正',tone:'danger'},
  unadopted:{label:'已保存 · 未采用',tone:'info'},
  pending:{label:'已采用 · 待确认',tone:'warning'},
  confirmed:{label:'已采用 · 已确认可用',tone:'success'},
  previous:{label:'曾确认可用',tone:'neutral'}
 };
 function templateHeadingState(){
  const b=buffer(),t=template(),v=version(templateId,versionN);let id,context='';
  if(b){
   const error=b.valid===false?'检查配置字段':(!templateScratch||b.validationRequested)?templateError(b):'';
   id=error?'invalid':templateScratch?'new':templateSaveChanges(b).length?'dirty':'clean';
   context=error||(!templateScratch?'基于 '+versionText(templateId,b.base):'保存后生成首个版本');
  }else{
   id=({'尚未采用':'unadopted','已采用 · 待确认':'pending','已采用 · 已确认可用':'confirmed','曾确认可用':'previous'})[versionState(templateId,versionN)];
   context=v?.summary||v?.status||'';
  }
  const usageId=v?({'尚未采用':'unadopted','已采用 · 待确认':'pending','已采用 · 已确认可用':'confirmed','曾确认可用':'previous'})[versionState(templateId,v.n)]:null;
  return {id,...templateStates[id],context,usage:usageId?{id:usageId,...templateStates[usageId]}:null};
 }
 function templateHeadingStatusHTML(){
  const state=templateHeadingState();
  return '<span class="mp-template-state '+state.tone+'" data-template-state="'+state.id+'">'+esc(state.label)+'</span>'+(state.id==='clean'&&state.usage?'<span class="mp-template-state '+state.usage.tone+'" data-template-usage="'+state.usage.id+'">'+esc(state.usage.label)+'</span>':'')+'<span class="mp-template-state-context" title="'+esc(state.context)+'">'+esc(state.context)+'</span>';
 }
 function templateStatusAnnotation(){return mpAnn('04','模版的 8 种主要状态','编辑状态：新模版 · 未保存（橙色）；未修改（中性）；未保存修改（橙色）；配置待修正（红色）。<br>已保存版本：已保存 · 未采用（蓝色）；已采用 · 待确认（橙色）；已采用 · 已确认可用（绿色）；曾确认可用（中性）。<br>模版统一使用编辑页，未修改时同时显示该保存版本的采用状态。保存生成新版本，不原地改历史规则；没有变化不新增版本。旧版本在同一表单查看和编辑，有修改时先保存或返回放弃，再切换 / 采用。已采用与确认以保存的渠道配置和用户实际确认记录为准，保存成功不代表已经验证可靠。默认版本只决定以后采用时的预选；内置基线永久保留，来源标记与默认/采用状态分开，不提供归档或手动保留；批量删除需勾选并确认。');}
 function policyReferencesVersion(policy,ref){return !!policy&&((policy.globalRefs&&Object.values(policy.globalRefs).includes(ref))||(policy.providers||[]).some(p=>Object.values(p.refs||{}).includes(ref)||(p.models||[]).some(m=>m.templateRef===ref)));}
 function versionProtectionReasons(id,n,includeDefault=true){
  const t=templates.find(t=>t.id===id),v=t?.versions.find(v=>v.n===Number(n));if(!v)return ['版本不存在'];const ref=id+':'+n,reasons=[];
  if(v.systemBaseline)reasons.push('系统初始内置基线');else if(v.systemCurrent)reasons.push('当前内置基线');
  if(includeDefault&&t.main===Number(n))reasons.push('默认版本');
  if(policyReferencesVersion(saved,ref))reasons.push('已保存配置引用');if(policyReferencesVersion(draft,ref))reasons.push('草稿引用');
  if(records.some(r=>policyReferencesVersion(r.before,ref)||policyReferencesVersion(r.policy,ref))||confirmedConfigurations.some(c=>policyReferencesVersion(c.policy,ref)))reasons.push('恢复记录依赖');
  if(policyReferencesVersion(incoming?.policy,ref))reasons.push('待审配置依赖');
  if(buffers[id]?.base===Number(n)&&templateEditorChanged(id))reasons.push('未保存编辑基准');
  if(generating&&generationOwner===id&&buffers[id]?.base===Number(n))reasons.push('生成任务基准');
  if(['running','ready'].includes(updateRun?.status)&&updateRun.items.some(i=>i.id===id&&i.n===Number(n)))reasons.push('更新任务依赖');
  // 已复制的外部说明不等于软件内正在执行的任务；旧基准删除/变更由提交时拒绝，不长期锁定版本。
  return reasons;
 }
 function versionProtected(id,n){return versionProtectionReasons(id,n).length>0;}
 function nextVersionN(t){return Math.max(t.nextRevision||1,1,...t.versions.map(v=>v.n+1));}
 function versionOptionLabel(t,v,compact=false){if(compact&&v.systemBaseline)return v.versionLabel+' · 内置基线';return v.versionLabel+(v.n===t.main?' · 默认':'')+(v.systemBaseline?' · 内置基线':v.systemCurrent?' · 当前内置':'');}
 function compactVersions(t,current){const selected=new Set(t.versions.slice(-versionDropdownRecent).map(v=>v.n));if(current)selected.add(Number(current));if(t.main)selected.add(t.main);return t.versions.slice().reverse().filter(v=>selected.has(v.n));}
 function compactVersionOptions(t,current){return opts(compactVersions(t,current).map(v=>[v.n,versionOptionLabel(t,v,true)]),current)+'<option value="history">版本管理…</option>';}
 function templateOriginHTML(t){return '<span class="mp-template-origin '+(t.originType==='builtin'?'builtin':'user')+'" data-template-origin="'+(t.originType==='builtin'?'builtin':'user')+'">'+(t.originType==='builtin'?'内置':'自建')+'</span>';}
 function versionHTML(v){if(!v)return '<span class="mp-muted">未设定</span>';return '<span class="mp-version-label" title="'+esc(v.versionLabel)+'"><time datetime="'+esc(v.savedAt)+'">'+esc(v.versionLabel.split(' · ')[0])+'</time><code>'+esc(v.shortId)+'</code></span>';}
 const buffer=()=>buffers[templateId], dirty=()=>JSON.stringify(draft)!==JSON.stringify(saved), busy=()=>operation?.status==='running';
 function defaultMatch(){return {include:[],exclude:[],caseSensitive:false,channelMode:'all',channels:[],priority:0};}
 const channelMode=match=>match.channelMode||(match.channels?.length?'whitelist':'all');
 function variantFor(content,modelId){const normalize=x=>content.match?.caseSensitive?x:x.toLowerCase();return (content.variants||[]).filter(v=>v.suffix&&normalize(modelId).endsWith(normalize(v.suffix))).sort((a,b)=>b.suffix.length-a.suffix.length)[0];}
 function templateLayers(ref,m){const [id,n]=String(ref||'').split(':'),v=version(id,n),variant=v&&variantFor(v,m.id);return [['模版 '+refText(ref),v?.rules||{}],...(variant?[['版本或后缀 '+variant.name+' · 后缀 '+variant.suffix,variant.rules]]:[])];}
 const canonicalKeys=obj=>Object.fromEntries(Object.entries(obj||{}).map(([k,v])=>[legacyFamilies[k]||k,clone(v)]));
 function matchLines(rows=[]){return rows.map(r=>r.type+':'+r.value).join('\n');}
 function parseMatchLines(text){return text.split(/\r?\n/).map(x=>x.trim()).filter(Boolean).map(x=>{const parts=x.match(/^(exact|prefix|glob):(.*)$/);return parts?{type:parts[1],value:parts[2]}:{type:'glob',value:x};});}
 function matchPattern(rule,id,sensitive){const normalize=x=>sensitive?x:x.toLowerCase(),v=normalize(rule.value),text=normalize(id);if(rule.type==='exact')return text===v;if(rule.type==='prefix')return text.startsWith(v);if(rule.type==='glob'){const parts=v.split('*').map(x=>x.split('?').map(y=>y.replace(/[.*+?^{}()|[\]\\]/g,'\\$&').replaceAll('$','\\$')).join('.'));return new RegExp('^'+parts.join('.*')+'$').test(text);}return false;}
 function matchResult(match,m,p){const mode=channelMode(match),listed=match.channels?.includes(p.id);if(mode==='whitelist'&&!listed||mode==='blacklist'&&listed)return 'scope';if(!match.include.some(r=>matchPattern(r,m.id,match.caseSensitive)))return 'unmatched';return match.exclude.some(r=>matchPattern(r,m.id,match.caseSensitive))?'excluded':'matched';}
 function templateMatches(t,m,p,v=null){return !!t&&matchResult((v||version(t.id,t.main)||t.versions[0])?.match||defaultMatch(),m,p)==='matched';}
 function templateResolution(p,m,policy=draft){
  if(m.templateRef){const [id,n]=m.templateRef.split(':');return version(id,n)?{ref:m.templateRef,id,error:''}:{ref:m.templateRef,id,error:'精确模版版本不存在'};}
  const refs={...canonicalKeys(policy.globalRefs),...canonicalKeys(p.refs)},hits=[];
  for(const [tid,ref] of Object.entries(refs)){const [id,n]=String(ref).split(':'),v=version(id,n);if(!v)return {ref,id,error:'已采用模版版本不存在：'+ref};if(id!==tid)return {ref,id,error:'模版引用 ID 不一致'};const t=templates.find(t=>t.id===id);if(templateMatches(t,m,p,v))hits.push({ref,id,priority:v.match?.priority||0});}
  hits.sort((a,b)=>b.priority-a.priority);if(hits.length>1&&hits[0].priority===hits[1].priority)return {ref:null,id:null,error:'模版匹配冲突：'+hits.filter(x=>x.priority===hits[0].priority).map(x=>x.ref).join(' / ')};return {...(hits[0]||{ref:null,id:null}),error:''};
 }
 function templateLabel(p,m){const names=templates.filter(t=>templateMatches(t,m,p)).map(t=>t.name);return names.join(' / ')||'未匹配模版';}
 function familyOverride(p,m,policy=draft){const chosen=templateResolution(p,m,policy);return canonicalKeys(p.familyOverrides)[chosen.id]||{};}
 function templateVersionSnapshot(t,v){return summaryEngine.snapshot({name:v?.nameAtSave||t.name,description:v?.description||'',match:v?.match||defaultMatch(),rules:v?.rules||{},variants:v?.variants||[]});}
 function storedVersionSummary(v){return v?.summaryFacts?.text||'变更依据缺失';}
 function storedVersionSummaryFull(v){return v?.summaryFacts?.fullText||'变更依据缺失';}
 function versionSummaryAnnotation(){return mpAnn('10','版本摘要的生成依据','名称下面的摘要描述右侧所选保存版本的真实变化，不用于判断采用、默认版本、来源或可靠性。<br>人工保存、Agent更新和外部结果均以来源保存版本的完整快照与最终保存内容比较，计入人工未保存修改及实际接受的建议；不按版本序号或执行方式预填“新增候选”等文案。<br>按固定顺序列名称/说明、匹配、通用配置与型号规则的新增/删除/字段/顺序变化；字段采用设置、调整、恢复继承，集合只改排列不算字段变化。列表最多三项，更多项显示“另N项”，完整摘要可悬停及在来源/历史中查阅。<br>内置首版按可信来源显示内置基线；无来源的用户首版显示首次创建模版，复制按复制源快照列复制新建及实际修改。无内容变化不新增版本，来源缺失且没有已保存比较记录时明确显示变更依据缺失，不猜测。摘要在保存时生成并与来源版本身份/字段事实一起保留，后续采用、换默认、归档不会改写摘要。');}
 function templateContent(){const t=template(),v=version(t.id,versionN);return clone(buffer()||{name:t.name,description:v?.description||'',match:v?.match||defaultMatch(),rules:v?.rules||{},...(v?.variants?.length?{variants:v.variants}:{})});}
 const editorSections=[['basic','基本信息'],['match','匹配范围'],['common','族通用配置'],['variants','版本或后缀'],['sources','来源与版本']];
 function templateForm(b){
  const mode=channelMode(b.match),rows=b.variants||[];
  const content={};
  content.basic='<h3>基本信息</h3><p class="mp-muted">一个模版就是一个自定义模型族。名称可在顶部直接修改。</p><div class="mp-template-form"><label>说明<input id="mpTemplateDescription" data-template-meta="description" class="input" value="'+esc(b.description)+'"></label></div>';
  content.match='<div class="mp-section-heading"><h3>匹配范围</h3>'+button('匹配预览','match-preview')+'</div><div class="mp-template-scope"><label class="mp-label">渠道范围<select id="mpMatchChannelMode" class="mp-select" data-template-match="channelMode">'+opts([['all','全部渠道'],['whitelist','白名单 · 仅所选渠道'],['blacklist','黑名单 · 排除所选渠道']],mode)+'</select></label>'+(mode==='all'?'<p class="mp-muted">所有现有和以后新增渠道都可参与匹配。</p>':'<div class="mp-checks" role="group" aria-label="'+(mode==='whitelist'?'允许':'排除')+'的渠道">'+draft.providers.map(p=>'<label><input id="mpMatchChannel-'+esc(p.id)+'" type="checkbox" data-template-channel="'+esc(p.id)+'" '+(b.match.channels.includes(p.id)?'checked':'')+'>'+esc(p.name)+'</label>').join('')+'</div><p class="mp-muted">'+(mode==='whitelist'?'未选渠道不参与；空白名单不匹配任何渠道。':'所选渠道被排除；空黑名单允许全部渠道，包括以后新增渠道。')+'</p>')+'</div><div class="mp-template-form"><label>包含规则<textarea id="mpMatchInclude" data-template-match="include" placeholder="prefix:gpt-">'+esc(matchLines(b.match.include))+'</textarea></label><label>排除规则<textarea id="mpMatchExclude" data-template-match="exclude" placeholder="glob:*-embedding*">'+esc(matchLines(b.match.exclude))+'</textarea></label><label>匹配优先级<input id="mpMatchPriority" class="input" type="number" data-template-match="priority" value="'+b.match.priority+'"></label><label class="mp-label"><input id="mpMatchCase" type="checkbox" data-template-match="caseSensitive" '+(b.match.caseSensitive?'checked':'')+'>区分大小写</label></div><p class="mp-muted">匹配上游模型 ID；每行 exact:精确 ID、prefix:前缀、glob:通配符。排除优先，同级冲突需明确选择。匹配不自动勾选模型。</p>';
  content.common='<h3>族通用配置</h3><p class="mp-muted">先定义共同配置。版本或后缀只覆盖自定义字段，其余继续继承。</p>'+fieldEditor(b.rules,null,false);
  content.variants='<div class="mp-toolbar"><h3>版本或后缀 <span class="mp-badge">'+rows.length+'</span></h3>'+button('添加版本或后缀','add-variant')+'</div><p class="mp-muted">按上游 ID 的字面后缀识别，例如 5.6、5.5、luna。可自行添加版本号或文字后缀；多个命中取最长后缀，只应用一个条目的配置。</p>'+rows.map(v=>'<div class="mp-variant-row"><div><strong>'+esc(v.name)+'</strong><code>后缀 '+esc(v.suffix)+'</code><small>'+Object.keys(v.rules).length+' 项自定义 · 其余继承族通用配置</small></div><div class="mp-actions">'+button('配置','edit-variant','data-id="'+esc(v.id)+'"')+button('删除','delete-variant','data-id="'+esc(v.id)+'"',false,true)+'</div></div>').join('')+(rows.length?'':'<div class="mp-empty">暂未添加版本或后缀，全部使用族通用配置。需要时可自行添加任意数量。</div>');
  const v=version(templateId,b.base);content.sources='<h3>来源与版本</h3>'+templateOriginHTML(template())+(v?'<p class="mp-version-protection">'+esc(versionProtectionReasons(templateId,v.n).join(' · ')||'普通保存版本')+'</p>':'')+'<div class="mp-version-context"><div>'+versionHTML(v)+'<small>'+esc(v?.origin||'人工新建')+' · '+esc(v?storedVersionSummaryFull(v):'尚未保存')+'</small></div><div><span>基于来源版本</span><strong>'+esc(v?.basedOn?template().versions.find(x=>x.revisionId===v.basedOn)?.versionLabel||'来源版本未在本机':'初始版本')+'</strong></div></div><p>'+esc(v?.source||'人工新建；能力待核对')+'</p><p class="mp-muted">'+esc(v?versionState(templateId,v.n):'保存后生成首个版本')+'。修改后保存为新版本，历史版本保持。</p><p class="mp-muted">'+esc(v?templateUses(saved,templateId,v.n).join('、')||'该版本尚未被保存的渠道配置引用':'尚未采用')+'</p>';

  const nav=editorSections.map(([id,name],i)=>'<button class="mp-nav-item" data-mp="editor-section" data-id="'+id+'" title="'+name+'" aria-controls="mpTemplateSection-'+id+'" aria-current="'+(editorSection===id?'location':'false')+'"><span>'+String(i+1).padStart(2,'0')+'</span><span class="mp-nav-label">'+name+'</span></button>').join('');
  const sections=editorSections.map(([id,name])=>'<section id="mpTemplateSection-'+id+'" class="mp-template-section" data-template-section="'+id+'" aria-label="'+name+'">'+content[id]+'</section>').join('');
  return '<div id="mpTemplateWorkspace" class="mp-template-workspace '+(editorNavCollapsed?'nav-collapsed':'')+'"><nav class="mp-template-nav" aria-label="模版设置分区">'+button(icon('menu'),'toggle-editor-nav','id="mpTemplateNavToggle" title="'+(editorNavCollapsed?'展开':'收起')+'设置导航" aria-label="'+(editorNavCollapsed?'展开':'收起')+'设置导航" aria-expanded="'+!editorNavCollapsed+'"')+nav+'<div class="mp-template-tools" role="group" aria-label="模版操作">'+templateActions(true)+'</div>'+templateSaveStatus()+'</nav><div class="mp-template-sections">'+sections+'</div></div><div id="mpTemplateError" class="mp-alert" role="alert"></div>';
 }
 function editorNodes(){return editorSections.map(([id])=>$('#mpTemplateSection-'+id,host)).filter(Boolean);}
 function setEditorSection(id){
  if(!editorSections.some(([key])=>key===id))return;editorSection=id;
  $$('.mp-nav-item',host).forEach(node=>node.setAttribute('aria-current',node.dataset.id===id?'location':'false'));
 }
 function detailHeaderInsets(scroller){
  const style=scroller?window.getComputedStyle?.(scroller):null;
  const value=key=>Math.max(0,parseFloat(style?.[key])||0);
  host.style?.setProperty?.('--mp-header-inset-left',value('paddingLeft')+'px');
  host.style?.setProperty?.('--mp-header-inset-right',value('paddingRight')+'px');
  return value('paddingTop');
 }
 function syncTabBar(scroller=host.closest('.main')){
  const inset=detailHeaderInsets(scroller),height=$('#mpTabsBar',host)?.getBoundingClientRect().height||0;
  host.style?.setProperty?.('--mp-tabs-top',-inset+'px');
  host.style?.setProperty?.('--mp-tabs-height',height+'px');
  return {inset,height};
 }
 function editorOffset(scroller=host.closest('.main')){
  const {inset,height:tabsHeight}=syncTabBar(scroller);
  const height=$('#mpTemplateHeader',host)?.getBoundingClientRect().height||0;
  const offset=tabsHeight+height+(scroller?.clientTop||0)+12;
  host.style?.setProperty?.('--mp-editor-top',tabsHeight-inset+'px');
  host.style?.setProperty?.('--mp-editor-offset',offset+'px');
  host.style?.setProperty?.('--mp-editor-nav-top',tabsHeight+height+12-inset+'px');
  if(scroller)host.style?.setProperty?.('--mp-editor-nav-height',Math.max(0,scroller.clientHeight-tabsHeight-height-24)+'px');
  return offset;
 }
 function editorScrollPosition(scroller){
  if(!scroller)return null;const top=scroller.getBoundingClientRect().top,sections=editorNodes();
  const offset=editorOffset(scroller),anchor=sections.find(node=>node.getBoundingClientRect().bottom>top+offset)||sections.at(-1);
  return {top:scroller.scrollTop,id:anchor?.id,offset:anchor?anchor.getBoundingClientRect().top-top:0};
 }
 function restoreEditorScroll(scroller,position,measure=editorOffset){
  if(!scroller||!position)return;measure(scroller);const anchor=position.id&&$('#'+position.id,host);
  scroller.scrollTop=anchor?scroller.scrollTop+anchor.getBoundingClientRect().top-scroller.getBoundingClientRect().top-position.offset:position.top;
 }
 function syncEditorSection(){
  const route=document.documentElement?.dataset.route;
  if(tab!=='templates'||!templateDetail||route&&route!=='models')return;
  const scroller=host.closest('.main');if(!scroller)return;const offset=editorOffset(scroller);
  const sections=editorNodes();if(!buffer()||!sections.length)return;
  const top=scroller.getBoundingClientRect().top+offset+12;let active=sections[0];
  for(const section of sections)if(section.getBoundingClientRect().top<=top)active=section;
  if(scroller.scrollHeight>scroller.clientHeight&&scroller.scrollTop+scroller.clientHeight>=scroller.scrollHeight-2)active=sections.at(-1);
  setEditorSection(active.dataset.templateSection);
 }
 function scrollEditorSection(id){
  if(tab!=='templates'||!templateDetail||!buffer()||!editorSections.some(([key])=>key===id))return;
  const section=$('#mpTemplateSection-'+id,host),scroller=host.closest('.main');if(!section)return;
  setEditorSection(id);if(!scroller)return;
  const top=scroller.scrollTop+section.getBoundingClientRect().top-scroller.getBoundingClientRect().top-editorOffset(scroller);
  scroller.scrollTo({top,behavior:window.matchMedia?.('(prefers-reduced-motion: reduce)')?.matches?'auto':'smooth'});
 }
 function toggleEditorNav(){
  const workspace=$('#mpTemplateWorkspace',host),toggle=$('#mpTemplateNavToggle',host);if(!workspace||!toggle)return;
  const scroller=host.closest('.main'),position=editorScrollPosition(scroller);editorNavCollapsed=!editorNavCollapsed;
  workspace.classList.toggle('nav-collapsed',editorNavCollapsed);
  toggle.setAttribute('aria-expanded',String(!editorNavCollapsed));toggle.setAttribute('aria-label',(editorNavCollapsed?'展开':'收起')+'设置导航');toggle.setAttribute('title',(editorNavCollapsed?'展开':'收起')+'设置导航');
  restoreEditorScroll(scroller,position);syncEditorSection();
 }
 function editVariant(id=''){
  const b=buffer();if(!b)return;const original=b.variants?.find(v=>v.id===id),item=clone(original||{id:'variant-'+Date.now()+'-'+(++editorDigest),name:'',suffix:'',rules:{}});
  modal(original?'配置版本或后缀':'添加版本或后缀','<div class="mp-template-form"><label>条目名称<input id="mpVariantName" class="input" value="'+esc(item.name)+'" placeholder="例如 5.6、5.5 或 luna"></label><label>匹配后缀<input id="mpVariantSuffix" class="input" value="'+esc(item.suffix)+'" placeholder="例如 5.6、5.5 或 luna"></label></div><p class="mp-muted">按后缀匹配，仅自定义字段覆盖族通用配置。未填字段继承；条目名称仅用于显示，不会改写上游 ID。</p>'+fieldEditor(item.rules,{values:b.rules,sources:Object.fromEntries(Object.keys(b.rules).map(k=>[k,'族通用配置']))},false)+'<div id="mpVariantError" class="mp-alert" role="alert"></div>','加入候选',()=>{
   item.name=$('#mpVariantName').value.trim();item.suffix=$('#mpVariantSuffix').value.trim();const next=clone(b);next.variants=(next.variants||[]).filter(v=>v.id!==item.id).concat([item]);const error=templateError(next);if(error){$('#mpVariantError').textContent=error;return;}b.variants=next.variants;editorDigest++;closeModal();render();
  },{confirmKeepsOpen:true});bindFields($('[data-field-editor]',$('#modalBody')),item.rules,{values:b.rules,sources:Object.fromEntries(Object.keys(b.rules).map(k=>[k,'族通用配置']))},(rules,valid)=>{item.rules=rules;$('#modalConfirm').disabled=!valid;});
 }
 function templateRulesError(r){
  const object=x=>!!x&&typeof x==='object'&&!Array.isArray(x);
  if(!object(r)||Object.keys(r).some(k=>!['protocol','context','modalities','efforts','default','fast','force','omit','authority'].includes(k)))return '模型配置字段无效';
  if(Object.hasOwn(r,'protocol')&&!protocols.includes(r.protocol))return '模版输出协议无效';
  if(Object.hasOwn(r,'efforts')&&(!Array.isArray(r.efforts)||r.efforts.some(x=>!levels.includes(x))))return '推理档位无效';
  if(Object.hasOwn(r,'default')&&(!levels.includes(r.default)||r.efforts&&!r.efforts.includes(r.default)))return '默认档位不在可选档位中';
  if(Object.hasOwn(r,'context')&&(!Number.isSafeInteger(r.context)||r.context<=0))return '上下文窗口须为正整数';
  if(Object.hasOwn(r,'modalities')&&(!Array.isArray(r.modalities)||!r.modalities.length||r.modalities.some(x=>!['text','image','audio'].includes(x))))return '输入模态无效';
  if(Object.hasOwn(r,'fast')&&!['unknown','supported','unsupported'].includes(r.fast))return 'Fast 声明无效';
  if(Object.hasOwn(r,'omit')&&typeof r.omit!=='boolean')return '不传参数须为布尔值';
  if(Object.hasOwn(r,'authority')&&r.authority!=='force')return '声明模式无效';
  if(Object.hasOwn(r,'force')&&(!levels.includes(r.force)||r.efforts&&!r.efforts.includes(r.force)))return '强制档不在可选列表';
  if(r.force&&r.omit)return '强制档与不传参数冲突';return '';
 }
 function templateError(c){
  const object=x=>!!x&&typeof x==='object'&&!Array.isArray(x);
  if(!object(c)||typeof c.name!=='string'||!c.name.trim())return '请填写模版名称';
  if(typeof c.description!=='string'||!object(c.match)||!Array.isArray(c.match.include)||!Array.isArray(c.match.exclude)||!Array.isArray(c.match.channels)||typeof c.match.caseSensitive!=='boolean')return '匹配规则结构无效';
  if(!c.match.include.length)return '请至少填写一条包含规则';
  if(!Number.isSafeInteger(c.match.priority))return '匹配优先级须为整数';
  if(!['all','whitelist','blacklist'].includes(channelMode(c.match)))return '渠道范围模式无效';
  if(c.match.channels.some(id=>typeof id!=='string'||!draft.providers.some(p=>p.id===id)))return '匹配渠道范围无效';
  for(const r of [...c.match.include,...c.match.exclude])if(!object(r)||Object.keys(r).some(k=>!['type','value'].includes(k))||!['exact','prefix','glob'].includes(r.type)||typeof r.value!=='string'||!r.value.trim()||r.value.length>256)return '匹配规则无效或过长';
  const error=templateRulesError(c.rules);if(error)return error;
  if(c.variants!==undefined&&!Array.isArray(c.variants))return '版本或后缀条目须为数组';
  const ids=new Set(),suffixes=new Set();
  for(const v of c.variants||[]){
   if(!object(v)||Object.keys(v).some(k=>!['id','name','suffix','rules'].includes(k))||typeof v.id!=='string'||!v.id||ids.has(v.id))return '条目 ID 无效或重复';ids.add(v.id);
   if(typeof v.name!=='string'||!v.name.trim()||typeof v.suffix!=='string'||!v.suffix.trim()||v.suffix.length>256)return '请填写条目名称和匹配后缀（最多 256 字符）';
   const suffix=c.match.caseSensitive?v.suffix:v.suffix.toLowerCase();if(suffixes.has(suffix))return '匹配后缀重复，请合并规则';suffixes.add(suffix);
   const e=templateRulesError(v.rules)||templateRulesError({...c.rules,...v.rules});if(e)return v.name+'：'+e;
  }return '';
 }
 function updateTemplateInput(el){const b=buffer();if(!b)return;if(el.dataset.templateMeta)b[el.dataset.templateMeta]=el.value;else {const k=el.dataset.templateMatch;if(k==='include'||k==='exclude')b.match[k]=parseMatchLines(el.value);else if(k==='channels'){b.match.channels=el.value?[el.value]:[];if(!b.match.channelMode||b.match.channelMode==='all')b.match.channelMode='whitelist';}else if(k==='channelMode'){b.match.channelMode=el.value;render();}else if(k==='priority')b.match.priority=Number(el.value);else if(k==='caseSensitive')b.match.caseSensitive=el.checked;}editorDigest++;updateTemplateSaveStatus();}
 function matchPreviewHTML(t,c){const rows=draft.providers.flatMap(p=>p.models.map(m=>({p,m,result:matchResult(c.match,m,p)}))),hits=rows.filter(x=>x.result==='matched'),other=rows.filter(x=>x.result!=='matched');
  const table=items=>'<div class="mp-match-results"><table><thead><tr><th>渠道 / 模型</th><th>匹配结果与原因</th><th>当前状态</th></tr></thead><tbody>'+items.map(({p,m,result})=>{let reason=result==='scope'?(channelMode(c.match)==='blacklist'?'范围外：渠道在黑名单中':'范围外：渠道未列入白名单'):result==='unmatched'?'未命中任何包含规则':result==='excluded'?'排除优先：':'命中：';const rule=(result==='excluded'?c.match.exclude:c.match.include).find(r=>matchPattern(r,m.id,c.match.caseSensitive));if(['matched','excluded'].includes(result)&&rule)reason+=rule.type+':'+rule.value;const overlap=result==='matched'?templates.filter(x=>x.id!==t.id&&templateMatches(x,m,p)&&version(x.id,x.main)?.match.priority===c.match.priority):[];if(result==='matched'){const variant=variantFor(c,m.id);reason+=' · '+(variant?'版本或后缀 '+variant.name+' / 后缀 '+variant.suffix:'族通用配置');}if(overlap.length)reason+=' · 同级重叠 '+overlap.map(t=>t.name).join('、')+'（仅已采用集合产生运行冲突）';return '<tr><td>'+esc(p.name)+'<br><code>'+esc(m.id)+'</code></td><td>'+esc(reason)+'</td><td>'+esc((p.enabled?'启用':'停用')+' · '+(selected(p).some(x=>x.id===m.id)?'已选':'未选')+(Object.keys(m.overrides).length?' · 保留自定义':''))+'</td></tr>';}).join('')+'</tbody></table></div>';
  return '<p>命中 '+hits.length+' · 排除 '+rows.filter(x=>x.result==='excluded').length+' · 未匹配 '+rows.filter(x=>x.result==='unmatched').length+' · 范围外 '+rows.filter(x=>x.result==='scope').length+'</p>'+(!hits.length?'<p class="mp-alert">零命中，请检查模型 ID、规则和渠道范围。</p>':table(hits))+(other.length?'<details class="mp-details"><summary>查看排除、未匹配与范围外（'+other.length+'）</summary>'+table(other)+'</details>':'')+'<p class="mp-muted">预览读取 mock 清单，含停用和未勾选模型；仅已采用版本参与实际解析，不自动勾选或写入。</p>';
 }

 function matchPreview(target=template(),content=null,label=''){
  const c=content||templateContent(),error=templateError(c);if(error){modal('匹配预览','<p class="mp-alert">'+esc(error)+'</p>','返回编辑');return;}
  modal('匹配预览 · '+c.name,(label?'<p class="mp-muted">'+esc(label)+'</p>':'')+matchPreviewHTML(target,c),label?'关闭':'返回编辑');
 }

 function portablePolicy(policy){const p=clone(policy);p.providers.forEach(x=>{delete x.connection;(x.nodes||[]).forEach(n=>delete n.url);delete x.addedInPrototype;delete x.availability;});return p;}
 const policyDigest=()=>JSON.stringify(draft),baseDigest=()=>JSON.stringify(portablePolicy(saved));
 const selected=p=>p.mode==='none'?[]:p.models.filter(m=>p.mode==='all'||p.mode==='native'||m.selected);
 function resolve(p,m,policy=draft,skipLocal=false){
  const ref=p.id==='native'?null:templateResolution(p,m,policy).ref, [id,n]=String(ref||'').split(':');
  const layers=[['官方声明',m.official],...templateLayers(ref,m),['渠道默认',p.defaults],['渠道族覆盖',familyOverride(p,m,policy)]];
  if(!skipLocal)layers.push(['单模型例外',m.overrides]);
  const values={},sources={};for(const [source,rules] of layers)for(const [k,v] of Object.entries(rules||{})){values[k]=v;sources[k]=source;}
  const route=resolveRoute(p,m,policy,skipLocal);values.protocol=route.protocol;sources.protocol=route.source;return {values,sources,ref,route};
 }
 // 协议独立解析，不以最终协议反过来重新匹配模版。
 function serviceNodes(p){return p?.id&&p.id!=='native'?p.nodes||[]:[];}
 function activeNode(p){return serviceNodes(p).find(n=>n.id===p.activeNodeId)||null;}
 function nodeConnection(p){return {...clone(p.connection||{auth:'apikey',keyConfigured:false}),baseUrl:activeNode(p)?.url||''};}
 function connectionBound(p){const c=p?.connection;return !!activeNode(p)?.url&&['apikey','none'].includes(c?.auth)&&(c.auth!=='apikey'||!!c.keyConfigured);}
 function resolveRoute(p,m,policy=draft,skipLocal=false){
  if(p.id==='native')return {protocol:'原生',source:'官方管理',nodeId:null,error:'',needsReview:false};
  let protocol=p.defaults.protocol||p.protocol,source='渠道默认';
  const ref=templateResolution(p,m,policy).ref;
  for(const [label,layer] of [...templateLayers(ref,m),['渠道族覆盖',familyOverride(p,m,policy)],...(!skipLocal?[['单模型例外',m.overrides]]:[])])if(Object.hasOwn(layer,'protocol')){protocol=layer.protocol;source=label;}
  const node=activeNode(p);let error=templateResolution(p,m,policy).error||(!protocols.includes(protocol)?'协议未支持':!node?'当前服务节点不存在':'');
  if(!error&&!connectionBound(p))error='当前节点地址或渠道认证尚未配置';
  return {protocol,source,nodeId:node?.id||null,error,needsReview:protocol!==p.protocol};
 }
 function routePlan(policy=draft){
  const groups=[],errors=[],picker=[],reviews=[];
  for(const p of policy.providers.filter(p=>p.enabled&&p.id!=='native')){
   const routes=selected(p).map(m=>({m,r:resolveRoute(p,m,policy)}));
   for(const {m,r} of routes){if(r.error)errors.push(p.name+' / '+m.id+'：'+r.error);if(r.needsReview)reviews.push(p.name+' / '+m.id+'：切换协议后推理 / Fast 待核对');}
   const valid=routes.filter(x=>!x.r.error),native=[...new Set(valid.map(x=>x.r.protocol).filter(x=>!['Chat','Responses'].includes(x)))];
   const sets=native.length<=1?[valid]:native.map(w=>valid.filter(x=>x.r.protocol===w)).concat([valid.filter(x=>['Chat','Responses'].includes(x.r.protocol))]).filter(x=>x.length);
   for(const entries of sets.filter(x=>x.length)){
    const wire=entries.find(x=>!['Chat','Responses'].includes(x.r.protocol))?.r.protocol||(['Chat','Responses'].includes(p.protocol)&&entries.some(x=>x.r.protocol===p.protocol)?p.protocol:entries[0].r.protocol);
    const modelAdapters=Object.fromEntries(entries.filter(x=>x.r.protocol!==wire&&['Chat','Responses'].includes(x.r.protocol)).map(x=>[x.m.id,adapters[x.r.protocol]]));
    // 所有协议共用一个当前节点和渠道认证；运行分组身份不随节点切换而派生。
    const group={logicalId:p.id,id:p.id+'__'+adapters[wire],adapter:adapters[wire],nodeId:p.activeNodeId,selectedModels:entries.map(x=>x.m.id),modelAdapters};groups.push(group);
    for(const {m,r} of entries)picker.push({logicalId:p.id,key:m.key,modelId:m.id,protocol:r.protocol,nodeId:r.nodeId,providerId:group.id});
   }
  }
  const order=projection(policy).filter(x=>x.p!=='system').map(x=>{const parent=picker.find(y=>y.key===(x.type==='fast'?x.key.slice(0,-6):x.key));return {...x,...(parent?{providerId:parent.providerId,protocol:parent.protocol}:{} )};});
  return {groups,errors,reviews,order};
 }
 function routePlanHTML(policy=draft){const plan=routePlan(policy);return '<p>模型协议与节点地址独立；同渠道模型共用当前节点及认证。当前计划 '+plan.groups.length+' 个托管渠道分组；实际路由 ID 需由运行目录读回。</p>'+diffHTML(plan.groups.map(g=>g.logicalId+' → '+g.id+' · '+g.adapter+' · '+g.selectedModels.length+' 个基础模型'+(Object.keys(g.modelAdapters).length?' · 单模型协议覆盖 '+JSON.stringify(g.modelAdapters):'')))+(plan.errors.length?'<div class="mp-alert">'+diffHTML(plan.errors)+'</div>':'')+(plan.reviews.length?'<details class="mp-details"><summary>协议切换后的能力核对</summary>'+diffHTML(plan.reviews)+'</details>':'')+'<p class="mp-muted">原型仅演示编译计划。真实凭据、硬绑定、推理和 Fast wire 校验仍是实现交付门。</p>';}
 function projection(policy=draft){
  const rows=[];policy.providers.filter(p=>p.enabled).forEach(p=>selected(p).forEach(m=>{
   rows.push({p:p.id,name:p.name,id:m.id,key:m.key,type:'base'});
   if(p.id!=='native'&&policy.fastRows&&p.fastEnabled!=='off'&&resolve(p,m,policy).values.fast==='supported')rows.push({p:p.id,name:p.name,id:m.id+'--fast',key:m.key+'::fast',type:'fast'});
  }));for(let i=0;i<policy.hidden;i++)rows.push({p:'system',name:'系统占用',id:'隐藏目录条目 '+(i+1),key:'hidden:'+i,type:'hidden'});return rows;
 }
 const count=()=>projection().length;
 function change(fn){if(busy())return;fn();render();}
 const notice=text=>toast(esc(text),{kind:'info'});
 function modal(title,body,confirm='确定',fn=null,extra={}){
  delete $('#modalBody').dataset.mpConversation;delete $('#modalBody').dataset.mpExternalTask;
  openModal(title,'<div class="mp-dialog">'+body+'</div>',confirm,fn,{hideNote:true,...extra});
  $('#modal').classList.add('mp-modal');$('#modalConfirm').disabled=false;
  window.__modalOnClose=()=>{delete $('#modalBody').dataset.mpConversation;delete $('#modalBody').dataset.mpExternalTask;$('#modal').classList.remove('mp-modal','mp-external-modal');$('#modalConfirm').disabled=false;extra.onClose?.();};
 }
 function ruleText(k,v){if(v==null)return '未知';if(k==='protocol')return adapters[v]||v;if(k==='efforts')return v.map(x=>labels[x]).join(' / ')||'不显示档位';if(k==='default')return labels[v]||v;if(k==='context')return v.toLocaleString()+' tokens';if(k==='modalities')return v.join(' / ');if(k==='fast')return {supported:'支持',unsupported:'不支持',unknown:'未知'}[v]||v;return v;}
 function fieldEditor(rules,inherited=null,readonly=false){
  const effective=inherited?.values||{efforts:levels,default:'medium'}, disabled=readonly?'disabled':'';
  function field(key,label,body){const custom=Object.hasOwn(rules,key),v=custom?rules[key]:effective[key];
   return '<div class="mp-field" data-field="'+key+'"><div class="mp-field-label">'+label+'</div><select class="mp-select" data-field-mode="'+key+'" aria-label="'+label+'设置方式" '+disabled+'>'+opts([['inherit','继承'],['custom','自定义']],custom?'custom':'inherit')+'</select><div class="mp-field-control" '+(!custom?'hidden':'')+'>'+body(v)+'</div><div class="mp-field-source">'+(inherited?'解析值：'+esc(ruleText(key,v))+' · 来源：'+esc(custom?'本层自定义':inherited.sources[key]||'未知'):'规则：'+(custom?esc(ruleText(key,v)):'继承目标声明；未选目标时不推测最终能力'))+'</div></div>';
  }
  const checks=(key,items,v)=>'<div class="mp-checks">'+items.map(x=>'<label><input type="checkbox" data-field-check="'+key+'" value="'+x+'" '+((v||[]).includes(x)?'checked':'')+' '+disabled+'>'+esc(labels[x]||x)+'</label>').join('')+'</div>';
  return '<div class="mp-editor" data-field-editor><div class="mp-section"><h3>协议与基本能力</h3>'+field('protocol','模型协议',v=>'<select class="mp-select" data-field-value="protocol" aria-label="模型协议" '+disabled+'>'+protocolOptions(v||'Responses')+'</select><span class="mp-muted">模版可决定协议；模型自定义优先，恢复继承回到父层</span>')+field('context','上下文窗口',v=>'<input class="input" type="number" min="1" step="1" data-field-value="context" aria-label="上下文窗口 tokens" placeholder="填写 tokens" value="'+(v??'')+'" '+disabled+'><span class="mp-muted">tokens · 声明不会扩容</span>')+field('modalities','输入模态',v=>checks('modalities',['text','image','audio'],v))+'</div><div class="mp-section"><h3>推理</h3>'+field('efforts','可选推理档位',v=>checks('efforts',levels,v))+field('default','默认推理档位',v=>'<select class="mp-select" data-field-value="default" aria-label="默认推理档位" '+disabled+'>'+opts((rules.efforts??effective.efforts??levels).map(x=>[x,labels[x]]),v)+'</select><span class="mp-muted">可独立设置，不必覆盖可选列表</span>')+'</div><div class="mp-section"><h3>快速模式</h3>'+field('fast','Fast 能力声明',v=>'<select class="mp-select" data-field-value="fast" aria-label="Fast 能力声明" '+disabled+'>'+opts([['unknown','未知'],['supported','支持（需依据）'],['unsupported','不支持']],v||'unknown')+'</select><span class="mp-muted">声明能力不会自动开启 Fast 策略</span>')+'</div><details class="mp-details"><summary>高级推理设置</summary><p>“无”是档位。隐藏档位、强制档、不传参数和标签映射是不同规则，不能由多选自动开启。</p><label class="mp-label"><input type="checkbox" data-field-hide '+(rules.efforts?.length===0?'checked':'')+' '+disabled+'>不显示推理档位（显式清空列表）</label><div class="mp-row" style="margin-top:12px"><label class="mp-label">请求强制档 <select class="mp-select" data-field-force '+disabled+'>'+opts([['','继承 / 不强制'],...levels.map(x=>[x,labels[x]])],rules.force||'')+'</select></label></div><div class="mp-row" style="margin-top:12px"><label class="mp-label"><input type="checkbox" data-field-omit '+(rules.omit?'checked':'')+' '+disabled+'>请求不传推理参数</label></div><label class="mp-label">声明模式 <select class="mp-select" data-field-authority '+disabled+'>'+opts([['','继承'],['force','强制使用本层声明']],rules.authority||'')+'</select></label><p>协议标签映射由适配器核对；权威声明、强制档与不传参数相互独立。恢复继承不会清除父层。</p></details><div class="mp-alert" data-field-error role="alert"></div></div>';
 }
 function readFields(root,rules){const next=clone(rules);
  $$('[data-field-mode]',root).forEach(el=>{const k=el.dataset.fieldMode;if(el.value==='inherit'){delete next[k];return;}if(k==='context')next[k]=Number($('[data-field-value="context"]',root).value);else if(k==='efforts'||k==='modalities')next[k]=$$('[data-field-check="'+k+'"]:checked',root).map(x=>x.value);else next[k]=$('[data-field-value="'+k+'"]',root).value;});
  if($('[data-field-hide]',root).checked)next.efforts=[];
  const force=$('[data-field-force]',root).value;if(force)next.force=force;else delete next.force;if($('[data-field-omit]',root).checked)next.omit=true;else delete next.omit;const authority=$('[data-field-authority]',root).value;if(authority)next.authority=authority;else delete next.authority;return next;
 }
 function validate(root,rules,inherited){let error='';const final=rules.efforts??inherited?.values.efforts??levels;
  if(rules.protocol&&!protocols.includes(rules.protocol))error='协议无效。';
  if(Object.hasOwn(rules,'context')&&(!Number.isSafeInteger(rules.context)||rules.context<=0))error='上下文窗口须为正整数 tokens。';
  if(rules.modalities&&!rules.modalities.length)error='输入模态至少选择一项，或恢复继承。';
  if(rules.efforts?.length===0&&!$('[data-field-hide]',root).checked)error='请恢复继承，或在高级区明确选择“不显示推理档位”。';
  if(rules.default&&!final.includes(rules.default))error='默认档位不在最终可选列表中，请调整或恢复继承。';
  if(rules.force&&!final.includes(rules.force))error='强制档不在最终可选列表中。';if(rules.force&&rules.omit)error='不传参数与请求强制档冲突，请只保留一个。';
  $('[data-field-error]',root).textContent=error;return !error;
 }
 function bindFields(root,rules,inherited,callback){
  function edited(e){
   if(e.target.matches('[data-field-mode]')){const field=e.target.closest('[data-field]');$('.mp-field-control',field).hidden=e.target.value==='inherit';if(e.target.value==='custom'&&e.target.dataset.fieldMode==='modalities'&&!$$('[data-field-check]:checked',field).length)$('[data-field-check]',field).checked=true;}
   if(e.target.matches('[data-field-check="efforts"],[data-field-mode="efforts"],[data-field-hide]')){
    const choice=$('[data-field-value="default"]',root),old=choice.value,next=readFields(root,rules),final=next.efforts??inherited?.values.efforts??levels;
    choice.innerHTML=opts(final.map(x=>[x,labels[x]]),old);
    if(!final.includes(old))choice.insertAdjacentHTML('afterbegin','<option selected value="'+esc(old)+'">原默认 '+esc(labels[old]||'未知')+'（需调整）</option>');
   }
   const next=readFields(root,rules);callback(next,validate(root,next,inherited));
  }
  root.addEventListener('change',edited);root.addEventListener('input',e=>{if(e.target.type==='number')edited(e);});
 }
 function editFields(title,rules,inherited,commit,extra=''){
  let next=clone(rules);modal(title,extra+fieldEditor(next,inherited)+'<p class="mp-muted">确定只加入工作区草稿，渠道模型页保存后才写入。</p>','确定',()=>{
   const root=$('[data-field-editor]',$('#modalBody'));next=readFields(root,next);if(!validate(root,next,inherited))return;closeModal();commit(next);
  },{confirmKeepsOpen:true,extraLabel:'恢复继承',onExtra:()=>{next={};const root=$('[data-field-editor]',$('#modalBody'));root.outerHTML=fieldEditor({},inherited);bindFields($('[data-field-editor]',$('#modalBody')),next,inherited,(v,ok)=>{next=v;$('#modalConfirm').disabled=!ok;});$('#modalConfirm').disabled=false;}});
  bindFields($('[data-field-editor]',$('#modalBody')),next,inherited,(v,ok)=>{next=v;$('#modalConfirm').disabled=!ok;});
 }
 function availabilityText(p){const kind=p.availability?.kind||'unknown';return {unknown:'未验证',available:'连接检查通过','no-credit':'无额度'+(p.availability?.source==='manual'?'（手工）':''),'auth-failed':'认证失败','connection-failed':'连接失败',unavailable:'上游不可用'}[kind]||'未验证';}
 function providerSwitch(p){
  if(p.id==='native')return '<div class="mp-channel-state mp-muted" title="官方 / 原生来源由官方管理，此处只读">官方管理</div>';
  return '<div class="mp-channel-state"><div class="mp-channel-toggle"><button class="toggle" type="button" role="switch" aria-checked="'+p.enabled+'" aria-label="'+esc(p.name)+' 启用状态" data-mp="toggle-provider" data-id="'+esc(p.id)+'" title="切换只加入草稿，保存后提示重启；停用保留白名单与顺序" '+(busy()?'disabled':'')+'></button><span>'+ (p.enabled?'启用':'停用')+'</span></div></div>';
 }
 const visibleProviders=()=>draft.providers.filter(p=>p.name.toLowerCase().includes(channelSearch.toLowerCase()));
 const scopeProviders=()=>draft.providers.filter(p=>p.id!=='native'&&scope.has(p.id));
 function scopeState(){const rows=visibleProviders().filter(p=>p.id!=='native'),n=rows.filter(p=>scope.has(p.id)).length;return {rows,checked:rows.length>0&&n===rows.length,mixed:n>0&&n<rows.length};}
 function channels(){const selection=scopeState(),rows=visibleProviders();return '<div class="mp-provider-table" role="table" aria-label="渠道列表"><div class="mp-provider-heading" role="row"><div class="mp-row"><input type="checkbox" data-scope-all aria-label="全选当前结果中的自定义渠道" title="全选当前搜索结果；保留搜索外已选渠道，官方 / 原生除外" aria-checked="'+(selection.mixed?'mixed':selection.checked)+'" '+(selection.checked?'checked ':'')+(busy()||!selection.rows.length?'disabled':'')+'></div><span>渠道</span><span>服务节点</span><span>模型</span><span>启用</span><span>操作</span></div>'+rows.map(p=>{
  const i=draft.providers.indexOf(p),locked=channelSearch||busy(),hint=p.id==='native'?'只读':availabilityText(p),discovery={loading:'获取中',failed:'获取失败',empty:'清单为空'}[p.refresh];
  return '<div class="mp-provider" role="row" data-provider="'+esc(p.id)+'" '+(!locked?'draggable="true"':'')+'><div class="mp-row"><input type="checkbox" data-scope="'+esc(p.id)+'" aria-label="将 '+esc(p.name)+' 加入配置范围" '+(scope.has(p.id)?'checked':'')+' '+(p.id==='native'?'disabled title="原生目录只读"':busy()?'disabled':'')+'><span class="mp-grip" title="拖动渠道排序">⠿</span></div><div class="mp-provider-name"><strong>'+esc(p.name)+'</strong><small>'+(serviceNodes(p).length?serviceNodes(p).length+' 节点 · ':'')+esc(hint)+(discovery?' · '+discovery:'')+'</small></div><div class="mp-provider-node-cell">'+quickNodeControls(p)+'</div><div class="mp-cell">'+selected(p).length+' / '+p.models.length+'<small>已选 / 发现</small></div>'+providerSwitch(p)+'<div class="mp-row mp-provider-actions">'+button('编辑','edit-provider','data-id="'+esc(p.id)+'" aria-label="编辑 '+esc(p.name)+' 渠道与模型"'+(p.id==='native'?' title="打开同一渠道编辑页；官方管理字段不可修改"':''))+(p.id==='native'?'':button('复制','copy-provider','data-id="'+esc(p.id)+'" aria-label="复制 '+esc(p.name)+' 为新渠道" '+(busy()?'disabled':''))+button('删除','delete-provider','data-id="'+esc(p.id)+'" aria-label="删除 '+esc(p.name)+' 渠道" '+(busy()?'disabled':''),false,true))+'<div class="mp-mini-pair">'+mini('↑','move-provider','data-id="'+esc(p.id)+'" data-direction="-1" aria-label="上移 '+esc(p.name)+'" '+(i===0||locked?'disabled':''))+mini('↓','move-provider','data-id="'+esc(p.id)+'" data-direction="1" aria-label="下移 '+esc(p.name)+'" '+(i===draft.providers.length-1||locked?'disabled':''))+'</div></div></div>';
 }).join('')+(rows.length?'':'<div class="mp-empty">没有匹配渠道，搜索外的已选范围保留。</div>')+'</div>'; }
 function copyProvider(id){
  const source=draft.providers.find(p=>p.id===id);if(!source||source.id==='native'||busy())return;
  if(stagedChannelConnections().some(([pid])=>pid===id)||stagedModelEdits().some(([key])=>source.models.some(m=>m.key===key))){notice('先确认或还原源渠道的待处理编辑，再复制');return;}
  const baseline=channelDraftFingerprint(),baseId=source.id.slice(0,54)+'-copy';let newId=baseId,newName=source.name.slice(0,73)+' 副本',suffix=2;
  while(draft.providers.some(p=>p.id.toLowerCase()===newId.toLowerCase()))newId=baseId+'-'+suffix++;
  suffix=2;while(draft.providers.some(p=>p.name.toLowerCase()===newName.toLowerCase()))newName=source.name.slice(0,70)+' 副本 '+suffix++;
  const canReuse=source.connection?.auth==='apikey'&&source.connection.keyConfigured;
  modal('复制为新渠道','<div class="mp-agent-form"><label>新渠道名称<input id="mpCopyProviderName" class="input" maxlength="80" value="'+esc(newName)+'"></label><label>新渠道标识<input id="mpCopyProviderId" class="input" maxlength="64" value="'+esc(newId)+'"></label></div><p class="mp-muted">保留节点、当前节点、模型清单 / 选择、顺序、模版引用和覆盖规则。副本默认停用，加入草稿后可继续编辑，保存才写入。</p>'+(source.connection?.auth==='apikey'?'<label class="mp-label"><input id="mpCopyProviderKey" type="checkbox" '+(canReuse?'':'disabled')+'>复用源渠道 API Key</label><p class="mp-muted">'+(canReuse?'未勾选时新渠道需配置自己的 Key。':'源渠道尚未配置 Key，副本需另行配置。')+'</p>':'')+'<div id="mpCopyProviderError" class="mp-alert" role="alert"></div>','创建副本',()=>{
   const error=$('#mpCopyProviderError');if(busy()||baseline!==channelDraftFingerprint()){error.textContent='源配置或草稿已变化，请关闭后重新复制。';return;}
   const name=$('#mpCopyProviderName').value.trim(),pid=$('#mpCopyProviderId').value.trim();
   if(!name||name.length>80){error.textContent='请填写 1–80 字的新渠道名称。';return;}
   if(!/^[a-zA-Z0-9][a-zA-Z0-9._-]{0,63}$/.test(pid)||['native','openai','system','constructor','prototype','__proto__'].includes(pid.toLowerCase())){error.textContent='渠道标识无效或为保留标识。';return;}
   if(draft.providers.some(p=>p.id.toLowerCase()===pid.toLowerCase()||p.name.toLowerCase()===name.toLowerCase())){error.textContent='新渠道名称或标识已存在。';return;}
   const reuse=canReuse&&!!$('#mpCopyProviderKey')?.checked,next=clone(source);
   Object.assign(next,{id:pid,name,enabled:false,models:source.models.map((m,i)=>({...clone(m),key:pid+'::'+i})),connection:{auth:source.connection?.auth||'apikey',keyConfigured:reuse},refresh:'idle',refreshToken:0,updated:'待刷新（复制）',addedInPrototype:true,availability:reuse&&source.availability?.kind==='no-credit'?clone(source.availability):{kind:'unknown'},copiedFrom:source.id});
   closeModal();nodeAuthRevision[pid]=(nodeAuthRevision[pid]||0)+1;change(()=>{draft.providers.push(next);current=pid;channelDetail=true;tab='channels';search='';family='';onlySelected=false;channelSearch='';});notice('新渠道副本已加入草稿，默认停用；保存后生效');
  },{confirmKeepsOpen:true});
 }
 function setProvidersEnabled(enabled){const targets=scopeProviders().filter(p=>p.enabled!==enabled);if(targets.length)change(()=>targets.forEach(p=>p.enabled=enabled));}
 function deleteProviders(targetIds=null){const targets=targetIds?draft.providers.filter(p=>p.id!=='native'&&targetIds.includes(p.id)):scopeProviders();if(!targets.length||busy())return;const ids=new Set(targets.map(p=>p.id)),digest=policyDigest(),hidden=targets.filter(p=>!visibleProviders().includes(p)).length;
  modal(targetIds?'删除 '+targets[0].name+' 渠道？':'删除 '+targets.length+' 个渠道？','<p>'+(!targetIds&&hidden?'包含搜索外的 '+hidden+' 个已选渠道。':'')+'将移除以下渠道及其模型策略，族模版库保留。</p>'+diffHTML(targets.map(p=>p.name+' · 已选 '+selected(p).length+' / 发现 '+p.models.length))+(ids.has(agent.providerId)?'<p class="mp-alert">包含 Agent 执行渠道；保存后自动生成将不可用，执行偏好保留，请重新选择。</p>':'')+'<p class="mp-muted">删除先加入草稿，可在渠道列表顶部还原修改；页面保存后才写入并提示重启。保存后可由应用记录生成恢复草稿。</p>','删除并加入草稿',()=>{if(busy()||digest!==policyDigest()){notice('渠道配置已变化，请重新确认删除范围');closeModal();return;}closeModal();change(()=>draft.providers=draft.providers.filter(p=>p.id==='native'||!ids.has(p.id)));},{confirmKeepsOpen:true});
 }
 function mini(name,action,attrs){return '<button class="mp-mini" data-mp="'+action+'" '+attrs+'>'+name+'</button>';}
 function mpAnn(num,title,body){
  return '<div class="proto-annotation" id="ann-mp-'+num+'" data-annotation-number="'+num+'" data-annotation-title="'+title+'" data-annotation-category="实现边界 · 模型配置"><button type="button" class="annotation-marker" aria-expanded="false" aria-controls="annotationPopover" aria-label="原型注释 '+num+'：'+title+'"><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 2.5h7l3 3v8H3zM10 2.5v3h3M5.5 8h5M5.5 10.5h3"/></svg><span>原型</span><span class="annotation-number">'+num+'</span><span class="annotation-marker-plus" aria-hidden="true">＋</span></button><div class="annotation-source" hidden><p>'+body+'</p><div class="annotation-footer">实现边界 · 模型配置</div></div></div>';
 }
 function toolbar(){
  const editing=tab==='channels'&&channelDetail||tab==='templates'&&templateDetail&&!!buffer();
  const actions=tab==='channels'&&!channelDetail?status():!editing&&tab==='templates'?'<div class="mp-actions">'+button('Agent 配置','settings')+'</div>':'';
  return '<div id="mpTabsBar" class="mp-toolbar mp-tabs-bar" data-page-tabs><div class="settings-tabs" role="tablist" aria-label="模型配置分区">'+['channels','templates'].map(x=>'<button role="tab" aria-selected="'+(tab===x)+'" class="'+(tab===x?'active':'')+'" data-mp="tab" data-tab="'+x+'">'+(x==='channels'?'渠道模型':'模型模版')+'</button>').join('')+'</div>'+actions+mpAnn('01','渠道与模版均为内存演示','Tab 栏滚至主内容区顶部后固定，详情标题固定在其下，分区锚点避让两层高度。模版名称表达模型族或自定义用途，不自动附协议；渠道列表和标题只展示名称、节点、数量和状态，不用渠道默认协议概括全部模型。协议在配置字段及具体模型解析中查看，模版通用或具体型号可定义、单模型可覆盖。渠道顺序、白名单与模版策略全部是内存示例；保存、生成与同步仅演示状态与提示，重启与写入真实配置不在原型内发生。')+'</div>';
 }

 function channelAction(name,action,mark,attrs='',primary=false){const title=/\btitle=/.test(attrs)?'':' title="'+esc(name)+'"';return '<button class="btn '+(primary?'primary':'ghost')+' mp-nav-action" data-mp="'+action+'" '+attrs+title+' aria-label="'+esc(name)+'">'+icon(mark)+'<span class="mp-nav-action-label">'+esc(name)+'</span></button>';}
 // 保存状态在标题集中展示；列表顶部直接提供还原修改与保存。
 const channelPolicyStates={clean:['未修改','neutral'],dirty:['未保存修改','warning'],staged:['有待确认编辑','warning'],'over-budget':['超出激活预算','danger'],saving:['正在保存','info'],pending:['已保存 · 待重启','warning'],failed:['保存失败','danger'],stopped:['保存已取消','neutral'],uncertain:['保存结果待核对','warning']};
 function channelPolicyState(){
  const uncertain=operation?.status==='timeout'||['failed','stopped'].includes(operation?.status)&&operation.step>=2;
  const id=busy()?'saving':uncertain?'uncertain':operation?.status==='failed'&&dirty()?'failed':operation?.status==='stopped'&&dirty()?'stopped':stagedModelEdits().length||stagedChannelConnections().length?'staged':count()>100?'over-budget':dirty()?'dirty':pending?'pending':'clean';
  const [label,tone]=channelPolicyStates[id];return {id,label,tone};
 }
 function channelPolicyStatusHTML(framed=false){
  const state=channelPolicyState(),previous=pending&&state.id!=='pending',context='全部渠道配置的保存状态'+(previous?'；上次保存待重启':'');
  return '<span class="mp-template-state '+(framed?'mp-policy-chip ':'')+state.tone+'" data-channel-state="'+state.id+'" data-status-scope="all-channels" title="'+context+'" '+(framed?'role="status" aria-live="polite" aria-busy="'+busy()+'"':'')+'>'+esc(state.label)+(framed&&previous?'<span class="mp-policy-pending" data-channel-pending>上次保存待重启</span>':'')+'</span>'+(!framed&&previous?'<span class="mp-template-state warning" data-channel-pending>上次保存待重启</span>':'');
 }
 function policyStatusAnnotation(){
  const styles={clean:'灰色描边 · 保存弱化并禁用',dirty:'橙色状态框 · 可提交时保存主色强调',staged:'橙色状态框 · 先确认行 / 连接编辑，保存禁用','over-budget':'红色状态框与激活数 · 保存禁用',saving:'蓝色状态框与转动环 · 保存禁用',pending:'橙色状态框 · 重新检查入口可用',failed:'红色状态框 · 草稿保留，可重试保存',stopped:'灰色状态框 · 草稿保留，可继续保存',uncertain:'橙色状态框 · 先核对可能已经写入的结果'};
  return mpAnn('08','渠道列表的九种状态与保存样式',Object.entries(channelPolicyStates).map(([id,[label,tone]])=>'<span class="mp-policy-example-row"><span class="mp-template-state mp-policy-chip '+tone+'" data-policy-example="'+id+'">'+esc(label)+'</span><span>'+styles[id]+'</span></span>').join('<br>')+'<br>从左到右：预览选择器 → 状态 → 还原修改 → 保存；还原修改为次级按钮，无修改或保存中禁用，先确认全部渠道范围，再回到最近成功保存的配置，不等同客户端当前运行配置。列表移除保存与恢复及单项更多入口；导入导出继续在同步页。激活总数与预算在选择器预览弹窗内显示。状态框和保存按钮等高。未保存修改与待重启可同时存在，保存状态和客户端加载 / 连接验证分开。待确认编辑和超限仍阻止保存，即使存在修改也不会伪装成可提交；保存继续先预览全渠道差异再确认。');
 }
 function channelHeadingStatusHTML(p=provider()){
  const badge=(label,tone,attribute)=>'<span class="mp-template-state '+tone+'" '+attribute+'>'+esc(label)+'</span>';
  const kind=p.availability?.kind||'unknown',connectionTone=kind==='available'?'success':kind==='no-credit'?'warning':['auth-failed','connection-failed','unavailable'].includes(kind)?'danger':'neutral';
  const discovery={loading:['正在获取清单','info'],failed:['获取清单失败','danger'],empty:['清单为空','warning']}[p.refresh];
  return channelPolicyStatusHTML()+(p.id==='native'?badge('官方管理','neutral','data-channel-managed'):badge(p.enabled?'已启用':'已停用',p.enabled?'success':'neutral','data-channel-enabled="'+p.enabled+'"')+badge(availabilityText(p),connectionTone,'data-channel-availability="'+esc(kind)+'"'))+(discovery?badge(discovery[0],discovery[1],'data-channel-discovery="'+p.refresh+'"'):'')+'<span class="mp-template-state-context">已选 '+selected(p).length+' / 发现 '+p.models.length+'</span>';
 }
 function channelStatusAnnotation(){return mpAnn('05','渠道状态与保存范围','标题分开显示全渠道保存状态、当前渠道启停、连接状态和清单获取状态，沿用模版详情的彩色标记。<br>保存状态：未修改（中性）；未保存修改 / 有待确认编辑 / 已保存 · 待重启（橙色）；超出激活预算 / 保存失败（红色）；正在保存（蓝色）；保存已取消（中性）；保存结果待核对（橙色）。已有待重启结果时独立保留。<br>启用与连接检查通过为绿色，停用 / 未验证 / 官方管理为中性，无额度为橙色，认证或连接失败为红色。连接检查通过不等于客户端已加载。<br>保存仍先预览全部渠道差异，确认后才写入；待确认行 / 连接编辑和超限会阻止保存。返回只退出详情，不写入配置，工作区草稿保留。撤销 / 重做入口已移除；需要还原草稿时返回渠道列表，点击顶部“还原修改”并确认全部范围。');}
 function updateChannelStatus(){
  const node=$('#mpPolicyStatus',host);if(node)node.outerHTML=status();
  const nodes=$('#mpChannelSection-nodes',host);if(nodes)nodes.innerHTML=serviceNodeList(provider());
  const heading=$('#mpChannelHeadingStatus',host);if(heading){heading.innerHTML=channelHeadingStatusHTML();channelOffset(host.closest('.main'));}else syncTabBar(host.closest('.main'));
 }
 function status(){
  const over=count()>100,staged=stagedModelEdits().length,connections=stagedChannelConnections().length,local=staged||connections,sidebar=tab==='channels'&&channelDetail,changed=dirty()||local,canSave=!(local||over||busy()||!dirty());
  const helper=(name,a,mark)=>iconButton(name,a,mark,busy()?'disabled':'');
  const helpers=(staged?helper('处理 '+staged+' 行待确认编辑','review-model-rows','edit'):'')+(connections?helper('处理 '+connections+' 个待确认连接','review-channel-connection','settings'):'')+(incoming?helper('查看收到的待审候选','incoming','draft'):'')+(pending?helper('重启后重新检查','client-check','refresh'):'');
  const save='<button id="mpPolicySave" class="btn '+(sidebar||canSave?'primary':'ghost')+' mp-nav-action mp-template-save'+(!sidebar?' mp-top-policy-save'+(changed?' is-changed':''):'')+'" data-mp="save" title="保存" aria-label="保存" data-save-scope="all-channels" '+(!canSave?'disabled':'')+'>'+icon('save')+'<span class="mp-nav-action-label">保存</span></button>';
  const reset='<button id="mpPolicyReset" class="btn ghost mp-nav-action mp-top-policy-reset" data-mp="discard" title="'+(busy()?'正在保存，完成后再还原':changed?'还原全部渠道的修改，回到最近成功保存的配置':'没有未保存修改')+'" aria-label="还原修改" '+(busy()||!changed?'disabled':'')+'>'+icon('reset')+'<span class="mp-nav-action-label">还原修改</span></button>';
  const budget=sidebar?channelAction('预览选择器','selector-preview','preview','id="mpBudget" title="预览选择器 · 模型激活数 '+count()+'/100"'): '<button class="mp-budget '+(over?'mp-alert':'')+'" id="mpBudget" data-mp="selector-preview" title="预览选择器 · 模型激活数 '+count()+'/100" aria-label="预览选择器，模型激活数 '+count()+'/100">'+icon('preview')+'<span>预览选择器</span></button>';
  const tools=helpers?'<div class="mp-rail-helpers" role="group" aria-label="需要处理的配置">'+helpers+'</div>':'';
  return '<div id="mpPolicyStatus" class="'+(sidebar?'mp-channel-status':'mp-policy-status')+(over?' over-budget':'')+'">'+(sidebar?'<div class="mp-channel-save-controls">'+save+'</div>'+budget+tools:'<div class="mp-top-policy-controls">'+budget+channelPolicyStatusHTML(true)+'<div class="mp-channel-save-controls">'+reset+save+'</div></div>'+tools+policyStatusAnnotation())+'</div>';
 }
 function channelDraftDirty(){return dirty()||stagedModelEdits().length>0||stagedChannelConnections().length>0;}
 function channelDraftFingerprint(){return JSON.stringify({saved,draft,modelEdits,channelConnections});}
 function discardChannelDrafts(){
  if(busy()){notice('正在保存，完成后再处理草稿');return;}if(!channelDraftDirty()){notice('没有未保存修改');return;}
  const baseline=channelDraftFingerprint(),groups=policyChanges(saved,draft),rows=stagedModelEdits(),connections=stagedChannelConnections(),names=new Set(groups.map(g=>g.name));
  const pendingRows=rows.map(([key])=>{const p=draft.providers.find(p=>p.models.some(m=>m.key===key)),m=p?.models.find(m=>m.key===key);if(p)names.add(p.name+' · '+p.id);return (p?.name||'已移除渠道')+' / '+(m?.id||key)+' · 未确认的模型编辑';});
  const pendingConnections=connections.map(([id])=>{const p=draft.providers.find(p=>p.id===id);names.add((p?.name||id)+' · '+id);return (p?.name||id)+' · 未确认的连接编辑';});
  modal('还原修改','<p>将还原全部渠道的未保存修改和未确认编辑，回到最近成功保存的配置。</p><p><strong>受影响范围：</strong>'+esc([...names].join('；'))+'</p>'+(groups.length?policyDiffHTML(saved,draft,false,true):'')+(pendingRows.length||pendingConnections.length?'<h3>尚未确认的编辑</h3>'+diffHTML([...pendingRows,...pendingConnections]):'')+'<p class="mp-muted">已保存配置、待重启状态、模版版本、Agent 执行设置和收到的待审候选保留。</p><div id="mpDiscardError" class="mp-alert" role="alert"></div>','还原修改',()=>{
   if(busy()||baseline!==channelDraftFingerprint()){$('#mpDiscardError').textContent='配置或未确认编辑已变化，请关闭并重新核对范围。';return;}
   closeModal();draft=clone(saved);modelEdits={};channelConnections={};restoration=null;render();notice('已还原到最近成功保存的配置');
  },{confirmKeepsOpen:true,cancelLabel:'保留修改'});
 }
 function templateSaveStatus(){
  const b=buffer();if(!b)return '';
  const invalid=templateHeadingState().id==='invalid'||!templateEditorChanged();
  return '<div id="mpTemplateSaveStatus" class="mp-template-save-status"><button class="btn primary mp-nav-action mp-template-save" data-mp="save-template" title="保存" aria-label="保存" '+(invalid?'disabled':'')+'>'+icon('save')+'<span class="mp-nav-action-label">保存</span></button></div>';
 }
 function updateTemplateSaveStatus(){
  const node=$('#mpTemplateSaveStatus',host);if(node)node.outerHTML=templateSaveStatus();
  const heading=$('#mpTemplateHeadingStatus',host);if(heading)heading.innerHTML=templateHeadingStatusHTML();
  const error=$('#mpTemplateError',host);if(error&&buffer()?.validationRequested)error.textContent=templateError(buffer());
  const selector=$('#mpTemplateVersion',host),adoptButton=$('#mpTemplateAdopt',host),changed=templateEditorChanged();
  if(selector){selector.disabled=changed;selector.title=changed?'先保存修改或返回放弃编辑，再切换版本':'';}
  if(adoptButton)adoptButton.disabled=changed;
  window.prototypeSelect?.enhance();
  if(templateDetail)editorOffset(host.closest('.main'));
 }
 function syncSaveNotification(){window.modelSaveNotificationBridge?.(operation?{id:operation.id,status:operation.status,step:operation.step,pending}:null);}
 function preview(){
  const policy=draft,rows=projection(policy),base=rows.filter(r=>r.type==='base').length,fast=rows.filter(r=>r.type==='fast').length,hidden=rows.filter(r=>r.type==='hidden').length,over=rows.length>100;
  let groups=policy.providers.map(p=>{
   const locked=p.id==='native',off=!p.enabled;
   const title=locked?'<h3 class="mp-preview-group-title is-locked">'+icon('preview')+'<span>'+esc(p.name)+'</span><small>官方管理 · 只读</small></h3>':off?'<button type="button" class="mp-preview-group-title is-off" data-mp="preview-channel-toggle" data-id="'+esc(p.id)+'" aria-pressed="true" title="点击恢复该渠道"><span class="mp-preview-group-eye">'+icon('preview')+'</span><strong>'+esc(p.name)+'</strong><small>已停用 · 点击恢复</small><span class="mp-preview-group-action">恢复</span></button>':'<button type="button" class="mp-preview-group-title" data-mp="preview-channel-toggle" data-id="'+esc(p.id)+'" aria-pressed="false" title="点击停用该渠道"><span class="mp-preview-group-eye">'+icon('preview')+'</span><strong>'+esc(p.name)+'</strong><small>启用 '+selected(p).length+' / '+p.models.length+'</small><span class="mp-preview-group-action">停用</span></button>';
   if(off)return '<section class="mp-preview-group is-off">'+title+'</section>';
   const items=p.models.map(m=>{const on=modelActive(p,m),supported=resolve(p,m).values.fast==='supported';return '<button type="button" class="mp-preview-item '+(on?'is-on':'is-off')+(supported?' has-fast':'')+'" data-mp="preview-model-toggle" data-id="'+esc(m.key)+'" aria-pressed="'+on+'" title="'+(on?'点击停用':'点击启用')+' '+esc(m.id)+(supported?' · 支持 Fast':'')+'" '+(locked?'disabled':'')+'><span>'+esc(m.id)+'</span>'+(supported?'<em class="mp-preview-fast">Fast</em>':'')+'</button>';}).join('');
   return '<section class="mp-preview-group">'+title+'<div class="mp-preview-items">'+(items||'<p class="mp-muted">该渠道暂无模型。</p>')+'</div></section>';
  });const system=rows.filter(r=>r.type==='hidden');if(system.length)groups.push('<section class="mp-preview-group"><h3>系统占用<small>只读</small></h3><div class="mp-preview-items">'+system.map(r=>'<div class="mp-preview-item hidden-item" title="'+esc(r.id)+'"><span>'+esc(r.id)+'</span></div>').join('')+'</div></section>');groups=groups.join('');
  return '<div class="mp-selector-preview" aria-label="预览选择器"><div id="mpPreviewBudget" class="mp-preview-budget-summary '+(over?'over-budget':'')+'" role="status"><strong>模型激活数 '+rows.length+'/100</strong><span>基础模型 '+base+' · Fast '+fast+' · 系统占用 '+hidden+'</span>'+(over?'<span class="mp-alert">超出激活预算，请减少发布项后再保存。</span>':'')+(pending?'<span class="mp-muted">已保存配置仍待重启 / 客户端核对。</span>':'')+'</div><div class="mp-preview-order" aria-label="完整选择器顺序">'+(groups||'<p class="mp-muted">没有计划发布的目录条目。</p>')+'</div><p class="mp-muted">预览固定当前草稿。点击渠道名称停用 / 恢复该组；点击型号标签启停单个型号，灰色标签保留以便恢复。修改仍由页面「保存」统一提交。</p></div>';
 }
 function selectorPreviewDialog(){
  modal('预览选择器',preview(),'关闭',null,{prototypeAnnotation:{title:'选择器预览与快速启停',body:'预览固定当前草稿，不提供来源切换。点击渠道名称停用或恢复整组；停用后渠道标题置灰并保留恢复入口，模型标签全部隐藏。点击型号标签启停单个型号；停用标签保留灰色，可再次点击恢复。颜色只表达启停，Fast 能力另用文字标记。所有修改进入同一渠道草稿，关闭弹窗保留，页面统一保存后才写入运行配置。系统隐藏项计入预算但不提供启停。100 是原型的兼容阈值，真实上限 / 分页与目录读回待实现核验；本轮仍为内存 mock。'}});
 }
 function refreshPreview(focusSelector=''){
  const body=$('#modalBody'),node=body?.querySelector('.mp-selector-preview');if(!node)return;
  const top=body.scrollTop;body.innerHTML='<div class="mp-dialog">'+preview()+'</div>';window.prototypeSelect?.enhance(body);
  if(focusSelector)body.querySelector(focusSelector)?.focus({preventScroll:true});
  body.scrollTop=top;
 }
 function togglePreviewChannel(pid){
  const p=draft.providers.find(p=>p.id===pid);if(!p||p.id==='native'||busy())return;
  change(()=>p.enabled=!p.enabled);
  refreshPreview('[data-mp="preview-channel-toggle"][data-id="'+pid+'"]');
  notice((p.enabled?'已恢复 ':'已停用 ')+p.name+'；修改保留在当前草稿，页面保存后生效');
 }
 function togglePreviewModel(key){
  const p=draft.providers.find(p=>p.models.some(m=>m.key===key)),m=p?.models.find(m=>m.key===key);
  if(!p||!m||p.id==='native'||busy())return;
  const error=commitModelPatches(p.id,[{key,enabled:!modelActive(p,m)}]);
  if(error){notice(error);return;}
  const nextChannel=draft.providers.find(item=>item.models.some(model=>model.key===key)),nextModel=nextChannel?.models.find(model=>model.key===key);
  refreshPreview('[data-mp="preview-model-toggle"][data-id="'+key+'"]');
  notice((nextChannel&&nextModel&&modelActive(nextChannel,nextModel)?'已启用 ':'已停用 ')+m.id+'；修改保留在当前草稿，页面保存后生效');
 }
 const visibleModels=()=>provider().models.filter(m=>(!onlySelected||selected(provider()).includes(m))&&(!family||templateMatches(templates.find(t=>t.id===family),m,provider()))&&(m.id+' '+(m.naming?.alias||'')+' '+(m.naming?.displayName||'')).toLowerCase().includes(search.toLowerCase()));
 function batchToolbar(){const targets=scopeProviders();if(!targets.length)return '';const hidden=targets.filter(p=>!visibleProviders().includes(p)).length;return '<div class="mp-batch"><span>已选 '+targets.length+' 个渠道'+(hidden?'（含搜索外 '+hidden+' 个）':'')+'</span><div class="mp-actions">'+button('启用','enable-channels',busy()||targets.every(p=>p.enabled)?'disabled':'')+button('停用','disable-channels',busy()||targets.every(p=>!p.enabled)?'disabled':'')+button('删除','delete-channels',busy()?'disabled':'',false,true)+button('按族批量配置','batch',busy()?'disabled':'')+button('套用模版','adopt',busy()?'disabled':'')+'</div></div>';}

 // 复选与展开仅属于界面；批量操作冻结勾选项，快捷排序进入草稿。
 let modelSelection=new Set(),modelExpanded=new Set(),modelEdits={};
 const modelActive=(p,m)=>selected(p).includes(m);
 const pickedModels=(p=provider())=>p.models.filter(m=>modelSelection.has(m.key));
 const modelTargets=(p=provider())=>p.id==='native'?[]:pickedModels(p);
 const modelSelectionState=()=>{const rows=provider().id==='native'?[]:visibleModels(),n=rows.filter(m=>modelSelection.has(m.key)).length;return {rows,checked:rows.length>0&&n===rows.length,mixed:n>0&&n<rows.length};};
 const editBaseline=m=>JSON.stringify({overrides:m.overrides});
 const stagedModelEdits=()=>Object.entries(modelEdits).filter(([key,e])=>JSON.stringify({overrides:e.rules})!==e.baseline);
 const contextPresets=[32000,64000,128000,200000,256000,1000000];
 function namingError(p){
  const seen=new Set();
  for(const m of p.models){const alias=m.naming?.alias;if(m.naming!=null&&(typeof m.naming!=='object'||Array.isArray(m.naming)||Object.entries(m.naming).some(([k,v])=>!['alias','displayName'].includes(k)||typeof v!=='string')))return m.id+'：别名和显示名称须为文本。';if(!alias)continue;
   if(!/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(alias))return m.id+'：别名仅允许字母、数字、点、下划线和连字符，首字符须为字母或数字。';
   if(/^(gpt-|o1-|o3-|o4-|codex-)/i.test(alias))return m.id+'：别名不可使用保留前缀 gpt- / o1- / o3- / o4- / codex-。';
   const value=alias.toLowerCase();if(seen.has(value)||p.models.some(x=>x.key!==m.key&&x.id.toLowerCase()===value))return m.id+'：别名重复或与渠道内其他上游 ID 冲突（不区分大小写）。';seen.add(value);
  }return '';
 }
 function modelRuleError(p,m,policy){
  const r=m.overrides,v=resolve(p,m,policy).values;
  if(r.protocol&&!protocols.includes(r.protocol))return '协议无效。';
  if(Object.hasOwn(r,'context')&&(!Number.isSafeInteger(r.context)||r.context<=0))return '上下文须为正整数 tokens。';
  if(Object.hasOwn(r,'efforts')&&(!Array.isArray(r.efforts)||r.efforts.some(x=>!levels.includes(x))||new Set(r.efforts).size!==r.efforts.length))return '推理档位无效。';
  if((r.default||r.efforts)&&(v.efforts||[]).length&&v.default&&!(v.efforts||[]).includes(v.default))return '默认推理档位不在最终多选列表中，请一并修改默认档位。';
  if(r.default&&!(v.efforts||[]).includes(r.default))return '默认推理档位不在最终多选列表中。';
  if(v.force&&!(v.efforts||[]).includes(v.force))return '强制档位不在最终多选列表中。';
  if(v.force&&v.omit)return '强制档位和不传参数冲突。';
  if(Object.hasOwn(r,'fast')&&!['supported','unsupported','unknown'].includes(r.fast))return '快速模式能力声明无效。';
  return '';
 }
 function commitModelPatches(pid,updates){
  const p=draft.providers.find(p=>p.id===pid);if(busy()||!p||p.id==='native')return '当前渠道不可编辑。';
  if(!updates.length)return '没有可编辑的模型。';
  const next=clone(draft),target=next.providers.find(p=>p.id===pid),active=new Set(selected(target).map(m=>m.key));
  for(const update of updates){const m=target.models.find(m=>m.key===update.key);if(!m)return '模型清单已变化，请重新选择。';
   if(Object.hasOwn(update,'endpointId')||Object.hasOwn(update,'nodeId'))return '模型不单独指定节点，请在渠道的服务节点中切换。';
   if(update.baseline&&update.baseline!==editBaseline(p.models.find(x=>x.key===update.key)))return '该模型的配置已变化，请还原本行后重新编辑。';
   if(update.rules)m.overrides=clone(update.rules);
   if(Object.hasOwn(update,'naming')){const naming=Object.fromEntries(Object.entries(update.naming||{}).map(([k,v])=>[k,String(v).trim()]).filter(([,v])=>v));if(Object.keys(naming).length)m.naming=naming;else delete m.naming;}
   if(Object.hasOwn(update,'enabled')){update.enabled?active.add(m.key):active.delete(m.key);}
  }
  if(updates.some(u=>Object.hasOwn(u,'enabled')&&u.enabled!==modelActive(p,p.models.find(m=>m.key===u.key)))){target.models.forEach(m=>m.selected=active.has(m.key));target.mode=active.size?'whitelist':'none';}
  const aliasError=namingError(target);if(aliasError)return aliasError;
  for(const u of updates){const m=target.models.find(m=>m.key===u.key),error=u.rules?modelRuleError(target,m,next):'';if(error)return m.id+'：'+error;}
  if(JSON.stringify(next)!==JSON.stringify(draft))change(()=>{draft=next;updates.filter(u=>u.rules).forEach(u=>delete modelEdits[u.key]);});else {updates.filter(u=>u.rules).forEach(u=>delete modelEdits[u.key]);render();}
  return '';
 }
 function modelError(key,error){const node=$('[data-model-error="'+key+'"]',host);if(node)node.textContent=error;else notice(error);}
 function startModelEdit(m){return modelEdits[m.key]||(modelEdits[m.key]={rules:clone(m.overrides),baseline:editBaseline(m)});}
 function quickConfig(rules,inherited,prefix,bulk=false){
  const effective=inherited?.values||{},mode=key=>Object.hasOwn(rules,key)?'custom':bulk?'keep':'inherit',modes=key=>opts([...(bulk?[['keep','不修改']]:[]),['inherit','继承'],['custom','自定义']],mode(key));
  const val=key=>rules[key]??effective[key],control=(key,title,content)=>'<div class="mp-quick-field" data-quick-group="'+key+'"><div class="mp-quick-label"><span>'+title+'</span><select id="'+prefix+'Mode-'+key+'" class="mp-select" data-quick-mode="'+key+'" aria-label="'+title+'设置方式">'+modes(key)+'</select></div><div data-quick-control="'+key+'" '+(mode(key)!=='custom'?'hidden':'')+'>'+content+'</div>'+(bulk?'':'<small class="mp-muted">当前 '+esc(ruleText(key,val(key)))+' · '+esc(Object.hasOwn(rules,key)?'单模型自定义':inherited?.sources[key]||'未声明')+'</small>')+'</div>';
  const context=val('context'),preset=contextPresets.includes(context)?String(context):'custom',efforts=val('efforts')||[];
  return '<div class="mp-quick-grid">'+control('protocol','协议','<select id="'+prefix+'Protocol" class="mp-select" data-quick-value="protocol" aria-label="模型协议">'+protocolOptions(val('protocol')||'Responses')+'</select>')+
   control('context','上下文窗口','<select id="'+prefix+'ContextPreset" class="mp-select" data-quick-preset aria-label="上下文大小选择">'+opts([...contextPresets.map(n=>[n,(n/1000)+'k tokens']),['custom','自定义 tokens']],preset)+'</select><input id="'+prefix+'Context" class="input" type="number" min="1" step="1" data-quick-value="context" aria-label="自定义上下文 tokens" value="'+(context??'')+'" '+(preset!=='custom'?'hidden':'')+'>')+
   control('efforts','推理等级','<select id="'+prefix+'Efforts" class="mp-select" multiple data-quick-value="efforts" aria-label="可选推理等级（多选）">'+levels.map(x=>'<option value="'+x+'" '+(efforts.includes(x)?'selected':'')+'>'+labels[x]+'</option>').join('')+'</select><small class="mp-muted">多选至少保留一项；不显示档位请在编辑明细中设置。</small>')+
   control('default','默认推理等级','<select id="'+prefix+'Default" class="mp-select" data-quick-value="default" aria-label="默认推理等级">'+opts(levels.map(x=>[x,labels[x]]),val('default')||'medium')+'</select>')+
   control('fast','支持快速模式','<select id="'+prefix+'Fast" class="mp-select" data-quick-value="fast" aria-label="快速模式能力">'+opts([['unknown','未知'],['supported','支持'],['unsupported','不支持']],val('fast')||'unknown')+'</select><small class="mp-muted">能力声明；实际启用遵循快速模式策略。</small>')+'</div>';
 }
 function quickRuleError(rules,original={}){return rules.efforts?.length===0&&original.efforts?.length!==0?'请至少选择一个推理等级，或恢复继承；如需不显示档位，请在“编辑明细”的高级区明确设置。':'';}
 function readQuick(root,original={}){
  const rules=clone(original);$$('[data-quick-mode]',root).forEach(el=>{const k=el.dataset.quickMode;if(el.value==='keep')return;if(el.value==='inherit'){delete rules[k];return;}
   const field=$('[data-quick-value="'+k+'"]',root);rules[k]=k==='efforts'?[...field.selectedOptions].map(x=>x.value):k==='context'?Number($('[data-quick-preset]',root).value==='custom'?field.value:$('[data-quick-preset]',root).value):field.value;
  });return rules;
 }
 function quickChanged(el){if(busy())return;const root=el.closest('[data-model-quick],[data-model-batch]');if(!root)return;
  if(el.dataset.quickMode){$('[data-quick-control="'+el.dataset.quickMode+'"]',root).hidden=el.value!=='custom';}
  if(el.matches('[data-quick-preset]'))$('[data-quick-value="context"]',root).hidden=el.value!=='custom';
  if(root.dataset.modelQuick){const m=provider().models.find(m=>m.key===root.dataset.modelQuick);if(!m)return;const edit=startModelEdit(m);edit.rules=readQuick(root,edit.rules);const error=quickRuleError(edit.rules,m.overrides)||modelRuleError(provider(),{...m,overrides:edit.rules},draft);$('[data-model-quick-error]',root).textContent=error;$('[data-model-pending]',root).textContent='本行待确认 · 加入草稿后统一保存';updateChannelStatus();}
 }
 function modelQuick(p,m){const edit=startModelEdit(m),prefix='mpQuick-'+m.key.replace(/[^A-Za-z0-9_-]/g,'-'),inherited=resolve(p,m,draft,true);
  return '<div id="'+prefix+'" class="mp-model-quick" data-model-quick="'+esc(m.key)+'">'+quickConfig(edit.rules,inherited,prefix)+'<p class="mp-muted">未修改的字段保持继承；上下文声明不会扩容。切换协议后需核对推理和快速模式；访问地址统一使用渠道当前节点。</p><div class="mp-alert" data-model-quick-error="'+esc(m.key)+'" role="alert"></div><div class="mp-quick-footer"><span class="mp-muted" data-model-pending>本行编辑后加入草稿，再统一保存</span><div class="mp-actions">'+button('还原本行','reset-model-row','data-id="'+esc(m.key)+'"')+button('加入草稿','apply-model-row','data-id="'+esc(m.key)+'"',true)+'</div></div></div>';
 }
 function modelRow(p,m,filtered){const r=resolve(p,m),i=p.models.indexOf(m),readonly=p.id==='native',locked=busy()||readonly,expanded=modelExpanded.has(m.key),enabled=modelActive(p,m);
  return '<div class="mp-model-item"><div class="mp-model-row" data-model="'+esc(m.key)+'" '+(!filtered&&!locked?'draggable="true"':'')+'><div class="mp-row"><input type="checkbox" data-model-pick="'+esc(m.key)+'" aria-label="批量选择 '+esc(m.id)+'" '+(modelSelection.has(m.key)?'checked ':'')+(locked?'disabled':'')+'><span class="mp-grip" title="拖动排序">⠿</span></div><div class="mp-model-identity"><strong title="'+esc(m.id)+'">'+esc(m.id)+'</strong><input class="input mp-model-alias" data-model-alias="'+esc(m.key)+'" aria-label="'+esc(m.id)+' 别名" placeholder="别名（可选）" value="'+esc(m.naming?.alias||'')+'" '+(locked?'disabled':'')+'><span class="mp-model-meta" title="'+esc(templateLabel(p,m))+' · '+esc(r.sources.protocol)+'"><span class="mp-model-meta-line"><span>'+esc(ruleText('protocol',r.values.protocol))+'</span><span>'+esc(r.values.context?ruleText('context',r.values.context):'上下文未知')+'</span></span><span class="mp-model-meta-line"><span>推理 '+esc(ruleText('efforts',r.values.efforts))+'</span><span>快速 '+esc(ruleText('fast',r.values.fast))+'</span></span>'+(r.route.error?'<span class="mp-alert">需配置节点或协议</span>':'')+'</span><span class="mp-alert" data-model-error="'+esc(m.key)+'" role="alert"></span></div><div class="mp-model-enabled">'+(readonly?'<span class="mp-muted">官方管理</span>':'<button class="toggle" role="switch" type="button" data-mp="toggle-model" data-id="'+esc(m.key)+'" aria-label="'+esc(m.id)+' 启用状态" aria-checked="'+enabled+'" '+(busy()?'disabled':'')+'></button><small>'+ (enabled?'启用':'停用')+'</small>')+'</div><div class="mp-model-actions">'+(readonly?'':button((expanded?'收起':'展开')+'编辑','expand-model','data-id="'+esc(m.key)+'" aria-expanded="'+expanded+'" aria-controls="mpQuick-'+m.key.replace(/[^A-Za-z0-9_-]/g,'-')+'" '+(busy()?'disabled':'')))+button(readonly?'查看明细':'编辑明细','model-config','data-id="'+esc(m.key)+'" '+(busy()?'disabled':''))+'<div class="mp-mini-pair">'+mini('↑','move-model','data-id="'+esc(m.key)+'" data-direction="-1" aria-label="上移 '+esc(m.id)+'" '+(i===0||filtered||locked?'disabled':''))+mini('↓','move-model','data-id="'+esc(m.key)+'" data-direction="1" aria-label="下移 '+esc(m.id)+'" '+(i===p.models.length-1||filtered||locked?'disabled':''))+'</div></div></div>'+(expanded&&!readonly?modelQuick(p,m):'')+'</div>';
 }
 const modelSortOptions=[['name-asc','名称 A → Z'],['name-desc','名称 Z → A'],['alias-asc','别名 A → Z'],['protocol','协议分组'],['context-desc','上下文 大 → 小'],['context-asc','上下文 小 → 大'],['enabled','启用优先']];
 function sortModels(rule){
  const p=provider();if(busy()||p.id==='native'||search||family||onlySelected||!modelSortOptions.some(([key])=>key===rule))return;
  const compareText=(a,b)=>String(a).localeCompare(String(b),'zh-CN',{numeric:true,sensitivity:'base'}),rows=p.models.map((m,index)=>({m,index,values:resolve(p,m).values,enabled:modelActive(p,m)}));
  const compareContext=(a,b)=>{const av=a.values.context,bv=b.values.context,knownA=Number.isSafeInteger(av)&&av>0,knownB=Number.isSafeInteger(bv)&&bv>0;return knownA!==knownB?(knownA?-1:1):!knownA?0:rule==='context-desc'?bv-av:av-bv;};
  const compare=(a,b)=>rule==='name-asc'?compareText(a.m.id,b.m.id):rule==='name-desc'?compareText(b.m.id,a.m.id):rule==='alias-asc'?compareText(a.m.naming?.alias||a.m.id,b.m.naming?.alias||b.m.id):rule==='protocol'?compareText(adapters[a.values.protocol]||a.values.protocol||'',adapters[b.values.protocol]||b.values.protocol||''):rule.startsWith('context-')?compareContext(a,b):Number(b.enabled)-Number(a.enabled);
  const next=rows.sort((a,b)=>compare(a,b)||a.index-b.index).map(row=>row.m);
  if(next.every((m,index)=>m.key===p.models[index].key)){render();notice('模型顺序已符合所选规则');return;}
  change(()=>p.models=next);notice('模型顺序已加入草稿，保存后生效');
 }
 function modelBatchToolbar(){const p=provider(),rows=visibleModels(),picked=pickedModels(p),hidden=picked.filter(m=>!rows.includes(m)).length,locked=busy()||p.id==='native',filtered=search||family||onlySelected;
  const tools='<div class="mp-model-tools"><div class="mp-actions">'+button('全部展开','expand-all-models','title="展开当前显示的模型" '+(locked||!rows.length||rows.every(m=>modelExpanded.has(m.key))?'disabled':''))+button('全部收起','collapse-all-models','title="收起当前渠道全部模型，保留待确认的编辑" '+(locked||!p.models.some(m=>modelExpanded.has(m.key))?'disabled':''))+'</div><div class="mp-actions"><select id="mpModelSort" class="mp-select" data-model-sort aria-label="快捷排序" title="'+(filtered?'清除筛选后可调整整个渠道的模型顺序':'调整当前渠道完整顺序；名称按模型 ID，别名为空使用 ID，未知上下文排最后；保存后生效')+'" '+(locked||filtered||rows.length<2?'disabled':'')+'>'+opts([['','快捷排序'],...modelSortOptions],'')+'</select>'+(filtered?button('清除筛选后排序','clear-filters'):'')+'</div></div>';
  return tools+(!picked.length||p.id==='native'?'':'<div class="mp-model-bulk" role="region" aria-label="已勾选模型的批量操作"><span class="mp-muted">已选 '+picked.length+(hidden?'（含筛选外 '+hidden+' 项）':'')+'</span><div class="mp-actions">'+button('批量启用','enable-models',locked?'disabled':'')+button('批量停用','disable-models',locked?'disabled':'')+button('批量编辑','edit-models',locked?'disabled':'')+button('恢复字段继承','reset-models',locked?'disabled':'')+button('清除选择','clear-model-picks',locked?'disabled':'')+'</div></div>');
 }
 function bulkModelDialog(action){const p=provider(),targets=modelTargets().map(m=>clone(m));if(busy()||p.id==='native'||!targets.length)return;
  const digest=policyDigest(),pid=p.id,scopeLabel='勾选模型',hidden=targets.filter(m=>!visibleModels().some(x=>x.key===m.key)).length;
  if(targets.some(m=>stagedModelEdits().some(([key])=>key===m.key))){notice('勾选项内有未加入草稿的行编辑，请先确认或还原本行。');return;}
  const summary='<p>'+esc(p.name)+' · '+scopeLabel+' · '+targets.length+' 项'+(hidden?'（包含筛选外 '+hidden+' 项）':'')+'</p><details class="mp-details"><summary>查看完整操作范围</summary>'+diffHTML(targets.map(m=>m.id))+'</details>';
  const editing=action==='edit',reset=action==='reset';
  const body=editing?'<div data-model-batch>'+quickConfig({},null,'mpBulk',true)+'<div class="mp-bulk-extra"><label>启停<select id="mpBulkEnabled" class="mp-select">'+opts([['keep','不修改'],['on','启用'],['off','停用']],'keep')+'</select></label><label>别名<select id="mpBulkAliasMode" class="mp-select">'+opts([['keep','不修改'],['clear','清空别名'],['pattern','按规则生成']],'keep')+'</select></label><label>别名规则<input id="mpBulkAliasPattern" class="input" placeholder="例如 model-{n}"><small class="mp-muted">{id} = 原始 ID，{n} = 范围内序号；先校验重复与合法性。</small></label></div></div>':reset?'<p>清除单模型的字段覆盖和指定端点，继续继承父层。别名、启停、精确模版引用和顺序保留。</p>':'<p>仅修改模型启停，保留别名、配置与顺序。'+(action==='disable'?'停用后从计划选择器移除。':'渠道本身停用时，这些模型在启用渠道后才出现。')+'</p>';
  modal(editing?'批量编辑模型':reset?'恢复模型字段继承':action==='enable'?'批量启用模型':'批量停用模型',summary+body+'<div id="mpModelBatchError" class="mp-alert" role="alert"></div><p class="mp-muted">确认后一次加入草稿；统一保存后提示重启。</p>','加入草稿',()=>{
   const errorNode=$('#mpModelBatchError');if(busy()||digest!==policyDigest()){errorNode.textContent='配置已变化，请重新打开并确认范围。';return;}
   const root=$('[data-model-batch]',$('#modalBody')),enabled=editing?$('#mpBulkEnabled').value:action==='enable'?'on':action==='disable'?'off':'keep',aliasMode=editing?$('#mpBulkAliasMode').value:'keep',pattern=editing?$('#mpBulkAliasPattern').value.trim():'';
   if(aliasMode==='pattern'&&!pattern){errorNode.textContent='请填写别名规则。';return;}
   const updates=targets.map((m,i)=>({key:m.key,...(editing?{rules:readQuick(root,m.overrides)}:reset?{rules:{}}:{}),...(enabled!=='keep'?{enabled:enabled==='on'}:{}),...(aliasMode!=='keep'?{naming:{...m.naming,alias:aliasMode==='clear'?'':pattern.replaceAll('{id}',m.id).replaceAll('{n}',String(i+1))}}:{})}));
   const empty=editing&&updates.find((u,i)=>quickRuleError(u.rules,targets[i].overrides));const error=empty?empty.key+'：'+quickRuleError(empty.rules,targets.find(m=>m.key===empty.key).overrides):commitModelPatches(pid,updates);if(error){errorNode.textContent=error;return;}closeModal();notice(targets.length+' 个模型已加入草稿');
  },{confirmKeepsOpen:true});
 }

 const channelSections=[['connection','渠道配置'],['nodes','服务节点'],['defaults','模型默认'],['models','模型列表']];
 const connectionValue=p=>({name:p.name,protocol:p.protocol,connection:nodeConnection(p)});
 const connectionBaseline=p=>JSON.stringify(connectionValue(p));
 function channelConnectionBuffer(p=provider()){
  if(!p||p.id==='native')return null;
  return channelConnections[p.id]||=({...connectionValue(p),baseline:connectionBaseline(p),retainedKeyConfigured:!!p.connection?.keyConfigured,keyChanged:0});
 }
 const stagedChannelConnections=()=>Object.entries(channelConnections).filter(([id,b])=>draft.providers.some(p=>p.id===id)&&(b.keyChanged||JSON.stringify({name:b.name,protocol:b.protocol,connection:b.connection})!==b.baseline));
 function updateChannelConnection(el){
  if(busy()||provider()?.id==='native')return;const b=channelConnectionBuffer(),key=el.dataset.channelConfig;
  if(key==='key'){if(el.value){b.keyChanged++;b.retainedKeyConfigured=true;}b.connection.keyConfigured=b.connection.auth==='apikey'&&b.retainedKeyConfigured;}
  else if(key==='name'||key==='protocol')b[key]=el.value;
  else if(key==='auth'){b.connection.auth=el.value;b.connection.keyConfigured=el.value==='apikey'&&b.retainedKeyConfigured;}
  if(key==='auth')render();else {updateChannelStatus();const changed=stagedChannelConnections().some(([id])=>id===provider().id),stale=b.baseline!==connectionBaseline(provider()),reset=$('#mpChannelReset',host),pendingLabel=$('#mpChannelConnectionPending',host),keyStatus=$('#mpChannelKeyStatus',host);if(reset)reset.disabled=!changed&&!stale;if(pendingLabel)pendingLabel.textContent=changed?'有连接修改，尚未加入草稿':'连接配置与草稿一致';if(keyStatus)keyStatus.textContent=b.keyChanged?'已记录替换意图，不保存输入值。':'仅记录已配置状态，不保存输入值。';}
 }
 function inlineConnection(p){
  if(p.id==='native')return '<h3>渠道配置</h3><p class="mp-muted">官方 / 原生连接由 Codex 管理，这里仅展示模型信息。</p>';
  const b=channelConnectionBuffer(p),locked=busy()?'disabled':'',changed=stagedChannelConnections().some(([id])=>id===p.id),stale=b.baseline!==connectionBaseline(p);
  const input=(key,value,extra='')=>'<input id="mpChannelInline-'+key+'" class="input" data-channel-config="'+key+'" value="'+esc(value)+'" '+extra+' '+locked+'>';
  return '<h3>渠道配置</h3><div class="mp-channel-form"><label>渠道名称'+input('name',b.name,'maxlength="80"')+'</label><label>渠道标识<input class="input" value="'+esc(p.id)+'" readonly aria-label="渠道标识"><span class="mp-muted">标识固定，名称可修改。</span></label><label>默认接口协议<select id="mpChannelInline-protocol" class="mp-select" data-channel-config="protocol" '+locked+'>'+protocolOptions(b.protocol)+'</select></label><label>认证方式<select id="mpChannelInline-auth" class="mp-select" data-channel-config="auth" '+locked+'>'+opts([['apikey','API Key'],['none','无需认证']],b.connection.auth)+'</select></label>'+(b.connection.auth==='apikey'?'<label class="mp-channel-key">API Key'+input('key','','type="password" autocomplete="off" placeholder="'+(b.connection.keyConfigured?'已配置（不回显）；留空保留':'仅填演示值，请勿输入真实密钥')+'"')+'<span id="mpChannelKeyStatus" class="mp-muted">'+(b.keyChanged?'已记录替换意图，不保存输入值。':'仅记录已配置状态，不保存输入值。')+'</span></label>':'')+'</div><p class="mp-muted">认证与 Key 由全部服务节点共用；地址在服务节点中维护。更换协议或认证后，确认模型清单再加入草稿。静态演示，不连接上游；请勿输入真实密钥。</p>'+(stale?'<p class="mp-alert">连接基准已变化，请还原配置后重新编辑。</p>':'')+'<div class="mp-quick-footer"><span id="mpChannelConnectionPending" class="mp-muted">'+(changed?'有连接修改，尚未加入草稿':'连接配置与草稿一致')+'</span><div class="mp-actions">'+button('还原配置','reset-channel-connection','id="mpChannelReset" '+(locked||!changed&&!stale?'disabled':''))+button('确认连接与模型','apply-channel-connection',locked||stale?'disabled':'',true)+'</div></div>';
 }
 function nodeSpeedKey(pid,nid){return pid+'::'+nid;}
 function nodeSpeedFingerprint(p,n){return JSON.stringify([n?.url,p.protocol,p.connection,nodeAuthRevision[p.id]||0]);}
 function nodeSpeedResult(p,n){const r=nodeSpeedResults[nodeSpeedKey(p.id,n.id)];return r?.fingerprint===nodeSpeedFingerprint(p,n)?r:null;}
 function nodeSpeedStatusHTML(p,n,detect=false){
  const r=n&&nodeSpeedResult(p,n),state=r?.status||'untested';
  const words={untested:'未测速',waiting:'等待测速',running:'测速中…',success:r?.ms+' ms',failed:'连接失败',timeout:'超时',cancelled:'已停止',invalid:'地址待配置',auth:'认证待配置'};
  const tone=state==='success'?'success':['failed','invalid','auth'].includes(state)?'danger':state==='timeout'?'warning':['waiting','running'].includes(state)?'info':'neutral';
  const label=detect?String(words[state]).replaceAll('测速','检测'):words[state];
  return '<span class="mp-node-speed '+tone+'" data-speed-status="'+state+'" title="'+esc(r?.finishedAt?'接口响应耗时 · '+r.finishedAt:'接口响应耗时')+'">'+esc(label)+'</span>';
 }
 function nodeSpeedSummaryHTML(p){
  const run=nodeSpeedRuns[p.id];if(!run)return '';
  const done=run.items.filter(x=>!['waiting','running','cancelled'].includes(x.status)).length,stale=run.items.filter(x=>x.status==='stale').length;
  return '<span class="mp-muted">'+(run.status==='running'?'正在测速':run.status==='cancelled'?'测速已停止':'测速完成')+' '+done+' / '+run.items.length+(stale?' · '+stale+' 项配置已变化':'')+'</span>';
 }
 function nodeSpeedButton(p,nid='',all=false,detect=false){
  const running=nodeSpeedRuns[p.id]?.status==='running',target=nid?serviceNodes(p).find(n=>n.id===nid):activeNode(p),item=running&&nodeSpeedRuns[p.id].items.find(x=>x.nodeId===target?.id);
  const baseLabel=all?(running?'停止测速':'一键测速'):item?.status==='running'?'测速中…':'测速',label=detect?baseLabel.replaceAll('测速','检测'):baseLabel;
  const disabled=all?(!running&&(busy()||!serviceNodes(p).length)):busy()||running||!target;
  return button(label,all?(running?'node-speed-stop':'node-speed-all'):(nid?'node-speed-one':'node-speed-current'),(detect?'id="'+(all?'mpNodePickerAll':'mpNodePickerTest-'+esc(nid))+'" ':'')+'data-channel="'+esc(p.id)+'" '+(nid?'data-id="'+esc(nid)+'" ':'')+'data-speed-control="'+(all?'all':nid?'one':'current')+'" '+(detect?'data-speed-word="detect" ':'')+'aria-label="'+esc(p.name+' · '+(all?(running?'停止全部节点测速':'全部节点测速'):(target?.name||'当前节点')+'测速'))+'" '+(disabled?'disabled':''));
 }
 function quickNodeControls(p){
  if(p.id==='native')return '<span class="mp-muted">官方管理</span>';
  const nodes=serviceNodes(p),blocked=busy()||stagedChannelConnections().some(([id])=>id===p.id),active=activeNode(p);
  return '<button id="mpChannelNode-'+esc(p.id)+'" class="mp-node-picker-trigger" data-mp="node-picker-open" data-channel="'+esc(p.id)+'" aria-haspopup="dialog" aria-expanded="false" aria-label="'+esc(p.name)+' 服务节点选择与检测" title="'+esc(active?active.name+' · '+active.url:'选择服务节点')+'" '+(blocked||!nodes.length?'disabled':'')+'><span class="mp-node-picker-value">'+esc(active?.name||'请选择节点')+'</span><span id="mpNodeSpeedCurrent-'+esc(p.id)+'" class="mp-node-picker-current-speed">'+nodeSpeedStatusHTML(p,active,true)+'</span><span class="mp-node-picker-chevron" aria-hidden="true">▾</span></button>';
 }
 function nodePickerHTML(p){
  const blocked=busy()||stagedChannelConnections().some(([id])=>id===p.id);
  return window.PrototypeNodeView.picker({titleId:'mpNodePickerTitle',title:p.name+' · 服务节点',close:button('关闭','node-picker-close','aria-label="关闭节点选择器"'),all:nodeSpeedButton(p,'',true,true),summaryId:'mpNodeSpeedSummary-'+p.id,summary:nodeSpeedSummaryHTML(p),rows:serviceNodes(p).map(n=>window.PrototypeNodeView.row({id:n.id,choiceId:'mpNodePickerChoice-'+n.id,choiceAttrs:'data-mp="node-picker-choose" data-channel="'+esc(p.id)+'" data-id="'+esc(n.id)+'"',selected:n.id===p.activeNodeId,title:n.name+' · '+n.url,disabled:blocked,name:n.name,speedId:'mpNodePickerSpeed-'+n.id,speed:nodeSpeedStatusHTML(p,n,true),test:nodeSpeedButton(p,n.id,false,true)})).join('')});
 }
 function closeNodePicker(focus=false){
  if(!nodePicker)return;const {element,anchor,pid}=nodePicker;nodePicker=null;
  try{if(element.matches?.(':popover-open'))element.hidePopover();}catch(_){}
  element.remove();anchor?.setAttribute('aria-expanded','false');
  if(focus)$('#mpChannelNode-'+pid,host)?.focus({preventScroll:true});
 }
 function positionNodePicker(){
  if(!nodePicker)return;const {element,anchor}=nodePicker,rect=anchor.getBoundingClientRect();
  const viewport={left:0,top:0,right:window.innerWidth||1280,bottom:window.innerHeight||800,width:window.innerWidth||1280,height:window.innerHeight||800};
  const bounds=$('.window')?.getBoundingClientRect()||viewport,width=Math.max(160,Math.min(400,bounds.width-16)),maxHeight=Math.max(120,bounds.bottom-bounds.top-16);
  element.style.width=width+'px';element.style.maxHeight=maxHeight+'px';
  const size=element.getBoundingClientRect(),left=Math.max(bounds.left+8,Math.min(rect.left||bounds.left,bounds.right-width-8)),top=rect.bottom+4+size.height<=bounds.bottom-8?rect.bottom+4:Math.max(bounds.top+8,rect.top-size.height-4);
  element.style.left=left+'px';element.style.top=top+'px';
 }
 function openNodePicker(pid,anchor=$('#mpChannelNode-'+pid,host)){
  const p=draft.providers.find(p=>p.id===pid);if(!p||p.id==='native'||!anchor||anchor.disabled||busy()||stagedChannelConnections().some(([id])=>id===pid))return;
  if(nodePicker?.pid===pid){closeNodePicker(true);return;}
  closeNodePicker();window.prototypeActionMenu?.close();window.prototypeSelect?.close();
  const element=document.createElement('div');element.id='mpNodePicker';element.className='mp-node-picker';element.setAttribute('role','dialog');element.setAttribute('aria-modal','false');element.setAttribute('aria-labelledby','mpNodePickerTitle');element.innerHTML=nodePickerHTML(p);document.body.appendChild(element);
  nodePicker={pid,anchor,element};anchor.setAttribute('aria-controls','mpNodePicker');anchor.setAttribute('aria-expanded','true');
  if(typeof element.showPopover==='function'){element.setAttribute('popover','manual');try{element.showPopover();}catch(_){element.removeAttribute('popover');}}
  positionNodePicker();element.querySelector('[data-mp="node-picker-choose"][aria-pressed="true"]')?.focus({preventScroll:true});
 }
 function chooseNodeFromPicker(pid,nid){
  if(nodePicker?.pid!==pid)return;
  const p=draft.providers.find(p=>p.id===pid);if(!nodeEditable(p)||!serviceNodes(p).some(n=>n.id===nid))return;
  closeNodePicker();useServiceNode(nid,pid);$('#mpChannelNode-'+pid,host)?.focus({preventScroll:true});
 }
 function serviceNodeList(p){
  if(p.id==='native')return '<h3>服务节点</h3><p class="mp-muted">官方 / 原生连接由 Codex 管理。</p>';
  const nodes=serviceNodes(p),locked=busy()||stagedChannelConnections().some(([id])=>id===p.id);
  return '<div class="mp-toolbar"><h3>服务节点</h3><div class="mp-actions">'+nodeSpeedButton(p,'',true)+button('新增节点','add-node',locked?'disabled':'')+'</div></div><p class="mp-muted">所有节点共用渠道认证与 API Key；模型统一使用当前节点。</p><div id="mpNodeSpeedSummary-'+esc(p.id)+'" role="status">'+nodeSpeedSummaryHTML(p)+'</div>'+nodes.map(n=>'<div class="mp-service-node-row"><div><strong>'+esc(n.name)+(n.id===p.activeNodeId?' <span class="mp-template-state info">当前使用</span>':'')+'</strong><small>'+esc(n.url||'待配置地址')+'</small><div id="mpNodeSpeed-'+esc(p.id)+'-'+esc(n.id)+'" role="status">'+nodeSpeedStatusHTML(p,n)+'</div></div><div class="mp-actions">'+nodeSpeedButton(p,n.id)+(n.id===p.activeNodeId?'':button('设为当前','use-node','data-id="'+esc(n.id)+'" '+(locked?'disabled':'')))+button('编辑','edit-node','data-id="'+esc(n.id)+'" '+(locked?'disabled':''))+button('删除','delete-node','data-id="'+esc(n.id)+'" '+(locked||n.id===p.activeNodeId||nodes.length<=1?'disabled':'') ,false,true)+'</div></div>').join('')+(nodes.length?'':'<p class="mp-alert">尚无服务节点，请添加地址。</p>')+mpAnn('06','服务节点、测速与协议的边界','一个渠道维护一份认证和 Key，节点仅含名称及 URL。新增渠道的 Base URL 自动成为默认节点；后续地址只在这里编辑。<br>列表服务节点独立成列，单行触发器显示当前节点及耗时；选择器内逐节点展示检测结果 / 单独检测，顶部一键检测 / 停止。选节点与检测动作独立，检测不自动选择。详情支持单节点 / 一键测速。接口响应耗时以 ms 显示，不代表模型生成速度、模型能力或额度。测速结果仅本机会话，分别显示等待 / 测速中 / 成功 / 失败 / 超时 / 停止。<br>本原型使用确定性 mock，不请求网络、不读取或发出真实 Key；结果场景在右侧工具板设置。一键测速最多两个模拟任务并行，可停止，结果不自动切节点。节点地址、协议或认证变化后旧结果失效，晚回不会覆盖新配置。<br>节点切换加入全渠道草稿，保存确认展示地址变化，成功后仍需重启 / 核对。节点不复制模型清单或凭据，不为每个模型分别指定。当前节点不可直接删除，至少保留一条；未确认渠道配置先处理。没有自动故障转移或请求重发，真实测速的轻量探测与适配方法待实施。');
 }
 // 测速是会话级观察，独立于策略、保存和连接验证；任何晚回都核对测量对象。
 function nodeSpeedUI(){
  for(const p of draft.providers.filter(p=>p.id!=='native')){
   const currentNode=$('#mpNodeSpeedCurrent-'+p.id,host);if(currentNode)currentNode.innerHTML=nodeSpeedStatusHTML(p,activeNode(p),true);
   for(const n of serviceNodes(p)){const cell=$('#mpNodeSpeed-'+p.id+'-'+n.id,host);if(cell)cell.innerHTML=nodeSpeedStatusHTML(p,n);}
   const summary=$('#mpNodeSpeedSummary-'+p.id,host);if(summary)summary.innerHTML=nodeSpeedSummaryHTML(p);
  }
  if(nodePicker){const p=draft.providers.find(p=>p.id===nodePicker.pid);if(!p){closeNodePicker();}else{for(const n of serviceNodes(p)){const cell=$('#mpNodePickerSpeed-'+n.id,nodePicker.element);if(cell)cell.innerHTML=nodeSpeedStatusHTML(p,n,true);}const summary=$('#mpNodeSpeedSummary-'+p.id,nodePicker.element);if(summary)summary.innerHTML=nodeSpeedSummaryHTML(p);}}
  [host,...(nodePicker?[nodePicker.element]:[])].flatMap(root=>$$('[data-speed-control]',root)).forEach(el=>{
   const p=draft.providers.find(p=>p.id===el.dataset.channel);if(!p)return;
   const all=el.dataset.speedControl==='all',running=nodeSpeedRuns[p.id]?.status==='running',n=el.dataset.id?serviceNodes(p).find(n=>n.id===el.dataset.id):activeNode(p),item=running&&nodeSpeedRuns[p.id].items.find(x=>x.nodeId===n?.id);
   el.dataset.mp=all?(running?'node-speed-stop':'node-speed-all'):el.dataset.speedControl==='one'?'node-speed-one':'node-speed-current';
   const text=all?(running?'停止测速':'一键测速'):item?.status==='running'?'测速中…':'测速';el.textContent=el.dataset.speedWord==='detect'?text.replaceAll('测速','检测'):text;
   el.setAttribute('aria-label',p.name+' · '+(all?(running?'停止全部节点测速':'全部节点测速'):(n?.name||'当前节点')+'测速'));
   el.disabled=all?(!running&&(busy()||!serviceNodes(p).length)):busy()||running||!n;
  });
 }
 function startNodeSpeed(pid,nid=''){
  const p=draft.providers.find(p=>p.id===pid);if(!p||p.id==='native'||busy()||nodeSpeedRuns[pid]?.status==='running')return;
  if(stagedChannelConnections().some(([id])=>id===pid)){notice('先确认或还原渠道认证配置，再测速');return;}
  const targets=nid?serviceNodes(p).filter(n=>n.id===nid):serviceNodes(p);if(!targets.length)return;
  const run={id:++nodeSpeedSerial,status:'running',scene:nodeSpeedScene,items:targets.map(n=>({nodeId:n.id,url:n.url,fingerprint:nodeSpeedFingerprint(p,n),status:'waiting'}))};nodeSpeedRuns[pid]=run;
  run.items.forEach(item=>nodeSpeedResults[nodeSpeedKey(pid,item.nodeId)]={status:'waiting',fingerprint:item.fingerprint});
  let cursor=0,inflight=0;
  const pump=()=>{
   if(nodeSpeedRuns[pid]!==run||run.status!=='running')return;
   while(inflight<2&&cursor<run.items.length){
    const index=cursor++,item=run.items[index];item.status='running';inflight++;nodeSpeedResults[nodeSpeedKey(pid,item.nodeId)]={status:'running',fingerprint:item.fingerprint};
    setTimeout(()=>{
     if(nodeSpeedRuns[pid]!==run||run.status!=='running')return;
     const current=draft.providers.find(p=>p.id===pid),n=current&&serviceNodes(current).find(n=>n.id===item.nodeId);inflight--;
     if(!n||item.fingerprint!==nodeSpeedFingerprint(current,n)){item.status='stale';}
     else{
      let status='success';try{nodeURL(n.url);}catch{status='invalid';}
      if(status==='success'&&current.connection?.auth==='apikey'&&!current.connection.keyConfigured)status='auth';
      if(status==='success'&&run.scene==='timeout')status='timeout';else if(status==='success'&&run.scene==='mixed'&&(index===1||run.items.length===1))status='failed';
      const seed=[...n.url].reduce((total,c)=>total+c.charCodeAt(0),0),ms=58+seed%230;
      item.status=status;nodeSpeedResults[nodeSpeedKey(pid,n.id)]={status,fingerprint:item.fingerprint,...(status==='success'?{ms}:{}),finishedAt:new Date().toLocaleString('zh-CN',{timeZone:'Asia/Shanghai',hour12:false})};
     }
     if(cursor===run.items.length&&inflight===0)run.status='complete';else pump();nodeSpeedUI();
    },360+index*50);
   }
   nodeSpeedUI();
  };pump();
 }
 function stopNodeSpeed(pid){
  const run=nodeSpeedRuns[pid];if(run?.status!=='running')return;run.status='cancelled';
  run.items.filter(item=>['waiting','running'].includes(item.status)).forEach(item=>{item.status='cancelled';nodeSpeedResults[nodeSpeedKey(pid,item.nodeId)]={status:'cancelled',fingerprint:item.fingerprint};});nodeSpeedUI();
 }
 function setNodeSpeedScene(value){
  if(!['normal','mixed','timeout'].includes(value))return;
  nodeSpeedSerial++;nodeSpeedResults={};nodeSpeedRuns={};nodeSpeedScene=value;nodeSpeedUI();notice('节点测速演示：'+({normal:'正常',mixed:'部分失败',timeout:'超时'})[value]);
 }
 function channelDefaults(p){return '<div class="mp-toolbar"><h3>模型默认</h3>'+button('编辑默认配置','provider-config',p.id==='native'||busy()?'disabled':'')+'</div><p class="mp-muted">作用于本渠道全部模型。未覆盖字段继续使用模版或模型声明。</p><div class="mp-rule-summary">'+[['protocol','协议'],['context','上下文窗口'],['efforts','推理等级'],['default','默认档位'],['fast','快速模式']].map(([key,name])=>'<div><span>'+name+'</span><strong>'+esc(Object.hasOwn(p.defaults,key)?ruleText(key,p.defaults[key]):'继承')+'</strong></div>').join('')+'</div>';}
 function channelNodes(){return channelSections.map(([id])=>$('#mpChannelSection-'+id,host)).filter(Boolean);}
 function channelOffset(scroller=host.closest('.main')){
  const {inset,height:tabsHeight}=syncTabBar(scroller),height=$('#mpChannelHeader',host)?.getBoundingClientRect().height||0,offset=tabsHeight+height+(scroller?.clientTop||0)+12;
  host.style?.setProperty?.('--mp-editor-top',tabsHeight-inset+'px');host.style?.setProperty?.('--mp-editor-offset',offset+'px');host.style?.setProperty?.('--mp-editor-nav-top',tabsHeight+height+12-inset+'px');
  if(scroller)host.style?.setProperty?.('--mp-editor-nav-height',Math.max(0,scroller.clientHeight-tabsHeight-height-24)+'px');return offset;
 }
 function channelScrollPosition(scroller){if(!scroller)return null;const top=scroller.getBoundingClientRect().top,offset=channelOffset(scroller),sections=channelNodes(),anchor=sections.find(node=>node.getBoundingClientRect().bottom>top+offset)||sections.at(-1);return {top:scroller.scrollTop,id:anchor?.id,offset:anchor?anchor.getBoundingClientRect().top-top:0};}
 function setChannelSection(id){if(!channelSections.some(([key])=>key===id))return;channelSection=id;$$('[data-mp="channel-section"]',host).forEach(node=>{if(node.classList.contains('mp-nav-item'))node.setAttribute('aria-current',node.dataset.id===id?'location':'false');});}
 function syncChannelSection(){const route=document.documentElement?.dataset.route;if(tab!=='channels'||!channelDetail||route&&route!=='models')return;const scroller=host.closest('.main');if(!scroller)return;const sections=channelNodes();if(!sections.length)return;const top=scroller.getBoundingClientRect().top+channelOffset(scroller)+12;let active=sections[0];for(const section of sections)if(section.getBoundingClientRect().top<=top)active=section;if(scroller.scrollHeight>scroller.clientHeight&&scroller.scrollTop+scroller.clientHeight>=scroller.scrollHeight-2)active=sections.at(-1);setChannelSection(active.dataset.channelSection);}
 function scrollChannelSection(id){if(tab!=='channels'||!channelDetail||!channelSections.some(([key])=>key===id))return;const section=$('#mpChannelSection-'+id,host),scroller=host.closest('.main');if(!section)return;setChannelSection(id);if(scroller)scroller.scrollTo({top:scroller.scrollTop+section.getBoundingClientRect().top-scroller.getBoundingClientRect().top-channelOffset(scroller),behavior:window.matchMedia?.('(prefers-reduced-motion: reduce)')?.matches?'auto':'smooth'});}
 function toggleChannelNav(){const workspace=$('#mpChannelWorkspace',host),toggle=$('#mpChannelNavToggle',host);if(!workspace||!toggle)return;const scroller=host.closest('.main'),position=channelScrollPosition(scroller);channelNavCollapsed=!channelNavCollapsed;workspace.classList.toggle('nav-collapsed',channelNavCollapsed);toggle.setAttribute('aria-expanded',String(!channelNavCollapsed));toggle.setAttribute('aria-label',(channelNavCollapsed?'展开':'收起')+'渠道导航');toggle.setAttribute('title',(channelNavCollapsed?'展开':'收起')+'渠道导航');restoreEditorScroll(scroller,position,channelOffset);syncChannelSection();}
 function syncDetailSection(){const route=document.documentElement?.dataset.route;if(route&&route!=='models')return;syncTabBar();syncEditorSection();syncChannelSection();}

 function channelPage(){
  if(!channelDetail)return '<div class="mp-list-toolbar"><input class="input" data-channel-search aria-label="搜索渠道" placeholder="搜索渠道" value="'+esc(channelSearch)+'"><div class="mp-actions">'+button('新增渠道','new-channel',busy()?'disabled':'',true)+(scope.size?button('清除选择','clear-channels',busy()?'disabled':''):'')+'</div></div>'+batchToolbar()+channels()+'<p class="mp-footer-note">启用开关修改草稿，保存后提示重启；停用保留已选模型与配置。拖动渠道调整整组顺序。</p>';
  const p=provider(),rows=visibleModels(),filtered=search||family||onlySelected,locked=busy();
  const models='<div class="mp-section-heading"><h3>模型列表 <span class="mp-badge">'+rows.length+'</span></h3><div class="mp-actions">'+(p.id==='native'?'':button(p.refresh==='loading'?'取消等待':'获取模型列表',p.refresh==='loading'?'cancel-refresh':'refresh',locked?'disabled':'')+button('手动添加','manual-model',locked?'disabled':''))+'</div></div>'+(['loading','failed','empty'].includes(p.refresh)?'<div class="mp-inline-notice">'+(p.refresh==='loading'?'正在获取清单…':p.refresh==='failed'?'获取失败，旧清单与选择保留':'上游返回空清单，原选择保留')+'</div>':'')+'<div class="mp-model-filter"><label class="mp-model-scope"><span class="sr-only">模型公开范围</span><select class="mp-select" data-mode aria-label="模型公开范围">'+opts([['whitelist','仅所选模型'],['all','全部公开'],['none','全部隐藏']],p.mode)+'</select></label><input class="input" data-model-search aria-label="搜索模型 ID 或别名" placeholder="搜索型号或名称" value="'+esc(search)+'"><label class="mp-model-family"><select class="mp-select" data-family aria-label="筛选模型族">'+opts([['','全部族'],...families()],family)+'</select></label><label class="mp-label"><input type="checkbox" data-only-selected '+(onlySelected?'checked':'')+'>仅启用</label><span class="mp-model-count">'+rows.length+' / '+p.models.length+' 项</span></div>'+modelBatchToolbar()+'<div class="mp-table" aria-label="模型管理列表"><div class="mp-table-head"><input type="checkbox" data-model-pick-all aria-label="批量选择当前筛选结果" aria-checked="'+(modelSelectionState().mixed?'mixed':modelSelectionState().checked)+'" '+(modelSelectionState().checked?'checked ':'')+(p.id==='native'||locked||!rows.length?'disabled':'')+'><span>模型标识 / 别名与配置</span><span>启停</span><span>编辑与排序</span></div>'+rows.map(m=>modelRow(p,m,filtered)).join('')+(rows.length?'':'<div class="mp-empty">没有匹配项，启停与批量选择保留。</div>')+'</div><p class="mp-footer-note">'+(!p.enabled?'渠道已停用，模型配置仍可编辑；启用渠道后恢复计划选择器。':'复选框仅用于批量选择；启停独立控制。别名不改变上游 ID，行编辑加入草稿后统一保存。')+'</p>';
  const header='<header id="mpChannelHeader" class="mp-editor-head">'+iconButton('返回渠道列表','back-channels','back')+'<div class="mp-editor-heading"><h2>'+esc(p.name)+'</h2><div class="mp-template-heading-meta"><div id="mpChannelHeadingStatus" class="mp-template-heading-status" role="status">'+channelHeadingStatusHTML(p)+'</div>'+channelStatusAnnotation()+'</div></div><select class="mp-select" data-current aria-label="当前渠道">'+opts(draft.providers.map(p=>[p.id,p.name]),current)+'</select></header>';
  const nav=channelSections.map(([id,name],i)=>'<button class="mp-nav-item" data-mp="channel-section" data-id="'+id+'" title="'+name+'" aria-controls="mpChannelSection-'+id+'" aria-current="'+(channelSection===id?'location':'false')+'"><span>'+String(i+1).padStart(2,'0')+'</span><span class="mp-nav-label">'+name+'</span></button>').join('');
  const content={models,connection:inlineConnection(p),nodes:serviceNodeList(p),defaults:channelDefaults(p)};
  return header+'<div id="mpChannelWorkspace" class="mp-channel-workspace '+(channelNavCollapsed?'nav-collapsed':'')+'"><nav class="mp-template-nav mp-channel-nav" aria-label="渠道管理分区">'+button(icon('menu'),'toggle-channel-nav','id="mpChannelNavToggle" title="'+(channelNavCollapsed?'展开':'收起')+'渠道导航" aria-label="'+(channelNavCollapsed?'展开':'收起')+'渠道导航" aria-expanded="'+!channelNavCollapsed+'"')+nav+status()+'</nav><div class="mp-channel-content">'+channelSections.map(([id,name])=>'<section id="mpChannelSection-'+id+'" class="mp-template-section" data-channel-section="'+id+'" aria-label="'+name+'">'+content[id]+'</section>').join('')+'</div></div>';
 }


 function listTemplateVersion(t){
  return (templateEditorChanged(t.id)?version(t.id,buffers[t.id]?.base):null)||version(t.id,templateListVersions[t.id])||version(t.id,t.main)||t.versions.at(-1)||null;
 }
 function savedTemplateContent(t,v){return {name:t.name,description:v.description||'',match:clone(v.match),rules:clone(v.rules),...(v.variants?.length?{variants:clone(v.variants)}:{})};}
 function templateListVersionCell(t,v){
  const editing=templateEditorChanged(t.id);
  return '<div class="mp-template-version-cell"><select id="mpTemplateListVersion-'+esc(t.id)+'" class="mp-select" data-template-row-version="'+esc(t.id)+'" aria-label="'+esc(t.name)+' 选择操作版本" title="选择版本不更改默认版本或已采用配置" '+(editing?'disabled':'')+'>'+compactVersionOptions(t,v?.n)+'</select>'+(editing?'<small class="mp-muted">有未保存编辑</small>':'')+'</div>';
 }
 function selectedListTemplate(id){const t=templates.find(t=>t.id===id),v=t&&listTemplateVersion(t);return t&&v?{t,v}:null;}
 function copyListTemplate(id){
  const target=selectedListTemplate(id);if(!target)return;
  const {t,v}=target;newTemplate(true,savedTemplateContent(t,v),{templateId:id,revisionId:v.revisionId,includesUnsaved:false});
 }
 function templateUsageHTML(id,n){
  const t=templates.find(t=>t.id===id),v=version(id,n);if(!t||!v)return;
  const ref=id+':'+n;
  const rows=policy=>{
   const result=[];if(canonicalKeys(policy.globalRefs)[id]===ref)result.push({name:'全局族默认',type:'全局引用',scope:'供没有更具体引用的渠道继承'});
   for(const p of policy.providers.filter(p=>p.id!=='native')){
    if(canonicalKeys(p.refs)[id]===ref)result.push({name:p.name,type:'渠道族引用',scope:p.id});
    for(const m of p.models.filter(m=>m.templateRef===ref))result.push({name:p.name+' / '+m.id,type:'单模型精确引用',scope:m.key});
   }
   return result;
  };
  const renderRows=(title,items)=>'<section class="mp-template-usage-section"><h3>'+title+'</h3>'+(items.length?'<div class="mp-template-usage-list">'+items.map(row=>'<div><strong>'+esc(row.name)+'</strong><span>'+esc(row.type)+'</span><small>'+esc(row.scope)+'</small></div>').join('')+'</div>':'<p class="mp-muted">此版本没有配置引用。</p>')+'</section>';
  const before=rows(saved),after=rows(draft);
  const affected=policy=>policy.providers.filter(p=>p.id!=='native').flatMap(p=>p.models.filter(m=>templateResolution(p,m,policy).ref===ref&&!templateResolution(p,m,policy).error).map(m=>p.name+' / '+m.id+(p.enabled&&selected(p).some(x=>x.key===m.key)?'':' · 未发布')));
  const used=affected(saved);
  return renderRows('已保存配置',before)+(JSON.stringify(before)!==JSON.stringify(after)?renderRows('当前草稿（尚未保存）',after):'')+(used.length?'<details class="mp-details"><summary>此版本解析到的 '+used.length+' 个模型</summary>'+diffHTML(used)+'</details>':'')+'<p class="mp-muted">引用按精确版本列出；匹配命中不等于已经采用，停用或未发布的配置引用也保留。</p>';
 }
 function requestListVersionDelete(id,n,returnTo=false){
  const t=templates.find(t=>t.id===id),v=version(id,n);if(!t||!v||templateEditorChanged(id)||versionProtected(id,n)){notice('版本有默认、引用、恢复或编辑依赖，请保留');return;}
  const snapshot=JSON.stringify(t);
  modal('删除所选版本','<p>'+esc(t.name)+' · '+esc(v.versionLabel)+'</p><p>只删除这一个未引用版本。'+(t.versions.length===1?'这是最后一个版本，删除后模版也将从列表移除。':'其他版本保留。')+'</p><div id="mpListVersionDeleteError" class="mp-alert" role="alert"></div>','删除',()=>{
   const error=$('#mpListVersionDeleteError');if(!error)return;
   if(snapshot!==JSON.stringify(templates.find(t=>t.id===id))||templateEditorChanged(id)||versionProtected(id,n)){error.textContent='版本或依赖已变化，请重新核对。';return;}
   const view=returnTo&&templateHistoryView?.id===id?clone(templateHistoryView):null;deleteTemplateVersion(id+':'+n);if(view&&templates.some(t=>t.id===id))openTemplateHistory(id,view.context,view.query);
  },{confirmKeepsOpen:true});
 }
 function removeTemplateVersions(id,ns){
  const t=templates.find(t=>t.id===id);if(!t)return;const removed=new Set(ns.map(Number));
  t.versions=t.versions.filter(v=>!removed.has(v.n));if(removed.has(buffers[id]?.base))delete buffers[id];
  if(!t.versions.length){templates.splice(templates.indexOf(t),1);templateSelection.delete(id);delete templateListVersions[id];delete buffers[id];delete assistantInputs[id];conversation=conversation.filter(m=>m.templateId!==id);if(templateId===id){templateId=templates[0]?.id||'gpt';versionN=template()?.main||1;templateDetail=false;assistantOpen=false;}}
  else{const fallback=t.main||t.versions.at(-1).n;if(removed.has(templateListVersions[id]))templateListVersions[id]=fallback;if(templateId===id&&removed.has(versionN))versionN=fallback;}
 }
 function templateRetentionFingerprint(id){return JSON.stringify({template:templates.find(t=>t.id===id),buffers,saved,draft,records,confirmedConfigurations,incoming,updateRun,externalTask,externalTasks:[...externalTasks.values()],generating,generationOwner});}
 function templateRetentionAnnotation(){return mpAnn('09','查看、默认与删除的规则','查看／编辑只打开这个版本，不改默认或当前渠道引用。设为默认决定以后采用时默认选择的版本，已有采用仍引用精确版本。<br>系统初始 / 当前内置基线、默认版本、已保存或草稿引用、恢复记录和正在编辑 / 待审任务依赖由系统自动保护，不能删除。删除单版或勾选批量都先显示名单，确认前重新核对整个计划；任何目标变化都不执行旧确认。<br>不提供归档、手动保留或独立清理；不会按数量自动删版本。下拉显示最近5个及当前 / 默认，其余从版本管理查看。');}
 function openTemplateHistory(id,context='list',query=''){
  const t=templates.find(t=>t.id===id);if(!t)return;
  const old=templateHistoryView,blocked=templateEditorChanged(id),term=query.toLowerCase();
  const picked=(old?.id===id&&old.context===context?old.picked||[]:[]).filter(n=>version(id,n)&&!versionProtected(id,n));
  templateHistoryView={id,context,query,picked};
  const rows=t.versions.slice().reverse().filter(v=>(v.versionLabel+' '+storedVersionSummaryFull(v)+' '+v.origin).toLowerCase().includes(term));
  const selectable=blocked?[]:rows.filter(v=>!versionProtected(id,v.n)),all=selectable.length>0&&selectable.every(v=>picked.includes(v.n));
  const controls=v=>{const reasons=versionProtectionReasons(id,v.n),attrs='data-id="'+esc(id)+'" data-n="'+v.n+'" ',def=t.main===v.n;
   return button('查看／编辑','history-select-version','id="mpVersionOpen-'+esc(id)+'-'+v.n+'" '+attrs+'title="打开此版本，修改后保存为新版本" '+(blocked?'disabled':''))+
    button(def?'默认版本':'设为默认','history-set-main','id="mpVersionDefault-'+esc(id)+'-'+v.n+'" '+attrs+'title="以后采用时默认选这一版，已有引用不变" '+(blocked||def?'disabled':''))+
    button('删除','history-delete-version','id="mpVersionDelete-'+esc(id)+'-'+v.n+'" '+attrs+(blocked||reasons.length?'disabled title="'+esc(reasons.join('、')||'先处理编辑')+'"':''),false,true);
  };
  const content='<div class="mp-version-history-toolbar"><input id="mpVersionHistorySearch" class="input" data-history-search aria-label="搜索版本" placeholder="搜索时间、标识或说明" value="'+esc(query)+'"><label class="mp-label"><input id="mpVersionDeleteAll" type="checkbox" data-version-pick-all '+(all?'checked ':'')+(!selectable.length?'disabled':'')+'>全选可删除版本</label>'+button('删除所选','history-delete-selected','id="mpDeleteVersions" data-id="'+esc(id)+'" '+(blocked||!picked.length?'disabled':'') ,false,true)+(picked.length?'<span class="mp-muted">已选 '+picked.length+' 项</span>':'')+'</div><div class="mp-version-history-list" aria-label="全部保存版本">'+rows.map(v=>{const reasons=versionProtectionReasons(id,v.n);return '<section class="mp-version-history-row"><input type="checkbox" data-version-pick="'+v.n+'" aria-label="勾选删除 '+esc(v.versionLabel)+'" '+(picked.includes(v.n)?'checked ':'')+(blocked||reasons.length?'disabled title="'+esc(reasons.join('、')||'先处理编辑')+'"':'')+'><div class="mp-version-history-info"><strong>'+esc(versionOptionLabel(t,v))+'</strong><small>'+esc(v.origin||'人工')+' · '+esc(storedVersionSummaryFull(v))+'</small>'+(reasons.length?'<span class="mp-version-protection" title="'+esc(reasons.join('、'))+'">受保护：'+esc(reasons[0])+'</span>':'')+'</div><div class="mp-actions">'+controls(v)+'</div></section>';}).join('')+(rows.length?'':'<p class="mp-muted">没有匹配的保存版本。</p>')+'</div>'+templateRetentionAnnotation();
  modal('版本管理 · '+t.name,content,'关闭',null,{hideConfirm:true,cancelLabel:'关闭'});$('#modal').classList.add('modal-wide');const allInput=$('#mpVersionDeleteAll');if(allInput)allInput.indeterminate=selectable.some(v=>picked.includes(v.n))&&!all;window.prototypeSelect?.enhance($('#modalBody'));
 }
 function pickVersions(el){
  const view=templateHistoryView;if(!view||templateEditorChanged(view.id)||el.disabled)return;
  const t=templates.find(t=>t.id===view.id);if(!t)return;let picked=new Set(view.picked||[]);
  const targets=el.matches('[data-version-pick-all]')?t.versions.filter(v=>(v.versionLabel+' '+storedVersionSummaryFull(v)+' '+v.origin).toLowerCase().includes(view.query.toLowerCase())&&!versionProtected(t.id,v.n)).map(v=>v.n):[Number(el.dataset.versionPick)].filter(n=>version(t.id,n)&&!versionProtected(t.id,n));
  for(const n of targets)el.checked?picked.add(n):picked.delete(n);view.picked=[...picked];openTemplateHistory(view.id,view.context,view.query);
 }
 function chooseHistoryVersion(id,n){
  const view=templateHistoryView,v=version(id,n);if(!view||view.id!==id||!v||templateEditorChanged(id))return;
  closeModal();openTemplateEditor(id,n);
 }
 function deletePickedVersions(id){
  const t=templates.find(t=>t.id===id),view=templateHistoryView;if(!t||view?.id!==id||templateEditorChanged(id))return;
  const ns=[...new Set(view.picked||[])];if(!ns.length||ns.some(n=>versionProtected(id,n)))return;
  const snapshot=templateRetentionFingerprint(id),back=clone(view);
  modal('删除所选版本','<p>将删除 '+ns.length+' 个版本。取消不会删除。</p>'+diffHTML(ns.map(n=>versionText(id,n)))+(ns.length===t.versions.length?'<p>删除最后一批版本后，此自建模版将从列表移除。</p>':'')+'<div id="mpVersionDeleteError" class="mp-alert" role="alert"></div>','删除',()=>{
   if(snapshot!==templateRetentionFingerprint(id)||templateEditorChanged(id)||ns.some(n=>versionProtected(id,n))){$('#mpVersionDeleteError').textContent='版本或依赖已变化，请重新核对。';return;}
   removeTemplateVersions(id,ns);closeModal();render();if(templates.some(t=>t.id===id)){templateHistoryView={...back,picked:[]};openTemplateHistory(id,back.context,back.query);}
  },{confirmKeepsOpen:true});
 }
 function templateInspection(id){
  const target=selectedListTemplate(id);if(!target)return;const {t,v}=target;
  modal('匹配与使用 · '+t.name,'<p class="mp-muted">'+esc(v.versionLabel)+'</p><h3>实际使用</h3>'+templateUsageHTML(id,v.n)+'<section class="mp-template-match-section"><h3>规则匹配</h3>'+matchPreviewHTML(t,savedTemplateContent(t,v))+'</section>'+mpAnn('11','实际引用与规则匹配的区别','同一所选保存版本同时查看已保存 / 草稿的精确引用和规则命中。命中只是可匹配范围，不自动采用；默认指针不等于当前引用，停用 / 未发布的引用也保留。查看不修改模版或渠道策略。'),'关闭',null,{hideConfirm:true,cancelLabel:'关闭'});
  $('#modal').classList.add('modal-wide');
 }
 function templatePage(){
  if(!templateDetail)return (templateScratch?'<div class="mp-inline-notice"><span>有未保存的新模版</span>'+button('继续编辑','new-template')+'</div>':'')+
   '<div class="mp-list-toolbar"><input class="input" data-template-search aria-label="搜索模版" placeholder="搜索模版或协议" value="'+esc(templateSearch)+'"><div class="mp-actions">'+button('新建模版','new-template','',true)+'<button id="mpUpdateAll" class="btn" data-mp="update-all" title="检查全部模版，查看差异后保存新版本" '+(updateRun?.status==='running'||generating?'disabled':'')+'>检查全部更新</button>'+button('外部任务','external-agent')+'</div></div>'+
   (templateSelection.size?'<div class="mp-update-bulk" role="group" aria-label="所选模版操作"><span>已选 '+templateSelection.size+' 个模版</span>'+button('检查所选更新','update-selected')+button('清除勾选','clear-template-picks')+'</div>':'')+
   '<div class="mp-template-table"><div class="mp-template-heading"><span>模版 / 自定义模型族</span><span>版本</span><span>操作</span></div>'+templates.filter(t=>(t.name+' '+t.versions.map(v=>v.rules.protocol||'').join(' ')).toLowerCase().includes(templateSearch.trim().toLowerCase())).map(t=>{
    const v=listTemplateVersion(t);return '<div class="mp-template-row"><div class="mp-row"><input type="checkbox" data-template-select="'+esc(t.id)+'" aria-label="选择 '+esc(t.name)+'" '+(templateSelection.has(t.id)?'checked':'')+'><div class="mp-template-name"><strong>'+esc(t.name)+templateOriginHTML(t)+'</strong><small title="'+esc(storedVersionSummaryFull(v))+'">'+esc(storedVersionSummary(v))+'</small>'+'</div></div>'+templateListVersionCell(t,v)+'<div class="mp-template-list-actions">'+button('编辑','open-template-editor','data-id="'+esc(t.id)+'"')+button('复制','copy-list-template','data-id="'+esc(t.id)+'" '+(templateScratch?'disabled':'') )+button('版本管理','template-version-history','data-id="'+esc(t.id)+'"')+button('匹配与使用','template-inspect','data-id="'+esc(t.id)+'"')+'</div></div>';
   }).join('')+'</div><p class="mp-footer-note">版本按北京时间精确到秒，并附唯一短标识；保存不覆盖旧版。下拉显示最近5个版本及当前/默认，其余在版本管理中选择。</p>'+mpAnn('07','模版列表版本与动作范围','列表只保留一个版本列，时间与短标识唯一识别版本；默认版本在下拉选项中标记。选版本不打开详情，不改默认、采用或保存内容；编辑 / 复制 / 匹配与使用作用于所选版本；版本管理只提供查看／编辑、设为默认和单个 / 勾选删除，列表不再提供更多。<br>复制以所选已保存版本新建，不携带原历史或采用关系；有未保存编辑时先继续处理，再切换版本。默认设置单独确认，已采用配置保留；内置基线、默认与引用依赖自动保护，删除检查默认、采用、草稿、恢复记录和编辑依赖；不按数量自动删除，勾选删除统一确认。全部规则见版本管理中的原型标签。')+versionSummaryAnnotation();
  const t=template(),v=version(t.id,versionN),b=buffer(),changed=templateEditorChanged();
  const title=templateOriginHTML(t)+'<input id="mpTemplateName" data-template-meta="name" class="mp-editor-name" aria-label="模版名称" title="直接编辑模版名称" value="'+esc(b.name)+'" placeholder="自由命名模型族">';
  const header='<header id="mpTemplateHeader" class="mp-editor-head">'+iconButton('返回模版列表','back-templates','back')+'<div class="mp-editor-heading" title="'+esc(t.id)+' · 模版即模型族"><h2>'+title+'</h2><div class="mp-template-heading-meta"><div id="mpTemplateHeadingStatus" class="mp-template-heading-status" role="status" aria-live="polite">'+templateHeadingStatusHTML()+'</div>'+templateStatusAnnotation()+'</div></div><div class="mp-actions mp-editor-versions">'+(t.versions.length?'<select id="mpTemplateVersion" class="mp-select" data-version aria-label="模版版本" '+(changed?'disabled title="先保存修改或返回放弃编辑，再切换版本"':'')+'>'+compactVersionOptions(t,b.base)+'</select>'+button('版本与历史','version-menu','aria-haspopup="menu" aria-expanded="false"'):'')+'</div></header>';
  return header+templateForm(b)+(proposal?.id===t.id?'<div class="mp-inline-notice"><span>生成结果待比较</span>'+button('查看差异与纠偏','conversation')+'</div>':'')+(v?'<div class="mp-editor-footer">'+button('采用此版本','adopt-version','id="mpTemplateAdopt" '+(changed?'disabled title="先保存模版修改，再采用已保存版本"':''),true)+'</div>':'');
 }

 function render(){
  closeNodePicker();window.prototypeActionMenu?.close();
  if(tab==='templates'&&templateDetail)ensureTemplateBuffer();
  const scroller=host.closest('.main'),channelEditing=tab==='channels'&&channelDetail,sameChannel=channelEditing&&channelViewId===current,channelPosition=sameChannel?channelScrollPosition(scroller):null,editing=tab==='templates'&&templateDetail&&!!buffer(),sameEditor=editing&&editorViewId===templateId,position=sameEditor?editorScrollPosition(scroller):null;
  if(editing&&!sameEditor)editorSection='basic';if(channelEditing&&!sameChannel)channelSection=channelSections[0][0];
  for(const id of Object.keys(channelConnections))if(!draft.providers.some(p=>p.id===id))delete channelConnections[id];
  const validKeys=new Set(draft.providers.flatMap(p=>p.models.map(m=>m.key)));modelSelection=new Set([...modelSelection].filter(k=>validKeys.has(k)));modelExpanded=new Set([...modelExpanded].filter(k=>validKeys.has(k)));for(const key of Object.keys(modelEdits))if(!validKeys.has(key))delete modelEdits[key];
  if(!provider()){current=draft.providers[0]?.id;channelDetail=false;}scope=new Set([...scope].filter(id=>draft.providers.some(p=>p.id===id&&id!=='native')));const active=document.activeElement,focus=active?.matches('[data-model-search]')?'[data-model-search]':active?.matches('[data-channel-search]')?'[data-channel-search]':active?.matches('[data-template-search]')?'[data-template-search]':active?.matches('[data-model-alias]')?'[data-model-alias="'+active.dataset.modelAlias+'"]':null,caret=active?.selectionStart,fieldId=active?.id&&host.querySelector('#'+active.id)?active.id:null;host.innerHTML=toolbar()+(tab==='channels'?channelPage():templatePage());
  $$('[data-model-quick] input,[data-model-quick] select,[data-model-quick] button',host).forEach(el=>el.disabled=busy());
  const selection=scopeState();$$('[data-scope-all]').forEach(el=>el.indeterminate=selection.mixed);$$('[data-model-pick-all]',host).forEach(el=>el.indeterminate=modelSelectionState().mixed);
  if(tab==='templates'&&templateDetail&&buffer())bindFields($('[data-field-editor]',host),buffer().rules,null,(rules,valid)=>{buffer().rules=rules;buffer().valid=valid;editorDigest++;updateTemplateSaveStatus();});
  if(focus||fieldId){const el=$(focus||'#'+fieldId);if(el){el.focus({preventScroll:true});if(caret!=null&&el.setSelectionRange)el.setSelectionRange(caret,active.selectionEnd??caret);}}else if(active?.matches('[data-scope-all]'))$('[data-scope-all]',host)?.focus();else if(active?.matches('[data-scope]'))$$('[data-scope]',host).find(el=>el.dataset.scope===active.dataset.scope)?.focus();if(active?.matches('[data-model-pick-all]'))$('[data-model-pick-all]',host)?.focus({preventScroll:true});else if(active?.matches('[data-model-pick]'))$$('[data-model-pick]',host).find(el=>el.dataset.modelPick===active.dataset.modelPick)?.focus({preventScroll:true});renderSettings();renderAssistant();
  syncTabBar(scroller);channelViewId=tab==='channels'&&channelDetail?current:null;if(channelViewId&&scroller){channelOffset(scroller);if(sameChannel)restoreEditorScroll(scroller,channelPosition,channelOffset);else scroller.scrollTop=0;channelOffset(scroller);syncChannelSection();}
  editorViewId=editing?templateId:null;if(tab==='templates'&&templateDetail&&scroller)editorOffset(scroller);if(editing&&scroller){if(sameEditor)restoreEditorScroll(scroller,position);else scroller.scrollTop=0;syncEditorSection();}syncSaveNotification();
 }
 function agentProvider(){return saved.providers.find(p=>p.id===agent.providerId&&p.id!=='native');}
 function agentModel(){return agentProvider()?.models.find(m=>m.id===agent.modelId);}
 function agentBlocked(p){return !p?.enabled||['no-credit','auth-failed','connection-failed','unavailable'].includes(p.availability?.kind);}
/** 工具板演示：切换“尚未配置 / 已配置模型”的内存投影，不写真实配置。 */
 function applyAgentDemo(mode){
  if(mode==='configured'){
   const p=saved.providers.find(p=>p.id!=='native'&&p.enabled&&p.models.some(m=>m.selected!==false));
   const m=p?.models.find(m=>m.selected!==false);
   agent=p&&m?{providerId:p.id,modelId:m.id,effort:'inherit',fast:false}:{providerId:'',modelId:'',effort:'inherit',fast:false};
  } else {
   agent={providerId:'',modelId:'',effort:'inherit',fast:false};
  }
  renderSettings();
  window.toast?.('原型模拟：Agent 执行模型已切换为'+(mode==='configured'?'已配置模型':'尚未配置')+'；不写真实配置。');
 }
 function agentInfo(selection=agent){
  const p=saved.providers.find(p=>p.id===selection.providerId&&p.id!=='native'),m=p?.models.find(m=>m.id===selection.modelId);
  let reason=!selection.providerId?'尚未选择执行模型':!p?'执行渠道已删除或不存在':!p.enabled?'执行渠道已停用':agentBlocked(p)?availabilityText(p)||'执行渠道不可用':!m?'模型已从目录移除':'';
  const resolved=p&&m?resolve(p,m,saved):null,values=resolved?.values||{};
  if(!reason&&resolved?.route.error)reason=resolved.route.error;
  if(!reason&&Array.isArray(values.modalities)&&!values.modalities.includes('text'))reason='此模型未声明文本输入能力';
  if(!reason&&selection.effort&&selection.effort!=='inherit'&&!(values.efforts||[]).includes(selection.effort))reason='已选推理档位不再受支持';
  return {p,m,values,reason};
 }
 function agentReady(){return !agentInfo().reason;}
 function agentExecution(){const info=agentInfo();return {...clone(agent),effectiveEffort:agent.effort==='inherit'?info.values.default??null:agent.effort};}
 function agentFastSummary(selection,values){
  if(values.fast==='supported')return selection.fast?'快速已开启':'快速已关闭';
  if(values.fast==='unsupported')return selection.fast?'已选快速 · 当前模型不支持':'不支持快速模式';
  return selection.fast?'已选快速 · 能力待核对':'快速能力待核对';
 }
 function agentSummary(selection=agent){const info=agentInfo(selection);if(info.reason)return info.reason+' · 请配置执行模型';const actual=selection.effort==='inherit'?info.values.default:selection.effort;return info.p.name+' / '+info.m.id+' · '+(labels[actual]||'推理未知')+(selection.effort==='inherit'?'（模型默认）':'')+' · '+agentFastSummary(selection,info.values);}
 function agentChoices(){
  return saved.providers.flatMap(p=>p.models.map(m=>{
   const info=agentInfo({providerId:p.id,modelId:m.id,effort:'inherit',fast:false});
   return {ref:p.id==='native'?{kind:'native',nativeModelId:m.id}:{kind:'channel',channelId:p.id,modelId:m.id},channelName:p.name,name:m.naming?.displayName||m.id,alias:m.naming?.alias||'',context:info.values.context??null,effort:info.values.default??null,efforts:info.values.efforts||[],fastCapability:info.values.fast||'unknown',fastDefault:false,modalities:info.values.modalities||[],reason:p.id==='native'?'原生目录未绑定管理器执行连接':info.reason};
  }));
 }
 let agentSettingsDraft=null,agentSettingsReturn='',agentSettingsOrigin='models';
 function selectAgentModel(){
  if(!agentSettingsDraft)return;
  const next=clone(agentSettingsDraft),returnTo=agentSettingsReturn,route=agentSettingsOrigin;
  const selected=next.providerId&&next.modelId?[{kind:'channel',channelId:next.providerId,modelId:next.modelId,...(next.effort!=='inherit'?{reasoningEffort:next.effort}:{}),fast:next.fast}]:[];
  const picker=window.modelPickerPrototype;
  if(!picker?.choose){notice('公共模型选择器尚未加载，请刷新页面');return;}
  picker.choose({title:'选择模型模版 Agent 的执行模型',mode:'single',reasoning:true,fast:true,models:selected,getCatalog:agentChoices,
   onSelect:result=>{
    const ref=result.models[0];if(!ref||ref.kind!=='channel')return;
    const chosen={providerId:ref.channelId,modelId:ref.modelId,effort:ref.reasoningEffort||'inherit',fast:ref.fast===true},info=agentInfo(chosen);
    if(info.values.fast!=='supported')chosen.fast=false;
    if(document.documentElement.dataset.route!==route)return;agentSettings(returnTo,chosen);
   },
   onCancel:result=>{if(!result?.routeChanged&&document.documentElement.dataset.route===route)agentSettings(returnTo,next);}
  });
 }
 function agentSettings(returnTo='',selection=null){
  const next=clone(selection||agent),info=agentInfo(next);agentSettingsDraft=next;agentSettingsReturn=returnTo;agentSettingsOrigin=document.documentElement.dataset.route;
  modal('模型模版 Agent · 共用执行设置','<p>本机共用设置：自动填充、单个更新、所选更新和全部更新均使用这里的模型。</p><label class="mp-agent-choice-label">执行模型</label><button type="button" id="mpAgentModelChoice" class="btn mp-agent-choice" data-mp="agent-pick-model" aria-haspopup="dialog"><span><strong>'+esc(info.m?info.p.name+' / '+info.m.id:'选择执行模型')+'</strong><small>'+esc(info.m?agentSummary(next):'选择模型、推理档位与快速模式')+'</small></span>'+icon('collapse')+'</button>'+(info.reason&&next.providerId?'<p class="mp-alert">'+esc(info.reason)+'</p>':'')+'<div id="mpAgentSettingsError" class="mp-alert" role="alert"></div>','保存',()=>{
   const chosen=clone(agentSettingsDraft);
   const fresh=agentInfo(chosen);if(fresh.reason){$('#mpAgentSettingsError').textContent=fresh.reason+'，请重新选择';return;}
   if(chosen.fast&&fresh.values.fast!=='supported'){$('#mpAgentSettingsError').textContent='Fast 支持已变化，请重新选择';return;}
   agent=chosen;agentSettingsDraft=null;closeModal();render();if(returnTo==='generate')generateDialog();else if(returnTo==='conversation')conversationDialog();else if(returnTo==='update-all'||returnTo==='update-selected')startTemplateUpdate(returnTo.slice(7));notice('本机模型模版 Agent 共用设置已保存');
  },{confirmKeepsOpen:true,prototypeAnnotation:{title:'Agent 设置的保存与任务边界',body:'选择后先返回这里，保存才修改共用配置；取消保留原设置。运行中的任务继续使用启动时的设置。原型仅保存本页内存状态，刷新重置；实际执行与持久化尚未接入。'}});$('#modalConfirm').disabled=!!info.reason;
 }

 function renderSettings(){
  const node=$('#mpSettingsContent');if(!node)return;
  const notes='本机模型模版助手的自动填充、持续纠偏、单个 / 所选 / 全部更新共用执行设置，与 Codex 的命名、搜索、子 Agent 配置独立。型号、推理和快速模式统一在公共选择器内设置，确认返回缓冲后再保存；修改不影响已启动任务。原型只保存内存状态，真实执行 / 持久化 / 同步过滤待实现。正式执行偏好拟放 manager-state/preferences.json 的 modelTemplateAgent 分区，本机引用不进入默认同步或普通导出；模版规则、渠道策略及本机绑定分别属于 model-family-templates.json、model-policy.json、model-policy-bindings.json。不再提供高级模型配置及单独的全局 Fast 总开关；模型规则中的 Fast 支持能力保留。已有全局策略与独立 Fast 目录兼容字段先保留，真实客户端是否需要兼容项待核实；选择器的 Fast 偏好不修改这些运行字段，也不要求额外发布同型号条目。';
  const info=agentInfo(),ready=!info.reason;
  const fact=(label,value)=>'<div class="mp-agent-fact"><dt>'+esc(label)+'</dt><dd>'+esc(value)+'</dd></div>';
  const facts=ready?[
   fact('执行渠道',info.p.name),
   fact('型号',info.m.id),
   fact('推理档位',labels[info.values.default]||'未知'),
   fact('快速模式',info.values.fast==='supported'?'可用':'不可用')
  ].join(''):'';
  const body=ready
   ? '<div class="mp-agent-configured"><dl class="mp-agent-facts">'+facts+'</dl>'
     +'<div class="mp-agent-usage"><h4>适用范围</h4><ul>'
     +'<li>模版自动填充、单个更新、所选更新与批量检查共用此设置。</li>'
     +'<li>不改变 Codex 的命名、搜索与子 Agent 配置。</li>'
     +'<li>修改只影响之后启动的任务；已启动任务继续使用启动时的设置。</li>'
     +'</ul></div>'
     +'<div class="mp-agent-note"><strong>与运行配置的边界</strong><p>保存的是本机执行偏好，写入 manager-state/preferences.json 的 modelTemplateAgent 分区；不进入默认同步或普通导出。</p></div></div>'
   : '<p class="mp-agent-empty">尚未选择执行模型。自动填充与模版检查需要先配置一个可用的执行模型。</p>';
  node.innerHTML='<h2>Agent 配置</h2>'+mpAnn('02','Agent 配置与实现边界',notes)
   +'<div class="mp-settings-card mp-agent-card"><div class="mp-settings-title"><div><h3>模型模版助手 · 本机共用</h3><p id="mpAgentSummary" class="mp-muted" role="status">'+esc(agentSummary())+'</p></div><button class="btn'+(ready?'':' primary')+'" data-mp="agent-settings">'+(ready?'更换执行模型':'Agent 配置')+'</button></div>'
   +body+'</div>';
 }
 function diff(a,b){const changes=[];if(a.fastMode!==b.fastMode)changes.push('全局 Fast：'+a.fastMode+' → '+b.fastMode);if(a.fastRows!==b.fastRows)changes.push('独立 Fast 项：'+a.fastRows+' → '+b.fastRows);if(JSON.stringify(a.globalRefs)!==JSON.stringify(b.globalRefs))changes.push('全局族来源发生变更');if(a.providers.map(p=>p.id).join()!==b.providers.map(p=>p.id).join())changes.push('渠道顺序发生变更');a.providers.filter(p=>!b.providers.some(x=>x.id===p.id)).forEach(p=>changes.push(p.name+' 移除'));b.providers.forEach(p=>{const old=a.providers.find(x=>x.id===p.id);if(!old){changes.push(p.name+' 新增');return;}if(old.enabled!==p.enabled)changes.push(p.name+'：'+(old.enabled?'启用':'停用')+' → '+(p.enabled?'启用':'停用')+'（保留白名单、顺序和设置）');const before=clone(old),after=clone(p);delete before.enabled;delete after.enabled;if(JSON.stringify(before)!==JSON.stringify(after)){const add=selected(p).filter(m=>!selected(old).some(x=>x.key===m.key)).length,remove=selected(old).filter(m=>!selected(p).some(x=>x.key===m.key)).length;changes.push(p.name+'：白名单 +'+add+' / −'+remove+'；模式 '+old.mode+' → '+p.mode+'；字段 / 顺序将逐项核对');}});return changes;}
 const diffHTML=items=>'<div class="mp-diff">'+(items.length?items:['没有策略差异']).map(x=>'<div class="mp-diff-row">'+esc(x)+'</div>').join('')+'</div>';
 function policyChanges(a,b){
  const groups=[],same=(x,y)=>JSON.stringify(x)===JSON.stringify(y),list=xs=>xs.map((x,i)=>(i+1)+'. '+x).join('\n')||'无',rulesLabels={protocol:'模型协议',context:'上下文窗口',modalities:'输入模态',efforts:'可选推理档位',default:'默认推理档位',fast:'Fast 能力',force:'请求强制档',omit:'不传推理参数',authority:'声明模式'};
  const mode=v=>({whitelist:'指定模型',all:'全部模型',none:'全部隐藏',native:'官方管理'}[v]||'无'),fast=v=>({on:'开启',off:'关闭',inherit:'继承'}[v]||'无');
  function group(id,name){const g={id,name,rows:[]};groups.push(g);return g;}
  function add(g,label,x,y,format=v=>v==null?'继承':String(v)){if(!same(x,y))g.rows.push({label,before:format(x),after:format(y)});}
  function rules(g,prefix,x={},y={}){for(const key of new Set([...Object.keys(x),...Object.keys(y)]))add(g,prefix+(rulesLabels[key]||key),x[key],y[key],v=>v===undefined?'继承':key==='force'?(labels[v]||v):key==='omit'?(v?'是':'否'):key==='authority'?(v==='force'?'强制使用本层声明':v):ruleText(key,v));}
  function refs(g,prefix,x={},y={}){for(const f of new Set([...Object.keys(x),...Object.keys(y)]))add(g,prefix+f,x[f],y[f]);}
  const global=group('global','全局设置');
  add(global,'Fast 请求策略',a.fastMode,b.fastMode,fast);add(global,'独立 Fast 项',a.fastRows,b.fastRows,v=>v?'显示':'隐藏');add(global,'系统占用',a.hidden,b.hidden);
  refs(global,'族模版 · ',a.globalRefs,b.globalRefs);
  add(global,'渠道顺序',a.providers.map(p=>p.id),b.providers.map(p=>p.id),list);
  for(const id of new Set([...a.providers.map(p=>p.id),...b.providers.map(p=>p.id)])){
   const old=a.providers.find(p=>p.id===id),next=b.providers.find(p=>p.id===id),g=group(id,(next||old).name+' · '+id),x=old||{},y=next||{};
   add(g,'渠道对象',!!old,!!next,v=>v?'存在':'不存在');
   add(g,'名称',x.name,y.name,v=>v||'无');add(g,'接口协议',x.protocol,y.protocol,v=>v||'无');add(g,'启用',x.enabled,y.enabled,v=>v==null?'无':v?'启用':'停用');
   add(g,'服务节点',serviceNodes(x),serviceNodes(y),v=>v.map(n=>n.name+' · '+n.id+' · '+(n.url||'待配置地址')).join('\n')||'无');
   add(g,'当前节点',x.activeNodeId,y.activeNodeId,v=>v||'未设置');
   const xc=x.connection||{},yc=y.connection||{};
   add(g,'当前节点 URL',activeNode(x)?.url,activeNode(y)?.url,v=>v||'无');add(g,'认证方式',xc.auth,yc.auth,v=>v==='apikey'?'API Key':v==='none'?'无需认证':'无');
   add(g,'密钥配置状态',xc.keyConfigured,yc.keyConfigured,v=>v?'已配置（不显示密钥）':'未配置');
   add(g,'模型选择模式',x.mode,y.mode,mode);add(g,'渠道 Fast 策略',x.fastEnabled,y.fastEnabled,fast);
   add(g,'白名单 / 有效选择',old?selected(old).map(m=>m.id):[],next?selected(next).map(m=>m.id):[],list);
   const xm=x.models||[],ym=y.models||[];
   add(g,'发现清单',xm.map(m=>m.id).sort(),ym.map(m=>m.id).sort(),list);
   if(old&&next){add(g,'内部模型顺序',xm.map(m=>m.id),ym.map(m=>m.id),list);add(g,'发现更新时间',x.updated,y.updated,v=>v||'未获取');}
   rules(g,'渠道默认 · ',x.defaults,y.defaults);refs(g,'渠道族模版 · ',x.refs,y.refs);
   for(const f of new Set([...Object.keys(x.familyOverrides||{}),...Object.keys(y.familyOverrides||{})]))rules(g,f+' 族覆盖 · ',x.familyOverrides?.[f],y.familyOverrides?.[f]);
   for(const key of new Set([...xm.map(m=>m.key),...ym.map(m=>m.key)])){
    const before=xm.find(m=>m.key===key)||{},after=ym.find(m=>m.key===key)||{},prefix=(after.id||before.id)+' · ';
    if(before.id&&after.id)add(g,prefix+'模型族',before.family,after.family);add(g,prefix+'模版版本',before.templateRef,after.templateRef);
    add(g,prefix+'别名',before.naming?.alias,after.naming?.alias,v=>v||'未设置');add(g,prefix+'显示名称',before.naming?.displayName,after.naming?.displayName,v=>v||'上游 ID');
    if(old&&next&&before.id&&after.id)add(g,prefix+'启停',modelActive(old,before),modelActive(next,after),v=>v?'启用':'停用');
     if(old&&next&&before.id&&after.id){add(g,prefix+'最终协议',resolveRoute(old,before,a).protocol,resolveRoute(next,after,b).protocol);add(g,prefix+'协议来源',resolveRoute(old,before,a).source,resolveRoute(next,after,b).source);}
    rules(g,prefix+'模型例外 · ',before.overrides,after.overrides);
    if(old&&next&&before.id&&after.id)rules(g,prefix+'上游声明 · ',before.official,after.official);
   }
   // 清单获取会更新本机记录；不把它伪装成推理 / Fast 能力改变。
   if(!g.rows.length&&!same(x,y))g.rows.push({label:'发现记录',before:'已保存记录',after:'已更新记录（配置字段未变化）'});
  }
  return groups.filter(g=>g.rows.length);
 }
 function changeValueHTML(value){const text=String(value),n=text.split('\n').length;return n>4?'<details class="mp-change-values"><summary>'+n+' 项 · 展开查看</summary><div>'+esc(text)+'</div></details>':esc(text);}
 function changeRowsHTML(rows,before,after){return window.PrototypeTreeTable.rows(rows,r=>({className:'mp-change-row',depth:1,label:r.label,cells:['<span><small>'+esc(before)+'</small>'+changeValueHTML(r.before)+'</span>','<span><small>'+esc(after)+'</small>'+changeValueHTML(r.after)+'</span>']}));}
 function policyDiffHTML(a,b,locate=false,allOpen=false){const groups=policyChanges(a,b);return '<div class="mp-changes">'+groups.map((g,i)=>window.PrototypeTreeTable.details({label:g.name,open:allOpen||i===0,nameHtml:esc(g.name)+'<span class="mp-badge">'+g.rows.length+' 项变化</span>',body:(locate&&b.providers.some(p=>p.id===g.id)?'<div class="mp-change-actions">'+button(g.id==='native'?'查看':'管理','edit-provider','data-id="'+esc(g.id)+'"')+'</div>':'')+'<div class="mp-change-head"><span>配置项</span><span>已保存</span><span>当前草稿</span></div>'+changeRowsHTML(g.rows,'已保存','当前草稿')})).join('')+'</div>';}
 function setSelection(models,value){const p=provider();if(p.mode==='all'){p.models.forEach(m=>m.selected=true);p.mode='whitelist';}models.forEach(m=>m.selected=value);if(!p.models.some(m=>m.selected))p.mode='none';else if(p.mode==='none')p.mode='whitelist';}
 function selectModels(models,value){const p=provider(),remaining=selected(p).filter(m=>!models.includes(m)).length;if(!value&&remaining===0)modal('隐藏本渠道全部模型？','<p>将使用明确的“全部隐藏”模式，保留已发现清单和顺序；不会以空 selectedModels 冒充隐藏。</p>','加入草稿',()=>change(()=>setSelection(models,value)));else change(()=>setSelection(models,value));}
 function move(list,id,direction,key='id'){const i=list.findIndex(x=>x[key]===id),j=i+direction;if(i<0||j<0||j>=list.length)return;[list[i],list[j]]=[list[j],list[i]];}
 function discoveredModel(pid,id,source='upstream'){
  const family=/^gpt-/i.test(id)?'GPT':/^claude-/i.test(id)?'Claude':/^gemini-/i.test(id)?'Gemini':/^qwen/i.test(id)?'Qwen':/^deepseek/i.test(id)?'DeepSeek':/^grok/i.test(id)?'Grok':'未归族';
  return {key:pid+'::'+id,id,family,selected:false,source,official:{context:null,modalities:[],efforts:[],default:null,fast:'unknown'},overrides:{}};
 }
 function discoveryFixture(pid){return ['gpt-5.4','gpt-5.4-mini','gpt-5.3-codex','claude-sonnet-4-6','gemini-3-flash-preview','qwen3-coder','deepseek-v3.2','upstream-custom-demo'].map(id=>discoveredModel(pid,id));}
 function mergeDiscovered(old,found){const ids=new Set(found.map(m=>m.id));return [...old.map(m=>({...m,discovered:ids.has(m.id)})),...found.filter(m=>!old.some(x=>x.id===m.id))];}
 function refresh(){const p=provider(),token=++p.refreshToken;p.refresh='loading';render();setTimeout(()=>{if(p.refreshToken!==token||!draft.providers.includes(p))return;p.refresh=scenario==='discovery-fail'?'failed':scenario==='discovery-empty'?'empty':'idle';if(p.refresh==='idle'&&p.addedInPrototype)p.models=mergeDiscovered(p.models,[...discoveryFixture(p.id),discoveredModel(p.id,'upstream-added-demo')]);p.updated='刚刚（演示）';render();notice(p.refresh==='idle'?'模拟刷新完成，新增项未勾选；原白名单和位置保留':'未覆盖原清单或白名单');},650);}
 function channelConnection(edit=false,inline=false){
  const p=edit?provider():null;if(p?.id==='native'||busy())return;
  if(p&&stagedModelEdits().some(([key])=>p.models.some(m=>m.key===key))){notice('请先确认或还原本渠道的行编辑，再调整连接');return;}
  const b=p&&channelConnections[p.id];if(b&&b.baseline!==connectionBaseline(p)){notice('连接基准已变化，请还原配置后重新编辑');return;}
  channelWizard={editId:p?.id||'',id:p?.id||'',name:p?.name||'',protocol:p?.protocol||'Responses',connection:p?nodeConnection(p):{baseUrl:'',auth:'apikey',keyConfigured:false},models:clone(p?.models||[]),step:1,status:'idle',filter:'',only:false,keyChanged:0,previewKeyVersion:0,retainedKeyConfigured:p?.connection?.keyConfigured||false,error:'',retrieved:false};
  if(p){const chosen=new Set(selected(p).map(m=>m.id));channelWizard.models.forEach(m=>m.selected=chosen.has(m.id));}
  channelWizard.previous=p?JSON.stringify({protocol:p.protocol,connection:nodeConnection(p)}):'';
  channelWizard.providerDigest=p?JSON.stringify(p):'';channelWizard.bufferDigest=b?JSON.stringify(b):'';
  if(b){Object.assign(channelWizard,{name:b.name.trim(),protocol:b.protocol,connection:clone(b.connection),keyChanged:b.keyChanged,retainedKeyConfigured:b.retainedKeyConfigured});channelWizard.connection.baseUrl=channelWizard.connection.baseUrl.trim().replace(/\/+$/,'');}
  if(inline){channelWizard.error=validateConnection();if(channelWizard.error){notice(channelWizard.error);channelWizard=null;return;}const same=channelWizard.previous===JSON.stringify({protocol:channelWizard.protocol,connection:channelWizard.connection})&&!channelWizard.keyChanged;if(!same){channelWizard.models=[];channelWizard.retrieved=false;channelWizard.status='idle';}channelWizard.previous=JSON.stringify({protocol:channelWizard.protocol,connection:channelWizard.connection});channelWizard.previewKeyVersion=channelWizard.keyChanged;channelWizard.step=2;}
  wizardToken++;drawChannelWizard();
 }
 function captureConnection(){const w=channelWizard;if(!w||w.step!==1)return;w.name=$('#mpChannelName').value.trim();w.id=$('#mpChannelId').value.trim();w.protocol=$('#mpChannelProtocol').value;w.connection.baseUrl=$('#mpChannelURL').value.trim().replace(/\/+$/,'');w.connection.auth=$('#mpChannelAuth').value;w.connection.keyConfigured=w.connection.auth==='apikey'&&(w.retainedKeyConfigured||$('#mpChannelKey').value.length>0);}
 function validateConnection(){
  const w=channelWizard;if(!w.name)return '请填写渠道名称。';
  if(!/^[a-zA-Z0-9][a-zA-Z0-9._-]{0,63}$/.test(w.id)||['native','openai','system','constructor','prototype','__proto__'].includes(w.id.toLowerCase()))return '渠道标识使用字母、数字、点、下划线或连字符；不能使用保留标识。';
  if(draft.providers.some(p=>p.id!==w.editId&&(p.id.toLowerCase()===w.id.toLowerCase()||p.name.toLowerCase()===w.name.toLowerCase())))return '渠道名称或标识已存在，请换一个。';
  try{const url=new URL(w.connection.baseUrl);if(!['https:','http:'].includes(url.protocol)||url.username||url.password||url.search||url.hash)throw Error();}catch{return '请填写完整 HTTP(S) Base URL，不附密钥、查询参数或模型接口路径。';}
  if(/\/(models|responses|chat\/completions)\/?$/i.test(w.connection.baseUrl))return '请填写 Base URL，不填写 /models 或生成接口。';
  if(w.connection.auth==='apikey'&&!w.connection.keyConfigured)return '请填写演示 API Key，或选择无需认证。';return '';
 }
 function drawChannelWizard(){
  const w=channelWizard;if(!w)return;const first=w.step===1,loading=w.status==='loading',title=(w.editId?'编辑渠道连接':'新增渠道')+' · '+(first?'连接配置':'选择模型');
  const steps='<ol class="mp-wizard-steps" aria-label="新增渠道步骤"><li '+(first?'aria-current="step"':'')+'>1 连接配置</li><li '+(!first?'aria-current="step"':'')+'>2 选择模型</li></ol>';
  const help=w.protocol==='Ollama'?'Ollama：例如 http://localhost:11434；使用原生适配器。':w.protocol==='Anthropic'?'Anthropic：使用服务根地址或 /v1。':w.protocol==='Google'?'Google AI Studio：使用服务根地址，不附 /v1beta/models。':'OpenAI 兼容：例如 https://newapi.example.com/v1，系统按适配器获取模型。';
  const form='<div class="mp-agent-form"><label>渠道名称<input class="input" id="mpChannelName" maxlength="80" value="'+esc(w.name)+'" placeholder="例如：NewApi 备用"></label><label>渠道标识<input class="input" id="mpChannelId" maxlength="64" value="'+esc(w.id)+'" placeholder="例如：newapi-backup" '+(w.editId?'readonly':'')+'><span class="mp-muted">保存后保持稳定，用来绑定模型策略。</span></label><label>默认接口协议<select class="mp-select" id="mpChannelProtocol">'+protocolOptions(w.protocol)+'</select></label><label>'+(w.editId?'当前节点 URL':'Base URL')+'<input class="input" id="mpChannelURL" type="url" value="'+esc(w.connection.baseUrl)+'" placeholder="https://newapi.example.com/v1" '+(w.editId?'readonly':'')+'><span class="mp-muted">'+help+'</span></label><label>认证方式<select class="mp-select" id="mpChannelAuth">'+opts([['apikey','API Key'],['none','无需认证']],w.connection.auth)+'</select></label>'+(w.connection.auth==='apikey'?'<label>API Key<input class="input" id="mpChannelKey" type="password" autocomplete="off" placeholder="'+(w.connection.keyConfigured?'已配置（不回显）；留空保留':'仅填演示值，请勿输入真实密钥')+'"><span class="mp-muted">本原型只记录是否已配置，不保存或发出输入值。</span></label>':'<input id="mpChannelKey" type="hidden" value="">')+'</div>';
  const shown=w.models.filter(m=>(!w.only||m.selected)&&m.id.toLowerCase().includes(w.filter.toLowerCase())),selectedCount=w.models.filter(m=>m.selected).length;
  const messages={idle:'先获取模型列表，或手动添加模型 ID。',loading:'正在模拟获取上游清单…',ready:'已获取 '+w.models.filter(m=>m.source!=='manual').length+' 个模型；新发现项默认不勾选。',failed:'获取失败（演示网络错误）。可以重试、检查连接，或手动添加 ID。',empty:'上游返回空清单（演示）；原选择保留，可重试或手动添加。',unauthorized:'认证失败（演示 401）。返回检查 API Key；不会发推理请求。',unsupported:'该服务未提供可用的模型清单接口（演示），请手动添加 ID。'};
  const list='<div class="mp-inline-notice"><span>'+esc(w.name)+'</span>'+button('返回连接配置','channel-back',loading?'disabled':'')+'</div><div class="mp-row">'+button(loading?'取消等待':w.retrieved?'重新获取':'获取模型列表',loading?'channel-cancel-fetch':'channel-fetch','',true)+'<span class="mp-muted" role="status">'+messages[w.status]+'</span></div><div class="mp-filter"><input class="input" id="mpChannelSearch" value="'+esc(w.filter)+'" placeholder="搜索模型 ID" aria-label="搜索待加入模型"><label class="mp-label"><input id="mpChannelOnly" type="checkbox" '+(w.only?'checked':'')+'>仅已选</label></div><div class="mp-row"><span id="mpChannelCount">已选 '+selectedCount+' / 清单 '+w.models.length+'</span>'+button('选择结果 '+shown.length+' 项','channel-select',loading?'disabled':'')+button('取消结果','channel-unselect',loading?'disabled':'')+'</div><div class="mp-discovery-list">'+shown.map(m=>'<label class="mp-discovery-item"><input type="checkbox" data-channel-model="'+esc(m.id)+'" '+(m.selected?'checked':'')+' '+(loading?'disabled':'')+'><span>'+esc(m.id)+'<small>'+esc(templateLabel({id:channelWizard.id},m))+' · '+(m.source==='manual'?'手动添加 / 能力未知':'上游清单（演示） / 能力待核对')+(m.discovered===false?' · 本次未发现':'')+'</small></span></label>').join('')+(shown.length?'':'<div class="mp-empty">没有匹配的模型</div>')+'</div><details class="mp-details"><summary>手动添加模型</summary><p>每行一个精确模型 ID，不按逗号拆分；去重后仍需勾选，能力保持未知。</p><textarea id="mpChannelManual" aria-label="手动模型 ID" placeholder="例如：gpt-5.4"></textarea>'+button('添加到清单','channel-manual',loading?'disabled':'')+'</details><p class="mp-muted">加入后在渠道详情调整族规则和顺序。发现继续开启；本次只是列表获取，不验证推理或 Fast。</p>';
  modal(title,steps+'<p class="mp-muted">静态演示表单，请勿输入真实密钥；不会连接上游。</p>'+(first?form+(w.editId?'<p class="mp-muted">更换协议、地址或认证后，重新获取并选择模型；原策略在页级保存前保持。</p>':''):list)+'<div class="mp-alert" id="mpChannelError" role="alert">'+esc(w.error)+'</div>',first?'下一步':'加入草稿',()=>{
   if(first){captureConnection();w.error=validateConnection();if(w.error){$('#mpChannelError').textContent=w.error;return;}w.retainedKeyConfigured=w.connection.keyConfigured;$('#mpChannelKey').value='';const same=w.previous===JSON.stringify({protocol:w.protocol,connection:w.connection})&&w.previewKeyVersion===w.keyChanged;if(!same){w.models=[];w.retrieved=false;w.status='idle';}w.previous=JSON.stringify({protocol:w.protocol,connection:w.connection});w.previewKeyVersion=w.keyChanged;w.step=2;drawChannelWizard();return;}
   if(w.status==='loading'||!w.models.some(m=>m.selected))return;
   const old=draft.providers.find(p=>p.id===w.editId);
   if(busy()||w.editId&&(JSON.stringify(old)!==w.providerDigest||JSON.stringify(channelConnections[w.editId]||'')!== (w.bufferDigest||JSON.stringify(''))||stagedModelEdits().some(([key])=>old?.models.some(m=>m.key===key)))){$('#mpChannelError').textContent='渠道或连接编辑已变化，请关闭后重新确认。';return;}
   const next=old?clone(old):{id:w.id,name:w.name,protocol:w.protocol,enabled:true,mode:'whitelist',models:[],defaults:{},familyOverrides:{},refs:{},fastEnabled:'inherit',refresh:'idle',refreshToken:0,updated:'刚刚（演示）',addedInPrototype:true};
   Object.assign(next,{name:w.name,protocol:w.protocol,connection:{auth:w.connection.auth,keyConfigured:w.connection.keyConfigured},models:clone(w.models),mode:'whitelist'});if(!old){next.nodes=[{id:'primary',name:'默认节点',url:nodeURL(w.connection.baseUrl)}];next.activeNodeId='primary';}if(w.previous!==JSON.stringify({protocol:old?.protocol,connection:old?nodeConnection(old):null})||w.keyChanged)next.availability={kind:'unknown'};if(!old||w.keyChanged||JSON.stringify(old.connection)!==JSON.stringify(next.connection))nodeAuthRevision[next.id]=(nodeAuthRevision[next.id]||0)+1;if(w.editId)delete channelConnections[w.editId];closeModal();change(()=>{if(old)draft.providers[draft.providers.indexOf(old)]=next;else draft.providers.push(next);current=next.id;channelDetail=true;search='';family='';onlySelected=false;channelSearch='';});notice('渠道和白名单已加入草稿；页级保存后才写入，随后重启 Codex');
  },{confirmKeepsOpen:true,onClose:()=>{const key=$('#mpChannelKey');if(key)key.value='';wizardToken++;channelWizard=null;}});
  $('#modalConfirm').disabled=!first&&(loading||!selectedCount);
  if(first){
   const invalidate=()=>{captureConnection();wizardToken++;w.retrieved=false;w.status='idle';w.error='';};
   $('#mpChannelName').oninput=()=>captureConnection();$('#mpChannelId').oninput=()=>captureConnection();$('#mpChannelURL').oninput=invalidate;
   $('#mpChannelKey').oninput=()=>{w.keyChanged++;invalidate();};
   $('#mpChannelProtocol').onchange=()=>{invalidate();w.retainedKeyConfigured=w.connection.keyConfigured;drawChannelWizard();};$('#mpChannelAuth').onchange=()=>{invalidate();w.retainedKeyConfigured=w.connection.keyConfigured;drawChannelWizard();};
  }else{
   $('#mpChannelSearch').oninput=()=>{w.filter=$('#mpChannelSearch').value;drawChannelWizard();const input=$('#mpChannelSearch');input.focus();input.setSelectionRange(w.filter.length,w.filter.length);};
   $('#mpChannelOnly').onchange=()=>{w.only=$('#mpChannelOnly').checked;drawChannelWizard();};
  }
 }
 function fetchChannelModels(){const w=channelWizard;if(!w||w.status==='loading')return;const token=++wizardToken;w.status='loading';w.error='';drawChannelWizard();setTimeout(()=>{if(channelWizard!==w||wizardToken!==token)return;w.status={'discovery-fail':'failed','discovery-empty':'empty','discovery-unauthorized':'unauthorized','discovery-unsupported':'unsupported'}[scenario]||'ready';if(w.status==='ready'){w.models=mergeDiscovered(w.models,discoveryFixture(w.id));w.retrieved=true;}drawChannelWizard();},650);}
 function wizardSelection(value){const w=channelWizard;if(!w||w.status==='loading')return;w.models.filter(m=>(!w.only||m.selected)&&m.id.toLowerCase().includes(w.filter.toLowerCase())).forEach(m=>m.selected=value);drawChannelWizard();}
 function wizardManual(){const w=channelWizard;if(!w||w.status==='loading')return;const ids=$('#mpChannelManual').value.split(/\r?\n/).map(id=>id.trim()).filter(Boolean);if(!ids.length)return;
  if(ids.some(id=>id.length>200||/[\s\x00-\x1f]/.test(id))){$('#mpChannelError').textContent='模型 ID 不能含空白 / 控制字符，且须不超过 200 字符。';return;}
  [...new Set(ids)].forEach(id=>{if(!w.models.some(m=>m.id===id))w.models.push(discoveredModel(w.id,id,'manual'));});w.error='';drawChannelWizard();
 }
 function manualDetailModel(){
  const p=provider();if(!p||p.id==='native'||busy())return;
  const digest=channelDraftFingerprint(),pid=p.id;
  modal('手动添加模型','<p class="mp-muted">每行一个精确上游模型 ID；不按逗号拆分。新添加项默认停用、能力未知，可在列表中继续配置并统一保存。</p><label>模型 ID<textarea id="mpDetailManual" class="mp-json" rows="4" aria-label="手动的模型 ID" placeholder="例如：gpt-5.4"></textarea></label><div id="mpDetailManualError" class="mp-alert" role="alert"></div>','加入草稿',()=>{
   const error=$('#mpDetailManualError'),ids=$('#mpDetailManual').value.split(/\r?\n/).map(id=>id.trim()).filter(Boolean);
   if(busy()||digest!==channelDraftFingerprint()){error.textContent='渠道或编辑内容已变化，请关闭后重新添加。';return;}
   if(!ids.length){error.textContent='请至少填写一个模型 ID。';return;}
   if(ids.some(id=>id.length>200||/[\s\x00-\x1f]/.test(id))){error.textContent='模型 ID 不能含空白 / 控制字符，且须不超过 200 字符。';return;}
   const unique=[...new Set(ids)].filter(id=>!p.models.some(m=>m.id===id));
   if(!unique.length){error.textContent='这些模型已在当前清单中，无需重复添加。';return;}
   closeModal();change(()=>unique.forEach(id=>p.models.push(discoveredModel(pid,id,'manual'))));notice('已添加 '+unique.length+' 个模型到草稿，默认停用');
  },{confirmKeepsOpen:true});
 }
 let serviceNodeSerial=0;
 function nodeEditable(p){
  if(!p||p.id==='native'||busy())return false;
  if(stagedChannelConnections().some(([id])=>id===p.id)){notice('先确认或还原本渠道的认证配置，再管理节点');return false;}
  return true;
 }
 function nodeURL(value){
  if(value.length>2048)throw Error('节点 URL 过长。');
  let url;try{url=new URL(value.trim());}catch{throw Error('请填写完整 HTTP(S) Base URL。');}
  if(!['https:','http:'].includes(url.protocol)||url.username||url.password||url.search||url.hash||/\/(models|messages|responses|chat\/completions)\/?$/i.test(url.pathname))throw Error('请填写服务 Base URL，不附密钥、查询参数或生成接口路径。');
  return url.href.replace(/\/+$/,'');
 }
 function nodeChanged(p){delete channelConnections[p.id];p.refreshToken++;p.refresh='idle';if(p.availability?.kind!=='no-credit')p.availability={kind:'unknown'};}
 function editServiceNode(id=''){
  const p=provider();if(!nodeEditable(p))return;const old=serviceNodes(p).find(n=>n.id===id);if(id&&!old)return;
  const pid=p.id,baseline=channelDraftFingerprint();
  modal(old?'编辑服务节点':'新增服务节点','<div class="mp-agent-form"><label>节点名称<input id="mpServiceNodeName" class="input" maxlength="80" value="'+esc(old?.name||'')+'" placeholder="例如：备用节点"></label><label>URL<input id="mpServiceNodeURL" class="input" type="url" value="'+esc(old?.url||'')+'" placeholder="https://gateway.example.com/v1"></label></div><p class="mp-muted">共用渠道的认证与 API Key；新增节点后可手动设为当前。</p><div id="mpServiceNodeError" class="mp-alert" role="alert"></div>','加入草稿',()=>{
   const target=draft.providers.find(p=>p.id===pid),errorNode=$('#mpServiceNodeError');
   if(busy()||baseline!==channelDraftFingerprint()){errorNode.textContent='渠道或编辑内容已变化，请关闭后重新编辑节点。';return;}
   const name=$('#mpServiceNodeName').value.trim();let url;
   try{url=nodeURL($('#mpServiceNodeURL').value);}catch(e){errorNode.textContent=e.message;return;}
   if(!name||name.length>80){errorNode.textContent='请填写 1–80 字的节点名称。';return;}
   if(serviceNodes(target).some(n=>n.id!==id&&(n.url===url||n.name.toLowerCase()===name.toLowerCase()))){errorNode.textContent='节点名称或 URL 已存在。';return;}
   if(!old&&serviceNodes(target).length>=24){errorNode.textContent='最多配置 24 个节点。';return;}
   let nextId=old?.id;if(!nextId)do{nextId='node-'+(++serviceNodeSerial);}while(serviceNodes(target).some(n=>n.id===nextId));
   const entry={id:nextId,name,url},changed=!old||old.name!==name||old.url!==url;
   closeModal();if(!changed)return;
   change(()=>{target.nodes||=[];if(old)target.nodes[target.nodes.findIndex(n=>n.id===id)]=entry;else target.nodes.push(entry);if(!target.activeNodeId)target.activeNodeId=entry.id;if(target.activeNodeId===entry.id&&(!old||old.url!==url))nodeChanged(target);});
   notice('节点已加入草稿，保存后生效');
  },{confirmKeepsOpen:true});
 }
 function useServiceNode(id,pid=current){
  const p=draft.providers.find(p=>p.id===pid),node=serviceNodes(p).find(n=>n.id===id);if(!nodeEditable(p)||!node||p.activeNodeId===id)return;
  try{nodeURL(node.url);}catch(e){notice(e.message);return;}
  change(()=>{p.activeNodeId=id;nodeChanged(p);});notice('已将 '+node.name+' 设为草稿当前节点；保存后生效');
 }
 function deleteServiceNode(id){
  const p=provider(),node=serviceNodes(p).find(n=>n.id===id);if(!nodeEditable(p)||!node)return;
  if(id===p.activeNodeId||serviceNodes(p).length<=1){notice('当前节点不可删除，请先切换到其他节点；至少保留一条');return;}
  const pid=p.id,baseline=channelDraftFingerprint();
  modal('删除服务节点？','<p>删除 '+esc(node.name)+'（'+esc(node.url)+'）。渠道认证、模型清单与当前节点保留。</p><div id="mpServiceNodeError" class="mp-alert" role="alert"></div>','加入草稿',()=>{
   if(busy()||baseline!==channelDraftFingerprint()){$('#mpServiceNodeError').textContent='渠道或编辑内容已变化，请重新确认。';return;}
   closeModal();change(()=>{const p=draft.providers.find(p=>p.id===pid);p.nodes=p.nodes.filter(n=>n.id!==id);});
  },{confirmKeepsOpen:true});
 }
 function editModel(p,m){
  const edit=modelEdits[m.key],initialRules=clone(edit?.rules||m.overrides),inherited=resolve(p,m,draft,true),digest=policyDigest();let next=initialRules;
  const currentResolved=resolve(p,m),extra='<p>上游 ID：<code>'+esc(m.id)+'</code>。别名仅用于选择和识别；请求、模版匹配与排序保留上游 ID。</p><p class="mp-muted">当前协议 '+esc(currentResolved.values.protocol)+'，来源 '+esc(currentResolved.sources.protocol)+'</p><div class="mp-model-detail-naming"><label class="mp-label">别名<input id="mpModelAlias" class="input" value="'+esc(m.naming?.alias||'')+'" placeholder="可选"></label><label class="mp-label">显示名称<input id="mpModelDisplayName" class="input" value="'+esc(m.naming?.displayName||'')+'" placeholder="默认上游 ID"></label><label class="mp-label"><input type="checkbox" id="mpModelEnabled" '+(modelActive(p,m)?'checked':'')+'>启用此模型</label></div><p class="mp-muted">协议更改后核对推理和快速模式；访问地址统一使用渠道当前节点。</p>';
  const bind=()=>bindFields($('[data-field-editor]',$('#modalBody')),next,inherited,(v,ok)=>{next=v;$('#modalConfirm').disabled=!ok;});
  modal('编辑明细 — '+m.id,extra+fieldEditor(next,inherited)+'<div id="mpModelDetailError" class="mp-alert" role="alert"></div><p class="mp-muted">确认后加入草稿，统一保存后提示重启。</p>','加入草稿',()=>{
   const errorNode=$('#mpModelDetailError');if(busy()||digest!==policyDigest()){errorNode.textContent='配置已变化，请重新打开明细。';return;}const root=$('[data-field-editor]',$('#modalBody'));next=readFields(root,next);if(!validate(root,next,inherited))return;
   const error=commitModelPatches(p.id,[{key:m.key,rules:next,naming:{alias:$('#mpModelAlias').value,displayName:$('#mpModelDisplayName').value},enabled:$('#mpModelEnabled').checked,...(edit?{baseline:edit.baseline}:{})}]);if(error){errorNode.textContent=error;return;}closeModal();
  },{confirmKeepsOpen:true,extraLabel:'恢复字段继承',onExtra:()=>{next={};const root=$('[data-field-editor]',$('#modalBody'));root.outerHTML=fieldEditor({},inherited);bind();$('#modalConfirm').disabled=false;}});bind();
 }
 function providerConfig(){const p=provider();editFields(p.name+' · 渠道默认',p.defaults,{values:{},sources:{}},rules=>change(()=>p.defaults=rules),'<p>作用于渠道内所有模型；未覆盖字段继续沿各自模板或声明解析。</p>');}
 function batch(){const targets=draft.providers.filter(p=>scope.has(p.id)&&p.id!=='native');if(!targets.length){notice('请选择可配置的自定义渠道');return;}modal('按族批量配置','<p>范围：'+targets.map(p=>esc(p.name)).join('、')+'。按已采用精确版本匹配；先采用模版，再设置该族覆盖。白名单不因此改变。</p><label class="mp-label">模型族 / 模版<select id="mpBatchFamily" class="mp-select">'+opts(families(),'gpt')+'</select></label><div id="mpBatchError" class="mp-alert"></div>','下一步',()=>{const f=$('#mpBatchFamily').value,hits=targets.flatMap(p=>p.models.filter(m=>templateResolution(p,m).id===f));if(!hits.length){$('#mpBatchError').textContent='该范围没有采用并匹配此模版的模型，请先采用精确版本';return;}closeModal();editFields(f+' · '+targets.length+' 个渠道',{},null,rules=>change(()=>targets.forEach(p=>p.familyOverrides[f]=clone(rules))),'<p>匹配 '+hits.length+' 个模型，覆盖位置为渠道族；保留单模型例外。</p>');},{confirmKeepsOpen:true});}
 function adopt(exact=false){if(exact&&templateEditorChanged()){notice('先保存模版修改，再采用已保存版本');return;}const t=template(),v=version(t.id,versionN),chosen=exact?t.id:'gpt',targetIds=new Set(scope);modal('采用模版精确版本','<label class="mp-label">模版 <select id="mpAdoptTemplate" class="mp-select">'+opts(templates.map(x=>[x.id,x.name]),chosen)+'</select></label><label class="mp-label">精确版本 <select id="mpAdoptVersion" class="mp-select"></select></label><label class="mp-label">范围 <select id="mpAdoptScope" class="mp-select">'+opts([['channels','所选渠道的该模型族'],['global','全部渠道的全局族默认']],'channels')+'</select></label><div class="mp-checks" id="mpAdoptChannels">'+draft.providers.filter(p=>p.id!=='native').map(p=>'<label><input type="checkbox" id="mpAdoptChannel-'+esc(p.id)+'" '+(targetIds.has(p.id)?'checked':'')+'>'+esc(p.name)+(p.enabled?'':' · 停用')+'</label>').join('')+'</div><p id="mpAdoptInfo"></p><label class="mp-label"><input id="mpAdoptClear" type="checkbox">明确清除目标单模型例外</label><label class="mp-label"><input id="mpAdoptMain" type="checkbox">设为默认（独立主指针动作）</label><div id="mpAdoptError" class="mp-alert"></div>','采用到草稿',()=>{const id=$('#mpAdoptTemplate').value,n=Number($('#mpAdoptVersion').value),type=$('#mpAdoptScope').value,tt=templates.find(x=>x.id===id),vv=version(id,n),targets=draft.providers.filter(p=>targetIds.has(p.id)&&p.id!=='native');if(!vv){$('#mpAdoptError').textContent='请选择已保存版本';return;}if(type==='channels'&&!targets.length){$('#mpAdoptError').textContent='请勾选要采用的渠道；仅浏览渠道不自动加入范围';return;}const clear=$('#mpAdoptClear').checked,main=$('#mpAdoptMain').checked;closeModal();change(()=>{if(type==='global')draft.globalRefs[tt.id]=id+':'+n;else targets.forEach(p=>{p.refs[tt.id]=id+':'+n;if(clear)p.models.filter(m=>templateMatches(tt,m,p,vv)).forEach(m=>{m.overrides={};});});});if(main)tt.main=n;tab='channels';render();notice('精确版本已加入草稿；已有覆盖保留，保存后才写入'+(main?'。主指针已更新':''));},{confirmKeepsOpen:true});const fill=()=>{const tt=templates.find(x=>x.id===$('#mpAdoptTemplate').value);$('#mpAdoptVersion').innerHTML=opts(tt.versions.map(v=>[v.n,v.versionLabel+' · '+versionState(tt.id,v.n)]),exact&&tt.id===t.id?v.n:tt.main);$('#mpAdoptInfo').textContent='主版本仅影响默认版本。模板来源选择一份；输出协议覆盖渠道默认，已有族 / 单模型例外保留。当前节点地址和渠道认证须在保存前补齐。当前所选：'+[...targetIds].join('、');};fill();$('#mpAdoptTemplate').onchange=fill;draft.providers.filter(p=>p.id!=='native').forEach(p=>{const el=$('#mpAdoptChannel-'+p.id);el.onchange=()=>{el.checked?targetIds.add(p.id):targetIds.delete(p.id);fill();};});$('#mpAdoptScope').onchange=()=>{$('#mpAdoptChannels').hidden=$('#mpAdoptScope').value==='global';};}
 function saveDialog(){if(stagedChannelConnections().length){notice('请先确认或还原未加入草稿的连接配置');return;}if(stagedModelEdits().length){notice('请先处理未加入草稿的行编辑');return;}if(busy()||count()>100||!dirty())return;const namingErrors=draft.providers.filter(p=>p.id!=='native').map(namingError).filter(Boolean);if(namingErrors.length){modal('模型别名需要修正',diffHTML(namingErrors),'返回编辑');return;}const routeErrors=routePlan().errors;if(routeErrors.length){modal('协议配置尚未完成',diffHTML(routeErrors)+'<p>补齐端点或恢复协议继承后再保存；草稿保留。</p>','返回编辑');return;}const frozen=clone(draft),digest=policyDigest(),baseline=baseDigest(),items=diff(saved,frozen);modal('保存渠道模型配置','<p>以下修改将保存到全渠道配置；取消会保留草稿。</p>'+diffHTML(items)+policyDiffHTML(saved,frozen,false,true)+'<details class="mp-details"><summary>协议写入计划</summary>'+routePlanHTML(frozen)+'</details><p>完整投影 '+projection(frozen).length+'/100。默认提交全部草稿；冻结白名单、顺序、字段、Fast 与精确依赖。</p><p>先备份，再写入并核对目录，最后发布成功快照。保存后需要重启 Codex。</p><div id="mpSaveError" class="mp-alert"></div>','确认保存',()=>{if(stagedChannelConnections().length||stagedModelEdits().length||scenario==='drift'||digest!==policyDigest()||baseline!==baseDigest()){$('#mpSaveError').textContent='基准已变化，请关闭并重新预览。旧确认不会写入。';return;}closeModal();startSave(frozen);},{confirmKeepsOpen:true});}
 function startSave(plan){if(busy())return;const id=++operationId;operation={id,status:'running',step:0,plan,before:clone(saved),message:'校验冻结计划…'};render();function step(){if(operation?.id!==id||operation.status!=='running')return;operation.step++;if(operation.step===1&&scenario==='backup-fail'){operation.status='failed';operation.message='备份失败 · 未写入，草稿保留';render();return;}if(operation.step===3&&['partial','timeout'].includes(scenario)){operation.status=scenario==='timeout'?'timeout':'failed';operation.message=scenario==='timeout'?'写入响应超时 · 先核对，不重复提交':'部分写入失败 · 实际结果待核对，未发布成功快照';records.push({id,time:'刚刚',status:'部分 / 待核对',policy:clone(plan),before:clone(saved)});render();return;}if(operation.step>=5){saved=clone(plan);pending=true;client='运行目录已核对 · 客户端待重启';records.push({id,time:updates.stamp(records).versionLabel,restores:restoration?.recordId||null,status:'成功',policy:clone(plan),before:clone(operation.before),client});restoration=null;operation.status='success';operation.message='保存成功 · 待重启 Codex';render();return;}operation.message=['校验','备份','写入','核对目录','发布成功快照'][operation.step]+'…';render();setTimeout(step,350);}setTimeout(step,350);}
 function verify(){if(!operation)return;const partial=operation.step>=2;modal('核对实际结果','<p>'+ (partial?'演示核对：部分运行字段已有变化，但 Desktop 成功快照未发布。根据 journal 生成安全补偿，保留本次草稿。外部变化先比较，不能覆盖。':'演示核对：未发生运行写入，本次草稿保留。')+'</p>','完成核对',()=>{operation.status='verified';operation.message=partial?'已核对并模拟安全补偿 · 草稿保留，需重新保存':'已核对 · 未写入，草稿保留';render();});}
 function stopSave(){if(operation?.status!=='running')return;operation.status='stopped';operation.message=operation.step<2?'已取消 · 未写入，草稿保留':'已停止后续步骤 · 可能已有写入，先核对';render();}
 function checkSavedClient(){
  if(!pending)return;const id=operation?.id,baseline=JSON.stringify(saved);
  modal('重新检查 Codex 客户端','<p>演示目录核对通过；原型无法获取客户端加载证据。请目视确认模型顺序 / 选项后记录用户确认。</p><div id="mpClientCheckError" class="mp-alert" role="alert"></div>','我已目视确认',()=>{
   if(id!==operation?.id||baseline!==JSON.stringify(saved)){$('#mpClientCheckError').textContent='保存目标已变化，请重新检查最新配置。';return;}
   pending=false;client='用户已确认 · 非自动加载证据';closeModal();render();
  },{confirmKeepsOpen:true});
 }
 window.modelSaveNotificationAction=function(action,id){
  if(!operation||operation.id!==id)return;
  if(action==='stop')stopSave();
  else if(action==='verify'&&['timeout','failed','stopped'].includes(operation.status))verify();
  else if(action==='check'&&operation.status==='success')checkSavedClient();
 };
 function ensureTemplateBuffer(){
  if(buffer()){versionN=buffer().base;return;}const t=template(),v=version(templateId,versionN)||t?.versions[0];if(!t||!v)return;
  versionN=v.n;buffers[templateId]={name:t.name,description:v.description||'',match:clone(v.match),rules:clone(v.rules),...(v.variants?.length?{variants:clone(v.variants)}:{}),base:v.n,valid:true};editorDigest++;
 }
 function templateEditorChanged(id=templateId){const b=buffers[id];if(!b)return false;if(templateScratch?.id===id||b.valid===false)return true;const v=version(id,b.base),t=templates.find(t=>t.id===id);return !v||summaryEngine.summarize({...templateVersionSnapshot(t,v),name:t.name},b,{kind:'revision'}).changed;}
 function templateEdit(){ensureTemplateBuffer();render();}
 function openTemplateEditor(id,n=null){
  const t=templates.find(t=>t.id===id);if(!t)return;
  if(n&&version(id,n)&&buffers[id]?.base!==n&&!templateEditorChanged(id))delete buffers[id];
  templateId=id;versionN=buffers[id]?.base||n||t.main||t.versions[0]?.n||0;templateDetail=true;tab='templates';render();
 }
 function switchTemplateVersion(n){
  const v=version(templateId,n);if(!v)return;
  if(templateEditorChanged()){notice('先保存模版修改或返回放弃编辑，再切换版本');render();return;}
  if(generationOwner===templateId&&generating){generationToken++;generating=false;}if(proposal?.id===templateId)proposal=null;
  delete buffers[templateId];versionN=v.n;templateListVersions[templateId]=v.n;editorViewId=null;render();
 }

 function templateSaveChanges(b){
  const t=template(),v=version(t.id,b.base);if(v&&!summaryEngine.summarize({...templateVersionSnapshot(t,v),name:t.name},b,{kind:'revision'}).changed)return [];const before=v?{name:v.nameAtSave||t.name,description:v.description||'',match:v.match,rules:v.rules,variants:v.variants||[]}:{name:'',description:'',match:defaultMatch(),rules:{},variants:[]};
  const groups=[{name:'基本信息',rows:[]},{name:'匹配范围',rows:[]},{name:'族通用配置',rows:[]},{name:'版本或后缀',rows:[]}];
  const same=(x,y)=>JSON.stringify(x)===JSON.stringify(y),add=(g,label,x,y,format=v=>v==null||v===''?'未设置':String(v))=>{if(!same(x,y))g.rows.push({label,before:format(x),after:format(y)});};
  if(!v)groups[0].rows.push({label:'模版',before:'不存在',after:'新建自定义模型族'});
  add(groups[0],'名称',before.name,b.name.trim());add(groups[0],'说明',before.description,b.description);
  const mode=v=>({all:'全部渠道',whitelist:'白名单',blacklist:'黑名单'}[v]||v);
  const channels=xs=>xs.map(id=>(draft.providers.find(p=>p.id===id)?.name||id)+' · '+id).join('\n')||'无';
  add(groups[1],'渠道范围',channelMode(before.match),channelMode(b.match),mode);
  add(groups[1],'所选渠道',before.match.channels,b.match.channels,channels);
  for(const [key,label] of [['include','包含规则'],['exclude','排除规则']])add(groups[1],label,before.match[key],b.match[key],v=>matchLines(v)||'无');
  add(groups[1],'匹配优先级',before.match.priority,b.match.priority);
  add(groups[1],'区分大小写',before.match.caseSensitive,b.match.caseSensitive,v=>v?'是':'否');
  const names={protocol:'模型协议',context:'上下文窗口',modalities:'输入模态',efforts:'可选推理档位',default:'默认推理档位',fast:'Fast 能力',force:'请求强制档',omit:'不传推理参数',authority:'声明模式'};
  const rules=(g,prefix,x={},y={})=>{for(const k of new Set([...Object.keys(x),...Object.keys(y)]))add(g,prefix+(names[k]||k),x[k],y[k],v=>v===undefined?'继承 / 未声明':k==='force'?(labels[v]||v):k==='omit'?(v?'是':'否'):k==='authority'?(v==='force'?'强制使用本层声明':v):ruleText(k,v));};
  rules(groups[2],'',before.rules,b.rules);
  const old=before.variants,next=b.variants||[];
  for(const id of new Set([...old.map(v=>v.id),...next.map(v=>v.id)])){
   const x=old.find(v=>v.id===id),y=next.find(v=>v.id===id),prefix=(y?.name||x?.name)+' · ';
   add(groups[3],prefix+'条目',!!x,!!y,v=>v?'存在':'不存在');add(groups[3],prefix+'名称',x?.name,y?.name);add(groups[3],prefix+'后缀',x?.suffix,y?.suffix);rules(groups[3],prefix,x?.rules,y?.rules);
  }
  return groups.filter(g=>g.rows.length);
 }
 function templateSaveDiffHTML(groups){return groups.length?'<div class="mp-changes">'+groups.map(g=>'<section class="mp-template-change-group"><h3>'+esc(g.name)+' <span class="mp-badge">'+g.rows.length+' 项变化</span></h3><div class="mp-change-head"><span>配置项</span><span>基于版本</span><span>当前候选</span></div>'+changeRowsHTML(g.rows,'基于版本','当前候选')+'</section>').join('')+'</div>':'<p class="mp-inline-notice">内容与基于版本相同，不新增版本。</p>';}
 function discardTemplateEdit(id=templateId){
  if(generationOwner===id){generationToken++;generating=false;}if(proposal?.id===id)proposal=null;
  delete buffers[id];assistantOpen=false;
  if(templateScratch?.id===id){templateScratch=null;delete assistantInputs[id];conversation=conversation.filter(m=>m.templateId!==id);templateId=templates[0]?.id||'gpt';versionN=template()?.main||1;}
  editorDigest++;templateDetail=false;render();
 }
 function returnTemplates(){
  const b=buffer();if(!b){templateDetail=false;assistantOpen=false;render();return;}
  if(!templateScratch&&!templateSaveChanges(b).length){discardTemplateEdit();return;}
  saveTemplate(true);
 }
 function saveTemplate(returnAfter=false){
  const b=buffer();if(!b)return;const error=templateError(b)||(b.valid===false?'检查配置字段':'');
  if(error&&!returnAfter){b.validationRequested=true;$('#mpTemplateError').textContent=error;updateTemplateSaveStatus();return;}
  const t=template(),id=templateId,frozen=clone(b),digest=JSON.stringify(b),baseline=JSON.stringify(t),revision=editorDigest,n=nextVersionN(t),groups=templateSaveChanges(frozen);
  if(!groups.length&&!templateScratch){if(returnAfter)discardTemplateEdit(id);else notice('配置没有变化，不新增版本');return;}
  const unchanged=()=>templateId===id&&revision===editorDigest&&digest===JSON.stringify(buffer())&&baseline===JSON.stringify(template());
  const fail=()=>{$('#mpTemplateSaveError').textContent='编辑或基于版本已变化，请关闭并重新核对。';};
  const options={confirmKeepsOpen:true,...(returnAfter?{cancelLabel:'继续编辑',extraLabel:'放弃修改',onExtra:()=>{if(!unchanged()){fail();return;}closeModal();discardTemplateEdit(id);}}:{})};
  modal(returnAfter?'返回模版列表':'保存模版','<p>'+esc(returnAfter?'有未保存修改，选择保存新版本或放弃本次编辑。':frozen.base?'基于 '+versionText(id,frozen.base)+'，核对后保存新版本。':'核对后保存首个版本。')+'</p>'+templateSaveDiffHTML(groups)+(error?'<p class="mp-alert">'+esc(error)+'；可返回编辑修正，或放弃修改。</p>':'')+'<p class="mp-muted">保存只新增版本，已有默认指针与渠道采用关系保持。</p><div id="mpTemplateSaveError" class="mp-alert" role="alert"></div>',returnAfter?'保存并返回':'保存',()=>{
   if(!unchanged()){fail();return;}if(error){$('#mpTemplateSaveError').textContent=error;return;}
   const prior=version(id,frozen.base),summaryFacts=summaryEngine.summarize(prior?templateVersionSnapshot(t,prior):t.copySummaryBase||null,frozen,{kind:prior?'revision':t.copiedFrom?'copy':'create',templateId:prior?id:t.copiedFrom?.templateId||id,revisionId:prior?.revisionId||t.copiedFrom?.revisionId||null});if(!summaryFacts.changed&&!templateScratch){notice('内容无变化，不新增版本');return;}closeModal();t.name=frozen.name.trim();delete t.copySummaryBase;t.versions.push({...updates.stamp(templates.flatMap(t=>t.versions)),origin:t.copiedFrom&&!t.versions.length?'复制新建':'人工',...(t.copiedFrom&&!t.versions.length?{copiedFrom:clone(t.copiedFrom)}:{}),summaryFacts,summary:summaryFacts.text,nameAtSave:frozen.name.trim(),basedOn:version(id,frozen.base)?.revisionId||null,n,status:'候选',date:'刚刚',description:frozen.description,match:clone(frozen.match),rules:clone(frozen.rules),...(frozen.variants?.length?{variants:clone(frozen.variants)}:{}),source:'人工 / 演示候选；待核对'});
   t.nextRevision=n+1;if(templateScratch){templates.push(t);templateScratch=null;}versionN=n;templateListVersions[id]=n;delete buffers[id];editorDigest++;
   if(returnAfter){if(generationOwner===id){generationToken++;generating=false;}if(proposal?.id===id)proposal=null;templateDetail=false;assistantOpen=false;}else templateDetail=true;
   render();notice('已保存 '+versionText(id,n)+'；可比较、采用并恢复旧版本');
  },options);if(error)$('#modalConfirm').disabled=true;
 }
 function openTemplateActionMenu(anchor,label,items){
  if(!window.prototypeActionMenu){notice('动作菜单尚未加载，请刷新页面');return;}
  window.prototypeActionMenu.open({anchor,label,items});
 }
 function versionMenu(anchor){
  const t=template(),v=version(t.id,versionN);if(!v)return;const editing=templateEditorChanged();
  openTemplateActionMenu(anchor,'版本与历史',[
   {label:'版本管理',onSelect:()=>openTemplateHistory(t.id,'editor')},
   {label:'与版本比较',onSelect:()=>templateCompare()},
   {label:'恢复采用记录',onSelect:()=>templateRecovery(t.id)},
   {label:'确认这套配置可用',disabled:editing||!templateUses(saved,t.id,v.n).length||pending||dirty()||busy(),reason:editing?'先保存或退出编辑':'实际采用并核对后可确认',onSelect:()=>confirmTemplateUse()}
  ]);
 }
 function deleteTemplateVersion(ref){
  const [tid,value]=ref.split(':'),n=Number(value),t=templates.find(t=>t.id===tid);if(!t||versionProtected(tid,n)){notice('版本受保护：'+versionProtectionReasons(tid,n).join('、'));return;}
  removeTemplateVersions(tid,[n]);closeModal();render();
 }
 function setTemplateMain(ref,returnTo=false){const [tid,n]=ref.split(':'),view=returnTo&&templateHistoryView?.id===tid?clone(templateHistoryView):null;modal('设为默认','<p>默认版本改为 '+esc(versionText(tid,n))+'；已有渠道继续采用原版本。</p>','设为默认',()=>{const t=templates.find(t=>t.id===tid),v=version(tid,n);if(!t||!v||templateEditorChanged(tid)){notice('版本状态已变化，请重新核对');return;}t.main=Number(n);closeModal();render();if(view)openTemplateHistory(tid,view.context,view.query);});}

 function newTemplate(copy=false,copySource=null,copyOrigin=null){if(templateScratch){if(copy){notice('先保存或放弃已有的新模版，再复制');return;}templateId=templateScratch.id;templateDetail=true;tab='templates';render();notice('已返回未保存的新模版');return;}const source=copy?(copySource||templateContent()):null,copiedFrom=copy?(copyOrigin||{templateId,revisionId:version(templateId,buffer()?.base||versionN)?.revisionId||null,includesUnsaved:templateEditorChanged()}):null,id='custom-'+Date.now()+'-'+(++editorDigest);templateScratch={id,name:source?source.name+' 副本':'新模型模版',...(source?{copySummaryBase:clone(source)}:{}),originType:'user',createdVia:copy?'copy':'manual',main:0,nextRevision:1,versions:[],...(copiedFrom?{copiedFrom}:{})};templateId=id;versionN=0;buffers[id]={name:templateScratch.name,description:source?.description||'',match:source?clone(source.match):defaultMatch(),rules:source?clone(source.rules):{},...(source?.variants?.length?{variants:clone(source.variants)}:{}),base:0,valid:true};tab='templates';templateDetail=true;editorSection='basic';assistantOpen=false;generationTarget='';proposal=null;assistantTask='fill';render();}
 const updateFieldLabels={context:'上下文窗口',modalities:'输入模态',efforts:'可选推理档位',default:'默认推理档位',fast:'Fast 能力',include:'包含规则',exclude:'排除规则'};
 function updateContent(t,n){const b=buffers[t.id],v=version(t.id,n);return clone(b||{name:t.name,description:v.description||'',match:v.match,rules:v.rules,...(v.variants?.length?{variants:v.variants}:{})});}
 function openTemplateUpdate(selected=false){
  if(selected&&!templateSelection.size)return;
  if(!templateDetail){templateId=selected?[...templateSelection][0]:templateId;versionN=template().main||template().versions.at(-1)?.n;templateDetail=true;}
  assistantTask='update';updateScope=selected?'selected':'current';assistantOpen=true;tab='templates';render();
 }
 function startTemplateUpdate(scope='all'){
  if(updateRun?.status==='running'||generating){notice('已有检查或生成任务正在进行');return;}
  if(scope==='selected'&&!templateSelection.size){notice('请先勾选模版');return;}
  assistantTask='update';updateScope=scope;assistantOpen=true;tab='templates';render();
  checkTemplateUpdates({autoReview:true});
 }
 function renderUpdateProgress(){if(tab==='templates'&&!templateDetail)render();else renderAssistant();}
 function checkTemplateUpdates({autoReview=false}={}){
  if(updateRun?.status==='running'||generating)return;
  if(!agentReady()){notice('先配置 Agent 执行模型；当前编辑保留');agentSettings(autoReview?'update-'+updateScope:'conversation');return;}
  const targets=updateScope==='all'?templates:updateScope==='selected'?templates.filter(t=>templateSelection.has(t.id)):[template()];
  if(!targets.length||targets.some(t=>!t.versions.length)){notice('先保存新模版，再检查资料更新');return;}
  const id=++updateSerial;
  updateRun={id,status:'running',autoReview,owner:templateId,executor:agentSummary(),execution:agentExecution(),startedAt:Date.now(),items:targets.map(t=>{
   const n=buffers[t.id]?.base||(updateScope==='current'&&t.id===templateId?versionN:t.versions.at(-1).n)||t.versions.at(-1).n;
   return {id:t.id,name:t.name,n,content:updateContent(t,n),libraryDigest:JSON.stringify(t),bufferDigest:JSON.stringify(buffers[t.id]),rows:[],message:'正在核对资料',expanded:true};
  })};renderUpdateProgress();
  setTimeout(()=>{
   if(updateRun?.id!==id||updateRun.status!=='running')return;
   if(scenario==='update-fail'){updateRun.status='failed';updateRun.message='资料检索失败，已有版本和编辑内容保留。';renderUpdateProgress();return;}
   for(const item of updateRun.items){
    const seed=seedLibrary.find(s=>s.id===item.id);
    const suggestion={variants:seed?.models.map(m=>({id:m.id,name:m.name,suffix:m.suffix,rules:clone(m.rules)}))||[]};
    if(scenario==='update-conflict')suggestion.rules={context:1050000,default:'high'};
    item.rows=scenario==='update-none'?[]:updates.changes(item.content,suggestion,true);
    item.rows.forEach(row=>{row.label=updateFieldLabels[row.label]||row.label;row.source=(row.path[0]==='variants'?seed?.models.find(m=>m.id===row.path[1]||m.suffix===item.content.variants?.find(v=>v.id===row.path[1])?.suffix):null)||null;});
    item.message=item.rows.length?item.rows.length+' 项建议 · '+item.rows.filter(r=>r.conflict).length+' 项人工值保留':seed?.models.length?'资料已是最新':'暂无可自动采用的已核实型号资料';
   }
   updateRun.status='ready';renderUpdateProgress();
   if(autoReview&&tab==='templates'&&document.documentElement.dataset.route==='models'&&$('#modalMask')?.style.display!=='flex'){
    if(updateRun.items.some(i=>i.rows.length))reviewTemplateUpdates();else notice('全部所选模版已检查，没有需要保存的新版本');
   }
  },scenario==='update-late'?1400:650);
 }
 function updateSummary(){if(!updateRun)return '';const changed=updateRun.items.filter(i=>i.rows.length);return updateRun.status==='running'?'正在检查 '+updateRun.items.length+' 个模版':updateRun.status==='failed'?updateRun.message:changed.length?changed.length+' 个模版有更新建议':'没有需要保存的新版本';}
 function updateReviewHTML(){
  const run=updateRun;if(!run||run.status!=='ready')return '';
  return '<div class="mp-update-dialog"><p>逐项选择修改；已有人工值默认保留。保存为新版本，当前采用关系保持。</p><p class="mp-muted">'+(run.items.some(i=>buffers[i.id])?'涉及的当前编辑会一并保存；取消保留编辑。':'检查范围：'+run.items.length+' 个模版。')+'</p><div id="mpUpdateTree" class="proto-tree mp-update-tree" role="region" tabindex="0" aria-label="模型资料更新差异">'+run.items.map(item=>{
   return '<section class="mp-update-group"><header class="proto-tree-row mp-update-group-head"><input type="checkbox" data-update-group="'+esc(item.id)+'" aria-label="选择 '+esc(item.name)+' 的非冲突更新" '+(item.rows.some(r=>r.selected)?'checked':'')+(item.rows.length?'':' disabled')+'><button class="proto-tree-toggle" data-mp="update-expand" data-id="'+esc(item.id)+'" aria-expanded="'+item.expanded+'"><span class="proto-tree-chevron" aria-hidden="true"></span>'+esc(item.name)+'</button><small>'+esc(item.message)+'</small></header><div '+(item.expanded?'':'hidden')+'>'+item.rows.map(row=>{
    const key=item.id+':'+row.id,label=row.path[0]==='variants'&&!row.path[2]?row.after.name:row.label;
    return '<div class="proto-tree-row mp-update-row"><input type="checkbox" data-update-pick="'+esc(key)+'" aria-label="接受 '+esc(label)+'" '+(row.selected?'checked':'')+'><div class="proto-tree-leaf"><strong>'+esc(label)+'</strong><small>'+esc(row.conflict?'人工值默认保留 · 可明确勾选替换':row.before===undefined?'新增型号 / 字段':'资料变更')+'</small></div><div class="mp-update-values"><span>'+esc(row.before===undefined?'未配置':row.path[2]?ruleText(row.path[3],row.before):row.path[0]==='rules'?ruleText(row.path[1],row.before):JSON.stringify(row.before))+'</span><span>→ '+esc(row.path[0]==='variants'&&!row.path[2]?Object.keys(row.after.rules).length+' 项型号规则':row.path[2]?ruleText(row.path[3],row.after):row.path[0]==='rules'?ruleText(row.path[1],row.after):JSON.stringify(row.after))+'</span></div>'+button('查看详情','update-details','data-id="'+esc(key)+'"')+'</div>';
   }).join('')+(item.rows.length?'':'<p class="mp-muted mp-update-empty">'+esc(item.message)+'</p>')+'</div></section>';
  }).join('')+'</div><p id="mpUpdateSelection" class="mp-muted">'+run.items.flatMap(i=>i.rows).filter(r=>r.selected).length+' 项已选</p><div id="mpUpdateError" class="mp-alert" role="alert"></div></div>';
 }
 function reviewTemplateUpdates(){
  if(updateRun?.status!=='ready')return;const run=updateRun;
  modal('模型资料更新 · 差异选择',updateReviewHTML(),'保存新版本',()=>saveTemplateUpdates(run),{confirmKeepsOpen:true});syncUpdateChecks();
 }
 function syncUpdateChecks(){
  const run=updateRun;if(!run)return;
  for(const input of $$('[data-update-group]')){const rows=run.items.find(i=>i.id===input.dataset.updateGroup)?.rows||[],selected=rows.filter(r=>r.selected).length;input.checked=rows.length>0&&selected===rows.length;input.indeterminate=selected>0&&selected<rows.length;}
  const count=run.items.flatMap(i=>i.rows).filter(r=>r.selected).length;const note=$('#mpUpdateSelection');if(note)note.textContent=count+' 项已选';if($('#modalConfirm'))$('#modalConfirm').disabled=!count;
 }
 function redrawUpdateReview(){const root=$('#mpUpdateTree'),scroll=root?.scrollTop||0;$('#modalBody').innerHTML='<div class="mp-dialog">'+updateReviewHTML()+'</div>';const next=$('#mpUpdateTree');if(next)next.scrollTop=scroll;syncUpdateChecks();}
 function updateDetails(key){
  const [tid,rid]=key.split(':'),item=updateRun?.items.find(i=>i.id===tid),row=item?.rows.find(r=>r.id===rid);if(!row)return;
  const src=row.source;closeUpdateDetails();const layer=document.createElement('div');layer.id='mpUpdateDetail';layer.className='mp-update-detail';layer.setAttribute('role','dialog');layer.setAttribute('aria-modal','true');layer.setAttribute('aria-label','更新详情');
  const valueHTML=value=>value===undefined?'<p class="mp-muted">未配置</p>':row.path[0]==='variants'&&!row.path[2]?'<p><strong>'+esc(value.name)+'</strong> · 匹配后缀 '+esc(value.suffix)+'</p><dl class="mp-update-rule-details">'+Object.entries(value.rules).map(([k,v])=>'<dt>'+esc(updateFieldLabels[k]||k)+'</dt><dd>'+esc(ruleText(k,v))+'</dd>').join('')+'</dl><p class="mp-muted">其余字段继承。</p>':'<p>'+esc(ruleText(row.path[3]||row.path[1],value))+'</p>';
  const service={ 'vendor-api':'原厂 API','tencent-tokenhub':'腾讯 TokenHub','volcengine-ark':'火山方舟','alibaba-model-studio':'阿里百炼','minimax-m-plan-and-code':'MiniMax M Plan / Code' },life={documented:'已登记型号',preview:'预览版','preview-restricted':'限定访问预览版',rolling:'快速迭代版本'};
  layer.innerHTML='<section><header><h3>'+esc(row.label)+'</h3>'+button('关闭详情','update-details-close')+'</header><div class="mp-update-detail-body"><p>基于 '+esc(versionText(tid,item.n))+'</p><h4>当前值</h4>'+valueHTML(row.before)+'<h4>建议值</h4>'+valueHTML(row.after)+'<p>'+esc(row.conflict?'此项涉及人工配置；默认不替换，可返回差异页明确选择。':'仅在勾选后保存到新版本。')+'</p>'+(src?'<p>来源：<a href="'+esc(src.source)+'" target="_blank" rel="noopener noreferrer">官方型号资料</a> · '+esc(src.sourceAccess==='page'?'已取得正文':'仅索引摘要，待复核')+'</p><p>资料核对：2026-10-13 · '+esc(service[src.scope]||'按部署核对')+' · '+esc(life[src.lifecycle]||'待核对')+'</p>'+src.notes.map(note=>'<p class="mp-muted">'+esc(note)+'</p>').join(''):'<p>无型号级来源，建议为待核实示例。</p>')+'<p>本层只读，关闭后继续选择差异。</p></div></section>';
  layer.__returnFocus=document.activeElement;$('#modalBody').appendChild(layer);layer.querySelector('button')?.focus();
 }
 function closeUpdateDetails(){const layer=$('#mpUpdateDetail'),focus=layer?.__returnFocus;layer?.remove();focus?.focus();}
 document.addEventListener('keydown',event=>{
  const layer=$('#mpUpdateDetail');if(!layer)return;
  if(event.key==='Escape'){event.preventDefault();event.stopImmediatePropagation();closeUpdateDetails();}
  else if(event.key==='Tab'){const controls=$$('button,a[href]',layer),first=controls[0],last=controls.at(-1);if(event.shiftKey&&document.activeElement===first){event.preventDefault();last?.focus();}else if(!event.shiftKey&&document.activeElement===last){event.preventDefault();first?.focus();}}
 },true);
 function saveTemplateUpdates(run){
  const fail=message=>{const error=$('#mpUpdateError');if(error)error.textContent=message;};
  if(updateRun!==run||run.status!=='ready'){fail('任务已变化，请重新检查。');return;}
  const picked=run.items.filter(i=>i.rows.some(r=>r.selected));if(!picked.length){fail('请选择需要保存的修改。');return;}
  const plans=[];
  for(const item of picked){
   const t=templates.find(t=>t.id===item.id);if(!t||JSON.stringify(t)!==item.libraryDigest||JSON.stringify(buffers[item.id])!==item.bufferDigest){fail(item.name+' 的版本或编辑已变化，请重新检查。');return;}
   let content;try{content=updates.compose(item.content,item.rows);}catch(e){fail(e.message);return;}
   const error=templateError(content);if(error){fail(item.name+'：'+error);return;}
   const summaryFacts=summaryEngine.summarize(templateVersionSnapshot(t,version(t.id,item.n)),content,{kind:'revision',templateId:t.id,revisionId:version(t.id,item.n).revisionId});if(summaryFacts.changed)plans.push({t,item,content,summaryFacts});
  }
  if(!plans.length){run.status='unchanged';closeModal();render();notice('最终内容无变化，不新增版本');return;}
  const batch=updates.stamp(templates.flatMap(t=>t.versions));
  for(const {t,item,content,summaryFacts} of plans){
   const n=nextVersionN(t),meta=updates.stamp(templates.flatMap(t=>t.versions)),selected=item.rows.filter(r=>r.selected);
   t.versions.push({...meta,n,status:'Agent 更新',origin:'Agent 更新',basedOn:version(t.id,item.n).revisionId,nameAtSave:content.name,summaryFacts,summary:summaryFacts.text,batchId:batch.revisionId,description:content.description,match:clone(content.match),rules:clone(content.rules),variants:clone(content.variants||[]),source:'资料核对 2026-10-13；'+[...new Set(selected.map(r=>r.source?.source).filter(Boolean))].join('；')});
   t.nextRevision=n+1;t.name=content.name;delete buffers[t.id];templateListVersions[t.id]=n;item.savedN=n;
  }
  updateBatches.push({id:batch.revisionId,label:batch.versionLabel,versions:plans.map(({t,item})=>({id:t.id,before:item.n,after:item.savedN}))});
  run.status='saved';closeModal();editorDigest++;templateId=plans[0].t.id;versionN=plans[0].item.savedN;templateDetail=true;tab='templates';render();notice('已保存 '+plans.length+' 个新版本；采用后可试用，并保留恢复记录');
 }
 function templateRecovery(id=templateId){
  const t=templates.find(t=>t.id===id);if(!t)return;
  const entries=records.filter(r=>r.status==='成功'&&t.versions.some(v=>JSON.stringify(templateUses(r.before,id,v.n))!==JSON.stringify(templateUses(r.policy,id,v.n))));
  const confirmed=confirmedConfigurations.filter(c=>c.templateId===id).slice().reverse();
  modal('恢复 '+t.name+' 的采用配置','<p>选择上次采用记录或曾确认可用的配置。恢复先生成草稿，保存后才重新采用。</p><div class="mp-recovery-list">'+confirmed.map(c=>'<section><strong>确认可用 · '+esc(versionText(id,c.n))+'</strong><small>确认时间 '+esc(c.label)+' · '+esc(c.uses.join('、'))+'</small>'+button('恢复此配置','restore-confirmed','data-id="'+c.id+'"')+'</section>').join('')+entries.slice().reverse().map(r=>'<section><strong>'+esc(r.time)+' · 采用前配置</strong><small>'+esc(t.versions.filter(v=>templateUses(r.before,id,v.n).length).map(v=>v.versionLabel).join('、')||'此前未采用此模版')+'</small>'+button('恢复到采用前','restore-template-record','data-id="'+r.id+'" data-template="'+id+'"')+'</section>').join('')+(!entries.length&&!confirmed.length?'<p>暂无采用记录；旧版本可在版本选择器中查看并采用。</p>':'')+'</div>','关闭');
 }
 function confirmTemplateUse(){
  const t=template(),v=version(t.id,versionN),uses=templateUses(saved,t.id,v.n);if(!uses.length){notice('该版本尚未被保存的渠道配置采用');return;}
  if(pending||dirty()||busy()){notice('先完成渠道保存及重启核对，再确认实际使用情况');return;}
  const baseline=JSON.stringify(saved);modal('确认这套配置可用','<p>'+esc(v.versionLabel)+'</p><p>确认范围：'+esc(uses.join('、'))+'。请在实际使用后确认；保存成功本身不代表模型配置可靠。</p>','确认可用',()=>{if(baseline!==JSON.stringify(saved)||pending||dirty()){notice('采用配置已变化，请重新确认');return;}const meta=updates.stamp(confirmedConfigurations);confirmedConfigurations.push({id:meta.revisionId,revisionId:meta.revisionId,shortId:meta.shortId,label:meta.versionLabel,templateId:t.id,n:v.n,uses,policy:clone(saved)});closeModal();render();notice('已保留此版本及采用配置为可恢复的确认记录');},{confirmKeepsOpen:true});
 }
 function generateDialog(){if(assistantTask==='fill'&&!buffer())templateEdit();assistantOpen=true;renderAssistant();}
 function conversationDialog(){generateDialog();}
 function assistantHTML(){
  const messages=conversation.filter(m=>m.templateId===templateId),running=generating&&generationOwner===templateId;
  const model=agent.modelId||'选择执行模型',summary=agentSummary();
  const header='<header><div class="mp-assistant-heading"><strong>Agent 助手</strong><span class="mp-agent-scope">本机共用</span><button class="mp-assistant-model" data-mp="agent-settings" title="'+esc(summary)+'" aria-label="配置执行模型：'+esc(summary)+'"><span>'+esc(model)+'</span>'+icon('collapse')+'</button></div><div class="mp-assistant-window-actions">'+iconButton(assistantMaximized?'还原 Agent 对话':'放大 Agent 对话','assistant-size',assistantMaximized?'restore':'maximize','id="mpAssistantSizeToggle" aria-pressed="'+assistantMaximized+'"')+iconButton('收起 Agent 对话','assistant-close','collapse')+'</div></header>';
  const log='<div class="mp-conversation-log" role="log" aria-live="polite" aria-relevant="additions text" aria-label="Agent 对话">'+(messages.length?messages.map(m=>'<div class="mp-message '+m.role+'"><span class="mp-message-role">'+(m.role==='user'?'你':'Agent')+'</span>'+esc(m.text)+'</div>').join(''):'<div class="mp-empty">描述模型族、匹配规则和配置要求，发送后可继续对话微调。</div>')+'</div>';
  const body='<div id="mpAssistantConversation" class="mp-assistant-body">'+log+(running?'<div class="mp-row"><span role="status">正在生成演示结果…</span>'+button('停止生成','stop-generation')+'</div>':'')+(proposal?.id===templateId?'<div class="mp-message"><strong>结果待比较</strong><p>'+esc(proposal.summary)+'</p><div class="mp-row">'+button('查看并接受','accept-proposal')+button('保留当前','reject-proposal')+'</div></div>':'')+'</div>';
  const task='<div class="mp-assistant-task"><label>任务<select id="mpAssistantTask" class="mp-select" data-assistant-task>'+opts([['fill','自动填充'],['update','更新模型资料']],assistantTask)+'</select></label>'+(assistantTask==='update'?'<label>范围<select class="mp-select" data-update-scope id="mpUpdateScope">'+opts([['current','当前模版'],['selected','已勾选模版'],['all','全部模版']],updateScope)+'</select></label>':'')+'</div>';
  const context='<div class="mp-assistant-context"><label class="mp-label mp-assistant-sources">资料<select class="mp-select" id="mpGenerationSources" data-assistant-sources aria-label="参考资料" title="当前资料包含当前配置与提供的资料">'+opts([['provided','当前资料'],['search','公开检索 · 待接入']],generationSources)+'</select></label><label class="mp-label" title="锁定人工上下文窗口，生成建议须保留人工值"><input id="mpAssistantLock" data-lock-field type="checkbox" '+(assistantLock?'checked':'')+'>锁定上下文</label>'+iconButton('导出上下文','task-pack','export')+'</div>';
  const composer='<div class="mp-assistant-composer"><textarea id="mpGenerationTarget" data-assistant-input rows="2" aria-label="对话输入" placeholder="描述要求，或继续纠偏…">'+esc(assistantInputs[templateId]||'')+'</textarea>'+button(icon('send')+'<span>发送</span>','generate-run','aria-label="发送对话" '+(!agentReady()||generating?'disabled':''),true)+'</div>';
  const updateBody='<div class="mp-assistant-body"><p>检查新增型号和能力资料，逐项比较后保存新版本。</p><p>人工配置默认保留；已采用版本与恢复记录受保护。</p><p class="mp-muted" role="status">'+esc(updateSummary()||'尚未检查')+'</p>'+(updateRun?.status==='ready'?'<div class="mp-actions">'+button('查看更新差异','update-review')+'</div>':'')+(updateRun?.status==='saved'?'<p>新版本已保存，可采用后试用。</p>':'')+'</div>';
  const updateFooter='<footer><p class="mp-muted">执行：'+esc(summary)+'</p><div class="mp-actions">'+button(updateRun?.status==='running'?'正在检查…':'检查更新','update-check',updateRun?.status==='running'||generating?'disabled':'',true)+(updateRun?.status==='running'?button('停止检查','update-stop'):'')+'</div></footer>';
  return '<section id="mpAssistantPanel" class="mp-assistant-panel" role="dialog" aria-modal="false" aria-label="模版 Agent 助手">'+header+task+(assistantTask==='update'?updateBody+updateFooter:body+'<footer>'+context+composer+'</footer>')+'</section>';
 }

 function syncAssistantSize(){
  const node=$('#mpAssistantHost'),panel=$('#mpAssistantPanel'),control=$('#mpAssistantSizeToggle'),expanded=assistantOpen&&assistantMaximized&&!!panel;
  node?.classList.toggle('assistant-maximized',expanded);panel?.classList.toggle('is-maximized',expanded);
  if(control){const label=expanded?'还原 Agent 对话':'放大 Agent 对话';control.setAttribute('title',label);control.setAttribute('aria-label',label);control.setAttribute('aria-pressed',String(expanded));control.innerHTML=icon(expanded?'restore':'maximize');}
 }
 function renderAssistant(){const node=$('#mpAssistantHost');if(!node)return;
  const visible=tab==='templates'&&(templateDetail||assistantTask==='update');
  const active=document.activeElement,id=active?.id,focused=id&&node.querySelector('#'+id),start=active?.selectionStart,end=active?.selectionEnd,scroll=node.querySelector('#mpAssistantConversation')?.scrollTop||0;
  node.innerHTML=visible&&assistantOpen?assistantHTML():'';
  $('#mpTemplateAgentAction',host)?.setAttribute('aria-expanded',String(visible&&assistantOpen));
  syncAssistantSize();
  const body=node.querySelector('#mpAssistantConversation');if(body)body.scrollTop=scroll;if(focused){const field=node.querySelector('#'+id);field?.focus({preventScroll:true});if(start!=null&&field?.setSelectionRange)field.setSelectionRange(start,end??start);}
 }
 function refreshConversation(){renderAssistant();}
 function runGeneration(message,correction){
  if(generating||!agentReady())return;if(!buffer())templateEdit();
  const token=++generationToken,id=templateId,digest=editorDigest,baseContent=templateContent(),base=baseContent.rules,locked=assistantLock,executor=agentSummary(),execution=agentExecution(),sources=generationSources;
  generationOwner=id;conversation.push({templateId:id,role:'user',text:message});generating=true;assistantInputs[id]='';correctionDraft='';render();
  setTimeout(()=>{if(token!==generationToken)return;generating=false;
   const rules=clone(base),match=clone(baseContent.match),variants=clone(baseContent.variants||[]),suffix=message.match(/后缀\s*([a-zA-Z0-9_./-]+)/)?.[1];
   const effort=/高/.test(message)?'high':/低/.test(message)?'low':'medium';
   if(suffix){const item=variants.find(v=>v.suffix===suffix);if(item)item.rules={...item.rules,default:effort};else variants.push({id:'agent-variant-'+token,name:suffix,suffix,rules:{default:effort}});}
   else if(/默认|档/.test(message))rules.default=effort;
   const pattern=message.match(/匹配\s*([a-zA-Z0-9_.*?/-]+)/)?.[1];if(pattern)match.include=[{type:'glob',value:pattern}];
   if(/继承.*窗口|窗口.*继承/.test(message)&&!locked)delete rules.context;
   if(locked&&Object.hasOwn(base,'context'))rules.context=base.context;
   proposal={id,execution,baseDigest:digest,baseContent,rules,match,variants,locked,summary:(digest!==editorDigest||id!==templateId?'基准已变化，保留为待比较结果；':'')+'执行 '+executor+'（演示）。匹配 '+matchLines(match.include)+'；'+(suffix?'版本或后缀 '+suffix:'默认 '+(labels[rules.default]||'继承'))+'；来源 '+(sources==='search'?'检索未接入':'已有资料未核对')+'。'};
   conversation.push({templateId:id,role:'agent',text:executor+'：配置差异已生成，尚未入库或采用。'});render();
  },scenario==='late-generation'?1400:650);
 }
 function proposalReview(){if(!proposal||proposal.id!==templateId){notice('结果属于另一模版，请切回原目标比较');return;}if(!buffer())templateEdit();const p=clone(proposal),local=templateContent(),base=p.baseContent||local,proposed={...base,rules:p.rules,match:p.match||base.match},merged=clone(local),conflicts=[],changes=[];for(const section of ['rules','match'])for(const k of new Set([...Object.keys(base[section]),...Object.keys(proposed[section])])){const a=base[section][k],b=proposed[section][k],now=local[section][k];if(JSON.stringify(a)===JSON.stringify(b))continue;changes.push(section+'.'+k+'：'+JSON.stringify(now)+' → '+JSON.stringify(b));if(k==='context'&&p.locked!==false&&Object.hasOwn(local.rules,'context'))continue;if(JSON.stringify(now)!==JSON.stringify(a)&&JSON.stringify(now)!==JSON.stringify(b)){conflicts.push(section+'.'+k);continue;}if(b===undefined)delete merged[section][k];else merged[section][k]=clone(b);}if(p.variants){if(JSON.stringify(base.variants||[])!==JSON.stringify(p.variants)){changes.push('版本或后缀：'+JSON.stringify(local.variants||[])+' → '+JSON.stringify(p.variants));if(JSON.stringify(local.variants||[])!==JSON.stringify(base.variants||[])&&JSON.stringify(local.variants||[])!==JSON.stringify(p.variants))conflicts.push('版本或后缀（保留人工配置）');else if(p.variants.length)merged.variants=clone(p.variants);else delete merged.variants;}}
  if(p.locked!==false)for(const item of local.variants||[]){if(!Object.hasOwn(item.rules,'context'))continue;const next=merged.variants?.find(v=>v.id===item.id);if(!next){merged.variants=clone(local.variants);conflicts.push('版本或后缀删除涉及已锁定人工上下文（保留人工配置）');break;}if(next.rules.context!==item.rules.context){next.rules.context=item.rules.context;conflicts.push('版本或后缀 '+item.name+' 的人工上下文已锁定');}}
  modal('接受本轮差异',diffHTML(changes)+(conflicts.length?'<p class="mp-alert">人工修改冲突：'+esc(conflicts.join('、'))+'；默认保留当前字段。</p>':'')+'<p>只更新候选缓冲，保存前仍可人工修改。</p>','接受到候选',()=>{if(JSON.stringify(local)!==JSON.stringify(templateContent())){notice('候选再次变化，请重新比较');closeModal();return;}const error=templateError(merged);if(error){notice(error);return;}buffers[templateId]={...buffer(),...merged};editorDigest++;proposal=null;closeModal();render();},{confirmKeepsOpen:true}); }

 function externalFieldContract(){return {
  identity:{idPattern:"^[a-z][a-z0-9_-]{1,79}$",operations:"1–100 项；每批每个 ID 只出现一次",revision:"新版本号和完整revisionId均由软件分配；外部只回填基准版本和摘要，不提交新版本号",create:"使用全新 ID，baseDigest 留空字符串",revisionBaseline:"只能修改 targets 中的 ID，baseDigest 原样回填；库或人工缓冲变化须重新生成任务",sources:"可选字符串数组，记录实际资料与未知项"},
  content:{required:["name","description","match","rules"],optional:['variants'],variants:{items:{required:['id','name','suffix','rules'],additionalFields:false,id:'版本内唯一稳定字符串',name:'非空条目名称',suffix:'完整上游 ID 的字面后缀，1–256 字符；最长命中，重复拒绝',rules:'与族通用 rules 同字段，缺省继承族通用；合并后须合法'},empty:'省略或空数组只使用族通用'},additionalFields:false,name:"非空字符串，自由命名模型族",description:"字符串，可为空",match:{required:["include","exclude","caseSensitive","channels","priority"],additionalFields:false,include:"至少一条规则",exclude:"规则数组，可为空",rule:{required:["type","value"],additionalFields:false,type:["exact","prefix","glob"],value:"非空字符串，最多 256 字符"},caseSensitive:"布尔值，默认 false；同样作用于型号后缀",channelMode:{enum:["all","whitelist","blacklist"],legacy:"省略时，channels 空视为 all，非空视为 whitelist"},channels:"渠道 ID 数组；all 忽略列表，空白名单不匹配，空黑名单匹配全部",priority:"安全整数，较大优先"},
   rules:{additionalFields:false,inheritance:"以下字段均可省略；缺省表示继承，不能用 null 代替。提交完整预期内容，不提交局部补丁",protocol:{enum:protocols,encoding:"本原型使用显示值，由 protocolCatalog.generic 映射适配器 ID"},context:"正的安全整数，单位 tokens；目标已有锁定值必须原样保留",modalities:{type:"非空数组",items:["text","image","audio"]},efforts:{type:"数组",items:levels,empty:"隐藏可选档位，不等同强制默认、不传参数或 none"},default:{enum:levels,constraint:"独立默认值；同时设置 efforts 时须属于列表，不能与空列表同用"},fast:{enum:["unknown","supported","unsupported"],constraint:"仅为能力声明，不授予实际能力，不改变执行者的快速模式偏好"},force:{enum:levels,constraint:"设置 efforts 时须属于列表；不能与 omit=true 同用"},omit:"可选布尔值，true 表示不传推理参数",authority:{enum:["force"]}}}
 };}
 function externalMatchingContract(){return {variants:'先族通用再最长字面后缀型号；不累加多个型号；匹配原始上游 ID，别名和显示名称不参与',channelScope:'all 忽略列表；whitelist 空=无；blacklist 空=全部；兼容旧版缺失 mode',include:"多条包含规则任一命中即可",exclude:"排除优先于包含",wildcards:"glob 中 * 匹配任意长度，? 匹配一个字符，其余按字面处理",caseSensitivity:"默认不敏感，不改写模型 ID",participation:"仅已采用的精确版本参与运行；候选和主指针改变不自动采用",references:"显式单模型引用优先；否则渠道同 ID 引用替换全局，其余全局 ID 保留",priority:"最高整数优先；最高同分多个命中报告冲突，不按列表顺序选取",effects:"预览匹配不加入白名单、不启用模型；协议与能力采用到渠道后仍须校验"};}
 function taskPack(){externalAgentDialog();}
 function templateCompare(){
  const t=template(),currentContent=templateContent(),defaultN=t.main||t.versions[0]?.n;
  modal('与版本比较','<label class="mp-label">比较基准<select id="mpCompareVersion" class="mp-select">'+opts(t.versions.map(v=>[v.n,v.versionLabel+(v.n===t.main?' · 默认版本':'')]),defaultN)+'</select></label><div id="mpVersionComparison"></div><p class="mp-muted">只读比较；不改变编辑内容、默认版本或采用配置。</p>','关闭');
  const fill=()=>{const v=version(t.id,$('#mpCompareVersion').value||defaultN);$('#mpVersionComparison').innerHTML=diffHTML(['说明：'+(v?.description||'')+' → '+currentContent.description,'匹配：'+JSON.stringify(v?.match)+' → '+JSON.stringify(currentContent.match),'版本或后缀：'+JSON.stringify(v?.variants||[])+' → '+JSON.stringify(currentContent.variants||[]),...Object.keys({...v?.rules,...currentContent.rules}).filter(k=>JSON.stringify(v?.rules[k])!==JSON.stringify(currentContent.rules[k])).map(k=>k+'：'+ruleText(k,v?.rules[k])+' → '+ruleText(k,currentContent.rules[k]))]);};$('#mpCompareVersion').onchange=fill;fill();
 }

 function externalScopeLabel(task){return {current:'当前模版',selected:'勾选的模版',all:'全部模版',create:'仅新建模版'}[task.scope?.mode]||'兼容任务';}
 function externalStateLabel(task){return {waiting:'等待外部提交',ready:'校验通过，待提交',saved:'已保存新版本',reviewed:'检查完成，未修改',unchanged:'内容无变化',invalid:'结果需修正',conflict:'基准已变化',cancelled:'已取消'}[task.state]||'等待外部提交';}
 const externalScenes={
  docs:{label:'从资料制模',intent:'根据用户提供或实际查到的资料整理模版，记录来源和适用型号；无证据的能力保持继承。',existing:false},
  update:{label:'更新型号资料',intent:'核对目标模版的最新型号、上下文和推理能力资料；只修订有依据的差异，保留人工字段。',existing:true},
  batch:{label:'批量检查与修订',intent:'检查目标模版的匹配遗漏、重叠和继承问题，按模版逐项修订；保留人工锁并说明每项依据。',existing:true},
  private:{label:'私有模型配置',intent:'根据用户提供的私有接口资料定义型号、匹配和能力；不要求凭据，无法核实的内容标记未知。',existing:false},
  personal:{label:'按偏好调整',intent:'按照用户补充要求调整目标模版的匹配、默认推理或其他支持字段；未要求的字段保持，人工锁不变。',existing:true},
  audit:{label:'仅检查，不保存',intent:'只检查目标模版的匹配、继承和资料矛盾，给出有依据的问题与建议；不提交模版修改，不新增版本。',existing:true,readOnly:true}
 };
 function externalAgentDialog(resume=undefined){
  const use=resume===false?null:resume||((externalTask&&!externalTask.completed&&externalTask.targets.some(t=>t.id===templateId)&&templateDetail)?externalTask:null);
  const range=use?.scope?.mode||(templateSelection.size?'selected':templateDetail&&!templateScratch?'current':'create');
  const scene=use?.scene||((range==='create')?'docs':'update'),note=use?.userNote||'';
  modal('外部任务','<div class="mp-external-form"><div class="mp-external-options"><label class="mp-label">范围<select id="mpExternalScope" class="mp-select">'+opts([['current','当前模版'],['selected','勾选的模版'],['all','全部模版'],['create','仅新建模版']],range)+'</select></label><label class="mp-label">场景<select id="mpExternalScene" class="mp-select">'+opts(Object.entries(externalScenes).map(([id,s])=>[id,s.label]),scene)+'</select></label></div><label class="mp-label">补充要求（可选）<textarea id="mpExternalRequest" class="input" rows="3" placeholder="填写资料、型号或需要保留的偏好；场景规则已预置">'+esc(note)+'</textarea></label><label class="mp-label" id="mpExternalCreateOption"><input type="checkbox" id="mpExternalAllowCreate" '+(use?.scope?.mode!=='create'&&use?.scope?.allowCreate?'checked':'')+'>同时允许新增模版</label><label class="mp-label">完整任务内容<textarea id="mpExternalPrompt" class="mp-json mp-external-prompt" aria-label="可一次复制的提示词与结构资料" readonly></textarea></label><div id="mpExternalReceiptHost"></div><p id="mpExternalTaskError" class="mp-alert" role="alert" hidden></p></div>','复制完整任务',()=>copyExternalPrompt(),{cancelLabel:'关闭',confirmKeepsOpen:true,prototypeAnnotation:{title:'外部任务 · 场景、复制与校验说明',body:'范围、场景和补充要求直接生成一份完整内容，提示词与目标/字段契约一并复制；复制只登记提交基准，不写模版库。后台校验范围/人工锁/基准并分配版本，提交错误和保存回执仍属于业务结果。原型只有内存演示，没有真实CLI、MCP、写文件或权限沙箱。',blocks:[{id:'mpExternalSceneHint',label:'当前场景',text:''},{id:'mpCopyStatus',label:'复制说明',text:'选好范围和场景后，一次复制给外部 Agent 即可。'},{id:'mpExternalError',label:'当前范围校验',text:''}]}});
  $('#modal').classList.add('mp-external-modal');
  $('#mpExternalScope').value=range;$('#mpExternalScene').value=scene;
  if(use){externalTask=use;$('#mpExternalSceneHint').textContent=externalScenes[scene].intent;$('#mpExternalCreateOption').hidden=!!use.scope?.readOnly||range==='create';$('#mpExternalAllowCreate').disabled=!!use.scope?.readOnly||range==='create';$('#mpExternalPrompt').value=externalCombinedText(use);$('#modalBody').dataset.mpExternalTask=use.taskId;$('#mpExternalError').textContent='已按原提交基准恢复；更改范围或要求后会重新准备内容。';refreshExternalTaskView(use);window.prototypeAnnotations?.refresh();}else updateExternalPreview();
 }
 function buildExternalPreview(){
  const range=$('#mpExternalScope').value,scene=$('#mpExternalScene').value,definition=externalScenes[scene];
  if(!definition)throw Error('请选择任务场景');
  const targets=range==='all'?templates:range==='selected'?templates.filter(t=>templateSelection.has(t.id)):range==='current'&&templateDetail?[template()]:[];
  if(range==='current'&&(!targets.length||templateScratch?.id===templateId))throw Error('先保存新模版，或选择仅新建模版；未保存编辑保留');
  if(range==='selected'&&!targets.length)throw Error('请先勾选模版');
  if(range==='create'&&definition.existing)throw Error('此场景需要已有模版，请选择当前、勾选或全部模版');
  const userNote=$('#mpExternalRequest').value.trim(),readOnly=!!definition.readOnly;
  const intent=definition.intent+(userNote?'\n用户补充要求：'+userNote:'');
  const snapshots=targets.map(t=>{const n=buffers[t.id]?.base||(templateDetail&&t.id===templateId?versionN:Number(templateListVersions[t.id])||t.main||t.versions[0]?.n),v=version(t.id,n),c=buffers[t.id]?clone(buffers[t.id]):templateVersionSnapshot(t,v);const content={name:c.name||t.name,description:c.description||'',match:clone(c.match),rules:clone(c.rules),...(c.variants?.length?{variants:clone(c.variants)}:{})};return {id:t.id,version:n,baseRevisionId:v.revisionId,baseVersionLabel:v.versionLabel,libraryDigest:contentDigest(t),bufferDigest:contentDigest(buffers[t.id]||null),nextRevision:nextVersionN(t),exists:true,baseDigest:contentDigest(content),content};});
  const scope={mode:range,readOnly,allowCreate:!readOnly&&(range==='create'||$('#mpExternalAllowCreate').checked),allowRevision:!readOnly&&range!=='create',targetIds:snapshots.map(t=>t.id)};
  const previewKey=contentDigest({scene,userNote,scope,targets:snapshots});
  if(externalTask?.previewKey===previewKey)return externalTask;
  const channelIds=new Set(snapshots.flatMap(t=>t.content.match.channelMode==='all'?[]:t.content.match.channels||[]));
  return {kind:'template-task-batch',schema:'prototype-task/1',contractVersion:externalSubmissionContract,appVersion:'0.2.0',taskId:'external-'+Date.now()+'-'+(++editorDigest),scene,userNote,intent,previewKey,scope,targets:snapshots,fieldContract:externalFieldContract(),matchingContract:externalMatchingContract(),protocolCatalog:{generic:adapters,specialBinding:'专用适配器须由已实现的连接能力确认，不能自行伪造绑定'},context:{lockedFields:['rules.context','variants[*].rules.context'],channels:draft.providers.filter(p=>channelIds.has(p.id)).map(p=>({id:p.id,name:p.name}))},state:'waiting',completed:false,cancelled:false,copied:false,lastSubmission:null,receipt:null,batch:null};
 }
 function updateExternalPreview(){
  const field=$('#mpExternalPrompt'),error=$('#mpExternalError');if(!field||!$('#mpExternalScene'))return false;
  const definition=externalScenes[$('#mpExternalScene').value],readOnly=!!definition?.readOnly,onlyCreate=$('#mpExternalScope').value==='create';
  $('#mpCopyStatus').textContent='选好范围和场景后，一次复制给外部 Agent 即可。';
  $('#mpExternalSceneHint').textContent=definition?.intent||'';
  $('#mpExternalCreateOption').hidden=readOnly||onlyCreate;$('#mpExternalAllowCreate').disabled=readOnly||onlyCreate;
  try{externalTask=buildExternalPreview();externalBatch=externalTask.batch||null;field.value=externalCombinedText(externalTask);error.textContent='当前范围与场景可准备任务。';$('#modalConfirm').disabled=false;$('#modalConfirm').removeAttribute('title');$('#mpExternalScope').removeAttribute('aria-invalid');$('#mpExternalScene').removeAttribute('aria-invalid');window.prototypeAnnotations?.refresh();$('#modalBody').dataset.mpExternalTask=externalTask.taskId;refreshExternalTaskView(externalTask);return true;}
  catch(e){field.value='';error.textContent=e.message;$('#modalConfirm').disabled=true;$('#modalConfirm').title=e.message;$('#mpExternalScope').setAttribute('aria-invalid','true');$('#mpExternalScene').setAttribute('aria-invalid','true');window.prototypeAnnotations?.refresh();delete $('#modalBody').dataset.mpExternalTask;$('#mpExternalReceiptHost').innerHTML='';return false;}
 }
 function contentDigest(v){let h=2166136261;for(const c of (JSON.stringify(v)??"undefined"))h=Math.imul(h^c.charCodeAt(0),16777619);return 'mock-fnv1a-'+(h>>>0).toString(16);}
 function externalTaskPack(task=externalTask){
  if(!task)return null;return clone({kind:'template-task',taskId:task.taskId,scene:{id:task.scene||'docs',label:externalScenes[task.scene]?.label||'按要求制模'},versions:{app:task.appVersion||'0.2.0',task:'prototype-task/1',content:task.contractVersion||externalSubmissionContract},intent:task.intent,scope:clone(task.scope),targets:task.targets.map(t=>({id:t.id,baseRevisionId:t.baseRevisionId,baseVersionLabel:t.baseVersionLabel,baseDigest:t.baseDigest,content:clone(t.content)})),context:clone(task.context),contract:{fields:task.fieldContract,matching:task.matchingContract,protocolCatalog:task.protocolCatalog,...(task.scope?.readOnly?{report:{kind:'template-task-report',findings:'数组，可为空；每项仅含templateId、severity、message及可选sources',templateId:'必须属于本次目标',severity:['info','warning','error'],message:'非空字符串，最多4000字符',sources:'可选字符串数组；区分实际来源和未确认'}}:{})},operations:{read:'读取任务目标及只读基准',contract:'读取本任务契约',validate:'预检完整内容并返回差异；不写入',submit:task.scope?.readOnly?'不允许提交模版修改；仅返回检查报告':'校验基准后新增版本，由管理器分配身份',receipt:'按任务及提交ID查询真实结果'},submission:task.scope?.readOnly?{kind:'template-task-report',contractVersion:task.contractVersion,taskId:task.taskId,submissionId:task.taskId+'-submission-1',findings:[]}: {kind:'template-task-submission',contractVersion:task.contractVersion||externalSubmissionContract,taskId:task.taskId,submissionId:task.taskId+'-submission-1',operations:[task.targets.length?{action:'revision',id:task.targets[0].id,baseRevisionId:task.targets[0].baseRevisionId,baseDigest:task.targets[0].baseDigest,content:clone(task.targets[0].content),sources:[]}:{action:'create',id:'new-template',baseDigest:'',content:{name:'新建模版示例',description:'按任务替换示例；能力保持继承',match:{...defaultMatch(),include:[{type:'prefix',value:'company-'}]},rules:{}},sources:[]}]},boundary:{submissionExample:'只展示合法结构；修订示例为原内容，原样提交不产生新版本。请按任务修改内容并记录实际来源。重试同一内容保留submissionId。',entryAvailable:false,mode:'prototype-only',write:'仅经统一提交服务新增版本；不直接编辑库、默认指针、渠道策略、运行配置或凭据',versionOwner:'软件分配新版本；外部不提供新版本号',unknown:'无依据保持继承，不能以发现型号推断能力'}});
 }
 function externalText(task=externalTask){return '完成制模任务 '+task.taskId+'。场景：'+(externalScenes[task.scene]?.label||'按要求制模')+'。要求：'+task.intent+'\n先读取随附任务资料中的目标、基准和字段契约，按需核对实际资料；保留人工锁，未知保持继承。'+(task.scope?.readOnly?'本场景仅检查，按约定返回检查报告；不新增版本、不提交模版修改。':'通过任务的校验与提交操作新增版本，再按同一提交ID核对回执；基准变化先报告冲突。')+'新版本由软件分配，不直接编辑配置文件或改变默认/采用关系。\n原型资料用于评审；真实任务入口及提交能力待开发。';}
 function externalReceiptHTML(task){
  const receipt=task.receipt;if(!receipt)return '';
  return '<section class="mp-external-receipt"><h3>提交回执</h3><p>'+esc(receipt.status==='reviewed'?'检查完成，未修改模版':receipt.status==='unchanged'?'核对完成，内容无变化':receipt.status==='saved'?'新版本已保存':'未保存')+'</p>'+(receipt.findings?'<ul class="mp-external-findings">'+receipt.findings.map(item=>'<li><strong>'+esc(item.templateId||'检查结果')+'</strong> '+esc(item.message)+'</li>').join('')+(receipt.findings.length?'':'<li>没有报告需处理的问题；不代表已验证实际模型能力。</li>')+'</ul>':'')+receipt.versions.map(v=>'<div class="mp-external-version"><strong>'+esc(v.name)+'</strong><span>'+esc(v.versionLabel)+'</span><small>'+esc(v.summary)+'</small>'+button('编辑','external-open-version','data-template="'+esc(v.templateId)+'" data-version="'+v.n+'"')+'</div>').join('')+'<p class="mp-muted">默认和采用关系保持。'+esc(receipt.submissionId||'兼容提交')+'</p></section>';
 }
 function externalContext(){externalAgentDialog(externalTask);}
 function refreshExternalTaskView(task){
  if($('#modalBody').dataset.mpExternalTask!==task.taskId)return;const label=$('#mpExternalTaskState'),receipt=$('#mpExternalReceiptHost'),error=$('#mpExternalTaskError');if(label){label.textContent=externalStateLabel(task);label.className='mp-template-state '+(['invalid','conflict'].includes(task.state)?'warning':task.completed?'info':'neutral');}if(receipt)receipt.innerHTML=externalReceiptHTML(task);if(error){error.textContent=task.error||'';error.hidden=!task.error;}
 }
 function openExternalTask(id){const task=externalTasks.get(id);if(!task)return false;externalTask=task;externalBatch=task.batch||null;if(document.documentElement.dataset.route!=='models')setRoute('models');externalContext();return true;}
 function cancelExternalTask(){if(!externalTask||externalTask.completed)return;externalTask.cancelled=true;externalTask.completed=true;externalTask.state='cancelled';externalTask.batch=null;externalBatch=null;syncExternalNotification(externalTask);externalContext();}
 function externalCombinedText(task=externalTask){return externalText(task)+'\n\n以下是本次目标、字段契约和提交约定（与任务说明一并提供）：\n'+JSON.stringify(externalTaskPack(task),null,2);}
 async function copyExternalTask(){return copyExternalPrompt();}
 async function copyExternalPrompt(){
  if(!$('#mpExternalPrompt')?.value||!externalTask||$('#modalConfirm').disabled)return;
  const field=$('#mpExternalPrompt'),task=externalTask,text=externalCombinedText(task);field.value=text;
  const registered=externalTasks.get(task.taskId);if(!registered){task.copied=true;externalTasks.set(task.taskId,task);}
  try{if(!navigator.clipboard?.writeText)throw Error('unavailable');await navigator.clipboard.writeText(text);if($('#mpCopyStatus')&&$('#modalBody').dataset.mpExternalTask===task.taskId){$('#mpCopyStatus').textContent='完整任务已复制，直接交给外部 Agent 即可。';window.prototypeAnnotations?.refresh();}}
  catch{field.select();if($('#mpCopyStatus')&&$('#modalBody').dataset.mpExternalTask===task.taskId){$('#mpCopyStatus').textContent='完整任务已选中，请按 Cmd/Ctrl+C 一次复制。';window.prototypeAnnotations?.refresh();}}
 }
 function externalSample(){if(externalTask.scope?.readOnly)return {kind:'template-task-report',contractVersion:externalTask.contractVersion,taskId:externalTask.taskId,submissionId:externalTask.taskId+'-submission-1',findings:[]};const operations=externalTask.targets.map(t=>({action:'revision',id:t.id,baseRevisionId:t.baseRevisionId,baseDigest:t.baseDigest,content:{...clone(t.content),description:'外部 Agent 演示候选 · 能力未核对'},sources:['演示 fixture；未检索']}));if(!operations.length&&externalTask.scope?.allowCreate)operations.push({action:'create',id:'private-coder-'+Date.now(),baseDigest:'',content:{name:'企业私有编码模型',description:'外部 Agent 新建示例',match:{...defaultMatch(),include:[{type:'prefix',value:'private-coder-'}]},rules:{}},sources:['演示 fixture；能力继承']});return {kind:'template-task-submission',contractVersion:externalTask.contractVersion,taskId:externalTask.taskId,submissionId:externalTask.taskId+'-submission-1',operations};}
 function externalDemo(){externalSubmitDemo('normal');}
 function externalPreflightDemo(){if(!externalTask||externalTask.completed)return;receiveExternalSubmission(externalSample(),false);}
 function externalSubmitDemo(scene){
  if(!externalTask){externalAgentDialog();return;}const sample=externalTask.lastSubmission?clone(externalTask.lastSubmission):externalSample();
  if(scene==='invalid'&&sample.operations[0]){sample.submissionId+='-invalid';sample.operations[0].content.rules.fakeField=true;}
  receiveExternalSubmission(sample,true);
 }
 function syncExternalNotification(task){window.modelExternalTaskNotificationBridge?.({taskId:task.taskId,state:task.state,submissionId:task.lastSubmission?.submissionId||'',versions:task.receipt?.versions.map(v=>({templateId:v.templateId,revisionId:v.revisionId}))||[]});}
 function receiveExternalSubmission(data,commit=true){
  const task=externalTasks.get(data?.taskId);if(!task)return {ok:false,state:'invalid',message:'任务不存在，请从软件创建任务'};
  const digest=contentDigest(data);
  if(task.lastSubmission?.submissionId===data.submissionId&&task.submissionDigest===digest&&task.receipt)return {ok:true,replayed:true,receipt:clone(task.receipt)};
  if(task.cancelled)return {ok:false,state:'cancelled',message:'任务已取消，迟到提交不保存'};
  if(task.completed)return {ok:false,state:'invalid',message:'任务已完成；修改内容请创建新任务'};
  const previousTask=externalTask,previousBatch=externalBatch;
  try{
   if(task.lastSubmission?.submissionId===data.submissionId&&task.submissionDigest&&task.submissionDigest!==digest)throw Error('同一提交ID的内容已变化，请使用新的提交ID');
   externalTask=task;if(task.scope?.readOnly){const report=validateExternalReport(task,data);task.lastSubmission=clone(data);task.submissionDigest=digest;if(commit){task.receipt=report;task.state='reviewed';task.completed=true;task.error='';}else{task.state='ready';task.error='';}syncExternalNotification(task);return {ok:true,state:task.state,receipt:clone(task.receipt)};}externalBatch=validateExternalBatch(data);task.lastSubmission=clone(data);task.submissionDigest=digest;task.batch=externalBatch;task.state='ready';task.error='';
   if(commit){const result=commitExternalBatch(task,externalBatch);if(!result.ok)throw Error(result.message);}
   syncExternalNotification(task);return {ok:true,state:task.state,receipt:clone(task.receipt)};
  }catch(error){task.batch=null;task.state=/基准|已变化|目标/.test(error.message)?'conflict':'invalid';task.error=error.message;syncExternalNotification(task);return {ok:false,state:task.state,message:error.message};}
  finally{externalTask=previousTask;externalBatch=previousTask===task?task.batch:previousBatch;refreshExternalTaskView(task);}
 }
 function importedExternalTask(input){
  if(!input||input.kind!=='template-task-batch'||typeof input.taskId!=='string'||!input.taskId||input.taskId.length>200||!Array.isArray(input.targets)||input.targets.length>100)throw Error('原任务格式无效，请粘贴生成时的完整任务包');
  const ids=new Set();for(const target of input.targets){
   if(!target||typeof target.id!=='string'||ids.has(target.id)||!Number.isSafeInteger(target.version))throw Error('原任务的目标身份无效或重复');ids.add(target.id);
   const t=templates.find(t=>t.id===target.id),v=version(target.id,target.version);if(!t||!v||target.libraryDigest!==contentDigest(t)||target.bufferDigest!==contentDigest(buffers[target.id]||null))throw Error('原任务的版本或人工编辑基准已变化，请重新生成任务');
   const error=templateError(target.content);if(error||target.baseDigest!==contentDigest(target.content))throw Error('原任务内容或基准校验失败');
  }
  return {...clone(input),scope:input.scope||{mode:'create',allowCreate:!input.targets.length,allowRevision:!!input.targets.length},state:'waiting',completed:false};
 }
 function externalResultDialog(sample=null){
  const previous=externalTask;
  const association=previous?'<textarea id="mpExternalTaskJSON" hidden aria-label="兼容任务关联"></textarea>':'<details class="mp-details" '+(!previous?'open':'')+'><summary>'+(!previous?'关联原任务':'更换关联任务（可选）')+'</summary><p>粘贴生成时复制的任务包；校验版本和人工修改基准后才接受结果。</p><textarea id="mpExternalTaskJSON" class="mp-json" aria-label="关联外部任务包" placeholder="粘贴原任务 JSON"></textarea></details>';
  modal('外部任务 · 兼容回传','<p>'+esc(previous?'已关联当前任务。':'尚未关联原任务。')+'结果先校验并比较，确认后只新增候选版本。</p>'+association+'<label>外部结果<textarea id="mpExternalJSON" class="mp-json" aria-label="外部 Agent 批量结果">'+esc(sample?JSON.stringify(sample,null,2):previous?JSON.stringify({kind:'template-batch-result',taskId:previous.taskId,operations:[]},null,2):'')+'</textarea></label><div id="mpExternalResultError" class="mp-alert" role="alert"></div>','校验并查看差异',()=>{
   try{
    const supplied=$('#mpExternalTaskJSON').value.trim(),candidate=supplied?importedExternalTask(JSON.parse(supplied)):previous;
    if(!candidate)throw Error('请关联原任务包；不会转去生成任务或覆盖当前版本');
    externalTask=candidate;const data=JSON.parse($('#mpExternalJSON').value);externalBatch=validateExternalBatch(data);externalTask.batch=externalBatch;externalTask.state='ready';externalTask.error='';externalTasks.set(candidate.taskId,candidate);closeModal();externalBatchPreview();
   }catch(e){externalTask=previous;$('#mpExternalResultError').textContent=e.message;}
  },{confirmKeepsOpen:true});
 }

 function validateExternalReport(task,data){
  if(data.kind!=='template-task-report'||data.contractVersion!==task.contractVersion||typeof data.submissionId!=='string'||!data.submissionId||data.submissionId.length>200||Object.keys(data).some(k=>!['kind','contractVersion','taskId','submissionId','findings'].includes(k)))throw Error('仅检查场景只接受检查报告，不允许模版写入');
  if(!Array.isArray(data.findings)||data.findings.length>100)throw Error('检查报告格式无效');
  for(const item of data.findings)if(!item||typeof item!=='object'||Array.isArray(item)||Object.keys(item).some(k=>!['templateId','severity','message','sources'].includes(k))||!task.targets.some(t=>t.id===item.templateId)||!['info','warning','error'].includes(item.severity)||typeof item.message!=='string'||!item.message.trim()||item.message.length>4000||item.sources!==undefined&&(!Array.isArray(item.sources)||item.sources.some(x=>typeof x!=='string')))throw Error('检查项字段或目标范围无效');
  for(const target of task.targets){const t=templates.find(t=>t.id===target.id);if(target.libraryDigest!==contentDigest(t)||target.bufferDigest!==contentDigest(buffers[target.id]||null))throw Error('检查基准已变化，请重新准备任务');}
  return {taskId:task.taskId,submissionId:data.submissionId,status:'reviewed',versions:[],findings:clone(data.findings)};
 }
 function validateExternalBatch(data){
  const modern=data?.kind==='template-task-submission',task=externalTask;if(task?.scope?.readOnly)throw Error('仅检查场景不允许保存模版或新增版本');
  if(!data||!['template-task-submission','template-batch-result'].includes(data.kind)||data.taskId!==task?.taskId)throw Error('结果类型或 taskId 不匹配，请重新创建任务');
  if(task.cancelled)throw Error('任务已取消，迟到提交不保存');
  const outer=modern?['kind','contractVersion','taskId','submissionId','operations']:['kind','taskId','operations'];
  if(Object.keys(data).some(k=>!outer.includes(k)))throw Error('结果不得修改主指针或策略');
  if(modern&&(data.contractVersion!==task.contractVersion||typeof data.submissionId!=='string'||!data.submissionId||data.submissionId.length>200))throw Error('提交契约版本或提交ID无效');
  if(!Array.isArray(data.operations)||!data.operations.length||data.operations.length>100)throw Error('需要 1–100 个模版操作');
  for(const target of task.targets){const t=templates.find(t=>t.id===target.id);if(target.libraryDigest!==contentDigest(t)||target.bufferDigest!==contentDigest(buffers[target.id]||null))throw Error('任务基准已变化，请保留结果并重新创建任务');}
  const ids=new Set();
  return data.operations.map(op=>{
   const allowed=modern?['action','id','baseRevisionId','baseDigest','content','sources']:['action','id','revision','baseDigest','content','sources'];
   if(!op||typeof op!=='object'||Array.isArray(op)||Object.keys(op).some(k=>!allowed.includes(k)))throw Error('操作字段无效；新版本身份由软件分配');
   if(!/^[a-z][a-z0-9_-]{1,79}$/.test(op.id)||ids.has(op.id))throw Error('模版 ID 无效或一批重复');ids.add(op.id);
   const t=templates.find(t=>t.id===op.id),target=task.targets.find(t=>t.id===op.id);
   if(!['create','revision'].includes(op.action))throw Error('只支持新增或新版本');
   if(op.action==='create'&&task.scope?.allowCreate===false||op.action==='revision'&&task.scope?.allowRevision===false)throw Error('操作超出任务授权范围');
   if(op.action==='create'&&(t||templateScratch?.id===op.id))throw Error('新增 ID 已存在');
   if(op.action==='revision'&&(!t||!target||op.baseDigest!==target.baseDigest||modern&&op.baseRevisionId!==target.baseRevisionId))throw Error('新版本目标或基准不匹配');
   if(op.action==='create'&&(op.baseDigest||op.baseRevisionId))throw Error('新增模版不携旧版本基准');
   const revision=t?nextVersionN(t):1;
   if(!modern&&(!Number.isSafeInteger(op.revision)||op.revision!==revision))throw Error('版本号须为软件分配的未使用递增正整数');
   const c=op.content;
   if(!c||typeof c!=='object'||Array.isArray(c)||Object.keys(c).some(k=>!['name','description','match','rules','variants'].includes(k)))throw Error('模版内容字段无效');
   if(typeof c.name!=='string'||typeof c.description!=='string'||!c.match||!Array.isArray(c.match.include)||!Array.isArray(c.match.exclude)||!Array.isArray(c.match.channels)||typeof c.match.caseSensitive!=='boolean'||c.match.channels.some(id=>!draft.providers.some(p=>p.id===id)))throw Error('名称、说明或匹配规则结构无效');
   if(Object.keys(c.match).some(k=>!['include','exclude','channels','channelMode','caseSensitive','priority'].includes(k)))throw Error('匹配字段无效');
   if(!c.rules||typeof c.rules!=='object'||Array.isArray(c.rules)||Object.keys(c.rules).some(k=>!['protocol','context','modalities','efforts','default','fast','force','omit','authority'].includes(k)))throw Error('模型配置字段无效');
   if(c.rules.fast&&!['unknown','supported','unsupported'].includes(c.rules.fast)||c.rules.modalities&&(!Array.isArray(c.rules.modalities)||!c.rules.modalities.length||c.rules.modalities.some(x=>!['text','image','audio'].includes(x)))||c.rules.force&&!levels.includes(c.rules.force))throw Error('能力或推理配置无效');
   for(const v of target?.content.variants||[])if(Object.hasOwn(v.rules,'context')&&c.variants?.find(x=>x.id===v.id)?.rules.context!==v.rules.context)throw Error('锁定的版本或后缀人工上下文不可修改或删除');
   if(target&&Object.hasOwn(target.content.rules,'context')&&c.rules.context!==target.content.rules.context)throw Error('锁定的人工上下文不可修改，请保留原值');
   const error=templateError(c);if(error)throw Error(error);
   if(op.sources!==undefined&&(!Array.isArray(op.sources)||op.sources.some(x=>typeof x!=='string')))throw Error('来源格式无效');
   return {action:op.action,id:op.id,revision,baseDigest:op.baseDigest,content:clone(c),sources:clone(op.sources||[]),libraryDigest:JSON.stringify(t||null),bufferDigest:JSON.stringify(buffers[op.id]||null)};
  });
 }
 function externalPlans(task,batch){return batch.map(op=>{const t=templates.find(t=>t.id===op.id),target=task.targets.find(x=>x.id===op.id),base=t&&target?version(op.id,target.version):null,before=base?templateVersionSnapshot(t,base):null;return {op,t,target,before,summaryFacts:summaryEngine.summarize(before,op.content,{kind:op.action==='create'?'create':'revision',templateId:op.id,revisionId:base?.revisionId||null})};});}
 function commitExternalBatch(task,batch){
  if(task.cancelled)return {ok:false,message:'任务已取消'};if(task.scope?.readOnly)return {ok:false,message:'仅检查场景不允许保存模版或新增版本'};
  if(task.receipt)return {ok:true,replayed:true,receipt:clone(task.receipt)};
  if(!batch?.length)return {ok:false,message:'请先校验提交内容'};
  for(const target of task.targets){const t=templates.find(t=>t.id===target.id);if(target.libraryDigest!==contentDigest(t)||target.bufferDigest!==contentDigest(buffers[target.id]||null))return {ok:false,message:'模版库或人工缓冲已变化，请重新创建任务和比较'};}
  for(const op of batch){const t=templates.find(t=>t.id===op.id);if(op.action==='create'&&templateScratch?.id===op.id)return {ok:false,message:'新增目标已被未保存编辑占用，请重新核对'};if(op.libraryDigest!==JSON.stringify(t||null)||op.bufferDigest!==JSON.stringify(buffers[op.id]||null)||op.revision!==(t?nextVersionN(t):1))return {ok:false,message:'模版库或人工缓冲已变化，请重新创建任务和比较'};const error=templateError(op.content);if(error)return {ok:false,message:error};}
  const plans=externalPlans(task,batch),changed=plans.filter(p=>p.summaryFacts.changed),allocated=templates.flatMap(t=>t.versions),records=[];
  // 全批准备完毕后再改内存库；真实锁/原子落盘由正式服务实现，原型不写文件。
  for(const {op,summaryFacts} of changed){const meta=updates.stamp(allocated);allocated.push(meta);records.push({op,record:{...meta,nameAtSave:op.content.name.trim(),origin:'外部 Agent',summaryFacts,summary:summaryFacts.text,basedOn:summaryFacts.basis.revisionId,n:op.revision,status:'外部候选',date:'刚刚',description:op.content.description,match:clone(op.content.match),rules:clone(op.content.rules),...(op.content.variants?.length?{variants:clone(op.content.variants)}:{}),source:op.sources.join('；')||'外部来源未提供'}});}
  const receipt={taskId:task.taskId,submissionId:task.lastSubmission?.submissionId||'legacy-'+contentDigest(batch),status:records.length?'saved':'unchanged',versions:records.map(({op,record})=>({templateId:op.id,name:op.content.name,n:record.n,revisionId:record.revisionId,versionLabel:record.versionLabel,summary:record.summary,contentDigest:contentDigest(templateVersionSnapshot({name:op.content.name},record))}))};
  for(const {op,record} of records){let t=templates.find(t=>t.id===op.id);if(!t){t={id:op.id,name:op.content.name.trim(),originType:'user',createdVia:'external-task',main:0,nextRevision:1,versions:[]};templates.push(t);}const keepBuffer=templateEditorChanged(op.id);t.name=op.content.name.trim();t.versions.push(record);t.nextRevision=record.n+1;templateListVersions[t.id]=record.n;if(!keepBuffer)delete buffers[t.id];}
  task.receipt=receipt;task.completed=true;task.state=receipt.status;task.batch=null;task.error='';if(externalTask===task)externalBatch=null;externalTasks.set(task.taskId,task);editorDigest++;render();syncExternalNotification(task);return {ok:true,receipt:clone(receipt)};
 }
 function externalBatchPreview(){
  const task=externalTask,batch=externalBatch;if(!task||!batch)return;
  const plans=externalPlans(task,batch),changed=plans.filter(p=>p.summaryFacts.changed);
  const body=plans.map(({op,target,before,summaryFacts})=>'<details class="mp-details" open><summary>'+esc(op.content.name)+' · '+(op.action==='create'?'新增模版':summaryFacts.changed?'新增版本':'内容无变化')+' · 基于 '+esc(before?versionText(op.id,target.version):'新模版')+'</summary>'+diffHTML([summaryFacts.fullText,'ID：'+op.id,'名称：'+(before?.name||'未创建')+' → '+op.content.name,'说明：'+(before?.description||'无')+' → '+op.content.description,'匹配：'+JSON.stringify(before?.match||null)+' → '+JSON.stringify(op.content.match),'配置：'+JSON.stringify(before?.rules||{})+' → '+JSON.stringify(op.content.rules),'版本或后缀：'+JSON.stringify(before?.variants||[])+' → '+JSON.stringify(op.content.variants||[]),'来源：'+op.sources.join('；')])+'</details>').join('');
  modal('外部任务 · 差异核对',body+'<p>'+changed.length+' 项有实际变化。保存只新增版本；无变化项不新增，默认与采用关系保持。</p><div id="mpExternalCommitError" class="mp-alert" role="alert"></div>',changed.length?'保存新版本':'完成核对',()=>{
   const result=commitExternalBatch(task,batch);if(!result.ok){task.state='conflict';task.error=result.message;$('#mpExternalCommitError').textContent=result.message;syncExternalNotification(task);return;}
   closeModal();if(changed.length){const first=changed[0].op;tab='templates';templateId=first.id;versionN=buffers[first.id]?.base??first.revision;templateDetail=true;}render();externalContext();
  },{confirmKeepsOpen:true});
 }
 function jsonPreview(title,data,note=''){modal(title,'<p>'+esc(note)+'</p><textarea class="mp-json" id="mpJSON" aria-label="JSON 内容">'+esc(JSON.stringify(data,null,2))+'</textarea><p class="mp-muted">原型用预览和复制模拟文件导出，不写入任何运行配置。</p>','选中内容',()=>{$('#mpJSON').select();},{confirmKeepsOpen:true});}
 function syncEntry(action){
  const ids=tab==='templates'?(templateSelection.size?[...templateSelection]:templateDetail?[templateId]:templates.map(t=>t.id)).map(id=>'template:'+id):(scope.size?[...scope]:channelDetail?[current]:saved.providers.filter(p=>p.id!=='native').map(p=>p.id)).map(id=>'channel:'+id);
  closeModal();window.syncPrototype?.open({tab:'file',scope:ids,action});
 }
 function exportDialog(){syncEntry('export');}
 function importDialog(){syncEntry('import');}
 function mergePolicy(base,part){const next=clone(base);function reorder(list,order,key){if(!Array.isArray(order))return;const wanted=order.map(id=>list.find(x=>x[key]===id)).filter(Boolean),ids=new Set(wanted.map(x=>x[key]));let i=0;for(let n=0;n<list.length;n++)if(ids.has(list[n][key]))list[n]=wanted[i++];}for(const key of ['fastMode','fastRows'])if(Object.hasOwn(part,key))next[key]=clone(part[key]);if(part.globalRefs)next.globalRefs={...next.globalRefs,...canonicalKeys(part.globalRefs)};if(part.providers)part.providers.forEach(p=>{const target=next.providers.find(x=>x.id===p.id);if(!target||target.id==='native')return;for(const key of ['enabled','mode','defaults','fastEnabled','protocol'])if(Object.hasOwn(p,key))target[key]=clone(p[key]);for(const key of ['refs','familyOverrides'])if(Object.hasOwn(p,key))target[key]={...target[key],...canonicalKeys(p[key])};if(p.nodes)target.nodes=p.nodes.map(n=>({id:n.id,name:n.name,url:target.nodes?.find(x=>x.id===n.id)?.url||''}));if(Object.hasOwn(p,'activeNodeId'))target.activeNodeId=p.activeNodeId;if(p.models)p.models.forEach(m=>{const targetModel=target.models.find(x=>x.id===m.id);if(targetModel)for(const key of ['selected','overrides','templateRef','naming'])if(Object.hasOwn(m,key))targetModel[key]=clone(m[key]);});reorder(target.models,p.modelOrder,'id');});reorder(next.providers,part.providerOrder,'id');return next;}
 function incomingDialog(){if(!incoming)return;const proposed=mergePolicy(saved,incoming.policy),overlap=dirty();modal('待审候选 · '+incoming.source,diffHTML(diff(saved,proposed))+'<p>来源 '+esc(incoming.kind)+'；基准 '+(incoming.baseDigest===baseDigest()?'一致':'过期 / 不可得，手动比较')+'。'+(overlap?'本机另有草稿，重叠字段需选择。':'当前没有未提交草稿。')+'</p><label class="mp-label">冲突处理 <select id="mpConflict" class="mp-select">'+opts([['local','保留本机草稿'],['incoming','采用候选涉及字段']],'local')+'</select></label><p>采用仅进入草稿；主指针保持。取消保留候选。</p>','采用到草稿',()=>{const choice=$('#mpConflict').value;if(!overlap||choice==='incoming'){const candidate=mergePolicy(draft,incoming.policy),error=candidate.providers.filter(p=>p.id!=='native').map(namingError).find(Boolean);if(error){notice(error);return;}}closeModal();if(!overlap||choice==='incoming')change(()=>draft=mergePolicy(draft,incoming.policy));else notice('保留本机草稿，候选仍待审');if(!overlap||choice==='incoming'){incoming=null;render();}},{confirmKeepsOpen:true});}
 function recoveryPreview(before,after,label,recordId,fullAvailable=true){
  if(busy()){notice('正在保存，完成后再恢复');return;}
  const baseline=JSON.stringify(draft),savedBaseline=JSON.stringify(saved),safe=updates.restore(draft,before,after),full=updates.restore(draft,before,saved);
  modal('恢复采用配置','<p>'+esc(label)+'</p><p>旧模版版本保留；恢复生成草稿，再经保存、重启核对。后续未保存编辑发生冲突时保留当前值。</p>'+(fullAvailable?'<label class="mp-label">恢复范围<select id="mpRestoreScope" class="mp-select"><option value="impact">撤回该次提交影响字段</option><option value="full">回到当时整套配置（比较后续修改）</option></select></label>':'<p>恢复的是当时确认的整套渠道配置；后续已保存变化会列入恢复差异。</p>')+diffHTML(diff(draft,safe.value))+(safe.conflicts.length?'<details class="mp-details"><summary>'+safe.conflicts.length+' 项后续编辑保留</summary><p>'+esc(safe.conflicts.join('；'))+'</p></details>':'')+(fullAvailable?'<details class="mp-details"><summary>整套恢复的额外差异</summary>'+diffHTML(diff(draft,full.value))+'</details>':'')+'<div id="mpRestoreError" class="mp-alert"></div>','生成恢复草稿',()=>{
   if(busy()||baseline!==JSON.stringify(draft)||savedBaseline!==JSON.stringify(saved)){const node=$('#mpRestoreError');if(node)node.textContent='采用配置或草稿已变化，请重新比较恢复。';return;}
   const next=fullAvailable&&$('#mpRestoreScope')?.value==='full'?full.value:safe.value;
   if(JSON.stringify(next)===JSON.stringify(draft)){notice('没有可恢复的差异；后续编辑已保留');return;}
   closeModal();change(()=>{draft=clone(next);});restoration={recordId,label};tab='channels';render();notice('恢复草稿已准备，保存后采用；旧版本仍可查看');
  },{confirmKeepsOpen:true});
 }
 function restore(id){const record=records.find(r=>r.id===Number(id));if(record?.status==='成功')recoveryPreview(record.before,record.policy,'采用记录 '+record.time,record.id);}
 function restoreConfirmed(id){const record=confirmedConfigurations.find(c=>c.id===id);if(record)recoveryPreview(record.policy,saved,'确认可用 '+versionText(record.templateId,record.n)+' · '+record.uses.join('、'),record.id,false);}

 function scene(name){templateHistoryView=null;templateListVersions={};nodeSpeedSerial++;nodeSpeedResults={};nodeSpeedRuns={};nodeAuthRevision={};nodeSpeedScene='normal';channelConnections={};channelViewId=null;channelSection=channelSections[0][0];modelSelection.clear();modelExpanded.clear();modelEdits={};if(channelWizard)closeModal();wizardToken++;channelWizard=null;scenario=name;updateSerial++;updateRun=null;restoration=null;operationId++;operation=null;generationToken++;generating=false;draft=clone(initial);saved=clone(initial);pending=false;client='目录已核对 · 客户端待验证';scope=new Set();current='newapi';tab='channels';channelDetail=false;templateDetail=false;search='';family='';onlySelected=false;incoming=null;proposal=null;
  if(name==='template-history'){const t=templates.find(t=>t.id==='gpt'),base=t.versions[0];while(t.versions.length<18){const n=nextVersionN(t);t.versions.push({...clone(base),...updates.stamp(templates.flatMap(t=>t.versions)),n,systemBaseline:false,systemCurrent:false,origin:'人工',nameAtSave:t.name,basedOn:base.revisionId,rules:{...clone(base.rules),context:64000+n*1000}});const saved=t.versions.at(-1);saved.summaryFacts=summaryEngine.summarize(templateVersionSnapshot(t,base),templateVersionSnapshot(t,saved),{kind:'revision',templateId:t.id,revisionId:base.revisionId});saved.summary=saved.summaryFacts.text;t.nextRevision=n+1;}tab='templates';templateDetail=false;templateId='gpt';templateListVersions.gpt=1;}
  if(name==='over-budget'){draft.providers[1].mode='all';}
  if(name==='pending-draft'){pending=true;draft.providers[0].models[1].selected=false;}
  if(name==='incoming'){draft.fastMode='inherit';incoming={kind:'policy',source:'WebDAV 演示下载',scope:'fields',baseDigest:baseDigest(),policy:{fastMode:'off'}};}
  if(['backup-fail','partial','timeout','drift'].includes(name))draft.providers[0].models[1].selected=false;
  if(name==='late-generation'){tab='templates';templateDetail=true;templateId='gpt';versionN=3;templateEdit();}
  if(name.startsWith('update-')){tab='templates';templateDetail=true;templateId='gpt';versionN=template().main;assistantTask='update';assistantOpen=true;}setRoute('models');render();notice('已切换演示场景：'+name);}
 document.addEventListener('click',e=>{const el=e.target.closest('[data-mp]');if(!el||el.disabled)return;const a=el.dataset.mp,id=el.dataset.id;
  if(busy()&&['apply-channel-connection','reset-channel-connection','batch','adopt','model-config','provider-config','new-channel','channel-connection','add-node','edit-node','use-node','delete-node','discard','save','clear-channels','enable-channels','disable-channels','delete-channels','delete-provider','copy-provider','toggle-model','expand-model','apply-model-row','reset-model-row','enable-models','disable-models','edit-models','reset-models','expand-all-models','collapse-all-models','clear-model-picks'].includes(a))return;
  if(a==='node-picker-open')openNodePicker(el.dataset.channel,$('#mpChannelNode-'+el.dataset.channel,host));
  else if(a==='node-picker-close')closeNodePicker(true);
  else if(a==='node-picker-choose')chooseNodeFromPicker(el.dataset.channel,id);
  else if(a==='tab'){tab=el.dataset.tab;render();}
  else if(a==='copy-provider')copyProvider(id);
  else if(a==='edit-provider'){closeModal();current=id;channelDetail=true;search='';family='';render();}
  else if(a==='channel-section')scrollChannelSection(id);
  else if(a==='toggle-channel-nav')toggleChannelNav();
  else if(a==='apply-channel-connection')channelConnection(true,true);
  else if(a==='reset-channel-connection'){delete channelConnections[current];render();}
  else if(a==='review-channel-connection'){const item=stagedChannelConnections()[0];if(item){current=item[0];tab='channels';channelDetail=true;setRoute('models');render();scrollChannelSection('connection');}}
  else if(a==='back-channels'){channelDetail=false;render();}
  else if(a==='back-templates')returnTemplates();
  else if(a==='selector-preview')selectorPreviewDialog();
  else if(a==='preview-channel-toggle')togglePreviewChannel(id);
  else if(a==='preview-model-toggle')togglePreviewModel(id);
    else if(a==='agent-pick-model')selectAgentModel();
  else if(a==='agent-settings'){const field=$('#mpGenerationTarget');if(field)assistantInputs[templateId]=field.value;agentSettings(assistantOpen?'conversation':'');}
  else if(a==='conversation')conversationDialog();
  else if(a==='open-template-editor'){const t=templates.find(t=>t.id===id);openTemplateEditor(id,!templateDetail&&t?listTemplateVersion(t)?.n:null);}
  else if(a==='copy-list-template')copyListTemplate(id);
  else if(a==='template-inspect')templateInspection(id);
  else if(a==='clear-channels'){scope.clear();render();}
  else if(a==='enable-channels'||a==='disable-channels')setProvidersEnabled(a==='enable-channels');
  else if(a==='delete-channels')deleteProviders();
  else if(a==='delete-provider')deleteProviders([id]);
  else if(a==='toggle-provider'){const p=draft.providers.find(p=>p.id===id);if(p&&p.id!=='native'){change(()=>p.enabled=!p.enabled);$$('[data-mp="toggle-provider"]',host).find(x=>x.dataset.id===id)?.focus();}}
  else if(a==='move-provider'){change(()=>move(draft.providers,id,Number(el.dataset.direction)));}
  else if(a==='move-model'){change(()=>move(provider().models,id,Number(el.dataset.direction),'key'));}
  else if(a==='clear-filters'){search='';family='';onlySelected=false;render();}
  else if(a==='expand-all-models'){if(provider().id!=='native'){visibleModels().forEach(m=>modelExpanded.add(m.key));render();}}
  else if(a==='collapse-all-models'){if(provider().id!=='native'){provider().models.forEach(m=>modelExpanded.delete(m.key));render();}}
  else if(a==='clear-model-picks'){provider().models.forEach(m=>modelSelection.delete(m.key));render();}
  else if(a==='expand-model'){const m=provider().models.find(m=>m.key===id);if(!m||provider().id==='native')return;modelExpanded=modelExpanded.has(id)?new Set():new Set([id]);render();$$('[data-model="'+id+'"] [data-mp="expand-model"]',host)[0]?.focus({preventScroll:true});}
  else if(a==='reset-model-row'){delete modelEdits[id];render();}
  else if(a==='apply-model-row'){const edit=modelEdits[id];if(!edit)return;const error=quickRuleError(edit.rules,provider().models.find(m=>m.key===id).overrides)||commitModelPatches(current,[{key:id,rules:edit.rules,baseline:edit.baseline}]);if(error){const node=$('[data-model-quick-error="'+id+'"]',host);if(node)node.textContent=error;else modelError(id,error);}}
  else if(a==='toggle-model'){const m=provider().models.find(m=>m.key===id);if(!m)return;const error=commitModelPatches(current,[{key:id,enabled:!modelActive(provider(),m)}]);if(error)modelError(id,error);$$('[data-mp="toggle-model"]',host).find(x=>x.dataset.id===id)?.focus({preventScroll:true});}
  else if(['enable-models','disable-models','edit-models','reset-models'].includes(a))bulkModelDialog({'enable-models':'enable','disable-models':'disable','edit-models':'edit','reset-models':'reset'}[a]);
  else if(a==='review-model-rows'){const [key]=stagedModelEdits()[0]||[];const p=draft.providers.find(p=>p.models.some(m=>m.key===key));if(p){tab='channels';current=p.id;channelDetail=true;search='';family='';onlySelected=false;modelExpanded.add(key);render();$('[data-model-quick="'+key+'"]',host)?.scrollIntoView({block:'center'});}}
  else if(a==='select-result'||a==='unselect-result')selectModels(visibleModels(),a==='select-result');
  else if(a==='new-channel')channelConnection();
  else if(a==='channel-connection')channelConnection(true);
  else if(a==='node-speed-one')startNodeSpeed(el.dataset.channel,id);
  else if(a==='node-speed-current'){const p=draft.providers.find(p=>p.id===el.dataset.channel);if(p?.activeNodeId)startNodeSpeed(p.id,p.activeNodeId);}
  else if(a==='node-speed-all')startNodeSpeed(el.dataset.channel);
  else if(a==='node-speed-stop')stopNodeSpeed(el.dataset.channel);
  else if(a==='node-speed-scene')setNodeSpeedScene(el.dataset.scene);
  else if(a==='add-node')editServiceNode();
  else if(a==='edit-node')editServiceNode(id);
  else if(a==='use-node')useServiceNode(id);
  else if(a==='delete-node')deleteServiceNode(id);
  else if(a==='channel-fetch')fetchChannelModels();
  else if(a==='channel-cancel-fetch'&&channelWizard){wizardToken++;channelWizard.status='idle';drawChannelWizard();}
  else if(a==='channel-back'&&channelWizard&&channelWizard.status!=='loading'){channelWizard.step=1;drawChannelWizard();}
  else if(a==='channel-select'||a==='channel-unselect')wizardSelection(a==='channel-select');
  else if(a==='channel-manual')wizardManual();
  else if(a==='manual-model')manualDetailModel();
  else if(a==='refresh')refresh();
  else if(a==='cancel-refresh'){provider().refreshToken++;provider().refresh='idle';render();}
  else if(a==='provider-config')providerConfig();
  else if(a==='batch')batch();
  else if(a==='model-config'){const p=draft.providers.find(p=>p.models.some(m=>m.key===id)),m=p.models.find(m=>m.key===id);if(p.id==='native'){modal('模型详情 · 只读',fieldEditor({},resolve(p,m),true)+'<p>原生目录由官方管理，此处仅查看声明。</p>','关闭');return;}editModel(p,m);}
  else if(a==='adopt'||a==='adopt-version')adopt(a==='adopt-version');
  else if(a==='save')saveDialog();
  else if(a==='stop')stopSave();
  else if(a==='verify')verify();
  else if(a==='client-check')checkSavedClient();
  else if(a==='discard')discardChannelDrafts();
  else if(a==='settings'){closeModal();setRoute('settings');showSettingsSection('models');}
  else if(a==='go-channels'){tab='channels';setRoute('models');render();}
  else if(a==='editor-section')scrollEditorSection(id);
  else if(a==='toggle-editor-nav')toggleEditorNav();
  else if(a==='add-variant')editVariant();
  else if(a==='edit-variant')editVariant(id);
  else if(a==='delete-variant'){const b=buffer(),v=b?.variants?.find(v=>v.id===id);if(v)modal('删除版本或后缀','<p>删除 '+esc(v.name)+' 的候选配置，命中模型恢复族通用配置。已保存版本保留。</p>','删除',()=>{b.variants=b.variants.filter(v=>v.id!==id);if(!b.variants.length)delete b.variants;editorDigest++;render();});}
  else if(a==='edit-template')templateEdit();
  else if(a==='cancel-template'){closeModal();generationToken++;generating=false;proposal=null;delete buffers[templateId];if(templateScratch){templateScratch=null;templateId='gpt';versionN=1;templateDetail=false;}editorDigest++;render();}
  else if(a==='save-template')saveTemplate();
  else if(a==='compare')templateCompare();
  else if(a==='version-menu')versionMenu(el);
  else if(a==='template-version-history')openTemplateHistory(id||templateId,templateDetail?'editor':'list');
  else if(a==='history-select-version')chooseHistoryVersion(id,Number(el.dataset.n));
  else if(a==='history-delete-version')requestListVersionDelete(id,Number(el.dataset.n),true);
  else if(a==='history-set-main')setTemplateMain(id+':'+el.dataset.n,true);
  else if(a==='history-delete-selected')deletePickedVersions(id);
  else if(a==='update-all')startTemplateUpdate('all');
  else if(a==='update-selected')startTemplateUpdate('selected');
  else if(a==='clear-template-picks'){templateSelection.clear();render();}
  else if(a==='update-check')checkTemplateUpdates();
  else if(a==='update-review')reviewTemplateUpdates();
  else if(a==='update-stop'){updateSerial++;if(updateRun)updateRun.status='stopped';renderUpdateProgress();}
  else if(a==='update-expand'){const item=updateRun?.items.find(i=>i.id===id);if(item){item.expanded=!item.expanded;redrawUpdateReview();}}
  else if(a==='update-details')updateDetails(id);
  else if(a==='update-details-close')closeUpdateDetails();
  else if(a==='template-recovery')templateRecovery(id||templateId);
  else if(a==='confirm-template-use')confirmTemplateUse();
  else if(a==='restore-template-record')restore(id);
  else if(a==='restore')restore(id);
  else if(a==='restore-confirmed')restoreConfirmed(id);
  else if(a==='set-main')setTemplateMain(id);
  else if(a==='archive-version'){const [tid,n]=id.split(':');listVersionArchive(tid,Number(n));}
  else if(a==='delete-version')deleteTemplateVersion(id);
  else if(a==='new-template'||a==='copy-template')newTemplate(a==='copy-template');
  else if(a==='generate')generateDialog();

  else if(a==='assistant-size'){if(!assistantOpen||!$('#mpAssistantPanel'))return;assistantMaximized=!assistantMaximized;syncAssistantSize();$('#mpAssistantSizeToggle')?.focus({preventScroll:true});}
  else if(a==='assistant-close'){assistantOpen=false;renderAssistant();$('#mpTemplateAgentAction',host)?.focus({preventScroll:true});}
  else if(a==='generate-run'){const field=$('#mpGenerationTarget'),message=field?.value.trim();if(message){assistantInputs[templateId]=message;generationSources=$('#mpGenerationSources').value;assistantLock=$('#mpAssistantLock').checked;runGeneration(message,conversation.some(m=>m.templateId===templateId));}}
  else if(a==='correct'){const message=$('[data-correction]').value.trim();if(message)runGeneration(message,true);}
  else if(a==='stop-generation'){generationToken++;generating=false;conversation.push({templateId:generationOwner||templateId,role:'agent',text:'已停止；当前候选和上下文保留。'});render();refreshConversation();}
  else if(a==='accept-proposal')proposalReview();
  else if(a==='reject-proposal'){proposal=null;render();refreshConversation();}
  else if(a==='task-pack')taskPack();
  else if(a==='match-preview')matchPreview();
  else if(a==='external-agent')externalAgentDialog();
  else if(a==='external-new')externalAgentDialog(false);

  else if(a==='external-open-version'){const t=templates.find(t=>t.id===el.dataset.template),v=t&&version(t.id,Number(el.dataset.version));if(t&&v){closeModal();tab='templates';templateId=t.id;versionN=buffers[t.id]?.base??v.n;templateDetail=true;render();}}
  else if(a==='external-cancel')cancelExternalTask();
  else if(a==='external-receipt')externalContext();
  else if(a==='external-preflight')externalPreflightDemo();
  else if(a==='external-submit')externalSubmitDemo(el.dataset.scene||'normal');
  else if(a==='external-view-changes')externalBatchPreview();
  else if(a==='external-context')externalContext();
  else if(a==='external-copy')copyExternalPrompt();
  else if(a==='external-task-copy')copyExternalTask();
  else if(a==='external-result')externalResultDialog();
  else if(a==='external-demo')externalDemo();
  else if(a==='open-sync')syncEntry();
  else if(a==='import')importDialog();
  else if(a==='export')exportDialog();
  else if(a==='incoming')incomingDialog();
  else if(a==='sync-incoming'){closeModal();window.syncPrototype?.open({tab:'webdav'});}
  else if(a==='scene')scene(el.dataset.scene);
 });
 document.addEventListener('change',e=>{const el=e.target;if(['mpExternalScope','mpExternalScene','mpExternalAllowCreate'].includes(el.id)){updateExternalPreview();return;}if(el.matches('[data-version-pick]')||el.matches('[data-version-pick-all]'))pickVersions(el);else if(el.matches('[data-template-select]')){el.checked?templateSelection.add(el.dataset.templateSelect):templateSelection.delete(el.dataset.templateSelect);render();}
  else if(el.matches('[data-assistant-task]')){assistantTask=el.value==='update'?'update':'fill';if(assistantTask==='fill'&&!buffer())templateEdit();renderAssistant();}
  else if(el.matches('[data-update-scope]'))updateScope=el.value;
  else if(el.matches('[data-update-pick]')){const [tid,rid]=el.dataset.updatePick.split(':'),row=updateRun?.items.find(i=>i.id===tid)?.rows.find(r=>r.id===rid);if(row){row.selected=el.checked;syncUpdateChecks();}}
  else if(el.matches('[data-update-group]')){const item=updateRun?.items.find(i=>i.id===el.dataset.updateGroup);if(item){item.rows.forEach(r=>r.selected=el.checked&&!r.conflict);redrawUpdateReview();}}
  else if(el.matches('[data-assistant-sources]'))generationSources=el.value;
  else if(el.matches('[data-lock-field]'))assistantLock=el.checked;
  else if(el.matches('[data-template-match]'))updateTemplateInput(el);
  else if(el.matches('[data-template-channel]')){const b=buffer();if(!b)return;const id=el.dataset.templateChannel;if(!draft.providers.some(p=>p.id===id)||channelMode(b.match)==='all')return;b.match.channels=el.checked?[...new Set([...b.match.channels,id])]:b.match.channels.filter(x=>x!==id);editorDigest++;updateTemplateSaveStatus();}
  else if(el.matches('[data-channel-model]')&&channelWizard&&channelWizard.status!=='loading'){const m=channelWizard.models.find(m=>m.id===el.dataset.channelModel);if(m)m.selected=el.checked;drawChannelWizard();}
  else if(el.matches('[data-scope-all]')){if(busy()||el.disabled)return;scopeState().rows.forEach(p=>el.checked?scope.add(p.id):scope.delete(p.id));render();}
  else if(el.matches('[data-scope]')){if(busy()||el.disabled||!draft.providers.some(p=>p.id===el.dataset.scope&&p.id!=='native'))return;el.checked?scope.add(el.dataset.scope):scope.delete(el.dataset.scope);render();}
  else if(el.matches('[data-model-pick]')){if(busy()||el.disabled||provider().id==='native'||!provider().models.some(m=>m.key===el.dataset.modelPick))return;el.checked?modelSelection.add(el.dataset.modelPick):modelSelection.delete(el.dataset.modelPick);render();}
  else if(el.matches('[data-model-pick-all]')){if(busy()||el.disabled||provider().id==='native')return;visibleModels().forEach(m=>el.checked?modelSelection.add(m.key):modelSelection.delete(m.key));render();}
  else if(el.matches('[data-channel-config]'))updateChannelConnection(el);
  else if(el.matches('[data-model-sort]')){const rule=el.value;el.value='';if(!el.disabled)sortModels(rule);}
  else if(el.matches('[data-model-alias]')){if(busy()||provider().id==='native')return;const m=provider().models.find(m=>m.key===el.dataset.modelAlias);if(!m)return;const error=commitModelPatches(current,[{key:m.key,naming:{...m.naming,alias:el.value}}]);if(error){el.value=m.naming?.alias||'';el.removeAttribute('aria-invalid');modelError(m.key,'别名未加入草稿：'+error);}else el.removeAttribute('aria-invalid');}
  else if(el.matches('[data-quick-mode],[data-quick-value],[data-quick-preset]'))quickChanged(el);
  else if(el.matches('[data-mode]')){const p=provider(),value=el.value;if(value==='none')modal('隐藏本渠道全部模型？','<p>保留发现和顺序；选择模式明确为全部隐藏。</p>','加入草稿',()=>change(()=>p.mode=value),{onClose:render});else change(()=>p.mode=value);}
  else if(el.matches('[data-family]')){family=el.value;render();}
  else if(el.matches('[data-only-selected]')){onlySelected=el.checked;render();}
  else if(el.matches('[data-current]')){current=el.value;search='';family='';render();}
  else if(el.matches('[data-template-current]'))openTemplateEditor(el.value);
  else if(el.matches('[data-template-row-version]')){const id=el.dataset.templateRowVersion;if(el.value==='history'){el.value=String(listTemplateVersion(templates.find(t=>t.id===id))?.n||'');window.prototypeSelect?.enhance(host);openTemplateHistory(id,'list');return;}const n=Number(el.value);if(version(id,n)&&!templateEditorChanged(id))templateListVersions[id]=n;else notice('先处理未保存编辑，再切换列表版本');render();window.prototypeSelect?.enhance(host);const source=$('#mpTemplateListVersion-'+id,host),trigger=source?.closest?.('.select')?.querySelector?.('.select-trigger');(trigger||source)?.focus({preventScroll:true});}
  else if(el.matches('[data-version]')){if(el.value==='history'){el.value=String(versionN);window.prototypeSelect?.enhance(host);openTemplateHistory(templateId,'editor');}else switchTemplateVersion(Number(el.value));}
 });
 document.addEventListener('input',e=>{if(e.target.id==='mpExternalRequest'){updateExternalPreview();return;}if(e.target.matches('[data-history-search]')&&templateHistoryView){const view=templateHistoryView,caret=e.target.selectionStart;openTemplateHistory(view.id,view.context,e.target.value);const field=$('#mpVersionHistorySearch');field?.focus({preventScroll:true});if(caret!=null)field?.setSelectionRange?.(caret,caret);}else if(e.target.matches('[data-channel-config]'))updateChannelConnection(e.target);else if(e.target.matches('[data-quick-value="context"]'))quickChanged(e.target);else if(e.target.matches('[data-assistant-input]'))assistantInputs[templateId]=e.target.value;else if(e.target.matches('[data-template-meta],[data-template-match]'))updateTemplateInput(e.target);else if(e.target.matches('[data-template-prompt]'))generationTarget=e.target.value;else if(e.target.matches('[data-model-search]')){search=e.target.value;render();}else if(e.target.matches('[data-channel-search]')){channelSearch=e.target.value;render();}else if(e.target.matches('[data-template-search]')){templateSearch=e.target.value;render();}else if(e.target.matches('[data-correction]')){correctionDraft=e.target.value;}});
/* 拖拽/操作区只看手势，不阻止输入与正文选择；普通点击不会清空用户既有选择。 */
 let gesture=null;
 document.addEventListener('selectstart',e=>{if(gesture?.dragging&&e.target.closest?.('.mp-page'))e.preventDefault();},true);
 document.addEventListener('pointerdown',e=>{
  if(nodePicker&&!nodePicker.element.contains(e.target)&&!nodePicker.anchor.contains(e.target))closeNodePicker();
  const target=e.target.closest?.('.mp-page .mp-grip,.mp-page .mp-provider-actions,.mp-page .mp-model-actions,.mp-page .mp-template-tools,.mp-page .mp-nav-item');
  gesture=target?{target,dragging:!!e.target.closest?.('.mp-grip'),selectable:false}:null;
 },true);
 document.addEventListener('dragstart',e=>{if(gesture)gesture.dragging=true;},true);
 document.addEventListener('dragend',()=>{gesture=null;},true);
 document.addEventListener('pointerup',()=>{gesture=null;},true);
 document.addEventListener('pointercancel',()=>{gesture=null;},true);
 window.addEventListener?.('blur',()=>{gesture=null;});
 document.addEventListener('focusin',e=>{if(nodePicker&&!nodePicker.element.contains(e.target)&&!nodePicker.anchor.contains(e.target))closeNodePicker();});
 document.addEventListener('keydown',e=>{
  if(!nodePicker)return;if(e.key==='Escape'){e.preventDefault();e.stopImmediatePropagation();closeNodePicker(true);return;}
  if(!['ArrowDown','ArrowUp','Home','End'].includes(e.key)||!nodePicker.element.contains(e.target))return;
  const choices=$$('[data-mp="node-picker-choose"]:not(:disabled)',nodePicker.element);if(!choices.length)return;e.preventDefault();
  const index=choices.indexOf(document.activeElement),next=e.key==='Home'?0:e.key==='End'?choices.length-1:e.key==='ArrowDown'?(index+1)%choices.length:(index-1+choices.length)%choices.length;choices[next]?.focus({preventScroll:true});
 },true);
 document.addEventListener('scroll',e=>{if(nodePicker&&!nodePicker.element.contains(e.target))closeNodePicker();},true);
 window.addEventListener?.('resize',()=>closeNodePicker(),{passive:true});
 window.addEventListener?.('hashchange',()=>closeNodePicker());
 document.addEventListener('scroll',e=>{if(e.target===host.closest('.main'))syncDetailSection();},{capture:true,passive:true});
 window.addEventListener?.('resize',syncDetailSection,{passive:true});
 if(typeof ResizeObserver!=='undefined'){const observer=new ResizeObserver(syncDetailSection);observer.observe(host);const scroller=host.closest('.main');if(scroller)observer.observe(scroller);}
 let drag=null,dragOver=null;
 const clearDragMarks=()=>{$$('.mp-page .dragging,.mp-page .drag-over',host).forEach(node=>node.classList.remove('dragging','drag-over'));};
 document.addEventListener('dragstart',e=>{if(e.target.closest('input,button,select,.select')){e.preventDefault();return;}const el=e.target.closest('[draggable="true"]');if(!el||!el.closest('#route-models'))return;drag=el.dataset.provider?{type:'provider',id:el.dataset.provider}:{type:'model',id:el.dataset.model};el.classList.add('dragging');e.dataTransfer.setData('text/plain',drag.id);if(e.dataTransfer)e.dataTransfer.effectAllowed='move';});
 document.addEventListener('dragover',e=>{const el=e.target.closest('[draggable="true"]');if(!drag||!el||el===dragOver)return;e.preventDefault();dragOver?.classList.remove('drag-over');dragOver=el;el.classList.add('drag-over');});
 document.addEventListener('dragleave',e=>{if(e.target===dragOver&&!dragOver?.contains(e.relatedTarget)){dragOver.classList.remove('drag-over');dragOver=null;}});
 document.addEventListener('drop',e=>{const el=e.target.closest('[draggable="true"]');if(!el||!drag)return;e.preventDefault();const target=drag.type==='provider'?el.dataset.provider:el.dataset.model;clearDragMarks();if(target){const list=drag.type==='provider'?draft.providers:provider().models,key=drag.type==='provider'?'id':'key';change(()=>{const from=list.findIndex(x=>x[key]===drag.id),to=list.findIndex(x=>x[key]===target);if(from>=0&&to>=0)list.splice(to,0,list.splice(from,1)[0]);});}drag=null;dragOver=null;});
 document.addEventListener('dragend',()=>{clearDragMarks();drag=null;dragOver=null;});
 window.modelPrototype={applyAgentDemo,openExternalTask,receiveExternalSubmission,externalTaskPack,externalSample,externalCombinedText,updateExternalPreview,state:()=>clone({draft,saved,pending,client,operation,templates,scope:[...scope],buffers,proposal,incoming,records,scenario,agent,modelSelection:[...modelSelection],modelExpanded:[...modelExpanded],modelEdits,channelDetail,templateDetail,templateScratch,templateSelection:[...templateSelection],templateListVersions,templateHistoryView,externalTask,externalTasks:[...externalTasks.values()],externalBatch,editorSection,editorNavCollapsed,channelSection,channelNavCollapsed,channelConnections,assistantOpen,assistantMaximized,assistantInputs,conversation,assistantTask,updateScope,updateRun,updateBatches,confirmedConfigurations,restoration,nodePickerChannel:nodePicker?.pid||null,nodeSpeedResults,nodeSpeedRuns,nodeSpeedScene}),templateHeadingState:()=>clone(templateHeadingState()),agentChoices,agentInfo:selection=>clone(agentInfo(selection)),versionText,refText,versionProtectionReasons,checkTemplateUpdates,reviewTemplateUpdates,openTemplateUpdate,templateRecovery,confirmTemplateUse,scene,setNodeSpeedScene,commitModelPatches,modelTargets:()=>modelTargets().map(m=>m.key),readQuick,stagedModelEdits,quickConfig,stagedChannelConnections,projection:()=>projection(),routePlan,resolveRoute:(pid,key,policy=draft)=>{const p=policy.providers.find(p=>p.id===pid);return resolveRoute(p,p.models.find(m=>m.key===key),policy);},resolve:(pid,key)=>{const p=draft.providers.find(p=>p.id===pid);return resolve(p,p.models.find(m=>m.key===key));},templateMatches:(tid,modelId,pid='newapi',n)=>{const p=draft.providers.find(p=>p.id===pid);return templateMatches(templates.find(t=>t.id===tid),{id:modelId},p,n?version(tid,n):null);},templateResolution:(pid,key)=>{const p=draft.providers.find(p=>p.id===pid);return templateResolution(p,p.models.find(m=>m.key===key));},render};
 if(typeof MutationObserver!=='undefined'){new MutationObserver(()=>{const route=document.documentElement.dataset.route;if(route&&route!=='models'){closeNodePicker();$('#mpAssistantHost').hidden=true;}else {$('#mpAssistantHost').hidden=false;syncDetailSection();renderAssistant();}}).observe(document.documentElement,{attributes:true,attributeFilter:['data-route','data-effects','data-theme']});}
 render();
})();
