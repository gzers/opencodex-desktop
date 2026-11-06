/* L1 dtoVersion=2 / contentVersion=2 view. Declared responses, never a matching engine. */
(function(root,factory){const api=factory(root);if(typeof module==='object'&&module.exports)module.exports=api;else{root.modelConditions023=api;api.mount();}})(typeof window==='undefined'?globalThis:window,function(root){
  'use strict';
  const copy=value=>JSON.parse(JSON.stringify(value));
  const uuid=n=>'00000000-0000-4000-8000-'+String(n).padStart(12,'0');
  const digest='sha256:'+'a'.repeat(64);
  const fields=[['protocol','模型请求协议','protocol'],['contextWindow','上下文窗口','number'],['inputModalities','输入模态','array'],['reasoningEfforts','推理档位','array'],['defaultEffort','默认档位','text'],['fast','Fast','boolean'],['pinnedEffort','强制档位','text'],['effortMap','档位映射','object'],['noReasoning','不传推理参数','boolean'],['reasoningAuthoritative','推理列表权威','boolean'],['fastWire','Fast wire','nullable']];
  const protocols=['openai-chat','openai-responses','anthropic','google','ollama-native'];
  const inherited=()=>Object.fromEntries(fields.map(([key])=>[key,{mode:'inherit'}]));
  function content(family='gpt'){
    return {description:'合成条件编辑场景',matching:{channelMode:'all',channels:[],caseSensitive:false,priority:0,include:[{kind:'prefix',value:family+'-'}],exclude:[],rejectRepeatedLiteralPrefix:true},nameLayouts:[{id:'layout-demo',literalPrefix:family+'-',versionCapture:'complete_numeric_components',numericJoiners:['.','-'],tailCapture:'literal_remainder',trailingDatePolicy:'retain_valid_YYYYMMDD_after_hyphen',ambiguousNumericTail:'no_equivalence'}],commonRules:inherited(),conditionOverrides:[{id:uuid(1),name:family==='claude'?'Opus 5.5':'Sol 6.1',selector:{all:[{kind:'numeric_version',components:family==='claude'?['5','5']:['6','1']},{kind:'literal_segment',value:family==='claude'?'opus':'sol',position:'any',boundary:'hyphen_or_string_edge'}],layoutId:'layout-demo',literalTail:family==='claude'?'-opus':'-sol'},rules:{...inherited(),protocol:{mode:'custom',value:family==='claude'?'anthropic':'openai-responses'}},lockedFields:[],sourceExactId:family==='claude'?'claude-5.5-opus':'gpt-6.1-sol',scope:'known_identity_only'}],lockedFields:[],sources:[]};
  }
  const scenes=[['matched','数字版本 + 名称 · 命中'],['joiner','声明槽位点/连字符等价'],['before','名称在版本前'],['after','名称在版本后'],['tail','精确规格不同 · 不覆盖'],['arity','完整版本段不同'],['conflict','多个覆盖同时命中'],['duplicate','等价条件重复'],['missing','协议待配置'],['unsupported','来源协议暂不支持'],['provenance','来源证据不可确认'],['manual','单模型人工覆盖保留'],['empty','新覆盖项'],['stale','预览晚回/基准变化']];
  function fixture(scene){
    const c=content(scene==='before'?'claude':'gpt'),r=c.conditionOverrides[0];
    let raw='gpt-6.1-sol',view=raw,outcome='matched',reason='数字版本、名称片段与精确尾部均命中（声明响应）';
    if(scene==='joiner')raw=view='gpt-6-1-sol';
    if(scene==='before'){raw=view='claude-opus-5.5';c.nameLayouts[0].literalPrefix='claude-opus-';r.selector.all[1].position='before_version';r.selector.literalTail='';r.sourceExactId=raw;}
    if(scene==='after')r.selector.all[1].position='after_version';
    if(scene==='tail'){raw=view='gpt-6.1-sol-mini';outcome='unmatched';reason='精确剩余文本 -sol-mini 不等于 -sol；保留来源，不扩展能力。';}
    if(scene==='arity'){raw=view='gpt-6.10-sol';outcome='unmatched';reason='完整数字段 [6,10] 与 [6,1] 不同。';}
    if(scene==='conflict'||scene==='duplicate'){c.conditionOverrides.push({...copy(r),id:uuid(2),name:'另一覆盖项'});outcome='conflict';reason=scene==='duplicate'?'model_selector_duplicate：等价身份重复，保存拒绝。':'model_condition_overlap：多个条件同时命中，保留原有效配置。';}
    if(scene==='empty'){c.conditionOverrides=[{...r,name:'',selector:{all:[]},rules:inherited()}];outcome='pending';reason='请填写覆盖项名称并添加条件。';}
    if(['missing','unsupported','provenance'].includes(scene))r.rules.protocol={mode:'inherit'};
    const origin={kind:'source',evidence:{kind:'imported_file',sourceDigest:digest,providerId:'demo-provider',pointer:'/provider/demo/adapter'}};
    const protocol=scene==='missing'?{kind:'missing'}:scene==='unsupported'?{kind:'unsupported',value:'unknown-adapter',origin}: {kind:'resolved',value:scene==='before'?'anthropic':scene==='manual'?'openai-chat':outcome==='matched'?'openai-responses':'openai-chat',origin:scene==='manual'?{kind:'explicit_model'}:outcome==='matched'?{kind:'template_condition',reference:{templateId:uuid(10),revisionId:uuid(11),digest},conditionId:r.id}:origin};
    const result={dtoVersion:2,rawModelId:raw,matchingView:view,matched:outcome==='matched',reason,conditionResults:[{kind:'numeric_version',matched:!['arity','empty'].includes(scene),detail:scene==='arity'?'6.10 ≠ 6.1':'完整版本段：'+r.selector.all[0]?.components?.join('.')},{kind:'literal_segment',matched:scene!=='empty',detail:'完整片段 / 位置：'+(r.selector.all[1]?.position||'未选择')},{kind:'literal_tail',matched:scene!=='tail',detail:'精确剩余文本：'+(r.selector.literalTail??'未声明')}],effective:{channelId:uuid(20),rawModelId:raw,selected:true,enabled:true,observed:true,binding:{kind:scene==='manual'?'manual':'auto_read',reference:{templateId:uuid(10),revisionId:uuid(11),digest}},rules:inherited(),protocol,fieldOrigins:{protocol:protocol.origin||{kind:'unknown'}},matchedConditionId:outcome==='matched'?r.id:null,notices:[]},issues:['conflict','duplicate','provenance','stale'].includes(scene)?[{code:scene==='provenance'?'model_source_provenance_unknown':scene==='stale'?'model_read_stale':scene==='duplicate'?'model_selector_duplicate':'model_condition_overlap',conditionIds:c.conditionOverrides.map(x=>x.id),fieldPath:'conditionOverrides',recoveryActions:scene==='provenance'?['inspect_source','reestablish_source']:['edit_conditions']}]:[]};
    return {content:c,raw,result};
  }
  function create(){
    let f=fixture('matched'),state={content:f.content,raw:f.raw,scene:'matched',generation:1,result:null,dirty:false,revisions:[],base:uuid(11),error:''},fixtureKey=JSON.stringify([f.content,f.raw]);
    const invalidate=()=>{state.generation++;state.result=null;state.dirty=true;};
    function validate(){
      for(const c of state.content.conditionOverrides){
        if(!c.name.trim()||!c.selector.all.length)return '覆盖项名称及至少一条 AND 条件必填。';
        const nums=c.selector.all.filter(a=>a.kind==='numeric_version');
        if(!!c.selector.layoutId!==Object.hasOwn(c.selector,'literalTail')||c.selector.layoutId&&nums.length!==1||!c.selector.layoutId&&nums.length)return 'layoutId 与精确尾部成对；有布局须恰好一个数字版本条件。';
        for(const a of c.selector.all){if(a.kind==='numeric_version'&&(!a.components.length||a.components.some(x=>!/^\d+$/.test(x))))return '数字版本须为完整 ASCII 数字段，不做浮点转换。';if(a.kind==='literal_segment'&&(!a.value.trim()||a.position!=='any'&&!c.selector.layoutId))return '名称片段必填，版本前/后需要声明布局。';if(a.kind==='exact_id'&&!a.value.trim())return '完整模型 ID 必填。';}
      }
      for(const c of state.content.conditionOverrides)for(const [key,label,type] of fields){
        const f=c.rules[key];if(!f||f.mode==='inherit')continue;
        if(key==='protocol'&&!protocols.includes(f.value))return '请选择支持的模型请求协议。';
        if(type==='number'&&(!Number.isSafeInteger(f.value)||f.value<=0))return label+'须为正整数。';
        if(type==='boolean'&&typeof f.value!=='boolean')return label+'须为布尔值。';
        if(type==='array'&&!Array.isArray(f.value))return label+'须为数组。';
      }
      return '';
    }
    return {
      state:()=>copy(state),
      load(scene){if(!scenes.some(row=>row[0]===scene))return;const revisions=state.revisions;f=fixture(scene);state={...state,content:f.content,raw:f.raw,scene,result:null,generation:state.generation+1,dirty:false,error:'',revisions};fixtureKey=JSON.stringify([f.content,f.raw]);},
      edit(fn){fn(state.content,state);invalidate();},
      validate,
      preview(){const request={dtoVersion:2,contentVersion:2,content:copy(state.content),rawModelId:state.raw,draftGeneration:state.generation};state.error=validate();if(state.error){state.result=null;return request;}state.result=JSON.stringify([state.content,state.raw])===fixtureKey?copy(f.result):{dtoVersion:2,rawModelId:state.raw,pending:true,reason:'此编辑尚无预览结果；输入已保留，等待预览服务返回。'};return request;},
      prepareSave(){state.error=validate();if(state.error)return null;if(state.scene==='stale'){state.error='model_save_stale：保存基准已变化，请重新读取比较；当前草稿已保留。';return null;}if(['duplicate','conflict'].includes(state.scene)&&JSON.stringify([state.content,state.raw])===fixtureKey||state.result?.issues?.some(x=>['model_selector_duplicate','model_condition_overlap'].includes(x.code))){state.error='请先处理重复或重叠条件。';return null;}return {expectedGeneration:state.generation,baseRevision:state.base,content:copy(state.content),contentVersion:2};},
      save(plan){if(!plan||plan.expectedGeneration!==state.generation){state.error='model_save_stale：当前草稿已变，请重新预览。';return false;}const revision={id:uuid(100+state.revisions.length),contentVersion:2,content:copy(plan.content),baseRevision:state.base};state.revisions.push(revision);state.base=revision.id;state.dirty=false;return true;},
      staleResult(request){return request.draftGeneration!==state.generation;}
    };
  }
  const controllers=new Map();let active=null,scope='gpt',selected=0;
  function markup(id){return '<h3>条件覆盖</h3><p>数字版本、名称片段组合为 AND；预览显示逐条件与字段来源。新条件使用 contentVersion=2，旧字面后缀不自动转换。</p><button class="btn primary" type="button" data-o23-condition="'+String(id).replace(/[^\w-]/g,'')+'">编辑条件与预览</button>';}
  function render(){
    if(!active)return;const u=root.OCXD023,s=active.state(),c=s.content,entry=c.conditionOverrides[selected]||c.conditionOverrides[0],e=u.esc;
    let body='<div class="mp-toolbar">'+'<span class="tag">'+(s.dirty?'未保存修改':'草稿 / 已保存版本独立')+'</span></div><p class="mp-muted">编辑只改变当前草稿；保存为新版本，已保存历史保持。</p><div class="mp-actions">'+c.conditionOverrides.map((row,i)=>u.button(row.name||'未命名覆盖项','condition-pick-'+i)).join('')+u.button('新增覆盖项','condition-add')+'</div>'+
      '<div class="mp-template-form"><label>覆盖项名称<input class="input" aria-label="覆盖项名称" data-c23="name" value="'+e(entry.name)+'"></label><label>来源精确 ID<input class="input" data-c23="sourceExactId" value="'+e(entry.sourceExactId)+'"></label><label>名称结构<select class="mp-select" aria-label="名称结构" data-c23="layout">'+u.options([['','无数字结构'],...c.nameLayouts.map(l=>[l.id,l.literalPrefix+'〈完整数字版本〉〈精确剩余文本〉'])],entry.selector.layoutId||'')+'</select></label><label>精确剩余文本 / 规格<input class="input" aria-label="精确剩余文本" data-c23="tail" value="'+e(entry.selector.literalTail??'')+'" '+(!entry.selector.layoutId?'disabled':'')+'></label></div><details><summary>名称结构声明</summary>'+c.nameLayouts.map((l,i)=>'<label>结构前缀<input class="input" data-c23-layout="'+i+'" value="'+e(l.literalPrefix)+'"></label>').join('')+'<p>仅声明数字槽位内 . / - 等价；完整字符串数字段、日期、前导零及歧义数字尾不自动折叠。</p></details><h3>以下条件全部满足（AND）</h3>'+
      entry.selector.all.map((a,i)=>'<div class="mp-template-form"><label>条件类型<select class="mp-select" aria-label="条件类型 '+(i+1)+'" data-c23-atom="'+i+'" data-part="kind">'+u.options([['numeric_version','数字版本'],['literal_segment','名称片段'],['exact_id','完整模型 ID']],a.kind)+'</select></label><label>条件值<input class="input" aria-label="条件值 '+(i+1)+'" data-c23-atom="'+i+'" data-part="value" value="'+e(a.kind==='numeric_version'?a.components.join('.'):a.value)+'"></label>'+(a.kind==='literal_segment'?'<label>名称位置<select class="mp-select" aria-label="名称位置" data-c23-atom="'+i+'" data-part="position">'+u.options([['any','不限'],['before_version','版本前'],['after_version','版本后']],a.position)+'</select></label>':'')+u.button('删除此条件','condition-remove-'+i)+'</div>').join('')+
      '<div class="mp-actions">'+u.button('添加 AND 条件','condition-atom')+u.button('删除覆盖项','condition-delete',c.conditionOverrides.length===1)+'</div><h3>覆盖字段</h3><p class="mp-muted">未自定义字段继承；锁定只保护模版/Agent编辑，不等于模型人工覆盖。集合字段使用 JSON 数组，Fast wire 可明确 null。</p>'+fields.map(([key,label,type])=>{
        const field=entry.rules[key]||{mode:'inherit'},custom=field.mode==='custom';
        return '<details'+(key==='protocol'?' open':'')+'><summary>'+label+' · '+(custom?'自定义':'继承')+'</summary><div class="mp-template-form"><label>设置方式<select class="mp-select" aria-label="'+label+'设置方式" data-c23-rule="'+key+'" data-part="mode">'+u.options([['inherit','继承'],['custom','自定义']],field.mode)+'</select></label>'+(custom?'<label>'+label+(type==='protocol'?'<select class="mp-select" aria-label="模型请求协议" data-c23-rule="'+key+'" data-part="value">'+u.options([['','请选择协议'],...protocols.map(p=>[p,p])],field.value||'')+'</select>':'<input class="input" aria-label="'+label+'值" data-c23-rule="'+key+'" data-part="value" value="'+e(typeof field.value==='object'?JSON.stringify(field.value):field.value)+'">')+'</label>':'')+'<label><input type="checkbox" data-c23-lock="'+key+'"'+(entry.lockedFields.includes(key)?' checked':'')+'>锁定此模版字段</label></div></details>';
      }).join('')+'<h3>可解释预览</h3><label>原始模型 ID<input class="input" aria-label="预览原始模型 ID" data-c23="raw" value="'+e(s.raw)+'"></label><p>原始请求 ID 保持；匹配视图另列。</p><div class="mp-actions">'+u.button('预览匹配','condition-preview',false,true)+u.button('查看预览内容','condition-request')+u.button('查看已保存版本','condition-history')+'</div><div id="c23Result" aria-live="polite">'+resultHTML(s.result)+'</div><p id="c23Error" class="mp-alert" role="alert">'+e(s.error)+'</p>';
    u.dialog('条件覆盖 · '+scope,body,'保存新版本',()=>{
      const plan=active.prepareSave();if(!plan){showError();return;}
      u.dialog('保存条件覆盖新版本',u.facts([['来源版本',plan.baseRevision],['内容版本','2'],['覆盖项',String(plan.content.conditionOverrides.length)]])+'<p>确认仅更新本页内存库，旧 revision 保持；不后台改渠道配置。</p>','确认保存',()=>{if(active.save(plan)){render();}else showError();},{onExtra:()=>render(),extraLabel:'返回编辑'});
    },{onExtra:()=>{active.load('empty');selected=0;render();},extraLabel:'放弃当前草稿'});
  }
  function resultHTML(r){const u=root.OCXD023;if(!r)return '<p class="mp-muted">尚未预览，或输入已改变；旧命中结果已失效。</p>';if(r.pending)return '<p role="status">'+u.esc(r.reason)+'</p>';
    return '<p role="status">'+u.esc(r.reason)+'</p>'+u.facts([['原始 ID',r.rawModelId],['匹配视图',r.matchingView],...(r.conditionResults||[]).map(a=>[a.kind,(a.matched?'命中 · ':'未命中 · ')+a.detail])])+ '<h3>协议与字段来源</h3>'+u.facts([['实际导入事实','公开字段证据 /provider/demo/adapter（合成）'],['模版通用','继承来源'],['条件覆盖',r.matched?'明确覆盖':'无覆盖'],['单模型人工',r.effective.protocol.origin?.kind==='explicit_model'?'明确自定义 · 保留':'无'],['最终协议',r.effective.protocol.kind==='resolved'?r.effective.protocol.value:r.effective.protocol.kind==='missing'?'待配置':r.effective.protocol.value+' · 不支持']])+ '<details><summary>条件标识、精确版本与字段来源</summary><pre class="mp-code">'+u.esc(JSON.stringify(r.effective,null,2))+'</pre></details>'+(r.issues?.length?'<p class="mp-alert" role="alert">'+u.esc(r.issues.map(x=>x.code).join(' / '))+'</p>':'');
  }
  function showError(){const el=root.document.getElementById('c23Error');if(el)el.textContent=active.state().error;}
  function refreshResult(){const el=root.document.getElementById('c23Result');if(el)el.innerHTML=resultHTML(null);showError();}
  function open(id='gpt'){scope=id;if(!controllers.has(id))controllers.set(id,create());active=controllers.get(id);selected=0;render();}
  function mount(){
    const rail=root.document.getElementById('protoStdCards');if(rail){const tools=root.document.createElement('section');tools.className='proto-card';tools.dataset.protoContexts='models';tools.innerHTML='<div class="proto-note-title">0.2.23 · 条件声明响应</div>'+root.OCXD023.select('声明响应场景','condition-scene',scenes,'matched');rail.append(tools);}
    root.document.addEventListener('click',event=>{
      const target=event.target.closest('[data-o23-condition],[data-o23]');if(!target||target.disabled)return;
      if(target.dataset.o23Condition){open(target.dataset.o23Condition);return;}if(!active)return;
      const a=target.dataset.o23||'';
      if(a.startsWith('condition-pick-')){selected=Number(a.split('-').at(-1));render();}
      if(a==='condition-add'){active.edit(c=>c.conditionOverrides.push({id:uuid(1000+c.conditionOverrides.length),name:'',selector:{all:[]},rules:inherited(),lockedFields:[],sourceExactId:'',scope:'known_identity_only'}));selected=active.state().content.conditionOverrides.length-1;render();}
      if(a==='condition-delete'){active.edit(c=>c.conditionOverrides.splice(selected,1));selected=0;render();}
      if(a==='condition-atom'){active.edit(c=>c.conditionOverrides[selected].selector.all.push({kind:'literal_segment',value:'',position:'any',boundary:'hyphen_or_string_edge'}));render();}
      if(a.startsWith('condition-remove-')){active.edit(c=>c.conditionOverrides[selected].selector.all.splice(Number(a.split('-').at(-1)),1));render();}
      if(a==='condition-preview'){active.preview();const el=root.document.getElementById('c23Result');el.innerHTML=resultHTML(active.state().result);showError();}
      if(a==='condition-request'){const s=active.state();root.OCXD023.dialog('本次预览内容', '<pre class="mp-code">'+root.OCXD023.esc(JSON.stringify({dtoVersion:2,contentVersion:2,content:s.content,rawModelId:s.raw,draftGeneration:s.generation},null,2))+'</pre>','返回编辑',render);}
      if(a==='condition-history'){const s=active.state();root.OCXD023.dialog('不可变保存版本',s.revisions.length?s.revisions.map(v=>root.OCXD023.facts([['revision',v.id],['contentVersion',v.contentVersion],['基于版本',v.baseRevision]])).join(''):'<p>本页尚未模拟保存新 revision；旧原件与本轮草稿保持。</p>','返回编辑',render);}
    });
    root.document.addEventListener('input',event=>update(event,false));
    root.document.addEventListener('change',event=>{
      if(event.target.dataset.o23Select==='condition-scene'){active.load(event.target.value);selected=0;render();return;}
      if(event.target.tagName==='SELECT'||event.target.type==='checkbox')update(event,true);
    });
  }
  function update(event,redraw){
    if(!active||!event.target.closest('[data-o23-dialog]'))return;const el=event.target;
    if(!Object.keys(el.dataset).some(k=>k.startsWith('c23')))return;
    try{active.edit((c,s)=>{const r=c.conditionOverrides[selected];
      if(el.dataset.c23){const key=el.dataset.c23;if(key==='raw')s.raw=el.value;else if(key==='name'||key==='sourceExactId')r[key]=el.value;else if(key==='layout'){if(el.value){r.selector.layoutId=el.value;r.selector.literalTail='';}else{delete r.selector.layoutId;delete r.selector.literalTail;}}else r.selector.literalTail=el.value;}
      if(el.dataset.c23Layout!==undefined)c.nameLayouts[Number(el.dataset.c23Layout)].literalPrefix=el.value;
      if(el.dataset.c23Atom!==undefined){const i=Number(el.dataset.c23Atom),old=r.selector.all[i],part=el.dataset.part;if(part==='kind')r.selector.all[i]=el.value==='numeric_version'?{kind:el.value,components:[]}:el.value==='exact_id'?{kind:el.value,value:''}:{kind:el.value,value:'',position:'any',boundary:'hyphen_or_string_edge'};else if(part==='position')old.position=el.value;else if(old.kind==='numeric_version')old.components=el.value.split(/[.-]/);else old.value=el.value;}
      if(el.dataset.c23Rule){const key=el.dataset.c23Rule,type=fields.find(x=>x[0]===key)[2];if(el.dataset.part==='mode')r.rules[key]=el.value==='inherit'?{mode:'inherit'}:{mode:'custom',value:type==='boolean'?false:type==='number'?1:type==='array'?[]:type==='object'?{}:type==='nullable'?null:''};else{let value=el.value;if(type==='number')value=Number(value);if(['array','object','nullable','boolean'].includes(type))value=JSON.parse(value);r.rules[key]={mode:'custom',value};}}
      if(el.dataset.c23Lock){const key=el.dataset.c23Lock;r.lockedFields=el.checked?[...new Set([...r.lockedFields,key])]:r.lockedFields.filter(x=>x!==key);}
    });if(redraw)render();else refreshResult();}catch{const error=root.document.getElementById('c23Error');if(error)error.textContent='字段格式无效，请填写对应类型的值；原草稿未清空。';}
  }
  return {create,fixture,fields,scenes,markup,open,mount};
});
