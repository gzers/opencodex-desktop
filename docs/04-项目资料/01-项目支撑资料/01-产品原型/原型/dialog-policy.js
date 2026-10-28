/* 全局对话策略入口与上下文原型：内存状态，不操作真实 Codex。 */
(() => {
 'use strict';
 const $=(q,r=document)=>r.querySelector(q),$$=(q,r=document)=>[...r.querySelectorAll(q)];
 const esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 const page=$('#settings-dialog-policy'),host=$('#contextPolicyContent');
 if(!page||!host||!window.PrototypeContextPolicy)return;
 const store=window.PrototypeContextPolicy.create();
 let part='models',message='',formError='';
 const choices=(entries,value)=>entries.map(([id,label])=>'<option value="'+esc(id)+'" '+(id===value?'selected':'')+'>'+esc(label)+'</option>').join('');
 const button=(label,action,attrs='')=>'<button type="button" class="btn ghost btn-md" data-cp="'+action+'" '+attrs+'>'+label+'</button>';
 const descriptions={
  inherit:'保留 Codex 与当前 Provider 的既有行为，不额外指定上下文策略。',
  regular:'达到自动压缩阈值后整理上下文；压缩路径由下方 Provider 设置决定。',
  background:'在上下文接近阈值前准备压缩；轮次结束压缩单独设置。',
  notes:'按需查阅历史并维护模型笔记，仍保留执行端的兜底压缩。'
 };
 function annotation(){return '<div class="proto-annotation" id="ann-context-policy" data-annotation-number="03" data-annotation-title="上下文策略与 Superba 参考" data-annotation-category="渠道能力 · 版本未定"><button type="button" class="annotation-marker" aria-label="原型注释 03：上下文策略与 Superba 参考" aria-expanded="false" aria-controls="annotationPopover"><span>原型</span><span>03</span><span>＋</span></button><div class="annotation-source" hidden><p>参考 Superba v0.1.1-alpha.12 的 Provider 压缩三态与通用配置编辑。本页四种策略为本项目设计的预设，不代表 Superba 已有同款组合界面，也不把特定供应商的限制推广到全部渠道。</p><p>策略属于当前 Codex 实例；压缩协议按用户级 Provider 保存，不写入项目或对话覆盖。只演示内存草稿与编译目标；保存不重启、不请求压缩或读写历史。后台压缩与轮末压缩是不同机制。</p><p>当前 OpenCodex 第三方代理摘要不等于原生远程压缩；history/notes 入口仍待接通。配置已保存、重启后应用、对话实际观察分别展示。</p></div></div>';}
 function status(){const s=store.snapshot();return s.dirty?'有未保存修改':s.pending?'已保存 · 待重启 Codex':message||'沿用已保存设置';}
 function compatibility(s){
  const p=s.draft.providers[s.selected].protocol,proxy=s.selected==='opencodex',notes=s.draft.mode==='notes';
  if(s.draft.mode==='background'&&p==='local')return '后台压缩需要远程 v2 协议；当前选择为客户端流程，后台功能不会启用。';
  if(s.draft.mode==='background'&&p==='inherit')return '后台压缩需要远程 v2 协议；是否满足取决于此 Provider 的有效默认。';
  if(s.draft.mode==='background'&&proxy&&p==='v2')return '客户端可按远程 v2 调度后台整理；当前第三方上游仍由代理生成摘要。';
  if(proxy&&notes)return '当前 OpenCodex 第三方路径尚未接通历史与笔记；保存后仍需执行端支持。';
  if(proxy&&p==='v2')return '当前 OpenCodex 第三方路径仍生成代理摘要，尚不能执行原生远程压缩。';
  if(proxy)return '当前第三方路径使用代理摘要；实际处理方式请在具体对话中查看。';
  if(notes)return '历史与笔记需要支持的模型与 ChatGPT 认证；自定义 API Key 路径不能据此视为可用。';
  if(p==='v2')return '需要 Provider 支持原生远程压缩；选择此项只声明偏好，实际采用以对话记录为准。';
  if(p==='local')return '由客户端组织压缩请求；压缩仍可能使用远端模型。';
  return '保留此 Provider 的既有默认，不根据连接测试推断压缩能力。';
 }
 function render(){
  const opened=$$('.cp-advanced[open]',host).map(e=>$('summary',e)?.textContent);
  const s=store.snapshot(),d=s.draft;
  const numeric=(id,label,key,hint,placeholder)=>'<div class="cp-row"><div><label for="'+id+'">'+label+'</label><small>'+hint+'</small></div><input class="input" id="'+id+'" data-cp-field="'+key+'" type="text" inputmode="numeric" aria-label="'+label+'" placeholder="'+placeholder+'" value="'+esc(d[key])+'"></div>';
  host.innerHTML='<section class="cp-section"><div class="tp-heading"><div><h3>上下文</h3><p class="cp-scope">作用于当前 Codex 实例，项目和对话沿用此策略。</p></div>'+annotation()+'</div>'+
   '<div class="cp-row"><div><label for="cpMode">处理策略</label><small>决定何时整理上下文、是否使用历史与笔记。</small></div><select id="cpMode" aria-label="上下文处理策略">'+choices(Object.entries(s.modes),d.mode)+'</select></div><p class="cp-summary" id="cpModeSummary">'+descriptions[d.mode]+'</p>'+
   '<details class="cp-advanced"><summary>高级上下文设置</summary>'+numeric('cpAutoTokens','自动压缩阈值','autoTokens','Token 数；留空使用模型与客户端的默认阈值。','跟随模型默认')+(d.mode==='background'?numeric('cpBackgroundPercent','后台启动比例','backgroundPercent','占模型完整上下文窗口的比例，1–100%；留空跟随客户端。','跟随客户端'):'')+(d.mode!=='notes'?numeric('cpPostTurnPercent','轮次结束压缩比例','postTurnPercent','占上下文窗口的比例，0 关闭；留空不额外覆盖。','跟随客户端'):'')+'</details>'+
   '<section class="cp-group" aria-label="Provider 压缩路径"><h4>压缩路径</h4><p class="cp-summary">按 Provider 配置，与上方处理时机分别管理。</p><div class="cp-row"><label for="cpProvider">配置对象</label><select id="cpProvider" aria-label="压缩配置对象">'+choices(s.targets.map(t=>[t.id,t.name+' · '+t.id]),s.selected)+'</select></div><div class="cp-row"><div><label for="cpProtocol">压缩方式</label><small>选择客户端流程或 Provider 原生远程压缩。</small></div><select id="cpProtocol" aria-label="压缩方式">'+choices(Object.entries(s.protocols),d.providers[s.selected].protocol)+'</select></div><p class="cp-compatibility '+(s.selected==='opencodex'&&(d.mode==='notes'||d.providers[s.selected].protocol==='v2')?'warning':'')+'">'+esc(compatibility(s))+'</p></section>'+
   '<div class="cp-error" id="cpFormError" role="alert">'+esc(formError)+'</div><div class="tp-actions"><span class="cp-status '+(s.pending?'pending':'')+'" id="cpFormState" role="status">'+esc(status())+'</span>'+button('还原修改','discard',s.dirty?'':'disabled')+'<button type="button" class="btn primary btn-md" data-cp="save" '+(s.dirty?'':'disabled')+'>保存上下文配置</button></div>'+
   (s.pending?'<p class="cp-summary">完全退出并重新打开 Codex 后应用；正在进行的对话不会因保存而切换策略。</p>':'')+
   '<details class="cp-advanced"><summary>查看配置归属</summary><p>策略与阈值保存在用户级 Codex 配置；压缩方式保存在各自 Provider 下。恢复“跟随默认”后移除本页对应的显式覆盖。</p><pre class="cp-preview" id="cpPreview">'+esc(JSON.stringify(store.preview(),null,2))+'</pre></details></section>';
  for(const el of $$('.cp-advanced',host))el.open=opened.includes($('summary',el)?.textContent);
  window.prototypeSelect?.enhance(host);window.prototypePanel?.refresh();
 }
 function update(){const s=store.snapshot();$('#cpFormState',host).textContent=status();$('#cpFormState',host).classList.toggle('pending',s.pending);$('#cpFormError',host).textContent=formError;$('[data-cp=save]',host).disabled=!s.dirty;$('[data-cp=discard]',host).disabled=!s.dirty;$('#cpPreview',host).textContent=JSON.stringify(store.preview(),null,2);}
 function show(next){
  if(['models','context','connection'].includes(next))part=next;
  for(const el of $$('[data-dp-section]',page)){const active=el.dataset.dpSection===part;el.classList.toggle('active',active);el.setAttribute('aria-current',active?'page':'false');el.tabIndex=active?0:-1;}
  for(const el of $$('[data-dp-panel]',page))el.classList.toggle('active',el.dataset.dpPanel===part);
  document.documentElement.dataset.dialogPolicyPart=part;
  window.prototypePanel?.refresh();
 }
 function open(next='models'){
  window.openSettingsSection?.('dialog-policy');show(next);
  const params=new URLSearchParams({section:'dialog-policy',policy:part});
  history.replaceState(null,'','#settings?'+params.toString());
  $('.main')?.scrollTo({top:0});
 }
 function refreshObservations(){
  const fact=(label,value)=>'<div><dt>'+label+'</dt><dd>'+esc(value)+'</dd></div>';
  for(const slot of $$('[data-context-policy-observation]')){
   const key=slot.dataset.contextObject,remote=slot.dataset.contextSource==='dev',observation=!remote&&key?.startsWith('thread:')?store.observation(key):null;
   const reason=remote?'该来源未连接，不采用本机观察结果。':key?.startsWith('project:')?'请在具体对话中查看上下文处理记录。':'尚无此对话的上下文处理记录。';
   slot.innerHTML='<section class="cp-observation"><h4>上下文处理</h4><dl>'+(observation?fact('采用策略',observation.strategy)+fact('压缩与整理',observation.processor)+fact('历史读取',observation.history)+fact('模型笔记',observation.notes):fact('采用策略','未观察')+fact('压缩与整理','未观察')+fact('历史读取','未观察')+fact('模型笔记','未观察'))+'</dl><p class="cp-summary">'+esc(observation?.reason||reason)+'</p>'+button('管理全局上下文策略','open-context')+'</section>';
  }
 }
 function restart(){store.restart();message='已重新加载上下文配置 · 实际采用待观察';render();refreshObservations();}
 page.addEventListener('click',e=>{const b=e.target.closest?.('[data-dp-section]');if(!b)return;show(b.dataset.dpSection);history.replaceState(null,'','#settings?'+new URLSearchParams({section:'dialog-policy',policy:part}));});
 $('.dp-sections',page).addEventListener('keydown',e=>{if(!['ArrowLeft','ArrowRight','Home','End'].includes(e.key))return;const buttons=$$('[data-dp-section]',page),i=buttons.findIndex(b=>b===document.activeElement);if(i<0)return;e.preventDefault();const next=e.key==='Home'?0:e.key==='End'?buttons.length-1:(i+(e.key==='ArrowRight'?1:-1)+buttons.length)%buttons.length;buttons[next].click();buttons[next].focus();});
 host.addEventListener('change',e=>{
  if(e.target.id==='cpProvider'){store.select(e.target.value);render();return;}
  const field=e.target.id==='cpMode'?'mode':e.target.id==='cpProtocol'?'protocol':null;if(!field)return;store.edit(field,e.target.value);message=formError='';render();$('#'+e.target.id,host)?.focus({preventScroll:true});
 });
 host.addEventListener('input',e=>{const field=e.target.dataset.cpField;if(!field)return;store.edit(field,e.target.value);message='';formError=store.error();update();});
 document.addEventListener('click',e=>{
  const b=e.target.closest?.('[data-cp]');if(!b||b.disabled)return;const action=b.dataset.cp;
  if(action==='open-context'){open('context');return;}
  if(action==='save'){const result=store.save();formError=result.ok?'':result.error;message=result.ok?'上下文配置已保存':'';render();}
  else if(action==='discard'){store.discard();message=formError='';render();}
  else if(action==='restart'){window.transportPrototype?.restart();}
  else if(action==='observe'){store.observe(b.dataset.scene);refreshObservations();}
 });
 window.addEventListener('prototype:codex-restart',restart);
 const demo=document.createElement('div');demo.id='contextPolicyDemoCard';demo.className='proto-card';demo.dataset.protoContexts='settings:dialog-policy conversations';demo.dataset.policyParts='context';
 demo.innerHTML='<div class="proto-note-title">上下文 · 版本未定</div><p class="pctl-hint">固定内存示例；不执行真实压缩、历史读取或笔记写入。</p><div class="pctl-grid">'+button('模拟 Codex 完整重启','restart')+'</div><label class="mpk-demo-check"><input type="checkbox" id="cpSaveFailure">模拟保存失败</label><div class="mpk-demo-field">请求观察（仅“模型配置与对话边界”）</div><div class="pctl-grid">'+[['unknown','未观察'],['summary','代理摘要'],['native','原生远程压缩'],['notes','历史与笔记'],['unavailable','当前链路未接通']].map(([id,label])=>button(label,'observe','data-scene="'+id+'"')).join('')+'</div><p class="pctl-hint">原生与笔记为直连示例；不证明 OpenCodex 第三方路径已经支持。重启只应用已保存配置，并清空旧观察。</p>';
 ($('#protoStdCards .proto-group-body')||$('#protoStdCards'))?.appendChild(demo);
 demo.addEventListener('change',e=>{if(e.target.id==='cpSaveFailure'){store.setSaveFailure(e.target.checked);render();}});
 function fromHash(){const p=new URLSearchParams(location.hash.split('?')[1]||'');if(location.hash.startsWith('#settings')&&['dialog-policy','transport'].includes(p.get('section'))){window.showSettingsSection?.(p.get('section'));show(p.get('section')==='transport'?'connection':p.get('policy')||'models');}}
 window.addEventListener('hashchange',fromHash);
 window.dialogPolicyPrototype={show,open,state:()=>({part})};
 window.contextPolicyPrototype={state:store.snapshot,render,refreshObservations};
 window.openDialogPolicy=open;
 render();show('models');refreshObservations();fromHash();
})();
