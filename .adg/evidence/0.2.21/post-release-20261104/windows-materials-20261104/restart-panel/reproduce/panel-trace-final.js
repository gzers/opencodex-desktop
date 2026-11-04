(async()=>{
 const p=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia;p._s.get('routes').go('overview');await new Promise(r=>setTimeout(r,500));const t={start:performance.now(),events:[],polls:[]};
 const subscriptions=['app','panel','preferences','lifecycle'].map(id=>p._s.get(id)?.$onAction(({name,after,onError})=>{const start=performance.now(),event={kind:'action',store:id,name,startMs:start-t.start};t.events.push(event);after(()=>{event.endMs=performance.now()-t.start;event.elapsedMs=performance.now()-start});onError(error=>{event.endMs=performance.now()-t.start;event.error=String(error)})}));
 const listener=e=>t.events.push({kind:'native',detail:e.detail,atMs:performance.now()-t.start});window.addEventListener('ocxd-panel',listener);
 p._s.get('routes').go('panel');let readyAt=null;
 for(let i=0;i<160;i++){await new Promise(r=>setTimeout(r,100));const s={ms:performance.now()-t.start,loaded:p._s.get('app').panelLoaded,text:document.querySelector('.panel-frame')?.innerText?.slice(0,350)};t.polls.push(s);if(s.loaded&&s.text?.includes('官方面板已嵌入主窗口')){readyAt=s.ms;break}}
 window.removeEventListener('ocxd-panel',listener);subscriptions.forEach(stop=>stop?.());return{trace:JSON.parse(JSON.stringify(p._s.get('panel').loadTrace??null)),readyAt,events:t.events,polls:t.polls,final:{root:{...document.documentElement.dataset},loaded:p._s.get('app').panelLoaded,error:p._s.get('app').panelError,text:document.body.innerText.slice(-1200)}}
})()
