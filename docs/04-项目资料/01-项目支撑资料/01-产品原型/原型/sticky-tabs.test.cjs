/* 公共页面Tab注册与测量合同；不代替浏览器CSS sticky的视觉验证。 */
const {test}=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),path=require('node:path'),vm=require('node:vm');
function make(){
 const style=()=>{const values=new Map();return {setProperty:(k,v)=>values.set(k,v),getPropertyValue:k=>values.get(k)||'',values};};
 const main={style:style(),clientTop:1,top:0,scrollTop:0,padding:{paddingTop:'20px',paddingLeft:'22px',paddingRight:'30px'},getBoundingClientRect:()=>({top:main.top})},bars=[],frames=[],listeners={},mutation=[],observed=new Set();
 function bar(id,height=40,active=true,inner=false){const route={id,style:style(),active,classList:{contains:value=>value==='active'&&route.active}},tab={route,style:style(),height,top:0,isConnected:true,dataset:inner?{tabsScroll:'inner'}:{},closest:q=>q==='.main'?main:q==='.route-section'?route:null,getBoundingClientRect:()=>({height:tab.height,top:tab.top})};bars.push(tab);return tab;}
 const window={getComputedStyle:s=>s.padding,requestAnimationFrame:fn=>frames.push(fn),addEventListener:(name,fn)=>listeners[name]=fn};
 const context={window,document:{body:{},querySelectorAll:()=>bars.filter(b=>b.isConnected)},ResizeObserver:class{constructor(fn){this.fn=fn;}observe(node){observed.add(node);}unobserve(node){observed.delete(node);}},MutationObserver:class{constructor(fn){mutation.push(fn);}observe(){}},Set,Map,Promise,parseFloat};
 const api=()=>window.prototypeStickyTabs;
 const load=()=>vm.runInNewContext(fs.readFileSync(path.join(__dirname,'sticky-tabs.js'),'utf8'),context);
 const drain=()=>{let remaining=20;while(frames.length&&remaining-->0)frames.shift()();assert.ok(remaining>0);};
 return {main,bars,bar,load,api,listeners,mutation,observed,frames,drain};
}
test('六类页面主Tab注册，概览面板排除；背景边缘补偿和高度偏移不改其他状态',()=>{
 const h=make(),ids=['models','extensions','sync','logs','settings','conversations'];for(const id of ids)h.bar('route-'+id,40,true,id==='sync');const overview=h.bar('route-overview'),panel=h.bar('route-panel');h.load();
 assert.deepEqual(Array.from(h.api().state(),x=>x.route),ids.map(x=>'route-'+x));assert.equal(overview.style.values.size,0);assert.equal(panel.style.values.size,0);
 for(const b of h.bars.slice(0,6)){assert.equal(b.style.getPropertyValue('--page-tabs-left'),'22px');assert.equal(b.style.getPropertyValue('--page-tabs-right'),'30px');assert.equal(b.route.style.getPropertyValue('--page-tabs-height'),'40px');assert.equal(b.route.style.getPropertyValue('--page-tabs-offset'),'53px');}
 assert.equal(h.main.style.values.size,0);
});
test('动态重绘替换Tab、隐藏页重新进入和窗口变化均更新，不保留离线节点',()=>{
 const h=make(),old=h.bar('route-settings',40,false);h.load();assert.equal(old.style.values.size,0);old.route.active=true;h.mutation[0]();h.mutation[0]();assert.equal(h.frames.length,1);h.drain();assert.equal(old.route.style.getPropertyValue('--page-tabs-height'),'40px');
 old.isConnected=false;const fresh=h.bar('route-settings',76);h.main.padding.paddingTop='10px';h.listeners.resize();h.drain();assert.equal(h.observed.has(old),false);assert.equal(h.observed.has(fresh),true);assert.equal(fresh.style.getPropertyValue('--page-tabs-top'),'-10px');assert.equal(fresh.route.style.getPropertyValue('--page-tabs-height'),'76px');assert.equal(fresh.route.style.getPropertyValue('--page-tabs-offset'),'89px');
});
test('同步保留内部滚动，不改主容器；对话导航消费同一Tab高度与顶部偏移',()=>{
 const h=make(),sync=h.bar('route-sync',48,true,true),conversation=h.bar('route-conversations',48);h.main.style.setProperty('overflow','hidden');h.load();assert.equal(sync.style.getPropertyValue('--page-tabs-top'),'0px');assert.equal(conversation.route.style.getPropertyValue('--page-tabs-top'),'-20px');assert.equal(conversation.route.style.getPropertyValue('--page-tabs-height'),'48px');assert.equal(h.main.style.getPropertyValue('overflow'),'hidden');assert.equal(h.main.style.values.size,1);
 sync.height=80;conversation.height=80;h.api().refresh();assert.equal(sync.route.style.getPropertyValue('--page-tabs-height'),'80px');assert.equal(conversation.route.style.getPropertyValue('--page-tabs-height'),'80px');
});
test('未到顶不铺底色，到达吸附线才标记吸顶，回滚后取消',()=>{
 const h=make(),settings=h.bar('route-settings',40),models=h.bar('route-models',40,true,true);
 h.main.top=100;settings.top=400;models.top=400;h.load();
 // 吸附线 = 滚动容器可视顶边（含上边框）；负 sticky top 被浏览器夹在滚动口边缘，不再二次相加。
 const pin=h.main.top+h.main.clientTop;
 assert.equal(settings.dataset.stuck,'false','远离吸附线时保持透明');
 models.top=200;h.api().refresh();assert.equal(models.dataset.stuck,'false','内部滚动容器接近未到顶仍透明');
 settings.top=pin+1;h.api().refresh();assert.equal(settings.dataset.stuck,'false','接近但未到顶仍透明');
 settings.top=pin;h.api().refresh();assert.equal(settings.dataset.stuck,'true','到达吸附线标记吸顶');
 settings.top=pin-1;h.api().refresh();assert.equal(settings.dataset.stuck,'true','越过后保持吸顶');
 settings.top=200;h.api().refresh();assert.equal(settings.dataset.stuck,'false','回滚离开吸附线后取消');
 models.top=pin;h.api().refresh();assert.equal(models.dataset.stuck,'true','内部滚动容器吸附线为容器顶边');
 assert.equal(h.api().stuck().length,2);
});
