/* 概览 · 流量全景（Desktop 主角）：全部为内存样例，坐标按实际布局测量；只在概览可见且非环境门禁时构建与驱动动效。
   网关未运行时停止光点回放；运行状态、启停与维护卡仍由 index.html 与 overview-dual.js 负责。 */
(function(){
'use strict';
var script=document.currentScript;
var root=document.documentElement,host=document.getElementById('route-overview');
if(!host||host.dataset.heroReady)return;
host.dataset.heroReady='1';
/* VER 参与 iframe 地址，改动 Logo 形变页后要同步升版，否则浏览器会沿用缓存的 morph.js */
var NS='http://www.w3.org/2000/svg',VER='20261102-overview-orbit2';
var SRC='../../../03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型/候选/2026-09-28-Logo本体形变/index.html';
var R=72;/* 中枢外环接线半径；矮舞台时 build() 改为 56 */

/* ---------- 样例数据 ---------- */
/* 渠道样例：只列启用且窗口内有流量的渠道。cx / cl 为来自 codex 入口 / Claude Code 的记录条数，
   via 表示经 Desktop 实时适配；tps 是该渠道完成请求的平均输出速度（token/s），
   models 是同一窗口内按模型拆开的记录——这里能给的是“这条模型经过此渠道”的用量与速度。 */
/* 24 小时基准样例；CH 是“当前窗口”的派生结果，切窗口时按系数重算（见 buildWindow） */
var CH_BASE=[
 {k:'krill',n:'Krill',lb:'Kr',req:498,fail:3,tok:.58,ttft:.71,lat:124,tps:58,cx:498,cl:0,via:1,adapterMs:18,
  models:[{n:'gpt-6.1-sol',req:320,tok:.37,tps:63},{n:'gpt-6.1-mini',req:178,tok:.21,tps:51}]},
 {k:'newapi',n:'NewApi',lb:'NA',req:1069,fail:9,tok:1.18,ttft:.64,lat:86,tps:74,cx:1069,cl:0,
  models:[{n:'gpt-6.1-sol',req:700,tok:.78,tps:78},{n:'claude-sonnet-4.5',req:369,tok:.40,tps:68}]},
 {k:'a6',n:'A6api',lb:'A6',req:617,fail:19,tok:.79,ttft:1.08,lat:162,tps:41,cx:121,cl:496,
  models:[{n:'claude-sonnet-4.5',req:496,tok:.66,tps:38},{n:'gpt-6.1-sol',req:121,tok:.13,tps:47}]},
 {k:'or',n:'OpenRouter',lb:'OR',req:214,fail:2,tok:.31,ttft:.88,lat:140,tps:52,cx:214,cl:0,
  models:[{n:'deepseek-v3.2',req:132,tok:.19,tps:55},{n:'qwen3-max',req:82,tok:.12,tps:47}]},
 {k:'ds',n:'DeepSeek',lb:'DS',req:156,fail:1,tok:.21,ttft:.93,lat:150,tps:66,cx:156,cl:0,
  models:[{n:'deepseek-v3.2',req:156,tok:.21,tps:66}]},
 {k:'sf',n:'SiliconFlow',lb:'SF',req:58,fail:5,tok:.07,ttft:1.3,lat:210,tps:88,cx:58,cl:0,
  models:[{n:'qwen3-max',req:58,tok:.07,tps:88}]},
 {k:'ol',n:'Ollama',lb:'OL',req:19,fail:0,tok:.05,ttft:.4,lat:30,tps:120,cx:19,cl:0,
  models:[{n:'llama4-scout',req:19,tok:.05,tps:120}]}
];
/* 入口侧能拆到模型：codex 入口共用一个计量，但模型名是请求里带的，可以分开统计。 */
var CMODELS={codex:[],claude:[]},PLATFORMS=[];
var IDLE=['Azure OpenAI','Groq'],OFF=['Moonshot'];
var CH=[],CLD={req:0,tok:0,fail:0},CL_TX={app:0,opencode:0,cli:0,claude:0};
/* 窗口系数：6h=1/4，24h=1，3d=3，7d=7；7 天额外把平均速度压向中位，避免“窗口越长越快”的错觉 */
/* 1 个月按 30 天、1 年按 365 天；自定义区间用实际跨度换算，见 winScale() 的自定义分支 */
function winScale(){
  /* 自定义区间按实际跨度折算成天；其余按固定档位 */
  if(customFrom&&customTo){var a=rngParse(customFrom),b=rngParse(customTo);
    if(a&&b&&b>a)return Math.max(1/24,(b-a)/864e5)}
  return win==='1 小时'?1/24:win==='1 个月'?30:win==='1 年'?365:1}
/* 长窗口把平均速度压向中位：1 个月 0.8、1 年 0.6，避免“查得越久速度越快”的错觉 */
function buildWindow(){var f=winScale(),tpsF=win==='1 年'?.6:win==='1 个月'?.8:1;
  CH=CH_BASE.map(function(b){var c={};Object.keys(b).forEach(function(k){c[k]=b[k]});
    c.req=Math.max(1,Math.round(b.req*f));c.fail=Math.round(b.fail*f);c.tok=+(b.tok*f).toFixed(3);
    c.cx=Math.round(b.cx*f);c.cl=Math.round(b.cl*f);c.tps=Math.round(b.tps*tpsF);
    c.models=b.models.map(function(m){return{n:m.n,req:Math.max(1,Math.round(m.req*f)),tok:+(m.tok*f).toFixed(3),tps:m.tps}});
    return c});
  CMODELS.codex=[{n:'gpt-6.1-sol',req:Math.round(1600*f),tok:+(1.62*f).toFixed(2),tps:71},
    {n:'gpt-6.1-mini',req:Math.round(535*f),tok:+(.91*f).toFixed(2),tps:55}];
  CLD={req:Math.round(496*f),tok:+(.66*f).toFixed(2),fail:Math.round(13*f)};
  CMODELS.claude=[{n:'claude-sonnet-4.5',req:CLD.req,tok:CLD.tok,tps:38}];
  CL_TX={app:+(1.62*f).toFixed(2),opencode:+(.62*f).toFixed(2),cli:+(.29*f).toFixed(2),claude:CLD.tok};
  if(window.OverviewPlatformData){PLATFORMS=window.OverviewPlatformData.build(CH);PLATFORMS.forEach(function(p){CL_TX[p.k]=p.tok})}
  recompute()}
function sum(a,f){return a.reduce(function(t,c){return t+(c[f]||0)},0)}
function fmt(n){return String(Math.round(n)).replace(/\B(?=(\d{3})+(?!\d))/g,',')}
function okr(c){return c.req?(c.req-c.fail)/c.req*100:100}
function p1(x){return(Math.round(x*10)/10).toFixed(1)}
function mm(x){return(Math.round(x*100)/100).toFixed(2)}
var T={req:sum(CH,'req'),tok:sum(CH,'tok'),fail:sum(CH,'fail')};T.ok=okr(T);
var CXT={req:T.req-CLD.req,tok:T.tok-CLD.tok,fail:T.fail-CLD.fail};CXT.ok=okr(CXT);
var SET={gemini:['gemini']};/* 其余由 renderCh() 按可见渠道生成 */
var ACT_CH=['查看请求','渠道设置'],ACT_CL=['查看请求','分配独立密钥'];
var GO={'查看请求':'logs','查看实时请求':'logs','网关日志':'logs','渠道设置':'models','全部渠道':'models'};
var INFO={
  desktop:{n:'OpenCodeX-Desktop',tag:'内置网关 :10100',c:'core',p:'本卡为当前窗口内各渠道记录的合计；未记录到的用量不计入，管理界面不转发日常请求',d:[['请求',fmt(T.req)+' 次'],['Token',mm(T.tok)+'M'],['成功率',p1(T.ok)+'%'],['输出速度',Math.round(sum(CH,'tps')/CH.length)+' t/s'],['TTFT P50','0.82s']],a:['网关日志','网关设置'],seed:4,models:CMODELS.codex},
  adapter:{n:'实时适配',tag:'Desktop 代理',c:'krill',p:'Desktop 把请求转换为 Krill 协议并实时转发，所以这条线多一次协议转换',d:[['请求','498 次'],['附加延迟','+18ms'],['类型','协议适配']],a:['查看实时请求','适配设置'],seed:6},
  codex:{n:'codex 入口',tag:'入口',c:'codex',p:'Codex App、OpenCode、Codex CLI 的请求在这里汇合；网关记的是入口这一层的用量',d:[['请求',fmt(CXT.req)+' 次'],['Token',mm(CXT.tok)+'M'],['成功率',p1(CXT.ok)+'%'],['记录到的渠道',CH.filter(function(c){return c.cx>0}).length+' 个'],['输出速度',Math.round((1600*71+535*55)/2135)+' t/s']],a:ACT_CL,seed:2,models:CMODELS.codex},
  claude:{n:'Claude Code',tag:'surface: claude',c:'claude',p:'全部发往 A6api',d:[['请求','496 次'],['Token','0.66M'],['成功率','97.4%'],['失败','13 次'],['输出速度','38 t/s'],['TTFT P50','1.12s']],a:ACT_CL,seed:9,models:CMODELS.claude},
  gemini:{n:'Gemini CLI',tag:'直连',c:'gemini',p:'直接请求 Google 官方接口，不经 OpenCodex，因此没有用量与日志',d:[['请求','—'],['Token','—'],['接管','未接入']],a:['接入 OpenCodex','了解直连'],seed:0}
};
var ROUTES=[];/* renderCh() 按可见渠道生成 */
var ST={running:'运行中',starting:'启动中',stopping:'停止中',stopped:'已停止',failed:'启动失败',problem:'有异常',not_ready:'未就绪',confirming:'确认中',stale:'状态过期'};
var AMB=['starting','stopping','confirming','stale','not_ready'],BAD=['failed','problem','stopped'];
/* RATE 是每秒生成几批光点，SPD 是光点在线上每秒走的像素；
   两个值都按“基准速度”取，实际再乘以速度档位。基准调高一些，画面才不至于慢吞吞。 */
/* 基础速率：SPD 是光点在线上每秒走的像素。按评审意见取 80px/s（基准档位 1.0 时的速度）。
   速度越慢，光点在线上停留越久，同屏点数会成倍增加，生成率同步降到 5.5 次/秒维持疏密。 */
var TOT=0,RATE=5.5,SPD=80,tempo=1;
/* ---------- 速度模型 ----------
   全部动效由两个输入驱动：请求速率（每秒记录到的请求数）与各渠道的输出速度（token/s）。
   Hz = 光点在线上移动的速度档位，涟漪、圆环扫光、Logo 游走都换算自同一个值，所以“画面越快 = 实际越快”。 */
var TPS_MED=52;            /* 视为“正常”的输出速度，用于把 token/s 换算成 0.55–1.9 的档位 */
function ratePerSec(){return T.req/Math.max(1,winSec())}
function winSec(){return win==='1 小时'?3600:win==='1 个月'?2592000:win==='1 年'?31536000:86400}
/* 收纳卡等没有单一渠道速度的节点，退回 1（中速），保证 v 永远是有限数 */
/* ---------- 原型侧栏的流量控制 ----------
   只改内存样例：场景调整各渠道 tps / 失败数，两个滑块调整全站速度与请求速率倍率。
   改动后重算速度档位并重绘，让“调得多快、画面就多快”能被评审直接演示。 */
var FSCALE={gear:1,rate:1,scene:'normal'},SCENE_BASE=null;
function flowSnapshot(){if(!SCENE_BASE)SCENE_BASE=CH.map(function(c){return{k:c.k,tps:c.tps,fail:c.fail,models:c.models?c.models.map(function(m){return{n:m.n,tps:m.tps}}):null}})}
function applyFlowScene(name){flowSnapshot();FSCALE.scene=name;
  CH.forEach(function(c){var b=SCENE_BASE.filter(function(x){return x.k===c.k})[0];if(!b)return;
    c.tps=b.tps;c.fail=b.fail;if(b.models&&c.models)c.models.forEach(function(m){var bm=b.models.filter(function(x){return x.n===m.n})[0];if(bm)m.tps=bm.tps})});
  if(name==='slow'){CH.forEach(function(c){c.tps=Math.round(c.tps*.35);if(c.models)c.models.forEach(function(m){m.tps=Math.round(m.tps*.35)})})}
  else if(name==='fast'){/* 只让低量渠道提速，用来演示“同样请求量下速度不同”的画面差异 */
    CH.forEach(function(c){if(c.req<250){c.tps=Math.round(c.tps*3.2);if(c.models)c.models.forEach(function(m){m.tps=Math.round(m.tps*3.2)})}})}
  else if(name==='fail'){var a=CH.filter(function(c){return c.k==='a6'})[0];if(a){a.fail=Math.round(a.req*.28);if(a.models&&a.models[0])a.models.forEach(function(m){m.tps=Math.round(m.tps*.3)})}}
  /* 场景改的是各渠道速度，必须重算档位并把动效参数重新下发，否则画面停在旧速度 */
  if(window.OverviewPlatformData)PLATFORMS=window.OverviewPlatformData.build(CH);
  renderCh(K);paintSpeed();syncFlowUi();if(built)build();else if(journey&&journey.active())journey.refresh();if(focus)apply()
}
/* 把当前速度档位写成 CSS 变量，并同步给 Logo 游走：圆环扫光、客户端边缘、涟漪都读它 */
function paintSpeed(){var gear=globalGear(),tone=hero?hero.dataset.tone:'run';
  /* 扫光一圈的基础时长单列，避免它跟着“光点生成率”一起被放大 */
  hero.style.setProperty('--ph-sweep-dur',(3.4/gear*(tone==='amber'?3:1)).toFixed(2)+'s');
  hero.style.setProperty('--ph-gear',gear.toFixed(2));
  tempo=tone==='run'?gear:tone==='amber'?gear/3:0;
  post()
}
function syncFlowUi(){var g=document.getElementById('flowGearReadout');
  if(g)g.textContent=globalGear().toFixed(2)+'×';
  var hint=document.getElementById('flowGearHint'),gg=globalGear();
  if(hint)hint.textContent=gg<.7?'— 慢速：光点稀疏、扫光放慢':gg<1.15?'— 中速：接近基准':gg<1.55?'— 较快：光点更密、扫光加快':'— 高速：画面明显加速';
  var gr=document.getElementById('flowGearRange'),rr=document.getElementById('flowRateRange');
  if(gr)gr.value=String(Math.round(FSCALE.gear*100));
  if(rr)rr.value=String(Math.round(FSCALE.rate*100));
  if(gr)gr.setAttribute('aria-valuetext',FSCALE.gear.toFixed(2)+' 倍');
  if(rr)rr.setAttribute('aria-valuetext',FSCALE.rate.toFixed(2)+' 倍');
  document.querySelectorAll('[data-flow-scene]').forEach(function(b){
    var on=b.dataset.flowScene===FSCALE.scene;b.classList.toggle('active',on);b.setAttribute('aria-pressed',String(on))})}
/* 侧栏卡片可能被其它原型脚本重建，事件委托绑到 document 上，按钮换新也能用 */
function bindFlowControls(){if(bindFlowControls.done)return;bindFlowControls.done=true;
  document.addEventListener('click',function(e){
    var b=e.target.closest&&e.target.closest('[data-flow-scene]');
    if(b){applyFlowScene(b.dataset.flowScene);return}});
  document.addEventListener('input',function(e){
    var t=e.target;
    if(!t||!t.id)return;
    if(t.id==='flowGearRange'){FSCALE.gear=Math.max(.5,Math.min(2,Number(t.value)/100));paintSpeed();syncFlowUi()}
    else if(t.id==='flowRateRange'){FSCALE.rate=Math.max(.1,Math.min(3,Number(t.value)/100));syncFlowUi()}})}
function speedGear(tps){var v=Number.isFinite(tps)&&tps>0?tps/TPS_MED:1;return Math.max(.55,Math.min(1.9,v))}
/* 全站速度：请求速率占主导，再叠加各渠道输出速度的加权均值 */
function globalGear(){
  var w=0,s=0;CH.forEach(function(c){w+=c.req;s+=c.req*speedGear(c.tps)});
  /* 请求速率用对数尺度：样例 24h 仅约 2.6k 次（0.03 req/s），线性比值会被下限吃掉，
     这里把 0.03 req/s 视为中速，速率翻十倍约升到 1.55 档，速率这才真的能驱动画面。 */
  var byTok=w?s/w:1,rps=ratePerSec();
  var byRate=Math.max(.6,Math.min(1.9,Math.pow(Math.max(.001,rps)/.03,.36)));
  return Math.max(.7,Math.min(2.2,(byTok*.5+byRate*.5)*FSCALE.gear));
}

/* ---------- 标记 ---------- */
/* 节点本体：客户端用内置图标，渠道默认用中转名称缩写（后续允许上传统一规格的图标，o.img）。
   外环用节点自身颜色，按失败占比在环上留出红色刻痕；成功率低于 98% 时整环转琥珀。 */
function use(id){return'<svg class="ph-ico" viewBox="0 0 24 24" aria-hidden="true"><use href="#'+id+'"/></svg>'}
var CLI_ICO='<svg class="ph-ico" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3 4.5 6.5 8 3 11.5M8.5 11.8h4.5"/></svg>';
function orb(label,o){o=o||{};var body=o.img?'<img class="ph-img" src="'+o.img+'" alt="">':(o.ico||label);
  var warn=o.ok!=null&&o.ok<98?' ph-warn':'',ring='';
  return'<span class="ph-orb'+warn+'"><i class="ph-ring"'+ring+'></i>'+body+'</span>'}
/* 客户端用量由 buildWindow() 按窗口写入 CL_TX；这里只负责拼装 */
function clientsHtml(){return'<div class="ph-clients">'+
 '<div class="ph-grp">'+
  /* 每个客户端在自己的名称下写自己的 Token，不再用左侧列头“Token”统一指代 */
  '<div class="ph-node ph-client ph-unsplit" data-client="app" data-k="codex" data-f="codex" style="--c:var(--ph-codex)">'+orb('CX',{ico:use('ext-icon-codex')})+'<div class="ph-nl"><div class="ph-l1"><b>Codex App</b></div><div class="ph-l2"><span class="ph-tok">'+mm(CL_TX.app)+'M</span></div></div></div>'+
  '<div class="ph-node ph-client ph-unsplit" data-client="opencode" data-k="codex" data-f="codex" style="--c:var(--ph-opencode)">'+orb('OC',{ico:use('ext-icon-opencode')})+'<div class="ph-nl"><div class="ph-l1"><b>OpenCode</b></div><div class="ph-l2"><span class="ph-tok">'+mm(CL_TX.opencode)+'M</span></div></div></div>'+
  '<div class="ph-node ph-client ph-unsplit" data-client="cli" data-k="codex" data-f="codex" style="--c:var(--ph-cli)">'+orb('&gt;_',{ico:CLI_ICO})+'<div class="ph-nl"><div class="ph-l1"><b>Codex CLI</b></div><div class="ph-l2"><span class="ph-tok">'+mm(CL_TX.cli)+'M</span></div></div></div>'+
  '<div class="ph-node ph-client" data-client="claude" data-k="claude" data-f="claude" style="--c:var(--ph-claude)">'+orb('CC',{ico:use('ext-icon-claude'),ok:97.4})+'<div class="ph-nl"><div class="ph-l1"><b>Claude Code</b></div><div class="ph-l2"><span class="ph-tok">'+mm(CL_TX.claude)+'M</span><span class="ph-low">97.4%</span></div></div></div>'+
'</div></div>'}
/* 客户端列的用量与“最近记录到的渠道”会随窗口变化，需要单独重绘 */
function renderClients(){if(!stage)return;var box=stage.querySelector('.ph-clients');if(!box)return;
  var wrap=document.createElement('div');wrap.innerHTML=clientsHtml();
  box.replaceWith(wrap.firstChild);
  a11y(QA('.ph-clients [data-f]'));
}
/* 中枢下方控制区：运行状态 + 进程操作。按钮是顶部状态卡按钮的镜像，点击转发给原按钮，逻辑只有一套。 */
var IC={
 start:'<svg viewBox="0 0 16 16" fill="currentColor" aria-hidden="true"><path d="M5 3.2v9.6l7.6-4.8z"/></svg>',
 panel:'<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M13.5 8.6v3.1a1.8 1.8 0 0 1-1.8 1.8H4.3a1.8 1.8 0 0 1-1.8-1.8V4.3a1.8 1.8 0 0 1 1.8-1.8h3.1"/><path d="M9.6 2.5h3.9v3.9M13.1 2.9 7.6 8.4"/></svg>',
 stop:'<svg viewBox="0 0 16 16" fill="currentColor" aria-hidden="true"><rect x="4" y="4" width="8" height="8" rx="1.7"/></svg>',
 restart:'<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M13 8a5 5 0 1 1-1.5-3.6"/><path d="M11.7 2.3v2.7H9"/></svg>',
 logs:'<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3.6 2.5h5.9l2.9 2.9v8.1H3.6z"/><path d="M9.4 2.5v3h3M5.8 8.2h4.4M5.8 10.6h3.2"/></svg>'};
var CTL_ACT=[['start','启动'],['panel','打开面板'],['stop','停止'],['restart','重启'],['logs','查看日志']];
/* 运行状态已由中枢标题前的圆点与“· 已停止”这类后缀表达，这里不再重复一枚“未运行”文字标签；状态文字保留在隐藏节点里，读屏仍能读到 */
var CTL_HTML='<div class="ph-hubctl" id="phHubCtl"><span class="ph-cst" role="status" aria-live="polite" title="进程状态"><i class="ph-cdot"></i><b id="phCstT" class="sr-only">—</b></span><div class="ph-cbtns" role="group" aria-label="进程操作">'+
  CTL_ACT.map(function(a){return'<button type="button" class="ph-cbtn" data-ph-mirror="'+a[0]+'" aria-label="'+a[1]+'" title="'+a[1]+'">'+IC[a[0]]+'</button>'}).join('')+'</div></div>';

/* 中枢不再挂原生 title：玻璃卡已经说明了名称与端口，原生提示条会盖在卡片上 */
var HUB='<div class="ph-hub" id="phHub" data-f="desktop" aria-label="OpenCodeX-Desktop 内置网关">'+
 '<svg class="ph-hubring" viewBox="0 0 150 150" aria-hidden="true">'+
  '<circle class="ph-track" cx="75" cy="75" r="66" fill="none" stroke-width="5"/>'+
  '<g class="ph-dial" transform="rotate(-90 75 75)" fill="none" stroke-width="5">'+
 '</g></svg><i class="ph-sweep"></i></div>'+
 '<div class="ph-cap" id="phCap"><b><span class="ph-dot"></span>OpenCodeX-Desktop</b><span>内置网关 :10100 · 记录到 <em class="ph-capn">'+CH_BASE.length+'</em> 个渠道<em class="ph-st-cap"></em></span></div>'+CTL_HTML+
 '<div class="ph-node ph-adapter" id="phAdapter" data-k="krill" data-f="adapter" style="--c:var(--ph-krill)" title="Desktop 额外代理：把请求转换为 Krill 协议并实时转发"><span class="ph-orb"><i class="ph-pulse"></i><svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M2.5 5.5h10M10 3l2.5 2.5L10 8M13.5 10.5h-10M6 8l-2.5 2.5L6 13"/></svg></span><div class="ph-nl"><b>实时适配</b><span>Desktop 代理 · 协议转换</span></div></div>';
/* KPI 并入标题行：标签与数值同一行，细项放在 title 里 */
/* 顶部只汇总“本窗口内被记录到的量”。标签写“已记录”，避免被读成平台总量或占比基数。
   首字延迟 P50 由各渠道成功请求取中位数；分类失败与 P95 需要更细的观测，先不展示。 */
/* KPI 与所有派生文案都从当前 CH 生成：切窗口、切场景后调用 recompute() 一次刷新 */
function kpisHtml(){return'<div class="ph-kpis">'+
 '<div class="ph-kpi" title="本窗口内各渠道记录到的请求合计"><span class="ph-kl">已记录请求</span><b>'+fmt(T.req)+'</b></div>'+
 '<div class="ph-kpi" title="本窗口内各渠道记录到的输入 + 输出 Token 合计"><span class="ph-kl">Token</span><b>'+mm(T.tok)+'<small>M</small></b></div>'+
 '<div class="ph-kpi" title="'+fmt(T.req-T.fail)+' / '+fmt(T.req)+' 次成功，仅统计已记录的请求"><span class="ph-kl">成功率</span><b>'+p1(T.ok)+'<small>%</small></b></div>'+
 '<div class="ph-kpi" title="已记录请求中失败的次数；未按 429 / 超时 / 5xx 细分"><span class="ph-kl">失败</span><b>'+T.fail+'<small>次</small></b></div>'+
 '<div class="ph-kpi" title="各渠道成功请求的首字延迟中位数"><span class="ph-kl">首字 P50</span><b>0.82<small>s</small></b></div>'+
 '</div>'}
/* 重算汇总：T / CXT / 中枢与入口的悬停卡数据都跟着当前窗口走 */
function recompute(){
  T={req:sum(CH,'req'),tok:+sum(CH,'tok').toFixed(3),fail:sum(CH,'fail')};T.ok=okr(T);
  CXT={req:T.req-CLD.req,tok:+(T.tok-CLD.tok).toFixed(2),fail:T.fail-CLD.fail};CXT.ok=okr(CXT);
  var cps=Math.round((CMODELS.codex[0].req*CMODELS.codex[0].tps+CMODELS.codex[1].req*CMODELS.codex[1].tps)/(CMODELS.codex[0].req+CMODELS.codex[1].req));
  INFO.desktop.d=[['请求',fmt(T.req)+' 次'],['Token',mm(T.tok)+'M'],['成功率',p1(T.ok)+'%'],['输出速度',Math.round(sum(CH,'tps')/CH.length)+' t/s'],['TTFT P50','0.82s']];
  INFO.desktop.models=CMODELS.codex;
  INFO.codex.d=[['请求',fmt(CXT.req)+' 次'],['Token',mm(CXT.tok)+'M'],['成功率',p1(CXT.ok)+'%'],['记录到的渠道',CH.filter(function(c){return c.cx>0}).length+' 个'],['输出速度',cps+' t/s']];
  INFO.codex.models=CMODELS.codex;
  INFO.claude.d=[['请求',fmt(CLD.req)+' 次'],['Token',mm(CLD.tok)+'M'],['成功率',p1(okr({req:CLD.req,fail:CLD.fail}))+'%'],['失败',CLD.fail+' 次'],['输出速度','38 t/s'],['TTFT P50','1.12s']];
  INFO.claude.models=CMODELS.claude;
  if(hero){var old=hero.querySelector('#phKpis');if(old)old.innerHTML=kpisHtml();if(pop)delete pop.dataset.k;syncOverviewAvailability()}
  return T}
var NOTE='<div class="proto-annotation" id="ann-ov-hero" data-annotation-number="01" data-annotation-title="流量全景 · 内存模拟" data-annotation-category="页面导览 · 概览"><button type="button" class="annotation-marker" aria-expanded="false" aria-controls="annotationPopover" aria-label="原型注释 01：流量全景 · 内存模拟"><svg viewBox="0 0 16 16" aria-hidden="true"><path d="M3 2.5h7l3 3v8H3zM10 2.5v3h3M5.5 8h5M5.5 10.5h3"/></svg><span>原型</span><span class="annotation-number">01</span><span class="annotation-marker-plus" aria-hidden="true">＋</span></button><div class="annotation-source" hidden><p>概览主体展示 Desktop 内置 OpenCodex 网关分发日常请求、只为 Krill 做实时适配代理的目标形态。客户端、渠道与 Token 都是内存样例；统计窗口只切换副标题，光点是请求加速回放，网关未运行时停止回放。Gemini CLI 等直连客户端不在全景中展示。渠道列只列启用且窗口内有流量的渠道：Krill 置顶，其次是 Token 最多的渠道，成功率低于 98% 的渠道必显，其余按 Token 排；舞台高度放不下时把剩余渠道折成“+N 其他”节点，悬停或点击列出被收纳的渠道，中枢外环也把它们合并成一段。渠道列底部一行统计空闲与停用渠道，点击进入渠道设置。窗口拉高或全屏时自动多显示几个渠道。</p><p>读图：中枢外环按渠道 Token 占比分段，彗星扫光的转速跟随请求速率；网关停止时外环变灰、扫光停转，启动失败变红，启动中或确认中转琥珀并放慢。中枢 Logo 在环内随机漂移。节点外环用节点自身颜色，成功率低于 98% 转琥珀，红色刻痕表示失败占比；请求到达时节点泛起涟漪。线宽为 Token 占比，光点为请求回放，彗尾为经 Desktop 代理的实时转发，虚线为未细分的合并入口。背景光以中枢为中心铺满整页。</p><p>浮层中“查看请求”“渠道设置”“网关日志”跳到现有页面，其余动作暂未接入。运行状态与启停在顶部状态条，版本升级、文件同步、WebDAV 同步与最近事件在同页下方，整页不滚动。环境未就绪时整页仍切换为环境门禁。</p><div class="annotation-footer">页面导览 · 概览</div></div></div>';
/* 时间范围：1H / 1D / 1M / 1Y 四个快捷档 + 日历图标打开自定义区间 */
var CAL_ICO='<svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" aria-hidden="true"><rect x="2.2" y="3.4" width="11.6" height="10.4" rx="2"/><path d="M2.2 6.6h11.6M5.4 2.2v2.6M10.6 2.2v2.6"/></svg>';
var TOOLS='<div class="ph-seg" role="group" aria-label="统计窗口">'+
 [['1 小时','1H'],['1 天','1D'],['1 个月','1M'],['1 年','1Y']].map(function(w){return'<button type="button" data-ph-win="'+w[0]+'" aria-pressed="'+(w[1]==='1D')+'">'+w[1]+'</button>'}).join('')+
 '<button type="button" class="ph-ibtn ph-calbtn" data-ph-range aria-label="自定义时间范围" title="自定义时间范围" aria-haspopup="dialog">'+CAL_ICO+'</button></div>'+
 '<button type="button" class="ph-ibtn" data-ph-full aria-label="全屏" title="全屏"><svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5" aria-hidden="true"><path d="M2.5 6V2.5H6M10 2.5h3.5V6M13.5 10v3.5H10M6 13.5H2.5V10"/></svg></button>';

/* ---------- 自定义时间范围（概览内联一份，与组件库同构，尺寸适配概览） ---------- */
var rng=null,rngDraft=null,rngMonth=null,rngPick=null,rngHover=null,suppressRange=false;
function pad2(n){return String(n).padStart(2,'0')}
function rngField(s){return String(s||'').replace('T',' ').slice(0,16)}
function rngParse(s){var m=/^(\d{4})-(\d{2})-(\d{2})[ T](\d{2}):(\d{2})$/.exec(String(s||'').trim());if(!m)return null;
  var d=new Date(+m[1],+m[2]-1,+m[3],+m[4],+m[5]);return isNaN(d)?null:d}
function rngToLocal(d){return d.getFullYear()+'-'+pad2(d.getMonth()+1)+'-'+pad2(d.getDate())+'T'+pad2(d.getHours())+':'+pad2(d.getMinutes())}
function rngYmd(d){return d.getFullYear()+'-'+pad2(d.getMonth()+1)+'-'+pad2(d.getDate())}
function winFromRange(){var a=rngParse(customFrom),b=rngParse(customTo);return a&&b?Math.max(1,Math.round((b-a)/3600000))+' 小时':'1 天'}

var customFrom='',customTo='',rngTimePop=null;
function rngHours(){var o=[];for(var h=0;h<24;h++)o.push(pad2(h));return o}
function rngMinutes(){var o=[];for(var m=0;m<60;m++)o.push(pad2(m));return o}
function rngParts(v){var m=/^(\d{1,2}):(\d{1,2})$/.exec(String(v||'').trim());return m?[pad2(Math.min(23,+m[1])),pad2(Math.min(59,+m[2]))]:['00','00']}
function rngMonthHtml(base,side){
  var first=new Date(base.getFullYear(),base.getMonth(),1),pad=(first.getDay()+6)%7,days=new Date(base.getFullYear(),base.getMonth()+1,0).getDate();
  var a=rngDraft?rngDraft.from.slice(0,10):'',b=rngDraft?rngDraft.to.slice(0,10):'',cells=[];
  for(var i=0;i<pad;i++)cells.push('<span class="ph-cal-d is-empty"></span>');
  for(var d=1;d<=days;d++){var key=rngYmd(new Date(base.getFullYear(),base.getMonth(),d));
    var cls='ph-cal-d'+(key===a?' is-start':'')+(key===b?' is-end':'')+((b&&key>a&&key<b)?' is-in':'');
    cells.push('<button type="button" class="'+cls+'" data-rng-day="'+key+'">'+d+'</button>')}
  /* 补齐 6 行：两列月份不同时行数也不同，补齐后高度一致 */
  while(cells.length<42)cells.push('<span class="ph-cal-d is-empty"></span>');
  return'<div class="ph-cal-m"><div class="ph-cal-h">'+
    '<button type="button" class="ph-rng-x ph-cal-nav" data-rng-yprev="'+side+'" aria-label="上一年">«</button>'+
    '<button type="button" class="ph-rng-x ph-cal-nav" data-rng-mprev="'+side+'" aria-label="上个月">‹</button>'+
    '<b>'+base.getFullYear()+' 年 '+(base.getMonth()+1)+' 月</b>'+
    '<button type="button" class="ph-rng-x ph-cal-nav" data-rng-mnext="'+side+'" aria-label="下个月">›</button>'+
    '<button type="button" class="ph-rng-x ph-cal-nav" data-rng-ynext="'+side+'" aria-label="下一年">»</button></div>'+
    '<div class="ph-cal-w">'+['一','二','三','四','五','六','日'].map(function(w){return'<span>'+w+'</span>'}).join('')+'</div>'+
    '<div class="ph-cal-g">'+cells.join('')+'</div></div>'}
function rngDialogHtml(){
  var chips=[['今天','today'],['昨天','yesterday'],['最近 7 天','last7'],['最近 30 天','last30'],['本月','month']];
  var base=rngMonth||new Date(),next=new Date(base.getFullYear(),base.getMonth()+1,1);
  return'<div class="ph-rng-mask" data-rng-mask></div>'+
  '<section class="ph-rng" role="dialog" aria-modal="true" aria-label="自定义时间范围">'+
    '<header class="ph-rng-h"><div><h3>自定义时间范围</h3><p>结束时间不含在范围内，跨度上限 90 天。</p></div>'+
      window.PrototypeModal.closeButton('data-rng-close')+'</header>'+
    '<div class="ph-rng-f">'+
      '<div class="ph-rng-dt"><label>开始日期<input value="'+rngDraft.from.slice(0,10)+'" data-rng="fromDate" aria-label="开始日期"></label>'+
        '<label>开始时间<input value="'+rngDraft.from.slice(11,16)+'" data-rng="fromTime" aria-label="开始时间"></label></div>'+
      '<div class="ph-rng-dt"><label>结束日期<input value="'+rngDraft.to.slice(0,10)+'" data-rng="toDate" aria-label="结束日期"></label>'+
        '<label>结束时间<input value="'+rngDraft.to.slice(11,16)+'" data-rng="toTime" aria-label="结束时间"></label></div>'+
    '</div>'+
    '<div class="ph-rng-chips">'+chips.map(function(c){return'<button type="button" data-rng-chip="'+c[1]+'">'+c[0]+'</button>'}).join('')+'</div>'+
    '<div class="ph-cal"><div class="ph-cal-bar"><span>'+(rngPick?'已选 '+rngPick+'，再点一天作为结束':'点选开始日期，再点结束日期；每月的 « ‹ › » 各自翻页')+'</span></div>'+
      '<div class="ph-cal-pair">'+rngMonthHtml(base,'左')+rngMonthHtml(next,'右')+'</div></div>'+
    '<footer class="ph-rng-fo"><span class="ph-rng-note"><span data-rng-sum></span><span class="ph-rng-err" data-rng-err role="alert" hidden></span></span>'+
      '<span class="ph-rng-bt"><button type="button" class="btn ghost" data-rng-cancel>取消</button><button type="button" class="btn primary" data-rng-apply>应用</button></span></footer>'+
  '</section>'}

function redrawRangeCalendar(){if(!rng)return;var cal=rng.querySelector('.ph-cal');if(!cal)return;
  var base=rngMonth||new Date(),next=new Date(base.getFullYear(),base.getMonth()+1,1);
  cal.innerHTML='<div class="ph-cal-bar"><span>'+(rngPick?'已选 '+rngPick+'，再点一天作为结束':'点选开始日期，再点结束日期；每月的 « ‹ › » 各自翻页')+'</span></div>'+
    '<div class="ph-cal-pair">'+rngMonthHtml(base,'左')+rngMonthHtml(next,'右')+'</div>'}
function redrawRangeDates(){if(!rng)return;var f=rng.querySelector('[data-rng="fromDate"]'),t=rng.querySelector('[data-rng="toDate"]');
  if(f)f.value=rngDraft.from.slice(0,10);if(t)t.value=rngDraft.to.slice(0,10);redrawRangeCalendar()}

function bindRangeInput(){if(!rng||rng.dataset.bound)return;rng.dataset.bound="1";
  rng.addEventListener("input",function(e){
    var k=e.target.dataset&&e.target.dataset.rng;if(!k)return;var v=e.target.value.trim();
    if(k==="fromDate"||k==="toDate"){
      if(k==="fromDate")rngDraft.from=v+"T"+rngDraft.from.slice(11,16);else rngDraft.to=v+"T"+rngDraft.to.slice(11,16);
      rngMonth=new Date(rngParse(rngDraft.from)||new Date());redrawRangeCalendar();
    } else if(k==="fromTime")rngDraft.from=rngDraft.from.slice(0,10)+"T"+v;
    else if(k==="toTime")rngDraft.to=rngDraft.to.slice(0,10)+"T"+v;
    rngRender();})}
function rngRender(){if(!rng)return;var sum=rng.querySelector('[data-rng-sum]'),err=rng.querySelector('[data-rng-err]');
  var a=rngParse(rngDraft.from),b=rngParse(rngDraft.to),msg='';
  if(!a||!b)msg='请填写完整的开始与结束时间';
  else if(b<=a)msg='结束时间需要晚于开始时间';
  else if(b-a>90*864e5)msg='跨度超过 90 天，请缩小范围';
  if(err){err.hidden=!msg;err.textContent=msg||''}
  if(sum)sum.textContent=msg?'':rngField(rngDraft.from)+' → '+rngField(rngDraft.to)+' · '+Math.max(1,Math.round((b-a)/36e5))+' 小时'}
function openRangeDialog(){if(rng)return;
  rngDraft={from:customFrom||'2026-11-01T00:00',to:customTo||'2026-11-02T00:00'};
  rngMonth=new Date(rngParse(rngDraft.from)||new Date());rngPick=null;rngHover=null;
  rng=document.createElement('div');rng.className='ph-rng-host';rng.innerHTML=rngDialogHtml();
  hero.appendChild(rng);rngRender();fitRangeDialog();document.querySelectorAll('[data-ph-range]').forEach(function(b){b.setAttribute('aria-expanded','true')});
  bindRangeInput();
  var first=rng.querySelector('[data-rng="fromDate"]');if(first)first.focus({preventScroll:true})}
/* 弹窗挂在 hero 上，不受舞台容器查询约束：按舞台实际高度收紧尺寸并决定单/双月 */
function fitRangeDialog(){if(!rng||!stage)return;
  var sh=stage.clientHeight,sw=stage.clientWidth;if(!sh||!sw)return;
  var hr=hero.getBoundingClientRect(),sr=stage.getBoundingClientRect();
  rng.style.padding='6px';rng.style.top=Math.round(sr.top-hr.top)+'px';rng.style.height=Math.round(sr.height)+'px';rng.style.bottom='auto';
  var d=rng.querySelector('.ph-rng');if(!d)return;
  /* 高度不足 360 时只显示一个月，并去掉说明句，保证不出现滚动条 */
  var tight=sh<360,narrow=sw<640||tight;
  rng.classList.toggle('is-narrow',narrow);
  var months=rng.querySelectorAll('.ph-cal-m');if(months[1])months[1].style.display=narrow?'none':'block';
  var pair=rng.querySelector('.ph-cal-pair');if(pair)pair.style.gridTemplateColumns=narrow?'1fr':'repeat(2,minmax(0,1fr))';
  var f=rng.querySelector('.ph-rng-f');if(f)f.style.gridTemplateColumns='repeat(2,minmax(0,1fr))';
  var p=rng.querySelector('.ph-rng-h p');if(p)p.style.display=tight?'none':'block';
  /* 兜底：仍超出就把字号与日历行高再压一档 */
  /* 先按紧凑档量一次；仍溢出再压一档（隐藏说明与快捷区间） */
  rng.classList.toggle('is-tight',false);
  if(d.scrollHeight>d.clientHeight+1)rng.classList.add('is-tight');
  if(d.scrollHeight>d.clientHeight+1)rng.classList.add('is-ultra');
  /* 极矮舞台（约 900×600）：只保留表单与单月日历，日历按自然高度，不压缩不重叠 */
  if(sh<320){rng.classList.add('is-mini');
    var chipsEl=rng.querySelector('.ph-rng-chips');if(chipsEl)chipsEl.style.display='none';
    var hdr=rng.querySelector('.ph-rng-h');if(hdr)hdr.style.display='none';
    var cp=rng.querySelector('.ph-cal-pair');if(cp)cp.style.maxHeight='none';
    d.style.fontSize='10.5px'}else{rng.classList.remove('is-mini')}
}
function closeRangeDialog(){if(!rng)return;closeRngTime();rng.remove();rng=null;rngDraft=null;rngPick=null;rngHover=null;document.querySelectorAll('[data-ph-range]').forEach(function(b){b.setAttribute('aria-expanded','false')})}
function rngApply(){var a=rngParse(rngDraft.from),b=rngParse(rngDraft.to);
  var err=rng.querySelector('[data-rng-err]');
  if(!a||!b||b<=a||b-a>90*864e5){if(err){err.hidden=false;err.textContent=!a||!b?'请填写完整的开始与结束时间':b<=a?'结束时间需要晚于开始时间':'跨度超过 90 天，请缩小范围'}return}
  customFrom=rngDraft.from;customTo=rngDraft.to;
  win=Math.max(1,Math.round((b-a)/36e5))+' 小时';
  closeRangeDialog();toolsEl.querySelectorAll('[data-ph-win]').forEach(function(b2){b2.setAttribute('aria-pressed','false')});rebuildRange()}
/* 自定义区间：按实际小时数缩放样例，并把按钮状态交给日历图标 */
function rebuildRange(){
  /* 自定义区间同样走 buildWindow，保证请求、Token、来源拆分、客户端与模型一致 */
  buildWindow();renderClients();renderCh(K);paintSpeed();syncFlowUi();if(built)build();else if(journey&&journey.active())journey.refresh();if(focus)apply();writeSubtitle()}

function closeRngTime(){if(!rngTimePop)return;rngTimePop.remove();rngTimePop=null;suppressRange=false}
function openRngTime(which){var input=rng.querySelector('[data-rng="'+which+'"]');if(!input)return;
  if(rngTimePop&&rngTimePop.dataset.owner===which)return;closeRngTime();
  var parts=rngParts(input.value),pop=document.createElement('div');pop.className='ph-rng-tpop';pop.dataset.owner=which;
  pop.innerHTML='<div class="ph-tp-col">'+rngHours().map(function(h){return'<button type="button" data-rng-h="'+h+'"'+(h===parts[0]?' aria-selected="true"':'')+'>'+h+'</button>'}).join('')+'</div>'+
    '<div class="ph-tp-col">'+rngMinutes().map(function(m){return'<button type="button" data-rng-m="'+m+'"'+(m===parts[1]?' aria-selected="true"':'')+'>'+m+'</button>'}).join('')+'</div>'+
    '<div class="ph-tp-now-bar"><button type="button" class="ph-tp-now" data-rng-now>此刻</button></div>';
  var r=input.getBoundingClientRect();pop.style.left=Math.round(r.left)+'px';
  /* 先按实际渲染高度决定向上还是向下，避免底部被裁 */
  document.body.appendChild(pop);rngTimePop=pop;input.setAttribute('aria-expanded','true');
  var ph0=pop.offsetHeight,below=innerHeight-r.bottom-8,top=below>=ph0?r.bottom+4:Math.max(8,r.top-ph0-4);
  pop.style.top=Math.round(top)+'px';}
function setRngPart(which,kind,v){var input=rng.querySelector('[data-rng="'+which+'"]');if(!input)return;
  var parts=rngParts(input.value);parts[kind==='h'?0:1]=v;input.value=parts[0]+':'+parts[1];
  if(which==='fromTime')rngDraft.from=rngDraft.from.slice(0,10)+'T'+input.value;
  else if(which==='toTime')rngDraft.to=rngDraft.to.slice(0,10)+'T'+input.value;
  rngDraft[which]=(which==='fromTime'?rngDraft.from:rngDraft.to);rngRender();
  if(rngTimePop){rngTimePop.querySelectorAll('[data-rng-h]').forEach(function(b){b.setAttribute('aria-selected',String(b.dataset.rngH===parts[0]))});
    rngTimePop.querySelectorAll('[data-rng-m]').forEach(function(b){b.setAttribute('aria-selected',String(b.dataset.rngM===parts[1]))})}}

/* ---------- 状态 ---------- */
var hero,stage,svg,frame,glow,pop,pg,toolsEl,noteEl,hub,journey=null;
var win='1 天',paused=false,focus=null,locked=null,anchor=null,suppress=false,state='running';
var hoverCloseTimer=0,pointerInPop=false,selectingInPop=false;
var built=false,dirty=true,needWarm=true,warming=false,lastTheme=root.dataset.theme||'';
var P={},slot=null,mid=0,cy=0,W=0,H=0,geo=null;
var parts=[],lens={},acc=0,last=0,raf=0,sraf=0,timer=0,live=2;
var frameKey='',ready=false,lastMsg='';
var canAnimate=typeof SVGPathElement!=='undefined'&&!!(SVGPathElement.prototype&&SVGPathElement.prototype.getTotalLength);

function setup(){return host.dataset.mode==='setup'}
function visible(){return root.dataset.route==='overview'&&!document.hidden&&!setup()}
function flowing(){return state==='running'}
function syncOverviewAvailability(){if(!stage)return;
  var overall=!flowing();stage.classList.toggle('ph-overview-only',overall);
  if(overall&&journey&&journey.active())journey.exit();
  QA('.ph-clients,.ph-channels,#phAdapter').forEach(function(e){e.inert=overall;e.setAttribute('aria-hidden',String(overall))});
  if(!journey||!journey.active())QA('[data-f]').forEach(function(e){e.tabIndex=overall&&e!==hub?-1:0});
  var caption=Q('#phCap>span'),html=overall?'内置网关 :10100 · '+ST[state]:
    '内置网关 :10100 · 记录到 <em class="ph-capn">'+CH_BASE.length+'</em> 个渠道<em class="ph-st-cap"></em>';
  if(caption&&caption.innerHTML!==html)caption.innerHTML=html;
  if(overall&&focus&&focus!=='desktop'){locked=null;setFocus(null)}
}
function effectsLow(){return root.dataset.effects==='low'}
function subtitle(){var label=win,custom=false;
  if(customFrom&&customTo){var a=rngParse(customFrom),b=rngParse(customTo);
    if(a&&b){label=rngField(customFrom).slice(0,10)+' 至 '+rngField(customTo).slice(0,10);custom=true}}
  return(custom?'范围 '+label:'过去 '+label)+' · Desktop 内置网关记录到 '+CH_BASE.length+' 个渠道的流量'}
function Q(s){return stage.querySelector(s)}
function QA(s){return Array.prototype.slice.call(stage.querySelectorAll(s))}

/* ---------- 几何：以舞台内边距盒为原点，与 SVG（inset:0）和绝对定位元素一致 ---------- */
function box(e){var s=stage.getBoundingClientRect(),r=e.getBoundingClientRect(),ox=s.left+stage.clientLeft,oy=s.top+stage.clientTop;return{l:r.left-ox,t:r.top-oy,r:r.right-ox,b:r.bottom-oy,x:r.left-ox+r.width/2,y:r.top-oy+r.height/2,w:r.width,h:r.height}}
function mk(tag,at,par){var e=document.createElementNS(NS,tag);for(var k in at)if(at[k]!==undefined&&at[k]!==null)e.setAttribute(k,at[k]);(par||svg).appendChild(e);return e}
function f(n){return n.toFixed(1)}
/* 三次贝塞尔：a 点沿 ua 方向出发，b 点沿 ub 方向进入（ub 指向 b 的来向） */
function cc(a,ua,b,ub,k){var L=Math.hypot(b.x-a.x,b.y-a.y)*(k||.42);return'M'+f(a.x)+','+f(a.y)+' C'+f(a.x+ua[0]*L)+','+f(a.y+ua[1]*L)+' '+f(b.x+ub[0]*L)+','+f(b.y+ub[1]*L)+' '+f(b.x)+','+f(b.y)}
function line(a,b){return'M'+f(a.x)+','+f(a.y)+' L'+f(b.x)+','+f(b.y)}
function html(cls,x,y,inner,k){var d=document.createElement('div');d.className=cls+' ph-gen';if(inner)d.innerHTML=inner;d.style.left=Math.round(x)+'px';d.style.top=Math.round(y)+'px';if(k)d.dataset.k=k;stage.appendChild(d);return d}
function v(c){return'var(--ph-'+c+')'}

/* ---------- 多渠道收纳：舞台高度决定渠道槽位 K（4–8）；超出时最后一槽变成“+N 其他” ----------
   顺序：Krill（经实时适配）置顶 → Token 最多的渠道 → 成功率低于 98% 的渠道 → 其余按 Token。 */
var K=5,NODES=[],FOLD=[],NODE={};
function layout(k){K=Math.max(4,Math.min(8,k));var byTok=function(a,b){return b.tok-a.tok},all=CH.slice(),vis=all;FOLD=[];
  if(all.length>K){var rank=all.slice().sort(byTok),pri=[];
    function add(c){if(pri.indexOf(c)<0)pri.push(c)}
    all.filter(function(c){return c.via}).forEach(add);add(rank.filter(function(c){return!c.via})[0]);
    rank.filter(function(c){return okr(c)<98}).forEach(add);rank.forEach(add);
    vis=pri.slice(0,K-1);
    /* 连续场景中的渠道即使窗口缩小也保留原对象，其他渠道正常收纳。 */
    var selected=journey&&journey.active()?all.filter(function(c){return c.k===journey.state().channel})[0]:null;
    if(selected&&vis.indexOf(selected)<0)vis[vis.length-1]=selected;
    FOLD=rank.filter(function(c){return vis.indexOf(c)<0})}
  NODES=vis.slice().sort(function(a,b){return(b.via?1:0)-(a.via?1:0)||b.tok-a.tok});
  if(FOLD.length){var m={k:'more',n:'其他 '+FOLD.length+' 个',lb:'+'+FOLD.length,more:1,req:sum(FOLD,'req'),fail:sum(FOLD,'fail'),tok:+sum(FOLD,'tok').toFixed(2),cx:sum(FOLD,'cx'),cl:sum(FOLD,'cl'),tps:Math.round(sum(FOLD,'tps')/FOLD.length)};
    m.low=Math.min.apply(null,FOLD.map(okr));NODES.push(m)}
  var at=0;NODES.forEach(function(c,i){c.sh=i===NODES.length-1?100-at:Math.round(c.tok/T.tok*100);at+=c.sh});
  NODE={};NODES.forEach(function(c){NODE[c.k]=c})}
function chInfo(c){var ok=okr(c);
  /* 标签只说这条渠道被观测到的路径与状态，不推断“主力 / 排名 / 常用” */
  var tag=c.via?'经实时适配':c.cl>0?'codex + Claude Code':ok<98?'成功率偏低':'本窗口有流量';
  var p=c.via?'经 Desktop 实时适配代理转发；来源客户端以 codex 入口计'
    :c.cl>0?fmt(c.cx)+' 次来自 codex 入口，'+fmt(c.cl)+' 次来自 Claude Code'
    :'来源客户端以 codex 入口计；本窗口记录了 '+fmt(c.req)+' 次请求';
  var d=[['请求',fmt(c.req)+' 次'],['Token',mm(c.tok)+'M'],['成功率',p1(ok)+'%']];if(ok<98)d.push(['失败',c.fail+' 次']);
  d.push(['输出速度',c.tps+' t/s'],['TTFT P50',c.ttft.toFixed(2)+'s'],['延迟',c.lat+'ms']);
  return{n:c.n,tag:tag,c:c.k,p:p,d:d,a:ACT_CH,seed:CH.indexOf(c)*2+3,models:c.models}}
function moreInfo(m){return{n:'+'+FOLD.length+' 其他渠道',tag:'已收纳',c:'more',p:fmt(m.req)+' 次 · '+mm(m.tok)+'M · 成功率 '+p1(okr(m))+'%',list:FOLD,a:['全部渠道','查看请求']}}
/* 渠道节点只写两件能直接读到的事：名称、本窗口的 Token 绝对值。
   不再用线宽、圆环或百分比表达占比——分母取决于“是否观测完整”，原型无法保证。
   成功率低于 98% 时才多一行琥珀色百分比，让异常仍然一眼可见。 */
/* 渠道节点两行：名称 / 本窗口 Token。速度不进节点（避免又回到“数字墙”），放在悬停卡的模型明细里。
   成功率低于 98% 时多一行琥珀色百分比，异常仍然一眼可见。 */
/* 渠道节点第二行写本窗口的 Token 与输出速度；成功率只在低于 98% 时以琥珀色补在速度后面 */
function chNode(c){var ok=okr(c),low=ok<98;
  var cls='ph-node ph-chan'+(c.more?' ph-more':'');
  var head='<div class="'+cls+'" id="phCh-'+c.k+'" data-k="'+c.k+'" data-f="'+c.k+'" style="--c:'+v(c.k)+'">';
  var body='<div class="ph-nl"><div class="ph-l1"><b>'+c.n+'</b></div>'+
    '<div class="ph-l2"><span class="ph-tok">'+mm(c.tok)+'M</span>'+
    '<span class="ph-spd'+(low?' ph-low':'')+'">'+(c.tps||0)+' t/s</span></div></div>';
  return head+orb(c.lb,{ok:c.more?c.low:ok})+body+'</div>'}
/* 渠道列底部不再放“另有 N 个空闲 / 停用”的长句：
   空闲与停用渠道不承载流量，放进节点列只会把注意力从“在跑的流量”上带走；
   需要查看时走“+N 其他”节点或渠道设置。这里只保留一个不带文字的统计点，供无障碍读取。 */
function chFoot(){if(!IDLE.length&&!OFF.length)return'';
  var ti=[];if(IDLE.length)ti.push(IDLE.join('、')+' 空闲');if(OFF.length)ti.push(OFF.join('、')+' 停用');
  return'<span class="ph-chstat" role="note" aria-label="另有空闲与停用渠道" title="'+ti.join('；')+'"></span>'}
function renderCh(k){layout(k);
  SET={desktop:['desktop','codex','claude'],codex:['codex'],claude:['claude'],gemini:['gemini']};
  NODES.forEach(function(c){SET.desktop.push(c.k);if(c.cx)SET.codex.push(c.k);if(c.cl)SET.claude.push(c.k);
    SET[c.k]=[c.k].concat(c.cx?['codex']:[],c.cl?['claude']:[]);INFO[c.k]=c.more?moreInfo(c):chInfo(c)});
  SET.adapter=SET.krill;
  ROUTES=[];NODES.forEach(function(c){
    if(c.via){ROUTES.push({s:['trunk','coreAd','adKr'],live:{coreAd:1,adKr:1},c:c.k,k:['codex',c.k],w:c.cx,end:c.k});return}
    if(c.cx)ROUTES.push({s:['trunk','core_'+c.k],c:c.k,k:['codex',c.k],w:c.cx,end:c.k});
    if(c.cl)ROUTES.push({s:['claude','core_'+c.k],c:c.k,k:['claude',c.k],w:c.cl,end:c.k})});
  TOT=sum(ROUTES,'w');
  /* Token 已写在每个节点第二行，列头只保留“渠道”一项 */
  hero.querySelector('.ph-channels').innerHTML='<div class="ph-colh"><span>渠道</span><span></span></div><div class="ph-grp">'+NODES.map(chNode).join('')+chFoot()+'</div>';
  /* 中枢外环只表达健康状态：正常为完整细环，异常/停止整环变色。
     不再按 Token 拆分成不同长度的分段——那需要“总量”分母，原型拿不到可靠值。 */
  var dl=hero.querySelector('.ph-dial');while(dl.firstChild)dl.removeChild(dl.firstChild);
  var ring=document.createElementNS(NS,'circle');
  ring.setAttribute('class','ph-arc');ring.setAttribute('cx',75);ring.setAttribute('cy',75);ring.setAttribute('r',66);
  ring.style.setProperty('--ac',v('core'));
  dl.appendChild(ring);
  a11y(QA('.ph-channels [data-f]'));
  delete pop.dataset.k;clearParts();
  /* 重排后原锚点节点被替换：同一渠道仍可见就接回，被收纳则关闭详情 */
  if(anchor&&!anchor.isConnected){var na=focus?Q('.ph-channels [data-f="'+focus+'"]'):null;
    if(na){if(locked)locked=na;anchor=na}else{locked=null;setFocus(null)}}}
/* 按渠道列可用高度估算能放几个节点；jsdom 等无布局环境（节点高 0）保持默认 K */
function fitK(){var c=Q('.ph-channels'),n=c&&c.querySelector('.ph-node'),g=c&&c.querySelector('.ph-grp'),h=c&&c.querySelector('.ph-colh'),nh=n?n.offsetHeight:0;if(!nh)return false;
  var gap=parseFloat(getComputedStyle(g).rowGap)||0,avail=c.clientHeight-(h?h.offsetHeight:0)-44;
  var k=Math.max(4,Math.min(8,Math.floor((avail+gap)/(nh+gap)))),eff=function(x){return Math.min(x,CH.length)};
  if(eff(k)===eff(K))return false;renderCh(k);return true}

function build(){
  if(!stage)return;
  if(journey&&journey.active())journey.beforeLayout();
  fitK();
  QA('.ph-gen').forEach(function(e){e.remove()});
  while(svg.firstChild)svg.removeChild(svg.firstChild);
  var dark=root.dataset.theme==='dark';
  W=stage.clientWidth;H=stage.clientHeight;
  svg.setAttribute('viewBox','0 0 '+W+' '+H);
  /* 舞台越矮圆环越小，给上方详情卡腾出高度：≤300 用 48，≤340 用 56，其余 72 */
  R=H<300?48:H<340?56:72;stage.style.setProperty('--ph-hub',(2*R+6)+'px');
  var cl=box(Q('.ph-clients')),ch=box(Q('.ph-channels'));
  mid=Math.round((cl.r+ch.l)/2);stage.style.setProperty('--ph-mid',mid+'px');
  var cn=QA('.ph-clients .ph-node'),pc=cn.map(function(n){var o=box(n.querySelector('.ph-orb'));return{x:o.r+6,y:o.y}});
  var kn=QA('.ph-channels .ph-node'),ko=kn.map(function(n){return box(n.querySelector('.ph-orb'))}),pk=ko.map(function(o){return{x:o.l-6,y:o.y}});
  /* 渠道列底部要给列底统计行留位置，否则“另有空闲 / 停用渠道”会被舞台裁掉 */
  stage.style.setProperty('--ph-colb',(Q('#phHubCtl')?'44px':'12px'));
  var cap=Q('#phCap'),ctl=Q('#phHubCtl'),capH=cap.offsetHeight||36,ctlH=ctl?ctl.offsetHeight:0;
  /* 中枢与客户端节点组垂直居中对齐；中枢下方要放下标题和控制区，整体不出舞台 */
  var below=8+capH+6+ctlH;
  cy=flowing()?(box(cn[0]).t+box(cn[cn.length-1]).b)/2:H/2;cy=Math.max(R+10,Math.min(cy,H-8-below-R));
  cy=Math.round(cy);stage.style.setProperty('--ph-cy',cy+'px');cap.style.top=Math.round(cy+R+8)+'px';
  if(ctl)ctl.style.top=Math.round(cy+R+8+capH+6)+'px';
  function cpS(side,s){return{x:mid+side*R*Math.sqrt(1-s*s),y:cy+R*s,u:[side*Math.sqrt(1-s*s),s]}}
  function cp(side,ty){return cpS(side,Math.max(-.6,Math.min(.6,(ty-cy)/260)))}
  var defs=mk('defs',{}),gid=0;
  function grad(a,b,c1,c2){var id='phg'+(gid++),g=mk('linearGradient',{id:id,gradientUnits:'userSpaceOnUse',x1:f(a.x),y1:f(a.y),x2:f(b.x),y2:f(b.y)},defs);mk('stop',{offset:0,style:'stop-color:'+v(c1)},g);mk('stop',{offset:1,style:'stop-color:'+v(c2)},g);return'url(#'+id+')'}
  if(dark){var fl=mk('filter',{id:'phglow',filterUnits:'userSpaceOnUse',x:-60,y:-60,width:W+120,height:H+120},defs);mk('feGaussianBlur',{stdDeviation:4.5},fl)}
  var gBase=mk('g',{}),gGlow=mk('g',{filter:dark?'url(#phglow)':null,opacity:dark?.9:0}),gMain=mk('g',{}),gTop=mk('g',{});
  P={};
  function wire(name,d,o){
    var stroke=o.c2?grad(o.a,o.b,o.c1,o.c2):v(o.c1);
    if(o.base)mk('path',{d:d,fill:'none',stroke:'var(--ph-line)','stroke-width':o.w+5,'stroke-linecap':'round',class:'ph-dimmable','data-k':o.k},gBase);
    if(dark&&o.glow!==false)mk('path',{d:d,fill:'none',stroke:stroke,'stroke-width':o.w+2,'stroke-linecap':'round',class:'ph-dimmable','data-k':o.k},gGlow);
    P[name]=mk('path',{d:d,fill:'none',stroke:stroke,'stroke-width':o.w,'stroke-linecap':'round','stroke-dasharray':o.dash,class:o.k?'ph-dimmable':null,'data-k':o.k,opacity:o.op||(dark?.88:.62)},gMain);
    return P[name];
  }
  function sock(p,c,k){mk('circle',{cx:f(p.x),cy:f(p.y),r:3.2,fill:'var(--ph-bg)',stroke:v(c),'stroke-width':1.6,class:'ph-dimmable','data-k':k},gTop)}
  /* 客户端 → 汇合点 → Desktop */
  var cL=mid-R,mg={x:cl.r+(cL-cl.r)*.5,y:(pc[0].y*3+pc[1].y+pc[2].y)/5};
  wire('app',cc(pc[0],[1,0],mg,[-1,0],.45),{c1:'codex',w:3,k:'codex'});
  wire('oc',cc(pc[1],[1,0],mg,[-1,0],.45),{c1:'opencode',w:1.6,dash:'3 4',k:'codex',op:dark?.7:.5});
  wire('cli',cc(pc[2],[1,0],mg,[-1,0],.45),{c1:'cli',w:1.6,dash:'3 4',k:'codex',op:dark?.7:.5});
  var tp=cp(-1,mg.y);
  /* 进入中枢时沿外环法线从外侧切入，避免曲线先扎进环内再折回 */
  wire('trunk',cc(mg,[1,0],tp,tp.u,.45),{c1:'codex',c2:'core',a:mg,b:tp,w:8.2,k:'codex',base:1});
  html('ph-merge ph-dimmable',mg.x,mg.y,'','codex').title='codex 入口：App / OpenCode / CLI 合并计量';
  /* 合并用量标签：放在汇合点右上，右边界按中枢外环在标签高度处的弧线计算，宽度不够时只留数值 */
  var mw=html('ph-wl ph-dimmable',0,mg.y-21,'codex 入口 · <b>'+mm(CXT.tok)+'M</b>','codex');mw.title='codex 入口 · '+fmt(CXT.req)+' 次 · 成功率 '+p1(CXT.ok)+'%';
  var dy=Math.max(0,Math.abs(mg.y-21-cy)-9),lim2=mid-Math.sqrt(Math.max(0,(R+4)*(R+4)-dy*dy))-6;
  var ml=mg.x-12;if(ml+mw.offsetWidth>lim2){mw.innerHTML='<b>'+mm(CXT.tok)+'M</b>';ml=Math.min(ml,lim2-mw.offsetWidth)}
  mw.style.left=Math.round(ml+mw.offsetWidth/2)+'px';
  var cpC=cp(-1,pc[3].y);
  wire('claude',cc(pc[3],[1,0],cpC,cpC.u,.45),{c1:'claude',c2:'core',a:pc[3],b:cpC,w:3.8,k:'claude',base:1});
  sock(tp,'codex','codex');sock(cpC,'claude','claude');
  /* Desktop → 渠道；Krill 线路多经过一个 Desktop 实时适配节点 */
  var ad=Q('#phAdapter');ad.style.left=ad.style.top='';
  var ab=box(ad),ao=box(ad.querySelector('.ph-orb')),kO=ko[0];
  var ay=Math.round(kO.y),ax=Math.round(kO.l-Math.max(80,(kO.l-(mid+R))*.42)),ar=ao.w/2+4;
  ad.style.left=Math.round(ab.l+ax-ao.x)+'px';ad.style.top=Math.round(ab.t+ay-ao.y)+'px';
  var adL={x:ax-ar,y:ay},adR={x:ax+ar,y:ay},kp=cp(1,ay+40);
  wire('coreAd',cc(kp,kp.u,adL,[-1,0],.5),{c1:'core',c2:'krill',a:kp,b:adL,w:3.5,k:'krill',base:1});
  wire('adKr',line(adR,pk[0]),{c1:'krill',w:3.5,k:'krill',base:1});
  sock(kp,'krill','krill');
  /* 其余渠道沿外环右侧依次接出：接线点跟随节点高度，相邻至少错开 0.16。线宽统一——线代表“有流量经过”，不代表占比 */
  var lastS=(kp.y-cy)/R;
  for(var i=1;i<kn.length;i++){var ck=kn[i].dataset.k,cn_=NODE[ck],s=Math.min(.62,Math.max(lastS+.16,(pk[i].y-cy)/260));lastS=s;var p=cpS(1,s);
    wire('core_'+ck,cc(p,p.u,pk[i],[-1,0],.45),{c1:'core',c2:ck,a:p,b:pk[i],w:3,k:ck,base:1});sock(p,ck,ck)}
  /* 停靠槽：中枢标题与控制区下方够高时详情横向停在这里，否则就近浮出 */
  var capB=box(ctl||cap).b,sTop=Math.round(capB+12),sW=Math.round(Math.min(440,ch.l-cl.r-24)),sH=Math.round(H-14-sTop);
  slot=sW<300?null:sH>=100?{x:mid,top:sTop,w:sW,h:sH}:null;
  svg.appendChild(pg);
  built=true;dirty=false;
  if(focus)apply();
  var o=lens;measure();
  parts.forEach(function(p){var n=p.r.s[p.i];if(n&&o[n])p.d*=(lens[n]||0)/o[n]});
  draw();placeGlow();
  if(needWarm&&!effectsLow()&&flowing())warm();
  syncFrame();syncTimer();kick();
  if(journey&&journey.active())journey.refresh();
}

/* ---------- 光点：按请求量抽样，Krill 线路在适配节点前后带彗尾 ---------- */
function measure(){lens={};if(!canAnimate)return;for(var n in P)lens[n]=P[n].getTotalLength()}
function pick(){var x=Math.random()*TOT;for(var i=0;i<ROUTES.length;i++){x-=ROUTES[i].w;if(x<=0)return ROUTES[i]}return ROUTES[0]}
function el(tag,at){var e=document.createElementNS(NS,tag);for(var k in at)e.setAttribute(k,at[k]);return e}
function spawn(){var r=pick(),g=el('g',{}),tr=[];
  if(r.live)for(var i=0;i<4;i++){var t=el('circle',{r:2.6-i*.45,fill:'var(--ph-krill)',opacity:0});g.appendChild(t);tr.push(t)}
  var h=el('circle',{r:2.9,'stroke-width':1.7});g.appendChild(h);pg.appendChild(g);
  /* 每条线路的光点速度取对应渠道的输出速度档位；codex 入口没有单一渠道，用全站速度 */
  var nd=NODE[r.end],gear=nd?speedGear(nd.tps):globalGear();
  parts.push({r:r,i:0,d:0,g:g,h:h,tr:tr,v:SPD*gear*(.9+Math.random()*.2)})}
function hit(k){if(effectsLow())return;var c=k==='adapter'?Q('#phAdapter'):Q('.ph-channels [data-f="'+k+'"]'),o=c&&c.querySelector('.ph-orb');if(!o)return;
  if(o.querySelectorAll('.ph-ripple').length<3){var w=document.createElement('i');w.className='ph-ripple';o.appendChild(w);
    /* 涟漪时长按该渠道的输出速度：越快 → 扩散越短，看起来更急 */
    var nd=NODE[k],dur=Math.max(.34,Math.min(1.15,.7*(nd?speedGear(nd.tps):1)));
    w.style.animationDuration=dur.toFixed(2)+'s';
    var done=function(){w.remove()};w.addEventListener('animationend',done);setTimeout(done,Math.round(dur*1000)+60)}
  var a=k!=='adapter'&&hero.querySelector('.ph-arc[data-c="'+k+'"]');
  if(a){a.classList.add('ph-flash');clearTimeout(a._t);a._t=setTimeout(function(){a.classList.remove('ph-flash')},260)}}
/* 生成频率也跟速度：画面上的“热闹程度”对应真实请求速率，而不是固定节拍 */
/* 生成节奏 = 基础 RATE × 请求速率档位 × 侧栏倍率；速率档位同样用对数尺度，避免线性比值被压平 */
function step(dt){var rg=Math.max(.6,Math.min(1.9,Math.pow(Math.max(.001,ratePerSec())/.03,.36)));
  acc+=dt*RATE*rg*FSCALE.rate;while(acc>=1){acc-=1;spawn()}
  for(var j=parts.length-1;j>=0;j--){var p=parts[j];p.d+=p.v*dt;
    while(p.i<p.r.s.length&&p.d>=(lens[p.r.s[p.i]]||0)){p.d-=lens[p.r.s[p.i]]||0;if(p.r.s[p.i]==='coreAd'&&!warming)hit('adapter');p.i++}
    if(p.i>=p.r.s.length){p.g.remove();parts.splice(j,1);if(!warming)hit(p.r.end)}}}
function draw(){if(!canAnimate)return;var set=focus?SET[focus]:null;
  parts.forEach(function(p){var n=p.r.s[p.i],path=P[n];if(!path)return;var L=lens[n]||1,pt=path.getPointAtLength(Math.min(p.d,L));
    var lv=p.r.live&&p.r.live[n];
    p.h.setAttribute('cx',pt.x.toFixed(1));p.h.setAttribute('cy',pt.y.toFixed(1));
    if(lv){p.h.setAttribute('fill','var(--ph-krill)');p.h.setAttribute('stroke','var(--ph-bg)');p.h.setAttribute('r',3.3)}
    else{p.h.setAttribute('fill','var(--ph-bg)');p.h.setAttribute('stroke',v(p.r.c));p.h.setAttribute('r',2.9)}
    p.tr.forEach(function(t,i){if(!lv){t.setAttribute('opacity',0);return}var u=path.getPointAtLength(Math.max(0,Math.min(p.d,L)-(i+1)*6.5));
      t.setAttribute('cx',u.x.toFixed(1));t.setAttribute('cy',u.y.toFixed(1));t.setAttribute('opacity',(.5-i*.11).toFixed(2))});
    var on=!set||p.r.k.every(function(k){return set.indexOf(k)>=0});p.g.setAttribute('opacity',on?1:.1)})}
function clearParts(){parts.forEach(function(p){p.g.remove()});parts=[];acc=0;needWarm=true}
/* 预热：先跑 6 秒让各线路铺满光点 */
function warm(){if(!canAnimate||!built)return;warming=true;for(var w=0;w<180;w++)step(1/30);warming=false;needWarm=false;draw()}
function canRun(){return built&&!paused&&flowing()&&visible()&&!effectsLow()&&canAnimate}
function loop(t){raf=0;if(!canRun()){last=0;return}var dt=last?Math.min(.05,(t-last)/1000):0;last=t;step(dt);draw();raf=requestAnimationFrame(loop)}
function kick(){if(raf||!canRun())return;last=0;raf=requestAnimationFrame(loop)}

/* ---------- 聚焦与详情浮层 ---------- */
function bars(seed){if(!seed)return'';var h='',x=seed*9301+49297;for(var i=0;i<24;i++){x=(x*9301+49297)%233280;var y=.25+.75*(x/233280)*(.55+.45*Math.sin((i+seed)/3.4));h+='<i style="height:'+Math.round(Math.max(.12,y)*100)+'%"></i>'}return'<div class="ph-bars" aria-hidden="true">'+h+'</div>'}
function mlist(a){var n=4,h=a.slice(0,n).map(function(c){var ok=okr(c);
  return'<li'+(ok<98?' class="ph-low"':'')+'><i style="background:'+v(c.k)+'"></i><span>'+c.n+'</span><b>'+mm(c.tok)+'M</b><small>'+fmt(c.req)+' 次 · '+p1(ok)+'%</small></li>'}).join('');
  if(a.length>n)h+='<li class="ph-mrest"><span>还有 '+(a.length-n)+' 个，见渠道设置</span></li>';return'<ul class="ph-mlist">'+h+'</ul>'}
/* 模型明细：同一窗口内这条渠道（或入口）按模型拆开的记录。速度是该模型完成请求的平均输出速度，
   用 mini 条形长度表示相对快慢（以本卡内最快的模型为满格），右侧写 token/s。 */
function mtable(list){
  if(!list||!list.length)return'';
  var top=Math.max.apply(null,list.map(function(m){return m.tps}));
  return'<div class="ph-mt" role="table" aria-label="按模型拆分">'+
    '<div class="ph-mt-h" role="row"><span>模型</span><span>请求</span><span>速度</span></div>'+
    list.map(function(m){var w=Math.max(8,Math.round(m.tps/top*100));
      return'<div class="ph-mt-r" role="row"><span class="ph-mt-n" title="'+m.n+'">'+m.n+'</span>'+
        '<span class="ph-mt-q">'+fmt(m.req)+'</span>'+
        '<span class="ph-mt-s"><i class="ph-mb"><i style="width:'+w+'%"></i></i><b>'+m.tps+'</b><small>t/s</small></span></div>'}).join('')+
    '</div>'}
function fill(k){var I=INFO[k];
  var platform=anchor&&anchor.dataset.client?PLATFORMS.filter(function(p){return p.k===anchor.dataset.client})[0]:null;
  if(platform)I={n:platform.n,tag:'Agent 平台',c:platform.c,p:'所选时间范围内，该平台记录到的模型与请求。',d:[['请求',fmt(platform.req)+' 次'],['Token',mm(platform.tok)+'M'],['模型',platform.models.length+' 个']],a:['查看请求'],seed:0};
  if(k==='desktop'&&!flowing())I={n:INFO.desktop.n,tag:'整体汇总',c:'core',p:'所选时间范围内的历史汇总；'+ST[state],
    d:[['请求',fmt(T.req)+' 次'],['Token',mm(T.tok)+'M'],['成功率',p1(T.ok)+'%'],['失败',fmt(T.fail)+' 次']],a:['网关日志'],seed:0};
  if(k==='adapter'){var via=CH.filter(function(c){return c.via})[0];if(via)I={n:'实时适配',tag:'Desktop 内置代理',c:via.k,
    p:'Desktop → 实时适配 → '+via.n+' · 协议转换',d:[['请求',fmt(via.req)+' 次'],['Token',mm(via.tok)+'M'],['附加耗时',Number.isFinite(via.adapterMs)?via.adapterMs+'ms':'未记录']],a:['查看实时请求'],seed:0};}
  pop.style.setProperty('--pc',v(I.c));pop.dataset.k=k;
  /* 收纳卡用列表而不是指标表：加个标记，横条档据此把合计句显示出来 */
  pop.classList.toggle('ph-haslist',!!I.list);
  /* 渠道 / 客户端 / 中枢都可能带模型明细；列表型（收纳）仍走渠道列表 */
  var models=I.list?null:(I.models||null);
  pop.classList.toggle('ph-hasm',!!models);
  pop.innerHTML='<div class="ph-pl"><h4>'+I.n+'<span class="ph-tag">'+I.tag+'</span></h4><p>'+I.p+'</p><div class="ph-acts">'+I.a.map(function(t){return'<button type="button" data-ph-act="'+t+'"'+(GO[t]?'':' aria-disabled="true" title="原型暂未接入"')+'>'+t+'</button>'}).join('')+'</div></div>'+
    '<div class="ph-pr">'+(I.list?mlist(I.list):'<dl>'+I.d.map(function(r){return'<div><dt>'+r[0]+'</dt><dd>'+(r[1]==='LIVE'?'<span class="ph-live-n">'+live+'</span>':r[1])+'</dd></div>'}).join('')+'</dl>'+(models?mtable(models):'')+bars(I.seed))+'</div>';
  if(journey&&platform)pop.querySelector('.ph-pl').insertAdjacentHTML('beforeend','<button type="button" class="pj-peek" data-ph-platform="'+platform.k+'">展开平台模型 →</button>');
  else if(journey&&CH.some(function(c){return c.k===k}))pop.querySelector('.ph-pl').insertAdjacentHTML('beforeend','<button type="button" class="pj-peek" data-ph-explore="'+k+'">展开模型 →</button>')}
/* 候选点来自锚点的四边和障碍物边界；按与锚点的距离选位，不再按舞台从上到下找空位。 */
function nearbyPosition(a,w,h,sw,sh,keep,side){
  var pad=10,gap=10,maxX=sw-w-pad,maxY=sh-h-pad;if(maxX<pad||maxY<pad)return null;
  var xs=[a.l-w-gap,a.r+gap,a.x-w/2],ys=[a.t-h-gap,a.b+gap,a.y-h/2];
  keep.forEach(function(b){xs.push(b.l-w-gap,b.r+gap);ys.push(b.t-h-gap,b.b+gap)});
  function axis(values,max){var points=[];values.forEach(function(v){
    points.push(Math.max(pad,Math.min(max,Math.floor(v))),Math.max(pad,Math.min(max,Math.ceil(v))))});
    return points.filter(function(v,i,all){return all.indexOf(v)===i})}
  xs=axis(xs,maxX);ys=axis(ys,maxY);var best=null;
  xs.forEach(function(x){ys.forEach(function(y){
    if(keep.some(function(b){return x+w>b.l-gap&&x<b.r+gap&&y+h>b.t-gap&&y<b.b+gap}))return;
    var dx=Math.max(a.l-x-w,x-a.r,0),dy=Math.max(a.t-y-h,y-a.b,0);
    var preferred=side==='right'?x>=a.r:side==='left'?x+w<=a.l:y+h<=a.t;
    var score=Math.hypot(dx,dy)+.18*Math.hypot(x+w/2-a.x,y+h/2-a.y)+(preferred?0:12);
    if(!best||score<best.score)best={x:x,y:y,score:score};
  })});return best}
function place(){if(!anchor)return;
  pop.classList.add('ph-hubc','ph-near');pop.classList.remove('ph-dock','ph-mini','ph-flat','ph-compact','ph-tiny','ph-bar');stage.classList.remove('ph-docked');
  pop.dataset.mode='full';pop.dataset.cols='3';delete pop.dataset.wide;pop.style.height='';pop.style.width='';
  /* 弹窗在主内容区浮层中定位，允许越过图形边框；缩放时统一换算布局坐标。 */
  var container=document.fullscreenElement===hero?hero:host.closest('.main')||host;
  var theme=getComputedStyle(host);for(var ti=0;ti<theme.length;ti++){var key=theme[ti];if(key.indexOf('--ph-')===0)pop.style.setProperty(key,theme.getPropertyValue(key))}
  if(pop.parentElement!==container)container.appendChild(pop);
  var sr=container.getBoundingClientRect(),sx=sr.width/container.offsetWidth||1,sy=sr.height/container.offsetHeight||1;
  function local(e){var r=e.getBoundingClientRect(),l=(r.left-sr.left)/sx-container.clientLeft,t=(r.top-sr.top)/sy-container.clientTop;
    return{l:l,t:t,r:l+r.width/sx,b:t+r.height/sy,x:l+r.width/sx/2,y:t+r.height/sy/2}}
  var a=local(hoverIcon(anchor)),sw=container.clientWidth,sh=container.clientHeight;
  /* 只保护当前节点；其他节点可以被临时覆盖，不再据此挤压卡片或把它推远。 */
  var side=anchor.classList.contains('ph-client')?'right':anchor.classList.contains('ph-chan')?'left':'above';
  var pw=pop.offsetWidth,ph=pop.offsetHeight,best=nearbyPosition(a,pw,ph,sw,sh,[a],side);
  /* 超出既定窗口档位时也保持卡片规格，只做边界平移。 */
  if(!best)best={x:Math.max(10,Math.min(sw-pw-10,a.x-pw/2)),y:Math.max(10,Math.min(sh-ph-10,a.t-ph-10))};
  pop.style.left=Math.round(best.x)+'px';pop.style.top=Math.round(best.y)+'px'}

function apply(){var set=focus?SET[focus]:null;stage.classList.toggle('ph-focusing',!!set);
  pop.classList.toggle('ph-visible',!!set);pop.classList.toggle('ph-interactive',!!locked);
  stage.classList.toggle('ph-hubfocus',focus==='desktop');
  stage.classList.toggle('ph-lock',!!locked);
  QA('[data-k]').forEach(function(e){e.classList.toggle('ph-dim',!!set&&set.indexOf(e.dataset.k)<0)});
  QA('.ph-locked').forEach(function(e){e.classList.remove('ph-locked')});
  QA('[data-f]').forEach(function(e){e.setAttribute('aria-expanded',String(!!set&&e===anchor))});
  if(locked&&anchor&&anchor.matches('.ph-node,.ph-hub'))anchor.classList.add('ph-locked');
  if(set){if(pop.dataset.k!==focus)fill(focus);place()}else{stage.classList.remove('ph-docked');delete pop.dataset.k}
  draw()}
function cancelHoverClose(){if(hoverCloseTimer){clearTimeout(hoverCloseTimer);hoverCloseTimer=0}}
function deferHoverClose(){
  if(!focus||locked||hoverCloseTimer)return;
  hoverCloseTimer=setTimeout(function(){hoverCloseTimer=0;
    if(!locked&&!pointerInPop&&!selectingInPop&&!pop.contains(document.activeElement))setFocus(null);
  },220)}
function setFocus(k,a){cancelHoverClose();if(k&&k!=='desktop'&&!flowing())k=null;if(anchor!==a&&pop)delete pop.dataset.k;focus=k;anchor=k?(a||null):null;
  if(!k){pointerInPop=false;selectingInPop=false}apply()}
function toggle(t){if(t){if(locked===t){locked=null;setFocus(null)}else{locked=t;setFocus(t.dataset.f,t)}}else if(locked){locked=null;setFocus(null)}}
function label(e){if(e===hub)return'OpenCodeX-Desktop';var b=e.querySelector('.ph-l1 b')||e.querySelector('.ph-nl>b'),t=b&&b.firstChild?b.firstChild.textContent:'';if(e.dataset.f==='more')return t+'渠道：'+FOLD.map(function(c){return c.n}).join('、');return e.dataset.f==='codex'?t+' · codex 入口':t}
function a11y(list){list.forEach(function(e){e.tabIndex=0;e.setAttribute('role','button');e.setAttribute('aria-label',label(e));e.setAttribute('aria-controls','phPop');e.setAttribute('aria-expanded','false')})}

/* ---------- 节点与详情组成同一悬停区 ----------
   弹窗可选字和操作；两者间隙留通道，退出整组后再延迟关闭，避免移动途中闪退。 */
function hoverIcon(e){return e===hub?e:e.querySelector('.ph-orb')||e}
function inIconBounds(b,x,y){if(!b.width||!b.height)return false;
  return Math.pow((x-b.left-b.width/2)/(b.width/2),2)+Math.pow((y-b.top-b.height/2)/(b.height/2),2)<=1}
/* 通道只占图标与弹窗相对边之间的重叠投影，宽度不超过图标，不向整张卡片扩张。 */
function inIconBridge(a,b,x,y){var l,t,r,d;
  if(a.right<=b.left){l=a.right;r=b.left;t=Math.max(a.top,b.top);d=Math.min(a.bottom,b.bottom)}
  else if(b.right<=a.left){l=b.right;r=a.left;t=Math.max(a.top,b.top);d=Math.min(a.bottom,b.bottom)}
  else if(a.bottom<=b.top){l=Math.max(a.left,b.left);r=Math.min(a.right,b.right);t=a.bottom;d=b.top}
  else if(b.bottom<=a.top){l=Math.max(a.left,b.left);r=Math.min(a.right,b.right);t=b.bottom;d=a.top}
  else return false;
  return r>l&&d>t&&x>=l&&x<=r&&y>=t&&y<=d}
function inHoverBridge(x,y){return !!(focus&&anchor&&inIconBridge(hoverIcon(anchor).getBoundingClientRect(),pop.getBoundingClientRect(),x,y))}
var hoverAnchor=null;
function hitAt(cx,cy){
  /* 直接用视口中的实际图标尺寸，缩放后也不额外扩充命中区域。 */
  hoverAnchor=null;
  var list=flowing()?QA('[data-f]'):[hub],best=null,dist=0;
  for(var i=0;i<list.length;i++){var e=list[i],b=hoverIcon(e).getBoundingClientRect();
    if(inIconBounds(b,cx,cy)){var d=Math.hypot(cx-b.left-b.width/2,cy-b.top-b.height/2);if(!best||d<dist){best=e;dist=d}}}
  hoverAnchor=best;return best?best.dataset.f:null}
/* 进程操作镜像：顶部状态卡隐藏后仍由原按钮执行，镜像只负责显示和转发点击 */
function syncCtl(){var card=document.getElementById('card-overview-status'),ctl=Q('#phHubCtl');if(!card||!ctl)return;
  var badge=card.querySelector('[data-b="statebadge"]'),txt=card.querySelector('[data-b="statetext"]');
  var dot=ctl.querySelector('.ph-cdot'),st=ctl.querySelector('#phCstT');
  if(dot&&badge){var cls=(badge.className||'').replace('ovc-badge','').trim();dot.className='ph-cdot'+(cls?' '+cls:'')}
  if(st&&txt){var lab=(txt.textContent||'').trim();if(lab&&st.textContent!==lab)st.textContent=lab;if(lab)ctl.title='进程状态：'+lab}
  CTL_ACT.forEach(function(a){var real=card.querySelector('[data-act="'+a[0]+'"]'),m=ctl.querySelector('[data-ph-mirror="'+a[0]+'"]');if(!m)return;
    var hidden=!real||real.hidden;if(m.hidden!==hidden)m.hidden=hidden;
    if(!real)return;var t=(real.textContent||'').trim()||a[1];if(m.title!==t){m.title=t;m.setAttribute('aria-label',t)}
    var dis=!!real.disabled||real.getAttribute('aria-disabled')==='true';if(m.disabled!==dis)m.disabled=dis})}

/* ---------- 背景 Logo 层：复用 Logo 本体形变页，只在构建后加载 ---------- */
/* 光层铺满 .main 可视区（全屏时铺满全景），iframe 以中枢为中心，旋转光场从中枢向整页扩散 */
function placeGlow(){if(!glow||!frame||!hub)return;var fs=document.fullscreenElement===hero,tg=fs?hero:host.closest('.main')||host;
  glow.style.left=glow.style.top='0px';glow.style.width=glow.style.height='0px';
  var o=glow.getBoundingClientRect(),t=tg.getBoundingClientRect(),sc=tg.offsetHeight?t.height/tg.offsetHeight:1;if(!sc)sc=1;
  var gl=(t.left-o.left)/sc+tg.clientLeft,gt=(t.top-o.top)/sc+tg.clientTop,gw=tg.clientWidth,gh=tg.clientHeight;
  glow.style.left=Math.round(gl)+'px';glow.style.top=Math.round(gt)+'px';glow.style.width=gw+'px';glow.style.height=gh+'px';
  var hb=hub.getBoundingClientRect(),hx=(hb.left+hb.width/2-o.left)/sc-gl,hy=(hb.top+hb.height/2-o.top)/sc-gt;
  var hw=Math.ceil(Math.max(hx,gw-hx,1)),hh=Math.ceil(Math.max(hy,gh-hy,1));
  frame.style.width=2*hw+'px';frame.style.height=2*hh+'px';frame.style.left=Math.round(hx-hw)+'px';frame.style.top=Math.round(hy-hh)+'px';
  geo={size:Math.round(Math.max(64,Math.min(140,R*1.55))),x:hw,y:hh}}
function mountGlow(){var want=document.fullscreenElement===hero?hero:host;if(glow.parentNode===want)return;ready=false;lastMsg='';want.appendChild(glow)}
function frameUrl(e,r){var base=script&&script.src?script.src:location.href;return new URL(SRC,base).href+'?embed=1&layout=hero&effects='+encodeURIComponent(e)+'&render='+encodeURIComponent(r)+'&orbit=1&v='+VER}
function syncFrame(){if(!built||!geo||!frame)return;var e=root.dataset.effects||'high',r=root.dataset.glowRender||'css',key=e+'|'+r;
  if(key!==frameKey){frameKey=key;ready=false;lastMsg='';frame.src=frameUrl(e,r)}post()}
function post(){if(!ready||!geo||!frame||!frame.contentWindow)return;
  var m={type:'opencodex-motion-preview',state:state,theme:root.dataset.theme==='dark'?'dark':'light',active:!paused&&visible(),tempo:tempo,presentation:{size:geo.size,x:geo.x,y:geo.y,glowX:.5,glowY:.5}};
  var j=JSON.stringify(m);if(j===lastMsg)return;lastMsg=j;
  frame.contentWindow.postMessage(m,location.protocol==='file:'?'*':location.origin)}
function markReady(){ready=true;lastMsg='';post()}

/* ---------- 构建调度与可见性 ---------- */
function canBuild(){return visible()&&hero&&!hero.hidden&&stage.clientWidth>0}
function schedule(){dirty=true;if(sraf)return;sraf=requestAnimationFrame(function(){sraf=0;if(dirty&&canBuild())build()})}
function syncTimer(){var on=built&&visible()&&!paused&&flowing();
  if(!on&&timer){clearInterval(timer);timer=0}}
/* 运行状态：与 overview-dual.js 同一映射；进程操作进行中优先显示启动中 / 停止中。 */
var MAP={loading:'confirming',not_found:'not_ready',stopped:'stopped',starting:'starting',pending:'starting',running:'running',starting_failed:'failed',at_risk:'stopped',external_takeover:'problem',unreachable:'problem'};
function heroState(){var t=document.body.dataset.taskop||'';if(t==='op_start')return'starting';if(t==='op_stop'||t==='op_restart')return'stopping';
  var c=typeof currentState!=='undefined'?currentState:'running';return MAP[c]||'confirming'}
function writeSubtitle(){if(typeof routes!=='undefined'&&routes.overview)routes.overview.subtitle=subtitle();
  if(root.dataset.route!=='overview'||setup())return;var st=document.getElementById('routeSubtitle');if(st&&st.textContent!==subtitle())st.textContent=subtitle()}
/* 就绪时主角舞台自带 Logo 光层，旧背景光层先卸载（保存地址），回到环境门禁时再恢复。 */
function backdrop(){var b=host.querySelector('.motion-backdrop');if(!b)return;var s=b.getAttribute('src')||'';
  if(!setup()){if(s&&s!=='about:blank'){b.dataset.ovSrc=s;b.setAttribute('src','about:blank')}}
  else if(b.dataset.ovSrc&&s==='about:blank')b.setAttribute('src',b.dataset.ovSrc)}
function refresh(){
  if(!hero)return;
  var on=!setup();hero.hidden=!on;glow.hidden=!on;
  if((!on||root.dataset.route!=='overview')&&(focus||locked)){locked=null;setFocus(null)}
  if((!on||root.dataset.route!=='overview')&&journey&&journey.active())journey.exit();
  setState(heroState());writeSubtitle();backdrop();syncCtl();
  var th=root.dataset.theme||'';if(th!==lastTheme){lastTheme=th;dirty=true}
  if(effectsLow()&&parts.length)clearParts();
  if(dirty&&visible())schedule();
  else if(built&&needWarm&&!effectsLow()&&flowing()&&visible())warm();
  syncFrame();syncTimer();kick()}
function syncPause(){hero.classList.toggle('ph-paused',paused)}
function setState(st){if(!ST[st])return;var c=AMB.indexOf(st)>=0?'amber':BAD.indexOf(st)>=0?'bad':'ok';hero.style.setProperty('--ph-stc',v(c));
  var tone=st==='running'?'run':st==='stopped'?'idle':c==='amber'?'amber':'bad';hero.dataset.tone=tone;
  /* 全站速度档位：请求速率 + 各渠道输出速度。圆环扫光、Logo 游走、光点、涟漪都换自它。 */
  paintSpeed();
  hero.querySelectorAll('.ph-st-cap').forEach(function(e){e.textContent=st==='running'?'':' · '+ST[st]});
  var was=state;state=st;hero.classList.toggle('ov-idle',!flowing());
  syncOverviewAvailability();
  if(was!==st){if(focus==='desktop'){delete pop.dataset.k;apply()}if(visible())schedule()}
  if(was!==st){if(!flowing()){clearParts();live=0}else if(!live)live=2;
    hero.querySelectorAll('.ph-live-n').forEach(function(b){b.textContent=live});
    if(flowing()&&built&&!effectsLow()&&visible())warm()}
  post();syncTimer();kick();if(journey&&journey.active())journey.syncMotion()}
/* ---------- 初始化 ---------- */
var inited=false;
function init(){
  if(inited)return;inited=true;
  /* 先按当前窗口生成样例，再拼 DOM，否则客户端与 KPI 会拿到空数据 */
  buildWindow();
  hero=document.createElement('section');hero.className='ov-hero';hero.id='ovHero';hero.setAttribute('aria-label','流量全景');
  /* 不再写“流量全景”标题；原型注释标不占位，叠在 KPI 行右端 */
  hero.innerHTML='<div class="ov-hero-head"><div id="phKpis">'+kpisHtml()+'</div><div class="ov-hero-note">'+NOTE+'</div><div class="ph-tools">'+TOOLS+'</div></div>'+
    '<section class="ph-stage" id="phStage"><svg class="ph-wires" id="phWires" aria-hidden="true"></svg>'+
    clientsHtml()+HUB+'<div class="ph-channels"></div>'+'<div class="ph-pop" id="phPop" role="group" aria-label="详情"></div></section>';
  glow=document.createElement('div');glow.className='ph-glow';glow.setAttribute('aria-hidden','true');
  glow.innerHTML='<iframe class="ph-motion" id="phMotion" title="" aria-hidden="true" tabindex="-1"></iframe>';
  var status=document.getElementById('card-overview-status');
  if(status&&status.parentNode===host)status.after(hero);else host.prepend(hero);
  host.appendChild(glow);
  toolsEl=hero.querySelector('.ph-tools');noteEl=hero.querySelector('.ov-hero-note');
  stage=hero.querySelector('#phStage');svg=hero.querySelector('#phWires');frame=glow.querySelector('#phMotion');pop=hero.querySelector('#phPop');hub=hero.querySelector('#phHub');
  pg=document.createElementNS(NS,'g');pg.id='phPg';svg.appendChild(pg);
  buildWindow();
  renderCh(K);
  bindFlowControls();syncFlowUi();
  a11y(QA('[data-f]'));
  try{paused=!!(window.matchMedia&&window.matchMedia('(prefers-reduced-motion: reduce)').matches)}catch(e){paused=false}
  syncPause();
  if(window.OverviewJourney){
    journey=window.OverviewJourney.mount({hero:hero,stage:stage,
      getData:function(){return{channels:CH,platforms:PLATFORMS,range:{key:win+'|'+customFrom+'|'+customTo,label:customFrom?winFromRange():win,from:customFrom,to:customTo,seconds:winScale()*86400}}},
      revealChannel:function(){renderCh(K)},
      getMotion:function(){return{running:flowing(),pixelSpeed:SPD}},
      canExplore:flowing,
      clearHover:function(){locked=null;setFocus(null)},
      restore:function(){if(visible())schedule()},
      focusReturn:function(node){suppress=true;try{node.focus({preventScroll:true})}finally{suppress=false}},
      onPose:function(p){if(!frame||!geo)return;frame.style.transformOrigin=geo.x+'px '+geo.y+'px';frame.style.transform=p?'translate('+p.dx+'px,'+p.dy+'px) scale('+p.scale+')':''}
    });
    var source=hero.querySelector('.annotation-source');if(source)source.insertAdjacentHTML('beforeend','<p>连续场景效果：运行后点击平台或渠道进入模型，再选择请求查看实际去向与阶段，两侧可查看同一请求。平台归属使用明确的固定 mock 配额，不从共用入口反推真实客户端。平台 / 模型汇总不混合不同渠道的输出速度，请求显示对应渠道速度；未知来源保留未区分。模型和请求每组三项，可翻组。顶部 KPI 为全局统计，时间变化清除单次请求，未运行只显示整体。路径按钮和 Esc 逐级返回；原对象保留连续位移，Logo 净空、实时适配与光点沿用；低特效与减少动态直接显示终态。尚未接真实日志与平台识别。</p>');
  }

  /* 节点 / 通道 / 弹窗共同保持悬停，选字拖动与弹窗内键盘焦点也延续详情。 */
  function over(e){if(locked||e.pointerType==='touch'||journey&&journey.active())return;
    pointerInPop=!!focus&&pop.contains(e.target);
    if(focus&&(pointerInPop||selectingInPop||pop.contains(document.activeElement))){cancelHoverClose();return}
    var sr=stage.getBoundingClientRect();
    if(e.clientX<sr.left||e.clientX>sr.right||e.clientY<sr.top||e.clientY>sr.bottom){
      if(inHoverBridge(e.clientX,e.clientY))cancelHoverClose();else deferHoverClose();return}
    var f=hitAt(e.clientX,e.clientY);
    if(f){cancelHoverClose();var t=hoverAnchor;if(f===focus&&t===anchor)return;if(t)setFocus(f,t);return}
    if(inHoverBridge(e.clientX,e.clientY))cancelHoverClose();else deferHoverClose()}
  document.addEventListener('pointermove',over);
  stage.addEventListener('pointerleave',function(e){if(locked||e.pointerType==='touch')return;
    if(e.relatedTarget&&pop.contains(e.relatedTarget)){cancelHoverClose();return}deferHoverClose()});
  pop.addEventListener('pointerenter',function(){pointerInPop=true;cancelHoverClose()});
  pop.addEventListener('pointerleave',function(){pointerInPop=false;deferHoverClose()});
  pop.addEventListener('pointerdown',function(e){if(e.button===0){selectingInPop=true;cancelHoverClose()}});
  function endSelection(){if(!selectingInPop)return;selectingInPop=false;
    if(!pointerInPop)deferHoverClose()}
  document.addEventListener('pointerup',endSelection);
  document.addEventListener('pointercancel',endSelection);
  pop.addEventListener('focusin',cancelHoverClose);
  pop.addEventListener('focusout',function(e){if(e.relatedTarget&&(pop.contains(e.relatedTarget)||stage.contains(e.relatedTarget)))return;deferHoverClose()});
  stage.addEventListener('click',function(e){var g=e.target.closest('[data-ph-go]');
    if(e.target.closest('.pj-scene'))return;
    var platformNode=e.target.closest('.ph-client[data-client]');
    if(platformNode&&journey&&flowing()){journey.platform(platformNode.dataset.client);return}
    if(e.target.closest('#phAdapter')&&journey&&flowing()){
      var via=CH.filter(function(c){return c.via})[0];if(journey.active()){journey.adapter();return}if(via){journey.enter(via.k);journey.adapter();return}}
    var channelNode=e.target.closest('.ph-chan[data-k]');
    if(channelNode&&channelNode.dataset.k!=='more'&&journey){journey.enter(channelNode.dataset.k);return}
    if(journey&&journey.active())return;
    if(g){var gb=document.querySelector('.nav button[data-route="'+g.dataset.phGo+'"]');locked=null;setFocus(null);if(gb)gb.click();return}
    if(e.target.closest('#phPop')||e.target.closest('[aria-disabled="true"]'))return;toggle(e.target.closest('[data-f]'))});
  stage.addEventListener('keydown',function(e){if(e.key!=='Enter'&&e.key!==' ')return;var t=e.target;if(!t.matches||!t.matches('[data-f]'))return;e.preventDefault();
    if(t.dataset.client&&journey&&flowing()){journey.platform(t.dataset.client);return}
    if(t.id==='phAdapter'&&journey&&flowing()){var via=CH.filter(function(c){return c.via})[0];if(journey.active()){journey.adapter();return}if(via){journey.enter(via.k);journey.adapter();return}}
    if(journey&&t.matches('.ph-chan')&&t.dataset.k!=='more'){journey.enter(t.dataset.k);return}if(!journey||!journey.active())toggle(t)});
  stage.addEventListener('focusin',function(e){if(suppress||locked||journey&&journey.active())return;var t=e.target;if(t.matches&&t.matches('[data-f]')&&(t.dataset.f!==focus||t!==anchor))setFocus(t.dataset.f,t)});
  stage.addEventListener('focusout',function(e){if(locked||(e.relatedTarget&&(stage.contains(e.relatedTarget)||pop.contains(e.relatedTarget))))return;if(focus&&anchor&&anchor.contains(e.target))deferHoverClose()});
  pop.addEventListener('click',function(e){var platform=e.target.closest('[data-ph-platform]');if(platform&&journey){journey.platform(platform.dataset.phPlatform);return}
    var explore=e.target.closest('[data-ph-explore]');if(explore&&journey){journey.enter(explore.dataset.phExplore);return}
    var b=e.target.closest('[data-ph-act]');if(!b||b.getAttribute('aria-disabled')==='true')return;
    var nb=GO[b.dataset.phAct]&&document.querySelector('.nav button[data-route="'+GO[b.dataset.phAct]+'"]');if(!nb)return;locked=null;setFocus(null);nb.click()});
  /* 自定义区间面板：遮罩关闭、日期点选、翻月、快捷区间、时分下拉、应用与取消 */
  /* 时间弹层挂在 body 上，所以监听放到 document，保证点选可达 */
  document.addEventListener('click',function(e){
    /* 时间弹层：点面板外部即关闭 */
    if(rngTimePop&&!e.target.closest('.ph-rng-tpop')&&!(e.target.dataset&&e.target.dataset.rng))closeRngTime();
    if(!rng)return;
    if(e.target.matches('[data-rng-mask]')){closeRangeDialog();return}
    if(e.target.closest('[data-rng-close]')||e.target.closest('[data-rng-cancel]')){closeRangeDialog();return}
    if(e.target.closest('[data-rng-apply]')){rngApply();return}
    if(e.target.closest('[data-rng-mprev]')||e.target.closest('[data-rng-mnext]')){
    var side=e.target.dataset.rngMprev?'mprev':'mnext',m= rngMonth||new Date();
    var mstep=(side==='mnext'?1:-1),shift=(e.target.dataset.rngMprev==='右'||e.target.dataset.rngMnext==='右')?1:0;
    rngMonth=new Date(m.getFullYear(),m.getMonth()+shift+mstep,1);redrawRangeCalendar();return}
    if(e.target.closest('[data-rng-yprev]')||e.target.closest('[data-rng-ynext]')){
    var ySide=e.target.dataset.rngYprev||e.target.dataset.rngYnext,ym=rngMonth||new Date();
    var yStep=e.target.dataset.rngYnext?12:-12,yShift=(ySide==='右')?1:0;
    rngMonth=new Date(ym.getFullYear(),ym.getMonth()+yShift+yStep,1);redrawRangeCalendar();return}
  if(e.target.closest('[data-rng-prev]')||e.target.closest('[data-rng-next]')){
      var dir=e.target.closest('[data-rng-prev]')?-1:1;rngMonth=new Date(rngMonth.getFullYear(),rngMonth.getMonth()+dir,1);
      redrawRangeCalendar();return}
    var chip=e.target.closest('[data-rng-chip]');
    if(chip){var end=new Date('2026-11-02T00:00'),start=new Date(end),k=chip.dataset.rngChip;
      if(k==='today')start.setHours(0,0,0,0);
      else if(k==='yesterday'){start.setDate(start.getDate()-1);start.setHours(0,0,0,0);end.setHours(0,0,0,0)}
      else if(k==='last7')start.setDate(start.getDate()-7);
      else if(k==='last30')start.setDate(start.getDate()-30);
      else if(k==='month'){start.setDate(1);start.setHours(0,0,0,0)}
      rngDraft.from=rngToLocal(start);rngDraft.to=rngToLocal(end);rngPick=null;rngMonth=new Date(start);
      redrawRangeDates();rngRender();return}
    var day=e.target.closest('[data-rng-day]');
    if(day){var key=day.dataset.rngDay;
      if(!rngPick){rngDraft.from=key+'T'+rngDraft.from.slice(11,16);rngPick=key}else{
        var lo=key<rngPick?key:rngPick,hi=key<rngPick?rngPick:key;
        rngDraft.from=lo+'T'+rngDraft.from.slice(11,16);rngDraft.to=hi+'T'+rngDraft.to.slice(11,16);rngPick=null}
      redrawRangeDates();rngRender();return}
    var h=e.target.closest('[data-rng-h]'),m=e.target.closest('[data-rng-m]');
    if(e.target.closest('[data-rng-now]')){
      var own=rngTimePop?rngTimePop.dataset.owner:'fromTime',now=new Date();
      var inp=rng.querySelector('[data-rng="'+own+'"]');
      if(inp){var vv=pad2(now.getHours())+':'+pad2(now.getMinutes());inp.value=vv;
        if(own==='fromTime')rngDraft.from=rngDraft.from.slice(0,10)+'T'+vv;else rngDraft.to=rngDraft.to.slice(0,10)+'T'+vv;
        rngRender()}return}
    if(h||m){var owner=rngTimePop?rngTimePop.dataset.owner:'fromTime';
      setRngPart(owner,h?'h':'m',h?h.dataset.rngH:m.dataset.rngM);return}
  });
  hero.addEventListener('focusin',function(e){var f=e.target.dataset&&e.target.dataset.rng;if(f&&(f==='fromTime'||f==='toTime')&&rng)openRngTime(f)});

  document.addEventListener('keydown',function(e){if(e.key!=='Escape'||!visible()||e.defaultPrevented)return;
    var m=document.getElementById('modalMask');if(m&&m.style.display==='flex')return;
    var ap=document.getElementById('annotationPopover');if(ap&&!ap.hidden)return;
    if(rngTimePop){e.preventDefault();closeRngTime();return}
    if(rng){e.preventDefault();closeRangeDialog();return}
    if(journey&&journey.active()){e.preventDefault();journey.back();return}
    if(!focus&&!locked)return;
    var back=pop.contains(document.activeElement)?anchor:null;locked=null;setFocus(null);
    if(back){suppress=true;try{back.focus()}finally{suppress=false}}});

  /* 工具条 */
  toolsEl.addEventListener('click',function(e){
    var w=e.target.closest('[data-ph-win]');
    if(w){win=w.dataset.phWin;customFrom='';customTo='';toolsEl.querySelectorAll('[data-ph-win]').forEach(function(b){b.setAttribute('aria-pressed',String(b===w))});
      /* 切窗口要重取数据：样例按窗口系数重算，再刷新 KPI、客户端、渠道与速度 */
      if(typeof buildWindow==='function'){buildWindow();renderClients();renderCh(K);paintSpeed();syncFlowUi();if(built)build();else if(journey&&journey.active())journey.refresh();if(focus)apply()}
      writeSubtitle();return}
    /* 日历图标：打开自定义区间，应用后按实际跨度重算 */
    if(e.target.closest('[data-ph-range]')){openRangeDialog();return}
    if(e.target.closest('[data-ph-full]')){var r;try{r=document.fullscreenElement?document.exitFullscreen():hero.requestFullscreen&&hero.requestFullscreen()}catch(x){r=null}if(r&&r.catch)r.catch(function(){})}});

  /* 中枢控制区：状态只读，操作按钮转发给顶部状态卡的同一颗按钮 */
  var ctl=hero.querySelector('#phHubCtl');
  if(ctl)ctl.addEventListener('click',function(e){var m=e.target.closest('[data-ph-mirror]');if(!m||m.disabled)return;
    var card=document.getElementById('card-overview-status'),real=card&&card.querySelector('[data-act="'+m.dataset.phMirror+'"]');
    if(real&&!real.hidden)real.click()});

  frame.addEventListener('load',function(){if(frame.getAttribute('src'))markReady()});
  window.addEventListener('message',function(e){if(frame.contentWindow&&e.source===frame.contentWindow&&e.data&&e.data.type==='opencodex-motion-ready')markReady()});
  window.addEventListener('resize',schedule);
  document.addEventListener('fullscreenchange',function(){mountGlow();schedule()});
  document.addEventListener('visibilitychange',refresh);
  if(window.ResizeObserver)try{new ResizeObserver(function(){if(built||visible())schedule()}).observe(stage)}catch(e){}
  if(document.fonts&&document.fonts.ready&&document.fonts.ready.then)document.fonts.ready.then(schedule,function(){});
  new MutationObserver(refresh).observe(root,{attributes:true,attributeFilter:['data-route','data-theme','data-effects','data-glow-render','data-overview-size']});
  new MutationObserver(refresh).observe(host,{attributes:true,attributeFilter:['data-mode']});
  new MutationObserver(refresh).observe(document.body,{attributes:true,attributeFilter:['data-taskop']});
  document.addEventListener('proto-task-op',refresh);
  /* 运行状态变化都经过 render()；overview-dual.js 已包装一层，这里在其后再接一层。 */
  if(typeof render==='function'){var base=render;render=function(){var r=base.apply(this,arguments);refresh();return r}}
  refresh();
}
init();

window.overviewHeroPrototype={
  state:function(){return{mode:host.dataset.mode||'',win:win,paused:paused,focus:focus,locked:!!locked,built:built,state:state}},
  /* 评审与自测用：读当前速度档位与各渠道速度，便于确认滑块/场景真的作用到画面 */
  speed:function(){var g=globalGear();return{gear:g,scale:FSCALE.gear,rate:FSCALE.rate,scene:FSCALE.scene,
    spawnPerSec:RATE*g*FSCALE.rate,partSpeed:SPD*g,pixelPerSec:parts.length?Math.round(parts[0].v):0,
    sweep:hero?hero.style.getPropertyValue('--ph-sweep-dur'):'',tps:CH.map(function(c){return[c.k,c.tps]})}},
  scene:function(name){applyFlowScene(name);return this.speed()},
  setState:setState,
  setFocus:function(k){if(!stage)return;if(!k||!SET[k]){locked=null;setFocus(null);return}var a=k==='desktop'?hub:stage.querySelector('[data-f="'+k+'"]');if(a){locked=a;setFocus(k,a)}},
  build:function(){if(canBuild())build();else schedule()},
  journey:function(){return journey},
  channels:function(){return{K:K,keys:NODES.map(function(c){return c.k}),fold:FOLD.map(function(c){return c.k})}}
};
})();
