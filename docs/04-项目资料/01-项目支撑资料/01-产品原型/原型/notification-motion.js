/* 通知视觉封装：吸入既有铃铛、存量卡片补位；不发布事件、不改变通知状态。 */
(() => {
 'use strict';
 const flights=new WeakMap(),moves=new WeakMap(),DURATION=420,REFLOW=240;
 let mutating=false;
 const reduced=()=>window.matchMedia('(prefers-reduced-motion: reduce)').matches;
 const surfaces=()=>[...document.querySelectorAll('#toastEl.show,#taskHost .task-card')].filter(el=>el.isConnected&&!flights.has(el)&&el.getClientRects().length);
 function capture(){return new Map(surfaces().map(el=>[el,el.getBoundingClientRect()]));}
 function settle(before){
  if(reduced())return;
  for(const [el,rect] of before){
   if(!el.isConnected||flights.has(el)||el.id==='toastEl'&&!el.classList.contains('show'))continue;
   const next=el.getBoundingClientRect(),dy=rect.top-next.top;
   if(Math.abs(dy)<.5)continue;
   moves.get(el)?.cancel();
   if(typeof el.animate!=='function')continue;
   const animation=el.animate([{transform:'translateY('+dy+'px)'},{transform:'translateY(0)'}],{duration:REFLOW,easing:'cubic-bezier(.2,.8,.2,1)'});
   moves.set(el,animation);animation.finished.then(()=>{if(moves.get(el)===animation)moves.delete(el);},()=>{});
  }
 }
 function reflow(change){
  if(mutating)return change();
  const before=capture();mutating=true;
  try{return change();}finally{mutating=false;settle(before);}
 }
 function cancel(el){flights.get(el)?.cancel();moves.get(el)?.cancel();moves.delete(el);}
 function collapse(el,{bell=document.getElementById('notificationBtn'),onDone=()=>{}}={}){
  if(!el||!bell){onDone();return Promise.resolve(true);}
  if(flights.has(el))return flights.get(el).promise;
  window.NotificationCountdown?.get(el)?.destroy();
  moves.get(el)?.cancel();moves.delete(el);
  const r=el.getBoundingClientRect(),b=bell.getBoundingClientRect();
  if(reduced()||!r.width||!r.height||!el.isConnected){reflow(onDone);return Promise.resolve(true);}
  let resolve,active=true,animation=null,timer=null;
  const promise=new Promise(done=>resolve=done),pointer=el.style.pointerEvents;
  const flight={promise,cancel:()=>finish(false)};flights.set(el,flight);
  const observer=new MutationObserver(()=>{if(!el.isConnected)finish(false);});
  if(el.parentNode)observer.observe(el.parentNode,{childList:true});
  el.dataset.notificationMotion='collapsing';el.style.pointerEvents='none';
  function cleanup(){
   clearTimeout(timer);observer.disconnect();animation?.cancel();delete el.dataset.notificationMotion;el.style.pointerEvents=pointer;
   if(flights.get(el)===flight)flights.delete(el);
  }
  function finish(commit){
   if(!active)return;active=false;
   if(commit){const hadFocus=el.contains(document.activeElement);reflow(()=>{cleanup();onDone();});if(hadFocus&&!el.contains(document.activeElement))bell.focus({preventScroll:true});}
   else cleanup();
   resolve(commit);
  }
  const dx=b.left+b.width/2-r.left-r.width/2,dy=b.top+b.height/2-r.top-r.height/2;
  const x=Math.max(.025,Math.min(.12,b.width/r.width*.42)),y=Math.max(.025,Math.min(.18,b.height/r.height*.42));
  if(typeof el.animate==='function'){
   animation=el.animate([
    {offset:0,transform:'translate(0,0) scale(1)',clipPath:'polygon(0 0,100% 0,100% 100%,0 100%)',opacity:1},
    {offset:.3,transform:'translate('+dx*.12+'px,'+dy*.12+'px) scale(.9,.96)',clipPath:'polygon(12% 0,100% 0,92% 100%,0 100%)',opacity:.95},
    {offset:.72,transform:'translate('+dx*.7+'px,'+dy*.65+'px) scale(.32,.55)',clipPath:'polygon(34% 0,100% 0,76% 100%,0 100%)',opacity:.65},
    {offset:1,transform:'translate('+dx+'px,'+dy+'px) scale('+x+','+y+')',clipPath:'polygon(50% 0,100% 0,75% 100%,25% 100%)',opacity:0}
   ],{duration:DURATION,easing:'cubic-bezier(.4,0,.2,1)',fill:'forwards'});
   animation.finished.then(()=>finish(true),()=>{});
  }
  // 无WAAPI时直接等待同一终点；每张卡有独立兜底，不使用全局收起计时器。
  timer=setTimeout(()=>finish(true),typeof el.animate==='function'?DURATION+80:DURATION);
  return promise;
 }
 window.NotificationMotion=Object.freeze({collapse,cancel,reflow,state:el=>flights.has(el)?'collapsing':'idle',durationMs:DURATION,reflowDurationMs:REFLOW});
})();
