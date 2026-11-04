/* 概览主体「流量全景」：验证 Desktop 主角已回写正式概览，连接地图回到三视图。 */
const test=require('node:test'),assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');
async function setup(t,hash='#overview'){const h=await make(hash);t.after(()=>{try{assert.deepEqual(h.errors,[]);assert.deepEqual(h.network,[]);}finally{h.dispose();}});return h;}
const hero=h=>h.document.querySelector('#ovHero');
const num=(h,id)=>h.document.getElementById(id)?.getAttribute('data-annotation-number');

test('流量全景紧跟状态行，是概览主体且不提前加载动效',async t=>{
 const h=await setup(t),d=h.document;
 assert.ok(hero(h));assert.equal(hero(h).hidden,false);
 assert.equal(d.querySelector('#card-overview-status').nextElementSibling,hero(h));
 for(const f of ['krill','newapi','a6','desktop','adapter'])assert.ok(hero(h).querySelector('[data-f="'+f+'"]'),f);
 assert.doesNotMatch(hero(h).textContent,/Aimami/);
 assert.equal(d.querySelector('#phMotion').getAttribute('src'),null);
 assert.match(d.querySelector('#routeSubtitle').textContent,/过去 1 天/);
 for(const s of ['[data-act=start]','#mods','#card-overview-events'])assert.ok(d.querySelector(s),s);
 assert.equal(num(h,'ann-ov-hero'),'01');assert.equal(num(h,'ann-ov-status'),'02');
});

test('网关未运行时全景静止，运行后恢复流动',async t=>{
 const h=await setup(t);
 h.w.eval("setState('stopped')");await h.settle();
 assert.ok(hero(h).classList.contains('ov-idle'));assert.equal(h.w.overviewHeroPrototype.state().state,'stopped');
 h.w.eval("setState('running')");await h.settle();
 assert.equal(hero(h).classList.contains('ov-idle'),false);
});

test('舞台直挂概览主体，工具栏留在标题行内',async t=>{
 const h=await setup(t),d=h.document,stage=d.querySelector('#phStage'),tools=d.querySelector('.ph-tools');
 assert.ok(stage&&tools);
 /* 曾因删按钮时误删分段组的闭合标签，舞台被塞进标题行，缩成右上角一小块 */
 assert.equal(tools.parentElement,hero(h).querySelector('.ov-hero-head'),'工具栏挂在标题行内');
 assert.equal(stage.parentElement,hero(h),'舞台直接挂在 hero 下，不能嵌进标题行');
 assert.equal(hero(h).firstElementChild.classList.contains('ov-hero-head'),true);
 assert.equal(tools.querySelector('[data-ph-pause]'),null,'暂停按钮已移除');
 /* jsdom 无布局，这里只锁结构：舞台直接是 hero 子级，且不在标题行内 */
 assert.equal(stage.closest('.ov-hero-head'),null,'舞台不能在标题行内');
 assert.equal(stage.querySelector('.ph-clients')!==null,true,'客户端列在舞台内');
 assert.equal(stage.querySelector('.ph-channels')!==null,true,'渠道列在舞台内');
});


test('环境未就绪时隐藏全景，恢复后重新显示',async t=>{
 const h=await setup(t);
 h.w.setEnv('missing_ocx');await h.settle();
 assert.equal(hero(h).hidden,true);assert.equal(h.w.overviewHeroPrototype.state().mode,'setup');
 h.w.setEnv('ready');await h.settle();
 assert.equal(hero(h).hidden,false);
});

test('共享入口的三个客户端各自成为弹窗锚点，切页清理主内容区浮层',async t=>{
 const h=await setup(t),d=h.document;
 h.w.eval("setState('running')");await h.settle();
 const clients=[...d.querySelectorAll('.ph-client[data-f="codex"]')];
 assert.equal(clients.length,3);
 for(const node of clients){
  node.focus();await h.settle();
  assert.equal(node.getAttribute('aria-expanded'),'true');
  for(const other of clients.filter(n=>n!==node))assert.equal(other.getAttribute('aria-expanded'),'false');
 }
 const pop=d.querySelector('#phPop');
 assert.equal(pop.parentElement,d.querySelector('.main'));
 assert.equal(pop.classList.contains('ph-visible'),true);
 h.click('.nav button[data-route="models"]');await h.settle();
 assert.equal(pop.classList.contains('ph-visible'),false);
 assert.equal(h.w.overviewHeroPrototype.state().focus,null);
});

test('连接地图回到三个视图，不再挂主角页签',async t=>{
 const h=await setup(t,'#connection-map'),d=h.document;
 assert.equal(d.querySelectorAll('#connectionMapPage [role=tab]').length,3);
 assert.equal(d.querySelector('#pvHero'),null);
});

test('节点到弹窗可连续悬停，弹窗聚焦与选字拖动保持，离开两者后关闭',async t=>{
 const h=await setup(t),d=h.document,node=d.querySelector('.ph-client'),pop=d.querySelector('#phPop'),stage=d.querySelector('#phStage');
 h.w.eval("setState('running')");await h.settle();
 const fire=(el,type,extra={})=>el.dispatchEvent(new h.w.MouseEvent(type,{bubbles:false,...extra}));
 node.focus();
 fire(stage,'pointerleave');await h.wait(60);
 assert.equal(pop.classList.contains('ph-visible'),true,'经过间隙不立即关闭');
 fire(pop,'pointerenter');await h.wait(260);
 assert.equal(pop.classList.contains('ph-visible'),true,'进入弹窗取消关闭计时');
 const button=pop.querySelector('button');button.focus();fire(pop,'pointerleave');await h.wait(260);
 assert.equal(pop.classList.contains('ph-visible'),true,'键盘焦点位于弹窗时保持');
 button.blur();await h.wait(260);
 assert.equal(pop.classList.contains('ph-visible'),false,'焦点与指针都离开后关闭');
 node.focus();fire(pop,'pointerenter');fire(pop,'pointerdown',{button:0});fire(pop,'pointerleave');await h.wait(260);
 assert.equal(pop.classList.contains('ph-visible'),true,'拖动选字越过弹窗边缘仍保持');
 fire(d,'pointerup');await h.wait(260);
 assert.equal(pop.classList.contains('ph-visible'),false,'结束拖动且位于区域外后关闭');
});
