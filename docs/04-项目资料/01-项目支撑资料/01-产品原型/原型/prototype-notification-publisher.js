/* 组件专题的注册事件适配器；唯一输出仍是现有 Toast/Task/通知中心宿主。 */
(() => {
 'use strict';
 const prefix='component-demo:',hidden=new Set();
 const definitions=new Map(window.MaintenanceEvents.events.filter(e=>e.scope==='prototype-only'&&e.id.startsWith('component.demo.')).map(e=>[e.id,e]));
 const phases=['准备样例','核对内容','完成'];
 const states={running:['正在处理示例任务','info',false],success:['示例任务已完成','info',true],failed:['示例任务未完成','warning',false],stopped:['示例任务已停止','info',true]};
 let lastToast='';
 function task(item){
  if(item.deleted||hidden.has(item.id))return;
  const state=item.demo.state,step=item.demo.step;
  showTask({notificationId:item.id,title:item.title,phase:item.detail,running:state==='running',kind:state==='stopped'?'neutral':item.kind,steps:phases.map((p,i)=>[p,i<step?'done':i===step&&state==='running'?'active':state==='failed'&&i===step?'failed':'pending']),acts:state==='running'?[['推进一步','component-demo-next'],['停止','component-demo-stop']]:state==='failed'?[['重试','component-demo-retry']]:[]});
 }
 function publish(eventId,options={}){
  const definition=definitions.get(eventId);if(!definition)throw Error('未注册的组件演示通知');
  const taskId=options.taskId||'a';if(!['a','b'].includes(taskId))throw Error('无效的演示任务ID');
  const state=options.state||'running',step=options.step??0;
  if(!states[state]||!Number.isInteger(step)||step<0||step>3)throw Error('无效的演示状态');
  if(eventId==='component.demo.feedback'){lastToast='示例操作已完成';toast(lastToast);return null;}
  const id=prefix+eventId+':'+taskId;
  let item=notifications.find(n=>n.id===id);
  if(item?.deleted)return id; // 用户删除后的晚回不复活；需显式重置演示。
  const revision=state+':'+step;if(item?.demo.revision===revision)return id;
  const [title,kind,resolved]=states[state];
  const detail=eventId==='component.demo.attention'?'示例需要确认，请在通知中心标记处理结果。':state==='running'?phases[Math.min(step,2)]+'…':state==='failed'?'样例校验未通过；可重试此演示。':state==='stopped'?'演示已停止。':'示例内容已核对完成。';
  if(!item){item={id,eventId,cat:definition.category,op:id,dedupe:id,read:false,resolved:false,deleted:false};notifications.unshift(item);}
  else if(item.demo.state!==state)item.read=false;
  Object.assign(item,{title:eventId==='component.demo.attention'?'示例待处理事项':title,kind,detail,time:clockLabel(),resolved:eventId==='component.demo.attention'?false:resolved,demo:{taskId,state,step,revision}});
  if(item.resolved)item.resolvedAt=item.time;
  renderNotifications();if(eventId==='component.demo.task')task(item);return id;
 }
 function open(id){
  const item=notifications.find(n=>n.id===id&&!n.deleted&&definitions.has(n.eventId));if(!item)return false;
  item.read=true;hidden.delete(id);closeNotificationPanel();renderNotifications();
  if(item.eventId==='component.demo.task')task(item);
  else openModal(item.title,'<p>'+logEsc(item.detail)+'</p>','关闭',null,{hideConfirm:true,cancelLabel:'关闭',hideNote:true,onExtra:item.resolved?null:()=>resolveNotification(id),extraLabel:'标记已解决',onDelete:()=>deleteNotification(id)});
  return true;
 }
 function action(name,id){
  const item=notifications.find(n=>n.id===id&&!n.deleted&&n.eventId==='component.demo.task');if(!item)return;
  const step=Math.min(3,item.demo.step+1),state=name==='stop'?'stopped':name==='retry'?'running':step===3?'success':'running';
  if(!['next','stop','retry'].includes(name))return;
  publish(item.eventId,{taskId:item.demo.taskId,state,step:name==='retry'?0:step});
 }
 function reset(){
  for(let i=notifications.length-1;i>=0;i--)if(definitions.has(notifications[i].eventId)){clearTask(notifications[i].id);notifications.splice(i,1);}
  hidden.clear();const el=document.getElementById('toastEl');if(lastToast&&el.textContent.includes(lastToast)){clearTimeout(window.__toastTimer);el.classList.remove('show');}lastToast='';renderNotifications();
 }
 // 删除不影响任务本体，只移除对应观察视图；业务通知不在此订阅范围内。
 new MutationObserver(()=>{for(const n of notifications)if(definitions.has(n.eventId)&&n.deleted)clearTask(n.id);}).observe(document.getElementById('notificationList'),{childList:true,subtree:true});
 window.prototypeNotificationPublisher={publish,open,action,hide:id=>hidden.add(id),reset,state:()=>notifications.filter(n=>definitions.has(n.eventId)).map(n=>({id:n.id,eventId:n.eventId,read:n.read,resolved:n.resolved,deleted:n.deleted,demo:{...n.demo}}))};
})();
