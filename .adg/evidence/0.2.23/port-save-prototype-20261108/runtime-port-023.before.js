/* PORT23: declared in-memory outcomes only. No sockets, APIs, files or persistence. */
(() => {
  'use strict';
  const $ = selector => document.querySelector(selector);
  const esc = window.OCXD023.esc;
  const panel = $('#settings-installation');
  if (!panel) return;
  const card = document.createElement('article');
  card.id = 'runtimePort023'; card.className = 'card rp023';
  panel.prepend(card);
  let saved = {mode:'auto',port:'10100'}, active = null, serial = 0, busy = false;
  let scene = 'normal', lastFailure = null, initialized = false;
  const failures = {
    conflict: port => '指定端口 '+port+' 已被占用。请修改端口或恢复自动分配后重试。',
    permission: () => '没有权限监听所选端口。请检查权限或改用其它端口。',
    other: () => '服务启动异常，尚未就绪。请查看日志后重试。'
  };
  function isRunning() { return currentState === 'running'; }
  function valid() { return $('#rp023Auto').checked || /^[0-9]+$/.test($('#rp023Port').value) && Number($('#rp023Port').value)>=1 && Number($('#rp023Port').value)<=65535; }
  function dirty() { return ($('#rp023Auto').checked?'auto':'fixed')!==saved.mode || saved.mode==='fixed' && $('#rp023Port').value!==saved.port; }
  function label(config) { return config.mode==='auto'?'自动分配':'指定端口 '+config.port; }
  function pending() { return isRunning() && active && (saved.mode!==active.mode || saved.mode==='fixed' && saved.port!==active.port); }
  function render() {
    card.innerHTML='<div class="card-head"><div><h2>OpenCodex 服务</h2><p>设置代理服务的运行端口。</p></div></div>'+
      '<div class="setting-row"><div><div class="setting-title">当前实际端口</div><div class="setting-desc">服务就绪后显示实际使用的端口。</div></div><strong id="rp023Actual">—</strong></div>'+
      '<fieldset class="rp023-field"><legend>运行端口</legend><div class="rp023-modes">'+
      '<label><input type="radio" name="rp023Mode" id="rp023Auto" value="auto"'+(saved.mode==='auto'?' checked':'')+'>自动分配</label>'+
      '<label><input type="radio" name="rp023Mode" id="rp023Fixed" value="fixed"'+(saved.mode==='fixed'?' checked':'')+'>指定端口</label></div>'+
      '<div id="rp023FixedField"'+(saved.mode==='auto'?' hidden':'')+'><label for="rp023Port">端口号</label><input class="input" id="rp023Port" type="text" inputmode="numeric" autocomplete="off" aria-describedby="rp023Hint rp023Error" value="'+esc(saved.port)+'"></div>'+
      '<p class="setting-desc" id="rp023Hint">默认由系统分配空闲端口。指定端口被占用时，需要手动调整。</p><p id="rp023Error" role="alert" hidden>请输入 1～65535 之间的整数端口。</p></fieldset>'+
      '<p class="rp023-applied" id="rp023Applied" aria-live="polite"></p><div class="controls"><button class="btn primary" data-rp023="save">保存设置</button><button class="btn ghost" data-rp023="revert">还原修改</button><button class="btn" data-rp023="restart" hidden>重启并应用</button><button class="btn" data-rp023="start">启动 OpenCodex</button></div>'+
      '<div id="rp023Failure" class="rp023-failure" hidden></div>';
    sync();
  }
  function sync() {
    if (!$('#rp023Actual')) return;
    if (!initialized && isRunning()) { active={...saved,actual:$('#card-overview-status [data-b=port]').textContent.trim()}; initialized=true; }
    $('#rp023Actual').textContent=isRunning()?(active?.actual||'读取中'):'未运行';
    $('#rp023FixedField').hidden=$('#rp023Auto').checked;
    const invalid=!valid(); $('#rp023Error').hidden=!invalid;
    $('#rp023Port').setAttribute('aria-invalid',String(invalid));
    for(const input of card.querySelectorAll('input'))input.disabled=busy;
    $('[data-rp023=save]').disabled=busy||invalid||!dirty();
    $('[data-rp023=revert]').disabled=busy||!dirty();
    $('[data-rp023=restart]').hidden=!pending();
    $('[data-rp023=restart]').disabled=busy;
    $('[data-rp023=start]').hidden=isRunning();
    $('[data-rp023=start]').disabled=busy||invalid||dirty()||!['stopped','starting_failed'].includes(currentState);
    $('#rp023Applied').textContent=busy?'正在等待服务就绪…':pending()?'已保存：'+label(saved)+'。当前端口保持不变，下次启动生效。':'已保存：'+label(saved)+'。'+(isRunning()?'当前设置已生效。':'下次启动生效。');
    const fail=$('#rp023Failure'); fail.hidden=!lastFailure||isRunning();
    if(!fail.hidden)fail.innerHTML='<strong>启动失败</strong><p>'+esc(lastFailure.detail)+'</p><div class="controls"><button class="btn" data-rp023="notice">查看失败详情</button><button class="btn" data-rp023="retry">重试启动</button></div>';
  }
  function openSettings() {
    window.closeModal(); window.closeNotificationPanel();
    window.openSettingsSection('installation');
    location.hash='settings?section=installation&card=runtimePort023';
    card.scrollIntoView({block:'start',behavior:'smooth'});
    $('#rp023Auto').focus({preventScroll:true});
  }
  function notice(item) {
    item.read=true; renderNotifications(); closeNotificationPanel();
    window.OCXD023.dialog('OpenCodex 启动失败','<p class="rp023-error">'+esc(item.detail)+'</p>'+window.OCXD023.facts([['失败类型',item.reason==='conflict'?'端口冲突':item.reason==='permission'?'权限不足':'启动异常'],['本次设置',item.setting],['处理状态',item.resolved?'已解决 · 后续启动已就绪':'未解决'],['原有服务','未停止其它实例']])+'<div class="controls"><button class="btn ghost" data-rp023="logs">查看日志</button></div>',item.resolved?'前往端口设置':'重试启动',item.resolved?openSettings:()=>{closeModal();start();},{onExtra:item.resolved?null:openSettings,extraLabel:'前往端口设置'});
  }
  function showState(state) {
    if(typeof suppressNextStateAnnounce==='function')suppressNextStateAnnounce();
    window.setState(state); sync();
  }
  function start() {
    if(busy||!['stopped','starting_failed'].includes(currentState))return;
    busy=true; initialized=true;
    const config={...saved}, attempt=++serial;
    const reason=scene==='conflict'&&config.mode==='fixed'?'conflict':['permission','other'].includes(scene)?scene:null;
    showState('starting');
    showTask({op:'rp023-start-'+attempt,title:'正在启动 OpenCodex',phase:config.mode==='auto'?'分配空闲端口并等待就绪':'监听端口 '+config.port+' 并等待就绪',steps:[['提交启动','done'],['等待服务就绪','active']],acts:[['后台运行','background']]});
    setTimeout(()=>{
      busy=false; clearTask();
      if(reason){
        active=null;
        const detail=failures[reason](config.port);
        lastFailure={id:'rp023-start-'+attempt,kind:'danger',cat:'run',title:'OpenCodex 启动失败',detail,reason,setting:label(config),time:clockLabel(),op:'rp023-start-'+attempt,dedupe:'rp023-start-'+attempt,read:false,resolved:false,deleted:false};
        notifications.unshift(lastFailure);showState('starting_failed');
        $('[data-b=branchhint]').textContent=detail;
        toast('',{kind:'error',title:'OpenCodex 启动失败',detail,action:'前往端口设置'});
        $('#toastEl .toast-act').dataset.rp023='settings';
      }else{
        active={...config,actual:config.mode==='auto'?'55581':config.port};
        scenarios.running.port=active.actual;
        for(const item of notifications.filter(item=>item.id.startsWith('rp023-start-')&&!item.deleted&&!item.resolved)){item.resolved=true;item.read=true;item.resolvedAt=clockLabel();}
        lastFailure=null; showState('running');
        toast('',{kind:'success',title:'OpenCodex 已启动',detail:'当前端口 '+active.actual+' · 服务已就绪'});
      }
      sync();
    },1000);
  }
  function restart() {
    if(busy||!isRunning())return;
    if(dirty()){toast('请先保存或还原端口修改。');return;}
    window.OCXD023.dialog('重启并应用端口设置','<p>将停止当前用户的 OpenCodex 服务，按已保存的设置重新启动。进行中的请求可能中断。</p>'+window.OCXD023.facts([['当前端口',active?.actual||'读取中'],['下次启动',label(saved)]]),'确认重启',()=>{closeModal();showState('stopped');start();});
  }
  render();
  // Listen after the existing core; capture prevents the old success-only timers.
  document.addEventListener('click',event=>{
    const button=event.target.closest('button'); if(!button||button.disabled)return;
    const action=button.dataset.rp023;
    const process=button.dataset.control||button.dataset.trayAction;
    const notif=button.closest('[data-notification]')?.dataset.notification;
    if(notif?.startsWith('rp023-start-')||notif==='start-failed'){event.preventDefault();event.stopImmediatePropagation();const item=notifications.find(n=>n.id===notif);if(item){if(notif==='start-failed'){item.reason='conflict';item.setting='指定端口 10100';}notice(item);}return;}
    if(action||['start','restart'].includes(process)){
      event.preventDefault();event.stopImmediatePropagation();
      if(action==='save'&&valid()&&!busy){saved={mode:$('#rp023Auto').checked?'auto':'fixed',port:$('#rp023Port').value};render();toast('',{kind:'success',title:'端口设置已保存',detail:isRunning()?'当前服务保持原端口，下次启动生效。':'下次启动时使用此设置。'});}
      else if(action==='revert')render();
      else if(action==='settings')openSettings();
      else if(action==='notice'&&lastFailure)notice(lastFailure);
      else if(action==='logs'){closeModal();setRoute('logs?tab=logs');activateDiagTab('logs');}
      else if(action==='restart'||process==='restart')restart();
      else if(['start','retry'].includes(action)||process==='start'){closeModal();start();}
    }
  },true);
  card.addEventListener('change',sync);card.addEventListener('input',sync);
  const rail=document.createElement('section');rail.className='proto-card';rail.dataset.protoContexts='settings:installation overview';
  rail.innerHTML='<div class="proto-note-title">0.2.23 · 端口与启动结果</div><div class="pctl-grid" role="group" aria-label="下次启动结果"><button type="button" class="active" data-rp023-scene="normal" aria-pressed="true">正常就绪</button><button type="button" data-rp023-scene="conflict" aria-pressed="false">固定端口被占用</button><button type="button" data-rp023-scene="permission" aria-pressed="false">权限不足</button><button type="button" data-rp023-scene="other" aria-pressed="false">其它启动失败</button></div><p class="pctl-hint">仅合成演示；自动端口示例为 55581。冲突仅在指定端口模式触发，不检测真实端口。</p><button class="btn" data-rp023-demo-stop>设为未运行</button>';
  $('#protoStdCards').append(rail);
  rail.addEventListener('click',event=>{const button=event.target.closest('[data-rp023-scene]');if(!button)return;scene=button.dataset.rp023Scene;for(const item of rail.querySelectorAll('[data-rp023-scene]')){item.classList.toggle('active',item===button);item.setAttribute('aria-pressed',String(item===button));}});
  $('[data-rp023-demo-stop]').addEventListener('click',()=>{if(!busy){active=null;showState('stopped');}});
  let queued=false;
  new MutationObserver(()=>{if(queued)return;queued=true;queueMicrotask(()=>{queued=false;sync();});}).observe($('#card-overview-status [data-b=statetext]'),{childList:true,characterData:true,subtree:true});
  window.runtimePort023={openSettings,notice:()=>lastFailure&&notice(lastFailure),snapshot:()=>({saved:{...saved},active:active&&{...active},busy,scene})};
})();
