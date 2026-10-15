/* 明确注册页面主Tab，按实际主内容内距测量；不改变路由、选项或滚动容器。 */
(() => {
 'use strict';
 const allowed=new Set(['route-models','route-extensions','route-sync','route-logs','route-settings','route-conversations']);
 const tracked=new Map();let queued=false;
 const observer=typeof ResizeObserver==='function'?new ResizeObserver(()=>schedule()):null;
 function measure(bar){
  const route=bar.closest('.route-section'),scroller=bar.closest('.main');
  if(!route||!allowed.has(route.id)||!scroller||!route.classList.contains('active'))return null;
  const style=window.getComputedStyle(scroller),px=key=>Math.max(0,parseFloat(style[key])||0),inner=bar.dataset.tabsScroll==='inner';
  const top=inner?0:-px('paddingTop');
  bar.style.setProperty('--page-tabs-top',top+'px');bar.style.setProperty('--page-tabs-left',px('paddingLeft')+'px');bar.style.setProperty('--page-tabs-right',px('paddingRight')+'px');
  const height=bar.getBoundingClientRect().height;
  route.style.setProperty('--page-tabs-top',top+'px');route.style.setProperty('--page-tabs-height',height+'px');route.style.setProperty('--page-tabs-offset',height+(scroller.clientTop||0)+12+'px');
  return {route:route.id,height,top,scroll:inner?'inner':'main'};
 }
 function refresh(){
  const bars=[...document.querySelectorAll('[data-page-tabs]')].filter(bar=>allowed.has(bar.closest('.route-section')?.id));
  for(const [bar,scroller] of tracked)if(!bar.isConnected||!bars.includes(bar)){observer?.unobserve(bar);tracked.delete(bar);if(![...tracked.values()].includes(scroller))observer?.unobserve(scroller);}
  for(const bar of bars){const scroller=bar.closest('.main');if(!scroller)continue;if(!tracked.has(bar)){tracked.set(bar,scroller);observer?.observe(bar);observer?.observe(scroller);}measure(bar);}
 }
 function schedule(){
  if(queued)return;queued=true;
  const apply=()=>{queued=false;refresh();};if(window.requestAnimationFrame)window.requestAnimationFrame(apply);else Promise.resolve().then(apply);
 }
 if(typeof MutationObserver==='function')new MutationObserver(schedule).observe(document.body,{childList:true,subtree:true,attributes:true,attributeFilter:['class','hidden','data-route','data-theme','data-effects']});
 window.addEventListener('resize',schedule,{passive:true});window.addEventListener('hashchange',schedule);
 window.prototypeStickyTabs={refresh,state:()=>[...tracked.keys()].map(bar=>measure(bar)).filter(Boolean)};
 refresh();
})();
