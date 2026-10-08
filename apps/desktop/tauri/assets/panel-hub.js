// 官方面板子 WebView 内注入的「面板快捷操作」浮层。
//
// 为什么注入到面板页而不是放在管理器 DOM 里：原生子 WebView 与主 WebView 不是同一合成层，
// 管理器 DOM 的浮层会被官方面板盖住（R-17）。因此浮层随面板一起渲染，只叠加在官方页面之上，
// 不重绘、不复制官方页面本身。
//
// 与原型 `#panelHubToggle` 对齐（IMP-07）：
// - 单一品牌悬浮球（品牌 logo），**点击**展开菜单（不再鼠标悬浮自动展开），保留悬浮视觉反馈；
// - 玻璃 + 分层阴影 + 顶部内高光＝高阶层级；低特效档退实底（见 effects.css）；
// - 按住进入拖拽模式，拖拽后不触发展开；键盘（Enter/Space）仍可切换。
//
// 主题来自管理器：面板页收不到管理器的 CSS 令牌，浮层自带的玻璃取值需与 UI规范.md 的通知玻璃一致。
(() => {
  const rootId = 'ocxd-panel-hub';
  // 只管「面板自身」的快捷操作：缩放 / 重载 / 浏览器。
  // 跨路由导航（概览等）由常驻左侧图标栏承担，不在这里重复放一份。
  const ACTIONS = ['zoom-out', 'zoom-in', 'reload', 'browser'];
  const emit = (action) => {
    if (!ACTIONS.includes(action)) return;
    window.location.href = `ocxd-panel://${action}`;
  };

  const LOGO = '<svg class="hub-logo" width="22" height="22" viewBox="0 0 24 24" aria-hidden="true"><defs><linearGradient id="ocxdHubGrad" gradientUnits="userSpaceOnUse" x1="12" y1="3" x2="12" y2="21"><stop stop-color="#B1A7FF"/><stop offset=".5" stop-color="#7A9DFF"/><stop offset="1" stop-color="#3941FF"/></linearGradient><mask id="ocxdHubKnock" maskUnits="userSpaceOnUse" x="0" y="0" width="24" height="24"><rect width="24" height="24" fill="#fff"/><svg x="7.5" y="7.5" width="9" height="9" viewBox="0 0 1024 1024" fill="#000"><path d="M512 150m-150 0a150 150 0 1 0 300 0 150 150 0 1 0-300 0Z"/><path d="M150 874m-150 0a150 150 0 1 0 300 0 150 150 0 1 0-300 0Z"/><path d="M150 699c10 0 19.8 0.9 29.4 2.5-18-43.5-27.1-89.6-27.1-137.2 0-48.6 9.5-95.7 28.2-140 18.1-42.8 44.1-81.3 77.1-114.4 30.1-30.1 64.7-54.3 103-72.1-15-25.7-23.6-55.8-23.6-87.8 0-3.7 0.1-7.4 0.4-11.1C170.1 207.7 52.3 372.2 52.3 564.3c0 52.7 8.9 103.3 25.2 150.4C99.6 704.6 124.1 699 150 699z"/><path d="M874 874m-150 0a150 150 0 1 0 300 0 150 150 0 1 0-300 0Z"/><path d="M699 871.7c-15 9.2-30.7 17.2-47 24.1-44.3 18.7-91.4 28.2-140 28.2s-95.7-9.5-140-28.2c-16.3-6.9-32-15-47-24.1v2.3c0 35.6-10.6 68.6-28.8 96.3C360.5 1004.6 434 1024 512 1024s151.5-19.4 215.8-53.7C709.6 942.6 699 909.6 699 874v-2.3zM766.3 310c33.1 33.1 59 71.5 77.1 114.4 18.7 44.3 28.2 91.4 28.2 140 0 47.6-9.1 93.7-27.1 137.2 9.6-1.6 19.4-2.5 29.4-2.5 25.9 0 50.4 5.6 72.5 15.7 16.3-47.1 25.2-97.7 25.2-150.4 0-192.1-117.8-356.6-285.1-425.4 0.2 3.7 0.4 7.3 0.4 11.1 0 32-8.6 62.1-23.7 87.9 38.4 17.7 73 41.8 103.1 72z"/></svg></mask></defs><path mask="url(#ocxdHubKnock)" fill="url(#ocxdHubGrad)" d="M9.064 3.344a4.578 4.578 0 012.285-.312c1 .115 1.891.54 2.673 1.275.01.01.024.017.037.021a.09.09 0 00.043 0 4.55 4.55 0 013.046.275l.047.022.116.057a4.581 4.581 0 012.188 2.399c.209.51.313 1.041.315 1.595a4.24 4.24 0 01-.134 1.223.123.123 0 00.03.115c.594.607.988 1.33 1.183 2.17.289 1.425-.007 2.71-.887 3.854l-.136.166a4.548 4.548 0 01-2.201 1.388.123.123 0 00-.081.076c-.191.551-.383 1.023-.74 1.494-.9 1.187-2.222 1.846-3.711 1.838-1.187-.006-2.239-.44-3.157-1.302a.107.107 0 00-.105-.024c-.388.125-.78.143-1.204.138a4.441 4.441 0 01-1.945-.466 4.544 4.544 0 01-1.61-1.335c-.152-.202-.303-.392-.414-.617a5.81 5.81 0 01-.37-.961 4.582 4.582 0 01-.014-2.298.124.124 0 00.006-.056.085.085 0 00-.027-.048 4.467 4.467 0 01-1.034-1.651 3.896 3.896 0 01-.251-1.192 5.189 5.189 0 01.141-1.6c.337-1.112.982-1.985 1.933-2.618.212-.141.413-.251.601-.33.215-.089.43-.164.646-.227a.098.098 0 00.065-.066 4.51 4.51 0 01.829-1.615 4.535 4.535 0 011.837-1.388z"/></svg>';

  const styles = (id) => `
    #${id}{position:fixed;right:20px;bottom:18px;z-index:2147483647;
      font-family:-apple-system,BlinkMacSystemFont,"PingFang SC",system-ui,sans-serif;
      --hub-fill:rgba(244,245,249,.52);--hub-border:rgba(13,13,13,.13);
      --hub-text:#0d0d0d;--hub-muted:#6e6e6e;--hub-press:rgba(13,13,13,.07);
      --hub-shadow:0 2px 6px rgba(16,24,40,.06),0 14px 34px rgba(16,24,40,.16),inset 0 1px 0 rgba(255,255,255,.55);
      --hub-shadow-hover:0 3px 8px rgba(16,24,40,.10),0 18px 42px rgba(16,24,40,.20),inset 0 1px 0 rgba(255,255,255,.68)}
    #${id}[data-theme="dark"]{
      --hub-fill:rgba(34,36,42,.56);--hub-border:rgba(255,255,255,.13);
      --hub-text:#ececec;--hub-muted:#a6a6a6;--hub-press:rgba(255,255,255,.10);
      --hub-shadow:0 2px 6px rgba(0,0,0,.25),0 14px 34px rgba(0,0,0,.40),inset 0 1px 0 rgba(255,255,255,.12);
      --hub-shadow-hover:0 3px 8px rgba(0,0,0,.30),0 18px 42px rgba(0,0,0,.48),inset 0 1px 0 rgba(255,255,255,.18)}
    #${id} .hub-toggle{position:relative;width:38px;height:38px;display:grid;place-items:center;padding:0;
      cursor:grab;border:1px solid var(--hub-border);border-radius:50%;background:var(--hub-fill);color:var(--hub-text);
      box-shadow:var(--hub-shadow);transition:transform .16s ease,box-shadow .16s ease,background .16s ease;
      -webkit-backdrop-filter:saturate(1.5) blur(8px);backdrop-filter:saturate(1.5) blur(8px)}
    #${id} .hub-toggle:hover{transform:translateY(-1px);box-shadow:var(--hub-shadow-hover);background:var(--hub-press)}
    #${id} .hub-toggle:active{cursor:grabbing}
    #${id} .hub-logo{display:block;filter:drop-shadow(0 1px 2px rgba(57,65,255,.18));pointer-events:none}
    #${id} .sr-only{position:absolute;width:1px;height:1px;margin:-1px;padding:0;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap;border:0}
    #${id} .hub-menu{display:none;position:absolute;right:0;bottom:46px;width:172px;padding:8px;
      border:1px solid var(--hub-border);border-radius:14px;background:var(--hub-fill);color:var(--hub-text);
      box-shadow:0 20px 48px rgba(16,24,40,.18),inset 0 1px 0 rgba(255,255,255,.45);
      -webkit-backdrop-filter:saturate(1.5) blur(8px);backdrop-filter:saturate(1.5) blur(8px)}
    #${id}.open .hub-menu{display:block}
    #${id} .hub-title{display:flex;align-items:baseline;justify-content:space-between;margin-bottom:7px}
    #${id} .hub-title b{font-size:13px;font-variant-numeric:tabular-nums}
    #${id} .hub-title span{color:var(--hub-muted);font-size:10px}
    #${id} .hub-zoom{display:grid;grid-template-columns:30px minmax(0,1fr) 30px;align-items:center;gap:4px;margin-bottom:7px}
    #${id} .hub-zoom button{min-width:30px;min-height:27px;border:1px solid var(--hub-border);border-radius:8px;
      background:var(--hub-press);color:var(--hub-text);font-size:14px;cursor:pointer}
    #${id} .hub-readout{display:grid;place-items:center;min-height:27px;border-radius:8px;
      background:var(--hub-press);color:var(--hub-text);font-size:11px;font-weight:700;font-variant-numeric:tabular-nums}
    #${id} .hub-sep{height:1px;margin:5px 0;background:var(--hub-border)}
    #${id} .hub-item{display:block;width:100%;min-height:29px;padding:5px 9px;border:0;border-radius:8px;
      background:transparent;color:var(--hub-text);font:inherit;font-size:12px;text-align:left;cursor:pointer}
    #${id} .hub-item:hover{background:var(--hub-press)}
    #${id} .hub-item:focus-visible,#${id} .hub-toggle:focus-visible,#${id} .hub-zoom button:focus-visible{
      outline:2px solid var(--hub-text);outline-offset:2px}
    #${id}:not([data-effects="high"]) *{transition:none!important;animation:none!important}
    #${id}[data-effects="low"] .hub-menu,#${id}[data-effects="low"] .hub-toggle{
      background:#f4f5f9;backdrop-filter:none;-webkit-backdrop-filter:none}
    #${id}[data-effects="low"][data-theme="dark"] .hub-menu,#${id}[data-effects="low"][data-theme="dark"] .hub-toggle{background:#22242a}
  `;

  const mount = () => {
    if (document.getElementById(rootId)) return;
    const root = document.createElement('div');
    root.id = rootId;
    root.setAttribute('data-theme', 'light');
    root.setAttribute('data-effects', 'high');
    root.innerHTML = [
      '<button type="button" class="hub-toggle" aria-haspopup="menu" aria-expanded="false" title="面板快捷操作">',
      LOGO, '<span class="sr-only" aria-live="polite"></span>',
      '</button>',
      '<div class="hub-menu" role="menu" aria-label="面板快捷操作菜单">',
      '<div class="hub-title"><b>界面 100%</b><span>缩放</span></div>',
      '<div class="hub-zoom" role="group" aria-label="界面缩放">',
      '<button type="button" data-action="zoom-out" title="缩小界面缩放">−</button>',
      '<span class="hub-readout" aria-live="polite">100%</span>',
      '<button type="button" data-action="zoom-in" title="放大界面缩放">+</button>',
      '</div>',
      '<div class="hub-sep"></div>',
      '<button type="button" class="hub-item" role="menuitem" data-action="reload">重载</button>',
      '<button type="button" class="hub-item" role="menuitem" data-action="browser">浏览器</button>',
      '</div>',
    ].join('');

    const style = document.createElement('style');
    style.textContent = styles(rootId);
    root.append(style);

    const toggle = root.querySelector('.hub-toggle');
    const badge = root.querySelector('.sr-only');
    const title = root.querySelector('.hub-title b');
    const readout = root.querySelector('.hub-readout');

    const setOpen = (open) => {
      root.classList.toggle('open', open);
      toggle.setAttribute('aria-expanded', open ? 'true' : 'false');
    };

    root.addEventListener('click', (event) => {
      const button = event.target.closest('[data-action]');
      if (!button) return;
      emit(button.dataset.action);
    });
    document.addEventListener('click', (event) => {
      if (!root.contains(event.target)) setOpen(false);
    });
    document.addEventListener('keydown', (event) => {
      // 只有键盘路径（Esc 关闭）才把焦点送回悬浮球；鼠标点击/拖拽关闭**不回焦**，
      // 否则 WKWebView 会把程序化 focus 判为 :focus-visible，点完就留下一圈焦点环
      // （用户报告「悬浮按钮点击有一圈黑色边」）。原型同样只在 Esc 时 focus。
      if (event.key === 'Escape' && root.classList.contains('open')) {
        setOpen(false);
        toggle.focus();
      }
    });

    // 交互：轻点展开/收起；按住拖动移动；拖拽过则不展开。
    let dragging = false, moved = false, startX = 0, startY = 0, originX = 0, originY = 0;
    toggle.addEventListener('click', (event) => {
      // 键盘 Enter/Space 触发的 click（detail===0）；鼠标轻点由 pointerup 处理。
      if (event.detail > 0) return;
      event.stopPropagation();
      setOpen(!root.classList.contains('open'));
    });
    toggle.addEventListener('pointerdown', (event) => {
      if (event.pointerType !== 'mouse') return;
      event.preventDefault();
      toggle.setPointerCapture(event.pointerId);
      dragging = true; moved = false;
      const rect = root.getBoundingClientRect();
      originX = rect.left; originY = rect.top;
      startX = event.clientX; startY = event.clientY;
    });
    toggle.addEventListener('pointermove', (event) => {
      if (!dragging) return;
      const dx = event.clientX - startX, dy = event.clientY - startY;
      if (!moved && (Math.abs(dx) > 2 || Math.abs(dy) > 2)) { moved = true; setOpen(false); }
      if (!moved) return;
      const rect = root.getBoundingClientRect();
      const x = Math.min(Math.max(12, originX + dx), window.innerWidth - rect.width - 12);
      const y = Math.min(Math.max(12, originY + dy), window.innerHeight - rect.height - 12);
      root.style.left = x + 'px'; root.style.top = y + 'px';
      root.style.right = 'auto'; root.style.bottom = 'auto';
    });
    toggle.addEventListener('pointerup', (event) => {
      if (!dragging) return;
      dragging = false;
      if (event.pointerType !== 'mouse') return;
      if (!moved) { event.stopPropagation(); setOpen(!root.classList.contains('open')); }
    });
    toggle.addEventListener('pointercancel', () => { dragging = false; moved = false; });

    const applyState = (detail) => {
      const state = detail || window.__ocxdPanelState || {};
      const scale = Number(state.scale);
      if (Number.isFinite(scale) && scale > 0) {
        const text = `${Math.round(scale)}%`;
        readout.textContent = text;
        title.textContent = `界面 ${text}`;
        badge.textContent = text;
      }
      if (state.theme === 'dark' || state.theme === 'light') {
        root.setAttribute('data-theme', state.theme);
      }
      root.setAttribute('data-effects', ['high','mid','low'].includes(state.effects) ? state.effects : 'high');
      // 明暗同步：沿用官方面板自己的存储键与 `data-theme`，不注入样式。
      // `system` 表示交回系统外观，与官方页面的语义一致。
      if (state.themeSetting === 'light' || state.themeSetting === 'dark' || state.themeSetting === 'system') {
        try {
          const page = document.documentElement;
          if (state.themeSetting === 'system') {
            window.localStorage.removeItem('ocx-theme');
            page.removeAttribute('data-theme');
          } else {
            window.localStorage.setItem('ocx-theme', state.themeSetting);
            page.setAttribute('data-theme', state.themeSetting);
          }
        } catch (error) {}
      }
    };
    window.addEventListener('ocxd-panel-state', (event) => applyState(event.detail));
    applyState(null);

    document.documentElement.append(root);
  };

  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', mount, { once: true });
  else mount();
  setTimeout(mount, 800);
})();
