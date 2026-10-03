/* 独立视觉候选。仅消费演示投影，不采集状态、不推定操作成功。 */
'use strict';
const NS='http://www.w3.org/2000/svg';
const $=id=>document.getElementById(id);
// shape = [节点半径增量, 连接显现, 柔性弯曲正弦分量, 余弦分量]。
const STATES=[
 {id:'not_ready',name:'待接入',short:'B · 离散漂浮',main:'待接入',op:'—',verb:'三球分离，云朵轻轻漂浮',meaning:'没有连接线。三个球围绕统一中心均衡分布，漂浮表达等待接入。',note:'演示：尚未接入运行来源，不视为故障。',shape:[.55,0,0,0]},
 {id:'stopped',name:'未运行',short:'B · 低位待命',main:'未运行',op:'—',verb:'三球规整待命，云朵落在较低的悬浮位置',meaning:'小幅起伏保持轻盈感，不使用正常运行的整圈旋转。',note:'演示：来源已接入，可启动。',shape:[.20,0,0,0]},
 {id:'starting',name:'正在启动',short:'B → C · 轻跃聚合',main:'未运行',op:'启动 / 执行中',verb:'轻跃蓄能，分离三球逐渐建立柔性连接',meaning:'内外反向旋转，柔性曲线保持完整圆润；只有运行核验通过才进入 A 完整内核。',note:'演示：启动执行中，尚未核验运行。',shape:[.16,.65,.16,0]},
 {id:'running',name:'运行中',short:'A · 稳定悬浮',main:'运行中',op:'—',verb:'完整内核固定形状旋转，云朵稳定悬浮',meaning:'连接线不再伸缩或碎裂；三球与连接整体匀速转动，外形保持原始比例。',note:'演示：新鲜观测确认运行。',shape:[0,1,0,0]},
 {id:'stopping',name:'正在停止',short:'A → C → B · 下沉',main:'运行中',op:'停止 / 核验中',verb:'内核减速，连接柔和退去，云朵缓慢下沉',meaning:'过渡结束后保持低位浮动。动画不自行宣布停止，主线等待核验。',note:'演示：等待停止核验。',shape:[.20,0,0,0]},
 {id:'failed',name:'启动失败',short:'C → B · 回落',main:'未运行',op:'启动 / 失败',verb:'接合退回，云朵短促回落后低位漂浮',meaning:'回落只在进入失败时发生一次，之后三球有限角回摆；不循环假装重新启动。',note:'演示：失败属于启动操作；可查看原因与重试。',shape:[.30,0,0,0]},
 {id:'problem',name:'运行中 · 待处理',short:'C · 柔性牵动',main:'运行中',op:'—',verb:'连续曲线轻微牵动，云朵不规则但平缓地悬浮',meaning:'保持完整连接和持续转动，以柔性张力、琥珀光和文字表达运行中问题。',note:'演示：仍在运行，存在待处理问题。',shape:[0,1,.22,0]},
 {id:'confirming',name:'正在确认',short:'B · 悬浮探寻',main:'正在确认',op:'读取状态',verb:'分离三球双向寻位，云朵缓慢横向漂移',meaning:'无连接的内核往返扫描，不表示正常运行，也不按时间猜测结果。',note:'演示：没有可用新鲜观测。',shape:[.35,0,0,0]},
 {id:'stale',name:'状态已过期',short:'保留结构 · 失重漂移',main:'正在确认',op:'等待重新采集',verb:'保留上一结构，褪色后缓缓漂移',meaning:'保留最后的内部结构，转速平滑衰减后停住，只留下低幅整体漂移和雾灰残光。',note:'演示：上次结构仅为旧事实，当前状态已过期。',shape:[0,1,0,0]}
];
const BRAND=['#B1A7FF','#7A9DFF','#3941FF'];
const PALETTES={
  not_ready:{colors:['#CCD3EA','#A4B3D6','#778CB7'],name:'雾蓝 · 待聚合'},
  stopped:{colors:['#C7B5F4','#929ADF','#6473BB'],name:'柔紫 · 静置'},
  starting:{colors:['#E0A0FF','#8775F5','#398AF2'],name:'鸢紫 → 电光蓝'},
  running:{colors:['#80E9CD','#3BAED1','#5861E8'],name:'薄荷青 → 靛蓝'},
  stopping:{colors:['#B7CAEF','#9494DC','#7675B3'],name:'暮蓝 · 收束'},
  failed:{colors:['#FFC39E','#F08093','#B35BB4'],name:'珊瑚 → 莓紫'},
  problem:{colors:['#FFE0A0','#ECAF68','#8A83D9'],name:'琥珀 → 薰衣草'},
  confirming:{colors:['#B2DCF8','#8EAAF2','#9F80E7'],name:'冰蓝 → 浅紫'},
  stale:{colors:['#CFD4DF','#A2ABBE','#828DA4'],name:'雾灰 · 缓动'}
};
const rgb=hex=>hex.match(/[a-f\d]{2}/gi).map(v=>parseInt(v,16));
// 调色板覆写：仅改变取色，不新增/删除状态（状态数量冻结在 9）。
// at_risk（应用内「存在风险」）沿用 problem 的「柔性牵动」结构，但改用**更纯的琥珀**取色，
// 与 external_takeover / unreachable 的「琥珀→薰衣草」区分开：同为问题结构，色相更黄更暖。
const PALETTE_OVERRIDES={at_risk:{colors:['#FFE9B0','#E8A22F','#9A6A16'],name:'纯琥珀 · 运行中待处理'}};
let paletteHint='';
const paletteOf=i=>{const ov=PALETTE_OVERRIDES[paletteHint];return (ov?ov.colors:PALETTES[STATES[i].id].colors).map(rgb)};
const colorText=c=>`rgb(${c.map(v=>Math.round(v)).join(',')})`;
// 渐变扫光跟随**状态相位**（不再用 .65/.85 两条独立频率），与自转同节拍，避免看起来随机。
function tint(mark,colors,phase=0,moving=false){
  mark.stops.forEach((stop,i)=>stop.setAttribute('stop-color',colorText(colors[i])));
  const shift=moving?Math.sin(phase)*3:0;
  mark.gradient.setAttribute('x1',String(8+shift));mark.gradient.setAttribute('x2',String(16-shift));
  mark.stops[1].setAttribute('offset',String(.5+(moving?Math.sin(phase)*.14:0)));
}

