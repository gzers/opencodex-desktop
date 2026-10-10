/* 现有页面的 0.1.10 维护交互，所有事务只改变本页内存。 */
(() => {
  'use strict';
  const M=window.MaintenanceModel, model=M.create(), state=model.state;
  const $=selector=>document.querySelector(selector);
  const esc=value=>String(value).replace(/[&<>"']/g, char=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[char]));
  const labels={idle:'尚未检查', checking:'检查中…', available:'有可用更新', latest:'已是最新', failed:'失败，可重试', applying:'更新中…', complete:'更新完成', 'restart-required':'等待重启'};
  const busy=u=>['checking','applying'].includes(u.phase);
  const btn=(action, text, target='', style='', disabled=false)=>'<button type="button" class="btn '+style+'" data-maint-action="'+action+'" data-target="'+target+'" '+(disabled?'disabled':'')+'>'+text+'</button>';
  const timers=new Map(), delivered=new Set();
  let scenario='available', backupValid=true, clockOffset=0;
  const notesState={manager:'ready',runtime:'ready'};
  const notesRequests={manager:0,runtime:0};
  // 固定排版 fixture；链接仅为官方发布列表 / 包页面参考，不伪造候选版本的网页。
  const releases={
    manager:{url:'https://github.com/gzers/opencodex-desktop/releases',sections:[
      ['概览更新入口','检查后集中查看管理器与面板更新，分别确认每个对象的更新范围。概览与设置共用状态，切换页面后仍可查看当前进度。'],
      ['更新详情与反馈','更新前阅读版本说明与来源，确认后展示下载、校验和应用阶段。网络中断与校验失败保留当前版本，并提供恢复操作。'],
      ['备份范围更清晰','更新前可备份管理器偏好、清单与完整性摘要。数据与备份中可查看位置、保留期限、固定项与清理预览。'],
      ['通知策略','检查触发支持启动、概览切入、到期评估和恢复联网。重复提醒合并，后台检查不会直接开始安装。'],
      ['界面一致性','标准按钮统一取消顶部高光，保留语义色、悬停和键盘焦点。概览背景光覆盖内容面板的顶部与左右边距。'],
      ['更新后的确认','管理器安装后需重启，并在重新启动后回读版本确认生效。备份和更新记录保留各自状态，不提前宣告完成。']
    ]},
    runtime:{url:'https://www.npmjs.com/package/@bitkyc08/opencodex',sections:[
      ['功能改进','优化面板的日常操作反馈，调整配置读取与状态展示。更新后请确认面板能够正常访问。'],
      ['稳定性','改善连接中断后的状态恢复，并保留失败时的可读错误信息。'],
      ['兼容性说明','更新前核对当前运行来源、目标前缀与版本，避免修改其他安装位置。'],
      ['更新范围','更新只针对已登记的 OpenCodex 包体。管理器版本、账户登录状态与会话不属于包体更新范围。'],
      ['更新准备','执行前检查面板运行状态和安装归属。运行中的面板若需停止，应在确认范围后按受控流程处理。'],
      ['完成核验','安装结果需回读包名、版本与入口，并核对完整性。命令返回成功不能替代可用性确认。']
    ]}
  };
  const stageLabels={checking:'正在检查版本',preparing:'准备更新',downloading:'正在下载安装包',verifying:'正在校验完整性',installing:'正在应用更新','restart-required':'安装就绪，等待重启',complete:'更新完成'};
  const cancellable=u=>busy(u)&&u.progress?.stage!=='installing';
  function progressMarkup(target) {
    const u=state.updates[target], stage=u.progress?.stage||u.phase;
    if(!busy(u)&&!['restart-required','complete','failed'].includes(u.phase)&&stage!=='cancelled')return '';
    const percent=u.phase==='applying'&&stage==='downloading'?u.progress.percent:null;
    const text=stage==='cancelled'?'已取消，当前版本保留':u.phase==='failed'?'更新失败，当前版本保留':stageLabels[stage]||labels[u.phase];
    return '<div class="mnt-progress" role="status" aria-live="polite"><div class="mnt-progress-title"><strong>'+text+'</strong><span>'+(percent===null?'':percent+'%')+'</span></div>'+(!busy(u)?'':'<div class="mnt-progress-track" role="progressbar" aria-label="'+esc(text)+'" '+(percent!==null?'aria-valuemin="0" aria-valuemax="100" aria-valuenow="'+percent+'"':'aria-valuetext="进度尚未知"')+'><i class="'+(percent===null?'indeterminate':'')+'" style="width:'+(percent===null?'32':percent)+'%"></i></div>')+'<p>'+(u.phase==='restart-required'?'当前仍为 '+esc(u.current)+'。重启后核验 '+esc(u.candidate)+' 是否生效。':u.phase==='failed'?'查看失败详情并重新检查；不会继续应用未通过校验的包。':u.phase==='complete'?'版本已回读确认。':stage==='installing'?'正在写入与核验，请稍候。此阶段不可取消。':'可关闭此窗口，稍后从概览或设置继续查看。')+'</p></div>';
  }
  function notesContent(target,compact=false) {
    const status=notesState[target];
    return status==='loading'?'<p role="status">正在读取本版本更新说明…</p>':status==='failed'?'<p role="alert">更新说明暂时无法加载。版本检查结果仍有效。</p>'+btn('notes-retry','重试说明',target):status==='empty'?'<p>发布者尚未提供本版本的更新说明。</p>':releases[target].sections.slice(0,compact?2:undefined).map(([title,body])=>'<section><h5>'+esc(title)+'</h5><p>'+esc(body)+'</p></section>').join('');
  }
  function releaseNotes(target,compact=false) {
    return '<div class="mnt-release-heading"><h4>更新内容</h4><a href="'+releases[target].url+'" target="_blank" rel="noopener noreferrer">'+(target==='manager'?'查看官方发布记录':'查看官方包页面')+' ↗</a></div><div class="mnt-release-notes '+(compact?'compact':'')+'" tabindex="0" role="region" aria-label="'+esc(state.updates[target].label)+'更新内容">'+notesContent(target,compact)+'</div>';
  }
  function wideDialog(title,body,label,handler,opts={}) {
    dialog(title,'<div class="mnt-update-flow">'+body+'</div>',label,handler,opts);
    $('#modal').classList.add('modal-wide');
  }
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
      card.querySelector('.mod-actions').innerHTML=btn('check-all','检查更新','','primary',pending)+Object.entries(state.updates).filter(([,u])=>u.phase==='applying'||u.phase==='restart-required').map(([key])=>btn('progress','查看'+(key==='manager'?'管理器':'面板')+'进度',key)).join('');
      let progress=card.querySelector('.mnt-overview-progress');
      if(!progress){card.insertAdjacentHTML('beforeend','<div class="mnt-overview-progress"></div>');progress=card.querySelector('.mnt-overview-progress');}
      progress.innerHTML=Object.entries(state.updates).filter(([,u])=>busy(u)||u.phase==='restart-required').map(([key])=>progressMarkup(key)).join('');
      for (const [key,u] of Object.entries(state.updates)) {
        const node=$('#card-upgrade-'+(key==='runtime'?'official':'manager'));
        node.setAttribute('aria-busy',String(busy(u)));
        node.innerHTML='<div class="card-head"><div><h2>'+u.label+'</h2><p>'+esc(key==='manager'?'下载校验后重启管理器生效。':'使用官方包更新面板；更新前确认范围与运行状态。')+'</p></div><span class="tag">示例版本</span></div><div class="setting-list"><div class="setting-row"><div><div class="setting-title">当前 '+esc(u.current)+(['available','applying'].includes(u.phase)?' → 候选 '+esc(u.candidate):'')+'</div><div class="setting-desc">'+esc(u.source)+'</div></div><b class="mnt-status">'+labels[u.phase]+'</b></div></div><div class="controls">'+btn('check','检查更新',key,'',busy(u))+btn('apply-dialog',u.phase==='failed'?'查看失败与重试':'查看更新并确认',key,'primary',u.phase!=='available'&&u.phase!=='failed')+(cancellable(u)?btn('cancel','取消更新',key):'')+(u.phase==='restart-required'?btn('restart','重启并完成更新',key,'primary'):'')+(['applying','restart-required','complete'].includes(u.phase)?btn('progress','查看进度',key):'')+'</div>'+progressMarkup(key)+(u.phase==='failed'?'<p class="mnt-note danger">示例：网络不可达或更新校验失败。当前版本保留；重试前可检查网络与更新源。</p>':'');
      }
      $('#card-upgrade-manager').insertAdjacentHTML('beforeend','<div class="setting-list mnt-update-policy"><div class="setting-row"><div><div class="setting-title">更新通道与频率</div><div class="setting-desc">仅自动检查，不自动安装。管理器 stable 每 24h / beta 每 6h；面板每 24h。</div></div><label><span class="mnt-sr">更新通道与频率</span><select id="maintenanceUpdatePolicy" aria-label="更新通道与频率"><option value="stable" '+(state.updatePolicy.channel==='stable'?'selected':'')+'>stable · 每 24 小时</option><option value="beta" '+(state.updatePolicy.channel==='beta'?'selected':'')+'>beta · 每 6 小时</option><option value="manual" '+(state.updatePolicy.channel==='manual'?'selected':'')+'>手动检查</option></select></label></div></div>');
      const policy=$('#maintenanceUpdatePolicy');if(policy)policy.disabled=state.updates.manager.phase==='restart-required'||(state.updates.manager.phase==='applying'&&state.updates.manager.progress?.stage==='installing');
      window.prototypeSelect?.enhance();
    });
    refreshProgressDialog();
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
  function cancel(target) {
    if(!cancellable(state.updates[target]))return;
    clearTimeout(timers.get(target));model.cancel(target,time());renderUpdates();
  }
  function showUpdates(target) {
    if(target){
      const u=state.updates[target];if(!u)return;
      if(['applying','restart-required','complete'].includes(u.phase)){progressDialog(target);return;}
      if(u.phase==='available'||u.phase==='failed'){applyDialog(target);return;}
      wideDialog(u.label+'更新详情','<p class="mnt-kicker">'+labels[u.phase]+'</p><div class="mnt-version-hero"><strong>'+esc(u.current)+'</strong></div><p class="mnt-note">'+esc(u.source)+'</p>','',null,{hideConfirm:true,cancelLabel:'关闭'});return;
    }
    wideDialog('检查更新结果','<p class="mnt-note">分别查看更新内容与范围，确认后才会开始更新。</p><div class="mnt-update-options">'+Object.entries(state.updates).map(([key,u])=>'<section class="mnt-update-option"><div class="mnt-result-heading"><h4>'+u.label+'</h4><span class="tag">'+labels[u.phase]+'</span></div><div class="mnt-version-hero"><span>'+esc(u.current)+'</span>'+(u.phase==='available'?'<span aria-hidden="true">→</span><strong>'+esc(u.candidate)+'</strong>':'')+'</div><p class="mnt-note">'+esc(u.source)+'</p>'+(u.phase==='available'?releaseNotes(key,true):'<p>'+(u.phase==='failed'?'未能取得候选版本，请重新检查。':'当前没有待确认的更新。')+'</p>')+'<div class="controls">'+btn('update-detail',u.phase==='failed'?'查看失败与重试':'查看详情',key,'',busy(u)||u.phase==='idle')+'</div></section>').join('')+'</div>','',null,{hideConfirm:true,cancelLabel:'关闭'});
  }
  function progressActions(target) {
    const u=state.updates[target];
    return u.progress?.stage==='cancelled'?btn('update-detail','返回更新详情',target):cancellable(u)?btn('cancel','取消更新',target):u.phase==='restart-required'?btn('restart','重启并完成更新',target,'primary'):u.phase==='failed'?btn('check','重新检查',target,'primary'):'';
  }
  function refreshProgressDialog() {
    const node=$('#maintenanceProgress');
    if(!node||$('#modalMask').style.display!=='flex'||!node.dataset.target)return;
    if(!$('#modalBody').querySelector('#maintenanceProgress'))return;
    const target=node.dataset.target;
    node.innerHTML=progressMarkup(target);
    const actions=$('#maintenanceProgressActions');if(actions)actions.innerHTML=progressActions(target);
  }
  function progressDialog(target) {
    const u=state.updates[target];
    wideDialog(u.label+'更新进度','<div class="mnt-version-hero"><span>'+esc(u.current)+'</span><span aria-hidden="true">→</span><strong>'+esc(u.candidate)+'</strong></div><p class="mnt-note">'+esc(u.source)+'</p><div id="maintenanceProgress" data-target="'+target+'">'+progressMarkup(target)+'</div><ol class="mnt-stage-list">'+['准备','下载','校验',target==='manager'?'安装与重启':'应用与核验'].map(text=>'<li>'+text+'</li>').join('')+'</ol><div id="maintenanceProgressActions" class="controls">'+progressActions(target)+'</div>','',null,{hideConfirm:true,cancelLabel:'关闭窗口'});
    $('#maintenanceProgress').dataset.target=target;
  }
  function runApply(target,token,outcome) {
    const steps=[['downloading',18,450],['downloading',46,500],['downloading',78,500],['downloading',100,450],['verifying',null,650],['installing',null,800]];
    function next(index) {
      const u=state.updates[target];if(u.generation!==token||u.phase!=='applying')return;
      if(index===steps.length){model.finish(target,token,outcome==='failed'?'failed':target==='manager'?'restart-required':'complete',time());renderUpdates();return;}
      const [stage,percent,delay]=steps[index];
      if(stage==='installing'&&outcome==='failed'){model.finish(target,token,'failed',time());renderUpdates();return;}
      model.progress(target,token,stage,percent);renderUpdates();
      timers.set(target,setTimeout(()=>next(index+1),delay));
    }
    timers.set(target,setTimeout(()=>next(0),600));
  }
  function applyDialog(target) {
    const u=state.updates[target];if(!u)return;
    if(u.phase==='failed') {wideDialog('更新失败','<p class="mnt-kicker">'+esc(u.label)+'</p><div class="mnt-version-hero"><strong>'+esc(u.current)+'</strong><span>当前版本保留</span></div>'+progressMarkup(target)+'<p class="mnt-note">请检查网络与更新源后重试；校验失败的安装包不会应用。</p>','重试检查',()=>check(target));return;}
    if(u.phase!=='available')return;
    const version=u.candidate,generation=u.generation,outcome=scenario;
    wideDialog(u.label+'更新详情与确认',
      '<div class="mnt-result-heading"><p class="mnt-kicker">版本升级</p><span class="tag">有可用更新</span></div><div class="mnt-version-hero"><span>'+esc(u.current)+'</span><span aria-hidden="true">→</span><strong>'+esc(version)+'</strong></div><p class="mnt-note">'+esc(u.source)+' · '+(target==='manager'?'安装后需重启管理器':'更新已登记的面板包体')+'</p>'+releaseNotes(target)+'<div class="mnt-confirm-scope"><label class="mnt-backup-option"><input type="checkbox" id="maintenancePreBackup" checked aria-describedby="maintenancePreBackupScope">更新前备份管理器偏好（建议）</label><p id="maintenancePreBackupScope" class="mnt-note">保存到当前数据根的 backups。包含管理器偏好、清单与完整性摘要；不包含面板完整配置、会话、登录态或安装包。</p></div>',
      '确认更新',()=>{
        const withBackup=$('#maintenancePreBackup').checked;
        const token=model.confirmApply(target,generation,version,withBackup,time(),backupValid);
        if(token===null){toast(withBackup&&!backupValid?'备份失败，更新未开始。请重试备份或重新选择。':'版本状态已变化，请重新查看。');consumeEvents();return;}
        renderBackups();renderUpdates();progressDialog(target);runApply(target,token,outcome);
      },{confirmKeepsOpen:true});
  }
  const backupNodes=[
    {id:'root',name:'backups',type:'folder',parts:[],level:0,description:'按年月与动作分类存放备份。'},
    {id:'year',parent:'root',name:'2026',type:'folder',parts:['2026'],level:1,description:'UTC 年份'},
    {id:'month',parent:'year',name:'10',type:'folder',parts:['2026','10'],level:2,description:'UTC 月份'},
    {id:'action',parent:'month',name:'upgrade',type:'folder',parts:['2026','10','upgrade'],level:3,description:'升级前备份；实际范围以清单为准。'},
    {id:'record',parent:'action',name:'bk_20261010093000_1a2b3c4d',type:'folder',parts:['2026','10','upgrade','bk_20261010093000_1a2b3c4d'],level:4,description:'一份管理器偏好备份，两个文件整组保留。'},
    {id:'preferences',parent:'record',name:'preferences.json',type:'file',parts:['2026','10','upgrade','bk_20261010093000_1a2b3c4d','preferences.json'],level:5,description:'管理器偏好快照。来源 manager-state/preferences.json，用于恢复界面、启动、更新、备份及通知等偏好。'},
    {id:'manifest',parent:'record',name:'backup-manifest.json',type:'file',parts:['2026','10','upgrade','bk_20261010093000_1a2b3c4d','backup-manifest.json'],level:5,description:'备份编号、动作、源路径、时间、格式版本与校验清单。SHA-256 和字节数保存在此文件内。'}
  ];
  // 初始只显示根目录、年份和月份；下级目录按需逐层展开。
  const collapsedBackupNodes=new Set(backupNodes.filter(node=>node.type==='folder'&&node.level>=2).map(node=>node.id));
  function backupRoot() {return window.dataPathsPrototype?.current().backups || '当前数据目录 / backups';}
  function backupPath() {return esc(backupRoot());}
  function backupNodePath(node) {
    const separator=window.dataPathsPrototype?.current().platform==='windows'?'\\':'/';
    return [backupRoot(),...node.parts].join(separator);
  }
  function backupFileIcon(type) {
    const shape=type==='folder'?'<path d="M3 7V5a2 2 0 0 1 2-2h5l2 3h7a2 2 0 0 1 2 2v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2Z"/>':'<path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8Z"/><path d="M14 2v6h6M8 13h8M8 17h6"/>';
    return '<svg class="mnt-file-icon proto-tree-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true">'+shape+'</svg>';
  }
  function backupFilesMarkup() {
    const rows=window.PrototypeTreeTable.rows(window.PrototypeTreeTable.visible(backupNodes,collapsedBackupNodes),node=>{
      const folder=node.type==='folder', expanded=!collapsedBackupNodes.has(node.id);
      return {table:true,attrs:'id="maintenanceFileRow-'+node.id+'"',depth:node.level,label:node.name,
        toggleAttrs:folder?'data-maint-action="backup-tree-toggle" data-target="'+node.id+'"':null,
        toggleClass:'mnt-tree-toggle',expanded,icon:backupFileIcon(node.type),
        cells:[esc(node.description),btn('backup-node-open',folder?'打开目录':'打开文件',node.id,'ghost')]};
    });
    return '<section id="maintenanceBackupFiles" class="mnt-backup-files" aria-labelledby="maintenanceBackupFilesTitle"><h3 id="maintenanceBackupFilesTitle">备份文件与目录</h3><p class="mnt-note">管理器偏好备份 · 不包含面板完整配置或运行数据</p><p class="mnt-path">保存位置：'+backupPath()+'</p><div class="mnt-tree-scroll proto-tree" tabindex="0" role="region" aria-label="备份文件树表"><table class="mnt-backup-tree"><caption class="mnt-sr">备份目录层级、文件用途与打开操作</caption><thead><tr><th scope="col">名称</th><th scope="col">用途</th><th scope="col">操作</th></tr></thead><tbody>'+rows+'</tbody></table></div><p class="mnt-note">两个文件应整组保留。恢复前重新校验快照与清单；没有单独的校验摘要文件。其他事务按各自清单列明。</p></section>';
  }
  function openBackupNode(id) {
    const node=backupNodes.find(item=>item.id===id);if(!node)return;
    const folder=node.type==='folder', path=backupNodePath(node);
    dialog(folder?'打开备份目录':'打开备份文件',
      '<p class="modal-lead">'+(folder?'将在系统文件管理器中打开目录。':'将在系统默认应用中打开文件。')+'</p><p class="mnt-path">'+esc(path)+'</p><p>'+(folder?'请勿直接移动或删除备份文件，否则可能无法恢复。':'请勿直接修改文件内容，否则可能导致校验失败、无法恢复。')+'备份清理与恢复请使用管理器的对应操作。</p>',
      '继续打开',()=>{if(backupNodePath(node)!==path){toast('目录已变化，请重新选择打开目标。');return;}toast('原型：已确认打开'+(folder?'目录':'文件')+'，未访问本机文件。');});
  }
  function backupDialog() {
    dialog('生成管理器偏好备份', '<p class="modal-lead">只备份桌面管理器偏好及清单、完整性摘要。</p><p>不包含 OpenCodex 完整配置、模型 / 账户 / 会话、登录态、Skills / MCP 内容或运行时安装包。</p><p class="mnt-path">保存位置：'+backupPath()+'</p><p>官方运行时配置需要使用对应的导出 / 同步功能，不能用本备份替代。</p>', '生成示例备份', ()=>{model.backup('手动',time());renderBackups();consumeEvents();}, {extraLabel:'打开数据与备份',onExtra:()=>openSettingsSection('backup')});
  }
  function candidates() {return M.cleanupCandidates(state.backups,state.policy,time());}
  function renderBackups() {
    preserveFocus(()=>{
      const items=state.backups, policy=state.policy, expired=candidates();
      $('#card-backup-data').innerHTML='<div class="card-head"><div><h2>数据与备份</h2><p>管理器偏好备份 · '+items.length+' 份 · '+items.filter(item=>item.pinned).length+' 份固定</p></div>'+btn('backup-dialog','生成备份','','primary')+'</div>'+backupFilesMarkup()+'<form id="maintenancePolicy" class="mnt-policy"><label>最近份数<input class="input" type="number" min="1" max="100" name="count" value="'+policy.count+'" required></label><label>保留天数<input class="input" type="number" min="1" max="365" name="days" value="'+policy.days+'" required></label><label>清理方式<select name="cleanup" aria-label="备份清理方式"><option value="manual" '+(policy.cleanup==='manual'?'selected':'')+'>手动确认（默认）</option><option value="after-create" '+(policy.cleanup==='after-create'?'selected':'')+'>新备份成功后自动轮换</option></select></label><button type="submit" class="btn">应用策略</button><p id="maintenancePolicyError" role="alert" hidden>请输入整数：份数 1–100，天数 1–365。</p></form><p class="mnt-note">最近份数或保留天数满足其一即保留；固定项额外保留，不占最近份数。当前可清理 '+expired.length+' 份。更改策略不会立即删除；自动轮换只在新备份成功后执行。固定保留与自动轮换是 0.1.10 待评审方案。</p><div class="controls">'+btn('cleanup','预览清理','','',expired.length===0)+btn('backup-empty','查看空列表示例')+'</div><ul class="mnt-backup-list">'+items.map(item=>'<li><div><strong>'+esc(item.name)+'</strong><span>'+new Date(item.created).toLocaleDateString('zh-CN')+' · '+esc(item.reason)+' · '+(item.integrity==='valid'?'校验通过':'校验不通过')+(item.pinned?' · 固定保留':'')+'</span></div><div class="controls">'+btn('pin',item.pinned?'取消固定':'固定保留',item.id,'ghost')+btn('restore','恢复',item.id,'',item.integrity!=='valid')+'</div></li>').join('')+'</ul>';
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
    dialog('通知事件注册表 · 0.1.10 候选', '<p>开发维护集中配置；本表按产品流程规划 '+M.registry.length+' 项；prototype 标记项已接原型状态，planned 项可独立触发示例，生产调用点仍待映射。检查周期与提醒周期独立。</p><div class="mnt-registry">'+M.registry.map(spec=>'<details><summary><code>'+spec.id+'</code><span>'+spec.module+' · '+spec.category+' / '+spec.severity+' · '+spec.implemented+'</span></summary><dl><dt>触发条件 / 来源</dt><dd>'+spec.trigger+' / '+spec.source+'<br>'+spec.triggers.map(id=>esc(M.config.triggers[id])+' <code>'+id+'</code>').join('<br>')+'</dd><dt>性质 / 检查 / 提醒</dt><dd>'+spec.nature+' / '+spec.checkPeriod+' / '+spec.reminderPeriod+'</dd><dt>去重 / 冷却</dt><dd>'+spec.dedupe+' / '+spec.cooldown/1000+' 秒</dd><dt>恢复规则</dt><dd>'+(spec.recovery.map(rule=>{const fields=spec.recoveryMatch?.[rule.split(':')[0]]||[];return esc(rule)+(fields.length?'（同 '+fields.map(esc).join(' / ')+'）':'');}).join('、')||'无自动解除事件；任务结束不代表恢复')+'</dd><dt>投递 / 保留</dt><dd>'+spec.delivery.join('、')+' / '+spec.retention+'</dd><dt>脱敏</dt><dd>'+spec.redaction+'</dd></dl>'+btn('event','触发示例',spec.id)+'</details>').join('')+'</div>', '',null,{hideConfirm:true,cancelLabel:'关闭'});
    $('#modal').classList.add('modal-wide');
  }
  function historyDialog() {
    dialog('维护事件投递记录', state.deliveries.length?'<ul class="about-list">'+state.deliveries.map(item=>'<li><code>'+item.id+'</code> · '+esc(item.object)+' · '+esc(item.origin)+' → '+item.delivery.join('、')+'</li>').join('')+'</ul><p>通知中心事件同时出现在应用铃铛与通知历史；日志、系统投递在这里模拟，不写日志或发系统通知。</p>':'<p class="list-empty">还没有事件；检查更新或生成备份后再查看。</p>', '',null,{hideConfirm:true,cancelLabel:'关闭'});
  }
  function tools() {
    const old=$('[data-proto-contexts="settings:upgrade"]');
    const backupTool=$('[data-proto-contexts="settings:backup"]');
    backupTool?.remove();
    old.outerHTML='<div class="proto-card" id="maintenanceDemoCard" data-proto-contexts="overview settings:upgrade settings:backup logs:notifications"><div class="proto-note-title">0.1.10 · 运行维护</div><div class="pctl-grid">'+btn('goto-upgrade','版本升级')+btn('goto-backup','数据与备份')+'</div><label class="proto-mock-field"><span>下一次更新结果</span><select id="maintenanceScenario" aria-label="维护更新结果"><option value="available">发现更新 / 更新成功</option><option value="latest">已是最新</option><option value="failed">网络或校验失败</option></select></label><label class="proto-mock-field"><span>更新说明（均为排版示例）</span><select id="maintenanceNotesScenario"><option value="ready">有更新内容</option><option value="empty">发布者未提供说明</option><option value="failed">说明加载失败，可重试</option><option value="loading">说明加载中</option></select></label><div class="pctl-grid">'+btn('reset-updates','重置更新状态')+btn('registry','通知注册表')+btn('history','投递记录')+'</div><label class="proto-mock-field"><span>升级前备份结果</span><select id="maintenanceBackupResult"><option value="valid">成功</option><option value="failed">失败，阻止已勾选的更新</option></select></label><div class="pctl-grid">'+['app.ready','route.overview.enter','window.resume','network.online','schedule.due','retry.due'].map(id=>btn('trigger',M.config.triggers[id],id)).join('')+btn('advance','模拟经过 24 小时')+'</div><p class="pctl-hint">共享概览 / 设置状态。备份树表为现有格式的独立示例，不对应下方备份记录；展开、打开与警告只演示交互，不检查文件存在或调用系统应用。更新内容与下载百分比为排版示例；网页链接是官方发布列表 / 包页面参考，非候选版本的真实说明。重启按钮只模拟版本回读。所有版本、路径、更新、备份和事件都为内存示例，不读写本机数据。启动延迟 15 秒、路由与页面计时仅演示调度；可模拟触发与经过时间，刷新重置。</p><div id="maintenanceLive" class="mnt-sr" role="status" aria-live="polite"></div></div>';
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
    else if(action==='progress')progressDialog(target);
    else if(action==='restart'){model.restart(target,time());renderUpdates();}
    else if(action==='notes-retry'){
      notesState[target]='loading';const generation=state.updates[target].generation,request=++notesRequests[target];
      const region=el.closest('.mnt-update-option')||el.closest('.mnt-update-flow');
      region.querySelector('.mnt-release-notes').innerHTML=notesContent(target);
      setTimeout(()=>{if(state.updates[target].generation!==generation||notesRequests[target]!==request)return;notesState[target]='ready';if(region.isConnected)region.querySelector('.mnt-release-notes').innerHTML=notesContent(target,region.classList.contains('mnt-update-option'));},650);
    }
    else if(action==='trigger') toast('原型：'+M.config.triggers[target]+'，开始 '+trigger(target)+' 项检查；其余被缓存 / 互斥 / 策略合并。');
    else if(action==='advance') {clockOffset+=86400000;toast('原型：时间前进 24h，开始 '+trigger('schedule.due')+' 项检查。');}
    else if(action==='apply-dialog') applyDialog(target);
    else if(action==='backup-dialog') backupDialog();
    else if(action==='backup-tree-toggle') {if(!backupNodes.some(node=>node.id===target&&node.type==='folder'))return;if(collapsedBackupNodes.has(target))collapsedBackupNodes.delete(target);else collapsedBackupNodes.add(target);renderBackups();}
    else if(action==='backup-node-open') openBackupNode(target);
    else if(action==='cleanup') cleanupDialog();
    else if(action==='pin') {model.pin(target);renderBackups();}
    else if(action==='restore') restoreDialog(target);
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
    if(event.target.id==='maintenanceNotesScenario'){for(const key of Object.keys(notesState)){notesState[key]=event.target.value;notesRequests[key]++;}}
    if(event.target.id==='maintenanceUpdatePolicy') {
      const channel=event.target.value;
      if(!['stable','beta','manual'].includes(channel)) return;
      if(!model.setUpdatePolicy(channel,time())){event.target.value=state.updatePolicy.channel;return;}renderUpdates();trigger('policy.changed');
      toast('原型：管理器通道已改；面板缓存与事务保留。手动模式关闭自动检查，不自动安装。');
    }
  });
  document.addEventListener('submit',event=>{
    if(event.target.id!=='maintenancePolicy') return;
    event.preventDefault();
    const form=event.target;
    if(!model.setPolicy(Number(form.elements.count.value),Number(form.elements.days.value),form.elements.cleanup.value)) {$('#maintenancePolicyError').hidden=false;return;}
    renderBackups();toast('原型：策略已应用，未立即删除备份。');
  });
  window.addEventListener('prototype:data-paths',renderBackups);
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
