/* 原型视图适配：与业务页共用结构，controller/状态由调用者持有。 */
(() => {
 const esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 window.PrototypeNodeView={
  row:o=>'<div class="mp-node-picker-row" data-node-row="'+esc(o.id)+'"><button id="'+esc(o.choiceId)+'" class="mp-node-picker-choice" '+o.choiceAttrs+' aria-pressed="'+!!o.selected+'" title="'+esc(o.title)+'" '+(o.disabled?'disabled':'')+'><span class="mp-node-picker-check" aria-hidden="true">'+(o.selected?'✓':'')+'</span><span class="mp-node-picker-name">'+esc(o.name)+'</span></button><span id="'+esc(o.speedId)+'" class="mp-node-picker-speed" role="status">'+o.speed+'</span>'+o.test+'</div>',
  picker:o=>'<div class="mp-node-picker-head"><strong id="'+esc(o.titleId)+'">'+esc(o.title)+'</strong>'+o.close+'</div><div class="mp-node-picker-toolbar">'+o.all+'<div id="'+esc(o.summaryId)+'" role="status">'+o.summary+'</div></div><div class="mp-node-picker-list" role="group" aria-label="选择节点与单节点检测">'+o.rows+'</div>'
 };
 // 托盘以既有菜单DOM作为唯一维护模板；克隆时移除全部业务行为、ID和状态订阅。
 // 不加载另一套页面，也不会调用原托盘的全局controller。
 window.PrototypePlatformViews={tray(os,state,settings={}){
  const source=document.querySelector('#route-tray .tray-menu.'+(os==='windows'?'windows':'mac'));
  if(!source)return '';const menu=source.cloneNode(true);
  menu.classList.remove('hidden');menu.classList.add('open');
  const labels={stopped:'未运行',running:'运行中',starting:'启动中',failed:'启动失败',at_risk:'未运行'};
  menu.querySelector('[data-tray-label]').textContent=labels[state]||labels.stopped;
  menu.querySelectorAll('button').forEach(b=>{
   const action=b.dataset.trayAction||b.dataset.traySetting;
   b.dataset.ctTray=action;
   const blocked=state==='starting'?['start','stop','restart','open-panel'].includes(action):state==='running'?action==='start':['stop','restart','open-panel'].includes(action);
   b.disabled=blocked;const hint=b.querySelector('[data-tray-hint]');
   if(hint)hint.textContent=blocked?(state==='starting'?'请等待':action==='start'?'已运行':'需运行'):'可用';
   if(b.hasAttribute('data-tray-setting')){const value=!!settings[action];b.setAttribute('aria-checked',String(value));b.querySelector('.toggle-mark').textContent=value?'开启':'关闭';}
  });
  [menu,...menu.querySelectorAll('*')].forEach(el=>{for(const a of [...el.attributes])if(a.name==='id'||a.name.startsWith('data-tray-')||a.name==='data-prototype-action')el.removeAttribute(a.name);});
  return menu.outerHTML;
 }};
})();
