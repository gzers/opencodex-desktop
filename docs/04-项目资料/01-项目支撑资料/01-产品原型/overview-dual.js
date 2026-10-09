/* 正式原型的概览适配层：共用原有运行/环境事实、动作与 Logo 动效。 */
(() => {
  const source = new URL(document.currentScript.src);
  const root = document.documentElement;
  const route = document.getElementById('route-overview');
  const card = document.getElementById('card-overview-status');
  const gate = document.getElementById('envGate');
  const details = document.getElementById('card-overview-details');
  if (!route || !card || !gate || !details) return;
  const escape = value => String(value ?? '—').replace(/[&<>"']/g, ch => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[ch]));
  const pairs = rows => '<dl class="motion-facts">' + rows.map(([key,value]) => `<div><dt>${escape(key)}</dt><dd>${escape(value)}</dd></div>`).join('') + '</dl>';
  const read = key => details.querySelector(`[data-b="${key}"]`)?.textContent || '—';
  const oldLabel = card.querySelector('[data-b="statetext"]');
  oldLabel?.closest('.ovc-fact')?.remove();
  const identity = document.createElement('div');
  identity.className = 'motion-identity';
  identity.innerHTML = '<div class="motion-heading"><button type="button" class="motion-detail-icon" title="查看运行详情（含运行环境）" aria-label="查看运行详情，包含运行环境"><span class="motion-mainline" data-b="statetext"></span><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.7" aria-hidden="true"><path d="m9 5 7 7-7 7"/></svg></button></div><p class="motion-caption"></p>';
  card.querySelector('.ovb-summary').prepend(identity);
  const label = identity.querySelector('.motion-mainline');
  const caption = identity.querySelector('.motion-caption');
  const detailButton = identity.querySelector('button');
  const frame = document.createElement('iframe');
  frame.className = 'motion-backdrop'; frame.title = '运行状态形象'; frame.tabIndex = -1; frame.setAttribute('aria-hidden','true');
  card.prepend(frame);
  details.hidden = true; details.dataset.retired = 'true'; details.open = false;
  // 原详情节点仅作为原有渲染的数据宿主，环境项不再另占摘要行。
  card.querySelector('.motion-env-row')?.remove();
  const eventCard = document.getElementById('card-overview-events');
  const originalOpen = openModal;
  openModal = function (...args) {
    document.getElementById('modal').classList.remove('motion-standard-modal','motion-runtime-modal');
    return originalOpen.apply(this,args);
  };
  function showDetail(title,body,kind = 'environment') {
    const restore = document.activeElement;
    openModal(title,body,null,null,{hideConfirm:true,cancelLabel:'关闭',hideNote:true,headActions:'<button class="btn ghost" type="button" aria-label="关闭详情" id="motionModalClose">✕</button>'});
    const modal = document.getElementById('modal');
    modal.classList.add('motion-standard-modal');
    modal.classList.toggle('motion-runtime-modal',kind === 'runtime');
    document.getElementById('motionModalClose').onclick = closeModal;
    document.getElementById('motionModalClose').focus();
    window.__overviewRestoreFocus = restore;
  }
  const originalClose = closeModal;
  closeModal = function (...args) {
    const restore = window.__overviewRestoreFocus;
    window.__overviewRestoreFocus = null;
    const result = originalClose.apply(this,args);
    if (restore?.isConnected && !restore.disabled) restore.focus();
    return result;
  };
  function checks() {
    return (envStates[envState] || envStates.checking).checks.map(([name,value,tone],index) => ({
      name, value:envState === 'checking' && index ? '待检查' : value === '未检查' ? '待检查' : value,
      tone:envState === 'checking' && index || value === '未检查' ? 'idle' : tone
    }));
  }
  function checkList() {
    return '<ol class="motion-check-list">' + checks().map((item,index) => `<li><span class="motion-step">${index+1}</span><b>${escape(item.name)}</b><span class="motion-check-value" data-tone="${item.tone}">${escape(item.value)}</span></li>`).join('') + '</ol>';
  }
  function commands(cfg) {
    return cfg.commands.map(([name,command]) => `<div class="motion-command"><span>${escape(name)}</span><button class="cli-copy-btn" data-prototype-action="cli-copy-command" data-command="${escape(command)}"><code>${escape(command)}</code></button></div>`).join('');
  }
  function runtimeValues(rows,className) {
    return `<dl class="${className}">` + rows.map(([key,value]) =>
      `<div><dt>${escape(key)}</dt><dd><span class="motion-runtime-value" title="${escape(value)}">${escape(value)}</span></dd></div>`
    ).join('') + '</dl>';
  }
  detailButton.onclick = () => {
    const model = STATE_MODEL[currentState] || STATE_MODEL.loading;
    const result = card.querySelector('[data-b="opresult"]');
    showDetail('运行详情',
      '<div class="motion-runtime-grid"><section aria-labelledby="runtimeProcessTitle"><h4 class="motion-section-title" id="runtimeProcessTitle">进程与就绪</h4>' +
      pairs([['运行状态',MAINLINE[model.main].label],['健康',read('health')],['PID / 进程',read('pid')],['端口',scenarios[currentState].port],['当前操作',model.branch?.label || '无']]) +
      '</section><section class="motion-runtime-environment" aria-labelledby="runtimeEnvironmentTitle"><h4 class="motion-section-title" id="runtimeEnvironmentTitle">运行环境</h4>' + checkList() +
      '</section></div><section class="motion-runtime-source" aria-labelledby="runtimeSourceTitle"><h4 class="motion-section-title" id="runtimeSourceTitle">来源与目录</h4>' +
      runtimeValues([['版本',card.querySelector('[data-b="version"]')?.textContent],['运行时',read('runtime')]],'motion-runtime-meta') +
      runtimeValues([['数据目录',read('root')],['OPENCODEX_HOME',read('home')]],'motion-runtime-paths') +
      '</section>' +
      (result && !result.hidden ? '<section class="motion-runtime-result" aria-labelledby="runtimeResultTitle"><h4 class="motion-section-title" id="runtimeResultTitle">操作结果</h4><div class="motion-operation-detail">' + result.innerHTML + '</div></section>' : ''),
      'runtime');
  };
  function environmentDetail() {
    const cfg = envStates[envState] || envStates.checking;
    showDetail('运行环境', `<p class="motion-modal-sub">${escape(cfg.title)}</p>${checkList()}<h4 class="motion-section-title">完整指引</h4><p class="motion-modal-copy">${escape(cfg.desc)}</p>${commands(cfg)}<div class="motion-modal-next"><button class="btn ghost" data-prototype-action="env-guide">安装设置</button></div>`);
  }
  gate.addEventListener('click',event => { if (event.target.closest('[data-overview-env-detail]')) environmentDetail(); });
  document.getElementById('modalBody').addEventListener('click',event => {
    if (document.getElementById('modal').classList.contains('motion-standard-modal') &&
        event.target.closest('[data-prototype-action="env-guide"]')) closeModal();
  });
  const guidance = {
    checking:['正在检查','先确认 Node.js，再依次检查 npm 和 OpenCodex。'],
    missing_node_brew:['安装 Node.js','已检测到 Homebrew，可以使用下面的命令安装；完成后重新检查。'],
    missing_node_nobrew:['安装 Node.js','请从 Node.js LTS 官方渠道完成安装，再回来重新检查。'],
    missing_npm:['修复 npm','请检查 Node.js 安装完整性或重新安装 Node.js LTS，完成后重新检查。'],
    missing_ocx:['接入 OpenCodex','基础环境已通过。可由管理器安装 OpenCodex，或导入已有的离线包。']
  };
  const gateTitle = {
    checking:'检查启动条件', missing_node_brew:'缺少 Node.js', missing_node_nobrew:'缺少 Node.js',
    missing_npm:'npm 不可用', missing_ocx:'尚未接入 OpenCodex'
  };
  let gateKey = '';
  function renderGate() {
    const blocked = envGateActive && envState !== 'ready';
    gate.hidden = !blocked;
    if (!blocked) return;
    const cfg = envStates[envState] || envStates.checking;
    const items = checks();
    const key = JSON.stringify([envState,items]);
    if (key === gateKey) return;
    gateKey = key;
    const [title,description] = guidance[envState] || guidance.checking;
    const count = items.filter(item => item.tone === 'ok').length;
    const checkRows = items.map((item,index) => `<li><span class="overview-check-index">0${index+1}</span><b>${escape(item.name)}</b><span class="overview-check-value" data-tone="${item.tone}">${item.tone === 'ok' ? '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" aria-hidden="true"><path d="m5 12 4 4L19 6"/></svg>' : ''}${escape(item.value)}</span></li>`).join('');
    // 主卡只显示最相关的一条命令，其余校验命令可通过完整指引查看。
    const command = envState !== 'missing_ocx' ? cfg.commands[0] : null;
    const commandHtml = command ? `<div class="env-command"><div class="env-command-label">${escape(command[0])}</div><button class="cli-copy-btn" data-prototype-action="cli-copy-command" data-command="${escape(command[1])}"><code>${escape(command[1])}</code></button></div>` : '';
    const actions = envState === 'checking' ? '<button class="btn" disabled>检查中…</button>' :
      envState === 'missing_ocx' ? '<button class="btn primary" data-prototype-action="runtime-install">安装 OpenCodex</button><button class="btn" data-prototype-action="runtime-import">导入离线包</button><button class="btn ghost" data-prototype-action="env-recheck">重新检查</button>' :
      (envState === 'missing_node_nobrew' ? '<button class="btn primary" data-prototype-action="env-guide">安装指引</button>' : '') + '<button class="btn" data-prototype-action="env-recheck">重新检查</button>';
    const note = envState === 'missing_ocx' ? '安装仅写管理器数据目录；已有安装可在设置中指定来源。' : '未执行的项目显示「待检查」；补全前置条件后继续。';
    gate.className = 'card env-gate';
    gate.innerHTML = `<div class="overview-gate-head"><div><h2>${gateTitle[envState] || gateTitle.checking}</h2><p>${envState === 'checking' ? '确认完成前暂不提供运行操作。' : '补全启动条件后，即可继续使用 OpenCodex。'}</p></div><span class="overview-gate-count">${envState === 'checking' ? '检查中' : count + ' / 3 项通过'}</span></div><div class="overview-gate-body"><ol class="overview-checks" aria-label="环境检查结果">${checkRows}</ol><section class="overview-repair"><h3>${title}</h3><p>${description}</p>${commandHtml}<div class="overview-repair-actions">${actions}</div></section></div><div class="overview-gate-footer"><p>${note}</p><button type="button" data-overview-env-detail>完整指引 ›</button></div>`;
  }
  const originalEnv = renderEnv;
  renderEnv = function (...args) {
    const result = originalEnv.apply(this,args);
    // 旧渲染会替换 gate 正文，即使重复选择同一状态也必须重新投影。
    gateKey = ''; renderGate(); return result;
  };
  const map = {loading:'confirming',not_found:'not_ready',stopped:'stopped',starting:'starting',pending:'starting',running:'running',starting_failed:'failed',at_risk:'stopped',external_takeover:'problem',unreachable:'problem'};
  let previous = '', ready = false, lastEffects = '', lastRender = '';
  function sync() {
    const model = STATE_MODEL[currentState] || STATE_MODEL.loading;
    const blocked = envGateActive && envState !== 'ready';
    route.dataset.mode = blocked ? 'setup' : 'ready';
    if (currentRoute === 'overview') {
      const subtitle = document.querySelector('.page-title p');
      if (subtitle) subtitle.textContent = blocked ? '完成启动前的环境准备，再继续运行操作。' : routes.overview.subtitle;
    }
    label.textContent = MAINLINE[model.main].label;
    detailButton.disabled = blocked;
    detailButton.title = blocked ? '当前运行状态；请先完成环境准备' : '查看运行详情（含运行环境）';
    detailButton.setAttribute('aria-label',blocked ? MAINLINE[model.main].label + '，环境准备中' : '查看运行详情，包含运行环境');
    // at_risk 的进程事实就是「未运行」，概览与「未运行」逐字一致（2026-10-03 用户确认：
    // 原型的「未运行」呈现才是对的），不再另给「启动保护未启用」说明句；成因走通知中心/诊断。
    caption.textContent = blocked ? envState === 'checking' ? '正在确认启动条件' : '启动条件未满足 · 请按下方指引处理' :
      currentState === 'loading' ? '等待首次观测' : currentState === 'pending' ? '进程已起 · 待就绪' : currentState === 'starting' ? '启动中 · 待核验' : currentState === 'starting_failed' ? '启动失败' : model.problem ? '有待处理问题' : currentState === 'not_found' ? '接入后可启动' : '';
    const result = card.querySelector('[data-b="opresult"]');
    if (!blocked && result && !result.hidden) caption.textContent = result.querySelector('.ovb-result-head')?.textContent.trim() || caption.textContent;
    caption.title = model.branch?.hint || caption.textContent;
    details.hidden = true; details.open = false;
    document.getElementById('mods').hidden = blocked;
    if (eventCard) eventCard.hidden = blocked;
    renderGate();
    const compact = root.dataset.overviewSize === 'compact';
    const effects = root.dataset.effects || 'high';
    // 背景光渲染：默认 mesh（WEBGL 网格渐变 + 颗粒）；环境不支持 WEBGL 时父页已把值降级为 css。
    const render = root.dataset.glowRender === 'css' ? 'css' : 'mesh';
    if (effects !== lastEffects || render !== lastRender) {
      const url = new URL('../../03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型/候选/2026-09-28-Logo本体形变/index.html',source);
      url.search = '?embed=1&layout=hero&v=dual&effects=' + effects + '&render=' + render;
      frame.src = url.href; lastEffects = effects; lastRender = render; ready = false;
    }
    const task = document.body.dataset.taskop || '';
    const taskMark = task === 'op_start' ? 'starting' : task === 'op_stop' || task === 'op_restart' ? 'stopping' : '';
    const envMark = blocked ? envState === 'checking' ? 'confirming' : 'not_ready' : '';
    const payload = {type:'opencodex-motion-preview',state:taskMark || envMark || map[currentState] || 'confirming',palette:currentState === 'at_risk' ? 'at_risk' : '',theme:root.dataset.theme,active:currentRoute === 'overview' && !document.hidden,
      presentation:blocked ? compact ? {size:74,padding:40} : {size:90,padding:52} : compact ? {size:100,padding:85} : {size:140,padding:103}};
    const key = JSON.stringify(payload);
    if (ready && key !== previous) { frame.contentWindow.postMessage(payload,location.protocol === 'file:' ? '*' : location.origin); previous = key; }
  }
  const originalEvents = renderRecentEvents;
  renderRecentEvents = function (...args) {
    const result = originalEvents.apply(this,args);
    // 摘要显示最新记录；原有全部条目仍保留于数据及 DOM。
    const list = document.getElementById('recentEventsList');
    if (list) list.append(...[...list.children].reverse());
    return result;
  };
  const originalRender = render;
  render = function (...args) { const result = originalRender.apply(this,args); sync(); return result; };
  function updateSize() {
    const win = document.querySelector('.window');
    const measured = win.getBoundingClientRect().height;
    const height = measured || parseFloat(root.style.getPropertyValue('--win-h')) || 760;
    const mode = height < 680 ? 'compact' : 'standard';
    if (root.dataset.overviewSize !== mode) { root.dataset.overviewSize = mode; sync(); }
  }
  if ('ResizeObserver' in window) new ResizeObserver(updateSize).observe(document.querySelector('.window'));
  const originalSize = setWinSize;
  setWinSize = function (...args) { const result = originalSize.apply(this,args); updateSize(); return result; };
  addEventListener('resize',updateSize);
  addEventListener('message',event => { if (event.source === frame.contentWindow && event.data?.type === 'opencodex-motion-ready') { ready = true; previous = ''; sync(); } });
  frame.addEventListener('load',() => { ready = true; previous = ''; sync(); });
  new MutationObserver(sync).observe(root,{attributes:true,attributeFilter:['data-theme','data-route','data-effects','data-glow-render']});
  document.addEventListener('proto-task-op',sync); document.addEventListener('visibilitychange',sync);
  routes.overview.subtitle = '运行状态与常用操作；环境明细收进运行详情。';
  root.dataset.overviewSize = 'standard'; updateSize(); sync();
})();