function el(tag,attrs={}){const n=document.createElementNS(NS,tag);for(const [k,v] of Object.entries(attrs))n.setAttribute(k,v);return n}
// 布局中心取原云形包围盒中心；节点圆周、连接和旋转共用一个中心。
const probe=el('svg',{width:0,height:0,'aria-hidden':'true'}),outline=el('path',{d:LOGO_GEOMETRY.body});
probe.style.position='absolute';probe.append(outline);document.body.append(probe);
const bounds=outline.getBBox();
const CENTER={x:bounds.x+bounds.width/2,y:bounds.y+bounds.height/2};probe.remove();
let serial=0;
function createMark(host){
 const uid='mark-'+serial++,svg=el('svg',{viewBox:'0 0 24 24','aria-hidden':'true'}),defs=el('defs');
 svg.style.transformOrigin=`${CENTER.x/24*100}% ${CENTER.y/24*100}%`;
 const gradient=el('linearGradient',{id:uid+'-g',gradientUnits:'userSpaceOnUse',x1:12,x2:12,y1:3,y2:21});
 BRAND.forEach((color,i)=>gradient.append(el('stop',{offset:i/2,'stop-color':color})));
 const mask=el('mask',{id:uid+'-m',maskUnits:'userSpaceOnUse',x:-4,y:-4,width:32,height:32});
 mask.append(el('rect',{x:-4,y:-4,width:32,height:32,fill:'white'}));
 const core=el('g',{'data-layer':'core'}),originalCore=el('g',{'data-layer':'original',transform:'translate(7.5 7.5) scale(0.0087890625)',display:'none'});
  const links=Array.from({length:3},()=>el('path',{fill:'none',stroke:'black','stroke-width':.78,'stroke-linecap':'round','stroke-linejoin':'round'}));
 const nodes=Array.from({length:3},()=>el('circle',{r:1.18,fill:'black'}));
 core.append(...links,...nodes);LOGO_GEOMETRY.holes.forEach(d=>originalCore.append(el('path',{d,fill:'black'})));
 mask.append(core,originalCore);
 const body=el('path',{d:LOGO_GEOMETRY.body,fill:`url(#${uid}-g)`,mask:`url(#${uid}-m)`});
 defs.append(gradient,mask);svg.append(defs,body);host.append(svg);
 return {svg,core,originalCore,gradient,stops:[...gradient.children],links,nodes,body};
}
function paint(mark,shape,amount=1){
 const key=shape.join(',')+':'+amount;if(mark.geometryKey===key)return;mark.geometryKey=key;
 const [spread,connection,bendSin,bendCos]=shape,radius=3.5+spread*amount;
 const points=Array.from({length:3},(_,i)=>{const a=-Math.PI/2+i*Math.PI*2/3;return {a,x:CENTER.x+radius*Math.cos(a),y:CENTER.y+radius*Math.sin(a)}});
 points.forEach((p,i)=>{mark.nodes[i].setAttribute('cx',p.x);mark.nodes[i].setAttribute('cy',p.y)});
 // 120° 圆弧的三次贝塞尔近似。曲线延伸至圆心、被圆节点覆盖，没有短线碎片。
 const tangent=4/3*Math.tan(Math.PI/6)*radius;
 points.forEach((p,i)=>{
  const q=points[(i+1)%3],bend=amount*(bendSin*Math.cos(i*Math.PI*2/3)+bendCos*Math.sin(i*Math.PI*2/3));
  const mid=p.a+Math.PI/3,bx=Math.cos(mid)*bend,by=Math.sin(mid)*bend;
  const d=`M${p.x} ${p.y} C${p.x-Math.sin(p.a)*tangent+bx} ${p.y+Math.cos(p.a)*tangent+by} ${q.x+Math.sin(q.a)*tangent+bx} ${q.y-Math.cos(q.a)*tangent+by} ${q.x} ${q.y}`;
  mark.links[i].setAttribute('d',d);mark.links[i].setAttribute('opacity',Math.max(0,Math.min(1,connection)));
 });
}
const hero=createMark($('hero')),mini=createMark($('mini'));
STATES.forEach((s,i)=>{
  const button=document.createElement('button');button.className='tile pctl-tile';button.dataset.state=s.id;button.setAttribute('aria-pressed','false');
  const mark=createMark(button);paint(mark,s.shape);tint(mark,paletteOf(i));
  if(s.id==='stale')mark.svg.style.filter='saturate(.15)';
  const b=document.createElement('b');b.textContent=s.name;const small=document.createElement('span');small.textContent=s.short;
  button.append(b,small);button.onclick=()=>{stopDemo();setState(i)};$('states').append(button);
});
let selected=1,frame=0,current=[...STATES[1].shape],from=[...current],transitionStart=0,clock=0,lastTime=0;
let paused=false,original=false,strength=1.25,demoTimer=null,demoStep=0;
// calm：「关于」页小图标用——保留内结构形变与配色过渡，去掉整体漂浮/自转与过期淡出，避免图标在 40px 框里「挪出去进来」。
let calm=false;
let currentColors=paletteOf(1),fromColors=currentColors.map(c=>[...c]);
const media=matchMedia('(prefers-reduced-motion: reduce)');
// 特效档（父页 iframe 协议）：mid → 光场静止；low → 无光场 + 静态 logo（都复用 reduced 路径：
// 它会**直接落到当前状态的形状**并重绘，所以低档仍保留一个「静态多状态 logo」，只是不播动画）。
const fxMode=(new URLSearchParams(location.search).get('effects')||'').toLowerCase();
const reduced=()=>fxMode==='mid'||fxMode==='low'||media.matches||$('reduce').checked;
// 背景光渲染：WEBGL mesh 渐变 + 颗粒着色器（默认）；挂载失败或不支持时自动回退 CSS 极光。
let meshGlow=null;
function mountMeshGlow(){
  if(!document.documentElement.classList.contains('glow-mesh'))return;
  const host=$('stage'),api=window.OpenCodexMeshGlow;
  if(!host||!api||!api.supported()){document.documentElement.classList.remove('glow-mesh');return}
  const ctrl=api.mount(host,{dark:document.body.classList.contains('dark')});
  if(!ctrl.ok){document.documentElement.classList.remove('glow-mesh');return}
  const canvas=host.querySelector('.mesh-glow-canvas');
  if(canvas)host.insertBefore(canvas,host.firstChild);
  ctrl.setReduced(reduced());
  meshGlow=ctrl;
  draw();
}
const lerp=(a,b,t)=>a.map((v,i)=>v+(b[i]-v)*t);
// 所有状态循环，状态语义由连接基态与文字保持；不按时间推进业务结论。
const EFFECTS={
 not_ready:{outer:0,inner:0,rock:18,period:7,energy:.25,title:'B · 离散漂浮',body:'轻缓上下漂浮',core:'三球分离、低速探寻',light:'雾蓝散光'},
 stopped:{outer:0,inner:0,rock:7,period:6,energy:.20,title:'B · 低位待命',body:'较低悬浮位置、小幅起伏',core:'三球待命、无连接线',light:'柔紫光晕与近地投影'},
 starting:{outer:30,inner:-120,rock:0,period:3.2,energy:1,title:'B → C · 轻跃蓄能',body:'小幅跃起＋整体慢转',core:'柔性连接建立、内核反转',light:'蓝紫聚光、投影随高度变化'},
 running:{outer:0,inner:60,rock:0,period:6,energy:.75,title:'A · 稳定悬浮',body:'平稳悬浮、不改变比例',core:'完整内核固定形状 · 6s/圈',light:'青蓝流光与柔和投影'},
 stopping:{outer:0,inner:0,rock:6,period:5,energy:.35,title:'A → C → B · 下沉',body:'缓慢下沉后低位浮动',core:'减速、曲线柔和退去',light:'暮蓝收束、投影聚拢'},
 failed:{outer:0,inner:0,rock:18,period:5,energy:.40,title:'C → B · 回落',body:'一次短促回落、随后低浮',core:'连接退去、三球回摆',light:'珊瑚柔光，无闪烁'},
 problem:{outer:0,inner:34,rock:0,period:5,energy:.55,title:'C · 柔性牵动',body:'轻微横向浮动',core:'连续曲线轻弯、保持连接',light:'琥珀局部流光'},
 confirming:{outer:12,inner:0,rock:65,period:4.5,energy:.55,title:'B · 悬浮探寻',body:'缓慢横向漂移',core:'分离三球、双向寻位',light:'冰蓝往返光场'},
 stale:{outer:0,inner:0,rock:0,period:9,energy:.15,title:'旧结构 · 失重漂移',body:'低幅漂浮、去饱和',core:'保留旧结构、平滑减速停住',light:'雾灰残光'}
};
let floatPose=[0,0],floatFrom=[0,0],staleShape=[0,1,0,0];
// 悬浮纵向带「重力感」：上升慢、下落快（相位非线性，波形仍连续可导、周期不变）。
// 注意纵向位移取 +1 = 屏幕下方，故下降段用「相位推进更快」的一侧。
function gravity(phase){return Math.sin(phase+.45*Math.sin(phase))}
function floatTarget(id,time,age){
 const phase=motionPhase,g=gravity(phase),gc=gravity(phase+Math.PI/2);
 switch(id){
 case 'starting':return [.30*Math.sin(phase),-.30-1.05*Math.pow((1-Math.cos(phase))/2,2)];
 case 'running':return [.30*Math.sin(phase),-.55+1.00*g];
 case 'stopping':return [0,.65+.35*g];
 case 'failed':return [.26*Math.sin(phase),.60+.40*g+.35*Math.exp(-Math.max(0,age)/.6)];
 case 'not_ready':return [.60*Math.sin(phase),-.10+1.10*gc];
 case 'stopped':return [0,.65+.50*g];
 case 'problem':return [.60*Math.sin(phase),-.25+.60*gravity(phase*2)];
 case 'confirming':return [1.00*Math.sin(phase),-.15+.60*gc];
 default:return [.30*Math.sin(phase),.45*gc];
 }
}
let outerAngle=0,innerAngle=0,outerVelocity=0,innerVelocity=0;
function rotateStep(dt){
  const id=STATES[selected].id,e=EFFECTS[id],age=(clock-transitionStart)/1000;
  
  const phase=motionPhase;
  let inside=e.inner+e.rock*(Math.PI*2/e.period)*Math.cos(phase);
  if(id==='starting')inside*=.78+.22*Math.sin(phase);
  if(id==='problem')inside*=.65+.35*Math.sin(phase);
  if(id==='stopping')inside+=50*Math.exp(-Math.max(0,age)/1.3);
  // 无整体自转的状态平滑回到最近的品牌正向角度。
  const error=((outerAngle+180)%360+360)%360-180;
  const outside=id==='stale'?0:(e.outer||-error*2.5);
  const smooth=1-Math.exp(-dt*5);
  outerVelocity+=(outside-outerVelocity)*smooth;innerVelocity+=(inside-innerVelocity)*smooth;
  outerAngle+=outerVelocity*dt;innerAngle+=innerVelocity*dt;
}
function liveShape(s,t){
 if(s.id==='stale')return [...staleShape];
 const p=[...s.shape];if(reduced()||s.id==='running')return p;
 const phase=motionPhase;
 if(s.id==='starting'){p[0]+=.10*(1+Math.sin(phase));p[1]=.62+.16*Math.sin(phase);p[2]=.16*Math.sin(phase);p[3]=.16*Math.cos(phase);}
 else if(s.id==='problem'){p[2]=.22*Math.sin(phase);p[3]=.22*Math.cos(phase);}
 else if(s.id==='not_ready'||s.id==='confirming')p[0]+=.10*Math.sin(phase);
 return p;
}
function flowing(){return $('flow').checked&&!reduced()&&!original}
function atmosphereMoving(){return $('fx').checked&&!reduced()&&!original}

