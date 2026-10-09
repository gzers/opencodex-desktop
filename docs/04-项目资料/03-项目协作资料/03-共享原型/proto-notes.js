/* ProtoNotes —— 原型注释便签通用组件（所有原型页共用，页面只声明内容与锚点）
   ─────────────────────────────────────────────────────────────────────
   用法：
     ProtoNotes.register(group, [ {id, title, body, anchor, dx?, dy?}, ... ])
     ProtoNotes.setGroups(['side', 'overview'])        // 当前激活的注释组
     ProtoNotes.patch(id, { title?, body? })           // 动态更新注释内容
     ProtoNotes.setModalNote('文本' | null)            // 弹窗标题旁的瞬态注释
     ProtoNotes.bindPanel({root, search, list, count}) // 右侧汇总面板（默认绑定约定 id）
   行为：
     · 每条注释在锚点右上角渲染小标记（fixed 定位，不占正文空间、不改布局）。
     · 点击标记展开半透明浮层；再点、外点或 Esc 收起；Esc 优先关浮层。
     · 标记随滚动 / 缩放 / 路由切换实时跟随；锚点不可见时标记随之隐藏。
     · 面板汇总当前激活组的全部注释，支持搜索、点击定位并自动展开。
   主题：跟随 html[data-theme]；特效低档（data-effects="low"）退为实底。 */
