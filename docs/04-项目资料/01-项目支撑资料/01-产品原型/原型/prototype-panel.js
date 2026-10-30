/* 原型工具：公共设置与当前页面 / Tab 的 Mock 共用一个滚动区域。 */
(() => {
  const side = document.querySelector('.prototype-side');
  if (!side) return;
  const $ = selector => document.querySelector(selector);
  const windowsCard = $('#windowsDemoCard');
  if (windowsCard) {
    windowsCard.dataset.protoContexts = 'windows';
    ($('#protoStdCards .proto-group-body') || $('#protoStdCards')).appendChild(windowsCard);
  }
  // 顶层「全局评审条件」默认收起；锚点跳到其中的分区时先展开，避免滚到隐藏元素。
  const commonGroup = $('#protoCommon');
  side.addEventListener('click', event => {
    const link = event.target.closest?.('.proto-rail .pctl-bar a[href^="#"]');
    if (!link || !commonGroup) return;
    const target = document.getElementById(link.getAttribute('href').slice(1));
    if (target && commonGroup.contains(target) && !commonGroup.open) commonGroup.open = true;
  }, true);
  const labels = {components:'组件与样式','connection-map':'全景概览','model-picker':'公共渠道模型选择器', annotations:'注释卡片', overview:'概览', panel:'面板', models:'模型', conversations:'对话 · 版本未定', extensions:'拓展', sync:'同步', logs:'诊断', settings:'设置', tray:'托盘', notify:'通知与反馈', windows:'Windows 窗口'};
  const tabs = {transport:'对话策略','dialog-policy':'对话策略', models:'Agent 配置', single:'单选打样', multiple:'多选打样', global:'全局设置', objects:'项目与对话', file:'文件同步', webdav:'WebDAV 同步', channels:'渠道模型', templates:'模型模版', doctor:'诊断检查', logs:'日志历史', notifications:'通知历史', skills:'Skills', mcp:'MCP', clients:'客户端'};
  function currentContext() {
    if (document.documentElement.dataset.route !== 'components' && document.documentElement.dataset.prototypeDialog === 'model-picker') return {page:'model-picker', tab:document.documentElement.dataset.prototypePickerMode || 'single'};
    if (document.body.classList.contains('windows-mode')) return {page:'windows', tab:''};
    if (document.body.classList.contains('notif-mode')) return {page:'notify', tab:''};
    const page = document.documentElement.dataset.route || 'overview';
    const params = new URLSearchParams(location.hash.split('?')[1] || '');
    let tab = '';
    if (page === 'model-picker') tab = window.modelPickerPrototype?.state().mode || 'single';
    if (page === 'sync') tab = window.syncPrototype?.state().tab || params.get('tab') || 'file';
    if (page === 'conversations') tab = window.conversationPrototype?.state().tab || 'global';
    if (page === 'models') tab = $('[data-mp="tab"][aria-selected="true"]')?.dataset.tab || 'channels';
    if (page === 'logs') tab = $('[data-diag-tab].active')?.dataset.diagTab || 'doctor';
    if (page === 'extensions') tab = window.extensionPrototype?.active() ? window.extensionPrototype.tab() : $('[data-ext-tab].active')?.dataset.extTab || 'skills';
    if (page === 'settings') tab = $('.settings-tabs [data-settings-section].active')?.dataset.settingsSection || params.get('section') || 'general';
    return {page, tab};
  }
  function refresh() {
    const context = currentContext();
    const key = context.page + (context.tab ? ':' + context.tab : '');
    const name = (labels[context.page] || context.page) + (context.tab ? ' / ' + (tabs[context.tab] || $('#protoNoteTitle')?.textContent || context.tab) : '');
    side.dataset.context = key;
    $('#protoGroupMeta').textContent = name;
    $('#protoContextHint').textContent = '仅展示' + name + '的测试场景；全局评审条件在上方折叠区。';
    side.querySelectorAll('[data-proto-contexts]').forEach(card => {
      card.hidden = card.id === 'conversationDemoCard' && !['conversations','capabilities'].includes(window.conversationPrototype?.scope()) || !card.dataset.protoContexts.split(/\s+/).some(value => value === context.page || value === key);
      if(context.page==='settings'&&context.tab==='dialog-policy'&&card.dataset.policyParts)card.hidden=card.hidden||!card.dataset.policyParts.split(/\s+/).includes(window.dialogPolicyPrototype?.state().part||'models');
      if (context.page === 'extensions') {
        const extended = window.extensionPrototype?.active();
        if (card.id === 'extensionManagerDemoCard') card.hidden = !extended;
        else if (extended && card.dataset.protoContexts.includes('extensions')) card.hidden = true;
      } else if (card.id === 'extensionManagerDemoCard') card.hidden = true;
    });
    const contextNote = $('.proto-context-note');
    if (contextNote) contextNote.hidden = ['windows', 'notify', 'annotations'].includes(context.page);
    if (context.page === 'sync') {
      const state = window.syncPrototype?.state();
      $('#sxImportSample').value = state?.importSample || 'encrypted';
      $('#sxMockScenario').value = state?.scenario || 'normal';
      $('#syncMockTitle').textContent = (tabs[context.tab] || '同步') + ' · 同步场景';
      $('#syncMockHint').textContent = context.tab === 'webdav'
        ? '选择范围后点击“同步”，直接查看上传、下载或失败重试结果。'
        : '在同一个导入窗口选择文件与范围后导入，直接更新示例对象。';
    }
  }
  let queued = false;
  function schedule() {
    if (queued) return;
    queued = true;
    requestAnimationFrame(() => {queued = false; refresh();});
  }
  new MutationObserver(schedule).observe(document.documentElement, {attributes:true, attributeFilter:['data-route', 'data-design-scope', 'data-prototype-dialog', 'data-prototype-picker-mode']});
  new MutationObserver(schedule).observe(document.body, {attributes:true, attributeFilter:['class']});
  const main = $('.main');
  if (main) new MutationObserver(schedule).observe(main, {subtree:true, childList:true, attributes:true, attributeFilter:['class', 'aria-selected']});
  window.addEventListener('hashchange', schedule);
  side.addEventListener('change', event => {
    if (event.target.id === 'sxImportSample') window.syncPrototype?.setImportSample(event.target.value);
    if (event.target.id === 'sxMockScenario') window.syncPrototype?.setScenario(event.target.value);
  });
  // 各页面 / Tab 的专属 Mock 控件：同一分组内单选高亮 + 一句原型提示，不写真实配置。
  side.addEventListener('click', event => {
    const button = event.target.closest?.('button');
    if (!button || !side.contains(button)) return;
    if (!button.closest('#protoStdCards')) return;
    const group = button.closest('.pctl-chips, .pctl-grid, .pctl-tiles');
    if (!group) return;
    const flag = ['panelSrc','panelZoom','panelHub','extTarget','extState','extSync','mcpTarget','mcpState','mcpFlag','notifHist','notifHistCat','notifHistAct','settingsScale','settingsPanel','settingsAgent','backupKeep','backupDigest','settingsSkillsSync','settingsMcpConflict','settingsPaths','logCleanup','notifCleanup','cliFlag','cliTarget','upgradeCodex','upgradeManager','upgradeChannel','aboutView','aboutAct']
      .some(key => key in button.dataset);
    if (!flag) return;
    const switches = group.querySelectorAll('button[role="switch"]');
    if (switches.length && button.getAttribute('role') === 'switch') {
      const next = button.getAttribute('aria-checked') !== 'true';
      button.setAttribute('aria-checked', String(next));
      button.classList.toggle('active', next);
      window.toast?.('原型模拟：' + (button.textContent || '').trim() + ' 已' + (next ? '开启' : '关闭') + '；不写真实配置。');
      return;
    }
    if (['notifHistAct', 'aboutAct'].some(key => key in button.dataset)) {
      window.toast?.('原型模拟：' + (button.textContent || '').trim() + '；此处不执行真实动作。');
      return;
    }
    group.querySelectorAll('button').forEach(item => item.classList.toggle('active', item === button));
    if (button.dataset.panelSrc && window.windowPanelMock) window.windowPanelMock(button.dataset.panelSrc);
    if (button.dataset.settingsAgent) { window.modelPrototype?.applyAgentDemo(button.dataset.settingsAgent); return; }
    window.toast?.('原型模拟：' + (button.textContent || '').trim() + '；只切 mock 投影。');
  });
  // S18-02: process health and embedded page loading are different mock facts.
  const panelStates = $('#panelStates');
  if (panelStates) {
    const failed = document.createElement('button');
    failed.type='button';failed.dataset.panelSrc='load-failed';failed.textContent='已运行 · 页面失败';panelStates.appendChild(failed);
    const note=document.createElement('p');note.className='pctl-hint';note.textContent='S18-02反馈方案：页面失败不改为启动失败；重试只加载页面。这里只切换mock，Windows根因及实机验证待后续排查。';panelStates.parentElement.appendChild(note);
  }
  let panelScene='snapshot';
  function showPanelScene(mode) {
    panelScene=mode;
    window.setRoute?.('panel');
    const frame=$('#panelReal'),mock=$('#panelMock'),empty=$('#panelEmpty'),retry=$('#panelRetryBtn');
    const unavailable=mode==='empty'||mode==='load-failed';
    frame?.classList.toggle('show',mode==='snapshot');mock?.classList.toggle('show',mode==='mock');
    if(empty){
      empty.style.display=unavailable?'flex':'none';
      empty.querySelector('strong').textContent=mode==='load-failed'?'官方面板加载失败':'OpenCodex 未运行';
      empty.querySelector('span').textContent=mode==='load-failed'?'OpenCodex 正在运行，内嵌页面未能加载。':'请从概览启动 OpenCodex 后再打开面板。';
      let browser=$('#panelErrorBrowser');if(!browser){browser=document.createElement('button');browser.id='panelErrorBrowser';browser.className='btn ghost';browser.textContent='浏览器打开';browser.addEventListener('click',()=>window.toast?.('原型：将通过浏览器打开面板；不访问真实服务。'));empty.appendChild(browser);}
      browser.hidden=mode!=='load-failed';
    }
    if(retry){retry.textContent=mode==='load-failed'?'重试加载':'返回概览';retry.onclick=()=>{if(mode==='empty')window.setRoute?.('overview');else showPanelScene('snapshot');};}
    if(mode==='load-failed'){window.setState?.('running');$('#toastEl')?.classList.remove('show');}
  }
  window.windowPanelMock=showPanelScene;
  window.prototypePanel = {context:currentContext, refresh, panelScene:()=>panelScene};
  refresh();
})();
