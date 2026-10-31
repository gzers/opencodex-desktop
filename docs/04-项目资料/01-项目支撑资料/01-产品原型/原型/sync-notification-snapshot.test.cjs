/* 独立运行真实事件目录与公共 publisher；壳层宿主用 spy，不加载总原型 UI。 */
const test=require('node:test'),assert=require('node:assert/strict');
const fs=require('node:fs'),path=require('node:path'),vm=require('node:vm');
const catalog=require('./maintenance-events.js');
const script=new vm.Script(fs.readFileSync(path.join(__dirname,'prototype-notification-publisher.js'),'utf8'),{filename:'prototype-notification-publisher.js'});
const plain=value=>JSON.parse(JSON.stringify(value));
const spec=id=>catalog.events.find(event=>event.id===id);

function make(){
 const notifications=[],toasts=[],modals=[],receipts=[],tasks=[],ui={renders:0,closed:0,receiptAvailable:true};
 const action={dataset:{}},list={},listeners=new Map();
 const toastEl={classList:{remove:name=>{assert.equal(name,'show');ui.toastHidden=true;}},addEventListener:(name,fn,capture)=>{assert.equal(capture,true);listeners.set(name,fn);}};
 const document={
  getElementById:id=>{assert.ok(['toastEl','notificationList'].includes(id));return id==='toastEl'?toastEl:list;},
  querySelector:selector=>{assert.equal(selector,'#toastEl .toast-act');return action;}
 };
 const window={MaintenanceEvents:catalog,syncPrototype:{openReceipt:id=>{receipts.push(id);return ui.receiptAvailable;}}};
 const context=vm.createContext({window,document,notifications,
  MutationObserver:class{observe(target){assert.equal(target,list);}},
  clockLabel:()=>String(toasts.length+1),renderNotifications:()=>ui.renders++,
  toast:(title,options)=>toasts.push({title,...plain(options)}),
  closeNotificationPanel:()=>ui.closed++,openModal:(...args)=>modals.push(args),
  logEsc:value=>value.replace(/[&<>"]/g,char=>({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[char])),
  deleteNotification:id=>{notifications.find(item=>item.id===id).deleted=true;},
  showTask:options=>tasks.push(options)
 });
 script.runInContext(context);
 return {publisher:window.prototypeNotificationPublisher,notifications,toasts,modals,receipts,tasks,ui,action,listeners};
}

test('同步与导入四种终态合并旧计数，Toast和铃铛共用摘要且payload保持兼容',()=>{
 for(const eventId of ['sync.complete','sync.import.complete','sync.failed','sync.import.failed']){
  const h=make(),failed=eventId.endsWith('.failed');
  const payload={operationId:'101',total:failed?9:7,downloaded:2,uploaded:3,skipped:2,failed:failed?2:0,errorCode:failed?'partial':'none'};
  assert.equal(h.publisher.publish(eventId,payload),'sync-demo:101');
  const item=h.notifications[0],toast=h.toasts[0],delivery=h.publisher.deliveries()[0];
  assert.equal(item.detail,failed?'部分对象未完成，可仅重试失败项。 已更新 5 项，未变化 2 项，失败 2 项。':'已更新 5 项，未变化 2 项。');
  assert.equal(item.title,failed?(eventId.startsWith('sync.import.')?'部分导入完成':'部分同步完成'):(eventId==='sync.complete'?'同步完成':'配置导入完成'));
  assert.equal(toast.detail,item.detail);assert.equal(toast.title,item.title);
  assert.doesNotMatch(item.title+item.detail,/上传|下载|跳过/);
  assert.deepEqual(plain(item.sync),payload);assert.deepEqual(plain(delivery.payload),payload);
  assert.deepEqual(plain(delivery.delivery),['toast','center','log']);assert.equal(delivery.eventId,eventId);
  assert.equal(item.cat,spec(eventId).category);assert.equal(item.kind,spec(eventId).severity);
  assert.equal(toast.kind,failed?(spec(eventId).severity==='danger'?'error':spec(eventId).severity):'success');
  assert.equal(toast.action,'查看结果');assert.equal(toast.duration,8000);assert.equal(toast.duration,spec(eventId).toastDurationMs);
  assert.equal(item.read,!failed);assert.equal(item.resolved,!failed);
  if(failed)assert.equal(Object.hasOwn(item,'resolvedAt'),false);else assert.equal(item.resolvedAt,item.time);
  assert.deepEqual(h.tasks,[]);assert.equal(h.action.dataset.syncNotice,item.id);
 }
});

test('零更新显示配置已一致，单边旧字段和上限计数仍按下载加上传计算',()=>{
 for(const counts of [{skipped:6},{downloaded:3},{uploaded:4},{downloaded:100000,uploaded:100000}]){
  const h=make();h.publisher.publish('sync.complete',{operationId:102,total:6,...counts});
  const item=h.notifications[0],updated=(counts.downloaded||0)+(counts.uploaded||0);
  assert.equal(item.title,updated===0?'配置已一致':'同步完成');
  assert.equal(item.detail,'已更新 '+updated+' 项，未变化 '+(counts.skipped||0)+' 项。');
  assert.equal(item.sync.operationId,'102');assert.equal(item.sync.failed,0);assert.equal(item.sync.errorCode,'none');
 }
 const h=make();h.publisher.publish('sync.import.complete',{operationId:'103',total:6,skipped:6});
 assert.equal(h.notifications[0].title,'配置导入完成');assert.equal(h.notifications[0].detail,'已更新 0 项，未变化 6 项。');
});

test('备份失败使用固定原因，不混同写入错误，也不泄露来源或原始异常',()=>{
 for(const eventId of ['sync.failed','sync.import.failed']){
  const h=make();h.publisher.publish(eventId,{operationId:'104',total:4,skipped:1,failed:3,errorCode:'backup'});
  const item=h.notifications[0];
  assert.equal(item.detail,'更新前备份未完成，原内容保留。 已更新 0 项，未变化 1 项，失败 3 项。');
  assert.equal(item.resolved,false);assert.equal(h.toasts[0].detail,item.detail);
  assert.equal(h.publisher.deliveries()[0].payload.errorCode,'backup');
  assert.doesNotMatch(item.detail,/无法写入|上传|下载/);
 }
 for(const [errorCode,reason] of [['write','目标文件无法写入，原内容保留。'],['connection','连接未完成，请检查连接设置。'],['none','']]){
  const h=make();h.publisher.publish('sync.failed',{operationId:'105',total:1,failed:1,errorCode});
  assert.equal(h.notifications[0].title,'同步未完成');
  assert.equal(h.notifications[0].detail,reason+' 已更新 0 项，未变化 0 项，失败 1 项。');
 }
});

test('只允许原有数字字段和固定错误枚举，拒绝自由文本与第二套通知字段且无副作用',()=>{
 const h=make(),base={operationId:'106'};
 const invalid=[
  ...['detail','path','url','credentialRef','password','message','updated','unchanged'].map(key=>({...base,[key]:'/Users/private/secret'})),
  ...['0','01','-1','1.5','1e2','1000000000','secret'].map(operationId=>({operationId})),
  ...['total','downloaded','uploaded','skipped','failed'].flatMap(key=>[-1,1.5,100001,Number.MAX_SAFE_INTEGER,Infinity,NaN,'1',true].map(value=>({...base,[key]:value}))),
  ...['secret','/Users/private/secret','toString','__proto__'].map(errorCode=>({...base,errorCode}))
 ];
 for(const options of invalid)assert.throws(()=>h.publisher.publish('sync.complete',options));
 for(const id of ['sync.unknown','sync.conflict'])assert.throws(()=>h.publisher.publish(id,base));
 assert.deepEqual(h.notifications,[]);assert.deepEqual(h.toasts,[]);assert.deepEqual(plain(h.publisher.deliveries()),[]);assert.equal(h.ui.renders,0);
 h.publisher.publish('sync.complete',base);
 const expected={operationId:'106',total:0,downloaded:0,uploaded:0,skipped:0,failed:0,errorCode:'none'};
 assert.deepEqual(plain(h.notifications[0].sync),expected);assert.deepEqual(plain(h.publisher.deliveries()[0].payload),expected);
 assert.doesNotMatch(JSON.stringify(h.publisher.deliveries()),/Users|secret|credentialRef|password/);
});

test('重复终态去重，修订仍更新同一实体；同操作重试恢复不影响其它失败',()=>{
 for(const [failure,success] of [['sync.failed','sync.complete'],['sync.import.failed','sync.import.complete']]){
  const h=make(),payload={operationId:'107',total:4,downloaded:1,uploaded:1,skipped:1,failed:1,errorCode:'partial'};
  h.publisher.publish(failure,payload);const item=h.notifications[0],renders=h.ui.renders;
  h.publisher.publish(failure,payload);
  assert.equal(h.publisher.deliveries().length,1);assert.equal(h.toasts.length,1);assert.equal(h.ui.renders,renders);
  h.publisher.publish(failure,{...payload,errorCode:'backup'});
  assert.equal(h.publisher.deliveries().length,2);assert.equal(h.notifications[0],item);
  assert.match(item.detail,/^更新前备份未完成/);
  h.publisher.publish(failure,{...payload,operationId:'108'});const other=h.notifications[0];
  h.publisher.publish(success,{...payload,downloaded:2,failed:0,errorCode:'none'});
  assert.equal(h.notifications.length,2);assert.equal(h.notifications.find(n=>n.id==='sync-demo:107'),item);
  assert.equal(item.eventId,success);assert.equal(item.resolved,true);assert.equal(item.read,true);assert.equal(item.resolvedAt,item.time);
  assert.equal(item.detail,'已更新 3 项，未变化 1 项。');assert.equal(other.resolved,false);
  h.publisher.publish(failure,{...payload,errorCode:'backup'});
  assert.equal(item.eventId,success);assert.equal(h.toasts.length,4);
 }
});

test('已删除操作的晚回只记日志，不重新弹出或复活；新操作照常投递',()=>{
 const h=make(),payload={operationId:'109',total:2,failed:2,errorCode:'backup'};
 h.publisher.publish('sync.failed',payload);const item=h.notifications[0];item.deleted=true;
 const renders=h.ui.renders;
 h.publisher.publish('sync.complete',{...payload,downloaded:2,failed:0,errorCode:'none'});
 assert.equal(h.notifications.length,1);assert.equal(item.deleted,true);assert.equal(item.resolved,false);
 assert.equal(h.toasts.length,1);assert.equal(h.ui.renders,renders);assert.equal(h.publisher.open(item.id),false);
 const delivery=h.publisher.deliveries()[0];assert.deepEqual(plain(delivery.delivery),['log']);assert.equal(delivery.suppressed,'deleted');
 h.publisher.publish('sync.complete',{operationId:'110',total:2,downloaded:2});
 assert.equal(h.notifications.length,2);assert.equal(h.toasts.length,2);assert.equal(h.notifications[0].deleted,false);
});

test('查看结果沿原Toast动作和铃铛回执入口，缺回执时复用共享弹窗',()=>{
 const h=make();h.publisher.publish('sync.complete',{operationId:'111',total:2,downloaded:1,uploaded:1});
 let prevented=false,stopped=false;
 h.listeners.get('click')({target:{closest:selector=>{assert.equal(selector,'[data-sync-notice]');return h.action;}},preventDefault:()=>{prevented=true;},stopImmediatePropagation:()=>{stopped=true;}});
 assert.equal(prevented,true);assert.equal(stopped,true);assert.deepEqual(h.receipts,['111']);assert.equal(h.ui.closed,1);assert.equal(h.ui.toastHidden,true);assert.deepEqual(h.modals,[]);
 h.ui.receiptAvailable=false;assert.equal(h.publisher.open('sync-demo:111'),true);
 const [title,body,,confirm,options]=h.modals[0];
 assert.equal(title,'同步完成');assert.equal(body,'<p>已更新 2 项，未变化 0 项。</p>');assert.equal(confirm,null);assert.equal(options.hideConfirm,true);
 options.onDelete();assert.equal(h.notifications[0].deleted,true);assert.equal(h.publisher.open('sync-demo:111'),false);assert.equal(h.publisher.open('sync-demo:999'),false);
});

test('开始、连接和导出继续使用既有标题、摘要与注册投递',()=>{
 const cases=[
  ['sync.started','正在同步','正在处理所选 7 项内容。'],
  ['sync.import.started','正在导入','正在处理所选 7 项内容。'],
  ['sync.connected','WebDAV连接成功','服务连接测试通过。'],
  ['sync.connection.saved','连接配置已保存','后续同步使用本机保存的连接配置。'],
  ['sync.export.complete','配置已导出','7 项配置已导出，可查看保存位置。'],
  ['sync.export.directory.opened','已打开导出目录','当前导出包的保存位置已打开。'],
  ['sync.export.failed','配置导出未完成','目标文件无法写入，原内容保留。 已完成 3 项，失败 2 项。'],
  ['sync.connection.failed','WebDAV连接失败','连接未完成，请检查连接设置。 已完成 3 项，失败 2 项。']
 ];
 for(const [eventId,title,detail] of cases){
  const h=make();h.publisher.publish(eventId,{operationId:'112',total:7,downloaded:2,uploaded:1,skipped:2,failed:2,errorCode:eventId==='sync.connection.failed'?'connection':'write'});
  const delivery=h.publisher.deliveries()[0];assert.deepEqual(plain(delivery.delivery),spec(eventId).delivery);assert.equal(delivery.eventId,eventId);
  if(eventId.endsWith('started')){assert.deepEqual(h.notifications,[]);assert.deepEqual(h.toasts,[]);}
  else{assert.equal(h.notifications[0].title,title);assert.equal(h.notifications[0].detail,detail);assert.equal(h.toasts[0].title,title);assert.equal(h.toasts[0].detail,detail);assert.equal(h.toasts[0].duration,8000);}
  assert.deepEqual(h.tasks,[]);
 }
});

test('投递历史保留原上限，返回payload与通道副本不能修改内部通知或去重身份',()=>{
 const h=make();
 for(let i=1;i<=105;i++)h.publisher.publish('sync.complete',{operationId:String(i),total:1,uploaded:1});
 const records=h.publisher.deliveries();assert.equal(records.length,100);assert.equal(records[0].operationId,'105');assert.equal(records.at(-1).operationId,'6');
 records[0].payload.uploaded=999;records[0].delivery.length=0;
 assert.equal(h.publisher.deliveries()[0].payload.uploaded,1);assert.deepEqual(plain(h.publisher.deliveries()[0].delivery),['toast','center','log']);assert.equal(h.notifications[0].sync.uploaded,1);
 h.publisher.publish('sync.complete',{operationId:'105',total:1,uploaded:1});assert.equal(h.toasts.length,105);
});
