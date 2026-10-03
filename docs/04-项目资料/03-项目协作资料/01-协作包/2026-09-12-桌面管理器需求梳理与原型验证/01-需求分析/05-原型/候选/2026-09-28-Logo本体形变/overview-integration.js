/* 把形象接入现行原型已有状态投影；候选 iframe 仅用于复用动效研究页。 */
(()=>{
 routes.overview.subtitle='运行状态、常用操作与同块详情。';
 const root=document.documentElement,card=document.getElementById('card-overview-status');
 const identity=document.createElement('div');identity.className='motion-identity';
 const frame=document.createElement('iframe');frame.title='运行状态形象';frame.tabIndex=-1;frame.setAttribute('aria-hidden','true');frame.src='./index.html?embed=1&layout=hero&v=20260930';
 const label=document.createElement('span');label.className='motion-mainline';label.dataset.b='statetext';
 const caption=document.createElement('div');caption.className='motion-caption';
 const old=card.querySelector('[data-b="statetext"]');old.closest('.ovc-fact').remove();
 const detailButton=document.createElement('button');detailButton.type='button';detailButton.className='motion-detail-icon';detailButton.title='查看运行详情';detailButton.setAttribute('aria-label','查看运行详情');
 detailButton.innerHTML='<svg viewBox="0 0 24 24" width="16" height="16" fill="none" stroke="currentColor" stroke-width="1.7" aria-hidden="true"><path d="m9 5 7 7-7 7"/></svg>';
 detailButton.prepend(label);
 const heading=document.createElement('div');heading.className='motion-heading';heading.append(detailButton);
 identity.append(heading,caption);card.querySelector('.ovb-summary').prepend(identity);card.prepend(frame);frame.className='motion-backdrop';

 const details=card.querySelector('.ovb-detail');details.dataset.retired='true';details.hidden=true;
 const envRow=document.createElement('button');envRow.type='button';envRow.className='motion-env-row';
 envRow.innerHTML='<strong>环境</strong><span class="motion-env-summary"></span><span class="motion-env-action">查看 ›</span>';card.append(envRow);
 const escape=value=>String(value??'—').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 const pairs=rows=>'<dl class="motion-facts">'+rows.map(([k,v])=>`<div><dt>${escape(k)}</dt><dd>${escape(v)}</dd></div>`).join('')+'</dl>';
 const read=key=>details.querySelector(`[data-b="${key}"]`)?.textContent||'—';
 const baseOpen=openModal;openModal=function(...args){document.getElementById('modal').classList.remove('motion-standard-modal');return baseOpen.apply(this,args)};
 function showDetail(title,body){
  openModal(title,body,null,null,{hideConfirm:true,cancelLabel:'关闭',hideNote:true,headActions:'<button class="btn ghost" type="button" aria-label="关闭详情" id="motionModalClose">✕</button>'});
  document.getElementById('modal').classList.add('motion-standard-modal');document.getElementById('motionModalClose').onclick=closeModal;
 }
 detailButton.onclick=()=>{
  const model=STATE_MODEL[currentState]||STATE_MODEL.loading;
  const op=document.querySelector('[data-b="opresult"]');
  showDetail('运行详情',`<p class="motion-modal-sub">${escape(MAINLINE[model.main].label)} · 当前原型观测</p><h4 class="motion-section-title">进程与就绪</h4>`+
   pairs([['运行状态',MAINLINE[model.main].label],['健康',read('health')],['PID / 进程',read('pid')],['端口',scenarios[currentState].port],['当前操作',model.branch?.label||'无']])+
   '<h4 class="motion-section-title">来源与目录</h4>'+pairs([['版本',card.querySelector('[data-b="version"]')?.textContent],['Runtime',read('runtime')],['数据目录',read('root')],['OPENCODEX_HOME',read('home')]])+
   (op&&!op.hidden?'<h4 class="motion-section-title">操作结果</h4><div class="motion-operation-detail">'+op.innerHTML+'</div>':''));
 };
 function checks(){return (envStates[envState]||envStates.checking).checks.map(([name,value,tone],i)=>({name,value:envState==='checking'&&i?'待检查':value==='未检查'?'待检查':value,tone:envState==='checking'&&i?'idle':value==='未检查'?'idle':tone}))}
 envRow.onclick=()=>{
  const cfg=envStates[envState]||envStates.checking;
  const table='<ol class="motion-check-list">'+checks().map((c,i)=>`<li><span class="motion-step">${i+1}</span><b>${escape(c.name)}</b><span class="motion-check-value" data-tone="${c.tone}">${escape(c.value)}</span></li>`).join('')+'</ol>';
  const commands=cfg.commands.map(([name,command])=>`<div class="motion-command"><span>${escape(name)}</span><button class="cli-copy-btn" data-prototype-action="cli-copy-command" data-command="${escape(command)}"><code>${escape(command)}</code></button></div>`).join('');
  const actions=envState==='missing_ocx'?'<button class="btn primary" data-prototype-action="runtime-install">安装 OpenCodex</button><button class="btn" data-prototype-action="runtime-import">导入离线包</button>':envState==='ready'||envState==='checking'?'':'<button class="btn" data-prototype-action="env-guide">查看安装指引</button>';
  showDetail('运行环境',`<p class="motion-modal-sub">${escape(cfg.title)}</p>${table}`+(envState==='ready'?'':`<h4 class="motion-section-title">${envState==='checking'?'检查顺序':'下一步'}</h4><p class="motion-modal-copy">${escape(cfg.desc)}</p>${commands}`)+`<div class="motion-modal-next">${actions}${envState==='checking'?'':'<button class="btn ghost" data-prototype-action="env-recheck">重新检查</button>'}</div>`);
 };
 const map={loading:'confirming',not_found:'not_ready',stopped:'stopped',starting:'starting',pending:'starting',running:'running',starting_failed:'failed',at_risk:'problem',external_takeover:'problem',unreachable:'problem'};
 let previous='',ready=false;
 function sync(force=false){
  const model=STATE_MODEL[currentState]||STATE_MODEL.loading;
  label.textContent=MAINLINE[model.main].label;
  const envText={checking:'正在检查 Node.js…',missing_node_brew:'缺少 Node.js，后续检查暂停',missing_node_nobrew:'缺少 Node.js，后续检查暂停',missing_npm:'npm 不可用，后续检查暂停',missing_ocx:'基础环境已就绪，尚未接入 OpenCodex',ready:'前置条件已满足'};
  envRow.querySelector('.motion-env-summary').innerHTML=checks().map(c=>`<span class="motion-env-item"><b>${escape(c.name)}</b><span data-tone="${c.tone}">${escape(c.value)}</span></span>`).join('');
  envRow.title=envText[envState]||envText.checking;
  envRow.querySelector('.motion-env-action').textContent=envState==='ready'||envState==='checking'?'查看 ›':envState==='missing_ocx'?'接入 ›':'处理 ›';

  caption.textContent=currentState==='loading'?'等待首次观测':currentState==='pending'?'进程已起 · 待就绪':currentState==='starting'?'启动中 · 待核验':currentState==='starting_failed'?'启动失败':currentState==='at_risk'?'启动保护未启用':model.problem?'有待处理问题':currentState==='not_found'?'接入后可启动':'';
  const result=card.querySelector('[data-b="opresult"]');
  const resultTitle=result&&!result.hidden?result.querySelector('.ovb-result-head')?.textContent.trim():'';
  if(resultTitle)caption.textContent=resultTitle;
  caption.title=resultTitle||(model.branch?.hint||caption.textContent);
  details.hidden=true;
  // 环境监测也驱动**顶部主 Logo**的效果（复用既有形象，不新增状态、不改状态数量）：
  //   环境检查中 → 正在确认（双向寻位）；环境未通过（缺 Node/npm/OpenCodex）→ 待接入（三球分离待命）；
  //   环境通过 → 回到运行主线自己的形象。
  // 进行中的启停操作也驱动顶部形象：停止中＝下沉、启动中＝轻跃聚合（主线文字不受影响）。
  const taskOp=document.body.dataset.taskop||'';
  const taskMark=taskOp==='op_start'?'starting':(taskOp==='op_stop'||taskOp==='op_restart')?'stopping':'';
  const envMark=envState==='checking'?'confirming':envState==='ready'?'':'not_ready';
  // at_risk 沿用问题结构，但传 palette 提示让嵌入层改用**更纯的琥珀**取色（不新增状态数量）。
 const palette=currentState==='at_risk'?'at_risk':'';
 const payload={type:'opencodex-motion-preview',state:taskMark||envMark||map[currentState]||'confirming',palette,theme:root.dataset.theme,active:currentRoute==='overview'&&!document.hidden};
  const key=JSON.stringify(payload);
  if(ready&&(force||key!==previous)){frame.contentWindow.postMessage(payload,location.protocol==='file:'?'*':location.origin);previous=key}
 }
 const baseRender=render;render=function(...args){const result=baseRender.apply(this,args);sync();return result};
 addEventListener('message',e=>{if(e.source===frame.contentWindow&&e.data?.type==='opencodex-motion-ready'){ready=true;sync(true)}});
 frame.addEventListener('load',()=>{ready=true;sync(true)});
 new MutationObserver(()=>sync()).observe(root,{attributes:true,attributeFilter:['data-theme','data-route']});
document.addEventListener('proto-task-op',()=>sync());
 document.addEventListener('visibilitychange',()=>sync());
 const review=document.createElement('section');review.className='motion-review';review.innerHTML=`<strong>A · 居中悬浮 / 无框融合</strong><p>Logo、状态和主要操作居中排列；扩大光场融入页面。结构幅度固定为已采纳的 1.25。</p><p>点击状态与箭头打开运行详情；环境摘要取代原详情行。动画区高度固定，底部保留原三卡。下方测试器可任意切态，光场持续平滑过渡。</p>`;
 document.querySelector('.prototype-side').prepend(review);root.dataset.density='compact';sync();
})();