// 连续相位 + 保留位置/速度的临界阻尼追踪。切态只换目标，不重置画面或速度。
let motionPhase=0,phaseSpeed=Math.PI*2/6;
const tracks=new Map();
function track(key,values,target,dt,omega=9){
 let v=tracks.get(key);if(!v){v=values.map(()=>0);tracks.set(key,v)}
 return values.map((x,i)=>{
  const offset=x-target[i],j=v[i]+omega*offset,decay=Math.exp(-omega*dt);
  const next=target[i]+(offset+j*dt)*decay;
  v[i]=(v[i]-omega*j*dt)*decay;
  if(Math.abs(next-target[i])<1e-7&&Math.abs(v[i])<1e-6){v[i]=0;return target[i]}
  return next;
 });
}
// 光场三条不同节奏的叠加（用户反馈：要看出「绕中心公转」「各自自旋」「涨缩有先有后」）：
//  1) 公转 ORBIT   —— 三团相隔 120° 绕中心 logo 转，角速度 = 状态相位的一半（运行态约 12s 一圈）；
//  2) 自旋 SPIN    —— 每个光源各有自己的转向与转速（光源形状是不规则椭圆，转起来才看得出来）；
//  3) 涨缩 BREATHE —— 共用同一角速度、但时差 120°，先后放大缩小，不同时。
const ORBIT_RATE=.5, SPIN=[42,-33,54], BREATHE_LAG=Math.PI*2/3;
function ambientTarget(){
 const effect=EFFECTS[STATES[selected].id],phase=motionPhase,dark=document.body.classList.contains('dark');
 const clouds=Array.from({length:3},(_,i)=>{
  const orbit=phase*ORBIT_RATE+i*BREATHE_LAG, rx=50+effect.energy*70, ry=34+effect.energy*48;
  const lag=phase+i*BREATHE_LAG;
  return [Math.cos(orbit)*rx,Math.sin(orbit)*ry,
   1+Math.sin(lag)*.22,(dark?.44:.34)+Math.sin(lag)*.07,phase*SPIN[i]];
 }).flat();
 return [...clouds,.9+effect.energy*.2+Math.sin(phase)*.08,(.10+effect.energy*.20)*(1+Math.sin(phase)*.14)];
}
let ambient=ambientTarget(),beamAngle=0,beamVelocity=0,appearance=[1,1];
function draw(){
 document.body.classList.toggle('original',original);
 const colors=original?BRAND.map(rgb):currentColors,time=clock/1000;
 for(const host of [$('stage'),$('mini-stage')]){
  ['a','b','c'].forEach((key,i)=>host.style.setProperty('--glow-'+key,colorText(colors[i])));
  const beam=host.querySelector('.energy');
  if(beam){beam.style.transform=`rotate(${beamAngle}deg) scale(${ambient[15]})`;beam.style.opacity=String(ambient[16])}
  const floor=host.querySelector('.light-floor');
  if(floor){const height=!original&&!reduced()&&!calm?floatPose[1]:0;floor.style.transform=`scaleX(${1-height*.13})`;floor.style.opacity=String(.20+height*.06);floor.style.filter=`blur(${host.id==='mini-stage'?4:11-height*2}px)`}
  host.querySelectorAll('.cloud').forEach((cloud,i)=>{
   const [x,y,scale,opacity,rot]=ambient.slice(i*5,i*5+5),size=host.id==='mini-stage'?.32:1;
   cloud.style.transform=`translate(${x*size}px,${y*size}px) scale(${scale}) rotate(${rot}deg)`;cloud.style.opacity=String(opacity);
  });
 }
 // 着色器层与 CSS 极光共用同一套状态投影配色（--glow-a/b/c）。
 if(meshGlow){const st=$('stage');if(st)meshGlow.setColors(['a','b','c'].map(k=>st.style.getPropertyValue('--glow-'+k)||'#B1A7FF'))}
 for(const mark of [hero,mini]){
  paint(mark,current,strength);
  mark.core.setAttribute('display',original?'none':'inline');mark.originalCore.setAttribute('display',original?'inline':'none');
  tint(mark,colors,motionPhase,flowing());
  const moving=!original&&!reduced();
  const outer=moving&&!calm, drift=outer?floatPose:[0,0];
  mark.svg.style.transform=`translate(${drift[0]/24*100}%,${drift[1]/24*100}%) rotate(${outer?outerAngle:0}deg)`;
  mark.core.setAttribute('transform',`rotate(${moving?innerAngle:0} ${CENTER.x} ${CENTER.y})`);
  if(original){mark.gradient.setAttribute('x1','12');mark.gradient.setAttribute('x2','12')}
  mark.svg.style.filter=`saturate(${original?1:appearance[0]})`;mark.svg.style.opacity=String(original?1:appearance[1]);
 }
}

