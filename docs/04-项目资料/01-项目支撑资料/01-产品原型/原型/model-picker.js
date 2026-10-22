/* 公共选择器打样页与业务弹窗共享渲染；所有目录、能力和返回值都是内存 fixture。 */
(() => {
 'use strict';
 const $=(q,r=document)=>r.querySelector(q), $$=(q,r=document)=>[...r.querySelectorAll(q)];
 const esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 const clone=v=>JSON.parse(JSON.stringify(v));
 const modal=$('#modal'),body=$('#modalBody'),page=$('#modelPickerPage');if(!modal||!page)return;
 const button=(name,action,attrs='')=>'<button type="button" class="btn ghost" data-mpk="'+action+'" '+attrs+'>'+name+'</button>';
 const keyFor=ref=>ref.kind==='native'?JSON.stringify(['native',ref.nativeModelId]):JSON.stringify(['channel',ref.channelId,ref.modelId]);
 const refLabel=ref=>(ref.channelId||'官方 / 原生')+' / '+(ref.modelId||ref.nativeModelId);
 const levels={none:'无',minimal:'最低',low:'低',medium:'中',high:'高',xhigh:'极高',max:'最高'};
 const levelOrder=['none','minimal','low','medium','high','xhigh','max'];
 const MIN_LOADING_MS=200;let loadingSince=0;
 let presentation='dialog',mode='single',scenario='normal',orderEnabled=true,recommendedEnabled=true,reasoningEnabled=true,fastEnabled=true,isOpen=false;
 let catalog=[],channels=[],pending=[],channel='all',query='',channelQuery='',sort={key:null,direction:0},opener=null;
 let unresolved=new Map(),browseScroll=0,channelScroll=0,previousRoute='conversations',effortChoices=new Map(),fastChoices=new Map();
 let businessSession=null;
 const committed={single:{kind:'models',models:[{kind:'channel',channelId:'newapi',modelId:'gpt-5.4'}]},multiple:{kind:'models',models:[{kind:'channel',channelId:'newapi',modelId:'gpt-5.4'},{kind:'channel',channelId:'aimami',modelId:'gpt-5.4-mini'}]}};
 const outcomes={single:'尚未返回选择。',multiple:'尚未返回选择。'};
 const recommendedRefs=[{kind:'channel',channelId:'newapi',modelId:'gpt-5.4-mini'},{kind:'channel',channelId:'aimami',modelId:'gpt-5.4'}];
 const active=()=>!!businessSession?.container||isOpen||document.documentElement.dataset.route==='model-picker';
 const locationFor=id=>{const m=catalog.find(m=>m.key===id);return m?m.channelName+' / '+m.rawId:unresolved.get(id)?.description||'引用无法解析';};
 const resultFor=ids=>({kind:'models',models:ids.map(id=>{const ref=catalog.find(m=>m.key===id)?.ref||unresolved.get(id)?.ref;if(!ref)return null;const chosen=reasoningEnabled?effortChoices.get(id):null;return {...clone(ref),...(chosen?{reasoningEffort:chosen}:{}),...(fastEnabled&&fastChoices.has(id)?{fast:fastChoices.get(id)}:{})};}).filter(Boolean)});
 const supportedEfforts=m=>levelOrder.filter(level=>m?.efforts.includes(level));
 const defaultEffort=m=>levels[m?.effort]?m.effort:null;
 const effectiveEffort=m=>effortChoices.get(m?.key)||defaultEffort(m);
 const effortName=id=>levels[effectiveEffort(catalog.find(m=>m.key===id))]||'未知';
 const canFast=m=>m?.fastCapability==='supported';
 const effectiveFast=m=>fastChoices.has(m?.key)?fastChoices.get(m.key):m?.fastDefault===true;
 const invalidFast=id=>fastEnabled&&fastChoices.get(id)===true&&!canFast(catalog.find(m=>m.key===id));
 const effortAttrs=(m,source)=>(reasoningEnabled||fastEnabled)&&!m.reason?' data-effort-key="'+esc(m.key)+'" data-effort-source="'+source+'" aria-haspopup="dialog" aria-expanded="false" aria-controls="mpkEffortPopover"':'';
 const invalidEffort=id=>reasoningEnabled&&effortChoices.get(id)&&!supportedEfforts(catalog.find(m=>m.key===id)).includes(effortChoices.get(id));
 const effortPopover=document.createElement('div');effortPopover.id='mpkEffortPopover';effortPopover.className='mpk-effort-popover';effortPopover.hidden=true;effortPopover.setAttribute('role','dialog');effortPopover.setAttribute('aria-label','选择推理档位');document.body.appendChild(effortPopover);
 let effortAnchor=null,effortKey=null,effortSteps=[],effortIndex=0,hoverTimer=null,leaveTimer=null,rangePointer=false,effortPinned=false,effortSelectionIntent=false,effortSnapshot=null,effortSortPending=false;
 const cancelTimers=()=>{clearTimeout(hoverTimer);clearTimeout(leaveTimer);hoverTimer=leaveTimer=null;};
 function closeEffort(restoreFocus=false){cancelTimers();const anchor=effortAnchor;anchor?.setAttribute('aria-expanded','false');effortPopover.hidden=true;effortKey=null;effortAnchor=null;rangePointer=false;effortPinned=false;effortSelectionIntent=false;effortSnapshot=null;if(effortSortPending){effortSortPending=false;sortEffortRows();}if(restoreFocus&&anchor?.isConnected){skipEffortFocus=true;anchor.focus({preventScroll:true});skipEffortFocus=false;}}
 let skipEffortFocus=false;
 function positionEffort(){
  if(effortPopover.hidden||!effortAnchor?.isConnected)return;
  const windowRect=$('.window').getBoundingClientRect(),anchorRect=effortAnchor.getBoundingClientRect();
  const left=Math.max(8,windowRect.left+12),right=Math.min(innerWidth-8,windowRect.right-12),top=Math.max(8,windowRect.top+12),bottom=Math.min(innerHeight-8,windowRect.bottom-12);
  effortPopover.style.width=Math.min(268,right-left)+'px';effortPopover.style.maxHeight=Math.max(120,bottom-top)+'px';
  const width=effortPopover.offsetWidth,height=effortPopover.offsetHeight;
  const x=Math.max(left,Math.min(anchorRect.left+(anchorRect.width-width)/2,right-width));
  const above=anchorRect.top-8-height,y=above>=top?above:Math.min(bottom-height,Math.max(top,anchorRect.bottom+8));
  effortPopover.style.left=x+'px';effortPopover.style.top=Math.max(top,Math.min(bottom-height,y))+'px';
 }
 function previewEffort(index){
  effortIndex=Number(index)<0?-1:Math.max(0,Math.min(effortSteps.length-1,Number(index)));const choice=effortSteps[effortIndex],label=levels[choice]||effortName(effortKey);
  const value=$('#mpkEffortValue',effortPopover);if(value)value.textContent=label;
  const range=$('#mpkEffortRange',effortPopover);if(range){range.value=Math.max(0,effortIndex);range.classList.toggle('is-unresolved',effortIndex<0);range.setAttribute('aria-valuetext',label);range.style.setProperty('--effort-progress',100*Math.max(0,effortIndex)/Math.max(1,effortSteps.length-1)+'%');}
  $$('[data-effort-step]',effortPopover).forEach(b=>b.setAttribute('aria-pressed',String(Number(b.dataset.effortStep)===effortIndex)));
 }
 function applyEffort(index=effortIndex){chooseEffort(effortSteps[index]||null);}
 function refreshEffortDraft(){
  const id=effortKey;if(!id)return;
  effortSelectionIntent=true;effortPinned=true;
  if(mode==='single')pending=[id];else if(!pending.includes(id))pending.push(id);
  refreshSelection();previewEffort(effortIndex);syncFastControl();
 }
 function chooseEffort(choice){
  const id=effortKey,m=catalog.find(m=>m.key===id);if(!reasoningEnabled||!id||!m||m.reason||choice&&!supportedEfforts(m).includes(choice))return;
  effortChoices.set(id,choice||null);effortIndex=effortSteps.indexOf(effectiveEffort(m));refreshEffortDraft();
 }
 function toggleFast(){
  const m=catalog.find(m=>m.key===effortKey);if(!fastEnabled||!m||m.reason||!canFast(m)&&!effectiveFast(m))return;
  fastChoices.set(m.key,!effectiveFast(m));refreshEffortDraft();
 }
 function syncFastControl(){
  const m=catalog.find(m=>m.key===effortKey),control=$('[data-effort-fast]',effortPopover);if(control){control.setAttribute('aria-checked',String(effectiveFast(m)));control.disabled=!canFast(m)&&!effectiveFast(m);control.setAttribute('aria-disabled',String(control.disabled));}
 }
 function dismissEffort(){closeEffort();}
 function cancelEffort(){
  const snapshot=effortSnapshot,anchor=effortAnchor;
  if(snapshot){pending=snapshot.pending;effortChoices=snapshot.efforts;fastChoices=snapshot.fast;}
  closeEffort();renderLists();const next=$$('[data-effort-key]').find(el=>el.dataset.effortKey===anchor?.dataset.effortKey&&el.dataset.effortSource===anchor?.dataset.effortSource);
  if(next){skipEffortFocus=true;next.focus({preventScroll:true});skipEffortFocus=false;}
 }
 function showEffort(anchor,focusRange=false){
  if(!(reasoningEnabled||fastEnabled)||!active())return;const id=anchor?.dataset.effortKey,m=catalog.find(m=>m.key===id);if(!m||m.reason)return;
  if(effortPinned&&!effortPopover.hidden&&effortAnchor!==anchor&&!focusRange)return;
  cancelTimers();if(effortKey===id&&effortAnchor===anchor&&!effortPopover.hidden){if(focusRange){effortPinned=true;($('#mpkEffortRange',effortPopover)||$('[data-effort-fast]',effortPopover)||$('[data-effort-apply]',effortPopover))?.focus();}return;}
  effortAnchor?.setAttribute('aria-expanded','false');effortAnchor=anchor;anchor.setAttribute('aria-expanded','true');effortKey=id;effortPinned=focusRange;effortSelectionIntent=false;effortSnapshot={pending:pending.slice(),efforts:new Map(effortChoices),fast:new Map(fastChoices)};effortSteps=supportedEfforts(m);effortIndex=effortSteps.indexOf(effectiveEffort(m));
  const reset=reasoningEnabled?'<button type="button" class="mpk-effort-icon" data-effort-reset aria-label="恢复默认推理档位" title="恢复默认"><svg viewBox="0 0 20 20" aria-hidden="true"><path d="M4 7a7 7 0 1 1-1 5M4 3v4h4"/></svg></button>':'';
  const slider=!reasoningEnabled?'':effortSteps.length>0?'<div class="mpk-effort-slider"><input type="range" id="mpkEffortRange" min="0" max="'+(effortSteps.length-1)+'" step="1" value="'+Math.max(0,effortIndex)+'" aria-label="'+esc(locationFor(id))+' 推理档位"><div class="mpk-effort-stops" style="--effort-stops:'+effortSteps.length+'">'+effortSteps.map((step,i)=>'<button type="button" data-effort-step="'+i+'" aria-label="选择推理 '+esc(levels[step])+'"><span class="mpk-effort-dot" aria-hidden="true"></span><span>'+esc(levels[step])+'</span></button>').join('')+'</div></div>':'<p class="mpk-effort-unknown">未提供可选档位</p>';
  const fastButton=fastEnabled?'<button type="button" class="mpk-effort-fast" data-effort-fast role="switch" aria-label="快速模式" aria-checked="'+effectiveFast(m)+'" '+(!canFast(m)&&!effectiveFast(m)?'disabled aria-disabled="true"':'')+' title="'+(canFast(m)?'快速模式':effectiveFast(m)?'快速模式支持已变化，点击关闭':m.fastCapability==='unsupported'?'此模型不支持快速模式':'快速模式支持未知')+'"><svg viewBox="0 0 20 20" aria-hidden="true"><path d="m11 2-7 9h5l-1 7 8-10h-5z"/></svg></button>':'<span></span>';
  effortPopover.innerHTML='<div class="mpk-effort-top">'+fastButton+(reasoningEnabled?'<button type="button" class="mpk-effort-current" data-effort-apply aria-label="采用当前推理档位"><span id="mpkEffortValue"></span></button>':'<span class="mpk-fast-title">快速模式</span>')+reset+'</div><div class="mpk-effort-model">'+esc(locationFor(id))+'</div>'+slider;
  effortPopover.hidden=false;previewEffort(effortIndex);positionEffort();if(focusRange)($('#mpkEffortRange',effortPopover)||$('[data-effort-fast]:not(:disabled)',effortPopover)||$('[data-effort-apply]',effortPopover))?.focus({preventScroll:true});
 }
 function scheduleEffortClose(){clearTimeout(leaveTimer);leaveTimer=setTimeout(()=>{if(!effortPinned&&!effortPopover.contains(document.activeElement))closeEffort();},220);}
 document.addEventListener('pointerover',e=>{
  if(effortPopover.contains(e.target)){cancelTimers();return;}const anchor=e.target.closest('[data-effort-key]');if(!anchor||anchor===e.relatedTarget?.closest?.('[data-effort-key]'))return;
  cancelTimers();if(!effortPopover.hidden)showEffort(anchor);else hoverTimer=setTimeout(()=>showEffort(anchor),70);
 });
 document.addEventListener('pointerout',e=>{const anchor=e.target.closest('[data-effort-key]');if(anchor&&!anchor.contains(e.relatedTarget)&&!effortPopover.contains(e.relatedTarget))scheduleEffortClose();if(effortPopover.contains(e.target)&&!effortPopover.contains(e.relatedTarget)&&!effortAnchor?.contains(e.relatedTarget))scheduleEffortClose();});
 document.addEventListener('focusin',e=>{const anchor=e.target.closest('[data-effort-key]');if(anchor&&!skipEffortFocus)showEffort(anchor);else if(!effortPopover.contains(e.target))dismissEffort();});
 document.addEventListener('pointerdown',e=>{if(e.target.id==='mpkEffortRange'){rangePointer=true;effortPinned=true;}else if(!effortPopover.hidden&&!effortPopover.contains(e.target)&&!e.target.closest('[data-effort-key]')){const t=e.target,action=t.closest('[data-mpk]')?.dataset.mpk,cancel=t.closest('#modalCancel')||t.id==='modalMask'||action==='cancel'||action==='back-page'||t.closest('.prototype-side')||t.closest('.nav');if(cancel)closeEffort();else dismissEffort();}},true);
 document.addEventListener('pointerup',e=>{if(rangePointer){rangePointer=false;applyEffort();}});
 document.addEventListener('pointercancel',()=>{rangePointer=false;closeEffort();});
 effortPopover.addEventListener('input',e=>{if(e.target.id==='mpkEffortRange'){previewEffort(e.target.value);applyEffort();}});
 effortPopover.addEventListener('click',e=>{const step=e.target.closest('[data-effort-step]');if(step)applyEffort(Number(step.dataset.effortStep));else if(e.target.closest('[data-effort-fast]'))toggleFast();else if(e.target.closest('[data-effort-reset]'))chooseEffort(null);else if(e.target.closest('[data-effort-apply]'))applyEffort();});
 effortPopover.addEventListener('wheel',e=>{if(effortSteps.length<2)return;e.preventDefault();if(e.deltaY){previewEffort(effortIndex+(e.deltaY>0?1:-1));applyEffort();}},{passive:false});
 document.addEventListener('keydown',e=>{if(effortPopover.hidden)return;if(e.key==='Tab'&&effortPopover.contains(e.target)){const items=$$('button:not([disabled]),input:not([disabled])',effortPopover),edge=e.shiftKey?items[0]:items.at(-1);if(e.target===edge){e.preventDefault();const anchor=effortAnchor;dismissEffort();if(isOpen){const next=$$('[data-effort-key]').find(el=>el.dataset.effortKey===anchor?.dataset.effortKey&&el.dataset.effortSource===anchor?.dataset.effortSource)||$('#mpkSearch');skipEffortFocus=true;next?.focus({preventScroll:true});skipEffortFocus=false;}return;}}if(e.key==='Escape'){e.preventDefault();e.stopImmediatePropagation();cancelEffort();}else if(e.key==='Enter'&&e.target.id==='mpkEffortRange'){e.preventDefault();e.stopImmediatePropagation();applyEffort();}},true);
 document.addEventListener('scroll',e=>{if(!effortPopover.hidden&&!effortPopover.contains(e.target))closeEffort();},true);
 window.addEventListener('resize',()=>closeEffort());
 function loadCatalog(){
  if(businessSession){
   const rows=businessSession.options.getCatalog?.()||[];catalog=rows.map((m,i)=>({...clone(m),key:keyFor(m.ref),channel:m.ref.kind==='native'?'native':m.ref.channelId,rawId:m.ref.modelId||m.ref.nativeModelId,position:i,reason:m.reason||'',alias:m.alias||'',efforts:Array.isArray(m.efforts)?m.efforts:[],modalities:m.modalities||[],fastCapability:m.fastCapability||'unknown',fastDefault:m.fastDefault===true}));
   const found=new Map();catalog.forEach(m=>{if(!found.has(m.channel))found.set(m.channel,{id:m.channel,name:m.channelName,position:found.size});});channels=[...found.values()];
   unresolved=new Map();for(const ref of businessSession.options.models||[])if(!catalog.some(m=>m.key===keyFor(ref)))unresolved.set(keyFor(ref),{ref:clone(ref),description:refLabel(ref)});
   return;
  }
  const saved=window.modelPrototype?.state().saved;channels=(saved?.providers||[]).map((p,i)=>({id:p.id,name:p.name,position:i}));catalog=[];
  for(const p of saved?.providers||[])for(const m of p.models){
   const ref=p.id==='native'?{kind:'native',nativeModelId:m.id}:{kind:'channel',channelId:p.id,modelId:m.id};
   const reason=!p.enabled?'渠道已停用':({'no-credit':'无额度（示例）','auth-failed':'认证未通过','connection-failed':'连接不可用',unavailable:'上游不可用'})[p.availability?.kind]||'';
   const ctx=m.overrides?.context??p.defaults?.context??m.official?.context;
   const effort=m.overrides?.default??p.defaults?.default??m.official?.default;
   const efforts=m.overrides?.efforts??p.defaults?.efforts??m.official?.efforts??[];
   const fastCapability=m.overrides?.fast??p.defaults?.fast??m.official?.fast??'unknown';
   catalog.push({key:keyFor(ref),ref,channel:p.id,channelName:p.name,name:m.naming?.displayName||(m.id==='gpt-5.4'?'GPT 5.4':m.id==='gpt-5.4-mini'?'GPT 5.4 mini':m.id),rawId:m.id,alias:m.naming?.alias||'',position:catalog.length,mainPickerSelected:!!m.selected,reason,context:typeof ctx==='number'?ctx:null,effort:typeof effort==='string'?effort:null,efforts:Array.isArray(efforts)?efforts:[],modalities:m.official?.modalities||[],fastCapability,fastDefault:false});
  }
  if(scenario==='long'){
   channels.push({id:'enterprise-demo',name:'企业自建模型网关 · 长渠道名称示例',position:channels.length});
   for(let i=0;i<8;i++){const ref={kind:'channel',channelId:'enterprise-demo',modelId:'company-development-model-with-a-very-long-upstream-identifier-'+(i+1)};catalog.push({key:keyFor(ref),ref,channel:'enterprise-demo',channelName:channels.at(-1).name,name:'企业研发模型 · 长名称换行 '+(i+1),rawId:ref.modelId,alias:'company-'+(i+1),position:catalog.length,reason:'',context:null,effort:null,efforts:[],modalities:[]});}
  }
  if(scenario==='efforts'){for(const m of catalog){if(m.rawId==='gpt-5.4-mini'){m.efforts=['minimal','low','medium'];m.effort='medium';}else if(m.rawId==='qwen3-max'){m.efforts=['none','low','high'];m.effort='low';}else if(m.rawId==='gemini-3.1-pro-preview'){m.efforts=[];m.effort=null;}}}
  if(scenario==='empty')catalog=[];
  unresolved=new Map();for(const result of Object.values(committed))for(const ref of result.models||[])if(!catalog.some(m=>m.key===keyFor(ref)))unresolved.set(keyFor(ref),{ref:clone(ref),description:refLabel(ref)});
 }
 function resetSelection(){const selected=businessSession?.options.models||committed[mode].models||[];pending=selected.map(keyFor);effortChoices=new Map(selected.map(ref=>[keyFor(ref),ref.reasoningEffort||null]));fastChoices=new Map(selected.filter(ref=>typeof ref.fast==='boolean').map(ref=>[keyFor(ref),ref.fast]));
  if(businessSession)return;
  if(scenario==='missing'){const ref={kind:'channel',channelId:'retired-demo',modelId:'removed-model'},id=keyFor(ref);unresolved.set(id,{ref,description:'已移除渠道 / removed-model'});if(mode==='single')pending=[id];else if(!pending.includes(id))pending.push(id);}
 }
 const matches=m=>!query.trim()||[m.name,m.rawId,m.alias].some(x=>x.toLocaleLowerCase().includes(query.toLocaleLowerCase().trim()));
 function sortValue(m,key){if(key==='name')return m.name;if(key==='channel')return m.channelName;if(key==='context')return m.context;if(key==='effort'){const chosen=reasoningEnabled?effectiveEffort(m):m.effort;return chosen?levelOrder.indexOf(chosen):null;}if(key==='status')return m.reason?m.reason:'可选';if(key==='recommendation'){const n=recommendedRefs.findIndex(r=>keyFor(r)===m.key);return n<0?null:n;}return null;}
 function filtered(){
  const rows=catalog.filter(m=>(channel==='all'||m.channel===channel)&&matches(m));
  if(sort.key)rows.sort((a,b)=>{const av=sortValue(a,sort.key),bv=sortValue(b,sort.key);if(av==null&&bv==null)return a.position-b.position;if(av==null)return 1;if(bv==null)return -1;const c=typeof av==='number'?av-bv:String(av).localeCompare(String(bv),'zh-CN');return sort.direction*c||a.position-b.position;});
  return rows;
 }
 const eligible=()=>filtered().filter(m=>!m.reason).map(m=>m.key);
 function channelHTML(){
  if(scenario==='loading')return '<button type="button" class="mpk-channel" aria-current="true" aria-disabled="true"><span>全部</span><small>—</small></button><div class="mpk-channels-list">'+[62,74,56,84,68,60].map(width=>'<button type="button" class="mpk-channel mpk-skeleton-channel" aria-hidden="true" tabindex="-1"><span class="mpk-skeleton mpk-skeleton-line" style="width:'+width+'%"></span></button>').join('')+'</div>';
  const count=id=>catalog.filter(m=>(id==='all'||m.channel===id)&&matches(m)).length;
  const row=(id,name)=>'<button type="button" class="mpk-channel" data-mpk="channel" data-id="'+esc(id)+'" aria-label="查看'+esc(name)+'模型" aria-current="'+(channel===id)+'"><span>'+esc(name)+'</span><small>'+count(id)+'</small></button>';
  const found=channels.filter(c=>(c.name+' '+c.id).toLocaleLowerCase().includes(channelQuery.trim().toLocaleLowerCase()));
  return row('all','全部')+'<div class="mpk-channels-list">'+(found.length?found.map(c=>row(c.id,c.name)).join(''):'<div class="mpk-channel-empty">没有匹配的渠道<br>'+button('清空筛选','clear-channels')+'</div>')+'</div>';
 }
 function annotation(){const id=isOpen?'ann-mpk-dialog':'ann-mpk-page';return '<div class="proto-annotation" id="'+id+'" data-annotation-number="01" data-annotation-title="模型选择与排序的操作规则" data-annotation-category="公共组件 · 操作"><button type="button" class="annotation-marker" aria-label="原型注释 01：模型选择与排序的操作规则" aria-expanded="false" aria-controls="annotationPopover"><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 2.5h7l3 3v8H3zM10 2.5v3h3M5.5 8h5M5.5 10.5h3"/></svg><span>原型</span><span class="annotation-number">01</span><span class="annotation-marker-plus">＋</span></button><div class="annotation-source" hidden><p>选择形式由业务决定。单选点击模型只替换唯一待选项；启用推理或快速模式时可连续调整，点击浮层外 / 焦点移出只收起浮层。单选和多选均点击主弹窗“确定选择”后返回型号、推理与 Fast，取消保持原设置。单选底部不提供删除。调用方可启用推理选择：悬停模型或已选标签打开玻璃浮层，松开滑杆或点击档位更新当前草稿并保持浮层；快速模式开关同样保持。方向键 / 滚轮同样更新草稿；Esc 撤回本次浮层修改，主弹窗取消撤回全部未返回修改。未额外指定时显示调用方提供的实际默认档位，滑杆只包含模型支持的真实档位。恢复默认清除显式覆盖，不写入显示值；默认来源未知时如实显示未知，不猜测。配置保存由调用页面负责。</p><p>点表头按升序、降序、原目录顺序循环；浏览排序不改变已选优先级。左右搜索分别查渠道和当前范围模型，跨查询选择保持。</p><p>本页使用已保存模型 mock 的目录和示例能力，不读取真实配置、不调用模型或网络。组件只返回具体模型引用；继承与来源由业务配置页面负责。</p></div></div>';}
 function columns(){return [{key:'name',label:'模型'},{key:'channel',label:'渠道'},{key:'context',label:'上下文'},{key:'effort',label:'推理档位'},{key:'status',label:'状态'},...(recommendedEnabled?[{key:'recommendation',label:'推荐'}]:[])];}
 function tableHead(){const ids=eligible(),count=ids.filter(id=>pending.includes(id)).length;return '<tr>'+(mode==='multiple'?'<th class="mpk-selection-head"><input type="checkbox" id="mpkSelectResults" aria-label="全选当前可选结果" title="全选当前结果中的 '+ids.length+' 个可选模型" '+(ids.length&&count===ids.length?'checked':'')+(ids.length?'':' disabled')+'></th>':'')+columns().map(c=>{const dir=sort.key===c.key?sort.direction:0;return '<th scope="col" aria-sort="'+(dir===1?'ascending':dir===-1?'descending':'none')+'"><button type="button" data-mpk="sort" data-sort="'+c.key+'" aria-label="按'+c.label+'排序" title="点击切换升序、降序、原顺序">'+c.label+'<span class="mpk-table-sort" aria-hidden="true">'+(dir===1?'↑':dir===-1?'↓':'↕')+'</span></button></th>';}).join('')+'</tr>';}
 function rowHTML(m){
  const picked=pending.includes(m.key),canSelect=!m.reason;
  const recommendation=recommendedEnabled&&recommendedRefs.some(r=>keyFor(r)===m.key);
  const input=mode==='multiple'?'<td><input type="checkbox" data-mpk-pick="'+esc(m.key)+'" aria-label="选择 '+esc(m.channelName+' / '+m.rawId)+'" '+(picked?'checked':'')+(!canSelect&&!picked?' disabled':'')+'></td>':'';
  return '<tr class="mpk-row '+(picked?'is-picked ':'')+(!canSelect?'is-disabled':'')+'" data-mpk="'+(mode==='single'?'pick':'toggle')+'" data-key="'+esc(m.key)+'" aria-selected="'+picked+'" '+(!canSelect?'aria-disabled="true"':'')+'>'+input+'<td><button type="button" class="mpk-model-name" data-mpk="'+(mode==='single'?'pick':'toggle')+'" data-key="'+esc(m.key)+'"'+effortAttrs(m,'row')+' aria-label="选择 '+esc(m.channelName+' / '+m.rawId)+'" '+(!canSelect&&!picked?'disabled':'')+'>'+esc(m.name)+(mode==='single'&&picked?'<span class="mpk-current-check"><span aria-hidden="true">✓</span> 已选</span>':'')+'</button><small>'+esc(m.rawId)+(m.alias?'<br>别名：'+esc(m.alias):'')+'</small></td><td>'+esc(m.channelName)+'</td><td>'+ (m.context?m.context.toLocaleString('en-US'):'未知')+'</td><td>'+(reasoningEnabled?'<button type="button" class="mpk-effort-trigger" data-mpk="effort" data-key="'+esc(m.key)+'"'+effortAttrs(m,'row')+' aria-label="选择推理档位 '+esc(m.channelName+' / '+m.rawId)+'" '+(m.reason?'disabled':'')+'>'+'<span class="mpk-effort-label">'+effortName(m.key)+'</span> <span aria-hidden="true">⌄</span></button>':esc(levels[m.effort]||'未知'))+(m.efforts.length?'<small>'+supportedEfforts(m).map(x=>esc(levels[x]||x)).join(' / ')+'</small>':'<small>档位未知</small>')+'</td><td>'+(m.reason?'<span class="mpk-status-warning">'+esc(m.reason)+'</span>':'可选')+'</td>'+(recommendedEnabled?'<td>'+ (recommendation?'推荐':'—')+'</td>':'')+'</tr>';
 }
 /** 目录读取中的骨架：列宽与真实表一致，加载完成不跳版；行内容对辅助技术隐藏。 */
 function skeletonHTML(){
  const cols=columns();
  const skelRow='<tr class="mpk-skeleton-row" aria-hidden="true">'+(mode==='multiple'?'<td><span class="mpk-skeleton mpk-skeleton-box"></span></td>':'')+'<td><span class="mpk-skeleton mpk-skeleton-line" style="width:72%"></span><span class="mpk-skeleton mpk-skeleton-line is-faint" style="width:46%"></span></td><td><span class="mpk-skeleton mpk-skeleton-line" style="width:58%"></span></td><td><span class="mpk-skeleton mpk-skeleton-line" style="width:44%"></span></td><td><span class="mpk-skeleton mpk-skeleton-line" style="width:52%"></span></td><td><span class="mpk-skeleton mpk-skeleton-line" style="width:38%"></span></td>'+(recommendedEnabled?'<td><span class="mpk-skeleton mpk-skeleton-line" style="width:32%"></span></td>':'')+'</tr>';
  return '<div class="mpk-skeleton-status" role="status"><span class="mpk-skeleton-spin" aria-hidden="true"></span><span>正在读取模型目录</span><small>已有选择保持</small>'+button('显示加载完成示例','load-finished')+'</div>'
   +'<table class="mpk-table is-loading" aria-label="渠道模型列表 · 正在读取" aria-busy="true"><colgroup>'+(mode==='multiple'?'<col class="mpk-col-pick">':'')+cols.map(c=>'<col class="mpk-col-'+c.key+'">').join('')+'</colgroup><thead>'+tableHead()+'</thead><tbody>'+Array.from({length:8},()=>skelRow).join('')+'</tbody></table>';
 }
 function listHTML(){
  if(scenario==='loading')return skeletonHTML();
  if(scenario==='failed')return '<div class="mpk-empty" role="status">目录读取失败<br>已有选择保持。'+button('重试读取示例','load-finished')+'</div>';
  const rows=filtered();if(!rows.length)return '<div class="mpk-empty" role="status">'+(catalog.length?'没有匹配的模型<br>清空模型搜索或切换全部渠道。':'目录暂无模型，原选择保持。')+'<div>'+button('清空模型搜索','clear-search')+(channel==='all'?'':button('查看全部','all'))+'</div></div>';
  return '<table class="mpk-table" aria-label="渠道模型列表"><colgroup>'+(mode==='multiple'?'<col class="mpk-col-pick">':'')+columns().map(c=>'<col class="mpk-col-'+c.key+'">').join('')+'</colgroup><thead>'+tableHead()+'</thead><tbody>'+rows.map(rowHTML).join('')+'</tbody></table>';
 }
 function selectedHTML(){
  const ordered=mode==='multiple'&&orderEnabled;
  const rows=!pending.length?'<span class="mpk-picked-empty">尚未选择模型</span>':pending.map((id,i)=>{
   const m=catalog.find(m=>m.key===id),canAdjust=(reasoningEnabled||fastEnabled)&&m&&!m.reason;
   const label='<span class="mpk-chip-check" aria-hidden="true">✓</span><span class="mpk-chip-model">'+(ordered?'<span class="mpk-chip-number">'+(i+1)+'.</span> ':'')+esc(locationFor(id))+'</span>'+(scenario==='loading'?'<span class="mpk-chip-pending">正在核对</span>':'')+(reasoningEnabled?'<span class="mpk-chip-effort '+(effortChoices.get(id)?'is-custom':'')+'">'+esc(effortName(id))+'</span>':'')+(fastEnabled&&effectiveFast(m)?'<span class="mpk-chip-fast">Fast</span>':'')+(scenario==='loading'?'':!m?'<span class="mpk-chip-warning">目录缺失</span>':invalidEffort(id)?'<span class="mpk-chip-warning">档位不支持</span>':'');
   const main=canAdjust?'<button type="button" class="mpk-chip-label" data-mpk="effort" data-key="'+esc(id)+'"'+effortAttrs(m,'picked')+' aria-label="调整推理档位 '+esc(locationFor(id))+'">'+label+'</button>':'<span class="mpk-chip-label">'+label+'</span>';
   const controls=(ordered?button('↑','move','data-key="'+esc(id)+'" data-direction="-1" aria-label="上移 '+esc(locationFor(id))+'" '+(i===0?'disabled':''))+button('↓','move','data-key="'+esc(id)+'" data-direction="1" aria-label="下移 '+esc(locationFor(id))+'" '+(i===pending.length-1?'disabled':'')):'')+(mode==='multiple'?button('×','remove','data-key="'+esc(id)+'" aria-label="移除 '+esc(locationFor(id))+'"'):'');
   return '<div class="mpk-picked-row" data-key="'+esc(id)+'">'+main+(controls?'<span class="mpk-chip-actions">'+controls+'</span>':'')+'</div>';
  }).join('');
  return '<div class="mpk-picked-heading"><span>已选模型 <span class="mpk-picked-count">'+pending.length+'</span></span>'+(mode==='multiple'?button('清空选择','clear-picked',!pending.length?'disabled':''):'')+'</div><div class="mpk-picked-items">'+rows+'</div>';
 }
 const cannotConfirm=()=>mode==='single'&&pending.length!==1||pending.some(id=>!catalog.some(m=>m.key===id)||catalog.find(m=>m.key===id)?.reason||invalidEffort(id)||invalidFast(id));
 function shellHTML(){return '<div class="mpk-root"><div class="mpk-body"><nav class="mpk-channels" aria-label="模型渠道"><div class="mpk-channel-heading">渠道</div><div class="mpk-channel-search"><input class="input" id="mpkChannelSearch" type="text" role="searchbox" inputmode="search" autocomplete="off" aria-label="搜索渠道" placeholder="搜索渠道" value="'+esc(channelQuery)+'"><button type="button" class="mpk-clear-search" id="mpkClearChannels" data-mpk="clear-channels" aria-label="清空渠道搜索" '+(!channelQuery?'hidden':'')+'>'+'<svg viewBox="0 0 16 16" aria-hidden="true"><path d="m4 4 8 8M12 4l-8 8"/></svg>'+'</button></div><div id="mpkChannelRows" class="mpk-channels-list-root"></div></nav><section class="mpk-browser" aria-label="模型浏览"><div class="mpk-scope-line"><strong id="mpkScopeName">模型</strong><span id="mpkResultCount"></span></div><div class="mpk-toolbar"><div class="mpk-search-box"><input class="input" id="mpkSearch" type="text" role="searchbox" inputmode="search" autocomplete="off" placeholder="搜索模型名称 / ID / 别名" aria-label="搜索模型" value="'+esc(query)+'"><button type="button" class="mpk-clear-search" id="mpkClearSearch" data-mpk="clear-search" aria-label="清空模型搜索" '+(!query?'hidden':'')+'>'+'<svg viewBox="0 0 16 16" aria-hidden="true"><path d="m4 4 8 8M12 4l-8 8"/></svg>'+'</button></div></div><div class="mpk-models" id="mpkModels" aria-label="模型表格浏览" tabindex="0"></div></section></div><div class="mpk-picked" id="mpkPicked"></div></div>';}
 function footerHTML(){const visible=new Set(filtered().map(m=>m.key)),outside=pending.filter(id=>!visible.has(id)).length;return '<span class="mpk-summary" role="status">'+pending.length+' 项已选'+(outside?' · 列表外 '+outside+' 项':'')+'</span>'+button('取消','cancel')+'<button type="button" class="btn primary" data-mpk="accept" '+(cannotConfirm()?'disabled':'')+'>确定选择</button>';}
 // Selection edits preserve the table and its focused anchors; browsing still uses renderLists.
 function refreshActions(){
  const batch=$('#mpkSelectResults');if(batch){const ids=eligible(),n=ids.filter(id=>pending.includes(id)).length;batch.checked=ids.length>0&&n===ids.length;batch.indeterminate=n>0&&n<ids.length;}
  if(isOpen){$('#modalConfirm').disabled=cannotConfirm();const visible=new Set(filtered().map(m=>m.key)),outside=pending.filter(id=>!visible.has(id)).length;let summary=$('#mpkFooter');if(!summary){summary=document.createElement('span');summary.id='mpkFooter';summary.className='mpk-summary';$('.modal-actions',modal).prepend(summary);}summary.textContent=pending.length+' 项已选'+(outside?' · 列表外 '+outside+' 项':'');}
  else $('#mpkInlineActions').innerHTML=footerHTML();
 }
 function refreshSelection(){
  if(!active()||!$('#mpkModels'))return;
  const list=$('#mpkModels'),rows=$$('.mpk-row',list),selected=new Set(pending);
  for(const row of rows){
   const id=row.dataset.key,picked=selected.has(id),name=$('.mpk-model-name',row);
   if(row.getAttribute('aria-selected')!==String(picked)){
    row.classList.toggle('is-picked',picked);row.setAttribute('aria-selected',String(picked));
    if(mode==='single'){const badge=$('.mpk-current-check',name);if(picked&&!badge){const mark=document.createElement('span');mark.className='mpk-current-check';mark.innerHTML='<span aria-hidden="true">✓</span> 已选';name.appendChild(mark);}else if(!picked&&badge)badge.remove();}
    else{const input=$('[data-mpk-pick]',row);if(input)input.checked=picked;}
   }
   if(id===effortKey){const label=$('.mpk-effort-label',row),value=effortName(id);if(label&&label.textContent!==value)label.textContent=value;}
  }
  if(sort.key==='effort'){if(!effortPopover.hidden)effortSortPending=true;else sortEffortRows();}
  const picked=$('#mpkPicked'),chips=$$('.mpk-picked-row',picked);
  if(chips.length!==pending.length||chips.some((chip,i)=>chip.dataset.key!==pending[i]))picked.innerHTML=selectedHTML();
  else for(const chip of chips){
   const id=chip.dataset.key,m=catalog.find(m=>m.key===id),label=$('.mpk-chip-label',chip),effort=$('.mpk-chip-effort',chip);if(effort){const value=effortName(id);if(effort.textContent!==value)effort.textContent=value;effort.classList.toggle('is-custom',!!effortChoices.get(id));}
   const fast=$('.mpk-chip-fast',chip);if(fastEnabled&&effectiveFast(m)){if(!fast){const badge=document.createElement('span');badge.className='mpk-chip-fast';badge.textContent='Fast';label.appendChild(badge);}}else fast?.remove();
  }
  refreshActions();if(!effortPopover.hidden)positionEffort();
 }
 function sortEffortRows(){
  if(sort.key!=='effort'||!active())return;const list=$('#mpkModels'),rows=$$('.mpk-row',list),ordered=filtered(),tbody=$('tbody',list);
  if(tbody&&ordered.some((m,i)=>rows[i]?.dataset.key!==m.key)){const scroll=list.scrollTop,byKey=new Map(rows.map(row=>[row.dataset.key,row]));for(const m of ordered){const row=byKey.get(m.key);if(row)tbody.appendChild(row);}list.scrollTop=scroll;}
 }
 function renderLists(resetScroll=false,keepEffort=false){
  const anchorInfo=keepEffort&&effortAnchor?{key:effortAnchor.dataset.effortKey,source:effortAnchor.dataset.effortSource,name:effortAnchor.classList.contains('mpk-model-name')}:null;
  if(!keepEffort)closeEffort();if(!active()||!$('#mpkModels'))return;const list=$('#mpkModels'),nav=$('.mpk-channels-list');browseScroll=resetScroll?0:list.scrollTop;if(nav)channelScroll=nav.scrollTop;
  $('#mpkChannelRows').innerHTML=channelHTML();$('.mpk-channels-list').scrollTop=channelScroll;
  $('#mpkModels').innerHTML=listHTML();$('#mpkModels').scrollTop=browseScroll;
  $('#mpkModels').setAttribute('aria-label','模型表格：'+(channel==='all'?'全部渠道':channels.find(c=>c.id===channel)?.name||'当前渠道'));$('#mpkResultCount').textContent=scenario==='loading'?'读取中':filtered().length+' 个结果';$('#mpkModels').setAttribute('aria-busy',String(scenario==='loading'));
  $('#mpkPicked').innerHTML=selectedHTML();$('#mpkClearSearch').hidden=!query;$('#mpkClearChannels').hidden=!channelQuery;
  refreshActions();
  if(anchorInfo&&!effortPopover.hidden){effortAnchor=$$('[data-effort-key]').find(el=>el.dataset.effortKey===anchorInfo.key&&el.dataset.effortSource===anchorInfo.source&&el.classList.contains('mpk-model-name')===anchorInfo.name);effortAnchor?.setAttribute('aria-expanded','true');positionEffort();}
 }
 function resultHTML(){const r=committed[mode];return '<strong>选择结果</strong><p>'+esc(outcomes[mode])+'</p>'+'<ol>'+(r.models||[]).map(ref=>'<li>'+esc(refLabel(ref))+(ref.reasoningEffort?' <span class="mpk-result-effort">'+esc(levels[ref.reasoningEffort]||ref.reasoningEffort)+'</span>':reasoningEnabled?' <span class="mpk-result-effort">'+esc(levels[defaultEffort(catalog.find(m=>m.key===keyFor(ref)))]||'未知')+'</span>':'')+(ref.fast?' <span class="mpk-result-effort">Fast</span>':'')+'</li>').join('')+'</ol>';}
 function publishResult(){const result=$('#mpkDemoResult');if(result)result.innerHTML=resultHTML();const r=$('#mpkLabResult');if(r)r.innerHTML=resultHTML();}
 function renderPage(reset=true){closeEffort();
  if(document.documentElement.dataset.route!=='model-picker')return;
  if(reset){loadCatalog();resetSelection();}
  page.innerHTML='<div class="mpk-lab-top"><div><h2>选择模型</h2><p>渠道模型选择器</p></div><div class="cv-actions">'+button('返回原页面','back-page')+button('打开弹窗形态','open-dialog')+'</div>'+(!isOpen&&presentation==='inline'?'<div class="mpk-help">'+annotation()+'</div>':'')+'</div>'+(isOpen||presentation==='dialog'?'':'<div class="mpk-inline"><div class="mpk-inline-main">'+shellHTML()+'</div><div class="mpk-inline-actions" id="mpkInlineActions"></div></div>')+'<div class="mpk-lab-result" id="mpkLabResult" role="status"></div>';
  if(!isOpen)renderLists();publishResult();window.prototypePanel?.refresh();
 }
 function launchPage(showDefault=true){if(document.documentElement.dataset.route!=='model-picker')previousRoute=document.documentElement.dataset.route||'overview';if($('#modalMask').style.display==='flex')closeModal();channel='all';query='';channelQuery='';sort={key:null,direction:0};setRoute('model-picker');$('#protoCommon').open=false;renderPage();page.closest('.main').scrollTop=0;if(showDefault&&presentation==='dialog')open();}
 function accept(ids=pending){
  if(businessSession)loadCatalog();
  if(mode==='single'&&ids.length!==1||ids.some(id=>!catalog.some(m=>m.key===id)||catalog.find(m=>m.key===id)?.reason||invalidEffort(id)||invalidFast(id))){if(businessSession)renderLists();return;}
  if(businessSession){const session=businessSession,result=resultFor(ids);if(session.container){session.options.models=clone(result.models);session.options.onSelect?.(clone(result));resetSelection();renderLists();return;}session.accepted=true;closeModal();session.options.onSelect?.(clone(result));return;}
  committed[mode]=resultFor(ids);outcomes[mode]=ids.length?'已返回 '+ids.length+' 个模型引用。':'已返回空列表。';publishResult();if(isOpen)closeModal();else{resetSelection();renderLists();}
 }
 function unmount(){
  if(!businessSession?.container)return;const session=businessSession;closeEffort();session.container.replaceChildren();businessSession=null;
  ({presentation,mode,scenario,orderEnabled,recommendedEnabled,reasoningEnabled,fastEnabled,pending,effortChoices,fastChoices,channel,query,channelQuery,sort}=session.previous);loadCatalog();
 }
 function mount(container,options){
  if(!container?.isConnected||typeof options?.getCatalog!=='function'||typeof options?.onSelect!=='function')return false;
  if(businessSession?.container===container)return true;
  if(isOpen)return false;unmount();closeEffort();page.replaceChildren();
  const previous={presentation,mode,scenario,orderEnabled,recommendedEnabled,reasoningEnabled,fastEnabled,pending:pending.slice(),effortChoices:new Map(effortChoices),fastChoices:new Map(fastChoices),channel,query,channelQuery,sort:clone(sort)};
  businessSession={options,previous,container,route:document.documentElement.dataset.route};mode=options.mode==='multiple'?'multiple':'single';markScenario(options.scenario||'normal');presentation='inline';reasoningEnabled=options.reasoning===true;fastEnabled=options.fast===true;orderEnabled=options.ordered===true;recommendedEnabled=false;
  loadCatalog();resetSelection();channel='all';query=channelQuery='';sort={key:null,direction:0};browseScroll=channelScroll=0;
  container.innerHTML='<div class="mpk-inline"><div class="mpk-inline-main">'+shellHTML()+'</div><div class="mpk-inline-actions" id="mpkInlineActions"></div></div>';renderLists();return true;
 }
 function choose(options){
  unmount();
  if(typeof options?.getCatalog!=='function'||typeof options?.onSelect!=='function')return false;
  if($('#modalMask').style.display==='flex')closeModal();closeEffort();
  const previous={presentation,mode,scenario,orderEnabled,recommendedEnabled,reasoningEnabled,fastEnabled,pending:pending.slice(),effortChoices:new Map(effortChoices),fastChoices:new Map(fastChoices),channel,query,channelQuery,sort:clone(sort)};
  businessSession={options,previous,route:document.documentElement.dataset.route,accepted:false};
  mode=options.mode==='multiple'?'multiple':'single';markScenario(options.scenario||'normal');presentation='dialog';reasoningEnabled=options.reasoning===true;fastEnabled=options.fast===true;orderEnabled=options.ordered===true;recommendedEnabled=false;
  loadCatalog();resetSelection();channel='all';query=channelQuery='';sort={key:null,direction:0};browseScroll=channelScroll=0;opener=document.activeElement;isOpen=true;
  openModal(options.title||'选择模型',shellHTML(),'确定选择',()=>accept(),{hideNote:true,hideConfirm:false,confirmKeepsOpen:true,headActions:'<div class="mpk-help">'+annotation()+'</div>'});
  modal.classList.add('mpk-dialog');document.documentElement.dataset.prototypeDialog='model-picker';
  window.__modalOnClose=()=>{
   const session=businessSession;if(!session)return;closeEffort();businessSession=null;isOpen=false;body.replaceChildren();$('#mpkFooter')?.remove();modal.classList.remove('mpk-dialog');delete document.documentElement.dataset.prototypeDialog;
   ({presentation,mode,scenario,orderEnabled,recommendedEnabled,reasoningEnabled,fastEnabled,pending,effortChoices,fastChoices,channel,query,channelQuery,sort}=session.previous);loadCatalog();
   if(opener?.isConnected)opener.focus({preventScroll:true});
   if(!session.accepted)session.options.onCancel?.({routeChanged:document.documentElement.dataset.route!==session.route});
  };
  renderLists();requestAnimationFrame(()=>$('#mpkSearch')?.focus({preventScroll:true}));return true;
 }
 function open(){closeEffort();if(document.documentElement.dataset.route!=='model-picker'){launchPage(false);}if(isOpen)closeModal();loadCatalog();resetSelection();channel='all';query='';channelQuery='';sort={key:null,direction:0};browseScroll=channelScroll=0;opener=document.activeElement;isOpen=true;renderPage(false);
  openModal('选择模型',shellHTML(),'确定选择',()=>accept(),{hideNote:true,hideConfirm:false,confirmKeepsOpen:true,headActions:'<div class="mpk-help">'+annotation()+'</div>'});modal.classList.add('mpk-dialog');document.documentElement.dataset.prototypeDialog='model-picker';
  window.__modalOnClose=()=>{closeEffort();isOpen=false;body.replaceChildren();$('#mpkFooter')?.remove();modal.classList.remove('mpk-dialog');delete document.documentElement.dataset.prototypeDialog;renderPage();window.prototypePanel?.refresh();if(opener?.isConnected)opener.focus({preventScroll:true});else $('[data-mpk="open-dialog"]')?.focus({preventScroll:true});};renderLists();requestAnimationFrame(()=>$('#mpkSearch')?.focus({preventScroll:true}));
 }
 function updateMode(value){mode=value==='multiple'?'multiple':'single';$$('[data-mpk-mode]').forEach(b=>b.setAttribute('aria-pressed',String(b.dataset.mpkMode===mode)));document.documentElement.dataset.prototypePickerMode=mode;if(isOpen||active()&&presentation==='dialog')open();else renderPage();publishResult();}
 function markScenario(value){scenario=value;if(value==='loading')loadingSince=Date.now();}
 function updateScenario(value){markScenario(value);if(isOpen||active()&&presentation==='dialog')open();else renderPage();}
 function updatePresentation(value){presentation=value==='inline'?'inline':'dialog';$$('[data-mpk-presentation]').forEach(b=>b.setAttribute('aria-pressed',String(b.dataset.mpkPresentation===presentation)));if(isOpen)closeModal();if(active()){if(presentation==='dialog')open();else renderPage();}}
 function toggle(id,checked){const m=catalog.find(m=>m.key===id);if(checked&&m?.reason)return;if(!checked){effortChoices.delete(id);fastChoices.delete(id);}pending=checked?(pending.includes(id)?pending:[...pending,id]):pending.filter(k=>k!==id);renderLists();}
 document.addEventListener('click',event=>{
  if(event.target.matches('input'))return;const t=event.target.closest('[data-mpk]');if(!t||t.disabled)return;const a=t.dataset.mpk,id=t.dataset.id,key=t.dataset.key;
  if(a==='launch'){launchPage();return;}if(a==='back-page'){if(isOpen)closeModal();setRoute(previousRoute==='model-picker'?'overview':previousRoute);return;}if(a==='open-dialog'||a==='reopen'){open();return;}if(!active())return;
  if(a==='cancel'){if(isOpen)closeModal();else{resetSelection();renderLists();}}else if(a==='accept')accept();
  else if(a==='channel'){channel=id;renderLists(true);$$('[data-mpk="channel"]').find(el=>el.dataset.id===id)?.focus({preventScroll:true});}
  else if(a==='all'){channel='all';renderLists(true);}else if(a==='clear-search'){query='';$('#mpkSearch').value='';renderLists(true);$('#mpkSearch').focus();}else if(a==='clear-channels'){channelQuery='';$('#mpkChannelSearch').value='';renderLists();$('#mpkChannelSearch').focus();}
  else if(a==='effort'){showEffort(t,true);}
  else if(a==='pick'){const m=catalog.find(m=>m.key===key);if(m&&!m.reason){if(reasoningEnabled||fastEnabled){const anchor=t.tagName==='BUTTON'?t:$('button[data-mpk=pick]',t);showEffort(anchor,true);effortSelectionIntent=true;effortPinned=true;if(mode==='single')pending=[key];refreshSelection();}else{pending=[key];refreshSelection();}}}
  else if(a==='toggle'){const m=catalog.find(m=>m.key===key);if(m&&(!m.reason||pending.includes(key)))toggle(key,!pending.includes(key));}
  else if(a==='clear-picked'&&mode==='multiple'){pending=[];effortChoices.clear();fastChoices.clear();renderLists();}else if(a==='remove'&&mode==='multiple'){pending=pending.filter(k=>k!==key);effortChoices.delete(key);fastChoices.delete(key);renderLists();}
  else if(a==='move'){const i=pending.indexOf(key),to=i+Number(t.dataset.direction);if(i>=0&&to>=0&&to<pending.length){[pending[i],pending[to]]=[pending[to],pending[i]];renderLists();}}
  else if(a==='sort'){const field=t.dataset.sort;sort=sort.key!==field?{key:field,direction:1}:sort.direction===1?{key:field,direction:-1}:{key:null,direction:0};renderLists(true);$('[data-mpk="sort"][data-sort="'+field+'"]')?.focus({preventScroll:true});}
  else if(a==='reset-sort'){sort={key:null,direction:0};const p=$('.mpk-scope-line');p.querySelector('[data-mpk="reset-sort"]')?.remove();renderLists(true);}
  else if(a==='load-finished'){
   const finish=()=>{markScenario('normal');if(businessSession)businessSession.options.onScenarioReady?.();else $('#mpkDemoScenario').value='normal';loadCatalog();renderLists(true);};
   // 目录回得太快时骨架一闪更晃眼：加载态至少展示 MIN_LOADING_MS 再切到结果。
   const remaining=scenario==='loading'?MIN_LOADING_MS-(Date.now()-loadingSince):0;
   if(remaining>0)setTimeout(()=>{if(scenario==='loading')finish();},remaining);else finish();
  }
 });
 document.addEventListener('input',e=>{if(!active())return;if(e.target.id==='mpkSearch'){query=e.target.value;renderLists(true);}if(e.target.id==='mpkChannelSearch'){channelQuery=e.target.value;renderLists();}});
 document.addEventListener('change',e=>{const el=e.target;if(el.id==='mpkDemoFast'){fastEnabled=el.checked;closeEffort();if(isOpen)open();else renderPage();return;}if(el.id==='mpkDemoReasoning'){reasoningEnabled=el.checked;closeEffort();if(isOpen)open();else renderPage();return;}if(el.id==='mpkDemoScenario'){updateScenario(el.value);return;}if(el.id==='mpkDemoRecommendations'){recommendedEnabled=el.checked;sort={key:null,direction:0};if(isOpen)open();else renderPage();return;}if(el.id==='mpkDemoOrder'){orderEnabled=el.checked;if(active())renderLists();return;}if(!active())return;if(el.dataset.mpkPick){const id=el.dataset.mpkPick;toggle(id,el.checked);$$('[data-mpk-pick]').find(i=>i.dataset.mpkPick===id)?.focus({preventScroll:true});}else if(el.id==='mpkSelectResults'){const ids=eligible();if(!el.checked)ids.forEach(id=>{effortChoices.delete(id);fastChoices.delete(id);});pending=el.checked?[...pending,...ids.filter(id=>!pending.includes(id))]:pending.filter(id=>!ids.includes(id));renderLists();$('#mpkSelectResults')?.focus({preventScroll:true});}});
 document.addEventListener('keydown',e=>{if(!active()||e.key==='Escape')return;if(isOpen&&e.key==='Tab'&&modal.contains(document.activeElement)){const items=$$('button:not([disabled]),input:not([disabled]),[tabindex="0"]',modal).filter(el=>el.getClientRects().length),first=items[0],last=items.at(-1);if(e.shiftKey&&document.activeElement===first){e.preventDefault();last?.focus();}else if(!e.shiftKey&&document.activeElement===last){e.preventDefault();first?.focus();}}
  if(e.key==='ArrowDown'&&e.target.id==='mpkSearch'){const item=$('.mpk-row:not(.is-disabled)');e.preventDefault();if(item)(mode==='multiple'?$('input',item):$('button',item))?.focus();}
  else if(['ArrowDown','ArrowUp','Home','End'].includes(e.key)&&e.target.closest('.mpk-row')){const items=$$('.mpk-row:not(.is-disabled)').map(row=>mode==='multiple'?$('input',row):$('button',row)),i=items.indexOf(document.activeElement);if(i<0)return;e.preventDefault();const n=e.key==='Home'?0:e.key==='End'?items.length-1:Math.max(0,Math.min(items.length-1,i+(e.key==='ArrowDown'?1:-1)));items[n]?.focus();}
 });
 const demo=document.createElement('div');demo.className='proto-card';demo.id='modelPickerDemoCard';demo.dataset.protoContexts='model-picker';demo.innerHTML='<div class="proto-note-title">渠道模型选择器 · 组件打样</div><div class="mpk-demo-field">展示形态</div><div class="mpk-demo-mode"><button type="button" class="btn ghost" data-mpk-presentation="dialog" aria-pressed="true">弹窗（默认）</button><button type="button" class="btn ghost" data-mpk-presentation="inline" aria-pressed="false">页内展示</button></div><div class="mpk-demo-mode"><button type="button" class="btn ghost" data-mpk-mode="single" aria-pressed="true">单选模式</button><button type="button" class="btn ghost" data-mpk-mode="multiple" aria-pressed="false">多选模式</button></div><label class="mpk-demo-field">目录场景<select id="mpkDemoScenario"><option value="normal">完整目录 · 超过 100 项</option><option value="long">长渠道名 / 长模型 ID</option><option value="efforts">推理能力差异</option><option value="empty">目录为空</option><option value="loading">读取中</option><option value="failed">读取失败</option><option value="missing">当前已选引用缺失</option></select></label><label class="mpk-demo-check"><input id="mpkDemoReasoning" type="checkbox" checked>允许选择推理档位</label><label class="mpk-demo-check"><input id="mpkDemoFast" type="checkbox" checked>允许选择快速模式</label><label class="mpk-demo-check"><input id="mpkDemoRecommendations" type="checkbox" checked>提供明确的推荐列表</label><label class="mpk-demo-check"><input id="mpkDemoOrder" type="checkbox" checked>已选列表有优先级</label>'+button('打开弹窗形态','open-dialog')+'<div class="mpk-demo-result" id="mpkDemoResult" role="status"></div><p class="pctl-hint">单选 / 多选由调用方决定，组件内部不展示模式标签。表头只改变浏览排序；本页与业务弹窗使用同一渲染；模型模版 Agent 已在原型中接入，所有数据为 mock。</p>';($('#protoStdCards .proto-group-body')||$('#protoStdCards')).appendChild(demo);
 demo.addEventListener('click',e=>{const p=e.target.closest('[data-mpk-presentation]');if(p)updatePresentation(p.dataset.mpkPresentation);const b=e.target.closest('[data-mpk-mode]');if(b)updateMode(b.dataset.mpkMode);});
 new MutationObserver(()=>{if(businessSession){if(document.documentElement.dataset.route!==businessSession.route){if(businessSession.container)unmount();else closeModal();}return;}if(document.documentElement.dataset.route==='model-picker'){if(!isOpen){renderPage();if(presentation==='dialog')open();}}else if(isOpen)closeModal();}).observe(document.documentElement,{attributes:true,attributeFilter:['data-route']});
 window.modelPickerPrototype={open,choose,mount,unmount,launchPage,updateMode,updateScenario,updatePresentation,state:()=>clone({presentation,mode,scenario,isOpen,channel,query,channelQuery,sort,pending,reasoningEnabled,fastEnabled,fastChoices:Object.fromEntries(fastChoices),effortChoices:Object.fromEntries(effortChoices),committed,catalogSize:catalog.length,visible:filtered().map(m=>({ref:m.ref,name:m.name,reason:m.reason,context:m.context,effort:m.effort,efforts:m.efforts})),businessIntegration:!!businessSession})};
 loadCatalog();publishResult();if(document.documentElement.dataset.route==='model-picker'){renderPage();open();}window.prototypePanel?.refresh();
})();
