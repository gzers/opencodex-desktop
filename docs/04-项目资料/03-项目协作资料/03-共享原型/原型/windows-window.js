/* Windows 专题：复用主原型，只模拟外观/生命周期；不调用 Tauri 或系统 API。 */
(() => {
  const body=document.body, root=document.documentElement;
  const stage=document.querySelector('.stage'), frame=document.querySelector('.window');
  const titlebar=document.querySelector('.titlebar');
  const state={layout:'integrated',platform:'win11',transparency:'on',focus:'active',theme:'light',close:'tray',size:'1180',visibility:'visible',maximized:false};
  state.theme=root.dataset.theme==='dark'?'dark':'light';
  let previous=null;
  const caption=document.createElement('div');
  caption.className='windows-caption';
  caption.innerHTML='<div class="windows-drag" title="双击模拟最大化 / 还原"><svg viewBox="0 0 24 24" aria-hidden="true"><use href="#tray-logo"/></svg><span>OpenCodeX Desktop</span></div><div class="windows-buttons"><button type="button" data-win-action="minimize" aria-label="最小化" title="最小化"><svg viewBox="0 0 16 16"><path d="M3 8.5h10"/></svg></button><button type="button" data-win-action="maximize" aria-label="最大化" title="最大化"><svg viewBox="0 0 16 16"><rect x="3.5" y="3.5" width="9" height="9"/></svg></button><button type="button" data-win-action="close" aria-label="关闭窗口" title="关闭窗口"><svg viewBox="0 0 16 16"><path d="m3.5 3.5 9 9m0-9-9 9"/></svg></button></div>';
  titlebar.append(caption);
  const menu=document.createElement('div');menu.className='windows-legacy-menu';
  menu.textContent='进程　 编辑　 视图　 诊断';menu.setAttribute('aria-label','现有 Windows 原生菜单的静态对照');titlebar.append(menu);
  const hidden=document.createElement('div');hidden.className='windows-hidden';hidden.hidden=true;
  hidden.innerHTML='<svg viewBox="0 0 24 24" aria-hidden="true"><use href="#tray-logo"/></svg><strong id="winHiddenTitle"></strong><p id="winHiddenHint"></p><button class="btn" type="button" data-win-action="reopen">重新打开窗口（模拟）</button>';
  stage.append(hidden);
  const card=document.createElement('section');card.id='windowsDemoCard';card.className='proto-card';
  card.setAttribute('aria-label','Windows 窗口专题测试器');
  card.innerHTML='<div class="proto-note-title">Windows 窗口专题</div>'+[
    ['layout','顶部方案',[['integrated','单行整合'],['legacy','现有双栏']]],
    ['platform','系统兼容',[['win11','Windows 11'],['win10','Windows 10']]],
    ['theme','主题',[['light','浅色'],['dark','深色']]],
    ['transparency','透明效果 / 降级',[['on','允许'],['off','关闭 / 不可用']]],
    ['focus','窗口焦点',[['active','活动'],['inactive','非活动']]],
    ['close','关闭窗口后保持代理运行',[['tray','开启 · 留托盘'],['exit','关闭 · 退出']]],
    ['size','窗口尺寸',[['1180','1180 × 760'],['960','960 × 640']]]
  ].map(([key,label,values])=>'<div class="pctl-sub">'+label+'</div><div class="pctl-chips" role="group" aria-label="'+label+'">'+values.map(([value,text])=>'<button type="button" data-win-key="'+key+'" data-win-value="'+value+'">'+text+'</button>').join('')+'</div>').join('')+
  '<p class="pctl-hint" id="winMaterialHint"></p><div class="pctl-sub">窗口状态</div><p class="pctl-hint" id="winStatus" role="status" aria-live="polite"></p><div class="pctl-grid"><button type="button" data-win-action="reopen">重新打开</button><button type="button" data-win-action="reset">复位场景</button></div><button class="pctl-back" type="button" data-win-action="leave">返回常规原型</button>';
  document.getElementById('protoStdCards').before(card);
  if(window.ProtoNotes){
    ProtoNotes.register('windows',[
      {id:'win-topic',title:'Windows 窗口专题',body:'独立平台专题，复用当前产品内容。所有窗口行为均为浏览器模拟，不调用 Tauri 或系统 API。',anchor:'#windowsDemoCard'},
      {id:'win-menu',title:'菜单功能去向',body:'进程启停 → 概览 / 托盘；编辑 → 标准快捷键与输入框上下文菜单；缩放 → 应用内缩放入口；诊断 → 左侧诊断。本轮只展示窗口，生产接线待实施。',anchor:'#windowsDemoCard',dx:4,dy:30},
      {id:'win-acceptance',title:'原生验收仍需执行',body:'拖拽、边缘缩放、双击最大化、Snap 布局、Alt+F4、DPI 与应用缩放、关闭到托盘及恢复，均须在 Windows 实机验收。网页不能证明这些行为或真实 Mica。',anchor:'#windowsDemoCard',dx:8,dy:60}
    ]);
  }

  function topicURL(on){const url=new URL(location.href);if(on)url.searchParams.set('topic','windows');else url.searchParams.delete('topic');history.replaceState(null,'',url);}
  function render(message){
    if(!body.classList.contains('windows-mode'))return;
    const solid=state.platform==='win10'||state.transparency==='off'||state.focus==='inactive'||state.layout==='legacy';
    body.dataset.winLayout=state.layout;body.dataset.winMaterial=solid?'solid':'mica';body.dataset.winFocus=state.focus;
    root.dataset.theme=state.theme;
    frame.classList.toggle('windows-maximized',state.maximized);
    frame.hidden=state.visibility!=='visible';hidden.hidden=state.visibility==='visible';
    frame.inert=frame.hidden;
    const max=caption.querySelector('[data-win-action="maximize"]');
    max.setAttribute('aria-label',state.maximized?'还原':'最大化');max.title=state.maximized?'还原':'最大化';
    max.innerHTML=state.maximized?'<svg viewBox="0 0 16 16"><path d="M5.5 3.5v-1h8v8h-1"/><rect x="2.5" y="5.5" width="8" height="8"/></svg>':'<svg viewBox="0 0 16 16"><rect x="3.5" y="3.5" width="9" height="9"/></svg>';
    setWinSize(state.maximized?Math.max(960,window.innerWidth-(window.innerWidth>1470?386:44)):Number(state.size),state.maximized?Math.max(640,window.innerHeight-44):(state.size==='960'?640:760));
    card.querySelectorAll('[data-win-key]').forEach(b=>b.setAttribute('aria-pressed',String(state[b.dataset.winKey]===b.dataset.winValue)));
    document.getElementById('winMaterialHint').textContent=solid?'当前：实色回退。Windows 10、透明不可用或非活动窗口采用稳定实底。':'当前：Mica 外观示意。CSS 的低对比染色仅表达视觉目标；真实材质由 Windows 11 系统合成。';
    document.getElementById('winStatus').textContent=message||'窗口可见；代理状态由主原型 mock 决定。可双击标题空白区切换最大化。';
    const labels={minimized:['窗口已最小化','代理 mock 状态保留；从任务栏重新打开。'],tray:['窗口已收起到托盘','代理 mock 状态保留；从托盘重新打开。'],exited:['桌面壳已退出（模拟）','此处只展示退出状态；重新打开会重新初始化本专题的代理 mock。']};
    const label=labels[state.visibility]||['',''];
    document.getElementById('winHiddenTitle').textContent=label[0];document.getElementById('winHiddenHint').textContent=label[1];
    document.querySelector('[data-launch="windows"]').classList.add('active');
  }
  function action(name){
    if(name==='leave'){exit();return;}
    if(name==='maximize'){state.maximized=!state.maximized;render(state.maximized?'已最大化（模拟）；再点可还原。':'已还原到原窗口尺寸（模拟）。');return;}
    if(name==='minimize'){state.visibility='minimized';render('已最小化（模拟）；代理 mock 状态未改变。');}
    if(name==='close'){state.visibility=state.close==='tray'?'tray':'exited';if(state.visibility==='exited')setState('stopped');render(state.visibility==='tray'?'已收起到托盘（模拟）；代理 mock 状态未改变。':'已退出（模拟）；代理 mock 停止。');}
    if(name==='reopen'){const exited=state.visibility==='exited';state.visibility='visible';if(exited)setState('stopped');render(exited?'重新启动专题（模拟）；代理 mock 为停止。':'窗口已重新打开（模拟）；保留代理 mock 状态。');}
    if(name==='reset'){Object.assign(state,{layout:'integrated',platform:'win11',transparency:'on',focus:'active',theme:'light',close:'tray',size:'1180',visibility:'visible',maximized:false});setRoute('overview',true);setState('stopped');render('场景已复位。');}
    if(state.visibility!=='visible')hidden.querySelector('button').focus();
  }
  function enter(){
    if(!previous)previous={theme:root.dataset.theme,w:root.style.getPropertyValue('--win-w'),h:root.style.getPropertyValue('--win-h')};
    exitNotifMode();body.classList.add('windows-mode');
    document.querySelectorAll('#launchGrid button').forEach(b=>b.classList.remove('active'));
    topicURL(true);render();
  }
  function exit(){
    if(!previous)return;
    body.classList.remove('windows-mode');delete body.dataset.winLayout;delete body.dataset.winMaterial;delete body.dataset.winFocus;
    frame.hidden=false;frame.inert=false;hidden.hidden=true;frame.classList.remove('windows-maximized');
    root.dataset.theme=previous.theme;
    for(const [key,value] of [['--win-w',previous.w],['--win-h',previous.h]]){if(value)root.style.setProperty(key,value);else root.style.removeProperty(key);}
    previous=null;topicURL(false);syncLaunchActive();updateSizeReadout();
    if(window.ProtoNotes&&typeof window.prototypeGroupKey==='function')ProtoNotes.setGroups(['side',window.prototypeGroupKey(currentRoute)]);
  }
  card.addEventListener('click',e=>{const b=e.target.closest('button');if(!b)return;if(b.dataset.winKey){state[b.dataset.winKey]=b.dataset.winValue;render();}else action(b.dataset.winAction);});
  caption.addEventListener('click',e=>{const b=e.target.closest('button');if(b)action(b.dataset.winAction);});
  caption.querySelector('.windows-drag').addEventListener('dblclick',()=>action('maximize'));
  hidden.querySelector('button').addEventListener('click',()=>action('reopen'));
  new MutationObserver(()=>{
    if(!body.classList.contains('windows-mode'))return;
    state.theme=root.dataset.theme==='dark'?'dark':'light';
    card.querySelectorAll('[data-win-key="theme"]').forEach(b=>b.setAttribute('aria-pressed',String(b.dataset.winValue===state.theme)));
  }).observe(root,{attributes:true,attributeFilter:['data-theme']});
  window.windowsPrototype={enter,exit,state:()=>({...state})};
  if(new URLSearchParams(location.search).get('topic')==='windows')enter();
})();
