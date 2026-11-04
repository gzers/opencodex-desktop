/* POST021：纯内存版本观察/首次范围；不联网、不读写配置。 */
(function(root,factory){const api=factory();if(typeof module==='object'&&module.exports)module.exports=api;else{root.Post021Sync=api;api.mount(root);}})(typeof window==='object'?window:globalThis,function(){
  'use strict';
  const views={
    unread:{connection:'已配置 · 未检测',time:'—',hint:'尚未读取'},
    reading:{connection:'正在检测',time:'—',hint:'读取中'},
    absent:{connection:'最近检测成功',time:'—',hint:'未发现快照'},
    failed:{connection:'最近检测失败',time:'—',hint:'读取失败'},
    unknown:{connection:'最近检测成功',time:'—',hint:'已读取 · 保存时间未知'},
    known:{connection:'最近检测成功',time:'11:02',hint:'上次观察'},
    matched:{connection:'最近检查一致',time:'—',hint:'已读取 · 保存时间未知'},
  };
  function firstScope(local,cloud,remembered){return remembered===null?[...new Set([...local,...cloud])]:[...remembered];}
  function versionView(state){return {...(views[state]||views.unread)};}
  function publication(confirmed){return confirmed?'已确认云端版本':'云端可能已写入，结果尚未确认；请重新核对原操作';}
  function mount(win){
    const doc=win.document;if(!doc)return;
    const show=state=>{
      win.renderWebdav?.(state==='failed'?'recent-failure':state==='reading'?'checking':state==='unread'?'configured':'recent-success');
      const value=versionView(state);
      for(const [id,text] of [['syncConn',value.connection],['syncRemoteTime',value.time],['syncDirectionText',value.hint]]){const node=doc.getElementById(id);if(node){node.textContent=text;node.hidden=false;}}
      for(const id of ['syncSep','syncDirection']){const node=doc.getElementById(id);if(node)node.hidden=id==='syncDirection';}
      const time=doc.getElementById('syncRemoteTime');if(time)time.title=state==='known'?'保存版本：2026-11-04 11:02；观察时间：2026-11-04 11:05':state==='unknown'||state==='matched'?'已读取快照；来源内容没有完整的可靠保存时间，不补为当前时间':value.hint;
      const actions=doc.getElementById('syncActions');if(actions&&!actions.querySelector('[data-post21-retry]')){const b=doc.createElement('button');b.className='btn ghost';b.textContent='检测版本';b.dataset.post21Retry='';b.disabled=state==='reading';b.onclick=()=>show('known');actions.append(b);}
    };
    const install=()=>{
      const host=doc.getElementById('webdavStates');if(!host||doc.getElementById('post21VersionStates'))return;
      const group=doc.createElement('div');group.id='post21VersionStates';group.className='tool-group';group.innerHTML='<b>发布后修复 · 版本观察</b>';
      for(const [state,label] of [['unread','未检测'],['reading','读取中'],['absent','无快照'],['failed','读取失败'],['unknown','时间未知'],['known','已读取'],['matched','检查一致']]){const b=doc.createElement('button');b.className='btn';b.textContent=label;b.dataset.post21Version=state;b.onclick=()=>show(state);group.append(b);}
      const note=doc.createElement('p');note.className='muted';note.textContent='已配置连接启动仅检测版本，不自动应用配置。首次完整目录全选，之后新增不自动选。';group.append(note);host.after(group);
    };
    if(doc.readyState==='loading')doc.addEventListener('DOMContentLoaded',install,{once:true});else install();
    win.post21SyncPreview=show;
  }
  return {versionView,firstScope,publication,mount};
});
