/* 注释标记绝对定位，内容在独立浮层展示；不参与产品布局。 */
(() => {
  const preview = document.getElementById('route-annotations');
  const main = document.querySelector('.main');
  const group = document.getElementById('protoAnnotations');
  if (!main || !group) return;
  const escape = value => String(value).replace(/[&<>"']/g, char => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[char]));
  const icon = '<svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 2.5h7l3 3v8H3zM10 2.5v3h3M5.5 8h5M5.5 10.5h3"/></svg>';
  function card(id, number, title, content, category) {
    return '<div class="proto-annotation" id="'+id+'" data-annotation-title="'+escape(title)+'" data-annotation-category="'+escape(category)+'" data-annotation-number="'+number+'">' +
      '<button type="button" class="annotation-marker" aria-label="原型注释 '+number+'：'+escape(title)+'" aria-expanded="false" aria-controls="annotationPopover">'+icon+'<span>原型</span><span class="annotation-number">'+number+'</span><span class="annotation-marker-plus" aria-hidden="true">＋</span></button>' +
      '<div class="annotation-source" hidden>'+content+'<div class="annotation-footer">'+escape(category)+'</div></div></div>';
  }
  preview.innerHTML =
    '<div class="ap-intro"><div><div class="ap-kicker">ANNOTATION / FLOATING 02</div><h2>注释浮在上面，布局留在原位。</h2><p>点击淡紫色标记展开说明，再点收起。半透明浮层跟随标记位置，右侧目录汇总本页注释。</p></div><span class="ap-version">浮层打样</span></div>' +
    '<div class="ap-specimen"><div class="ap-section-head"><h3>模型配置</h3><span>当前模型 · gpt-5.4</span></div>' +
    '<div class="ap-form-row"><label for="apModelName">显示名称</label><input id="apModelName" class="ap-field" value="日常工作模型"></div>' +
    '<div class="ap-form-row annotation-anchor"><label for="apReasoning">默认推理档位</label><select id="apReasoning" class="ap-field"><option>继承模版 · 均衡</option><option>深度推理</option><option>快速响应</option></select>' +
    card('annotation-inherit', '02', '继承与人工覆盖的边界', '<p>未单独设置的字段沿用模版值。人工覆盖只作用于当前模型；点击「恢复继承」时，移除本层覆盖并重新显示父层结果。</p>', '字段规则 · 模型配置') + '</div>' +
    '<div class="ap-form-row"><label for="apModelAlias">模型标识</label><input id="apModelAlias" class="ap-field" value="gpt-5.4"></div>' +
    '<div class="ap-save-row"><span>修改仅保存到当前配置</span><div class="ap-action-anchor annotation-anchor"><button type="button" class="ap-button ap-primary">保存配置</button>' +
    card('annotation-save', '01', '保存前，先核对本次修改', '<p>点击「保存」后，展示<strong>已保存内容与当前草稿</strong>的逐项差异。确认后才保存；取消时保留当前修改，方便继续编辑。</p><p>这里演示确认流程，实际配置写入与备份仍待实现。</p>', '行为约定 · 保存流程') + '</div></div></div>' +
    '<div class="ap-specimen"><div class="ap-section-head"><h3>配置同步</h3><span>最近同步 · 尚未同步</span></div><p class="ap-description">选择同步方式，核对导入内容与本机配置的差异。</p>' +
    '<div class="ap-sync-row annotation-anchor"><div><strong>WebDAV 同步</strong><p>在多台设备之间同步模型与应用配置</p></div><button type="button" class="ap-button">配置连接</button>' +
    card('annotation-sync', '03', '同步能力的演示范围', '<p>本页使用虚构的配置与账户信息，展示<strong>选择范围 → 比较差异 → 确认应用</strong>的完整交互。</p><p>「同步配置」只切换内存中的演示结果。真实 WebDAV 连接、凭据加密和账户恢复尚未接入；刷新后会回到初始样例。</p><p>包含凭据的导出，需在确认环节展示加密要求与口令输入。</p>', '实现边界 · 配置同步') + '</div></div>' +
    '<div class="ap-specimen ap-navigation"><div class="ap-section-head annotation-anchor"><h3>注释导航</h3>' +
    card('annotation-location', '04', '点击目录，展开并定位注释', '<p>点击右侧「本页注释」中的条目，定位对应标记并打开浮层。每次展开一条说明，避免多张卡片遮挡内容。</p><p>再次点击标记、点击浮层外部或按 Esc，都可以收起。展开与收起不改变字段位置、内容高度或窗口尺寸。</p>', '交互说明 · 注释导航') + '</div><p class="ap-description">四个标记各自贴近说明对象。试试右侧目录，或者暂时隐藏标记查看原始布局。</p></div>' +
    '<p class="ap-footnote">组件打样 · 浮层无需预留高度，可用于现有原型的逐页注释整理。</p>';

  const list = document.getElementById('protoAnnotationList');
  const search = document.getElementById('protoAnnotationSearch');
  const count = document.getElementById('protoAnnotationCount');
  const empty = document.getElementById('protoAnnotationEmpty');
  let entries = [], signature = '', context = '', located = '', highlightTimer, queued = false;
  let active = null, repositionQueued = false;
  const popover = document.createElement('aside');
  popover.id = 'annotationPopover';
  popover.className = 'annotation-popover';
  popover.hidden = true;
  popover.setAttribute('role', 'dialog');
  popover.setAttribute('aria-modal', 'false');
  popover.setAttribute('aria-labelledby', 'annotationPopoverTitle');
  popover.tabIndex = -1;
  document.body.appendChild(popover);
  const visibilityLabel = document.createElement('label');
  visibilityLabel.className = 'annotation-visibility';
  visibilityLabel.innerHTML = '<input type="checkbox" checked>显示页内标记<span>不占布局空间</span>';
  search.closest('label').before(visibilityLabel);
  const visibilityToggle = visibilityLabel.querySelector('input');
  visibilityToggle.addEventListener('change', () => {
    close();
    document.documentElement.classList.toggle('annotations-muted', !visibilityToggle.checked);
  });
  function close(restoreFocus = false) {
    const previous = active;
    active = null;
    popover.hidden = true;
    previous?.querySelector('button').setAttribute('aria-expanded', 'false');
    located = '';
    renderList();
    if (restoreFocus && previous && visible(previous)) previous.querySelector('button').focus({preventScroll:true});
  }
  function position() {
    if (!active || popover.hidden) return;
    const rect = active.getBoundingClientRect();
    // 托盘等路由 .main 会整体隐藏（rect 归零），此时退回视口作为横向/纵向边界。
    const hostEl = active.closest('#modalBody') || main;
    const host = hostEl.getClientRects().length > 0 ? hostEl : null;
    const bounds = host ? host.getBoundingClientRect() : {left: 0, right: innerWidth, top: 0, bottom: innerHeight};
    // 横向仍贴正文容器；纵向以视口为准，标记在窗口外壳上缘被裁剪时浮层仍可显示。
    const left = Math.max(10, bounds.left + 12), right = Math.min(innerWidth - 10, bounds.right - 12);
    const top = 10, bottom = innerHeight - 10;
    // 标记只要与可视区仍有交集就显示浮层（浮层贴容器边缘夹取），完全出界才隐藏。
    if (!(rect.bottom > top && rect.top < bottom && rect.right > left && rect.left < right)) {
      popover.style.visibility = 'hidden';
      return;
    }
    popover.style.visibility = 'visible';
    popover.style.width = Math.min(356, right-left)+'px';
    popover.style.maxHeight = Math.max(80, bottom-top)+'px';
    const height = popover.getBoundingClientRect().height;
    const visibleRect = { top: Math.max(rect.top, top), bottom: Math.min(rect.bottom, bottom) };
    const below = bottom-visibleRect.bottom-10, above = visibleRect.top-top-10;
    const y = below >= height || below >= above ? Math.min(visibleRect.bottom+10, bottom-height) : visibleRect.top-height-10;
    popover.style.left = Math.max(left, Math.min(rect.right-popover.offsetWidth, right-popover.offsetWidth))+'px';
    popover.style.top = Math.max(top, Math.min(y, bottom-height))+'px';
  }
  function schedulePosition() {
    if (repositionQueued) return;
    repositionQueued = true;
    requestAnimationFrame(() => {repositionQueued=false; position();});
  }
  function open(node) {
    close();
    visibilityToggle.checked = true;
    document.documentElement.classList.remove('annotations-muted');
    active = node;
    located = node.id;
    node.querySelector('button').setAttribute('aria-expanded', 'true');
    popover.innerHTML = '<div class="annotation-popover-head"><span class="annotation-label">'+icon+'原型</span><span class="annotation-number">'+escape(node.dataset.annotationNumber)+'</span><span class="annotation-floating-hint">悬浮注释</span><button type="button" class="annotation-close" aria-label="收起注释">×</button></div><h3 id="annotationPopoverTitle">'+escape(node.dataset.annotationTitle)+'</h3><div class="annotation-content">'+node.querySelector('.annotation-source').innerHTML+'</div><div class="annotation-popover-bottom">再次点击标记可收起<span>ESC</span></div>';
    popover.hidden = false;
    position();
    renderList();
  }
  document.addEventListener('click', event => {
    const marker = event.target.closest('.annotation-marker');
    if (marker) {
      const node = marker.closest('.proto-annotation');
      if (active === node) close(); else open(node);
    } else if (event.target.closest('.annotation-close')) close(true);
    // 目录会在定位时重绘，使用事件目标属性识别原来的目录按钮。
    else if (active && !popover.contains(event.target) && !event.target.closest('[data-annotation-target]')) close();
  });
  document.addEventListener('keydown', event => {
    if (event.key === 'Escape' && active) {event.preventDefault(); close(true);}
  });
  document.addEventListener('scroll', schedulePosition, true);
  window.addEventListener('resize', schedulePosition);
  new ResizeObserver(schedulePosition).observe(main);
  function visible(node) {
    // 收起的注释仍注册；页面 / Tab / 弹窗隐藏时移出当页目录。
    return node.getClientRects().length > 0 && !node.closest('[hidden], [inert]');
  }
  function renderList() {
    const query = search.value.trim().toLocaleLowerCase();
    const filtered = entries.filter(node => (node.dataset.annotationTitle+' '+node.dataset.annotationCategory+' '+node.textContent).toLocaleLowerCase().includes(query));
    count.textContent = (query ? filtered.length+' / '+entries.length : entries.length)+' 条';
    list.innerHTML = filtered.map(node => '<button type="button" data-annotation-target="'+escape(node.id)+'" '+(node.id===located?'aria-current="true"':'')+'><span class="annotation-number">'+escape(node.dataset.annotationNumber||'')+'</span><span><strong>'+escape(node.dataset.annotationTitle)+'</strong><small>'+escape(node.dataset.annotationCategory||'原型说明')+'</small></span><span class="annotation-arrow" aria-hidden="true">↗</span></button>').join('');
    empty.hidden = filtered.length > 0;
  }
  function refresh() {
    const nextContext = window.prototypePanel?.context();
    const key = nextContext ? nextContext.page+':'+nextContext.tab : document.documentElement.dataset.route;
    const changed = key !== context;
    if (changed) {context=key; search.value=''; close();}
    // 托盘 section 在 .main 之外（#trayStage），单独纳入扫描。
    entries = [...document.querySelectorAll('.main .proto-annotation, #modalBody .proto-annotation, #trayStage .proto-annotation')].filter(visible).sort((a,b) => Number(a.dataset.annotationNumber)-Number(b.dataset.annotationNumber));
    // 专题覆盖层不属于底下仍保留的产品路由。
    if (['windows','notify'].includes(nextContext?.page)) entries=[];
    if (active && !entries.includes(active)) close();
    group.hidden = entries.length === 0;
    const nextSignature = JSON.stringify(entries.map(node => [node.id,node.dataset.annotationTitle,node.dataset.annotationNumber,node.dataset.annotationCategory,node.textContent]));
    if (!changed && nextSignature===signature) return;
    signature=nextSignature; renderList();
  }
  function schedule() {
    if (queued) return;
    queued=true; requestAnimationFrame(() => {queued=false;refresh();});
  }
  function locate(id) {
    const node = entries.find(item => item.id===id);
    if (!node) return;
    clearTimeout(highlightTimer);
    document.querySelectorAll('.proto-annotation.is-located').forEach(item => item.classList.remove('is-located'));
    for (let parent=node.parentElement; parent && parent!==document.body; parent=parent.parentElement) {
      if (parent.tagName==='DETAILS') parent.open=true;
    }
    open(node);
    node.classList.add('is-located');
    // 逐层选择真正能覆盖目标位置的滚动容器：scrollIntoView 会连 overflow:hidden 的窗口外壳一起滚走，
    // 单一容器又可能滚不出标记（如工具条标记在 .main 顶上方，需 window 配合）。
    const candidates=[];
    for (let p=node.parentElement; p && p!==document.body; p=p.parentElement) {
      const cs=getComputedStyle(p);
      if (/(auto|scroll)/.test(cs.overflowY) && p.scrollHeight>p.clientHeight) candidates.push(p);
    }
    candidates.push(document.scrollingElement||document.documentElement);
    const targetRect0=node.getBoundingClientRect();
    const goal = (() => {
      const mTop = Math.max(24, Math.min(innerHeight-48, (innerHeight-targetRect0.height)/2));
      return targetRect0.top - mTop; // 需要向回滚的像素（正数向下滚视口，负数向上）
    })();
    let scroller=null, best={cost:Infinity, delta:0};
    for (const c of candidates) {
      const maxScroll=Math.max(0, c.scrollHeight-c.clientHeight);
      const delta=Math.max(-c.scrollTop, Math.min(goal, maxScroll-c.scrollTop));
      const cost=Math.abs(goal-delta);
      if (cost<best.cost) {best={cost, delta}; scroller=c;}
      if (cost===0) break;
    }
    if (scroller) {
      const targetRect=node.getBoundingClientRect(), hostRect=scroller.getBoundingClientRect();
      scroller.scrollTo({top: scroller.scrollTop + best.delta, behavior: 'instant'});
    }
    position();
    popover.focus({preventScroll:true});
    highlightTimer=setTimeout(() => node.classList.remove('is-located'),2400);
  }
  search.addEventListener('input',renderList);
  list.addEventListener('click',event => {
    const button=event.target.closest('[data-annotation-target]');
    if (button) locate(button.dataset.annotationTarget);
  });
  const observer = new MutationObserver(schedule);
  observer.observe(main,{subtree:true,childList:true,characterData:true,attributes:true,attributeFilter:['class','hidden','aria-selected','data-annotation-title','data-annotation-category','data-annotation-number']});
  const modal = document.getElementById('modalMask');
  if (modal) observer.observe(modal,{subtree:true,childList:true,attributes:true,attributeFilter:['class','hidden','style']});
  observer.observe(document.documentElement,{attributes:true,attributeFilter:['data-route']});
  observer.observe(document.body,{attributes:true,attributeFilter:['class']});
  const trayStage = document.getElementById('trayStage');
  if (trayStage) observer.observe(trayStage,{subtree:true,childList:true,attributes:true,attributeFilter:['class','hidden']});
  window.addEventListener('hashchange',schedule);
  window.prototypeAnnotations={refresh,locate,entries:()=>entries.map(node=>({id:node.id,title:node.dataset.annotationTitle}))};
  refresh();
})();
