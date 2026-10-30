/* 通知自动收起倒计时组件。仅DOM与单调时钟；不发布事件、不读写设置。 */
(function(root,factory){const api=factory();if(typeof module==='object'&&module.exports)module.exports=api;else root.NotificationCountdown=api;})(typeof globalThis==='object'?globalThis:this,function(){
 'use strict';
 const mounted=new WeakMap();let serial=0;
 function clock(durationMs,now){
  if(!Number.isFinite(durationMs)||durationMs<=0)throw Error('倒计时需要正数时长');
  let remaining=durationMs,last=now(),stopped=false;const pauses=new Set();
  function sample(){const time=now();if(!stopped&&!pauses.size)remaining=Math.max(0,remaining-Math.max(0,time-last));last=time;return {durationMs,remainingMs:remaining,ratio:remaining/durationMs,phase:remaining===0?'elapsed':stopped?'stopped':pauses.size?'paused':'running'};}
  return {sample,pause(reason){sample();pauses.add(reason);return sample();},resume(reason){sample();pauses.delete(reason);return sample();},stop(){sample();stopped=true;return sample();}};
 }
 function mount(host,{durationMs=8000,onElapsed=()=>{}}={}){
  if(!host?.ownerDocument||typeof onElapsed!=='function')throw Error('倒计时挂载参数无效');
  mounted.get(host)?.destroy();
  const doc=host.ownerDocument,win=doc.defaultView,time=clock(durationMs,()=>win.performance.now());
  const track=doc.createElement('div'),fill=doc.createElement('span'),help=doc.createElement('span');
  track.className='notification-countdown';track.setAttribute('aria-hidden','true');fill.className='notification-countdown-fill';track.appendChild(fill);
  help.className='notification-countdown-help';help.id='notificationCountdownHelp-'+(++serial);help.textContent='本通知将在'+Math.round(durationMs/1000)+'秒后收起；鼠标悬停或键盘焦点停留时暂停。';
  const describedBy=host.getAttribute('aria-describedby')||'';host.setAttribute('aria-describedby',[describedBy,help.id].filter(Boolean).join(' '));
  host.classList.add('has-notification-countdown');host.append(track,help);
  let frame=null,active=true,lastStep=null;const reduced=win.matchMedia('(prefers-reduced-motion: reduce)');
  const handlers=[];
  function listen(target,type,fn){target.addEventListener(type,fn);handlers.push(()=>target.removeEventListener(type,fn));}
  function paint(){
   if(!active)return;
   const state=time.sample(),ratio=reduced.matches?Math.min(1,Math.ceil(state.remainingMs/1000)*1000/durationMs):state.ratio;
   if(lastStep!==ratio){fill.style.transform='scaleX('+ratio+')';lastStep=ratio;}
   track.dataset.phase=state.phase;
   if(state.phase==='elapsed'){destroy();onElapsed();return;}
   if(state.phase==='running')frame=win.requestAnimationFrame(paint);
  }
  function pause(reason){if(!active)return;time.pause(reason);if(frame!==null)win.cancelAnimationFrame(frame);frame=null;paint();}
  function resume(reason){if(!active)return;time.resume(reason);if(frame!==null)win.cancelAnimationFrame(frame);frame=null;paint();}
  function destroy(){
   if(!active)return;active=false;time.stop();if(frame!==null)win.cancelAnimationFrame(frame);frame=null;observer.disconnect();handlers.forEach(off=>off());
   track.remove();help.remove();host.classList.remove('has-notification-countdown');
   const remaining=(host.getAttribute('aria-describedby')||'').split(/\s+/).filter(id=>id&&id!==help.id);if(remaining.length)host.setAttribute('aria-describedby',remaining.join(' '));else host.removeAttribute('aria-describedby');
   if(mounted.get(host)===controller)mounted.delete(host);
  }
  const observer=new win.MutationObserver(()=>{if(!host.isConnected||track.parentNode!==host||host.classList.contains('toast')&&!host.classList.contains('show'))destroy();});
  observer.observe(host,{childList:true,attributes:true,attributeFilter:['class']});if(host.parentElement)observer.observe(host.parentElement,{childList:true});
  listen(host,'pointerenter',()=>pause('pointer'));listen(host,'pointerleave',()=>resume('pointer'));
  listen(host,'focusin',()=>pause('focus'));listen(host,'focusout',e=>{if(!host.contains(e.relatedTarget))resume('focus');});
  listen(doc,'visibilitychange',()=>doc.hidden?pause('hidden'):resume('hidden'));
  const controller={pause:()=>pause('manual'),resume:()=>resume('manual'),destroy,state:()=>({...time.sample(),active})};mounted.set(host,controller);
  if(doc.hidden)time.pause('hidden');if(host.contains(doc.activeElement))time.pause('focus');if(host.matches(':hover'))time.pause('pointer');paint();return controller;
 }
 return Object.freeze({clock,mount,get:host=>mounted.get(host)||null});
});
