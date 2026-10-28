const test=require('node:test'),assert=require('node:assert/strict');
const {create}=require('./transport-state.js');
test('按用户级 provider 隔离草稿与保存，保存后仍待重启',()=>{
 const s=create();s.edit('policy','prefer');assert.equal(s.snapshot().saved.opencodex.policy,'inherit');s.select('krill');assert.equal(s.snapshot().draft.krill.policy,'inherit');s.edit('policy','http');s.save();assert.deepEqual(s.snapshot().pending,['krill']);s.select('opencodex');assert.equal(s.snapshot().draft.opencodex.policy,'prefer');s.save();assert.equal(s.snapshot().applied.opencodex.policy,'inherit');assert.deepEqual(s.snapshot().pending,['opencodex','krill']);
});
test('重启只采用已保存配置，不提交其它草稿，不生成实际传输观察',()=>{
 const s=create();s.edit('policy','prefer');s.save();s.select('krill');s.edit('policy','http');s.restart();assert.equal(s.snapshot().applied.opencodex.policy,'prefer');assert.equal(s.snapshot().applied.krill.policy,'inherit');assert.deepEqual(s.snapshot().dirty,['krill']);assert.equal(s.observation('thread:config'),null);
});
test('保存失败与还原保留既有保存态，超时只接受非负整数或空值',()=>{
 const s=create();s.edit('policy','prefer');s.setSaveFailure(true);assert.equal(s.save().ok,false);assert.equal(s.snapshot().draft.opencodex.policy,'prefer');assert.equal(s.snapshot().saved.opencodex.policy,'inherit');s.discard();assert.equal(s.snapshot().draft.opencodex.policy,'inherit');s.setSaveFailure(false);for(const bad of ['-1','1.5','abc','9007199254740992']){s.edit('timeout',bad);assert.equal(s.save().ok,false);}for(const good of ['','0','15000']){s.edit('timeout',good);assert.equal(s.save().ok,true);}
});
test('本地偏好统一编译两个字段，直连不改代理入口，跟随默认清除显式覆盖',()=>{
 const s=create();s.edit('policy','prefer');assert.deepEqual(s.preview(),{provider:'opencodex',supports_websockets:true,websockets:true});s.select('krill');s.edit('policy','prefer');s.edit('timeout','15000');assert.deepEqual(s.preview(),{provider:'krill',supports_websockets:true,websocket_connect_timeout_ms:15000});s.edit('policy','inherit');s.edit('timeout','');assert.deepEqual(s.preview(),{provider:'krill'});
});
test('握手与保存重启不冒充请求观察，未知默认不猜成开启',()=>{
 const s=create();assert.match(s.probe(),/未确认/);s.observe('partial');assert.equal(s.observation('thread:config'),null);s.edit('policy','prefer');s.save();assert.match(s.probe(),/未确认/);s.restart();assert.match(s.probe(),/握手通过/);assert.equal(s.observation('thread:config'),null);
});
test('观察区分入口与上游，严格绑定对话，清空与重启不沿用旧结果',()=>{
 const s=create();s.edit('policy','prefer');s.save();s.restart();s.observe('partial');assert.equal(s.observation('thread:config').ingress,'WebSocket');assert.equal(s.observation('thread:config').upstream,'HTTP/SSE');assert.equal(s.observation('project:tools'),null);assert.equal(s.observation('thread:review'),null);s.observe('end-to-end');assert.equal(s.observation('thread:config').upstream,'WebSocket');s.observe('fallback');assert.match(s.observation('thread:config').reason,/上游连接不可用/);s.observe('unknown');assert.equal(s.observation('thread:config'),null);s.observe('partial');s.restart();assert.equal(s.observation('thread:config'),null);
});
test('Link 即使偏好 WebSocket 也只声明 HTTP，开启状态不推导实际上游',()=>{
 const s=create();s.edit('policy','prefer');s.save();s.restart();s.setMode('link');assert.equal(s.preview().supports_websockets,false);assert.equal(Object.hasOwn(s.preview(),'websockets'),false);assert.match(s.probe(),/未通过/);s.observe('end-to-end');assert.equal(s.observation('thread:config').ingress,'HTTP');assert.equal(s.observation('thread:config').upstream,'HTTP/SSE');assert.match(s.observation('thread:config').reason,/Link/);
});
test('仅HTTP阻止WebSocket示例，快照修改不污染内部已保存配置',()=>{
 const s=create();s.edit('policy','http');s.save();s.restart();s.observe('end-to-end');assert.equal(s.observation('thread:config').ingress,'HTTP');const state=s.snapshot();state.saved.opencodex.policy='prefer';assert.equal(s.snapshot().saved.opencodex.policy,'http');
});
