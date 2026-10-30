/* 时间与DOM行为检查；布局/材质/真实帧另由浏览器检查。 */
const test=require('node:test'),assert=require('node:assert/strict');
const countdown=require('./notification-countdown.js');
const {make}=require('./component-topic-harness.cjs');
test('剩余停留时间由同一时钟计算，多种暂停原因必须全部解除',()=>{
 let now=0;const c=countdown.clock(8000,()=>now);now=2000;assert.equal(c.sample().remainingMs,6000);c.pause('pointer');now=4000;c.pause('focus');c.resume('pointer');now=9000;assert.equal(c.sample().remainingMs,6000);c.resume('focus');now=14999;assert.equal(c.sample().remainingMs,1);now=15000;assert.equal(c.sample().phase,'elapsed');
});
test('独立实例互不覆盖；停止和无效时长不能生成伪进度',()=>{
 let now=0;for(const n of [0,-1,NaN,Infinity])assert.throws(()=>countdown.clock(n,()=>now));const a=countdown.clock(8000,()=>now),b=countdown.clock(4000,()=>now);now=2000;a.stop();now=4000;assert.equal(a.sample().remainingMs,6000);assert.equal(b.sample().phase,'elapsed');
});
async function setup(t){const h=await make('#components?section=notifications');t.after(()=>{assert.deepEqual(h.errors,[]);assert.deepEqual(h.network,[]);h.dispose();});let now=0,id=0;const frames=new Map();h.w.performance.now=()=>now;h.w.requestAnimationFrame=fn=>{frames.set(++id,fn);return id;};h.w.cancelAnimationFrame=id=>frames.delete(id);return {...h,step(ms){now+=ms;const current=[...frames.values()];frames.clear();current.forEach(fn=>fn(now));},frameCount:()=>frames.size,frameIds:()=>[...frames.keys()]};}
function surface(h){const el=h.document.createElement('div');el.className='toast k-card show';el.innerHTML='<button>查看结果</button>';h.document.body.appendChild(el);return el;}
test('线条和自动收起共用终点，悬停与焦点暂停，归零仅执行一次',async t=>{
 const h=await setup(t),host=surface(h);let elapsed=0;const timer=h.w.NotificationCountdown.mount(host,{onElapsed:()=>elapsed++});h.step(2000);assert.equal(timer.state().remainingMs,6000);assert.equal(host.querySelector('.notification-countdown-fill').style.transform,'scaleX(0.75)');
 host.dispatchEvent(new h.w.Event('pointerenter'));host.querySelector('button').focus();h.step(9000);assert.equal(elapsed,0);assert.equal(timer.state().phase,'paused');host.dispatchEvent(new h.w.Event('pointerleave'));h.step(1000);assert.equal(elapsed,0);host.querySelector('button').blur();h.step(5999);assert.equal(elapsed,0);h.step(1);assert.equal(elapsed,1);h.step(9000);assert.equal(elapsed,1);assert.equal(h.frameCount(),0);assert.equal(host.querySelector('.notification-countdown'),null);
});
test('同一表面重新挂载会取消旧倒计时，旧回调不能收起新通知',async t=>{
 const h=await setup(t),host=surface(h);let old=0,current=0;const before=h.w.NotificationCountdown.mount(host,{onElapsed:()=>old++});h.step(5000);h.w.NotificationCountdown.mount(host,{onElapsed:()=>current++});assert.equal(before.state().active,false);assert.equal(host.querySelectorAll('.notification-countdown').length,1);h.step(3000);assert.equal(old+current,0);h.step(5000);assert.equal(old,0);assert.equal(current,1);
});
test('替换正文或手动收起后解除计时，保留原有无障碍关联',async t=>{
 const h=await setup(t),host=surface(h);host.setAttribute('aria-describedby','existing-help');let calls=0;const timer=h.w.NotificationCountdown.mount(host,{onElapsed:()=>calls++});assert.equal(host.querySelector('.notification-countdown').getAttribute('aria-hidden'),'true');assert.match(host.getAttribute('aria-describedby'),/existing-help notificationCountdownHelp/);
 host.innerHTML='<button>另一条通知</button>';await h.settle();assert.equal(timer.state().active,false);assert.equal(host.getAttribute('aria-describedby'),'existing-help');h.step(9000);assert.equal(calls,0);
 const beforeFrames=new Set(h.frameIds()),next=h.w.NotificationCountdown.mount(host,{onElapsed:()=>calls++}),ownedFrames=h.frameIds().filter(id=>!beforeFrames.has(id));host.classList.remove('show');await h.settle();assert.equal(next.state().active,false);assert.ok(ownedFrames.every(id=>!h.frameIds().includes(id)));
});
test('减少动态使用按秒线条，保持相同8秒到期语义',async t=>{
 const h=await setup(t),host=surface(h);h.w.matchMedia=()=>({matches:true});let done=0;h.w.NotificationCountdown.mount(host,{onElapsed:()=>done++});h.step(1200);assert.equal(host.querySelector('.notification-countdown-fill').style.transform,'scaleX(0.875)');h.step(6799);assert.equal(done,0);h.step(1);assert.equal(done,1);
});
test('页面不可见时暂停，返回后只走剩余时长',async t=>{
 const h=await setup(t),host=surface(h);let hidden=false,done=0;Object.defineProperty(h.document,'hidden',{configurable:true,get:()=>hidden});const timer=h.w.NotificationCountdown.mount(host,{onElapsed:()=>done++});h.step(2000);hidden=true;h.document.dispatchEvent(new h.w.Event('visibilitychange'));h.step(30000);assert.equal(done,0);assert.equal(timer.state().remainingMs,6000);hidden=false;h.document.dispatchEvent(new h.w.Event('visibilitychange'));h.step(6000);assert.equal(done,1);
});
test('组件示例先注册发布，Toast和完成任务复用原宿主，铃铛记录不被到期删除',async t=>{
 const h=await setup(t);h.click('[data-ct=notify-countdown-success]');assert.equal(h.document.querySelectorAll('#toastEl .notification-countdown').length,1);const notice=h.w.eval('notifications.find(n=>n.eventId==="component.demo.countdown")');assert.equal(notice.resolved,true);h.step(8000);assert.equal(h.document.querySelector('#toastEl').classList.contains('show'),false);assert.equal(notice.deleted,false);
 h.click('[data-ct=notify-countdown-task]');const card=h.document.querySelector('#taskHost [data-task-notification="component-demo:countdown:task"]');assert.ok(card.querySelector('.notification-countdown'));h.step(8000);assert.equal(card.isConnected,false);assert.ok(h.w.eval('notifications.find(n=>n.id==="component-demo:countdown:task"&&!n.deleted)'));
 const component=h.w.PrototypeComponentCatalog.components.find(c=>c.id==='feedback.dismiss-countdown');assert.equal(component.status.production,false);assert.throws(()=>h.w.prototypeNotificationPublisher.publish('component.demo.countdown',{variant:'invalid'}));
});
test('业务通知接入公共倒计时，替换后只有新计时生效，运行任务与历史列表不自动装条',async t=>{
 const h=await setup(t);h.click('[data-ct=notify-countdown-error]');const host=h.document.querySelector('#toastEl'),timer=h.w.NotificationCountdown.get(host);h.w.syncPrototype.open({tab:'file',scope:['preferences:desktop'],action:'export'});h.click('#modalConfirm');await h.settle();assert.equal(timer.state().active,false);assert.equal(host.querySelectorAll('.notification-countdown').length,1);assert.equal(host.querySelector('.notification-countdown').dataset.tone,'success');const current=h.w.NotificationCountdown.get(host);assert.equal(current.state().durationMs,8000);h.step(7999);assert.equal(host.classList.contains('show'),true);h.step(1);assert.equal(host.classList.contains('show'),false);
 h.w.prototypeNotificationPublisher.publish('component.demo.task',{taskId:'a'});assert.equal(h.document.querySelectorAll('#taskHost [data-task-notification="component-demo:component.demo.task:a"] .notification-countdown').length,0);assert.equal(h.document.querySelectorAll('#notificationPanel .notification-countdown').length,0);
});
test('共享Toast保留各自时效与语义色，悬停暂停计时不会被旧timeout抢先收起',async t=>{
 const h=await setup(t);h.w.toast('信息示例',{kind:'info',duration:8000});const host=h.document.querySelector('#toastEl');assert.equal(host.querySelector('.notification-countdown').dataset.tone,'info');h.step(2000);host.dispatchEvent(new h.w.Event('pointerenter'));h.step(30000);assert.equal(host.classList.contains('show'),true);assert.equal(h.w.NotificationCountdown.get(host).state().remainingMs,6000);host.dispatchEvent(new h.w.Event('pointerleave'));h.step(6000);assert.equal(host.classList.contains('show'),false);
 h.w.toast('短反馈');assert.equal(h.w.NotificationCountdown.get(host).state().durationMs,2200);h.step(2200);assert.equal(host.classList.contains('show'),false);
 h.w.toast('错误',{kind:'error'});assert.equal(host.querySelector('.notification-countdown').dataset.tone,'error');assert.equal(h.w.NotificationCountdown.get(host).state().durationMs,5200);
});
test('各任务独立挂载，运行转成功/失败沿正确配色，手动收起和再次运行均清理旧计时',async t=>{
 const h=await setup(t),find=id=>h.document.querySelector('[data-task-notification="'+id+'"]');
 h.w.showTask({notificationId:'host-a',title:'运行中',running:true,kind:'info',steps:[],acts:[]});assert.equal(find('host-a').querySelector('.notification-countdown'),null);
 h.w.showTask({notificationId:'host-a',title:'完成',running:false,kind:'success',steps:[],acts:[]});const a=find('host-a'),old=h.w.NotificationCountdown.get(a);assert.equal(a.querySelector('.notification-countdown').dataset.tone,'success');
 h.w.showTask({notificationId:'host-b',title:'失败',running:false,kind:'danger',steps:[],acts:[]});assert.equal(find('host-b').querySelector('.notification-countdown').dataset.tone,'error');
 h.step(2000);a.dispatchEvent(new h.w.Event('pointerenter'));h.step(6000);assert.equal(find('host-b'),null);assert.ok(find('host-a'));assert.equal(old.state().remainingMs,6000);
 h.w.showTask({notificationId:'host-a',title:'再次运行',running:true,kind:'info',steps:[],acts:[]});assert.equal(old.state().active,false);assert.equal(find('host-a').querySelector('.notification-countdown'),null);h.step(8000);assert.ok(find('host-a'));h.w.clearTask('host-a');assert.equal(h.w.eval('taskTimers.has("host-a")'),false);
});
test('业务通知手动收起只停展示，铃铛保留注册实体，组件重置不清业务计时',async t=>{
 const h=await setup(t);h.w.syncPrototype.open({tab:'webdav',scope:['skill:review']});h.click('[data-sx=check]');await h.wait(750);const host=h.document.querySelector('#toastEl'),timer=h.w.NotificationCountdown.get(host),notice=h.w.eval('notifications.find(n=>n.eventId==="sync.complete")');assert.ok(timer);h.w.prototypeNotificationPublisher.reset();assert.equal(h.w.NotificationCountdown.get(host),timer);h.click('#toastEl [data-collapse=toast]');assert.equal(timer.state().active,false);await h.wait(300);assert.equal(host.classList.contains('show'),false);assert.equal(notice.deleted,false);h.click('#notificationBtn');assert.equal(h.document.querySelectorAll('#notificationPanel .notification-countdown').length,0);
});
