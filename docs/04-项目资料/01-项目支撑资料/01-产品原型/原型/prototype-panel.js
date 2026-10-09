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
  const labels = {overview:'概览', panel:'面板', models:'模型', extensions:'拓展', sync:'同步', logs:'诊断', settings:'设置', tray:'托盘', notify:'通知与反馈', windows:'Windows 窗口'};
  const tabs = {file:'文件同步', webdav:'WebDAV 同步', channels:'渠道模型', templates:'模型模版', doctor:'诊断检查', logs:'日志历史', notifications:'通知历史', skills:'Skills', mcp:'MCP'};
  function currentContext() {
    if (document.body.classList.contains('windows-mode')) return {page:'windows', tab:''};
    if (document.body.classList.contains('notif-mode')) return {page:'notify', tab:''};
    const page = document.documentElement.dataset.route || 'overview';
    const params = new URLSearchParams(location.hash.split('?')[1] || '');
    let tab = '';
    if (page === 'sync') tab = window.syncPrototype?.state().tab || params.get('tab') || 'file';
    if (page === 'models') tab = $('[data-mp="tab"][aria-selected="true"]')?.dataset.tab || 'channels';
    if (page === 'logs') tab = $('[data-diag-tab].active')?.dataset.diagTab || 'doctor';
    if (page === 'extensions') tab = $('[data-ext-tab].active')?.dataset.extTab || 'skills';
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
      card.hidden = !card.dataset.protoContexts.split(/\s+/).some(value => value === context.page || value === key);
    });
    $('.proto-context-note').hidden = ['windows', 'notify'].includes(context.page);
    if (context.page === 'sync') {
      const state = window.syncPrototype?.state();
      $('#sxImportSample').value = state?.importSample || 'encrypted';
      $('#sxMockScenario').value = state?.scenario || 'normal';
      $('#syncMockTitle').textContent = (tabs[context.tab] || '同步') + ' · 应用结果';
      $('#syncMockHint').textContent = context.tab === 'webdav'
        ? '在“检查变化”后核对差异并应用，查看此场景的结果。'
        : '选择示例文件并读取后，确认应用导入内容，查看此场景的结果。';
    }
  }
  let queued = false;
  function schedule() {
    if (queued) return;
    queued = true;
    requestAnimationFrame(() => {queued = false; refresh();});
  }
  new MutationObserver(schedule).observe(document.documentElement, {attributes:true, attributeFilter:['data-route']});
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
    const flag = ['panelSrc','panelZoom','panelHub','extTarget','extState','extSync','mcpTarget','mcpState','mcpFlag','notifHist','notifHistCat','notifHistAct','settingsScale','settingsPanel','backupKeep','backupDigest','settingsSkillsSync','settingsMcpConflict','settingsPaths','logCleanup','notifCleanup','cliFlag','cliTarget','upgradeCodex','upgradeManager','upgradeChannel','aboutView','aboutAct']
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
    window.toast?.('原型模拟：' + (button.textContent || '').trim() + '；只切 mock 投影。');
  });
  window.prototypePanel = {context:currentContext, refresh};
  refresh();
})();