function tick(now){
 frame=0;if(paused||document.hidden)return;
 const dt=lastTime?Math.min(now-lastTime,50)/1000:0;clock+=dt*1000;lastTime=now;
 const s=STATES[selected],age=(clock-transitionStart)/1000;
 if(!reduced()&&!original){
  phaseSpeed+=(Math.PI*2/EFFECTS[s.id].period-phaseSpeed)*(1-Math.exp(-dt*5));
  motionPhase+=phaseSpeed*dt;rotateStep(dt);
  let target=liveShape(s,clock/1000);
  if(s.id==='stopping'&&age<1)target[2]+=.12*Math.sin(Math.PI*age)**2;
  current=track('shape',current,target,dt);
  floatPose=track('float',floatPose,floatTarget(s.id,clock/1000,age),dt,8);
  currentColors=currentColors.map((c,i)=>track('color'+i,c,paletteOf(selected)[i],dt,8));
  appearance=track('appearance',appearance,s.id==='stale'&&!calm?[.15,.62]:[1,1],dt,8);
  if(atmosphereMoving()){
   ambient=track('ambient',ambient,ambientTarget(),dt,7);
   const targetVelocity=s.id==='confirming'?65*phaseSpeed*Math.cos(motionPhase):innerVelocity*.55;
   beamVelocity+=(targetVelocity-beamVelocity)*(1-Math.exp(-dt*5));beamAngle+=beamVelocity*dt;
  }
 }
 draw();if(!reduced()&&!original)frame=requestAnimationFrame(tick);
}

