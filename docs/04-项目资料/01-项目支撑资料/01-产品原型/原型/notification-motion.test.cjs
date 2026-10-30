/* 铃铛收起与补位的生命周期检查；实际曲线/布局另由浏览器验证。 */
const test=require('node:test'),assert=require('node:assert/strict');
const {make}=require('./component-topic-harness.cjs');
async function setup(t){const h=await make('#overview');t.after(()=>{try{assert.deepEqual(h.errors,[]);assert.deepEqual(h.network,[]);}finally{h.dispose();}});return h;}
function cards(h){return [...h.document.querySelectorAll('[data-task-notification^="component-demo:overview:"]')];}
test('概览三条通知先注册再由原宿主展示，收起不改变通知实体且下一条保留',async t=>{
 const h=await setup(t);h.click('[data-overview-notification=stack]');const before=cards(h);assert.equal(before.length,3);const first=h.w.eval('notifications.find(n=>n.id==="'+before[0].dataset.taskNotification+'")'),flags=JSON.stringify([first.read,first.resolved,first.deleted]);
 h.click('[data-task-notification="'+first.id+'"] [data-collapse=task]');assert.equal(h.w.NotificationMotion.state(before[0]),'collapsing');assert.equal(before[0].isConnected,true);assert.equal(JSON.stringify([first.read,first.resolved,first.deleted]),flags);
 await h.wait(h.w.NotificationMotion.durationMs+40);assert.equal(before[0].isConnected,false);assert.equal(cards(h).length,2);assert.equal(JSON.stringify([first.read,first.resolved,first.deleted]),flags);h.click('#notificationBtn');h.click('[data-notification="'+first.id+'"]');assert.equal(cards(h).length,3);
});
test('并行收起各自完成，不再用一个全局timeout互相取消',async t=>{
 const h=await setup(t);h.click('[data-overview-notification=stack]');const before=cards(h);h.click('[data-task-notification="'+before[0].dataset.taskNotification+'"] [data-collapse=task]');h.click('[data-task-notification="'+before[1].dataset.taskNotification+'"] [data-collapse=task]');await h.wait(h.w.NotificationMotion.durationMs+40);assert.equal(cards(h).length,1);assert.equal(before[2].isConnected,true);
});
test('新Toast替换正在收起的旧Toast会取消旧回调，不提前隐藏新内容',async t=>{
 const h=await setup(t);h.w.toast('旧',{kind:'success',duration:8000});const el=h.document.querySelector('#toastEl');h.click('#toastEl [data-collapse=toast]');assert.equal(h.w.NotificationMotion.state(el),'collapsing');h.w.toast('新',{kind:'warning',duration:8000});assert.equal(h.w.NotificationMotion.state(el),'idle');await h.wait(h.w.NotificationMotion.durationMs+40);assert.equal(el.classList.contains('show'),true);assert.match(el.textContent,/新/);assert.equal(el.querySelectorAll('.notification-countdown').length,1);
});
test('减少动态立即收起，不吸入/补位动画也不改变已读与解决',async t=>{
 const h=await setup(t);h.w.matchMedia=()=>({matches:true});h.click('[data-overview-notification=stack]');const first=cards(h)[0],id=first.dataset.taskNotification;h.click('[data-task-notification="'+id+'"] [data-collapse=task]');assert.equal(first.isConnected,false);assert.equal(cards(h).length,2);assert.ok(h.w.eval('notifications.some(n=>n.id==="'+id+'"&&!n.deleted)'));
});
test('重置只清概览模拟并取消未到达项，不移除真实同步通知',async t=>{
 const h=await setup(t);h.w.prototypeNotificationPublisher.publish('sync.complete',{operationId:'999',total:1});h.click('[data-overview-notification=staggered]');assert.equal(cards(h).length,1);h.click('[data-overview-notification=reset]');await h.wait(2300);assert.equal(cards(h).length,0);assert.ok(h.w.eval('notifications.some(n=>n.id==="sync-demo:999"&&!n.deleted)'));
});
test('模拟只接受登记场景，已删除示例不因再次播放复活，另两条仍可观察',async t=>{
 const h=await setup(t);assert.throws(()=>h.w.prototypeNotificationPublisher.publish('overview.demo.notification',{scene:'arbitrary'}));assert.throws(()=>h.w.prototypeNotificationPublisher.publish('overview.demo.notification',{scene:'single',title:'bad'}));h.click('[data-overview-notification=stack]');const id=cards(h)[0].dataset.taskNotification;h.w.eval('notifications.find(n=>n.id==="'+id+'").deleted=true');h.w.clearTask(id);h.click('[data-overview-notification=stack]');assert.equal(cards(h).length,2);assert.equal(h.w.eval('notifications.filter(n=>n.id==="'+id+'").length'),1);
});
test('任务在收起中更新为新状态时旧动画取消，不能删除更新后的同一身份任务',async t=>{
 const h=await setup(t);h.w.showTask({notificationId:'race',title:'旧结果',running:false,kind:'info',steps:[],acts:[]});const old=h.document.querySelector('[data-task-notification=race]');h.click('[data-task-notification=race] [data-collapse=task]');h.w.showTask({notificationId:'race',title:'新任务',running:true,kind:'info',steps:[],acts:[]});assert.equal(h.w.NotificationMotion.state(old),'idle');await h.wait(h.w.NotificationMotion.durationMs+40);const next=h.document.querySelector('[data-task-notification=race]');assert.match(next.textContent,/新任务/);assert.equal(next.querySelector('.notification-countdown'),null);
});
test('通知面板使用相同收起动画，重新打开不会被旧的延迟完成关掉',async t=>{
 const h=await setup(t);h.click('#notificationBtn');const panel=h.document.querySelector('#notificationPanel');h.click('#notificationPanel [data-collapse=panel]');assert.equal(h.w.NotificationMotion.state(panel),'collapsing');h.click('#notificationBtn');h.click('#notificationBtn');await h.wait(h.w.NotificationMotion.durationMs+40);assert.equal(h.w.NotificationMotion.state(panel),'idle');assert.equal(panel.classList.contains('show'),true);
});
test('收起表面被外部移除时撤销结束回调，不会在稍后改写其它内容',async t=>{
 const h=await setup(t);h.click('[data-overview-notification=single]');const card=cards(h)[0];let done=0;const result=h.w.NotificationMotion.collapse(card,{onDone:()=>done++});card.remove();await h.settle();assert.equal(h.w.NotificationMotion.state(card),'idle');assert.equal(await result,false);await h.wait(h.w.NotificationMotion.durationMs+40);assert.equal(done,0);
});
test('动画关键帧收窄并进入铃铛，目标移除后存量卡片有向上补位关键帧',async t=>{
 const h=await setup(t);h.click('[data-overview-notification=stack]');const [first,next]=cards(h),record=[];
 first.getBoundingClientRect=()=>({left:400,top:200,right:792,bottom:290,width:392,height:90});
 next.getBoundingClientRect=()=>({left:400,top:first.isConnected?300:200,right:792,bottom:first.isConnected?390:290,width:392,height:90});
 let finish;first.animate=(frames,opts)=>{record.push({frames,opts});return {finished:new Promise(r=>finish=r),cancel(){}};};
 next.animate=(frames,opts)=>{record.push({frames,opts});return {finished:Promise.resolve(),cancel(){}};};
 h.click('[data-task-notification="'+first.dataset.taskNotification+'"] [data-collapse=task]');assert.equal(record[0].opts.duration,420);assert.ok(record[0].frames[2].clipPath.includes('34%'));assert.equal(record[0].frames.at(-1).opacity,0);finish();await h.settle();assert.equal(first.isConnected,false);assert.equal(record[1].opts.duration,240);assert.equal(record[1].frames[0].transform,'translateY(100px)');assert.equal(record[1].frames[1].transform,'translateY(0)');
});
