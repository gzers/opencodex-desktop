/* 现有页面的 0.1.10 维护交互，所有事务只改变本页内存。 */
(() => {
  'use strict';
  const M=window.MaintenanceModel, model=M.create(), state=model.state;
  const $=selector=>document.querySelector(selector);
  const esc=value=>String(value).replace(/[&<>"']/g, char=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[char]));
  const labels={idle:'尚未检查', checking:'检查中…', available:'有可用更新', latest:'已是最新', failed:'失败，可重试', applying:'下载 / 校验 / 应用中…', complete:'演示更新完成'};
  const busy=u=>['checking','applying'].includes(u.phase);
  const btn=(action, text, target='', style='', disabled=false)=>'<button type="button" class="btn '+style+'" data-maint-action="'+action+'" data-target="'+target+'" '+(disabled?'disabled':'')+'>'+text+'</button>';
  const timers=new Map(), delivered=new Set();
  let scenario='available', backupValid=true, clockOffset=0;
  const time=()=>Date.now()+clockOffset;
  function dialog(title,body,label,handler,opts={}) {
    openModal(title,body,label,handler,{hideNote:true,...opts});
    const note=$('#ann-modal-demo .annotation-source p');
    if(note)note.textContent+=' 0.1.10：版本、更新源、路径、通知与调度均为内存示例；不执行真实安装、写入或系统通知。';
  }
  function preserveFocus(render) {
    const el=document.activeElement;
    const key=el?.dataset?.maintAction, target=el?.dataset?.target;
    const parent=el?.closest?.('[id]')?.id;
    render();
    if (key && parent && !el.isConnected) {
      const root=document.getElementById(parent);
      const same=root?.querySelector('[data-maint-action="'+key+'"][data-target="'+target+'"]');
      (same&&!same.disabled?same:root?.querySelector('button:not(:disabled)'))?.focus();
    }
  }
  function consumeEvents() {
    state.deliveries.slice().reverse().forEach(item=>{
      if (delivered.has(item)) return;
      delivered.add(item);
      const spec=M.registry.find(spec=>spec.id===item.id);
      if (item.delivery.includes('toast')) window.toast?.('原型：'+spec.trigger);
    });
    window.maintenanceNotificationBridge?.(state.notifications, M.registry);
    const live=$('#maintenanceLive');
    if (live) live.textContent=Object.values(state.updates).map(u=>u.label+'：'+labels[u.phase]).join('；');
  }
  function renderUpdates() {
    preserveFocus(()=>{
      const card=$('#card-overview-upgrade');
      card.setAttribute('aria-busy',String(Object.values(state.updates).some(busy)));
      card.querySelector('.tag').textContent=Object.values(state.updates).some(u=>u.phase==='available')?'有更新':'版本';
      card.querySelector('.mod-status').innerHTML=Object.entries(state.updates).map(([key,u])=>'<button type="button" class="mnt-version-row" data-maint-action="update-detail" data-target="'+key+'" '+(u.phase==='idle'||busy(u)?'disabled':'')+' aria-label="查看'+esc(u.label)+'更新详情">'+esc(key==='manager'?'管理器':'面板')+' <b>'+esc(u.current)+'</b><span class="mnt-status '+(u.phase==='failed'?'danger':'')+'">'+labels[u.phase]+'</span><span aria-hidden="true">›</span></button>').join('');
      const pending=Object.values(state.updates).some(busy);
      card.querySelector('.mod-actions').innerHTML=btn('check-all','检查更新','','primary',pending);
      for (const [key,u] of Object.entries(state.updates)) {
        const node=$('#card-upgrade-'+(key==='runtime'?'official':'manager'));
        node.setAttribute('aria-busy',String(busy(u)));
        node.innerHTML='<div class="card-head"><div><h2>'+u.label+'</h2><p>'+esc(key==='manager'?'下载校验后重启管理器生效。':'使用官方包更新面板；更新前确认范围与运行状态。')+'</p></div><span class="tag">示例版本</span></div><div class="setting-list"><div class="setting-row"><div><div class="setting-title">当前 '+esc(u.current)+(['available','applying'].includes(u.phase)?' → 候选 '+esc(u.candidate):'')+'</div><div class="setting-desc">'+esc(u.source)+'</div></div><b class="mnt-status">'+labels[u.phase]+'</b></div></div><div class="controls">'+btn('check','检查更新',key,'',busy(u))+btn('apply-dialog',u.phase==='failed'?'查看失败与重试':'查看更新并确认',key,'primary',u.phase!=='available'&&u.phase!=='failed')+(busy(u)?btn('cancel','取消',key):'')+'</div>'+(u.phase==='failed'?'<p class="mnt-note danger">示例：网络不可达或更新校验失败。当前版本保留；重试前可检查网络与更新源。</p>':'');
      }
      $('#card-upgrade-manager').insertAdjacentHTML('beforeend','<div class="setting-list mnt-update-policy"><div class="setting-row"><div><div class="setting-title">更新通道与频率</div><div class="setting-desc">仅自动检查，不自动安装。管理器 stable 每 24h / beta 每 6h；面板每 24h。</div></div><label><span class="mnt-sr">更新通道与频率</span><select id="maintenanceUpdatePolicy" aria-label="更新通道与频率"><option value="stable" '+(state.updatePolicy.channel==='stable'?'selected':'')+'>stable · 每 24 小时</option><option value="beta" '+(state.updatePolicy.channel==='beta'?'selected':'')+'>beta · 每 6 小时</option><option value="manual" '+(state.updatePolicy.channel==='manual'?'selected':'')+'>手动检查</option></select></label></div></div>');
      window.prototypeSelect?.enhance();
    });
    consumeEvents();
  }
  function check(target, onComplete, token) {
    const outcome=scenario;
    const generation=token??model.begin(target);
    if (generation===null) return;
    renderUpdates();
    timers.set(target,setTimeout(()=>{
      const u=state.updates[target];
      const accepted=model.finish(target,generation,outcome==='failed'?'failed':u.current===u.candidate?'latest':outcome,time());
      renderUpdates();
      if(accepted)onComplete?.();
    },850));
  }
  function trigger(origin) {
    const requests=model.requestChecks(origin,time());
    requests.forEach(item=>check(item.target,null,item.generation));
    return requests.length;
  }
  function checkAll() {
    const requests=model.requestChecks('user.action',time());
    let remaining=requests.length;
    requests.forEach(item=>check(item.target,()=>{
      if(--remaining===0 && requests.every(item=>state.updates[item.target].generation===item.generation) && document.documentElement.dataset.route==='overview' && $('#modalMask').style.display!=='flex')showUpdates();
    },item.generation));
  }
  function cancel(target) {clearTimeout(timers.get(target)); model.cancel(target,time()); renderUpdates();}
  function showUpdates(target) {
    if(target){
      const u=state.updates[target];
      if(!u)return;
      if(u.phase==='available'||u.phase==='failed'){applyDialog(target);return;}
      dialog(u.label+'更新详情','<p class="modal-lead">当前 '+esc(u.current)+'</p><p>'+labels[u.phase]+'</p><p>来源：'+esc(u.source)+'</p>','关闭',null);return;
    }
    dialog('检查更新结果', '<div class="mnt-update-options">'+Object.entries(state.updates).map(([key,u])=>'<div class="mnt-update-option"><strong>'+u.label+'</strong><p>'+esc(u.current)+(u.phase==='available'?' → '+esc(u.candidate):'')+' · '+labels[u.phase]+'</p>'+btn('update-detail',u.phase==='failed'?'查看失败与重试':'查看详情',key,'primary',busy(u)||u.phase==='idle')+'</div>').join('')+'</div>', '',null,{hideConfirm:true,cancelLabel:'关闭'});
  }
  function applyDialog(target) {
    const u=state.updates[target];
    if (!u) return;
    if (u.phase==='failed') {dialog('更新失败', '<p>当前版本 '+esc(u.current)+' 保留。请检查网络与更新源后重试。</p>','重试检查',()=>check(target)); return;}
    if (u.phase!=='available') return;
    const version=u.candidate, generation=u.generation, outcome=scenario;
    dialog(u.label+'更新详情与确认',
      '<p class="modal-lead">'+esc(u.current)+' → '+esc(version)+'</p><p>来源：'+esc(u.source)+'</p><p>'+(target==='manager'?'下载与签名校验通过后，重启管理器生效。':'使用官方包受控更新面板；执行前检查运行状态。')+'</p><label class="mnt-backup-option"><input type="checkbox" id="maintenancePreBackup" checked aria-describedby="maintenancePreBackupScope"> 更新前备份管理器偏好（建议）</label><p id="maintenancePreBackupScope" class="mnt-note">保存在当前数据根的 backups 中，只包含管理器偏好、清单与完整性摘要；不包含面板完整配置、会话、登录态或安装包。</p>',
      '确认更新',()=>{
        const withBackup=$('#maintenancePreBackup').checked;
        const token=model.confirmApply(target,generation,version,withBackup,time(),backupValid);
        if(token===null){toast(withBackup&&!backupValid?'备份失败，更新未开始。请重试备份或重新选择。':'版本状态已变化，请重新查看。');consumeEvents();return;}
        closeModal();renderBackups();renderUpdates();
        timers.set(target,setTimeout(()=>{model.finish(target,token,outcome==='failed'?'failed':'complete',time());renderUpdates();},1300));
      },{confirmKeepsOpen:true});
  }
  function backupPath() {return '当前数据根 / backups / 管理器偏好（路径示例）';}
  function backupDialog() {
    dialog('生成管理器偏好备份', '<p class="modal-lead">只备份桌面管理器偏好及清单、完整性摘要。</p><p>不包含 OpenCodex 完整配置、模型 / 账户 / 会话、登录态、Skills / MCP 内容或运行时安装包。</p><p class="mnt-path">保存位置：'+backupPath()+'</p><p>官方运行时配置需要使用对应的导出 / 同步功能，不能用本备份替代。</p>', '生成示例备份', ()=>{model.backup('手动',time());renderBackups();consumeEvents();}, {extraLabel:'打开数据与备份',onExtra:()=>openSettingsSection('backup')});
  }
  function candidates() {return M.cleanupCandidates(state.backups,state.policy,time());}
  function renderBackups() {
    preserveFocus(()=>{
      const items=state.backups, policy=state.policy, expired=candidates();
      $('#card-backup-data').innerHTML='<div class="card-head"><div><h2>数据与备份</h2><p>管理器偏好备份 · '+items.length+' 份 · '+items.filter(item=>item.pinned).length+' 份固定</p></div>'+btn('backup-dialog','生成备份','','primary')+'</div><div class="setting-list"><div class="setting-row"><div><div class="setting-title">备份内容</div><div class="setting-desc">管理器偏好、清单、完整性摘要（示例校验）。不是 OpenCodex 完整配置备份。</div><div class="setting-desc">升级前、手动与恢复前创建；导入 / 同步范围另由对应事务列明。</div></div></div><div class="setting-row"><div><div class="setting-title">保存位置</div><div class="mnt-path">'+backupPath()+'</div></div>'+btn('path','查看位置')+'</div><div class="setting-row"><div><div class="setting-title">保留与轮换</div><div class="setting-desc">保留最近 N 份，或仍在 D 天内的备份；满足任一条件保留。固定项另外保留，不占 N 份。</div></div></div></div><form id="maintenancePolicy" class="mnt-policy"><label>最近份数<input class="input" type="number" min="1" max="100" name="count" value="'+policy.count+'" required></label><label>保留天数<input class="input" type="number" min="1" max="365" name="days" value="'+policy.days+'" required></label><label>清理方式<select name="cleanup" aria-label="备份清理方式"><option value="manual" '+(policy.cleanup==='manual'?'selected':'')+'>手动确认（默认）</option><option value="after-create" '+(policy.cleanup==='after-create'?'selected':'')+'>新备份成功后自动轮换</option></select></label><button type="submit" class="btn">应用策略</button><p id="maintenancePolicyError" role="alert" hidden>请输入整数：份数 1–100，天数 1–365。</p></form><p class="mnt-note">当前可清理 '+expired.length+' 份。更改策略不会立即删除；自动轮换只在新备份成功后执行。固定保留与自动轮换是 0.1.10 待评审方案。</p><div class="controls">'+btn('cleanup','预览清理','','',expired.length===0)+btn('backup-empty','查看空列表示例')+'</div><ul class="mnt-backup-list">'+items.map(item=>'<li><div><strong>'+esc(item.name)+'</strong><span>'+new Date(item.created).toLocaleDateString('zh-CN')+' · '+esc(item.reason)+' · '+(item.integrity==='valid'?'校验通过':'校验不通过')+(item.pinned?' · 固定保留':'')+'</span></div><div class="controls">'+btn('pin',item.pinned?'取消固定':'固定保留',item.id,'ghost')+btn('restore','恢复',item.id,'',item.integrity!=='valid')+'</div></li>').join('')+'</ul>';
      window.prototypeSelect?.enhance();
    });
  }
  function cleanupDialog() {
    const list=candidates();
    dialog('清理预览', '<p>仅清理不在最近 '+state.policy.count+' 份且超过 '+state.policy.days+' 天的未固定备份。固定项不会删除。</p><ul class="about-list">'+list.map(item=>'<li>'+esc(item.name)+' · '+new Date(item.created).toLocaleDateString('zh-CN')+'</li>').join('')+'</ul><p>本次 '+list.length+' 份；确认时重新核对保护状态。</p>','确认清理',()=>{const count=model.clean(list.map(item=>item.id),time());renderBackups();consumeEvents();toast('原型：清理 '+count+' 份，固定或已受保护项目保留。');});
  }
  function restoreDialog(id) {
    const item=state.backups.find(item=>item.id===id);
    if (!item || item.integrity!=='valid') return;
    dialog('恢复管理器偏好', '<p class="modal-lead">'+esc(item.name)+'</p><p>清单与完整性校验通过后，先创建并固定当前偏好的保护备份，再替换管理器偏好。</p><p>不恢复 OpenCodex 配置、账户、会话或运行时版本；取消不会修改任何内容。</p>', '确认演示恢复',()=>{model.restore(id,time());renderBackups();consumeEvents();});
  }
  function registryDialog() {
    dialog('通知事件注册表 · 0.1.10 候选', '<p>开发维护集中配置；本表按产品流程规划 '+M.registry.length+' 项；prototype 标记项已接原型状态，planned 项可独立触发示例，生产调用点仍待映射。检查周期与提醒周期独立。</p><div class="mnt-registry">'+M.registry.map(spec=>'<details><summary><code>'+spec.id+'</code><span>'+spec.module+' · '+spec.category+' / '+spec.severity+' · '+spec.implemented+'</span></summary><dl><dt>触发条件 / 来源</dt><dd>'+spec.trigger+' / '+spec.source+'<br>'+spec.triggers.map(id=>esc(M.config.triggers[id])+' <code>'+id+'</code>').join('<br>')+'</dd><dt>性质 / 检查 / 提醒</dt><dd>'+spec.nature+' / '+spec.checkPeriod+' / '+spec.reminderPeriod+'</dd><dt>去重 / 冷却</dt><dd>'+spec.dedupe+' / '+spec.cooldown/1000+' 秒</dd><dt>恢复规则</dt><dd>'+(spec.recovery.join('、')||'事务结束；无持续状态')+'</dd><dt>投递 / 保留</dt><dd>'+spec.delivery.join('、')+' / '+spec.retention+'</dd><dt>脱敏</dt><dd>'+spec.redaction+'</dd></dl>'+btn('event','触发示例',spec.id)+'</details>').join('')+'</div>', '',null,{hideConfirm:true,cancelLabel:'关闭'});
    $('#modal').classList.add('modal-wide');
  }
  function historyDialog() {
    dialog('维护事件投递记录', state.deliveries.length?'<ul class="about-list">'+state.deliveries.map(item=>'<li><code>'+item.id+'</code> · '+esc(item.object)+' · '+esc(item.origin)+' → '+item.delivery.join('、')+'</li>').join('')+'</ul><p>通知中心事件同时出现在应用铃铛与通知历史；日志、系统投递在这里模拟，不写日志或发系统通知。</p>':'<p class="list-empty">还没有事件；检查更新或生成备份后再查看。</p>', '',null,{hideConfirm:true,cancelLabel:'关闭'});
  }
  function tools() {
    const old=$('[data-proto-contexts="settings:upgrade"]');
    const backupTool=$('[data-proto-contexts="settings:backup"]');
    backupTool?.remove();
    old.outerHTML='<div class="proto-card" id="maintenanceDemoCard" data-proto-contexts="overview settings:upgrade settings:backup logs:notifications"><div class="proto-note-title">0.1.10 · 运行维护</div><div class="pctl-grid">'+btn('goto-upgrade','版本升级')+btn('goto-backup','数据与备份')+'</div><label class="proto-mock-field"><span>下一次更新结果</span><select id="maintenanceScenario" aria-label="维护更新结果"><option value="available">发现更新 / 更新成功</option><option value="latest">已是最新</option><option value="failed">网络或校验失败</option></select></label><div class="pctl-grid">'+btn('reset-updates','重置更新状态')+btn('registry','通知注册表')+btn('history','投递记录')+'</div><label class="proto-mock-field"><span>升级前备份结果</span><select id="maintenanceBackupResult"><option value="valid">成功</option><option value="failed">失败，阻止已勾选的更新</option></select></label><div class="pctl-grid">'+['app.ready','route.overview.enter','window.resume','network.online','schedule.due','retry.due'].map(id=>btn('trigger',M.config.triggers[id],id)).join('')+btn('advance','模拟经过 24 小时')+'</div><p class="pctl-hint">共享概览 / 设置状态。所有版本、路径、更新、备份和事件都为内存示例，不读写本机数据。启动延迟 15 秒、路由与页面计时仅演示调度；可模拟触发与经过时间，刷新重置。</p><div id="maintenanceLive" class="mnt-sr" role="status" aria-live="polite"></div></div>';
    const group=$('#launchGrid .proto-launch-group:last-child .pctl-tiles');
    const entry=document.createElement('button');entry.type='button';entry.textContent='0.1.10 维护';entry.dataset.maintAction='entry';
    // 在原有 launch 监听器安装后加入，交给本模块，不注册一个不存在的产品路由。
    group.appendChild(entry);
    window.prototypePanel?.refresh();
  }
  document.addEventListener('click',event=>{
    const el=event.target.closest?.('[data-maint-action]');
    if (!el || el.disabled) return;
    const action=el.dataset.maintAction, target=el.dataset.target;
    if(action==='check-all') checkAll();
    else if(action==='check') check(target);
    else if(action==='cancel') cancel(target);
    else if(action==='updates') showUpdates();
    else if(action==='update-detail') showUpdates(target);
    else if(action==='trigger') toast('原型：'+M.config.triggers[target]+'，开始 '+trigger(target)+' 项检查；其余被缓存 / 互斥 / 策略合并。');
    else if(action==='advance') {clockOffset+=86400000;toast('原型：时间前进 24h，开始 '+trigger('schedule.due')+' 项检查。');}
    else if(action==='apply-dialog') applyDialog(target);
    else if(action==='backup-dialog') backupDialog();
    else if(action==='cleanup') cleanupDialog();
    else if(action==='pin') {model.pin(target);renderBackups();}
    else if(action==='restore') restoreDialog(target);
    else if(action==='path') dialog('备份保存位置','<p class="mnt-path">'+backupPath()+'</p><p>真实实现从当前数据根解析绝对路径；此处不打开 Finder / Explorer。</p>','打开安装配置',()=>openSettingsSection('installation'));
    else if(action==='backup-empty') dialog('数据与备份 · 空列表','<p class="list-empty">尚无管理器偏好备份。升级前或点击“生成备份”后出现。</p>','知道了',null);
    else if(action==='entry') {window.windowsPrototype?.exit();window.exitNotifMode?.();setRoute('overview');$('#protoCommon').open=false;$('#protoStdCards').open=true;window.prototypePanel?.refresh();}
    else if(action==='goto-upgrade') openSettingsSection('upgrade');
    else if(action==='goto-backup') openSettingsSection('backup');
    else if(action==='registry') registryDialog();
    else if(action==='history') historyDialog();
    else if(action==='event') {model.emit(target,'示例对象',time());consumeEvents();toast('示例已触发；重复事件按冷却与去重规则投递。');}
    else if(action==='reset-updates') {Object.keys(state.updates).forEach(key=>{cancel(key);model.setScenario(key,'idle');state.updates[key].lastChecked=null;state.updates[key].failures=0;state.updates[key].nextDue=time()+M.config.jobs.find(job=>job.target===key).startupDelay;});renderUpdates();}
  });
  document.addEventListener('change',event=>{
    if(event.target.id==='maintenanceBackupResult') backupValid=event.target.value==='valid';
    if(event.target.id==='maintenanceScenario') scenario=event.target.value;
    if(event.target.id==='maintenanceUpdatePolicy') {
      const channel=event.target.value;
      if(!['stable','beta','manual'].includes(channel)) return;
      model.setUpdatePolicy(channel,time());renderUpdates();trigger('policy.changed');
      toast('原型：更新策略已改；自动模式重新检查，手动模式等待操作。不自动安装。');
    }
  });
  document.addEventListener('submit',event=>{
    if(event.target.id!=='maintenancePolicy') return;
    event.preventDefault();
    const form=event.target;
    if(!model.setPolicy(Number(form.elements.count.value),Number(form.elements.days.value),form.elements.cleanup.value)) {$('#maintenancePolicyError').hidden=false;return;}
    renderBackups();toast('原型：策略已应用，未立即删除备份。');
  });
  tools();renderUpdates();renderBackups();
  window.maintenancePrototype={model,check,trigger,backupDialog,updates:showUpdates};
  setTimeout(()=>trigger('app.ready'),M.config.jobs.find(job=>job.target==='manager').startupDelay);
  // 页面调度只改变内存；挂起后只执行一次到期检查，不补跑错过的周期。
  setInterval(()=>{if(!document.hidden)trigger('schedule.due');},60000);
  new MutationObserver(()=>{if(document.documentElement.dataset.route==='overview')trigger('route.overview.enter');}).observe(document.documentElement,{attributes:true,attributeFilter:['data-route']});
  document.addEventListener('visibilitychange',()=>{if(!document.hidden)trigger('window.resume');});
  window.addEventListener('online',()=>trigger('network.online'));
  if(new URLSearchParams(location.search).get('topic')==='maintenance') $('[data-maint-action="entry"]').click();
})();
