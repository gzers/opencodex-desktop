(async()=>{
 const p=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia,l=p._s.get('lifecycle');
 const before=await window.__TAURI_INTERNALS__.invoke('get_status_snapshot');
 if(!before.facts.opencodex_home.includes('ocxd-0219-native-20261101')||before.port!==60385)throw Error('sandbox mismatch');
 p._s.get('notifications').panelOpen=false;p._s.get('routes').go('overview');
 await l.requestProcessAction('stop');
 const stopped=await window.__TAURI_INTERNALS__.invoke('get_status_snapshot');
 if(stopped.matrix.runtime!=='stopped')throw Error('not stopped');l.setStatusSnapshot(stopped);
 const start=Date.now();await l.requestProcessAction('start');const samples=[];
 while(Date.now()-start<35000){const s=await window.__TAURI_INTERNALS__.invoke('get_status_snapshot');l.setStatusSnapshot(s);samples.push({ms:Date.now()-start,status:s,busy:l.processActionBusy,error:l.processProgressError,completed:l.processProgressCompleted});if(s.matrix.runtime==='running')break;await new Promise(r=>setTimeout(r,300));}
 const end=samples.at(-1);if(end.status.matrix.runtime!=='running'||end.error||!end.completed)throw Error('cold start did not converge '+JSON.stringify(samples));
 l.dismissProcessProgress();p._s.get('routes').go('panel');const panelAt=Date.now();
 while(Date.now()-panelAt<12000&&!p._s.get('app').panelLoaded)await new Promise(r=>setTimeout(r,200));
 return {samples,loaded:p._s.get('app').panelLoaded,text:document.body.innerText};
})()
