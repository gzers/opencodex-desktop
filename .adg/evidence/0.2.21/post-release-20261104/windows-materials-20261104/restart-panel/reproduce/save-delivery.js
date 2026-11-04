(async()=>{
 const root=await window.__TAURI_INTERNALS__.invoke('get_data_root_config');if(!root.currentDataRoot.includes('post021-restart-panel'))throw Error('wrong isolated identity')
 const p=document.querySelector('#app').__vue_app__.config.globalProperties.$pinia;p._s.get('model-save-feedback').dismiss();p._s.get('routes').go('models');await new Promise(r=>setTimeout(r,500));const m=p._s.get('model-config');await m.load();const snapshot=JSON.parse(JSON.stringify(m.snapshot));
 const policy=JSON.parse(JSON.stringify(snapshot.policy)),channel=policy.channels.find(c=>c.connection.providerId==='FixtureRenamed');if(!channel)throw Error('synthetic channel not found')
 channel.defaults.contextWindow={mode:'custom',value:65539}
 const saved=await window.__TAURI_INTERNALS__.invoke('model_file_save',{request:{expectedFile:snapshot.policyDigest,policy,credentials:[]}})
 await m.load();p._s.get('feedback').clearToast();p._s.get('notifications').panelOpen=false;p._s.get('routes').go('models');p._s.get('model-save-feedback').showSaved();
 return{changed:saved.changed,policyDigest:saved.snapshot.policyDigest,channels:saved.snapshot.policy.channels.map(c=>({id:c.id,label:c.label,providerId:c.connection.providerId,contextWindow:c.defaults.contextWindow})),root,mode:'real model_file_save command, synthetic provider, no remote request'}
})()
