/* 注册事件到现有通知宿主的应用层适配；业务任务正文保留在任务页。 */
(() => {
 'use strict';
 const eventId='model.template.external',spec=window.MaintenanceEvents.events.find(e=>e.id===eventId),revisions=new Map();
 const labels={reviewed:['外部检查已完成','info','检查报告已返回，未修改模版。'],ready:['外部模版结果待提交','info','结果已通过预检，可在任务中核对差异。'],invalid:['外部模版结果需修正','warning','提交未通过校验，尚未保存。'],conflict:['外部任务基准已变化','warning','保留外部结果，请重新核对任务基准。'],saved:['外部任务已保存新版本','info','新版本已保存，默认和渠道采用关系保持。'],unchanged:['外部任务核对完成','info','内容无变化，没有新增版本。'],cancelled:['外部任务已取消','info','迟到提交不再保存。']};
 window.modelExternalTaskNotificationBridge=snapshot=>{
  if(!spec||!snapshot||typeof snapshot.taskId!=='string'||!snapshot.taskId||!labels[snapshot.state])return false;
  const {taskId,state}=snapshot,key='model-external:'+taskId,revision=state+':'+String(snapshot.submissionId||'');
  if(revisions.get(key)===revision)return true;revisions.set(key,revision);
  let item=notifications.find(n=>n.id===key);if(item?.deleted)return true;
  const [title,kind,detail]=labels[state],resolved=['saved','unchanged','reviewed','cancelled'].includes(state);
  if(!item&&resolved){if(state!=='cancelled')toast(title);return true;}
  if(!item){item={id:key,eventId,cat:spec.category,op:taskId,dedupe:key,deleted:false};notifications.unshift(item);}
  Object.assign(item,{title,kind,detail,time:clockLabel(),resolved,read:resolved?item.read||false:false,externalTaskId:taskId});if(resolved)item.resolvedAt=item.time;
  renderNotifications();return true;
 };
 window.openExternalTaskNotification=id=>{
  const item=notifications.find(n=>n.id===id&&!n.deleted&&n.eventId===eventId);if(!item)return false;
  item.read=true;renderNotifications();closeNotificationPanel();return window.modelPrototype?.openExternalTask(item.externalTaskId)||false;
 };
})();
