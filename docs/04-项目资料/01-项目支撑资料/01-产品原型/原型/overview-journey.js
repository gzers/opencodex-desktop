/* 概览连续场景：平台与渠道双向探索同一模型 / 请求；仅内存样例，不访问网络或持久化。 */
(function(){'use strict';
var NS='http://www.w3.org/2000/svg';
function escape(s){return String(s).replace(/[&<>"']/g,function(c){return{'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]})}
function number(n){return Math.round(n).toLocaleString('en-US')}
function sampleRequests(channel,model,range){
  var end=range.to?new Date(range.to).getTime():new Date('2026-11-02T00:00').getTime();
  var span=range.seconds*1000,start=end-span;
  return [0,1,2].slice(0,Math.min(3,Math.max(0,model.req))).map(function(i){
    var speed=i===2?null:Math.max(1,Math.round(model.tps*(i===1?.68:1))),output=i===2?0:[480,760][i];
    var ttft=+(channel.ttft*(i===1?1.7:1)).toFixed(2),duration=i===2?1.84:+(ttft+output/speed).toFixed(2);
    var at=new Date(end-Math.min(span*.5,(i+1)*70000));
    if(at.getTime()<start)at=new Date(start+span*.5);
    return{id:'req_'+channel.k+'_'+model.n.replace(/[^a-z0-9]/gi,'').slice(0,15)+'_'+(i+1),
      channel:channel.k,channelName:channel.n,platform:window.OverviewPlatformData?window.OverviewPlatformData.owner(channel.k,model.n,i):null,model:model.n,via:!!channel.via,adapterMs:channel.via&&Number.isFinite(channel.adapterMs)?channel.adapterMs:null,at:at.toISOString(),time:at.toLocaleTimeString('zh-CN',{hour12:false}),
      output:output,tps:speed,ttft:i===2?null:ttft,duration:duration,status:i===2?'failed':'success'};
  });
}
function requestPhases(r){
 var phases=[];
 if(r.via)phases.push({name:'适配',value:Number.isFinite(r.adapterMs)?r.adapterMs+'ms':'未记录',at:Number.isFinite(r.adapterMs)?r.adapterMs/1000:Infinity});
 if(r.status==='failed')phases.push({name:'连接',value:'开始',at:r.via?(Number.isFinite(r.adapterMs)?r.adapterMs/1000:Infinity):0},{name:'首字',value:'未收到',at:Infinity},{name:'失败',value:r.duration.toFixed(2)+'s',at:r.duration});
 else phases.push({name:'首字',value:r.ttft.toFixed(2)+'s',at:r.ttft},{name:'输出',value:number(r.output)+' t',at:r.ttft},{name:'完成',value:r.duration.toFixed(2)+'s',at:r.duration});
 return phases;
}
function mount(o){
 var hero=o.hero,stage=o.stage,scene=document.createElement('div');scene.className='pj-scene';scene.setAttribute('aria-label','渠道与模型探索');scene.setAttribute('aria-hidden','true');scene.inert=true;
 scene.innerHTML='<header class="pj-head"><nav aria-label="探索路径"></nav><span class="pj-window"></span><button class="pj-close" type="button" data-pj="home" aria-label="返回全局概览">×</button></header>'+
  '<svg class="pj-lines" aria-hidden="true"></svg><div class="pj-models"></div><div class="pj-record-caption"></div><div class="pj-records"></div>'+
  '<div class="pj-pages pj-model-pages"><button type="button" data-pj="models-prev" aria-label="上一组模型">‹</button><span></span><button type="button" data-pj="models-next" aria-label="下一组模型">›</button></div>'+
  '<div class="pj-pages pj-request-pages"><button type="button" data-pj="requests-prev" aria-label="上一组请求">‹</button><span></span><button type="button" data-pj="requests-next" aria-label="下一组请求">›</button></div>'+
  '<section class="pj-inspector" aria-label="请求阶段"><div class="pj-request-head"><span class="pj-request-id"></span><button type="button" data-pj="play">回放路径 ↻</button></div>'+
  '<div class="pj-request-origin"></div><div class="pj-phase-line"><i></i></div><div class="pj-phases"></div><div class="pj-play-status" role="status" aria-live="polite"></div><button type="button" class="pj-counterpart" data-pj="counterpart"></button></section>'+
  '<section class="pj-adapter-detail" id="pjAdapterDetail" role="group" aria-label="实时适配详情"><header><b>实时适配</b><button type="button" data-pj="adapter-close" aria-label="关闭适配详情">×</button></header><p>Desktop 内置代理 · 协议转换</p><div class="pj-adapter-route"><span>Desktop</span><i>→</i><span>实时适配</span><i>→</i><b data-pj-adapter-target></b></div><div class="pj-adapter-window"></div><dl><div><dt>请求</dt><dd data-pj-adapter-requests></dd></div><div><dt>Token</dt><dd data-pj-adapter-tokens></dd></div><div><dt>附加耗时</dt><dd data-pj-adapter-latency></dd></div></dl><p class="pj-adapter-note">附加耗时已包含在首字等待时间内。</p><button class="pj-adapter-return" type="button" data-pj="adapter-close">返回当前详情 →</button></section>'+
  '<div class="pj-scope" role="status" aria-live="polite"></div><div class="pj-help">选择模型，沿路径查看请求</div>';
 stage.appendChild(scene);
 var nav=scene.querySelector('nav'),modelsHost=scene.querySelector('.pj-models'),recordsHost=scene.querySelector('.pj-records'),inspector=scene.querySelector('.pj-inspector'),lines=scene.querySelector('svg'),adapterPanel=scene.querySelector('.pj-adapter-detail');
 var adapterOpen=false,platformKey=null,platform=null,sourceNode=null,modelPage=0,requestPage=0;
 var channelKey=null,modelKey=null,requestKey=null,channel=null,model=null,requests=[],windowKey='',modelButtons=[],requestButtons=[],wireMap={},frame=0,playing=false,elapsed=0,playRate=1,origin=null,selectedNode=null,unwindTimer=0;
 var adapterNode=stage.querySelector('#phAdapter'),adapterTitle=adapterNode&&adapterNode.getAttribute('title'),adapterLabel=adapterNode&&adapterNode.getAttribute('aria-label'),adapterControls=adapterNode&&adapterNode.getAttribute('aria-controls');
 var traffic=document.createElementNS(NS,'g');traffic.setAttribute('class','pj-traffic');lines.appendChild(traffic);
 var ambient=[],flowFrame=0,flowLast=0,trafficKey='',trafficRevision=0;
 function particle(kind){var g=document.createElementNS(NS,'g');g.setAttribute('class','pj-particle '+kind);g.style.display='none';
  ['halo','tail','head'].forEach(function(name,i){var c=document.createElementNS(NS,'circle');c.setAttribute('class','pj-particle-'+name);c.setAttribute('r',[7.5,2.2,3.2][i]);g.appendChild(c)});traffic.appendChild(g);return g}
 var replayParticle=particle('pj-replay-particle');
 var motion=window.matchMedia?window.matchMedia('(prefers-reduced-motion: reduce)'):null;
 function quiet(){return document.documentElement.dataset.effects==='low'||!!(motion&&motion.matches)}
 function active(){return !!(platformKey||channelKey)}
 function level(){return requestKey?'request':modelKey?'model':platformKey?'platform':channelKey?'channel':'overview'}
 function subject(){return platform||channel}
 function modelList(){return subject()?subject().models||[]:[]}
 function point(e){
  /* 布局坐标排除正在插值的 translate / scale，连续点击也不会累计位移。 */
  var x=0,y=0,n=e,w=e.offsetWidth,h=e.offsetHeight;
  while(n&&n!==stage){x+=n.offsetLeft;y+=n.offsetTop;n=n.offsetParent}
  if(e.id==='phHub')return{x:x,y:y,w:w,h:h};
  if(e.id==='phCap'||e.id==='phHubCtl')return{x:x,y:y+h/2,w:w,h:h};
  return{x:x+w/2,y:y+h/2,w:w,h:h};
 }
 function pose(e,x,y){if(!e)return;var p=point(e);e.style.translate=(x-p.x)+'px '+(y-p.y)+'px'}
 function resetPose(){
  [stage.querySelector('#phHub'),stage.querySelector('#phCap'),stage.querySelector('#phHubCtl'),selectedNode,sourceNode,adapterNode].filter(Boolean).forEach(function(e){e.style.translate='';e.style.scale=''});
  if(o.onPose)o.onPose(null);
 }
 function restoreAdapter(){if(!adapterNode)return;
  adapterNode.classList.remove('pj-adapter-active');adapterNode.setAttribute('role','button');
  if(adapterTitle!==null)adapterNode.setAttribute('title',adapterTitle);if(adapterLabel!==null)adapterNode.setAttribute('aria-label',adapterLabel);
  if(adapterControls!==null)adapterNode.setAttribute('aria-controls',adapterControls);
  var stat=adapterNode.querySelector('[data-pj-adapter-ms]');if(stat)stat.remove();
 }
 function clearLogoBackdrop(hp,hubBase){
  /* Logo 在环境光 iframe 中；旧图形虽已淡化，仍比 iframe 层高。
     只在当前 Logo、标题及操作区的范围内擦去后景，不动前景连线或动态 Logo。 */
  restoreBackdrop();
  var cap=stage.querySelector('#phCap'),ctl=stage.querySelector('#phHubCtl'),r=hubBase.h*.31;
  var half=Math.max(hubBase.w*.31,cap.offsetWidth*.42,ctl.offsetWidth*.42)+12;
  var top=hp.y-r-12,bottom=hp.y+r+57+ctl.offsetHeight*.42+12;
  var clear={x:hp.x,y:(top+bottom)/2,rx:half*1.8,ry:(bottom-top)*.9};
  var sr=stage.getBoundingClientRect(),sx=sr.width/stage.offsetWidth||1,sy=sr.height/stage.offsetHeight||1;
  stage.querySelectorAll(platformKey?'.ph-client:not(.pj-source),.ph-wires,.ph-gen,.ph-chan:not(.pj-selected)':'.ph-clients,.ph-wires,.ph-gen,.ph-chan:not(.pj-selected)').forEach(function(e){
    var b=e.getBoundingClientRect(),l=(b.left-sr.left)/sx-stage.clientLeft,t=(b.top-sr.top)/sy-stage.clientTop;
    e.classList.add('pj-logo-clear');
    e.style.setProperty('--pj-mask-x',(clear.x-l)+'px');e.style.setProperty('--pj-mask-y',(clear.y-t)+'px');
    e.style.setProperty('--pj-mask-rx',clear.rx+'px');e.style.setProperty('--pj-mask-ry',clear.ry+'px');
  });
 }
 function restoreBackdrop(){stage.querySelectorAll('.pj-logo-clear').forEach(function(e){
  e.classList.remove('pj-logo-clear');['x','y','rx','ry'].forEach(function(k){e.style.removeProperty('--pj-mask-'+k)});
 })}
 function beforeLayout(){stage.classList.add('pj-measuring');resetPose();void stage.offsetWidth}
 function upstreamPathKeys(){return channel&&channel.via?['hub-adapter','adapter-channel']:['hub-channel']}
 function replayPathKeys(){var mi=modelList().indexOf(model),ri=requests.findIndex(function(r){return r.id===requestKey});
  return (platformKey?['platform-hub']:[]).concat(upstreamPathKeys(),['model-'+mi,'request-'+ri]);}
 function path(key,a,b,kind){
  var p=wireMap[key];if(!p){p=document.createElementNS(NS,'path');p.setAttribute('pathLength','1');p.setAttribute('class','pj-wire '+(kind||''));p.dataset.path=key;lines.appendChild(p);wireMap[key]=p}
  var dx=(b.x-a.x)*.52,d=kind==='pj-downstream'?
    'M'+a.x+','+a.y+' C'+(b.x-18)+','+a.y+' '+(b.x-18)+','+b.y+' '+b.x+','+b.y:
    'M'+a.x+','+a.y+' C'+(a.x+dx)+','+a.y+' '+(b.x-dx)+','+b.y+' '+b.x+','+b.y;
  p.setAttribute('d',d);p.style.d='path("'+d+'")';p.classList.add('is-visible');return p;
 }
 function pin(el,x,y){el.style.transform='translate('+Math.round(x)+'px,'+Math.round(y)+'px)'}
 function syncData(){
  var data=o.getData(),range=data.range,key=range.key;
  platform=platformKey?(data.platforms||[]).filter(function(p){return p.k===platformKey})[0]:null;
  if(platformKey&&!platform){exit();return false}
  if(windowKey&&key!==windowKey){stopPlay();requestKey=null;requestPage=0;if(platformKey){resetPose();channelKey=null;selectedNode=null;adapterOpen=false}}
  windowKey=key;
  channel=channelKey?data.channels.filter(function(c){return c.k===channelKey})[0]:null;
  if(channelKey&&!channel){exit();return false}
  if(platformKey){sourceNode=stage.querySelector('.ph-client[data-client="'+platformKey+'"]');if(!sourceNode){exit();return false}}
  if(channelKey&&(!selectedNode||!selectedNode.isConnected)){selectedNode=stage.querySelector('[data-k="'+channelKey+'"].ph-chan');
    if(!selectedNode&&o.revealChannel){o.revealChannel(channelKey);selectedNode=stage.querySelector('[data-k="'+channelKey+'"].ph-chan')}
    if(!selectedNode){exit();return false}}
  var available=modelList(),signature=(platformKey||channelKey)+'|'+available.map(function(m){return m.n}).join('|');
  if(modelsHost.dataset.signature!==signature){
    modelsHost.dataset.signature=signature;modelsHost.innerHTML=available.map(function(m,i){return'<button type="button" class="pj-model" data-pj="model" data-model="'+escape(m.n)+'"><span class="pj-model-orb">'+(i+1)+'</span><span class="pj-model-copy"><b>'+escape(m.n)+'</b><span data-pj-model-stat></span></span><span class="pj-go" aria-hidden="true">›</span></button>'}).join('');
    modelButtons=Array.prototype.slice.call(modelsHost.children);
  }
  modelButtons.forEach(function(b,i){var m=available[i];b.querySelector('[data-pj-model-stat]').textContent=number(m.req)+' 次 · '+(platformKey?m.channels.length+' 个渠道':m.tps+' t/s');b.setAttribute('aria-pressed',String(m.n===modelKey))});
  model=available.filter(function(m){return m.n===modelKey})[0]||null;
  if(modelKey&&!model){modelKey=null;requestKey=null;stopPlay()}
  if(model){
    if(platformKey){requests=[];model.channels.forEach(function(branch){var c=data.channels.filter(function(c){return c.k===branch.k})[0],m=c&&c.models.filter(function(m){return m.n===modelKey})[0];
      if(m)requests=requests.concat(sampleRequests(c,m,range).filter(function(r){return r.platform===platformKey}))});
      requests.sort(function(a,b){return b.at.localeCompare(a.at)||a.id.localeCompare(b.id)});
    }else requests=sampleRequests(channel,model,range);
    var signature2=(platformKey||channelKey)+'|'+modelKey+'|'+windowKey+'|'+JSON.stringify(requests);
    if(recordsHost.dataset.signature!==signature2){
      if(playing)stopPlay();
      recordsHost.dataset.signature=signature2;recordsHost.innerHTML=requests.map(function(r,i){return'<button type="button" class="pj-record" data-pj="request" data-request="'+r.id+'" title="'+escape((r.platform||'来源未区分')+' → '+r.channelName)+'"><i class="pj-status-dot '+(r.status==='failed'?'is-bad':'')+'"></i><time>'+r.time+'</time><span>'+(platformKey?escape(r.channelName)+(r.status==='failed'?' · 失败':''):r.status==='failed'?'连接失败':number(r.output)+' 输出 Token')+'</span><b>'+(r.status==='failed'?'1.84 s':r.tps+' t/s')+'</b><span aria-hidden="true">›</span></button>'}).join('');
      requestButtons=Array.prototype.slice.call(recordsHost.children);
    }
  }
  scene.querySelector('.pj-window').textContent=range.label;
  scene.querySelector('.pj-window').title=range.from?range.from.replace('T',' ')+' → '+range.to.replace('T',' '):range.label;
  var total=model||subject();scene.querySelector('.pj-scope').innerHTML='<b>'+escape(total.n)+'</b><span>'+number(total.req)+' 次 · '+total.tok.toFixed(2)+'M Token'+(model&&!platformKey?' · '+model.tps+' t/s':'')+'</span>';
  return true;
 }
 function navButton(action,text){return'<button type="button" data-pj="'+action+'">'+escape(text)+'</button>'}
 function updateNavigation(){
  var crumbs=navButton('home','概览')+'<span>›</span>'+navButton('channel',subject().n);
  if(model)crumbs+='<span>›</span>'+navButton('model-back',model.n);
  if(requestKey)crumbs+='<span>›</span><b>单次请求</b>';
  if(adapterOpen)crumbs+='<span>›</span><b>适配详情</b>';
  nav.innerHTML=crumbs;
  scene.dataset.level=level();hero.dataset.journey=level();
  scene.querySelector('.pj-help').textContent=adapterOpen?'Esc 返回当前详情':!modelKey?'选择模型，沿路径查看请求':level()==='model'?'选择请求，查看渠道与阶段':'Esc 返回上一级';
 }
 function refresh(){if(!active())return;if(o.canExplore&&!o.canExplore()){exit();return}if(!syncData())return;
  stage.classList.add('pj-active');hero.classList.add('pj-exploring');scene.inert=false;scene.setAttribute('aria-hidden','false');
  stage.classList.toggle('pj-adapted',!!(channel&&channel.via));stage.classList.toggle('pj-platform',!!platformKey);
  scene.style.setProperty('--pj-color','var(--ph-'+(channel?channel.k:platform.c)+')');
  stage.querySelectorAll('.ph-client').forEach(function(n){n.classList.toggle('pj-source',n===sourceNode&&!!platformKey)});
  stage.querySelectorAll('.ph-chan').forEach(function(n){n.classList.toggle('pj-selected',n===selectedNode)});
  stage.querySelectorAll('[data-f]').forEach(function(n){n.tabIndex=n===selectedNode||n===sourceNode||n===adapterNode&&channel&&channel.via?0:-1;n.setAttribute('aria-expanded',String(n===selectedNode||n===sourceNode||n===adapterNode&&adapterOpen))});
  if(adapterNode){
    adapterNode.classList.toggle('pj-adapter-active',!!(channel&&channel.via));
    adapterNode.setAttribute('role','button');
    if(channel&&channel.via){var detail='Desktop 协议转换 · '+(Number.isFinite(channel.adapterMs)?channel.adapterMs+'ms':'耗时未记录');
      adapterNode.setAttribute('aria-label','实时适配，'+detail+'，查看详情');adapterNode.setAttribute('title',detail);adapterNode.setAttribute('aria-controls',adapterPanel.id);
      var stat=adapterNode.querySelector('[data-pj-adapter-ms]');
      if(!stat){stat=document.createElement('small');stat.dataset.pjAdapterMs='';adapterNode.querySelector('.ph-nl').appendChild(stat)}
      stat.textContent=Number.isFinite(channel.adapterMs)?channel.adapterMs+'ms':'未记录';
    }else restoreAdapter();
  }
  updateNavigation();layout();
 }
 function layout(){if(!active())return;stage.classList.remove('pj-measuring');
  var w=stage.clientWidth||800,h=stage.clientHeight||440,small=h<360;
  var hp={x:platformKey?w*.235:Math.max(49,w*.105),y:h*.53},cp={x:w*(platformKey?.43:.32),y:h*.53},mx=w*(platformKey?.55:.49),mw=Math.min(282,w*(platformKey?.45:.49)-18);
  var hub=stage.querySelector('#phHub'),hubBase=point(hub),dx=hp.x-hubBase.x,dy=hp.y-hubBase.y;
  hub.style.translate=dx+'px '+dy+'px';
  var cap=stage.querySelector('#phCap'),ctl=stage.querySelector('#phHubCtl');
  pose(cap,hp.x,hp.y+hubBase.h*.31+22,.84);pose(ctl,hp.x,hp.y+hubBase.h*.31+57,.84);
  if(o.onPose)o.onPose({dx:dx,dy:dy,scale:.62});
  if(selectedNode){var orb=selectedNode.querySelector('.ph-orb'),ob=point(orb);selectedNode.style.translate=(cp.x-ob.x)+'px '+(cp.y-ob.y)+'px'}
  clearLogoBackdrop(hp,hubBase);
  Object.keys(wireMap).forEach(function(k){wireMap[k].classList.remove('is-visible')});
  var from={x:hp.x+hubBase.w*.31,y:hp.y},to={x:cp.x-21,y:cp.y};
  if(platformKey&&sourceNode){var sourceAt={x:Math.max(27,w*.06),y:hp.y},base=point(sourceNode.querySelector('.ph-orb'));
    sourceNode.style.translate=(sourceAt.x-base.x)+'px '+(sourceAt.y-base.y)+'px';
    path('platform-hub',{x:sourceAt.x+18,y:sourceAt.y},{x:hp.x-hubBase.w*.31,y:hp.y},'pj-trunk');}
  if(channel&&channel.via&&adapterNode){var ap={x:(from.x+to.x)/2,y:hp.y},aorb=adapterNode.querySelector('.ph-orb'),ab=point(aorb),ar=ab.w/2;
    adapterNode.style.translate=(ap.x-ab.x)+'px '+(ap.y-ab.y)+'px';
    path('hub-adapter',from,{x:ap.x-ar-3,y:ap.y},'pj-trunk');
    path('adapter-channel',{x:ap.x+ar+3,y:ap.y},to,'pj-trunk');
  }else if(channel)path('hub-channel',from,to,'pj-trunk');
  modelButtons.forEach(function(b,i){
    var chosen=b.dataset.model===modelKey,visible=modelKey?chosen:Math.floor(i/3)===modelPage;
    var count=Math.min(3,modelButtons.length-modelPage*3),row=i-modelPage*3;
    var y=modelKey||!visible?(small?48:67):(h/2+(row-(count-1)/2)*(small?67:84)-23);
    pin(b,mx,y);b.style.width=mw+'px';b.classList.toggle('is-visible',visible);b.inert=!visible;
    if(visible)path('model-'+i,channel?{x:cp.x+21,y:cp.y}:from,{x:mx,y:y+23},chosen?'is-selected':'');
  });
  var ry=small?116:151,rh=small?39:45;
  var caption=scene.querySelector('.pj-record-caption');pin(caption,mx,ry-16);caption.textContent=requests.length?'最近请求 · '+requests.length+' 条':'此范围暂无可展开的请求记录';caption.classList.toggle('is-visible',!!modelKey&&!requestKey);
  requestButtons.forEach(function(b,i){
    var chosen=b.dataset.request===requestKey,visible=!!modelKey&&(requestKey?chosen:Math.floor(i/3)===requestPage);
    var y=requestKey||!visible?ry:ry+(i-requestPage*3)*(rh+9);pin(b,mx,y);b.style.width=mw+'px';b.style.height=rh+'px';b.classList.toggle('is-visible',visible);b.inert=!visible;
    b.setAttribute('aria-pressed',String(chosen));
    if(visible)path('request-'+i,{x:mx+16,y:(small?48:67)+(small?52:59)},{x:mx,y:y+rh/2},'pj-downstream');
  });
  inspector.classList.toggle('is-visible',!!requestKey);inspector.inert=!requestKey;
  function pages(selector,total,page,shown){var host=scene.querySelector(selector),count=Math.ceil(total/3);host.classList.toggle('is-visible',shown&&count>1);host.inert=!shown||count<=1;
    host.querySelector('span').textContent=(page+1)+' / '+count;host.querySelector('button').disabled=page===0;host.querySelector('button:last-child').disabled=page>=count-1;
    pin(host,mx,shown&&modelKey?ry+3*(rh+9):h-49);}
  pages('.pj-model-pages',modelButtons.length,modelPage,!modelKey&&!adapterOpen);
  pages('.pj-request-pages',requestButtons.length,requestPage,!!modelKey&&!requestKey&&!adapterOpen);
  pin(inspector,mx,ry+rh+9);inspector.style.width=mw+'px';
  var selectedRequest=request(),inspectorKey=(platformKey||'channel')+'|'+channelKey+'|'+modelKey+'|'+requestKey+'|'+windowKey+'|'+(selectedRequest?selectedRequest.duration+'|'+selectedRequest.via+'|'+selectedRequest.adapterMs:'');
  if(requestKey&&inspector.dataset.key!==inspectorKey){inspector.dataset.key=inspectorKey;renderRequest()}
  if(requestKey){var iy=Math.max(small?108:136,Math.min(ry+rh+9,h-inspector.offsetHeight-8));
    pin(inspector,mx,iy);if(iy<ry+rh+9){var recordIndex=requests.findIndex(function(r){return r.id===requestKey}),record=requestButtons[recordIndex],recordY=iy-rh-8;
      if(record)pin(record,mx,recordY);path('request-'+recordIndex,{x:mx+16,y:(small?48:67)+(small?52:59)},{x:mx,y:recordY+rh/2},'pj-downstream')}}
  scene.classList.toggle('pj-adapter-open',adapterOpen);adapterPanel.classList.toggle('is-visible',adapterOpen);adapterPanel.inert=!adapterOpen;
  modelsHost.inert=adapterOpen;recordsHost.inert=adapterOpen;inspector.inert=adapterOpen||!requestKey;
  if(adapterOpen){
    scene.querySelector('[data-pj-adapter-target]').textContent=channel.n;
    scene.querySelector('.pj-adapter-window').textContent=o.getData().range.label+' · '+channel.n+' 汇总';
    scene.querySelector('[data-pj-adapter-requests]').textContent=number(channel.req)+' 次';
    scene.querySelector('[data-pj-adapter-tokens]').textContent=channel.tok.toFixed(2)+'M';
    scene.querySelector('[data-pj-adapter-latency]').textContent=Number.isFinite(channel.adapterMs)?channel.adapterMs+'ms':'未记录';
    adapterPanel.style.width=mw+'px';pin(adapterPanel,mx,Math.max(small?45:58,(h-adapterPanel.offsetHeight)/2));
  }
  syncTraffic();
 }
 /* 详情流光独立于被淡出的全局 SVG，沿当前展开路径移动；80px/s 为基准，模型速度只调整档位。 */
 function flowOptions(){return o.getMotion?o.getMotion():{running:false,pixelSpeed:80}}
 function canFlow(){return active()&&!requestKey&&!quiet()&&!document.hidden&&flowOptions().running}
 function measurable(p){return p&&typeof p.getTotalLength==='function'&&typeof p.getPointAtLength==='function'}
 function putParticle(g,p,d){if(!measurable(p))return;
  var length=p.getTotalLength(),at=p.getPointAtLength(Math.max(0,Math.min(length,d))),tail=p.getPointAtLength(Math.max(0,Math.min(length,d-8)));
  g.style.display='';Array.prototype.forEach.call(g.children,function(c,i){var pt=i===1?tail:at;c.setAttribute('cx',pt.x.toFixed(2));c.setAttribute('cy',pt.y.toFixed(2))});
 }
 function stopTraffic(){if(flowFrame)cancelAnimationFrame(flowFrame);flowFrame=0;flowLast=0;traffic.dataset.moving='false';ambient.forEach(function(a){a.g.style.display='none'})}
 function pathSpeed(key){var tps=model?model.tps:channel?channel.tps:52,match=/^model-(\d+)$/.exec(key);
  if(match&&modelList()[+match[1]])tps=modelList()[+match[1]].tps;
  var req=/^request-(\d+)$/.exec(key);if(req&&requests[+req[1]])tps=requests[+req[1]].tps;
  return (flowOptions().pixelSpeed||80)*Math.max(.55,Math.min(1.9,(tps||52)/52));
 }
 function syncTraffic(){lines.appendChild(traffic);
  if(!canFlow()){stopTraffic();return}
  var keys=Object.keys(wireMap).filter(function(k){var p=wireMap[k],m=/^request-(\d+)$/.exec(k);
    return p.classList.contains('is-visible')&&measurable(p)&&(!adapterOpen||upstreamPathKeys().indexOf(k)>=0)&&!(m&&requests[+m[1]]&&requests[+m[1]].status==='failed')});
  var signature=keys.map(function(k){return k+'|'+wireMap[k].getAttribute('d')+'|'+pathSpeed(k)}).join(';');
  if(signature!==trafficKey){stopTraffic();ambient.forEach(function(a){a.g.remove()});ambient=[];trafficKey=signature;trafficRevision++;
    keys.forEach(function(k,i){var p=wireMap[k],length=p.getTotalLength(),count=k==='hub-channel'||length>250?2:1;
      for(var n=0;n<count;n++){var g=particle('pj-ambient-particle');g.dataset.path=k;ambient.push({g:g,p:p,key:k,d:length*(.18+n*.48+i*.11)%Math.max(1,length)})}})}
  if(!ambient.length)return;
  ambient.forEach(function(a){putParticle(a.g,a.p,a.d)});traffic.dataset.moving='true';
  if(!flowFrame)flowFrame=requestAnimationFrame(flowStep);
 }
 function flowStep(t){flowFrame=0;if(!canFlow()){stopTraffic();return}
  var dt=flowLast?Math.min(.05,(t-flowLast)/1000):0;flowLast=t;
  ambient.forEach(function(a){var length=a.p.getTotalLength();a.d=(a.d+pathSpeed(a.key)*dt)%Math.max(1,length);putParticle(a.g,a.p,a.d)});
  flowFrame=requestAnimationFrame(flowStep);
 }
 function paintReplay(progress){if(!playing||quiet()||document.hidden){replayParticle.style.display='none';return}
  var route=replayPathKeys().map(function(k){return wireMap[k]}).filter(measurable);
  var lengths=route.map(function(p,i){return p.getTotalLength()*(i===route.length-1&&request().status==='failed'?.82:1)}),total=lengths.reduce(function(n,l){return n+l},0),distance=progress*total;
  for(var i=0;i<route.length;i++){if(distance<=lengths[i]||i===route.length-1){putParticle(replayParticle,route[i],distance);replayParticle.dataset.path=route[i].dataset.path;break}distance-=lengths[i]}
  replayParticle.classList.toggle('is-failed',request().status==='failed');
 }
 function request(){return requests.filter(function(r){return r.id===requestKey})[0]}
 function renderRequest(){var r=request();if(!r)return;
  scene.querySelector('.pj-request-id').textContent=r.id;
  var owner=(o.getData().platforms||[]).filter(function(p){return p.k===r.platform})[0];
  scene.querySelector('.pj-request-origin').textContent=(owner?owner.n:'来源未区分')+' → '+r.channelName;
  scene.querySelector('.pj-counterpart').textContent=platformKey?'从渠道查看同一请求 →':'从平台查看同一请求 →';scene.querySelector('.pj-counterpart').hidden=!platformKey&&!owner;
  var phases=requestPhases(r),phaseHost=scene.querySelector('.pj-phases');
  phaseHost.innerHTML=phases.map(function(p){return'<div><i></i><b>'+p.name+'</b><span>'+p.value+'</span></div>'}).join('');
  inspector.style.setProperty('--pj-phase-count',phases.length);inspector.style.setProperty('--pj-phase-inset',(50/phases.length)+'%');
  phaseHost.classList.toggle('is-failed',r.status==='failed');paintProgress(0);
 }
 function paintProgress(progress){var r=request();if(!r)return;
  inspector.style.setProperty('--pj-progress',progress);
  var nodes=scene.querySelectorAll('.pj-phases>div'),time=progress*r.duration;
  var phases=requestPhases(r);
  Array.prototype.forEach.call(nodes,function(n,i){n.classList.toggle('is-reached',time>=phases[i].at)});
  Object.keys(wireMap).forEach(function(k){wireMap[k].classList.toggle('is-replaying',playing&&wireMap[k].classList.contains('is-visible'))});
  requestButtons.forEach(function(b){b.classList.toggle('is-replaying',playing&&b.dataset.request===requestKey)});
  var text=progress>=1?'回放结束 · '+r.duration.toFixed(2)+'s':progress===0?'记录时长 '+r.duration.toFixed(2)+'s':'回放 '+playRate.toFixed(1)+'× · '+time.toFixed(2)+' / '+r.duration.toFixed(2)+'s';
  /* 逐帧读数不进入 live region，只有播放起止才公告。 */
  var status=scene.querySelector('.pj-play-status');status.setAttribute('aria-live',playing?'off':'polite');status.textContent=text;
  scene.querySelector('[data-pj="play"]').textContent=playing?'重新回放 ↻':'回放路径 ↻';
  paintReplay(progress);
 }
 function stopPlay(){if(frame){cancelAnimationFrame(frame);frame=0}playing=false;
  Object.keys(wireMap).forEach(function(k){wireMap[k].classList.remove('is-replaying')});
  requestButtons.forEach(function(b){b.classList.remove('is-replaying')});
  replayParticle.style.display='none';
  if(requestKey)paintProgress(1);
 }
 function play(){if(!active()||o.canExplore&&!o.canExplore())return;var r=request();if(!r)return;stopPlay();
  if(quiet()){paintProgress(1);return}
  playing=true;var duration=Math.min(5000,Math.max(1800,r.duration*550)),start=null;playRate=r.duration/(duration/1000);
  function step(t){if(!playing||!requestKey)return;if(start===null)start=t;elapsed=Math.min(1,(t-start)/duration);
    if(elapsed===1)playing=false;paintProgress(elapsed);if(playing)frame=requestAnimationFrame(step);else frame=0;
  }frame=requestAnimationFrame(step);
 }
 function enter(key){if(o.canExplore&&!o.canExplore())return false;
  var data=o.getData(),c=data.channels.filter(function(c){return c.k===key})[0];if(!c)return false;
  if(unwindTimer){clearTimeout(unwindTimer);unwindTimer=0}o.clearHover();resetPose();
  stopPlay();adapterOpen=false;platformKey=null;platform=null;sourceNode=null;modelPage=0;requestPage=0;channelKey=key;modelKey=null;requestKey=null;requests=[];selectedNode=null;origin=stage.querySelector('.ph-chan[data-k="'+key+'"]');
  refresh();return active();
 }
 function enterPlatform(key){if(o.canExplore&&!o.canExplore())return false;
  var p=(o.getData().platforms||[]).filter(function(p){return p.k===key})[0];if(!p)return false;
  if(unwindTimer){clearTimeout(unwindTimer);unwindTimer=0}o.clearHover();stopPlay();resetPose();
  adapterOpen=false;platformKey=key;platform=p;channelKey=null;channel=null;modelKey=null;requestKey=null;requests=[];selectedNode=null;sourceNode=null;modelPage=0;requestPage=0;
  origin=stage.querySelector('.ph-client[data-client="'+key+'"]');refresh();return active();}
 function clearDestination(){if(platformKey){if(selectedNode){selectedNode.style.translate='';selectedNode.classList.remove('pj-selected')}selectedNode=null;channelKey=null;channel=null;if(adapterNode)adapterNode.style.translate='';restoreAdapter()}}
 function chooseModel(name){if(!active()||o.canExplore&&!o.canExplore()||!modelList().some(function(m){return m.n===name}))return false;
  stopPlay();clearDestination();adapterOpen=false;modelKey=name;requestKey=null;requestPage=0;modelPage=Math.floor(modelList().findIndex(function(m){return m.n===name})/3);refresh();return true}
 function chooseRequest(id){if(!active()||o.canExplore&&!o.canExplore()||!requests.some(function(r){return r.id===id}))return false;
  var r=requests.filter(function(r){return r.id===id})[0];stopPlay();clearDestination();if(platformKey)channelKey=r.channel;
  adapterOpen=false;requestKey=id;requestPage=Math.floor(requests.indexOf(r)/3);refresh();play();return true}
 function focusAt(selector){var el=scene.querySelector(selector);if(el)el.focus({preventScroll:true})}
 function adapter(open){if(!active()||o.canExplore&&!o.canExplore()||!channel||!channel.via)return false;stopPlay();adapterOpen=typeof open==='boolean'?open:!adapterOpen;refresh();
  if(adapterOpen)adapterPanel.querySelector('button').focus({preventScroll:true});else adapterNode.focus({preventScroll:true});return true}
 function back(){if(!active())return false;stopPlay();
  if(adapterOpen){adapter(false);return true}
  if(requestKey){var id=requestKey;clearDestination();requestKey=null;refresh();focusAt('[data-request="'+id+'"]')}
  else if(modelKey){var name=modelKey;clearDestination();modelKey=null;refresh();focusAt('[data-model="'+name+'"]')}
  else exit(true);return true;
 }
 function exit(focusBack){if(!active())return;var source=platformKey?sourceNode:selectedNode&&selectedNode.isConnected?selectedNode:origin;stopPlay();resetPose();platformKey=null;platform=null;sourceNode=null;channelKey=null;modelKey=null;requestKey=null;windowKey='';
  adapterOpen=false;scene.classList.remove('pj-adapter-open');adapterPanel.classList.remove('is-visible');adapterPanel.inert=true;
  stopTraffic();restoreBackdrop();stage.classList.remove('pj-active','pj-measuring','pj-adapted','pj-platform');hero.classList.remove('pj-exploring');delete hero.dataset.journey;
  restoreAdapter();
  stage.querySelectorAll('.pj-selected,.pj-source').forEach(function(n){n.classList.remove('pj-selected','pj-source')});
  stage.querySelectorAll('[data-f]').forEach(function(n){n.tabIndex=0;n.setAttribute('aria-expanded','false')});
  scene.setAttribute('aria-hidden','true');scene.inert=true;
  if(unwindTimer)clearTimeout(unwindTimer);
  unwindTimer=setTimeout(function(){unwindTimer=0;if(!active())o.restore()},quiet()?0:460);
  if(focusBack&&source&&source.isConnected)o.focusReturn(source);
 }
 scene.addEventListener('click',function(e){var b=e.target.closest('[data-pj]');if(!b)return;
  var action=b.dataset.pj;
  if(action==='home')exit(true);
  else if(action==='channel'){stopPlay();clearDestination();adapterOpen=false;modelKey=null;requestKey=null;refresh()}
  else if(action==='model-back'){stopPlay();clearDestination();adapterOpen=false;requestKey=null;refresh()}
  else if(action==='adapter-close')adapter(false);
  else if(action==='model')chooseModel(b.dataset.model);
  else if(action==='request')chooseRequest(b.dataset.request);
  else if(action==='play')play();
  else if(action==='counterpart'){var r=request(),name=r.model,id=r.id,pk=r.platform,ck=r.channel,wasPlatform=!!platformKey;
    if(wasPlatform?enter(ck):enterPlatform(pk)){chooseModel(name);chooseRequest(id)}}
  else if(action==='models-prev'||action==='models-next'){modelPage+=action==='models-next'?1:-1;refresh();focusAt('.pj-model.is-visible')}
  else if(action==='requests-prev'||action==='requests-next'){requestPage+=action==='requests-next'?1:-1;refresh();focusAt('.pj-record.is-visible')}
 });
 function syncMotion(){if(quiet()||document.hidden)stopPlay();syncTraffic()}
 document.addEventListener('visibilitychange',syncMotion);
 if(motion&&motion.addEventListener)motion.addEventListener('change',syncMotion);
 return{active:active,enter:enter,platform:enterPlatform,back:back,exit:exit,beforeLayout:beforeLayout,refresh:refresh,model:chooseModel,request:chooseRequest,play:play,adapter:adapter,syncMotion:syncMotion,
  traffic:function(){return{moving:!!flowFrame,particles:ambient.filter(function(a){return a.g.style.display!=='none'}).length,replay:replayParticle.style.display!=='none',revision:trafficRevision,speeds:ambient.map(function(a){return{path:a.key,speed:pathSpeed(a.key)}})}},
  state:function(){return{level:level(),entry:platformKey?'platform':'channel',platform:platformKey,channel:channelKey,model:modelKey,request:requestKey,adapterOpen:adapterOpen,window:windowKey,playing:playing,requests:modelKey?requests.map(function(r){return Object.assign({},r)}):[]}}};
}
window.OverviewJourney={mount:mount,sampleRequests:sampleRequests};
})();
