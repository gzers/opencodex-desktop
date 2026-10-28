/* COL-LOCAL-20261016-01：共用状态只驻留内存；不读写运行配置或执行网络测试。 */
(() => {
 'use strict';
 const $=(s,r=document)=>r.querySelector(s),$$=(s,r=document)=>[...r.querySelectorAll(s)];
 const esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 const host=$('#transportContent');if(!host||!window.PrototypeTransportState)return;
 const store=window.PrototypeTransportState.create();let message='',formError='';
 const button=(text,action,attrs='')=>'<button type="button" class="btn ghost btn-md" data-tp="'+action+'" '+attrs+'>'+text+'</button>';
 const options=(entries,value)=>entries.map(([id,label])=>'<option value="'+esc(id)+'" '+(id===value?'selected':'')+'>'+esc(label)+'</option>').join('');
 function annotation(){return '<div class="proto-annotation" id="ann-transport-global" data-annotation-number="02" data-annotation-title="连接配置与实际传输的区别" data-annotation-category="渠道能力 · 版本未定"><button type="button" class="annotation-marker" aria-label="原型注释 02：连接配置与实际传输的区别" aria-expanded="false" aria-controls="annotationPopover"><span>原型</span><span>02</span><span>＋</span></button><div class="annotation-source" hidden><p>本专题归渠道特殊能力与代理兼容包，版本和生产开发时机待定。原型只保存内存，测试连接、重启和请求观察均为右侧工具驱动的虚构状态，不读写真实文件、不发网络请求或执行命令。配置对象是用户级 Codex provider，不是某个项目或对话。</p><p>OpenCodex 本地代理偏好统一映射本地 websockets 与 Codex supports_websockets 声明，避免两个重复开关；直连只映射对应 provider。跟随默认不创建显式覆盖。当前安装包 2.84.0 对任意第三方上游保留 HTTP/SSE；示例官方上游状态不作为第三方已经打通的证据。保存与模拟重启都不能产生实际传输观察。</p></div></div>';}
 function formState(){const s=store.snapshot(),id=s.selected;return s.dirty.includes(id)?'有未保存修改':s.pending.includes(id)?'已保存 · 待重启 Codex':message||'已保存';}
 function renderForm(){
  const openSections=$$('.tp-details[open]',host).map(el=>$('summary',el)?.textContent);
  const s=store.snapshot(),d=s.draft[s.selected],target=s.targets.find(t=>t.id===s.selected),applied=s.applied[s.selected];
  const configured=s.pending.includes(s.selected)?'待重启':s.mode==='link'&&target.proxy?'Link 使用 HTTP':applied.policy==='prefer'?'允许 WebSocket · 实际使用待观察':applied.policy==='http'?'仅 HTTP':'跟随已有默认 · 实际使用待观察';
  const preview=store.preview(),keys=Object.entries(preview).filter(([key])=>!['provider','websockets'].includes(key)).map(([key,value])=>key+' = '+value);
  host.innerHTML='<div class="tp-heading"><div><h2>连接与传输</h2><p>用户级连接偏好，按 Provider 保存，对使用它的项目与对话生效。</p></div>'+annotation()+'</div><div class="tp-list"><div class="tp-row"><label for="tpProvider" class="tp-label">配置对象</label><select id="tpProvider" aria-label="连接配置对象">'+options(s.targets.map(t=>[t.id,t.name+' · '+t.id]),s.selected)+'</select></div><div class="tp-row"><div><label for="tpPolicy" class="tp-label">传输偏好</label><p class="tp-muted">支持时优先使用 WebSocket，必要时使用 HTTP。</p></div><select id="tpPolicy" aria-label="传输偏好">'+options(Object.entries(s.policies),d.policy)+'</select></div></div><details class="tp-details"><summary>高级连接设置</summary><div class="tp-row"><div><label for="tpTimeout" class="tp-label">WebSocket 连接超时</label><p class="tp-muted">毫秒，留空沿用客户端设置。</p></div><input class="input" id="tpTimeout" type="text" inputmode="numeric" aria-label="WebSocket 连接超时" placeholder="跟随客户端" value="'+esc(d.timeout)+'"></div></details><div id="tpFormError" class="tp-error" role="alert">'+esc(formError)+'</div><div class="tp-actions"><span id="tpFormState" class="tp-status '+(s.pending.includes(s.selected)?'pending':'')+'" role="status">'+esc(formState())+'</span>'+button('还原修改','discard',s.dirty.includes(s.selected)?'':'disabled')+'<button type="button" class="btn primary btn-md" data-tp="save" '+(!s.dirty.includes(s.selected)?'disabled':'')+'>保存连接配置</button></div><section class="tp-section" aria-label="当前连接状态"><h3>当前连接状态</h3><dl class="tp-facts"><div><dt>作用范围</dt><dd>'+esc(target.name)+' · 所有使用此 Provider 的对话</dd></div><div><dt>连接配置</dt><dd>'+esc(configured)+'</dd></div></dl>'+(s.pending.includes(s.selected)?'<p class="tp-muted">完全退出并重新打开 Codex 后应用；保存不会切换正在进行的连接。</p>':'')+(target.proxy?'<p class="tp-muted">本地入口与上游连接分别判断。入口允许 WebSocket，不代表所选渠道的上游也使用 WebSocket。</p>':'<p class="tp-muted">仅配置直连 Provider，不改变 OpenCodex 内部渠道或子 Agent 并发上限。</p>')+'<div class="tp-actions">'+button('测试 WebSocket 连接','probe')+'<span class="tp-muted" id="tpHandshake" role="status">'+esc(s.handshakes[s.selected]||'尚未测试')+'</span></div><p class="tp-muted">连接测试只检查握手；实际传输以请求记录为准。</p></section><details class="tp-details"><summary>查看配置归属</summary><p>保存在用户级配置，不写入项目或对话覆盖。恢复“跟随默认”后，不再写入管理器的显式偏好。</p><pre class="tp-config">'+esc('[model_providers.'+s.selected+']\n'+(keys.length?keys.join('\n'):'# 不生成显式传输覆盖'))+'</pre>'+(target.proxy?'<p>本地入口设置由同一选择同步管理；第三方上游仍遵循当前代理支持范围。</p>':'')+'</details>';
  for(const section of $$('.tp-details',host))section.open=openSections.includes($('summary',section)?.textContent);
  window.prototypeSelect?.enhance();window.prototypePanel?.refresh();
 }
 function updateForm(){const s=store.snapshot();$('#tpFormState').textContent=formState();$('[data-tp="save"]',host).disabled=!s.dirty.includes(s.selected);$('[data-tp="discard"]',host).disabled=!s.dirty.includes(s.selected);$('#tpFormError').textContent=formError;const keys=Object.entries(store.preview()).filter(([key])=>!['provider','websockets'].includes(key)).map(([key,value])=>key+' = '+value);$('.tp-config',host).textContent='[model_providers.'+s.selected+']\n'+(keys.length?keys.join('\n'):'# 不生成显式传输覆盖');}
 const fact=(label,value)=>'<div><dt>'+label+'</dt><dd>'+esc(value)+'</dd></div>';
 function refreshContexts(){
  const s=store.snapshot();
  for(const slot of $$('[data-transport-context="conversation"]')){
   const key=slot.dataset.transportObject;
   const observation=key?store.observation(key):null,remote=slot.dataset.transportSource==='dev';
   const title='实际传输';
   const text='传输偏好由用户级 Provider 配置管理，此处不提供项目或对话覆盖。';
   const row=observation&&!remote?fact('Codex → OpenCodex',observation.ingress)+fact('OpenCodex → 上游',observation.upstream):fact('Codex → OpenCodex','未观察')+fact('OpenCodex → 上游','未观察');
   const reason=observation&&!remote?observation.reason:remote?'该来源未连接，不采用本机观察结果。':key?.startsWith('project:')?'请在具体对话中查看传输记录。':'暂无此对象的请求观察记录。';
   slot.innerHTML='<div class="tp-context"><h4>'+title+'</h4><p class="tp-muted">'+text+'</p>'+'<dl class="tp-facts">'+row+'</dl><p class="tp-muted">'+esc(reason)+'</p>'+(s.pending.includes('opencodex')?'<p class="tp-muted">本地代理连接配置已保存，待重启 Codex。</p>':'')+button('管理全局对话策略','open-settings')+'</div>';
  }
 }
 function openSettings(){if(window.openDialogPolicy)window.openDialogPolicy('connection');else window.openSettingsSection?.('transport');$('.main')?.scrollTo({top:0});window.prototypePanel?.refresh();}
 function restart(){store.restart();message='已重新加载连接配置 · 实际使用待观察';renderForm();refreshContexts();window.dispatchEvent(new CustomEvent('prototype:codex-restart'));}
 document.addEventListener('change',e=>{if(e.target.id==='tpProvider'){store.select(e.target.value);message=formError='';renderForm();}else if(e.target.id==='tpPolicy'){store.edit('policy',e.target.value);message=formError='';updateForm();}});
 host.addEventListener('input',e=>{if(e.target.id==='tpTimeout'){store.edit('timeout',e.target.value);message='';formError=store.error();updateForm();}});
 document.addEventListener('click',e=>{const target=e.target.closest?.('[data-tp]');if(!target||target.disabled)return;const action=target.dataset.tp;
  if(action==='open-settings'){openSettings();return;}
  if(action==='save'){const result=store.save();formError=result.ok?'':result.error;message=result.ok?'连接配置已保存':'';renderForm();refreshContexts();}
  else if(action==='discard'){store.discard();message=formError='';renderForm();}
  else if(action==='probe'){store.probe();renderForm();}
  else if(action==='simulate-restart')restart();
  else if(action==='observe'){store.observe(target.dataset.scene);refreshContexts();}
 });
 const demo=document.createElement('div');demo.id='transportDemoCard';demo.className='proto-card';demo.dataset.protoContexts='settings:dialog-policy conversations';demo.dataset.policyParts='connection';
 demo.innerHTML='<div class="proto-note-title">连接与传输 · 版本未定</div><p class="pctl-hint">只切内存示例。保存不重启，握手测试不调用模型。</p><div class="pctl-grid">'+button('模拟 Codex 完整重启','simulate-restart')+'</div><div class="mpk-demo-field">请求观察（仅“模型配置与对话边界”）</div><div class="pctl-grid">'+[['unknown','未观察'],['partial','入口 WS／上游 HTTP'],['end-to-end','官方上游 WS'],['fallback','回退 HTTP']].map(([id,label])=>button(label,'observe','data-scene="'+id+'"')).join('')+'</div><label class="mpk-demo-check"><input id="tpDemoLink" type="checkbox">Link 模式（入口仅 HTTP）</label><label class="mpk-demo-check"><input id="tpDemoSaveFailure" type="checkbox">模拟保存失败</label><p class="pctl-hint">观察场景不修改已保存配置；官方上游 WS 示例不代表第三方链路已支持。实际线路、重启和持久化均未接入。</p>';
 ($('#protoStdCards .proto-group-body')||$('#protoStdCards'))?.appendChild(demo);
 demo.addEventListener('change',e=>{if(e.target.id==='tpDemoLink')store.setMode(e.target.checked?'link':'local');if(e.target.id==='tpDemoSaveFailure')store.setSaveFailure(e.target.checked);renderForm();refreshContexts();});
 window.transportPrototype={state:store.snapshot,refreshContexts,openSettings,restart};
 renderForm();refreshContexts();
})();
