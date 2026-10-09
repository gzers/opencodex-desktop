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
  let scenario='available';
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
      if (spec.delivery.includes('toast')) window.toast?.('原型：'+spec.trigger);
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
      card.querySelector('.mod-status').innerHTML=Object.entries(state.updates).map(([key,u])=>'<span>'+esc(key==='manager'?'管理器':'运行时')+' <b>'+esc(u.current)+'</b><span class="mnt-status '+(u.phase==='failed'?'danger':'')+'">'+labels[u.phase]+'</span></span>').join('');
      const pending=Object.values(state.updates).some(busy);
      card.querySelector('.mod-actions').innerHTML=btn('check-all','检查更新','','primary',pending)+btn('updates','查看更新')+btn('backup-dialog','升级前备份');
      for (const [key,u] of Object.entries(state.updates)) {
        const node=$('#card-upgrade-'+(key==='runtime'?'official':'manager'));
        node.setAttribute('aria-busy',String(busy(u)));
        node.innerHTML='<div class="card-head"><div><h2>'+u.label+'</h2><p>'+esc(key==='manager'?'下载校验后重启管理器生效。':'沿用受控官方运行时更新流程；更新前明确备份覆盖。')+'</p></div><span class="tag">示例版本</span></div><div class="setting-list"><div class="setting-row"><div><div class="setting-title">当前 '+esc(u.current)+(['available','applying'].includes(u.phase)?' → 候选 '+esc(u.candidate):'')+'</div><div class="setting-desc">'+esc(u.source)+'</div></div><b class="mnt-status">'+labels[u.phase]+'</b></div></div><div class="controls">'+btn('check','检查更新',key,'',busy(u))+btn('apply-dialog',u.phase==='failed'?'查看失败与重试':'查看更新并确认',key,'primary',u.phase!=='available'&&u.phase!=='failed')+(busy(u)?btn('cancel','取消',key):'')+'</div>'+(u.phase==='failed'?'<p class="mnt-note danger">示例：网络不可达或更新校验失败。当前版本保留；重试前可检查网络与更新源。</p>':'');
      }
      $('#card-upgrade-manager').insertAdjacentHTML('beforeend','<div class="setting-list mnt-update-policy"><div class="setting-row"><div><div class="setting-title">更新通道与频率</div><div class="setting-desc">仅自动检查，不自动安装；周期为示例配置，不运行计时调度。</div></div><label><span class="mnt-sr">更新通道与频率</span><select id="maintenanceUpdatePolicy" aria-label="更新通道与频率"><option value="stable" '+(state.updatePolicy.channel==='stable'?'selected':'')+'>stable · 每 24 小时</option><option value="beta" '+(state.updatePolicy.channel==='beta'?'selected':'')+'>beta · 每 6 小时</option><option value="manual" '+(state.updatePolicy.channel==='manual'?'selected':'')+'>手动检查</option></select></label></div></div>');
      window.prototypeSelect?.enhance();
    });
    consumeEvents();
  }
  function check(target) {
    const outcome=scenario;
    const generation=model.begin(target);
    if (generation===null) return;
    renderUpdates();
    timers.set(target,setTimeout(()=>{
      const u=state.updates[target];
      model.finish(target,generation,outcome==='failed'?'failed':u.current===u.candidate?'latest':outcome);
      renderUpdates();
    },850));
  }
  function cancel(target) {clearTimeout(timers.get(target)); model.cancel(target); renderUpdates();}
  function showUpdates() {
    openModal('版本升级', '<div class="mnt-update-options">'+Object.entries(state.updates).map(([key,u])=>'<div class="mnt-update-option"><strong>'+u.label+'</strong><p>'+esc(u.current)+(u.phase==='available'?' → '+esc(u.candidate):'')+' · '+labels[u.phase]+'</p>'+btn('apply-dialog','查看并确认',key,'primary',u.phase!=='available')+'</div>').join('')+'</div><p class="mnt-note">版本与来源均为示例；概览与设置共用状态。</p>', '打开升级设置', ()=>openSettingsSection('upgrade'));
  }
  function applyDialog(target) {
    const u=state.updates[target];
    if (!u) return;
    if (u.phase==='failed') {openModal('更新失败', '<p>当前版本 '+esc(u.current)+' 保留。原型可在右侧选择“失败”，检查完成后重试；不运行真实更新。</p>','重试检查',()=>check(target)); return;}
    if (u.phase!=='available') return;
    const version=u.candidate, generation=u.generation, outcome=scenario;
    openModal('确认更新 '+u.label,
      '<p class="modal-lead">'+esc(u.current)+' → '+esc(version)+'</p><p>来源：'+esc(u.source)+'</p><p>'+(target==='manager'?'演示签名校验、下载与重启后生效；不会操作真实应用或代理。':'演示受控官方更新；真实实施需核对运行状态与官方安装边界。')+'</p><p>更新前生成<strong>管理器偏好备份</strong>，不包含 OpenCodex 完整配置、会话、登录态或安装包。</p>',
      '确认演示更新',()=>{
        if (u.generation!==generation || u.candidate!==version || u.phase!=='available') {toast('版本状态已变化，请重新查看。'); return;}
        model.backup('升级前');
        const token=model.begin(target,'applying');
        renderBackups(); renderUpdates();
        timers.set(target,setTimeout(()=>{model.finish(target,token,outcome==='failed'?'failed':'complete'); renderUpdates();},1300));
      });
  }
  function backupPath() {return '当前数据根 / backups / 管理器偏好（路径示例）';}
  function backupDialog() {
    openModal('升级前备份：管理器偏好', '<p class="modal-lead">只备份桌面管理器偏好及清单、完整性摘要。</p><p>不包含 OpenCodex 完整配置、模型 / 账户 / 会话、登录态、Skills / MCP 内容或运行时安装包。</p><p class="mnt-path">保存位置：'+backupPath()+'</p><p>官方运行时配置需要使用对应的导出 / 同步功能，不能用本备份替代。</p>', '生成示例备份', ()=>{model.backup('升级前');renderBackups();consumeEvents();}, {extraLabel:'打开数据与备份',onExtra:()=>openSettingsSection('backup')});
  }
  function candidates() {return M.cleanupCandidates(state.backups,state.policy,Date.now());}
  function renderBackups() {
    preserveFocus(()=>{
      const items=state.backups, policy=state.policy, expired=candidates();
      $('#card-backup-data').innerHTML='<div class="card-head"><div><h2>数据与备份</h2><p>管理器偏好备份 · '+items.length+' 份 · '+items.filter(item=>item.pinned).length+' 份固定</p></div>'+btn('backup-dialog','生成备份','','primary')+'</div><div class="setting-list"><div class="setting-row"><div><div class="setting-title">备份内容</div><div class="setting-desc">管理器偏好、清单、完整性摘要（示例校验）。不是 OpenCodex 完整配置备份。</div><div class="setting-desc">升级前、手动与恢复前创建；导入 / 同步范围另由对应事务列明。</div></div></div><div class="setting-row"><div><div class="setting-title">保存位置</div><div class="mnt-path">'+backupPath()+'</div></div>'+btn('path','查看位置')+'</div><div class="setting-row"><div><div class="setting-title">保留与轮换</div><div class="setting-desc">保留最近 N 份，或仍在 D 天内的备份；满足任一条件保留。固定项另外保留，不占 N 份。</div></div></div></div><form id="maintenancePolicy" class="mnt-policy"><label>最近份数<input class="input" type="number" min="1" max="100" name="count" value="'+policy.count+'" required></label><label>保留天数<input class="input" type="number" min="1" max="365" name="days" value="'+policy.days+'" required></label><label>清理方式<select name="cleanup" aria-label="备份清理方式"><option value="manual" '+(policy.cleanup==='manual'?'selected':'')+'>手动确认（默认）</option><option value="after-create" '+(policy.cleanup==='after-create'?'selected':'')+'>新备份成功后自动轮换</option></select></label><button type="submit" class="btn">应用策略</button><p id="maintenancePolicyError" role="alert" hidden>请输入整数：份数 1–100，天数 1–365。</p></form><p class="mnt-note">当前可清理 '+expired.length+' 份。更改策略不会立即删除；自动轮换只在新备份成功后执行。固定保留与自动轮换是 0.1.10 待评审方案。</p><div class="controls">'+btn('cleanup','预览清理','','',expired.length===0)+btn('backup-empty','查看空列表示例')+'</div><ul class="mnt-backup-list">'+items.map(item=>'<li><div><strong>'+esc(item.name)+'</strong><span>'+new Date(item.created).toLocaleDateString('zh-CN')+' · '+esc(item.reason)+' · '+(item.integrity==='valid'?'校验通过':'校验不通过')+(item.pinned?' · 固定保留':'')+'</span></div><div class="controls">'+btn('pin',item.pinned?'取消固定':'固定保留',item.id,'ghost')+btn('restore','恢复',item.id,'',item.integrity!=='valid')+'</div></li>').join('')+'</ul>';
      window.prototypeSelect?.enhance();
    });
  }
  function cleanupDialog() {
    const list=candidates();
    openModal('清理预览', '<p>仅清理不在最近 '+state.policy.count+' 份且超过 '+state.policy.days+' 天的未固定备份。固定项不会删除。</p><ul class="about-list">'+list.map(item=>'<li>'+esc(item.name)+' · '+new Date(item.created).toLocaleDateString('zh-CN')+'</li>').join('')+'</ul><p>本次 '+list.length+' 份；确认时重新核对保护状态。</p>','确认清理',()=>{const count=model.clean(list.map(item=>item.id));renderBackups();consumeEvents();toast('原型：清理 '+count+' 份，固定或已受保护项目保留。');});
  }
  function restoreDialog(id) {
    const item=state.backups.find(item=>item.id===id);
    if (!item || item.integrity!=='valid') return;
    openModal('恢复管理器偏好', '<p class="modal-lead">'+esc(item.name)+'</p><p>清单与完整性校验通过后，先创建并固定当前偏好的保护备份，再替换管理器偏好。</p><p>不恢复 OpenCodex 配置、账户、会话或运行时版本；取消不会修改任何内容。</p>', '确认演示恢复',()=>{model.restore(id);renderBackups();consumeEvents();});
  }
  function registryDialog() {
    openModal('通知事件注册表 · 0.1.10 候选', '<p>开发维护集中配置；本表覆盖本次原型和跨模块示例，生产发出点的全量映射仍待实施盘点。检查周期与提醒周期独立。</p><div class="mnt-registry">'+M.registry.map(spec=>'<details><summary><code>'+spec.id+'</code><span>'+spec.module+' · '+spec.category+' / '+spec.severity+'</span></summary><dl><dt>触发条件 / 来源</dt><dd>'+spec.trigger+' / '+spec.source+'</dd><dt>性质 / 检查 / 提醒</dt><dd>'+spec.nature+' / '+spec.checkPeriod+' / '+spec.reminderPeriod+'</dd><dt>去重 / 冷却</dt><dd>'+spec.dedupe+' / '+spec.cooldown/1000+' 秒</dd><dt>恢复规则</dt><dd>'+spec.recovery+'</dd><dt>投递 / 保留</dt><dd>'+spec.delivery.join('、')+' / '+spec.retention+'</dd><dt>脱敏</dt><dd>'+spec.redaction+'</dd></dl>'+btn('event','触发示例',spec.id)+'</details>').join('')+'</div>', '',null,{hideConfirm:true,cancelLabel:'关闭'});
    $('#modal').classList.add('modal-wide');
  }
  function historyDialog() {
    openModal('维护事件投递记录', state.deliveries.length?'<ul class="about-list">'+state.deliveries.map(item=>'<li><code>'+item.id+'</code> · '+esc(item.object)+' → '+item.delivery.join('、')+'</li>').join('')+'</ul><p>通知中心事件同时出现在应用铃铛与通知历史；日志、系统投递在这里模拟，不写日志或发系统通知。</p>':'<p class="list-empty">还没有事件；检查更新或生成备份后再查看。</p>', '',null,{hideConfirm:true,cancelLabel:'关闭'});
  }
  function tools() {
    const old=$('[data-proto-contexts="settings:upgrade"]');
    const backupTool=$('[data-proto-contexts="settings:backup"]');
    backupTool?.remove();
    old.outerHTML='<div class="proto-card" id="maintenanceDemoCard" data-proto-contexts="overview settings:upgrade settings:backup logs:notifications"><div class="proto-note-title">0.1.10 · 运行维护</div><div class="pctl-grid">'+btn('goto-upgrade','版本升级')+btn('goto-backup','数据与备份')+'</div><label class="proto-mock-field"><span>下一次更新结果</span><select id="maintenanceScenario" aria-label="维护更新结果"><option value="available">发现更新 / 更新成功</option><option value="latest">已是最新</option><option value="failed">网络或校验失败</option></select></label><div class="pctl-grid">'+btn('reset-updates','重置更新状态')+btn('registry','通知注册表')+btn('history','投递记录')+'</div><p class="pctl-hint">共享概览 / 设置状态。所有版本、路径、更新、备份和事件都为内存示例，不读写本机数据。</p><div id="maintenanceLive" class="mnt-sr" role="status" aria-live="polite"></div></div>';
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
    if(action==='check-all') Object.keys(state.updates).forEach(check);
    else if(action==='check') check(target);
    else if(action==='cancel') cancel(target);
    else if(action==='updates') showUpdates();
    else if(action==='apply-dialog') applyDialog(target);
    else if(action==='backup-dialog') backupDialog();
    else if(action==='cleanup') cleanupDialog();
    else if(action==='pin') {model.pin(target);renderBackups();}
    else if(action==='restore') restoreDialog(target);
    else if(action==='path') openModal('备份保存位置','<p class="mnt-path">'+backupPath()+'</p><p>真实实现从当前数据根解析绝对路径；此处不打开 Finder / Explorer。</p>','打开安装配置',()=>openSettingsSection('installation'));
    else if(action==='backup-empty') openModal('数据与备份 · 空列表','<p class="list-empty">尚无管理器偏好备份。升级前或点击“生成备份”后出现。</p>','知道了',null);
    else if(action==='entry') {window.windowsPrototype?.exit();window.exitNotifMode?.();setRoute('overview');$('#protoCommon').open=false;$('#protoStdCards').open=true;window.prototypePanel?.refresh();}
    else if(action==='goto-upgrade') openSettingsSection('upgrade');
    else if(action==='goto-backup') openSettingsSection('backup');
    else if(action==='registry') registryDialog();
    else if(action==='history') historyDialog();
    else if(action==='event') {model.emit(target,'示例对象');consumeEvents();toast('示例已触发；重复事件按冷却与去重规则投递。');}
    else if(action==='reset-updates') {Object.keys(state.updates).forEach(key=>{cancel(key);model.setScenario(key,'idle');});renderUpdates();}
  });
  document.addEventListener('change',event=>{
    if(event.target.id==='maintenanceScenario') scenario=event.target.value;
    if(event.target.id==='maintenanceUpdatePolicy') {
      const channel=event.target.value;
      if(!['stable','beta','manual'].includes(channel)) return;
      state.updatePolicy={channel,checkHours:channel==='stable'?24:channel==='beta'?6:0};
      state.updates.manager.source=(channel==='manual'?'stable':channel)+' · 签名更新源（示例）';
      model.cancel('manager'); model.setScenario('manager','idle');renderUpdates();
      toast('原型：更新策略已改；下次重新检查，不自动安装。');
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
  window.maintenancePrototype={model,check,backupDialog,updates:showUpdates};
  if(new URLSearchParams(location.search).get('topic')==='maintenance') $('[data-maint-action="entry"]').click();
})();