window.ProtoNotes = (function () {
  'use strict';
  var groups = new Map();      // groupKey -> notes[]
  var byId = new Map();        // id -> note
  var activeGroups = [];
  var openId = null;
  var layer = null, pop = null, popTitle = null, popBody = null;
  var panel = null;            // {root, search, list, count}
  var searchQuery = '';
  var modalNoteId = null;
  var scheduled = false;

  function q(sel) { return document.querySelector(sel); }
  function esc(s) {
    return String(s == null ? '' : s).replace(/[&<>"]/g, function (c) {
      return { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c];
    });
  }
  function anchorEl(n) { return typeof n.anchor === 'string' ? q(n.anchor) : n.anchor; }
  function visibleEl(n) {
    var el = anchorEl(n);
    return el && el.getClientRects().length > 0 ? el : null;
  }
  function activeNotes() {
    var out = [];
    activeGroups.forEach(function (g) { (groups.get(g) || []).forEach(function (n) { out.push(n); }); });
    return out;
  }

  function ensureDom() {
    if (layer || !document.body) return;
    layer = document.createElement('div');
    layer.className = 'pn-layer';
    document.body.appendChild(layer);
    pop = document.createElement('div');
    pop.className = 'pn-pop';
    pop.hidden = true;
    pop.setAttribute('role', 'dialog');
    pop.setAttribute('aria-label', '原型注释');
    pop.innerHTML = '<div class="pn-pop-head"><strong class="pn-pop-title"></strong>' +
      '<button type="button" class="pn-pop-close" aria-label="收起注释">×</button></div>' +
      '<div class="pn-pop-body"></div>';
    layer.appendChild(pop);
    popTitle = pop.querySelector('.pn-pop-title');
    popBody = pop.querySelector('.pn-pop-body');
    pop.querySelector('.pn-pop-close').addEventListener('click', close);
    layer.addEventListener('click', function (e) {
      var b = e.target.closest('.pn-badge');
      if (!b) return;
      e.stopPropagation();
      toggle(b.dataset.pnId);
    });
    window.addEventListener('resize', schedule);
    document.addEventListener('scroll', schedule, { capture: true, passive: true });
    document.addEventListener('click', onDocClick, true);
    document.addEventListener('keydown', onKeydown, true);
    if (window.MutationObserver) {
      new MutationObserver(schedule).observe(document.body, {
        attributes: true, attributeFilter: ['class', 'style', 'hidden'], subtree: true
      });
    }
    if (document.fonts && document.fonts.ready && document.fonts.ready.then) {
      document.fonts.ready.then(schedule).catch(function () {});
    }
  }

  function schedule() {
    if (!layer || scheduled) return;
    scheduled = true;
    requestAnimationFrame(function () { scheduled = false; refresh(); });
  }

  function refresh() {
    if (!layer) return;
    var seen = new Map(); // 同一锚点多个标记时纵向错开
    activeNotes().forEach(function (n) {
      var el = visibleEl(n);
      var badge = n.__badge;
      if (!el) {
        if (badge) badge.remove();
        n.__badge = null;
        if (openId === n.id) close();
        return;
      }
      if (!badge) {
        badge = document.createElement('button');
        badge.type = 'button';
        badge.className = 'pn-badge';
        badge.textContent = 'i';
        layer.appendChild(badge);
        n.__badge = badge;
      }
      badge.dataset.pnId = n.id;
      badge.title = n.title || '注释';
      badge.setAttribute('aria-label', '注释：' + (n.title || ''));
      var r = el.getBoundingClientRect();
      var left = r.right - 8 + (n.dx || 0);
      var top = r.top - 8 + (n.dy || 0);
      var k = Math.round(left) + ',' + Math.round(top);
      var c = seen.get(k) || 0;
      seen.set(k, c + 1);
      if (c) top += 20 * c;
      badge.style.left = left + 'px';
      badge.style.top = top + 'px';
      badge.classList.toggle('is-open', openId === n.id);
    });
    if (openId && byId.get(openId)) placePop(); else close();
    renderPanel();
  }

  function refreshBadges() {
    byId.forEach(function (n) {
      if (n.__badge) n.__badge.classList.toggle('is-open', openId === n.id);
    });
  }

  function placePop() {
    var n = byId.get(openId);
    if (!n || !n.__badge) { close(); return; }
    pop.hidden = false;
    popTitle.textContent = n.title || '注释';
    popBody.innerHTML = n.body || '';
    var pw = Math.min(340, window.innerWidth - 24);
    var ph = pop.offsetHeight || 180;
    var bx = n.__badge.offsetLeft, by2 = n.__badge.offsetTop;
    var left = (bx + 18 + pw <= window.innerWidth - 12) ? bx + 18 : Math.max(12, bx - pw - 12);
    var top = Math.max(12, Math.min(by2 - 10, window.innerHeight - ph - 12));
    pop.style.left = left + 'px';
    pop.style.top = top + 'px';
    pop.style.maxHeight = Math.min(420, window.innerHeight - 24) + 'px';
  }

  function open(id) {
    var n = byId.get(id);
    ensureDom();
    if (!n || !visibleEl(n)) return;
    openId = id;
    placePop();
    refreshBadges();
    renderPanel();
  }
  function close() {
    openId = null;
    if (pop) pop.hidden = true;
    refreshBadges();
    renderPanel();
  }
  function toggle(id) { if (openId === id) close(); else open(id); }

  function onDocClick(e) {
    if (!pop || pop.hidden) return;
    if (pop.contains(e.target)) return;
    if (e.target.closest && e.target.closest('.pn-badge')) return;
    close();
  }
  function onKeydown(e) {
    if (e.key !== 'Escape' || !pop || pop.hidden) return;
    e.preventDefault();
    e.stopImmediatePropagation(); // 浮层先收起，不影响弹窗自身的 Esc 逻辑
    close();
  }

  function stripHtml(html) {
    var d = document.createElement('div');
    d.innerHTML = html || '';
    return (d.textContent || '').replace(/\s+/g, ' ').trim().slice(0, 64);
  }
  function renderPanel() {
    if (!panel || !panel.list) return;
    var notes = activeNotes().filter(visibleEl);
    var kw = searchQuery.trim().toLowerCase();
    var shown = kw ? notes.filter(function (n) {
      return ((n.title || '') + ' ' + stripHtml(n.body)).toLowerCase().indexOf(kw) >= 0;
    }) : notes;
    if (panel.count) panel.count.textContent = notes.length ? notes.length + ' 条' : '0 条';
    // 只在内容真正变化时重建列表：避免 MutationObserver 与 rAF 循环互相触发，
    // 导致面板项在点击命中前被换掉（元素 detached）。
    var sig = shown.map(function (n) { return n.id + ':' + (openId === n.id ? '1' : '0'); }).join('|');
    if (panel.list.dataset.pnSig !== sig) {
      panel.list.dataset.pnSig = sig;
      panel.list.innerHTML = shown.length ? shown.map(function (n) {
        return '<button type="button" class="pn-item' + (openId === n.id ? ' is-open' : '') +
          '" data-pn-goto="' + esc(n.id) + '"><b>' + esc(n.title || '注释') + '</b><span>' +
          esc(stripHtml(n.body)) + '</span></button>';
      }).join('') : '<p class="pn-empty">' + (kw ? '没有匹配的注释。' : '当前页面暂无注释。') + '</p>';
    }
  }

  function locate(id) {
    var n = byId.get(id);
    if (!n) return;
    var el = anchorEl(n);
    if (!el) return;
    ensureDom();
    openId = id;
    el.scrollIntoView({
      block: 'center',
      behavior: window.matchMedia && matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth'
    });
    setTimeout(function () {
      if (openId === id && byId.get(id)) { placePop(); refreshBadges(); }
    }, 260);
    refreshBadges();
    renderPanel();
  }

  function register(group, notes) {
    var arr = groups.get(group) || [];
    notes.forEach(function (n) {
      var copy = Object.assign({ dx: 0, dy: 0 }, n);
      if (byId.has(copy.id)) { // 重复注册按覆盖处理，便于场景重放
        var old = byId.get(copy.id);
        if (old.__badge) old.__badge.remove();
        var gi = (groups.get(group) || []).indexOf(old);
        if (gi >= 0) groups.get(group).splice(gi, 1);
      }
      byId.set(copy.id, copy);
      arr.push(copy);
    });
    groups.set(group, arr);
    schedule();
  }
  function removeNote(id) {
    var n = byId.get(id);
    if (!n) return;
    if (n.__badge) n.__badge.remove();
    byId.delete(id);
    if (openId === id) close();
    groups.forEach(function (arr) {
      var i = arr.indexOf(n);
      if (i >= 0) arr.splice(i, 1);
    });
  }
  function setGroups(list) {
    activeGroups = list.slice();
    if (openId && !activeNotes().some(function (n) { return n.id === openId; })) close();
    schedule();
  }
  function patch(id, fields) {
    var n = byId.get(id);
    if (!n) return;
    Object.assign(n, fields || {});
    if (openId === id && pop && !pop.hidden) placePop();
    schedule();
  }
  function setModalNote(text) {
    ensureDom();
    if (text == null) { removeNote(modalNoteId); modalNoteId = null; return; }
    if (!modalNoteId) {
      modalNoteId = '__modal__';
      register('modal', [{ id: modalNoteId, title: '原型演示', body: text, anchor: '#modalTitle', dx: -8, dy: 6 }]);
    } else {
      patch(modalNoteId, { body: text });
    }
    if (activeGroups.indexOf('modal') < 0) activeGroups.push('modal');
    schedule();
  }
  function bindPanel(opts) {
    panel = {
      root: q(opts.root) || null,
      search: q(opts.search) || null,
      list: q(opts.list) || null,
      count: q(opts.count) || null
    };
    if (panel.search && !panel.search.dataset.pnBound) {
      panel.search.dataset.pnBound = '1';
      panel.search.addEventListener('input', function () { searchQuery = panel.search.value; renderPanel(); });
    }
    if (panel.list && !panel.list.dataset.pnBound) {
      panel.list.dataset.pnBound = '1';
      panel.list.addEventListener('click', function (e) {
        var b = e.target.closest('[data-pn-goto]');
        if (b) locate(b.dataset.pnGoto);
      });
    }
    renderPanel();
  }

  function init() {
    ensureDom();
    if (!panel) bindPanel({ root: '#protoNotesPanel', search: '#protoNotesSearch', list: '#protoNotesList', count: '#protoNotesCount' });
    refresh();
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', init);
  else init();

  return {
    register: register,
    hasGroup: function (g) { return groups.has(g) && groups.get(g).length > 0; },
    setGroups: setGroups,
    patch: patch,
    open: open,
    close: close,
    locate: locate,
    setModalNote: setModalNote,
    bindPanel: bindPanel,
    refresh: schedule
  };
})();
