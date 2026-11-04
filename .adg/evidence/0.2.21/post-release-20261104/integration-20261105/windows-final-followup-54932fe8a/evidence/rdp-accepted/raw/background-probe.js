(async()=>{
 const p=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia,e=p._s.get('effects');
 const canvas=document.querySelector('.motion-mesh-host canvas'),gl=canvas?.getContext('webgl'),original=gl?.drawArrays;let draws=0;
 if(gl)gl.drawArrays=function(...args){draws++;return original.apply(this,args)};
 try{await new Promise(r=>setTimeout(r,2000));const native=await window.__TAURI_INTERNALS__.invoke('window_appearance');return {native,foreground:await window.__TAURI_INTERNALS__.invoke('app_foreground'),root:{...document.documentElement.dataset},strategy:JSON.parse(JSON.stringify(e.strategy)),draws,durationMs:2000,documentFocus:document.hasFocus(),dialogs:[...document.querySelectorAll('dialog[open]')].map(d=>({title:d.querySelector('h2')?.textContent,blur:getComputedStyle(d).backdropFilter,bg:getComputedStyle(d).backgroundColor,rect:d.getBoundingClientRect().toJSON()})),preferences:{theme:p._s.get('preferences').data.theme,visualEffects:p._s.get('preferences').data.visualEffects,interfaceScale:p._s.get('preferences').data.interfaceScale}}}finally{if(gl)gl.drawArrays=original}
})()
