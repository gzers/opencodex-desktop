/* 注册事件的公共原型适配器；只投影既有 Toast/Task/铃铛，无真实投递或存储。 */
(() => {
 'use strict';
 const prefix='component-demo:',hidden=new Set();let countdownRevision=0;
 const overviewDefinition=window.MaintenanceEvents.events.find(e=>e.id==='overview.demo.notification'&&e.implemented==='prototype');
 const overviewPrefix='component-demo:overview:',overviewTimers=new Set();let overviewGeneration=0;
 const overviewSamples=[{title:'配置已导出',detail:'本次所选配置已导出。',kind:'success',resolved:true},{title:'同步部分完成',detail:'其余项已完成，1项可稍后重试。',kind:'warning',resolved:false},{title:'模型配置已写入',detail:'可从通知中心查看本次结果。',kind:'info',resolved:true}];
 function clearOverviewViews(){
  overviewGeneration++;overviewTimers.forEach(clearTimeout);overviewTimers.clear();
  for(const n of notifications.filter(n=>n.eventId==='overview.demo.notification')){clearTask(n.id);hidden.add(n.id);}
 }
 function overview(options){
  if(!overviewDefinition||Object.keys(options).some(k=>k!=='scene')||!overviewDefinition.variants.includes(options.scene))throw Error('概览通知场景未注册');
  clearOverviewViews();
  if(options.scene==='reset'){
   for(let i=notifications.length-1;i>=0;i--)if(notifications[i].eventId==='overview.demo.notification')notifications.splice(i,1);
   renderNotifications();return;
  }
  const generation=overviewGeneration;
  function render(index){
   if(generation!==overviewGeneration)return;
   const sample=overviewSamples[index],id=overviewPrefix+index;
   let item=notifications.find(n=>n.id===id);if(item?.deleted)return;
   if(!item){item={id,eventId:overviewDefinition.id,cat:overviewDefinition.category,op:id,dedupe:id,deleted:false};notifications.unshift(item);}
   Object.assign(item,{...sample,time:clockLabel(),read:sample.resolved,overview:{index,generation}});if(sample.resolved)item.resolvedAt=item.time;
   hidden.delete(id);renderNotifications();showTask({notificationId:id,title:item.title,phase:item.detail,running:false,kind:item.kind,steps:[],acts:[]});
  }
  if(options.scene==='single'){render(0);return;}
  overviewSamples.forEach((_,index)=>{
   if(options.scene!=='staggered'||index===0){render(index);return;}
   const timer=setTimeout(()=>{overviewTimers.delete(timer);render(index);},index*1100);overviewTimers.add(timer);
  });
 }
 const definitions=new Map(window.MaintenanceEvents.events.filter(e=>e.scope==='prototype-only'&&e.id.startsWith('component.demo.')).map(e=>[e.id,e]));
 const syncDefinitions=new Map(window.MaintenanceEvents.events.filter(e=>e.id.startsWith('sync.')&&e.implemented==='prototype'&&e.consumer==='syncPrototype'&&e.publisher==='prototypeNotificationPublisher').map(e=>[e.id,e]));
 const syncDeliveries=[],syncRevisions=new Map();
 const syncTitles={'sync.export.complete':'配置已导出','sync.export.failed':'配置导出未完成','sync.import.complete':'配置导入完成','sync.import.failed':'配置导入未完成','sync.connected':'WebDAV连接成功','sync.connection.failed':'WebDAV连接失败','sync.connection.saved':'连接配置已保存','sync.complete':'同步完成','sync.failed':'同步未完成','sync.started':'正在同步','sync.import.started':'正在导入','sync.export.directory.opened':'已打开导出目录'};
 function publishSync(eventId,options){
  const spec=syncDefinitions.get(eventId);if(!spec)throw Error('同步事件未注册或尚未接入：'+eventId);
  const allowed=['operationId','total','downloaded','uploaded','skipped','failed','errorCode'];
  if(Object.keys(options).some(k=>!allowed.includes(k))||!/^[1-9][0-9]{0,8}$/.test(String(options.operationId)))throw Error('同步事件仅允许固定操作身份与计数');
  const payload={operationId:String(options.operationId)};
  for(const key of ['total','downloaded','uploaded','skipped','failed']){const n=options[key]??0;if(!Number.isSafeInteger(n)||n<0||n>100000)throw Error('同步事件计数无效');payload[key]=n;}
  const errors={none:'',write:'目标文件无法写入，原内容保留。',connection:'连接未完成，请检查连接设置。',partial:'部分对象未完成，可仅重试失败项。'};
  payload.errorCode=options.errorCode||'none';if(!Object.hasOwn(errors,payload.errorCode))throw Error('同步事件错误类型未注册');
  const revision=eventId+':'+JSON.stringify(payload),key=eventId+':'+payload.operationId;
  if(syncRevisions.get(key)===revision)return 'sync-demo:'+payload.operationId;
  syncRevisions.set(key,revision);
  const failed=eventId.endsWith('.failed'),title=['sync.failed','sync.import.failed'].includes(eventId)&&payload.failed&&payload.failed<payload.total?'部分'+(eventId.startsWith('sync.import.')?'导入':'同步')+'完成':syncTitles[eventId];
  const detail=eventId==='sync.export.complete'?payload.total+' 项配置已导出，可查看保存位置。':eventId==='sync.connected'?'服务连接测试通过。':eventId==='sync.connection.saved'?'后续同步使用本机保存的连接配置。':eventId==='sync.export.directory.opened'?'当前导出包的保存位置已打开。':eventId.includes('started')?'正在处理所选 '+payload.total+' 项内容。':failed?errors[payload.errorCode]+' 已完成 '+(payload.downloaded+payload.uploaded)+' 项，失败 '+payload.failed+' 项。':'下载更新 '+payload.downloaded+' 项，上传 '+payload.uploaded+' 项，跳过 '+payload.skipped+' 项。';
  const id='sync-demo:'+payload.operationId;
  let item=notifications.find(n=>n.id===id);
  syncDeliveries.unshift({eventId,operationId:payload.operationId,delivery:item?.deleted?['log']:[...spec.delivery],...(item?.deleted?{suppressed:'deleted'}:{}),payload:{...payload}});
  if(syncDeliveries.length>100)syncDeliveries.length=100;
  if(item?.deleted)return id; // 同一操作的晚回/重试不复活已删除通知。
  if(spec.delivery.includes('center')){
   if(!item){item={id,cat:spec.category,op:payload.operationId,dedupe:id,deleted:false};notifications.unshift(item);}
   Object.assign(item,{eventId,title,detail,kind:spec.severity,time:clockLabel(),read:!failed,resolved:!failed,sync:{...payload}});
   if(item.resolved)item.resolvedAt=item.time;else delete item.resolvedAt;
   renderNotifications();
  }
  if(spec.delivery.includes('toast')){
   toast(title,{kind:failed?(spec.severity==='danger'?'error':spec.severity):'success',detail,action:'查看结果',duration:spec.toastDurationMs});
   document.querySelector('#toastEl .toast-act').dataset.syncNotice=id;
  }
  return id;
 }
 function openSync(id){
  const item=notifications.find(n=>n.id===id&&!n.deleted&&syncDefinitions.has(n.eventId));if(!item)return false;
  item.read=true;closeNotificationPanel();renderNotifications();document.getElementById('toastEl').classList.remove('show');
  if(window.syncPrototype?.openReceipt(item.sync.operationId))return true;
  openModal(item.title,'<p>'+logEsc(item.detail)+'</p>','关闭',null,{hideConfirm:true,cancelLabel:'关闭',onDelete:()=>deleteNotification(id)});return true;
 }
 const phases=['准备样例','核对内容','完成'];
 const states={running:['正在处理示例任务','info',false],success:['示例任务已完成','info',true],failed:['示例任务未完成','warning',false],stopped:['示例任务已停止','info',true]};
 let lastToast='';
 function renderCountdown(item){
  const variant=item.countdown.variant,duration=definitions.get(item.eventId).toastDurationMs;
  if(variant==='task'){
   showTask({notificationId:item.id,title:item.title,phase:item.detail,running:false,kind:'success',steps:[],acts:[]});
  }else{
   lastToast=item.title;toast(item.title,{kind:variant,detail:item.detail,duration});
  }
 }
 function publishCountdown(options){
  if(Object.keys(options).some(k=>k!=='variant')||!['success','error','warning','info','task'].includes(options.variant||'success'))throw Error('倒计时演示仅允许已登记样式');
  const variant=options.variant||'success',id=prefix+'countdown:'+variant;
  let item=notifications.find(n=>n.id===id);if(item?.deleted)return id;
  const title=variant==='error'?'示例操作未完成':variant==='warning'?'示例操作需要注意':variant==='info'?'示例信息通知':variant==='task'?'示例任务已完成':'示例操作已完成';
  if(!item){item={id,eventId:'component.demo.countdown',cat:'system',op:id,dedupe:id,deleted:false};notifications.unshift(item);}
  const attention=variant==='error'||variant==='warning';
  Object.assign(item,{title,kind:variant==='error'?'danger':variant==='warning'?'warning':'info',detail:attention?'可在通知中心查看处理事项。':'结果已保留，可从铃铛查看。',time:clockLabel(),read:!attention,resolved:!attention,demo:{taskId:variant,state:attention?'failed':'success',step:3,revision:++countdownRevision},countdown:{variant}});
  if(item.resolved)item.resolvedAt=item.time;
  hidden.delete(id);renderNotifications();renderCountdown(item);return id;
 }
 function task(item){
  if(item.deleted||hidden.has(item.id))return;
  const state=item.demo.state,step=item.demo.step;
  showTask({notificationId:item.id,title:item.title,phase:item.detail,running:state==='running',kind:state==='stopped'?'neutral':item.kind,steps:phases.map((p,i)=>[p,i<step?'done':i===step&&state==='running'?'active':state==='failed'&&i===step?'failed':'pending']),acts:state==='running'?[['推进一步','component-demo-next'],['停止','component-demo-stop']]:state==='failed'?[['重试','component-demo-retry']]:[]});
 }
 function publish(eventId,options={}){
  if(eventId==='overview.demo.notification')return overview(options);
  if(eventId.startsWith('sync.'))return publishSync(eventId,options);
  const definition=definitions.get(eventId);if(!definition)throw Error('未注册的组件演示通知');
  if(eventId==='component.demo.countdown')return publishCountdown(options);
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
  const overviewItem=notifications.find(n=>n.id===id&&!n.deleted&&n.eventId==='overview.demo.notification');
  if(overviewItem){overviewItem.read=true;hidden.delete(id);closeNotificationPanel();renderNotifications();showTask({notificationId:id,title:overviewItem.title,phase:overviewItem.detail,running:false,kind:overviewItem.kind,steps:[],acts:[]});return true;}
  if(notifications.some(n=>n.id===id&&syncDefinitions.has(n.eventId)))return openSync(id);
  const item=notifications.find(n=>n.id===id&&!n.deleted&&definitions.has(n.eventId));if(!item)return false;
  item.read=true;hidden.delete(id);closeNotificationPanel();renderNotifications();
  if(item.eventId==='component.demo.task')task(item);
  else if(item.eventId==='component.demo.countdown')renderCountdown(item);
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
  hidden.clear();const el=document.getElementById('toastEl');if(lastToast&&el.textContent.includes(lastToast)){window.NotificationCountdown.get(el)?.destroy();el.classList.remove('show');}lastToast='';renderNotifications();
 }
 // 删除不影响任务本体，只移除对应观察视图；业务通知不在此订阅范围内。
 new MutationObserver(()=>{for(const n of notifications)if(definitions.has(n.eventId)&&n.deleted)clearTask(n.id);}).observe(document.getElementById('notificationList'),{childList:true,subtree:true});
 document.getElementById('toastEl').addEventListener('click',event=>{const button=event.target.closest('[data-sync-notice]');if(!button)return;event.preventDefault();event.stopImmediatePropagation();openSync(button.dataset.syncNotice);},true);
 window.prototypeNotificationPublisher={publish,open,action,hide:id=>{hidden.add(id);const host=[...document.querySelectorAll('#taskHost .task-card')].find(el=>el.dataset.taskNotification===id);if(host)window.NotificationCountdown.get(host)?.destroy();},reset,state:()=>notifications.filter(n=>definitions.has(n.eventId)).map(n=>({id:n.id,eventId:n.eventId,read:n.read,resolved:n.resolved,deleted:n.deleted,demo:{...n.demo}})),deliveries:()=>JSON.parse(JSON.stringify(syncDeliveries))};
})();