function wake(){lastTime=0;if(!frame&&!paused&&!document.hidden)frame=requestAnimationFrame(tick)}
function setState(i,immediate=false){
  if(STATES[i].id==='stale'&&STATES[selected].id!=='stale')staleShape=[...current];
  from=[...current];floatFrom=[...floatPose];fromColors=currentColors.map(c=>[...c]);selected=i;transitionStart=clock;
  const s=STATES[i];
  $('label').textContent=s.name;$('mini-label').textContent=s.name;
  $('sub').textContent=`主线：${s.main}${s.op==='—'?'':` · ${s.op}`}`;
  $('index').textContent=`${String(i+1).padStart(2,'0')} / 09`;
  $('verb').textContent=s.verb;$('meaning').textContent=s.meaning;
  $('mainline').textContent=s.main;$('operation').textContent=s.op;
  $('freshness').textContent=s.id==='stale'?'已过期':s.id==='confirming'?'尚无观测':'新鲜（模拟）';
  $('issue').textContent=s.note;
  $('motion-note').textContent=reduced()?'减少动态 · 静态展示':EFFECTS[s.id].title+' · 持续循环';
  $('effect-body').textContent=EFFECTS[s.id].body;$('effect-core').textContent=EFFECTS[s.id].core;$('effect-light').textContent=EFFECTS[s.id].light; 
  $('palette-label').textContent=(PALETTE_OVERRIDES[paletteHint]?.name)||PALETTES[s.id].name;
  document.querySelectorAll('.tile').forEach(b=>b.setAttribute('aria-pressed',String(b.dataset.state===s.id)));
  if(immediate||reduced()){
   current=s.id==='stale'?[...staleShape]:[...s.shape];currentColors=paletteOf(i);
   appearance=s.id==='stale'&&!calm?[.15,.62]:[1,1];ambient=ambientTarget();tracks.clear();draw();
  }
  // 暂停时仅更新目标与文字，画面保留；继续后再平滑趋近。
  wake();
}
function stopDemo(){clearTimeout(demoTimer);demoTimer=null;$('demo').textContent='播放启停场景';$('timeline').textContent='可直接点选任意状态，观察连续形变'}
const sequence=[{i:1,ms:1800},{i:2,ms:3600},{i:3,ms:2600},{i:6,ms:2400},{i:3,ms:1800},{i:4,ms:2200},{i:1,ms:1800}];
function advanceDemo(){
  if(paused||document.hidden){demoTimer=setTimeout(advanceDemo,300);return}
  if(demoStep>=sequence.length){stopDemo();$('timeline').textContent='场景结束 · 已核验停止（模拟）';return}
  const step=sequence[demoStep++];setState(step.i);$('timeline').textContent=`模拟场景 ${demoStep} / ${sequence.length} · 定时步骤仅用于演示`;
  demoTimer=setTimeout(advanceDemo,step.ms);
}
$('demo').onclick=()=>{
  if(demoTimer){stopDemo();return}
  paused=false;$('pause').textContent='暂停动画';$('pause').setAttribute('aria-pressed','false');
  original=false;$('original').setAttribute('aria-pressed','false');$('original').textContent='对照原始 Logo';
  demoStep=0;$('demo').textContent='结束场景';advanceDemo();
};
$('pause').onclick=()=>{paused=!paused;$('pause').setAttribute('aria-pressed',String(paused));$('pause').textContent=paused?'继续动画':'暂停动画';if(paused){cancelAnimationFrame(frame);frame=0}else wake()};
$('replay').onclick=()=>{stopDemo();original=false;$('original').setAttribute('aria-pressed','false');$('original').textContent='对照原始 Logo';const i=selected;current=[...(i===1||i===4?STATES[3]:STATES[1]).shape];setState(i)};
$('original').onclick=()=>{original=!original;$('original').setAttribute('aria-pressed',String(original));$('original').textContent=original?'返回形变效果':'对照原始 Logo';draw();wake()};
$('theme').onclick=()=>{const dark=document.body.classList.toggle('dark');$('theme').setAttribute('aria-pressed',String(dark));$('theme').textContent=dark?'浅色':'深色';if(meshGlow)meshGlow.setTheme(dark)};
$('fx').onchange=()=>{document.body.classList.toggle('no-fx',!$('fx').checked);draw();wake()};
$('flow').onchange=()=>{draw();wake()};
$('strength').oninput=e=>{strength=Number(e.target.value);draw()};
$('reduce').onchange=()=>{setState(selected,true);if(meshGlow)meshGlow.setReduced(reduced())};
media.addEventListener('change',()=>{setState(selected,true);if(meshGlow)meshGlow.setReduced(reduced())});
document.addEventListener('visibilitychange',()=>{if(document.hidden){cancelAnimationFrame(frame);frame=0;lastTime=0}else wake()});
$('shuffle').onclick=()=>{stopDemo();setState((selected+1+Math.floor(Math.random()*(STATES.length-1)))%STATES.length)};
setState(1,true);
mountMeshGlow();

