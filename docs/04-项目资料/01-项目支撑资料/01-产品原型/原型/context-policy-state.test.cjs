const test=require('node:test'),assert=require('node:assert/strict');
const {create}=require('./context-policy-state.js');
test('默认不固化功能值；后台与轮末使用独立参数',()=>{
 const s=create();assert.deepEqual(s.preview(),{});s.edit('mode','background');s.edit('backgroundPercent','65');s.edit('postTurnPercent','80');const p=s.preview();assert.equal(p.features.async_compaction.enabled,true);assert.equal(p.features.async_compaction.model_async_compact_threshold_percent,65);assert.equal(p.model_post_turn_compact_threshold_percent,80);assert.equal(p.features.token_budget.use_history_notes_extension,false);s.edit('mode','regular');assert.equal(s.preview().features.async_compaction.enabled,false);assert.equal(s.preview().features.async_compaction.model_async_compact_threshold_percent,undefined);assert.equal(s.preview().model_post_turn_compact_threshold_percent,80);
});
test('历史与笔记为一组预设，返回常规时移除其启用意图',()=>{
 const s=create();s.edit('mode','notes');assert.equal(s.preview().features.context_management.experimental_mode,true);assert.equal(s.preview().features.token_budget.use_history_notes_extension,true);assert.equal(s.preview().features.async_compaction.enabled,false);s.edit('mode','regular');assert.equal(s.preview().features.context_management.experimental_mode,false);assert.equal(s.preview().features.token_budget.use_history_notes_extension,false);s.edit('mode','inherit');assert.deepEqual(s.preview(),{});
});
test('Provider 协议独立保存，策略切换不覆盖另一个 Provider',()=>{
 const s=create();s.edit('protocol','v2');s.select('krill');s.edit('protocol','local');s.save();s.edit('mode','background');assert.deepEqual(s.preview().model_providers,{opencodex:{capabilities:{remote_compaction:'v2'}},krill:{capabilities:{remote_compaction:'unsupported'}}});s.edit('mode','inherit');s.select('opencodex');s.edit('protocol','inherit');assert.equal(s.preview().model_providers.opencodex,undefined);assert.equal(s.preview().model_providers.krill.capabilities.remote_compaction,'unsupported');
});
test('保存与重启分离，未保存草稿不被重启采用',()=>{
 const s=create();s.edit('mode','background');s.save();assert.equal(s.snapshot().pending,true);assert.equal(s.snapshot().applied.mode,'inherit');s.edit('mode','notes');s.restart();assert.equal(s.snapshot().applied.mode,'background');assert.equal(s.snapshot().draft.mode,'notes');assert.equal(s.snapshot().dirty,true);assert.equal(s.snapshot().pending,false);assert.equal(s.observation('thread:config'),null);
});
test('阈值范围、可见字段校验与保存失败均保留输入',()=>{
 const s=create();s.edit('mode','background');for(const bad of ['0','101','-1','1.5','abc']){s.edit('backgroundPercent',bad);assert.equal(s.save().ok,false);assert.equal(s.snapshot().draft.backgroundPercent,bad);}s.edit('mode','regular');assert.equal(s.save().ok,true);s.edit('autoTokens','9007199254740992');assert.equal(s.save().ok,false);s.edit('autoTokens','100000');s.edit('postTurnPercent','0');s.setSaveFailure(true);assert.equal(s.save().ok,false);assert.equal(s.snapshot().draft.autoTokens,'100000');assert.equal(s.snapshot().saved.autoTokens,'');s.setSaveFailure(false);assert.equal(s.save().ok,true);s.discard();assert.equal(s.snapshot().draft.autoTokens,'100000');
});
test('观察绑定具体对话，不由保存、协议声明或重启自动生成',()=>{
 const s=create();s.edit('protocol','v2');s.edit('mode','notes');s.save();assert.equal(s.observation('thread:config'),null);s.observe('summary');assert.match(s.observation('thread:config').processor,/代理摘要/);assert.equal(s.observation('project:tools'),null);assert.equal(s.observation('thread:review'),null);s.restart();assert.equal(s.observation('thread:config'),null);s.observe('notes');assert.match(s.observation('thread:config').notes,/直连示例/);s.observe('unknown');assert.equal(s.observation('thread:config'),null);
});
