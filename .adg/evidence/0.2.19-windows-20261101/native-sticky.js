(async()=>{
 const p=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia,q=p._s.get('preferences'),results=[];
 for(const theme of ['light','dark'])for(const tier of ['high','mid','low']){
  await q.save({...q.data,theme,visualEffects:tier});p._s.get('theme').apply(theme);p._s.get('effects').setSetting(tier);p._s.get('notifications').panelOpen=false;p._s.get('routes').go('settings');
  await new Promise(r=>setTimeout(r,200));document.querySelector('.main').scrollTop=300;await new Promise(r=>setTimeout(r,300));
  const e=document.querySelector('.sticky-page-tabs'),c=getComputedStyle(e),r={theme,tier,stuck:e.dataset.stuck,background:c.backgroundColor,blur:c.backdropFilter};
  if(r.stuck!=='true'||r.background==='rgba(0, 0, 0, 0)'||(tier==='low'?r.blur!=='none':!r.blur.includes('blur(8px)')))throw Error(JSON.stringify(r));results.push(r);
 }return results;
})()
