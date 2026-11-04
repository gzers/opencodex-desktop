/* Superba 参照实验：独立于现有全景的内存 mock 视图。 */
(function(){
 'use strict';
 var host=document.getElementById('connectionMapPage');
 if(!host||host.dataset.referenceReady==='true')return;
 host.dataset.referenceReady='true';
 var $=function(q,r){return (r||host).querySelector(q);};
 var $$=function(q,r){return Array.prototype.slice.call((r||host).querySelectorAll(q));};
 var view='current',mode='routed',laneMode='on',laneFlow=true,root=null,lanes=null,tabs=null;
 var laneTools=[
  {id:'codex',name:'Codex',mono:'cx',color:'blue',via:'router',provider:'Krill · 海外线路',model:'gpt-6.1-sol',tier:'速度档 · 标准'},
  {id:'claude',name:'Claude Code',mono:'cc',color:'orange',via:'router',provider:'Aimami · 默认节点',model:'claude-sonnet-4-6',tier:''},
  {id:'opencode',name:'OpenCode',mono:'oc',color:'purple',via:'direct',provider:'NewApi · 主节点',model:'gpt-6.1-sol',tier:''},
  {id:'gemini',name:'Gemini CLI',mono:'gm',color:'teal',via:'direct',provider:'Gateway · 默认节点',model:'gemini-3.1-pro',tier:''}
 ];

 function icon(name){
  var paths={
   window:'<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M3 8h18M3 15h18"/>',
   route:'<circle cx="5" cy="6" r="2"/><circle cx="19" cy="6" r="2"/><circle cx="12" cy="18" r="2"/><path d="M7 6h10M6 8v4a2 2 0 0 0 2 2h2M18 8v4a2 2 0 0 1-2 2h-2"/>',
   cloud:'<path d="M7 18h10a4 4 0 0 0 .7-7.94A6 6 0 0 0 6.2 9.1 4.5 4.5 0 0 0 7 18Z"/>',
   cube:'<path d="m12 3 7 4v10l-7 4-7-4V7Z"/><path d="m5 7 7 4 7-4M12 11v10"/>',
   sync:'<path d="M20 7v5h-5M4 17v-5h5M6 6a8 8 0 0 1 14 6M18 18a8 8 0 0 1-14-6"/>',
   arrow:'<path d="M5 12h14m-5-5 5 5-5 5"/>'
  };
  return '<svg class="pv-ref-icon" viewBox="0 0 24 24" aria-hidden="true">'+(paths[name]||paths.window)+'</svg>';
 }
 function node(id,name,title,detail,extra){
  return '<div class="pv-ref-node '+(extra||'')+'" data-ref-node="'+id+'">'+
   '<div class="pv-ref-tile pv-ref-tile-'+id+'">'+icon(name)+'<span class="pv-ref-marker" aria-hidden="true">!</span></div>'+
   '<strong>'+title+'</strong><small>'+detail+'</small></div>';
 }
 function lineRow(name,sub,latency,points,ok){
  return '<div class="pv-ref-line-row"><span class="pv-ref-line-dot '+(ok?'ok':'')+'"></span><div class="pv-ref-line-name"><strong>'+name+'</strong><small>'+sub+'</small></div><svg class="pv-ref-spark" viewBox="0 0 72 18" aria-hidden="true"><polyline points="'+points+'"/></svg><span class="pv-ref-line-ms">'+latency+'</span></div>';
 }
 function referenceMarkup(){
  return [
   '<div class="pv-reference" id="pvReference" role="tabpanel" aria-labelledby="pv-view-reference" hidden>',
   '<div class="pv-ref-head"><div><div class="pv-ref-eyebrow">REFERENCE EXPERIMENT · MOCK</div><div class="pv-ref-title-row"><h2>概览</h2><p>Codex 的请求去向与花费</p></div></div>',
   '<div class="pv-ref-head-side"><span class="pv-ref-manage"><i></i>Desktop 管理 · 运行中</span><div class="pv-ref-scenarios" role="group" aria-label="实验场景">',
   '<button type="button" data-ref-mode="routed" aria-pressed="true">路由中</button><button type="button" data-ref-mode="direct" aria-pressed="false">直连旁路</button><button type="button" data-ref-mode="failed" aria-pressed="false">上游异常</button>',
   '</div></div></div>',
   '<section class="pv-ref-panel pv-ref-hero" id="pvRefHero"><div class="pv-ref-panel-head"><strong>请求路径</strong><span id="pvRefProbed">测速：刚刚</span></div>',
   '<div class="pv-ref-path" id="pvRefPath">',
   node('client','window','本机','CLI 0.164.0 · App 0.164'),
   node('router','route','OpenCodex','运行中 · 本地路由'),
   node('provider','cloud','Krill · 国内直连','PAT · 余额 $148.93'),
   node('model','cube','gpt-6.1-sol','速度档 · 标准'),
   '</div><svg class="pv-ref-edge" id="pvRefEdge" aria-hidden="true"></svg><span class="pv-ref-latency" id="pvRefLatency"><i></i><span>41 ms</span></span></section>',
   '<div class="pv-ref-grid"><div class="pv-ref-left">',
   '<section class="pv-ref-panel pv-ref-today"><div class="pv-ref-panel-head"><strong>今日</strong><span>本月 $33.29</span></div>',
   '<div class="pv-ref-today-head"><div class="pv-ref-kpi"><b>$6.25</b><span>昨日同期没有用量</span></div><div class="pv-ref-today-meta"><span>68 次请求</span><span class="warn">2 次失败</span></div></div>',
   '<div class="pv-ref-bar"><i style="width:56%"></i><i style="width:27%"></i><i style="width:17%"></i></div>',
   '<div class="pv-ref-legend"><span><b>缓存在用</b> 4.65 M</span><span><b>未缓存输入</b> 492 K</span><span><b>输出</b> 85.8 K</span></div>',
   '<div class="pv-ref-legend pv-ref-legend-right"><span>缓存命中率 90%</span></div><div class="pv-ref-heat-wrap"><div class="pv-ref-heat" id="pvRefHeat" aria-label="最近七日活动格"></div><div class="pv-ref-heat-axis" aria-hidden="true"><span>00</span><span>06</span><span>12</span><span>18</span><span>少 <i></i><i></i><i></i><i></i> 多</span></div></div></section>',
   '<section class="pv-ref-panel pv-ref-sessions"><div class="pv-ref-panel-head"><strong>最近会话</strong><span>最近 7 天</span><button type="button" class="pv-ref-link">全部用量 →</button></div>',
   '<div class="pv-ref-table-wrap"><table class="pv-ref-table"><thead><tr><th>会话</th><th>主机</th><th>模型</th><th>Tokens</th><th>最近活动</th><th>费用</th></tr></thead><tbody>',
   '<tr><td><b>Review the catalog overlay</b></td><td>本机 · App</td><td>gpt-6.1-sol</td><td>2.07 M</td><td>8 分钟前</td><td>$3.35</td></tr>',
   '<tr><td><b>Check the failover order</b></td><td>本机 · 终端</td><td>gpt-5.4</td><td>908 K</td><td>9 分钟前</td><td>$0.47</td></tr>',
   '<tr><td><b>Polish the overview cards</b></td><td>本机 · App</td><td>gpt-6.1-sol</td><td>4.16 M</td><td>9 分钟前</td><td>$5.62</td></tr>',
   '<tr><td><b>Plan the database migration</b></td><td>本机 · App</td><td>gpt-6.1-sol</td><td>166 K</td><td>20 分钟前</td><td>$0.55</td></tr><tr><td><b>Explain the stats line</b></td><td>本机 · App</td><td>gpt-6.1-sol</td><td>705 K</td><td>2 小时前</td><td>$0.60</td></tr>',
   '</tbody></table></div></section>',
   '</div><div class="pv-ref-right">',
   '<section class="pv-ref-panel pv-ref-lines"><div class="pv-ref-panel-head"><strong>线路 <span>Krill</span></strong><button type="button" class="pv-ref-probe">◉ 测速</button></div>',
   lineRow('海外线路','127.0.0.1 · 使用中','99 ms','0,14 8,12 16,13 24,9 32,11 40,7 48,10 56,5 64,8 72,4',true),
   lineRow('国内直连','127.0.0.1 · 备用','41 ms','0,12 8,11 16,13 24,10 32,12 40,8 48,10 56,9 64,11 72,7',true),
   lineRow('三网优化 CDN','127.0.0.1 · 备用','157 ms','0,10 8,11 16,9 24,12 32,10 40,13 48,11 56,12 64,10 72,11',false),
   '<p class="pv-ref-line-note">直连：只用当前线路。开启本地路由后，按编号顺序自动切换。</p></section>',
   '<section class="pv-ref-panel pv-ref-todo"><div class="pv-ref-panel-head"><strong>待办 <span>2</span></strong></div>',
   '<div class="pv-ref-todo-row"><span class="pv-ref-todo-icon warn">!</span><div><strong>Key 以明文文件保存</strong><small>这台电脑没有可用的系统钥匙串</small></div><button type="button" class="pv-ref-outline">查看</button></div>',
   '<div class="pv-ref-todo-row"><span class="pv-ref-todo-icon info">i</span><div><strong>1 个模型没有价格</strong><small>gpt-5.3-codex-spark 的费用未知</small></div><button type="button" class="pv-ref-outline">设置价格</button></div>',
   '</section></div></div>',
   '<p class="pv-ref-footnote">参照 Superba main（f78d77a）的概览绘制参数；客户端、线路、余额、请求与用量均为 mock 数据，不读取本机配置。</p>',
   '</div>'
  ].join('');
 }
 function ensure(){
  var workspace=$('.pv-workspace');if(!workspace||tabs)return;
  var toolbar=$('.pv-toolbar',workspace),settings=$('.pv-settings-trigger',workspace);
  tabs=document.createElement('div');tabs.className='pv-view-tabs';tabs.setAttribute('role','tablist');tabs.setAttribute('aria-label','全景概览视图');
  tabs.innerHTML='<button type="button" role="tab" id="pv-view-current" data-pv-view="current" aria-selected="true">现有全景</button><button type="button" role="tab" id="pv-view-reference" data-pv-view="reference" aria-selected="false">Superba 参照实验</button><button type="button" role="tab" id="pv-view-lanes" data-pv-view="lanes" aria-selected="false">多工具泳道</button>';
  toolbar.insertBefore(tabs,settings);
  workspace.insertAdjacentHTML('beforeend',referenceMarkup()+lanesMarkup());
  root=$('#pvReference');lanes=$('#pvLanes');
  renderHeat();renderLaneRows();
  tabs.addEventListener('keydown',function(e){
   var list=$$('[data-pv-view]',tabs),i=list.indexOf(e.target);
   if(i>=0&&['ArrowLeft','ArrowRight','Home','End'].indexOf(e.key)>=0){e.preventDefault();var n=e.key==='Home'?0:e.key==='End'?list.length-1:(i+(e.key==='ArrowRight'?1:-1)+list.length)%list.length;setView(list[n].dataset.pvView);list[n].focus();}
  });
 }
 function renderHeat(){
  var heat=$('#pvRefHeat');if(!heat)return;
  var cells=[];
  for(var i=0;i<7*24;i++){var v=(i*7+11)%13;cells.push('<i class="pv-ref-heat-cell level-'+(v<5?0:v<8?1:v<11?2:3)+'"></i>');}
  heat.innerHTML=cells.join('');
 }
 function setMode(next){
  if(['routed','direct','failed'].indexOf(next)<0)return;
  mode=next;
  $$('[data-ref-mode]',root).forEach(function(b){b.setAttribute('aria-pressed',String(b.dataset.refMode===mode));});
  updateState();
  requestAnimationFrame(drawPath);
 }
 function updateState(){
  if(!root)return;
  var router=$('[data-ref-node="router"]',root),provider=$('[data-ref-node="provider"]',root);
  router.classList.toggle('is-ghost',mode==='direct');
  $('small',router).textContent=mode==='direct'?'未开启 · 直连':mode==='failed'?'运行中 · 转发失败':'运行中 · 本地路由';
  provider.classList.toggle('has-error',mode==='failed');
  $('small',provider).textContent=mode==='failed'?'首次超时 · 已保留配置':'PAT · 余额 $148.93';
  root.dataset.mode=mode;
  var probed=$('#pvRefProbed');if(probed)probed.textContent=mode==='failed'?'上次探测：刚刚':'测速：刚刚';
  var latency=$('#pvRefLatency');if(latency){latency.classList.toggle('is-error',mode==='failed');$('span',latency).textContent=mode==='failed'?'超时':'41 ms';}
 }
 function setView(next){
  if(['reference','lanes'].indexOf(next)<0)next='current';
  view=next;ensure();
  host.dataset.view=view;
  host.dataset.reference=String(view!=='current');
  var pageTitle=document.getElementById('routeTitle'),pageSubtitle=document.getElementById('routeSubtitle');
  var title=view==='reference'?'参照实验':view==='lanes'?'多工具泳道':'全景概览';
  var subtitle=view==='reference'?'按 Superba 最新源码复刻的内存实验视图。':view==='lanes'?'多个编码工具共用路由、直连与折叠的内存实验视图。':'本机客户端、聚合渠道与管理服务。';
  if(pageTitle)pageTitle.textContent=title;
  if(pageSubtitle)pageSubtitle.textContent=subtitle;
  document.documentElement.dataset.connectionView=view;
  $$('[data-pv-view]',tabs).forEach(function(b){b.setAttribute('aria-selected',String(b.dataset.pvView===view));});
  if(root)root.hidden=view!=='reference';
  if(lanes)lanes.hidden=view!=='lanes';
  var side=document.querySelector('.prototype-side');if(side)side.dataset.reference=String(view!=='current');
  var crumb=host.querySelector('.pv-breadcrumb strong');if(crumb)crumb.textContent='连接全景';
  if(view==='reference'){updateState();requestAnimationFrame(()=>{drawPath();root.scrollTo(0,0);});}
  else if(view==='lanes'){updateLaneState();requestAnimationFrame(()=>{drawLanes();lanes.scrollTo(0,0);});}
  else{if(root)root.scrollTop=0;if(lanes)lanes.scrollTop=0;}
 }
 function laneRouteState(t){
  return t.via==='direct'?'direct':laneMode==='on'?'router':'off';
 }
 function laneRowMarkup(t){
  var state=laneRouteState(t);
  var label=state==='direct'?'直连 · 不经 OpenCodex':state==='off'?'路由未开启':'经 OpenCodex';
  return '<div class="pv-lane-row" data-lane-tool="'+t.id+'" data-via="'+t.via+'" data-route-state="'+state+'">'+
   '<div class="pv-lane-tool"><span class="pv-lane-mono pv-lane-mono-'+t.color+'">'+t.mono+'</span><div><strong>'+t.name+'</strong><small>'+label+'</small></div></div>'+
   '<div class="pv-lane-gap" aria-hidden="true"></div>'+
   '<div class="pv-lane-router-slot" aria-hidden="true"></div>'+
   '<div class="pv-lane-gap" aria-hidden="true"></div>'+
   '<div class="pv-lane-provider"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M7 18h10a4 4 0 0 0 .7-7.94A6 6 0 0 0 6.2 9.1 4.5 4.5 0 0 0 7 18Z"/></svg><span>'+t.provider+'</span></div>'+
   '<div class="pv-lane-gap" aria-hidden="true"></div>'+
   '<div class="pv-lane-model"><span>'+t.model+'</span>'+(t.tier?'<small>'+t.tier+'</small>':'')+'</div>'+
  '</div>';
 }
 function laneRowsMarkup(){return laneTools.map(laneRowMarkup).join('');}
 function lanesMarkup(){
  return '<div class="pv-lanes" id="pvLanes" role="tabpanel" aria-labelledby="pv-view-lanes" hidden>'+
   '<div class="pv-ref-head"><div><div class="pv-ref-eyebrow">REFERENCE EXPERIMENT · MULTI-TOOL</div><div class="pv-ref-title-row"><h2>请求路径</h2><p>编码工具如何离开本机</p></div></div>'+
   '<div class="pv-ref-head-side"><span class="pv-ref-manage"><i></i>Desktop 管理 · 运行中</span><div class="pv-ref-scenarios" role="group" aria-label="泳道场景">'+
   '<button type="button" data-lane-mode="on" aria-pressed="true">路由开启</button><button type="button" data-lane-mode="off" aria-pressed="false">路由关闭</button><button type="button" data-lane-flow="true" aria-pressed="true" title="Superba 原生泳道没有流动光点，此项为增强演示">流动增强</button></div></div></div>'+
   '<section class="pv-lane-card"><div class="pv-lane-card-head"><div class="pv-lane-caption"><strong>请求路径</strong><button type="button" class="pv-lane-folded" title="WorkBuddy · Pi">其他工具 +2</button></div><span class="pv-lane-probed">测速：刚刚</span></div>'+
   '<div class="pv-lane-board" id="pvLaneBoard"><div class="pv-lane-desktop" id="pvLaneDesktop" role="note" aria-label="OpenCodex Desktop 管理节点"><svg viewBox="0 0 24 24" aria-hidden="true"><rect x="3" y="4" width="18" height="13" rx="2"/><path d="M8 21h8m-4-4v4"/></svg><span><strong>OpenCodex Desktop</strong><small>管理 · 运行中</small></span></div><div class="pv-lane-grid" id="pvLaneGrid">'+laneRowsMarkup()+'</div><svg class="pv-lane-edge" id="pvLaneEdge" aria-hidden="true"></svg><button type="button" class="pv-lane-router" id="pvLaneRouter" aria-label="OpenCodex 本地路由" title="OpenCodex 本地路由"><svg viewBox="0 0 24 24" aria-hidden="true"><path d="M5 6h14M8 6v5a2 2 0 0 0 2 2h4a2 2 0 0 1 2 2v3"/><circle cx="5" cy="6" r="2"/><circle cx="19" cy="6" r="2"/><circle cx="16" cy="18" r="2"/></svg><span><strong>OpenCodex 路由</strong><small>共享本地路由 · 127.0.0.1:10100</small></span></button></div>'+
   '<div class="pv-lane-legend"><span><i class="solid"></i>经本地路由</span><span><i class="dashed"></i>路由未开启</span><span><i class="muted"></i>直连绕过</span><span class="pv-lane-legend-note">拖动窗口宽度时保持工具行和线路对齐</span></div></section>'+
   '<div class="pv-lane-summary" id="pvLaneSummary"><span><b>2</b> 经 OpenCodex</span><span><b>2</b> 直连</span><span><b>2</b> 其他工具</span><span class="pv-lane-hint" id="pvLaneHint">路由关闭时，标为路由的工具不会自动改走直连。</span></div>'+
   '<p class="pv-ref-footnote">泳道几何来自 Superba main（f78d77a）；流动光点是本项目增强演示，Superba 原生没有。工具、线路、模型与请求均为 mock 数据。</p></div>';
 }
 function tablet(){
  var h=$('#pvRefHero'),svg=$('#pvRefEdge');if(!h||!svg)return null;
  var hr=h.getBoundingClientRect(),w=hr.width,hh=hr.height;if(!w||!hh)return null;
  svg.setAttribute('viewBox','0 0 '+w+' '+hh);
  var tiles={};
  $$('[data-ref-node]',root).forEach(function(n){var t=$('.pv-ref-tile',n);if(!t)return;var b=t.getBoundingClientRect();tiles[n.dataset.refNode]={left:b.left-hr.left,right:b.right-hr.left,y:b.top-hr.top+b.height/2};});
  return {w:w,h:hh,tiles:tiles,svg:svg};
 }
 function arrow(x1,x2,y,cls,marker){
  if(x2-x1<8)return '';
  return '<path d="M '+x1+' '+y+' H '+x2+'" class="pv-ref-link '+cls+'" marker-end="url(#'+marker+')"/>';
 }
 function drawPath(){
  if(!root||view!=='reference')return;
  var ref=$('#pvReference'),compact=ref&&(!ref.clientHeight||ref.clientHeight<690);
  if(ref&&ref.dataset.compact!==String(compact))ref.dataset.compact=String(compact);
  var g=tablet(),svg=$('#pvRefEdge');if(!g||!svg)return;
  var t=g.tiles,defs='<defs><marker id="pvRefArrowOk" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" markerUnits="userSpaceOnUse" orient="auto"><path d="M1 1L7 4L1 7Z" fill="#2e9d48"/></marker><marker id="pvRefArrowMuted" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" markerUnits="userSpaceOnUse" orient="auto"><path d="M1 1L7 4L1 7Z" fill="#c9c9c9"/></marker><marker id="pvRefArrowError" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" markerUnits="userSpaceOnUse" orient="auto"><path d="M1 1L7 4L1 7Z" fill="#da2f35"/></marker></defs>';
  var body='',latRect=null;
  if(mode==='direct'){
   var y=t.client.y;
   body+=arrow(t.client.right+10,t.provider.left-10,y,'ok','pvRefArrowOk');
   body+=directDash(y,t.router.y,t.router);
   latRect={x:(t.client.right+t.provider.left)/2,y:y};
   body+=arrow(t.provider.right+10,t.model.left-10,y,'muted','pvRefArrowMuted');
  }else{
   var y0=t.client.y;
   body+=arrow(t.client.right+10,t.router.left-10,y0,mode==='failed'?'muted':'ok',mode==='failed'?'pvRefArrowMuted':'pvRefArrowOk');
   body+=arrow(t.router.right+10,t.provider.left-10,y0,mode==='failed'?'error':'ok',mode==='failed'?'pvRefArrowError':'pvRefArrowOk');
   latRect={x:(t.router.right+t.provider.left)/2,y:y0};
   body+=arrow(t.provider.right+10,t.model.left-10,y0,'muted','pvRefArrowMuted');
  }
  svg.innerHTML=defs+body;
  var lat=$('#pvRefLatency');if(lat&&latRect){lat.style.left=latRect.x+'px';lat.style.top=latRect.y+'px';}
 }
 function renderLaneRows(){
  if(!lanes)return;
  var grid=$('#pvLaneGrid',lanes);if(!grid)return;
  grid.innerHTML=laneRowsMarkup();
  updateLaneState();
 }
 function updateLaneState(){
  if(!lanes)return;
  $$('[data-lane-mode]',lanes).forEach(function(b){b.setAttribute('aria-pressed',String(b.dataset.laneMode===laneMode));});
  var router=$('#pvLaneRouter',lanes);
  if(router){
   router.classList.toggle('is-off',laneMode==='off');
   var small=$('small',router);if(small)small.textContent=laneMode==='off'?'路由未开启 · 请求会失败':'本地路由 · 127.0.0.1:10100';
  }
  lanes.dataset.mode=laneMode;
  lanes.dataset.flow=String(laneFlow);
  $$('[data-lane-flow]',lanes).forEach(function(b){b.setAttribute('aria-pressed',String(laneFlow));});
  var hint=$('#pvLaneHint',lanes);
  if(hint)hint.textContent=laneMode==='off'?'路由关闭时，标为路由的工具不会自动改走直连。':'实线路径经过 OpenCodex；直连路径不经过 OpenCodex。';
 }
 function setLaneMode(next){
  if(['on','off'].indexOf(next)<0)return;
  laneMode=next;renderLaneRows();updateLaneState();requestAnimationFrame(drawLanes);
 }
 function setLaneFlow(next){
  laneFlow=!!next;updateLaneState();requestAnimationFrame(drawLanes);
 }
 function laneBox(el,board){
  var r=el.getBoundingClientRect(),br=board.getBoundingClientRect();
  return {left:r.left-br.left,right:r.right-br.left,top:r.top-br.top,bottom:r.bottom-br.top,width:r.width,height:r.height};
 }
 function laneArrow(x1,x2,y,cls,marker){
  if(x2-x1<8)return '';
  return '<path d="M '+x1+' '+y+' H '+x2+'" class="pv-lane-link '+cls+'" marker-end="url(#'+marker+')"/>';
 }
 function drawLanes(){
  if(!lanes||view!=='lanes')return;
  var board=$('#pvLaneBoard',lanes),svg=$('#pvLaneEdge',lanes);if(!board||!svg)return;
  var br=board.getBoundingClientRect(),w=br.width,h=br.height;if(!w||!h)return;
  svg.setAttribute('viewBox','0 0 '+w+' '+h);
  var rows=$$('.pv-lane-row',lanes),routed=rows.filter(function(r){return r.dataset.via==='router';});
  var router=$('#pvLaneRouter',lanes),slots=routed.map(function(r){return $('.pv-lane-router-slot',r);}).filter(Boolean);
  var desk=$('#pvLaneDesktop',lanes),deskBox=null,routerBox=null;
  if(router&&slots.length){
   var first=laneBox(slots[0],board),last=laneBox(slots[slots.length-1],board);
   router.hidden=false;
   router.style.left=(first.left+3)+'px';router.style.top=(first.top+3)+'px';
   router.style.width=Math.max(110,first.width-6)+'px';router.style.height=(last.bottom-first.top-6)+'px';
   routerBox=laneBox(router,board);
   if(desk){
    desk.hidden=false;desk.style.left=(routerBox.left+routerBox.width/2)+'px';desk.style.top='0px';
    deskBox=laneBox(desk,board);
   }
  }else{
   if(router)router.hidden=true;if(desk)desk.hidden=true;
  }
  var defs='<defs><linearGradient id="pvLaneGradOn" gradientUnits="userSpaceOnUse" x1="0" y1="0" x2="1000" y2="0"><stop offset="0" stop-color="#b8c8e8"/><stop offset=".55" stop-color="#7d9dff"/><stop offset="1" stop-color="#5b7cf5"/></linearGradient><linearGradient id="pvLaneGradDirect" gradientUnits="userSpaceOnUse" x1="0" y1="0" x2="1000" y2="0"><stop offset="0" stop-color="#d4d4d4"/><stop offset=".6" stop-color="#b4a8e8"/><stop offset="1" stop-color="#8b79d9"/></linearGradient><linearGradient id="pvLaneGradOff" gradientUnits="userSpaceOnUse" x1="0" y1="0" x2="1000" y2="0"><stop offset="0" stop-color="#f0c674"/><stop offset="1" stop-color="#e59700"/></linearGradient><marker id="pvLaneArrowOn" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" markerUnits="userSpaceOnUse" orient="auto"><path d="M1 1L7 4L1 7Z" fill="#6f8fe8"/></marker><marker id="pvLaneArrowOff" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" markerUnits="userSpaceOnUse" orient="auto"><path d="M1 1L7 4L1 7Z" fill="#e59700"/></marker><marker id="pvLaneArrowMuted" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" markerUnits="userSpaceOnUse" orient="auto"><path d="M1 1L7 4L1 7Z" fill="#9a8fd8"/></marker></defs>';
  var body='';
  if(deskBox&&routerBox){
   var mx=deskBox.left+deskBox.width/2,my=deskBox.bottom,mz=routerBox.top-2;
   body+='<path d="M '+mx+' '+my+' V '+mz+'" class="pv-lane-link manage"/>';
   body+=laneFlow?'<g class="pv-lane-flow"><path pathLength="1000" d="M '+mx+' '+my+' V '+mz+'" class="pv-lane-pulse"/></g>':'';
  }
  rows.forEach(function(row){
   var tool=$('.pv-lane-tool',row),prov=$('.pv-lane-provider',row),model=$('.pv-lane-model',row);
   if(!tool||!prov||!model)return;
   var a=laneBox(tool,board),p=laneBox(prov,board),m=laneBox(model,board),y=a.top+a.height/2,state=row.dataset.routeState;
   if(state==='direct'){
    var directD='M '+(a.right+4)+' '+y+' H '+(p.left-4);
    body+=laneArrow(a.right+4,p.left-4,y,'muted','pvLaneArrowMuted');
    if(laneFlow)body+='<path pathLength="1000" d="'+directD+'" class="pv-lane-pulse is-direct"/>';
    body+='<text class="pv-lane-direct-label" x="'+((a.right+p.left)/2)+'" y="'+(y+11)+'">直连</text>';
   }else{
    var routerRect=router?laneBox(router,board):null;
    var off=state==='off';
    if(routerRect){
     body+=laneArrow(a.right+4,routerRect.left-3,y,off?'off':'on',off?'pvLaneArrowOff':'pvLaneArrowOn');
     body+=laneArrow(routerRect.right+3,p.left-4,y,off?'off':'on',off?'pvLaneArrowOff':'pvLaneArrowOn');
     if(laneFlow&&!off){
      body+='<path pathLength="1000" d="M '+(a.right+4)+' '+y+' H '+(routerRect.left-3)+'" class="pv-lane-pulse"/>';
      body+='<path pathLength="1000" d="M '+(routerRect.right+3)+' '+y+' H '+(p.left-4)+'" class="pv-lane-pulse is-delayed"/>';
     }
    }
   }
   body+=laneArrow(p.right+4,m.left-4,y,'muted','pvLaneArrowMuted');
   if(laneFlow&&state!=='off')body+='<path pathLength="1000" d="M '+(p.right+4)+' '+y+' H '+(m.left-4)+'" class="pv-lane-pulse is-out"/>';
  });
  svg.innerHTML=defs+body;
 }
 function bind(){
  host.addEventListener('click',function(e){
   var t=e.target.closest('[data-pv-view]');if(t){setView(t.dataset.pvView);return;}
   var m=e.target.closest('[data-ref-mode]');if(m){setMode(m.dataset.refMode);return;}
   var lm=e.target.closest('[data-lane-mode]');if(lm){setLaneMode(lm.dataset.laneMode);return;}
   var lf=e.target.closest('[data-lane-flow]');if(lf){setLaneFlow(lf.getAttribute('aria-pressed')!=='true');}
  });
  window.addEventListener('resize',function(){if(view==='reference')requestAnimationFrame(drawPath);else if(view==='lanes')requestAnimationFrame(drawLanes);});
  if(window.ResizeObserver)new ResizeObserver(function(){if(view==='reference')requestAnimationFrame(drawPath);else if(view==='lanes')requestAnimationFrame(drawLanes);}).observe(host);
  setView('current');
 }
 var observer=new MutationObserver(function(){if(!tabs&&$('.pv-workspace')){ensure();bind();observer.disconnect();}});
 observer.observe(host,{subtree:true,childList:true});
 if($('.pv-workspace')){ensure();bind();observer.disconnect();}
 window.connectionMapReferencePrototype={state:function(){return {view:view,mode:mode,laneMode:laneMode,laneFlow:laneFlow,ready:!!root};},setView:setView,setMode:setMode,setLaneMode:setLaneMode,setLaneFlow:setLaneFlow,draw:drawPath,drawLanes:drawLanes};
})();
