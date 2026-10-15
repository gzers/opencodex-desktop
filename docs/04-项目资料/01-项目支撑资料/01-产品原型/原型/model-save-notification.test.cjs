/* 模型保存必须注册到应用壳层通知中心 / 任务卡，不得回到模型页内自建进度卡。 */
const {test}=require('node:test');
const assert=require('node:assert/strict');
const fs=require('node:fs');
const path=require('node:path');
const events=require('./maintenance-events.js');

const dir=__dirname;
const source=()=>fs.readFileSync(path.join(dir,'index.html'),'utf8');
const model=()=>fs.readFileSync(path.join(dir,'model-config.js'),'utf8');

test('模型保存事件注册到共享任务与通知中心',()=>{
 const spec=events.events.find(item=>item.id==='model.policy.save');
 assert.ok(spec);
 assert.deepEqual(spec.delivery,['task','center']);
 assert.equal(spec.implemented,'prototype');
 assert.equal(spec.consumer,'modelSaveNotificationBridge');
 assert.deepEqual(spec.dedupeFields,['operationId']);
});

test('模型页移除独立进度卡，保存状态只经通知宿主桥接',()=>{
 const html=source(),js=model();
 assert.match(html,/id="notifHost"/);
 assert.match(html,/id="notificationPanel"/);
 assert.match(html,/modelSaveNotificationBridge/);
 assert.match(js,/modelSaveNotificationBridge\?\./);
 assert.doesNotMatch(js,/function progress\(\)/);
 assert.doesNotMatch(js,/mp-progress/);
});

test('共享任务卡支持收起到铃铛和模型保存动作',()=>{
 const html=source();
 assert.match(html,/data-collapse="task"/);
 assert.match(html,/data-task-act=/);
 assert.match(html,/modelSaveNotificationAction/);
 assert.match(html,/openModelSaveNotification/);
});
