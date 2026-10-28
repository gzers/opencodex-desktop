/* 全景概览专题：只使用内存 fixture；不扫描本机、不写配置、不发请求。 */
(() => {
 'use strict';
 const root=document.documentElement,host=document.getElementById('connectionMapPage');if(!host)return;
 const $=(q,r=host)=>r.querySelector(q),$$=(q,r=host)=>[...r.querySelectorAll(q)];
 const esc=v=>String(v??'').replace(/[&<>"']/g,c=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}[c]));
 const clone=v=>JSON.parse(JSON.stringify(v));
 const clients=[
  {id:'codex',name:'Codex App',icon:'codex',version:'0.164',installed:true,evidence:'应用安装',connected:true,target:'newapi',model:'gpt-6.1-sol',wire:'WS'},
  {id:'claude',name:'Claude Code',icon:'claude',version:'2.1',installed:true,evidence:'可执行文件',connected:true,target:'aimami',model:'claude-sonnet-4-6',wire:'HTTP'},
  {id:'opencode',name:'OpenCode',icon:'opencode',version:'1.1',installed:true,evidence:'可执行文件',connected:true,target:'krill',model:'gpt-6.1-sol',wire:'HTTP'},
  {id:'codex-cli',name:'Codex CLI',icon:'terminal',version:'0.164',installed:true,evidence:'可执行文件',connected:true,target:'newapi',model:'gpt-6.1-sol',wire:'HTTP'},
  {id:'gemini',name:'Gemini CLI',icon:'gemini',version:'0.30',installed:true,evidence:'可执行文件',connected:false,target:null,model:null},
  {id:'cursor',name:'Cursor',icon:'terminal',version:null,installed:false,evidence:'仅残留配置，未确认安装',connected:false,target:null}
 ];
 const channels=[
  {id:'newapi',name:'NewApi',glyph:'N',model:'gpt-6.1-sol',node:'主节点',probe:'86 ms',color:'blue'},
  {id:'krill',name:'Krill',glyph:'K',model:'gpt-6.1-sol',node:'国内直连',probe:'124 ms',color:'purple'},
  {id:'aimami',name:'Aimami',glyph:'A',model:'claude-sonnet-4-6',node:'默认节点',probe:'103 ms',color:'teal'},
  {id:'official',name:'OpenAI',glyph:'O',model:'gpt-6.1-sol',node:'官方节点',probe:'未检测',color:'neutral'},
  {id:'a6api',name:'A6api',glyph:'A6',model:'gpt-5.4',node:'备用节点',probe:'未检测',color:'neutral'},
  {id:'gateway',name:'Gateway',glyph:'G',model:'gemini-3.1-pro',node:'默认节点',probe:'未检测',color:'neutral'}
 ];
 const trafficPhases={requesting:'sending',streaming:'streaming',completed:'completed',cancelled:'cancelled',failed:'failed'};
 const scenes={requesting:'请求发送中',streaming:'生成中 · token 流动',completed:'请求完成',cancelled:'请求已取消',failed:'请求失败',recent:'聚合与 Desktop 适配',configured:'只有配置',adapter:'Desktop 适配目标',fallback:'一次请求回退',direct:'客户端直连',stopped:'OpenCodex 未运行',many:'大量渠道',unavailable:'状态读取失败'};
 const defaults={clients:['codex','claude','opencode'],pins:[],detail:false,management:true,sync:true,backup:true,flow:true,usage:true};
 let layoutBudget=0;
 let preferences=clone(defaults),draft=null,scene='recent',selected=null,requestFocus=false,temporary=null,temporaryClient=null,dialog=null,search='',operation='',ready=false,frameKey='',payloadKey='';
 let trafficTimer=null,trafficOutput=384;
 const modelPreviews={};
 const effortLabels={none:'无',minimal:'最低',low:'低',medium:'中',high:'高',xhigh:'极高',max:'最高'};
 const active=()=>root.dataset.route==='connection-map'&&!document.hidden;
 const icons={terminal:'<path d="m5 6 5 6-5 6m7 0h7"/>',manager:'<rect x="3" y="4" width="18" height="13" rx="2"/><path d="M8 21h8m-4-4v4"/>',settings:'<path d="m10 3-.7 2.1-2 .9-2.1-.4L3 9l1.4 1.6v2.8L3 15l2.2 3.4 2.1-.4 2 .9L10 21h4l.7-2.1 2-.9 2.1.4L21 15l-1.4-1.6v-2.8L21 9l-2.2-3.4-2.1.4-2-.9L14 3Z"/><circle cx="12" cy="12" r="3"/>',sync:'<path d="M20 7v5h-5M4 17v-5h5M6 6a8 8 0 0 1 14 6M18 18a8 8 0 0 1-14-6"/>',adapter:'<path d="M5 3v6h5V3M7.5 9v6a4 4 0 0 0 4 4H16m-2-4 4 4-4 4M16 3v6h5V3"/>',panel:'<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16"/>',restart:'<path d="M19 8a8 8 0 1 0 1 7M19 3v5h-5"/>',close:'<path d="m6 6 12 12M6 18 18 6"/>',chevron:'<path d="m9 5 7 7-7 7"/>',more:'<circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/>'};
 const icon=id=>['codex','claude','opencode','gemini'].includes(id)?'<svg class="pv-brand-icon" viewBox="0 0 24 24" aria-hidden="true"><use href="#ext-icon-'+id+'"/></svg>':'<svg viewBox="0 0 24 24" aria-hidden="true">'+(icons[id]||icons.terminal)+'</svg>';
 const button=(label,action,attrs='',cls='')=>'<button type="button" class="pv-action '+cls+'" data-pv="'+action+'" '+attrs+'>'+label+'</button>';
 function directory(){return scene==='many'?[...channels,...Array.from({length:24},(_,i)=>({id:'extra-'+i,name:'渠道 '+String(i+1).padStart(2,'0'),glyph:String(i+1).padStart(2,'0'),model:'gpt-5.4',node:'默认节点',probe:'未检测',color:'neutral'}))]:channels;}
 function state(){
  const known=scene!=='unavailable',running=scene!=='stopped'&&operation!=='stopped',observed=!['configured','direct','stopped','unavailable'].includes(scene)&&running;
  const chosenClients=clients.filter(c=>c.installed&&preferences.clients.includes(c.id));
  if(temporaryClient)chosenClients.sort((a,b)=>Number(b.id===temporaryClient)-Number(a.id===temporaryClient));
  const visibleClients=chosenClients.slice(0,3),hiddenClients=chosenClients.slice(3);
  const trafficClient=visibleClients.find(c=>c.connected&&c.target!=='krill');
  const livePhase=running&&known&&trafficClient?trafficPhases[scene]||null:null;
  const flowing=['sending','streaming'].includes(livePhase);
  const focused=livePhase?trafficClient:visibleClients.find(c=>c.id===selected)||visibleClients.find(c=>c.connected)||null;
  const actualRequest=(!!livePhase||requestFocus||scene==='fallback'||scene==='adapter')&&observed;
  const adapterConfigured=!['direct','fallback'].includes(scene),targetOnly=scene==='adapter';
  const target=scene==='adapter'?'krill':scene==='fallback'?'newapi':focused?.target||'newapi';
  const required=actualRequest?(scene==='fallback'?['krill','newapi']:[target]):clients.some(c=>c.id===selected)&&focused?.connected?[target]:directory().some(c=>c.id===selected)?[selected]:[];
  const all=directory();
  const recent=observed?['newapi','krill','aimami']:[];
  const wanted=[...new Set([temporary,...required,...preferences.pins,...recent,...(preferences.backup?['newapi','krill','aimami']:['newapi'])].filter(Boolean))].filter(id=>all.some(c=>c.id===id));
  const budget=(canvas.clientWidth||host.clientWidth||880)<680?2:3;
  const visibleChannels=wanted.slice(0,budget).map(id=>all.find(c=>c.id===id));
  // 适配目标放在下方，与 Desktop 支路对齐；仅改变画面顺序，不改选路。
  visibleChannels.sort((a,b)=>scene==='direct'?visibleClients.findIndex(c=>c.target===a.id)-visibleClients.findIndex(c=>c.target===b.id):adapterConfigured&&a.id==='krill'?1:adapterConfigured&&b.id==='krill'?-1:a.id===temporary?-1:b.id===temporary?1:all.indexOf(a)-all.indexOf(b));
  const adapter=adapterConfigured&&visibleChannels.some(c=>c.id==='krill')&&(!actualRequest||livePhase||targetOnly||target==='krill');
  return {known,running,observed,budget,visibleClients,hiddenClients,visibleChannels,hiddenChannels:all.filter(c=>!visibleChannels.some(v=>v.id===c.id)),focused,target,actualRequest,livePhase,flowing,usage:livePhase?{input:livePhase==='sending'?null:12840,output:livePhase==='sending'?null:livePhase==='completed'?1286:trafficOutput}:null,adapter,adapterConfigured,targetOnly,direct:scene==='direct',status:!known?'正在确认':running?'运行中':'未运行',motion:!known?'stale':running?'running':'stopped'};
 }
 function mark(kind,text){return '<span class="pv-state '+kind+'"><i></i>'+esc(text)+'</span>';}
 function clientNode(c,i,total,s){
  const y=s.direct?34+i*(42/Math.max(total-1,1)):total===1?39:19+i*(54/Math.max(total-1,1));
  const pending=s.targetOnly||s.adapterConfigured&&c.target==='krill';
  return '<section class="pv-node pv-client '+(selected===c.id?'is-selected':'')+'" data-node="'+c.id+'" style="--x:10%;--y:'+y+'%"><button type="button" class="pv-node-main" data-pv="node" data-id="'+c.id+'" aria-label="查看 '+c.name+'" aria-haspopup="dialog"><span class="pv-tile">'+icon(c.icon)+'</span><strong>'+c.name+'</strong></button><div class="pv-node-info"><span>v'+c.version+'</span><span>'+(!c.connected?'未接入':s.livePhase&&c.id===s.focused?.id?({sending:'发送中',streaming:'生成中',completed:'已完成',cancelled:'已取消',failed:'已中断'}[s.livePhase]):s.direct?'配置直连':pending?'适配配置':s.observed?'近期请求':'已配置')+'</span></div>'+(preferences.detail?'<small>'+c.evidence+'</small>':'')+'</section>';
 }
 function channelNode(c,i,total,s){
  const directIndex=s.visibleClients.findIndex(cl=>cl.target===c.id);
  const y=s.direct?34+Math.max(0,directIndex)*(42/Math.max(s.visibleClients.length-1,1)):total===1?31:total===2?19+i*47:19+i*27;
  const pending=s.targetOnly||s.adapterConfigured&&c.id==='krill';
  const isAttempt=scene==='fallback'&&['krill','newapi'].includes(c.id),used=!pending&&s.observed&&(s.actualRequest?[s.target,...(scene==='fallback'?['krill']:[])].includes(c.id):s.visibleClients.some(cl=>cl.connected&&cl.target===c.id));
  return '<section class="pv-node pv-channel '+(selected===c.id?'is-selected':'')+'" data-node="'+c.id+'" style="--x:87%;--y:'+y+'%"><button type="button" class="pv-node-main" data-pv="node" data-id="'+c.id+'" aria-label="查看渠道 '+c.name+'" aria-haspopup="dialog"><span class="pv-tile pv-provider '+c.color+'">'+c.glyph+'</span><strong>'+c.name+'</strong></button><div class="pv-node-info"><span>'+c.node+'</span>'+mark(isAttempt&&c.id==='krill'?'warning':used?'success':'',s.livePhase&&c.id===s.target?({sending:'等待响应',streaming:'生成中',completed:'已完成',cancelled:'已取消',failed:'已中断'}[s.livePhase]):isAttempt&&c.id==='krill'?'首次超时':pending?'适配待观察':used?'近期采用':'已配置')+'</div><small>'+esc(modelPreviews[c.id]?.label||c.model)+'</small>'+(preferences.detail?'<small>连接探测 '+c.probe+'</small>':'')+'</section>';
 }
 const frame=document.createElement('iframe');frame.className='pv-backdrop';frame.title='OpenCodex Desktop 形象与背景光';frame.tabIndex=-1;frame.setAttribute('aria-hidden','true');
 const frameSource=new URL('../../../03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型/候选/2026-09-28-Logo本体形变/index.html',document.currentScript.src);
 host.innerHTML='<div class="pv-workspace"><div class="pv-note"><div class="proto-annotation" id="ann-connection-map-01" data-annotation-number="01" data-annotation-title="全景概览 · 内存模拟" data-annotation-category="设计探索 · 版本未定"><button class="annotation-marker" type="button" aria-label="原型注释 01：全景概览 · 内存模拟" aria-expanded="false" aria-controls="annotationPopover"><span>原型</span><span>01</span><span>＋</span></button><div class="annotation-source" hidden><p>整页可视化专题，尚未替换正式概览。客户端存在性、版本、聚合渠道、请求、更新和同步均为虚构样例，不扫描本机、不请求模型、不写配置。请求中／生成中提供合成数据流动演示；动画速度不代表token速度或计费。动态图形仅属于 OpenCodex Desktop，OpenCodex 服务使用自有终端云标识。后台适配合并在 Desktop 产品节点，能力适配场景是目标链路示例，不证明当前压缩或历史能力已打通。显示设置只改本专题内存，原有业务草稿和路由不变。</p></div></div></div><header class="pv-toolbar"><div class="pv-breadcrumb">本机 <span>/</span> <strong>连接全景</strong></div>'+button(icon('settings')+'概览设置','settings','','pv-settings-trigger')+'</header><div class="pv-canvas" id="pvCanvas"><svg class="pv-edges" aria-hidden="true"></svg><div class="pv-activity" id="pvActivity" hidden></div><div id="pvNodes"></div><div class="pv-desktop-main" data-node="desktop"><button type="button" class="pv-desktop-logo-hit" data-pv="node" data-id="desktop" aria-label="查看 OpenCodex Desktop 节点" aria-haspopup="dialog"></button><button type="button" class="pv-desktop-title" data-pv="node" data-id="desktop" aria-haspopup="dialog">OpenCodex Desktop '+icon('chevron')+'</button><div class="pv-desktop-meta"><span>v0.2.17</span><span>管理与后台服务</span>'+button('检查更新','node','data-id="desktop" data-tab="updates"','pv-update')+'</div><button type="button" class="pv-desktop-capability" id="pvDesktopCapability" data-pv="node" data-id="desktop" data-tab="connection" aria-haspopup="dialog"></button><button type="button" class="pv-desktop-sync" id="pvDesktopSync" data-pv="node" data-id="desktop" data-tab="sync" aria-haspopup="dialog">'+icon('sync')+'<span>14:20 已同步</span></button></div><section class="pv-node pv-service" data-node="proxy"><button type="button" class="pv-node-main" data-pv="node" data-id="proxy" aria-label="查看 OpenCodex 节点" aria-haspopup="dialog"><span class="pv-tile pv-service-mark"><img class="pv-service-logo-light" src="../../../04-品牌与图标素材/01-原始素材/opencodex-mark-light.svg" alt="" width="58" height="58"><img class="pv-service-logo-dark" src="../../../04-品牌与图标素材/01-原始素材/opencodex-mark-dark.svg" alt="" width="58" height="58"><span class="pv-service-fallback" aria-hidden="true">'+icon('adapter')+'</span></span><strong>OpenCodex</strong></button><div id="pvStatus" class="pv-service-state"></div><div class="pv-node-info"><span>v2.84.0</span><span>端口 10100</span></div><small>聚合路由 · 认证与转发</small>'+button('有更新 ↗','node','data-id="proxy" data-tab="updates"','pv-update')+'</section></div><footer class="pv-footer"><span class="pv-legend"><i></i>近期请求 <i class="config"></i>配置指向 <i class="manage"></i>管理关系</span><button type="button" data-pv="request" class="pv-request" id="pvRequest"></button></footer><div class="pv-notice" id="pvNotice" role="status" hidden></div></div>';
 const canvas=$('#pvCanvas');canvas.prepend(frame);
 function render(){
  const s=state();layoutBudget=s.budget;
  host.dataset.scene=scene;host.dataset.detail=String(preferences.detail);
  host.dataset.flowing=String(s.flowing&&preferences.flow);
  $('#pvNodes').innerHTML=
   s.visibleClients.map((c,i)=>clientNode(c,i,s.visibleClients.length,s)).join('')+
   (s.hiddenClients.length?'<button type="button" class="pv-more-clients" data-pv="clients">其他客户端 <span>'+s.hiddenClients.length+'</span> '+icon('chevron')+'</button>':'')+
   (!s.visibleClients.length?'<div class="pv-empty">尚未选择展示的客户端<br>'+button('选择客户端','settings')+'</div>':'')+
   s.visibleChannels.map((c,i)=>channelNode(c,i,s.visibleChannels.length,s)).join('')+
   (s.hiddenChannels.length?'<button type="button" class="pv-more-channels" data-pv="channels">其他渠道 <span>'+s.hiddenChannels.length+'</span> '+icon('chevron')+'</button>':'');
  $('#pvDesktopCapability').hidden=!s.adapterConfigured;
  $('#pvDesktopCapability').textContent='后台适配 · 目标配置';
  $('#pvDesktopSync').hidden=!preferences.sync;
  $('#pvStatus').innerHTML=mark(s.known&&s.running?'success':!s.known?'warning':'',s.status);
  $('#pvRequest').textContent=!s.known?'读取失败 · 上次采样 14:30':!s.running?'代理未运行 · 未观察请求':s.direct?'已配置直连 · 尚未观察请求':!s.observed?'尚未观察请求 · 显示已保存配置':scene==='adapter'?'目标链路示例 · Krill 经能力适配':s.adapterConfigured&&s.focused?.target==='krill'?'目标配置 · OpenCodex → Desktop 后台适配 → Krill':scene==='fallback'?'14:32 · Krill 超时 → NewApi 成功':'14:32 · 最近请求  '+(s.focused?.name||'Codex App')+' → '+(channels.find(c=>c.id===s.target)?.name||'NewApi');
  $('#pvRequest').disabled=!s.observed||s.targetOnly||s.adapterConfigured&&s.focused?.target==='krill';$('#pvRequest').setAttribute('aria-pressed',String(requestFocus));
  if(operation==='stopped')$('#pvRequest').textContent='代理已停止 · 保留配置关系';
  updateTraffic(s);syncTraffic();syncFrame();scheduleDraw();window.prototypePanel?.refresh();
  $$('[data-pv-scene]',document).forEach(b=>b.setAttribute('aria-pressed',String(b.dataset.pvScene===scene)));
 }
 function connections(){
  const s=state(),lines=[];
  if(preferences.management&&['desktop','proxy'].includes(selected)&&!s.livePhase)lines.push({from:'desktop',to:'proxy',kind:'manage',vertical:true});
  for(const c of s.visibleClients){
   if(!c.connected)continue;
   if(s.actualRequest&&!s.livePhase&&c.id!==s.focused?.id)continue;
   const live=s.flowing&&c.id===s.focused?.id,focus=s.livePhase?c.id===s.focused?.id:!selected||['proxy','desktop',c.id].includes(selected);
   if(s.direct){if(s.visibleChannels.some(v=>v.id===c.target))lines.push({from:c.id,to:c.target,kind:'config',bypass:true,dim:!focus,port:s.visibleClients.indexOf(c)});continue;}
   const kind=!s.running?'warning':s.targetOnly||s.adapterConfigured&&c.target==='krill'?'config':s.observed?'observed':'config';
   lines.push({from:c.id,to:'proxy',kind,live,reverse:s.livePhase==='streaming',dim:!focus,port:s.visibleClients.indexOf(c),count:s.visibleClients.length});
  }
  for(const c of s.visibleChannels){
   if(s.direct)continue;
   if(s.actualRequest&&!s.livePhase&&![s.target,...(scene==='fallback'?['krill']:[])].includes(c.id))continue;
   const related=s.visibleClients.some(cl=>cl.connected&&cl.target===c.id),focus=s.livePhase?c.id===s.target:!selected||['proxy','desktop',c.id].includes(selected)||s.focused?.target===c.id;
   const kind=scene==='fallback'&&c.id==='krill'?'warning':s.observed&&related?'observed':'config';
   if(s.adapter&&c.id==='krill')lines.push({from:'proxy',to:'desktop',kind:'config',dim:!focus},{from:'desktop',to:c.id,kind:'config',dim:!focus});
   else lines.push({from:'proxy',to:c.id,kind:!s.running?'config':kind,live:s.flowing&&c.id===s.target,reverse:s.livePhase==='streaming',dim:!focus,port:s.visibleChannels.indexOf(c),count:s.visibleChannels.length});
  }
  return lines;
 }
 function updateTraffic(s=state()){
  const el=$('#pvActivity');el.hidden=!s.livePhase;
  if(!s.livePhase)return;
  const labels={sending:'正在发送',streaming:'正在生成',completed:'已完成',cancelled:'已取消',failed:'连接中断'};
  const count=s.usage?.output;
  el.dataset.phase=s.livePhase;
  el.innerHTML='<span class="pv-activity-state"><i></i>'+labels[s.livePhase]+'</span><span class="pv-activity-path">'+esc(s.focused.name)+'<span> / </span>'+esc(directory().find(c=>c.id===s.target)?.name||'')+'</span>'+(preferences.usage?'<span class="pv-token-count">'+(s.livePhase==='sending'?'用量待返回':'输入 <b>12,840</b><span> / </span>输出 <b>'+count.toLocaleString('en-US')+'</b> token')+'</span>':'')+(['failed','cancelled'].includes(s.livePhase)?'<span class="pv-usage-pending">用量待核对</span>':'');
  $('#pvRequest').textContent=s.livePhase==='streaming'?'回复返回中 · 用量为模拟数据':s.livePhase==='sending'?'请求发送中 · 用量待上游返回':s.livePhase==='completed'?'本次请求已完成 · 用量为模拟数据':'请求已停止 · 最终用量待核对';
  $('#pvRequest').disabled=true;
 }
 function syncTraffic(){
  clearTimeout(trafficTimer);trafficTimer=null;
  if(!active()||state().livePhase!=='streaming')return;
  trafficTimer=setTimeout(()=>{
   trafficTimer=null;if(!active()||state().livePhase!=='streaming')return;
   trafficOutput=Math.min(1286,trafficOutput+12);
   if(trafficOutput===1286){scene='completed';render();return;}
   updateTraffic();syncTraffic();
  },600);
 }
 let drawPending=false;
 function scheduleDraw(){if(drawPending)return;drawPending=true;requestAnimationFrame(()=>{drawPending=false;draw();});}
 function draw(){
  if(!active())return;
  const r=canvas.getBoundingClientRect(),width=canvas.clientWidth||r.width,height=canvas.clientHeight||r.height;if(!r.width||!r.height||!width||!height)return;
  const sx=r.width/width,sy=r.height/height;
  function point(id,side,offset=0){
   const el=$('[data-node="'+id+'"]',canvas);if(!el)return null;
   const target=id==='desktop'?$('.pv-desktop-logo-hit',el):$('.pv-tile',el)||el,b=target.getBoundingClientRect();
   const cx=(b.left+b.width/2-r.left)/sx,cy=(b.top+b.height/2-r.top)/sy,rx=b.width/sx/2,ry=b.height/sy/2;
   if(side==='t'||side==='b')return{x:cx,y:cy+(side==='t'?-1:1)*(ry+8)};
   const dy=Math.max(-ry*.65,Math.min(ry*.65,offset)),dx=Math.sqrt(Math.max(0,rx*rx-dy*dy));
   return{x:cx+(side==='l'?-1:1)*(dx+10),y:cy+dy};
  }
  function elbow(a,b,rail){
   if(Math.abs(a.y-b.y)<12)return 'M '+a.x+' '+a.y+' L '+b.x+' '+b.y;
   const dir=b.y>a.y?1:-1,radius=Math.max(1,Math.min(18,Math.abs(b.y-a.y)/2,(rail-a.x)/2,(b.x-rail)/2));
   return 'M '+a.x+' '+a.y+' H '+(rail-radius)+' Q '+rail+' '+a.y+' '+rail+' '+(a.y+dir*radius)+' V '+(b.y-dir*radius)+' Q '+rail+' '+b.y+' '+(rail+radius)+' '+b.y+' H '+b.x;
  }
  const edges=connections(),svg=$('.pv-edges',canvas);svg.setAttribute('viewBox','0 0 '+width+' '+height);
  svg.innerHTML='<defs>'+['observed','config','warning','return'].map(kind=>'<marker id="pv-arrow-'+kind+'" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="7" markerHeight="7" markerUnits="userSpaceOnUse" orient="auto-start-reverse"><path d="M1 1L7 4L1 7Z" class="pv-arrow '+kind+'"/></marker>').join('')+'</defs>'+edges.map(e=>{
   const port=(e.port-(e.count-1)/2)*(width<680?12:16)||0;
   const a=point(e.from,e.vertical?'t':'r',e.from==='proxy'?e.to==='desktop'?18:port:0),b=point(e.to,e.vertical?'b':'l',e.to==='proxy'?port:0);if(!a||!b)return '';
   let d;
   if(e.vertical){const x=width*.22;d='M '+a.x+' '+a.y+' H '+x+' V '+b.y+' H '+b.x;}
   else if(e.bypass){const rail=width*(.42+e.port*.06);d=elbow(a,b,rail);}
   else if(e.to==='proxy'){
    const labelHalf=width<680?80:95,maxRail=width*.37-labelHalf-14;
    const rail=Math.max(a.x+12,Math.min(maxRail,width*.255)-(e.count-1-e.port)*10);
    d=elbow(a,b,rail);
   }else if(e.to==='desktop')d=elbow(a,b,b.x-24);
   else if(e.from==='desktop')d=elbow(a,b,(a.x+b.x)/2);
   else d=elbow(a,b,width*(.67+e.port*.025));
   const cls=(e.dim?' is-dim':'')+(e.live?' is-live':'')+(e.live&&e.reverse?' is-return':'');
   const flow=e.live&&preferences.flow?'<g class="pv-flow '+(e.reverse?'is-return':'is-send')+'" data-flow-from="'+(e.reverse?e.to:e.from)+'" data-flow-to="'+(e.reverse?e.from:e.to)+'">'+[0,1,2].map(i=>'<path class="pv-flow-pulse" pathLength="1000" d="'+d+'" style="--delay:-'+(i*.9)+'s"/>').join('')+'</g>':'';
   return '<path d="'+d+'" class="pv-wire-halo'+(e.dim?' is-dim':'')+'"/><path data-from="'+e.from+'" data-to="'+e.to+'" '+(e.kind!=='manage'?(e.live&&e.reverse?'marker-start="url(#pv-arrow-return)"':'marker-end="url(#pv-arrow-'+e.kind+')"'):'')+' d="'+d+'" class="pv-wire '+e.kind+cls+'"/>'+flow;
  }).join('');
 }
 function syncFrame(){
  const on=active(),effects=root.dataset.effects||'high',renderer=root.dataset.glowRender||'css',key=effects+'|'+renderer;
  if(!on&&!frameKey)return;
  if(on&&key!==frameKey){const url=new URL(frameSource);url.search='?embed=1&layout=hero&effects='+effects+'&render='+renderer;frame.src=url.href;frameKey=key;ready=false;payloadKey='';}
  const w=canvas.clientWidth||880,h=canvas.clientHeight||530,size=w<680?68:86;
  // 动态聚合云仅表示Desktop产品形象；OpenCodex服务另用自身静态图标。
  const logoX=w*.64,logoY=h*(scene==='direct'?.10:w<680?.66:.73);
  frame.style.width=(w+100)+'px';frame.style.height=(h+180)+'px';frame.style.left=(logoX-(w+100)/2)+'px';frame.style.top='-60px';
  const payload={type:'opencodex-motion-preview',state:state().motion,theme:root.dataset.theme,active:on,presentation:{size,padding:logoY+60-size/2}},serial=JSON.stringify(payload);
  if(ready&&serial!==payloadKey){frame.contentWindow?.postMessage(payload,location.protocol==='file:'?'*':location.origin);payloadKey=serial;}
  host.dataset.active=String(on);
 }
 function facts(id){
  const s=state(),c=clients.find(c=>c.id===id),channel=directory().find(c=>c.id===id);
  if(c)return {title:c.name,rows:[['本机存在',c.installed?'已确认 · '+c.evidence:'未确认'],['版本',c.version||'未知'],['接入',!c.connected?'未接入':s.direct?'配置直连':'OpenCodex 本地实例'],['路径依据',s.adapterConfigured&&c.target==='krill'?'目标配置，适配待观察':s.observed?'近期请求记录（示例）':'已保存配置（示例）']]};
  if(channel)return {title:channel.name,rows:[['服务节点',channel.node],['模型',channel.model],['连接探测',channel.probe],['路径',s.adapterConfigured&&id==='krill'?'Desktop 后台适配（目标配置）':'OpenCodex 聚合选路'],['请求记录',s.adapterConfigured&&id==='krill'?'适配待观察':s.observed?'14:32 · 示例':'尚未观察']]};
  if(id==='desktop')return {title:'OpenCodex Desktop',rows:[['版本','0.2.17（示例）'],['标识','本项目聚合云图标与动态图形'],['职责','状态、配置、更新和同步'],['能力适配','按需后台托管'],['日常请求','不经管理界面转发']]};
  return {title:'OpenCodex',rows:[['状态',s.status],['标识','OpenCodex 自有终端云标识'],['版本','2.84.0（示例）'],['端口','10100'],['渠道',directory().length+' 个已配置渠道'],['聚合','由原版 OpenCodex 选路与回退']]};
 }
 function nodeTabs(id){
  if(id==='proxy')return [['info','概况'],['connection','连接'],['actions','操作'],['updates','更新']];
  if(id==='desktop')return [['info','概况'],['connection','后台适配'],['sync','同步'],['updates','更新']];
  if(clients.some(c=>c.id===id))return [['info','概况'],['connection','接入'],['actions','操作']];
  return [['info','概况'],['connection','服务节点'],['model','模型']];
 }
 function tabMarkup(items,current){return '<div class="pv-dialog-tabs" role="tablist" aria-label="'+(dialog.type==='settings'?'概览设置分组':'节点信息分组')+'">'+items.map(([id,name])=>'<button type="button" role="tab" id="pv-tab-'+id+'" aria-controls="pv-tabpanel" aria-selected="'+(id===current)+'" tabindex="'+(id===current?'0':'-1')+'" data-pv="tab" data-tab="'+id+'">'+name+'</button>').join('')+'</div>';}
 function description(text){return '<p class="pv-dialog-copy">'+text+'</p>';}
 function factMarkup(rows){return '<dl class="pv-dialog-facts">'+rows.map(([k,v])=>'<div><dt>'+esc(k)+'</dt><dd>'+esc(v)+'</dd></div>').join('')+'</dl>';}
 function settingsBody(){
  if(dialog.tab==='clients')return description('只选择已确认存在的本机客户端。显示调整不改变客户端接入。')+'<div class="pv-option-list">'+clients.map(c=>'<label class="pv-option '+(!c.installed?'is-disabled':'')+'"><input type="checkbox" data-client="'+c.id+'" '+(draft.clients.includes(c.id)?'checked':'')+' '+(!c.installed?'disabled':'')+'><span>'+c.name+'<small>'+c.evidence+(!c.connected&&c.installed?' · 未接入':'')+'</small></span></label>').join('')+'</div>';
  if(dialog.tab==='channels')return description('固定常用渠道，其他项仍可搜索展开。默认窗口显示三项，窄窗两项；只调整展示，不修改选路。')+'<div class="pv-pin-options">'+channels.map(c=>'<label><input type="checkbox" data-pin="'+c.id+'" '+(draft.pins.includes(c.id)?'checked':'')+'>'+c.name+'</label>').join('')+'</div>';
  return description('主题、背景光和特效档位沿用全局设置。管理关系在选中相关节点时展示。后台适配与同步状态归入 Desktop 节点。')+'<div class="pv-option-list">'+[['detail','详细节点信息'],['management','选中时显示管理关系'],['sync','配置同步'],['backup','备用渠道'],['flow','请求流动效果'],['usage','本次 token 用量']].map(([id,name])=>'<label class="pv-option"><input type="checkbox" data-option="'+id+'" '+(draft[id]?'checked':'')+'>'+name+'</label>').join('')+'</div>'+button('恢复默认显示','settings-reset','','pv-reset');
 }
 function nodeBody(){
  const id=dialog.id,tab=dialog.tab,s=state(),c=clients.find(c=>c.id===id),channel=directory().find(c=>c.id===id);
  if(tab==='info')return factMarkup(facts(id).rows);
  if(tab==='updates')return factMarkup([['当前版本',id==='desktop'?'0.2.17':'2.84.0'],['更新状态','待检查'],['更新入口','沿用现有更新中心']])+'<div class="pv-dialog-actions">'+button('检查更新','check-update','data-id="'+id+'"','pv-primary')+'</div><p id="pvUpdateResult" class="pv-dialog-result" role="status"></p>';
  if(id==='proxy'&&tab==='actions')return description('对本机 OpenCodex 执行操作。停止后仍保留客户端和渠道配置。')+'<div class="pv-dialog-actions">'+button(icon('panel')+'打开面板','panel','','pv-primary')+button(icon('restart')+(s.running?'重启':'启动'),s.running?'restart':'start')+(s.running?button('停止 OpenCodex','stop','','pv-danger'):'')+button('查看日志','navigate','data-route="logs"')+'</div>';
  if(id==='desktop'&&tab==='sync')return description('文件同步与 WebDAV 属于 Desktop，由这里进入对应操作。')+factMarkup([['最近同步','14:20 · WebDAV 已同步（示例）'],['对话配置','默认排除'],['对话历史','仅用于本地重装恢复']])+'<div class="pv-dialog-actions">'+button('打开文件同步','sync','data-tab="file"','pv-primary')+button('打开 WebDAV','sync','data-tab="webdav"')+'</div>';
  if(channel&&tab==='model'){const preview=modelPreviews[id];return factMarkup([['型号',preview?.model||channel.model],['推理 / 快速',preview?.options||'跟随已有模型设置'],['选择范围','渠道模型、推理档位与快速模式']])+'<div class="pv-dialog-actions">'+button('选择模型','model','data-id="'+id+'"','pv-primary')+button('打开渠道管理','navigate','data-route="models"')+'</div>';}
  if(c&&tab==='actions')return '<div class="pv-dialog-actions">'+button('查看接入配置','tab','data-tab="connection"','pv-primary')+button('在画布聚焦此客户端','focus-path','data-id="'+id+'"')+'</div>';
  if(tab==='connection'){
   if(id==='desktop')return description('后台适配是 OpenCodex Desktop 的一项能力，与管理、更新和同步共用产品节点。后台组件位于 OpenCodex 选路之后，仅处理明确配置的出站支路。')+factMarkup([['产品归属','Desktop 后台能力'],['目标路径','客户端 → OpenCodex → Desktop 后台适配 → 已选渠道'],['普通渠道','保持 OpenCodex 原通路'],['当前证据','目标配置示例，未接入生产'],['能力边界','不代表原生压缩或 history/notes 已打通']]);
   if(id==='proxy')return factMarkup([['入站','已接入的本机客户端'],['聚合选路','原版 OpenCodex'],['普通出站','OpenCodex → 已选渠道'],['适配支路','OpenCodex → Desktop 后台适配 → 对应渠道'],['观察','入口与出口分别判断']]);
   if(c)return factMarkup([['本机存在',c.installed?'已确认 · '+c.evidence:'未确认'],['接入状态',!c.connected?'未接入':s.direct?'配置直连':'OpenCodex 本地实例'],['下游路径',!c.connected?'没有已确认接入路径':s.direct?'客户端 → 直连渠道':s.adapterConfigured&&c.target==='krill'?'OpenCodex → Desktop 后台适配 → Krill':'OpenCodex → '+(directory().find(ch=>ch.id===c.target)?.name||'已选渠道')],['请求依据',s.adapterConfigured&&c.target==='krill'?'适配配置待观察':s.observed?'近期请求示例':'尚未观察']])+'<div class="pv-dialog-actions">'+button('在画布聚焦此客户端','focus-path','data-id="'+id+'"')+'</div>';
   if(channel)return factMarkup([['服务节点',channel.node],['连接探测',channel.probe],['出站路径',s.adapterConfigured&&id==='krill'?'OpenCodex → Desktop 后台适配 → Krill':'OpenCodex → '+channel.name],['管理入口','现有渠道与服务节点页面']])+'<div class="pv-dialog-actions">'+button('打开渠道管理','navigate','data-route="models"','pv-primary')+'</div>';
  }
  return '';
 }
 function collectionBody(){
  if(dialog.type==='clients')return description('已勾选但在画布中收起的本机客户端。展开仅改变本次显示。')+'<div class="pv-channel-results">'+state().hiddenClients.map(c=>'<button type="button" data-pv="expand-client" data-id="'+c.id+'"><span>'+c.name+'</span><small>'+c.evidence+'</small></button>').join('')+'</div>';
  const hidden=state().hiddenChannels,results=hidden.filter(c=>(c.name+' '+c.model).toLowerCase().includes(search.trim().toLowerCase()));
  return '<label class="pv-search-label" for="pvChannelSearch">搜索渠道或模型</label><input class="input" id="pvChannelSearch" type="search" aria-label="搜索其他渠道" placeholder="输入名称或型号" value="'+esc(search)+'"><div class="pv-channel-results" id="pvChannelResults">'+channelResults(results)+'</div>';
 }
 function channelResults(rows){return rows.length?rows.map(c=>'<button type="button" data-pv="expand" data-id="'+c.id+'"><span>'+esc(c.name)+'</span><small>'+esc(c.model)+'</small></button>').join(''):'<p>没有匹配的渠道</p>';}
 function renderDialog(){
  if(!dialog)return;const body=document.getElementById('modalBody');
  if(dialog.type==='settings'){const tabs=[['clients','客户端'],['channels','渠道'],['display','显示']];body.innerHTML=tabMarkup(tabs,dialog.tab)+'<div id="pv-tabpanel" class="pv-dialog-panel" role="tabpanel" aria-labelledby="pv-tab-'+dialog.tab+'">'+settingsBody()+'</div>';}
  else if(dialog.type==='node')body.innerHTML=tabMarkup(nodeTabs(dialog.id),dialog.tab)+'<div id="pv-tabpanel" class="pv-dialog-panel" role="tabpanel" aria-labelledby="pv-tab-'+dialog.tab+'">'+nodeBody()+'</div>';
  else body.innerHTML='<div class="pv-dialog-panel">'+collectionBody()+'</div>';
 }
 function nodeOpener(id){return $('[data-pv="node"][data-id="'+id+'"]')||$('.pv-settings-trigger');}
 function openDialog(type,id=null,tab=null){
  if(dialog)closeModal();
  const opener=document.activeElement,session={type,id,tab:tab||(type==='settings'?'clients':'info'),opener};
  dialog=session;if(type==='settings')draft=clone(preferences);search='';
  const title=type==='settings'?'概览设置':type==='channels'?'其他渠道 · '+state().hiddenChannels.length:type==='clients'?'其他本机客户端':facts(id).title;
  openModal(title,'',type==='settings'?'应用显示':null,()=>{
   if(dialog!==session||!draft)return;preferences=clone(draft);draft=null;closeModal();if(clients.some(c=>c.id===selected)&&!preferences.clients.includes(selected))selected=null;render();$('.pv-settings-trigger').focus({preventScroll:true});
  },{hideConfirm:type!=='settings',confirmKeepsOpen:true,cancelLabel:type==='settings'?'取消':'关闭',prototypeAnnotation:{title:'全景概览 · 居中弹窗',body:'本专题仅修改内存显示和样例状态，不扫描安装、不请求网络或写入运行配置。后台适配是目标配置示例，未证明上下文能力已打通。'}});
  const modal=document.getElementById('modal');session.modalAttrs=['role','aria-modal','aria-labelledby'].map(name=>[name,modal.getAttribute(name)]);modal.classList.add('pv-dialog');modal.dataset.pvDialog=type;modal.setAttribute('role','dialog');modal.setAttribute('aria-modal','true');modal.setAttribute('aria-labelledby','modalTitle');
  window.__modalOnClose=()=>{
   if(dialog!==session)return;dialog=null;draft=null;modal.classList.remove('pv-dialog');delete modal.dataset.pvDialog;for(const [name,value] of session.modalAttrs){if(value===null)modal.removeAttribute(name);else modal.setAttribute(name,value);}
   requestAnimationFrame(()=>{if(!active()||document.getElementById('modalMask').style.display==='flex')return;const target=type==='settings'?$('.pv-settings-trigger'):opener?.isConnected?opener:nodeOpener(id);target?.focus({preventScroll:true});});
  };
  renderDialog();
  requestAnimationFrame(()=>{if(dialog===session)(type==='channels'?document.getElementById('pvChannelSearch'):document.querySelector('#modal .pv-dialog-tabs [aria-selected="true"]')||document.getElementById('modalCancel'))?.focus({preventScroll:true});});
 }
 function openNode(id,tab='info'){selected=id;requestFocus=false;render();nodeOpener(id)?.focus({preventScroll:true});openDialog('node',id,tab);}
 function showNotice(text){$('#pvNotice').textContent=text;$('#pvNotice').hidden=false;clearTimeout(showNotice.timer);showNotice.timer=setTimeout(()=>{$('#pvNotice').hidden=true;},2600);}
 function modelChoice(id){
  const c=directory().find(c=>c.id===id);if(!c)return;
  const prior={id,tab:'model'},selectedRef=modelPreviews[id]?.ref||{kind:'channel',channelId:c.id,modelId:c.model};closeModal();
  window.modelPickerPrototype?.choose({title:'选择渠道模型',mode:'single',reasoning:true,fast:true,models:[clone(selectedRef)],getCatalog:()=>window.modelPrototype?.agentChoices?.()||[],onSelect:result=>{
   const ref=result.models[0];if(ref){modelPreviews[id]={ref:clone(ref),model:ref.modelId||ref.nativeModelId,options:(effortLabels[ref.reasoningEffort]||'模型默认')+(ref.fast?' · Fast':''),label:(ref.modelId||ref.nativeModelId)+(ref.reasoningEffort?' · '+(effortLabels[ref.reasoningEffort]||ref.reasoningEffort):'')+(ref.fast?' · Fast':'')};}
   render();openDialog('node',prior.id,prior.tab);
  },onCancel:result=>{if(active()&&!result?.routeChanged)openDialog('node',prior.id,prior.tab);}});
 }
 function handleClick(e){
  const modalScope=e.target.closest('#modal.pv-dialog');if(!host.contains(e.target)&&!modalScope)return;
  const b=e.target.closest('[data-pv]');if(!b)return;const a=b.dataset.pv,id=b.dataset.id;
  if(a==='settings')openDialog('settings');
  else if(a==='node')openNode(id,b.dataset.tab||'info');
  else if(a==='tab'){if(!dialog)return;dialog.tab=b.dataset.tab;renderDialog();document.getElementById('pv-tab-'+dialog.tab)?.focus({preventScroll:true});}
  else if(a==='settings-reset'){draft=clone(defaults);renderDialog();document.querySelector('#modal [data-pv="settings-reset"]')?.focus({preventScroll:true});}
  else if(a==='channels')openDialog('channels');
  else if(a==='clients')openDialog('clients');
  else if(a==='expand'||a==='expand-client'){if(a==='expand')temporary=id;else temporaryClient=id;selected=id;requestFocus=false;closeModal();render();nodeOpener(id)?.focus({preventScroll:true});}
  else if(a==='focus-path'){selected=id;requestFocus=false;closeModal();render();nodeOpener(id)?.focus({preventScroll:true});}
  else if(a==='request'){requestFocus=!requestFocus;render();$('#pvRequest').focus({preventScroll:true});}
  else if(a==='stop'){operation='stopped';render();renderDialog();showNotice('OpenCodex 已停止（模拟），配置关系保留。');}
  else if(a==='restart'||a==='start'){operation='';if(scene==='stopped')scene='recent';render();renderDialog();showNotice('OpenCodex 已就绪（模拟）。');}
  else if(a==='check-update'){document.getElementById('pvUpdateResult').textContent='检查已完成（内存示例），未连接更新服务。';}
  else if(a==='model')modelChoice(id);
  else if(a==='panel'){closeModal();setRoute('panel');}
  else if(a==='sync'){closeModal();setRoute('sync?tab='+b.dataset.tab);}
  else if(a==='navigate'){closeModal();setRoute(b.dataset.route);}
 }
 document.addEventListener('click',handleClick);
 document.addEventListener('change',e=>{
  if(!draft||!e.target.closest('#modal.pv-dialog'))return;const el=e.target;
  if(el.dataset.client&&clients.some(c=>c.id===el.dataset.client&&c.installed)){draft.clients=draft.clients.filter(id=>id!==el.dataset.client);if(el.checked)draft.clients.push(el.dataset.client);}
  if(el.dataset.pin){draft.pins=draft.pins.filter(id=>id!==el.dataset.pin);if(el.checked)draft.pins.push(el.dataset.pin);}
  if(el.dataset.option)draft[el.dataset.option]=el.checked;
 });
 document.addEventListener('input',e=>{if(dialog?.type!=='channels'||e.target.id!=='pvChannelSearch')return;search=e.target.value;const results=state().hiddenChannels.filter(c=>(c.name+' '+c.model).toLowerCase().includes(search.trim().toLowerCase()));document.getElementById('pvChannelResults').innerHTML=channelResults(results);});
 document.addEventListener('keydown',e=>{
  if(!dialog||!e.target.closest('#modal.pv-dialog'))return;
  const tabs=[...document.querySelectorAll('#modal .pv-dialog-tabs [role="tab"]')],index=tabs.indexOf(e.target);
  if(index>=0&&['ArrowLeft','ArrowRight','Home','End'].includes(e.key)){e.preventDefault();const next=e.key==='Home'?0:e.key==='End'?tabs.length-1:(index+(e.key==='ArrowRight'?1:-1)+tabs.length)%tabs.length;tabs[next].click();return;}
  if(e.key==='Tab'){const items=[...document.querySelectorAll('#modal.pv-dialog button:not([disabled]):not([hidden]),#modal.pv-dialog input:not([disabled])')].filter(el=>el.getClientRects().length&&el.tabIndex>=0),first=items[0],last=items.at(-1);if(e.shiftKey&&e.target===first){e.preventDefault();last?.focus();}else if(!e.shiftKey&&e.target===last){e.preventDefault();first?.focus();}}
 });

 host.addEventListener('error',e=>{if(e.target.matches?.('.pv-service-mark img'))e.target.closest('.pv-service-mark').classList.add('asset-missing');},true);
 const tool=document.createElement('div');tool.id='connectionMapDemoCard';tool.className='proto-card';tool.dataset.protoContexts='connection-map';
 tool.innerHTML='<div class="proto-note-title">全景概览 · 场景</div><p class="pctl-hint">Desktop 聚合云与 OpenCodex 服务图标分别展示；全部为内存样例，不扫描本机或修改配置。可在软件右上角调整展示客户端与固定渠道。</p><div class="pv-demo-scenes">'+Object.entries(scenes).map(([id,name])=>'<button type="button" class="btn ghost" data-pv-scene="'+id+'" aria-pressed="'+(scene===id)+'">'+name+'</button>').join('')+'</div><p class="pctl-hint">适配为 Desktop 后台目标示例，未证明上下文缺口已打通。近期请求保持静态；请求发送／生成场景才有定向流动，token为合成用量，流速不表示计费速度。“本机存在”的依据在概览设置中模拟。</p>';
 (document.querySelector('#protoStdCards .proto-group-body')||document.getElementById('protoStdCards')).appendChild(tool);
 function setScene(id){if(!scenes[id])return;if(dialog)closeModal();scene=id;trafficOutput=384;operation='';selected=null;temporary=null;temporaryClient=null;requestFocus=false;render();}
 tool.addEventListener('click',e=>{const b=e.target.closest('[data-pv-scene]');if(b)setScene(b.dataset.pvScene);});
 frame.addEventListener('load',()=>{ready=true;payloadKey='';syncFrame();});
 window.addEventListener('message',e=>{if(e.source===frame.contentWindow&&e.data?.type==='opencodex-motion-ready'){ready=true;payloadKey='';syncFrame();}});
 new MutationObserver(()=>{if(!active()&&dialog)closeModal();syncTraffic();syncFrame();scheduleDraw();}).observe(root,{attributes:true,attributeFilter:['data-route','data-theme','data-effects','data-glow-render']});
 if(window.ResizeObserver)new ResizeObserver(()=>{const next=(canvas.clientWidth||host.clientWidth||880)<680?2:3;if(next!==layoutBudget)render();else{syncFrame();scheduleDraw();}}).observe(canvas);
 document.addEventListener('visibilitychange',()=>{syncTraffic();syncFrame();});window.addEventListener('resize',()=>{syncFrame();scheduleDraw();});
 window.connectionMapPrototype={state:()=>({scene,selected,requestFocus,temporary,preferences:clone(preferences),draft:clone(draft),dialog:dialog?{type:dialog.type,id:dialog.id,tab:dialog.tab}:null,modelPreviews:clone(modelPreviews),...state(),connections:connections()}),setScene,draw};
 render();
})();