// 候选嵌入协议：仅接收直接父窗口，file 预览允许 opaque origin。
const embedParams=new URLSearchParams(location.search);
if(embedParams.has('embed')){
 // cycle=1：「关于」页本应用图标——按顺序循环全部 9 个状态，套用同一套形变/取色插值（不新增效果、无背景光）。
 calm=embedParams.has('calm');
 if(embedParams.has('cycle')){
  setInterval(()=>{if(!paused&&!document.hidden)setState((selected+1)%STATES.length)},2600);
 }
 const receive=e=>{
  if(e.source!==parent||e.data?.type!=='opencodex-motion-preview')return;
  if(location.protocol!=='file:'&&e.origin!==location.origin)return;
  const msg=e.data,index=STATES.findIndex(s=>s.id===msg.state);
  paletteHint=typeof msg.palette==='string'?msg.palette:'';
  // 原型概览只调整装饰层尺寸，内部形变幅度和状态来源保持原值。
  if(msg.presentation&&typeof msg.presentation==='object'){
   const {size,padding}=msg.presentation;
   if(Number.isFinite(size))document.documentElement.style.setProperty('--embedded-mark-size',Math.max(64,Math.min(140,size))+'px');
   if(Number.isFinite(padding))document.documentElement.style.setProperty('--embedded-padding',Math.max(24,Math.min(120,padding))+'px');
  }
  document.body.classList.toggle('dark',msg.theme==='dark');
  if(meshGlow)meshGlow.setTheme(msg.theme==='dark');
  // 嵌入层始终透明；主题只改变 Logo 与光效，不注入独立底色。
  document.body.style.setProperty('--card','transparent');
  if(index>=0&&index!==selected)setState(index);
  if(typeof msg.active==='boolean'){
   paused=!msg.active;
   if(paused){cancelAnimationFrame(frame);frame=0;lastTime=0;if(meshGlow)meshGlow.pause()}else{wake();if(meshGlow)meshGlow.resume()}
  }
 };
 addEventListener('message',receive);
 parent.postMessage({type:'opencodex-motion-ready'},location.protocol==='file:'?'*':location.origin);
}
